//! Invoke operations handler for MitoVM system calls
//!
//! This module handles cross-program invocation (CPI) operations using INVOKE
//! and INVOKE_SIGNED opcodes. It manages Solana program invocation with
//! stack-based account handling and instruction data processing.

use crate::{
    context::ExecutionManager,
    debug_log,
    error::{CompactResult, VMErrorCode},
};
use five_protocol::{opcodes::*, ValueRef};
use pinocchio::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction, Seed, Signer},
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
};

const MAX_CPI_DATA_LEN: usize = 255;

/// Handle invoke operations for cross-program invocation
#[inline(always)]
pub fn handle_invoke_ops(opcode: u8, ctx: &mut ExecutionManager) -> CompactResult<()> {
    use crate::error_log;
    match opcode {
        INVOKE => {

            // Pop parameters from stack
            let count_val = ctx.pop()?;
            let accounts_count = count_val.as_u8().ok_or_else(|| {
                error_log!("INVOKE: accounts_count type mismatch. Got TypeID: {}", count_val.type_id() as u64);
                VMErrorCode::TypeMismatch
            })?;

            // Validate account count
            const MAX_ACCOUNTS: usize = 16;
            if accounts_count as usize > MAX_ACCOUNTS {
                return Err(VMErrorCode::InvalidOperation);
            }

            // Pop account indices
            let mut account_indices: [usize; MAX_ACCOUNTS] = [0; MAX_ACCOUNTS];
            for i in 0..accounts_count {
                let val = ctx.pop()?;
                let idx = val.as_u8().ok_or_else(|| {
                    error_log!("INVOKE: account_index[{}] type mismatch. Got TypeID: {}", i as u64, val.type_id() as u64);
                    VMErrorCode::TypeMismatch
                })?;
                account_indices[(accounts_count - 1 - i) as usize] = idx as usize;
            }

            // Pop instruction data and program ID.
            let data_ref = ctx.pop()?;
            let program_id_ref = ctx.pop()?;

            let mut instruction_data_owned = [0u8; MAX_CPI_DATA_LEN];
            let instruction_data: &[u8];
            match data_ref {
                ValueRef::U64(amount) => {
                    // ... (U64 case)
                    let discriminator_bytes = 2u32.to_le_bytes();
                    let amount_bytes = amount.to_le_bytes();
                    instruction_data_owned[0..4].copy_from_slice(&discriminator_bytes);
                    instruction_data_owned[4..12].copy_from_slice(&amount_bytes);
                    instruction_data = &instruction_data_owned[..12];
                }
                ValueRef::TempRef(offset, len) => {
                    let start = offset as usize;
                    let end = start + len as usize;
                    if end > ctx.temp_buffer().len() {
                        return Err(VMErrorCode::MemoryViolation);
                    }
                    // Reject instruction data larger than VM CPI payload limit.
                    if len as usize > MAX_CPI_DATA_LEN {
                        return Err(VMErrorCode::InvalidOperation);
                    }
                    instruction_data = &ctx.temp_buffer()[start..end];
                }
                ValueRef::ArrayRef(array_id) => {
                    let start = array_id as usize;
                    let temp = ctx.temp_buffer();
                    if start + 2 > temp.len() {
                        return Err(VMErrorCode::MemoryViolation);
                    }

                    let element_count = temp[start] as usize;
                    if element_count > MAX_CPI_DATA_LEN {
                        return Err(VMErrorCode::InvalidOperation);
                    }

                    let mut offset = start + 2;
                    let mut write_offset = 0usize;

                    for _i in 0..element_count {
                        if offset >= temp.len() {
                            return Err(VMErrorCode::MemoryViolation);
                        }

                        let type_id = temp[offset];

                        if type_id == five_protocol::types::U8
                            || type_id == five_protocol::types::BOOL
                        {
                            if offset + 1 >= temp.len() {
                                return Err(VMErrorCode::MemoryViolation);
                            }
                            if write_offset + 1 > instruction_data_owned.len() {
                                return Err(VMErrorCode::InvalidOperation);
                            }
                            instruction_data_owned[write_offset] = temp[offset + 1];
                            write_offset += 1;
                            offset += 2;
                        } else if type_id == five_protocol::types::U64
                            || type_id == five_protocol::types::I64
                        {
                            if offset + 8 >= temp.len() {
                                return Err(VMErrorCode::MemoryViolation);
                            }
                            if write_offset + 8 > instruction_data_owned.len() {
                                return Err(VMErrorCode::InvalidOperation);
                            }
                            instruction_data_owned[write_offset..write_offset + 8]
                                .copy_from_slice(&temp[offset + 1..offset + 9]);
                            write_offset += 8;
                            offset += 9;
                        } else if type_id == five_protocol::types::U32 {
                            if offset + 4 >= temp.len() {
                                return Err(VMErrorCode::MemoryViolation);
                            }
                            if write_offset + 4 > instruction_data_owned.len() {
                                return Err(VMErrorCode::InvalidOperation);
                            }
                            instruction_data_owned[write_offset..write_offset + 4]
                                .copy_from_slice(&temp[offset + 1..offset + 5]);
                            write_offset += 4;
                            offset += 5;
                        } else if type_id == five_protocol::types::PUBKEY {
                            // ValueRef::PubkeyRef(u16): [type_id][offset_lo][offset_hi]
                            if offset + 2 >= temp.len() {
                                return Err(VMErrorCode::MemoryViolation);
                            }
                            if write_offset + 32 > instruction_data_owned.len() {
                                return Err(VMErrorCode::InvalidOperation);
                            }

                            let mut ref_bytes = [0u8; 2];
                            ref_bytes.copy_from_slice(&temp[offset + 1..offset + 3]);
                            let pk_ref = ValueRef::PubkeyRef(u16::from_le_bytes(ref_bytes));
                            let pk_bytes = ctx.extract_pubkey(&pk_ref)?;
                            instruction_data_owned[write_offset..write_offset + 32]
                                .copy_from_slice(&pk_bytes);

                            write_offset += 32;
                            offset += 3;
                        } else if type_id == five_protocol::types::ACCOUNT {
                            // ValueRef::AccountRef(u8, u16): [type_id][account_idx][offset_lo][offset_hi]
                            // For CPI data packing, AccountRef only supports offset=0 and resolves
                            // to the account address (pubkey), not account data bytes.
                            if offset + 3 >= temp.len() {
                                return Err(VMErrorCode::MemoryViolation);
                            }
                            if write_offset + 32 > instruction_data_owned.len() {
                                return Err(VMErrorCode::InvalidOperation);
                            }

                            let account_idx = temp[offset + 1] as usize;
                            let mut off_bytes = [0u8; 2];
                            off_bytes.copy_from_slice(&temp[offset + 2..offset + 4]);
                            let account_offset = u16::from_le_bytes(off_bytes);
                            if account_offset != 0 {
                                return Err(VMErrorCode::TypeMismatch);
                            }
                            let accounts = ctx.accounts();
                            if account_idx >= accounts.len() {
                                return Err(VMErrorCode::InvalidAccountIndex);
                            }

                            instruction_data_owned[write_offset..write_offset + 32]
                                .copy_from_slice(accounts[account_idx].key().as_ref());
                            write_offset += 32;
                            offset += 4;
                        } else if type_id == 16 {
                            // ValueRef::TempRef(u8, u8): [type_id][temp_offset][len]
                            // Accept TempRef(len=32) as pubkey bytes.
                            if offset + 2 >= temp.len() {
                                return Err(VMErrorCode::MemoryViolation);
                            }
                            if write_offset + 32 > instruction_data_owned.len() {
                                return Err(VMErrorCode::InvalidOperation);
                            }

                            let temp_offset = temp[offset + 1] as usize;
                            let temp_len = temp[offset + 2] as usize;
                            if temp_len != 32 {
                                return Err(VMErrorCode::TypeMismatch);
                            }

                            let end = temp_offset + 32;
                            if end > ctx.temp_buffer().len() {
                                return Err(VMErrorCode::MemoryViolation);
                            }
                            instruction_data_owned[write_offset..write_offset + 32]
                                .copy_from_slice(&ctx.temp_buffer()[temp_offset..end]);

                            write_offset += 32;
                            offset += 3;
                        } else if type_id == five_protocol::types::U128 {
                            error_log!("INVOKE: Data Element {} TypeID U128 not supported", _i as u64);
                            return Err(VMErrorCode::TypeMismatch);
                        } else {
                            error_log!("INVOKE: Data Element {} Unknown TypeID: {}", _i as u64, type_id as u64);
                            return Err(VMErrorCode::TypeMismatch);
                        }
                    }

                    instruction_data = &instruction_data_owned[..write_offset];
                }
                _ => {
                    error_log!("INVOKE: instruction_data type mismatch. Got TypeID: {}", data_ref.type_id() as u64);
                    return Err(VMErrorCode::TypeMismatch);
                }
            };
            
            let program_id_bytes = ctx.extract_pubkey(&program_id_ref).inspect_err(|_e| {
                 error_log!("INVOKE: extract_pubkey failed for TypeID: {}", program_id_ref.type_id() as u64);
            })?;
            let program_id = Pubkey::from(program_id_bytes);

            debug_log!(
                "MitoVM: INVOKE instruction_data len: {}",
                instruction_data.len() as u32
            );
            
            // FORCE LOGGING loop - Cleaned up for production
            // error_log!("INVOKE Data Len: {}", instruction_len as u64);
            // for (idx, byte) in instruction_data[..instruction_len].iter().enumerate() {
            //     error_log!("Byte {}: {}", idx as u64, *byte as u64);
            // }

            // Validate account indices
            let accounts = ctx.accounts();
            for i in 0..accounts_count as usize {
                let idx = account_indices[i];
                if idx >= accounts.len() {
                    debug_log!("MitoVM: INVOKE invalid account index: {}", idx as u32);
                    return Err(VMErrorCode::InvalidAccountIndex);
                }
            }

            // Create account metas using stack array (no heap!)
            let mut account_metas: [AccountMeta; MAX_ACCOUNTS] =
                core::array::from_fn(|_| AccountMeta {
                    pubkey: accounts[0].key(),
                    is_signer: false,
                    is_writable: false,
                });

            // Fix: Stack pops arguments in reverse order (LIFO), so we must reverse accounts to match definition order (mint, dest, auth)
            account_indices[..accounts_count as usize].reverse();
            for i in 0..accounts_count as usize {
                let idx = account_indices[i];
                let account = &accounts[idx];
                account_metas[i] = AccountMeta {
                    pubkey: account.key(),
                    is_signer: account.is_signer(),
                    is_writable: account.is_writable(),
                };
            }

            // Create instruction with stack slice
            let instruction = Instruction {
                program_id: &program_id,
                accounts: &account_metas[..accounts_count as usize], // Use only the filled portion
                data: instruction_data, // Pinocchio requires slice here
            };

            // Collect account infos for the invoke using stack array
            let mut invoke_accounts: [&AccountInfo; MAX_ACCOUNTS] = [&accounts[0]; MAX_ACCOUNTS];
            for i in 0..accounts_count as usize {
                invoke_accounts[i] = &accounts[account_indices[i]];
            }

            // Execute the invoke
            debug_log!("MitoVM: Executing invoke");
            match invoke::<MAX_ACCOUNTS>(&instruction, &invoke_accounts) {
                Ok(()) => {
                    debug_log!("MitoVM: INVOKE completed successfully");

                    // CRITICAL FIX: Refresh account pointers after CPI
                    // When the Solana runtime executes invoke(), it may reallocate account data,
                    // rendering previously cached account info pointers stale.
                    // This call forces Pinocchio to recalculate data pointers for affected accounts.
                    let _ = ctx.refresh_account_pointers_after_cpi(&account_indices[..accounts_count as usize]);

                    ctx.push(ValueRef::Bool(true))?;
                }
                Err(e) => {
                    debug_log!("MitoVM: INVOKE failed");
                    // Map specific program errors to VM errors
                    let vm_error_code = match e {
                        ProgramError::MissingRequiredSignature => VMErrorCode::AccountNotSigner,
                        ProgramError::NotEnoughAccountKeys => VMErrorCode::InvalidAccountIndex,
                        ProgramError::InvalidAccountData => VMErrorCode::AccountError,
                        ProgramError::Custom(1104) => VMErrorCode::ExternalAccountLamportSpend,
                        _ => VMErrorCode::InvokeError,
                    };
                    return Err(vm_error_code);
                }
            }
        }
        INVOKE_SIGNED => {
            debug_log!("MitoVM: INVOKE_SIGNED operation");

            // Pop core CPI arguments
            let accounts_count = ctx.pop()?.as_u8().ok_or(VMErrorCode::TypeMismatch)?;
            let instruction_data_ref = ctx.pop()?;
            let program_id_ref = ctx.pop()?;

            // Pop seed count
            let seeds_count = ctx.pop()?.as_u8().ok_or(VMErrorCode::TypeMismatch)?;
            debug_log!("MitoVM: INVOKE_SIGNED seeds_count: {}", seeds_count);

            // Use stack-allocated seed storage (no heap!)
            const MAX_SEEDS: usize = 8;
            const MAX_SEED_LEN: usize = 32;

            if seeds_count as usize > MAX_SEEDS {
                return Err(VMErrorCode::InvalidOperation);
            }

            let mut seed_storage: [[u8; MAX_SEED_LEN]; MAX_SEEDS] =
                [[0u8; MAX_SEED_LEN]; MAX_SEEDS];
            let mut seed_lengths: [u8; MAX_SEEDS] = [0u8; MAX_SEEDS];

            for i in 0..seeds_count as usize {
                let seed_len = ctx.pop()?.as_u8().ok_or(VMErrorCode::TypeMismatch)?;
                let seed_value_ref = ctx.pop()?;

                if seed_len as usize > MAX_SEED_LEN {
                    return Err(VMErrorCode::InvalidOperation);
                }

                let seed_slice = match seed_value_ref {
                    ValueRef::TempRef(offset, len) => {
                        if len != seed_len {
                            return Err(VMErrorCode::InvalidOperation);
                        }
                        let start = offset as usize;
                        let end = start + len as usize;
                        &ctx.temp_buffer()[start..end]
                    }
                    _ => return Err(VMErrorCode::TypeMismatch),
                };

                // Copy to stack storage (no heap allocation)
                seed_storage[i][..seed_len as usize].copy_from_slice(seed_slice);
                seed_lengths[i] = seed_len;
            }

            if seeds_count == 0 {
                return Err(VMErrorCode::InvalidSeedArray);
            }

            // Resolve program_id and instruction_data
            let program_id_bytes = ctx.extract_pubkey(&program_id_ref)?;
            let program_id = Pubkey::from(program_id_bytes);

            // Extract instruction data using stack buffer (no heap!)
            let mut instruction_data_buf: [u8; MAX_CPI_DATA_LEN] = [0u8; MAX_CPI_DATA_LEN];
            let instruction_data_len: usize;

            match instruction_data_ref {
                ValueRef::TempRef(offset, len) => {
                    if len as usize > instruction_data_buf.len() {
                        return Err(VMErrorCode::InvalidOperation);
                    }
                    let start = offset as usize;
                    let end = start + len as usize;
                    instruction_data_buf[..len as usize]
                        .copy_from_slice(&ctx.temp_buffer()[start..end]);
                    instruction_data_len = len as usize;
                }
                _ => return Err(VMErrorCode::TypeMismatch),
            };

            // Create AccountMetas and invoke_accounts array using stack arrays (no heap!)
            const MAX_ACCOUNTS: usize = 16; // Same limit as INVOKE
            if accounts_count as usize > MAX_ACCOUNTS {
                return Err(VMErrorCode::InvalidOperation);
            }

            // Collect account indices first to avoid borrowing conflicts
            let mut account_indices: [usize; MAX_ACCOUNTS] = [0; MAX_ACCOUNTS];
            for i in 0..accounts_count as usize {
                let account_idx = ctx.pop()?.as_u8().ok_or(VMErrorCode::TypeMismatch)? as usize;
                if account_idx >= ctx.accounts().len() {
                    return Err(VMErrorCode::InvalidAccountIndex);
                }
                account_indices[(accounts_count as usize - 1) - i] = account_idx;
                // Reverse order due to stack pop
            }

            // Initialize account_metas after we're done with ctx.pop()
            let accounts_ref = ctx.accounts();
            let mut account_metas: [AccountMeta; MAX_ACCOUNTS] =
                core::array::from_fn(|_| AccountMeta {
                    pubkey: accounts_ref[0].key(),
                    is_signer: false,
                    is_writable: false,
                });

            // Create invoke_accounts array after we have all indices
            let mut invoke_accounts: [&AccountInfo; MAX_ACCOUNTS] =
                [&accounts_ref[0]; MAX_ACCOUNTS];
            for i in 0..accounts_count as usize {
                let account_idx = account_indices[i];
                invoke_accounts[i] = &accounts_ref[account_idx];
            }

            for i in 0..accounts_count as usize {
                let account = invoke_accounts[i];
                account_metas[i] = AccountMeta {
                    pubkey: account.key(),
                    is_signer: account.is_signer(),
                    is_writable: account.is_writable(),
                };
            }

            let instruction = Instruction {
                program_id: &program_id,
                accounts: &account_metas[..accounts_count as usize],
                data: &instruction_data_buf[..instruction_data_len],
            };

            // Create signer from seeds using stack arrays (no heap!)
            let mut seeds_refs: [Seed; MAX_SEEDS] =
                core::array::from_fn(|_| Seed::from(&[0u8][..])); // Default empty seed

            for i in 0..seeds_count as usize {
                let seed_slice = &seed_storage[i][..seed_lengths[i] as usize];
                seeds_refs[i] = Seed::from(seed_slice);
            }
            let signer = Signer::from(&seeds_refs[..seeds_count as usize]);

            // Execute the invoke_signed
            debug_log!("MitoVM: Executing invoke_signed");

            invoke_signed::<MAX_ACCOUNTS>(&instruction, &invoke_accounts, &[signer]).map_err(
                |e| {
                    debug_log!("MitoVM: INVOKE_SIGNED failed");
                    match e {
                        ProgramError::MissingRequiredSignature => VMErrorCode::AccountNotSigner,
                        _ => VMErrorCode::InvokeError,
                    }
                },
            )?;

            debug_log!("MitoVM: INVOKE_SIGNED completed successfully");

            // CRITICAL FIX: Refresh account pointers after CPI (same as INVOKE)
            let _ = ctx.refresh_account_pointers_after_cpi(&account_indices[..accounts_count as usize]);

            ctx.push(ValueRef::Bool(true))?;
        }
        _ => {
            debug_log!("MitoVM: Invoke opcode {} not implemented", opcode);
            return Err(VMErrorCode::InvalidInstruction);
        }
    }
    Ok(())
}
