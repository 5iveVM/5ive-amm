//! Tests for local_base and parameter window isolation under the optimized header.

mod support;

use five_protocol::opcodes::*;
use five_vm_mito::{FIVE_VM_PROGRAM_ID, MitoVM, Value};
use support::script_builder::ScriptBuilder;

fn run_script(build: impl FnOnce(&mut ScriptBuilder)) -> Option<Value> {
    let mut builder = ScriptBuilder::new();
    build(&mut builder);
    let script = builder.build().expect("script assembly succeeds");
    MitoVM::execute_direct(&script, &[], &[], &FIVE_VM_PROGRAM_ID).unwrap()
}

#[test]
fn test_local_base_isolation_two_levels() {
    let result = run_script(|script| {
        script
            .public_function("main", |f| {
                f.push_u64(42)
                    .emit(SET_LOCAL)
                    .emit(0)
                    .call("callee", 0)
                    .emit(GET_LOCAL)
                    .emit(0)
                    .emit(ADD)
                    .return_value();
            })
            .unwrap();
        script
            .private_function("callee", |f| {
                f.push_u64(100)
                    .emit(SET_LOCAL)
                    .emit(0)
                    .emit(GET_LOCAL)
                    .emit(0)
                    .return_value();
            })
            .unwrap();
    });

    assert_eq!(result, Some(Value::U64(142)));
}

#[test]
fn test_local_base_restoration_after_return() {
    let result = run_script(|script| {
        script
            .public_function("outer", |f| {
                f.push_u64(10)
                    .emit(SET_LOCAL)
                    .emit(0)
                    .call("middle", 0)
                    .emit(GET_LOCAL)
                    .emit(0)
                    .emit(ADD)
                    .return_value();
            })
            .unwrap();
        script
            .private_function("middle", |f| {
                f.push_u64(20)
                    .emit(SET_LOCAL)
                    .emit(0)
                    .call("inner", 0)
                    .emit(GET_LOCAL)
                    .emit(0)
                    .emit(ADD)
                    .return_value();
            })
            .unwrap();
        script
            .private_function("inner", |f| {
                f.push_u64(30)
                    .emit(SET_LOCAL)
                    .emit(0)
                    .emit(GET_LOCAL)
                    .emit(0)
                    .return_value();
            })
            .unwrap();
    });

    assert_eq!(result, Some(Value::U64(60)));
}

#[test]
fn test_local_base_four_level_nesting() {
    let result = run_script(|script| {
        script
            .public_function("f0", |f| {
                f.push_u64(1)
                    .emit(SET_LOCAL)
                    .emit(0)
                    .call("f1", 0)
                    .emit(GET_LOCAL)
                    .emit(0)
                    .emit(ADD)
                    .return_value();
            })
            .unwrap();
        script
            .private_function("f1", |f| {
                f.push_u64(2)
                    .emit(SET_LOCAL)
                    .emit(0)
                    .call("f2", 0)
                    .emit(GET_LOCAL)
                    .emit(0)
                    .emit(ADD)
                    .return_value();
            })
            .unwrap();
        script
            .private_function("f2", |f| {
                f.push_u64(3)
                    .emit(SET_LOCAL)
                    .emit(0)
                    .call("f3", 0)
                    .emit(GET_LOCAL)
                    .emit(0)
                    .emit(ADD)
                    .return_value();
            })
            .unwrap();
        script
            .private_function("f3", |f| {
                f.push_u64(4)
                    .emit(SET_LOCAL)
                    .emit(0)
                    .emit(GET_LOCAL)
                    .emit(0)
                    .return_value();
            })
            .unwrap();
    });

    assert_eq!(result, Some(Value::U64(10)));
}

#[test]
fn test_parameter_window_isolation() {
    let result = run_script(|script| {
        script
            .public_function("main", |f| {
                f.push_u64(30)
                    .emit(SET_LOCAL)
                    .emit(0)
                    .push_u64(10)
                    .push_u64(20)
                    .call("add", 2)
                    .emit(GET_LOCAL)
                    .emit(0)
                    .emit(ADD)
                    .return_value();
            })
            .unwrap();
        script
            .private_function("add", |f| {
                f.load_param(1).load_param(2).emit(ADD).return_value();
            })
            .unwrap();
    });

    assert_eq!(result, Some(Value::U64(60)));
}

#[test]
fn test_multiple_locals_per_frame() {
    let result = run_script(|script| {
        script
            .public_function("main", |f| {
                f.push_u64(10)
                    .emit(SET_LOCAL)
                    .emit(0)
                    .push_u64(20)
                    .emit(SET_LOCAL)
                    .emit(1)
                    .push_u64(30)
                    .emit(SET_LOCAL)
                    .emit(2)
                    .call("helper", 0)
                    .emit(GET_LOCAL)
                    .emit(0)
                    .emit(GET_LOCAL)
                    .emit(1)
                    .emit(ADD)
                    .emit(GET_LOCAL)
                    .emit(2)
                    .emit(ADD)
                    .emit(ADD)
                    .return_value();
            })
            .unwrap();
        script
            .private_function("helper", |f| {
                f.push_u64(100)
                    .emit(SET_LOCAL)
                    .emit(0)
                    .push_u64(200)
                    .emit(SET_LOCAL)
                    .emit(1)
                    .emit(GET_LOCAL)
                    .emit(0)
                    .emit(GET_LOCAL)
                    .emit(1)
                    .emit(ADD)
                    .return_value();
            })
            .unwrap();
    });

    assert_eq!(result, Some(Value::U64(360)));
}
