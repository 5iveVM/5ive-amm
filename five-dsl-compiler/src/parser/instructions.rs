use crate::ast::{
    AccountSerializer, AstNode, Attribute, BlockKind, InstructionParameter, PdaConfig,
    TestAttribute,
};
use crate::parser::{types, DslParser};
use crate::session_support;
use crate::tokenizer::{Token, TokenKind};
use five_vm_mito::error::VMError;

fn parse_account_serializer(parser: &mut DslParser) -> Result<AccountSerializer, VMError> {
    let serializer = match &parser.current_token {
        Token::StringLiteral(s) => {
            let out = s.clone();
            parser.advance();
            out
        }
        Token::Identifier(s) => {
            let out = s.clone();
            parser.advance();
            out
        }
        _ => return Err(parser.parse_error("serializer name (identifier or string literal)")),
    };

    match serializer.as_str() {
        "raw" => Ok(AccountSerializer::Raw),
        "borsh" => Ok(AccountSerializer::Borsh),
        "bincode" => Ok(AccountSerializer::Bincode),
        _ => Err(parser.parse_error("valid serializer: raw, borsh, or bincode")),
    }
}

fn is_reserved_function_name_token(token: &Token) -> bool {
    matches!(
        token,
        Token::Init
            | Token::Fn
            | Token::Let
            | Token::If
            | Token::Return
            | Token::For
            | Token::While
            | Token::Match
            | Token::Pub
            | Token::Mut
            | Token::Account
            | Token::Interface
            | Token::Enum
            | Token::TypeDecl
            | Token::True
            | Token::False
            | Token::Break
            | Token::Continue
            | Token::Use
            | Token::Import
            | Token::When
            | Token::Event
            | Token::Emit
            | Token::Require
            | Token::Error
            | Token::As
    )
}

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
        token if is_reserved_function_name_token(token) => {
            use heapless::String as HString;
            let mut expected = HString::<32>::new();
            let _ = expected.push_str("reserved keyword function name");
            let mut found = HString::<32>::new();
            let _ = found.push_str(&parser.token_to_string(token));
            return Err(VMError::ParseError {
                expected,
                found,
                position: parser.position,
            });
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
        let mut pda_config = None;

        // Allow optional leading attributes placed before the parameter name:
        // e.g., @signer @mut @requires(amount > 0) param: Account
        let mut leading_attributes: Vec<Attribute> = Vec::new();
        while matches!(
            parser.current_token,
            Token::AtSigner | Token::AtMut | Token::AtInit | Token::At
        ) {
            match &parser.current_token {
                Token::AtSigner => {
                    leading_attributes.push(Attribute {
                        name: "signer".to_string(),
                        args: vec![],
                    });
                    parser.advance();
                }
                Token::AtMut => {
                    leading_attributes.push(Attribute {
                        name: "mut".to_string(),
                        args: vec![],
                    });
                    parser.advance();
                }
                Token::AtInit => {
                    is_init = true;
                    leading_attributes.push(Attribute {
                        name: "init".to_string(),
                        args: vec![],
                    });
                    parser.advance();

                    // Parse optional (payer=name, space=256, seeds=[...], bump=name) syntax using shared helper
                    let (payer_name, explicit_space, explicit_seeds, explicit_bump) =
                        parse_init_arguments(parser)?;

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
                    if name == "pda" {
                        let (seeds, bump) = parse_pda_arguments(parser)?;
                        pda_config = Some(PdaConfig { seeds, bump });
                        continue;
                    } else if name == "session" {
                        let args = parse_session_arguments(parser)?;
                        leading_attributes.push(Attribute { name, args });
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

        // Keep explicit bump when provided. If bump is omitted for seeded @init,
        // leave it as None so codegen can derive canonical bump via FIND_PDA.

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
        let mut trailing_attributes: Vec<Attribute> = Vec::new();

        while matches!(
            parser.current_token,
            Token::AtSigner | Token::AtMut | Token::AtInit | Token::At
        ) {
            match &parser.current_token {
                Token::AtSigner => {
                    trailing_attributes.push(Attribute {
                        name: "signer".to_string(),
                        args: vec![],
                    });
                    parser.advance();
                }
                Token::AtMut => {
                    trailing_attributes.push(Attribute {
                        name: "mut".to_string(),
                        args: vec![],
                    });
                    parser.advance();
                }
                Token::AtInit => {
                    is_init = true;
                    trailing_attributes.push(Attribute {
                        name: "init".to_string(),
                        args: vec![],
                    });
                    parser.advance(); // consume @init token

                    // Parse optional (payer=name, space=256, seeds=[...], bump=name) syntax using shared helper
                    let (payer_name, explicit_space, explicit_seeds, explicit_bump) =
                        parse_init_arguments(parser)?;

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

                        init_config = Some(crate::ast::InitConfig {
                            seeds: if seeds.is_empty() { None } else { Some(seeds) },
                            bump: None,
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
                    if name == "pda" {
                        let (seeds, bump) = parse_pda_arguments(parser)?;
                        pda_config = Some(PdaConfig { seeds, bump });
                        continue;
                    } else if name == "session" {
                        let args = parse_session_arguments(parser)?;
                        trailing_attributes.push(Attribute { name, args });
                        continue;
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
            serializer: param_serializer,
            pda_config,
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
        Some(Box::new(types::parse_return_type(parser)?))
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
pub(crate) fn parse_init_arguments(
    parser: &mut DslParser,
) -> Result<
    (
        Option<String>,
        Option<u64>,
        Option<Vec<AstNode>>,
        Option<String>,
    ),
    VMError,
> {
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

pub(crate) fn parse_pda_arguments(
    parser: &mut DslParser,
) -> Result<(Vec<AstNode>, Option<String>), VMError> {
    let mut seeds: Option<Vec<AstNode>> = None;
    let mut bump: Option<String> = None;

    if !matches!(parser.current_token, Token::LeftParen) {
        return Err(parser.parse_error("'(' to start @pda arguments"));
    }
    parser.advance(); // consume '('

    while !matches!(parser.current_token, Token::RightParen | Token::Eof) {
        let key = match &parser.current_token {
            Token::Identifier(name) => name.clone(),
            _ => return Err(parser.parse_error("argument key in @pda()")),
        };
        parser.advance();

        if !matches!(parser.current_token, Token::Assign) {
            return Err(parser.parse_error("'=' after argument key in @pda"));
        }
        parser.advance();

        match key.as_str() {
            "seeds" => {
                if !matches!(parser.current_token, Token::LeftBracket) {
                    return Err(parser.parse_error("'[' for @pda seed list"));
                }
                parser.advance();

                let mut seed_list = Vec::new();
                while !matches!(parser.current_token, Token::RightBracket | Token::Eof) {
                    seed_list.push(parser.parse_expression()?);

                    if matches!(parser.current_token, Token::Comma) {
                        parser.advance();
                    } else if !matches!(parser.current_token, Token::RightBracket) {
                        return Err(parser.parse_error("',' or ']' in @pda seed list"));
                    }
                }

                if !matches!(parser.current_token, Token::RightBracket) {
                    return Err(parser.parse_error("']' to close @pda seed list"));
                }
                parser.advance();
                seeds = Some(seed_list);
            }
            "bump" => {
                bump = match &parser.current_token {
                    Token::Identifier(name) => {
                        let n = name.clone();
                        parser.advance();
                        Some(n)
                    }
                    _ => return Err(parser.parse_error("bump identifier in @pda")),
                };
            }
            _ => return Err(parser.parse_error("'seeds' or 'bump' in @pda()")),
        }

        if matches!(parser.current_token, Token::Comma) {
            parser.advance();
        } else {
            break;
        }
    }

    if !matches!(parser.current_token, Token::RightParen) {
        return Err(parser.parse_error("')' to close @pda arguments"));
    }
    parser.advance();

    let seeds = seeds.ok_or_else(|| parser.parse_error("'seeds=[...]' in @pda()"))?;
    if seeds.is_empty() {
        return Err(parser.parse_error("non-empty seed list in @pda()"));
    }

    Ok((seeds, bump))
}

pub(crate) fn parse_session_arguments(parser: &mut DslParser) -> Result<Vec<AstNode>, VMError> {
    if !matches!(parser.current_token, Token::LeftParen) {
        return Ok(Vec::new());
    }
    parser.advance(); // consume '('

    let mut args: Vec<AstNode> = Vec::new();
    let mut saw_keyed = false;
    let mut saw_positional = false;

    while !matches!(parser.current_token, Token::RightParen | Token::Eof) {
        // key=value form
        if matches!(parser.current_token, Token::Identifier(_)) && parser.peek_kind(1) == TokenKind::Assign {
            saw_keyed = true;
            let key = match &parser.current_token {
                Token::Identifier(name) => name.clone(),
                _ => return Err(parser.parse_error("session argument key")),
            };
            parser.advance(); // key
            parser.advance(); // '='
            let value = parser.parse_expression()?;
            args.push(AstNode::Assignment {
                target: key,
                value: Box::new(value),
            });
        } else {
            // positional form
            saw_positional = true;
            args.push(parser.parse_expression()?);
        }

        if matches!(parser.current_token, Token::Comma) {
            parser.advance();
        } else {
            break;
        }
    }

    if !matches!(parser.current_token, Token::RightParen) {
        return Err(parser.parse_error("')' to close session arguments"));
    }
    parser.advance(); // consume ')'
    if saw_positional && session_support::session_deprecation_warnings_enabled() {
        if saw_keyed {
            eprintln!(
                "warning: mixed positional and keyed @session arguments are deprecated; use keyed arguments only"
            );
        } else {
            eprintln!("warning: positional @session(...) arguments are deprecated; use keyed arguments");
        }
    }
    Ok(args)
}
