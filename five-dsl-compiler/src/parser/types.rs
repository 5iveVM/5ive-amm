use crate::ast::{StructField, TypeNode};
use crate::parser::DslParser;
use crate::tokenizer::{Token};
use five_vm_mito::error::VMError;

fn parse_sized_suffix(parser: &mut DslParser, base_type: String) -> Result<TypeNode, VMError> {
    parser.advance(); // consume '<'

    let size = match &parser.current_token {
        Token::NumberLiteral(n) => *n,
        _ => return Err(parser.parse_error("size number literal in sized type")),
    };
    parser.advance();

    // Nested generic closers can be tokenized as >> or >>>.
    parser.split_generic_closer();
    if !matches!(parser.current_token, Token::GT) {
        return Err(parser.parse_error("'>' to end sized type"));
    }
    parser.advance(); // consume '>'

    Ok(TypeNode::Sized { base_type, size })
}

fn parse_generic_args(parser: &mut DslParser) -> Result<Vec<TypeNode>, VMError> {
    parser.advance(); // consume '<'
    let mut args = Vec::new();

    loop {
        parser.split_generic_closer();
        if matches!(parser.current_token, Token::GT) || matches!(parser.current_token, Token::Eof) {
            break;
        }

        if let Token::NumberLiteral(n) = parser.current_token {
            // Support const-like generic numeric args, e.g. Vec<u64, 64>.
            // Encoded via a sentinel sized type consumed by specific generic handlers.
            args.push(TypeNode::Sized {
                base_type: "__const".to_string(),
                size: n,
            });
            parser.advance();
        } else {
            args.push(parse_type(parser)?);
        }

        if matches!(parser.current_token, Token::Comma) {
            parser.advance(); // consume ','
        }
    }

    parser.split_generic_closer();
    if !matches!(parser.current_token, Token::GT) {
        return Err(parser.parse_error("'>' to end generic type"));
    }
    parser.advance(); // consume '>'

    Ok(args)
}

