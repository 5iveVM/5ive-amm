//! Syscall Coverage Tests
//!
//! This suite verifies that system calls (CALL_NATIVE) are correctly dispatched and handled.
//! It focuses on syscalls that are not covered by other specific test suites, such as
//! cryptographic operations and compute unit management.

use five_protocol::{opcodes::*, Value, FIVE_HEADER_OPTIMIZED_SIZE, FIVE_MAGIC};
use five_vm_mito::{FIVE_VM_PROGRAM_ID, MitoVM, Result as VmResult, stack::StackStorage};

// Syscall IDs (must match five-vm-mito/src/handlers/syscalls.rs)
const SYSCALL_REMAINING_COMPUTE_UNITS: u8 = 50;
const SYSCALL_SHA256: u8 = 80;
const SYSCALL_KECCAK256: u8 = 81;
const SYSCALL_POSEIDON: u8 = 83;
const SYSCALL_SECP256K1_RECOVER: u8 = 84;

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

fn push_string_buffer(script: &mut Vec<u8>, length: u32) {
    script.push(PUSH_STRING);
    script.extend_from_slice(&(length as u32).to_le_bytes());
    // Append zeros for the string content
    for _ in 0..length {
        script.push(0);
    }
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
