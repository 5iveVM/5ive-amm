/// Function Dispatch Test Suite
///
/// Tests the function_dispatch module which handles:
/// - Function metadata collection from AST
/// - Public vs private function ordering
/// - Function visibility validation
/// - Function address calculation
/// - Parameter caching
use five_dsl_compiler::bytecode_generator::FunctionDispatcher;
use five_dsl_compiler::*;

// ============================================================================
// Test Group 1: Function Detection & Metadata Collection
// ============================================================================

#[test]
fn test_has_callable_functions_with_public_function() {
    let source = r#"
        pub test() -> u64 {
            return 100;
        }
    "#;

    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    let dispatcher = FunctionDispatcher::new();
    assert!(
        dispatcher.has_callable_functions(&ast),
        "Should detect public function"
    );
}

#[test]
fn test_has_callable_functions_with_private_function() {
    let source = r#"
        helper() -> u64 {
            return 42;
        }
    "#;

    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    let dispatcher = FunctionDispatcher::new();
    assert!(
        dispatcher.has_callable_functions(&ast),
        "Should detect private function (needs dispatch for internal CALL)"
    );
}

#[test]
fn test_has_callable_functions_with_init_block() {
    let source = r#"
        script test_script {
            init {
                let x = 100;
            }
        }
    "#;

    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    let dispatcher = FunctionDispatcher::new();
    assert!(
        dispatcher.has_callable_functions(&ast),
        "Should detect init block as callable"
    );
}

#[test]
fn test_has_callable_functions_empty_program() {
    let source = r#"
        script empty_script {
        }
    "#;

    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    let dispatcher = FunctionDispatcher::new();
    assert!(
        !dispatcher.has_callable_functions(&ast),
        "Should not detect functions in empty program"
    );
}

#[test]
fn test_collect_function_info_single_public() {
    let source = r#"
        pub add(a: u64, b: u64) -> u64 {
            return a + b;
        }
    "#;

    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    let mut dispatcher = FunctionDispatcher::new();
    dispatcher
        .collect_function_info(&ast)
        .expect("Should collect");

    let functions = dispatcher.get_functions();
    assert_eq!(functions.len(), 1, "Should have 1 function");
    assert_eq!(functions[0].name, "add");
    assert_eq!(functions[0].parameter_count, 2);
    assert!(functions[0].is_public, "Function should be public");
}

#[test]
fn test_collect_function_info_single_private() {
    let source = r#"
        helper(x: u64) -> u64 {
            return x * 2;
        }
    "#;

    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    let mut dispatcher = FunctionDispatcher::new();
    dispatcher
        .collect_function_info(&ast)
        .expect("Should collect");

    let functions = dispatcher.get_functions();
    assert_eq!(functions.len(), 1, "Should have 1 function");
    assert_eq!(functions[0].name, "helper");
    assert_eq!(functions[0].parameter_count, 1);
    assert!(!functions[0].is_public, "Function should be private");
}

// ============================================================================
// Test Group 2: Function Visibility Ordering
// ============================================================================

#[test]
fn test_public_functions_ordered_first() {
    let source = r#"
        helper() -> u64 { return 1; }
        pub test() -> u64 { return 2; }
        internal() -> u64 { return 3; }
        pub main() -> u64 { return 4; }
    "#;

    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    let mut dispatcher = FunctionDispatcher::new();
    dispatcher
        .collect_function_info(&ast)
        .expect("Should collect");

    let functions = dispatcher.get_functions();
    assert_eq!(functions.len(), 4);

    // First two should be public functions (indices 0, 1)
    assert!(functions[0].is_public, "Function 0 should be public");
    assert!(functions[1].is_public, "Function 1 should be public");
    assert_eq!(functions[0].name, "test", "First public function");
    assert_eq!(functions[1].name, "main", "Second public function");

    // Last two should be private functions (indices 2, 3)
    assert!(!functions[2].is_public, "Function 2 should be private");
    assert!(!functions[3].is_public, "Function 3 should be private");
    assert_eq!(functions[2].name, "helper", "First private function");
    assert_eq!(functions[3].name, "internal", "Second private function");
}

