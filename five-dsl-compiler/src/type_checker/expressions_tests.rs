use crate::type_checker::types::TypeCheckerContext;
use crate::ast::{AstNode, TypeNode};
use five_protocol::Value;
use five_vm_mito::error::VMError;

#[test]
fn test_check_literal_u64() {
    let mut checker = TypeCheckerContext::new();
    let node = AstNode::Literal(Value::U64(123));
    assert!(checker.check_expression(&node).is_ok());
}

#[test]
fn test_check_literal_bool() {
    let mut checker = TypeCheckerContext::new();
    let node = AstNode::Literal(Value::Bool(true));
    assert!(checker.check_expression(&node).is_ok());
}

#[test]
fn test_check_identifier_not_found() {
    let mut checker = TypeCheckerContext::new();
    let node = AstNode::Identifier("x".to_string());
    assert!(matches!(
        checker.check_expression(&node),
        Err(VMError::UndefinedIdentifier | VMError::UndefinedIdentifierWithContext { .. })
    ));
}

#[test]
fn test_check_identifier_found() {
    let mut checker = TypeCheckerContext::new();
    // Simulate variable 'x' in symbol table
    checker.symbol_table.insert("x".to_string(), (TypeNode::Primitive("u64".to_string()), false));
    let node = AstNode::Identifier("x".to_string());
    assert!(checker.check_expression(&node).is_ok());
}

#[test]
fn test_check_binary_expression_valid() {
    let mut checker = TypeCheckerContext::new();
    // x + 1
    // Need 'x' in scope
    checker.symbol_table.insert("x".to_string(), (TypeNode::Primitive("u64".to_string()), false));

    let node = AstNode::BinaryExpression {
        left: Box::new(AstNode::Identifier("x".to_string())),
        operator: "+".to_string(),
        right: Box::new(AstNode::Literal(Value::U64(1))),
    };

    assert!(checker.check_expression(&node).is_ok());
}

#[test]
fn test_check_binary_expression_type_mismatch() {
    let mut checker = TypeCheckerContext::new();
    // x + true (u64 + bool) -> Mismatch
    checker.symbol_table.insert("x".to_string(), (TypeNode::Primitive("u64".to_string()), false));

    let node = AstNode::BinaryExpression {
        left: Box::new(AstNode::Identifier("x".to_string())),
        operator: "+".to_string(),
        right: Box::new(AstNode::Literal(Value::Bool(true))),
    };

    // infer_type does the strict check for binary ops usually
    // check_expression mainly calls check_types on children, but infer_type is where validation happens
    // check_expression calls check_types(left) and check_types(right).
    // It does NOT check compatibility of left and right for BinaryExpression,
    // unless infer_type is called.
    // See `expressions.rs`.
    /*
    AstNode::BinaryExpression { operator: _, left, right } => {
        self.check_types(left)?;
        self.check_types(right)?;
        Ok(())
    }
    */
    // Ah, `check_expression` for BinaryExpression only checks children!
    // It doesn't validate operand compatibility? That seems like a weakness in `check_expression`
    // or maybe `check_types` delegates to `infer_type` somewhere else?
    // Usually type checking involves inference to verify compatibility.
    // If I look at `expressions.rs`, `MethodCall` does check compatibility.
    // But `BinaryExpression` implementation in `check_expression` (which I read earlier)
    // simply recurses.

    // So this test might actually PASS even with mismatch if I just call `check_expression`.
    // Verify `check_expression` behavior for BinaryExpression.
    // If so, I should test `infer_type` instead for these cases if I want to verify type safety.

    // However, `MethodCall` has logic:
    /*
    if matches!(method.as_str(), "add" | "sub" | "mul" | "div") {
       ...
    }
    */
    // The parser converts some ops to method calls?
    // No, `parser/expressions.rs` converts `&&` `||` `>` `<` etc to method calls,
    // but `+` `-` `*` `/` are kept as `BinaryExpression` in `parse_additive` / `parse_multiplicative`.
    // See `parser/expressions.rs`.

    // `parse_additive`:
    // left = AstNode::BinaryExpression { ... operator: "+", ... }

    // So `+` is `BinaryExpression`.
    // And `type_checker/expressions.rs`:
    /*
    AstNode::BinaryExpression { .. } => {
        self.check_types(left)?;
        self.check_types(right)?;
        Ok(())
    }
    */
    // This confirms `check_expression` does NOT check type compatibility for binary expressions.
    // `infer_type` does.
    // So to test type safety of binary expressions, I must call `infer_type`.

    // Add a test for `infer_type` here as well.
    // Since `infer_type` is pub(crate), I can access it.

    // For this test, I expect it to OK if I only call check_expression.
    assert!(checker.check_expression(&node).is_ok());
}

