//! Syscall Coverage Tests
//!
//! This suite verifies that system calls (CALL_NATIVE) are correctly dispatched and handled.
//! It focuses on syscalls that are not covered by other specific test suites, such as
//! cryptographic operations and compute unit management.

use five_protocol::{opcodes::*, Value, FIVE_HEADER_OPTIMIZED_SIZE, FIVE_MAGIC};
use five_vm_mito::{
    stack::StackStorage, AccountInfo, MitoVM, Result as VmResult, VMError, FIVE_VM_PROGRAM_ID,
};

// Syscall IDs (must match five-vm-mito/src/handlers/syscalls.rs)
const SYSCALL_REMAINING_COMPUTE_UNITS: u8 = 50;
const SYSCALL_GET_EPOCH_SCHEDULE_SYSVAR: u8 = 21;
const SYSCALL_GET_FEES_SYSVAR: u8 = 24;
const SYSCALL_SHA256: u8 = 80;
const SYSCALL_KECCAK256: u8 = 81;
const SYSCALL_BLAKE3: u8 = 82;
const SYSCALL_POSEIDON: u8 = 83;
const SYSCALL_SECP256K1_RECOVER: u8 = 84;
const SYSCALL_VERIFY_ED25519_INSTRUCTION: u8 = 92;
const SYSCALL_ALT_BN128_GROUP_OP: u8 = 86;
const ED25519_PROGRAM_ID_BYTES: [u8; 32] = [
    0x03, 0x7d, 0x46, 0xd6, 0x7c, 0x93, 0xfb, 0xbe, 0x12, 0xf9, 0x42, 0x8f, 0x83, 0x8d, 0x40, 0xff,
    0x05, 0x70, 0x74, 0x49, 0x27, 0xf4, 0x8a, 0x64, 0xfc, 0xca, 0x70, 0x44, 0x80, 0x00, 0x00, 0x00,
];

