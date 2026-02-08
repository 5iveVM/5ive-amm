use crate::{error, state::FIVEVMState};
use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, ProgramResult,
};

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
        let mut lamports = 0;

        // Case 1: Success
        {
            let mut script_data = vec![0u8; 100];
            let mut vm_data = vec![0u8; FIVEVMState::LEN];

            // Initialize VM state
            {
                let state = FIVEVMState::from_account_data_mut(&mut vm_data).unwrap();
                state.initialize(Pubkey::default());
            }

            let script_account = create_account_info(&Pubkey::default(), false, false, &mut lamports, &mut script_data, &program_id);
            let vm_account = create_account_info(&Pubkey::default(), false, false, &mut lamports, &mut vm_data, &program_id);

            assert_eq!(validate_vm_and_script_accounts(&program_id, &script_account, &vm_account), Ok(()));
        }

        // Case 2: VM not initialized
        {
            let mut script_data = vec![0u8; 100];
            let mut vm_data = vec![0u8; FIVEVMState::LEN];
            // No init

            let script_account = create_account_info(&Pubkey::default(), false, false, &mut lamports, &mut script_data, &program_id);
            let vm_account = create_account_info(&Pubkey::default(), false, false, &mut lamports, &mut vm_data, &program_id);

            assert_eq!(validate_vm_and_script_accounts(&program_id, &script_account, &vm_account), Err(error::program_not_initialized_error()));
        }

        // Case 3: Script not owned
        {
            let mut script_data = vec![0u8; 100];
            let mut vm_data = vec![0u8; FIVEVMState::LEN];

            // Initialize VM state
            {
                let state = FIVEVMState::from_account_data_mut(&mut vm_data).unwrap();
                state.initialize(Pubkey::default());
            }

            let other_owner = Pubkey::default();
            let script_not_owned = create_account_info(&Pubkey::default(), false, false, &mut lamports, &mut script_data, &other_owner);
            let vm_account = create_account_info(&Pubkey::default(), false, false, &mut lamports, &mut vm_data, &program_id);

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
pub fn validate_vm_and_script_accounts(
    program_id: &Pubkey,
    script_account: &AccountInfo,
    vm_state_account: &AccountInfo,
) -> ProgramResult {
    verify_program_owned(script_account, program_id)?;
    verify_program_owned(vm_state_account, program_id)?;
    let data = unsafe { vm_state_account.borrow_data_unchecked() };
    let state = FIVEVMState::from_account_data(data)?;
    if !state.is_initialized() {
        return Err(error::program_not_initialized_error());
    }
    Ok(())
}
