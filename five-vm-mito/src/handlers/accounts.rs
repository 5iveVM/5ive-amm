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
use pinocchio::pubkey::Pubkey;

/// Execute Solana account operations including creation, metadata access, and lamport management.
/// Handles the 0x50-0x5F opcode range.
#[inline(never)]
pub fn handle_accounts(opcode: u8, ctx: &mut ExecutionManager) -> CompactResult<()> {
    match opcode {
        CREATE_ACCOUNT => {
            let owner_ref = ctx.pop()?;
            let space = ctx.pop()?.as_u64().ok_or(VMErrorCode::TypeMismatch)?;
            let lamports = ctx.pop()?.as_u64().ok_or(VMErrorCode::TypeMismatch)?;
            let account_idx = ctx.pop()?.as_account_idx().ok_or(VMErrorCode::TypeMismatch)?;

            // Extract owner pubkey
            let owner_bytes = ctx.extract_pubkey(&owner_ref)?;
            let owner = Pubkey::from(owner_bytes);

            debug_log!(
                "MitoVM: CREATE_ACCOUNT idx: {}, lamports: {}, space: {}",
                account_idx,
                lamports,
                space
            );

            // Try to create the account using CPI
            // If it fails (e.g. already exists), we check if it matches requirements
            match ctx.create_account(account_idx, space, lamports, &owner) {
                Ok(_) => {
                    ctx.push(ValueRef::Bool(true))?;
                }
                Err(_) => {
                    // Fallback: Check if existing account satisfies requirements
                    let account = ctx.get_account(account_idx)?;
                    if account.lamports() < lamports
                        // SAFETY: Read-only access to check length
                        || unsafe { account.borrow_data_unchecked() }.len() < space as usize
                    {
                        ctx.push(ValueRef::Bool(false))?;
                    } else {
                        // Account exists and is sufficient
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
            let account = ctx.get_account(account_idx)?;
            let key_bytes = *account.key();

            // Store pubkey in temp buffer and return TempRef for zero-copy access
            let temp_offset = ctx.alloc_temp(32)?;
            ctx.temp_buffer_mut()[temp_offset as usize..(temp_offset as usize + 32)]
                .copy_from_slice(&key_bytes);
            ctx.push(ValueRef::TempRef(temp_offset, 32))?;
            debug_log!(
                "MitoVM: GET_KEY account {} -> TempRef({}, 32)",
                account_idx,
                temp_offset
            );
        }
        GET_DATA => {
            let account_idx = ctx.fetch_byte()?;

            // Zero-copy approach: Return direct AccountRef without temp buffer allocation
            // This maintains MitoVM's zero-allocation principle
            let data_len = {
                let account = ctx.get_account(account_idx)?;
                // SAFETY: We only need the length, not the data itself
                unsafe { account.borrow_data_unchecked() }.len()
            };

            // Ensure data length fits in u16 for AccountRef offset
            let _data_len_u16 = u16::try_from(data_len).map_err(|_| VMErrorCode::MemoryError)?;

            // Return direct reference to account data (zero-copy)
            ctx.push(ValueRef::AccountRef(account_idx, 0))?;

            debug_log!(
                "MitoVM: GET_DATA account {} -> AccountRef({}, 0)",
                account_idx,
                account_idx
            );
        }
        GET_OWNER => {
            let account_idx = ctx.fetch_byte()?;
            let account = ctx.get_account(account_idx)?;
            let owner_bytes = *account.owner();

            // Store owner pubkey in temp buffer and return TempRef for zero-copy access
            let temp_offset = ctx.alloc_temp(32)?;
            ctx.temp_buffer_mut()[temp_offset as usize..(temp_offset as usize + 32)]
                .copy_from_slice(&owner_bytes);
            ctx.push(ValueRef::TempRef(temp_offset, 32))?;
            debug_log!(
                "MitoVM: GET_OWNER account {} -> TempRef({}, 32)",
                account_idx,
                temp_offset
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
        _ => return Err(VMErrorCode::InvalidInstruction.into()),
    }
    Ok(())
}
