use crate::parser::DslParser;
use crate::tokenizer::Token;
use crate::ast::{AstNode, BlockKind, TypeNode};
use five_protocol::Value;

fn parse_stmt(tokens: Vec<Token>) -> AstNode {
    let mut parser = DslParser::new(tokens);
    parser.parse_statement().expect("Failed to parse statement")
}

#[test]
fn test_parse_let_statement() {
    // let x = 1;
    let tokens = vec![
        Token::Let,
        Token::Identifier("x".to_string()),
        Token::Assign,
        Token::NumberLiteral(1),
        Token::Semicolon,
        Token::Eof,
    ];
    let ast = parse_stmt(tokens);
    match ast {
        AstNode::LetStatement { name, type_annotation, is_mutable, value } => {
            assert_eq!(name, "x");
            assert!(type_annotation.is_none());
            assert!(!is_mutable);
            match *value {
                AstNode::Literal(Value::U64(v)) => assert_eq!(v, 1),
                _ => panic!("Expected Literal value"),
            }
        }
        _ => panic!("Expected LetStatement"),
    }
}

#[test]
fn test_parse_let_mut_statement() {
    // let mut x = 1;
    let tokens = vec![
        Token::Let,
        Token::Mut,
        Token::Identifier("x".to_string()),
        Token::Assign,
        Token::NumberLiteral(1),
        Token::Eof,
    ];
    let ast = parse_stmt(tokens);
    match ast {
        AstNode::LetStatement { name, is_mutable, .. } => {
            assert_eq!(name, "x");
            assert!(is_mutable);
        }
        _ => panic!("Expected LetStatement"),
    }
}

#[test]
fn test_parse_let_typed_statement() {
    // let x: u64 = 1;
    let tokens = vec![
        Token::Let,
        Token::Identifier("x".to_string()),
        Token::Colon,
        Token::Type("u64".to_string()),
        Token::Assign,
        Token::NumberLiteral(1),
        Token::Eof,
    ];
    let ast = parse_stmt(tokens);
    match ast {
        AstNode::LetStatement { name, type_annotation, .. } => {
            assert_eq!(name, "x");
            assert!(type_annotation.is_some());
            match *type_annotation.unwrap() {
                 TypeNode::Primitive(t) => assert_eq!(t, "u64"),
                 _ => panic!("Expected Primitive type"),
            }
        }
        _ => panic!("Expected LetStatement"),
    }
}

#[test]
fn test_parse_assignment() {
    // x = 2;
    let tokens = vec![
        Token::Identifier("x".to_string()),
        Token::Assign,
        Token::NumberLiteral(2),
        Token::Semicolon,
        Token::Eof,
    ];
    let ast = parse_stmt(tokens);
    match ast {
        AstNode::Assignment { target, value } => {
            assert_eq!(target, "x");
            match *value {
                AstNode::Literal(Value::U64(v)) => assert_eq!(v, 2),
                _ => panic!("Expected Literal value"),
            }
        }
        _ => panic!("Expected Assignment"),
    }
}

#[test]
fn test_parse_field_assignment() {
    // x.y = 3;
    let tokens = vec![
        Token::Identifier("x".to_string()),
        Token::Dot,
        Token::Identifier("y".to_string()),
        Token::Assign,
        Token::NumberLiteral(3),
        Token::Eof,
    ];
    let ast = parse_stmt(tokens);
    match ast {
        AstNode::FieldAssignment { object, field, value } => {
             match *object {
                AstNode::Identifier(name) => assert_eq!(name, "x"),
                _ => panic!("Expected Identifier object"),
             }
             assert_eq!(field, "y");
             match *value {
                AstNode::Literal(Value::U64(v)) => assert_eq!(v, 3),
                _ => panic!("Expected Literal value"),
             }
        }
        _ => panic!("Expected FieldAssignment"),
    }
}

