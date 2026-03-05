use crate::ast::{AccountSerializer, AstNode, Attribute, InstructionParameter};
use crate::parser::instructions::parse_pda_arguments;
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
    let value = parse_serializer_name(parser)?;
    match value.as_str() {
        "raw" => Ok(AccountSerializer::Raw),
        "borsh" => Ok(AccountSerializer::Borsh),
        "bincode" => Ok(AccountSerializer::Bincode),
        "anchor" => Ok(AccountSerializer::Anchor),
        _ => Err(parser.parse_error("valid serializer: raw, borsh, bincode, or anchor")),
    }
}

fn parse_discriminator_args(
    parser: &mut DslParser,
) -> Result<(Option<u8>, Option<Vec<u8>>), VMError> {
    if !matches!(parser.current_token, Token::LeftParen) {
        return Err(parser.parse_error("'(' after discriminator keyword"));
    }
    parser.advance(); // consume '('

    let parsed = if matches!(parser.current_token, Token::LeftBracket) {
        parser.advance(); // consume '['
        let mut bytes = Vec::new();
        while !matches!(parser.current_token, Token::RightBracket)
            && !matches!(parser.current_token, Token::Eof)
        {
            let b = match &parser.current_token {
                Token::NumberLiteral(n) if *n <= u8::MAX as u64 => *n as u8,
                _ => return Err(parser.parse_error("number literal (0-255) for discriminator")),
            };
            bytes.push(b);
            parser.advance();
            if matches!(parser.current_token, Token::Comma) {
                parser.advance();
            } else {
                break;
            }
        }
        if !matches!(parser.current_token, Token::RightBracket) {
            return Err(parser.parse_error("']' after discriminator bytes"));
        }
        parser.advance(); // consume ']'
        (None, Some(bytes))
    } else {
        let disc = match &parser.current_token {
            Token::NumberLiteral(n) if *n <= u8::MAX as u64 => Some(*n as u8),
            _ => return Err(parser.parse_error("number literal (0-255) for discriminator")),
        };
        parser.advance();
        (disc, None)
    };

    if !matches!(parser.current_token, Token::RightParen) {
        return Err(parser.parse_error("')' after discriminator value"));
    }
    parser.advance(); // consume ')'
    Ok(parsed)
}

fn parse_discriminator_bytes_args(parser: &mut DslParser) -> Result<Vec<u8>, VMError> {
    if !matches!(parser.current_token, Token::LeftParen) {
        return Err(parser.parse_error("'(' after discriminator_bytes keyword"));
    }
    parser.advance(); // consume '('

    let mut bytes = Vec::new();
    if matches!(parser.current_token, Token::LeftBracket) {
        parser.advance(); // consume '['
        while !matches!(parser.current_token, Token::RightBracket)
            && !matches!(parser.current_token, Token::Eof)
        {
            let b = match &parser.current_token {
                Token::NumberLiteral(n) if *n <= u8::MAX as u64 => *n as u8,
                _ => {
                    return Err(parser.parse_error("number literal (0-255) for discriminator_bytes"))
                }
            };
            bytes.push(b);
            parser.advance();
            if matches!(parser.current_token, Token::Comma) {
                parser.advance();
            } else {
                break;
            }
        }
        if !matches!(parser.current_token, Token::RightBracket) {
            return Err(parser.parse_error("']' after discriminator_bytes values"));
        }
        parser.advance(); // consume ']'
    } else {
        while !matches!(parser.current_token, Token::RightParen)
            && !matches!(parser.current_token, Token::Eof)
        {
            let b = match &parser.current_token {
                Token::NumberLiteral(n) if *n <= u8::MAX as u64 => *n as u8,
                _ => {
                    return Err(parser.parse_error("number literal (0-255) for discriminator_bytes"))
                }
            };
            bytes.push(b);
            parser.advance();
            if matches!(parser.current_token, Token::Comma) {
                parser.advance();
            } else {
                break;
            }
        }
    }

    if !matches!(parser.current_token, Token::RightParen) {
        return Err(parser.parse_error("')' after discriminator_bytes values"));
    }
    parser.advance(); // consume ')'
    Ok(bytes)
}

