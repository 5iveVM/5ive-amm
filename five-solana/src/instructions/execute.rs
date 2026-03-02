use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, ProgramResult,
};

use crate::{
    common::{
        has_permission, verify_hardcoded_fee_vault_account, verify_hardcoded_vm_state_account,
        verify_program_owned, PERMISSION_POST_BYTECODE,
    },
    error,
    state::{FIVEVMState, ScriptAccountHeader},
};
use five_vm_mito::error::VMErrorCode;
#[cfg(feature = "debug-logs")]
use five_vm_mito::VMError;
use five_vm_mito::{MitoVM, StackStorage};

use super::{
    fees::{should_bypass_fee_path, transfer_fee, validate_fee_transfer_accounts},
    require_min_accounts,
};

fn decode_execute_payload<'a>(payload: &'a [u8]) -> (u8, &'a [u8]) {
    if payload.len() >= 3 && payload[0] == 0xFF && payload[1] == 0x53 {
        (payload[2], &payload[3..])
    } else {
        (0, payload)
    }
}

/// Execute a script with optional pre/post bytecode hooks.
pub fn execute(program_id: &Pubkey, accounts: &[AccountInfo], params: &[u8]) -> ProgramResult {
    require_min_accounts(accounts, 5)?;

    let script_account = &accounts[0];
    let vm_state_account = &accounts[1];

    verify_program_owned(script_account, program_id).map_err(|_| ProgramError::Custom(7801))?;
    verify_hardcoded_vm_state_account(vm_state_account, program_id)
        .map_err(|_| ProgramError::Custom(7802))?;
    verify_program_owned(vm_state_account, program_id).map_err(|_| ProgramError::Custom(7803))?;

    // SAFETY: state account was verified program-owned and read-only here.
    let vm_state_data = unsafe { vm_state_account.borrow_data_unchecked() };
    let vm_state =
        FIVEVMState::from_account_data(&vm_state_data).map_err(|_| ProgramError::Custom(7804))?;
    if !vm_state.is_initialized() {
        return Err(error::program_not_initialized_error());
    }
    let fee = vm_state.execute_fee_lamports as u64;
    let (fee_shard_index, vm_params) = decode_execute_payload(params);
    if !should_bypass_fee_path(fee_shard_index) {
        if fee_shard_index >= vm_state.fee_vault_shard_count() {
            return Err(ProgramError::InvalidInstructionData);
        }
        let last = accounts.len() - 1;
        let fee_vault = &accounts[last - 1];
        let payer = &accounts[last - 2];
        let system_program = &accounts[last];

        verify_hardcoded_fee_vault_account(fee_vault, program_id, fee_shard_index)?;
        if system_program.key().as_ref() != &[0u8; 32] {
            return Err(ProgramError::Custom(7806));
        }
        validate_fee_transfer_accounts(program_id, payer, fee_vault, system_program)
            .map_err(|_| ProgramError::Custom(7807))?;
        transfer_fee(program_id, payer, fee_vault, fee, Some(system_program))
            .map_err(|_| ProgramError::Custom(7808))?;
    }
    // VM sees [vm_state, ...remaining execution accounts].
    let vm_accounts = &accounts[1..];

    // Parse script header from script account
    let script_data = unsafe { script_account.borrow_data_unchecked() };

    let header = ScriptAccountHeader::from_account_data(&script_data)
        .map_err(|_| ProgramError::Custom(7809))?;

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
    if let Err(vm_error) =
        MitoVM::execute_direct(bytecode, vm_params, vm_accounts, program_id, &mut *storage)
    {
        pinocchio::msg!("MitoVM MAIN execution failed");
        pinocchio::log::sol_log_64(VMErrorCode::from(vm_error.clone()) as u64, 0, 0, 0, 0);
        return Err(vm_error.to_program_error());
    }

    // Run post-execution hook if permission is set
    if has_permission(header.permissions, PERMISSION_POST_BYTECODE) {
        debug_log!("POST-BYTECODE permission set; secondary full-bytecode replay disabled");
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
            state.initialize(admin_key, 0);
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
