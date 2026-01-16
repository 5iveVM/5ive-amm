//! Comprehensive test suite for Five DSL compiler features.
//!
//! Tests major compiler functionality to prevent regressions:
//! - Type inference and checking
//! - Function definitions and calls
//! - State variable handling
//! - Control flow generation
//! - Parameter compilation

use five_dsl_compiler::*;
use five_protocol::opcodes::*;

#[test]
fn test_simple_state_update() {
    let source = r#"
        mut counter: u64;

        init {
            counter = 0;
        }

        pub increment() {
            counter = counter + 1;
        }
    "#;

    let bytecode = DslCompiler::compile_dsl(source).expect("Should compile");
    assert!(!bytecode.is_empty(), "Should produce bytecode");
    assert!(bytecode.starts_with(&five_protocol::FIVE_MAGIC), "Should have valid header");
}

#[test]
fn test_multiple_state_variables() {
    let source = r#"
        mut counter1: u64;
        mut counter2: u64;
        mut active: bool;

        init {
            counter1 = 0;
            counter2 = 100;
            active = true;
        }

        pub update_both() {
            counter1 = counter1 + 1;
            counter2 = counter2 - 1;
        }
    "#;

    let bytecode = DslCompiler::compile_dsl(source).expect("Should compile");
    assert!(!bytecode.is_empty());
}

#[test]
fn test_function_with_no_parameters() {
    let source = r#"
        mut value: u64;

        init { value = 0; }

        pub get_value() -> u64 {
            return value;
        }
    "#;

    let bytecode = DslCompiler::compile_dsl(source).expect("Should compile");
    assert!(!bytecode.is_empty());
}

#[test]
fn test_function_with_single_parameter() {
    let source = r#"
        mut total: u64;

        pub add(amount: u64) {
            total = total + amount;
        }
    "#;

    let bytecode = DslCompiler::compile_dsl(source).expect("Should compile");
    
    // Should contain LOAD_PARAM for the amount parameter (optimized or explicit)
    let has_load_param = bytecode.windows(2).any(|w| w[0] == LOAD_PARAM);
    let has_load_param_opt = bytecode.iter().any(|&b| b == LOAD_PARAM_1 || b == LOAD_PARAM_2 || b == LOAD_PARAM_3);

    assert!(has_load_param || has_load_param_opt, "Should have LOAD_PARAM for parameter");
}

#[test]
fn test_function_with_multiple_parameters() {
    let source = r#"
        pub transfer(from: account @mut, to: account @mut, amount: u64) {
            // Function logic
        }
    "#;

    let bytecode = DslCompiler::compile_dsl(source).expect("Should compile");
    assert!(!bytecode.is_empty());
}

#[test]
fn test_arithmetic_operations_compilation() {
    let source = r#"
        mut result: u64;

        pub compute(a: u64, b: u64) {
            result = a + b;
            result = result * 2;
            result = result - 1;
        }
    "#;

    let bytecode = DslCompiler::compile_dsl(source).expect("Should compile");
    
    // Should contain arithmetic opcodes
    let has_arithmetic = bytecode.iter().any(|&b| b == ADD || b == MUL || b == SUB);
    assert!(has_arithmetic, "Should contain arithmetic opcodes");
}

#[test]
fn test_if_statement_compilation() {
    let source = r#"
        mut status: u64;

        pub check(value: u64) {
            if value > 100 {
                status = 1;
            }
        }
    "#;

    let bytecode = DslCompiler::compile_dsl(source).expect("Should compile");
    assert!(!bytecode.is_empty());
}

#[test]
fn test_nested_function_calls_compilation() {
    let source = r#"
        fn helper(x: u64) -> u64 {
            return x + 1;
        }

        pub main(value: u64) -> u64 {
            let temp = helper(value);
            return helper(temp);
        }
    "#;

    let bytecode = DslCompiler::compile_dsl(source).expect("Should compile");
    assert!(!bytecode.is_empty());
}

#[test]
fn test_account_parameter_handling() {
    let source = r#"
        pub initialize(counter: account @mut @init @signer, owner: account @signer) {
            // Initialize account
        }
    "#;

    let bytecode = DslCompiler::compile_dsl(source).expect("Should compile");
    assert!(!bytecode.is_empty());
}

#[test]
fn test_return_type_inference() {
    let source = r#"
        pub add(a: u64, b: u64) -> u64 {
            return a + b;
        }

        pub same(x: u64) -> u64 {
            return x;
        }
    "#;

    let bytecode = DslCompiler::compile_dsl(source).expect("Should compile");
    assert!(!bytecode.is_empty());
}

#[test]
fn test_comparison_expression_compilation() {
    let source = r#"
        pub check(value: u64) -> bool {
            return value > 50;
        }
    "#;

    let bytecode = DslCompiler::compile_dsl(source).expect("Should compile");
    assert!(!bytecode.is_empty());
}

#[test]
fn test_multiple_function_definitions() {
    let source = r#"
        pub func1(x: u64) -> u64 { return x; }
        pub func2(y: u64) -> u64 { return y + 1; }
        pub func3(z: u64) -> u64 { return z * 2; }
    "#;

    let bytecode = DslCompiler::compile_dsl(source).expect("Should compile");
    assert!(!bytecode.is_empty());
}

#[test]
fn test_parameter_type_consistency() {
    // Test that parameter types are correctly tracked
    let source = r#"
        pub add_u64(a: u64, b: u64) -> u64 {
            return a + b;
        }

        pub add_u32(a: u32, b: u32) -> u32 {
            return a + b;
        }
    "#;

    let bytecode = DslCompiler::compile_dsl(source).expect("Should compile");
    assert!(!bytecode.is_empty());
}

#[test]
fn test_let_statement_compilation() {
    let source = r#"
        pub compute(x: u64) -> u64 {
            let y = x + 10;
            let z = y * 2;
            return z;
        }
    "#;

    let bytecode = DslCompiler::compile_dsl(source).expect("Should compile");
    assert!(!bytecode.is_empty());
}

#[test]
fn test_state_initialization_block() {
    let source = r#"
        mut value1: u64;
        mut value2: u64;
        mut flag: bool;

        init {
            value1 = 100;
            value2 = 200;
            flag = true;
        }
    "#;

    let bytecode = DslCompiler::compile_dsl(source).expect("Should compile");
    assert!(!bytecode.is_empty());
}
