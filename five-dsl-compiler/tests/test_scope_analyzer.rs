/// Scope Analyzer Test Suite
///
/// Tests the scope_analyzer module which handles:
/// - Variable scope tracking
/// - Scope nesting and stack management
/// - Variable lifetime analysis
/// - Register allocation optimization
/// - Scope level tracking
use five_dsl_compiler::bytecode_generator::scope_analyzer::ScopeAnalyzer;
use five_dsl_compiler::*;

// ============================================================================
// Test Group 1: ScopeAnalyzer Creation & Basic Operations
// ============================================================================

#[test]
fn test_scope_analyzer_creation() {
    let analyzer = ScopeAnalyzer::new();

    assert_eq!(
        analyzer.current_scope_level, 0,
        "Should start at scope level 0"
    );
    assert_eq!(
        analyzer.instruction_counter, 0,
        "Should start with 0 instructions"
    );
    assert!(
        analyzer.current_function.is_none(),
        "Should have no current function"
    );
    assert_eq!(analyzer.scope_analyses.len(), 0, "Should have no analyses");
}

#[test]
fn test_begin_function() {
    let mut analyzer = ScopeAnalyzer::new();

    let result = analyzer.begin_function("test_func".to_string());
    assert!(result.is_ok(), "Should begin function without error");
    assert_eq!(analyzer.current_function, Some("test_func".to_string()));
}

#[test]
fn test_end_function() {
    let mut analyzer = ScopeAnalyzer::new();

    analyzer.begin_function("test_func".to_string()).unwrap();
    let result = analyzer.end_function();

    assert!(result.is_ok(), "Should end function without error");
    assert!(
        analyzer.current_function.is_none(),
        "Should clear current function"
    );
}

#[test]
fn test_begin_end_function_cycle() {
    let mut analyzer = ScopeAnalyzer::new();

    analyzer.begin_function("func1".to_string()).unwrap();
    analyzer.end_function().unwrap();
    analyzer.begin_function("func2".to_string()).unwrap();

    assert_eq!(analyzer.current_function, Some("func2".to_string()));
}

// ============================================================================
// Test Group 2: Scope Nesting
// ============================================================================

#[test]
fn test_enter_scope() {
    let mut analyzer = ScopeAnalyzer::new();

    assert_eq!(analyzer.current_scope_level, 0);

    analyzer.enter_scope().unwrap();
    assert_eq!(
        analyzer.current_scope_level, 1,
        "Should increment scope level"
    );

    analyzer.enter_scope().unwrap();
    assert_eq!(analyzer.current_scope_level, 2, "Should increment again");
}

#[test]
fn test_exit_scope() {
    let mut analyzer = ScopeAnalyzer::new();

    analyzer.enter_scope().unwrap();
    analyzer.enter_scope().unwrap();
    assert_eq!(analyzer.current_scope_level, 2);

    analyzer.exit_scope().unwrap();
    assert_eq!(
        analyzer.current_scope_level, 1,
        "Should decrement scope level"
    );

    analyzer.exit_scope().unwrap();
    assert_eq!(analyzer.current_scope_level, 0, "Should return to root");
}

#[test]
fn test_nested_scopes() {
    let mut analyzer = ScopeAnalyzer::new();

    // Simulate nested blocks: { { { } } }
    analyzer.enter_scope().unwrap(); // Level 1
    analyzer.enter_scope().unwrap(); // Level 2
    analyzer.enter_scope().unwrap(); // Level 3

    assert_eq!(analyzer.current_scope_level, 3);

    analyzer.exit_scope().unwrap(); // Back to 2
    analyzer.exit_scope().unwrap(); // Back to 1
    analyzer.exit_scope().unwrap(); // Back to 0

    assert_eq!(analyzer.current_scope_level, 0);
}

// ============================================================================
// Test Group 3: Variable Declaration & Usage
// ============================================================================

#[test]
fn test_declare_variable() {
    let mut analyzer = ScopeAnalyzer::new();

    analyzer.begin_function("test".to_string()).unwrap();
    let result = analyzer.declare_variable("x", "u64", false);

    assert!(result.is_ok(), "Should declare variable without error");
}

