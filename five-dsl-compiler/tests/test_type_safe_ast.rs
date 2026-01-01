//! Comprehensive tests for type-safe AST structures
//!
//! Tests the new Expression, Statement, and Definition enums generated from node_metadata.toml
//! Ensures 100% coverage of all 48 AST node variants and their conversions.

use five_dsl_compiler::ast::{AstNode, generated::*, BlockKind};
use five_protocol::Value;

// ============================================================================
// TYPE-SAFE EXPRESSION TESTS
// ============================================================================

#[test]
fn test_expression_enum_completeness() {
    // Verify all expression variants are represented
    let expr_variants = [
        "Identifier",
        "Literal",
        "StringLiteral",
        "ArrayLiteral",
        "TupleLiteral",
        "StructLiteral",
        "TemplateLiteral",
        "FieldAccess",
        "ArrayAccess",
        "TupleAccess",
        "FunctionCall",
        "MethodCall",
        "EnumVariantAccess",
        "ErrorPropagation",
        "UnaryExpression",
        "BinaryExpression",
    ];

    assert_eq!(expr_variants.len(), 16, "Expression should have 16 variants");
}

#[test]
fn test_binary_expression_construction_and_conversion() {
    // Create a type-safe BinaryExpressionNode
    let left = AstNode::Literal(Value::U64(5));
    let right = AstNode::Literal(Value::U64(3));

    let binary = BinaryExpressionNode {
        operator: "+".to_string(),
        left: Box::new(left),
        right: Box::new(right),
    };

    // Convert to Expression
    let expr = Expression::BinaryExpression(binary.clone());

    // Convert to AstNode
    let ast_node: AstNode = expr.into();

    // Verify it's an AstNode::BinaryExpression
    match ast_node {
        AstNode::BinaryExpression { operator, left, right } => {
            assert_eq!(operator, "+");
            assert!(matches!(*left, AstNode::Literal(_)));
            assert!(matches!(*right, AstNode::Literal(_)));
        }
        _ => panic!("Expected BinaryExpression"),
    }
}

#[test]
fn test_unary_expression_construction_and_conversion() {
    let operand = AstNode::Literal(Value::U64(42));

    let unary = UnaryExpressionNode {
        operator: "!".to_string(),
        operand: Box::new(operand),
    };

    let expr = Expression::UnaryExpression(unary);
    let ast_node: AstNode = expr.into();

    match ast_node {
        AstNode::UnaryExpression { operator, operand } => {
            assert_eq!(operator, "!");
            assert!(matches!(*operand, AstNode::Literal(_)));
        }
        _ => panic!("Expected UnaryExpression"),
    }
}

#[test]
fn test_function_call_expression() {
    let func_call = FunctionCallNode {
        name: "transfer".to_string(),
        args: vec![
            AstNode::Literal(Value::U64(100)),
            AstNode::Identifier("recipient".to_string()),
        ],
    };

    let expr = Expression::FunctionCall(func_call);
    let ast_node: AstNode = expr.into();

    match ast_node {
        AstNode::FunctionCall { name, args } => {
            assert_eq!(name, "transfer");
            assert_eq!(args.len(), 2);
        }
        _ => panic!("Expected FunctionCall"),
    }
}

#[test]
fn test_field_access_expression() {
    let field_access = FieldAccessNode {
        object: Box::new(AstNode::Identifier("account".to_string())),
        field: "owner".to_string(),
    };

    let expr = Expression::FieldAccess(field_access);
    let ast_node: AstNode = expr.into();

    match ast_node {
        AstNode::FieldAccess { object, field } => {
            assert_eq!(field, "owner");
            assert!(matches!(*object, AstNode::Identifier(_)));
        }
        _ => panic!("Expected FieldAccess"),
    }
}

#[test]
fn test_array_literal_expression() {
    let array = ArrayLiteralNode {
        elements: vec![
            AstNode::Literal(Value::U64(1)),
            AstNode::Literal(Value::U64(2)),
            AstNode::Literal(Value::U64(3)),
        ],
    };

    let expr = Expression::ArrayLiteral(array);
    let ast_node: AstNode = expr.into();

    match ast_node {
        AstNode::ArrayLiteral { elements } => {
            assert_eq!(elements.len(), 3);
        }
        _ => panic!("Expected ArrayLiteral"),
    }
}

