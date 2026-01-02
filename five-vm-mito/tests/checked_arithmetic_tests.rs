//! Tests for checked arithmetic opcodes (ADD_CHECKED, SUB_CHECKED, MUL_CHECKED)
//! Validates overflow detection and error handling (Task 2.2)

use five_protocol::{encoding::VLE, opcodes::*, FIVE_HEADER_OPTIMIZED_SIZE, FIVE_MAGIC};
use five_vm_mito::{FIVE_VM_PROGRAM_ID, MitoVM, VMError, Value};

fn script_header(public_fn_count: u8, total_fn_count: u8) -> Vec<u8> {
    let mut script = Vec::with_capacity(FIVE_HEADER_OPTIMIZED_SIZE);
    script.extend_from_slice(&FIVE_MAGIC);
    // Header V2 (Optimized): features(4 bytes LE) + public_function_count(1) + total_function_count(1)
    script.push(0x00); // features byte 0
    script.push(0x00); // features byte 1
    script.push(0x00); // features byte 2
    script.push(0x00); // features byte 3
    script.push(public_fn_count);
    script.push(total_fn_count);
    script
}

fn push_u64_instr(script: &mut Vec<u8>, value: u64) {
    script.push(PUSH_U64);
    let (len, encoded) = VLE::encode_u64(value);
    script.extend_from_slice(&encoded[..len]);
}

fn single_function_script(build: impl FnOnce(&mut Vec<u8>)) -> Vec<u8> {
    let mut script = script_header(1, 1);
    build(&mut script);
    script
}

#[test]
fn test_add_checked_success() {
    let bytecode = single_function_script(|script| {
        push_u64_instr(script, 100);
        push_u64_instr(script, 50);
        script.push(ADD_CHECKED);
        script.push(RETURN_VALUE);
    });

    match MitoVM::execute_direct(&bytecode, &[], &[], &FIVE_VM_PROGRAM_ID) {
        Ok(Some(Value::U64(result))) => assert_eq!(result, 150),
        Ok(r) => panic!("Expected U64(150), got {:?}", r),
        Err(e) => panic!("Should not error: {:?}", e),
    }
}

#[test]
fn test_add_checked_overflow() {
    let bytecode = single_function_script(|script| {
        push_u64_instr(script, u64::MAX);
        push_u64_instr(script, 1);
        script.push(ADD_CHECKED);
        script.push(RETURN_VALUE);
    });

    match MitoVM::execute_direct(&bytecode, &[], &[], &FIVE_VM_PROGRAM_ID) {
        Ok(r) => panic!("Should error on overflow, got {:?}", r),
        Err(VMError::ArithmeticOverflow) => {} // Expected
        Err(e) => panic!("Wrong error type: {:?}", e),
    }
}

#[test]
fn test_sub_checked_success() {
    let bytecode = single_function_script(|script| {
        push_u64_instr(script, 100);
        push_u64_instr(script, 30);
        script.push(SUB_CHECKED);
        script.push(RETURN_VALUE);
    });

    match MitoVM::execute_direct(&bytecode, &[], &[], &FIVE_VM_PROGRAM_ID) {
        Ok(Some(Value::U64(result))) => assert_eq!(result, 70),
        Ok(r) => panic!("Expected U64(70), got {:?}", r),
        Err(e) => panic!("Should not error: {:?}", e),
    }
}

#[test]
fn test_sub_checked_underflow() {
    let bytecode = single_function_script(|script| {
        push_u64_instr(script, 0);
        push_u64_instr(script, 1);
        script.push(SUB_CHECKED);
        script.push(RETURN_VALUE);
    });

    match MitoVM::execute_direct(&bytecode, &[], &[], &FIVE_VM_PROGRAM_ID) {
        Ok(r) => panic!("Should error on underflow, got {:?}", r),
        Err(VMError::ArithmeticOverflow) => {} // Expected
        Err(e) => panic!("Wrong error type: {:?}", e),
    }
}

#[test]
fn test_mul_checked_success() {
    let bytecode = single_function_script(|script| {
        push_u64_instr(script, 100);
        push_u64_instr(script, 5);
        script.push(MUL_CHECKED);
        script.push(RETURN_VALUE);
    });

    match MitoVM::execute_direct(&bytecode, &[], &[], &FIVE_VM_PROGRAM_ID) {
        Ok(Some(Value::U64(result))) => assert_eq!(result, 500),
        Ok(r) => panic!("Expected U64(500), got {:?}", r),
        Err(e) => panic!("Should not error: {:?}", e),
    }
}