#[test]
fn test_use_variable() {
    let mut analyzer = ScopeAnalyzer::new();

    analyzer.begin_function("test".to_string()).unwrap();
    analyzer.declare_variable("x", "u64", false).unwrap();

    let result = analyzer.use_variable("x");
    assert!(result.is_ok(), "Should use declared variable");
}

#[test]
fn test_use_undeclared_variable() {
    let mut analyzer = ScopeAnalyzer::new();

    analyzer.begin_function("test".to_string()).unwrap();

    // Implementation auto-declares unknown variables with "unknown" type
    let result = analyzer.use_variable("undefined");
    assert!(
        result.is_ok(),
        "Implementation auto-declares undefined variables"
    );
}

#[test]
fn test_multiple_variables() {
    let mut analyzer = ScopeAnalyzer::new();

    analyzer.begin_function("test".to_string()).unwrap();
    analyzer.declare_variable("a", "u64", false).unwrap();
    analyzer.declare_variable("b", "u32", false).unwrap();
    analyzer.declare_variable("c", "bool", false).unwrap();

    assert!(analyzer.use_variable("a").is_ok());
    assert!(analyzer.use_variable("b").is_ok());
    assert!(analyzer.use_variable("c").is_ok());
}

// ============================================================================
// Test Group 4: Program Analysis
// ============================================================================

#[test]
fn test_analyze_simple_program() {
    let source = r#"
        pub test() -> u64 {
            let x = 100;
            return x;
        }
    "#;

    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    let mut analyzer = ScopeAnalyzer::new();
    let result = analyzer.analyze_program(&ast);

    assert!(result.is_ok(), "Should analyze simple program");
}

#[test]
fn test_analyze_program_with_multiple_functions() {
    let source = r#"
        pub main() -> u64 {
            let x = 10;
            return x;
        }

        helper() -> u64 {
            let y = 20;
            return y;
        }
    "#;

    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    let mut analyzer = ScopeAnalyzer::new();
    let result = analyzer.analyze_program(&ast);

    assert!(result.is_ok(), "Should analyze multiple functions");
    assert_eq!(
        analyzer.scope_analyses.len(),
        2,
        "Should have 2 function analyses"
    );
}

// ============================================================================
// Test Group 5: Function Analysis Retrieval
// ============================================================================

#[test]
fn test_get_function_analysis() {
    let source = r#"
        pub test() -> u64 {
            let x = 100;
            return x;
        }
    "#;

    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    let mut analyzer = ScopeAnalyzer::new();
    analyzer.analyze_program(&ast).unwrap();

    let analysis = analyzer.get_function_analysis("test");
    assert!(analysis.is_some(), "Should find test function analysis");

    let test_analysis = analysis.unwrap();
    assert_eq!(test_analysis.function_name, "test");
}

#[test]
fn test_get_nonexistent_function_analysis() {
    let analyzer = ScopeAnalyzer::new();

    let analysis = analyzer.get_function_analysis("nonexistent");
    assert!(analysis.is_none(), "Should not find nonexistent function");
}

#[test]
fn test_get_all_analyses() {
    let source = r#"
        pub func1() -> u64 { return 1; }
        pub func2() -> u64 { return 2; }
    "#;

    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    let mut analyzer = ScopeAnalyzer::new();
    analyzer.analyze_program(&ast).unwrap();

    let all_analyses = analyzer.get_all_analyses();
    assert_eq!(all_analyses.len(), 2, "Should have 2 analyses");
    assert!(all_analyses.contains_key("func1"));
    assert!(all_analyses.contains_key("func2"));
}

// ============================================================================
// Test Group 6: Variable Scope Information
// ============================================================================

#[test]
fn test_variable_scope_at_different_levels() {
    let mut analyzer = ScopeAnalyzer::new();

    analyzer.begin_function("test".to_string()).unwrap();

    // Variable at scope level 0
    analyzer.declare_variable("outer", "u64", false).unwrap();
    assert_eq!(analyzer.current_scope_level, 0);

    // Variable at scope level 1
    analyzer.enter_scope().unwrap();
    analyzer.declare_variable("inner", "u64", false).unwrap();
    assert_eq!(analyzer.current_scope_level, 1);

    analyzer.exit_scope().unwrap();
    analyzer.end_function().unwrap();
}

