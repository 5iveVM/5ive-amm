//! System sysvar operations handler for MitoVM.

use crate::{
    context::ExecutionManager,
    debug_log,
    error::{CompactResult, VMErrorCode},
};
use five_protocol::{opcodes::*, ValueRef};
use pinocchio::sysvars::{clock::Clock, rent::Rent, Sysvar};

/// Serialize Clock sysvar data to buffer in little-endian format.
/// Layout: [slot:8][epoch_start_timestamp:8][epoch:8][leader_schedule_epoch:8][unix_timestamp:8]
#[inline(always)]
pub fn serialize_clock_to_buffer(clock: &Clock, buffer: &mut [u8]) {
    buffer[0..8].copy_from_slice(&clock.slot.to_le_bytes());
    buffer[8..16].copy_from_slice(&clock.epoch_start_timestamp.to_le_bytes());
    buffer[16..24].copy_from_slice(&clock.epoch.to_le_bytes());
    buffer[24..32].copy_from_slice(&clock.leader_schedule_epoch.to_le_bytes());
    buffer[32..40].copy_from_slice(&clock.unix_timestamp.to_le_bytes());
}

/// Handle system sysvar operations.
#[inline(never)]
pub fn handle_sysvar_ops(opcode: u8, ctx: &mut ExecutionManager) -> CompactResult<()> {
    match opcode {
        GET_CLOCK => {
            debug_log!("MitoVM: GET_CLOCK operation");

            let clock = Clock::get().map_err(|_| VMErrorCode::InvalidOperation)?;

            // Clock structure requires 40 bytes.
            let temp_buffer = ctx.temp_buffer_mut();
            if temp_buffer.len() < 40 {
                return Err(VMErrorCode::MemoryViolation);
            }

            serialize_clock_to_buffer(&clock, temp_buffer);
            ctx.push(ValueRef::TupleRef(0, 40))?;
        }
        GET_RENT => {
            debug_log!("MitoVM: GET_RENT operation");
            
            let space = ctx.pop()?.as_u64().ok_or(VMErrorCode::TypeMismatch)?;
            let rent = Rent::get().map_err(|_| VMErrorCode::InvalidOperation)?;

            let min_balance = rent.minimum_balance(space as usize);
            ctx.push(ValueRef::U64(min_balance))?;
        }
        _ => {
            return Err(VMErrorCode::InvalidInstruction);
        }
    }
    Ok(())
}

/// Handle sol_get_clock_sysvar syscall.
#[inline(never)]
pub fn handle_syscall_get_clock_sysvar(ctx: &mut ExecutionManager) -> CompactResult<()> {
    debug_log!("MitoVM: SYSCALL_GET_CLOCK_SYSVAR");

    let clock = Clock::get().map_err(|_| VMErrorCode::InvalidOperation)?;

    let temp_buffer = ctx.temp_buffer_mut();
    if temp_buffer.len() < 40 {
        return Err(VMErrorCode::MemoryViolation);
    }

    serialize_clock_to_buffer(&clock, temp_buffer);
    ctx.push(ValueRef::TupleRef(0, 40))?;
    Ok(())
}

/// Handle sol_get_rent_sysvar syscall.
#[inline(never)]
pub fn handle_syscall_get_rent_sysvar(ctx: &mut ExecutionManager) -> CompactResult<()> {
    debug_log!("MitoVM: SYSCALL_GET_RENT_SYSVAR");

    let rent = Rent::get().map_err(|_| VMErrorCode::InvalidOperation)?;

    #[allow(deprecated)]
    let lamports_per_byte_year = rent.lamports_per_byte_year;
    ctx.push(ValueRef::U64(lamports_per_byte_year))?;
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
