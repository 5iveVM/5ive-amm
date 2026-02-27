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
    program::{invoke_signed_with_bounds, invoke_with_bounds},
    program_error::ProgramError,
    pubkey::Pubkey,
};
#[cfg(target_os = "solana")]
use pinocchio::pubkey::create_program_address;

const MAX_CPI_DATA_LEN: usize = 255;
const MAX_CPI_ACCOUNTS: usize = 16;
const MAX_SIGNER_GROUPS: usize = 4;
const MAX_SIGNER_SEEDS: usize = 8;
const MAX_SIGNER_SEED_LEN: usize = 32;
const GROUPED_SIGNER_WORKSPACE_CAPACITY: usize =
    1 + MAX_SIGNER_GROUPS * (1 + MAX_SIGNER_SEEDS * (1 + MAX_SIGNER_SEED_LEN));

fn parse_array_value_refs<const N: usize>(
    ctx: &ExecutionManager,
    array_ref: ValueRef,
    out: &mut [ValueRef; N],
) -> CompactResult<usize> {
    let ValueRef::ArrayRef(array_id) = array_ref else {
        return Err(VMErrorCode::TypeMismatch);
    };

    let start = array_id as usize;
    let temp = ctx.temp_buffer();
    if start + 2 > temp.len() {
        return Err(VMErrorCode::MemoryViolation);
    }

    let element_count = temp[start] as usize;
    if element_count > out.len() {
        return Err(VMErrorCode::InvalidOperation);
    }

    let mut cursor = start + 2;
    for slot in out.iter_mut().take(element_count) {
        if cursor >= temp.len() {
            return Err(VMErrorCode::MemoryViolation);
        }
        let value_ref =
            ValueRef::deserialize_from(&temp[cursor..]).map_err(|_| VMErrorCode::TypeMismatch)?;
        *slot = value_ref;
        cursor += value_ref.serialized_size();
    }

    Ok(element_count)
}

fn map_invoke_error(err: ProgramError) -> VMErrorCode {
    match err {
        ProgramError::MissingRequiredSignature => VMErrorCode::AccountNotSigner,
        ProgramError::NotEnoughAccountKeys => VMErrorCode::InvalidAccountIndex,
        ProgramError::InvalidAccountData => VMErrorCode::AccountError,
        ProgramError::Custom(1104) => VMErrorCode::ExternalAccountLamportSpend,
        _ => VMErrorCode::InvokeError,
    }
}

fn mark_matching_signer_meta(
    account_metas: &mut [AccountMeta; MAX_CPI_ACCOUNTS],
    invoke_accounts: &[&AccountInfo; MAX_CPI_ACCOUNTS + 1],
    accounts_count: usize,
    signer_pubkey: &Pubkey,
) -> bool {
    for i in 0..accounts_count {
        if invoke_accounts[i].key() == signer_pubkey {
            account_metas[i].is_signer = true;
            return true;
        }
    }
    false
}

fn derive_pda_from_seed_slices(seed_slices: &[&[u8]], program_id: &Pubkey) -> CompactResult<Pubkey> {
    #[cfg(target_os = "solana")]
    {
        create_program_address(seed_slices, program_id).map_err(|_| VMErrorCode::PdaDerivationFailed)
    }
    #[cfg(not(target_os = "solana"))]
    {
        crate::utils::derive_pda_offchain(seed_slices, program_id)
    }
}

fn write_seed_value_into_slice(
    ctx: &ExecutionManager,
    seed_value: ValueRef,
    out: &mut [u8; MAX_SIGNER_SEED_LEN],
) -> CompactResult<usize> {
    match seed_value {
        ValueRef::U64(val) => {
            out[..8].copy_from_slice(&val.to_le_bytes());
            Ok(8)
        }
        ValueRef::U8(val) => {
            out[0] = val;
            Ok(1)
        }
        ValueRef::PubkeyRef(_) => {
            let bytes = ctx.extract_pubkey(&seed_value)?;
            out[..32].copy_from_slice(&bytes);
            Ok(32)
        }
        ValueRef::AccountRef(account_idx, account_offset) => {
            if account_offset != 0 {
                return Err(VMErrorCode::TypeMismatch);
            }
            let account = ctx
                .accounts()
                .get(account_idx as usize)
                .ok_or(VMErrorCode::InvalidAccountIndex)?;
            out[..32].copy_from_slice(account.key().as_ref());
            Ok(32)
        }
        ValueRef::TempRef(offset, len) => {
            let start = offset as usize;
            let end = start + len as usize;
            if end > ctx.temp_buffer().len() {
                return Err(VMErrorCode::MemoryViolation);
            }
            let copy_len = usize::from(len.min(MAX_SIGNER_SEED_LEN as u8));
            out[..copy_len].copy_from_slice(&ctx.temp_buffer()[start..start + copy_len]);
            Ok(copy_len)
        }
        ValueRef::StringRef(_) | ValueRef::HeapString(_) => {
            let (len, bytes) = ctx.extract_string_slice(&seed_value)?;
            let copy_len = (len as usize).min(MAX_SIGNER_SEED_LEN);
            out[..copy_len].copy_from_slice(&bytes[..copy_len]);
            Ok(copy_len)
        }
        ValueRef::ArrayRef(array_id) => {
            let start = array_id as usize;
            if start + 2 > ctx.temp_buffer().len() {
                return Err(VMErrorCode::MemoryViolation);
            }
            let len = ctx.temp_buffer()[start] as usize;
            let data_start = start + 2;
            let data_end = data_start + len;
            if data_end > ctx.temp_buffer().len() {
                return Err(VMErrorCode::MemoryViolation);
            }
            let copy_len = len.min(MAX_SIGNER_SEED_LEN);
            out[..copy_len].copy_from_slice(&ctx.temp_buffer()[data_start..data_start + copy_len]);
            Ok(copy_len)
        }
        _ => Err(VMErrorCode::TypeMismatch),
    }
}

