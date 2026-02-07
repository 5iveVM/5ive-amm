//! Tests for checked arithmetic opcodes (ADD_CHECKED, SUB_CHECKED, MUL_CHECKED)
//! Validates overflow detection and error handling (Task 2.2)

use five_protocol::{opcodes::*, FIVE_HEADER_OPTIMIZED_SIZE, FIVE_MAGIC};
use five_vm_mito::{FIVE_VM_PROGRAM_ID, MitoVM, VMError, Value, stack::StackStorage, AccountInfo};

fn execute_test(bytecode: &[u8], input: &[u8], accounts: &[AccountInfo]) -> five_vm_mito::Result<Option<Value>> {
    let mut storage = StackStorage::new();
    MitoVM::execute_direct(bytecode, input, accounts, &FIVE_VM_PROGRAM_ID, &mut storage)
}

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
    script.extend_from_slice(&value.to_le_bytes());
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

    match execute_test(&bytecode, &[], &[]) {
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

    match execute_test(&bytecode, &[], &[]) {
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

    match execute_test(&bytecode, &[], &[]) {
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

    match execute_test(&bytecode, &[], &[]) {
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

    match execute_test(&bytecode, &[], &[]) {
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

    match execute_test(&bytecode, &[], &[]) {
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

    match execute_test(&bytecode, &[0, 0, 0, 0], &[]) {
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

    // Provide input: [Func=0 (u32), Count=2 (u32), Param1(0)=u64(0), Param2(0)=u64(0)]
    // Total: 4 + 4 + 8 + 8 = 24 bytes? Or are params u8?
    // LOAD_PARAM loads from parameters array.
    // If we use `LOAD_PARAM`, we need params in `ctx.parameters`.
    // The previous test used `[0, 2, 0, 0]`. It assumed [func(u8), count(u8), p1(u8), p2(u8)]?
    // But `LOAD_PARAM` operates on `u64`.
    // If the input parsing now expects u32 func index and u32 count...
    // We should construct it properly.

    // However, if `execute_direct` just parses func/count and ignores the rest (or parses params based on count),
    // and `LOAD_PARAM` accesses `ctx.parameters`.
    // We need `ctx.parameters` to be populated.

    // Let's assume input format: [func_idx(u32), param_count(u32), p1(u64), p2(u64)...]?
    // Or did I define param format as something else?
    // In `five-wasm`, I used `[func_idx(u32), param_count(u32), ...]` and params were variable.
    // But `MitoVM` parses input.
    // Let's assume params are u64 if not specified otherwise (legacy `LOAD_INPUT` used u8, but `LOAD_PARAM` uses `u64`).
    // Actually, `LOAD_PARAM` returns `Value::U64`.
    // The parameters in `ctx` are `[u64; MAX_PARAMETERS]`.
    // So the input parser must parse them as u64?
    // Let's construct a safe input:
    let mut input = vec![];
    input.extend_from_slice(&0u32.to_le_bytes()); // Func 0
    input.extend_from_slice(&2u32.to_le_bytes()); // Count 2
    // Param 1 (0) - encoded as ?
    // If `MitoVM` expects Typed params (0x80 sentinel), we should use that?
    // Or does it assume untyped u64?
    // Given the changes, it likely expects:
    // [func(u32), count(u32), Type(u8), Value(u64), Type(u8), Value(u64)]
    // Use Type 4 (U64) for both params
    input.push(4); // Type U64
    input.extend_from_slice(&0u64.to_le_bytes());
    input.push(4); // Type U64
    input.extend_from_slice(&0u64.to_le_bytes());

    match execute_test(&bytecode, &input, &[]) {
        Ok(Some(Value::U64(result))) => assert_eq!(result, 160),
        Ok(r) => panic!("Expected U64(160), got {:?}", r),
        Err(e) => panic!("Should not error: {:?}", e),
    }
}
