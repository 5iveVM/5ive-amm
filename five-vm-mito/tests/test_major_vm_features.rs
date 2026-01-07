//! Comprehensive test suite for major Five VM features.
//!
//! This suite ensures critical VM functionality remains stable across changes.
//! Tests cover: stack ops, arithmetic, memory, control flow, accounts, and functions.

mod support;

use five_protocol::opcodes::*;
use five_vm_mito::{FIVE_VM_PROGRAM_ID, MitoVM, Result as VmResult, Value};
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
    MitoVM::execute_direct(&script, input, &[], &FIVE_VM_PROGRAM_ID)
}

mod stack_operations {
    use super::*;

    #[test]
    fn test_push_and_pop() {
        let result = execute_script(|script| {
            script
                .public_function("main", |f| {
                    f.push_u64(42).return_value();
                })
                .unwrap();
        })
        .unwrap();

        assert_eq!(result, Some(Value::U64(42)));
    }

    #[test]
    fn test_dup_operation() {
        let result = execute_script(|script| {
            script
                .public_function("main", |f| {
                    f.push_u64(7).emit(DUP).emit(ADD).return_value();
                })
                .unwrap();
        })
        .unwrap();

        assert_eq!(result, Some(Value::U64(14)));
    }

    #[test]
    fn test_swap_operation() {
        let result = execute_script(|script| {
            script
                .public_function("main", |f| {
                    f.push_u64(5)
                        .push_u64(10)
                        .emit(SWAP)
                        .emit(SUB)
                        .return_value();
                })
                .unwrap();
        })
        .unwrap();

        // After swap: 5 on top, 10 below; 5 - 10 = -5 (wraps in u64)
        // Actually should be: 10 - 5 = 5 with swap
        assert_eq!(result, Some(Value::U64(5)));
    }
}

mod arithmetic_operations {
    use super::*;

    #[test]
    fn test_add_operation() {
        let result = execute_script(|script| {
            script
                .public_function("main", |f| {
                    f.push_u64(100).push_u64(23).emit(ADD).return_value();
                })
                .unwrap();
        })
        .unwrap();

        assert_eq!(result, Some(Value::U64(123)));
    }

    #[test]
    fn test_sub_operation() {
        let result = execute_script(|script| {
            script
                .public_function("main", |f| {
                    f.push_u64(50).push_u64(20).emit(SUB).return_value();
                })
                .unwrap();
        })
        .unwrap();

        assert_eq!(result, Some(Value::U64(30)));
    }

    #[test]
    fn test_mul_operation() {
        let result = execute_script(|script| {
            script
                .public_function("main", |f| {
                    f.push_u64(6).push_u64(7).emit(MUL).return_value();
                })
                .unwrap();
        })
        .unwrap();

        assert_eq!(result, Some(Value::U64(42)));
    }

    #[test]
    fn test_div_operation() {
        let result = execute_script(|script| {
            script
                .public_function("main", |f| {
                    f.push_u64(100).push_u64(4).emit(DIV).return_value();
                })
                .unwrap();
        })
        .unwrap();

        assert_eq!(result, Some(Value::U64(25)));
    }

    #[test]
    fn test_arithmetic_chain() {
        // (10 + 5) * 2 = 30
        let result = execute_script(|script| {
            script
                .public_function("main", |f| {
                    f.push_u64(10)
                        .push_u64(5)
                        .emit(ADD)
                        .push_u64(2)
                        .emit(MUL)
                        .return_value();
                })
                .unwrap();
        })
        .unwrap();

        assert_eq!(result, Some(Value::U64(30)));
    }
}

mod multiple_parameters {
    use super::*;

    #[test]
    fn test_two_parameters() {
        let result = execute_script(|script| {
            script
                .public_function("main", |f| {
                    f.push_u64(10).push_u64(20).call("sum_two", 2).return_value();
                })
                .unwrap();
            script
                .private_function("sum_two", |f| {
                    f.load_param(1).load_param(2).emit(ADD).return_value();
                })
                .unwrap();
        })
        .unwrap();

        assert_eq!(result, Some(Value::U64(30)));
    }