#[inline(never)]
fn derive_and_mark_workspace_group_signer(
    ctx: &ExecutionManager,
    workspace_base: u32,
    group_offset: u16,
    caller_program_id: &Pubkey,
    account_metas: &mut [AccountMeta; MAX_CPI_ACCOUNTS],
    invoke_accounts: &[&AccountInfo; MAX_CPI_ACCOUNTS + 1],
    accounts_count: usize,
) -> CompactResult<()> {
    let mut seed_storage = [[0u8; MAX_SIGNER_SEED_LEN]; MAX_SIGNER_SEEDS];
    let mut seed_lengths = [0usize; MAX_SIGNER_SEEDS];
    let mut seed_slices: [&[u8]; MAX_SIGNER_SEEDS] = [&[]; MAX_SIGNER_SEEDS];
    let group_start = workspace_base + u32::from(group_offset);
    let seed_count = ctx.get_heap_data(group_start, 1)?[0] as usize;
    let mut cursor = group_start + 1;

    for seed_idx in 0..seed_count {
        let seed_len = ctx.get_heap_data(cursor, 1)?[0] as usize;
        cursor += 1;
        let seed_bytes = ctx.get_heap_data(cursor, seed_len as u32)?;
        seed_storage[seed_idx][..seed_len].copy_from_slice(seed_bytes);
        seed_lengths[seed_idx] = seed_len;
        cursor += seed_len as u32;
    }
    for seed_idx in 0..seed_count {
        seed_slices[seed_idx] = &seed_storage[seed_idx][..seed_lengths[seed_idx]];
    }

    let signer_pubkey =
        derive_pda_from_seed_slices(&seed_slices[..seed_count], caller_program_id)?;
    if !mark_matching_signer_meta(account_metas, invoke_accounts, accounts_count, &signer_pubkey) {
        return Err(VMErrorCode::ConstraintViolation);
    }

    Ok(())
}

#[inline(never)]
fn materialize_grouped_signer_workspace(
    ctx: &mut ExecutionManager,
    signer_groups_ref: ValueRef,
    group_offsets: &mut [u16; MAX_SIGNER_GROUPS],
) -> CompactResult<(u32, usize)> {
    let mut group_refs = [ValueRef::Bool(false); MAX_SIGNER_GROUPS];
    let group_count = parse_array_value_refs(ctx, signer_groups_ref, &mut group_refs)?;
    if group_count == 0 {
        return Err(VMErrorCode::InvalidSeedArray);
    }

    let workspace_base = ctx.heap_alloc(GROUPED_SIGNER_WORKSPACE_CAPACITY)?;
    ctx.get_heap_data_mut(workspace_base, 1)?[0] = group_count as u8;

    let mut cursor = workspace_base + 1;
    let mut inner_refs = [ValueRef::Bool(false); MAX_SIGNER_SEEDS];

    for group_idx in 0..group_count {
        let inner_count = parse_array_value_refs(ctx, group_refs[group_idx], &mut inner_refs)?;
        if inner_count == 0 {
            return Err(VMErrorCode::InvalidSeedArray);
        }

        group_offsets[group_idx] = (cursor - workspace_base) as u16;
        ctx.get_heap_data_mut(cursor, 1)?[0] = inner_count as u8;
        cursor += 1;

        for seed_idx in 0..inner_count {
            let mut seed_buf = [0u8; MAX_SIGNER_SEED_LEN];
            let written = write_seed_value_into_slice(ctx, inner_refs[seed_idx], &mut seed_buf)?;
            if written == 0 {
                return Err(VMErrorCode::InvalidSeedArray);
            }
            ctx.get_heap_data_mut(cursor, 1)?[0] = written as u8;
            cursor += 1;
            ctx.get_heap_data_mut(cursor, written as u32)?
                .copy_from_slice(&seed_buf[..written]);
            cursor += written as u32;
        }

    }

    Ok((workspace_base, group_count))
}

