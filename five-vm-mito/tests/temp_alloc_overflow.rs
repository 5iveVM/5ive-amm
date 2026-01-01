use five_protocol::opcodes::HALT;
use five_vm_mito::{ExecutionContext, StackStorage, TEMP_BUFFER_SIZE};
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

    // Allocation larger than the buffer should fail
    assert!(ctx.alloc_temp((TEMP_BUFFER_SIZE + 1) as u8).is_err());

    // Allocate entire buffer and ensure further allocation fails
    ctx.alloc_temp(TEMP_BUFFER_SIZE as u8)
        .expect("should allocate full buffer");
    assert!(ctx.alloc_temp(1).is_err());
}
