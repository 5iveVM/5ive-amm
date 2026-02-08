use crate::error::{CompactResult, VMErrorCode};
use crate::lazy_validation::{LazyAccountValidator, ValidationStats};
use crate::debug_log;
use pinocchio::{
    account_info::AccountInfo,
    instruction::Signer,
    pubkey::Pubkey,
};

#[cfg(any(target_os = "solana", test))]
use pinocchio::instruction::{AccountMeta, Instruction, Seed};
#[cfg(any(target_os = "solana", test))]
use pinocchio::program::invoke_signed;

// System program ID constant
const SYSTEM_PROGRAM_ID: [u8; 32] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

pub struct AccountManager<'a> {
    pub accounts: &'a [AccountInfo],
    pub lazy_validator: LazyAccountValidator,
    pub program_id: Pubkey,
}

impl<'a> AccountManager<'a> {
    #[inline(always)]
    pub fn new(accounts: &'a [AccountInfo], program_id: Pubkey) -> Self {
        Self {
            accounts,
            lazy_validator: LazyAccountValidator::new(accounts.len()),
            program_id,
        }
    }

    #[inline(always)]
    pub fn get(&self, index: u8) -> CompactResult<&'a AccountInfo> {
        self.lazy_validator.ensure_validated(index, self.accounts)?;