#[inline(never)]
fn mark_grouped_signer_metas(
    ctx: &ExecutionManager,
    workspace_base: u32,
    group_count: usize,
    group_offsets: &[u16; MAX_SIGNER_GROUPS],
    caller_program_id: &Pubkey,
    account_metas: &mut [AccountMeta; MAX_CPI_ACCOUNTS],
    invoke_accounts: &[&AccountInfo; MAX_CPI_ACCOUNTS + 1],
    accounts_count: usize,
 ) -> CompactResult<()> {
    for group_idx in 0..group_count {
        derive_and_mark_workspace_group_signer(
            ctx,
            workspace_base,
            group_offsets[group_idx],
            caller_program_id,
            account_metas,
            invoke_accounts,
            accounts_count,
        )?;
    }
    Ok(())
}

#[inline(never)]
fn invoke_signed_with_grouped_workspace(
    ctx: &ExecutionManager,
    workspace_base: u32,
    group_count: usize,
    group_offsets: &[u16; MAX_SIGNER_GROUPS],
    program_id: &Pubkey,
    account_metas: &[AccountMeta; MAX_CPI_ACCOUNTS],
    accounts_count: usize,
    instruction_data: &[u8],
    invoke_accounts: &[&AccountInfo; MAX_CPI_ACCOUNTS + 1],
    invoke_account_len: usize,
) -> CompactResult<()> {
    let mut signer_seed_arrays: [[Seed; MAX_SIGNER_SEEDS]; MAX_SIGNER_GROUPS] =
        core::array::from_fn(|_| core::array::from_fn(|_| Seed::from(&[0u8][..])));
    let mut signers: [Signer; MAX_SIGNER_GROUPS] = core::array::from_fn(|_| Signer::from(&[]));

    for group_idx in 0..group_count {
        let group_start = workspace_base + u32::from(group_offsets[group_idx]);
        let seed_count = ctx.get_heap_data(group_start, 1)?[0] as usize;
        let mut cursor = group_start + 1;

        for seed_idx in 0..seed_count {
            let seed_len = ctx.get_heap_data(cursor, 1)?[0] as usize;
            cursor += 1;
            let seed_bytes = ctx.get_heap_data(cursor, seed_len as u32)?;
            signer_seed_arrays[group_idx][seed_idx] = Seed::from(seed_bytes);
            cursor += seed_len as u32;
        }
    }
    for group_idx in 0..group_count {
        let group_start = workspace_base + u32::from(group_offsets[group_idx]);
        let seed_count = ctx.get_heap_data(group_start, 1)?[0] as usize;
        signers[group_idx] = Signer::from(&signer_seed_arrays[group_idx][..seed_count]);
    }

    let instruction = Instruction {
        program_id,
        accounts: &account_metas[..accounts_count],
        data: instruction_data,
    };

    invoke_signed_with_bounds::<{ MAX_CPI_ACCOUNTS + 1 }>(
        &instruction,
        &invoke_accounts[..invoke_account_len],
        &signers[..group_count],
    )
    .map_err(map_invoke_error)
}

