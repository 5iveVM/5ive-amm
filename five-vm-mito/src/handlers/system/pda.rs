//! Program Derived Address (PDA) operations handler for MitoVM
//!
//! This module handles PDA operations including DERIVE_PDA and FIND_PDA.
//! It manages stack-based seed handling and Solana PDA derivation with
//! zero-heap allocation for optimal performance.

use crate::{
    context::ExecutionManager,
    debug_log,
    error::{CompactResult, VMErrorCode},
};
use five_protocol::{opcodes::*, ValueRef};
use pinocchio::pubkey::Pubkey;
#[cfg(target_os = "solana")]
use pinocchio::pubkey::{create_program_address, find_program_address};

/// Process a single seed value and store it in the seed array.
/// Returns the length of the seed data written.
#[inline(always)]
pub fn process_seed_value(
    seed_value: ValueRef,
    seeds: &mut [[u8; 32]],
    seed_idx: usize,
    ctx: &ExecutionManager,
) -> CompactResult<usize> {
    match seed_value {
        ValueRef::U32(val) => {
            let bytes = val.to_le_bytes();
            seeds[seed_idx][..4].copy_from_slice(&bytes);
            Ok(4)
        }
        ValueRef::U16(val) => {
            let bytes = val.to_le_bytes();
            seeds[seed_idx][..2].copy_from_slice(&bytes);
            Ok(2)
        }
        ValueRef::U64(val) => {
            let bytes = val.to_le_bytes();
            seeds[seed_idx][..8].copy_from_slice(&bytes);
            Ok(8)
        }
        ValueRef::U8(val) => {
            seeds[seed_idx][0] = val;
            Ok(1)
        }
        ValueRef::I32(val) => {
            let bytes = val.to_le_bytes();
            seeds[seed_idx][..4].copy_from_slice(&bytes);
            Ok(4)
        }
        ValueRef::I16(val) => {
            let bytes = val.to_le_bytes();
            seeds[seed_idx][..2].copy_from_slice(&bytes);
            Ok(2)
        }
        ValueRef::I8(val) => {
            seeds[seed_idx][0] = val as u8;
            Ok(1)
        }
        ValueRef::PubkeyRef(_) => {
            let bytes = ctx.extract_pubkey(&seed_value)?;
            seeds[seed_idx][..32].copy_from_slice(&bytes);
            Ok(32)
        }
        ValueRef::AccountRef(account_idx, account_offset) => {
            if account_offset != 0 {
                return Err(VMErrorCode::TypeMismatch);
            }
            let account = ctx.get_account(account_idx)?;
            seeds[seed_idx][..32].copy_from_slice(account.key().as_ref());
            Ok(32)
        }
        ValueRef::TempRef(offset, len) => {
            let start = offset as usize;
            let end = start + len as usize;
            if end > ctx.temp_buffer().len() {
                return Err(VMErrorCode::MemoryViolation);
            }
            let copy_len = len.min(32);
            seeds[seed_idx][..copy_len as usize]
                .copy_from_slice(&ctx.temp_buffer()[start..start + copy_len as usize]);
            Ok(copy_len as usize)
        }
        ValueRef::StringRef(offset) => {
            let start = offset as usize;
            if start + 2 > ctx.temp_buffer().len() {
                return Err(VMErrorCode::MemoryViolation);
            }
            let len = ctx.temp_buffer()[start];
            let data_start = start + 2;
            let data_end = data_start + len as usize;

            if data_end > ctx.temp_buffer().len() {
                return Err(VMErrorCode::MemoryViolation);
            }

            let copy_len = (len as usize).min(32);
            seeds[seed_idx][..copy_len]
                .copy_from_slice(&ctx.temp_buffer()[data_start..data_start + copy_len]);
            Ok(copy_len)
        }
        ValueRef::ArrayRef(id) => {
            let start = id as usize;
            if start + 2 > ctx.temp_buffer().len() {
                return Err(VMErrorCode::MemoryViolation);
            }
            let len = ctx.temp_buffer()[start];
            let data_start = start + 2;
            let data_end = data_start + len as usize;

            if data_end > ctx.temp_buffer().len() {
                return Err(VMErrorCode::MemoryViolation);
            }

            let copy_len = (len as usize).min(32);
            seeds[seed_idx][..copy_len]
                .copy_from_slice(&ctx.temp_buffer()[data_start..data_start + copy_len]);
            Ok(copy_len)
        }
        _ => Err(VMErrorCode::TypeMismatch),
    }
}

