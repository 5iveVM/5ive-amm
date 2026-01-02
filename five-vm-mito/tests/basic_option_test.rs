//! Basic Option/Result tests using the production header format.

use five_protocol::{opcodes::*, Value, FIVE_HEADER_OPTIMIZED_SIZE, FIVE_MAGIC};
use five_vm_mito::{FIVE_VM_PROGRAM_ID, MitoVM};

fn build_script(body: &[u8]) -> Vec<u8> {
    let mut script = Vec::with_capacity(FIVE_HEADER_OPTIMIZED_SIZE + body.len());
    script.extend_from_slice(&FIVE_MAGIC);
    // Header V3: features(4 bytes LE) + public_function_count(1) + total_function_count(1)
    script.push(0); // features byte 0
    script.push(0); // features byte 1
    script.push(0); // features byte 2
    script.push(0); // features byte 3
    script.push(0); // no public functions
    script.push(0); // no internal functions
    script.extend_from_slice(body);
    script
}

#[test]
fn test_basic_option_creation() {
    // Very simple test: create Option::Some(42) and check if it's Some
    let body = [
        PUSH_U64,
        0x2A,             // VLE-encoded 42
        OPTIONAL_SOME,    // Wrap in Option::Some
        OPTIONAL_IS_SOME, // Check if Some
        HALT,             // Stop execution
    ];

    let script = build_script(&body);

    let result =
        MitoVM::execute_direct(&script, &[], &[], &FIVE_VM_PROGRAM_ID).expect("VM should execute option bytecode");

    assert_eq!(result, Some(Value::Bool(true)));
}

#[test]
fn test_basic_result_creation() {
    // Very simple test: create Result::Ok(123) and check if it's Ok
    let body = [
        PUSH_U64,
        0x7B,         // VLE-encoded 123
        RESULT_OK,    // Wrap in Result::Ok
        RESULT_IS_OK, // Check if Ok
        HALT,         // Stop execution
    ];

    let script = build_script(&body);

    let result =
        MitoVM::execute_direct(&script, &[], &[], &FIVE_VM_PROGRAM_ID).expect("VM should execute result bytecode");

    assert_eq!(result, Some(Value::Bool(true)));
}
