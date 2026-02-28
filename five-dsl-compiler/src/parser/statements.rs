use crate::ast::{AstNode, BlockKind, EventFieldAssignment, MatchArm};
use crate::parser::{types, DslParser};
use crate::tokenizer::Token;
use five_vm_mito::error::VMError;

impl DslParser {
    pub(crate) fn parse_statement(&mut self) -> Result<AstNode, VMError> {
        eprintln!(
            "DEBUG_PARSER: parse_statement current_token={:?} kind={:?}",
            self.current_token,
            self.current_token.kind()
        );
        match &self.current_token {
            // Handle tuple destructuring assignment: (target1, target2) = value;
            Token::LeftParen => {
                self.advance(); // consume '('
                let mut targets = Vec::new();
                while !matches!(self.current_token, Token::RightParen)
                    && !matches!(self.current_token, Token::Eof)
                {
                    // Targets can be identifiers or field accesses
                    let target_expr = self.parse_field_access()?; // parse_field_access handles identifiers and field accesses
                    match target_expr {
                        AstNode::Identifier(_) | AstNode::FieldAccess { .. } => {
                            targets.push(target_expr);
                        }
                        _ => {
                            return Err(self.parse_error(
                                "identifier or field access in destructuring assignment",
                            ))
                        }
                    }

                    if matches!(self.current_token, Token::Comma) {
                        self.advance();
                    } else if matches!(self.current_token, Token::Semicolon) {
                        self.advance();
                    } else {
                        break;
                    }
                }
                if !matches!(self.current_token, Token::RightParen) {
                    return Err(self.parse_error("')' to end destructuring assignment"));
                }
                self.advance(); // consume ')'

                if !matches!(self.current_token, Token::Assign) {
                    return Err(self.parse_error("'=' for destructuring assignment"));
                }
                self.advance(); // consume '='
                eprintln!("DEBUG_PARSER: destructuring assignment value next");

                let value = Box::new(self.parse_expression()?);

                if matches!(self.current_token, Token::Semicolon) {
                    self.advance();
                }

                Ok(AstNode::TupleAssignment { targets, value })
            }
            Token::If => self.parse_if_statement(),
            Token::Match => self.parse_match_expression(),
            Token::Require => self.parse_require_statement(),
            Token::Return => self.parse_return_statement(),
            // Loop statements
            Token::While => self.parse_while_loop(),
            Token::For => self.parse_for_loop(),
            Token::Do => self.parse_do_while_loop(),
            Token::Let => {
                self.advance(); // consume 'let'

                if matches!(self.current_token, Token::LeftParen) {
                    // Peek ahead to ensure it's a tuple destructuring (identifier follows paren)
                    eprintln!(
                        "DEBUG_PARSER: let destructuring candidate: current={:?} next={:?}",
                        self.current_token,
                        self.peek_kind(1)
                    );
                    self.advance(); // consume '('
                    let mut targets = Vec::new();
                    while !matches!(self.current_token, Token::RightParen)
                        && !matches!(self.current_token, Token::Eof)
                    {
                        if let Token::Identifier(name) = &self.current_token {
                            targets.push(name.clone());
                            self.advance();
                        } else {
                            return Err(self.parse_error("identifier in destructuring assignment"));
                        }
                        if matches!(self.current_token, Token::Comma) {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    if !matches!(self.current_token, Token::RightParen) {
                        eprintln!(
                            "DEBUG_PARSER: E0001 close list expected found {:?}",
                            self.current_token
                        );
                        return Err(self.parse_error("')' to end destructuring assignment"));
                    }
                    self.advance(); // consume ')'

                    if !matches!(self.current_token, Token::Assign) {
                        eprintln!(
                            "DEBUG_PARSER: E0001 assign expected found {:?}",
                            self.current_token
                        );
                        return Err(self.parse_error("'=' for destructuring assignment"));
                    }
                    self.advance(); // consume '='

                    let value = Box::new(self.parse_expression()?);

                    if matches!(self.current_token, Token::Semicolon) {
                        self.advance();
                    }

                    Ok(AstNode::TupleDestructuring { targets, value })
                } else {
                    // Check for 'mut' keyword
                    let is_mutable = if matches!(self.current_token, Token::Mut) {
                        self.advance(); // consume 'mut'
                        true
                    } else {
                        false
                    };

                    // Parse variable name
                    let name = match &self.current_token {
                        Token::Identifier(name) => name.clone(),
                        _ => return Err(self.parse_error("variable name identifier")),
                    };
                    self.advance();
                    eprintln!(
                        "DEBUG_PARSER: let name={} current_token={:?}",
                        name, self.current_token
                    );

                    // Parse optional type annotation: : Type
                    let type_annotation = if matches!(self.current_token, Token::Colon) {
                        self.advance(); // consume ':'
                        Some(Box::new(types::parse_type(self)?))
                    } else {
                        None
                    };

                    // Parse assignment: = value
                    if !matches!(self.current_token, Token::Assign) {
                        return Err(self.parse_error("'=' for variable assignment"));
                    }
                    self.advance(); // consume '='

                    let value = Box::new(self.parse_expression()?);

                    // Optional semicolon
                    if matches!(self.current_token, Token::Semicolon) {
                        self.advance();
                    }

                    Ok(AstNode::LetStatement {
                        name,
                        type_annotation,
                        is_mutable,
                        value,
                    })
                }
            }
            // Testing system: Handle assertion statements
            Token::AssertEq
            | Token::AssertTrue
            | Token::AssertFalse
            | Token::AssertFails
            | Token::AssertApproxEq => self.parse_statement(),
            Token::Emit => {
                self.advance(); // consume 'emit'

                // Parse event name
                let event_name = match &self.current_token {
                    Token::Identifier(name) => name.clone(),
                    _ => return Err(self.parse_error("event name identifier")),
                };
                self.advance();

                // Parse field assignments: { field1: value1, field2: value2 }
                if !matches!(self.current_token, Token::LeftBrace) {
                    return Err(self.parse_error("'{' to start event field assignments"));
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
                        _ => return Err(self.parse_error("field name identifier")),
                    };
                    self.advance();

                    if !matches!(self.current_token, Token::Colon) {
                        return Err(self.parse_error("':' after field name"));
                    }
                    self.advance(); // consume ':'

                    let value = Box::new(self.parse_expression()?);

                    fields.push(EventFieldAssignment { field_name, value });

                    if matches!(self.current_token, Token::Comma) {
                        self.advance(); // consume ','
                    } else {
                        break;
                    }
                }

                if !matches!(self.current_token, Token::RightBrace) {
                    return Err(self.parse_error("'}' to end event field assignments"));
                }
                self.advance(); // consume '}'

                // Optional semicolon
                if matches!(self.current_token, Token::Semicolon) {
                    self.advance();
                }

                Ok(AstNode::EmitStatement { event_name, fields })
            }
            Token::Identifier(_) | Token::Account => {
                // Parse the full expression first (which might be a field access)
                let expr = self.parse_expression()?;

                if matches!(
                    self.current_token,
                    Token::Assign
                        | Token::PlusAssign
                        | Token::MinusAssign
                        | Token::MultiplyAssign
                        | Token::DivideAssign
                        | Token::LeftShiftAssign
                        | Token::RightShiftAssign
                        | Token::BitwiseAndAssign
                        | Token::BitwiseOrAssign
                        | Token::BitwiseXorAssign
                ) {
                    // Assignment or compound assignment
                    let assign_tok = self.current_token.clone();
                    self.advance(); // consume assignment token

                    let rhs = Box::new(self.parse_expression()?);

                    if matches!(self.current_token, Token::Semicolon) {
                        self.advance();
                    }

                    // Determine if we need to synthesize a binary expression for compound ops
                    let op_str: Option<&str> = match assign_tok {
                        Token::Assign => None,
                        Token::PlusAssign => Some("+"),
                        Token::MinusAssign => Some("-"),
                        Token::MultiplyAssign => Some("*"),
                        Token::DivideAssign => Some("/"),
                        Token::LeftShiftAssign => Some("<<"),
                        Token::RightShiftAssign => Some(">>"),
                        Token::BitwiseAndAssign => Some("&"),
                        Token::BitwiseOrAssign => Some("|"),
                        Token::BitwiseXorAssign => Some("^"),
                        _ => None,
                    };

                    match expr {
                        AstNode::Identifier(name) => {
                            let value = if let Some(op) = op_str {
                                Box::new(AstNode::BinaryExpression {
                                    left: Box::new(AstNode::Identifier(name.clone())),
                                    operator: op.to_string(),
                                    right: rhs,
                                })
                            } else {
                                rhs
                            };
                            Ok(AstNode::Assignment {
                                target: name,
                                value,
                            })
                        }
                        AstNode::FieldAccess { object, field } => {
                            let value = if let Some(op) = op_str {
                                Box::new(AstNode::BinaryExpression {
                                    left: Box::new(AstNode::FieldAccess {
                                        object: object.clone(),
                                        field: field.clone(),
                                    }),
                                    operator: op.to_string(),
                                    right: rhs,
                                })
                            } else {
                                rhs
                            };
                            Ok(AstNode::FieldAssignment {
                                object,
                                field,
                                value,
                            })
                        }
                        _ => Err(self.parse_error("identifier or field access for assignment")),
                    }
                } else {
                    // Check if this was a function or method call expression; otherwise treat as expression statement.
                    match expr {
                        AstNode::FunctionCall { name, args } => {
                            if matches!(self.current_token, Token::Semicolon) {
                                self.advance();
                            }

                            // Handle require as a special statement type (matching TypeScript parser)
                            if name == "require" {
                                if args.len() != 1 {
                                    return Err(self.parse_error(
                                        "exactly one argument for require statement",
                                    ));
                                }
                                let Some(arg) = args.into_iter().next() else {
                                    return Err(self.parse_error(
                                        "expected one boolean expression inside require(...)",
                                    ));
                                };
                                Ok(AstNode::RequireStatement {
                                    condition: Box::new(arg),
                                })
                            } else {
                                // Allow other function calls as statements
                                Ok(AstNode::FunctionCall { name, args })
                            }
                        }
                        AstNode::MethodCall {
                            object,
                            method,
                            args,
                        } => {
                            if matches!(self.current_token, Token::Semicolon) {
                                self.advance();
                            }
                            Ok(AstNode::MethodCall {
                                object,
                                method,
                                args,
                            })
                        }
                        // Allow bare identifiers/field accesses/etc. as expression statements
                        other_expr => {
                            if matches!(self.current_token, Token::Semicolon) {
                                self.advance();
                            }
                            Ok(other_expr)
                        }
                    }
                }
            }
            // Handle expression statements (like Ok(()), Some(value), etc.)
            _ => {
                // Try to parse as an expression statement
                let expr = self.parse_expression()?;

                // Optional semicolon
                if matches!(self.current_token, Token::Semicolon) {
                    self.advance();
                }

                Ok(expr)
            }
        }
    }
    pub(crate) fn parse_if_statement(&mut self) -> Result<AstNode, VMError> {
        // Consume 'if' keyword
        if !matches!(self.current_token, Token::If) {
            return Err(self.parse_error("'if' keyword"));
        }
        self.advance();

        // Parse condition expression
        let condition = Box::new(self.parse_expression()?);

        // Parse then branch (block)
        let then_branch = Box::new(self.parse_block(BlockKind::Regular)?);

        // Check for optional else branch
        let else_branch = if matches!(self.current_token, Token::Else) {
            self.advance(); // consume 'else'

            // Handle both 'else if' and 'else { ... }'
            if matches!(self.current_token, Token::If) {
                Some(Box::new(self.parse_if_statement()?))
            } else {
                Some(Box::new(self.parse_block(BlockKind::Regular)?))
            }
        } else {
            None
        };

        Ok(AstNode::IfStatement {
            condition,
            then_branch,
            else_branch,
        })
    }

    pub(crate) fn parse_match_expression(&mut self) -> Result<AstNode, VMError> {
        // Consume 'match' keyword
        if !matches!(self.current_token, Token::Match) {
            return Err(self.parse_error("'match' keyword"));
        }
        self.advance();

        // Parse expression to match on
        let expression = Box::new(self.parse_expression()?);

        // Parse match arms: { pattern => body, ... }
        if !matches!(self.current_token, Token::LeftBrace) {
            return Err(self.parse_error("'{' to start match arms"));
        }
        self.advance(); // consume '{'

        let mut arms = Vec::new();

        while !matches!(self.current_token, Token::RightBrace)
            && !matches!(self.current_token, Token::Eof)
        {
            // Parse pattern
            let pattern = Box::new(self.parse_expression()?);

            // Optional guard: `if <expr>`
            let guard = if matches!(self.current_token, Token::If) {
                self.advance(); // consume 'if'
                Some(Box::new(self.parse_expression()?))
            } else {
                None
            };

            // Expect '=>' (accept either FatArrow or Arrow token)
            if !matches!(self.current_token, Token::FatArrow | Token::Arrow) {
                return Err(self.parse_error("'=>' after match pattern"));
            }
            // consume '=>' or '->'
            self.advance();

            // Parse arm body (statement or block)
            let body = if matches!(self.current_token, Token::LeftBrace) {
                Box::new(self.parse_block(BlockKind::Regular)?)
            } else {
                Box::new(self.parse_statement()?)
            };

            arms.push(MatchArm {
                pattern,
                guard,
                body,
            });

            // Optional comma between arms
            if matches!(self.current_token, Token::Comma) {
                self.advance();
            }
        }

        if !matches!(self.current_token, Token::RightBrace) {
            return Err(self.parse_error("'}' to end match expression"));
        }
        self.advance(); // consume '}'

        Ok(AstNode::MatchExpression { expression, arms })
    }

    pub(crate) fn parse_return_statement(&mut self) -> Result<AstNode, VMError> {
        // Consume 'return' keyword
        if !matches!(self.current_token, Token::Return) {
            return Err(self.parse_error("'return' keyword"));
        }
        self.advance();

        // Parse optional return value
        let value = if matches!(self.current_token, Token::Semicolon | Token::RightBrace) {
            None // Early return without value
        } else {
            let first = self.parse_expression()?;
            if matches!(self.current_token, Token::Comma) {
                let mut elements = vec![first];
                while matches!(self.current_token, Token::Comma) {
                    self.advance(); // consume ','
                    elements.push(self.parse_expression()?);
                }
                Some(Box::new(AstNode::TupleLiteral { elements }))
            } else {
                Some(Box::new(first))
            }
        };

        // Optional semicolon
        if matches!(self.current_token, Token::Semicolon) {
            self.advance();
        }

        Ok(AstNode::ReturnStatement { value })
    }

    pub(crate) fn parse_require_statement(&mut self) -> Result<AstNode, VMError> {
        // Consume 'require' keyword
        if !matches!(self.current_token, Token::Require) {
            return Err(self.parse_error("'require' keyword"));
        }
        self.advance();

        // Expect '(' to start condition
        if !matches!(self.current_token, Token::LeftParen) {
            return Err(self.parse_error("'(' to start require condition"));
        }
        self.advance(); // consume '('

        // Parse condition expression
        let condition = Box::new(self.parse_expression()?);

        // Expect ')' to end condition
        if !matches!(self.current_token, Token::RightParen) {
            return Err(self.parse_error("')' to end require condition"));
        }
        self.advance(); // consume ')'

        // Expect semicolon
        if !matches!(self.current_token, Token::Semicolon) {
            return Err(self.parse_error("';' after require statement"));
        }
        self.advance(); // consume ';'

        Ok(AstNode::RequireStatement { condition })
    }

    // Parse while loop: while (condition) { body }
    pub(crate) fn parse_while_loop(&mut self) -> Result<AstNode, VMError> {
        // Consume 'while' keyword
        if !matches!(self.current_token, Token::While) {
            return Err(self.parse_error("'while' keyword"));
        }
        self.advance();

        // Parse condition in parentheses
        if !matches!(self.current_token, Token::LeftParen) {
            return Err(self.parse_error("'(' after 'while'"));
        }
        self.advance(); // consume '('

        let condition = Box::new(self.parse_expression()?);

        if !matches!(self.current_token, Token::RightParen) {
            return Err(self.parse_error("')' after while condition"));
        }
        self.advance(); // consume ')'

        // Parse body
        let body = Box::new(self.parse_block_or_statement()?);

        Ok(AstNode::WhileLoop { condition, body })
    }

    // Parse for loop: for (init; condition; update) { body } or for (variable in iterable) { body }
    pub(crate) fn parse_for_loop(&mut self) -> Result<AstNode, VMError> {
        // Consume 'for' keyword
        if !matches!(self.current_token, Token::For) {
            return Err(self.parse_error("'for' keyword"));
        }
        self.advance();

        if !matches!(self.current_token, Token::LeftParen) {
            return Err(self.parse_error("'(' after 'for'"));
        }
        self.advance(); // consume '('

        // Look ahead to determine for-in vs C-style for loop
        let saved_position = self.position;
        let mut lookahead_count = 0;
        let mut found_in = false;

        // Scan until we find 'in' or ';' to determine loop type
        while lookahead_count < 10 && self.position < self.tokens.len() {
            match &self.current_token {
                Token::In => {
                    found_in = true;
                    break;
                }
                Token::Semicolon | Token::RightParen => {
                    break;
                }
                _ => {
                    self.advance();
                    lookahead_count += 1;
                }
            }
        }

        // Restore position
        self.position = saved_position;
        self.current_token = self
            .tokens
            .get(self.position)
            .cloned()
            .unwrap_or(Token::Eof);

        if found_in {
            // for-in loop: for (variable in iterable) { body }
            let variable = match &self.current_token {
                Token::Identifier(name) => name.clone(),
                _ => return Err(self.parse_error("variable name for for-in loop")),
            };
            self.advance();

            if !matches!(self.current_token, Token::In) {
                return Err(self.parse_error("'in' keyword in for-in loop"));
            }
            self.advance(); // consume 'in'

            let iterable = Box::new(self.parse_expression()?);

            if !matches!(self.current_token, Token::RightParen) {
                return Err(self.parse_error("')' after for-in clause"));
            }
            self.advance(); // consume ')'

            let body = Box::new(self.parse_block_or_statement()?);

            Ok(AstNode::ForInLoop {
                variable,
                iterable,
                body,
            })
        } else {
            // C-style for loop: for (init; condition; update) { body }
            let init = if matches!(self.current_token, Token::Semicolon) {
                None
            } else {
                Some(Box::new(self.parse_for_init_statement()?))
            };

            if !matches!(self.current_token, Token::Semicolon) {
                return Err(self.parse_error("';' after for loop init"));
            }
            self.advance(); // consume ';'

            let condition = if matches!(self.current_token, Token::Semicolon) {
                None
            } else {
                Some(Box::new(self.parse_expression()?))
            };

            if !matches!(self.current_token, Token::Semicolon) {
                return Err(self.parse_error("';' after for loop condition"));
            }
            self.advance(); // consume ';'

            let update = if matches!(self.current_token, Token::RightParen) {
                None
            } else {
                Some(Box::new(self.parse_for_update_expression()?))
            };

            if !matches!(self.current_token, Token::RightParen) {
                return Err(self.parse_error("')' after for loop clause"));
            }
            self.advance(); // consume ')'

            let body = Box::new(self.parse_block_or_statement()?);

            Ok(AstNode::ForLoop {
                init,
                condition,
                update,
                body,
            })
        }
    }

    // Parse do-while loop: do { body } while (condition);
    pub(crate) fn parse_do_while_loop(&mut self) -> Result<AstNode, VMError> {
        // Consume 'do' keyword
        if !matches!(self.current_token, Token::Do) {
            return Err(self.parse_error("'do' keyword"));
        }
        self.advance();

        // Parse body
        let body = Box::new(self.parse_block_or_statement()?);

        // Expect 'while' keyword
        if !matches!(self.current_token, Token::While) {
            return Err(self.parse_error("'while' keyword after do body"));
        }
        self.advance(); // consume 'while'

        // Parse condition in parentheses
        if !matches!(self.current_token, Token::LeftParen) {
            return Err(self.parse_error("'(' after 'while' in do-while"));
        }
        self.advance(); // consume '('

        let condition = Box::new(self.parse_expression()?);

        if !matches!(self.current_token, Token::RightParen) {
            return Err(self.parse_error("')' after do-while condition"));
        }
        self.advance(); // consume ')'

        // Optional semicolon
        if matches!(self.current_token, Token::Semicolon) {
            self.advance();
        }

        Ok(AstNode::DoWhileLoop { body, condition })
    }

    pub(crate) fn parse_for_update_expression(&mut self) -> Result<AstNode, VMError> {
        // Check if this is an assignment by looking ahead
        if let Token::Identifier(_) = &self.current_token {
            let saved_position = self.position;
            self.advance(); // consume identifier

            let is_assignment = matches!(self.current_token, Token::Assign);

            // Restore position
            self.position = saved_position;
            self.current_token = self
                .tokens
                .get(self.position)
                .cloned()
                .unwrap_or(Token::Eof);

            if is_assignment {
                // Parse as assignment
                let target = match &self.current_token {
                    Token::Identifier(name) => name.clone(),
                    _ => return Err(self.parse_error("variable name for assignment")),
                };
                self.advance(); // consume identifier

                if !matches!(self.current_token, Token::Assign) {
                    return Err(self.parse_error("'=' for assignment"));
                }
                self.advance(); // consume '='

                let value = Box::new(self.parse_expression()?);

                return Ok(AstNode::Assignment { target, value });
            }
        }

        // Parse as regular expression
        self.parse_expression()
    }

    // Parse for loop init statement without consuming semicolon
    pub(crate) fn parse_for_init_statement(&mut self) -> Result<AstNode, VMError> {
        match &self.current_token {
            Token::Let => {
                self.advance(); // consume 'let'

                // Check for 'mut' keyword
                let is_mutable = if matches!(self.current_token, Token::Mut) {
                    self.advance(); // consume 'mut'
                    true
                } else {
                    false
                };

                // Parse variable name
                let name = match &self.current_token {
                    Token::Identifier(name) => name.clone(),
                    _ => return Err(self.parse_error("variable name identifier")),
                };
                self.advance();

                // Parse optional type annotation: : Type
                let type_annotation = if matches!(self.current_token, Token::Colon) {
                    self.advance(); // consume ':'
                    Some(Box::new(types::parse_type(self)?))
                } else {
                    None
                };

                // Parse assignment: = value
                if !matches!(self.current_token, Token::Assign) {
                    return Err(self.parse_error("'=' for variable assignment"));
                }
                self.advance(); // consume '='

                let value = Box::new(self.parse_expression()?);

                // Don't consume semicolon - let the for loop parser handle it

                Ok(AstNode::LetStatement {
                    name,
                    type_annotation,
                    is_mutable,
                    value,
                })
            }
            _ => {
                // For non-let statements, parse as expression
                self.parse_expression()
            }
        }
    }
}
