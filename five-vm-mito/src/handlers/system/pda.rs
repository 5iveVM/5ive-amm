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
use pinocchio::pubkey::{create_program_address, find_program_address, Pubkey};

/// Process a single seed value and store it in the seed array.
/// Returns the length of the seed data written.
///
/// This helper eliminates duplication across DERIVE_PDA, FIND_PDA, and constraint validation.
#[inline(always)]
pub fn process_seed_value(
    seed_value: ValueRef,
    seeds: &mut [[u8; 32]],
    seed_idx: usize,
    ctx: &ExecutionManager,
) -> CompactResult<usize> {
    match seed_value {
        ValueRef::U64(val) => {
            let bytes = val.to_le_bytes();
            seeds[seed_idx][..8].copy_from_slice(&bytes);
            Ok(8)
        }
        ValueRef::U8(val) => {
            seeds[seed_idx][0] = val;
            Ok(1)
        }
        ValueRef::TempRef(offset, len) => {
            // Get string or byte array from temp buffer
            let start = offset as usize;
            let end = start + len as usize;
            if end > ctx.temp_buffer().len() {
                return Err(VMErrorCode::MemoryViolation);
            }
            let copy_len = len.min(32); // Clamp to seed max size
            seeds[seed_idx][..copy_len as usize]
                .copy_from_slice(&ctx.temp_buffer()[start..start + copy_len as usize]);
            Ok(copy_len as usize)
        }
        _ => Err(VMErrorCode::TypeMismatch),
    }
}

