/// Integration tests for multi-module compilation
///
/// Tests the full pipeline for multi-module Five DSL programs:
/// - Module discovery and dependency resolution
/// - Multi-module type checking with visibility enforcement
/// - Module merging with proper visibility handling
/// - Bytecode generation from merged modules
use five_dsl_compiler::{
    AstNode, BlockKind, DslParser, DslTokenizer, ModuleMerger, ModuleScope, ModuleSymbol, TypeNode,
    Visibility,
};

/// Helper to create a simple public function
fn create_public_function(name: &str) -> AstNode {
    AstNode::InstructionDefinition {
        name: name.to_string(),
        visibility: Visibility::Public,
        is_public: true,
        parameters: vec![],
        return_type: Some(Box::new(TypeNode::Primitive("bool".to_string()))),
        body: Box::new(AstNode::Block {
            statements: vec![AstNode::ReturnStatement {
                value: Some(Box::new(AstNode::Literal(five_protocol::Value::Bool(true)))),
            }],
            kind: BlockKind::Regular,
        }),
    }
}

/// Helper to create a simple internal function
fn create_internal_function(name: &str) -> AstNode {
    AstNode::InstructionDefinition {
        name: name.to_string(),
        visibility: Visibility::Internal,
        is_public: false,
        parameters: vec![],
        return_type: Some(Box::new(TypeNode::Primitive("u64".to_string()))),
        body: Box::new(AstNode::Block {
            statements: vec![AstNode::ReturnStatement {
                value: Some(Box::new(AstNode::Literal(five_protocol::Value::U64(42)))),
            }],
            kind: BlockKind::Regular,
        }),
    }
}

/// Helper to create a program AST
fn create_program(name: &str, functions: Vec<AstNode>) -> AstNode {
    AstNode::Program {
        program_name: name.to_string(),
        field_definitions: vec![],
        instruction_definitions: functions,
        event_definitions: vec![],
        account_definitions: vec![],
        interface_definitions: vec![],
        import_statements: vec![],
        init_block: None,
        constraints_block: None,
    }
}

#[test]
fn test_module_scope_with_public_functions() {
    let mut scope = ModuleScope::new("main".to_string());
    scope.add_module("helper".to_string());
    scope.register_import("main".to_string(), "helper".to_string());

    // Add a public function to helper
    scope.set_current_module("helper".to_string());
    let symbol = ModuleSymbol {
        type_info: TypeNode::Primitive("bool".to_string()),
        is_mutable: false,
        visibility: Visibility::Public,
    };
    scope.add_symbol_to_current("validate".to_string(), symbol);

    // Resolve from main
    let resolved = scope.resolve_symbol("validate", "main");
    assert!(resolved.is_some());

    let symbol = resolved.unwrap();
    assert_eq!(symbol.visibility, Visibility::Public);
    assert!(symbol.visibility.is_importable());
    assert!(symbol.visibility.is_on_chain_callable());
}

#[test]
fn test_module_scope_with_internal_functions() {
    let mut scope = ModuleScope::new("main".to_string());
    scope.add_module("helper".to_string());
    scope.register_import("main".to_string(), "helper".to_string());

    // Add internal functions to helper
    scope.set_current_module("helper".to_string());
    let symbol = ModuleSymbol {
        type_info: TypeNode::Primitive("u64".to_string()),
        is_mutable: false,
        visibility: Visibility::Internal,
    };
    scope.add_symbol_to_current("calculate_internal".to_string(), symbol);

    // Resolve from main - should work (Internal is importable)
    let resolved = scope.resolve_symbol("calculate_internal", "main");
    assert!(resolved.is_some());

    let symbol = resolved.unwrap();
    assert_eq!(symbol.visibility, Visibility::Internal);
    assert!(symbol.visibility.is_importable());
    assert!(!symbol.visibility.is_on_chain_callable());
}

#[test]
fn test_module_scope_visibility_barrier_private() {
    let mut scope = ModuleScope::new("main".to_string());
    scope.add_module("helper".to_string());
    scope.register_import("main".to_string(), "helper".to_string());

    // Add a private function to helper
    scope.set_current_module("helper".to_string());
    let symbol = ModuleSymbol {
        type_info: TypeNode::Primitive("u64".to_string()),
        is_mutable: false,
        visibility: Visibility::Private,
    };
    scope.add_symbol_to_current("private_impl".to_string(), symbol);

    // Try to resolve from main - should fail (Private is not importable)
    let resolved = scope.resolve_symbol("private_impl", "main");
    assert!(resolved.is_none());
}

