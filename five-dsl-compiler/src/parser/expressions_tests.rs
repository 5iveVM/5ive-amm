use crate::parser::DslParser;
use crate::tokenizer::Token;
use crate::ast::AstNode;
use five_protocol::Value;

fn parse_expr(tokens: Vec<Token>) -> AstNode {
    let mut parser = DslParser::new(tokens);
    parser.parse_expression().expect("Failed to parse expression")
}

#[test]
fn test_parse_literal_u64() {
    let tokens = vec![Token::NumberLiteral(123), Token::Eof];
    let ast = parse_expr(tokens);
    match ast {
        AstNode::Literal(Value::U64(v)) => assert_eq!(v, 123),
        _ => panic!("Expected Literal(U64(123)), got {:?}", ast),
    }
}

#[test]
fn test_parse_literal_bool() {
    let tokens = vec![Token::True, Token::Eof];
    let ast = parse_expr(tokens);
    match ast {
        AstNode::Literal(Value::Bool(v)) => assert!(v),
        _ => panic!("Expected Literal(Bool(true)), got {:?}", ast),
    }

    let tokens = vec![Token::False, Token::Eof];
    let ast = parse_expr(tokens);
    match ast {
        AstNode::Literal(Value::Bool(v)) => assert!(!v),
        _ => panic!("Expected Literal(Bool(false)), got {:?}", ast),
    }
}

#[test]
fn test_parse_literal_string() {
    let tokens = vec![Token::StringLiteral("hello".to_string()), Token::Eof];
    let ast = parse_expr(tokens);
    match ast {
        AstNode::StringLiteral { value } => assert_eq!(value, "hello"),
        _ => panic!("Expected StringLiteral, got {:?}", ast),
    }
}

#[test]
fn test_parse_identifier() {
    let tokens = vec![Token::Identifier("x".to_string()), Token::Eof];
    let ast = parse_expr(tokens);
    match ast {
        AstNode::Identifier(name) => assert_eq!(name, "x"),
        _ => panic!("Expected Identifier, got {:?}", ast),
    }
}

#[test]
fn test_parse_binary_expression_add() {
    // x + 1
    let tokens = vec![
        Token::Identifier("x".to_string()),
        Token::Plus,
        Token::NumberLiteral(1),
        Token::Eof,
    ];
    let ast = parse_expr(tokens);
    match ast {
        AstNode::BinaryExpression { left, operator, right } => {
            assert_eq!(operator, "+");
            match *left {
                AstNode::Identifier(name) => assert_eq!(name, "x"),
                _ => panic!("Expected left identifier"),
            }
            match *right {
                AstNode::Literal(Value::U64(v)) => assert_eq!(v, 1),
                _ => panic!("Expected right literal"),
            }
        }
        _ => panic!("Expected BinaryExpression, got {:?}", ast),
    }
}

#[test]
fn test_parse_binary_expression_precedence() {
    // 1 + 2 * 3
    // Should be parsed as 1 + (2 * 3)
    let tokens = vec![
        Token::NumberLiteral(1),
        Token::Plus,
        Token::NumberLiteral(2),
        Token::Multiply,
        Token::NumberLiteral(3),
        Token::Eof,
    ];
    let ast = parse_expr(tokens);

    match ast {
        AstNode::BinaryExpression { left, operator, right } => {
            assert_eq!(operator, "+");
            // Left should be 1
            match *left {
                AstNode::Literal(Value::U64(v)) => assert_eq!(v, 1),
                _ => panic!("Expected left literal 1"),
            }
            // Right should be (2 * 3)
            match *right {
                AstNode::BinaryExpression { left: r_left, operator: r_op, right: r_right } => {
                    assert_eq!(r_op, "*");
                    match *r_left {
                        AstNode::Literal(Value::U64(v)) => assert_eq!(v, 2),
                        _ => panic!("Expected inner left literal 2"),
                    }
                    match *r_right {
                        AstNode::Literal(Value::U64(v)) => assert_eq!(v, 3),
                        _ => panic!("Expected inner right literal 3"),
                    }
                }
                _ => panic!("Expected inner BinaryExpression"),
            }
        }
        _ => panic!("Expected BinaryExpression"),
    }
}

