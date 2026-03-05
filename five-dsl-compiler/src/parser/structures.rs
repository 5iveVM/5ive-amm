use crate::ast::{AccountSerializer, AstNode, ErrorVariant, StructField};
use crate::parser::{types, DslParser};
use crate::tokenizer::Token;
use five_vm_mito::error::VMError;

fn parse_serializer_name(parser: &mut DslParser) -> Result<String, VMError> {
    match &parser.current_token {
        Token::StringLiteral(s) => {
            let out = s.clone();
            parser.advance();
            Ok(out)
        }
        Token::Identifier(s) => {
            let out = s.clone();
            parser.advance();
            Ok(out)
        }
        _ => Err(parser.parse_error("serializer name (identifier or string literal)")),
    }
}

fn parse_account_serializer(parser: &mut DslParser) -> Result<AccountSerializer, VMError> {
    let name = parse_serializer_name(parser)?;
    match name.as_str() {
        "raw" => Ok(AccountSerializer::Raw),
        "borsh" => Ok(AccountSerializer::Borsh),
        "bincode" => Ok(AccountSerializer::Bincode),
        "anchor" => Ok(AccountSerializer::Anchor),
        _ => Err(parser.parse_error("valid serializer: raw, borsh, bincode, or anchor")),
    }
}

#[allow(dead_code)]
pub(crate) fn parse_field_definition(parser: &mut DslParser) -> Result<AstNode, VMError> {
    // Check for 'pub' keyword to determine visibility
    let visibility = if matches!(parser.current_token, Token::Pub) {
        parser.advance(); // consume 'pub'
        crate::Visibility::Public
    } else {
        crate::Visibility::Internal
    };

    // Handle optional mutability: mut field_name
    let is_mutable = if matches!(parser.current_token, Token::Mut) {
        parser.advance(); // consume 'mut'
        true
    } else {
        false
    };

    // Handle 'field' keyword if present
    if matches!(parser.current_token, Token::Field) {
        parser.advance(); // consume 'field'
    }

    // Parse field name
    let name = match &parser.current_token {
        Token::Identifier(name) => name.clone(),
        Token::Account => "account".to_string(),
        _ => return Err(parser.parse_error("field name identifier")),
    };
    parser.advance();

    // Check for optional marker: name?
    let is_optional = if matches!(parser.current_token, Token::Question) {
        parser.advance(); // consume '?'
        true
    } else {
        false
    };

    // Parse type annotation: : Type
    if !matches!(parser.current_token, Token::Colon) {
        return Err(parser.parse_error("':' after field name for type annotation"));
    }
    parser.advance(); // consume ':'

    let field_type = Box::new(types::parse_type(parser)?);

    // Parse optional default value: = expression
    let default_value = if matches!(parser.current_token, Token::Assign) {
        parser.advance(); // consume '='
        Some(Box::new(parser.parse_expression()?))
    } else {
        None
    };

    // Optional semicolon
    if matches!(parser.current_token, Token::Semicolon) {
        parser.advance();
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

// Parse an argument expression, allowing optional trailing @mut/@signer markers in callsites
pub(crate) fn parse_event_definition(parser: &mut DslParser) -> Result<AstNode, VMError> {
    // Check for 'pub' keyword to determine visibility
    let visibility = if matches!(parser.current_token, Token::Pub) {
        parser.advance(); // consume 'pub'
        crate::Visibility::Public
    } else {
        crate::Visibility::Internal
    };

    // Consume 'event' keyword
    if !matches!(parser.current_token, Token::Event) {
        return Err(parser.parse_error("'event' keyword"));
    }
    parser.advance();

    // Parse event name
    let name = match &parser.current_token {
        Token::Identifier(name) => name.clone(),
        _ => return Err(parser.parse_error("event name identifier")),
    };
    parser.advance();

    // Parse event fields: { field1: Type, field2: Type }
    if !matches!(parser.current_token, Token::LeftBrace) {
        return Err(parser.parse_error("'{' to start event fields"));
    }
    parser.advance(); // consume '{'

    let mut fields = Vec::new();

    while !matches!(parser.current_token, Token::RightBrace)
        && !matches!(parser.current_token, Token::Eof)
    {
        // Parse field name
        let field_name = match &parser.current_token {
            Token::Identifier(name) => name.clone(),
            Token::Account => "account".to_string(),
            _ => return Err(parser.parse_error("event field name identifier")),
        };
        parser.advance();

        if !matches!(parser.current_token, Token::Colon) {
            return Err(parser.parse_error("':' after field name"));
        }
        parser.advance(); // consume ':'

        let field_type = types::parse_type(parser)?;

        fields.push(StructField {
            name: field_name,
            field_type,
            is_mutable: false,  // Event fields are immutable
            is_optional: false, // Event fields are required by default
        });

        if matches!(parser.current_token, Token::Comma) {
            parser.advance(); // consume ','
        } else {
            break;
        }
    }

    if !matches!(parser.current_token, Token::RightBrace) {
        return Err(parser.parse_error("'}' to end event fields"));
    }
    parser.advance(); // consume '}'

    Ok(AstNode::EventDefinition {
        name,
        fields,
        visibility,
    })
}

pub(crate) fn parse_error_type_definition(parser: &mut DslParser) -> Result<AstNode, VMError> {
    parser.advance(); // consume 'enum'

    // Parse enum name
    let name = match &parser.current_token {
        Token::Identifier(name) => name.clone(),
        _ => return Err(parser.parse_error("enum name identifier")),
    };
    parser.advance();

    // Parse enum body: { variant1, variant2, ... }
    if !matches!(parser.current_token, Token::LeftBrace) {
        return Err(parser.parse_error("'{' to start enum body"));
    }
    parser.advance(); // consume '{'

    let mut variants = Vec::new();

    while !matches!(parser.current_token, Token::RightBrace)
        && !matches!(parser.current_token, Token::Eof)
    {
        // Parse variant name
        let variant_name = match &parser.current_token {
            Token::Identifier(name) => name.clone(),
            _ => return Err(parser.parse_error("enum variant name identifier")),
        };
        parser.advance();

        // Parse optional variant data
        let mut fields = Vec::new();

        // Tuple variant: Variant(T1, T2)
        if matches!(parser.current_token, Token::LeftParen) {
            parser.advance(); // consume '('
            let mut index = 0;

            while !matches!(parser.current_token, Token::RightParen)
                && !matches!(parser.current_token, Token::Eof)
            {
                let field_type = types::parse_type(parser)?;
                fields.push(StructField {
                    name: format!("field{}", index),
                    field_type,
                    is_mutable: false,
                    is_optional: false,
                });
                index += 1;

                if matches!(parser.current_token, Token::Comma) {
                    parser.advance();
                } else {
                    break;
                }
            }

            if !matches!(parser.current_token, Token::RightParen) {
                return Err(parser.parse_error("')' to end tuple variant"));
            }
            parser.advance(); // consume ')'

        // Struct variant: Variant { field: Type }
        } else if matches!(parser.current_token, Token::LeftBrace) {
            parser.advance(); // consume '{'

            while !matches!(parser.current_token, Token::RightBrace)
                && !matches!(parser.current_token, Token::Eof)
            {
                let field_name = match &parser.current_token {
                    Token::Identifier(name) => name.clone(),
                    _ => return Err(parser.parse_error("struct variant field name identifier")),
                };
                parser.advance();

                if !matches!(parser.current_token, Token::Colon) {
                    return Err(parser.parse_error("':' after struct variant field name"));
                }
                parser.advance(); // consume ':'

                let field_type = types::parse_type(parser)?;
                fields.push(StructField {
                    name: field_name,
                    field_type,
                    is_mutable: false,
                    is_optional: false,
                });

                if matches!(parser.current_token, Token::Comma) {
                    parser.advance();
                } else {
                    break;
                }
            }

            if !matches!(parser.current_token, Token::RightBrace) {
                return Err(parser.parse_error("'}' to end struct variant"));
            }
            parser.advance(); // consume '}'
        }

        variants.push(ErrorVariant {
            name: variant_name,
            fields,
        });

        // Handle comma separation
        if matches!(parser.current_token, Token::Comma) {
            parser.advance();
        } else if !matches!(parser.current_token, Token::RightBrace) {
            return Err(parser.parse_error("',' between enum variants or '}' to end enum"));
        }
    }

    if !matches!(parser.current_token, Token::RightBrace) {
        return Err(parser.parse_error("'}' to end enum body"));
    }
    parser.advance(); // consume '}'

    Ok(AstNode::ErrorTypeDefinition { name, variants })
}

pub(crate) fn parse_account_definition(parser: &mut DslParser) -> Result<AstNode, VMError> {
    // Check for 'pub' keyword to determine visibility
    let visibility = if matches!(parser.current_token, Token::Pub) {
        parser.advance(); // consume 'pub'
        crate::Visibility::Public
    } else {
        crate::Visibility::Internal
    };

    parser.advance(); // consume 'account'

    // Parse account name
    let name = match &parser.current_token {
        Token::Identifier(name) => name.clone(),
        _ => return Err(parser.parse_error("account name identifier")),
    };
    parser.advance();

    // Optional serializer hint: @serializer("borsh") or serializer("borsh")
    let mut serializer: Option<AccountSerializer> = None;
    if matches!(parser.current_token, Token::At) {
        let saved_pos = parser.position;
        parser.advance(); // consume '@'
        let is_serializer_attr = matches!(&parser.current_token, Token::Identifier(name) if name == "serializer")
            || matches!(parser.current_token, Token::Serializer);
        if is_serializer_attr {
            parser.advance(); // consume serializer token
            if !matches!(parser.current_token, Token::LeftParen) {
                return Err(parser.parse_error("'(' after serializer attribute"));
            }
            parser.advance(); // consume '('
            serializer = Some(parse_account_serializer(parser)?);
            if !matches!(parser.current_token, Token::RightParen) {
                return Err(parser.parse_error("')' after serializer name"));
            }
            parser.advance(); // consume ')'
        } else {
            parser.position = saved_pos;
            parser.current_token = parser
                .tokens
                .get(parser.position)
                .cloned()
                .unwrap_or(Token::Eof);
        }
    } else if matches!(&parser.current_token, Token::Identifier(name) if name == "serializer")
        || matches!(parser.current_token, Token::Serializer)
    {
        parser.advance(); // consume serializer token
        if !matches!(parser.current_token, Token::LeftParen) {
            return Err(parser.parse_error("'(' after serializer keyword"));
        }
        parser.advance(); // consume '('
        serializer = Some(parse_account_serializer(parser)?);
        if !matches!(parser.current_token, Token::RightParen) {
            return Err(parser.parse_error("')' after serializer name"));
        }
        parser.advance(); // consume ')'
    }

    // Parse account fields: { field1: Type, field2: Type }
    if !matches!(parser.current_token, Token::LeftBrace) {
        return Err(parser.parse_error("'{' to start account fields"));
    }
    parser.advance(); // consume '{'

    let mut fields = Vec::new();

    while !matches!(parser.current_token, Token::RightBrace)
        && !matches!(parser.current_token, Token::Eof)
    {
        // Handle optional mutability: mut field_name
        let is_mutable = if matches!(parser.current_token, Token::Mut) {
            parser.advance(); // consume 'mut'
            true
        } else {
            false
        };

        // Parse field name
        let field_name = match &parser.current_token {
            Token::Identifier(name) => name.clone(),
            Token::Account => "account".to_string(),
            _ => return Err(parser.parse_error("account field name identifier")),
        };
        parser.advance();

        // Check for optional marker: name?
        let is_optional = if matches!(parser.current_token, Token::Question) {
            parser.advance(); // consume '?'
            true
        } else {
            false
        };

        if !matches!(parser.current_token, Token::Colon) {
            return Err(parser.parse_error("':' after account field name"));
        }
        parser.advance(); // consume ':'

        let field_type = Box::new(types::parse_type(parser)?);

        fields.push(StructField {
            name: field_name,
            field_type: *field_type,
            is_mutable,
            is_optional,
        });

        // Allow either ',' or ';' as field separators (Rust-like flexibility)
        if matches!(parser.current_token, Token::Comma)
            || matches!(parser.current_token, Token::Semicolon)
        {
            parser.advance(); // consume separator
        } else {
            // No explicit separator; allow immediate '}' to end the fields
            // or break to validate closing brace below.
            break;
        }
    }

    if !matches!(parser.current_token, Token::RightBrace) {
        return Err(parser.parse_error("'}' to end account fields"));
    }
    parser.advance(); // consume '}'

    Ok(AstNode::AccountDefinition {
        name,
        fields,
        serializer,
        visibility,
    })
}

pub(crate) fn parse_type_definition(parser: &mut DslParser) -> Result<AstNode, VMError> {
    let visibility = if matches!(parser.current_token, Token::Pub) {
        parser.advance(); // consume 'pub'
        crate::Visibility::Public
    } else {
        crate::Visibility::Internal
    };

    if !matches!(parser.current_token, Token::TypeDecl) {
        return Err(parser.parse_error("'type' keyword"));
    }
    parser.advance(); // consume 'type'

    let name = match &parser.current_token {
        Token::Identifier(name) => name.clone(),
        _ => return Err(parser.parse_error("type name identifier")),
    };
    parser.advance();

    let definition = if matches!(parser.current_token, Token::Assign) {
        parser.advance(); // consume '='
        Box::new(types::parse_type(parser)?)
    } else if matches!(parser.current_token, Token::LeftBrace) {
        parser.advance(); // consume '{'
        let mut fields = Vec::new();

        while !matches!(parser.current_token, Token::RightBrace)
            && !matches!(parser.current_token, Token::Eof)
        {
            let is_mutable = if matches!(parser.current_token, Token::Mut) {
                parser.advance();
                true
            } else {
                false
            };

            let field_name = match &parser.current_token {
                Token::Identifier(name) => name.clone(),
                Token::Account => "account".to_string(),
                _ => return Err(parser.parse_error("type field name identifier")),
            };
            parser.advance();

            let is_optional = if matches!(parser.current_token, Token::Question) {
                parser.advance();
                true
            } else {
                false
            };

            if !matches!(parser.current_token, Token::Colon) {
                return Err(parser.parse_error("':' after type field name"));
            }
            parser.advance();

            let field_type = types::parse_type(parser)?;
            fields.push(StructField {
                name: field_name,
                field_type,
                is_mutable,
                is_optional,
            });

            if matches!(parser.current_token, Token::Comma)
                || matches!(parser.current_token, Token::Semicolon)
            {
                parser.advance();
            } else {
                break;
            }
        }

        if !matches!(parser.current_token, Token::RightBrace) {
            return Err(parser.parse_error("'}' to end type fields"));
        }
        parser.advance();
        Box::new(crate::ast::TypeNode::Struct { fields })
    } else {
        return Err(parser.parse_error("'=' or '{' after type name"));
    };

    if matches!(parser.current_token, Token::Semicolon) {
        parser.advance();
    }

    Ok(AstNode::TypeDefinition {
        name,
        definition,
        visibility,
    })
}