#[test]
fn test_module_scope_without_import_no_access() {
    let mut scope = ModuleScope::new("main".to_string());
    scope.add_module("helper".to_string());
    // Note: NOT registering import

    scope.set_current_module("helper".to_string());
    let symbol = ModuleSymbol {
        type_info: TypeNode::Primitive("u64".to_string()),
        is_mutable: false,
        visibility: Visibility::Public,
    };
    scope.add_symbol_to_current("public_fn".to_string(), symbol);

    // Without import, should not resolve even if public
    let resolved = scope.resolve_symbol("public_fn", "main");
    assert!(resolved.is_none());
}

#[test]
fn test_module_merger_basic() {
    let mut merger = ModuleMerger::new();

    let main = create_program("main", vec![create_public_function("main_fn")]);
    let helper = create_program("helper", vec![create_public_function("helper_fn")]);

    merger.set_main_ast(main);
    merger.add_module("helper".to_string(), helper);

    let result = merger.merge();
    assert!(result.is_ok());

    if let AstNode::Program {
        instruction_definitions,
        ..
    } = result.unwrap()
    {
        assert_eq!(instruction_definitions.len(), 2);

        let names: Vec<String> = instruction_definitions
            .iter()
            .filter_map(|node| {
                if let AstNode::InstructionDefinition { name, .. } = node {
                    Some(name.clone())
                } else {
                    None
                }
            })
            .collect();

        assert!(names.contains(&"main_fn".to_string()));
        assert!(names.contains(&"helper::helper_fn".to_string()));
    }
}

#[test]
fn test_module_merger_respects_visibility() {
    let mut merger = ModuleMerger::new();

    let main = create_program("main", vec![create_public_function("main_fn")]);
    let helper = create_program(
        "helper",
        vec![
            create_public_function("public_helper"),
            create_internal_function("internal_helper"),
            AstNode::InstructionDefinition {
                name: "private_helper".to_string(),
                visibility: Visibility::Private,
                is_public: false,
                parameters: vec![],
                return_type: None,
                body: Box::new(AstNode::Block {
                    statements: vec![],
                    kind: BlockKind::Regular,
                }),
            },
        ],
    );

    merger.set_main_ast(main);
    merger.add_module("helper".to_string(), helper);

    let result = merger.merge();
    assert!(result.is_ok());

    if let AstNode::Program {
        instruction_definitions,
        ..
    } = result.unwrap()
    {
        // Should have: main_fn, public_helper, internal_helper (but NOT private_helper)
        assert_eq!(instruction_definitions.len(), 3);

        let names: Vec<String> = instruction_definitions
            .iter()
            .filter_map(|node| {
                if let AstNode::InstructionDefinition { name, .. } = node {
                    Some(name.clone())
                } else {
                    None
                }
            })
            .collect();

        assert!(names.contains(&"main_fn".to_string()));
        assert!(names.contains(&"helper::public_helper".to_string()));
        assert!(names.contains(&"helper::internal_helper".to_string()));
        assert!(!names.contains(&"helper::private_helper".to_string()));
    }
}

#[test]
fn test_module_merger_multiple_modules() {
    let mut merger = ModuleMerger::new();

    let main = create_program("main", vec![create_public_function("main_fn")]);
    let helper1 = create_program("helper1", vec![create_public_function("helper1_fn")]);
    let helper2 = create_program("helper2", vec![create_public_function("helper2_fn")]);

    merger.set_main_ast(main);
    merger.add_module("helper1".to_string(), helper1);
    merger.add_module("helper2".to_string(), helper2);

    let result = merger.merge();
    assert!(result.is_ok());

    if let AstNode::Program {
        instruction_definitions,
        ..
    } = result.unwrap()
    {
        assert_eq!(instruction_definitions.len(), 3);

        let names: Vec<String> = instruction_definitions
            .iter()
            .filter_map(|node| {
                if let AstNode::InstructionDefinition { name, .. } = node {
                    Some(name.clone())
                } else {
                    None
                }
            })
            .collect();

        assert!(names.contains(&"main_fn".to_string()));
        assert!(names.contains(&"helper1::helper1_fn".to_string()));
        assert!(names.contains(&"helper2::helper2_fn".to_string()));
    }
}