#[test]
fn test_parse_nested_expression() {
    // (1 + 2) * 3
    let tokens = vec![
        Token::LeftParen,
        Token::NumberLiteral(1),
        Token::Plus,
        Token::NumberLiteral(2),
        Token::RightParen,
        Token::Multiply,
        Token::NumberLiteral(3),
        Token::Eof,
    ];
    let ast = parse_expr(tokens);

    match ast {
        AstNode::BinaryExpression { left, operator, right } => {
            assert_eq!(operator, "*");
            // Right should be 3
            match *right {
                AstNode::Literal(Value::U64(v)) => assert_eq!(v, 3),
                _ => panic!("Expected right literal 3"),
            }
            // Left should be (1 + 2)
            match *left {
                AstNode::BinaryExpression { left: l_left, operator: l_op, right: l_right } => {
                    assert_eq!(l_op, "+");
                    match *l_left {
                        AstNode::Literal(Value::U64(v)) => assert_eq!(v, 1),
                        _ => panic!("Expected inner left literal 1"),
                    }
                    match *l_right {
                        AstNode::Literal(Value::U64(v)) => assert_eq!(v, 2),
                        _ => panic!("Expected inner right literal 2"),
                    }
                }
                _ => panic!("Expected inner BinaryExpression"),
            }
        }
        _ => panic!("Expected BinaryExpression"),
    }
}

#[test]
fn test_parse_method_call() {
    // x.foo(1)
    let tokens = vec![
        Token::Identifier("x".to_string()),
        Token::Dot,
        Token::Identifier("foo".to_string()),
        Token::LeftParen,
        Token::NumberLiteral(1),
        Token::RightParen,
        Token::Eof,
    ];
    let ast = parse_expr(tokens);

    match ast {
        AstNode::MethodCall { object, method, args } => {
            match *object {
                AstNode::Identifier(name) => assert_eq!(name, "x"),
                _ => panic!("Expected object identifier"),
            }
            assert_eq!(method, "foo");
            assert_eq!(args.len(), 1);
            match args[0] {
                AstNode::Literal(Value::U64(v)) => assert_eq!(v, 1),
                _ => panic!("Expected argument literal"),
            }
        }
        _ => panic!("Expected MethodCall"),
    }
}

#[test]
fn test_parse_field_access() {
    // x.y
    let tokens = vec![
        Token::Identifier("x".to_string()),
        Token::Dot,
        Token::Identifier("y".to_string()),
        Token::Eof,
    ];
    let ast = parse_expr(tokens);

    match ast {
        AstNode::FieldAccess { object, field } => {
            match *object {
                AstNode::Identifier(name) => assert_eq!(name, "x"),
                _ => panic!("Expected object identifier"),
            }
            assert_eq!(field, "y");
        }
        _ => panic!("Expected FieldAccess"),
    }
}

#[test]
fn test_parse_array_access() {
    // arr[0]
    let tokens = vec![
        Token::Identifier("arr".to_string()),
        Token::LeftBracket,
        Token::NumberLiteral(0),
        Token::RightBracket,
        Token::Eof,
    ];
    let ast = parse_expr(tokens);

    match ast {
        AstNode::ArrayAccess { array, index } => {
            match *array {
                AstNode::Identifier(name) => assert_eq!(name, "arr"),
                _ => panic!("Expected array identifier"),
            }
            match *index {
                AstNode::Literal(Value::U64(v)) => assert_eq!(v, 0),
                _ => panic!("Expected index literal"),
            }
        }
        _ => panic!("Expected ArrayAccess"),
    }
}

#[test]
fn test_parse_logical_and() {
    // true && false
    let tokens = vec![
        Token::True,
        Token::LogicalAnd,
        Token::False,
        Token::Eof,
    ];
    let ast = parse_expr(tokens);

    // Logical AND parses as a binary expression
    match ast {
        AstNode::BinaryExpression { left, operator, right } => {
            assert_eq!(operator, "&&");
            match *left {
                AstNode::Literal(Value::Bool(v)) => assert!(v),
                _ => panic!("Expected left boolean"),
            }
            match *right {
                AstNode::Literal(Value::Bool(v)) => assert!(!v),
                _ => panic!("Expected right boolean"),
            }
        }
        _ => panic!("Expected BinaryExpression for Logical AND"),
    }
}

#[test]
fn test_parse_logical_or() {
    // true || false
    let tokens = vec![
        Token::True,
        Token::LogicalOr,
        Token::False,
        Token::Eof,
    ];
    let ast = parse_expr(tokens);

    // Logical OR parses as a binary expression
    match ast {
        AstNode::BinaryExpression { left, operator, right } => {
            assert_eq!(operator, "||");
            match *left {
                AstNode::Literal(Value::Bool(v)) => assert!(v),
                _ => panic!("Expected left boolean"),
            }
            match *right {
                AstNode::Literal(Value::Bool(v)) => assert!(!v),
                _ => panic!("Expected right boolean"),
            }
        }
        _ => panic!("Expected BinaryExpression for Logical OR"),
    }
}

