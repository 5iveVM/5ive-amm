//! Tree-sitter grammar validation tests.

use five_dsl_compiler::ast::NODE_REGISTRY;
use std::collections::HashSet;

// Helper to get grammar rule string
fn get_grammar_rule_str(node: &five_dsl_compiler::ast::registry::NodeMetadata) -> String {
    node.grammar
        .as_ref()
        .map(|g| g.rule.clone())
        .unwrap_or_default()
}

// ============================================================================
// REGISTRY COMPLETENESS TESTS
// ============================================================================

#[test]
fn test_node_registry_has_48_nodes() {
    let registry = &*NODE_REGISTRY;
    assert_eq!(
        registry.nodes.len(),
        48,
        "Expected exactly 48 AST nodes in registry"
    );
}

#[test]
fn test_node_registry_categories_are_organized() {
    let registry = &*NODE_REGISTRY;

    let expression_nodes = registry.get_by_category("expression");
    let statement_nodes = registry.get_by_category("statement");
    let definition_nodes = registry.get_by_category("definition");

    assert!(
        !expression_nodes.is_empty(),
        "Should have expression nodes"
    );
    assert!(
        !statement_nodes.is_empty(),
        "Should have statement nodes"
    );
    assert!(
        !definition_nodes.is_empty(),
        "Should have definition nodes"
    );

    println!(
        "Node organization: {} expressions, {} statements, {} definitions",
        expression_nodes.len(),
        statement_nodes.len(),
        definition_nodes.len()
    );
}

#[test]
fn test_required_expression_nodes_exist() {
    let registry = &*NODE_REGISTRY;

    let required = vec![
        "BinaryExpression",
        "UnaryExpression",
        "FunctionCall",
        "MethodCall",
        "FieldAccess",
        "ArrayAccess",
        "Identifier",
        "Literal",
        "ArrayLiteral",
    ];

    for node_name in required {
        assert!(
            registry.get_node(node_name).is_some(),
            "Missing required node: {}",
            node_name
        );
    }
}

#[test]
fn test_required_statement_nodes_exist() {
    let registry = &*NODE_REGISTRY;

    let required = vec![
        "Assignment",
        "LetStatement",
        "IfStatement",
        "ReturnStatement",
        "WhileLoop",
        "RequireStatement",
    ];

    for node_name in required {
        assert!(
            registry.get_node(node_name).is_some(),
            "Missing required statement: {}",
            node_name
        );
    }
}

#[test]
fn test_required_definition_nodes_exist() {
    let registry = &*NODE_REGISTRY;

    let required = vec![
        "FieldDefinition",
        "InstructionDefinition",
        "AccountDefinition",
        "EventDefinition",
    ];

    for node_name in required {
        assert!(
            registry.get_node(node_name).is_some(),
            "Missing required definition: {}",
            node_name
        );
    }
}

// ============================================================================
// NODE FIELD VALIDATION TESTS
// ============================================================================

#[test]
fn test_all_field_definitions_are_valid() {
    let registry = &*NODE_REGISTRY;

    let mut field_count = 0;
    for (node_name, node) in registry.nodes.iter() {
        for (field_name, field) in node.fields.iter() {
            field_count += 1;

            // Field name should not be empty
            assert!(
                !field_name.is_empty(),
                "Node '{}' has empty field name",
                node_name
            );

            // Field type should not be empty
            assert!(
                !field.field_type.is_empty(),
                "Node '{}' field '{}' has empty type",
                node_name,
                field_name
            );
        }
    }

    println!("Total node fields: {}", field_count);
    assert!(field_count > 100, "Should have hundreds of fields across all nodes");
}

#[test]
fn test_binary_expression_fields() {
    let registry = &*NODE_REGISTRY;

    let binary = registry
        .get_node("BinaryExpression")
        .expect("BinaryExpression should exist");

    assert!(
        binary.fields.contains_key("operator"),
        "BinaryExpression missing 'operator' field"
    );
    assert!(
        binary.fields.contains_key("left"),
        "BinaryExpression missing 'left' field"
    );
    assert!(
        binary.fields.contains_key("right"),
        "BinaryExpression missing 'right' field"
    );
}

