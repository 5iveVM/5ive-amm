use five_protocol::opcodes::HALT;
use five_vm_mito::{ExecutionContext, FIVE_VM_PROGRAM_ID, StackStorage, TEMP_BUFFER_SIZE};
use pinocchio::pubkey::Pubkey;

#[test]
fn test_alloc_temp_overflow() {
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

    // TEMP_BUFFER_SIZE is 256
    // We cannot allocate 256 bytes in one go because alloc_temp takes u8 (max 255)
    
    // 1. Allocate first chunk (255 bytes) - should succeed
    ctx.alloc_temp(255).expect("should allocate first chunk");

    // 2. Allocate remaining 1 byte - should succeed
    ctx.alloc_temp(1).expect("should allocate remaining byte");

    // 3. Attempt to allocate 1 more byte - should fail (overflow)
    assert!(ctx.alloc_temp(1).is_err());
}