fn build_script(build: impl FnOnce(&mut Vec<u8>)) -> Vec<u8> {
    let mut script = Vec::with_capacity(FIVE_HEADER_OPTIMIZED_SIZE + 256);
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

fn execute_with_accounts(
    build: impl FnOnce(&mut Vec<u8>),
    accounts: &[AccountInfo],
) -> VmResult<Option<Value>> {
    let script = build_script(build);
    let mut storage = StackStorage::new();
    MitoVM::execute_direct(&script, &[], accounts, &FIVE_VM_PROGRAM_ID, &mut storage)
}

fn push_string_buffer(script: &mut Vec<u8>, length: u32) {
    script.push(PUSH_STRING);
    script.extend_from_slice(&(length as u32).to_le_bytes());
    // Append zeros for the string content
    for _ in 0..length {
        script.push(0);
    }
}

fn push_string_bytes(script: &mut Vec<u8>, bytes: &[u8]) {
    script.push(PUSH_STRING);
    script.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
    script.extend_from_slice(bytes);
}

fn push_u64_instr(script: &mut Vec<u8>, value: u64) {
    script.push(PUSH_U64);
    script.extend_from_slice(&value.to_le_bytes());
}

#[test]
fn test_syscall_remaining_compute_units() {
    match execute(|script| {
        script.push(CALL_NATIVE);
        script.push(SYSCALL_REMAINING_COMPUTE_UNITS);
        script.push(RETURN_VALUE);
    }) {
        Ok(Some(Value::U64(units))) => {
            println!("✅ SYSCALL_REMAINING_COMPUTE_UNITS returned: {}", units);
            // Mock implementation returns 200_000
            assert_eq!(units, 200_000, "Mock compute units should be 200,000");
        }
        Ok(result) => panic!("❌ Expected U64, got {:?}", result),
        Err(e) => panic!("❌ Execution failed: {:?}", e),
    }
}

#[test]
fn test_syscall_sha256() {
    match execute(|script| {
        // 1. Create data buffer (empty string)
        push_string_buffer(script, 0);

        // 2. Create result buffer (32 bytes)
        push_string_buffer(script, 32);

        // 3. Call SHA256
        script.push(CALL_NATIVE);
        script.push(SYSCALL_SHA256);

        // 4. Return success; SHA256 writes into the result buffer and pushes nothing.
        // Or push a value and return it.
        script.push(PUSH_U8);
        script.push(1);
        script.push(RETURN_VALUE);
    }) {
        Ok(Some(Value::U8(1))) => {
            println!("✅ SYSCALL_SHA256 executed successfully");
        }
        Ok(result) => panic!("❌ Expected U8(1), got {:?}", result),
        Err(e) => panic!("❌ Execution failed: {:?}", e),
    }
}

#[test]
fn test_syscall_keccak256() {
    match execute(|script| {
        push_string_buffer(script, 0); // Data
        push_string_buffer(script, 32); // Result

        script.push(CALL_NATIVE);
        script.push(SYSCALL_KECCAK256);

        script.push(PUSH_U8);
        script.push(1);
        script.push(RETURN_VALUE);
    }) {
        Ok(Some(Value::U8(1))) => println!("✅ SYSCALL_KECCAK256 executed successfully"),
        Err(e) => panic!("❌ Execution failed: {:?}", e),
        _ => panic!("Unexpected result"),
    }
}

#[test]
fn test_syscall_blake3() {
    match execute(|script| {
        push_string_buffer(script, 0); // Data
        push_string_buffer(script, 32); // Result

        script.push(CALL_NATIVE);
        script.push(SYSCALL_BLAKE3);
        script.push(PUSH_U8);
        script.push(1);
        script.push(RETURN_VALUE);
    }) {
        Ok(Some(Value::U8(1))) => println!("✅ SYSCALL_BLAKE3 executed successfully"),
        Ok(result) => panic!("❌ Expected U8(1), got {:?}", result),
        Err(e) => panic!("❌ Execution failed: {:?}", e),
    }
}

#[test]
fn test_syscall_poseidon() {
    match execute(|script| {
        // Stack: result, vals, endianness, parameters.
        // Push order: parameters, endianness, vals, result

        push_u64_instr(script, 0); // parameters
        push_u64_instr(script, 0); // endianness
        push_string_buffer(script, 32); // vals
        push_string_buffer(script, 32); // result

        script.push(CALL_NATIVE);
        script.push(SYSCALL_POSEIDON);

        script.push(PUSH_U8);
        script.push(1);
        script.push(RETURN_VALUE);
    }) {
        Ok(Some(Value::U8(1))) => println!("✅ SYSCALL_POSEIDON executed successfully"),
        Err(e) => panic!("❌ Execution failed: {:?}", e),
        _ => panic!("Unexpected result"),
    }
}

#[test]
fn test_syscall_secp256k1_recover() {
    match execute(|script| {
        // Stack: result, signature, recovery_id, hash.
        // Push order: hash, recovery_id, signature, result

        push_string_buffer(script, 32); // Hash (32 bytes)

        push_u64_instr(script, 0); // Recovery ID

        push_string_buffer(script, 64); // Signature (64 bytes -> HeapString)

        push_string_buffer(script, 64); // Result (64 bytes -> HeapString)

        script.push(CALL_NATIVE);
        script.push(SYSCALL_SECP256K1_RECOVER);

        script.push(PUSH_U8);
        script.push(1);
        script.push(RETURN_VALUE);
    }) {
        Ok(Some(Value::U8(1))) => println!("✅ SYSCALL_SECP256K1_RECOVER executed successfully"),
        Err(e) => panic!("❌ Execution failed: {:?}", e),
        _ => panic!("Unexpected result"),
    }
}

#[test]
fn test_syscall_verify_ed25519_instruction() {
    let expected_pubkey = [11u8; 32];
    let signature = [22u8; 64];
    let message = [33u8; 32];

    // Construct a minimal instructions-sysvar-style payload for one instruction.
    let mut ed_data = vec![0u8; 144];
    ed_data[0] = 1; // one signature
    ed_data[1] = 0; // padding

    // Offsets struct starts at byte 2.
    ed_data[2..4].copy_from_slice(&48u16.to_le_bytes()); // signature_offset
    ed_data[4..6].copy_from_slice(&u16::MAX.to_le_bytes()); // signature_instruction_index
    ed_data[6..8].copy_from_slice(&16u16.to_le_bytes()); // public_key_offset
    ed_data[8..10].copy_from_slice(&u16::MAX.to_le_bytes()); // public_key_instruction_index
    ed_data[10..12].copy_from_slice(&112u16.to_le_bytes()); // message_data_offset
    ed_data[12..14].copy_from_slice(&32u16.to_le_bytes()); // message_data_size
    ed_data[14..16].copy_from_slice(&u16::MAX.to_le_bytes()); // message_instruction_index

    ed_data[16..48].copy_from_slice(&expected_pubkey);
    ed_data[48..112].copy_from_slice(&signature);
    ed_data[112..144].copy_from_slice(&message);

    let mut sysvar_payload = vec![];
    sysvar_payload.extend_from_slice(&1u16.to_le_bytes()); // instruction_count
    sysvar_payload.extend_from_slice(&4u16.to_le_bytes()); // first instruction offset
    sysvar_payload.extend_from_slice(&0u16.to_le_bytes()); // account_count
    sysvar_payload.extend_from_slice(&ED25519_PROGRAM_ID_BYTES); // program id
    sysvar_payload.extend_from_slice(&(ed_data.len() as u16).to_le_bytes());
    sysvar_payload.extend_from_slice(&ed_data);

    let mut sysvar_lamports = 1u64;
    let mut sysvar_data = sysvar_payload;
    let sysvar_key = [42u8; 32];

    let sysvar_account = AccountInfo::new(
        &sysvar_key,
        false,
        false,
        &mut sysvar_lamports,
        &mut sysvar_data,
        &FIVE_VM_PROGRAM_ID,
        false,
        0,
    );
    let accounts = [sysvar_account];

    match execute_with_accounts(
        |script| {
            script.push(GET_ACCOUNT);
            script.push(0); // instruction_sysvar account index

            script.push(PUSH_PUBKEY);
            script.extend_from_slice(&expected_pubkey);

            push_string_bytes(script, &message);
            push_string_bytes(script, &signature);

            script.push(CALL_NATIVE);
            script.push(SYSCALL_VERIFY_ED25519_INSTRUCTION);
            script.push(RETURN_VALUE);
        },
        &accounts,
    ) {
        Ok(Some(Value::Bool(true))) => {
            println!("✅ SYSCALL_VERIFY_ED25519_INSTRUCTION returned true");
        }
        Ok(result) => panic!("❌ Expected Bool(true), got {:?}", result),
        Err(e) => panic!("❌ Execution failed: {:?}", e),
    }
}

#[test]
fn test_unsupported_sysvar_syscalls_return_runtime_integration_required() {
    match execute(|script| {
        script.push(CALL_NATIVE);
        script.push(SYSCALL_GET_EPOCH_SCHEDULE_SYSVAR);
        script.push(RETURN_VALUE);
    }) {
        Err(VMError::RuntimeIntegrationRequired) => {}
        other => panic!(
            "❌ Expected RuntimeIntegrationRequired for epoch schedule syscall, got {:?}",
            other
        ),
    }

    match execute(|script| {
        script.push(CALL_NATIVE);
        script.push(SYSCALL_GET_FEES_SYSVAR);
        script.push(RETURN_VALUE);
    }) {
        Err(VMError::RuntimeIntegrationRequired) => {}
        other => panic!(
            "❌ Expected RuntimeIntegrationRequired for fees syscall, got {:?}",
            other
        ),
    }
}

#[test]
fn test_unsupported_curve_syscalls_return_runtime_integration_required() {
    match execute(|script| {
        script.push(CALL_NATIVE);
        script.push(SYSCALL_ALT_BN128_GROUP_OP);
        script.push(RETURN_VALUE);
    }) {
        Err(VMError::RuntimeIntegrationRequired) => {}
        other => panic!(
            "❌ Expected RuntimeIntegrationRequired for alt_bn128_group_op, got {:?}",
            other
        ),
    }
}
