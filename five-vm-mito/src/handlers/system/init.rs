//! Account initialization operations for MitoVM
//!
//! This module handles account creation operations via System Program integration:
//! - INIT_ACCOUNT: Create regular accounts
//! - INIT_PDA_ACCOUNT: Create Program Derived Address accounts
//!
//! These handlers implement @init constraint functionality for automatic account creation.

use crate::{
    context::ExecutionManager,
    debug_log,
    error::{CompactResult, VMErrorCode},
    utils::value_ref_to_seed_bytes,
};
use five_protocol::{opcodes::*, ValueRef};
use heapless::Vec;
#[cfg(target_os = "solana")]
use pinocchio::pubkey::Pubkey;
#[cfg(target_os = "solana")]
use pinocchio::pubkey::create_program_address;

const MAX_ACCOUNT_SIZE: u64 = 10 * 1024 * 1024; // 10MB limit

/// Extract a 32-byte owner pubkey from a [`ValueRef`].
///
/// Only `ValueRef::TempRef` is supported. The bytes are copied from the
/// context's temporary buffer into a stack-allocated array to avoid heap usage.
fn extract_owner_pubkey(owner_ref: ValueRef, ctx: &ExecutionManager) -> CompactResult<[u8; 32]> {
    match owner_ref {
        ValueRef::TempRef(offset, len) => {
            if len != 32 {
                return Err(VMErrorCode::TypeMismatch);
            }
            let start = offset as usize;
            let end = start + 32;
            let buf = ctx.temp_buffer();
            if end > buf.len() {
                return Err(VMErrorCode::MemoryViolation);
            }
            let mut pubkey = [0u8; 32];
            pubkey.copy_from_slice(&buf[start..end]);
            Ok(pubkey)
        }
        // Zero (0) indicates usage of the current program ID as owner
        ValueRef::U64(0) => Ok(ctx.program_id), // Use program_id field directly
        _ => Err(VMErrorCode::TypeMismatch),
    }
}

/// Handle account initialization operations
#[inline(always)]
pub fn handle_init_ops(opcode: u8, ctx: &mut ExecutionManager) -> CompactResult<()> {
    match opcode {
        INIT_ACCOUNT => handle_init_account(ctx),
        INIT_PDA_ACCOUNT => handle_init_pda_account(ctx),
        _ => Err(VMErrorCode::InvalidOpcode),
    }
}

