use crate::ast::{AstNode, ImportItem, ModuleSpecifier, NamespaceSpecifier};
use crate::parser::DslParser;
use crate::tokenizer::{Token, TokenKind};
use five_vm_mito::error::VMError;

const NAMESPACE_SYMBOLS: [char; 5] = ['!', '@', '#', '$', '%'];

fn is_valid_namespace_segment(segment: &str) -> bool {
    !segment.is_empty()
        && segment
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-')
}

fn parse_scoped_namespace_literal(input: &str) -> Option<NamespaceSpecifier> {
    let mut chars = input.chars();
    let symbol = chars.next()?;
    if !NAMESPACE_SYMBOLS.contains(&symbol) {
        return None;
    }
    let rest: String = chars.collect();
    let (domain, subprogram) = rest.split_once('/')?;
    if !is_valid_namespace_segment(domain) || !is_valid_namespace_segment(subprogram) {
        return None;
    }
    Some(NamespaceSpecifier::new(
        symbol,
        domain.to_ascii_lowercase(),
        subprogram.to_ascii_lowercase(),
    ))
}

fn parse_namespace_symbol(token: &Token) -> Option<char> {
    match token {
        Token::At => Some('@'),
        Token::Bang => Some('!'),
        Token::Hash => Some('#'),
        Token::Dollar => Some('$'),
        Token::Percent => Some('%'),
        _ => None,
    }
}

fn parse_namespace_segment(parser: &mut DslParser) -> Result<String, VMError> {
    let mut segment = String::new();
    let mut saw_part = false;

    loop {
        match &parser.current_token {
            Token::Identifier(part) => {
                segment.push_str(part);
                saw_part = true;
                parser.advance();
            }
            Token::NumberLiteral(part) => {
                segment.push_str(&part.to_string());
                saw_part = true;
                parser.advance();
            }
            Token::Minus if saw_part => {
                segment.push('-');
                parser.advance();
            }
            _ => break,
        }
    }

    if !saw_part || !is_valid_namespace_segment(&segment) {
        return Err(parser.parse_error("namespace segment (lowercase alnum + '-', one level path)"));
    }

    Ok(segment.to_ascii_lowercase())
}

fn parse_scoped_namespace(parser: &mut DslParser) -> Result<ModuleSpecifier, VMError> {
    let symbol = parse_namespace_symbol(&parser.current_token)
        .ok_or_else(|| parser.parse_error("namespace top-level symbol (!, @, #, $, %)"))?;
    parser.advance();

    let domain = parse_namespace_segment(parser)?;

    if !matches!(parser.current_token, Token::Divide | Token::Slash) {
        return Err(parser.parse_error("'/' between namespace domain and subprogram"));
    }
    parser.advance();

    let subprogram = parse_namespace_segment(parser)?;

    Ok(ModuleSpecifier::Namespace(NamespaceSpecifier::new(
        symbol, domain, subprogram,
    )))
}

/// Parse use statement: use account_address; import account_address; or use account_address::function_name;
pub(crate) fn parse_use_statement(parser: &mut DslParser) -> Result<AstNode, VMError> {
    // Consume 'use' or 'import' keyword
    if !matches!(parser.current_token, Token::Use | Token::Import) {
        return Err(parser.parse_error("'use' or 'import' keyword"));
    }
    parser.advance();

    // Parse module specifier
    let module_specifier = match &parser.current_token {
        Token::StringLiteral(addr) => {
            // External: use "0x123" or use "seeds";
            let addr = addr.clone();
            parser.advance();
            if let Some(ns) = parse_scoped_namespace_literal(&addr) {
                ModuleSpecifier::Namespace(ns)
            } else {
                ModuleSpecifier::External(addr)
            }
        }
        Token::At | Token::Bang | Token::Hash | Token::Dollar | Token::Percent => {
            parse_scoped_namespace(parser)?
        }
        Token::Identifier(name) => {
            // Local or Nested
            let mut path = vec![name.clone()];
            parser.advance();

            while matches!(parser.current_token, Token::DoubleColon) {
                // Peek to see if next token is Identifier (part of path) or '{' (import list)
                if parser.peek_kind(1) == TokenKind::Identifier {
                    parser.advance(); // consume ::
                    if let Token::Identifier(segment) = &parser.current_token {
                        path.push(segment.clone());
                        parser.advance();
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
        _ => {
            return Err(
                parser.parse_error("module identifier, quoted address, or scoped namespace target")
            )
        }
    };

    // Parse optional member imports: ::function_name or ::{foo, Bar, baz}
    let imported_items = if matches!(parser.current_token, Token::DoubleColon) {
        parser.advance(); // consume '::'

        if matches!(parser.current_token, Token::LeftBrace) {
            // Multiple imports: use account::{func1, func2}
            parser.advance(); // consume '{'
            let mut items = Vec::new();

            while !matches!(parser.current_token, Token::RightBrace)
                && !matches!(parser.current_token, Token::Eof)
            {
                let item = match &parser.current_token {
                    Token::Identifier(name) => {
                        let out = ImportItem::Unqualified(name.clone());
                        parser.advance();
                        out
                    }
                    _ => {
                        return Err(parser.parse_error("import member identifier"));
                    }
                };
                items.push(item);

                if matches!(parser.current_token, Token::Comma) {
                    parser.advance();
                } else {
                    break;
                }
            }

            if !matches!(parser.current_token, Token::RightBrace) {
                return Err(parser.parse_error("'}' to close import list"));
            }
            parser.advance(); // consume '}'

            Some(items)
        } else if let Token::Identifier(func_name) = &parser.current_token {
            // Single import: use account::function_name
            let name = func_name.clone();
            parser.advance();
            Some(vec![ImportItem::Unqualified(name)])
        } else {
            return Err(parser.parse_error("import member name or '{' after '::'"));
        }
    } else {
        None // Import all functions
    };

    // Optional semicolon
    if matches!(parser.current_token, Token::Semicolon) {
        parser.advance();
    }

    Ok(AstNode::ImportStatement {
        module_specifier,
        imported_items,
    })
}
