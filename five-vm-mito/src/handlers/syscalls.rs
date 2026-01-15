//! Native syscall handlers for MitoVM CALL_NATIVE opcode
//!
//! This module provides access to Solana/Pinocchio syscalls through the Five VM,
//! enabling contracts to access native blockchain functionality while maintaining
//! zero-allocation execution principles.
//!
//! The CALL_NATIVE opcode takes a single byte parameter (syscall_id) that identifies
//! which Pinocchio syscall to execute. All parameters are passed through the VM's
//! stack using ValueRef for zero-copy efficiency.

use crate::{
    context::ExecutionManager,
    debug_log,
    error::{CompactResult, VMErrorCode},
};
pub use crate::handlers::system::pda::{
    handle_syscall_create_program_address, handle_syscall_try_find_program_address,
};
pub use crate::handlers::system::sysvars::{
    handle_syscall_get_clock_sysvar, handle_syscall_get_epoch_rewards_sysvar,
    handle_syscall_get_epoch_schedule_sysvar, handle_syscall_get_epoch_stake,
    handle_syscall_get_fees_sysvar, handle_syscall_get_last_restart_slot,
    handle_syscall_get_rent_sysvar, handle_syscall_get_sysvar,
};
pub use crate::handlers::system::logging::{
    handle_syscall_log, handle_syscall_log_64, handle_syscall_log_compute_units,
    handle_syscall_log_data, handle_syscall_log_pubkey,
};
pub use crate::handlers::system::compute::handle_syscall_remaining_compute_units;
pub use crate::handlers::system::program::{
    handle_syscall_get_return_data, handle_syscall_set_return_data,
    handle_syscall_get_stack_height, handle_syscall_get_processed_sibling_instruction,
};
pub use crate::handlers::system::memory::{
    handle_syscall_memcpy, handle_syscall_memmove, handle_syscall_memset, handle_syscall_memcmp,
};
pub use crate::handlers::system::crypto::{
    handle_syscall_sha256, handle_syscall_keccak256, // handle_syscall_blake3,
    handle_syscall_poseidon, handle_syscall_secp256k1_recover,
    handle_syscall_alt_bn128_compression, handle_syscall_alt_bn128_group_op,
    handle_syscall_big_mod_exp, handle_syscall_curve_group_op,
    handle_syscall_curve_multiscalar_mul, handle_syscall_curve_pairing_map,
    handle_syscall_curve_validate_point,
};
pub use crate::handlers::system::cpi::{
    handle_syscall_invoke_signed_c, handle_syscall_invoke_signed_rust,
};

// ===== SYSCALL ID CONSTANTS =====

pub const SYSCALL_ABORT: u8 = 1;
pub const SYSCALL_PANIC: u8 = 2;

pub const SYSCALL_CREATE_PROGRAM_ADDRESS: u8 = 10;
pub const SYSCALL_TRY_FIND_PROGRAM_ADDRESS: u8 = 11;

pub const SYSCALL_GET_CLOCK_SYSVAR: u8 = 20;
pub const SYSCALL_GET_EPOCH_SCHEDULE_SYSVAR: u8 = 21;
pub const SYSCALL_GET_EPOCH_REWARDS_SYSVAR: u8 = 22;
pub const SYSCALL_GET_EPOCH_STAKE: u8 = 23;
pub const SYSCALL_GET_FEES_SYSVAR: u8 = 24;
pub const SYSCALL_GET_RENT_SYSVAR: u8 = 25;
pub const SYSCALL_GET_LAST_RESTART_SLOT: u8 = 26;
pub const SYSCALL_GET_SYSVAR: u8 = 27;

pub const SYSCALL_GET_RETURN_DATA: u8 = 30;
pub const SYSCALL_SET_RETURN_DATA: u8 = 31;
pub const SYSCALL_GET_PROCESSED_SIBLING_INSTRUCTION: u8 = 32;
pub const SYSCALL_GET_STACK_HEIGHT: u8 = 33;

pub const SYSCALL_INVOKE_SIGNED_C: u8 = 40;
pub const SYSCALL_INVOKE_SIGNED_RUST: u8 = 41;

pub const SYSCALL_REMAINING_COMPUTE_UNITS: u8 = 50;

pub const SYSCALL_LOG: u8 = 60;
pub const SYSCALL_LOG_64: u8 = 61;
pub const SYSCALL_LOG_COMPUTE_UNITS: u8 = 62;
pub const SYSCALL_LOG_DATA: u8 = 63;
pub const SYSCALL_LOG_PUBKEY: u8 = 64;

pub const SYSCALL_MEMCPY: u8 = 70;
pub const SYSCALL_MEMMOVE: u8 = 71;
pub const SYSCALL_MEMSET: u8 = 72;
pub const SYSCALL_MEMCMP: u8 = 73;

pub const SYSCALL_SHA256: u8 = 80;
pub const SYSCALL_KECCAK256: u8 = 81;
pub const SYSCALL_BLAKE3: u8 = 82;
pub const SYSCALL_POSEIDON: u8 = 83;
pub const SYSCALL_SECP256K1_RECOVER: u8 = 84;
pub const SYSCALL_ALT_BN128_COMPRESSION: u8 = 85;
pub const SYSCALL_ALT_BN128_GROUP_OP: u8 = 86;
pub const SYSCALL_BIG_MOD_EXP: u8 = 87;
pub const SYSCALL_CURVE_GROUP_OP: u8 = 88;
pub const SYSCALL_CURVE_MULTISCALAR_MUL: u8 = 89;
pub const SYSCALL_CURVE_PAIRING_MAP: u8 = 90;
pub const SYSCALL_CURVE_VALIDATE_POINT: u8 = 91;

// ===== CONTROL SYSCALLS =====

/// Handle sol_abort syscall - immediate program termination.
pub fn handle_syscall_abort(_ctx: &mut ExecutionManager) -> CompactResult<()> {
    debug_log!("MitoVM: SYSCALL_ABORT - terminating execution");
    Err(VMErrorCode::ExecutionTerminated)
}

/// Handle sol_panic_ syscall - program panic with optional message.
pub fn handle_syscall_panic(ctx: &mut ExecutionManager) -> CompactResult<()> {
    debug_log!("MitoVM: SYSCALL_PANIC - program panic");

    if ctx.size() > 0 {
        if let Ok(_msg_ref) = ctx.pop() {
            #[cfg(feature = "debug-logs")]
            {
                use heapless::String as HString;
                let mut s = HString::<256>::new();
                let _ = core::fmt::write(&mut s, format_args!("{:?}", _msg_ref));
                debug_log!("MitoVM: SYSCALL_PANIC with message: {}", s.as_str());
            }
        }
    }

    Err(VMErrorCode::ExecutionTerminated)
}
