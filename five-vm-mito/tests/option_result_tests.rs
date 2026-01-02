//! Comprehensive tests for Option and Result type operations in MitoVM
//!
//! These tests verify that the Option and Result opcodes work correctly
//! with proper temp buffer management and type safety.

use five_protocol::{encoding::VLE, opcodes::*, Value, FIVE_HEADER_OPTIMIZED_SIZE, FIVE_MAGIC};
use five_vm_mito::{FIVE_VM_PROGRAM_ID, MitoVM, Result as VmResult};

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
    MitoVM::execute_direct(&script, &[], &[], &FIVE_VM_PROGRAM_ID)
}

fn push_u64_instr(script: &mut Vec<u8>, value: u64) {
    script.push(PUSH_U64);
    let (len, encoded) = VLE::encode_u64(value);
    script.extend_from_slice(&encoded[..len]);
}

#[test]
fn test_option_some_creation_and_unwrap() {
    match execute(|script| {
        push_u64_instr(script, 42);
        script.push(OPTIONAL_SOME);
        script.push(OPTIONAL_UNWRAP);
        script.push(RETURN_VALUE);
    }) {
        Ok(Some(Value::U64(42))) => println!("✅ Option::Some unwrap test passed"),
        Ok(result) => panic!("❌ Expected Some(42), got {:?}", result),
        Err(e) => panic!("❌ Execution failed: {:?}", e),
    }
}

#[test]
fn test_option_none_creation_and_check() {
    match execute(|script| {
        script.extend_from_slice(&[OPTIONAL_NONE, OPTIONAL_IS_NONE, RETURN_VALUE]);
    }) {
        Ok(Some(Value::Bool(true))) => println!("✅ Option::None check test passed"),
        Ok(result) => panic!("❌ Expected Bool(true), got {:?}", result),
        Err(e) => panic!("❌ Execution failed: {:?}", e),
    }
}

#[test]
fn test_option_some_is_some_check() {
    match execute(|script| {
        script.push(PUSH_U8);
        script.push(100);
        script.push(OPTIONAL_SOME);
        script.push(OPTIONAL_IS_SOME);
        script.push(RETURN_VALUE);
    }) {
        Ok(Some(Value::Bool(true))) => println!("✅ Option::Some is_some test passed"),
        Ok(result) => panic!("❌ Expected Bool(true), got {:?}", result),
        Err(e) => panic!("❌ Execution failed: {:?}", e),
    }
}

#[test]
fn test_option_unwrap_none_errors() {
    match execute(|script| {
        script.extend_from_slice(&[OPTIONAL_NONE, OPTIONAL_UNWRAP, RETURN_VALUE]);
    }) {
        Err(_) => println!("✅ Option::None unwrap panic test passed"),
        Ok(result) => panic!("❌ Expected error, but got result: {:?}", result),
    }
}

#[test]
fn test_result_ok_creation_and_unwrap() {
    match execute(|script| {
        push_u64_instr(script, 123);
        script.push(RESULT_OK);
        script.push(RESULT_UNWRAP);
        script.push(RETURN_VALUE);
    }) {
        Ok(Some(Value::U64(123))) => println!("✅ Result::Ok unwrap test passed"),
        Ok(result) => panic!("❌ Expected Some(123), got {:?}", result),
        Err(e) => panic!("❌ Execution failed: {:?}", e),
    }
}

#[test]
fn test_result_err_creation_and_check() {
    match execute(|script| {
        script.extend_from_slice(&[PUSH_U8, 5, RESULT_ERR, RESULT_IS_ERR, RETURN_VALUE]);
    }) {
        Ok(Some(Value::Bool(true))) => println!("✅ Result::Err check test passed"),
        Ok(result) => panic!("❌ Expected Bool(true), got {:?}", result),
        Err(e) => panic!("❌ Execution failed: {:?}", e),
    }
}

#[test]
fn test_result_ok_is_ok_check() {
    match execute(|script| {
        script.extend_from_slice(&[PUSH_BOOL, 1, RESULT_OK, RESULT_IS_OK, RETURN_VALUE]);
    }) {
        Ok(Some(Value::Bool(true))) => println!("✅ Result::Ok is_ok test passed"),
        Ok(result) => panic!("❌ Expected Bool(true), got {:?}", result),
        Err(e) => panic!("❌ Execution failed: {:?}", e),
    }
}

#[test]
fn test_result_unwrap_err_errors() {
    match execute(|script| {
        script.extend_from_slice(&[PUSH_U8, 42, RESULT_ERR, RESULT_UNWRAP, RETURN_VALUE]);
    }) {
        Err(_) => println!("✅ Result::Err unwrap panic test passed"),
        Ok(result) => panic!("❌ Expected error, but got result: {:?}", result),
    }
}

#[test]
fn test_result_get_error_code() {
    match execute(|script| {
        script.extend_from_slice(&[PUSH_U8, 99, RESULT_ERR, RESULT_GET_ERROR, RETURN_VALUE]);
    }) {
        Ok(Some(Value::U8(99))) => println!("✅ Result error code extraction test passed"),
        Ok(result) => panic!("❌ Expected U8(99), got {:?}", result),
        Err(e) => panic!("❌ Execution failed: {:?}", e),
    }
}

#[test]
fn test_option_result_type_integration() {
    match execute(|script| {
        script.push(PUSH_U8);
        script.push(255);
        script.push(RESULT_OK);
        script.push(OPTIONAL_SOME);
        script.push(DUP);
        script.push(OPTIONAL_IS_SOME);
        script.push(POP);
        script.push(OPTIONAL_GET_VALUE);
        script.push(DUP);
        script.push(RESULT_IS_OK);
        script.push(POP);
        script.push(RESULT_UNWRAP);
        script.push(RETURN_VALUE);
    }) {
        Ok(Some(Value::U8(255))) => println!("✅ Option/Result integration test passed"),
        Ok(result) => panic!("❌ Expected U8(255), got {:?}", result),
        Err(e) => panic!("❌ Execution failed: {:?}", e),
    }
}

#[test]
fn test_multiple_options_temp_buffer_usage() {
    match execute(|script| {
        // Create Some(10)
        push_u64_instr(script, 10);
        script.push(OPTIONAL_SOME);
        // Create Some(20)
        push_u64_instr(script, 20);
        script.push(OPTIONAL_SOME);
        // Stack: [Some(10), Some(20)]
        // Extract both values and add them
        script.push(OPTIONAL_GET_VALUE); // Pop Some(20), push 20. Stack: [Some(10), 20]
        script.push(SWAP); // Stack: [20, Some(10)]
        script.push(OPTIONAL_GET_VALUE); // Pop Some(10), push 10. Stack: [20, 10]
        script.push(ADD); // Pop 10, pop 20, push 30. Stack: [30]
        script.push(RETURN_VALUE);
    }) {
        Ok(Some(Value::U64(30))) => {
            println!("✅ Multiple Option temp buffer test passed")
        }
        Ok(result) => panic!("❌ Expected U64(30), got {:?}", result),
        Err(e) => panic!("❌ Execution failed: {:?}", e),
    }
}
