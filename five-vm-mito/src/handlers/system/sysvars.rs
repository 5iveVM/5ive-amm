//! System sysvar operations handler for MitoVM
//!
//! This module handles blockchain sysvar access operations including:
//! - GET_CLOCK: Access Clock sysvar for timestamp, slot, epoch info
//! - GET_RENT: Access Rent sysvar for rent exemption calculations  
//!
//! All operations provide real blockchain data for time-dependent and
//! economic contract logic.

use crate::{
    context::ExecutionManager,
    debug_log,
    error::{CompactResult, VMErrorCode},
};
use five_protocol::{opcodes::*, ValueRef};
use pinocchio::sysvars::{clock::Clock, rent::Rent, Sysvar};

/// Serialize Clock sysvar data to buffer in little-endian format.
/// Layout: [slot:8][epoch_start_timestamp:8][epoch:8][leader_schedule_epoch:8][unix_timestamp:8]
/// Total: 40 bytes
#[inline(always)]
pub fn serialize_clock_to_buffer(clock: &Clock, buffer: &mut [u8]) {
    buffer[0..8].copy_from_slice(&clock.slot.to_le_bytes());
    buffer[8..16].copy_from_slice(&clock.epoch_start_timestamp.to_le_bytes());
    buffer[16..24].copy_from_slice(&clock.epoch.to_le_bytes());
    buffer[24..32].copy_from_slice(&clock.leader_schedule_epoch.to_le_bytes());
    buffer[32..40].copy_from_slice(&clock.unix_timestamp.to_le_bytes());
}

/// Handle system sysvar operations for blockchain data access
#[inline(never)]
pub fn handle_sysvar_ops(opcode: u8, ctx: &mut ExecutionManager) -> CompactResult<()> {
    match opcode {
        GET_CLOCK => {
            debug_log!(
                "MitoVM: GET_CLOCK operation - accessing real Solana Clock sysvar via Pinocchio"
            );

            // Access real Solana Clock sysvar using Pinocchio
            // Clock::get() provides direct access to the Clock sysvar without requiring accounts
            let clock = Clock::get().map_err(|_| {
                debug_log!("MitoVM: GET_CLOCK failed to access Clock sysvar");
                VMErrorCode::InvalidOperation
            })?;

            // With V3 presized headers, temp buffer should be preallocated to exactly the size we need
            // Clock structure requires 40 bytes: 5 fields × 8 bytes each
            // The VM should have allocated temp_buffer_size >= 40 based on ResourceRequirements
            let temp_buffer = ctx.temp_buffer_mut();

            if temp_buffer.len() < 40 {
                debug_log!(
                    "MitoVM: GET_CLOCK temp buffer too small: {} bytes, need 40 bytes",
                    temp_buffer.len() as u32
                );
                debug_log!("MitoVM: Use V3 header with ResourceRequirements.temp_buffer_size >= 40 for Clock operations");
                return Err(VMErrorCode::MemoryViolation);
            }

            // Write complete Clock structure to preallocated temp buffer
            // Layout: [slot:8][epoch_start_timestamp:8][epoch:8][leader_schedule_epoch:8][unix_timestamp:8]
            serialize_clock_to_buffer(&clock, temp_buffer);

            // Return TupleRef to complete Clock structure containing all fields
            // This provides access to: slot, epoch_start_timestamp, epoch, leader_schedule_epoch, unix_timestamp
            ctx.push(ValueRef::TupleRef(0, 40))?;

            debug_log!("MitoVM: GET_CLOCK pushed complete Clock sysvar data - slot={}, epoch={}, timestamp={}", 
                      clock.slot, clock.epoch, clock.unix_timestamp);
        }
        GET_RENT => {
            debug_log!("MitoVM: GET_RENT operation - accessing real Solana Rent sysvar");
            
            // Pop space from stack (compiler expects GET_RENT to consume space)
            let space = ctx.pop()?.as_u64().ok_or(VMErrorCode::TypeMismatch)?;

            // Access real Solana Rent sysvar using Pinocchio
            let rent = Rent::get().map_err(|_| {
                debug_log!("MitoVM: GET_RENT failed to access Rent sysvar");
                VMErrorCode::InvalidOperation
            })?;

            // Calculate minimum balance for rent exemption
            let min_balance = rent.minimum_balance(space as usize);
            ctx.push(ValueRef::U64(min_balance))?;

            debug_log!(
                "MitoVM: GET_RENT calculated min_balance={} for space={}",
                min_balance,
                space
            );
        }
        _ => {
            debug_log!("MitoVM: Sysvar opcode {} not implemented", opcode);
            return Err(VMErrorCode::InvalidInstruction);
        }
    }
    Ok(())
}

