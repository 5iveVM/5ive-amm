use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, ProgramResult,
};

use crate::{
    common::{
        validate_vm_and_script_accounts, verify_fee_vault_account, has_permission, PERMISSION_POST_BYTECODE,
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

fn decode_execute_payload<'a>(payload: &'a [u8]) -> (u8, Option<u8>, &'a [u8]) {
    if payload.len() >= 4 && payload[0] == 0xFF && payload[1] == 0x53 {
        (payload[2], Some(payload[3]), &payload[4..])
    } else {
        (0, None, payload)
    }
}

/// Execute a script with optional pre/post bytecode hooks.
pub fn execute(program_id: &Pubkey, accounts: &[AccountInfo], params: &[u8]) -> ProgramResult {
    require_min_accounts(accounts, 5)?;

    let script_account = &accounts[0];
    let vm_state_account = &accounts[1];

    if let Err(e) = validate_vm_and_script_accounts(program_id, script_account, vm_state_account) {
         return Err(e);
    }

    // SAFETY: state account is program-owned and read-only here.
    let vm_state_data = unsafe { vm_state_account.borrow_data_unchecked() };
    let vm_state = FIVEVMState::from_account_data(&vm_state_data)?;
    let fee = vm_state.execute_fee_lamports as u64;
    let (fee_shard_index, fee_vault_bump, vm_params) = decode_execute_payload(params);
    let last = accounts.len() - 1;
    let fee_vault = &accounts[last - 1];
    let payer = &accounts[last - 2];
    let system_program = &accounts[last];

    verify_fee_vault_account(
        fee_vault,
        program_id,
        fee_shard_index,
        fee_vault_bump,
    )?;
    if system_program.key().as_ref() != &[0u8; 32] {
        return Err(ProgramError::InvalidArgument);
    }
    if !payer.is_signer() || !payer.is_writable() {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if !fee_vault.is_writable() {
        return Err(ProgramError::InvalidArgument);
    }
    transfer_fee(program_id, payer, fee_vault, fee, Some(system_program))?;
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
    if let Err(vm_error) = MitoVM::execute_direct(bytecode, vm_params, vm_accounts, program_id, &mut *storage) {
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

        if let Err(vm_error) = MitoVM::execute_direct(bytecode, vm_params, vm_accounts, program_id, &mut *storage_retry) {
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
        let (vm_key, _vm_bump) = crate::common::derive_canonical_vm_state_pda(&program_id).unwrap();
        let admin_key = Pubkey::from([24u8; 32]);
        let payer_key = Pubkey::from([25u8; 32]);
        let (fee_vault_key, _fee_vault_bump) =
            crate::common::derive_fee_vault_pda(&program_id, 0).unwrap();
        let system_owner = Pubkey::default();

        let mut script_lamports = 1_000_000;
        let mut vm_lamports = 1_000_000;
        let mut fee_vault_lamports = 1_000_000;
        let mut payer_lamports = 1_000_000;

        let mut script_data = vec![0u8; ScriptAccountHeader::LEN];
        let mut vm_data = [0u8; FIVEVMState::LEN];
        {
            let state = FIVEVMState::from_account_data_mut(&mut vm_data).unwrap();
            state.initialize(admin_key);
            state.execute_fee_lamports = 1;
        }
        let mut fee_vault_data = [];
        let mut payer_data = [];
        let mut system_lamports = 1;
        let mut system_data = [];

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
        let fee_vault = create_account_info(
            &fee_vault_key,
            false,
            true,
            &mut fee_vault_lamports,
            &mut fee_vault_data,
            &program_id,
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
        let system_program = create_account_info(
            &system_owner,
            false,
            false,
            &mut system_lamports,
            &mut system_data,
            &system_owner,
        );

        let accounts = [script, vm, readonly_signer, fee_vault, system_program];
        let result = execute(&program_id, &accounts, &[]);
        assert_eq!(result, Err(ProgramError::MissingRequiredSignature));
    }
}