#[test]
fn test_parse_unary_not() {
    // !true
    let tokens = vec![
        Token::Bang,
        Token::True,
        Token::Eof,
    ];
    let ast = parse_expr(tokens);

    match ast {
        AstNode::UnaryExpression { operator, operand } => {
            assert_eq!(operator, "not");
            match *operand {
                AstNode::Literal(Value::Bool(v)) => assert!(v),
                _ => panic!("Expected operand boolean"),
            }
        }
        _ => panic!("Expected UnaryExpression"),
    }
}

#[test]
fn test_parse_comparison() {
    // 1 < 2
    let tokens = vec![
        Token::NumberLiteral(1),
        Token::LT,
        Token::NumberLiteral(2),
        Token::Eof,
    ];
    let ast = parse_expr(tokens);

    // Comparisons are parsed as method calls .lt()
    match ast {
        AstNode::MethodCall { object, method, args } => {
            assert_eq!(method, "lt");
            match *object {
                AstNode::Literal(Value::U64(v)) => assert_eq!(v, 1),
                _ => panic!("Expected left literal"),
            }
            assert_eq!(args.len(), 1);
            match args[0] {
                AstNode::Literal(Value::U64(v)) => assert_eq!(v, 2),
                _ => panic!("Expected right literal"),
            }
        }
        _ => panic!("Expected MethodCall for comparison"),
    }
}

#[test]
fn test_parse_struct_literal() {
    // { x: 1, y: 2 }
    let tokens = vec![
        Token::LeftBrace,
        Token::Identifier("x".to_string()),
        Token::Colon,
        Token::NumberLiteral(1),
        Token::Comma,
        Token::Identifier("y".to_string()),
        Token::Colon,
        Token::NumberLiteral(2),
        Token::RightBrace,
        Token::Eof,
    ];
    let ast = parse_expr(tokens);

    match ast {
        AstNode::StructLiteral { fields } => {
            assert_eq!(fields.len(), 2);

            assert_eq!(fields[0].field_name, "x");
            match fields[0].value.as_ref() {
                AstNode::Literal(Value::U64(v)) => assert_eq!(*v, 1),
                _ => panic!("Expected field x value"),
            }

            assert_eq!(fields[1].field_name, "y");
            match fields[1].value.as_ref() {
                AstNode::Literal(Value::U64(v)) => assert_eq!(*v, 2),
                _ => panic!("Expected field y value"),
            }
        }
        _ => panic!("Expected StructLiteral"),
    }
}

#[test]
fn test_parse_array_literal() {
    // [1, 2]
    let tokens = vec![
        Token::LeftBracket,
        Token::NumberLiteral(1),
        Token::Comma,
        Token::NumberLiteral(2),
        Token::RightBracket,
        Token::Eof,
    ];
    let ast = parse_expr(tokens);

    match ast {
        AstNode::ArrayLiteral { elements } => {
            assert_eq!(elements.len(), 2);
            match elements[0] {
                AstNode::Literal(Value::U64(v)) => assert_eq!(v, 1),
                _ => panic!("Expected element 1"),
            }
            match elements[1] {
                AstNode::Literal(Value::U64(v)) => assert_eq!(v, 2),
                _ => panic!("Expected element 2"),
            }
        }
        _ => panic!("Expected ArrayLiteral"),
    }
}

#[test]
fn test_parse_pubkey_zero_constructor_identifier_token() {
    let tokens = vec![
        Token::Identifier("pubkey".to_string()),
        Token::LeftParen,
        Token::NumberLiteral(0),
        Token::RightParen,
        Token::Eof,
    ];
    let ast = parse_expr(tokens);

    match ast {
        AstNode::Literal(Value::U64(v)) => assert_eq!(v, 0),
        _ => panic!("Expected Literal(U64(0)) for pubkey(0), got {:?}", ast),
    }
}

#[test]
fn test_parse_pubkey_zero_constructor_type_token() {
    let tokens = vec![
        Token::Type("pubkey".to_string()),
        Token::LeftParen,
        Token::NumberLiteral(0),
        Token::RightParen,
        Token::Eof,
    ];
    let ast = parse_expr(tokens);

    match ast {
        AstNode::Literal(Value::U64(v)) => assert_eq!(v, 0),
        _ => panic!("Expected Literal(U64(0)) for pubkey(0), got {:?}", ast),
    }
}
