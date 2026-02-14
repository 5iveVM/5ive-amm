use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, ProgramResult,
};

use crate::{
    common::{verify_canonical_vm_state_account, verify_program_owned},
    state::FIVEVMState,
};

use super::{require_min_accounts, require_signer};

const ERR_FEE_RECIPIENT_MISSING: u32 = 1110;
const ERR_FEE_PAYER_EQUALS_RECIPIENT: u32 = 1111;
const ERR_INVALID_FEE_RECIPIENT: u32 = 1113;

/// Transfer fee from payer to recipient
pub fn transfer_fee(
    program_id: &Pubkey,
    payer: &AccountInfo,
    recipient: &AccountInfo,
    amount: u64,
    system_program: Option<&AccountInfo>,
) -> ProgramResult {
    if amount == 0 {
        return Ok(());
    }
    if payer.key() == recipient.key() {
        return Err(ProgramError::Custom(ERR_FEE_PAYER_EQUALS_RECIPIENT));
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
        if payer.owner() != program_id {
            return Err(ProgramError::IllegalOwner);
        }

        // Program-owned account (direct modification)
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
    program_id: &Pubkey,
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
        let recipient_key = vm_state.fee_recipient;
        let recipient_account = accounts
            .iter()
            .find(|a| *a.key() == recipient_key && a.is_writable());

        if let Some(recipient) = recipient_account {
            let system_program = accounts.iter().find(|a| a.key().as_ref() == &[0u8; 32]);
            transfer_fee(
                program_id,
                payer,
                recipient,
                deploy_fee_lamports,
                system_program,
            )?;
        } else {
            return Err(ProgramError::Custom(ERR_FEE_RECIPIENT_MISSING));
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
    verify_canonical_vm_state_account(vm_state_account, program_id)?;
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

/// Set fee recipient treasury account (authority only).
pub fn set_fee_recipient(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    fee_recipient: Pubkey,
) -> ProgramResult {
    require_min_accounts(accounts, 2)?;
    if fee_recipient == Pubkey::default() {
        return Err(ProgramError::Custom(ERR_INVALID_FEE_RECIPIENT));
    }

    let vm_state_account = &accounts[0];
    let authority = &accounts[1];

    verify_canonical_vm_state_account(vm_state_account, program_id)?;
    verify_program_owned(vm_state_account, program_id)?;
    require_signer(authority)?;

    let vm_state_data = unsafe { vm_state_account.borrow_mut_data_unchecked() };
    let vm_state = FIVEVMState::from_account_data_mut(vm_state_data)?;
    if vm_state.authority != *authority.key() {
        return Err(ProgramError::Custom(0));
    }

    vm_state.fee_recipient = fee_recipient;
    debug_log!("Fee recipient updated successfully");
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
    fn transfer_fee_rejects_non_program_owned_direct_debit() {
        let program_id = Pubkey::from([1u8; 32]);
        let foreign_owner = Pubkey::from([2u8; 32]);
        let recipient_owner = program_id;
        let payer_key = Pubkey::from([3u8; 32]);
        let recipient_key = Pubkey::from([4u8; 32]);

        let mut payer_lamports = 1_000;
        let mut recipient_lamports = 100;
        let mut payer_data = [];
        let mut recipient_data = [];

        let payer = create_account_info(
            &payer_key,
            true,
            true,
            &mut payer_lamports,
            &mut payer_data,
            &foreign_owner,
        );
        let recipient = create_account_info(
            &recipient_key,
            false,
            true,
            &mut recipient_lamports,
            &mut recipient_data,
            &recipient_owner,
        );

        let result = transfer_fee(&program_id, &payer, &recipient, 10, None);
        assert_eq!(result, Err(ProgramError::IllegalOwner));
    }

    #[test]
    fn transfer_fee_allows_program_owned_direct_debit() {
        let program_id = Pubkey::from([9u8; 32]);
        let payer_key = Pubkey::from([10u8; 32]);
        let recipient_key = Pubkey::from([11u8; 32]);

        let mut payer_lamports = 1_000;
        let mut recipient_lamports = 100;
        let mut payer_data = [];
        let mut recipient_data = [];

        let payer = create_account_info(
            &payer_key,
            true,
            true,
            &mut payer_lamports,
            &mut payer_data,
            &program_id,
        );
        let recipient = create_account_info(
            &recipient_key,
            false,
            true,
            &mut recipient_lamports,
            &mut recipient_data,
            &program_id,
        );

        let result = transfer_fee(&program_id, &payer, &recipient, 10, None);
        assert_eq!(result, Ok(()));
        assert_eq!(payer.lamports(), 990);
        assert_eq!(recipient.lamports(), 110);
    }
}
