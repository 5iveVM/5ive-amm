use crate::{error, state::FIVEVMState};
use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, ProgramResult,
};
#[cfg(target_os = "solana")]
use pinocchio::pubkey::create_program_address;

pub const VM_STATE_SEED: &[u8] = b"vm_state";
// Reserved namespace for VM-level fee vault PDAs.
// This namespace is blocked from VM bytecode PDA creation paths.
pub const FEE_VAULT_SEED: &[u8] = b"\xFFfive_vm_fee_vault_v1";

#[cfg(not(target_os = "solana"))]
pub fn derive_canonical_vm_state_pda(program_id: &Pubkey) -> Result<(Pubkey, u8), ProgramError> {
    let mut pid = [0u8; 32];
    pid.copy_from_slice(program_id.as_ref());
    let (pda, bump) = five_vm_mito::utils::find_program_address_offchain(&[VM_STATE_SEED], &pid)
        .map_err(|_| ProgramError::InvalidArgument)?;
    Ok((Pubkey::from(pda), bump))
}

#[inline(always)]
pub fn validate_vm_state_pda_with_bump(
    vm_state_account: &AccountInfo,
    program_id: &Pubkey,
    bump: u8,
) -> ProgramResult {
    let bump_seed = [bump];
    #[cfg(not(target_os = "solana"))]
    {
        let mut pid = [0u8; 32];
        pid.copy_from_slice(program_id.as_ref());
        let pda = five_vm_mito::utils::derive_pda_offchain(&[VM_STATE_SEED, &bump_seed], &pid)
            .map_err(|_| ProgramError::InvalidArgument)?;
        if vm_state_account.key() != &Pubkey::from(pda) {
            return Err(ProgramError::InvalidArgument);
        }
        Ok(())
    }
    #[cfg(target_os = "solana")]
    {
        let expected = create_program_address(&[VM_STATE_SEED, &bump_seed], program_id)
            .map_err(|_| ProgramError::InvalidArgument)?;
        if vm_state_account.key() != &expected {
            return Err(ProgramError::InvalidArgument);
        }
        Ok(())
    }
}

#[cfg(target_os = "solana")]
pub fn derive_canonical_vm_state_pda(program_id: &Pubkey) -> Result<(Pubkey, u8), ProgramError> {
    for bump in (0u8..=255u8).rev() {
        let bump_seed = [bump];
        let seeds: &[&[u8]] = &[VM_STATE_SEED, &bump_seed];
        if let Ok(pda) = create_program_address(seeds, program_id) {
            return Ok((pda, bump));
        }
    }
    Err(ProgramError::InvalidArgument)
}

