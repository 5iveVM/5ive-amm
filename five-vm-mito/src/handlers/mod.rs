//! Handler modules for MitoVM opcode execution
//!
//! This module contains all the individual opcode handler modules that have been
//! extracted from the monolithic execution engine for better maintainability.

pub mod accounts;
pub mod arithmetic;
pub mod constraints;
pub mod control_flow;
pub mod logical;
pub mod memory;
pub mod stack_ops;
pub mod advanced;
pub mod arrays;
pub mod functions;
pub mod locals;
pub mod option_result;
pub mod syscalls;
pub mod system;

// Re-export handler functions for easy access
pub use accounts::handle_accounts;
pub use arithmetic::handle_arithmetic;
pub use constraints::handle_constraints;
pub use control_flow::handle_control_flow;
pub use logical::handle_logical;
pub use memory::handle_memory;
pub use stack_ops::handle_stack_ops;
pub use advanced::handle_advanced;
pub use arrays::handle_arrays;
pub use functions::handle_functions;
pub use locals::{handle_locals, handle_nibble_locals};
pub use option_result::handle_option_result_ops;
pub use syscalls::*;
pub use system::{
    handle_init_ops, handle_invoke_ops, handle_pda_ops, handle_system_ops, handle_sysvar_ops,
};
pub mod fused_ops;
