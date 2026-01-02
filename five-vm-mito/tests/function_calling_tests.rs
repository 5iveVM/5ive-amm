//! Function calling and parameter handling tests built on the optimized header format.
//!
//! Exercises call dispatch, parameter passing, locals, returns, and error paths using the
//! shared `ScriptBuilder` helpers so we can drop all legacy header fixtures.

mod support;

use five_protocol::opcodes::*;
use five_vm_mito::{FIVE_VM_PROGRAM_ID, MitoVM, Result as VmResult, VMError, Value};
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

mod basic_function_calls {
    use super::*;

    #[test]
    fn test_simple_function_call() {
        let result = execute_script(|script| {
            script
                .public_function("main", |f| {
                    f.call("const", 0).return_value();
                })
                .unwrap();
            script
                .private_function("const", |f| {
                    f.push_u64(42).return_value();
                })
                .unwrap();
        })
        .unwrap();

        assert_eq!(result, Some(Value::U64(42)));
    }

    #[test]
    fn test_function_with_parameters() {
        let result = execute_script(|script| {
            script
                .public_function("main", |f| {
                    f.push_u64(5).push_u64(3).call("add", 2).return_value();
                })
                .unwrap();
            script
                .private_function("add", |f| {
                    f.load_param(1).load_param(2).emit(ADD).return_value();
                })
                .unwrap();
        })
        .unwrap();

        assert_eq!(result, Some(Value::U64(8)));
    }

    #[test]
    fn test_nested_function_calls() {
        let result = execute_script(|script| {
            script
                .public_function("main", |f| {
                    f.call("f1", 0).return_value();
                })
                .unwrap();
            script
                .private_function("f1", |f| {
                    f.call("f2", 0).push_u64(50).emit(ADD).return_value();
                })
                .unwrap();
            script
                .private_function("f2", |f| {
                    f.push_u64(10).return_value();
                })
                .unwrap();
        })
        .unwrap();

        // f2 = 10, f1 = 60, main = 60
        assert_eq!(result, Some(Value::U64(60)));
    }

    #[test]
    fn test_function_call_stack_management() {
        let result = execute_script(|script| {
            script
                .public_function("main", |f| {
                    f.call("leaf", 0).call("leaf", 0).emit(ADD).return_value();
                })
                .unwrap();
            script
                .private_function("leaf", |f| {
                    f.push_u64(21).return_value();
                })
                .unwrap();
        })
        .unwrap();

        assert_eq!(result, Some(Value::U64(42)));
    }
}

mod parameter_handling {
    use super::*;

    #[test]
    fn test_load_param_basic() {
        let result = execute_script(|script| {
            script
                .public_function("main", |f| {
                    f.push_u64(100)
                        .push_u64(50)
                        .push_u64(10)
                        .call("accumulate", 3)
                        .return_value();
                })
                .unwrap();
            script
                .private_function("accumulate", |f| {
                    f.load_param(1)
                        .load_param(2)
                        .load_param(3)
                        .emit(ADD)
                        .emit(ADD)
                        .return_value();
                })
                .unwrap();
        })
        .unwrap();

        assert_eq!(result, Some(Value::U64(160)));
    }

    #[test]
    fn test_param_bounds_checking() {
        let err = execute_script(|script| {
            script
                .public_function("main", |f| {
                    f.push_u64(66).call("single_param", 1).return_value();
                })
                .unwrap();
            script
                .private_function("single_param", |f| {
                    f.load_param(2).return_value();
                })
                .unwrap();
        })
        .unwrap_err();

        assert!(
            matches!(err, VMError::InvalidParameter),
            "unexpected error: {:?}",
            err
        );
    }

    #[test]
    fn test_parameter_types() {
        let result = execute_script(|script| {
            script
                .public_function("main", |f| {
                    f.push_u64(42)
                        .push_u64(1)
                        .push_u64(255)
                        .push_u64(768)
                        .call("combine", 4)
                        .return_value();
                })
                .unwrap();
            script
                .private_function("combine", |f| {
                    f.load_param(1)
                        .load_param(2)
                        .load_param(3)
                        .load_param(4)
                        .emit(ADD)
                        .emit(ADD)
                        .emit(ADD)
                        .return_value();
                })
                .unwrap();
        });

        match result {
            Ok(value) => assert_eq!(value, Some(Value::U64(1066))),
            Err(err) => panic!("parameter type handling failed: {:?}", err),
        }
    }
}

mod local_variables {
    use super::*;

