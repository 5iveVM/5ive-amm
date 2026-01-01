//! System operations handler modules for MitoVM
//!
//! This module contains sub-modules for different categories of system operations:
//! - invoke: Cross-program invocation (INVOKE, INVOKE_SIGNED, CPI, CPI_SIGNED)
//! - pda: Program Derived Address operations (DERIVE_PDA, FIND_PDA)
//! - sysvars: Blockchain sysvar access (GET_CLOCK, GET_RENT)
//! - init: Account initialization operations (INIT_ACCOUNT, INIT_PDA_ACCOUNT)
//! - native: Direct syscall access via CALL_NATIVE

pub mod init;
pub mod invoke;
pub mod native;
pub mod pda;
pub mod sysvars;

// Re-export handler functions for easy access
pub use init::handle_init_ops;
pub use invoke::handle_invoke_ops;
pub use native::handle_native_ops;
pub use pda::{handle_pda_ops, process_seed_value};
pub use sysvars::{handle_sysvar_ops, serialize_clock_to_buffer};
