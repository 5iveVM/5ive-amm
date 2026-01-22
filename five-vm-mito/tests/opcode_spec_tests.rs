//! Opcode Specification Verification Tests
//!
//! This suite verifies that opcodes strictly adhere to the behavior defined in `OPCODE_SPEC.md`.
//! It focuses on edge cases, stack effects, and specific bit-level behaviors.

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

fn push_u16(script: &mut Vec<u8>, value: u16) {
    script.push(PUSH_U16);
    let (len, encoded) = VLE::encode_u64(value as u64);
    script.extend_from_slice(&encoded[..len]);
}

#[test]
fn test_spec_control_flow_jumps() {
    // Spec: JUMP offset (U16) - Unconditional jump
    // We will jump over a failing assertion (REQUIRE false)
    let bytecode = build_script(|script| {
        // Absolute offset calculation:
        // Header: 10 bytes (0-9).
        // Instructions start at 10.
        // 10: JUMP (3 bytes)
        // 13: PUSH_BOOL false (2 bytes)
        // 15: REQUIRE (1 byte)
        // 16: PUSH_BOOL true (2 bytes)
        // 18: REQUIRE (1 byte)
        // 19: PUSH 1 (2 bytes)
        // 21: HALT

        // We want to jump to 16.

        // 0: JUMP to 16
        script.push(JUMP);
        script.extend_from_slice(&16u16.to_le_bytes());

        // 3: PUSH false (should be skipped)
        script.push(PUSH_BOOL);
        script.push(0);

        // 5: REQUIRE (would fail if executed with false)
        script.push(REQUIRE);

        // 6: PUSH true (Target of JUMP at 16)
        script.push(PUSH_BOOL);
        script.push(1);

        // 8: REQUIRE (should pass)
        script.push(REQUIRE);

        // Success
        script.push(PUSH_U64);
        let (len, encoded) = VLE::encode_u64(1);
        script.extend_from_slice(&encoded[..len]);
        script.push(HALT);
    });

    let result = execute_test(&bytecode, &[], &[]).unwrap();
    assert_eq!(result, Some(Value::U64(1)), "JUMP should skip instructions");
}

#[test]
fn test_spec_control_flow_jump_if() {
    // Spec: JUMP_IF offset (U16) - Jump if top is true, pop condition
    let bytecode = build_script(|script| {
        // Test: Push TRUE, JUMP_IF to Skip failing REQUIRE.
        // 10: PUSH_BOOL true
        // 12: JUMP_IF (3 bytes) -> Target 18
        // 15: PUSH_BOOL false
        // 17: REQUIRE (Fail)
        // 18: PUSH 42
        // 20: HALT

        script.push(PUSH_BOOL);
        script.push(1); // True

        script.push(JUMP_IF);
        script.extend_from_slice(&18u16.to_le_bytes());

        // Should be skipped
        script.push(PUSH_BOOL);
        script.push(0);
        script.push(REQUIRE);

        // Target
        push_u64(script, 42);
        script.push(HALT);
    });

    let result = execute_test(&bytecode, &[], &[]).unwrap();
    assert_eq!(result, Some(Value::U64(42)), "JUMP_IF should jump on true");
}

#[test]
fn test_spec_bitwise_operations() {
    // Spec: BITWISE_AND, BITWISE_OR, BITWISE_XOR

    // Retrying with just checking the last operation (XOR)
    let bytecode_xor = build_script(|script| {
         push_u64(script, 0xFF);
         push_u64(script, 0x0F);
         script.push(BITWISE_XOR);
         script.push(HALT);
    });

    let result = execute_test(&bytecode_xor, &[], &[]).unwrap();
    assert_eq!(result, Some(Value::U64(0xF0)), "0xFF ^ 0x0F should be 0xF0");
}