#[inline(never)]
fn invoke_signed_grouped_from_array_ref(
    ctx: &mut ExecutionManager,
    signer_groups_ref: ValueRef,
    accounts_count: u8,
    instruction_data_ref: ValueRef,
    program_id_ref: ValueRef,
) -> CompactResult<()> {
    debug_log!("MitoVM: INVOKE_SIGNED grouped signer payload");

    if accounts_count as usize > MAX_CPI_ACCOUNTS {
        return Err(VMErrorCode::InvalidOperation);
    }

    let program_id_bytes = ctx.extract_pubkey(&program_id_ref)?;
    let program_id = Pubkey::from(program_id_bytes);

    let mut instruction_data_buf = [0u8; MAX_CPI_DATA_LEN];
    let instruction_data_len =
        materialize_instruction_data(ctx, instruction_data_ref, &mut instruction_data_buf)?;

    let mut account_indices = [0usize; MAX_CPI_ACCOUNTS];
    for i in 0..accounts_count as usize {
        let account_idx = ctx.pop()?.as_u8().ok_or(VMErrorCode::TypeMismatch)? as usize;
        if account_idx >= ctx.accounts().len() {
            return Err(VMErrorCode::InvalidAccountIndex);
        }
        account_indices[(accounts_count as usize - 1) - i] = account_idx;
    }

    let mut group_offsets = [0u16; MAX_SIGNER_GROUPS];
    let (workspace_base, group_count) =
        materialize_grouped_signer_workspace(ctx, signer_groups_ref, &mut group_offsets)?;

    let invoke_result = {
        let accounts_ref = ctx.accounts();
        let mut account_metas: [AccountMeta; MAX_CPI_ACCOUNTS] =
            core::array::from_fn(|_| AccountMeta {
                pubkey: accounts_ref[0].key(),
                is_signer: false,
                is_writable: false,
            });
        let mut invoke_accounts: [&AccountInfo; MAX_CPI_ACCOUNTS + 1] =
            [&accounts_ref[0]; MAX_CPI_ACCOUNTS + 1];

        for i in 0..accounts_count as usize {
            let account_idx = account_indices[i];
            invoke_accounts[i] = &accounts_ref[account_idx];
            let account = invoke_accounts[i];
            account_metas[i] = AccountMeta {
                pubkey: account.key(),
                is_signer: account.is_signer(),
                is_writable: account.is_writable(),
            };
        }
        let mut invoke_account_len = accounts_count as usize;
        if let Some(program_account) = accounts_ref
            .iter()
            .find(|account| account.key() == &program_id)
        {
            invoke_accounts[invoke_account_len] = program_account;
            invoke_account_len += 1;
        }
        mark_grouped_signer_metas(
            ctx,
            workspace_base,
            group_count,
            &group_offsets,
            &ctx.program_id,
            &mut account_metas,
            &invoke_accounts,
            accounts_count as usize,
        )?;

        invoke_signed_with_grouped_workspace(
            ctx,
            workspace_base,
            group_count,
            &group_offsets,
            &program_id,
            &account_metas,
            accounts_count as usize,
            &instruction_data_buf[..instruction_data_len],
            &invoke_accounts,
            invoke_account_len,
        )
    };

    invoke_result?;
    let _ = ctx.refresh_account_pointers_after_cpi(&account_indices[..accounts_count as usize]);
    ctx.push(ValueRef::Bool(true))?;
    Ok(())
}

fn append_serialized_value(
    ctx: &ExecutionManager,
    value_ref: ValueRef,
    out: &mut [u8; MAX_CPI_DATA_LEN],
    write_offset: &mut usize,
) -> CompactResult<()> {
    match value_ref {
        ValueRef::U8(byte) => {
            if *write_offset + 1 > out.len() {
                return Err(VMErrorCode::InvalidOperation);
            }
            out[*write_offset] = byte;
            *write_offset += 1;
        }
        ValueRef::Bool(flag) => {
            if *write_offset + 1 > out.len() {
                return Err(VMErrorCode::InvalidOperation);
            }
            out[*write_offset] = u8::from(flag);
            *write_offset += 1;
        }
        ValueRef::U64(word) => {
            if *write_offset + 8 > out.len() {
                return Err(VMErrorCode::InvalidOperation);
            }
            out[*write_offset..*write_offset + 8].copy_from_slice(&word.to_le_bytes());
            *write_offset += 8;
        }
        ValueRef::I64(word) => {
            if *write_offset + 8 > out.len() {
                return Err(VMErrorCode::InvalidOperation);
            }
            out[*write_offset..*write_offset + 8].copy_from_slice(&word.to_le_bytes());
            *write_offset += 8;
        }
        ValueRef::U128(_) => return Err(VMErrorCode::TypeMismatch),
        ValueRef::PubkeyRef(_) => {
            let bytes = ctx.extract_pubkey(&value_ref)?;
            if *write_offset + bytes.len() > out.len() {
                return Err(VMErrorCode::InvalidOperation);
            }
            out[*write_offset..*write_offset + bytes.len()].copy_from_slice(&bytes);
            *write_offset += bytes.len();
        }
        ValueRef::AccountRef(account_idx, account_offset) => {
            if account_offset != 0 {
                return Err(VMErrorCode::TypeMismatch);
            }
            let account = ctx
                .accounts()
                .get(account_idx as usize)
                .ok_or(VMErrorCode::InvalidAccountIndex)?;
            let bytes = account.key().as_ref();
            if *write_offset + bytes.len() > out.len() {
                return Err(VMErrorCode::InvalidOperation);
            }
            out[*write_offset..*write_offset + bytes.len()].copy_from_slice(bytes);
            *write_offset += bytes.len();
        }
        ValueRef::TempRef(offset, len) => {
            let start = offset as usize;
            let len = len as usize;
            let end = start + len;
            if end > ctx.temp_buffer().len() || *write_offset + len > out.len() {
                return Err(VMErrorCode::MemoryViolation);
            }
            out[*write_offset..*write_offset + len].copy_from_slice(&ctx.temp_buffer()[start..end]);
            *write_offset += len;
        }
        ValueRef::StringRef(_) | ValueRef::HeapString(_) => {
            let (len, bytes) = ctx.extract_string_slice(&value_ref)?;
            let len = len as usize;
            if *write_offset + len > out.len() {
                return Err(VMErrorCode::InvalidOperation);
            }
            out[*write_offset..*write_offset + len].copy_from_slice(bytes);
            *write_offset += len;
        }
        _ => return Err(VMErrorCode::TypeMismatch),
    }

    Ok(())
}

