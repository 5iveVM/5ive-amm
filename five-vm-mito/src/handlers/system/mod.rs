//! System operations handler modules for MitoVM
//!
//! This module contains sub-modules for different categories of system operations:
//! - invoke: Cross-program invocation (INVOKE, INVOKE_SIGNED, CPI, CPI_SIGNED)
//! - pda: Program Derived Address operations (DERIVE_PDA, FIND_PDA)
//! - sysvars: Blockchain sysvar access (GET_CLOCK, GET_RENT)
//! - init: Account initialization operations (INIT_ACCOUNT, INIT_PDA_ACCOUNT)

pub mod init;
pub mod invoke;
pub mod pda;
pub mod sysvars;

// Re-export handler functions for easy access
pub use init::handle_init_ops;
pub use invoke::handle_invoke_ops;
pub use pda::{handle_pda_ops, process_seed_value};
pub use sysvars::{handle_sysvar_ops, serialize_clock_to_buffer};

use crate::{
    context::ExecutionManager,
    error::{CompactResult, VMErrorCode},
};
use five_protocol::opcodes::*;

/// Dispatch system-level operations including CPI, PDA operations, and account initialization.
#[inline(never)]
pub fn handle_system_ops(opcode: u8, ctx: &mut ExecutionManager) -> CompactResult<()> {
    match opcode {
        // Cross-program invocation operations (INVOKE, INVOKE_SIGNED)
        INVOKE | INVOKE_SIGNED => handle_invoke_ops(opcode, ctx),
        // Blockchain sysvar operations (GET_CLOCK, GET_RENT)
        GET_CLOCK | GET_RENT => handle_sysvar_ops(opcode, ctx),
        // Account initialization operations (INIT_ACCOUNT, INIT_PDA_ACCOUNT)
        INIT_ACCOUNT | INIT_PDA_ACCOUNT => handle_init_ops(opcode, ctx),
        // Program Derived Address operations (DERIVE_PDA, FIND_PDA, etc.)
        DERIVE_PDA | FIND_PDA | DERIVE_PDA_PARAMS | FIND_PDA_PARAMS => {
            handle_pda_ops(opcode, ctx)
        }
        _ => Err(VMErrorCode::InvalidInstruction),
    }
}
