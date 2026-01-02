//! Minimal Option/Result test avoiding function handler issues

use five_protocol::opcodes::*;
use five_vm_mito::{ExecutionContext, FIVE_VM_PROGRAM_ID, StackStorage, ValueRef};
use pinocchio::pubkey::Pubkey;

#[test]
fn test_opcodes_are_defined() {
    // Just verify the opcodes have the expected values
    println!("Testing opcode definitions:");
    println!("OPTIONAL_SOME = 0x{:02X}", OPTIONAL_SOME);
    println!("OPTIONAL_NONE = 0x{:02X}", OPTIONAL_NONE);
    println!("OPTIONAL_IS_SOME = 0x{:02X}", OPTIONAL_IS_SOME);
    println!("OPTIONAL_IS_NONE = 0x{:02X}", OPTIONAL_IS_NONE);
    println!("RESULT_OK = 0x{:02X}", RESULT_OK);
    println!("RESULT_ERR = 0x{:02X}", RESULT_ERR);
    println!("RESULT_IS_OK = 0x{:02X}", RESULT_IS_OK);
    println!("RESULT_IS_ERR = 0x{:02X}", RESULT_IS_ERR);

    // Basic sanity checks
    assert_ne!(OPTIONAL_SOME, OPTIONAL_NONE);
    assert_ne!(RESULT_OK, RESULT_ERR);
    assert!(OPTIONAL_SOME >= 0xF0); // Should be in F0-FF range
}

#[test]
fn test_valueref_option_helpers() {
    // Test the ValueRef helper methods directly

    // Test option creation helpers
    let some_ref = ValueRef::option_some(10, 8); // offset=10, size=8
    let none_ref = ValueRef::option_none();

    println!("Some ref: {:?}", some_ref);
    println!("None ref: {:?}", none_ref);

    // Test option checking methods
    assert!(some_ref.is_option_some());
    assert!(!some_ref.is_option_none());

    assert!(!none_ref.is_option_some());
    assert!(none_ref.is_option_none());

    // Test data extraction
    assert_eq!(some_ref.get_option_data(), Some((10, 8)));
    assert_eq!(none_ref.get_option_data(), None);
}

#[test]
fn test_valueref_result_helpers() {
    // Test the ValueRef helper methods for Result

    let ok_ref = ValueRef::result_ok(20, 8); // offset=20, size=8
    let err_code = 7;
    let err_ref = ValueRef::result_err(err_code);

    println!("Ok ref: {:?}", ok_ref);
    println!("Err ref: {:?}", err_ref);

    // Test result checking methods
    assert!(ok_ref.is_result_ok());
    assert!(!ok_ref.is_result_err());

    assert!(!err_ref.is_result_ok());
    assert!(err_ref.is_result_err());

    // Test data extraction
    assert_eq!(ok_ref.get_result_data(), Ok((20, 8)));
    assert_eq!(err_ref.get_result_data(), Err(err_code));
}

#[test]
fn test_execution_context_temp_buffer_methods() {
    // Test the new temp buffer methods in ExecutionContext
    let bytecode = &[HALT]; // Minimal bytecode
    let accounts = &[];
    let program_id = Pubkey::default();
    let instruction_data = &[];

    let mut storage = StackStorage::new(bytecode);
    let mut ctx = ExecutionContext::new(
        bytecode,
        accounts,
        program_id,
        instruction_data,
        0,
        &mut storage,
        0,
        0,
    );

    // Test temp slot allocation
    let slot1 = ctx
        .allocate_temp_slot()
        .expect("Should allocate first slot");
    let slot2 = ctx
        .allocate_temp_slot()
        .expect("Should allocate second slot");
    let slot3 = ctx
        .allocate_temp_slot()
        .expect("Should allocate third slot");

    println!("Allocated slots: {}, {}, {}", slot1, slot2, slot3);

    // Slots should be sequential with 17-byte spacing (16 bytes for ValueRef + 1 byte tag)
    assert_eq!(slot2, slot1 + 17);
    assert_eq!(slot3, slot2 + 17);

    // Test temp buffer access
    let temp_buffer = ctx.temp_buffer_fixed_mut().expect("Should get temp buffer");
    assert_eq!(temp_buffer.len(), 256);

    // Test we can write to it
    temp_buffer[slot1 as usize] = 0x42;
    temp_buffer[slot2 as usize] = 0x43;
    temp_buffer[slot3 as usize] = 0x44;

    // Verify values
    assert_eq!(temp_buffer[slot1 as usize], 0x42);
    assert_eq!(temp_buffer[slot2 as usize], 0x43);
    assert_eq!(temp_buffer[slot3 as usize], 0x44);
}

#[test]
fn test_temp_buffer_exhaustion() {
    // Test what happens when we run out of temp buffer space
    let bytecode = &[HALT];
    let accounts = &[];
    let program_id = Pubkey::default();
    let instruction_data = &[];

    let mut storage = StackStorage::new(bytecode);
    let mut ctx = ExecutionContext::new(
        bytecode,
        accounts,
        program_id,
        instruction_data,
        0,
        &mut storage,
        0,
        0,
    );

    // Try to allocate too many slots (256 bytes / 17 bytes = 15 slots max)
    let mut slots = Vec::new();

    for i in 0..20 {
        match ctx.allocate_temp_slot() {
            Ok(slot) => {
                slots.push(slot);
                println!("Allocated slot {}: offset {}", i, slot);
            }
            Err(e) => {
                println!("Failed to allocate slot {} with error: {:?}", i, e);
                break;
            }
        }
    }

    // Should have allocated some slots but eventually failed
    assert!(!slots.is_empty());
    assert!(slots.len() < 20); // Should fail before 20 slots
    assert!(slots.len() >= 15); // Should allocate at least 15 slots

    println!(
        "Successfully allocated {} slots before exhaustion",
        slots.len()
    );
}

#[test]
fn test_temp_buffer_reset_allows_reuse() {
    let bytecode = &[HALT];
    let accounts = &[];
    let program_id = Pubkey::default();
    let instruction_data = &[];

    let mut storage = StackStorage::new(bytecode);
    let mut ctx = ExecutionContext::new(
        bytecode,
        accounts,
        program_id,
        instruction_data,
        0,
        &mut storage,
        0,
        0,
    );

    // Fill the temp buffer to capacity using slots (15 * 17 = 255 bytes, leaving 1 byte)
    for _ in 0..15 {
        ctx.allocate_temp_slot()
            .expect("initial allocation should succeed");
    }
    // Further allocation should fail due to exhaustion (need 17 bytes but only 1 left)
    assert!(ctx.allocate_temp_slot().is_err());

    // Reset the temp buffer and ensure allocations start over
    ctx.reset_temp_buffer();
    let slot = ctx
        .allocate_temp_slot()
        .expect("allocation after reset should succeed");
    assert_eq!(slot, 0);
}
