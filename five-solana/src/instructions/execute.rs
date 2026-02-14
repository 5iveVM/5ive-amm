use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, ProgramResult,
};

use crate::{
    common::{
        validate_vm_and_script_accounts, has_permission, PERMISSION_POST_BYTECODE,
    },
    state::{FIVEVMState, ScriptAccountHeader},
};
use five_vm_mito::{MitoVM, StackStorage};
#[cfg(feature = "debug-logs")]
use five_vm_mito::VMError;
#[cfg(feature = "debug-logs")]
use five_vm_mito::error::VMErrorCode;

use super::{
    fees::transfer_fee,
    require_min_accounts,
};

/// Execute a script with optional pre/post bytecode hooks.
pub fn execute(program_id: &Pubkey, accounts: &[AccountInfo], params: &[u8]) -> ProgramResult {
    require_min_accounts(accounts, 2)?;

    let script_account = &accounts[0];
    let vm_state_account = &accounts[1];

    if let Err(e) = validate_vm_and_script_accounts(program_id, script_account, vm_state_account) {
         return Err(e);
    }

    // SAFETY: state account is program-owned and read-only here.
    let vm_state_data = unsafe { vm_state_account.borrow_data_unchecked() };
    let vm_state = FIVEVMState::from_account_data(&vm_state_data)?;
    let fee = vm_state.execute_fee_lamports as u64;
    if fee > 0 {
        let fee_recipient_key = vm_state.fee_recipient;
        let mut fee_recipient_account: Option<&AccountInfo> = None;
        let mut payer_account: Option<&AccountInfo> = None;
        let mut system_program: Option<&AccountInfo> = None;

        // Skip [script, vm_state] by construction.
        for account in &accounts[2..] {
            if account.key().as_ref() == &[0u8; 32] {
                system_program = Some(account);
            }
            if *account.key() == fee_recipient_key && account.is_writable() {
                fee_recipient_account = Some(account);
            }
            if account.is_signer() && account.is_writable() {
                payer_account = match payer_account {
                    Some(current) if current.lamports() >= account.lamports() => Some(current),
                    _ => Some(account),
                };
            }
        }

        let recipient = fee_recipient_account.ok_or(ProgramError::Custom(1110))?;
        let payer = payer_account.ok_or(ProgramError::MissingRequiredSignature)?;
        transfer_fee(program_id, payer, recipient, fee, system_program)?;
    }
    // VM sees [vm_state, ...remaining execution accounts].
    let vm_accounts = &accounts[1..];

    // Parse script header from script account
    let script_data = unsafe { script_account.borrow_data_unchecked() };

    let header = ScriptAccountHeader::from_account_data(&script_data)?;

    if header.upload_mode() && !header.upload_complete() {
        return Err(ProgramError::Custom(7001));
    }
    // Validate header
    let bytecode_len = header.bytecode_len();

    let required_len = ScriptAccountHeader::LEN + bytecode_len as usize + header.metadata_len();
    if script_data.len() < required_len {
        return Err(ProgramError::Custom(7003));
    }

    // Extract bytecode slice
    let bytecode_start = ScriptAccountHeader::LEN + header.metadata_len();
    let bytecode_end = bytecode_start + bytecode_len;

    let bytecode = &script_data[bytecode_start..bytecode_end];

    // Initialize VM Storage using optimized heap allocation
    // Uses new_on_heap() which constructs directly in heap memory to avoid stack overflow
    let mut storage = StackStorage::new_on_heap();
    if let Err(vm_error) = MitoVM::execute_direct(bytecode, params, vm_accounts, program_id, &mut *storage) {
        #[cfg(feature = "debug-logs")]
        debug_log!(
            "MitoVM MAIN execution failed code={}",
            VMErrorCode::from(vm_error.clone()).message()
        );
        return Err(vm_error.to_program_error());
    }

    // Run post-execution hook if permission is set
    if has_permission(header.permissions, PERMISSION_POST_BYTECODE) {
        debug_log!("Running POST-BYTECODE hook");

        // Allocate new optimized heap storage for retry
        let mut storage_retry = StackStorage::new_on_heap();

        if let Err(vm_error) = MitoVM::execute_direct(bytecode, params, vm_accounts, program_id, &mut *storage_retry) {
            #[cfg(feature = "debug-logs")]
            debug_log!(
                "MitoVM POST hook failed code={}",
                VMErrorCode::from(vm_error.clone()).message()
            );
            return Err(vm_error.to_program_error());
        }
    }

    debug_log!("Script executed successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pinocchio::account_info::AccountInfo;

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
    fn execute_fee_ignores_readonly_signer_as_payer() {
        let program_id = Pubkey::from([21u8; 32]);
        let script_key = Pubkey::from([22u8; 32]);
        let vm_key = Pubkey::from([23u8; 32]);
        let admin_key = Pubkey::from([24u8; 32]);
        let payer_key = Pubkey::from([25u8; 32]);
        let system_owner = Pubkey::default();

        let mut script_lamports = 1_000_000;
        let mut vm_lamports = 1_000_000;
        let mut admin_lamports = 1_000_000;
        let mut payer_lamports = 1_000_000;

        let mut script_data = vec![0u8; ScriptAccountHeader::LEN];
        let mut vm_data = [0u8; FIVEVMState::LEN];
        {
            let state = FIVEVMState::from_account_data_mut(&mut vm_data).unwrap();
            state.initialize(admin_key);
            state.execute_fee_lamports = 1;
        }
        let mut admin_data = [];
        let mut payer_data = [];

        let script = create_account_info(
            &script_key,
            false,
            true,
            &mut script_lamports,
            script_data.as_mut_slice(),
            &program_id,
        );
        let vm = create_account_info(
            &vm_key,
            false,
            true,
            &mut vm_lamports,
            &mut vm_data,
            &program_id,
        );
        let admin = create_account_info(
            &admin_key,
            false,
            true,
            &mut admin_lamports,
            &mut admin_data,
            &system_owner,
        );
        // Readonly signer: must NOT be accepted as fee payer.
        let readonly_signer = create_account_info(
            &payer_key,
            true,
            false,
            &mut payer_lamports,
            &mut payer_data,
            &system_owner,
        );

        let accounts = [script, vm, admin, readonly_signer];
        let result = execute(&program_id, &accounts, &[]);
        assert_eq!(result, Err(ProgramError::MissingRequiredSignature));
    }
}