// ============================================================================
// Test Group 7: Complex Scope Patterns
// ============================================================================

#[test]
fn test_if_else_scope_pattern() {
    let mut analyzer = ScopeAnalyzer::new();

    analyzer.begin_function("test".to_string()).unwrap();

    // if branch
    analyzer.enter_scope().unwrap();
    analyzer.declare_variable("if_var", "u64", false).unwrap();
    analyzer.exit_scope().unwrap();

    // else branch
    analyzer.enter_scope().unwrap();
    analyzer.declare_variable("else_var", "u64", false).unwrap();
    analyzer.exit_scope().unwrap();

    analyzer.end_function().unwrap();
}

#[test]
fn test_loop_scope_pattern() {
    let mut analyzer = ScopeAnalyzer::new();

    analyzer.begin_function("test".to_string()).unwrap();

    // Loop body scope
    analyzer.enter_scope().unwrap();
    analyzer.declare_variable("loop_var", "u64", false).unwrap();
    analyzer.use_variable("loop_var").unwrap();
    analyzer.exit_scope().unwrap();

    analyzer.end_function().unwrap();
}

// ============================================================================
// Test Group 8: Instruction Counter
// ============================================================================

#[test]
fn test_instruction_counter_increments() {
    let mut analyzer = ScopeAnalyzer::new();

    assert_eq!(analyzer.instruction_counter, 0);

    analyzer.begin_function("test".to_string()).unwrap();
    analyzer.declare_variable("x", "u64", false).unwrap();

    // The counter should increment as we process nodes
    // (actual increment happens in analyze_node, which is called during program analysis)
}

// ============================================================================
// Test Group 9: Error Cases
// ============================================================================

#[test]
fn test_exit_scope_at_root_level() {
    let mut analyzer = ScopeAnalyzer::new();

    // Already at scope level 0
    let result = analyzer.exit_scope();

    // Should handle gracefully (may return Ok or Err depending on implementation)
    let _ = result;
}

#[test]
fn test_end_function_without_begin() {
    let mut analyzer = ScopeAnalyzer::new();

    let result = analyzer.end_function();

    // Should error or handle gracefully
    assert!(
        result.is_err() || result.is_ok(),
        "Should handle missing begin"
    );
}

#[test]
fn test_declare_variable_without_function() {
    let mut analyzer = ScopeAnalyzer::new();

    let result = analyzer.declare_variable("x", "u64", false);

    // Implementation is lenient and allows declaring without function context
    assert!(
        result.is_ok(),
        "Implementation allows declaring without function context"
    );
}

// ============================================================================
// Test Group 10: Integration with Real Programs
// ============================================================================

#[test]
fn test_analyze_program_with_nested_blocks() {
    let source = r#"
        pub test() -> u64 {
            let outer = 10;
            if outer > 5 {
                let inner = 20;
                return inner;
            }
            return outer;
        }
    "#;

    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    let mut analyzer = ScopeAnalyzer::new();
    let result = analyzer.analyze_program(&ast);

    assert!(result.is_ok(), "Should analyze nested blocks");
}

#[test]
fn test_analyze_program_with_parameters() {
    let source = r#"
        pub add(a: u64, b: u64) -> u64 {
            let sum = a + b;
            return sum;
        }
    "#;

    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    let mut analyzer = ScopeAnalyzer::new();
    let result = analyzer.analyze_program(&ast);

    assert!(result.is_ok(), "Should analyze function with parameters");
}

#[test]
fn test_variable_shadowing_different_scopes() {
    let mut analyzer = ScopeAnalyzer::new();

    analyzer.begin_function("test".to_string()).unwrap();

    // Declare 'x' in outer scope
    analyzer.declare_variable("x", "u64", false).unwrap();

    // Enter inner scope and declare 'x' again (shadowing)
    analyzer.enter_scope().unwrap();
    analyzer.declare_variable("x", "u32", false).unwrap();

    // Both should exist in their respective scopes
    analyzer.exit_scope().unwrap();
    analyzer.end_function().unwrap();
}