    #[test]
    fn test_local_variable_storage() {
        let result = execute_script(|script| {
            script
                .public_function("main", |f| {
                    f.push_u64(100).call("locals", 1).return_value();
                })
                .unwrap();
            script
                .private_function("locals", |f| {
                    f.emit(ALLOC_LOCALS)
                        .emit(1)
                        .load_param(1)
                        .push_u64(10)
                        .emit(ADD)
                        .emit(SET_LOCAL)
                        .emit(0)
                        .push_u64(5)
                        .emit(GET_LOCAL)
                        .emit(0)
                        .emit(MUL)
                        .return_value();
                })
                .unwrap();
        })
        .unwrap();

        assert_eq!(result, Some(Value::U64(550)));
    }

    #[test]
    fn test_local_variable_scope() {
        let result = execute_script(|script| {
            script
                .public_function("main", |f| {
                    f.call("first", 0)
                        .call("second", 0)
                        .emit(ADD)
                        .return_value();
                })
                .unwrap();
            script
                .private_function("first", |f| {
                    f.emit(ALLOC_LOCALS)
                        .emit(1)
                        .push_u64(66)
                        .emit(SET_LOCAL)
                        .emit(0)
                        .emit(GET_LOCAL)
                        .emit(0)
                        .return_value();
                })
                .unwrap();
            script
                .private_function("second", |f| {
                    f.emit(ALLOC_LOCALS)
                        .emit(1)
                        .push_u64(24)
                        .emit(SET_LOCAL)
                        .emit(0)
                        .emit(GET_LOCAL)
                        .emit(0)
                        .return_value();
                })
                .unwrap();
        })
        .unwrap();

        assert_eq!(result, Some(Value::U64(90)));
    }
}

mod return_handling {
    use super::*;

    #[test]
    fn test_return_value_propagation() {
        let result = execute_script(|script| {
            script
                .public_function("main", |f| {
                    f.call("mid", 0).push_u64(100).emit(ADD).return_value();
                })
                .unwrap();
            script
                .private_function("mid", |f| {
                    f.call("leaf", 0).push_u64(50).emit(ADD).return_value();
                })
                .unwrap();
            script
                .private_function("leaf", |f| {
                    f.push_u64(10).return_value();
                })
                .unwrap();
        })
        .unwrap();

        assert_eq!(result, Some(Value::U64(160)));
    }

    #[test]
    fn test_early_return() {
        let result = execute_script(|script| {
            script
                .public_function("main", |f| {
                    f.push_u64(5).call("branch", 1).return_value();
                })
                .unwrap();
            script
                .private_function("branch", |f| {
                    f.load_param(1).push_u64(10).emit(NEQ).jump_if("return_42");
                    f.push_u64(255).return_value();
                    f.label("return_42");
                    f.push_u64(42).return_value();
                })
                .unwrap();
        })
        .unwrap();

        assert_eq!(result, Some(Value::U64(42)));
    }
}

mod error_handling {
    use super::*;

    #[test]
    fn test_call_depth_limit() {
        let err = execute_script(|script| {
            script
                .public_function("main", |f| {
                    f.call("recur", 0).return_value();
                })
                .unwrap();
            script
                .private_function("recur", |f| {
                    f.call("recur", 0).return_value();
                })
                .unwrap();
        })
        .unwrap_err();

        assert!(matches!(err, VMError::CallStackOverflow));
    }

    #[test]
    fn test_invalid_function_index() {
        let err = execute_script(|script| {
            script
                .public_function("main", |f| {
                    f.call_raw(0, 0x7FFF);
                    f.return_value();
                })
                .unwrap();
        })
        .unwrap_err();

        assert!(
            matches!(
                err,
                VMError::InvalidInstructionPointer
                    | VMError::InvalidInstruction
                    | VMError::InvalidFunctionIndex
            ),
            "unexpected error: {:?}",
            err
        );
    }

    #[test]
    fn test_function_visibility_enforcement() {
        let script = {
            let mut builder = ScriptBuilder::new();
            builder
                .public_function("main", |f| {
                    f.push_u64(42).return_value();
                })
                .unwrap();
            builder
                .private_function("secret", |f| {
                    f.push_u64(99).return_value();
                })
                .unwrap();
            builder.build().expect("script")
        };

        let public_result = MitoVM::execute_direct(&script, &[0], &[], &FIVE_VM_PROGRAM_ID).unwrap();
        assert_eq!(public_result, Some(Value::U64(42)));

        let err = MitoVM::execute_direct(&script, &[1], &[], &FIVE_VM_PROGRAM_ID).unwrap_err();
        assert!(
            matches!(err, VMError::FunctionVisibilityViolation { .. }),
            "Expected FunctionVisibilityViolation, got: {:?}",
            err
        );
    }
}