/// Pop seeds from stack and process them into the provided buffers.
#[inline(always)]
pub fn pop_and_process_seeds(
    ctx: &mut ExecutionManager,
    seeds_count: u8,
    seeds: &mut [[u8; 32]; 8],
    seed_lens: &mut [usize; 8],
) -> CompactResult<()> {
    const MAX_SEEDS: usize = 8;
    if seeds_count as usize > MAX_SEEDS {
        return Err(VMErrorCode::InvalidOperation);
    }

    // Pop seeds from stack and store directly in stack arrays
    for i in 0..seeds_count {
        let seed_idx = (seeds_count - 1 - i) as usize; // Reverse order since we pop
        let seed_value = ctx.pop()?;
        debug_log!("MitoVM: PDA seed index: {}", seed_idx as u32);
        seed_lens[seed_idx] = process_seed_value(seed_value, seeds, seed_idx, ctx)?;
    }
    Ok(())
}

/// Handle sol_create_program_address syscall.
#[inline(always)]
pub fn handle_syscall_create_program_address(ctx: &mut ExecutionManager) -> CompactResult<()> {
    debug_log!("MitoVM: SYSCALL_CREATE_PROGRAM_ADDRESS");

    let program_id_ref = ctx.pop()?;
    let seeds_ref = ctx.pop()?;

    let program_id_bytes = ctx.extract_pubkey(&program_id_ref)?;
    let program_pubkey = Pubkey::from(program_id_bytes);

    // Process seeds. Currently supports a single seed (or array treated as one).
    const MAX_SEEDS: usize = 8;
    let mut seeds: [[u8; 32]; MAX_SEEDS] = [[0; 32]; MAX_SEEDS];

    let len = process_seed_value(seeds_ref, &mut seeds, 0, ctx)?;
    let seed_refs = [&seeds[0][..len]];

    debug_log!("MitoVM: SYSCALL_CREATE_PROGRAM_ADDRESS calling create_program_address");

    #[cfg(target_os = "solana")]
    let pda_result: Result<[u8; 32], pinocchio::program_error::ProgramError> =
        create_program_address(&seed_refs, &program_pubkey)
            .map_err(|_| pinocchio::program_error::ProgramError::Custom(9101));

    #[cfg(not(target_os = "solana"))]
    let pda_result: Result<[u8; 32], pinocchio::program_error::ProgramError> =
        crate::utils::derive_pda_offchain(&seed_refs, &program_pubkey).map_err(|e| e.into());

    match pda_result {
        Ok(pda_pubkey) => {
            debug_log!("MitoVM: SYSCALL_CREATE_PROGRAM_ADDRESS success");
            push_pubkey_result(ctx, pda_pubkey)
        }
        Err(_e) => {
            debug_log!("MitoVM: SYSCALL_CREATE_PROGRAM_ADDRESS failed");
            Err(VMErrorCode::InvokeError)
        }
    }
}

#[inline(always)]
fn push_pubkey_result(ctx: &mut ExecutionManager, pda_pubkey: [u8; 32]) -> CompactResult<()> {
    let pda_offset = ctx.alloc_temp(32)?;
    ctx.temp_buffer_mut()[pda_offset as usize..(pda_offset + 32) as usize]
        .copy_from_slice(&pda_pubkey);
    ctx.push(ValueRef::TempRef(pda_offset, 32))
}