/// Handle INIT_ACCOUNT opcode - Create regular account via System Program
///
/// Stack layout: [account_idx, space, payer_idx, lamports, owner_pubkey]
///
/// This creates a new account owned by the specified program with the given space and lamports.
/// The account must not already be initialized (checked by CHECK_UNINITIALIZED).
fn handle_init_account(ctx: &mut ExecutionManager) -> CompactResult<()> {
    // Pop parameters from stack
    let account_idx = ctx.pop()?.as_account_idx().ok_or(VMErrorCode::TypeMismatch)?;
    let space = ctx.pop()?.as_u64().ok_or(VMErrorCode::TypeMismatch)?;
    let payer_idx = ctx.pop()?.as_u8().ok_or(VMErrorCode::TypeMismatch)?;
    let lamports = ctx.pop()?.as_u64().ok_or(VMErrorCode::TypeMismatch)?;
    let owner_ref = ctx.pop()?;

    debug_log!(
        "INIT_ACCOUNT: account_idx={} space={} payer_idx={} lamports={} num_accounts={}",
        account_idx,
        space,
        payer_idx,
        lamports,
        ctx.accounts().len() as u32
    );

    debug_log!(
        "MitoVM: INIT_ACCOUNT starting - account_idx={}, space={}, payer_idx={}, lamports={}, num_accounts={}",
        account_idx,
        space,
        payer_idx,
        lamports,
        ctx.accounts().len() as u32
    );

    // Validate account index
    if account_idx >= ctx.accounts().len() as u8 {
        debug_log!(
            "MitoVM: INIT_ACCOUNT ERROR - account_idx {} >= num_accounts {}",
            account_idx,
            ctx.accounts().len() as u32
        );
        return Err(VMErrorCode::InvalidAccountIndex);
    }

    // Log account info before creation
    let account = ctx.get_account_unchecked(account_idx)?;
    let _key_bytes = account.key();
    debug_log!(
        "INIT_ACCOUNT: target account key {} {} {} {} data_len_before={}",
        _key_bytes[0], _key_bytes[1], _key_bytes[2], _key_bytes[3],
        account.data_len() as u32
    );

    debug_log!(
        "MitoVM: INIT_ACCOUNT target key: {} {} {} {}, data_len_before={}",
        _key_bytes[0], _key_bytes[1], _key_bytes[2], _key_bytes[3],
        account.data_len() as u32
    );

    // CRITICAL DEBUG: Capture pointer BEFORE modification
    let ptr_before = unsafe { account.borrow_data_unchecked().as_ptr() as usize };

    // Validate space parameter
    if space > MAX_ACCOUNT_SIZE {
        return Err(VMErrorCode::InvalidParameter);
    }

    // Extract owner pubkey from ValueRef
    let owner = extract_owner_pubkey(owner_ref, ctx)?;

    // Create account via System Program CPI with the actual target size
    // NOTE: The previous "Create 0-size + Resize" pattern was removed because
    // Solana's runtime doesn't allow modifying account data/metadata after
    // ownership transfer within the same instruction. This caused
    // "ExternalAccountDataModified" errors.
    match ctx.create_account_with_payer(account_idx, payer_idx, space, lamports, &owner) {
        Ok(()) => {
            debug_log!("INIT_ACCOUNT: SUCCESS with payer {} - created account {} with {} bytes", payer_idx, account_idx, space);
        }
        Err(e) => {
            debug_log!("INIT_ACCOUNT: FAILED - account {}", account_idx);
            return Err(e);
        }
    }


    // Log account info after creation
    let _account_after = ctx.get_account(account_idx)?;
    debug_log!(
        "INIT_ACCOUNT: after - data_len={} is_writable={}",
        _account_after.data_len() as u32,
        if _account_after.is_writable() { 1u8 } else { 0u8 }
    );

    debug_log!(
        "MitoVM: INIT_ACCOUNT completed for account {} - data_len_after={}, is_writable={}",
        account_idx,
        _account_after.data_len() as u32,
        if _account_after.is_writable() { 1u8 } else { 0u8 }
    );

    // CRITICAL DEBUG: Log pointers to detect stale references
    let ptr_after = unsafe { _account_after.borrow_data_unchecked().as_ptr() as usize };
    debug_log!(
        "INIT_ACCOUNT_PTRS: idx={} ptr_before={} ptr_after={} changed={}",
        account_idx,
        ptr_before,
        ptr_after,
        if ptr_before != ptr_after { 1u8 } else { 0u8 }
    );

    // NOTE: After CreateAccount CPI, the account data pointers may become stale.
    // The Solana runtime maintains the accounts array, and subsequent calls to
    // ctx.get_account() will return fresh AccountInfo references. This ensures
    // that any STORE_FIELD operations on this account after INIT_ACCOUNT will use
    // the correct (non-stale) pointers to the reallocated account data.
    // See: https://github.com/solana-labs/solana-program-library/issues/xxxx
    // (Pinocchio's data_ptr() is recalculated on each call to borrow_data_unchecked)

    Ok(())
}

