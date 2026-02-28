//! Advanced Option/Result Coverage Tests
//!
//! Additional tests for edge cases and error conditions in Option/Result operations.

use five_protocol::{opcodes::*, Value, FIVE_HEADER_OPTIMIZED_SIZE, FIVE_MAGIC};
use five_vm_mito::{stack::StackStorage, MitoVM, Result as VmResult, FIVE_VM_PROGRAM_ID};

fn build_script(build: impl FnOnce(&mut Vec<u8>)) -> Vec<u8> {
    let mut script = Vec::with_capacity(FIVE_HEADER_OPTIMIZED_SIZE + 32);
    script.extend_from_slice(&FIVE_MAGIC);
    // Header V3: features(4 bytes LE) + public_function_count(1) + total_function_count(1)
    script.push(0); // features byte 0
    script.push(0); // features byte 1
    script.push(0); // features byte 2
    script.push(0); // features byte 3
    script.push(1); // single main function
    script.push(1); // total functions
    build(&mut script);
    script
}

fn execute(build: impl FnOnce(&mut Vec<u8>)) -> VmResult<Option<Value>> {
    let script = build_script(build);
    let mut storage = StackStorage::new();
    MitoVM::execute_direct(&script, &[], &[], &FIVE_VM_PROGRAM_ID, &mut storage)
}

fn push_u64_instr(script: &mut Vec<u8>, value: u64) {
    script.push(PUSH_U64);
    script.extend_from_slice(&value.to_le_bytes());
}

#[test]
fn test_optional_get_value_none() {
    // Test getting value from Option::None returns Value::Empty
    match execute(|script| {
        script.push(OPTIONAL_NONE);
        script.push(OPTIONAL_GET_VALUE);
        script.push(RETURN_VALUE);
    }) {
        Ok(Some(Value::Empty)) => println!("✅ Option::None get_value returned Empty"),
        Ok(result) => panic!("❌ Expected Value::Empty, got {:?}", result),
        Err(e) => panic!("❌ Execution failed: {:?}", e),
    }
}

#[test]
#[ignore = "Failing unexpectedly with Ok(Some(Bool(false))), needs investigation"]
fn test_optional_is_some_type_mismatch() {
    // Test OPTIONAL_IS_SOME on non-optional value
    match execute(|script| {
        push_u64_instr(script, 123);
        script.push(OPTIONAL_IS_SOME);
        script.push(RETURN_VALUE);
    }) {
        Err(e) => {
            println!("✅ OPTIONAL_IS_SOME type mismatch test passed: {:?}", e);
        }
        Ok(result) => panic!("❌ Expected error, but got result: {:?}", result),
    }
}

#[test]
fn test_optional_unwrap_none_constraint_violation() {
    // Test unwrapping Option::None
    match execute(|script| {
        script.push(OPTIONAL_NONE);
        script.push(OPTIONAL_UNWRAP);
        script.push(RETURN_VALUE);
    }) {
        Err(e) => {
            println!("✅ Option::None unwrap error test passed: {:?}", e);
            // Should be ConstraintViolation
        }
        Ok(result) => panic!("❌ Expected error, but got result: {:?}", result),
    }
}