#[test]
fn test_if_statement_fields() {
    let registry = &*NODE_REGISTRY;

    let if_stmt = registry
        .get_node("IfStatement")
        .expect("IfStatement should exist");

    assert!(
        if_stmt.fields.contains_key("condition"),
        "IfStatement missing 'condition' field"
    );
    assert!(
        if_stmt.fields.contains_key("then_branch"),
        "IfStatement missing 'then_branch' field"
    );
    assert!(
        if_stmt.fields.contains_key("else_branch"),
        "IfStatement missing 'else_branch' field"
    );
}

#[test]
fn test_function_call_fields() {
    let registry = &*NODE_REGISTRY;

    let func_call = registry
        .get_node("FunctionCall")
        .expect("FunctionCall should exist");

    assert!(
        func_call.fields.contains_key("name"),
        "FunctionCall missing 'name' field"
    );
    assert!(
        func_call.fields.contains_key("args"),
        "FunctionCall missing 'args' field"
    );
}

// ============================================================================
// DOCUMENTATION COVERAGE TESTS
// ============================================================================

#[test]
fn test_nodes_have_documentation() {
    let registry = &*NODE_REGISTRY;

    let documented: usize = registry
        .nodes
        .values()
        .filter(|n| !n.doc.is_empty())
        .count();

    let total = registry.nodes.len();
    let coverage = (documented as f64 / total as f64) * 100.0;

    println!(
        "Documentation coverage: {}/{} ({:.1}%)",
        documented, total, coverage
    );

    // Should have at least 75% documented
    assert!(
        documented >= (total * 3 / 4),
        "Expected at least 75% documented nodes"
    );
}

#[test]
fn test_fields_have_documentation() {
    let registry = &*NODE_REGISTRY;

    let mut documented = 0;
    let mut total = 0;

    for node in registry.nodes.values() {
        for field in node.fields.values() {
            total += 1;
            if !field.doc.is_empty() {
                documented += 1;
            }
        }
    }

    let coverage = (documented as f64 / total as f64) * 100.0;

    println!(
        "Field documentation coverage: {}/{} ({:.1}%)",
        documented, total, coverage
    );

    // Fields are often less documented than nodes, but should still have good coverage
    assert!(
        documented >= (total / 2),
        "Expected at least 50% of fields documented"
    );
}

// ============================================================================
// GRAMMAR METADATA TESTS
// ============================================================================

#[test]
fn test_grammar_rules_exist_for_key_nodes() {
    let registry = &*NODE_REGISTRY;

    let key_nodes = vec![
        "BinaryExpression",
        "UnaryExpression",
        "IfStatement",
        "LetStatement",
        "FunctionCall",
    ];

    for node_name in key_nodes {
        let node = registry.get_node(node_name).unwrap_or_else(|| panic!("Node {} should exist", node_name));
        let rule = get_grammar_rule_str(node);

        println!("Node {}: grammar rule length = {}", node_name, rule.len());

        // Key nodes should have grammar rules
        assert!(
            !rule.is_empty(),
            "Key node {} should have a grammar rule",
            node_name
        );
    }
}

#[test]
fn test_grammar_rules_are_well_formed() {
    let registry = &*NODE_REGISTRY;

    for (name, node) in registry.nodes.iter() {
        let rule = get_grammar_rule_str(node);
        if rule.is_empty() {
            continue;
        }

        // Count parentheses and brackets for basic validation
        let open_parens = rule.matches('(').count();
        let close_parens = rule.matches(')').count();

        assert_eq!(
            open_parens, close_parens,
            "Node '{}' has mismatched parentheses",
            name
        );

        let open_brackets = rule.matches('[').count();
        let close_brackets = rule.matches(']').count();

        assert_eq!(
            open_brackets, close_brackets,
            "Node '{}' has mismatched brackets",
            name
        );
    }
}