#[test]
fn test_infer_binary_expression_type_mismatch() {
    let mut checker = TypeCheckerContext::new();
    checker.symbol_table.insert("x".to_string(), (TypeNode::Primitive("u64".to_string()), false));

    let node = AstNode::BinaryExpression {
        left: Box::new(AstNode::Identifier("x".to_string())),
        operator: "+".to_string(),
        right: Box::new(AstNode::Literal(Value::Bool(true))),
    };

    // infer_type should catch the mismatch
    assert!(matches!(checker.infer_type(&node), Err(VMError::TypeMismatch)));
}

#[test]
fn test_check_method_call() {
    let mut checker = TypeCheckerContext::new();
    // x.add(1)
    checker.symbol_table.insert("x".to_string(), (TypeNode::Primitive("u64".to_string()), false));

    let node = AstNode::MethodCall {
        object: Box::new(AstNode::Identifier("x".to_string())),
        method: "add".to_string(),
        args: vec![AstNode::Literal(Value::U64(1))],
    };

    // MethodCall in check_expression DOES check types:
    /*
    AstNode::MethodCall { .. } => {
        self.check_types(object)?;
        for arg in args { self.check_types(arg)?; }
        // Check for type mismatches in arithmetic operations ...
    }
    */
    // It has specific logic for add/sub/mul/div if object is identifier string?
    // See `type_checker/expressions.rs`.

    assert!(checker.check_expression(&node).is_ok());
}

#[test]
fn test_check_array_access_valid() {
    let mut checker = TypeCheckerContext::new();
    // arr[0]
    // arr: u64[10]
    let array_type = TypeNode::Array {
        element_type: Box::new(TypeNode::Primitive("u64".to_string())),
        size: Some(10),
    };
    checker.symbol_table.insert("arr".to_string(), (array_type, false));

    let node = AstNode::ArrayAccess {
        array: Box::new(AstNode::Identifier("arr".to_string())),
        index: Box::new(AstNode::Literal(Value::U64(0))),
    };

    assert!(checker.check_expression(&node).is_ok());
}

#[test]
fn test_check_array_access_invalid_index() {
    let mut checker = TypeCheckerContext::new();
    // arr[true]
    let array_type = TypeNode::Array {
        element_type: Box::new(TypeNode::Primitive("u64".to_string())),
        size: Some(10),
    };
    checker.symbol_table.insert("arr".to_string(), (array_type, false));

    let node = AstNode::ArrayAccess {
        array: Box::new(AstNode::Identifier("arr".to_string())),
        index: Box::new(AstNode::Literal(Value::Bool(true))),
    };

    // check_expression for ArrayAccess calls infer_type on index and checks if it is numeric
    assert!(matches!(checker.check_expression(&node), Err(VMError::InvalidScript)));
    // Note: check_expression returns InvalidScript for non-numeric index in ArrayAccess
}

#[test]
fn test_check_struct_literal() {
    let mut checker = TypeCheckerContext::new();
    // { x: 1 }
    let node = AstNode::StructLiteral {
        fields: vec![
            crate::ast::StructLiteralField {
                field_name: "x".to_string(),
                value: Box::new(AstNode::Literal(Value::U64(1))),
            }
        ],
    };

    assert!(checker.check_expression(&node).is_ok());
}

#[test]
fn test_infer_pubkey_zero_constructor_type() {
    let mut checker = TypeCheckerContext::new();
    let node = AstNode::FunctionCall {
        name: "pubkey".to_string(),
        args: vec![AstNode::Literal(Value::U64(0))],
    };

    let ty = checker.infer_type(&node).expect("pubkey(0) should infer");
    assert_eq!(ty, TypeNode::Primitive("pubkey".to_string()));
}

#[test]
fn test_infer_pubkey_constructor_rejects_non_zero_numeric() {
    let mut checker = TypeCheckerContext::new();
    let node = AstNode::FunctionCall {
        name: "pubkey".to_string(),
        args: vec![AstNode::Literal(Value::U64(1))],
    };

    assert!(matches!(checker.infer_type(&node), Err(VMError::TypeMismatch)));
}