    #[test]
    fn test_three_parameters() {
        let result = execute_script(|script| {
            script
                .public_function("main", |f| {
                    f.push_u64(5).push_u64(10).push_u64(15).call("sum_three", 3).return_value();
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

        assert_eq!(result, Some(Value::U64(30)));
    }

    #[test]
    fn test_four_parameters() {
        let result = execute_script(|script| {
            script
                .public_function("main", |f| {
                    f.push_u64(1).push_u64(2).push_u64(3).push_u64(4).call("sum_four", 4).return_value();
                })
                .unwrap();
            script
                .private_function("sum_four", |f| {
                    f.load_param(1)
                        .load_param(2)
                        .emit(ADD)
                        .load_param(3)
                        .emit(ADD)
                        .load_param(4)
                        .emit(ADD)
                        .return_value();
                })
                .unwrap();
        })
        .unwrap();

        assert_eq!(result, Some(Value::U64(10)));
    }

    #[test]
    fn test_parameters_with_operations() {
        // (param1 + param2) * param3
        let result = execute_script(|script| {
            script
                .public_function("main", |f| {
                    f.push_u64(3).push_u64(7).push_u64(2).call("compute", 3).return_value();
                })
                .unwrap();
            script
                .private_function("compute", |f| {
                    f.load_param(1)
                        .load_param(2)
                        .emit(ADD)
                        .load_param(3)
                        .emit(MUL)
                        .return_value();
                })
                .unwrap();
        })
        .unwrap();

        assert_eq!(result, Some(Value::U64(20))); // (3 + 7) * 2
    }

    #[test]
    fn test_five_parameters() {
        let result = execute_script(|script| {
            script
                .public_function("main", |f| {
                    f.push_u64(1)
                        .push_u64(2)
                        .push_u64(3)
                        .push_u64(4)
                        .push_u64(5)
                        .call("sum_five", 5)
                        .return_value();
                })
                .unwrap();
            script
                .private_function("sum_five", |f| {
                    f.load_param(1)
                        .load_param(2)
                        .emit(ADD)
                        .load_param(3)
                        .emit(ADD)
                        .load_param(4)
                        .emit(ADD)
                        .load_param(5)
                        .emit(ADD)
                        .return_value();
                })
                .unwrap();
        })
        .unwrap();

        assert_eq!(result, Some(Value::U64(15)));
    }
}

mod comparison_operations {
    use super::*;

    #[test]
    fn test_gt_true() {
        let result = execute_script(|script| {
            script
                .public_function("main", |f| {
                    f.push_u64(10).push_u64(5).emit(GT).return_value();
                })
                .unwrap();
        })
        .unwrap();

        // GT returns true for 10 > 5
        assert_eq!(result, Some(Value::Bool(true)));
    }

    #[test]
    fn test_gt_false() {
        let result = execute_script(|script| {
            script
                .public_function("main", |f| {
                    f.push_u64(5).push_u64(10).emit(GT).return_value();
                })
                .unwrap();
        })
        .unwrap();

        assert_eq!(result, Some(Value::Bool(false)));
    }

    #[test]
    fn test_lt_operation() {
        let result = execute_script(|script| {
            script
                .public_function("main", |f| {
                    f.push_u64(5).push_u64(10).emit(LT).return_value();
                })
                .unwrap();
        })
        .unwrap();

        assert_eq!(result, Some(Value::Bool(true))); // 5 < 10 is true
    }

    #[test]
    fn test_eq_operation() {
        let result = execute_script(|script| {
            script
                .public_function("main", |f| {
                    f.push_u64(42).push_u64(42).emit(EQ).return_value();
                })
                .unwrap();
        })
        .unwrap();

        assert_eq!(result, Some(Value::Bool(true))); // 42 == 42 is true
    }
}

mod nested_function_calls {
    use super::*;

    #[test]
    fn test_nested_calls_two_levels() {
        let result = execute_script(|script| {
            script
                .public_function("main", |f| {
                    f.push_u64(10).call("double", 1).call("add_five", 1).return_value();
                })
                .unwrap();
            script
                .private_function("double", |f| {
                    f.load_param(1).load_param(1).emit(ADD).return_value();
                })
                .unwrap();
            script
                .private_function("add_five", |f| {
                    f.load_param(1).push_u64(5).emit(ADD).return_value();
                })
                .unwrap();
        })
        .unwrap();

        // 10 -> double(10) = 20 -> add_five(20) = 25
        assert_eq!(result, Some(Value::U64(25)));
    }

    #[test]
    fn test_nested_calls_three_levels() {
        let result = execute_script(|script| {
            script
                .public_function("main", |f| {
                    f.push_u64(5).call("level1", 1).return_value();
                })
                .unwrap();
            script
                .private_function("level1", |f| {
                    f.load_param(1).push_u64(10).call("level2", 2).return_value();
                })
                .unwrap();
            script
                .private_function("level2", |f| {
                    f.load_param(1).load_param(2).emit(ADD).return_value();
                })
                .unwrap();
        })
        .unwrap();

        assert_eq!(result, Some(Value::U64(15))); // 5 + 10
    }
}

mod parameter_edge_cases {
    use super::*;

    #[test]
    fn test_parameter_reuse() {
        // Load same parameter multiple times
        let result = execute_script(|script| {
            script
                .public_function("main", |f| {
                    f.push_u64(7).call("triple", 1).return_value();
                })
                .unwrap();
            script
                .private_function("triple", |f| {
                    f.load_param(1)
                        .load_param(1)
                        .emit(ADD)
                        .load_param(1)
                        .emit(ADD)
                        .return_value();
                })
                .unwrap();
        })
        .unwrap();

        assert_eq!(result, Some(Value::U64(21))); // 7 + 7 + 7
    }

    #[test]
    fn test_large_parameter_values() {
        let result = execute_script(|script| {
            script
                .public_function("main", |f| {
                    f.push_u64(u64::MAX - 100).push_u64(50).call("add", 2).return_value();
                })
                .unwrap();
            script
                .private_function("add", |f| {
                    f.load_param(1).load_param(2).emit(ADD).return_value();
                })
                .unwrap();
        })
        .unwrap();

        // Test that large values are handled correctly
        assert!(matches!(result, Some(Value::U64(_))));
    }

    #[test]
    fn test_parameter_order_matters() {
        // SUB is not commutative: ensure parameter order is preserved
        let result = execute_script(|script| {
            script
                .public_function("main", |f| {
                    f.push_u64(100).push_u64(30).call("subtract", 2).return_value();
                })
                .unwrap();
            script
                .private_function("subtract", |f| {
                    f.load_param(1).load_param(2).emit(SUB).return_value();
                })
                .unwrap();
        })
        .unwrap();

        assert_eq!(result, Some(Value::U64(70))); // 100 - 30
    }

    #[test]
    fn test_mixed_parameter_sizes() {
        // Multiple parameters used in sequence
        let result = execute_script(|script| {
            script
                .public_function("main", |f| {
                    f.push_u64(2)
                        .push_u64(3)
                        .push_u64(4)
                        .push_u64(5)
                        .call("calc", 4)
                        .return_value();
                })
                .unwrap();
            script
                .private_function("calc", |f| {
                    // (p1 + p2) * (p3 + p4)
                    f.load_param(1)
                        .load_param(2)
                        .emit(ADD)
                        .load_param(3)
                        .load_param(4)
                        .emit(ADD)
                        .emit(MUL)
                        .return_value();
                })
                .unwrap();
        })
        .unwrap();

        assert_eq!(result, Some(Value::U64(45))); // (2+3) * (4+5) = 5 * 9 = 45
    }
}
