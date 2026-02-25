use crate::type_checker::types::TypeCheckerContext;
use crate::ast::{AstNode, TypeNode};
use five_protocol::Value;
use five_vm_mito::error::VMError;

#[test]
fn test_check_let_statement() {
    let mut checker = TypeCheckerContext::new();
    let node = AstNode::LetStatement {
        name: "x".to_string(),
        type_annotation: None,
        is_mutable: false,
        value: Box::new(AstNode::Literal(Value::U64(1))),
    };
    assert!(checker.check_statement(&node).is_ok());

    // Check symbol table
    let symbol = checker.symbol_table.get("x").expect("x should be defined");
    assert_eq!(symbol.0, TypeNode::Primitive("u64".to_string()));
    assert!(!symbol.1); // Not mutable
}

#[test]
fn test_check_let_mut_statement() {
    let mut checker = TypeCheckerContext::new();
    let node = AstNode::LetStatement {
        name: "x".to_string(),
        type_annotation: None,
        is_mutable: true,
        value: Box::new(AstNode::Literal(Value::U64(1))),
    };
    assert!(checker.check_statement(&node).is_ok());

    let symbol = checker.symbol_table.get("x").expect("x should be defined");
    assert!(symbol.1); // Mutable
}

#[test]
fn test_check_let_typed_valid() {
    let mut checker = TypeCheckerContext::new();
    let node = AstNode::LetStatement {
        name: "x".to_string(),
        type_annotation: Some(Box::new(TypeNode::Primitive("u64".to_string()))),
        is_mutable: false,
        value: Box::new(AstNode::Literal(Value::U64(1))),
    };
    assert!(checker.check_statement(&node).is_ok());
}

#[test]
fn test_check_let_typed_invalid() {
    let mut checker = TypeCheckerContext::new();
    let node = AstNode::LetStatement {
        name: "x".to_string(),
        type_annotation: Some(Box::new(TypeNode::Primitive("bool".to_string()))),
        is_mutable: false,
        value: Box::new(AstNode::Literal(Value::U64(1))),
    };
    assert!(matches!(checker.check_statement(&node), Err(VMError::TypeMismatch)));
}

#[test]
fn test_check_assignment_valid() {
    let mut checker = TypeCheckerContext::new();
    // let mut x = 1;
    checker.symbol_table.insert("x".to_string(), (TypeNode::Primitive("u64".to_string()), true));

    let node = AstNode::Assignment {
        target: "x".to_string(),
        value: Box::new(AstNode::Literal(Value::U64(2))),
    };

    assert!(checker.check_statement(&node).is_ok());
}

#[test]
fn test_check_assignment_type_mismatch() {
    let mut checker = TypeCheckerContext::new();
    checker.symbol_table.insert("x".to_string(), (TypeNode::Primitive("u64".to_string()), true));

    let node = AstNode::Assignment {
        target: "x".to_string(),
        value: Box::new(AstNode::Literal(Value::Bool(true))),
    };

    assert!(matches!(checker.check_statement(&node), Err(VMError::TypeMismatch)));
}

#[test]
fn test_check_if_statement() {
    let mut checker = TypeCheckerContext::new();
    let node = AstNode::IfStatement {
        condition: Box::new(AstNode::Literal(Value::Bool(true))),
        then_branch: Box::new(AstNode::Block { statements: vec![], kind: crate::ast::BlockKind::Regular }),
        else_branch: None,
    };
    assert!(checker.check_statement(&node).is_ok());
}

#[test]
fn test_check_if_statement_invalid_condition() {
    let mut checker = TypeCheckerContext::new();
    // Currently the type checker only validates that the condition is a valid expression,
    // not that it evaluates to boolean. This test documents current behavior.
    let node = AstNode::IfStatement {
        condition: Box::new(AstNode::Literal(Value::U64(1))),
        then_branch: Box::new(AstNode::Block { statements: vec![], kind: crate::ast::BlockKind::Regular }),
        else_branch: None,
    };
    assert!(checker.check_statement(&node).is_ok());
}

#[test]
fn test_check_while_loop() {
    let mut checker = TypeCheckerContext::new();
    let node = AstNode::WhileLoop {
        condition: Box::new(AstNode::Literal(Value::Bool(true))),
        body: Box::new(AstNode::Block { statements: vec![], kind: crate::ast::BlockKind::Regular }),
    };
    assert!(checker.check_statement(&node).is_ok());
}

#[test]
fn test_field_assignment_to_account_ctx_is_read_only() {
    let mut checker = TypeCheckerContext::new();
    checker.symbol_table.insert("vault".to_string(), (TypeNode::Account, true));

    let node = AstNode::FieldAssignment {
        object: Box::new(AstNode::FieldAccess {
            object: Box::new(AstNode::Identifier("vault".to_string())),
            field: "ctx".to_string(),
        }),
        field: "lamports".to_string(),
        value: Box::new(AstNode::Literal(Value::U64(1))),
    };

    assert!(matches!(checker.check_statement(&node), Err(VMError::ImmutableField)));
}