fn materialize_instruction_data(
    ctx: &ExecutionManager,
    data_ref: ValueRef,
    instruction_data_owned: &mut [u8; MAX_CPI_DATA_LEN],
) -> CompactResult<usize> {
    match data_ref {
        ValueRef::U64(amount) => {
            let discriminator_bytes = 2u32.to_le_bytes();
            let amount_bytes = amount.to_le_bytes();
            instruction_data_owned[0..4].copy_from_slice(&discriminator_bytes);
            instruction_data_owned[4..12].copy_from_slice(&amount_bytes);
            Ok(12)
        }
        ValueRef::TempRef(offset, len) => {
            let start = offset as usize;
            let end = start + len as usize;
            if end > ctx.temp_buffer().len() {
                return Err(VMErrorCode::MemoryViolation);
            }
            if len as usize > MAX_CPI_DATA_LEN {
                return Err(VMErrorCode::InvalidOperation);
            }
            instruction_data_owned[..len as usize].copy_from_slice(&ctx.temp_buffer()[start..end]);
            Ok(len as usize)
        }
        ValueRef::StringRef(_) | ValueRef::HeapString(_) => {
            let (len, bytes) = ctx.extract_string_slice(&data_ref)?;
            let len = len as usize;
            if len > MAX_CPI_DATA_LEN {
                return Err(VMErrorCode::InvalidOperation);
            }
            instruction_data_owned[..len].copy_from_slice(bytes);
            Ok(len)
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

            for _ in 0..element_count {
                if offset >= temp.len() {
                    return Err(VMErrorCode::MemoryViolation);
                }

                let value_ref = ValueRef::deserialize_from(&temp[offset..])
                    .map_err(|_| VMErrorCode::TypeMismatch)?;
                append_serialized_value(ctx, value_ref, instruction_data_owned, &mut write_offset)?;
                offset += value_ref.serialized_size();
            }

            Ok(write_offset)
        }
        _ => Err(VMErrorCode::TypeMismatch),
    }
}

