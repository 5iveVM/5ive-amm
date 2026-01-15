// DSL Parser Module
//
// Handles parsing tokens into an Abstract Syntax Tree (AST).

use crate::ast::{
    AstNode, Attribute, BlockKind, ErrorVariant, EventFieldAssignment, InstructionParameter, MatchArm,
    ModuleSpecifier, StructField, StructLiteralField, TestAttribute, TypeNode,
};
use crate::tokenizer::{Token, TokenKind};
use five_protocol::Value;
use five_vm_mito::error::VMError;
mod blocks;
mod expressions;
mod expression_builder;
mod statements;

/// Handles lexical analysis of .five DSL syntax into tokens.
pub struct DslParser {
    tokens: Vec<Token>,
    position: usize,
    current_token: Token,
}

impl DslParser {
    pub fn new(tokens: Vec<Token>) -> Self {
        let current_token = tokens.get(0).cloned().unwrap_or(Token::Eof);

        Self {
            tokens,
            position: 0,
            current_token,
        }
    }

    pub fn parse(&mut self) -> Result<AstNode, VMError> {
        let mut program_name = "Module".to_string();
        if matches!(self.current_token, Token::Script) {
            self.advance(); // consume 'script'
            program_name = match &self.current_token {
                Token::Identifier(name) => {
                    let name = name.clone();
                    self.advance();
                    name
                }
                _ => {
                    // TEMP DEBUG: Dump token stream and parser state when script name is missing.
                    // This is temporary instrumentation to help diagnose failing tests that
                    // hit this parse error. It prints the current token, full token vector,
                    // and the parser position.
                    eprintln!(
                        "PARSER_DEBUG: expected script name identifier but found: {:?}",
                        self.current_token
                    );
                    eprintln!("PARSER_DEBUG: tokens = {:?}", self.tokens);
                    eprintln!("PARSER_DEBUG: position = {}", self.position);
                    return Err(self.parse_error("script name identifier"));
                }
            };
            if !matches!(self.current_token, Token::LeftBrace) {
                return Err(self.parse_error("'{' to start script body"));
            }
            self.advance(); // consume '{'
            let ast = self.parse_module(program_name)?;
            if !matches!(self.current_token, Token::RightBrace) {
                return Err(self.parse_error("'}' to end script"));
            }
            self.advance(); // consume '}'
            if !matches!(self.current_token, Token::Eof) {
                return Err(self.parse_error("end of script"));
            }
            return Ok(ast);
        }

        self.parse_module(program_name)
    }

    fn parse_module(&mut self, program_name: String) -> Result<AstNode, VMError> {
        // Parse functions and fields until '}' or EOF
        let mut instruction_definitions = Vec::new();
        let mut field_definitions = Vec::new();
        let mut event_definitions = Vec::new();
        let mut account_definitions = Vec::new();
        let mut interface_definitions = Vec::new();
        let mut import_statements = Vec::new();
        let mut init_block = None;
        let mut constraints_block = None;

        while self.current_token.kind() != TokenKind::RightBrace
            && self.current_token.kind() != TokenKind::Eof
        {
            match self.current_token.kind() {
                TokenKind::Use | TokenKind::Import => {
                    import_statements.push(self.parse_use_statement()?);
                }
                TokenKind::Init => {
                    self.advance();
                    init_block = Some(Box::new(self.parse_block(BlockKind::Init)?));
                }
                TokenKind::Constraints => {
                    self.advance();
                    constraints_block = Some(Box::new(self.parse_block(BlockKind::Constraints)?));
                }
                TokenKind::Event => {
                    event_definitions.push(self.parse_event_definition()?);
                }
                TokenKind::Enum => {
                    field_definitions.push(self.parse_error_type_definition()?);
                }
                // Account system: Handle account type definitions
                TokenKind::Account => {
                    account_definitions.push(self.parse_account_definition()?);
                }
                // Interface system: Handle interface definitions
                TokenKind::Interface => {
                    interface_definitions.push(self.parse_interface_definition()?);
                }
                // Testing system: Handle test function definitions
                TokenKind::Hash => {
                    instruction_definitions.push(self.parse_test_function()?);
                }
                TokenKind::Instruction => {
                    self.advance(); // Consume 'instruction' keyword
                    instruction_definitions.push(self.parse_instruction_definition()?);
                }
                // Functions and fields
                TokenKind::Pub => {
                    // Public function definition: parse as instruction
                    instruction_definitions.push(self.parse_instruction_definition()?);
                }
                TokenKind::Fn => {
                    // Private function definition (fn) without pub
                    self.advance(); // consume 'fn'
                    instruction_definitions.push(self.parse_instruction_definition()?);
                }
                TokenKind::Test => {
                    // Allow plain function named 'test' without pub/fn prefix
                    instruction_definitions.push(self.parse_instruction_definition()?);
                }
                TokenKind::Identifier => {
                    // Decide between function definition and field definition by lookahead
                    if self.peek_kind(1) == TokenKind::LeftParen {
                        instruction_definitions.push(self.parse_instruction_definition()?);
                    } else {
                        field_definitions.push(self.parse_field_definition()?);
                    }
                }
                TokenKind::Mut => {
                    // Field definition starting with 'mut'
                    field_definitions.push(self.parse_field_definition()?);
                }
                _ => return Err(self.unexpected_token()),
            }
        }

        // Create program node with provided name
        Ok(AstNode::Program {
            program_name,
            field_definitions,
            instruction_definitions,
            event_definitions,
            account_definitions,
            interface_definitions,
            import_statements,
            init_block,
            constraints_block,
        })
    }

    fn peek_kind(&self, n: usize) -> TokenKind {
        self.tokens
            .get(self.position + n)
            .cloned()
            .unwrap_or(Token::Eof)
            .kind()
    }

    #[allow(dead_code)]
    fn parse_field_definition(&mut self) -> Result<AstNode, VMError> {
        // Check for 'pub' keyword to determine visibility
        let visibility = if matches!(self.current_token, Token::Pub) {
            self.advance(); // consume 'pub'
            crate::Visibility::Public
        } else {
            crate::Visibility::Internal
        };

        // Handle optional mutability: mut field_name
        let is_mutable = if matches!(self.current_token, Token::Mut) {
            self.advance(); // consume 'mut'
            true
        } else {
            false
        };

        // Parse field name
        let name = match &self.current_token {
            Token::Identifier(name) => name.clone(),
            Token::Account => "account".to_string(),
            _ => return Err(self.parse_error("field name identifier")),
        };
        self.advance();

        // Check for optional marker: name?
        let is_optional = if matches!(self.current_token, Token::Question) {
            self.advance(); // consume '?'
            true
        } else {
            false
        };

        // Parse type annotation: : Type
        if !matches!(self.current_token, Token::Colon) {
            return Err(self.parse_error("':' after field name for type annotation"));
        }
        self.advance(); // consume ':'

        let field_type = Box::new(self.parse_type()?);

        // Parse optional default value: = expression
        let default_value = if matches!(self.current_token, Token::Assign) {
            self.advance(); // consume '='
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };

        // Optional semicolon
        if matches!(self.current_token, Token::Semicolon) {
            self.advance();
        }

        Ok(AstNode::FieldDefinition {
            name,
            field_type,
            is_mutable,
            is_optional,
            default_value,
            visibility,
        })
    }