// ============================================================================
// CONSISTENCY TESTS
// ============================================================================

#[test]
fn test_no_duplicate_node_names() {
    let registry = &*NODE_REGISTRY;

    let names: Vec<&str> = registry.nodes.keys().map(|s| s.as_str()).collect();
    let unique_names: HashSet<&str> = names.iter().copied().collect();

    assert_eq!(
        names.len(),
        unique_names.len(),
        "Found duplicate node names in registry"
    );
}

#[test]
fn test_node_categories_are_consistent() {
    let registry = &*NODE_REGISTRY;

    let valid_categories = vec![
        "expression",
        "statement",
        "definition",
        "structure",
        "type_node",
        "program",  // Top-level program node
    ];

    for (name, node) in registry.nodes.iter() {
        assert!(
            valid_categories.contains(&node.category.as_str()),
            "Node '{}' has invalid category: {}",
            name,
            node.category
        );
    }
}

#[test]
fn test_category_distribution() {
    let registry = &*NODE_REGISTRY;

    let expression_nodes = registry.get_by_category("expression");
    let statement_nodes = registry.get_by_category("statement");
    let definition_nodes = registry.get_by_category("definition");

    let expr_count = expression_nodes.len();
    let stmt_count = statement_nodes.len();
    let def_count = definition_nodes.len();

    println!(
        "Category distribution: {} expressions, {} statements, {} definitions (total: {})",
        expr_count,
        stmt_count,
        def_count,
        expr_count + stmt_count + def_count
    );

    // Verify reasonable distribution
    assert!(expr_count > 10, "Should have more than 10 expression nodes");
    assert!(stmt_count > 10, "Should have more than 10 statement nodes");
    assert!(def_count > 5, "Should have more than 5 definition nodes");
}

// ============================================================================
// COVERAGE COMPLETENESS TESTS
// ============================================================================

#[test]
fn test_expression_type_coverage() {
    let registry = &*NODE_REGISTRY;

    // Basic expressions
    assert!(registry.get_node("Identifier").is_some());
    assert!(registry.get_node("Literal").is_some());

    // Compound expressions
    assert!(registry.get_node("BinaryExpression").is_some());
    assert!(registry.get_node("UnaryExpression").is_some());

    // Function and method calls
    assert!(registry.get_node("FunctionCall").is_some());
    assert!(registry.get_node("MethodCall").is_some());

    // Access operations
    assert!(registry.get_node("FieldAccess").is_some());
    assert!(registry.get_node("ArrayAccess").is_some());

    // Literals
    assert!(registry.get_node("ArrayLiteral").is_some());
    assert!(registry.get_node("TupleLiteral").is_some());

    println!("✅ All critical expression types present");
}

#[test]
fn test_statement_type_coverage() {
    let registry = &*NODE_REGISTRY;

    // Variable operations
    assert!(registry.get_node("LetStatement").is_some());
    assert!(registry.get_node("Assignment").is_some());

    // Control flow
    assert!(registry.get_node("IfStatement").is_some());
    assert!(registry.get_node("WhileLoop").is_some());
    assert!(registry.get_node("ForLoop").is_some());
    assert!(registry.get_node("ReturnStatement").is_some());

    // Constraints and special
    assert!(registry.get_node("RequireStatement").is_some());

    println!("✅ All critical statement types present");
}

#[test]
fn test_definition_type_coverage() {
    let registry = &*NODE_REGISTRY;

    // Core definitions
    assert!(registry.get_node("FieldDefinition").is_some());
    assert!(registry.get_node("InstructionDefinition").is_some());

    // Account and event systems
    assert!(registry.get_node("AccountDefinition").is_some());
    assert!(registry.get_node("EventDefinition").is_some());

    // Error handling
    assert!(registry.get_node("ErrorTypeDefinition").is_some());

    // Interfaces
    assert!(registry.get_node("InterfaceDefinition").is_some());

    println!("✅ All critical definition types present");
}
