//! System sysvar operations handler for MitoVM.

use crate::{
    context::ExecutionManager,
    debug_log,
    error::{CompactResult, VMErrorCode},
};
use five_protocol::{opcodes::*, ValueRef};
use pinocchio::sysvars::{
    clock::Clock,
    rent::{
        Rent, DEFAULT_BURN_PERCENT, DEFAULT_EXEMPTION_THRESHOLD, DEFAULT_LAMPORTS_PER_BYTE_YEAR,
    },
    Sysvar,
};

#[inline(always)]
fn host_fallback_clock() -> Clock {
    Clock {
        slot: 0,
        epoch_start_timestamp: 0,
        epoch: 0,
        leader_schedule_epoch: 0,
        unix_timestamp: 0,
    }
}

#[inline(always)]
#[allow(deprecated)]
fn host_fallback_rent() -> Rent {
    Rent {
        lamports_per_byte_year: DEFAULT_LAMPORTS_PER_BYTE_YEAR,
        exemption_threshold: DEFAULT_EXEMPTION_THRESHOLD,
        burn_percent: DEFAULT_BURN_PERCENT,
    }
}

#[inline(always)]
pub(crate) fn get_clock_cached(ctx: &mut ExecutionManager) -> CompactResult<Clock> {
    if let Some(cached) = ctx.cached_clock {
        return Ok(cached);
    }

    let clock = match Clock::get() {
        Ok(clock) => clock,
        Err(_) => {
            #[cfg(not(target_os = "solana"))]
            {
                host_fallback_clock()
            }
            #[cfg(target_os = "solana")]
            {
                return Err(VMErrorCode::InvalidOperation);
            }
        }
    };
    ctx.cached_clock = Some(clock);
    Ok(clock)
}

#[inline(always)]
pub(crate) fn get_rent_cached(ctx: &mut ExecutionManager) -> CompactResult<Rent> {
    if let Some(cached) = ctx.cached_rent {
        return Ok(cached);
    }

    let rent = match Rent::get() {
        Ok(rent) => rent,
        Err(_) => {
            #[cfg(not(target_os = "solana"))]
            {
                host_fallback_rent()
            }
            #[cfg(target_os = "solana")]
            {
                return Err(VMErrorCode::InvalidOperation);
            }
        }
    };
    ctx.cached_rent = Some(rent);
    Ok(rent)
}

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
#[inline(always)]
pub fn handle_sysvar_ops(opcode: u8, ctx: &mut ExecutionManager) -> CompactResult<()> {
    match opcode {
        GET_CLOCK => {
            debug_log!("MitoVM: GET_CLOCK operation");
            let clock = get_clock_cached(ctx)?;

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
            let rent = get_rent_cached(ctx)?;

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
#[inline(always)]
pub fn handle_syscall_get_clock_sysvar(ctx: &mut ExecutionManager) -> CompactResult<()> {
    debug_log!("MitoVM: SYSCALL_GET_CLOCK_SYSVAR");
    let clock = get_clock_cached(ctx)?;

    let temp_buffer = ctx.temp_buffer_mut();
    if temp_buffer.len() < 40 {
        return Err(VMErrorCode::MemoryViolation);
    }

    serialize_clock_to_buffer(&clock, temp_buffer);
    ctx.push(ValueRef::TupleRef(0, 40))?;
    Ok(())
}

/// Handle sol_get_rent_sysvar syscall.
#[inline(always)]
pub fn handle_syscall_get_rent_sysvar(ctx: &mut ExecutionManager) -> CompactResult<()> {
    debug_log!("MitoVM: SYSCALL_GET_RENT_SYSVAR");
    let rent = get_rent_cached(ctx)?;

    #[allow(deprecated)]
    let lamports_per_byte_year = rent.lamports_per_byte_year;
    ctx.push(ValueRef::U64(lamports_per_byte_year))?;
    Ok(())
}

#[inline(always)]
fn unsupported_sysvar_syscall(_ctx: &mut ExecutionManager, _name: &str) -> CompactResult<()> {
    debug_log!("MitoVM: {} - runtime integration required", _name);
    Err(VMErrorCode::RuntimeIntegrationRequired)
}

pub fn handle_syscall_get_epoch_schedule_sysvar(ctx: &mut ExecutionManager) -> CompactResult<()> {
    unsupported_sysvar_syscall(ctx, "SYSCALL_GET_EPOCH_SCHEDULE_SYSVAR")
}

pub fn handle_syscall_get_epoch_rewards_sysvar(ctx: &mut ExecutionManager) -> CompactResult<()> {
    unsupported_sysvar_syscall(ctx, "SYSCALL_GET_EPOCH_REWARDS_SYSVAR")
}

pub fn handle_syscall_get_epoch_stake(ctx: &mut ExecutionManager) -> CompactResult<()> {
    unsupported_sysvar_syscall(ctx, "SYSCALL_GET_EPOCH_STAKE")
}

pub fn handle_syscall_get_fees_sysvar(ctx: &mut ExecutionManager) -> CompactResult<()> {
    unsupported_sysvar_syscall(ctx, "SYSCALL_GET_FEES_SYSVAR")
}

pub fn handle_syscall_get_last_restart_slot(ctx: &mut ExecutionManager) -> CompactResult<()> {
    unsupported_sysvar_syscall(ctx, "SYSCALL_GET_LAST_RESTART_SLOT")
}

pub fn handle_syscall_get_sysvar(ctx: &mut ExecutionManager) -> CompactResult<()> {
    unsupported_sysvar_syscall(ctx, "SYSCALL_GET_SYSVAR")
}
