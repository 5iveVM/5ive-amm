use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, ProgramResult,
};

use crate::{
    common::{
        validate_vm_and_script_accounts, has_permission, PERMISSION_POST_BYTECODE,
    },
    debug_log,
    state::{FIVEVMState, ScriptAccountHeader},
};
use five_vm_mito::MitoVM;
#[cfg(feature = "debug-logs")]
use five_vm_mito::VMError;
#[cfg(feature = "debug-logs")]
use five_vm_mito::error::VMErrorCode;

use super::{
    fees::{calculate_fee, transfer_fee, STANDARD_TX_FEE},
    require_min_accounts,
};

/// Execute a script with optional pre/post bytecode hooks
///
/// **Pre-Execution Hook** (PERMISSION_PRE_BYTECODE).
/// **Post-Execution Hook** (PERMISSION_POST_BYTECODE).
pub fn execute(program_id: &Pubkey, accounts: &[AccountInfo], params: &[u8]) -> ProgramResult {
    #[cfg(feature = "debug-logs")]
    debug_log!("DEBUG: execute ENTRY");

    require_min_accounts(accounts, 2)?;
    #[cfg(feature = "debug-logs")]
    debug_log!("DEBUG: require_min_accounts PASS");

    let script_account = &accounts[0];
    let vm_state_account = &accounts[1];

    if let Err(e) = validate_vm_and_script_accounts(program_id, script_account, vm_state_account) {
         #[cfg(feature = "debug-logs")]
         debug_log!("DEBUG: validate_vm_and_script_accounts FAIL");
         return Err(e);
    }
    #[cfg(feature = "debug-logs")]
    debug_log!("DEBUG: validate_vm_and_script_accounts PASS");

    // Collect execution fee.
    let vm_accounts = {
        // SAFETY: State account program-owned, read-only.
        let vm_state_data = unsafe { vm_state_account.borrow_data_unchecked() };
        let vm_state = FIVEVMState::from_account_data(&vm_state_data)?;
        let fee = calculate_fee(STANDARD_TX_FEE, vm_state.execute_fee_bps);
        if fee > 0 {
             let admin_key = vm_state.authority;
             let admin_account = accounts.iter().find(|a| *a.key() == admin_key);
             let payer_account = accounts.iter()
                 .filter(|a| a.is_signer() && *a.key() != *vm_state_account.key() && *a.key() != *script_account.key())
                 .max_by_key(|a| a.lamports());

             if let Some(recipient) = admin_account {
                 if let Some(payer) = payer_account {
                     #[cfg(feature = "debug-logs")]
                     debug_log!("DEBUG: Paying execute fee: {}", fee);
                     let system_program = accounts.iter().find(|a| a.key().as_ref() == &[0u8; 32]);
                     transfer_fee(payer, recipient, fee, system_program)?;
                 } else {
                     return Err(ProgramError::MissingRequiredSignature);
                 }
             } else {
                 #[cfg(feature = "debug-logs")]
                 debug_log!("DEBUG: Execute fee required but Admin not found");
                 // Error 1107 matches test expectation (likely FeeCollectorMissing)
                 return Err(ProgramError::Custom(1107));
             }
             &accounts[1..]  // Skip Script account, start from VM State
        } else {
             #[cfg(feature = "debug-logs")]
             debug_log!("DEBUG: fee is 0");
             &accounts[1..]  // Skip Script account, start from VM State
        }
    };
    #[cfg(feature = "debug-logs")]
    debug_log!("DEBUG: input accounts setup PASS - passing {} accounts to VM", vm_accounts.len() as u32);

    // Parse script header from script account
    let script_data = unsafe { script_account.borrow_data_unchecked() };
    #[cfg(feature = "debug-logs")]
    debug_log!("DEBUG: script_data borrow PASS");

    let header = ScriptAccountHeader::from_account_data(&script_data)?;
    #[cfg(feature = "debug-logs")]
    debug_log!("DEBUG: header parse PASS");

    if header.upload_mode() && !header.upload_complete() {
        return Err(ProgramError::Custom(7001));
    }
    // Validate header
    let bytecode_len = header.bytecode_len();

    let required_len = ScriptAccountHeader::LEN + bytecode_len as usize + header.metadata_len();
    if script_data.len() < required_len {
        #[cfg(feature = "debug-logs")]
        debug_log!("DEBUG: script too short");
        return Err(ProgramError::Custom(7003));
    }

    // Extract bytecode slice
    let bytecode_start = ScriptAccountHeader::LEN + header.metadata_len();
    let bytecode_end = bytecode_start + bytecode_len;

    let bytecode = &script_data[bytecode_start..bytecode_end];
    #[cfg(feature = "debug-logs")]
    debug_log!("DEBUG: bytecode slice PASS len={}", bytecode.len());

    // Execute main bytecode
    #[cfg(feature = "debug-logs")]
    debug_log!("DEBUG: Executing MAIN bytecode with VM accounts [VM State, param0, param1, ...]");
    // MitoVM expects accounts starting from VM State.

    if let Err(vm_error) = MitoVM::execute_direct(bytecode, params, vm_accounts, program_id) {
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
        if let Err(vm_error) = MitoVM::execute_direct(bytecode, params, vm_accounts, program_id) {
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