/// Admin key that can deploy bytecode with special permissions.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_permission() {
        let perm_pre = 0x01u8;
        let perm_post = 0x02u8;
        let perm_special = 0x04u8;

        // Test individual permissions
        assert!(has_permission(perm_pre, PERMISSION_PRE_BYTECODE));
        assert!(!has_permission(perm_pre, PERMISSION_POST_BYTECODE));
        assert!(!has_permission(perm_pre, PERMISSION_PDA_SPECIAL_CHARS));

        assert!(!has_permission(perm_post, PERMISSION_PRE_BYTECODE));
        assert!(has_permission(perm_post, PERMISSION_POST_BYTECODE));
        assert!(!has_permission(perm_post, PERMISSION_PDA_SPECIAL_CHARS));

        assert!(!has_permission(perm_special, PERMISSION_PRE_BYTECODE));
        assert!(!has_permission(perm_special, PERMISSION_POST_BYTECODE));
        assert!(has_permission(perm_special, PERMISSION_PDA_SPECIAL_CHARS));

        // Test combined permissions
        let combined = perm_pre | perm_post; // 0x03
        assert!(has_permission(combined, PERMISSION_PRE_BYTECODE));
        assert!(has_permission(combined, PERMISSION_POST_BYTECODE));
        assert!(!has_permission(combined, PERMISSION_PDA_SPECIAL_CHARS));

        // Test no permissions
        let no_perm = 0x00u8;
        assert!(!has_permission(no_perm, PERMISSION_PRE_BYTECODE));
        assert!(!has_permission(no_perm, PERMISSION_POST_BYTECODE));
        assert!(!has_permission(no_perm, PERMISSION_PDA_SPECIAL_CHARS));
    }

    #[test]
    fn test_validate_permissions() {
        // Valid permissions (bits 0-2 used, 3-7 must be 0)
        assert!(validate_permissions(0x00).is_ok()); // No permissions
        assert!(validate_permissions(0x01).is_ok()); // PRE_BYTECODE
        assert!(validate_permissions(0x02).is_ok()); // POST_BYTECODE
        assert!(validate_permissions(0x04).is_ok()); // PDA_SPECIAL_CHARS
        assert!(validate_permissions(0x07).is_ok()); // All three permissions

        // Invalid permissions (reserved bits set)
        assert!(validate_permissions(0x08).is_err()); // Bit 3 set
        assert!(validate_permissions(0x10).is_err()); // Bit 4 set
        assert!(validate_permissions(0xF8).is_err()); // All reserved bits set
        assert!(validate_permissions(0xFF).is_err()); // All bits set
    }

    #[test]
    fn test_admin_key_validation() {
        // Admin key should be different from default
        let default_key = Pubkey::default();
        let test_admin_key = [42u8; 32];

        assert_ne!(test_admin_key, default_key);

        // Verify the function signature works with the new parameter
        // (Just a compile-time check that the function is accessible)
        let _ = verify_admin_signer;
    }

    // Helper to create a fake AccountInfo
    fn create_account_info<'a>(
        key: &'a Pubkey,
        is_signer: bool,
        is_writable: bool,
        lamports: &'a mut u64,
        data: &'a mut [u8],
        owner: &'a Pubkey,
    ) -> AccountInfo {
        AccountInfo::new(key, is_signer, is_writable, lamports, data, owner, false, 0)
    }

    #[test]
    fn test_verify_admin_signer_checks() {
        let admin_key = Pubkey::from([1u8; 32]);
        let other_key = Pubkey::from([2u8; 32]);
        let owner = Pubkey::default();
        let mut lamports = 0;
        let mut data = [];

        // 1. Success: Admin key + Signer
        let admin_signer = create_account_info(&admin_key, true, false, &mut lamports, &mut data, &owner);
        assert_eq!(verify_admin_signer(&admin_signer, &admin_key), Ok(()));

        // 2. Fail: Admin key but not signer
        let admin_no_signer = create_account_info(&admin_key, false, false, &mut lamports, &mut data, &owner);
        assert_eq!(verify_admin_signer(&admin_no_signer, &admin_key), Err(ProgramError::MissingRequiredSignature));

        // 3. Fail: Not admin key (even if signer)
        let other_signer = create_account_info(&other_key, true, false, &mut lamports, &mut data, &owner);
        assert_eq!(verify_admin_signer(&other_signer, &admin_key), Err(ProgramError::InvalidArgument));
    }

    #[test]
    fn test_verify_program_owned_checks() {
        let program_id = Pubkey::from([3u8; 32]);
        let other_program = Pubkey::from([4u8; 32]);
        let key = Pubkey::default();
        let mut lamports = 0;
        let mut data = [];

        // 1. Success: Owned by program
        let owned_account = create_account_info(&key, false, false, &mut lamports, &mut data, &program_id);
        assert_eq!(verify_program_owned(&owned_account, &program_id), Ok(()));

        // 2. Fail: Not owned by program
        let not_owned_account = create_account_info(&key, false, false, &mut lamports, &mut data, &other_program);
        assert_eq!(verify_program_owned(&not_owned_account, &program_id), Err(ProgramError::IllegalOwner));
    }

    #[test]
    fn test_validate_vm_and_script_accounts_checks() {
        let program_id = Pubkey::from([5u8; 32]);
        let (canonical_vm_state, _bump) =
            derive_canonical_vm_state_pda(&program_id).unwrap();
        let mut lamports = 0;

        // Case 1: Success
        {
            let mut script_data = vec![0u8; 100];
            let mut vm_data = vec![0u8; FIVEVMState::LEN];

            // Initialize VM state
            {
                let state = FIVEVMState::from_account_data_mut(&mut vm_data).unwrap();
                state.initialize(Pubkey::default(), 0);
            }

            let script_account = create_account_info(&Pubkey::default(), false, false, &mut lamports, &mut script_data, &program_id);
            let vm_account = create_account_info(&canonical_vm_state, false, false, &mut lamports, &mut vm_data, &program_id);

            assert_eq!(validate_vm_and_script_accounts(&program_id, &script_account, &vm_account), Ok(()));
        }

        // Case 2: VM not initialized
        {
            let mut script_data = vec![0u8; 100];
            let mut vm_data = vec![0u8; FIVEVMState::LEN];
            // Stamp version but keep uninitialized flag to assert not-initialized path.
            {
                let state = FIVEVMState::from_account_data_mut(&mut vm_data).unwrap();
                state.version = FIVEVMState::VERSION;
                state.is_initialized = 0;
            }

            let script_account = create_account_info(&Pubkey::default(), false, false, &mut lamports, &mut script_data, &program_id);
            let vm_account = create_account_info(&canonical_vm_state, false, false, &mut lamports, &mut vm_data, &program_id);

            assert_eq!(validate_vm_and_script_accounts(&program_id, &script_account, &vm_account), Err(error::program_not_initialized_error()));
        }

        // Case 3: Script not owned
        {
            let mut script_data = vec![0u8; 100];
            let mut vm_data = vec![0u8; FIVEVMState::LEN];

            // Initialize VM state
            {
                let state = FIVEVMState::from_account_data_mut(&mut vm_data).unwrap();
                state.initialize(Pubkey::default(), 0);
            }

            let other_owner = Pubkey::default();
            let script_not_owned = create_account_info(&Pubkey::default(), false, false, &mut lamports, &mut script_data, &other_owner);
            let vm_account = create_account_info(&canonical_vm_state, false, false, &mut lamports, &mut vm_data, &program_id);

            assert_eq!(validate_vm_and_script_accounts(&program_id, &script_not_owned, &vm_account), Err(ProgramError::IllegalOwner));
        }
    }
}

