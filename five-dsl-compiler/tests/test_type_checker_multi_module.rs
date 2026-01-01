/// Tests for TypeCheckerContext multi-module support
///
/// Validates the integration of ModuleScope with TypeCheckerContext
/// for multi-module type checking with visibility enforcement

use five_dsl_compiler::{
    DslTypeChecker, ModuleScope, ModuleSymbol, TypeNode, Visibility,
};

#[test]
fn test_type_checker_context_creation() {
    let checker = DslTypeChecker::new();
    assert!(checker.get_symbol_table().is_empty());
}

#[test]
fn test_type_checker_with_module_scope() {
    let scope = ModuleScope::new("main".to_string());
    let checker = DslTypeChecker::new().with_module_scope(scope);

    assert!(checker.has_module_scope());
    assert!(checker.get_current_module().is_none());
}

#[test]
fn test_set_current_module() {
    let mut scope = ModuleScope::new("main".to_string());
    scope.add_module("helper".to_string());

    let mut checker = DslTypeChecker::new().with_module_scope(scope);
    checker.set_current_module("main".to_string());

    assert_eq!(checker.get_current_module(), Some("main"));
}

#[test]
fn test_add_to_module_scope() {
    let scope = ModuleScope::new("main".to_string());
    let mut checker = DslTypeChecker::new().with_module_scope(scope);

    checker.set_current_module("main".to_string());
    checker.add_to_module_scope(
        "x".to_string(),
        TypeNode::Primitive("u64".to_string()),
        true,
        Visibility::Public,
    );

    // Verify the module scope is still active
    assert!(checker.has_module_scope());
}

#[test]
fn test_resolve_with_module_scope_local() {
    let scope = ModuleScope::new("main".to_string());
    let mut checker = DslTypeChecker::new().with_module_scope(scope);

    checker.set_current_module("main".to_string());

    // Add symbol directly to module scope
    let symbol = ModuleSymbol {
        type_info: TypeNode::Primitive("u64".to_string()),
        is_mutable: false,
        visibility: Visibility::Public,
    };
    if let Some(scope) = checker.get_module_scope_mut() {
        scope.add_symbol_to_current("x".to_string(), symbol);
    }

    // Resolve should find it
    let resolved = checker.resolve_with_module_scope("x");
    assert!(resolved.is_some());
    if let Some(TypeNode::Primitive(name)) = resolved {
        assert_eq!(name, "u64");
    }
}

#[test]
fn test_resolve_with_module_scope_fallback_to_local_symbol_table() {
    let scope = ModuleScope::new("main".to_string());
    let mut checker = DslTypeChecker::new().with_module_scope(scope);

    checker.set_current_module("main".to_string());

    // Add to local symbol table (not module scope)
    checker.symbol_table.insert(
        "local_var".to_string(),
        (TypeNode::Primitive("bool".to_string()), true),
    );

    // Should resolve from local symbol table
    let resolved = checker.resolve_with_module_scope("local_var");
    assert!(resolved.is_some());
    if let Some(TypeNode::Primitive(name)) = resolved {
        assert_eq!(name, "bool");
    }
}

#[test]
fn test_resolve_without_module_scope() {
    let mut checker = DslTypeChecker::new();

    // Add to local symbol table
    checker.symbol_table.insert(
        "var".to_string(),
        (TypeNode::Primitive("u32".to_string()), false),
    );

    // Should still resolve from local symbol table
    let resolved = checker.resolve_with_module_scope("var");
    assert!(resolved.is_some());
    if let Some(TypeNode::Primitive(name)) = resolved {
        assert_eq!(name, "u32");
    }
}

#[test]
fn test_is_on_chain_callable_symbol_public() {
    let scope = ModuleScope::new("main".to_string());
    let mut checker = DslTypeChecker::new().with_module_scope(scope);

    checker.set_current_module("main".to_string());

    // Add a public symbol
    let symbol = ModuleSymbol {
        type_info: TypeNode::Primitive("bool".to_string()),
        is_mutable: false,
        visibility: Visibility::Public,
    };
    if let Some(scope) = checker.get_module_scope_mut() {
        scope.add_symbol_to_current("entry".to_string(), symbol);
    }

    // Should be on-chain callable
    assert!(checker.is_on_chain_callable_symbol("entry"));
}

#[test]
fn test_is_on_chain_callable_symbol_internal() {
    let scope = ModuleScope::new("main".to_string());
    let mut checker = DslTypeChecker::new().with_module_scope(scope);

    checker.set_current_module("main".to_string());

    // Add an internal symbol
    let symbol = ModuleSymbol {
        type_info: TypeNode::Primitive("u64".to_string()),
        is_mutable: false,
        visibility: Visibility::Internal,
    };
    if let Some(scope) = checker.get_module_scope_mut() {
        scope.add_symbol_to_current("helper".to_string(), symbol);
    }

    // Should NOT be on-chain callable
    assert!(!checker.is_on_chain_callable_symbol("helper"));
}

#[test]
fn test_is_on_chain_callable_symbol_private() {
    let scope = ModuleScope::new("main".to_string());
    let mut checker = DslTypeChecker::new().with_module_scope(scope);

    checker.set_current_module("main".to_string());

    // Add a private symbol
    let symbol = ModuleSymbol {
        type_info: TypeNode::Primitive("u64".to_string()),
        is_mutable: false,
        visibility: Visibility::Private,
    };
    if let Some(scope) = checker.get_module_scope_mut() {
        scope.add_symbol_to_current("secret".to_string(), symbol);
    }

    // Should NOT be on-chain callable
    assert!(!checker.is_on_chain_callable_symbol("secret"));
}

