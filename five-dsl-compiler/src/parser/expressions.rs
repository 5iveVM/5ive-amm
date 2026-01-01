use super::*;

impl DslParser {
    pub(crate) fn parse_expression(&mut self) -> Result<AstNode, VMError> {
        self.parse_logical_or()
    }

    pub(crate) fn parse_logical_or(&mut self) -> Result<AstNode, VMError> {
        let mut left = self.parse_logical_and()?;

        while matches!(self.current_token, Token::LogicalOr) {
            self.advance(); // consume '||'
            let right = self.parse_logical_and()?;

            left = AstNode::MethodCall {
                object: Box::new(left),
                method: "or".to_string(),
                args: vec![right],
            };
        }

        Ok(left)
    }

    // Parse logical AND expressions: expr && expr
    pub(crate) fn parse_logical_and(&mut self) -> Result<AstNode, VMError> {
        let mut left = self.parse_bitwise_or()?;

        while matches!(self.current_token, Token::LogicalAnd) {
            self.advance(); // consume '&&'
            let right = self.parse_bitwise_or()?;

            left = AstNode::MethodCall {
                object: Box::new(left),
                method: "and".to_string(),
                args: vec![right],
            };
        }

        Ok(left)
    }

    // Parse bitwise OR: expr | expr
    pub(crate) fn parse_bitwise_or(&mut self) -> Result<AstNode, VMError> {
        let mut left = self.parse_bitwise_xor()?;

        while matches!(self.current_token, Token::BitwiseOr) {
            self.advance(); // consume '|'
            let right = self.parse_bitwise_xor()?;

            left = AstNode::BinaryExpression {
                left: Box::new(left),
                operator: "|".to_string(),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    // Parse bitwise XOR: expr ^ expr
    pub(crate) fn parse_bitwise_xor(&mut self) -> Result<AstNode, VMError> {
        let mut left = self.parse_bitwise_and()?;

        while matches!(self.current_token, Token::BitwiseXor) {
            self.advance(); // consume '^'
            let right = self.parse_bitwise_and()?;

            left = AstNode::BinaryExpression {
                left: Box::new(left),
                operator: "^".to_string(),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    // Parse bitwise AND: expr & expr
    pub(crate) fn parse_bitwise_and(&mut self) -> Result<AstNode, VMError> {
        let mut left = self.parse_comparison()?;

        while matches!(self.current_token, Token::BitwiseAnd) {
            self.advance(); // consume '&'
            let right = self.parse_comparison()?;

            left = AstNode::BinaryExpression {
                left: Box::new(left),
                operator: "&".to_string(),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    // Parse range expressions: start..end
    pub(crate) fn parse_range(&mut self) -> Result<AstNode, VMError> {
        let left = self.parse_additive()?;

        if matches!(self.current_token, Token::Range) {
            self.advance(); // consume '..'
            let right = self.parse_additive()?;

            // Create a range expression - we'll treat it as a binary expression for now
            Ok(AstNode::BinaryExpression {
                operator: "range".to_string(),
                left: Box::new(left),
                right: Box::new(right),
            })
        } else {
            Ok(left)
        }
    }

    // TDD Phase 1.3: Implement precedence climbing parser (arithmetic as BinaryExpression)
    pub(crate) fn parse_additive(&mut self) -> Result<AstNode, VMError> {
        let mut left = self.parse_multiplicative()?;

        while matches!(
            self.current_token,
            Token::Plus | Token::PlusChecked | Token::Minus | Token::MinusChecked
        ) {
            let op = self.current_token.clone();
            self.advance();
            let right = self.parse_multiplicative()?;

            let operator = match op {
                Token::Plus => "+",
                Token::PlusChecked => "+?",
                Token::Minus => "-",
                Token::MinusChecked => "-?",
                _ => "unknown",
            };

            // Represent arithmetic additive operators as BinaryExpression nodes
            left = AstNode::BinaryExpression {
                left: Box::new(left),
                operator: operator.to_string(),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    pub(crate) fn parse_multiplicative(&mut self) -> Result<AstNode, VMError> {
        let mut left = self.parse_unary()?;

        while matches!(
            self.current_token,
            Token::Multiply | Token::MultiplyChecked | Token::Divide | Token::Percent
        ) {
            let op = self.current_token.clone();
            self.advance();
            let right = self.parse_unary()?;

            let operator = match op {
                Token::Multiply => "*",
                Token::MultiplyChecked => "*?",
                Token::Divide => "/",
                Token::Percent => "%",
                _ => "unknown",
            };

            // Represent multiplicative operators as BinaryExpression nodes
            left = AstNode::BinaryExpression {
                left: Box::new(left),
                operator: operator.to_string(),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    pub(crate) fn parse_unary(&mut self) -> Result<AstNode, VMError> {
        match &self.current_token {
            Token::Bang => {
                self.advance(); // consume '!'
                let operand = Box::new(self.parse_unary()?);
                Ok(AstNode::UnaryExpression {
                    operator: "not".to_string(),
                    operand,
                })
            }
            Token::Minus => {
                self.advance(); // consume '-'
                let operand = Box::new(self.parse_unary()?);
                Ok(AstNode::UnaryExpression {
                    operator: "neg".to_string(),
                    operand,
                })
            }
            Token::Plus => {
                self.advance(); // consume '+'
                let operand = Box::new(self.parse_unary()?);
                Ok(AstNode::UnaryExpression {
                    operator: "pos".to_string(),
                    operand,
                })
            }
            Token::BitwiseTilde => {
                self.advance(); // consume '~'
                let operand = Box::new(self.parse_unary()?);
                Ok(AstNode::UnaryExpression {
                    operator: "~".to_string(),
                    operand,
                })
            }
            _ => self.parse_field_access(),
        }
    }

    pub(crate) fn parse_comparison(&mut self) -> Result<AstNode, VMError> {
        let left = self.parse_shift()?;

        // Handle comparison operators
        match &self.current_token {
            Token::GT => {
                self.advance();
                let right = self.parse_shift()?;
                Ok(AstNode::MethodCall {
                    object: Box::new(left),
                    method: "gt".to_string(),
                    args: vec![right],
                })
            }
            Token::LT => {
                self.advance();
                let right = self.parse_shift()?;
                Ok(AstNode::MethodCall {
                    object: Box::new(left),
                    method: "lt".to_string(),
                    args: vec![right],
                })
            }
            Token::Equal => {
                self.advance();
                let right = self.parse_shift()?;
                Ok(AstNode::MethodCall {
                    object: Box::new(left),
                    method: "eq".to_string(),
                    args: vec![right],
                })
            }
            Token::LessEqual => {
                self.advance();
                let right = self.parse_shift()?;
                Ok(AstNode::MethodCall {
                    object: Box::new(left),
                    method: "lte".to_string(),
                    args: vec![right],
                })
            }
            Token::GreaterEqual => {
                self.advance();
                let right = self.parse_shift()?;
                Ok(AstNode::MethodCall {
                    object: Box::new(left),
                    method: "gte".to_string(),
                    args: vec![right],
                })
            }
            Token::NotEqual => {
                self.advance();
                let right = self.parse_shift()?;
                Ok(AstNode::MethodCall {
                    object: Box::new(left),
                    method: "ne".to_string(),
                    args: vec![right],
                })
            }
            _ => Ok(left),
        }
    }

    // Parse shift expressions: expr << expr, expr >> expr, expr >>> expr, expr <<< expr
    pub(crate) fn parse_shift(&mut self) -> Result<AstNode, VMError> {
        let mut left = self.parse_range()?;

        while matches!(
            self.current_token,
            Token::LeftShift | Token::RightShift | Token::ArithRightShift | Token::RotateLeft
        ) {
            let op = self.current_token.clone();
            self.advance();
            let right = self.parse_range()?;

            let operator = match op {
                Token::LeftShift => "<<",
                Token::RightShift => ">>",
                Token::ArithRightShift => ">>>",
                Token::RotateLeft => "<<<",
                _ => "unknown",
            };

            left = AstNode::BinaryExpression {
                left: Box::new(left),
                operator: operator.to_string(),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    pub(crate) fn parse_field_access(&mut self) -> Result<AstNode, VMError> {
        let mut expr = self.parse_primary()?;

        // Handle postfix operators with left-associativity
        loop {
            match &self.current_token {
                // Rust-style cast: expr as Type (ignore at AST level, used only for type hints)
                Token::As => {
                    self.advance(); // consume 'as'
                                    // Parse and ignore the type annotation after 'as'
                    let _ = self.parse_type()?;
                    // Do not change expr; keep parsing further postfix ops
                }
                // Field access: object.field1.field2
                Token::Dot => {
                    self.advance(); // consume '.'

                    if let Token::NumberLiteral(index) = self.current_token {
                        self.advance(); // consume number
                        expr = AstNode::TupleAccess {
                            object: Box::new(expr),
                            index: index as u32,
                        };
                    } else {
                        let field_name = match &self.current_token {
                            Token::Identifier(name) => name.clone(),
                            Token::Type(name) if name == "lamports" => name.clone(),
                            _ => return Err(self.parse_error("expected field identifier after .")),
                        };
                        self.advance(); // consume field identifier

                        // Check if this field access is followed by parentheses (method call)
                        if matches!(self.current_token, Token::LeftParen) {
                            let (args, _trailing) = self.parse_list(
                                TokenKind::LeftParen,
                                TokenKind::RightParen,
                                TokenKind::Comma,
                                true,
                                |s| s.parse_argument_expr(),
                            )?;

                            expr = AstNode::MethodCall {
                                object: Box::new(expr),
                                method: field_name,
                                args,
                            };
                        } else {
                            expr = AstNode::FieldAccess {
                                object: Box::new(expr),
                                field: field_name,
                            };
                        }
                    }
                }
                // Array indexing: expr[index]
                Token::LeftBracket => {
                    self.advance(); // consume '['
                    let index = self.parse_expression()?;

                    if !matches!(self.current_token, Token::RightBracket) {
                        return Err(self.parse_error("']' to end array indexing"));
                    }
                    self.advance(); // consume ']'

                    expr = AstNode::ArrayAccess {
                        array: Box::new(expr),
                        index: Box::new(index),
                    };
                }
                // Error propagation: expression?
                Token::Question => {
                    self.advance(); // consume '?'
                    expr = AstNode::ErrorPropagation {
                        expression: Box::new(expr),
                    };
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    pub(crate) fn parse_primary(&mut self) -> Result<AstNode, VMError> {
        match &self.current_token {
            Token::NumberLiteral(n) => {
                let value = *n;
                self.advance();
                Ok(AstNode::Literal(Value::U64(value)))
            }
            Token::True => {
                self.advance();
                Ok(AstNode::Literal(Value::Bool(true)))
            }
            Token::False => {
                self.advance();
                Ok(AstNode::Literal(Value::Bool(false)))
            }
            Token::StringLiteral(s) => {
                let value = s.clone();
                self.advance();
                Ok(AstNode::StringLiteral { value })
            }
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance();

                // Check for enum variant access: Identifier::Variant
                if matches!(self.current_token, Token::DoubleColon) {
                    self.advance(); // consume '::'
                    let variant_name = match &self.current_token {
                        Token::Identifier(variant) => variant.clone(),
                        _ => return Err(self.parse_error("enum variant name identifier")),
                    };
                    self.advance(); // consume variant identifier

                    // Check if this is a namespaced function call: Module::Function(...)
                    if matches!(self.current_token, Token::LeftParen) {
                        let (args, _) = self.parse_list(
                            TokenKind::LeftParen,
                            TokenKind::RightParen,
                            TokenKind::Comma,
                            true,
                            |s| s.parse_argument_expr(),
                        )?;
                        
                        let full_name = format!("{}::{}", name, variant_name);
                        Ok(AstNode::FunctionCall { name: full_name, args })
                    } else {
                        Ok(AstNode::EnumVariantAccess {
                            enum_name: name,
                            variant_name,
                        })
                    }
                }
                // Check if this is a function call
                else if matches!(self.current_token, Token::LeftParen) {
                    let (args, _trailing) = self.parse_list(
                        TokenKind::LeftParen,
                        TokenKind::RightParen,
                        TokenKind::Comma,
                        true,
                        |s| s.parse_argument_expr(),
                    )?;
                    Ok(AstNode::FunctionCall { name, args })
                } else {
                    Ok(AstNode::Identifier(name))
                }
            }
            // Permit using keyword 'account' as an identifier in expressions
            Token::Account => {
                self.advance();
                Ok(AstNode::Identifier("account".to_string()))
            }
            Token::LeftParen => {
                // Use list parser to handle tuples, trailing commas, and parens
                let (items, had_trailing) = self.parse_list(
                    TokenKind::LeftParen,
                    TokenKind::RightParen,
                    TokenKind::Comma,
                    true,
                    |s| s.parse_expression(),
                )?;

                if items.is_empty() {
                    return Err(self.parse_error("expected expression inside parentheses"));
                }
                if items.len() == 1 {
                    if had_trailing {
                        return Ok(AstNode::TupleLiteral { elements: items });
                    }
                    return Ok(items.into_iter().next().unwrap());
                }
                Ok(AstNode::TupleLiteral { elements: items })
            }
            Token::Ok => {
                // Handle Ok(...) as a function call expression
                let name = "Ok".to_string();
                self.advance();

                if matches!(self.current_token, Token::LeftParen) {
                    let (args, _) = self.parse_list(
                        TokenKind::LeftParen,
                        TokenKind::RightParen,
                        TokenKind::Comma,
                        true,
                        |s| s.parse_argument_expr(),
                    )?;
                    Ok(AstNode::FunctionCall { name, args })
                } else {
                    // Just Ok without parentheses
                    Ok(AstNode::Identifier(name))
                }
            }
            Token::Some => {
                // Handle Some(...) as a function call expression
                let name = "Some".to_string();
                self.advance();

                if matches!(self.current_token, Token::LeftParen) {
                    let (args, _) = self.parse_list(
                        TokenKind::LeftParen,
                        TokenKind::RightParen,
                        TokenKind::Comma,
                        true,
                        |s| s.parse_argument_expr(),
                    )?;
                    Ok(AstNode::FunctionCall { name, args })
                } else {
                    // Just Some without parentheses
                    Ok(AstNode::Identifier(name))
                }
            }
            Token::None => {
                self.advance();
                Ok(AstNode::Identifier("None".to_string()))
            }
            Token::Err => {
                // Handle Err(...) as a function call expression
                let name = "Err".to_string();
                self.advance();

                if matches!(self.current_token, Token::LeftParen) {
                    let (args, _) = self.parse_list(
                        TokenKind::LeftParen,
                        TokenKind::RightParen,
                        TokenKind::Comma,
                        true,
                        |s| s.parse_expression(),
                    )?;
                    Ok(AstNode::FunctionCall { name, args })
                } else {
                    // Just Err without parentheses
                    Ok(AstNode::Identifier(name))
                }
            }
            Token::LeftBracket => {
                let (elements, _) = self.parse_list(
                    TokenKind::LeftBracket,
                    TokenKind::RightBracket,
                    TokenKind::Comma,
                    true,
                    |s| s.parse_expression(),
                )?;
                Ok(AstNode::ArrayLiteral { elements })
            }
            Token::LeftBrace => {
                let (fields, _) = self.parse_list(
                    TokenKind::LeftBrace,
                    TokenKind::RightBrace,
                    TokenKind::Comma,
                    true,
                    |s| {
                        let field_name = match &s.current_token {
                            Token::Identifier(name) => name.clone(),
                            _ => return Err(s.parse_error("struct field name identifier")),
                        };
                        s.advance();
                        if !matches!(s.current_token, Token::Colon) {
                            return Err(s.parse_error(":' after struct field name"));
                        }
                        s.advance();
                        let value = Box::new(s.parse_expression()?);
                        Ok(StructLiteralField { field_name, value })
                    },
                )?;
                Ok(AstNode::StructLiteral { fields })
            }
            _ => Err(self.parse_error(
                "primary expression (literal, identifier, or parenthesized expression)",
            )),
        }
    }
    pub(crate) fn parse_argument_expr(&mut self) -> Result<AstNode, VMError> {
        let expr = self.parse_expression()?;
        // Consume optional callsite modifiers (ignored in AST)
        while matches!(self.current_token, Token::AtMut | Token::AtSigner) {
            self.advance();
        }
        Ok(expr)
    }
}
