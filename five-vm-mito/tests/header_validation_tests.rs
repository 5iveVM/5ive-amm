//! Header validation tests for the optimized 10-byte header (V3).

mod support;

use five_protocol::{opcodes::*, Value};
use five_vm_mito::{MitoVM, VMError};
use support::script_builder::ScriptBuilder;

fn run_script(build: impl FnOnce(&mut ScriptBuilder)) -> Vec<u8> {
    ScriptBuilder::build_script(build)
}

#[test]
fn valid_header_executes() {
    let script = run_script(|script| {
        script
            .public_function("main", |f| {
                f.push_u64(42).return_value();
            })
            .unwrap();
    });

    let result = MitoVM::execute_direct(&script, &[], &[]).unwrap();
    assert_eq!(result, Some(Value::U64(42)));
}

#[test]
fn invalid_magic_fails() {
    let mut script = run_script(|script| {
        script
            .public_function("main", |f| {
                f.emit(HALT);
            })
            .unwrap();
    });
    script[0..4].copy_from_slice(b"FAKE");

    let err = MitoVM::execute_direct(&script, &[], &[]).unwrap_err();
    assert!(matches!(err, VMError::InvalidScript));
}

#[test]
fn public_count_exceeds_total_fails() {
    let mut script = run_script(|script| {
        script
            .public_function("main", |f| {
                f.emit(HALT);
            })
            .unwrap();
        script
            .private_function("helper", |f| {
                f.emit(HALT);
            })
            .unwrap();
    });
    script[8] = 3; // public count (at index 8 in V3 header)
    script[9] = 1; // total count (at index 9 in V3 header)

    let err = MitoVM::execute_direct(&script, &[], &[]).unwrap_err();
    assert!(matches!(err, VMError::InvalidScript));
}

#[test]
fn total_count_too_large_fails() {
    let mut script = run_script(|script| {
        script
            .public_function("main", |f| {
                f.emit(HALT);
            })
            .unwrap();
    });
    script[9] = 250; // too many functions for empty body (at index 9 in V3 header)

    let err = MitoVM::execute_direct(&script, &[], &[]).unwrap_err();
    assert!(matches!(err, VMError::InvalidScript));
}