pub(crate) fn parse_type(parser: &mut DslParser) -> Result<TypeNode, VMError> {
    let token = parser.current_token.clone();
    match &token {
        // Handle arrays: [T; N] (Rust style)
        Token::LeftBracket => {
            parser.advance(); // consume '['
            let element_type = Box::new(parse_type(parser)?);

            if matches!(parser.current_token, Token::Semicolon) {
                parser.advance(); // consume ';'

                // Parse array size
                let size = match &parser.current_token {
                    Token::NumberLiteral(n) => *n,
                    _ => return Err(parser.parse_error("array size number literal")),
                };
                parser.advance();

                if !matches!(parser.current_token, Token::RightBracket) {
                    return Err(parser.parse_error("']' to end array type declaration"));
                }
                parser.advance(); // consume ']'

                Ok(TypeNode::Array {
                    element_type,
                    size: Some(size),
                })
            } else {
                Err(parser.parse_error("';' in array type declaration"))
            }
        }

        // Handle tuples: (T1, T2, ...)
        Token::LeftParen => {
            parser.advance(); // consume '('
            let mut elements = Vec::new();

            while !matches!(parser.current_token, Token::RightParen)
                && !matches!(parser.current_token, Token::Eof)
            {
                elements.push(parse_type(parser)?);

                if matches!(parser.current_token, Token::Comma) {
                    parser.advance(); // consume ','
                } else {
                    break;
                }
            }

            if !matches!(parser.current_token, Token::RightParen) {
                return Err(parser.parse_error("')' to end tuple type"));
            }
            parser.advance(); // consume ')'

            Ok(TypeNode::Tuple { elements })
        }

        // Handle struct types: { field1: T1, field2: T2 }
        Token::LeftBrace => {
            parser.advance(); // consume '{'
            let mut fields = Vec::new();

            while !matches!(parser.current_token, Token::RightBrace)
                && !matches!(parser.current_token, Token::Eof)
            {
                // Handle optional mutability: mut field
                let is_mutable = if matches!(parser.current_token, Token::Mut) {
                    parser.advance(); // consume 'mut'
                    true
                } else {
                    false
                };

                // Parse field name
                let name = match &parser.current_token {
                    Token::Identifier(name) => name.clone(),
                    _ => return Err(parser.parse_error("struct field name identifier in type")),
                };
                parser.advance();

                // Check for optional marker: field?
                let is_optional = if matches!(parser.current_token, Token::Question) {
                    parser.advance(); // consume '?'
                    true
                } else {
                    false
                };

                if !matches!(parser.current_token, Token::Colon) {
                    return Err(parser.parse_error("':' after struct field name in type"));
                }
                parser.advance(); // consume ':'

                let field_type = parse_type(parser)?;

                fields.push(StructField {
                    name,
                    field_type,
                    is_mutable,
                    is_optional,
                });

                if matches!(parser.current_token, Token::Comma) {
                    parser.advance(); // consume ','
                } else {
                    break;
                }
            }

            if !matches!(parser.current_token, Token::RightBrace) {
                return Err(parser.parse_error("'}' to end struct type"));
            }
            parser.advance(); // consume '}'

            Ok(TypeNode::Struct { fields })
        }

        // Handle primitive types and generics
        Token::Type(type_name) => {
            let base_type = type_name.clone();
            parser.advance();

            // Check for sized types: string<32>
            if matches!(parser.current_token, Token::LT) {
                parse_sized_suffix(parser, base_type)
            } else {
                // Check for TypeScript-style arrays: pubkey[], string[], etc.
                if matches!(parser.current_token, Token::LeftBracket) {
                    parser.advance(); // consume '['

                    let size = match &parser.current_token {
                        Token::NumberLiteral(n) => Some(*n),
                        _ => None, // Dynamic array: pubkey[]
                    };

                    if size.is_some() {
                        parser.advance(); // consume size
                    }

                    if !matches!(parser.current_token, Token::RightBracket) {
                        return Err(parser.parse_error("']' to end TypeScript-style array type"));
                    }
                    parser.advance(); // consume ']'

                    Ok(TypeNode::Array {
                        element_type: Box::new(TypeNode::Primitive(type_name.clone())),
                        size,
                    })
                } else if type_name == "Account" {
                    Ok(TypeNode::Account)
                } else {
                    Ok(TypeNode::Primitive(type_name.clone()))
                }
            }
        }

        // Handle generic types: Result, Option, etc.
        Token::Result | Token::Option => {
            let base = match &parser.current_token {
                Token::Result => "Result".to_string(),
                Token::Option => "Option".to_string(),
                _ => unreachable!(),
            };
            parser.advance();

            // Check for generic arguments: <T, E>
            if matches!(parser.current_token, Token::LT) {
                let args = parse_generic_args(parser)?;
                Ok(TypeNode::Generic { base, args })
            } else {
                Ok(TypeNode::Named(base))
            }
        }

        // Handle built-in primitive types that may be tokenized as identifiers.
        Token::Identifier(name)
            if matches!(
                name.as_str(),
                "pubkey"
                    | "u8"
                    | "u16"
                    | "u32"
                    | "u64"
                    | "u128"
                    | "i8"
                    | "i16"
                    | "i32"
                    | "i64"
                    | "bool"
                    | "string"
                    | "lamports"
                    | "String"
                    | "str"
            ) =>
        {
            let type_name = name.clone();
            parser.advance();

            // Support sized built-ins like string<32> when built-ins are tokenized as identifiers.
            if matches!(parser.current_token, Token::LT) {
                parse_sized_suffix(parser, type_name)
            } else if matches!(parser.current_token, Token::LeftBracket) {
                // Support TypeScript-style arrays for identifier-tokenized built-ins: string[], u64[4], etc.
                parser.advance(); // consume '['

                let size = match &parser.current_token {
                    Token::NumberLiteral(n) => Some(*n),
                    _ => None, // Dynamic array: type[]
                };

                if size.is_some() {
                    parser.advance(); // consume size
                }

                if !matches!(parser.current_token, Token::RightBracket) {
                    return Err(parser.parse_error("']' to end TypeScript-style array type"));
                }
                parser.advance(); // consume ']'

                Ok(TypeNode::Array {
                    element_type: Box::new(TypeNode::Primitive(type_name)),
                    size,
                })
            } else {
                Ok(TypeNode::Primitive(type_name))
            }
        }

        // Handle built-in account type with implicit properties
        Token::Account => {
            parser.advance();
            Ok(TypeNode::Account)
        }


        // Handle custom/named types
        Token::Identifier(name) => {
            let mut type_name = name.clone();
            parser.advance();

            // Handle namespaced types: Module::Type
            while matches!(parser.current_token, Token::DoubleColon) {
                parser.advance(); // consume '::'

                if let Token::Identifier(part) = &parser.current_token {
                    type_name.push_str("::");
                    type_name.push_str(part);
                    parser.advance();
                } else {
                    return Err(parser.parse_error("identifier after '::' in type name"));
                }
            }

            // Check for generic type arguments: Type<T>, Vec<pubkey>, Foo::Bar<A, B>
            if matches!(parser.current_token, Token::LT) {
                let args = parse_generic_args(parser)?;
                return Ok(TypeNode::Generic {
                    base: type_name,
                    args,
                });
            }

            // Check for TypeScript-style arrays: Type[N]
            if matches!(parser.current_token, Token::LeftBracket) {
                parser.advance(); // consume '['

                let size = match &parser.current_token {
                    Token::NumberLiteral(n) => Some(*n),
                    _ => None, // Dynamic array
                };

                if size.is_some() {
                    parser.advance(); // consume size
                }

                if !matches!(parser.current_token, Token::RightBracket) {
                    return Err(parser.parse_error("']' to end TypeScript-style array type"));
                }
                parser.advance(); // consume ']'

                Ok(TypeNode::Array {
                    element_type: Box::new(TypeNode::Named(type_name)),
                    size,
                })
            } else {
                Ok(TypeNode::Named(type_name))
            }
        }

        _ => Err(parser.parse_error("type specification")),
    }
}

/// Parse a return type, allowing tuple shorthand:
/// - `-> T`
/// - `-> T1, T2, ...` (lowered to `TypeNode::Tuple`)
pub(crate) fn parse_return_type(parser: &mut DslParser) -> Result<TypeNode, VMError> {
    let first = parse_type(parser)?;
    if !matches!(parser.current_token, Token::Comma) {
        return Ok(first);
    }

    let mut elements = vec![first];
    while matches!(parser.current_token, Token::Comma) {
        parser.advance(); // consume ','
        elements.push(parse_type(parser)?);
    }
    Ok(TypeNode::Tuple { elements })
}
