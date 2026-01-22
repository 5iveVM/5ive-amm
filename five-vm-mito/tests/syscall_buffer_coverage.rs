//! Syscall Buffer Coverage Tests
//!
//! Tests for syscalls using different buffer types (TempRef, ArrayRef, StringRef)
//! and verifying memory safety checks.

use five_protocol::{encoding::VLE, opcodes::*, Value, FIVE_HEADER_OPTIMIZED_SIZE, FIVE_MAGIC};
use five_vm_mito::{FIVE_VM_PROGRAM_ID, MitoVM, Result as VmResult, stack::StackStorage};

const SYSCALL_SHA256: u8 = 80;

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
    let mut storage = StackStorage::new(&script);
    MitoVM::execute_direct(&script, &[], &[], &FIVE_VM_PROGRAM_ID, &mut storage)
}

fn push_string_buffer(script: &mut Vec<u8>, length: u32) {
    script.push(PUSH_STRING);
    let (len_len, encoded_len) = VLE::encode_u64(length as u64);
    script.extend_from_slice(&encoded_len[..len_len]);
    // Append zeros for the string content
    for _ in 0..length {
        script.push(0);
    }
}

#[test]
fn test_syscall_sha256_small_buffer() {
    // Test SHA256 with result buffer too small (10 bytes < 32 bytes)
    match execute(|script| {
        // 1. Data buffer (empty)
        push_string_buffer(script, 0);

        // 2. Result buffer (too small)
        push_string_buffer(script, 10);

        // 3. Call SHA256
        script.push(CALL_NATIVE);
        script.push(SYSCALL_SHA256);

        script.push(RETURN_VALUE);
    }) {
        Err(e) => {
             println!("✅ SHA256 small buffer failed as expected: {:?}", e);
             // Should be MemoryViolation
        }
        Ok(result) => panic!("❌ Expected MemoryViolation, got result: {:?}", result),
    }
}

#[test]
fn test_syscall_sha256_array_buffer() {
    // Test SHA256 using an Array as DATA buffer (input), and String as result
    // Note: Array as result buffer is hard to test because SHA256 needs 32 bytes,
    // and creating a 32-element ArrayRef exceeds temp buffer/stack limits in current config.
    match execute(|script| {
        // 1. Data buffer (Array of size 1)
        script.push(PUSH_U8);
        script.push(0); // Element 0
        script.push(PUSH_U8);
        script.push(1); // Capacity
        script.push(CREATE_ARRAY); // Stack: [Array(1)]

        // 2. Result buffer (String of size 32)
        push_string_buffer(script, 32); // Stack: [Array(1), String(32)]

        // 3. Call SHA256
        script.push(CALL_NATIVE);
        script.push(SYSCALL_SHA256);

        // 4. Return success marker
        script.push(PUSH_U8);
        script.push(1);
        script.push(RETURN_VALUE);
    }) {
        Ok(Some(Value::U8(1))) => println!("✅ SHA256 with Array input buffer succeeded"),
        Ok(result) => panic!("❌ Expected success, got {:?}", result),
        Err(e) => panic!("❌ Execution failed: {:?}", e),
    }
}
