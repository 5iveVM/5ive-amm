use crate::ast::{AstNode, ImportItem, ModuleSpecifier};
use crate::parser::DslParser;
use crate::tokenizer::{Token, TokenKind};
use five_vm_mito::error::VMError;

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
            ModuleSpecifier::External(addr)
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
        _ => return Err(parser.parse_error("module identifier or address string")),
    };

    // Parse optional member imports: ::function_name or ::{method foo, interface Bar, baz}
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
                    // Explicit interface import: interface Name
                    Token::Interface => {
                        parser.advance();
                        if let Token::Identifier(name) = &parser.current_token {
                            let out = ImportItem::Interface(name.clone());
                            parser.advance();
                            out
                        } else {
                            return Err(parser.parse_error("interface name after 'interface'"));
                        }
                    }
                    // Explicit method import: method foo
                    Token::Identifier(kind) if kind == "method" => {
                        parser.advance();
                        if let Token::Identifier(name) = &parser.current_token {
                            let out = ImportItem::Method(name.clone());
                            parser.advance();
                            out
                        } else {
                            return Err(parser.parse_error("method name after 'method'"));
                        }
                    }
                    // Backward compatible unqualified symbol import.
                    Token::Identifier(name) => {
                        let out = ImportItem::Unqualified(name.clone());
                        parser.advance();
                        out
                    }
                    _ => {
                        return Err(parser.parse_error(
                            "import member (identifier, 'method <name>', or 'interface <name>')",
                        ));
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
