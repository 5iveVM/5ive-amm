use crate::ast::{AstNode, TypeNode};
use crate::type_checker::types::TypeCheckerContext;
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
    checker.symbol_table.insert(
        "x".to_string(),
        (TypeNode::Primitive("u64".to_string()), false),
    );
    let node = AstNode::Identifier("x".to_string());
    assert!(checker.check_expression(&node).is_ok());
}

#[test]
fn test_check_binary_expression_valid() {
    let mut checker = TypeCheckerContext::new();
    // x + 1
    // Need 'x' in scope
    checker.symbol_table.insert(
        "x".to_string(),
        (TypeNode::Primitive("u64".to_string()), false),
    );

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
    checker.symbol_table.insert(
        "x".to_string(),
        (TypeNode::Primitive("u64".to_string()), false),
    );

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
    checker.symbol_table.insert(
        "x".to_string(),
        (TypeNode::Primitive("u64".to_string()), false),
    );

    let node = AstNode::BinaryExpression {
        left: Box::new(AstNode::Identifier("x".to_string())),
        operator: "+".to_string(),
        right: Box::new(AstNode::Literal(Value::Bool(true))),
    };

    // infer_type should catch the mismatch
    assert!(matches!(
        checker.infer_type(&node),
        Err(VMError::TypeMismatch)
    ));
}

#[test]
fn test_check_method_call() {
    let mut checker = TypeCheckerContext::new();
    // x.add(1)
    checker.symbol_table.insert(
        "x".to_string(),
        (TypeNode::Primitive("u64".to_string()), false),
    );

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
    checker
        .symbol_table
        .insert("arr".to_string(), (array_type, false));

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
    checker
        .symbol_table
        .insert("arr".to_string(), (array_type, false));

    let node = AstNode::ArrayAccess {
        array: Box::new(AstNode::Identifier("arr".to_string())),
        index: Box::new(AstNode::Literal(Value::Bool(true))),
    };

    // check_expression for ArrayAccess calls infer_type on index and checks if it is numeric
    assert!(matches!(
        checker.check_expression(&node),
        Err(VMError::InvalidScript)
    ));
    // Note: check_expression returns InvalidScript for non-numeric index in ArrayAccess
}

#[test]
fn test_check_struct_literal() {
    let mut checker = TypeCheckerContext::new();
    // { x: 1 }
    let node = AstNode::StructLiteral {
        fields: vec![crate::ast::StructLiteralField {
            field_name: "x".to_string(),
            value: Box::new(AstNode::Literal(Value::U64(1))),
        }],
    };

    assert!(checker.check_expression(&node).is_ok());
}

#[test]
fn test_check_account_ctx_core_fields() {
    let mut checker = TypeCheckerContext::new();
    checker
        .symbol_table
        .insert("vault".to_string(), (TypeNode::Account, false));

    let lamports_expr = AstNode::FieldAccess {
        object: Box::new(AstNode::FieldAccess {
            object: Box::new(AstNode::Identifier("vault".to_string())),
            field: "ctx".to_string(),
        }),
        field: "lamports".to_string(),
    };
    assert!(checker.check_expression(&lamports_expr).is_ok());
    assert_eq!(
        checker.infer_type(&lamports_expr).expect("infer lamports"),
        TypeNode::Primitive("u64".to_string())
    );
}

#[test]
fn test_check_account_ctx_bump_requires_seeded_init_context() {
    let mut checker = TypeCheckerContext::new();
    checker
        .symbol_table
        .insert("vault".to_string(), (TypeNode::Account, false));

    let bump_expr = AstNode::FieldAccess {
        object: Box::new(AstNode::FieldAccess {
            object: Box::new(AstNode::Identifier("vault".to_string())),
            field: "ctx".to_string(),
        }),
        field: "bump".to_string(),
    };

    assert!(matches!(
        checker.check_expression(&bump_expr),
        Err(VMError::UndefinedField)
    ));

    checker.init_bump_accounts.insert("vault".to_string());
    assert!(checker.check_expression(&bump_expr).is_ok());
    assert_eq!(
        checker.infer_type(&bump_expr).expect("infer bump"),
        TypeNode::Primitive("u8".to_string())
    );
}

