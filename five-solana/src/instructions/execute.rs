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
    fees::{calculate_fee, transfer_fee, STANDARD_TX_FEE},
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
    let fee = calculate_fee(STANDARD_TX_FEE, vm_state.execute_fee_bps);
    if fee > 0 {
        let admin_key = vm_state.authority;
        let mut admin_account: Option<&AccountInfo> = None;
        let mut payer_account: Option<&AccountInfo> = None;
        let mut system_program: Option<&AccountInfo> = None;

        // Skip [script, vm_state] by construction.
        for account in &accounts[2..] {
            if account.key().as_ref() == &[0u8; 32] {
                system_program = Some(account);
            }
            if *account.key() == admin_key {
                admin_account = Some(account);
            }
            if account.is_signer() {
                payer_account = match payer_account {
                    Some(current) if current.lamports() >= account.lamports() => Some(current),
                    _ => Some(account),
                };
            }
        }

        let recipient = admin_account.ok_or(ProgramError::Custom(1107))?;
        let payer = payer_account.ok_or(ProgramError::MissingRequiredSignature)?;
        transfer_fee(payer, recipient, fee, system_program)?;
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
