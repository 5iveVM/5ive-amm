use crate::debug_log;
use crate::error::{CompactResult, VMErrorCode};
use crate::lazy_validation::{LazyAccountValidator, ValidationStats};
use pinocchio::{account_info::AccountInfo, instruction::Signer, pubkey::Pubkey};

#[cfg(target_os = "solana")]
use pinocchio::instruction::{AccountMeta, Instruction, Seed};
#[cfg(target_os = "solana")]
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
    const SCRIPT_HEADER_MAGIC: [u8; 4] = [b'5', b'I', b'V', b'E'];

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

        // Protect VM state account (execution index 0) from script-level writes.
        if account_idx == 0 {
            return Err(VMErrorCode::ScriptNotAuthorized);
        }

        if account.data_len() == 0 {
            return Ok(());
        }

        let required_authority = *account.owner();
        if self.program_id != required_authority {
            debug_log!("Auth failed: owner mismatch");
            return Err(VMErrorCode::ScriptNotAuthorized);
        }

        // Disallow direct mutation of deployed script accounts from bytecode opcodes.
        // This prevents a script from rewriting other script accounts via SAVE/SET_LAMPORTS.
        if account.data_len() >= 4 {
            let data = unsafe { account.borrow_data_unchecked() };
            if data[..4] == Self::SCRIPT_HEADER_MAGIC {
                return Err(VMErrorCode::ScriptNotAuthorized);
            }
        }

        Ok(())
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
            // Use a single SystemProgram::CreateAccount CPI for both regular accounts
            // and PDAs (the latter satisfy the new-account signer requirement via seeds).
            let mut create_account_data = [0u8; 52];
            create_account_data[0..4].copy_from_slice(&0u32.to_le_bytes());
            create_account_data[4..12].copy_from_slice(&lamports.to_le_bytes());
            create_account_data[12..20].copy_from_slice(&space.to_le_bytes());
            create_account_data[20..52].copy_from_slice(owner.as_ref());

            let create_account_metas = [
                AccountMeta {
                    pubkey: payer.key(),
                    is_signer: true,
                    is_writable: true,
                },
                AccountMeta {
                    pubkey: new_account.key(),
                    is_signer: true,
                    is_writable: true,
                },
            ];

            let create_account_instruction = Instruction {
                program_id: system_program.key(),
                accounts: &create_account_metas,
                data: &create_account_data,
            };

            invoke_signed::<3>(
                &create_account_instruction,
                &[payer, new_account, system_program],
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

        self.lazy_validator
            .ensure_validated(payer_idx, self.accounts)?;

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
    pub fn refresh_account_pointers_after_cpi(
        &self,
        account_indices: &[usize],
    ) -> CompactResult<()> {
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

#[cfg(test)]
mod tests {
    use super::*;

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
    fn check_authorization_rejects_vm_state_index_zero() {
        let program_id = Pubkey::from([1u8; 32]);
        let key = Pubkey::from([2u8; 32]);
        let mut lamports = 1_000;
        let mut data = [0u8; 8];

        let account = create_account_info(&key, false, true, &mut lamports, &mut data, &program_id);
        let accounts = [account];
        let manager = AccountManager::new(&accounts, program_id);

        assert_eq!(
            manager.check_authorization(0),
            Err(VMErrorCode::ScriptNotAuthorized)
        );
    }

    #[test]
    fn check_authorization_rejects_script_header_accounts() {
        let program_id = Pubkey::from([7u8; 32]);
        let vm_key = Pubkey::from([8u8; 32]);
        let script_key = Pubkey::from([9u8; 32]);
        let mut vm_lamports = 1_000;
        let mut script_lamports = 1_000;
        let mut vm_data = [0u8; 8];
        let mut script_data = [0u8; 8];
        script_data[0..4].copy_from_slice(b"5IVE");

        let vm_state = create_account_info(
            &vm_key,
            false,
            true,
            &mut vm_lamports,
            &mut vm_data,
            &program_id,
        );
        let script = create_account_info(
            &script_key,
            false,
            true,
            &mut script_lamports,
            &mut script_data,
            &program_id,
        );
        let accounts = [vm_state, script];
        let manager = AccountManager::new(&accounts, program_id);

        assert_eq!(
            manager.check_authorization(1),
            Err(VMErrorCode::ScriptNotAuthorized)
        );
    }

    #[test]
    fn create_account_with_payer_updates_account_state() {
        let program_id = Pubkey::from([3u8; 32]);
        let payer_key = Pubkey::from([4u8; 32]);
        let new_account_key = Pubkey::from([5u8; 32]);
        let system_program_key = Pubkey::from(SYSTEM_PROGRAM_ID);
        let owner = Pubkey::from([6u8; 32]);

        let mut payer_lamports = 10_000;
        let mut new_account_lamports = 0;
        let mut system_program_lamports = 0;

        let mut payer_data = [0u8; 0];
        let mut new_account_data = [0u8; 0];
        let mut system_program_data = [0u8; 0];

        let payer = create_account_info(
            &payer_key,
            true,
            true,
            &mut payer_lamports,
            &mut payer_data,
            &program_id,
        );
        let new_account = create_account_info(
            &new_account_key,
            false,
            true,
            &mut new_account_lamports,
            &mut new_account_data,
            &system_program_key,
        );
        let system_program = create_account_info(
            &system_program_key,
            false,
            false,
            &mut system_program_lamports,
            &mut system_program_data,
            &system_program_key,
        );

        let accounts = [payer, new_account, system_program];
        let mut manager = AccountManager::new(&accounts, program_id);

        manager
            .create_account_with_payer(1, 0, 64, 1_500, &owner)
            .expect("create_account_with_payer should succeed");

        assert_eq!(accounts[0].lamports(), 8_500);
        assert_eq!(accounts[1].lamports(), 1_500);
        assert_eq!(accounts[1].data_len(), 64);
        assert_eq!(accounts[1].owner(), &owner);
    }
}
