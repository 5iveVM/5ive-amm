// DSL Parser Module
//
// Handles parsing tokens into an Abstract Syntax Tree (AST).

use crate::ast::{
    AstNode, BlockKind,
};
use crate::tokenizer::{Token, TokenKind};
use five_vm_mito::error::VMError;

mod blocks;
mod expressions;
mod expression_builder;
mod imports;
mod instructions;
mod interfaces;
mod statements;
mod structures;
mod types;

#[cfg(test)]
mod expressions_tests;
#[cfg(test)]
mod statements_tests;

/// Handles lexical analysis of .five DSL syntax into tokens.
pub struct DslParser {
    pub(crate) tokens: Vec<Token>,
    pub(crate) position: usize,
    pub(crate) current_token: Token,
}

impl DslParser {
    pub fn new(tokens: Vec<Token>) -> Self {
        let current_token = tokens.first().cloned().unwrap_or(Token::Eof);

        Self {
            tokens,
            position: 0,
            current_token,
        }
    }

    pub fn parse(&mut self) -> Result<AstNode, VMError> {
        eprintln!("DEBUG_PARSER: DslParser::parse started");
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
            eprintln!("DEBUG_PARSER: parse_module processing token={:?}", self.current_token);
            match self.current_token.kind() {
                TokenKind::Use | TokenKind::Import => {
                    import_statements.push(imports::parse_use_statement(self)?);
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
                    event_definitions.push(structures::parse_event_definition(self)?);
                }
                TokenKind::Enum => {
                    field_definitions.push(structures::parse_error_type_definition(self)?);
                }
                // Account system: Handle account type definitions
                TokenKind::Account => {
                    account_definitions.push(structures::parse_account_definition(self)?);
                }
                // Interface system: Handle interface definitions
                TokenKind::Interface => {
                    interface_definitions.push(interfaces::parse_interface_definition(self)?);
                }
                // Testing system: Handle test function definitions
                TokenKind::Hash => {
                    instruction_definitions.push(instructions::parse_test_function(self)?);
                }
                TokenKind::Instruction => {
                    self.advance(); // Consume 'instruction' keyword
                    instruction_definitions.push(instructions::parse_instruction_definition(self)?);
                }
                // Functions and fields
                TokenKind::Pub => {
                    // Public function definition: parse as instruction
                    eprintln!("DEBUG_PARSER: parse_module found pub, about to parse instruction");
                    instruction_definitions.push(instructions::parse_instruction_definition(self)?);
                }
                TokenKind::Fn => {
                    // Private function definition (fn) without pub
                    self.advance(); // consume 'fn'
                    instruction_definitions.push(instructions::parse_instruction_definition(self)?);
                }
                TokenKind::Test => {
                    // Allow plain function named 'test' without pub/fn prefix
                    instruction_definitions.push(instructions::parse_instruction_definition(self)?);
                }
                TokenKind::Identifier => {
                    // Decide between function definition and field definition by lookahead
                    if self.peek_kind(1) == TokenKind::LeftParen {
                        instruction_definitions.push(instructions::parse_instruction_definition(self)?);
                    } else {
                        field_definitions.push(structures::parse_field_definition(self)?);
                    }
                }
                TokenKind::Mut | TokenKind::Field => {
                    // Field definition starting with 'mut' or 'field'
                    field_definitions.push(structures::parse_field_definition(self)?);
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

    pub(crate) fn peek_kind(&self, n: usize) -> TokenKind {
        self.tokens
            .get(self.position + n)
            .cloned()
            .unwrap_or(Token::Eof)
            .kind()
    }

    pub(crate) fn advance(&mut self) {
        self.position += 1;
        self.current_token = self
            .tokens
            .get(self.position)
            .cloned()
            .unwrap_or(Token::Eof);
    }

    // Helper methods for better error reporting
    pub(crate) fn expect_punct(&mut self, k: TokenKind) -> Result<(), VMError> {
        if self.current_token.kind() == k {
            self.advance();
            Ok(())
        } else {
            let msg = format!("expected {:?}", k);
            if msg.len() <= 9 { // "expected " is 9 chars
                 Err(self.parse_error(&format!("DEBUG_EMPTY_TOKENKIND: {:?}", k)))
            } else {
                 Err(self.parse_error(&msg))
            }
        }
    }

    #[allow(dead_code)]
    pub(crate) fn expect_ident(&mut self) -> Result<String, VMError> {
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

    pub(crate) fn unexpected_token(&self) -> VMError {
        VMError::UnexpectedToken
    }

    #[allow(dead_code)]
    pub(crate) fn unexpected_end_of_input(&self, _expected: &str) -> VMError {
        VMError::UnexpectedEndOfInput
    }

    /// Helper to create structured parse errors
    pub(crate) fn parse_error(&self, expected: &str) -> VMError {
        use heapless::String as HString;
        let mut expected_str = HString::new();
        let _ = expected_str.push_str(expected);
        
        let mut found_str = HString::new();
        let _ = found_str.push_str(&self.token_to_string(&self.current_token));
        VMError::ParseError {
            expected: expected_str,
            found: found_str,
            position: self.position,
        }
    }

    /// Convert token to human-readable string
    pub(crate) fn token_to_string(&self, token: &Token) -> String {
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
            Token::Fn => "'fn'".to_string(),
            Token::Instruction => "'instruction'".to_string(),
            Token::Pub => "'pub'".to_string(),
            Token::Async => "'async'".to_string(),
            Token::Interface => "'interface'".to_string(),
            Token::Init => "'init'".to_string(),
            Token::Constraints => "'constraints'".to_string(),
            Token::Script => "'script'".to_string(),
            Token::Field => "'field'".to_string(),
            Token::When => "'when'".to_string(),
            Token::Query => "'query'".to_string(),
            Token::Of => "'of'".to_string(),
            Token::OrInit => "'or_init'".to_string(),
            Token::In => "'in'".to_string(),
            Token::Realloc => "'realloc'".to_string(),
            Token::Pda => "'pda'".to_string(),
            Token::While => "'while'".to_string(),
            Token::For => "'for'".to_string(),
            Token::Do => "'do'".to_string(),
            Token::Break => "'break'".to_string(),
            Token::Continue => "'continue'".to_string(),
            Token::Require => "'require'".to_string(),
            Token::Error => "'error'".to_string(),
            Token::As => "'as'".to_string(),
            Token::AssertEq => "'assert_eq'".to_string(),
            Token::AssertTrue => "'assert_true'".to_string(),
            Token::AssertFalse => "'assert_false'".to_string(),
            Token::AssertFails => "'assert_fails'".to_string(),
            Token::AssertApproxEq => "'assert_approx_eq'".to_string(),
            _ => format!("unknown token {:?}", token),
        }
    }

    /// Generic delimited list parser with optional trailing separator support
    /// Returns (items, had_trailing_separator)
    pub(crate) fn parse_list<T>(
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::TypeNode;

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
        match types::parse_type(&mut parser).expect("parse struct type") {
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