#[test]
fn test_legacy_metadata_field_access_provides_ctx_hint() {
    let mut checker = TypeCheckerContext::new();
    checker
        .symbol_table
        .insert("payer".to_string(), (TypeNode::Account, false));

    let expr = AstNode::FieldAccess {
        object: Box::new(AstNode::Identifier("payer".to_string())),
        field: "lamports".to_string(),
    };

    match checker.check_expression(&expr) {
        Err(VMError::UndefinedIdentifierWithContext {
            identifier,
            did_you_mean,
        }) => {
            assert_eq!(identifier.as_str(), "lamports");
            assert_eq!(
                did_you_mean.as_ref().map(|s| s.as_str()),
                Some("ctx.lamports")
            );
        }
        other => panic!("expected UndefinedIdentifierWithContext, got {:?}", other),
    }
}

#[test]
fn test_legacy_init_alias_identifier_provides_ctx_hint() {
    let mut checker = TypeCheckerContext::new();
    checker.init_bump_accounts.insert("vault".to_string());

    let expr = AstNode::Identifier("vault_bump".to_string());

    match checker.check_expression(&expr) {
        Err(VMError::UndefinedIdentifierWithContext {
            identifier,
            did_you_mean,
        }) => {
            assert_eq!(identifier.as_str(), "vault_bump");
            assert_eq!(
                did_you_mean.as_ref().map(|s| s.as_str()),
                Some("vault.ctx.bump")
            );
        }
        other => panic!("expected UndefinedIdentifierWithContext, got {:?}", other),
    }
}

#[test]
fn test_namespaced_account_field_access_resolves() {
    let mut checker = TypeCheckerContext::new();
    checker.account_definitions.insert(
        "std::interfaces::spl_token::Mint".to_string(),
        vec![crate::ast::StructField {
            name: "supply".to_string(),
            field_type: TypeNode::Primitive("u64".to_string()),
            is_mutable: false,
            is_optional: false,
        }],
    );
    checker.symbol_table.insert(
        "mint".to_string(),
        (TypeNode::Named("spl_token::Mint".to_string()), false),
    );

    let expr = AstNode::FieldAccess {
        object: Box::new(AstNode::Identifier("mint".to_string())),
        field: "supply".to_string(),
    };

    assert!(checker.check_expression(&expr).is_ok());
    assert_eq!(
        checker.infer_type(&expr).expect("infer namespaced field"),
        TypeNode::Primitive("u64".to_string())
    );
}

#[test]
fn test_named_account_metadata_field_provides_ctx_hint() {
    let mut checker = TypeCheckerContext::new();
    checker.account_definitions.insert(
        "std::interfaces::spl_token::Mint".to_string(),
        vec![crate::ast::StructField {
            name: "supply".to_string(),
            field_type: TypeNode::Primitive("u64".to_string()),
            is_mutable: false,
            is_optional: false,
        }],
    );
    checker.symbol_table.insert(
        "mint".to_string(),
        (TypeNode::Named("spl_token::Mint".to_string()), false),
    );

    let expr = AstNode::FieldAccess {
        object: Box::new(AstNode::Identifier("mint".to_string())),
        field: "key".to_string(),
    };

    match checker.check_expression(&expr) {
        Err(VMError::UndefinedIdentifierWithContext {
            identifier,
            did_you_mean,
        }) => {
            assert_eq!(identifier.as_str(), "key");
            assert_eq!(
                did_you_mean.as_ref().map(|s| s.as_str()),
                Some("mint.ctx.key")
            );
        }
        other => panic!("expected UndefinedIdentifierWithContext, got {:?}", other),
    }
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

    assert!(matches!(
        checker.infer_type(&node),
        Err(VMError::TypeMismatch)
    ));
}

#[test]
fn test_infer_close_account_type() {
    let mut checker = TypeCheckerContext::new();
    checker
        .symbol_table
        .insert("vault".to_string(), (TypeNode::Account, true));
    checker
        .symbol_table
        .insert("maker".to_string(), (TypeNode::Account, true));

    let node = AstNode::FunctionCall {
        name: "close_account".to_string(),
        args: vec![
            AstNode::Identifier("vault".to_string()),
            AstNode::Identifier("maker".to_string()),
        ],
    };

    let ty = checker
        .infer_type(&node)
        .expect("close_account should type-check");
    assert_eq!(ty, TypeNode::Primitive("void".to_string()));
}
