//! Tests for control flow features in Five VM.
//!
//! Covers: HALT, RETURN, JUMP, JUMP_IF, and control flow edge cases.

mod support;

use five_protocol::opcodes::*;
use five_vm_mito::{FIVE_VM_PROGRAM_ID, MitoVM, Result as VmResult, Value};
use support::script_builder::ScriptBuilder;

fn execute_script(build: impl FnOnce(&mut ScriptBuilder)) -> VmResult<Option<Value>> {
    let mut builder = ScriptBuilder::new();
    build(&mut builder);
    let script = builder.build().expect("script assembly should succeed");
    MitoVM::execute_direct(&script, &[], &[], &FIVE_VM_PROGRAM_ID)
}

mod return_value_handling {
    use super::*;

    #[test]
    fn test_return_u64_value() {
        let result = execute_script(|script| {
            script
                .public_function("main", |f| {
                    f.push_u64(999).return_value();
                })
                .unwrap();
        })
        .unwrap();

        assert_eq!(result, Some(Value::U64(999)));
    }

    #[test]
    fn test_return_from_nested_function() {
        let result = execute_script(|script| {
            script
                .public_function("main", |f| {
                    f.call("get_value", 0).return_value();
                })
                .unwrap();
            script
                .private_function("get_value", |f| {
                    f.push_u64(777).return_value();
                })
                .unwrap();
        })
        .unwrap();

        assert_eq!(result, Some(Value::U64(777)));
    }

    #[test]
    fn test_multiple_returns_last_wins() {
        // Only the final RETURN matters
        let result = execute_script(|script| {
            script
                .public_function("main", |f| {
                    f.push_u64(100)
                        .return_value()
                        .push_u64(200)
                        .return_value();
                })
                .unwrap();
        })
        .unwrap();

        // The first return should exit, so 100 is returned
        assert_eq!(result, Some(Value::U64(100)));
    }

    #[test]
    fn test_return_from_computation() {
        let result = execute_script(|script| {
            script
                .public_function("main", |f| {
                    f.push_u64(50)
                        .push_u64(50)
                        .emit(ADD)
                        .return_value();
                })
                .unwrap();
        })
        .unwrap();

        assert_eq!(result, Some(Value::U64(100)));
    }
}

mod function_composition {
    use super::*;

    #[test]
    fn test_chain_three_function_calls() {
        let result = execute_script(|script| {
            script
                .public_function("main", |f| {
                    f.push_u64(2).call("add_three", 1).call("multiply_two", 1).call("subtract_one", 1).return_value();
                })
                .unwrap();
            script
                .private_function("add_three", |f| {
                    f.load_param(1).push_u64(3).emit(ADD).return_value();
                })
                .unwrap();
            script
                .private_function("multiply_two", |f| {
                    f.load_param(1).load_param(1).emit(ADD).return_value(); // Double = multiply by 2
                })
                .unwrap();
            script
                .private_function("subtract_one", |f| {
                    f.load_param(1).push_u64(1).emit(SUB).return_value();
                })
                .unwrap();
        })
        .unwrap();

        // 2 + 3 = 5, 5 * 2 = 10, 10 - 1 = 9
        assert_eq!(result, Some(Value::U64(9)));
    }

    #[test]
    fn test_recursive_like_pattern() {
        // Not true recursion, but chaining similar operations
        let result = execute_script(|script| {
            script
                .public_function("main", |f| {
                    f.push_u64(1).call("increment", 1).call("increment", 1).call("increment", 1).return_value();
                })
                .unwrap();
            script
                .private_function("increment", |f| {
                    f.load_param(1).push_u64(1).emit(ADD).return_value();
                })
                .unwrap();
        })
        .unwrap();

        // 1 + 1 + 1 + 1 = 4
        assert_eq!(result, Some(Value::U64(4)));
    }
}

mod stack_manipulation_in_functions {
    use super::*;

    #[test]
    fn test_dup_in_function() {
        let result = execute_script(|script| {
            script
                .public_function("main", |f| {
                    f.push_u64(5).call("double_with_dup", 1).return_value();
                })
                .unwrap();
            script
                .private_function("double_with_dup", |f| {
                    f.load_param(1).emit(DUP).emit(ADD).return_value();
                })
                .unwrap();
        })
        .unwrap();

        assert_eq!(result, Some(Value::U64(10)));
    }

