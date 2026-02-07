//! Tests for parameter indexing regression prevention.
//! 
//! This test suite ensures that LOAD_PARAM correctly handles 1-based indexing
//! for function parameters.
//! 
//! Regression: Previously, data parameters used 0-based indexing, causing
//! LOAD_PARAM 0 to incorrectly try to access the function index instead of
//! the first parameter.

mod support;

use five_protocol::opcodes::*;
use five_vm_mito::{FIVE_VM_PROGRAM_ID, MitoVM, Result as VmResult, Value, stack::StackStorage};
use support::script_builder::ScriptBuilder;

fn execute_script(build: impl FnOnce(&mut ScriptBuilder)) -> VmResult<Option<Value>> {
    execute_script_with_input(&[], build)
}

fn execute_script_with_input(
    input: &[u8],
    build: impl FnOnce(&mut ScriptBuilder),
) -> VmResult<Option<Value>> {
    let mut builder = ScriptBuilder::new();
    build(&mut builder);
    let script = builder.build().expect("script assembly should succeed");
    let mut storage = StackStorage::new();
    MitoVM::execute_direct(&script, input, &[], &FIVE_VM_PROGRAM_ID, &mut storage)
}

mod parameter_indexing {
    use super::*;

    #[test]
    fn test_load_param_1_based_indexing() {
        // Test: LOAD_PARAM 1 should load the first parameter, not 0
        let result = execute_script(|script| {
            script
                .public_function("main", |f| {
                    f.push_u64(42).call("identity", 1).return_value();
                })
                .unwrap();
            script
                .private_function("identity", |f| {
                    // Load parameter at index 1 (the first parameter)
                    f.load_param(1).return_value();
                })
                .unwrap();
        })
        .unwrap();

        assert_eq!(result, Some(Value::U64(42)));
    }

    #[test]
    fn test_single_parameter_add() {
        // Test: Function with one parameter should work correctly
        let result = execute_script(|script| {
            script
                .public_function("main", |f| {
                    f.push_u64(100).call("add_one", 1).return_value();
                })
                .unwrap();
            script
                .private_function("add_one", |f| {
                    f.load_param(1)
                        .push_u64(1)
                        .emit(ADD)
                        .return_value();
                })
                .unwrap();
        })
        .unwrap();

        assert_eq!(result, Some(Value::U64(101)));
    }

    #[test]
    fn test_multiple_parameters_1_based_indexing() {
        // Test: Multiple parameters should use 1-based indexing: LOAD_PARAM 1, LOAD_PARAM 2, etc.
        let result = execute_script(|script| {
            script
                .public_function("main", |f| {
                    f.push_u64(10).push_u64(20).push_u64(30).call("sum_three", 3).return_value();
                })
                .unwrap();
            script
                .private_function("sum_three", |f| {
                    f.load_param(1)
                        .load_param(2)
                        .emit(ADD)
                        .load_param(3)
                        .emit(ADD)
                        .return_value();
                })
                .unwrap();
        })
        .unwrap();

        assert_eq!(result, Some(Value::U64(60)));
    }

    #[test]
    fn test_parameter_not_affected_by_function_index() {
        // Test: LOAD_PARAM 1 should never load function index, even if function index is 0
        // This specifically targets the regression where LOAD_PARAM 0 would load function index
        let result = execute_script(|script| {
            script
                .public_function("main", |f| {
                    f.push_u64(999).call("get_param", 1).return_value();
                })
                .unwrap();
            script
                .private_function("get_param", |f| {
                    // Explicitly load param 1 and return it
                    // If implementation regresses to using function index, this would fail
                    f.load_param(1).return_value();
                })
                .unwrap();
        })
        .unwrap();

        // Should return 999 (the parameter), not the function index
        assert_eq!(result, Some(Value::U64(999)));
    }

    #[test]
    fn test_add_amount_pattern() {
        // Test: Pattern from counter add_amount function
        // counter = counter + amount
        // where amount is a u64 parameter
        let result = execute_script(|script| {
            script
                .public_function("main", |f| {
                    // Push counter value (100) and amount parameter (25)
                    // Call add_to_counter with parameter count 1
                    f.push_u64(100).push_u64(25).call("add_to_counter", 1).return_value();
                })
                .unwrap();
            script
                .private_function("add_to_counter", |f| {
                    // amount is at parameter index 1
                    // counter is already on stack from caller
                    f.load_param(1).emit(ADD).return_value();
                })
                .unwrap();
        })
        .unwrap();

        assert_eq!(result, Some(Value::U64(125)));
    }

    #[test]
    fn test_parameter_consistency_across_calls() {
        // Test: Ensure parameter indexing is consistent across multiple function calls
        let result = execute_script(|script| {
            script
                .public_function("main", |f| {
                    f.push_u64(5)
                        .call("double", 1)
                        .push_u64(10)
                        .call("add", 2)
                        .return_value();
                })
                .unwrap();
            script
                .private_function("double", |f| {
                    f.load_param(1).load_param(1).emit(ADD).return_value();
                })
                .unwrap();
            script
                .private_function("add", |f| {
                    f.load_param(1).load_param(2).emit(ADD).return_value();
                })
                .unwrap();
        })
        .unwrap();

        // 5 doubled = 10, then 10 + 10 = 20
        assert_eq!(result, Some(Value::U64(20)));
    }
}