pub const PERMISSION_PRE_BYTECODE: u8 = 0x01;         // Bit 0
pub const PERMISSION_POST_BYTECODE: u8 = 0x02;        // Bit 1
#[allow(dead_code)]
pub const PERMISSION_PDA_SPECIAL_CHARS: u8 = 0x04;    // Bit 2
const KNOWN_PERMISSIONS: u8 =
    PERMISSION_PRE_BYTECODE | PERMISSION_POST_BYTECODE | PERMISSION_PDA_SPECIAL_CHARS;

#[inline(always)]
pub fn has_permission(permissions: u8, permission: u8) -> bool {
    permissions & permission != 0
}

#[inline(always)]
pub fn validate_permissions(permissions: u8) -> ProgramResult {
    // Bits outside known permission flags must be zero (reserved for future use)
    if permissions & !KNOWN_PERMISSIONS != 0 {
        return Err(ProgramError::InvalidArgument);
    }
    Ok(())
}

// ============================================================================
// Admin Authorization
// ============================================================================

/// Checks if the given account is the admin key and is a signer.
/// Returns error if:
/// - The account is not the admin key, OR
/// - The account is not a signer
#[inline(always)]
pub fn verify_admin_signer(account: &AccountInfo, admin_key: &Pubkey) -> ProgramResult {
    if account.key() != admin_key {
        return Err(ProgramError::InvalidArgument);
    }
    if !account.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }
    Ok(())
}

// ============================================================================
// Account Validation
// ============================================================================

#[inline(always)]
pub fn verify_program_owned(account: &AccountInfo, program_id: &Pubkey) -> ProgramResult {
    // SAFETY: The Solana runtime guarantees that the account's owner pointer is
    // valid for the lifetime of this instruction. We only read the value.
    if account.owner() != program_id {
        #[cfg(feature = "debug-logs")]
        {
            pinocchio::log::sol_log("DEBUG: verify_program_owned FAILED");
            pinocchio::log::sol_log("Account owner mismatch - script/state account not owned by program");
        }
        return Err(ProgramError::IllegalOwner);
    }
    Ok(())
}

