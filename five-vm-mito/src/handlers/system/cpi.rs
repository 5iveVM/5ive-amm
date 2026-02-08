//! CPI syscall handlers

use crate::{
    context::ExecutionManager,
    debug_log,
    error::{CompactResult, VMErrorCode},
};

#[cfg(target_os = "solana")]
use pinocchio::syscalls;

/// Handle sol_invoke_signed_c syscall
#[inline(always)]
pub fn handle_syscall_invoke_signed_c(_ctx: &mut ExecutionManager) -> CompactResult<()> {
    debug_log!("MitoVM: SYSCALL_INVOKE_SIGNED_C");
    // This expects raw pointers to C-structs.
    // Extremely dangerous/difficult to use correctly from managed VM stack unless user manually constructs memory layout.
    // We assume arguments on stack are pointers (TempRefs) to pre-constructed data.

    // instruction_addr, account_infos_addr, account_infos_len, signers_seeds_addr, signers_seeds_len

    // Stubbed: use VM opcodes instead.
    debug_log!("SYSCALL_INVOKE_SIGNED_C not supported directly");
    Err(VMErrorCode::InvalidOperation)
}

/// Handle sol_invoke_signed_rust syscall
#[inline(always)]
pub fn handle_syscall_invoke_signed_rust(_ctx: &mut ExecutionManager) -> CompactResult<()> {
    debug_log!("MitoVM: SYSCALL_INVOKE_SIGNED_RUST");
    // Same as above.
    debug_log!("SYSCALL_INVOKE_SIGNED_RUST not supported directly");
    Err(VMErrorCode::InvalidOperation)
}