/// Handle invoke operations for cross-program invocation
#[inline(always)]
pub fn handle_invoke_ops(opcode: u8, ctx: &mut ExecutionManager) -> CompactResult<()> {
    match opcode {
        INVOKE => {
            // Pop parameters from stack
            let count_val = ctx.pop()?;
            let accounts_count = count_val.as_u8().ok_or(VMErrorCode::TypeMismatch)?;

            // Validate account count
            if accounts_count as usize > MAX_CPI_ACCOUNTS {
                return Err(VMErrorCode::InvalidOperation);
            }

            // Pop account indices
            let mut account_indices: [usize; MAX_CPI_ACCOUNTS] = [0; MAX_CPI_ACCOUNTS];
            for i in 0..accounts_count {
                let val = ctx.pop()?;
                let idx = val.as_u8().ok_or(VMErrorCode::TypeMismatch)?;
                account_indices[(accounts_count - 1 - i) as usize] = idx as usize;
            }

            // Pop instruction data and program ID.
            let data_ref = ctx.pop()?;
            let program_id_ref = ctx.pop()?;

            let mut instruction_data_owned = [0u8; MAX_CPI_DATA_LEN];
            let instruction_data_len =
                materialize_instruction_data(ctx, data_ref, &mut instruction_data_owned)?;
            let instruction_data = &instruction_data_owned[..instruction_data_len];

            let program_id_bytes = ctx.extract_pubkey(&program_id_ref)?;
            let program_id = Pubkey::from(program_id_bytes);

            debug_log!(
                "MitoVM: INVOKE instruction_data len: {}",
                instruction_data.len() as u32
            );

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
            let mut account_metas: [AccountMeta; MAX_CPI_ACCOUNTS] =
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

            // Collect account infos for the invoke using stack array.
            // Real Solana CPI requires the program account info to be supplied in addition
            // to the instruction meta accounts.
            let mut invoke_accounts: [&AccountInfo; MAX_CPI_ACCOUNTS + 1] =
                [&accounts[0]; MAX_CPI_ACCOUNTS + 1];
            for i in 0..accounts_count as usize {
                invoke_accounts[i] = &accounts[account_indices[i]];
            }
            let mut invoke_account_len = accounts_count as usize;
            if let Some(program_account) =
                accounts.iter().find(|account| account.key() == &program_id)
            {
                invoke_accounts[invoke_account_len] = program_account;
                invoke_account_len += 1;
            }

            // Execute the invoke
            debug_log!("MitoVM: Executing invoke");
            match invoke_with_bounds::<{ MAX_CPI_ACCOUNTS + 1 }>(
                &instruction,
                &invoke_accounts[..invoke_account_len],
            ) {
                Ok(()) => {
                    debug_log!("MitoVM: INVOKE completed successfully");

                    // CRITICAL FIX: Refresh account pointers after CPI
                    // When the Solana runtime executes invoke(), it may reallocate account data,
                    // rendering previously cached account info pointers stale.
                    // This call forces Pinocchio to recalculate data pointers for affected accounts.
                    let _ = ctx.refresh_account_pointers_after_cpi(
                        &account_indices[..accounts_count as usize],
                    );

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
            const RESERVED_FEE_VAULT_NAMESPACE: &[u8] = b"\xFFfive_vm_fee_vault_v1";
            debug_log!("MitoVM: INVOKE_SIGNED operation");

            let signer_payload = ctx.pop()?;
            if let ValueRef::ArrayRef(_) = signer_payload {
                let accounts_count = ctx.pop()?.as_u8().ok_or(VMErrorCode::TypeMismatch)?;
                let instruction_data_ref = ctx.pop()?;
                let program_id_ref = ctx.pop()?;
                return invoke_signed_grouped_from_array_ref(
                    ctx,
                    signer_payload,
                    accounts_count,
                    instruction_data_ref,
                    program_id_ref,
                );
            }

            let accounts_count = signer_payload.as_u8().ok_or(VMErrorCode::TypeMismatch)?;
            let instruction_data_ref = ctx.pop()?;
            let program_id_ref = ctx.pop()?;
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
            if (0..seeds_count as usize).any(|i| {
                let len = seed_lengths[i] as usize;
                &seed_storage[i][..len] == RESERVED_FEE_VAULT_NAMESPACE
            }) {
                return Err(VMErrorCode::InvalidSeedArray);
            }

            if seeds_count == 0 {
                return Err(VMErrorCode::InvalidSeedArray);
            }

            // Resolve program_id and instruction_data
            let program_id_bytes = ctx.extract_pubkey(&program_id_ref)?;
            let program_id = Pubkey::from(program_id_bytes);

            let mut instruction_data_buf: [u8; MAX_CPI_DATA_LEN] = [0u8; MAX_CPI_DATA_LEN];
            let instruction_data_len =
                materialize_instruction_data(ctx, instruction_data_ref, &mut instruction_data_buf)?;

            // Create AccountMetas and invoke_accounts array using stack arrays (no heap!)
            if accounts_count as usize > MAX_CPI_ACCOUNTS {
                return Err(VMErrorCode::InvalidOperation);
            }

            // Collect account indices first to avoid borrowing conflicts
            let mut account_indices: [usize; MAX_CPI_ACCOUNTS] = [0; MAX_CPI_ACCOUNTS];
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
            let mut account_metas: [AccountMeta; MAX_CPI_ACCOUNTS] =
                core::array::from_fn(|_| AccountMeta {
                    pubkey: accounts_ref[0].key(),
                    is_signer: false,
                    is_writable: false,
                });

            // Create invoke_accounts array after we have all indices
            let mut invoke_accounts: [&AccountInfo; MAX_CPI_ACCOUNTS + 1] =
                [&accounts_ref[0]; MAX_CPI_ACCOUNTS + 1];
            for i in 0..accounts_count as usize {
                let account_idx = account_indices[i];
                invoke_accounts[i] = &accounts_ref[account_idx];
            }
            let mut invoke_account_len = accounts_count as usize;
            if let Some(program_account) = accounts_ref
                .iter()
                .find(|account| account.key() == &program_id)
            {
                invoke_accounts[invoke_account_len] = program_account;
                invoke_account_len += 1;
            }

            for i in 0..accounts_count as usize {
                let account = invoke_accounts[i];
                account_metas[i] = AccountMeta {
                    pubkey: account.key(),
                    is_signer: account.is_signer(),
                    is_writable: account.is_writable(),
                };
            }

            // Create signer from seeds using stack arrays (no heap!)
            let mut seeds_refs: [Seed; MAX_SEEDS] =
                core::array::from_fn(|_| Seed::from(&[0u8][..])); // Default empty seed

            for i in 0..seeds_count as usize {
                let seed_slice = &seed_storage[i][..seed_lengths[i] as usize];
                seeds_refs[i] = Seed::from(seed_slice);
            }
            let mut derived_seed_lengths = [0usize; MAX_SIGNER_SEEDS];
            for i in 0..seeds_count as usize {
                derived_seed_lengths[i] = seed_lengths[i] as usize;
            }
            let mut derived_seed_slices: [&[u8]; MAX_SIGNER_SEEDS] = [&[]; MAX_SIGNER_SEEDS];
            for i in 0..seeds_count as usize {
                derived_seed_slices[i] = &seed_storage[i][..derived_seed_lengths[i]];
            }
            let signer_pubkey = derive_pda_from_seed_slices(
                &derived_seed_slices[..seeds_count as usize],
                &ctx.program_id,
            )?;
            if !mark_matching_signer_meta(
                &mut account_metas,
                &invoke_accounts,
                accounts_count as usize,
                &signer_pubkey,
            ) {
                return Err(VMErrorCode::ConstraintViolation);
            }
            let instruction = Instruction {
                program_id: &program_id,
                accounts: &account_metas[..accounts_count as usize],
                data: &instruction_data_buf[..instruction_data_len],
            };
            let signer = Signer::from(&seeds_refs[..seeds_count as usize]);

            // Execute the invoke_signed
            debug_log!("MitoVM: Executing invoke_signed");

            invoke_signed_with_bounds::<{ MAX_CPI_ACCOUNTS + 1 }>(
                &instruction,
                &invoke_accounts[..invoke_account_len],
                &[signer],
            )
            .map_err(|e| {
                debug_log!("MitoVM: INVOKE_SIGNED failed");
                map_invoke_error(e)
            })?;

            debug_log!("MitoVM: INVOKE_SIGNED completed successfully");

            // CRITICAL FIX: Refresh account pointers after CPI (same as INVOKE)
            let _ =
                ctx.refresh_account_pointers_after_cpi(&account_indices[..accounts_count as usize]);

            ctx.push(ValueRef::Bool(true))?;
        }
        _ => {
            debug_log!("MitoVM: Invoke opcode {} not implemented", opcode);
            return Err(VMErrorCode::InvalidInstruction);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::handle_invoke_ops;
    use crate::{context::ExecutionContext, stack::StackStorage};
    use five_protocol::{opcodes::INVOKE_SIGNED, ValueRef};
    use pinocchio::{account_info::AccountInfo, pubkey::Pubkey};
    use std::panic::{catch_unwind, AssertUnwindSafe};

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

    fn store_array(ctx: &mut ExecutionContext, offset: u8, values: &[ValueRef]) -> ValueRef {
        let mut cursor = offset as usize;
        ctx.temp_buffer_mut()[cursor] = values.len() as u8;
        ctx.temp_buffer_mut()[cursor + 1] = 1;
        cursor += 2;
        for value in values {
            let written = value
                .serialize_into(&mut ctx.temp_buffer_mut()[cursor..])
                .expect("serialize value ref");
            cursor += written;
        }
        ValueRef::ArrayRef(offset)
    }

    #[test]
    fn invoke_signed_seed_tempref_path_does_not_panic_with_u8_bounded_ranges() {
        let program_id = Pubkey::from([91u8; 32]);
        let account_key = Pubkey::from([92u8; 32]);

        let mut lamports = 1;
        let mut account_data = [];
        let account = create_account_info(
            &account_key,
            false,
            false,
            &mut lamports,
            &mut account_data,
            &program_id,
        );
        let accounts = [account];

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

        // Stack push order is reverse of pops in INVOKE_SIGNED.
        ctx.push(ValueRef::TempRef(250, 10)).unwrap(); // seed_value_ref (out of bounds)
        ctx.push(ValueRef::U64(10)).unwrap(); // seed_len
        ctx.push(ValueRef::U64(1)).unwrap(); // seeds_count
        ctx.push(ValueRef::U64(0)).unwrap(); // program_id_ref (current program)
        ctx.push(ValueRef::TempRef(0, 1)).unwrap(); // instruction_data_ref (valid)
        ctx.push(ValueRef::U64(0)).unwrap(); // accounts_count

        let panicked = catch_unwind(AssertUnwindSafe(|| {
            let _ = handle_invoke_ops(INVOKE_SIGNED, &mut ctx);
        }));

        // TempRef offsets/sizes are u8 and temp buffer is 512 bytes, so this should not panic.
        assert!(panicked.is_ok());
    }

    #[test]
    fn invoke_signed_instruction_tempref_path_does_not_panic_with_u8_bounded_ranges() {
        let program_id = Pubkey::from([93u8; 32]);
        let account_key = Pubkey::from([94u8; 32]);

        let mut lamports = 1;
        let mut account_data = [];
        let account = create_account_info(
            &account_key,
            false,
            false,
            &mut lamports,
            &mut account_data,
            &program_id,
        );
        let accounts = [account];

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
        ctx.temp_buffer_mut()[0] = 7;

        // Valid seed, then invalid instruction_data_ref range.
        ctx.push(ValueRef::TempRef(0, 1)).unwrap(); // seed_value_ref
        ctx.push(ValueRef::U64(1)).unwrap(); // seed_len
        ctx.push(ValueRef::U64(1)).unwrap(); // seeds_count
        ctx.push(ValueRef::U64(0)).unwrap(); // program_id_ref (current program)
        ctx.push(ValueRef::TempRef(250, 10)).unwrap(); // instruction_data_ref (out of bounds)
        ctx.push(ValueRef::U64(0)).unwrap(); // accounts_count

        let panicked = catch_unwind(AssertUnwindSafe(|| {
            let _ = handle_invoke_ops(INVOKE_SIGNED, &mut ctx);
        }));

        // TempRef offsets/sizes are u8 and temp buffer is 512 bytes, so this should not panic.
        assert!(panicked.is_ok());
    }

    #[test]
    fn invoke_signed_grouped_payload_rejects_non_array_group_cleanly() {
        let program_id = Pubkey::from([95u8; 32]);
        let mut lamports = 1;
        let mut account_data = [];
        let account = create_account_info(
            &program_id,
            false,
            false,
            &mut lamports,
            &mut account_data,
            &program_id,
        );
        let accounts = [account];

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

        let signer_groups_ref = store_array(&mut ctx, 0, &[ValueRef::Bool(true)]);
        ctx.push(ValueRef::U64(0)).unwrap(); // program_id_ref
        ctx.push(ValueRef::TempRef(10, 0)).unwrap(); // instruction_data_ref
        ctx.push(ValueRef::U64(0)).unwrap(); // accounts_count
        ctx.push(signer_groups_ref).unwrap(); // signer_groups_ref

        let result = handle_invoke_ops(INVOKE_SIGNED, &mut ctx);
        assert_eq!(result, Err(crate::error::VMErrorCode::TypeMismatch));
    }

    #[test]
    fn invoke_signed_grouped_payload_accepts_accountref_seed_values() {
        let program_id = Pubkey::from([96u8; 32]);
        let mut lamports = 1;
        let mut account_data = [];
        let account = create_account_info(
            &program_id,
            false,
            false,
            &mut lamports,
            &mut account_data,
            &program_id,
        );
        let accounts = [account];

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

        let inner_group = store_array(&mut ctx, 0, &[ValueRef::AccountRef(0, 0)]);
        let signer_groups_ref = store_array(&mut ctx, 16, &[inner_group]);
        ctx.push(ValueRef::U64(0)).unwrap(); // program_id_ref
        ctx.push(ValueRef::TempRef(32, 0)).unwrap(); // instruction_data_ref
        ctx.push(ValueRef::U64(0)).unwrap(); // accounts_count
        ctx.push(signer_groups_ref).unwrap(); // signer_groups_ref

        let result = handle_invoke_ops(INVOKE_SIGNED, &mut ctx);
        assert_ne!(result, Err(crate::error::VMErrorCode::TypeMismatch));
        assert_ne!(result, Err(crate::error::VMErrorCode::InvalidSeedArray));
    }

    #[test]
    fn invoke_signed_grouped_payload_fails_when_signer_meta_is_missing() {
        let program_id = Pubkey::from([97u8; 32]);
        let mut lamports = 1;
        let mut account_data = [];
        let account = create_account_info(
            &program_id,
            false,
            false,
            &mut lamports,
            &mut account_data,
            &program_id,
        );
        let accounts = [account];

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

        let inner_group = store_array(&mut ctx, 0, &[ValueRef::AccountRef(0, 0)]);
        let signer_groups_ref = store_array(&mut ctx, 16, &[inner_group]);
        ctx.push(ValueRef::U64(0)).unwrap(); // program_id_ref
        ctx.push(ValueRef::TempRef(32, 0)).unwrap(); // instruction_data_ref
        ctx.push(ValueRef::U64(0)).unwrap(); // accounts_count
        ctx.push(signer_groups_ref).unwrap(); // signer_groups_ref

        let result = handle_invoke_ops(INVOKE_SIGNED, &mut ctx);
        assert_eq!(result, Err(crate::error::VMErrorCode::ConstraintViolation));
    }
}
