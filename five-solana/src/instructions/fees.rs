use pinocchio::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction, Seed, Signer},
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvars::Sysvar,
    ProgramResult,
};

use crate::{
    common::{
        verify_hardcoded_fee_vault_account, verify_hardcoded_vm_state_account,
        verify_program_owned, FEE_VAULT_SEED,
    },
    state::FIVEVMState,
};

use super::{require_min_accounts, require_signer};

const ERR_INVALID_FEE_RECIPIENT: u32 = 1113;
pub const FEE_BYPASS_SHARD_INDEX: u8 = u8::MAX;

#[inline(always)]
pub(crate) fn validate_fee_transfer_accounts(
    program_id: &Pubkey,
    payer: &AccountInfo,
    fee_vault_account: &AccountInfo,
    system_program: &AccountInfo,
) -> ProgramResult {
    let _ = program_id;
    if system_program.key().as_ref() != &[0u8; 32] {
        return Err(ProgramError::InvalidArgument);
    }
    if !payer.is_signer() || !payer.is_writable() {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if !fee_vault_account.is_writable() {
        return Err(ProgramError::InvalidArgument);
    }
    Ok(())
}

#[inline(always)]
pub(crate) fn build_system_transfer_ix(amount: u64) -> [u8; 12] {
    let mut data = [0u8; 12];
    data[0] = 2; // transfer discriminator
    data[4..12].copy_from_slice(&amount.to_le_bytes());
    data
}

#[inline(always)]
pub(crate) fn build_system_create_account_ix(
    lamports: u64,
    space: u64,
    owner: &Pubkey,
) -> [u8; 52] {
    let mut data = [0u8; 52];
    data[0..4].copy_from_slice(&0u32.to_le_bytes()); // create_account discriminator
    data[4..12].copy_from_slice(&lamports.to_le_bytes());
    data[12..20].copy_from_slice(&space.to_le_bytes());
    data[20..52].copy_from_slice(owner.as_ref());
    data
}

#[inline(always)]
pub fn should_bypass_fee_path(fee_shard_index: u8) -> bool {
    #[cfg(feature = "cu-bypass-fees")]
    {
        return fee_shard_index == FEE_BYPASS_SHARD_INDEX;
    }
    #[cfg(not(feature = "cu-bypass-fees"))]
    {
        let _ = fee_shard_index;
        false
    }
}

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

    if payer.lamports() < amount {
        return Err(ProgramError::InsufficientFunds);
    }

    // Check if payer is a system account
    let system_program_id = [0u8; 32];
    if payer.owner() == &system_program_id {
        // Must use CPI
        let system_program = system_program.ok_or(ProgramError::MissingRequiredSignature)?; // Just borrow error code

        let data = build_system_transfer_ix(amount);

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
        let new_recipient_lamports = recipient
            .lamports()
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
    payer: &AccountInfo,
    fee_vault_account: &AccountInfo,
    system_program: &AccountInfo,
    fee_shard_index: u8,
    _total_script_size: usize,
) -> ProgramResult {
    // SAFETY: The state account is program-owned and read-only here.
    let vm_state_data = unsafe { vm_state_account.borrow_data_unchecked() };
    let vm_state = FIVEVMState::from_account_data(&vm_state_data)?;
    collect_deploy_fee_with_state(
        program_id,
        payer,
        fee_vault_account,
        system_program,
        fee_shard_index,
        vm_state.deploy_fee_lamports as u64,
        vm_state.fee_vault_shard_count(),
    )
}

pub fn collect_deploy_fee_with_state(
    program_id: &Pubkey,
    payer: &AccountInfo,
    fee_vault_account: &AccountInfo,
    system_program: &AccountInfo,
    fee_shard_index: u8,
    deploy_fee_lamports: u64,
    fee_vault_shard_count: u8,
) -> ProgramResult {
    if should_bypass_fee_path(fee_shard_index) {
        return Ok(());
    }

    if fee_shard_index >= fee_vault_shard_count {
        return Err(ProgramError::InvalidInstructionData);
    }

    verify_hardcoded_fee_vault_account(fee_vault_account, program_id, fee_shard_index)?;
    validate_fee_transfer_accounts(program_id, payer, fee_vault_account, system_program)?;

    transfer_fee(
        program_id,
        payer,
        fee_vault_account,
        deploy_fee_lamports,
        Some(system_program),
    )?;
    Ok(())
}

/// Initialize a VM-scoped fee vault PDA shard.
/// Accounts: [vm_state, payer, fee_vault, system_program]
pub fn init_fee_vault(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    shard_index: u8,
    bump: u8,
) -> ProgramResult {
    require_min_accounts(accounts, 4)?;

    let vm_state_account = &accounts[0];
    let payer = &accounts[1];
    let fee_vault_account = &accounts[2];
    let system_program = &accounts[3];

    verify_hardcoded_vm_state_account(vm_state_account, program_id)?;
    verify_program_owned(vm_state_account, program_id)?;
    require_signer(payer)?;
    // Enforce canonical System Program identity.
    if system_program.key().as_ref() != &[0u8; 32] {
        return Err(ProgramError::InvalidArgument);
    }

    // Validate VM initialized.
    let vm_state_data = unsafe { vm_state_account.borrow_data_unchecked() };
    let vm_state = FIVEVMState::from_account_data(&vm_state_data)?;
    if !vm_state.is_initialized() {
        return Err(ProgramError::Custom(7000));
    }
    if shard_index >= vm_state.fee_vault_shard_count() {
        return Err(ProgramError::InvalidInstructionData);
    }

    #[cfg(not(test))]
    let expected_key = crate::common::get_hardcoded_fee_vault(shard_index)
        .ok_or(ProgramError::InvalidInstructionData)?;
    #[cfg(not(test))]
    let expected_bump = crate::common::get_hardcoded_fee_vault_bump(shard_index)
        .ok_or(ProgramError::InvalidInstructionData)?;
    #[cfg(test)]
    let (expected_key, expected_bump) =
        crate::common::derive_fee_vault_pda(program_id, shard_index)?;
    if fee_vault_account.key() != &expected_key || bump != expected_bump {
        return Err(ProgramError::InvalidArgument);
    }

    // Idempotent: already created by this program.
    if fee_vault_account.owner() == program_id {
        return Ok(());
    }
    if fee_vault_account.owner() != &Pubkey::default() {
        return Err(ProgramError::IllegalOwner);
    }

    let rent =
        pinocchio::sysvars::rent::Rent::get().map_err(|_| ProgramError::AccountNotRentExempt)?;
    let rent_lamports = rent.minimum_balance(0);

    let create_account_data = build_system_create_account_ix(rent_lamports, 0u64, program_id);

    let shard_seed = [shard_index];
    let bump_seed = [bump];
    let seeds: &[Seed] = &[
        Seed::from(FEE_VAULT_SEED),
        Seed::from(&shard_seed),
        Seed::from(&bump_seed),
    ];
    let signer = Signer::from(seeds);

    let metas = [
        AccountMeta {
            pubkey: payer.key(),
            is_signer: true,
            is_writable: true,
        },
        AccountMeta {
            pubkey: fee_vault_account.key(),
            is_signer: true,
            is_writable: true,
        },
    ];

    let instruction = Instruction {
        program_id: system_program.key(),
        accounts: &metas,
        data: &create_account_data,
    };
    invoke_signed::<3>(
        &instruction,
        &[payer, fee_vault_account, system_program],
        &[signer],
    )?;
    Ok(())
}

/// Withdraw fees from a VM-scoped fee-vault shard.
/// Accounts: [vm_state, authority, fee_vault, recipient]
pub fn withdraw_script_fees(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    _script: Pubkey,
    shard_index: u8,
    lamports: u64,
) -> ProgramResult {
    require_min_accounts(accounts, 4)?;

    let vm_state_account = &accounts[0];
    let authority = &accounts[1];
    let fee_vault_account = &accounts[2];
    let recipient = &accounts[3];

    verify_hardcoded_vm_state_account(vm_state_account, program_id)?;
    verify_program_owned(vm_state_account, program_id)?;
    verify_hardcoded_fee_vault_account(fee_vault_account, program_id, shard_index)?;
    require_signer(authority)?;
    if !fee_vault_account.is_writable() || !recipient.is_writable() {
        return Err(ProgramError::InvalidArgument);
    }

    let vm_state_data = unsafe { vm_state_account.borrow_data_unchecked() };
    let vm_state = FIVEVMState::from_account_data(&vm_state_data)?;
    if vm_state.authority != *authority.key() {
        return Err(ProgramError::Custom(0));
    }
    if shard_index >= vm_state.fee_vault_shard_count() {
        return Err(ProgramError::InvalidInstructionData);
    }

    let rent =
        pinocchio::sysvars::rent::Rent::get().map_err(|_| ProgramError::AccountNotRentExempt)?;
    let min_balance = rent.minimum_balance(0);
    let current = fee_vault_account.lamports();
    if current < min_balance {
        return Err(ProgramError::InsufficientFunds);
    }
    let available = current.saturating_sub(min_balance);
    if lamports > available {
        return Err(ProgramError::InsufficientFunds);
    }

    *fee_vault_account.try_borrow_mut_lamports()? -= lamports;
    let new_recipient_balance = recipient
        .lamports()
        .checked_add(lamports)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    *recipient.try_borrow_mut_lamports()? = new_recipient_balance;
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

    // Enforce mandatory fee-vault routing by requiring non-zero fees.
    if deploy_fee_lamports == 0 || execute_fee_lamports == 0 {
        return Err(ProgramError::InvalidInstructionData);
    }

    let vm_state_account = &accounts[0];
    let authority = &accounts[1];

    // Verify ownership
    verify_hardcoded_vm_state_account(vm_state_account, program_id)?;
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

    #[test]
    fn transfer_fee_allows_same_payer_and_recipient() {
        let program_id = Pubkey::from([12u8; 32]);
        let account_key = Pubkey::from([13u8; 32]);

        let mut lamports = 1_000;
        let mut data = [];

        let account = create_account_info(
            &account_key,
            true,
            true,
            &mut lamports,
            &mut data,
            &program_id,
        );

        let result = transfer_fee(&program_id, &account, &account, 10, None);
        assert_eq!(result, Ok(()));
        assert_eq!(account.lamports(), 1_000);
    }

    #[test]
    fn fee_validation_rejects_non_system_program_key() {
        let program_id = Pubkey::from([21u8; 32]);
        let payer_key = Pubkey::from([22u8; 32]);
        let vault_key = Pubkey::from([23u8; 32]);
        let fake_system_key = Pubkey::from([24u8; 32]);
        let system_owner = Pubkey::default();

        let mut payer_lamports = 1_000;
        let mut vault_lamports = 0;
        let mut system_lamports = 0;
        let mut payer_data = [];
        let mut vault_data = [];
        let mut system_data = [];

        let payer = create_account_info(
            &payer_key,
            true,
            true,
            &mut payer_lamports,
            &mut payer_data,
            &system_owner,
        );
        let fee_vault = create_account_info(
            &vault_key,
            false,
            true,
            &mut vault_lamports,
            &mut vault_data,
            &program_id,
        );
        let fake_system_program = create_account_info(
            &fake_system_key,
            false,
            false,
            &mut system_lamports,
            &mut system_data,
            &system_owner,
        );

        // Regression: only canonical System Program ID is accepted.
        assert_eq!(
            validate_fee_transfer_accounts(&program_id, &payer, &fee_vault, &fake_system_program),
            Err(ProgramError::InvalidArgument)
        );
    }

    #[test]
    fn init_fee_vault_rejects_non_system_program_identity_when_idempotent() {
        let program_id = Pubkey::from([31u8; 32]);
        let (vm_key, vm_bump) = crate::common::derive_canonical_vm_state_pda(&program_id).unwrap();
        let (fee_vault_key, fee_vault_bump) =
            crate::common::derive_fee_vault_pda(&program_id, 0).unwrap();
        let payer_key = Pubkey::from([32u8; 32]);
        let fake_system_key = Pubkey::from([33u8; 32]);
        let authority_key = Pubkey::from([34u8; 32]);
        let system_owner = Pubkey::default();

        let mut vm_lamports = 1_000_000;
        let mut payer_lamports = 1_000_000;
        let mut vault_lamports = 1_000_000;
        let mut fake_system_lamports = 1;
        let mut vm_data = vec![0u8; FIVEVMState::LEN];
        let mut payer_data = [];
        let mut vault_data = [];
        let mut fake_system_data = [];
        {
            let state = FIVEVMState::from_account_data_mut(vm_data.as_mut_slice()).unwrap();
            state.initialize(authority_key, vm_bump);
        }

        let vm_state = create_account_info(
            &vm_key,
            false,
            true,
            &mut vm_lamports,
            vm_data.as_mut_slice(),
            &program_id,
        );
        let payer = create_account_info(
            &payer_key,
            true,
            true,
            &mut payer_lamports,
            &mut payer_data,
            &system_owner,
        );
        let fee_vault = create_account_info(
            &fee_vault_key,
            false,
            true,
            &mut vault_lamports,
            &mut vault_data,
            &program_id,
        );
        let fake_system_program = create_account_info(
            &fake_system_key,
            false,
            false,
            &mut fake_system_lamports,
            &mut fake_system_data,
            &system_owner,
        );
        let accounts = [vm_state, payer, fee_vault, fake_system_program];

        assert_eq!(
            init_fee_vault(&program_id, &accounts, 0, fee_vault_bump),
            Err(ProgramError::InvalidArgument)
        );
    }
}