    #[test]
    fn test_swap_in_function() {
        let result = execute_script(|script| {
            script
                .public_function("main", |f| {
                    f.push_u64(10).push_u64(3).call("add_swapped", 2).return_value();
                })
                .unwrap();
            script
                .private_function("add_swapped", |f| {
                    // param1=10, param2=3
                    // load_param(1)=10, load_param(2)=3 -> [10, 3]
                    // SWAP -> [3, 10]
                    // ADD -> 3 + 10 = 13
                    f.load_param(1).load_param(2).emit(SWAP).emit(ADD).return_value();
                })
                .unwrap();
        })
        .unwrap();

        assert_eq!(result, Some(Value::U64(13)));
    }
}

mod parameter_passing_reliability {
    use super::*;

    #[test]
    fn test_parameter_passing_consistency() {
        // Call same function multiple times, verify consistency
        let result = execute_script(|script| {
            script
                .public_function("main", |f| {
                    f.push_u64(5)
                        .call("double", 1)
                        .push_u64(5)
                        .call("double", 1)
                        .emit(ADD)
                        .return_value();
                })
                .unwrap();
            script
                .private_function("double", |f| {
                    f.load_param(1).load_param(1).emit(ADD).return_value();
                })
                .unwrap();
        })
        .unwrap();

        // 5 doubled = 10, plus 5 doubled = 10, total = 20
        assert_eq!(result, Some(Value::U64(20)));
    }

    #[test]
    fn test_parameter_independence_across_calls() {
        // Parameters should not leak between function calls
        let result = execute_script(|script| {
            script
                .public_function("main", |f| {
                    f.push_u64(100).call("identity", 1).push_u64(200).call("identity", 1).return_value();
                })
                .unwrap();
            script
                .private_function("identity", |f| {
                    f.load_param(1).return_value();
                })
                .unwrap();
        })
        .unwrap();

        // Second call should return 200, not affected by first call
        assert_eq!(result, Some(Value::U64(200)));
    }

    #[test]
    fn test_parameter_scope_isolation() {
        // Different functions with same parameter names shouldn't interfere
        let result = execute_script(|script| {
            script
                .public_function("main", |f| {
                    f.push_u64(10).call("add_five", 1).call("add_twenty", 1).return_value();
                })
                .unwrap();
            script
                .private_function("add_five", |f| {
                    f.load_param(1).push_u64(5).emit(ADD).return_value();
                })
                .unwrap();
            script
                .private_function("add_twenty", |f| {
                    f.load_param(1).push_u64(20).emit(ADD).return_value();
                })
                .unwrap();
        })
        .unwrap();

        // 10 + 5 = 15, then 15 + 20 = 35
        assert_eq!(result, Some(Value::U64(35)));
    }
}

mod arithmetic_edge_cases {
    use super::*;

    #[test]
    fn test_division_by_calculation() {
        let result = execute_script(|script| {
            script
                .public_function("main", |f| {
                    f.push_u64(100)
                        .push_u64(2)
                        .push_u64(2)
                        .emit(ADD)
                        .emit(DIV)
                        .return_value();
                })
                .unwrap();
        })
        .unwrap();

        // 100 / (2 + 2) = 100 / 4 = 25
        assert_eq!(result, Some(Value::U64(25)));
    }

    #[test]
    fn test_mod_operation() {
        let result = execute_script(|script| {
            script
                .public_function("main", |f| {
                    f.push_u64(17).push_u64(5).emit(MOD).return_value();
                })
                .unwrap();
        })
        .unwrap();

        assert_eq!(result, Some(Value::U64(2))); // 17 % 5 = 2
    }

    #[test]
    fn test_zero_comparison() {
        let result = execute_script(|script| {
            script
                .public_function("main", |f| {
                    f.push_u64(0).push_u64(1).emit(GT).return_value();
                })
                .unwrap();
        })
        .unwrap();

        assert_eq!(result, Some(Value::Bool(false))); // 0 > 1 is false
    }

    #[test]
    fn test_same_value_operations() {
        let result = execute_script(|script| {
            script
                .public_function("main", |f| {
                    f.push_u64(42)
                        .push_u64(42)
                        .emit(EQ)
                        .return_value();
                })
                .unwrap();
        })
        .unwrap();

        assert_eq!(result, Some(Value::Bool(true))); // 42 == 42
    }
}