    fn parse_instruction_definition(&mut self) -> Result<AstNode, VMError> {
        // Check for 'pub' keyword to determine visibility
        let (is_public, visibility) = if matches!(self.current_token, Token::Pub) {
            self.advance(); // consume 'pub'
            (true, crate::Visibility::Public)
        } else {
            (false, crate::Visibility::Internal)
        };

        // Skip mut if present (instruction definitions don't use mut)
        if matches!(self.current_token, Token::Mut) {
            self.advance();
        }

        // Consume optional 'fn' or 'instruction' keywords that can follow visibility specifiers.
        if matches!(self.current_token, Token::Fn | Token::Instruction) {
            self.advance();
        }

        // Parse optional function/instruction name (allow shorthand 'test' prefix)
        // Some call sites invoke this parser after consuming 'fn'/'instruction'/'pub',
        // others call it directly when the current token is the function name.
        // If a 'test' keyword is present as a prefix, consume it and expect an identifier next.
        if matches!(self.current_token, Token::Test) {
            // consume the 'test' keyword used as a shorthand prefix
            self.advance();
        }
        // Now expect the function/instruction name identifier
        let name = match &self.current_token {
            Token::Identifier(n) => {
                let n = n.clone();
                self.advance();
                n
            }
            _ => return Err(self.parse_error("instruction/function name identifier")),
        };

        // Parse parameter list: (param1: Type, param2?: Type)
        if !matches!(self.current_token, Token::LeftParen) {
            return Err(self.parse_error("'(' to start parameter list"));
        }
        self.advance(); // consume '('

        let mut parameters = Vec::new();

        while !matches!(self.current_token, Token::RightParen)
            && !matches!(self.current_token, Token::Eof)
        {
            // Allow optional leading attributes placed before the parameter name:
            // e.g., @signer @mut @requires(amount > 0) param: Account
            let mut leading_attributes: Vec<Attribute> = Vec::new();
            while matches!(
                self.current_token,
                Token::AtSigner | Token::AtMut | Token::AtInit | Token::At
            ) {
                match &self.current_token {
                    Token::AtSigner => {
                        leading_attributes.push(Attribute { name: "signer".to_string(), args: vec![] });
                        self.advance();
                    }
                    Token::AtMut => {
                        leading_attributes.push(Attribute { name: "mut".to_string(), args: vec![] });
                        self.advance();
                    }
                    Token::AtInit => {
                        // Mark that this parameter has an init directive
                        leading_attributes.push(Attribute { name: "init".to_string(), args: vec![] });
                        self.advance();
                    }
                    Token::At => {
                        // Generic attribute: @name(args...)
                        self.advance(); // consume '@'
                        let name = self.expect_ident()?;
                        let mut args = Vec::new();
                        
                        if matches!(self.current_token, Token::LeftParen) {
                            self.advance(); // consume '('
                            while !matches!(self.current_token, Token::RightParen)
                                && !matches!(self.current_token, Token::Eof)
                            {
                                args.push(self.parse_expression()?);
                                if matches!(self.current_token, Token::Comma) {
                                    self.advance();
                                } else {
                                    break;
                                }
                            }
                            if !matches!(self.current_token, Token::RightParen) {
                                return Err(self.parse_error("')' to close attribute arguments"));
                            }
                            self.advance(); // consume ')'
                        }
                        
                        leading_attributes.push(Attribute { name, args });
                    }
                    _ => break,
                }
            }

            // Parse parameter name
            let param_name = match &self.current_token {
                Token::Identifier(name) => name.clone(),
                // Allow keyword 'account' as an identifier name in parameter position
                Token::Account => "account".to_string(),
                _ => return Err(self.parse_error("parameter name identifier")),
            };
            self.advance();

            // Check for optional marker: param?
            let is_optional = if matches!(self.current_token, Token::Question) {
                self.advance(); // consume '?'
                true
            } else {
                false
            };

            // Parse parameter type: : Type
            if !matches!(self.current_token, Token::Colon) {
                return Err(self.parse_error("':' after parameter name for type annotation"));
            }
            self.advance(); // consume ':'

            let param_type = self.parse_type()?;

            // Parse optional account attributes after type: @signer, @mut, @init
            let mut trailing_attributes: Vec<Attribute> = Vec::new();
            let mut is_init = false;
            let mut init_config = None;

            while matches!(
                self.current_token,
                Token::AtSigner | Token::AtMut | Token::AtInit | Token::At
            ) {
                match &self.current_token {
                    Token::AtSigner => {
                        trailing_attributes.push(Attribute { name: "signer".to_string(), args: vec![] });
                        self.advance();
                    }
                    Token::AtMut => {
                        trailing_attributes.push(Attribute { name: "mut".to_string(), args: vec![] });
                        self.advance();
                    }
                    Token::AtInit => {
                        is_init = true;
                        // DSL Compiler Parsing Debug
                        println!("DSL PARSER: Found @init attribute");
                        
                        trailing_attributes.push(Attribute { name: "init".to_string(), args: vec![] });
                        self.advance(); // consume @init token

                        // Check for optional [seeds] syntax
                        if matches!(self.current_token, Token::LeftBracket) {
                            self.advance(); // consume '['
                            println!("DSL PARSER: Found @init with seeds");

                            let mut seeds = Vec::new();
                            while !matches!(self.current_token, Token::RightBracket | Token::Eof) {
                                seeds.push(self.parse_expression()?);

                                if matches!(self.current_token, Token::Comma) {
                                    self.advance();
                                } else if !matches!(self.current_token, Token::RightBracket) {
                                    return Err(self.parse_error("',' or ']' in seed list"));
                                }
                            }

                            if !matches!(self.current_token, Token::RightBracket) {
                                return Err(self.parse_error("']' to close seed list"));
                            }
                            self.advance(); // consume ']'

                            // Auto-generate bump variable name
                            let bump_var = if !seeds.is_empty() {
                                Some(format!("{}_bump", param_name))
                            } else {
                                None
                            };

                            init_config = Some(crate::ast::InitConfig {
                                seeds: if seeds.is_empty() { None } else { Some(seeds) },
                                bump: bump_var,
                                space: None, // Will be calculated during compilation
                                payer: None,
                            });
                        } else {
                            // Simple @init without seeds
                            init_config = Some(crate::ast::InitConfig {
                                seeds: None,
                                bump: None,
                                space: None,
                                payer: None,
                            });
                        }
                    }
                    Token::At => {
                        // Generic attribute: @name(args...)
                        self.advance(); // consume '@'
                        let name = self.expect_ident()?;
                        let mut args = Vec::new();
                        
                        if matches!(self.current_token, Token::LeftParen) {
                            self.advance(); // consume '('
                            while !matches!(self.current_token, Token::RightParen)
                                && !matches!(self.current_token, Token::Eof)
                            {
                                args.push(self.parse_expression()?);
                                if matches!(self.current_token, Token::Comma) {
                                    self.advance();
                                } else {
                                    break;
                                }
                            }
                            if !matches!(self.current_token, Token::RightParen) {
                                return Err(self.parse_error("')' to close attribute arguments"));
                            }
                            self.advance(); // consume ')'
                        }
                        
                        trailing_attributes.push(Attribute { name, args });
                    }
                    _ => unreachable!(),
                }
            }

            // Parse optional default value: = expression
            let default_value = if matches!(self.current_token, Token::Assign) {
                self.advance(); // consume '='
                Some(Box::new(self.parse_expression()?))
            } else {
                None
            };

            // Combine leading and trailing attributes (leading attributes take precedence in ordering)
            let mut final_attributes = leading_attributes.clone();
            final_attributes.extend(trailing_attributes.into_iter());

            parameters.push(InstructionParameter {
                name: param_name,
                param_type,
                is_optional,
                default_value,
                attributes: final_attributes,
                is_init,
                init_config,
            });

            // Handle comma separator
            if matches!(self.current_token, Token::Comma) {
                self.advance(); // consume ','
            } else {
                break;
            }
        }

        if !matches!(self.current_token, Token::RightParen) {
            return Err(VMError::UnexpectedEndOfInput);
        }
        self.advance(); // consume ')'

        // Parse optional return type: -> ReturnType
        let return_type = if matches!(self.current_token, Token::Arrow) {
            self.advance(); // consume '->'
            Some(Box::new(self.parse_type()?))
        } else {
            None
        };

        // Parse function body: { ... }
        let body = Box::new(self.parse_block(BlockKind::Regular)?);

        Ok(AstNode::InstructionDefinition {
            name,
            parameters,
            return_type,
            body,
            visibility,
            is_public,
        })
    }

