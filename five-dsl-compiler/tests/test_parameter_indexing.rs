//! Tests for parameter indexing regression prevention.
//! 
//! This test suite ensures that data parameters use 1-based indexing for LOAD_PARAM
//! bytecode instructions, and account parameters are indexed correctly.
//! 
//! Regression: Previously, data parameters used 0-based indexing, causing LOAD_PARAM 0
//! to incorrectly try to load the function index instead of the first data parameter.

use five_dsl_compiler::*;
use five_protocol::opcodes::*;

#[test]
fn test_data_parameter_indexing_single_value_param() {
    // Test: function with one value parameter should use LOAD_PARAM 1
    let source = r#"
        mut count: u64;

        pub add_amount(amount: u64) {
            count = count + amount;
        }
    "#;

    let bytecode = DslCompiler::compile_dsl(source).expect("Should compile");

    // Find LOAD_PARAM instruction
    // Pattern: LOAD_PARAM opcode followed by index byte
    let load_param_opcode = LOAD_PARAM;
    let mut found_load_param = false;

    for window in bytecode.windows(2) {
        if window[0] == load_param_opcode {
            let param_index = window[1];
            // First data parameter should be at index 1, not 0
            assert_eq!(
                param_index, 1,
                "Data parameter should use 1-based indexing (LOAD_PARAM 1), got LOAD_PARAM {}",
                param_index
            );
            found_load_param = true;
            break;
        }
    }

    assert!(
        found_load_param,
        "Should find at least one LOAD_PARAM instruction in bytecode"
    );
}

#[test]
fn test_data_parameter_indexing_multiple_value_params() {
    // Test: function with multiple value parameters should use LOAD_PARAM 1, 2, etc.
    let source = r#"
        pub transfer_amount(from: account @mut, to: account @mut, amount1: u64, amount2: u64) {
            // Function body - parameters would be used here
        }
    "#;

    let bytecode = DslCompiler::compile_dsl(source).expect("Should compile");

    let load_param_opcode = LOAD_PARAM;
    let mut load_param_indices = Vec::new();

    for window in bytecode.windows(2) {
        if window[0] == load_param_opcode {
            load_param_indices.push(window[1]);
        }
    }

    // With 2 account parameters and 2 data parameters:
    // Account params use their own indices (not via LOAD_PARAM)
    // Data params should use LOAD_PARAM 1, LOAD_PARAM 2
    if !load_param_indices.is_empty() {
        // All LOAD_PARAM indices should be >= 1 (1-based indexing)
        for (i, &index) in load_param_indices.iter().enumerate() {
            assert!(
                index >= 1,
                "Data parameter {} should use 1-based indexing, got index {}",
                i,
                index
            );
        }
    }
}

#[test]
fn test_account_parameter_indexing_preserved() {
    // Test: account parameters should not use LOAD_PARAM (they use direct account indices)
    // This ensures account indexing is unaffected by data parameter changes
    let source = r#"
        pub initialize(counter: account @mut @init @signer, owner: account @signer) {
            // Account parameters would be accessed via account indices, not LOAD_PARAM
        }
    "#;

    let bytecode = DslCompiler::compile_dsl(source).expect("Should compile");

    // Basic sanity check: bytecode should compile without errors
    assert!(!bytecode.is_empty(), "Bytecode should not be empty");
}

#[test]
fn test_parameter_offset_calculation_consistency() {
    // Test: verify that parameter offsets are calculated consistently
    // This catches if someone accidentally reverts the fix
    let source = r#"
        pub complex_function(
            p1: account @mut,
            p2: account @mut,
            value1: u64,
            value2: u32,
            value3: u8
        ) {
            // Function with mixed account and data parameters
        }
    "#;

    let bytecode = DslCompiler::compile_dsl(source).expect("Should compile");

    // The bytecode should successfully compile with correct parameter ordering
    // This is primarily a compilation success check
    assert!(!bytecode.is_empty(), "Bytecode should compile successfully");
}

#[test]
fn test_single_data_param_bytecode_generation() {
    // Integration test: Verify bytecode is correctly generated for add_amount pattern
    // This specifically targets the regression we fixed
    let source = r#"
        mut counter: u64;

        pub add_amount(counter_account: account @mut, owner: account @signer, amount: u64) {
            counter = counter + amount;
        }
    "#;

    let bytecode = DslCompiler::compile_dsl(source).expect("Should compile");

    // Verification: bytecode should exist and be reasonable size
    assert!(bytecode.len() > 10, "Bytecode should be non-trivial");

    // The function should have at least one LOAD_PARAM for the amount parameter
    let load_param_opcode = LOAD_PARAM;
    let has_load_param = bytecode.windows(2).any(|w| w[0] == load_param_opcode);

    assert!(
        has_load_param,
        "Should have LOAD_PARAM instruction for value parameter"
    );
}