#[test]
fn test_mul_checked_overflow() {
    let two_pow_32 = 1u64 << 32;
    let bytecode = single_function_script(|script| {
        push_u64_instr(script, two_pow_32);
        push_u64_instr(script, two_pow_32);
        script.push(MUL_CHECKED);
        script.push(RETURN_VALUE);
    });

    match MitoVM::execute_direct(&bytecode, &[], &[], &FIVE_VM_PROGRAM_ID) {
        Ok(r) => panic!("Should error on overflow, got {:?}", r),
        Err(VMError::ArithmeticOverflow) => {} // Expected
        Err(e) => panic!("Wrong error type: {:?}", e),
    }
}

#[test]
fn test_checked_arithmetic_in_nested_calls() {
    // Test checked arithmetic within nested function calls
    // Combines Task 2.2 (checked arithmetic) with Task 1.2 (local isolation)
    let bytecode = {
        let mut script = script_header(3, 3);

        // Function 0 (entry)
        push_u64_instr(&mut script, 100);
        script.push(CALL);
        script.push(1); // param_count
        let call_f1_ptr = script.len();
        script.extend_from_slice(&[0, 0]);
        push_u64_instr(&mut script, 50);
        script.push(ADD_CHECKED);
        script.push(RETURN_VALUE);

        // Function 1
        let func1_ip = script.len();
        script.push(LOAD_PARAM);
        script.push(1); // Load first parameter (index 1)
        push_u64_instr(&mut script, 20);
        script.push(SUB_CHECKED);
        script.push(CALL);
        script.push(1); // param_count
        let call_f2_ptr = script.len();
        script.extend_from_slice(&[0, 0]);
        script.push(RETURN);

        // Function 2
        let func2_ip = script.len();
        script.push(LOAD_PARAM);
        script.push(1); // Load first parameter (index 1)
        push_u64_instr(&mut script, 2);
        script.push(MUL_CHECKED);
        script.push(RETURN);

        script[call_f1_ptr] = (func1_ip & 0xFF) as u8;
        script[call_f1_ptr + 1] = ((func1_ip >> 8) & 0xFF) as u8;
        script[call_f2_ptr] = (func2_ip & 0xFF) as u8;
        script[call_f2_ptr + 1] = ((func2_ip >> 8) & 0xFF) as u8;

        script
    };

    match MitoVM::execute_direct(&bytecode, &[0], &[], &FIVE_VM_PROGRAM_ID) {
        Ok(Some(Value::U64(result))) => {
            // f2 returns 160, f1 returns 160, f0 returns 160 + 50 = 210
            assert_eq!(result, 210);
        }
        Ok(r) => panic!("Expected U64(210), got {:?}", r),
        Err(e) => panic!("Should not error: {:?}", e),
    }
}

#[test]
fn test_checked_arithmetic_with_locals() {
    // Test checked arithmetic with local variables
    // Validates both Task 2.2 and Task 1.2 together
    let bytecode = single_function_script(|script| {
        push_u64_instr(script, 100);
        script.push(SET_LOCAL);
        script.push(0);
        push_u64_instr(script, 20);
        script.push(SET_LOCAL);
        script.push(1);
        script.push(GET_LOCAL);
        script.push(0);
        script.push(GET_LOCAL);
        script.push(1);
        script.push(SUB_CHECKED);
        push_u64_instr(script, 2);
        script.push(MUL_CHECKED);
        script.push(RETURN_VALUE);
    });

    // Provide dummy input parameters [0, 2, 0, 0] (Func=0, Count=2, Param1=0, Param2=0)
    // This forces allocation of 2 locals (Param 1->Local 0, Param 2->Local 1)
    match MitoVM::execute_direct(&bytecode, &[0, 2, 0, 0], &[], &FIVE_VM_PROGRAM_ID) {
        Ok(Some(Value::U64(result))) => assert_eq!(result, 160),
        Ok(r) => panic!("Expected U64(160), got {:?}", r),
        Err(e) => panic!("Should not error: {:?}", e),
    }
}
