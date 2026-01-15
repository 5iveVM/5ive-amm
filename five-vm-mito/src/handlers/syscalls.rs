//! Native syscall handlers for MitoVM CALL_NATIVE opcode
//!
//! This module provides access to Solana/Pinocchio syscalls through the Five VM,
//! enabling contracts to access native blockchain functionality while maintaining
//! zero-allocation execution principles.
//!
//! # Architecture
//!
//! The CALL_NATIVE opcode takes a single byte parameter (syscall_id) that identifies
//! which Pinocchio syscall to execute. All parameters are passed through the VM's
//! stack using ValueRef for zero-copy efficiency.
//!
//! # CU Cost Overview
//!
//! Each syscall has an associated compute unit (CU) cost based on Solana's runtime:
//! - Control: 50-100 CU (abort, panic)
//! - Sysvars: 200-400 CU (fast reads)
//! - PDAs: 1,500-3,000 CU (derivation complexity)
//! - Crypto: 2,000-150,000+ CU (varies by algorithm)
//! - Logging: 200+ CU (plus message size)
//! - Memory: 50+ CU (plus data size)
//!
//! # Safety
//!
//! All handlers maintain MitoVM's zero-allocation guarantee and use stack-based
//! parameter passing. Error conditions return VMError rather than panicking.

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
use five_protocol::ValueRef;

// ===== SYSCALL ID CONSTANTS =====
// These match the syscall_id values used in CALL_NATIVE

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

/// Handle sol_abort syscall - immediate program termination
///
/// # Description
/// Immediately terminates program execution, similar to C's abort() function.
/// This is the cleanest way to halt execution when an unrecoverable error occurs.
///
/// # Parameters
/// None
///
/// # Stack Effect
/// None (execution terminates)
///
/// # CU Cost
/// ~50 CU
///
/// # Errors
/// Always returns VMError::ExecutionTerminated to halt the VM
///
/// # Five DSL Usage
/// ```five
/// abort()  // Terminates execution immediately
/// ```
pub fn handle_syscall_abort(_ctx: &mut ExecutionManager) -> CompactResult<()> {
    debug_log!("MitoVM: SYSCALL_ABORT - terminating execution");
    // In Solana, abort() immediately terminates the program
    // We simulate this by returning a specific error
    Err(VMErrorCode::ExecutionTerminated)
}

/// Handle sol_panic_ syscall - program panic with optional message
///
/// # Description  
/// Terminates program execution with an optional panic message that can be
/// logged for debugging purposes. Provides more context than abort().
///
/// # Parameters
/// - Optional panic message (ValueRef) - if present on stack, will be logged
///
/// # Stack Effect
/// Consumes optional message from stack, then terminates
///
/// # CU Cost
/// ~50-100 CU (base cost + message size)
///
/// # Errors
/// Always returns VMError::ExecutionTerminated to halt the VM
///
/// # Five DSL Usage
/// ```five
/// panic()                    // Simple panic
/// panic("Error message")     // Panic with debug message  
/// ```
pub fn handle_syscall_panic(ctx: &mut ExecutionManager) -> CompactResult<()> {
    debug_log!("MitoVM: SYSCALL_PANIC - program panic");

    // Pop optional panic message from stack (if any)
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

// ===== PLACEHOLDER SYSCALL HANDLERS =====
// These are minimal implementations to demonstrate the pattern.
// Full implementations would require more complex parameter handling and native calls.

macro_rules! syscall_placeholder {
    ($name:ident, $log_msg:expr) => {
        pub fn $name(ctx: &mut ExecutionManager) -> CompactResult<()> {
            debug_log!("MitoVM: {} - placeholder implementation", $log_msg);
            ctx.push(ValueRef::result_ok(0, 0))?;
            Ok(())
        }
    };
}

// Program data syscalls
syscall_placeholder!(handle_syscall_get_return_data, "SYSCALL_GET_RETURN_DATA");
syscall_placeholder!(handle_syscall_set_return_data, "SYSCALL_SET_RETURN_DATA");
syscall_placeholder!(
    handle_syscall_get_processed_sibling_instruction,
    "SYSCALL_GET_PROCESSED_SIBLING_INSTRUCTION"
);
syscall_placeholder!(handle_syscall_get_stack_height, "SYSCALL_GET_STACK_HEIGHT");

// CPI syscalls
syscall_placeholder!(handle_syscall_invoke_signed_c, "SYSCALL_INVOKE_SIGNED_C");
syscall_placeholder!(
    handle_syscall_invoke_signed_rust,
    "SYSCALL_INVOKE_SIGNED_RUST"
);

// Compute syscalls
syscall_placeholder!(
    handle_syscall_remaining_compute_units,
    "SYSCALL_REMAINING_COMPUTE_UNITS"
);

// Logging syscalls
syscall_placeholder!(handle_syscall_log, "SYSCALL_LOG");
syscall_placeholder!(handle_syscall_log_64, "SYSCALL_LOG_64");
syscall_placeholder!(
    handle_syscall_log_compute_units,
    "SYSCALL_LOG_COMPUTE_UNITS"
);
syscall_placeholder!(handle_syscall_log_data, "SYSCALL_LOG_DATA");
syscall_placeholder!(handle_syscall_log_pubkey, "SYSCALL_LOG_PUBKEY");

// Memory syscalls
syscall_placeholder!(handle_syscall_memcpy, "SYSCALL_MEMCPY");
syscall_placeholder!(handle_syscall_memmove, "SYSCALL_MEMMOVE");
syscall_placeholder!(handle_syscall_memset, "SYSCALL_MEMSET");
syscall_placeholder!(handle_syscall_memcmp, "SYSCALL_MEMCMP");

// Cryptography syscalls
syscall_placeholder!(handle_syscall_sha256, "SYSCALL_SHA256");
syscall_placeholder!(handle_syscall_keccak256, "SYSCALL_KECCAK256");
syscall_placeholder!(handle_syscall_blake3, "SYSCALL_BLAKE3");
syscall_placeholder!(handle_syscall_poseidon, "SYSCALL_POSEIDON");
syscall_placeholder!(
    handle_syscall_secp256k1_recover,
    "SYSCALL_SECP256K1_RECOVER"
);
syscall_placeholder!(
    handle_syscall_alt_bn128_compression,
    "SYSCALL_ALT_BN128_COMPRESSION"
);
syscall_placeholder!(
    handle_syscall_alt_bn128_group_op,
    "SYSCALL_ALT_BN128_GROUP_OP"
);
syscall_placeholder!(handle_syscall_big_mod_exp, "SYSCALL_BIG_MOD_EXP");
syscall_placeholder!(handle_syscall_curve_group_op, "SYSCALL_CURVE_GROUP_OP");
syscall_placeholder!(
    handle_syscall_curve_multiscalar_mul,
    "SYSCALL_CURVE_MULTISCALAR_MUL"
);
syscall_placeholder!(
    handle_syscall_curve_pairing_map,
    "SYSCALL_CURVE_PAIRING_MAP"
);
syscall_placeholder!(
    handle_syscall_curve_validate_point,
    "SYSCALL_CURVE_VALIDATE_POINT"
);
