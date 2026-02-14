use crate::ast::{
    AstNode, Attribute, BlockKind, InstructionParameter, TestAttribute,
};
use crate::parser::{DslParser, types};
use crate::tokenizer::{Token};
use five_vm_mito::error::VMError;

pub(crate) fn parse_instruction_definition(parser: &mut DslParser) -> Result<AstNode, VMError> {
    eprintln!("DEBUG_PARSER: parse_instruction_definition entry");
    // Check for 'pub' keyword to determine visibility
    let (is_public, visibility) = if matches!(parser.current_token, Token::Pub) {
        parser.advance(); // consume 'pub'
        (true, crate::Visibility::Public)
    } else {
        (false, crate::Visibility::Internal)
    };

    // Skip mut if present (instruction definitions don't use mut)
    if matches!(parser.current_token, Token::Mut) {
        parser.advance();
    }

    // Consume optional 'fn' or 'instruction' keywords that can follow visibility specifiers.
    if matches!(parser.current_token, Token::Fn | Token::Instruction) {
        parser.advance();
    }

    // Parse optional function/instruction name (allow shorthand 'test' prefix)
    // Some call sites invoke this parser after consuming 'fn'/'instruction'/'pub',
    // others call it directly when the current token is the function name.
    // If a 'test' keyword is present as a prefix, consume it and expect an identifier next.
    if matches!(parser.current_token, Token::Test) {
        // consume the 'test' keyword used as a shorthand prefix
        parser.advance();
    }
    // Now expect the function/instruction name identifier
    let name = match &parser.current_token {
        Token::Identifier(n) => {
            let n = n.clone();
            parser.advance();
            n
        }
        // Check for reserved keywords being used as function names
        Token::Init | Token::Fn | Token::Let | Token::If | Token::Return
        | Token::For | Token::While | Token::Match | Token::Pub | Token::Mut
        | Token::Account | Token::Interface | Token::Enum
        | Token::True | Token::False | Token::Break | Token::Continue
        | Token::Use | Token::Import | Token::When | Token::Event | Token::Emit
        | Token::Require | Token::Error | Token::As => {
            let keyword = format!("{:?}", parser.current_token).to_lowercase();
            return Err(parser.parse_error(&format!(
                "non-reserved identifier (found reserved keyword '{}')",
                keyword
            )));
        }
        _ => return Err(parser.parse_error("instruction/function name identifier")),
    };

    // Parse parameter list: (param1: Type, param2?: Type)
    if !matches!(parser.current_token, Token::LeftParen) {
        return Err(parser.parse_error("'(' to start parameter list"));
    }
    parser.advance(); // consume '('

    let mut parameters = Vec::new();

    while !matches!(parser.current_token, Token::RightParen)
        && !matches!(parser.current_token, Token::Eof)
    {
        let mut is_init = false;
        let mut init_config = None;

        // Allow optional leading attributes placed before the parameter name:
        // e.g., @signer @mut @requires(amount > 0) param: Account
        let mut leading_attributes: Vec<Attribute> = Vec::new();
        while matches!(
            parser.current_token,
            Token::AtSigner | Token::AtMut | Token::AtInit | Token::At
        ) {
            match &parser.current_token {
                Token::AtSigner => {
                    leading_attributes.push(Attribute { name: "signer".to_string(), args: vec![] });
                    parser.advance();
                }
                Token::AtMut => {
                    leading_attributes.push(Attribute { name: "mut".to_string(), args: vec![] });
                    parser.advance();
                }
                Token::AtInit => {
                    is_init = true;
                    leading_attributes.push(Attribute { name: "init".to_string(), args: vec![] });
                    parser.advance();

                    // Parse optional (payer=name, space=256, seeds=[...], bump=name) syntax using shared helper
                    let (payer_name, explicit_space, explicit_seeds, explicit_bump) = parse_init_arguments(parser)?;

                    // Check for optional [seeds] syntax (legacy or standalone)
                    if explicit_seeds.is_some() {
                            init_config = Some(crate::ast::InitConfig {
                            seeds: explicit_seeds,
                            bump: explicit_bump,
                            space: explicit_space,
                            payer: payer_name,
                        });
                    } else if matches!(parser.current_token, Token::LeftBracket) {
                        parser.advance(); // consume '['

                        let mut seeds = Vec::new();
                        while !matches!(parser.current_token, Token::RightBracket | Token::Eof) {
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
                            bump: None, // Will fill in after parsing name
                            space: explicit_space,
                            payer: payer_name,
                        });

                    } else {
                        // Simple @init without seeds (may have payer/space)
                        init_config = Some(crate::ast::InitConfig {
                            seeds: None,
                            bump: None,
                            space: explicit_space,
                            payer: payer_name,
                        });
                    }
                }
                Token::At => {
                    // Generic attribute: @name(args...)
                    parser.advance(); // consume '@'
                    let name = parser.expect_ident()?;
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

                    leading_attributes.push(Attribute { name, args });
                }
                _ => break,
            }
        }

        // Parse parameter name
        let param_name = match &parser.current_token {
            Token::Identifier(name) => name.clone(),
            // Allow keyword 'account' as an identifier name in parameter position
            Token::Account => "account".to_string(),
            _ => return Err(parser.parse_error("parameter name identifier")),
        };
        parser.advance();

        // Patch bump in init_config if present and seeds exist
        if let Some(config) = &mut init_config {
            if config.seeds.is_some() {
                config.bump = Some(format!("{}_bump", param_name));
            }
        }

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

        // Parse optional account attributes after type: @signer, @mut, @init
        let mut trailing_attributes: Vec<Attribute> = Vec::new();

        while matches!(
            parser.current_token,
            Token::AtSigner | Token::AtMut | Token::AtInit | Token::At
        ) {
            match &parser.current_token {
                Token::AtSigner => {
                    trailing_attributes.push(Attribute { name: "signer".to_string(), args: vec![] });
                    parser.advance();
                }
                Token::AtMut => {
                    trailing_attributes.push(Attribute { name: "mut".to_string(), args: vec![] });
                    parser.advance();
                }
                Token::AtInit => {
                    is_init = true;
                    trailing_attributes.push(Attribute { name: "init".to_string(), args: vec![] });
                    parser.advance(); // consume @init token

                    // Parse optional (payer=name, space=256, seeds=[...], bump=name) syntax using shared helper
                    let (payer_name, explicit_space, explicit_seeds, explicit_bump) = parse_init_arguments(parser)?;

                    // Check for optional [seeds] syntax
                    if explicit_seeds.is_some() {
                        init_config = Some(crate::ast::InitConfig {
                            seeds: explicit_seeds,
                            bump: explicit_bump,
                            space: explicit_space,
                            payer: payer_name,
                        });
                    } else if matches!(parser.current_token, Token::LeftBracket) {
                        parser.advance(); // consume '['

                        let mut seeds = Vec::new();
                        while !matches!(parser.current_token, Token::RightBracket | Token::Eof) {
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

                        // Auto-generate bump variable name
                        let bump_var = if !seeds.is_empty() {
                            Some(format!("{}_bump", param_name))
                        } else {
                            None
                        };

                        init_config = Some(crate::ast::InitConfig {
                            seeds: if seeds.is_empty() { None } else { Some(seeds) },
                            bump: bump_var,
                            space: explicit_space,
                            payer: payer_name,
                        });
                    } else {
                        // Simple @init without seeds (may have payer/space)
                        init_config = Some(crate::ast::InitConfig {
                            seeds: None,
                            bump: None,
                            space: explicit_space,
                            payer: payer_name,
                        });
                    }
                }
                Token::At => {
                    // Generic attribute: @name(args...)
                    parser.advance(); // consume '@'
                    let name = parser.expect_ident()?;
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

                    trailing_attributes.push(Attribute { name, args });
                }
                _ => unreachable!(),
            }
        }

        // Parse optional default value: = expression
        let default_value = if matches!(parser.current_token, Token::Assign) {
            parser.advance(); // consume '='
            Some(Box::new(parser.parse_expression()?))
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
        if matches!(parser.current_token, Token::Comma) {
            parser.advance(); // consume ','
        } else {
            break;
        }
    }

    if !matches!(parser.current_token, Token::RightParen) {
        return Err(VMError::UnexpectedEndOfInput);
    }
    parser.advance(); // consume ')'

    // Parse optional return type: -> ReturnType
    let return_type = if matches!(parser.current_token, Token::Arrow) {
        parser.advance(); // consume '->'
        Some(Box::new(types::parse_type(parser)?))
    } else {
        None
    };

    // Parse function body: { ... }
    let body = Box::new(parser.parse_block(BlockKind::Regular)?);

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
pub(crate) fn is_instruction_definition(parser: &mut DslParser) -> Result<AstNode, VMError> {
    // Consume '#'
    parser.advance();

    // Parse optional attributes: #[attribute]
    let mut attributes = Vec::new();
    while matches!(parser.current_token, Token::LeftBracket) {
        parser.advance(); // consume '['
        if !matches!(parser.current_token, Token::Hash) {
            return Err(parser.parse_error("'#' after '[' for attribute"));
        }
        parser.advance(); // consume '#'

        let attribute_name = match &parser.current_token {
            Token::Identifier(name) => name.clone(),
            _ => return Err(parser.parse_error("attribute name identifier")),
        };
        parser.advance();

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
                return Err(parser.parse_error("')' to end attribute arguments"));
            }
            parser.advance(); // consume ')'
        }

        attributes.push(TestAttribute {
            name: attribute_name,
            args,
        });

        if !matches!(parser.current_token, Token::RightBracket) {
            return Err(parser.parse_error("']' to end attribute"));
        }
        parser.advance(); // consume ']'
    }

    // Parse 'test' keyword
    if !matches!(parser.current_token, Token::Test) {
        return Err(parser.parse_error("'test' keyword for test function"));
    }
    parser.advance();

    // Parse test function name
    let name = match &parser.current_token {
        Token::Identifier(name) => name.clone(),
        _ => return Err(parser.parse_error("test function name identifier")),
    };
    parser.advance();

    // Parse parameter list (empty for test functions)
    if !matches!(parser.current_token, Token::LeftParen) {
        return Err(parser.parse_error("'( ' to start test function parameter list"));
    }
    parser.advance(); // consume '('

    if !matches!(parser.current_token, Token::RightParen) {
        return Err(parser.parse_error("')' to end test function parameter list"));
    }
    parser.advance(); // consume ')'

    // Parse function body: { ... }
    let body = Box::new(parser.parse_block(BlockKind::Regular)?);

    Ok(AstNode::TestFunction {
        name,
        attributes,
        body,
    })
}

