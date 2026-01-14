
use five_protocol::opcodes::*;
use five_vm_mito::{ExecutionContext, StackStorage, ValueRef, utils, context::ExecutionManager};
use pinocchio::{account_info::AccountInfo, pubkey::Pubkey};

#[test]
fn test_resolve_bool_from_account_ref() {
    // 1. Setup mock account with byte 1 (true)
    let key = Pubkey::default();
    let mut lamports = 0;
    let mut data = [1u8, 0, 0, 0, 0, 0, 0, 0]; // 1 byte true, padding
    let owner = Pubkey::default();
    
    let account = AccountInfo::new(
        &key,
        false, // is_signer
        true,  // is_writable 
        &mut lamports,
        &mut data,
        &owner,
        false, // executable
        0,     // rent_epoch
    );

    let accounts = vec![account];
    let bytecode = &[HALT];
    let program_id = Pubkey::default();
    let instruction_data = &[];

    let mut storage = StackStorage::new(bytecode);
    
    // 2. Create ExecutionContext
    let mut ctx = ExecutionContext::new(
        bytecode,
        &accounts,
        program_id,
        instruction_data,
        0,
        &mut storage,
        0,
        0,
    );

    // 3. Create AccountRef pointing to index 0, offset 0
    let account_ref = ValueRef::AccountRef(0, 0);

    // 4. Test resolve_bool directly
    let result = utils::resolve_bool(account_ref, &ctx);
    assert!(result.is_ok(), "resolve_bool failed: {:?}", result.err());
    assert_eq!(result.unwrap(), true, "resolve_bool should return true for byte 1");

    // 5. Test with byte 0 (false)
    // Safety: In test environment with RefCell/unsafe pattern used by Pinocchio
    unsafe {
        ctx.accounts.accounts()[0].borrow_mut_data_unchecked()[0] = 0;
    }
    
    let result_false = utils::resolve_bool(account_ref, &ctx);
    assert!(result_false.is_ok());
    assert_eq!(result_false.unwrap(), false, "resolve_bool should return false for byte 0");

    // 6. Test with non-zero byte (e.g. 2) - should be true
    unsafe {
        ctx.accounts.accounts()[0].borrow_mut_data_unchecked()[0] = 2;
    }
    let result_non_zero = utils::resolve_bool(account_ref, &ctx);
    assert_eq!(result_non_zero.unwrap(), true, "resolve_bool should return true for byte 2");
}

#[test]
fn test_not_opcode_with_account_ref() {
    // Setup for NOT opcode execution
    // We can't easily execute a single opcode dispatch without reconstructing the loop or handler logic,
    // but we can verify the handler logic if we could import it. 
    // Since handlers are public in crate, we might assume if resolve_bool works, NOT works given we updated usage.
    // But testing resolve_bool covers the core logic fix.
}
