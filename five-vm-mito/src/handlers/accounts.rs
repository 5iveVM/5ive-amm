//! Account operations handler for MitoVM
//!
//! This module handles account operations including CREATE_ACCOUNT, LOAD_ACCOUNT,
//! SAVE_ACCOUNT, GET_LAMPORTS, GET_KEY, GET_DATA, GET_OWNER, and SET_LAMPORTS.
//! It manages Solana account state and metadata access.

use crate::{
    context::ExecutionManager,
    debug_log,
    error::{CompactResult, VMErrorCode},
    pop_u64,
};
use core::convert::TryFrom;
use five_protocol::{opcodes::*, ValueRef};
#[cfg(target_os = "solana")]
use pinocchio::instruction::{AccountMeta, Instruction};
#[cfg(target_os = "solana")]
use pinocchio::program::invoke_signed;
use pinocchio::pubkey::Pubkey;

const CLOSED_MARKER: [u8; 4] = *b"CLSD";
const PUBKEY_REF_KEY_TAG_BASE: u16 = 0xFF00;
const PUBKEY_REF_OWNER_TAG_BASE: u16 = 0xFE00;

/// Execute Solana account operations including creation, metadata access, and lamport management.
/// Handles the 0x50-0x5F opcode range.
#[inline(always)]
pub fn handle_accounts(opcode: u8, ctx: &mut ExecutionManager) -> CompactResult<()> {
    match opcode {
        CREATE_ACCOUNT => {
            let owner_ref = ctx.pop()?;
            let space = ctx.pop()?.as_u64().ok_or(VMErrorCode::TypeMismatch)?;
            let lamports = ctx.pop()?.as_u64().ok_or(VMErrorCode::TypeMismatch)?;
            let account_idx = ctx
                .pop()?
                .as_account_idx()
                .ok_or(VMErrorCode::TypeMismatch)?;

            let owner_bytes = ctx.extract_pubkey(&owner_ref)?;
            let owner = Pubkey::from(owner_bytes);

            debug_log!(
                "MitoVM: CREATE_ACCOUNT idx: {}, lamports: {}, space: {}",
                account_idx,
                lamports,
                space
            );

            match ctx.create_account(account_idx, space, lamports, &owner) {
                Ok(_) => {
                    ctx.initialize_state_owner_meta(account_idx)?;
                    ctx.push(ValueRef::Bool(true))?;
                }
                Err(_) => {
                    let account = ctx.get_account(account_idx)?;
                    if account.lamports() < lamports
                        // SAFETY: Read-only access to check length
                        || unsafe { account.borrow_data_unchecked() }.len() < space as usize
                    {
                        ctx.push(ValueRef::Bool(false))?;
                    } else {
                        ctx.push(ValueRef::Bool(true))?;
                    }
                }
            }
        }
        LOAD_ACCOUNT => {
            let account_idx = ctx
                .pop()?
                .as_account_idx()
                .ok_or(VMErrorCode::TypeMismatch)?;

            let account = ctx.get_account(account_idx)?;

            // SAFETY: The account is immutably borrowed and we only read the slice to
            // determine its length.
            let data_len = unsafe { account.borrow_data_unchecked() }.len() as u64;
            let lamports = account.lamports();

            ctx.push(ValueRef::U64(data_len))?;
            ctx.push(ValueRef::U64(lamports))?;

            debug_log!(
                "MitoVM: LOAD_ACCOUNT loaded - data_len: {}, lamports: {}",
                data_len as u32,
                lamports
            );
        }
        SAVE_ACCOUNT => {
            let data_value = ctx.pop()?.as_u64().ok_or(VMErrorCode::TypeMismatch)?;
            let offset = ctx.pop()?.as_u64().ok_or(VMErrorCode::TypeMismatch)? as usize;
            let account_idx = ctx
                .pop()?
                .as_account_idx()
                .ok_or(VMErrorCode::TypeMismatch)?;

            debug_log!(
                "MitoVM: SAVE_ACCOUNT idx: {}, offset: {}, value: {}",
                account_idx,
                offset as u32,
                data_value
            );

            ctx.check_bytecode_authorization(account_idx)?;

            let account = ctx.get_account(account_idx)?;
            if !account.is_writable() {
                return Err(VMErrorCode::AccountNotWritable);
            }

            // SAFETY: The account has been verified writable and no other borrows are
            // active, allowing a unique mutable reference to its data.
            let account_data = unsafe { account.borrow_mut_data_unchecked() };
            if offset + 8 > account_data.len() {
                return Err(VMErrorCode::InvalidAccountData);
            }

            account_data[offset..offset + 8].copy_from_slice(&data_value.to_le_bytes());
            debug_log!("MitoVM: SAVE_ACCOUNT completed successfully");
        }
        GET_ACCOUNT => {
            let account_idx = ctx.fetch_byte()?;
            // Validate account exists and is accessible in current context.
            let _ = ctx.get_account(account_idx)?;
            ctx.push(ValueRef::AccountRef(account_idx, 0))?;
            debug_log!(
                "MitoVM: GET_ACCOUNT account {} -> AccountRef({}, 0)",
                account_idx,
                account_idx
            );
        }
        GET_LAMPORTS => {
            let account_idx = ctx.fetch_byte()?;
            let account = ctx.get_account(account_idx)?;
            let lamports = account.lamports();
            ctx.push(ValueRef::U64(lamports))?;
            debug_log!(
                "MitoVM: GET_LAMPORTS account {} = {}",
                account_idx,
                lamports
            );
        }
        GET_KEY => {
            let account_idx = ctx.fetch_byte()?;
            // Encode account key refs in a tagged PubkeyRef range (0xFFxx).
            let pubkey_ref_offset = PUBKEY_REF_KEY_TAG_BASE | account_idx as u16;
            ctx.push(ValueRef::PubkeyRef(pubkey_ref_offset))?;
            debug_log!(
                "MitoVM: GET_KEY account {} pushed PubkeyRef(tagged={})",
                account_idx,
                pubkey_ref_offset
            );
        }
        GET_DATA => {
            let account_idx = ctx.fetch_byte()?;

            let data_len = {
                let account = ctx.get_account(account_idx)?;
                // SAFETY: Read-only access for length
                unsafe { account.borrow_data_unchecked() }.len()
            };

            let _data_len_u16 = u16::try_from(data_len).map_err(|_| VMErrorCode::MemoryError)?;

            ctx.push(ValueRef::AccountRef(account_idx, 0))?;

            debug_log!(
                "MitoVM: GET_DATA account {} -> AccountRef({}, 0)",
                account_idx,
                account_idx
            );
        }
        GET_OWNER => {
            let account_idx = ctx.fetch_byte()?;
            // Encode account owner refs in a tagged PubkeyRef range (0xFExx).
            let pubkey_ref_offset = PUBKEY_REF_OWNER_TAG_BASE | account_idx as u16;
            ctx.push(ValueRef::PubkeyRef(pubkey_ref_offset))?;
            debug_log!(
                "MitoVM: GET_OWNER account {} pushed PubkeyRef(tagged={})",
                account_idx,
                pubkey_ref_offset
            );
        }
        SET_LAMPORTS => {
            let account_idx = ctx.fetch_byte()?;
            let new_lamports = pop_u64!(ctx);

            ctx.check_bytecode_authorization(account_idx)?;

            let account = ctx.get_account(account_idx)?;

            if !account.is_writable() {
                return Err(VMErrorCode::AccountNotWritable);
            }

            // Set the account's lamports
            *unsafe { account.borrow_mut_lamports_unchecked() } = new_lamports;
            debug_log!(
                "MitoVM: SET_LAMPORTS account {} = {}",
                account_idx,
                new_lamports
            );
        }
        TRANSFER | TRANSFER_SIGNED => {
            let amount = pop_u64!(ctx);
            let to_idx = ctx
                .pop()?
                .as_account_idx()
                .ok_or(VMErrorCode::TypeMismatch)?;
            let from_idx = ctx
                .pop()?
                .as_account_idx()
                .ok_or(VMErrorCode::TypeMismatch)?;

            let from = ctx.get_account(from_idx)?;
            let to = ctx.get_account(to_idx)?;

            if !from.is_writable() || !to.is_writable() {
                return Err(VMErrorCode::AccountNotWritable);
            }
            if opcode == TRANSFER && !from.is_signer() && *from.owner() != ctx.program_id {
                return Err(VMErrorCode::ConstraintViolation);
            }

            #[cfg(target_os = "solana")]
            {
                if *from.owner() == ctx.program_id {
                    if from.lamports() < amount {
                        return Err(VMErrorCode::ConstraintViolation);
                    }
                    let to_next = to
                        .lamports()
                        .checked_add(amount)
                        .ok_or(VMErrorCode::ArithmeticOverflow)?;
                    unsafe {
                        *from.borrow_mut_lamports_unchecked() -= amount;
                        *to.borrow_mut_lamports_unchecked() = to_next;
                    }
                } else {
                    let system_program_id = Pubkey::from([0u8; 32]);
                    let system_program = ctx
                        .accounts()
                        .iter()
                        .find(|a| a.key() == &system_program_id)
                        .ok_or(VMErrorCode::AccountNotFound)?;

                    let mut transfer_data = [0u8; 12];
                    transfer_data[0..4].copy_from_slice(&2u32.to_le_bytes());
                    transfer_data[4..12].copy_from_slice(&amount.to_le_bytes());

                    let transfer_metas = [
                        AccountMeta {
                            pubkey: from.key(),
                            is_signer: true,
                            is_writable: true,
                        },
                        AccountMeta {
                            pubkey: to.key(),
                            is_signer: false,
                            is_writable: true,
                        },
                    ];

                    let ix = Instruction {
                        program_id: system_program.key(),
                        accounts: &transfer_metas,
                        data: &transfer_data,
                    };

                    invoke_signed::<3>(&ix, &[from, to, system_program], &[])
                        .map_err(|_| VMErrorCode::InvokeError)?;
                }
            }

            #[cfg(not(target_os = "solana"))]
            {
                if from.lamports() < amount {
                    return Err(VMErrorCode::ConstraintViolation);
                }
                unsafe {
                    *from.borrow_mut_lamports_unchecked() -= amount;
                    *to.borrow_mut_lamports_unchecked() += amount;
                }
            }

            debug_log!(
                "MitoVM: TRANSFER from {} to {} amount {}",
                from_idx,
                to_idx,
                amount
            );
        }
        CLOSE_ACCOUNT => {
            // Stack contract: [source_idx, destination_idx]
            let destination_idx = ctx
                .pop()?
                .as_account_idx()
                .ok_or(VMErrorCode::TypeMismatch)?;
            let source_idx = ctx
                .pop()?
                .as_account_idx()
                .ok_or(VMErrorCode::TypeMismatch)?;

            if source_idx == destination_idx {
                return Err(VMErrorCode::ConstraintViolation);
            }

            // Enforce script-level ownership isolation before draining lamports.
            ctx.check_bytecode_authorization(source_idx)?;

            let source = ctx.get_account(source_idx)?;
            let destination = ctx.get_account(destination_idx)?;

            if !source.is_writable() || !destination.is_writable() {
                return Err(VMErrorCode::AccountNotWritable);
            }
            if *source.owner() != ctx.program_id {
                return Err(VMErrorCode::ConstraintViolation);
            }
            if source.executable() {
                return Err(VMErrorCode::ConstraintViolation);
            }

            let source_lamports = source.lamports();
            if source_lamports == 0 {
                // Idempotent no-op for already-drained accounts.
                return Ok(());
            }

            let new_destination_lamports = destination
                .lamports()
                .checked_add(source_lamports)
                .ok_or(VMErrorCode::ArithmeticOverflow)?;

            unsafe {
                *source.borrow_mut_lamports_unchecked() = 0;
                *destination.borrow_mut_lamports_unchecked() = new_destination_lamports;
            }

            // Tombstone account data to make closed script/data accounts unusable.
            let source_data = unsafe { source.borrow_mut_data_unchecked() };
            source_data.fill(0);
            if source_data.len() >= CLOSED_MARKER.len() {
                source_data[..CLOSED_MARKER.len()].copy_from_slice(&CLOSED_MARKER);
            }

            debug_log!(
                "MitoVM: CLOSE_ACCOUNT source {} -> destination {} amount {}",
                source_idx,
                destination_idx,
                source_lamports
            );
        }
        _ => return Err(VMErrorCode::InvalidInstruction.into()),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::handle_accounts;
    use crate::{context::ExecutionContext, error::VMErrorCode, stack::StackStorage};
    use five_protocol::{opcodes::CLOSE_ACCOUNT, ValueRef};
    use pinocchio::{account_info::AccountInfo, pubkey::Pubkey};

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
    fn close_account_rejects_zero_data_source_with_active_script() {
        let program_id = Pubkey::from([31u8; 32]);
        let active_script = Pubkey::from([32u8; 32]);
        let source_key = Pubkey::from([33u8; 32]);
        let destination_key = Pubkey::from([34u8; 32]);
        let mut source_lamports = 2_500;
        let mut destination_lamports = 100;
        let mut source_data = [0u8; 0];
        let mut destination_data = [1u8; 8];

        let source = create_account_info(
            &source_key,
            false,
            true,
            &mut source_lamports,
            &mut source_data,
            &program_id,
        );
        let destination = create_account_info(
            &destination_key,
            false,
            true,
            &mut destination_lamports,
            &mut destination_data,
            &program_id,
        );
        let accounts = [source, destination];

        let mut storage = StackStorage::new();
        let mut ctx = ExecutionContext::new(
            &[],
            &accounts,
            program_id,
            &[],
            0,
            &mut storage,
            0,
            0,
            0,
            0,
            0,
            0,
        );
        ctx.set_active_script_key(Some(active_script));
        ctx.push(ValueRef::U8(0)).expect("push source index");
        ctx.push(ValueRef::U8(1)).expect("push destination index");

        assert_eq!(
            handle_accounts(CLOSE_ACCOUNT, &mut ctx),
            Err(VMErrorCode::ScriptNotAuthorized)
        );
    }
}
