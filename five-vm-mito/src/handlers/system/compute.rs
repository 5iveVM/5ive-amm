//! Compute unit syscall handlers

use crate::{
    context::ExecutionManager,
    debug_log,
    error::CompactResult,
};
use five_protocol::ValueRef;

#[cfg(target_os = "solana")]
use pinocchio::syscalls;

/// Handle sol_remaining_compute_units syscall
#[inline(never)]
pub fn handle_syscall_remaining_compute_units(ctx: &mut ExecutionManager) -> CompactResult<()> {
    debug_log!("MitoVM: SYSCALL_REMAINING_COMPUTE_UNITS");

    let remaining;
    #[cfg(target_os = "solana")]
    unsafe {
        remaining = syscalls::sol_remaining_compute_units();
    }
    #[cfg(not(target_os = "solana"))]
    {
        remaining = 200_000; // Mock value
    }

    ctx.push(ValueRef::U64(remaining))?;
    Ok(())
}