pub(crate) fn parse_test_function(parser: &mut DslParser) -> Result<AstNode, VMError> {
    parser.advance(); // consume '#'

    // Parse optional attributes: #[attribute]
    let mut attributes = Vec::new();
    while matches!(parser.current_token, Token::LeftBracket) {
        parser.advance(); // consume '['
        if !matches!(parser.current_token, Token::Hash) {
            return Err(parser.parse_error("'#' after '[' for attribute"));
        }
        parser.advance(); // consume '#'

        let attribute_name = match &parser.current_token {
            Token::Identifier(name) => name.clone(),
            _ => return Err(parser.parse_error("attribute name identifier")),
        };
        parser.advance();

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
                return Err(parser.parse_error("')' to end attribute arguments"));
            }
            parser.advance(); // consume ')'
        }

        attributes.push(TestAttribute {
            name: attribute_name,
            args,
        });

        if !matches!(parser.current_token, Token::RightBracket) {
            return Err(parser.parse_error("']' to end attribute"));
        }
        parser.advance(); // consume ']'
    }

    // Parse 'test' keyword
    if !matches!(parser.current_token, Token::Test) {
        return Err(parser.parse_error("'test' keyword for test function"));
    }
    parser.advance();

    // Parse test function name
    let name = match &parser.current_token {
        Token::Identifier(name) => name.clone(),
        _ => return Err(parser.parse_error("test function name identifier")),
    };
    parser.advance();

    // Parse parameter list (empty for test functions)
    if !matches!(parser.current_token, Token::LeftParen) {
        return Err(parser.parse_error("'( ' to start test function parameter list"));
    }
    parser.advance(); // consume '('

    if !matches!(parser.current_token, Token::RightParen) {
        return Err(parser.parse_error("')' to end test function parameter list"));
    }
    parser.advance(); // consume ')'

    // Parse function body: { ... }
    let body = Box::new(parser.parse_block(BlockKind::Regular)?);

    Ok(AstNode::TestFunction {
        name,
        attributes,
        body,
    })
}

