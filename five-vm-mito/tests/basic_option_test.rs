//! Basic Option/Result tests using the production header format.

use five_protocol::{opcodes::*, Value, FIVE_HEADER_OPTIMIZED_SIZE, FIVE_MAGIC};
use five_vm_mito::{FIVE_VM_PROGRAM_ID, MitoVM, stack::StackStorage};

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
    let mut body = vec![PUSH_U64];
    body.extend_from_slice(&42u64.to_le_bytes()); // Fixed 8-byte 42
    body.push(OPTIONAL_SOME);    // Wrap in Option::Some
    body.push(OPTIONAL_IS_SOME); // Check if Some
    body.push(RETURN_VALUE);             // Stop execution

    let script = build_script(&body);

    let mut storage = StackStorage::new();
    let result =
        MitoVM::execute_direct(&script, &[], &[], &FIVE_VM_PROGRAM_ID, &mut storage).expect("VM should execute option bytecode");

    assert_eq!(result, Some(Value::Bool(true)));
}

#[test]
fn test_basic_result_creation() {
    // Very simple test: create Result::Ok(123) and check if it's Ok
    let mut body = vec![PUSH_U64];
    body.extend_from_slice(&123u64.to_le_bytes()); // Fixed 8-byte 123
    body.push(RESULT_OK);    // Wrap in Result::Ok
    body.push(RESULT_IS_OK); // Check if Ok
    body.push(RETURN_VALUE);         // Stop execution

    let script = build_script(&body);

    let mut storage = StackStorage::new();
    let result =
        MitoVM::execute_direct(&script, &[], &[], &FIVE_VM_PROGRAM_ID, &mut storage).expect("VM should execute result bytecode");

    assert_eq!(result, Some(Value::Bool(true)));
}