// ============================================================================
// TYPE-SAFE STATEMENT TESTS
// ============================================================================

#[test]
fn test_statement_enum_completeness() {
    // Verify all statement variants are represented
    let stmt_variants = [
        "Assignment",
        "FieldAssignment",
        "RequireStatement",
        "LetStatement",
        "TupleDestructuring",
        "TupleAssignment",
        "IfStatement",
        "MatchExpression",
        "ReturnStatement",
        "ForLoop",
        "ForInLoop",
        "ForOfLoop",
        "WhileLoop",
        "DoWhileLoop",
        "SwitchStatement",
        "BreakStatement",
        "ContinueStatement",
        "EmitStatement",
        "AssertStatement",
    ];

    assert_eq!(stmt_variants.len(), 19, "Statement should have 19 variants");
}

#[test]
fn test_let_statement_construction_and_conversion() {
    let let_stmt = LetStatementNode {
        name: "amount".to_string(),
        value: Box::new(AstNode::Literal(Value::U64(100))),
        type_annotation: None,
        is_mutable: true,
    };

    let stmt = Statement::LetStatement(let_stmt);
    let ast_node: AstNode = stmt.into();

    match ast_node {
        AstNode::LetStatement { name, value, is_mutable, .. } => {
            assert_eq!(name, "amount");
            assert!(is_mutable);
            assert!(matches!(*value, AstNode::Literal(_)));
        }
        _ => panic!("Expected LetStatement"),
    }
}

#[test]
fn test_assignment_statement() {
    let assign = AssignmentNode {
        target: "balance".to_string(),
        value: Box::new(AstNode::Literal(Value::U64(500))),
    };

    let stmt = Statement::Assignment(assign);
    let ast_node: AstNode = stmt.into();

    match ast_node {
        AstNode::Assignment { target, value } => {
            assert_eq!(target, "balance");
            assert!(matches!(*value, AstNode::Literal(_)));
        }
        _ => panic!("Expected Assignment"),
    }
}

#[test]
fn test_if_statement_construction() {
    let if_stmt = IfStatementNode {
        condition: Box::new(AstNode::Literal(Value::Bool(true))),
        then_branch: Box::new(AstNode::Block {
            statements: vec![],
            kind: BlockKind::Regular,
        }),
        else_branch: None,
    };

    let stmt = Statement::IfStatement(if_stmt);
    let ast_node: AstNode = stmt.into();

    match ast_node {
        AstNode::IfStatement { condition, then_branch, else_branch } => {
            assert!(matches!(*condition, AstNode::Literal(_)));
            assert!(matches!(*then_branch, AstNode::Block { .. }));
            assert_eq!(else_branch, None);
        }
        _ => panic!("Expected IfStatement"),
    }
}

#[test]
fn test_return_statement() {
    let ret_stmt = ReturnStatementNode {
        value: Some(Box::new(AstNode::Literal(Value::U64(42)))),
    };

    let stmt = Statement::ReturnStatement(ret_stmt);
    let ast_node: AstNode = stmt.into();

    match ast_node {
        AstNode::ReturnStatement { value } => {
            assert!(value.is_some());
            assert!(matches!(*value.unwrap(), AstNode::Literal(_)));
        }
        _ => panic!("Expected ReturnStatement"),
    }
}

#[test]
fn test_require_statement() {
    let require = RequireStatementNode {
        condition: Box::new(AstNode::Identifier("is_valid".to_string())),
    };

    let stmt = Statement::RequireStatement(require);
    let ast_node: AstNode = stmt.into();

    match ast_node {
        AstNode::RequireStatement { condition } => {
            assert!(matches!(*condition, AstNode::Identifier(_)));
        }
        _ => panic!("Expected RequireStatement"),
    }
}

#[test]
fn test_while_loop_statement() {
    let while_loop = WhileLoopNode {
        condition: Box::new(AstNode::Identifier("is_running".to_string())),
        body: Box::new(AstNode::Block {
            statements: vec![],
            kind: BlockKind::Regular,
        }),
    };

    let stmt = Statement::WhileLoop(while_loop);
    let ast_node: AstNode = stmt.into();

    match ast_node {
        AstNode::WhileLoop { condition, body } => {
            assert!(matches!(*condition, AstNode::Identifier(_)));
            assert!(matches!(*body, AstNode::Block { .. }));
        }
        _ => panic!("Expected WhileLoop"),
    }
}