pub(crate) fn parse_interface_definition(parser: &mut DslParser) -> Result<AstNode, VMError> {
    let mut is_anchor_interface = false;

    // Support leading attribute form: @anchor interface ...
    if matches!(parser.current_token, Token::At) {
        let saved_pos = parser.position;
        parser.advance(); // consume '@'
        if matches!(&parser.current_token, Token::Identifier(name) if name == "anchor") {
            parser.advance(); // consume 'anchor'
            is_anchor_interface = true;
        } else {
            parser.position = saved_pos;
            parser.current_token = parser
                .tokens
                .get(parser.position)
                .cloned()
                .unwrap_or(Token::Eof);
        }
    }

    if !matches!(parser.current_token, Token::Interface) {
        return Err(parser.parse_error("'interface' keyword"));
    }
    parser.advance(); // consume 'interface'

    // Parse interface name
    let name = match &parser.current_token {
        Token::Identifier(name) => name.clone(),
        _ => return Err(parser.parse_error("interface name identifier")),
    };
    parser.advance();

    // Parse optional program ID: program("address") or @program("address")
    let mut program_id: Option<String> = None;
    let mut serializer: Option<String> = None;

    // Helper to check for @anchor
    let check_anchor = |parser: &mut DslParser| -> bool {
        if matches!(parser.current_token, Token::At) {
            let saved_pos = parser.position;
            parser.advance(); // consume '@'
            if matches!(&parser.current_token, Token::Identifier(name) if name == "anchor") {
                parser.advance(); // consume 'anchor'
                return true;
            }
            // Rewind if not anchor
            parser.position = saved_pos;
            parser.current_token = parser
                .tokens
                .get(parser.position)
                .cloned()
                .unwrap_or(Token::Eof);
        }
        false
    };

    // 1. Check for @anchor before other attributes
    if check_anchor(parser) {
        is_anchor_interface = true;
    }

    if matches!(&parser.current_token, Token::Identifier(name) if name == "program") {
        parser.advance(); // consume 'program'
        if !matches!(parser.current_token, Token::LeftParen) {
            return Err(parser.parse_error("'(' after program keyword"));
        }
        parser.advance(); // consume '('
        let id = match &parser.current_token {
            Token::StringLiteral(s) => s.clone(),
            _ => return Err(parser.parse_error("string literal for program ID")),
        };
        parser.advance();
        if !matches!(parser.current_token, Token::RightParen) {
            return Err(parser.parse_error("')' after program ID"));
        }
        parser.advance(); // consume ')'
        program_id = Some(id);
    } else if matches!(parser.current_token, Token::At) {
        // Attribute form: @program("...")
        let saved_pos = parser.position;
        parser.advance(); // consume '@'
        let is_program_attr =
            matches!(&parser.current_token, Token::Identifier(name) if name == "program");
        if is_program_attr {
            parser.advance(); // consume 'program' identifier
            if !matches!(parser.current_token, Token::LeftParen) {
                return Err(parser.parse_error("'(' after program attribute"));
            }
            parser.advance(); // '('
            let id = match &parser.current_token {
                Token::StringLiteral(s) => s.clone(),
                _ => return Err(parser.parse_error("string literal for program ID")),
            };
            parser.advance();
            if !matches!(parser.current_token, Token::RightParen) {
                return Err(parser.parse_error("')' after program ID"));
            }
            parser.advance(); // ')'
            program_id = Some(id);
        } else {
            // Not a @program attribute; rewind to saved position so later parsing can continue cleanly
            parser.position = saved_pos;
            parser.current_token = parser
                .tokens
                .get(parser.position)
                .cloned()
                .unwrap_or(Token::Eof);
        }
    }

    // 2. Check for @anchor after program (flexible ordering)
    if !is_anchor_interface && check_anchor(parser) {
        is_anchor_interface = true;
    }

    // Optional serializer hint: serializer("borsh") or @serializer("borsh")
    if serializer.is_none() {
        if matches!(&parser.current_token, Token::Identifier(name) if name == "serializer")
            || matches!(parser.current_token, Token::Serializer)
        {
            parser.advance(); // consume 'serializer'
            if !matches!(parser.current_token, Token::LeftParen) {
                return Err(parser.parse_error("'(' after serializer keyword"));
            }
            parser.advance(); // '('
            let ser = parse_serializer_name(parser)?;
            if !matches!(parser.current_token, Token::RightParen) {
                return Err(parser.parse_error("')' after serializer name"));
            }
            parser.advance(); // ')'
            serializer = Some(ser);
        } else if matches!(parser.current_token, Token::At) {
            let saved_pos = parser.position;
            parser.advance(); // consume '@'
            let is_serializer_attr = matches!(&parser.current_token, Token::Identifier(name) if name == "serializer")
                || matches!(parser.current_token, Token::Serializer);
            if is_serializer_attr {
                parser.advance(); // consume 'serializer'
                if !matches!(parser.current_token, Token::LeftParen) {
                    return Err(parser.parse_error("'(' after serializer attribute"));
                }
                parser.advance(); // '('
                let ser = parse_serializer_name(parser)?;
                if !matches!(parser.current_token, Token::RightParen) {
                    return Err(parser.parse_error("')' after serializer name"));
                }
                parser.advance(); // ')'
                serializer = Some(ser);
            } else {
                // Rewind if not serializer attribute
                parser.position = saved_pos;
                parser.current_token = parser
                    .tokens
                    .get(parser.position)
                    .cloned()
                    .unwrap_or(Token::Eof);
            }
        }
    }

    // 3. Check for @anchor after serializer (flexible ordering)
    if !is_anchor_interface && check_anchor(parser) {
        is_anchor_interface = true;
    }

    // Parse interface methods: { method1(), method2() }
    if !matches!(parser.current_token, Token::LeftBrace) {
        return Err(parser.parse_error("'{' to start interface methods"));
    }
    parser.advance(); // consume '{'

    let mut functions = Vec::new();

    while !matches!(parser.current_token, Token::RightBrace)
        && !matches!(parser.current_token, Token::Eof)
    {
        // Optional attributes before method signature
        let mut discriminator: Option<u8> = None;
        let mut discriminator_bytes: Option<Vec<u8>> = None;
        let mut is_method_anchor = false;

        while matches!(parser.current_token, Token::At) {
            parser.advance(); // consume '@'

            let is_disc = matches!(&parser.current_token, Token::Identifier(name) if name == "discriminator")
                || matches!(parser.current_token, Token::Discriminator);
            let is_disc_bytes = matches!(&parser.current_token, Token::Identifier(name) if name == "discriminator_bytes")
                || matches!(parser.current_token, Token::DiscriminatorBytes);
            let is_anchor =
                matches!(&parser.current_token, Token::Identifier(name) if name == "anchor");

            if is_disc {
                parser.advance(); // consume 'discriminator'
                let (disc, disc_bytes) = parse_discriminator_args(parser)?;
                discriminator = disc;
                discriminator_bytes = disc_bytes;
            } else if is_disc_bytes {
                parser.advance(); // consume 'discriminator_bytes'
                discriminator_bytes = Some(parse_discriminator_bytes_args(parser)?);
            } else if is_anchor {
                parser.advance(); // consume 'anchor'
                is_method_anchor = true;
            } else {
                return Err(parser.parse_error(
                    "supported method attribute (@anchor, @discriminator, @discriminator_bytes)",
                ));
            }
        }

        // Parse method name (optional `fn` keyword allowed for readability)
        if matches!(parser.current_token, Token::Fn) {
            parser.advance(); // consume 'fn'
        }

        let method_name = match &parser.current_token {
            Token::Identifier(name) => name.clone(),
            _ => return Err(parser.parse_error("method name identifier")),
        };
        parser.advance();

        // Optional attributes between method name and parameter list
        while matches!(parser.current_token, Token::At) {
            parser.advance(); // consume '@'

            let is_disc = matches!(&parser.current_token, Token::Identifier(name) if name == "discriminator")
                || matches!(parser.current_token, Token::Discriminator);
            let is_disc_bytes = matches!(&parser.current_token, Token::Identifier(name) if name == "discriminator_bytes")
                || matches!(parser.current_token, Token::DiscriminatorBytes);
            let is_anchor =
                matches!(&parser.current_token, Token::Identifier(name) if name == "anchor");

            if is_disc {
                parser.advance(); // consume 'discriminator'
                let (disc, disc_bytes) = parse_discriminator_args(parser)?;
                discriminator = disc;
                discriminator_bytes = disc_bytes;
            } else if is_disc_bytes {
                parser.advance(); // consume 'discriminator_bytes'
                discriminator_bytes = Some(parse_discriminator_bytes_args(parser)?);
            } else if is_anchor {
                parser.advance(); // consume 'anchor'
                is_method_anchor = true;
            } else {
                return Err(parser.parse_error(
                    "supported method attribute (@anchor, @discriminator, @discriminator_bytes)",
                ));
            }
        }

        // Parse parameter list: (param1: Type, param2?: Type)
        if !matches!(parser.current_token, Token::LeftParen) {
            return Err(parser.parse_error("'(' to start method parameter list"));
        }
        parser.advance(); // consume '('

        let mut parameters = Vec::new();

        while !matches!(parser.current_token, Token::RightParen)
            && !matches!(parser.current_token, Token::Eof)
        {
            // Parse parameter name
            let param_name = match &parser.current_token {
                Token::Identifier(name) => name.clone(),
                _ => return Err(parser.parse_error("parameter name identifier")),
            };
            parser.advance();

            // Check for optional marker: param?
            let is_optional = if matches!(parser.current_token, Token::Question) {
                parser.advance(); // consume '?'
                true
            } else {
                false
            };

            // Parse parameter type: : Type
            if !matches!(parser.current_token, Token::Colon) {
                return Err(parser.parse_error("':' after parameter name for type annotation"));
            }
            parser.advance(); // consume ':'

            let param_type = types::parse_type(parser)?;
            let mut param_serializer: Option<AccountSerializer> = None;

            // Parse optional account attributes after type: @signer, @mut, @init
            let mut attributes: Vec<Attribute> = Vec::new();
            let mut is_init = false;
            let mut init_config = None;

            while matches!(
                parser.current_token,
                Token::AtSigner | Token::AtMut | Token::AtInit | Token::At
            ) {
                match &parser.current_token {
                    Token::AtSigner => {
                        attributes.push(Attribute {
                            name: "signer".to_string(),
                            args: vec![],
                        });
                        parser.advance();
                    }
                    Token::AtMut => {
                        attributes.push(Attribute {
                            name: "mut".to_string(),
                            args: vec![],
                        });
                        parser.advance();
                    }
                    Token::AtInit => {
                        is_init = true;
                        attributes.push(Attribute {
                            name: "init".to_string(),
                            args: vec![],
                        });
                        parser.advance();

                        // For interfaces, we might not need full init config parsing with seeds,
                        // but let's handle the basic token consumption to be safe.
                        // If [seeds] are present, parse them to avoid syntax errors.
                        if matches!(parser.current_token, Token::LeftBracket) {
                            parser.advance(); // consume '['
                            let mut seeds = Vec::new();
                            while !matches!(parser.current_token, Token::RightBracket | Token::Eof)
                            {
                                seeds.push(parser.parse_expression()?);
                                if matches!(parser.current_token, Token::Comma) {
                                    parser.advance();
                                } else if !matches!(parser.current_token, Token::RightBracket) {
                                    return Err(parser.parse_error("',' or ']' in seed list"));
                                }
                            }
                            if !matches!(parser.current_token, Token::RightBracket) {
                                return Err(parser.parse_error("']' to close seed list"));
                            }
                            parser.advance(); // consume ']'

                            init_config = Some(crate::ast::InitConfig {
                                seeds: if seeds.is_empty() { None } else { Some(seeds) },
                                bump: None,
                                space: None,
                                payer: None,
                            });
                        } else {
                            init_config = Some(crate::ast::InitConfig {
                                seeds: None,
                                bump: None,
                                space: None,
                                payer: None,
                            });
                        }
                    }
                    Token::At => {
                        parser.advance(); // consume '@'
                        let name = parser.expect_ident()?;
                        if name == "pda" {
                            let _ = parse_pda_arguments(parser)?;
                            return Err(
                                parser.parse_error("@pda is not allowed on interface parameters")
                            );
                        } else if name == "serializer" {
                            if !matches!(parser.current_token, Token::LeftParen) {
                                return Err(parser.parse_error("'(' after serializer attribute"));
                            }
                            parser.advance(); // consume '('
                            param_serializer = Some(parse_account_serializer(parser)?);
                            if !matches!(parser.current_token, Token::RightParen) {
                                return Err(parser.parse_error("')' to close serializer arguments"));
                            }
                            parser.advance(); // consume ')'
                            continue;
                        }
                        let mut args = Vec::new();
                        if matches!(parser.current_token, Token::LeftParen) {
                            parser.advance(); // consume '('
                            while !matches!(parser.current_token, Token::RightParen)
                                && !matches!(parser.current_token, Token::Eof)
                            {
                                args.push(parser.parse_expression()?);
                                if matches!(parser.current_token, Token::Comma) {
                                    parser.advance();
                                } else {
                                    break;
                                }
                            }
                            if !matches!(parser.current_token, Token::RightParen) {
                                return Err(parser.parse_error("')' to close attribute arguments"));
                            }
                            parser.advance(); // consume ')'
                        }
                        attributes.push(Attribute { name, args });
                    }
                    _ => unreachable!(),
                }
            }

            // Interface methods don't have attributes or default values
            parameters.push(InstructionParameter {
                name: param_name,
                param_type,
                is_optional,
                default_value: None,
                attributes,
                is_init,
                init_config,
                serializer: param_serializer,
                pda_config: None,
            });

            // Handle comma separator
            if matches!(parser.current_token, Token::Comma) {
                parser.advance(); // consume ','
            } else {
                break;
            }
        }

        if !matches!(parser.current_token, Token::RightParen) {
            return Err(parser.parse_error("')' to end method parameter list"));
        }
        parser.advance(); // consume ')'

        // Parse optional return type: -> ReturnType
        let return_type = if matches!(parser.current_token, Token::Arrow) {
            parser.advance(); // consume '->'
            Some(Box::new(types::parse_return_type(parser)?))
        } else {
            None
        };

        // Parse optional discriminator after params: discriminator(N) or discriminator_bytes(...)
        let (discriminator, discriminator_bytes) =
            if discriminator.is_some() || discriminator_bytes.is_some() {
                (discriminator, discriminator_bytes)
            } else if matches!(parser.current_token, Token::Discriminator) {
                parser.advance(); // consume 'discriminator'
                parse_discriminator_args(parser)?
            } else if matches!(parser.current_token, Token::DiscriminatorBytes) {
                parser.advance(); // consume 'discriminator_bytes'
                (None, Some(parse_discriminator_bytes_args(parser)?))
            } else {
                (None, None)
            };

        // Optional semicolon
        if matches!(parser.current_token, Token::Semicolon) {
            parser.advance();
        }

        functions.push(AstNode::InterfaceFunction {
            name: method_name,
            parameters,
            return_type,
            discriminator,
            discriminator_bytes,
            is_anchor: is_method_anchor,
        });
    }

    if !matches!(parser.current_token, Token::RightBrace) {
        return Err(parser.parse_error("'}' to end interface methods"));
    }
    parser.advance(); // consume '}'

    Ok(AstNode::InterfaceDefinition {
        name,
        program_id,
        serializer,
        is_anchor: is_anchor_interface,
        functions,
    })
}
