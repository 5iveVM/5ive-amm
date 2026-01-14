use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, ProgramResult,
};

use crate::{
    common::verify_program_owned,
    debug_log,
    state::FIVEVMState,
};

use super::{require_min_accounts, require_signer};

/// Standard transaction fee in lamports (for fee calculation basis)
pub const STANDARD_TX_FEE: u64 = 5000;

/// Calculate fee based on amount and basis points (bps)
/// fee = (amount * bps) / 10000
pub fn calculate_fee(amount: u64, bps: u32) -> u64 {
    ((amount as u128 * bps as u128) / 10000) as u64
}

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

/// Set the deployment and execution fees (BPS)
pub fn set_fees(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    deploy_fee_bps: u32,
    execute_fee_bps: u32,
) -> ProgramResult {
    debug_log!(
        "FIVE VM: SetFees - deploy={}bps, execute={}bps",
        deploy_fee_bps, execute_fee_bps
    );

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

    vm_state.deploy_fee_bps = deploy_fee_bps;
    vm_state.execute_fee_bps = execute_fee_bps;

    debug_log!("Fees updated successfully");
    Ok(())
}
