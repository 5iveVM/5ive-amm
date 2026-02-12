use five_protocol::opcodes::HALT;
use five_vm_mito::{ExecutionContext, StackStorage, TEMP_BUFFER_SIZE};
use pinocchio::pubkey::Pubkey;

#[test]
fn test_alloc_temp_overflow() {
    let bytecode = &[HALT];
    let accounts = &[];
    let program_id = Pubkey::default();
    let instruction_data = &[];

    let mut storage = StackStorage::new();
    let mut ctx = ExecutionContext::new(
        bytecode,
        accounts,
        program_id,
        instruction_data,
        0,
        &mut storage,
        0,
        0,
        0,
        0,
        0,
        0,
    );

    assert!(TEMP_BUFFER_SIZE > 0);

    // Allocate byte-by-byte until the VM reports exhaustion.
    let mut allocated = 0usize;
    while ctx.alloc_temp(1).is_ok() {
        allocated += 1;
    }

    assert!(allocated > 0, "temp buffer should allow at least one byte");
    assert!(
        allocated <= TEMP_BUFFER_SIZE as usize,
        "allocated bytes {} should not exceed TEMP_BUFFER_SIZE {}",
        allocated,
        TEMP_BUFFER_SIZE
    );
    assert!(ctx.alloc_temp(1).is_err(), "further alloc must keep failing");
}