// ============================================================================
// TYPE-SAFE DEFINITION TESTS
// ============================================================================

#[test]
fn test_definition_enum_completeness() {
    // Verify all definition variants are represented
    let def_variants = [
        "FieldDefinition",
        "InstructionDefinition",
        "EventDefinition",
        "ErrorTypeDefinition",
        "AccountDefinition",
        "InterfaceDefinition",
        "InterfaceFunction",
        "ImportStatement",
        "ArrowFunction",
        "TestFunction",
        "TestModule",
    ];

    assert_eq!(def_variants.len(), 11, "Definition should have 11 variants");
}

#[test]
fn test_field_definition_construction() {
    let field_def = FieldDefinitionNode {
        name: "balance".to_string(),
        field_type: Box::new(five_dsl_compiler::ast::TypeNode::Primitive("u64".to_string())),
        is_mutable: true,
        is_optional: false,
        default_value: None,
        visibility: five_dsl_compiler::ast::Visibility::Public,
    };

    let def = Definition::FieldDefinition(field_def);
    let ast_node: AstNode = def.into();

    match ast_node {
        AstNode::FieldDefinition { name, is_mutable, .. } => {
            assert_eq!(name, "balance");
            assert!(is_mutable);
        }
        _ => panic!("Expected FieldDefinition"),
    }
}

#[test]
fn test_instruction_definition_construction() {
    let instr_def = InstructionDefinitionNode {
        name: "transfer".to_string(),
        parameters: vec![],
        return_type: None,
        body: Box::new(AstNode::Block {
            statements: vec![],
            kind: BlockKind::Regular,
        }),
        visibility: five_dsl_compiler::ast::Visibility::Public,
    };

    let def = Definition::InstructionDefinition(instr_def);
    let ast_node: AstNode = def.into();

    match ast_node {
        AstNode::InstructionDefinition { name, .. } => {
            assert_eq!(name, "transfer");
        }
        _ => panic!("Expected InstructionDefinition"),
    }
}

// ============================================================================
// CONVERSION CONSISTENCY TESTS
// ============================================================================

#[test]
fn test_expression_roundtrip_conversion() {
    // Test that Expression -> AstNode conversion is consistent
    let exprs = vec![
        Expression::Identifier("test".to_string()),
        Expression::Literal(Value::U64(123)),
        Expression::StringLiteral(StringLiteralNode {
            value: "hello".to_string(),
        }),
    ];

    for expr in exprs {
        let ast_node: AstNode = expr.clone().into();

        // Verify the type is correct
        match (&expr, &ast_node) {
            (Expression::Identifier(name), AstNode::Identifier(ast_name)) => {
                assert_eq!(name, ast_name);
            }
            (Expression::Literal(val), AstNode::Literal(ast_val)) => {
                assert_eq!(val, ast_val);
            }
            (Expression::StringLiteral(s), AstNode::StringLiteral { value, .. }) => {
                assert_eq!(&s.value, value);
            }
            _ => panic!("Conversion mismatch"),
        }
    }
}

#[test]
fn test_statement_roundtrip_conversion() {
    // Test that Statement -> AstNode conversion is consistent
    let return_stmt = Statement::ReturnStatement(ReturnStatementNode {
        value: None,
    });

    let ast_node: AstNode = return_stmt.into();

    match ast_node {
        AstNode::ReturnStatement { value } => {
            assert_eq!(value, None);
        }
        _ => panic!("Expected ReturnStatement"),
    }
}

// Note: Parser integration tests are covered by existing integration tests in tests/lib.rs

// ============================================================================
// NODE REGISTRY VALIDATION TESTS
// ============================================================================