#[test]
fn test_parse_return_statement() {
    // return 1;
    let tokens = vec![
        Token::Return,
        Token::NumberLiteral(1),
        Token::Semicolon,
        Token::Eof,
    ];
    let ast = parse_stmt(tokens);
    match ast {
        AstNode::ReturnStatement { value } => {
            assert!(value.is_some());
            match *value.unwrap() {
                AstNode::Literal(Value::U64(v)) => assert_eq!(v, 1),
                _ => panic!("Expected Literal value"),
            }
        }
        _ => panic!("Expected ReturnStatement"),
    }
}

#[test]
fn test_parse_return_empty_statement() {
    // return;
    let tokens = vec![
        Token::Return,
        Token::Semicolon,
        Token::Eof,
    ];
    let ast = parse_stmt(tokens);
    match ast {
        AstNode::ReturnStatement { value } => {
            assert!(value.is_none());
        }
        _ => panic!("Expected ReturnStatement"),
    }
}

#[test]
fn test_parse_if_statement() {
    // if true { }
    let tokens = vec![
        Token::If,
        Token::True,
        Token::LeftBrace,
        Token::RightBrace,
        Token::Eof,
    ];
    let ast = parse_stmt(tokens);
    match ast {
        AstNode::IfStatement { condition, then_branch, else_branch } => {
            match *condition {
                AstNode::Literal(Value::Bool(true)) => {},
                _ => panic!("Expected True condition"),
            }
            match *then_branch {
                AstNode::Block { statements, kind } => {
                    assert!(statements.is_empty());
                    assert_eq!(kind, BlockKind::Regular);
                },
                _ => panic!("Expected Block"),
            }
            assert!(else_branch.is_none());
        }
        _ => panic!("Expected IfStatement"),
    }
}

#[test]
fn test_parse_else_if_statement() {
    // if true { } else if false { } else { }
    let tokens = vec![
        Token::If,
        Token::True,
        Token::LeftBrace,
        Token::RightBrace,
        Token::Else,
        Token::If,
        Token::False,
        Token::LeftBrace,
        Token::RightBrace,
        Token::Else,
        Token::LeftBrace,
        Token::RightBrace,
        Token::Eof,
    ];
    let ast = parse_stmt(tokens);
    match ast {
        AstNode::IfStatement {
            condition,
            then_branch: _,
            else_branch,
        } => {
            match *condition {
                AstNode::Literal(Value::Bool(true)) => {}
                _ => panic!("Expected top-level true condition"),
            }

            let else_branch = else_branch.expect("Expected else branch");
            match *else_branch {
                AstNode::IfStatement {
                    condition: nested_condition,
                    then_branch: _,
                    else_branch: nested_else,
                } => {
                    match *nested_condition {
                        AstNode::Literal(Value::Bool(false)) => {}
                        _ => panic!("Expected nested false condition"),
                    }
                    assert!(nested_else.is_some(), "Expected final else block");
                }
                _ => panic!("Expected else-if to parse as nested IfStatement"),
            }
        }
        _ => panic!("Expected IfStatement"),
    }
}

#[test]
fn test_parse_while_loop() {
    // while (true) { }
    let tokens = vec![
        Token::While,
        Token::LeftParen,
        Token::True,
        Token::RightParen,
        Token::LeftBrace,
        Token::RightBrace,
        Token::Eof,
    ];
    let ast = parse_stmt(tokens);
    match ast {
        AstNode::WhileLoop { condition, body } => {
            match *condition {
                AstNode::Literal(Value::Bool(true)) => {},
                _ => panic!("Expected True condition"),
            }
            match *body {
                 AstNode::Block { .. } => {},
                 _ => panic!("Expected Block body"),
            }
        }
        _ => panic!("Expected WhileLoop"),
    }
}

#[test]
fn test_parse_require_statement() {
    // require(true);
    let tokens = vec![
        Token::Require,
        Token::LeftParen,
        Token::True,
        Token::RightParen,
        Token::Semicolon,
        Token::Eof,
    ];
    let ast = parse_stmt(tokens);
    match ast {
        AstNode::RequireStatement { condition } => {
             match *condition {
                AstNode::Literal(Value::Bool(true)) => {},
                _ => panic!("Expected True condition"),
            }
        }
        _ => panic!("Expected RequireStatement"),
    }
}