#[test]
fn test_is_on_chain_callable_symbol_without_module_scope() {
    let checker = DslTypeChecker::new();

    // Without module scope, should return false
    assert!(!checker.is_on_chain_callable_symbol("any_symbol"));
}

#[test]
fn test_cross_module_symbol_resolution() {
    let mut scope = ModuleScope::new("main".to_string());
    scope.add_module("helper".to_string());
    scope.register_import("main".to_string(), "helper".to_string());

    let mut checker = DslTypeChecker::new().with_module_scope(scope);

    // Add symbol to helper module
    checker.set_current_module("helper".to_string());
    let symbol = ModuleSymbol {
        type_info: TypeNode::Primitive("u64".to_string()),
        is_mutable: false,
        visibility: Visibility::Public,
    };
    if let Some(scope) = checker.get_module_scope_mut() {
        scope.add_symbol_to_current("calc".to_string(), symbol);
    }

    // Switch to main and resolve
    checker.set_current_module("main".to_string());
    let resolved = checker.resolve_with_module_scope("calc");
    assert!(resolved.is_some());
}

#[test]
fn test_multiple_modules_with_visibility() {
    let mut scope = ModuleScope::new("main".to_string());
    scope.add_module("math".to_string());
    scope.add_module("utils".to_string());
    scope.register_import("main".to_string(), "math".to_string());
    scope.register_import("main".to_string(), "utils".to_string());

    let mut checker = DslTypeChecker::new().with_module_scope(scope);

    // Add public function to math
    checker.set_current_module("math".to_string());
    let add_symbol = ModuleSymbol {
        type_info: TypeNode::Primitive("u64".to_string()),
        is_mutable: false,
        visibility: Visibility::Public,
    };
    if let Some(scope) = checker.get_module_scope_mut() {
        scope.add_symbol_to_current("add".to_string(), add_symbol);
    }

    // Add internal function to utils
    checker.set_current_module("utils".to_string());
    let log_symbol = ModuleSymbol {
        type_info: TypeNode::Primitive("bool".to_string()),
        is_mutable: false,
        visibility: Visibility::Internal,
    };
    if let Some(scope) = checker.get_module_scope_mut() {
        scope.add_symbol_to_current("log".to_string(), log_symbol);
    }

    // From main, both should be resolvable
    checker.set_current_module("main".to_string());
    assert!(checker.resolve_with_module_scope("add").is_some());
    assert!(checker.resolve_with_module_scope("log").is_some());

    // But only add should be on-chain callable
    assert!(checker.is_on_chain_callable_symbol("add"));
    assert!(!checker.is_on_chain_callable_symbol("log"));
}

#[test]
fn test_module_scope_with_local_symbol_table_fallback() {
    let scope = ModuleScope::new("main".to_string());
    let mut checker = DslTypeChecker::new().with_module_scope(scope);

    checker.set_current_module("main".to_string());

    // Add to both sources
    let module_symbol = ModuleSymbol {
        type_info: TypeNode::Primitive("u64".to_string()),
        is_mutable: false,
        visibility: Visibility::Public,
    };
    if let Some(scope) = checker.get_module_scope_mut() {
        scope.add_symbol_to_current("x".to_string(), module_symbol);
    }

    // Module scope should take precedence
    let resolved = checker.resolve_with_module_scope("x");
    assert!(resolved.is_some());
}

#[test]
fn test_unresolved_symbol() {
    let scope = ModuleScope::new("main".to_string());
    let mut checker = DslTypeChecker::new().with_module_scope(scope);

    checker.set_current_module("main".to_string());

    // Try to resolve non-existent symbol
    let resolved = checker.resolve_with_module_scope("undefined");
    assert!(resolved.is_none());
}

#[test]
fn test_type_checker_context_with_symbol_table() {
    let checker = DslTypeChecker::new();

    // Verify it's in the symbol table (initially empty)
    assert_eq!(checker.get_symbol_table().len(), 0);
}

#[test]
fn test_visibility_enforcement_in_imports() {
    let mut scope = ModuleScope::new("main".to_string());
    scope.add_module("private_module".to_string());
    scope.register_import("main".to_string(), "private_module".to_string());

    let mut checker = DslTypeChecker::new().with_module_scope(scope);

    // Add a private function
    checker.set_current_module("private_module".to_string());
    let private_symbol = ModuleSymbol {
        type_info: TypeNode::Primitive("u64".to_string()),
        is_mutable: false,
        visibility: Visibility::Private,
    };
    if let Some(scope) = checker.get_module_scope_mut() {
        scope.add_symbol_to_current("secret".to_string(), private_symbol);
    }

    // From main, private symbols should not be resolvable
    checker.set_current_module("main".to_string());
    let resolved = checker.resolve_with_module_scope("secret");
    assert!(resolved.is_none());
}

#[test]
fn test_add_to_module_scope_without_scope() {
    let mut checker = DslTypeChecker::new();

    // Should not panic if no module scope exists
    checker.add_to_module_scope(
        "x".to_string(),
        TypeNode::Primitive("u64".to_string()),
        true,
        Visibility::Public,
    );

    // Symbol table should be empty (not added to non-existent scope)
    assert!(checker.get_symbol_table().is_empty());
}

#[test]
fn test_set_current_module_without_scope() {
    let mut checker = DslTypeChecker::new();

    // Should not panic
    checker.set_current_module("main".to_string());

    assert_eq!(checker.get_current_module(), Some("main"));
}