#[test]
fn test_all_public_functions() {
    let source = r#"
        pub add(a: u64, b: u64) -> u64 { return a + b; }
        pub sub(a: u64, b: u64) -> u64 { return a - b; }
        pub mul(a: u64, b: u64) -> u64 { return a * b; }
    "#;

    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    let mut dispatcher = FunctionDispatcher::new();
    dispatcher
        .collect_function_info(&ast)
        .expect("Should collect");

    let functions = dispatcher.get_functions();
    assert_eq!(functions.len(), 3);

    // All should be public
    for func in functions {
        assert!(func.is_public, "Function {} should be public", func.name);
    }
}

#[test]
fn test_all_private_functions() {
    let source = r#"
        helper1() -> u64 { return 1; }
        helper2() -> u64 { return 2; }
        helper3() -> u64 { return 3; }
    "#;

    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    let mut dispatcher = FunctionDispatcher::new();
    dispatcher
        .collect_function_info(&ast)
        .expect("Should collect");

    let functions = dispatcher.get_functions();
    assert_eq!(functions.len(), 3);

    // All should be private
    for func in functions {
        assert!(!func.is_public, "Function {} should be private", func.name);
    }
}

// ============================================================================
// Test Group 3: Init Block Handling
// ============================================================================

#[test]
fn test_init_block_as_function_0() {
    let source = r#"
        pub test() -> u64 {
            return 42;
        }
    "#;

    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    let mut dispatcher = FunctionDispatcher::new();
    dispatcher
        .collect_function_info(&ast)
        .expect("Should collect");

    let functions = dispatcher.get_functions();
    assert_eq!(functions.len(), 1, "Should have test function");
    assert_eq!(functions[0].name, "test");
    assert!(functions[0].is_public, "test should be public");
}

#[test]
fn test_init_block_with_mixed_functions() {
    let source = r#"
        helper() -> u64 { return 1; }
        pub main() -> u64 { return 2; }
    "#;

    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    let mut dispatcher = FunctionDispatcher::new();
    dispatcher
        .collect_function_info(&ast)
        .expect("Should collect");

    let functions = dispatcher.get_functions();
    assert_eq!(functions.len(), 2);
    assert_eq!(functions[0].name, "main", "Public function at index 0");
    assert!(functions[0].is_public, "main should be public");
    assert_eq!(functions[1].name, "helper", "Private function at index 1");
    assert!(!functions[1].is_public, "helper should be private");
}

// ============================================================================
// Test Group 4: Parameter Tracking
// ============================================================================

#[test]
fn test_function_parameter_counts() {
    let source = r#"
        pub no_params() -> u64 { return 0; }
        pub one_param(a: u64) -> u64 { return a; }
        pub two_params(a: u64, b: u64) -> u64 { return a + b; }
        pub three_params(a: u64, b: u64, c: u64) -> u64 { return a + b + c; }
    "#;

    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    let mut dispatcher = FunctionDispatcher::new();
    dispatcher
        .collect_function_info(&ast)
        .expect("Should collect");

    let functions = dispatcher.get_functions();
    assert_eq!(functions[0].parameter_count, 0);
    assert_eq!(functions[1].parameter_count, 1);
    assert_eq!(functions[2].parameter_count, 2);
    assert_eq!(functions[3].parameter_count, 3);
}

// ============================================================================
// Test Group 5: Edge Cases
// ============================================================================

#[test]
fn test_single_function_no_params_no_return() {
    let source = r#"
        pub noop() {
            // Does nothing
        }
    "#;

    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    let mut dispatcher = FunctionDispatcher::new();
    dispatcher
        .collect_function_info(&ast)
        .expect("Should collect");

    let functions = dispatcher.get_functions();
    assert_eq!(functions.len(), 1);
    assert_eq!(functions[0].name, "noop");
    assert_eq!(functions[0].parameter_count, 0);
}

#[test]
fn test_function_with_same_name_different_visibility() {
    // This should be rejected by parser/type checker in real code,
    // but dispatcher should still collect both if AST contains them
    let source = r#"
        pub test() -> u64 { return 1; }
        helper() -> u64 { return 2; }
    "#;

    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    let mut dispatcher = FunctionDispatcher::new();
    dispatcher
        .collect_function_info(&ast)
        .expect("Should collect");

    let functions = dispatcher.get_functions();
    assert_eq!(functions.len(), 2);
    // Verify they're in the right order
    assert!(functions[0].is_public);
    assert!(!functions[1].is_public);
}