/// Handle PDA operations for program derived addresses
#[inline(never)]
pub fn handle_pda_ops(opcode: u8, ctx: &mut ExecutionManager) -> CompactResult<()> {
    match opcode {
        DERIVE_PDA => {
            debug_log!("MitoVM: DERIVE_PDA operation");

            // Pop program_id from stack
            let program_id_ref = ctx.pop()?;

            // Extract pubkey directly
            let program_id_bytes = ctx.extract_pubkey(&program_id_ref)?;
            let program_pubkey = Pubkey::from(program_id_bytes);

            // Pop seeds_count
            let seeds_count = ctx.pop()?.as_u8().ok_or(VMErrorCode::TypeMismatch)?;
            debug_log!("MitoVM: DERIVE_PDA seeds_count: {}", seeds_count);

            // Validate seeds count (stack-based limit)
            const MAX_SEEDS: usize = 8;
            if seeds_count as usize > MAX_SEEDS {
                return Err(VMErrorCode::InvalidOperation);
            }

            // Stack-allocated seed storage (no heap!)
            let mut seeds: [[u8; 32]; MAX_SEEDS] = [[0; 32]; MAX_SEEDS];
            let mut seed_lens: [usize; MAX_SEEDS] = [0; MAX_SEEDS];

            // Pop seeds from stack and store directly in stack arrays
            for i in 0..seeds_count {
                let seed_idx = (seeds_count - 1 - i) as usize; // Reverse order since we pop
                let seed_value = ctx.pop()?;
                debug_log!("MitoVM: DERIVE_PDA seed index: {}", seed_idx as u32);
                debug_log!("Seed type: {}", seed_value.type_id() as u32);

                // Convert seed value to bytes using helper function
                seed_lens[seed_idx] = process_seed_value(seed_value, &mut seeds, seed_idx, ctx)?;
            }

            // Create stack-based seed reference array (no heap!)
            let mut seed_refs: [&[u8]; MAX_SEEDS] = [&[]; MAX_SEEDS];
            for i in 0..seeds_count as usize {
                seed_refs[i] = &seeds[i][..seed_lens[i]];
            }

            debug_log!("MitoVM: DERIVE_PDA calling create_program_address");
            debug_log!("MitoVM: DERIVE_PDA program_id: [pubkey]");
            for _i in 0..seeds_count as usize {
                debug_log!("MitoVM: DERIVE_PDA seed: {}", _i as u32);
                debug_log!("Seed bytes: {}", seed_lens[_i] as u32);
            }

            // Use Pinocchio's create_program_address with stack-only seed slice
            match create_program_address(&seed_refs[..seeds_count as usize], &program_pubkey) {
                Ok(pda_pubkey) => {
                    debug_log!("MitoVM: DERIVE_PDA success: [pubkey]");

                    // Store PDA in temp buffer and push reference
                    let temp_offset = ctx.alloc_temp(32)?;
                    ctx.temp_buffer_mut()[temp_offset as usize..(temp_offset + 32) as usize]
                        .copy_from_slice(&pda_pubkey);
                    ctx.push(ValueRef::TempRef(temp_offset, 32))?;

                    // Note: create_program_address doesn't return bump, so push 0
                    ctx.push(ValueRef::U8(0))?;

                    // Create tuple (pubkey, u8) manually
                    const NUM_ELEMENTS: usize = 2;
                    let mut elements = [ValueRef::U64(0); NUM_ELEMENTS];
                    let mut total_size = 0;

                    // Pop elements in reverse order (stack is LIFO)
                    for i in 0..NUM_ELEMENTS {
                        let element = ctx.pop()?;
                        elements[NUM_ELEMENTS - 1 - i] = element;
                        total_size += element.serialized_size();
                    }

                    // Allocate temp buffer space and serialize elements
                    let tuple_offset = ctx.alloc_temp(total_size as u8)?;
                    let mut current_offset = tuple_offset as usize;

                    for i in 0..NUM_ELEMENTS {
                        let element = &elements[i];
                        let written_size = element
                            .serialize_into(&mut ctx.temp_buffer_mut()[current_offset..])
                            .map_err(|_| VMErrorCode::MemoryViolation)?;
                        current_offset += written_size;
                    }

                    // Push tuple reference to stack
                    ctx.push(ValueRef::TupleRef(tuple_offset, total_size as u8))?;
                }
                Err(_e) => {
                    debug_log!("MitoVM: DERIVE_PDA failed");
                    return Err(VMErrorCode::InvokeError);
                }
            }
        }
        FIND_PDA => {
            debug_log!("MitoVM: FIND_PDA operation");

            // Pop program_id from stack
            let program_id_ref = ctx.pop()?;

            // Extract pubkey directly
            let program_id_bytes = ctx.extract_pubkey(&program_id_ref)?;
            let program_pubkey = Pubkey::from(program_id_bytes);

            // Pop seeds_count
            let seeds_count = ctx.pop()?.as_u8().ok_or(VMErrorCode::TypeMismatch)?;
            debug_log!("MitoVM: FIND_PDA seeds_count: {}", seeds_count);

            // Validate seeds count (stack-based limit)
            const MAX_SEEDS: usize = 8;
            if seeds_count as usize > MAX_SEEDS {
                return Err(VMErrorCode::InvalidOperation);
            }

            // Stack-allocated seed storage (no heap!)
            let mut seeds: [[u8; 32]; MAX_SEEDS] = [[0; 32]; MAX_SEEDS];
            let mut seed_lens: [usize; MAX_SEEDS] = [0; MAX_SEEDS];

            // Pop seeds from stack and store directly in stack arrays
            for i in 0..seeds_count {
                let seed_idx = (seeds_count - 1 - i) as usize; // Reverse order since we pop
                let seed_value = ctx.pop()?;
                debug_log!("MitoVM: FIND_PDA seed[{}]: ValueRef", seed_idx as u32);

                // Convert seed value to bytes using helper function
                seed_lens[seed_idx] = process_seed_value(seed_value, &mut seeds, seed_idx, ctx)?;
            }

            // Create stack-based seed reference array (no heap!)
            let mut seed_refs: [&[u8]; MAX_SEEDS] = [&[]; MAX_SEEDS];
            for i in 0..seeds_count as usize {
                seed_refs[i] = &seeds[i][..seed_lens[i]];
            }

            debug_log!("MitoVM: FIND_PDA calling find_program_address");
            debug_log!("MitoVM: FIND_PDA program_id: [pubkey]");
            for _i in 0..seeds_count as usize {
                debug_log!("MitoVM: FIND_PDA seed: {}", _i as u32);
                debug_log!("Seed bytes: {}", seed_lens[_i] as u32);
            }

            // Use Pinocchio's find_program_address with stack-only seed slice (finds valid bump automatically)
            let (pda_pubkey, bump_seed) =
                find_program_address(&seed_refs[..seeds_count as usize], &program_pubkey);

            debug_log!("MitoVM: FIND_PDA success: [pubkey]");
            debug_log!("MitoVM: FIND_PDA bump: {}", bump_seed as u32);

            // Store PDA in temp buffer and push reference
            let temp_offset = ctx.temp_offset();
            if temp_offset + 32 > ctx.temp_buffer().len() {
                return Err(VMErrorCode::DataBufferOverflow);
            }
            ctx.temp_buffer_mut()[temp_offset..temp_offset + 32].copy_from_slice(&pda_pubkey);
            ctx.set_temp_offset(temp_offset + 32);
            ctx.push(ValueRef::TempRef(temp_offset as u8, 32))?;

            // Push the actual bump seed
            ctx.push(ValueRef::U8(bump_seed))?;

            // Create tuple (pubkey, u8) manually since we can't use CREATE_TUPLE opcode directly
            // (CREATE_TUPLE expects num_elements from bytecode stream)
            const NUM_ELEMENTS: usize = 2;
            let mut elements = [ValueRef::U64(0); NUM_ELEMENTS];
            let mut total_size = 0;

            // Pop elements in reverse order (stack is LIFO)
            for i in 0..NUM_ELEMENTS {
                let element = ctx.pop()?;
                elements[NUM_ELEMENTS - 1 - i] = element;
                total_size += element.serialized_size();
            }

            // Allocate temp buffer space and serialize elements
            let tuple_offset = ctx.alloc_temp(total_size as u8)?;
            let mut current_offset = tuple_offset as usize;

            for i in 0..NUM_ELEMENTS {
                let element = &elements[i];
                let written_size = element
                    .serialize_into(&mut ctx.temp_buffer_mut()[current_offset..])
                    .map_err(|_| VMErrorCode::MemoryViolation)?;
                current_offset += written_size;
            }

            // Push tuple reference to stack
            ctx.push(ValueRef::TupleRef(tuple_offset, total_size as u8))?;
        }
        _ => {
            debug_log!("MitoVM: PDA opcode {} not implemented", opcode);
            return Err(VMErrorCode::InvalidInstruction);
        }
    }
    Ok(())
}