#[test]
fn test_multi_module_cross_module_resolution() {
    // Scenario: main module imports from helper1, which imports from helper2
    let mut scope = ModuleScope::new("main".to_string());
    scope.add_module("helper1".to_string());
    scope.add_module("helper2".to_string());
    scope.register_import("main".to_string(), "helper1".to_string());
    scope.register_import("helper1".to_string(), "helper2".to_string());

    // Add a public function to helper2
    scope.set_current_module("helper2".to_string());
    let symbol = ModuleSymbol {
        type_info: TypeNode::Primitive("u64".to_string()),
        is_mutable: false,
        visibility: Visibility::Public,
    };
    scope.add_symbol_to_current("deep_fn".to_string(), symbol);

    // Can resolve from helper1 (directly imports helper2)
    let resolved = scope.resolve_symbol("deep_fn", "helper1");
    assert!(resolved.is_some());

    // Cannot resolve from main (no direct import of helper2)
    let resolved = scope.resolve_symbol("deep_fn", "main");
    assert!(resolved.is_none());
}

#[test]
fn test_public_functions_are_on_chain_callable() {
    let mut scope = ModuleScope::new("main".to_string());

    scope.set_current_module("main".to_string());
    let symbol = ModuleSymbol {
        type_info: TypeNode::Primitive("bool".to_string()),
        is_mutable: false,
        visibility: Visibility::Public,
    };
    scope.add_symbol_to_current("entry_point".to_string(), symbol);

    assert!(scope.is_on_chain_callable("entry_point"));
}

#[test]
fn test_internal_functions_are_not_on_chain_callable() {
    let mut scope = ModuleScope::new("main".to_string());

    scope.set_current_module("main".to_string());
    let symbol = ModuleSymbol {
        type_info: TypeNode::Primitive("u64".to_string()),
        is_mutable: false,
        visibility: Visibility::Internal,
    };
    scope.add_symbol_to_current("helper_fn".to_string(), symbol);

    assert!(!scope.is_on_chain_callable("helper_fn"));
}

#[test]
fn test_parse_and_merge_multi_module() {
    // Parse two separate programs
    let main_code = r#"
script MainApp {
    pub fn transfer(amount: u64) -> bool {
        return true;
    }
}
"#;

    let helper_code = r#"
script Helper {
    pub fn validate(amount: u64) -> bool {
        return amount > 0;
    }

    fn calculate(amount: u64) -> u64 {
        return amount * 2;
    }
}
"#;

    // Parse main
    let mut tokenizer = DslTokenizer::new(main_code);
    let tokens = tokenizer.tokenize().expect("Should tokenize main");
    let mut parser = DslParser::new(tokens);
    let main_ast = parser.parse().expect("Should parse main");

    // Parse helper
    let mut tokenizer = DslTokenizer::new(helper_code);
    let tokens = tokenizer.tokenize().expect("Should tokenize helper");
    let mut parser = DslParser::new(tokens);
    let helper_ast = parser.parse().expect("Should parse helper");

    // Merge
    let mut merger = ModuleMerger::new();
    merger.set_main_ast(main_ast);
    merger.add_module("Helper".to_string(), helper_ast);

    let result = merger.merge();
    assert!(result.is_ok());

    if let AstNode::Program {
        instruction_definitions,
        ..
    } = result.unwrap()
    {
        // Should have: transfer (public), validate (public), calculate (internal)
        assert_eq!(instruction_definitions.len(), 3);

        let names: Vec<String> = instruction_definitions
            .iter()
            .filter_map(|node| {
                if let AstNode::InstructionDefinition { name, .. } = node {
                    Some(name.clone())
                } else {
                    None
                }
            })
            .collect();

        assert!(names.contains(&"transfer".to_string()));
        assert!(names.contains(&"Helper::validate".to_string()));
        assert!(names.contains(&"Helper::calculate".to_string()));
    }
}