/// Parse @init arguments: @init(payer=authority, space=256)
/// Returns (payer: Option<String>, space: Option<u64>)
pub(crate) fn parse_init_arguments(parser: &mut DslParser) -> Result<(Option<String>, Option<u64>, Option<Vec<AstNode>>, Option<String>), VMError> {
    let mut payer: Option<String> = None;
    let mut space: Option<u64> = None;
    let mut seeds: Option<Vec<AstNode>> = None;
    let mut bump: Option<String> = None;

    if !matches!(parser.current_token, Token::LeftParen) {
        return Ok((None, None, None, None));
    }
    parser.advance(); // consume '('

    // Parse comma-separated key=value pairs
    while !matches!(parser.current_token, Token::RightParen | Token::Eof) {
        let key = match &parser.current_token {
            Token::Identifier(name) => name.clone(),
            _ => return Err(parser.parse_error("argument key in @init()")),
        };
        parser.advance(); // consume key

        if !matches!(parser.current_token, Token::Assign) {
            return Err(parser.parse_error("'=' after argument key in @init"));
        }
        parser.advance(); // consume '='

        match key.as_str() {
            "payer" => {
                payer = match &parser.current_token {
                    Token::Identifier(name) => {
                        let n = name.clone();
                        parser.advance();
                        Some(n)
                    }
                    Token::Account => {
                        parser.advance();
                        Some("account".to_string())
                    }
                    _ => return Err(parser.parse_error("payer account name")),
                };
            }
            "space" => {
                space = match &parser.current_token {
                    Token::NumberLiteral(n) => {
                        let s = *n as u64;
                        parser.advance();
                        Some(s)
                    }
                    _ => return Err(parser.parse_error("space size as number")),
                };
            }
            "seeds" => {
                if !matches!(parser.current_token, Token::LeftBracket) {
                    return Err(parser.parse_error("'[' for seed list"));
                }
                parser.advance(); // consume '['

                let mut seed_list = Vec::new();
                while !matches!(parser.current_token, Token::RightBracket | Token::Eof) {
                    seed_list.push(parser.parse_expression()?);

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
                seeds = Some(seed_list);
            }
            "bump" => {
                bump = match &parser.current_token {
                    Token::Identifier(name) => {
                        let n = name.clone();
                        parser.advance();
                        Some(n)
                    }
                    _ => return Err(parser.parse_error("bump variable name")),
                };
            }
            _ => return Err(parser.parse_error("'payer', 'space', 'seeds' or 'bump' in @init()")),
        }

        // Handle comma separator
        if matches!(parser.current_token, Token::Comma) {
            parser.advance();
        } else {
            break;
        }
    }

    if !matches!(parser.current_token, Token::RightParen) {
        return Err(parser.parse_error("')' to close @init arguments"));
    }
    parser.advance(); // consume ')'

    Ok((payer, space, seeds, bump))
}