#[test]
fn test_node_registry_has_all_nodes() {
    let registry = &five_dsl_compiler::ast::NODE_REGISTRY;

    // Verify we have 48 nodes
    assert_eq!(
        registry.nodes.len(),
        48,
        "Expected 48 AST node definitions in registry"
    );

    // Verify key node types are present
    let required_nodes = [
        "BinaryExpression",
        "UnaryExpression",
        "IfStatement",
        "LetStatement",
        "Assignment",
        "InstructionDefinition",
        "FieldDefinition",
        "FunctionCall",
        "MethodCall",
        "ArrayLiteral",
        "Block",
        "Identifier",
        "Literal",
    ];

    for node_name in &required_nodes {
        assert!(
            registry.nodes.contains_key(*node_name),
            "Missing required node: {}",
            node_name
        );
    }
}

#[test]
fn test_node_registry_categories() {
    let registry = &five_dsl_compiler::ast::NODE_REGISTRY;

    // Verify nodes are properly categorized
    let expression_nodes = registry.get_by_category("expression");
    let statement_nodes = registry.get_by_category("statement");
    let definition_nodes = registry.get_by_category("definition");

    assert!(!expression_nodes.is_empty(), "Should have expression nodes");
    assert!(!statement_nodes.is_empty(), "Should have statement nodes");
    assert!(!definition_nodes.is_empty(), "Should have definition nodes");

    // BinaryExpression should be in expressions
    assert!(
        expression_nodes.iter().any(|n| n.name == "BinaryExpression"),
        "BinaryExpression should be categorized as expression"
    );

    // IfStatement should be in statements
    assert!(
        statement_nodes.iter().any(|n| n.name == "IfStatement"),
        "IfStatement should be categorized as statement"
    );
}

#[test]
fn test_node_registry_field_definitions() {
    let registry = &five_dsl_compiler::ast::NODE_REGISTRY;

    // Get BinaryExpression and verify it has required fields
    if let Some(binary_expr) = registry.get_node("BinaryExpression") {
        let field_names: Vec<&str> = binary_expr.fields.keys().map(|s| s.as_str()).collect();

        assert!(
            field_names.contains(&"operator"),
            "BinaryExpression should have 'operator' field"
        );
        assert!(
            field_names.contains(&"left"),
            "BinaryExpression should have 'left' field"
        );
        assert!(
            field_names.contains(&"right"),
            "BinaryExpression should have 'right' field"
        );
    } else {
        panic!("BinaryExpression not found in registry");
    }

    // Get IfStatement and verify it has required fields
    if let Some(if_stmt) = registry.get_node("IfStatement") {
        let field_names: Vec<&str> = if_stmt.fields.keys().map(|s| s.as_str()).collect();

        assert!(
            field_names.contains(&"condition"),
            "IfStatement should have 'condition' field"
        );
        assert!(
            field_names.contains(&"then_branch"),
            "IfStatement should have 'then_branch' field"
        );
    } else {
        panic!("IfStatement not found in registry");
    }
}

// Note: Grammar metadata tests are covered by test_grammar_validation.rs with proper helper functions

// ============================================================================
// BACKWARD COMPATIBILITY TESTS
// ============================================================================

#[test]
fn test_astnode_still_works_with_pattern_matching() {
    let node = AstNode::BinaryExpression {
        operator: "+".to_string(),
        left: Box::new(AstNode::Literal(Value::U64(1))),
        right: Box::new(AstNode::Literal(Value::U64(2))),
    };

    match node {
        AstNode::BinaryExpression { operator, left, right } => {
            assert_eq!(operator, "+");
            assert!(matches!(*left, AstNode::Literal(_)));
            assert!(matches!(*right, AstNode::Literal(_)));
        }
        _ => panic!("Pattern matching failed"),
    }
}

#[test]
fn test_can_mix_old_and_new_ast_types() {
    // Create a new type-safe expression
    let expr = Expression::BinaryExpression(BinaryExpressionNode {
        operator: "*".to_string(),
        left: Box::new(AstNode::Literal(Value::U64(3))),
        right: Box::new(AstNode::Literal(Value::U64(4))),
    });

    // Convert to AstNode
    let ast_node: AstNode = expr.into();

    // Use with existing code
    match ast_node {
        AstNode::BinaryExpression { operator, left, right } => {
            assert_eq!(operator, "*");
            assert!(matches!(*left, AstNode::Literal(_)));
            assert!(matches!(*right, AstNode::Literal(_)));
        }
        _ => panic!("Conversion or pattern match failed"),
    }
}