        if index as usize >= self.accounts.len() {
            return Err(VMErrorCode::InvalidAccountIndex);
        }
        Ok(&self.accounts[index as usize])
    }

    #[inline(always)]
    pub fn get_unchecked(&self, index: u8) -> CompactResult<&'a AccountInfo> {
        if index as usize >= self.accounts.len() {
            return Err(VMErrorCode::InvalidAccountIndex);
        }
        Ok(&self.accounts[index as usize])
    }

    #[inline(always)]
    pub fn accounts(&self) -> &[AccountInfo] {
        self.accounts
    }

    // --- Lazy validation operations ---

    #[inline]
    pub fn validation_stats(&self) -> ValidationStats {
        ValidationStats::calculate(&self.lazy_validator)
    }

    #[inline]
    pub fn is_validated(&self, index: u8) -> bool {
        self.lazy_validator.is_validated(index)
    }

    #[inline]
    pub fn validated_count(&self) -> u8 {
        self.lazy_validator.validated_count()
    }

    #[inline]
    pub fn validate_bitwise_constraints(&self, constraints: u64) -> CompactResult<()> {
        self.lazy_validator
            .validate_constraints_bitwise(constraints, self.accounts)
    }

    #[inline]
    pub fn check_authorization(&self, account_idx: u8) -> CompactResult<()> {
        let account = self.get(account_idx)?;

        if account.data_len() == 0 {
            return Ok(());
        }

        let required_authority = *account.owner();
        if self.program_id == required_authority {
            Ok(())
        } else {
            debug_log!("Auth failed: owner mismatch");
            return Err(VMErrorCode::ScriptNotAuthorized);
        }
    }

    // --- Account creation ---

    #[inline]
    #[allow(unused_variables)]
    fn perform_create_account_cpi(
        &self,
        payer: &AccountInfo,
        new_account: &AccountInfo,
        system_program: &AccountInfo,
        lamports: u64,
        space: u64,
        owner: &Pubkey,
        signers: &[Signer],
    ) -> CompactResult<()> {
        #[cfg(target_os = "solana")]
        {
            // Step 1: Transfer
            if lamports > 0 {
                let mut transfer_data = [0u8; 12];
                transfer_data[0..4].copy_from_slice(&2u32.to_le_bytes());
                transfer_data[4..12].copy_from_slice(&lamports.to_le_bytes());

                let transfer_metas = [
                    AccountMeta {
                        pubkey: payer.key(),
                        is_signer: true,
                        is_writable: true,
                    },
                    AccountMeta {
                        pubkey: new_account.key(),
                        is_signer: false,
                        is_writable: true,
                    },
                ];

                let transfer_instruction = Instruction {
                    program_id: system_program.key(),
                    accounts: &transfer_metas,
                    data: &transfer_data,
                };

                invoke_signed::<3>(
                    &transfer_instruction,
                    &[payer, new_account, system_program],
                    signers,
                )
                .map_err(|_| VMErrorCode::InvokeError)?;
            }

            // Step 2: Allocate
            let mut allocate_data = [0u8; 12];
            allocate_data[0..4].copy_from_slice(&8u32.to_le_bytes());
            allocate_data[4..12].copy_from_slice(&space.to_le_bytes());

            let allocate_metas = [
                AccountMeta {
                    pubkey: new_account.key(),
                    is_signer: true,
                    is_writable: true,
                },
            ];

            let allocate_instruction = Instruction {
                program_id: system_program.key(),
                accounts: &allocate_metas,
                data: &allocate_data,
            };

            invoke_signed::<2>(
                &allocate_instruction,
                &[new_account, system_program],
                signers,
            )
            .map_err(|_| VMErrorCode::InvokeError)?;

            // Step 3: Assign
            let mut assign_data = [0u8; 36];
            assign_data[0..4].copy_from_slice(&1u32.to_le_bytes());
            assign_data[4..36].copy_from_slice(owner.as_ref());

            let assign_metas = [
                AccountMeta {
                    pubkey: new_account.key(),
                    is_signer: true,
                    is_writable: true,
                },
            ];

            let assign_instruction = Instruction {
                program_id: system_program.key(),
                accounts: &assign_metas,
                data: &assign_data,
            };

            invoke_signed::<2>(
                &assign_instruction,
                &[new_account, system_program],
                signers,
            )
            .map_err(|_| VMErrorCode::InvokeError)?;
        }

        #[cfg(not(target_os = "solana"))]
        {
            unsafe {
                if *payer.borrow_lamports_unchecked() < lamports {
                    return Err(VMErrorCode::InvokeError);
                }
                *payer.borrow_mut_lamports_unchecked() -= lamports;
                *new_account.borrow_mut_lamports_unchecked() += lamports;
                new_account
                    .resize(space as usize)
                    .map_err(|_| VMErrorCode::InvokeError)?;
                new_account.assign(owner);
            }
        }

        Ok(())
    }

    #[inline]
    pub fn create_account(
        &mut self,
        account_idx: u8,
        space: u64,
        lamports: u64,
        owner: &Pubkey,
    ) -> CompactResult<()> {
        self.lazy_validator.ensure_validated(0, self.accounts)?;

        let new_account = self.get_unchecked(account_idx)?;

        let mut payer = self.get_unchecked(0)?;
        let mut payer_found = false;

        for i in 0..self.accounts.len() {
            let acc = self.get_unchecked(i as u8)?;
            if acc.is_signer() && acc.is_writable() && acc.key() != new_account.key() {
                payer = acc;
                payer_found = true;
                break;
            }
        }

        if !payer_found {
             debug_log!("CreateAccount: WARNING - No valid payer found!");
        }

        let system_program_id = Pubkey::from(SYSTEM_PROGRAM_ID);

        let system_program = self
            .accounts
            .iter()
            .find(|a| a.key() == &system_program_id)
            .ok_or(VMErrorCode::AccountNotFound)?;

        self.perform_create_account_cpi(
            payer,
            new_account,
            system_program,
            lamports,
            space,
            owner,
            &[],
        )?;

        let _ = self.refresh_account_pointers_after_cpi(&[account_idx as usize]);

        Ok(())
    }

    #[inline]
    pub fn create_account_with_payer(
        &mut self,
        account_idx: u8,
        payer_idx: u8,
        space: u64,
        lamports: u64,
        owner: &Pubkey,
    ) -> CompactResult<()> {
        if account_idx as usize >= self.accounts.len() {
            return Err(VMErrorCode::InvalidAccountIndex);
        }
        if payer_idx as usize >= self.accounts.len() {
            return Err(VMErrorCode::InvalidAccountIndex);
        }

        self.lazy_validator.ensure_validated(payer_idx, self.accounts)?;

        let new_account = self.get_unchecked(account_idx)?;
        let payer = self.get_unchecked(payer_idx)?;

        if !payer.is_signer() {
            return Err(VMErrorCode::ConstraintViolation);
        }

        if !payer.is_writable() {
            return Err(VMErrorCode::ConstraintViolation);
        }

        const MAX_ACCOUNT_SIZE: u64 = 10 * 1024 * 1024;
        if space > MAX_ACCOUNT_SIZE {
            return Err(VMErrorCode::InvalidParameter);
        }

        let system_program_id = Pubkey::from(SYSTEM_PROGRAM_ID);
        let system_program = self
            .accounts
            .iter()
            .find(|a| a.key() == &system_program_id)
            .ok_or(VMErrorCode::AccountNotFound)?;

        self.perform_create_account_cpi(
            payer,
            new_account,
            system_program,
            lamports,
            space,
            owner,
            &[],
        )?;

        let _ = self.refresh_account_pointers_after_cpi(&[account_idx as usize]);

        Ok(())
    }

    #[inline]
    pub fn create_pda_account(
        &mut self,
        account_idx: u8,
        seeds: &[&[u8]],
        bump: u8,
        space: u64,
        lamports: u64,
        owner: &Pubkey,
        payer_idx: u8,
    ) -> CompactResult<()> {
        self.lazy_validator.ensure_validated(0, self.accounts)?;

        let new_account = self.get_unchecked(account_idx)?;

        if payer_idx as usize >= self.accounts.len() {
            return Err(VMErrorCode::InvalidAccountIndex);
        }

        let payer = self.get_unchecked(payer_idx)?;

        let system_program_id = Pubkey::from(SYSTEM_PROGRAM_ID);
        let system_program = self
            .accounts
            .iter()
            .find(|a| a.key() == &system_program_id)
            .ok_or(VMErrorCode::AccountNotFound)?;

        #[cfg(target_os = "solana")]
        {
            const MAX_SEEDS: usize = 8;
            let binding = [bump];
            let mut seed_vec: heapless::Vec<Seed, MAX_SEEDS> = heapless::Vec::new();
            for s in seeds.iter() {
                seed_vec
                    .push(Seed::from(*s))
                    .map_err(|_| VMErrorCode::TooManySeeds)?;
            }
            seed_vec
                .push(Seed::from(&binding))
                .map_err(|_| VMErrorCode::TooManySeeds)?;
            let signer = Signer::from(seed_vec.as_slice());

            self.perform_create_account_cpi(
                payer,
                new_account,
                system_program,
                lamports,
                space,
                owner,
                &[signer],
            )?;
        }
        #[cfg(not(target_os = "solana"))]
        {
            core::hint::black_box((seeds, bump, payer_idx));
            self.perform_create_account_cpi(
                payer,
                new_account,
                system_program,
                lamports,
                space,
                owner,
                &[],
            )?;
        }

        let _ = self.refresh_account_pointers_after_cpi(&[account_idx as usize]);

        Ok(())
    }

    #[inline]
    pub fn refresh_account_pointers_after_cpi(&self, account_indices: &[usize]) -> CompactResult<()> {
        for &idx in account_indices {
            if idx >= self.accounts.len() {
                continue;
            }
            let account = &self.accounts[idx];
            account.refresh_after_cpi();
        }
        Ok(())
    }
}
