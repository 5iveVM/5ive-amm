use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, ProgramResult,
};

use crate::{
    common::verify_program_owned,
    state::FIVEVMState,
};

use super::{require_min_accounts, require_signer};

/// Transfer fee from payer to recipient
pub fn transfer_fee(payer: &AccountInfo, recipient: &AccountInfo, amount: u64, system_program: Option<&AccountInfo>) -> ProgramResult {
    if amount == 0 || payer.key() == recipient.key() {
        return Ok(());
    }

    if payer.lamports() < amount {
        return Err(ProgramError::InsufficientFunds);
    }

    // Check if payer is a system account
    let system_program_id = [0u8; 32];
    if payer.owner() == &system_program_id {
        // Must use CPI
        let system_program = system_program.ok_or(ProgramError::MissingRequiredSignature)?; // Just borrow error code

        // Manual instruction construction for Transfer (discriminator 2)
        let mut data = [0u8; 12];
        data[0] = 2; // Transfer discriminator (u32 little endian: 2, 0, 0, 0)
        let amount_bytes = amount.to_le_bytes();
        data[4..12].copy_from_slice(&amount_bytes);

        let instruction = pinocchio::instruction::Instruction {
            program_id: system_program.key(),
            accounts: &[
                pinocchio::instruction::AccountMeta {
                    pubkey: payer.key(),
                    is_signer: true,
                    is_writable: true,
                },
                pinocchio::instruction::AccountMeta {
                    pubkey: recipient.key(),
                    is_signer: false,
                    is_writable: true,
                },
            ],
            data: &data,
        };

        pinocchio::program::invoke(&instruction, &[payer, recipient, system_program])?;
    } else {
        // Program-owned account (direct modification)
        // Verify we own it
        // Note: We don't check program_id here as we might be in a different context,
        // but generally only owned accounts can be modified.

        *payer.try_borrow_mut_lamports()? -= amount;

        // Use checked_add to prevent overflow in recipient lamports
        let new_recipient_lamports = recipient.lamports()
            .checked_add(amount)
            .ok_or(ProgramError::ArithmeticOverflow)?;
        *recipient.try_borrow_mut_lamports()? = new_recipient_lamports;
    }

    Ok(())
}

/// Collect deployment fee if configured in VM state
pub fn collect_deploy_fee(
    vm_state_account: &AccountInfo,
    accounts: &[AccountInfo],
    payer: &AccountInfo,
    _total_script_size: usize,
) -> ProgramResult {
    // SAFETY: The state account is program-owned and read-only here.
    let vm_state_data = unsafe { vm_state_account.borrow_data_unchecked() };
    let vm_state = FIVEVMState::from_account_data(&vm_state_data)?;

    let deploy_fee_lamports = vm_state.deploy_fee_lamports as u64;
    if deploy_fee_lamports > 0 {
        let admin_key = vm_state.authority;
        let admin_account = accounts.iter().find(|a| *a.key() == admin_key);

        if let Some(recipient) = admin_account {
            let system_program = accounts.iter().find(|a| a.key().as_ref() == &[0u8; 32]);
            transfer_fee(payer, recipient, deploy_fee_lamports, system_program)?;
        } else {
            // If fee is required but admin not present, fail
            return Err(ProgramError::MissingRequiredSignature);
        }
    }
    Ok(())
}

/// Set the deployment and execution fees (lamports)
pub fn set_fees(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    deploy_fee_lamports: u32,
    execute_fee_lamports: u32,
) -> ProgramResult {
    require_min_accounts(accounts, 2)?;

    let vm_state_account = &accounts[0];
    let authority = &accounts[1];

    // Verify ownership
    verify_program_owned(vm_state_account, program_id)?;
    require_signer(authority)?;

    // Update VM state
    // SAFETY: The state account is program-owned and uniquely borrowed here.
    let vm_state_data = unsafe { vm_state_account.borrow_mut_data_unchecked() };
    let vm_state = FIVEVMState::from_account_data_mut(vm_state_data)?;

    // Verify authority matches
    if vm_state.authority != *authority.key() {
        return Err(ProgramError::Custom(0)); // Unauthorized
    }

    vm_state.deploy_fee_lamports = deploy_fee_lamports;
    vm_state.execute_fee_lamports = execute_fee_lamports;

    debug_log!("Fees updated successfully");
    Ok(())
}