/// Handle sol_try_find_program_address syscall.
#[inline(always)]
pub fn handle_syscall_try_find_program_address(ctx: &mut ExecutionManager) -> CompactResult<()> {
    debug_log!("MitoVM: SYSCALL_TRY_FIND_PROGRAM_ADDRESS");

    let program_id_ref = ctx.pop()?;
    let seeds_ref = ctx.pop()?;

    let program_id_bytes = ctx.extract_pubkey(&program_id_ref)?;
    let program_pubkey = Pubkey::from(program_id_bytes);

    // Process seeds. Currently supports a single seed (or array treated as one).
    const MAX_SEEDS: usize = 8;
    let mut seeds: [[u8; 32]; MAX_SEEDS] = [[0; 32]; MAX_SEEDS];

    let len = process_seed_value(seeds_ref, &mut seeds, 0, ctx)?;
    let seed_refs = [&seeds[0][..len]];

    debug_log!("MitoVM: SYSCALL_TRY_FIND_PROGRAM_ADDRESS calling find_program_address");

    #[cfg(target_os = "solana")]
    let (pda_pubkey, bump_seed) = find_program_address(&seed_refs, &program_pubkey);

    #[cfg(not(target_os = "solana"))]
    let (pda_pubkey, bump_seed) =
        crate::utils::find_program_address_offchain(&seed_refs, &program_pubkey)?;

    debug_log!("MitoVM: SYSCALL_TRY_FIND_PROGRAM_ADDRESS success");
    push_pda_result(ctx, pda_pubkey, bump_seed)
}

/// Execute a closure with parsed PDA seeds and program ID.
#[inline(always)]
pub fn with_pda_seeds<F>(ctx: &mut ExecutionManager, f: F) -> CompactResult<()>
where
    F: FnOnce(&mut ExecutionManager, Pubkey, &[&[u8]]) -> CompactResult<()>,
{
    // Pop program_id from stack
    let program_id_ref = ctx.pop()?;

    // Extract pubkey directly
    let program_id_bytes = ctx.extract_pubkey(&program_id_ref)?;
    let program_pubkey = Pubkey::from(program_id_bytes);

    // Pop seeds_count
    let seeds_count = ctx.pop()?.as_u8().ok_or(VMErrorCode::TypeMismatch)?;
    debug_log!("MitoVM: PDA seeds_count: {}", seeds_count);

    const MAX_SEEDS: usize = 8;
    const MAX_EFFECTIVE_SEEDS: usize = MAX_SEEDS + 1;
    // Stack-allocated seed storage (no heap!)
    let mut seeds: [[u8; 32]; MAX_SEEDS] = [[0; 32]; MAX_SEEDS];
    let mut seed_lens: [usize; MAX_SEEDS] = [0; MAX_SEEDS];

    pop_and_process_seeds(ctx, seeds_count, &mut seeds, &mut seed_lens)?;

    // Create stack-based seed reference array (no heap!)
    let mut seed_refs: [&[u8]; MAX_SEEDS] = [&[]; MAX_SEEDS];
    for i in 0..seeds_count as usize {
        seed_refs[i] = &seeds[i][..seed_lens[i]];
    }

    // Script-scoped PDA model:
    // when deriving under the VM program id, prepend active_script_key so helper
    // derivations (e.g. auto bump via FIND_PDA) match INIT_PDA_ACCOUNT/INVOKE_SIGNED.
    if program_pubkey.as_ref() == &ctx.program_id {
        let active_script_key = ctx
            .active_script_key()
            .ok_or(VMErrorCode::ScriptNotAuthorized)?;
        let mut effective_seed_refs: [&[u8]; MAX_EFFECTIVE_SEEDS] =
            [&[]; MAX_EFFECTIVE_SEEDS];
        effective_seed_refs[0] = active_script_key.as_ref();
        for i in 0..seeds_count as usize {
            effective_seed_refs[i + 1] = seed_refs[i];
        }
        f(ctx, program_pubkey, &effective_seed_refs[..seeds_count as usize + 1])
    } else {
        f(ctx, program_pubkey, &seed_refs[..seeds_count as usize])
    }
}