/// Handle INIT_PDA_ACCOUNT opcode - Create PDA account via System Program
///
/// Stack layout: [account_idx, space, payer_idx, lamports, owner_pubkey, seeds_count, seed1, seed2, ..., bump]
///
/// This creates a new Program Derived Address account using the provided seeds and bump.
/// The PDA address is deterministically derived and the account is created with the specified parameters.
fn handle_init_pda_account(ctx: &mut ExecutionManager) -> CompactResult<()> {
    const RESERVED_FEE_VAULT_NAMESPACE: &[u8] = b"\xFFfive_vm_fee_vault_v1";
    // Pop basic parameters
    let account_idx = ctx.pop()?.as_account_idx().ok_or(VMErrorCode::TypeMismatch)?;
    let space = ctx.pop()?.as_u64().ok_or(VMErrorCode::TypeMismatch)?;
    let payer_idx = ctx.pop()?.as_u8().ok_or(VMErrorCode::TypeMismatch)?;
    let lamports = ctx.pop()?.as_u64().ok_or(VMErrorCode::TypeMismatch)?;
    let owner_ref = ctx.pop()?;
    let seeds_count = ctx.pop()?.as_u8().ok_or(VMErrorCode::TypeMismatch)?;

    // Validate parameters
    // Validate parameters
    if account_idx >= ctx.accounts().len() as u8 {
        return Err(VMErrorCode::InvalidAccountIndex);
    }

    debug_log!("INIT_PDA_ACCOUNT: Checking space. input={}, limit={}", space, MAX_ACCOUNT_SIZE);
    if space > MAX_ACCOUNT_SIZE {
        debug_log!("INIT_PDA_ACCOUNT: Space limit exceeded! {} > {}", space, MAX_ACCOUNT_SIZE);
        return Err(VMErrorCode::InvalidParameter);
    }

    const MAX_SEEDS: usize = 8;
    if seeds_count as usize > MAX_SEEDS {
        return Err(VMErrorCode::TooManySeeds);
    }

    if ctx.size() < seeds_count as usize {
        return Err(VMErrorCode::StackError);
    }
    // Collect seeds and restore original order
    let mut seeds: Vec<Vec<u8, 32>, MAX_SEEDS> = Vec::new();
    for _ in 0..seeds_count {
        seeds
            .push(value_ref_to_seed_bytes(ctx.pop()?, ctx, None)?)
            .unwrap();
    }
    seeds.reverse();
    if seeds
        .iter()
        .any(|seed| seed.as_slice() == RESERVED_FEE_VAULT_NAMESPACE)
    {
        return Err(VMErrorCode::InvalidSeedArray);
    }

    // Pop bump seed
    let bump = ctx.pop()?.as_u8().ok_or(VMErrorCode::TypeMismatch)?;

    // Extract owner pubkey
    let owner = extract_owner_pubkey(owner_ref, ctx)?;
    
    // Log the owner for debugging
    let owner_bytes = owner.as_ref();
    debug_log!(
        "INIT_PDA_ACCOUNT: owner={} {} {} {}",
        owner_bytes[0], owner_bytes[1], owner_bytes[2], owner_bytes[3]
    );

    // Create PDA account via System Program CPI (runtime integration required)
    // Convert seeds to slice references without heap allocation
    let mut seed_refs: Vec<&[u8], MAX_SEEDS> = Vec::new();
    for seed in seeds.iter() {
        seed_refs
            .push(seed.as_slice())
            .map_err(|_| VMErrorCode::TooManySeeds)?;
    }
    ctx.create_pda_account(
        account_idx,
        seed_refs.as_slice(),
        bump,
        space,
        lamports,
        &owner,
        payer_idx,
    )?;

    // Validate that the created account address matches the derived PDA
    {
        // Construct full seeds list including bump for validation
        let binding = [bump];
        let mut validation_seeds: Vec<&[u8], {MAX_SEEDS + 1}> = Vec::new();
        for s in seed_refs.iter() {
           validation_seeds.push(*s).unwrap();
        }
        validation_seeds.push(&binding).unwrap();

        let account = ctx.get_account(account_idx)?;
        
        #[cfg(target_os = "solana")]
        {
             let expected_pda = create_program_address(validation_seeds.as_slice(), &Pubkey::from(ctx.program_id))
                 .map_err(|_| VMErrorCode::InvokeError)?;
             
             if account.key() != &expected_pda {
                 debug_log!("INIT_PDA_ACCOUNT: Account address mismatch! Expected PDA but got different address");
                 return Err(VMErrorCode::AccountError);
             }
        }

        #[cfg(not(target_os = "solana"))]
        {
             let expected_pda = crate::utils::derive_pda_offchain(validation_seeds.as_slice(), &ctx.program_id)?;

             if account.key() != &expected_pda {
                  debug_log!("INIT_PDA_ACCOUNT: Account address mismatch! Computed PDA does not match account key");
                  return Err(VMErrorCode::AccountError);
             }
        }
    }

    debug_log!("MitoVM: INIT_PDA_ACCOUNT completed for account {} with {} seeds, bump {}, space {}, lamports {}",
               account_idx, seeds_count, bump, space, lamports);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::extract_owner_pubkey;
    use crate::context::ExecutionManager;
    use crate::stack::StackStorage;
    use crate::utils::value_ref_to_seed_bytes;
    use crate::error::VMErrorCode;
    use five_protocol::ValueRef;

    fn create_test_context() -> ExecutionManager<'static> {
        let script: &'static [u8] = &[];
        let accounts: &'static [pinocchio::account_info::AccountInfo] = &[];
        let input_data: &'static [u8] = b"test_seed";
        use pinocchio::pubkey::Pubkey;
        let program_id = Pubkey::default();
        let storage = Box::leak(Box::new(StackStorage::new()));
        ExecutionManager::new(script, accounts, program_id, input_data, 0, storage, 0, 0, 0, 0, 0, 0)
    }

    #[test]
    fn test_value_ref_to_seed_bytes_u64() {
        let mut ctx = create_test_context();
        let value = ValueRef::U64(42);
        let result = value_ref_to_seed_bytes(value, &mut ctx, None).unwrap();
        assert_eq!(result.as_slice(), 42u64.to_le_bytes().as_slice());
    }

    #[test]
    fn test_value_ref_to_seed_bytes_u8() {
        let mut ctx = create_test_context();
        let value = ValueRef::U8(123);
        let result = value_ref_to_seed_bytes(value, &mut ctx, None).unwrap();
        assert_eq!(result.as_slice(), &[123]);
    }

    #[test]
    fn test_value_ref_to_seed_bytes_temp_ref() {
        let mut ctx = create_test_context();

        // Write test data to temp buffer
        ctx.temp_buffer_mut()[0] = 0x42;
        ctx.temp_buffer_mut()[1] = 0x43;
        ctx.temp_buffer_mut()[2] = 0x44;
        ctx.temp_buffer_mut()[3] = 0x45;

        let value = ValueRef::TempRef(0, 4);
        let result = value_ref_to_seed_bytes(value, &mut ctx, None).unwrap();
        assert_eq!(result.as_slice(), &[0x42, 0x43, 0x44, 0x45]);
    }

    #[test]
    fn test_value_ref_to_seed_bytes_input_ref() {
        let mut ctx = create_test_context();
        // input_data is "test_seed"

        let value = ValueRef::InputRef(0);
        let result = value_ref_to_seed_bytes(value, &mut ctx, None).unwrap();
        assert_eq!(result.as_slice(), b"test_seed");
    }

    #[test]
    fn test_value_ref_to_seed_bytes_input_ref_offset() {
        let mut ctx = create_test_context();
        // input_data is "test_seed", offset 5 = "seed"

        let value = ValueRef::InputRef(5);
        let result = value_ref_to_seed_bytes(value, &mut ctx, Some(4)).unwrap();
        assert_eq!(result.as_slice(), b"seed");
    }

    #[test]
    fn test_value_ref_to_seed_bytes_input_ref_offset_out_of_bounds() {
        let mut ctx = create_test_context();
        let value = ValueRef::InputRef(9);
        let result = value_ref_to_seed_bytes(value, &mut ctx, Some(1));
        assert!(matches!(result, Err(VMErrorCode::MemoryViolation)));
    }

    #[test]
    fn test_value_ref_to_seed_bytes_input_ref_length_out_of_bounds() {
        let mut ctx = create_test_context();
        let value = ValueRef::InputRef(5);
        let result = value_ref_to_seed_bytes(value, &mut ctx, Some(5));
        assert!(matches!(result, Err(VMErrorCode::MemoryViolation)));
    }

    #[test]
    fn test_extract_owner_pubkey_temp_ref() {
        let mut ctx = create_test_context();
        for i in 0..32 {
            ctx.temp_buffer_mut()[i] = i as u8;
        }
        let value = ValueRef::TempRef(0, 32);
        let result = extract_owner_pubkey(value, &ctx).unwrap();
        let mut expected = [0u8; 32];
        for i in 0..32 {
            expected[i] = i as u8;
        }
        assert_eq!(result, expected);
    }
}