#[inline(always)]
pub fn verify_canonical_vm_state_account(
    vm_state_account: &AccountInfo,
    program_id: &Pubkey,
) -> ProgramResult {
    let data = vm_state_account.try_borrow_data().ok();
    if let Some(data) = data {
        if data.len() >= FIVEVMState::LEN {
            if let Ok(vm_state) = FIVEVMState::from_account_data(&data) {
                return validate_vm_state_pda_with_bump(vm_state_account, program_id, vm_state.vm_state_bump);
            }
        }
    }
    let (expected_vm_state, _bump) = derive_canonical_vm_state_pda(program_id)?;
    if vm_state_account.key() != &expected_vm_state {
        return Err(ProgramError::InvalidArgument);
    }
    Ok(())
}

#[inline(always)]
pub fn derive_fee_vault_pda(program_id: &Pubkey, shard_index: u8) -> Result<(Pubkey, u8), ProgramError> {
    let shard_seed = [shard_index];
    #[cfg(not(target_os = "solana"))]
    {
        let mut pid = [0u8; 32];
        pid.copy_from_slice(program_id.as_ref());
        let (pda, bump) = five_vm_mito::utils::find_program_address_offchain(
            &[FEE_VAULT_SEED, &shard_seed],
            &pid,
        )
        .map_err(|_| ProgramError::InvalidArgument)?;
        Ok((Pubkey::from(pda), bump))
    }
    #[cfg(target_os = "solana")]
    {
        for bump in (0u8..=255u8).rev() {
            let bump_seed = [bump];
            let seeds: &[&[u8]] = &[FEE_VAULT_SEED, &shard_seed, &bump_seed];
            if let Ok(pda) = create_program_address(seeds, program_id) {
                return Ok((pda, bump));
            }
        }
        Err(ProgramError::InvalidArgument)
    }
}

#[inline(always)]
pub fn derive_fee_vault_pda_with_bump(
    program_id: &Pubkey,
    shard_index: u8,
    bump: u8,
) -> Result<Pubkey, ProgramError> {
    let shard_seed = [shard_index];
    let bump_seed = [bump];
    #[cfg(not(target_os = "solana"))]
    {
        let mut pid = [0u8; 32];
        pid.copy_from_slice(program_id.as_ref());
        let pda = five_vm_mito::utils::derive_pda_offchain(
            &[FEE_VAULT_SEED, &shard_seed, &bump_seed],
            &pid,
        )
        .map_err(|_| ProgramError::InvalidArgument)?;
        Ok(Pubkey::from(pda))
    }
    #[cfg(target_os = "solana")]
    {
        create_program_address(&[FEE_VAULT_SEED, &shard_seed, &bump_seed], program_id)
            .map_err(|_| ProgramError::InvalidArgument)
    }
}

#[inline(always)]
pub fn verify_fee_vault_account(
    fee_vault_account: &AccountInfo,
    program_id: &Pubkey,
    shard_index: u8,
    expected_bump: Option<u8>,
) -> ProgramResult {
    if let Some(bump) = expected_bump {
        let expected_key = derive_fee_vault_pda_with_bump(program_id, shard_index, bump)?;
        if fee_vault_account.key() != &expected_key {
            return Err(ProgramError::InvalidArgument);
        }
    } else {
        let (expected_key, _derived_bump) = derive_fee_vault_pda(program_id, shard_index)?;
        if fee_vault_account.key() != &expected_key {
            return Err(ProgramError::InvalidArgument);
        }
    }
    if fee_vault_account.owner() != program_id {
        return Err(ProgramError::IllegalOwner);
    }
    Ok(())
}

#[inline(always)]
pub fn validate_vm_and_script_accounts(
    program_id: &Pubkey,
    script_account: &AccountInfo,
    vm_state_account: &AccountInfo,
) -> ProgramResult {
    verify_program_owned(script_account, program_id)?;
    verify_canonical_vm_state_account(vm_state_account, program_id)?;
    verify_program_owned(vm_state_account, program_id)?;
    let data = unsafe { vm_state_account.borrow_data_unchecked() };
    let state = FIVEVMState::from_account_data(data)?;
    if !state.is_initialized() {
        return Err(error::program_not_initialized_error());
    }
    Ok(())
}