    #[allow(dead_code)]
    fn is_instruction_definition(&mut self) -> Result<AstNode, VMError> {
        // Consume '#'
        self.advance();

        // Parse optional attributes: #[attribute]
        let mut attributes = Vec::new();
        while matches!(self.current_token, Token::LeftBracket) {
            self.advance(); // consume '['
            if !matches!(self.current_token, Token::Hash) {
                return Err(self.parse_error("'#' after '[' for attribute"));
            }
            self.advance(); // consume '#'

            let attribute_name = match &self.current_token {
                Token::Identifier(name) => name.clone(),
                _ => return Err(self.parse_error("attribute name identifier")),
            };
            self.advance();

            let mut args = Vec::new();
            if matches!(self.current_token, Token::LeftParen) {
                self.advance(); // consume '('
                while !matches!(self.current_token, Token::RightParen)
                    && !matches!(self.current_token, Token::Eof)
                {
                    args.push(self.parse_expression()?);
                    if matches!(self.current_token, Token::Comma) {
                        self.advance();
                    } else {
                        break;
                    }
                }
                if !matches!(self.current_token, Token::RightParen) {
                    return Err(self.parse_error("')' to end attribute arguments"));
                }
                self.advance(); // consume ')'
            }

            attributes.push(TestAttribute {
                name: attribute_name,
                args,
            });

            if !matches!(self.current_token, Token::RightBracket) {
                return Err(self.parse_error("']' to end attribute"));
            }
            self.advance(); // consume ']'
        }

        // Parse 'test' keyword
        if !matches!(self.current_token, Token::Test) {
            return Err(self.parse_error("'test' keyword for test function"));
        }
        self.advance();

        // Parse test function name
        let name = match &self.current_token {
            Token::Identifier(name) => name.clone(),
            _ => return Err(self.parse_error("test function name identifier")),
        };
        self.advance();

        // Parse parameter list (empty for test functions)
        if !matches!(self.current_token, Token::LeftParen) {
            return Err(self.parse_error("'( ' to start test function parameter list"));
        }
        self.advance(); // consume '('

        if !matches!(self.current_token, Token::RightParen) {
            return Err(self.parse_error("')' to end test function parameter list"));
        }
        self.advance(); // consume ')'

        // Parse function body: { ... }
        let body = Box::new(self.parse_block(BlockKind::Regular)?);

        Ok(AstNode::TestFunction {
            name,
            attributes,
            body,
        })
    }

    fn advance(&mut self) {
        self.position += 1;
        self.current_token = self
            .tokens
            .get(self.position)
            .cloned()
            .unwrap_or(Token::Eof);
    }

    // Helper methods for better error reporting
    fn expect_punct(&mut self, k: TokenKind) -> Result<(), VMError> {
        if self.current_token.kind() == k {
            self.advance();
            Ok(())
        } else {
            Err(self.parse_error(&format!("expected {:?}", k)))
        }
    }