/// Helper to push (PDA, bump) tuple result efficiently
#[inline(always)]
fn push_pda_result(
    ctx: &mut ExecutionManager,
    pda_pubkey: [u8; 32],
    bump: u8,
) -> CompactResult<()> {
    // Store PDA in temp buffer
    let pda_offset = ctx.alloc_temp(32)?;
    ctx.temp_buffer_mut()[pda_offset as usize..(pda_offset + 32) as usize]
        .copy_from_slice(&pda_pubkey);

    // Create tuple directly in temp buffer without stack ops
    let pda_ref = ValueRef::TempRef(pda_offset, 32);
    let bump_ref = ValueRef::U8(bump);

    // Calculate size
    let size = pda_ref.serialized_size() + bump_ref.serialized_size();
    let tuple_size = u8::try_from(size).map_err(|_| VMErrorCode::OutOfMemory)?;
    let tuple_offset = ctx.alloc_temp(tuple_size)?;

    // Serialize into temp buffer
    // Access temp_buffer via separate borrow to satisfy checker
    {
        let buffer = ctx.temp_buffer_mut();
        let mut current = tuple_offset as usize;
        let written = pda_ref
            .serialize_into(&mut buffer[current..])
            .map_err(|_| VMErrorCode::MemoryViolation)?;
        current += written;

        bump_ref
            .serialize_into(&mut buffer[current..])
            .map_err(|_| VMErrorCode::MemoryViolation)?;
    }

    // Push TupleRef
    ctx.push(ValueRef::TupleRef(tuple_offset, tuple_size))
}

/// Handle PDA operations for program derived addresses
#[inline(always)]
pub fn handle_pda_ops(opcode: u8, ctx: &mut ExecutionManager) -> CompactResult<()> {
    match opcode {
        DERIVE_PDA => {
            debug_log!("MitoVM: DERIVE_PDA operation");
            with_pda_seeds(ctx, |ctx, program_pubkey, seeds| {
                debug_log!("MitoVM: DERIVE_PDA calling create_program_address");

                #[cfg(target_os = "solana")]
                let pda_result: Result<
                    [u8; 32],
                    pinocchio::program_error::ProgramError,
                > = create_program_address(seeds, &program_pubkey)
                    .map_err(|_| pinocchio::program_error::ProgramError::Custom(9101));

                #[cfg(not(target_os = "solana"))]
                let pda_result: Result<
                    [u8; 32],
                    pinocchio::program_error::ProgramError,
                > = crate::utils::derive_pda_offchain(seeds, &program_pubkey).map_err(|e| e.into());

                match pda_result {
                    Ok(pda_pubkey) => {
                        debug_log!("MitoVM: DERIVE_PDA success: [pubkey]");
                        push_pda_result(ctx, pda_pubkey, 0)
                    }
                    Err(_e) => {
                        debug_log!("MitoVM: DERIVE_PDA failed");
                        Err(VMErrorCode::InvokeError)
                    }
                }
            })?;
        }
        FIND_PDA => {
            debug_log!("MitoVM: FIND_PDA operation");
            with_pda_seeds(ctx, |ctx, program_pubkey, seeds| {
                debug_log!("MitoVM: FIND_PDA calling find_program_address");

                #[cfg(target_os = "solana")]
                let (pda_pubkey, bump_seed) = find_program_address(seeds, &program_pubkey);

                #[cfg(not(target_os = "solana"))]
                let (pda_pubkey, bump_seed) =
                    crate::utils::find_program_address_offchain(seeds, &program_pubkey)?;

                debug_log!("MitoVM: FIND_PDA success: [pubkey]");
                debug_log!("MitoVM: FIND_PDA bump: {}", bump_seed as u32);

                push_pda_result(ctx, pda_pubkey, bump_seed)
            })?;
        }
        _ => {
            debug_log!("MitoVM: PDA opcode {} not implemented", opcode);
            return Err(VMErrorCode::InvalidInstruction);
        }
    }
    Ok(())
}