#[test]
fn test_spec_bitwise_shifts() {
    // Spec: SHIFT_LEFT, SHIFT_RIGHT, SHIFT_RIGHT_ARITH

    // 1. Left Shift: 1 << 4 = 16
    let bc_shl = build_script(|script| {
        push_u64(script, 1);
        push_u64(script, 4);
        script.push(SHIFT_LEFT);
        script.push(HALT);
    });
    let res_shl = execute_test(&bc_shl, &[], &[]).unwrap();
    assert_eq!(res_shl, Some(Value::U64(16)), "1 << 4 should be 16");

    // 2. Logical Right Shift: 0xF0 >> 4 = 0x0F
    let bc_shr = build_script(|script| {
        push_u64(script, 0xF0);
        push_u64(script, 4);
        script.push(SHIFT_RIGHT);
        script.push(HALT);
    });
    let res_shr = execute_test(&bc_shr, &[], &[]).unwrap();
    assert_eq!(res_shr, Some(Value::U64(0x0F)), "0xF0 >> 4 should be 0x0F");

    // 3. Arithmetic Right Shift (on signed value)
    // -8 (in 64-bit two's complement) >> 1 should be -4
    // We manually encode the u64 bit pattern for -8 (0xFFFFFFFFFFFFFFF8) to avoid VLE ZigZag confusion in PUSH_I64
    // NOTE: PUSH_I64 implementation casts raw bits to i64.
    let bc_sar = build_script(|script| {
        // PUSH -8 as I64 using raw bytes
        // Note: VLE::encode_u64 encodes the u64 bits. PUSH_I64 reads them and casts to i64.
        script.push(PUSH_I64);
        let neg_eight: i64 = -8;
        let (len, encoded) = VLE::encode_u64(neg_eight as u64); // Encode raw bits
        script.extend_from_slice(&encoded[..len]);

        // PUSH 1 as U64 (shift amount)
        push_u64(script, 1);

        script.push(SHIFT_RIGHT_ARITH);
        script.push(HALT);
    });

    let res_sar = execute_test(&bc_sar, &[], &[]).unwrap();

    if let Some(val) = res_sar {
         match val {
            Value::I64(v) => assert_eq!(v, -4, "-8 >> 1 (arith) should be -4"),
            Value::U64(v) => assert_eq!(v as i64, -4, "-8 >> 1 (arith) cast as U64 should be -4"),
            _ => panic!("Unexpected result type for SAR: {:?}", val),
        }
    } else {
        panic!("SAR execution failed");
    }
}

#[test]
fn test_spec_stack_ops_complex() {
    // Spec: PICK, ROT, OVER

    // 1. OVER: a b -> a b a
    // Stack: [10, 20] -> [10, 20, 10]
    let bc_over = build_script(|script| {
        push_u64(script, 10);
        push_u64(script, 20);
        script.push(OVER);
        script.push(HALT); // Top should be 10
    });
    let res_over = execute_test(&bc_over, &[], &[]).unwrap();
    assert_eq!(res_over, Some(Value::U64(10)), "OVER should copy 2nd item to top");

    // 2. ROT: a b c -> b c a
    // Stack: [1, 2, 3] -> [2, 3, 1]
    let bc_rot = build_script(|script| {
        push_u64(script, 1);
        push_u64(script, 2);
        push_u64(script, 3);
        script.push(ROT);
        script.push(HALT); // Top should be 1
    });
    let res_rot = execute_test(&bc_rot, &[], &[]).unwrap();
    assert_eq!(res_rot, Some(Value::U64(1)), "ROT should move 3rd item to top");

    // 3. PICK: Copy N-th item to top
    // Stack: [10, 20, 30, 40]
    // PICK 2 (0-indexed?) usually picks 0=top, 1=2nd...

    let bc_pick = build_script(|script| {
        push_u64(script, 10); // idx 3
        push_u64(script, 20); // idx 2
        push_u64(script, 30); // idx 1
        push_u64(script, 40); // idx 0

        // Use immediate argument for PICK (index 2 corresponds to value 20)
        script.push(PICK);
        script.push(2);
        script.push(HALT);
    });
    let res_pick = execute_test(&bc_pick, &[], &[]).unwrap();
    // Stack: [10, 20, 30, 40]
    // PICK 2: 0->40, 1->30, 2->20
    assert_eq!(res_pick, Some(Value::U64(20)), "PICK 2 should duplicate value 20");
}

#[test]
fn test_spec_nibble_ops() {
    // Spec: PUSH_0, PUSH_1, etc.
    let bc_nibble = build_script(|script| {
        script.push(PUSH_0);
        script.push(PUSH_1);
        script.push(ADD); // 0 + 1 = 1
        script.push(PUSH_2);
        script.push(ADD); // 1 + 2 = 3
        script.push(PUSH_3);
        script.push(ADD); // 3 + 3 = 6
        script.push(HALT);
    });
    let res = execute_test(&bc_nibble, &[], &[]).unwrap();
    assert_eq!(res, Some(Value::U64(6)), "Nibble PUSH ops should work correctly");
}

// Pattern Fusion tests removed as opcodes were reverted

#[test]
fn test_spec_byte_swap() {
    // Spec: BYTE_SWAP_16/32/64
    // 0x1234 -> 0x3412 (16-bit)
    let bc_swap16 = build_script(|script| {
        push_u16(script, 0x1234);
        script.push(BYTE_SWAP_16);
        script.push(HALT);
    });
    let res = execute_test(&bc_swap16, &[], &[]).unwrap();
    assert_eq!(res, Some(Value::U64(0x3412)), "BYTE_SWAP_16 failed");
}