    #[allow(dead_code)]
    fn expect_ident(&mut self) -> Result<String, VMError> {
        match &self.current_token {
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance();
                Ok(name)
            }
            // Permit keyword 'account' where an identifier is expected (e.g., parameter/field names)
            Token::Account => {
                self.advance();
                Ok("account".to_string())
            }
            _ => Err(self.parse_error("expected identifier")),
        }
    }

    fn unexpected_token(&self) -> VMError {
        VMError::UnexpectedToken
    }

    #[allow(dead_code)]
    fn unexpected_end_of_input(&self, _expected: &str) -> VMError {
        VMError::UnexpectedEndOfInput
    }

    /// Helper to create structured parse errors
    fn parse_error(&self, expected: &str) -> VMError {
        use heapless::String as HString;
        let mut expected_str = HString::new();
        let mut found_str = HString::new();
        let _ = expected_str.push_str(expected);
        let _ = found_str.push_str(&self.token_to_string(&self.current_token));
        VMError::ParseError {
            expected: expected_str,
            found: found_str,
            position: self.position,
        }
    }

    /// Convert token to human-readable string
    fn token_to_string(&self, token: &Token) -> String {
        match token {
            Token::LeftBrace => "'{'".to_string(),
            Token::RightBrace => "'}'".to_string(),
            Token::LeftParen => "'('".to_string(),
            Token::RightParen => "')'".to_string(),
            Token::LeftBracket => "'['".to_string(),
            Token::RightBracket => "']'".to_string(),
            Token::Colon => "':'".to_string(),
            Token::Semicolon => "';'".to_string(),
            Token::Comma => "','".to_string(),
            Token::Dot => "'.'".to_string(),
            Token::Arrow => "'->' or '=>'".to_string(),
            Token::Assign => "'='".to_string(),
            Token::Question => "'?'".to_string(),
            Token::Identifier(name) => format!("identifier '{}'", name),
            Token::NumberLiteral(n) => format!("number '{}'", n),
            Token::StringLiteral(s) => format!("string '{}'", s),
            Token::True => "'true'".to_string(),
            Token::False => "'false'".to_string(),
            Token::Let => "'let'".to_string(),
            Token::Mut => "'mut'".to_string(),
            Token::If => "'if'".to_string(),
            Token::Else => "'else'".to_string(),
            Token::Match => "'match'".to_string(),
            Token::Return => "'return'".to_string(),
            Token::Event => "'event'".to_string(),
            Token::Emit => "'emit'".to_string(),
            Token::Account => "'account'".to_string(),
            Token::Enum => "'enum'".to_string(),
            Token::Some => "'Some'".to_string(),
            Token::None => "'None'".to_string(),
            Token::Ok => "'Ok'".to_string(),
            Token::Err => "'Err'".to_string(),
            Token::Eof => "end of input".to_string(),
            _ => "unknown token".to_string(),
        }
    }

    /// Generic delimited list parser with optional trailing separator support
    /// Returns (items, had_trailing_separator)
    fn parse_list<T>(
        &mut self,
        open: TokenKind,
        close: TokenKind,
        sep: TokenKind,
        allow_trailing: bool,
        mut parse_item: impl FnMut(&mut Self) -> Result<T, VMError>,
    ) -> Result<(Vec<T>, bool), VMError> {
        // Expect opening delimiter
        self.expect_punct(open)?;

        // Empty list
        if self.current_token.kind() == close {
            self.advance();
            return Ok((Vec::new(), false));
        }

        let mut items: Vec<T> = Vec::new();
        let mut had_trailing = false;

        // First item
        items.push(parse_item(self)?);

        // Subsequent items
        loop {
            if self.current_token.kind() == sep {
                self.advance();

                // Trailing separator just before close
                if allow_trailing && self.current_token.kind() == close {
                    had_trailing = true;
                    self.advance();
                    break;
                }

                // Parse next item
                items.push(parse_item(self)?);
            } else {
                break;
            }
        }

        // Expect closing delimiter
        if self.current_token.kind() != close {
            return Err(self.parse_error(&format!("expected {:?} to close list", close)));
        }
        self.advance();

        Ok((items, had_trailing))
    }

    // Parse an argument expression, allowing optional trailing @mut/@signer markers in callsites
    fn parse_event_definition(&mut self) -> Result<AstNode, VMError> {
        // Check for 'pub' keyword to determine visibility
        let visibility = if matches!(self.current_token, Token::Pub) {
            self.advance(); // consume 'pub'
            crate::Visibility::Public
        } else {
            crate::Visibility::Internal
        };

        // Consume 'event' keyword
        if !matches!(self.current_token, Token::Event) {
            return Err(self.parse_error("'event' keyword"));
        }
        self.advance();

        // Parse event name
        let name = match &self.current_token {
            Token::Identifier(name) => name.clone(),
            _ => return Err(self.parse_error("event name identifier")),
        };
        self.advance();

        // Parse event fields: { field1: Type, field2: Type }
        if !matches!(self.current_token, Token::LeftBrace) {
            return Err(self.parse_error("'{' to start event fields"));
        }
        self.advance(); // consume '{'

        let mut fields = Vec::new();

        while !matches!(self.current_token, Token::RightBrace)
            && !matches!(self.current_token, Token::Eof)
        {
            // Parse field name
            let field_name = match &self.current_token {
                Token::Identifier(name) => name.clone(),
                Token::Account => "account".to_string(),
                _ => return Err(self.parse_error("event field name identifier")),
            };
            self.advance();

            if !matches!(self.current_token, Token::Colon) {
                return Err(self.parse_error("':' after field name"));
            }
            self.advance(); // consume ':'

            let field_type = self.parse_type()?;

            fields.push(StructField {
                name: field_name,
                field_type,
                is_mutable: false,  // Event fields are immutable
                is_optional: false, // Event fields are required by default
            });

            if matches!(self.current_token, Token::Comma) {
                self.advance(); // consume ','
            } else {
                break;
            }
        }

        if !matches!(self.current_token, Token::RightBrace) {
            return Err(self.parse_error("'}' to end event fields"));
        }
        self.advance(); // consume '}'

        Ok(AstNode::EventDefinition { name, fields, visibility })
    }

    fn parse_type(&mut self) -> Result<TypeNode, VMError> {
        let token = self.current_token.clone();
        match &token {
            // Handle arrays: [T; N] (Rust style)
            Token::LeftBracket => {
                self.advance(); // consume '['
                let element_type = Box::new(self.parse_type()?);

                if matches!(self.current_token, Token::Semicolon) {
                    self.advance(); // consume ';'

                    // Parse array size
                    let size = match &self.current_token {
                        Token::NumberLiteral(n) => *n,
                        _ => return Err(self.parse_error("array size number literal")),
                    };
                    self.advance();

                    if !matches!(self.current_token, Token::RightBracket) {
                        return Err(self.parse_error("']' to end array type declaration"));
                    }
                    self.advance(); // consume ']'

                    Ok(TypeNode::Array {
                        element_type,
                        size: Some(size),
                    })
                } else {
                    return Err(self.parse_error("';' in array type declaration"));
                }
            }

            // Handle tuples: (T1, T2, ...)
            Token::LeftParen => {
                self.advance(); // consume '('
                let mut elements = Vec::new();

                while !matches!(self.current_token, Token::RightParen)
                    && !matches!(self.current_token, Token::Eof)
                {
                    elements.push(self.parse_type()?);

                    if matches!(self.current_token, Token::Comma) {
                        self.advance(); // consume ','
                    } else {
                        break;
                    }
                }

                if !matches!(self.current_token, Token::RightParen) {
                    return Err(self.parse_error("')' to end tuple type"));
                }
                self.advance(); // consume ')'

                Ok(TypeNode::Tuple { elements })
            }

            // Handle struct types: { field1: T1, field2: T2 }
            Token::LeftBrace => {
                self.advance(); // consume '{'
                let mut fields = Vec::new();

                while !matches!(self.current_token, Token::RightBrace)
                    && !matches!(self.current_token, Token::Eof)
                {
                    // Handle optional mutability: mut field
                    let is_mutable = if matches!(self.current_token, Token::Mut) {
                        self.advance(); // consume 'mut'
                        true
                    } else {
                        false
                    };

                    // Parse field name
                    let name = match &self.current_token {
                        Token::Identifier(name) => name.clone(),
                        _ => return Err(self.parse_error("struct field name identifier in type")),
                    };
                    self.advance();

                    // Check for optional marker: field?
                    let is_optional = if matches!(self.current_token, Token::Question) {
                        self.advance(); // consume '?'
                        true
                    } else {
                        false
                    };

                    if !matches!(self.current_token, Token::Colon) {
                        return Err(self.parse_error("':' after struct field name in type"));
                    }
                    self.advance(); // consume ':'

                    let field_type = self.parse_type()?;

                    fields.push(StructField {
                        name,
                        field_type,
                        is_mutable,
                        is_optional,
                    });

                    if matches!(self.current_token, Token::Comma) {
                        self.advance(); // consume ','
                    } else {
                        break;
                    }
                }

                if !matches!(self.current_token, Token::RightBrace) {
                    return Err(self.parse_error("'}' to end struct type"));
                }
                self.advance(); // consume '}'

                Ok(TypeNode::Struct { fields })
            }

            // Handle primitive types and generics
            Token::Type(type_name) => {
                let base_type = type_name.clone();
                self.advance();

                // Check for sized types: string<32>
                if matches!(self.current_token, Token::LT) {
                    self.advance(); // consume '<'

                    let size = match &self.current_token {
                        Token::NumberLiteral(n) => *n,
                        _ => return Err(self.parse_error("size number literal in sized type")),
                    };
                    self.advance();

                    if !matches!(self.current_token, Token::GT) {
                        return Err(self.parse_error("'>' to end sized type"));
                    }
                    self.advance(); // consume '>'

                    Ok(TypeNode::Sized { base_type, size })
                } else {
                    // Check for TypeScript-style arrays: pubkey[], string[], etc.
                    if matches!(self.current_token, Token::LeftBracket) {
                        self.advance(); // consume '['

                        let size = match &self.current_token {
                            Token::NumberLiteral(n) => Some(*n),
                            _ => None, // Dynamic array: pubkey[]
                        };

                        if size.is_some() {
                            self.advance(); // consume size
                        }

                        if !matches!(self.current_token, Token::RightBracket) {
                            return Err(self.parse_error("']' to end TypeScript-style array type"));
                        }
                        self.advance(); // consume ']'

                        Ok(TypeNode::Array {
                            element_type: Box::new(TypeNode::Primitive(type_name.clone())),
                            size,
                        })
                    } else {
                        if type_name == "Account" {
                            Ok(TypeNode::Account)
                        } else {
                            Ok(TypeNode::Primitive(type_name.clone()))
                        }
                    }
                }
            }

            // Handle generic types: Result, Option, etc.
            Token::Result | Token::Option => {
                let base = match &self.current_token {
                    Token::Result => "Result".to_string(),
                    Token::Option => "Option".to_string(),
                    _ => unreachable!(),
                };
                self.advance();

                // Check for generic arguments: <T, E>
                if matches!(self.current_token, Token::LT) {
                    self.advance(); // consume '<'
                    let mut args = Vec::new();

                    while !matches!(self.current_token, Token::GT)
                        && !matches!(self.current_token, Token::Eof)
                    {
                        args.push(self.parse_type()?);

                        if matches!(self.current_token, Token::Comma) {
                            self.advance(); // consume ','
                        } else {
                            break;
                        }
                    }

                    if !matches!(self.current_token, Token::GT) {
                        return Err(self.parse_error("'>' to end generic type"));
                    }
                    self.advance(); // consume '>'

                    Ok(TypeNode::Generic { base, args })
                } else {
                    Ok(TypeNode::Named(base))
                }
            }

            // Handle built-in types that are identifiers (String, etc.)
            Token::Identifier(name) if matches!(name.as_str(), "String" | "str") => {
                let type_name = name.clone();
                self.advance();
                Ok(TypeNode::Primitive(type_name))
            }

            // Handle built-in account type with implicit properties
            Token::Account => {
                self.advance();
                Ok(TypeNode::Account)
            }


            // Handle custom/named types
            Token::Identifier(name) => {
                let type_name = name.clone();
                self.advance();

                // Check for TypeScript-style arrays: Type[N]
                if matches!(self.current_token, Token::LeftBracket) {
                    self.advance(); // consume '['

                    let size = match &self.current_token {
                        Token::NumberLiteral(n) => Some(*n),
                        _ => None, // Dynamic array
                    };

                    if size.is_some() {
                        self.advance(); // consume size
                    }

                    if !matches!(self.current_token, Token::RightBracket) {
                        return Err(self.parse_error("']' to end TypeScript-style array type"));
                    }
                    self.advance(); // consume ']'

                    Ok(TypeNode::Array {
                        element_type: Box::new(TypeNode::Named(type_name)),
                        size,
                    })
                } else {
                    Ok(TypeNode::Named(type_name))
                }
            }

            _ => Err(self.parse_error("type specification")),
        }
    }

    fn parse_error_type_definition(&mut self) -> Result<AstNode, VMError> {
        self.advance(); // consume 'enum'

        // Parse enum name
        let name = match &self.current_token {
            Token::Identifier(name) => name.clone(),
            _ => return Err(self.parse_error("enum name identifier")),
        };
        self.advance();

        // Parse enum body: { variant1, variant2, ... }
        if !matches!(self.current_token, Token::LeftBrace) {
            return Err(self.parse_error("'{' to start enum body"));
        }
        self.advance(); // consume '{'

        let mut variants = Vec::new();

        while !matches!(self.current_token, Token::RightBrace)
            && !matches!(self.current_token, Token::Eof)
        {
            // Parse variant name
            let variant_name = match &self.current_token {
                Token::Identifier(name) => name.clone(),
                _ => return Err(self.parse_error("enum variant name identifier")),
            };
            self.advance();

            // Parse optional variant data
            let mut fields = Vec::new();

            // Tuple variant: Variant(T1, T2)
            if matches!(self.current_token, Token::LeftParen) {
                self.advance(); // consume '('
                let mut index = 0;

                while !matches!(self.current_token, Token::RightParen)
                    && !matches!(self.current_token, Token::Eof)
                {
                    let field_type = self.parse_type()?;
                    fields.push(StructField {
                        name: format!("field{}", index),
                        field_type,
                        is_mutable: false,
                        is_optional: false,
                    });
                    index += 1;

                    if matches!(self.current_token, Token::Comma) {
                        self.advance();
                    } else {
                        break;
                    }
                }

                if !matches!(self.current_token, Token::RightParen) {
                    return Err(self.parse_error("')' to end tuple variant"));
                }
                self.advance(); // consume ')'

            // Struct variant: Variant { field: Type }
            } else if matches!(self.current_token, Token::LeftBrace) {
                self.advance(); // consume '{'

                while !matches!(self.current_token, Token::RightBrace)
                    && !matches!(self.current_token, Token::Eof)
                {
                    let field_name = match &self.current_token {
                        Token::Identifier(name) => name.clone(),
                        _ => return Err(self.parse_error("struct variant field name identifier")),
                    };
                    self.advance();

                    if !matches!(self.current_token, Token::Colon) {
                        return Err(self.parse_error("':' after struct variant field name"));
                    }
                    self.advance(); // consume ':'

                    let field_type = self.parse_type()?;
                    fields.push(StructField {
                        name: field_name,
                        field_type,
                        is_mutable: false,
                        is_optional: false,
                    });

                    if matches!(self.current_token, Token::Comma) {
                        self.advance();
                    } else {
                        break;
                    }
                }

                if !matches!(self.current_token, Token::RightBrace) {
                    return Err(self.parse_error("'}' to end struct variant"));
                }
                self.advance(); // consume '}'
            }

            variants.push(ErrorVariant {
                name: variant_name,
                fields,
            });

            // Handle comma separation
            if matches!(self.current_token, Token::Comma) {
                self.advance();
            } else if !matches!(self.current_token, Token::RightBrace) {
                return Err(self.parse_error("',' between enum variants or '}' to end enum"));
            }
        }

        if !matches!(self.current_token, Token::RightBrace) {
            return Err(self.parse_error("'}' to end enum body"));
        }
        self.advance(); // consume '}'

        Ok(AstNode::ErrorTypeDefinition { name, variants })
    }

    fn parse_account_definition(&mut self) -> Result<AstNode, VMError> {
        // Check for 'pub' keyword to determine visibility
        let visibility = if matches!(self.current_token, Token::Pub) {
            self.advance(); // consume 'pub'
            crate::Visibility::Public
        } else {
            crate::Visibility::Internal
        };

        self.advance(); // consume 'account'

        // Parse account name
        let name = match &self.current_token {
            Token::Identifier(name) => name.clone(),
            _ => return Err(self.parse_error("account name identifier")),
        };
        self.advance();

        // Parse account fields: { field1: Type, field2: Type }
        if !matches!(self.current_token, Token::LeftBrace) {
            return Err(self.parse_error("'{' to start account fields"));
        }
        self.advance(); // consume '{'

        let mut fields = Vec::new();

        while !matches!(self.current_token, Token::RightBrace)
            && !matches!(self.current_token, Token::Eof)
        {
            // Handle optional mutability: mut field_name
            let is_mutable = if matches!(self.current_token, Token::Mut) {
                self.advance(); // consume 'mut'
                true
            } else {
                false
            };

            // Parse field name
            let field_name = match &self.current_token {
                Token::Identifier(name) => name.clone(),
                Token::Account => "account".to_string(),
                _ => return Err(self.parse_error("account field name identifier")),
            };
            self.advance();

            // Check for optional marker: name?
            let is_optional = if matches!(self.current_token, Token::Question) {
                self.advance(); // consume '?'
                true
            } else {
                false
            };

            if !matches!(self.current_token, Token::Colon) {
                return Err(self.parse_error("':' after account field name"));
            }
            self.advance(); // consume ':'

            let field_type = Box::new(self.parse_type()?);

            fields.push(StructField {
                name: field_name,
                field_type: *field_type,
                is_mutable,
                is_optional,
            });

            // Allow either ',' or ';' as field separators (Rust-like flexibility)
            if matches!(self.current_token, Token::Comma)
                || matches!(self.current_token, Token::Semicolon)
            {
                self.advance(); // consume separator
            } else {
                // No explicit separator; allow immediate '}' to end the fields
                // or break to validate closing brace below.
                break;
            }
        }

        if !matches!(self.current_token, Token::RightBrace) {
            return Err(self.parse_error("'}' to end account fields"));
        }
        self.advance(); // consume '}'

        Ok(AstNode::AccountDefinition { name, fields, visibility })
    }

    fn parse_interface_definition(&mut self) -> Result<AstNode, VMError> {
        self.advance(); // consume 'interface'

        // Parse interface name
        let name = match &self.current_token {
            Token::Identifier(name) => name.clone(),
            _ => return Err(self.parse_error("interface name identifier")),
        };
        self.advance();

        // Parse optional program ID: program("address") or @program("address")
        let mut program_id: Option<String> = None;
        let mut serializer: Option<String> = None;
        if matches!(&self.current_token, Token::Identifier(name) if name == "program") {
            self.advance(); // consume 'program'
            if !matches!(self.current_token, Token::LeftParen) {
                return Err(self.parse_error("'(' after program keyword"));
            }
            self.advance(); // consume '('
            let id = match &self.current_token {
                Token::StringLiteral(s) => s.clone(),
                _ => return Err(self.parse_error("string literal for program ID")),
            };
            self.advance();
            if !matches!(self.current_token, Token::RightParen) {
                return Err(self.parse_error("')' after program ID"));
            }
            self.advance(); // consume ')'
            program_id = Some(id);
        } else if matches!(self.current_token, Token::At) {
            // Attribute form: @program("...")
            let saved_pos = self.position;
            self.advance(); // consume '@'
            let is_program_attr =
                matches!(&self.current_token, Token::Identifier(name) if name == "program");
            if is_program_attr {
                self.advance(); // consume 'program' identifier
                if !matches!(self.current_token, Token::LeftParen) {
                    return Err(self.parse_error("'(' after program attribute"));
                }
                self.advance(); // '('
                let id = match &self.current_token {
                    Token::StringLiteral(s) => s.clone(),
                    _ => return Err(self.parse_error("string literal for program ID")),
                };
                self.advance();
                if !matches!(self.current_token, Token::RightParen) {
                    return Err(self.parse_error("')' after program ID"));
                }
                self.advance(); // ')'
                program_id = Some(id);
            } else {
                // Not a @program attribute; rewind to saved position so later parsing can continue cleanly
                self.position = saved_pos;
                self.current_token = self
                    .tokens
                    .get(self.position)
                    .cloned()
                    .unwrap_or(Token::Eof);
            }
        }

        // Optional serializer hint: serializer("borsh") or @serializer("borsh")
        if serializer.is_none() {
            if matches!(&self.current_token, Token::Identifier(name) if name == "serializer") {
                self.advance(); // consume 'serializer'
                if !matches!(self.current_token, Token::LeftParen) {
                    return Err(self.parse_error("'(' after serializer keyword"));
                }
                self.advance(); // '('
                let ser = match &self.current_token {
                    Token::StringLiteral(s) => s.clone(),
                    _ => return Err(self.parse_error("string literal for serializer name")),
                };
                self.advance();
                if !matches!(self.current_token, Token::RightParen) {
                    return Err(self.parse_error("')' after serializer name"));
                }
                self.advance(); // ')'
                serializer = Some(ser);
            } else if matches!(self.current_token, Token::At) {
                let saved_pos = self.position;
                self.advance(); // consume '@'
                let is_serializer_attr =
                    matches!(&self.current_token, Token::Identifier(name) if name == "serializer");
                if is_serializer_attr {
                    self.advance(); // consume 'serializer'
                    if !matches!(self.current_token, Token::LeftParen) {
                        return Err(self.parse_error("'(' after serializer attribute"));
                    }
                    self.advance(); // '('
                    let ser = match &self.current_token {
                        Token::StringLiteral(s) => s.clone(),
                        _ => return Err(self.parse_error("string literal for serializer name")),
                    };
                    self.advance();
                    if !matches!(self.current_token, Token::RightParen) {
                        return Err(self.parse_error("')' after serializer name"));
                    }
                    self.advance(); // ')'
                    serializer = Some(ser);
                } else {
                    // Rewind if not serializer attribute
                    self.position = saved_pos;
                    self.current_token = self
                        .tokens
                        .get(self.position)
                        .cloned()
                        .unwrap_or(Token::Eof);
                }
            }
        }

        // Parse interface methods: { method1(), method2() }
        if !matches!(self.current_token, Token::LeftBrace) {
            return Err(self.parse_error("'{' to start interface methods"));
        }
        self.advance(); // consume '{'

        let mut functions = Vec::new();

        while !matches!(self.current_token, Token::RightBrace)
            && !matches!(self.current_token, Token::Eof)
        {
            // Parse method name (optional `fn` keyword allowed for readability)
            if matches!(self.current_token, Token::Fn) {
                self.advance(); // consume 'fn'
            }

            let method_name = match &self.current_token {
                Token::Identifier(name) => name.clone(),
                _ => return Err(self.parse_error("method name identifier")),
            };
            self.advance();

            // Optional attribute form for discriminator before parameter list: @discriminator(N)
            let mut discriminator: Option<u8> = None;
            let mut discriminator_bytes: Option<Vec<u8>> = None;
            if matches!(self.current_token, Token::At) {
                self.advance(); // consume '@'
                                // Accept either identifier("discriminator") or Token::Discriminator
                let is_disc = matches!(&self.current_token, Token::Identifier(name) if name == "discriminator")
                    || matches!(self.current_token, Token::Discriminator);
                let is_disc_bytes =
                    matches!(&self.current_token, Token::Identifier(name) if name == "discriminator_bytes")
                        || matches!(self.current_token, Token::DiscriminatorBytes);
                if is_disc {
                    self.advance(); // consume 'discriminator'
                    if !matches!(self.current_token, Token::LeftParen) {
                        return Err(self.parse_error("'(' after discriminator keyword"));
                    }
                    self.advance(); // '('
                    discriminator = match &self.current_token {
                        Token::NumberLiteral(n) => Some(*n as u8),
                        _ => return Err(self.parse_error("number literal for discriminator")),
                    };
                    self.advance();
                    if !matches!(self.current_token, Token::RightParen) {
                        return Err(self.parse_error("')' after discriminator value"));
                    }
                    self.advance(); // ')'
                } else if is_disc_bytes {
                    self.advance(); // consume 'discriminator_bytes'
                    if !matches!(self.current_token, Token::LeftParen) {
                        return Err(self.parse_error("'(' after discriminator_bytes keyword"));
                    }
                    self.advance(); // '('
                    let mut bytes = Vec::new();
                    while !matches!(self.current_token, Token::RightParen)
                        && !matches!(self.current_token, Token::Eof)
                    {
                        let b = match &self.current_token {
                            Token::NumberLiteral(n) if *n <= u8::MAX as u64 => *n as u8,
                            _ => {
                                return Err(
                                    self.parse_error("number literal (0-255) for discriminator_bytes"),
                                )
                            }
                        };
                        bytes.push(b);
                        self.advance();
                        if matches!(self.current_token, Token::Comma) {
                            self.advance(); // consume ',' and continue
                        } else {
                            break;
                        }
                    }
                    if !matches!(self.current_token, Token::RightParen) {
                        return Err(self.parse_error("')' after discriminator_bytes values"));
                    }
                    self.advance(); // ')'
                    discriminator_bytes = Some(bytes);
                } else {
                    // Unknown attribute before params; ignore gracefully by skipping identifier and any (...) group
                    // Best-effort skip
                }
            }

            // Parse parameter list: (param1: Type, param2?: Type)
            if !matches!(self.current_token, Token::LeftParen) {
                return Err(self.parse_error("'(' to start method parameter list"));
            }
            self.advance(); // consume '('

            let mut parameters = Vec::new();

            while !matches!(self.current_token, Token::RightParen)
                && !matches!(self.current_token, Token::Eof)
            {
                // Parse parameter name
                let param_name = match &self.current_token {
                    Token::Identifier(name) => name.clone(),
                    _ => return Err(self.parse_error("parameter name identifier")),
                };
                self.advance();

                // Check for optional marker: param?
                let is_optional = if matches!(self.current_token, Token::Question) {
                    self.advance(); // consume '?'
                    true
                } else {
                    false
                };

                // Parse parameter type: : Type
                if !matches!(self.current_token, Token::Colon) {
                    return Err(self.parse_error("':' after parameter name for type annotation"));
                }
                self.advance(); // consume ':'

                let param_type = self.parse_type()?;

                // Interface methods don't have attributes or default values
                parameters.push(InstructionParameter {
                    name: param_name,
                    param_type,
                    is_optional,
                    default_value: None,
                    attributes: Vec::new(),
                    is_init: false,
                    init_config: None,
                });

                // Handle comma separator
                if matches!(self.current_token, Token::Comma) {
                    self.advance(); // consume ','
                } else {
                    break;
                }
            }

            if !matches!(self.current_token, Token::RightParen) {
                return Err(self.parse_error("')' to end method parameter list"));
            }
            self.advance(); // consume ')'

            // Parse optional return type: -> ReturnType
            let return_type = if matches!(self.current_token, Token::Arrow) {
                self.advance(); // consume '->'
                Some(Box::new(self.parse_type()?))
            } else {
                None
            };

            // Parse optional discriminator after params: discriminator(N) or discriminator_bytes(...)
            let (discriminator, discriminator_bytes) = if discriminator.is_some() || discriminator_bytes.is_some() {
                (discriminator, discriminator_bytes)
            } else if matches!(self.current_token, Token::Discriminator) {
                self.advance(); // consume 'discriminator'
                if !matches!(self.current_token, Token::LeftParen) {
                    return Err(self.parse_error("'(' after discriminator keyword"));
                }
                self.advance(); // consume '('

                let disc = match &self.current_token {
                    Token::NumberLiteral(n) => Some(*n as u8),
                    _ => return Err(self.parse_error("number literal for discriminator")),
                };
                self.advance();

                if !matches!(self.current_token, Token::RightParen) {
                    return Err(self.parse_error("')' after discriminator value"));
                }
                self.advance(); // consume ')'
                (disc, None)
            } else if matches!(self.current_token, Token::DiscriminatorBytes) {
                self.advance(); // consume 'discriminator_bytes'
                if !matches!(self.current_token, Token::LeftParen) {
                    return Err(self.parse_error("'(' after discriminator_bytes keyword"));
                }
                self.advance(); // consume '('
                let mut bytes = Vec::new();
                while !matches!(self.current_token, Token::RightParen)
                    && !matches!(self.current_token, Token::Eof)
                {
                    let b = match &self.current_token {
                        Token::NumberLiteral(n) if *n <= u8::MAX as u64 => *n as u8,
                        _ => return Err(self.parse_error("number literal (0-255) for discriminator_bytes")),
                    };
                    bytes.push(b);
                    self.advance();
                    if matches!(self.current_token, Token::Comma) {
                        self.advance();
                    } else {
                        break;
                    }
                }
                if !matches!(self.current_token, Token::RightParen) {
                    return Err(self.parse_error("')' after discriminator_bytes values"));
                }
                self.advance(); // consume ')'
                (None, Some(bytes))
            } else {
                (None, None)
            };

            // Optional semicolon
            if matches!(self.current_token, Token::Semicolon) {
                self.advance();
            }

            functions.push(AstNode::InterfaceFunction {
                name: method_name,
                parameters,
                return_type,
                discriminator,
                discriminator_bytes,
            });
        }

        if !matches!(self.current_token, Token::RightBrace) {
            return Err(self.parse_error("'}' to end interface methods"));
        }
        self.advance(); // consume '}'

        Ok(AstNode::InterfaceDefinition {
            name,
            program_id,
            serializer,
            functions,
        })
    }

    fn parse_test_function(&mut self) -> Result<AstNode, VMError> {
        self.advance(); // consume '#'

        // Parse optional attributes: #[attribute]
        let mut attributes = Vec::new();
        while matches!(self.current_token, Token::LeftBracket) {
            self.advance(); // consume '['
            if !matches!(self.current_token, Token::Hash) {
                return Err(self.parse_error("'#' after '[' for attribute"));
            }
            self.advance(); // consume '#'

            let attribute_name = match &self.current_token {
                Token::Identifier(name) => name.clone(),
                _ => return Err(self.parse_error("attribute name identifier")),
            };
            self.advance();

            let mut args = Vec::new();
            if matches!(self.current_token, Token::LeftParen) {
                self.advance(); // consume '('
                while !matches!(self.current_token, Token::RightParen)
                    && !matches!(self.current_token, Token::Eof)
                {
                    args.push(self.parse_expression()?);
                    if matches!(self.current_token, Token::Comma) {
                        self.advance();
                    } else {
                        break;
                    }
                }
                if !matches!(self.current_token, Token::RightParen) {
                    return Err(self.parse_error("')' to end attribute arguments"));
                }
                self.advance(); // consume ')'
            }

            attributes.push(TestAttribute {
                name: attribute_name,
                args,
            });

            if !matches!(self.current_token, Token::RightBracket) {
                return Err(self.parse_error("']' to end attribute"));
            }
            self.advance(); // consume ']'
        }

        // Parse 'test' keyword
        if !matches!(self.current_token, Token::Test) {
            return Err(self.parse_error("'test' keyword for test function"));
        }
        self.advance();

        // Parse test function name
        let name = match &self.current_token {
            Token::Identifier(name) => name.clone(),
            _ => return Err(self.parse_error("test function name identifier")),
        };
        self.advance();

        // Parse parameter list (empty for test functions)
        if !matches!(self.current_token, Token::LeftParen) {
            return Err(self.parse_error("'( ' to start test function parameter list"));
        }
        self.advance(); // consume '('

        if !matches!(self.current_token, Token::RightParen) {
            return Err(self.parse_error("')' to end test function parameter list"));
        }
        self.advance(); // consume ')'

        // Parse function body: { ... }
        let body = Box::new(self.parse_block(BlockKind::Regular)?);

        Ok(AstNode::TestFunction {
            name,
            attributes,
            body,
        })
    }

    /// Parse use statement: use account_address; import account_address; or use account_address::function_name;
    fn parse_use_statement(&mut self) -> Result<AstNode, VMError> {
        // Consume 'use' or 'import' keyword
        if !matches!(self.current_token, Token::Use | Token::Import) {
            return Err(self.parse_error("'use' or 'import' keyword"));
        }
        self.advance();

        // Parse module specifier
        let module_specifier = match &self.current_token {
            Token::StringLiteral(addr) => {
                // External: use "0x123" or use "seeds";
                let addr = addr.clone();
                self.advance();
                ModuleSpecifier::External(addr)
            }
            Token::Identifier(name) => {
                // Local or Nested
                let mut path = vec![name.clone()];
                self.advance();

                while matches!(self.current_token, Token::DoubleColon) {
                    // Peek to see if next token is Identifier (part of path) or '{' (import list)
                    if self.peek_kind(1) == TokenKind::Identifier {
                        self.advance(); // consume ::
                        if let Token::Identifier(segment) = &self.current_token {
                            path.push(segment.clone());
                            self.advance();
                        }
                    } else {
                        break; // :: followed by something else (likely import list start)
                    }
                }

                if path.len() == 1 {
                    ModuleSpecifier::Local(path[0].clone())
                } else {
                    ModuleSpecifier::Nested(path)
                }
            }
            _ => return Err(self.parse_error("module identifier or address string")),
        };

        // Parse optional function imports: ::function_name or ::{func1, func2}
        let imported_items = if matches!(self.current_token, Token::DoubleColon) {
            self.advance(); // consume '::'

            if matches!(self.current_token, Token::LeftBrace) {
                // Multiple imports: use account::{func1, func2}
                self.advance(); // consume '{'
                let mut items = Vec::new();

                while !matches!(self.current_token, Token::RightBrace)
                    && !matches!(self.current_token, Token::Eof)
                {
                    if let Token::Identifier(name) = &self.current_token {
                        items.push(name.clone());
                        self.advance();

                        if matches!(self.current_token, Token::Comma) {
                            self.advance();
                        } else {
                            break;
                        }
                    } else {
                        return Err(self.parse_error("function name identifier in import list"));
                    }
                }

                if !matches!(self.current_token, Token::RightBrace) {
                    return Err(self.parse_error("'}' to close import list"));
                }
                self.advance(); // consume '}'

                Some(items)
            } else if let Token::Identifier(func_name) = &self.current_token {
                // Single import: use account::function_name
                let name = func_name.clone();
                self.advance();
                Some(vec![name])
            } else {
                return Err(self.parse_error("function name or '{' after '::'"));
            }
        } else {
            None // Import all functions
        };

        // Optional semicolon
        if matches!(self.current_token, Token::Semicolon) {
            self.advance();
        }

        Ok(AstNode::ImportStatement {
            module_specifier,
            imported_items,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_struct_type_with_mut_and_optional_fields() {
        let tokens = vec![
            Token::LeftBrace,
            Token::Mut,
            Token::Identifier("a".to_string()),
            Token::Colon,
            Token::Type("u64".to_string()),
            Token::Comma,
            Token::Identifier("b".to_string()),
            Token::Question,
            Token::Colon,
            Token::Type("string".to_string()),
            Token::Comma,
            Token::Mut,
            Token::Identifier("c".to_string()),
            Token::Question,
            Token::Colon,
            Token::Type("bool".to_string()),
            Token::RightBrace,
            Token::Eof,
        ];

        let mut parser = DslParser::new(tokens);
        match parser.parse_type().expect("parse struct type") {
            TypeNode::Struct { fields } => {
                assert_eq!(fields.len(), 3);

                assert_eq!(fields[0].name, "a");
                assert!(fields[0].is_mutable);
                assert!(!fields[0].is_optional);

                assert_eq!(fields[1].name, "b");
                assert!(!fields[1].is_mutable);
                assert!(fields[1].is_optional);

                assert_eq!(fields[2].name, "c");
                assert!(fields[2].is_mutable);
                assert!(fields[2].is_optional);
            }
            _ => panic!("expected struct type"),
        }
    }
}

    #[test]
    fn test_parse_instruction_def_attributes_order() {
        let source = "
            instruction test_func(
                account1: Account @mut @init,
                account2: Account @init @mut
            ) {}
        ";
        let mut tokenizer = crate::tokenizer::DslTokenizer::new(source);
        let tokens = tokenizer.tokenize().unwrap();
        let mut parser = DslParser::new(tokens);
        
        // Skip to instruction definition
        while parser.current_token.kind() != crate::tokenizer::TokenKind::Instruction {
            parser.advance();
        }
        
        let node = parser.parse_instruction_definition().unwrap();
        
        if let AstNode::InstructionDefinition { parameters, .. } = node {
            assert_eq!(parameters.len(), 2);
            
            // Check account1 (@mut @init)
            let p1 = &parameters[0];
            assert!(p1.is_init, "account1 should be init");
            assert!(p1.init_config.is_some(), "account1 init_config should be Some");
            
            // Check account2 (@init @mut)
            let p2 = &parameters[1];
            assert!(p2.is_init, "account2 should be init");
            assert!(p2.init_config.is_some(), "account2 init_config should be Some");
        } else {
            panic!("Expected InstructionDefinition");
        }
    }