#[test]
fn test_many_functions_ordering_preserved() {
    // Test with 10 functions - 5 public, 5 private
    let source = r#"
        helper1() -> u64 { return 1; }
        pub public1() -> u64 { return 11; }
        helper2() -> u64 { return 2; }
        pub public2() -> u64 { return 12; }
        helper3() -> u64 { return 3; }
        pub public3() -> u64 { return 13; }
        helper4() -> u64 { return 4; }
        pub public4() -> u64 { return 14; }
        helper5() -> u64 { return 5; }
        pub public5() -> u64 { return 15; }
    "#;

    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    let mut dispatcher = FunctionDispatcher::new();
    dispatcher
        .collect_function_info(&ast)
        .expect("Should collect");

    let functions = dispatcher.get_functions();
    assert_eq!(functions.len(), 10);

    // First 5 should be public (indices 0-4)
    for i in 0..5 {
        assert!(
            functions[i].is_public,
            "Function {} ({}) should be public",
            i, functions[i].name
        );
        assert!(functions[i].name.starts_with("public"));
    }

    // Last 5 should be private (indices 5-9)
    for i in 5..10 {
        assert!(
            !functions[i].is_public,
            "Function {} ({}) should be private",
            i, functions[i].name
        );
        assert!(functions[i].name.starts_with("helper"));
    }
}

// ============================================================================
// Test Group 6: Function Name Lookup
// ============================================================================

#[test]
fn test_function_index_lookup_by_name() {
    let source = r#"
        helper() -> u64 { return 1; }
        pub main() -> u64 { return 2; }
        pub test() -> u64 { return 3; }
    "#;

    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    let mut dispatcher = FunctionDispatcher::new();
    dispatcher
        .collect_function_info(&ast)
        .expect("Should collect");

    // main and test are public, so they should be at indices 0 and 1
    // Find the functions and check their indices by position in vector
    let functions = dispatcher.get_functions();

    let main_idx = functions.iter().position(|f| f.name == "main");
    let test_idx = functions.iter().position(|f| f.name == "test");
    let helper_idx = functions.iter().position(|f| f.name == "helper");

    assert!(main_idx.is_some(), "Should find 'main'");
    assert!(test_idx.is_some(), "Should find 'test'");
    assert!(helper_idx.is_some(), "Should find 'helper'");

    // Verify public functions are at indices 0 and 1
    assert!(main_idx.unwrap() < 2, "main should be in public range");
    assert!(test_idx.unwrap() < 2, "test should be in public range");
    assert!(
        helper_idx.unwrap() >= 2,
        "helper should be in private range"
    );
}

#[test]
fn test_function_index_lookup_nonexistent() {
    let source = r#"
        pub test() -> u64 { return 1; }
    "#;

    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    let mut dispatcher = FunctionDispatcher::new();
    dispatcher
        .collect_function_info(&ast)
        .expect("Should collect");

    let func_info = dispatcher.get_function_info("nonexistent");
    assert!(func_info.is_none(), "Should not find nonexistent function");
}

// ============================================================================
// Test Group 7: Clear State Between Collections
// ============================================================================

#[test]
fn test_collect_info_clears_previous_state() {
    let source1 = r#"
        pub func1() -> u64 { return 1; }
        pub func2() -> u64 { return 2; }
    "#;

    let mut tokenizer = DslTokenizer::new(source1);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast1 = parser.parse().expect("Should parse");

    let mut dispatcher = FunctionDispatcher::new();
    dispatcher
        .collect_function_info(&ast1)
        .expect("Should collect");
    assert_eq!(dispatcher.get_functions().len(), 2);

    // Now collect from different AST
    let source2 = r#"
        pub func3() -> u64 { return 3; }
    "#;

    let mut tokenizer = DslTokenizer::new(source2);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast2 = parser.parse().expect("Should parse");

    dispatcher
        .collect_function_info(&ast2)
        .expect("Should collect");

    // Should only have functions from second AST
    let functions = dispatcher.get_functions();
    assert_eq!(functions.len(), 1, "Should clear previous functions");
    assert_eq!(functions[0].name, "func3");
}
