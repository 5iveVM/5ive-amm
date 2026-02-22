//! Account operations handler for MitoVM
//!
//! This module handles account operations including CREATE_ACCOUNT, LOAD_ACCOUNT,
//! SAVE_ACCOUNT, GET_LAMPORTS, GET_KEY, GET_DATA, GET_OWNER, and SET_LAMPORTS.
//! It manages Solana account state and metadata access.

use crate::{
    context::ExecutionManager,
    debug_log,
    error::{CompactResult, VMErrorCode},
    error_log,
    pop_u64,
};
use core::convert::TryFrom;
use five_protocol::{opcodes::*, ValueRef};
use pinocchio::pubkey::Pubkey;
#[cfg(target_os = "solana")]
use pinocchio::instruction::{AccountMeta, Instruction};
#[cfg(target_os = "solana")]
use pinocchio::program::invoke_signed;

const CLOSED_MARKER: [u8; 4] = *b"CLSD";

/// Execute Solana account operations including creation, metadata access, and lamport management.
/// Handles the 0x50-0x5F opcode range.
#[inline(always)]
pub fn handle_accounts(opcode: u8, ctx: &mut ExecutionManager) -> CompactResult<()> {
    match opcode {
        CREATE_ACCOUNT => {
            let owner_ref = ctx.pop()?;
            let space = ctx.pop()?.as_u64().ok_or(VMErrorCode::TypeMismatch)?;
            let lamports = ctx.pop()?.as_u64().ok_or(VMErrorCode::TypeMismatch)?;
            let account_idx = ctx.pop()?.as_account_idx().ok_or(VMErrorCode::TypeMismatch)?;

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
            let account_idx = ctx.pop()?.as_account_idx().ok_or(VMErrorCode::TypeMismatch)?;

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
            let account_idx = ctx.pop()?.as_account_idx().ok_or(VMErrorCode::TypeMismatch)?;

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
            // SAFETY: Copy key bytes immediately to avoid holding account borrow
            let key_bytes = *ctx.get_account(account_idx)?.key();

            push_bytes_as_temp(ctx, &key_bytes, "GET_KEY", account_idx)?;
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
            // SAFETY: Copy owner bytes immediately to avoid holding account borrow
            let owner_bytes = *ctx.get_account(account_idx)?.owner();

            push_bytes_as_temp(ctx, &owner_bytes, "GET_OWNER", account_idx)?;
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
            let to_idx = ctx.pop()?.as_account_idx().ok_or(VMErrorCode::TypeMismatch)?;
            let from_idx = ctx.pop()?.as_account_idx().ok_or(VMErrorCode::TypeMismatch)?;

            let from = ctx.get_account(from_idx)?;
            let to = ctx.get_account(to_idx)?;

            if !from.is_writable() || !to.is_writable() {
                return Err(VMErrorCode::AccountNotWritable);
            }
            if opcode == TRANSFER && !from.is_signer() {
                return Err(VMErrorCode::ConstraintViolation);
            }

            #[cfg(target_os = "solana")]
            {
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
            let destination_idx = ctx.pop()?.as_account_idx().ok_or(VMErrorCode::TypeMismatch)?;
            let source_idx = ctx.pop()?.as_account_idx().ok_or(VMErrorCode::TypeMismatch)?;

            if source_idx == destination_idx {
                return Err(VMErrorCode::ConstraintViolation);
            }

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

/// Helper to allocate temp space, copy bytes, and push TempRef
/// Encapsulates common pattern for GET_KEY, GET_OWNER etc.
#[inline(always)]
fn push_bytes_as_temp(
    ctx: &mut ExecutionManager,
    bytes: &[u8],
    op_name: &str,
    account_idx: u8,
) -> CompactResult<()> {
    // We assume bytes.len() fits in u8 for these operations (32 bytes for Pubkey)
    let len = bytes.len() as u8;
    match ctx.alloc_temp(len) {
        Ok(temp_offset) => {
            debug_log!(
                "MitoVM: {} account {} allocated temp at offset {}",
                op_name,
                account_idx,
                temp_offset
            );
            ctx.temp_buffer_mut()[temp_offset as usize..(temp_offset as usize + bytes.len())]
                .copy_from_slice(bytes);
            ctx.push(ValueRef::TempRef(temp_offset, len))?;
            debug_log!(
                "MitoVM: {} account {} pushed TempRef({}, {})",
                op_name,
                account_idx,
                temp_offset,
                len
            );
            Ok(())
        }
        Err(e) => {
            error_log!(
                "MitoVM: {} account {} alloc_temp({}) FAILED - error code: {}",
                op_name,
                account_idx,
                len,
                e as u32
            );
            Err(e)
        }
    }
}