/// Handle sol_get_clock_sysvar syscall
#[inline(never)]
pub fn handle_syscall_get_clock_sysvar(ctx: &mut ExecutionManager) -> CompactResult<()> {
    debug_log!("MitoVM: SYSCALL_GET_CLOCK_SYSVAR");

    let clock = Clock::get().map_err(|_| {
        debug_log!("MitoVM: Failed to access Clock sysvar");
        VMErrorCode::InvalidOperation
    })?;

    // Use temp buffer to store clock data
    let temp_buffer = ctx.temp_buffer_mut();
    if temp_buffer.len() < 40 {
        return Err(VMErrorCode::MemoryViolation);
    }

    // Write Clock structure: slot, epoch_start_timestamp, epoch, leader_schedule_epoch, unix_timestamp
    serialize_clock_to_buffer(&clock, temp_buffer);

    // Push reference to complete clock structure
    ctx.push(ValueRef::TupleRef(0, 40))?;
    debug_log!("MitoVM: SYSCALL_GET_CLOCK_SYSVAR success");
    Ok(())
}

/// Handle sol_get_rent_sysvar syscall
#[inline(never)]
pub fn handle_syscall_get_rent_sysvar(ctx: &mut ExecutionManager) -> CompactResult<()> {
    debug_log!("MitoVM: SYSCALL_GET_RENT_SYSVAR");

    let rent = Rent::get().map_err(|_| {
        debug_log!("MitoVM: Failed to access Rent sysvar");
        VMErrorCode::InvalidOperation
    })?;

    // Push rent lamports per byte per year
    #[allow(deprecated)]
    let lamports_per_byte_year = rent.lamports_per_byte_year;
    ctx.push(ValueRef::U64(lamports_per_byte_year))?;
    debug_log!("MitoVM: SYSCALL_GET_RENT_SYSVAR success");
    Ok(())
}

// Placeholders for other sysvar syscalls
macro_rules! sysvar_syscall_placeholder {
    ($name:ident, $log_msg:expr) => {
        pub fn $name(ctx: &mut ExecutionManager) -> CompactResult<()> {
            debug_log!("MitoVM: {} - placeholder implementation", $log_msg);
            ctx.push(ValueRef::result_ok(0, 0))?;
            Ok(())
        }
    };
}

sysvar_syscall_placeholder!(handle_syscall_get_epoch_schedule_sysvar, "SYSCALL_GET_EPOCH_SCHEDULE_SYSVAR");
sysvar_syscall_placeholder!(handle_syscall_get_epoch_rewards_sysvar, "SYSCALL_GET_EPOCH_REWARDS_SYSVAR");
sysvar_syscall_placeholder!(handle_syscall_get_epoch_stake, "SYSCALL_GET_EPOCH_STAKE");
sysvar_syscall_placeholder!(handle_syscall_get_fees_sysvar, "SYSCALL_GET_FEES_SYSVAR");
sysvar_syscall_placeholder!(handle_syscall_get_last_restart_slot, "SYSCALL_GET_LAST_RESTART_SLOT");
sysvar_syscall_placeholder!(handle_syscall_get_sysvar, "SYSCALL_GET_SYSVAR");
