//! Logical Operations Tests
//!
//! This suite tests logical operations (AND, OR, NOT, XOR, BITWISE_*) and rotate operations.

use five_protocol::{encoding::VLE, opcodes::*, FIVE_HEADER_OPTIMIZED_SIZE, FIVE_MAGIC};
use five_vm_mito::{FIVE_VM_PROGRAM_ID, MitoVM, Value, stack::StackStorage, AccountInfo};

fn execute_test(bytecode: &[u8], input: &[u8], accounts: &[AccountInfo]) -> five_vm_mito::Result<Option<Value>> {
    let mut storage = StackStorage::new(bytecode);
    MitoVM::execute_direct(bytecode, input, accounts, &FIVE_VM_PROGRAM_ID, &mut storage)
}

fn build_script(build: impl FnOnce(&mut Vec<u8>)) -> Vec<u8> {
    let mut script = Vec::with_capacity(FIVE_HEADER_OPTIMIZED_SIZE + 16);
    script.extend_from_slice(&FIVE_MAGIC);
    // Header V3: features(4 bytes LE) + public_function_count(1) + total_function_count(1)
    script.push(0x00);
    script.push(0x00);
    script.push(0x00);
    script.push(0x00);
    script.push(0x00);
    script.push(0x00);
    build(&mut script);
    script
}

fn push_u64(script: &mut Vec<u8>, value: u64) {
    script.push(PUSH_U64);
    let (len, encoded) = VLE::encode_u64(value);
    script.extend_from_slice(&encoded[..len]);
}

fn push_bool(script: &mut Vec<u8>, value: bool) {
    script.push(PUSH_BOOL);
    script.push(if value { 1 } else { 0 });
}

#[test]
fn test_bitwise_not() {
    // Spec: BITWISE_NOT (~)
    // ~0 should be 0xFFFFFFFFFFFFFFFF
    let bc_not_zero = build_script(|script| {
        push_u64(script, 0);
        script.push(BITWISE_NOT);
        script.push(HALT);
    });
    let res = execute_test(&bc_not_zero, &[], &[]).unwrap();
    assert_eq!(res, Some(Value::U64(0xFFFFFFFFFFFFFFFF)), "~0 should be 0xFFFFFFFFFFFFFFFF");

    // ~0xFFFFFFFFFFFFFFFF should be 0
    let bc_not_max = build_script(|script| {
        push_u64(script, 0xFFFFFFFFFFFFFFFF);
        script.push(BITWISE_NOT);
        script.push(HALT);
    });
    let res = execute_test(&bc_not_max, &[], &[]).unwrap();
    assert_eq!(res, Some(Value::U64(0)), "~MAX should be 0");

    // ~0xF0F0F0F0F0F0F0F0 should be 0x0F0F0F0F0F0F0F0F
    let bc_not_alt = build_script(|script| {
        push_u64(script, 0xF0F0F0F0F0F0F0F0);
        script.push(BITWISE_NOT);
        script.push(HALT);
    });
    let res = execute_test(&bc_not_alt, &[], &[]).unwrap();
    assert_eq!(res, Some(Value::U64(0x0F0F0F0F0F0F0F0F)), "~0xF0... should be 0x0F...");
}

#[test]
fn test_bitwise_and() {
    // Spec: BITWISE_AND (&)
    // 0xFF & 0x0F = 0x0F
    let bc = build_script(|script| {
        push_u64(script, 0xFF);
        push_u64(script, 0x0F);
        script.push(BITWISE_AND);
        script.push(HALT);
    });
    let res = execute_test(&bc, &[], &[]).unwrap();
    assert_eq!(res, Some(Value::U64(0x0F)), "0xFF & 0x0F should be 0x0F");
}

#[test]
fn test_bitwise_or() {
    // Spec: BITWISE_OR (|)
    // 0xF0 | 0x0F = 0xFF
    let bc = build_script(|script| {
        push_u64(script, 0xF0);
        push_u64(script, 0x0F);
        script.push(BITWISE_OR);
        script.push(HALT);
    });
    let res = execute_test(&bc, &[], &[]).unwrap();
    assert_eq!(res, Some(Value::U64(0xFF)), "0xF0 | 0x0F should be 0xFF");
}

#[test]
fn test_bitwise_xor() {
    // Spec: BITWISE_XOR (^)
    // 0xFF ^ 0x0F = 0xF0
    let bc = build_script(|script| {
        push_u64(script, 0xFF);
        push_u64(script, 0x0F);
        script.push(BITWISE_XOR);
        script.push(HALT);
    });
    let res = execute_test(&bc, &[], &[]).unwrap();
    assert_eq!(res, Some(Value::U64(0xF0)), "0xFF ^ 0x0F should be 0xF0");
}

#[test]
fn test_rotate_left() {
    // Spec: ROTATE_LEFT (circular shift)
    // 1 rotl 1 = 2
    let bc_1 = build_script(|script| {
        push_u64(script, 1);
        push_u64(script, 1);
        script.push(ROTATE_LEFT);
        script.push(HALT);
    });
    let res = execute_test(&bc_1, &[], &[]).unwrap();
    assert_eq!(res, Some(Value::U64(2)), "1 rotl 1 should be 2");

    // 0x8000000000000000 rotl 1 = 1
    let bc_wrap = build_script(|script| {
        push_u64(script, 0x8000000000000000);
        push_u64(script, 1);
        script.push(ROTATE_LEFT);
        script.push(HALT);
    });
    let res = execute_test(&bc_wrap, &[], &[]).unwrap();
    assert_eq!(res, Some(Value::U64(1)), "MSB rotl 1 should wrap to 1");
}

#[test]
fn test_rotate_right() {
    // Spec: ROTATE_RIGHT (circular shift)
    // 2 rotr 1 = 1
    let bc_1 = build_script(|script| {
        push_u64(script, 2);
        push_u64(script, 1);
        script.push(ROTATE_RIGHT);
        script.push(HALT);
    });
    let res = execute_test(&bc_1, &[], &[]).unwrap();
    assert_eq!(res, Some(Value::U64(1)), "2 rotr 1 should be 1");

    // 1 rotr 1 = 0x8000000000000000
    let bc_wrap = build_script(|script| {
        push_u64(script, 1);
        push_u64(script, 1);
        script.push(ROTATE_RIGHT);
        script.push(HALT);
    });
    let res = execute_test(&bc_wrap, &[], &[]).unwrap();
    assert_eq!(res, Some(Value::U64(0x8000000000000000)), "1 rotr 1 should wrap to MSB");
}

#[test]
fn test_logical_xor() {
    // Spec: XOR (boolean)
    // true ^ true = false
    let bc_tt = build_script(|script| {
        push_bool(script, true);
        push_bool(script, true);
        script.push(XOR);
        script.push(HALT);
    });
    let res = execute_test(&bc_tt, &[], &[]).unwrap();
    assert_eq!(res, Some(Value::Bool(false)), "true ^ true should be false");

    // true ^ false = true
    let bc_tf = build_script(|script| {
        push_bool(script, true);
        push_bool(script, false);
        script.push(XOR);
        script.push(HALT);
    });
    let res = execute_test(&bc_tf, &[], &[]).unwrap();
    assert_eq!(res, Some(Value::Bool(true)), "true ^ false should be true");

    // false ^ false = false
    let bc_ff = build_script(|script| {
        push_bool(script, false);
        push_bool(script, false);
        script.push(XOR);
        script.push(HALT);
    });
    let res = execute_test(&bc_ff, &[], &[]).unwrap();
    assert_eq!(res, Some(Value::Bool(false)), "false ^ false should be false");
}
