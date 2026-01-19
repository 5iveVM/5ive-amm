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
#[inline(always)]
pub fn handle_syscall_remaining_compute_units(ctx: &mut ExecutionManager) -> CompactResult<()> {
    debug_log!("MitoVM: SYSCALL_REMAINING_COMPUTE_UNITS");

    let remaining;
    // Only use the syscall if the feature is enabled AND we're on Solana
    // The sol_remaining_compute_units syscall is not available on devnet v3.1.6 or earlier
    #[cfg(all(target_os = "solana", feature = "remaining-cu-syscall"))]
    unsafe {
        remaining = syscalls::sol_remaining_compute_units();
    }
    #[cfg(not(all(target_os = "solana", feature = "remaining-cu-syscall")))]
    {
        remaining = 200_000; // Mock value for devnet compatibility
    }

    ctx.push(ValueRef::U64(remaining))?;
    Ok(())
}