#[test]
fn test_multi_module_symbol_table_integration() {
    let mut scope = ModuleScope::new("main".to_string());
    scope.add_module("math".to_string());
    scope.add_module("utils".to_string());

    // Register imports
    scope.register_import("main".to_string(), "math".to_string());
    scope.register_import("main".to_string(), "utils".to_string());

    // Add symbols to math module
    scope.set_current_module("math".to_string());
    let add_fn = ModuleSymbol {
        type_info: TypeNode::Primitive("u64".to_string()),
        is_mutable: false,
        visibility: Visibility::Public,
    };
    scope.add_symbol_to_current("add".to_string(), add_fn.clone());

    let multiply_fn = ModuleSymbol {
        type_info: TypeNode::Primitive("u64".to_string()),
        is_mutable: false,
        visibility: Visibility::Public,
    };
    scope.add_symbol_to_current("multiply".to_string(), multiply_fn.clone());

    // Add symbols to utils module
    scope.set_current_module("utils".to_string());
    let log_fn = ModuleSymbol {
        type_info: TypeNode::Primitive("bool".to_string()),
        is_mutable: false,
        visibility: Visibility::Internal,
    };
    scope.add_symbol_to_current("log".to_string(), log_fn.clone());

    // Verify resolution from main
    let add_resolved = scope.resolve_symbol("add", "main");
    assert!(add_resolved.is_some());

    let multiply_resolved = scope.resolve_symbol("multiply", "main");
    assert!(multiply_resolved.is_some());

    let log_resolved = scope.resolve_symbol("log", "main");
    assert!(log_resolved.is_some());

    // Check module counts
    let math_table = scope.get_module_table("math");
    assert!(math_table.is_some());
    assert_eq!(math_table.unwrap().len(), 2);

    let utils_table = scope.get_module_table("utils");
    assert!(utils_table.is_some());
    assert_eq!(utils_table.unwrap().len(), 1);
}

#[test]
fn test_namespace_collision_prevention() {
    // Test that two modules with the same function names don't collide
    // due to namespace qualification
    let mut merger = ModuleMerger::new();

    // Two modules with identical function names (would collide without namespace)
    let helper1 = create_program(
        "helper1",
        vec![
            create_public_function("calculate"),
            create_public_function("process"),
        ],
    );

    let helper2 = create_program(
        "helper2",
        vec![
            create_public_function("calculate"),
            create_public_function("process"),
        ],
    );

    let main = create_program("main", vec![create_public_function("main_fn")]);

    merger.set_main_ast(main);
    merger.add_module("helper1".to_string(), helper1);
    merger.add_module("helper2".to_string(), helper2);

    let result = merger.merge();
    assert!(result.is_ok());

    if let AstNode::Program {
        instruction_definitions,
        ..
    } = result.unwrap()
    {
        // Should have 5 functions: main_fn, helper1::calculate, helper1::process, helper2::calculate, helper2::process
        assert_eq!(instruction_definitions.len(), 5);

        let names: Vec<String> = instruction_definitions
            .iter()
            .filter_map(|node| {
                if let AstNode::InstructionDefinition { name, .. } = node {
                    Some(name.clone())
                } else {
                    None
                }
            })
            .collect();

        // Verify all functions exist with proper qualification
        assert!(names.contains(&"main_fn".to_string()));
        assert!(names.contains(&"helper1::calculate".to_string()));
        assert!(names.contains(&"helper1::process".to_string()));
        assert!(names.contains(&"helper2::calculate".to_string()));
        assert!(names.contains(&"helper2::process".to_string()));

        // Verify unqualified names don't exist (they would collide)
        assert!(!names.iter().any(|n| n == "calculate" || n == "process"));
    }
}

#[test]
fn test_backward_compatibility_flat_namespace() {
    // Test that disabling namespace qualification works (backward compatibility)
    let mut merger = ModuleMerger::new().with_namespaces(false); // Disable namespace qualification

    let helper = create_program(
        "helper",
        vec![
            create_public_function("helper_fn"),
            create_internal_function("internal_fn"),
        ],
    );

    let main = create_program("main", vec![create_public_function("main_fn")]);

    merger.set_main_ast(main);
    merger.add_module("helper".to_string(), helper);

    let result = merger.merge();
    assert!(result.is_ok());

    if let AstNode::Program {
        instruction_definitions,
        ..
    } = result.unwrap()
    {
        // Should have 3 functions (flat names without qualification)
        assert_eq!(instruction_definitions.len(), 3);

        let names: Vec<String> = instruction_definitions
            .iter()
            .filter_map(|node| {
                if let AstNode::InstructionDefinition { name, .. } = node {
                    Some(name.clone())
                } else {
                    None
                }
            })
            .collect();

        // Verify flat namespace names (no module prefix)
        assert!(names.contains(&"main_fn".to_string()));
        assert!(names.contains(&"helper_fn".to_string()));
        assert!(names.contains(&"internal_fn".to_string()));

        // Verify NO qualified names exist
        assert!(!names.iter().any(|n| n.contains("::")));
    }
}
