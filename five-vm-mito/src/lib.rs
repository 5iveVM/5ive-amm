#![allow(unexpected_cfgs)]
//! MitoVM for FIVE with function calls.
//!
//! Optimized for Solana's stateless execution model:
//! - Zero allocations during execution
//! - Direct AccountInfo access
//! - Stack-based execution with minimal function call support
//! - Cold start optimized for low CU overhead
//! - Essential opcodes with function transport
//!
//! # Quick Start
//!
//! ```rust
//! use five_vm_mito::{MitoVM, Value, Pubkey, StackStorage};
//! use five_vm_mito::opcodes::{PUSH_U8, ADD, RETURN_VALUE};
//!
//! // Create bytecode that adds two numbers
//! // FIVE header (10 bytes): magic(4) + features(4) + public_count(1) + total_count(1)
//! let bytecode = &[
//!     b'5', b'I', b'V', b'E', // FIVE magic
//!     0x00, 0x00, 0x00, 0x00, // features
//!     0x01,                   // public_count
//!     0x01,                   // total_count
//!     PUSH_U8, 10,            // PUSH_U8 10
//!     PUSH_U8, 5,             // PUSH_U8 5
//!     ADD,                    // ADD
//!     RETURN_VALUE            // RETURN_VALUE
//! ];
//!
//! // Execute with no input data or accounts
//! let mut storage = five_vm_mito::StackStorage::new();
//! let result = MitoVM::execute_direct(bytecode, &[], &[], &Pubkey::default(), &mut storage)?;
//! assert_eq!(result, Some(Value::U64(15)));
//! # Ok::<(), five_vm_mito::VMError>(())
//! ```
//!
//! # Function Calls
//!
//! ```rust
//! use five_vm_mito::{MitoVM, Pubkey};
//! use five_vm_mito::opcodes::{LOAD_PARAM, PUSH_U8, MUL, RETURN_VALUE};
//!
//! // Bytecode with function header and simple function
//! // FIVE header: magic(4) + features(4) + public_count(1) + total_count(1) = 10 bytes
//! let bytecode = &[
//!     b'5', b'I', b'V', b'E', // FIVE magic
//!     0x00, 0x00, 0x00, 0x00, // features flags (4 bytes)
//!     0x01,                   // public function count: 1
//!     0x01,                   // total function count: 1
//!     // Main function: multiply param #1 by 2
//!     LOAD_PARAM, 1,          // LOAD_PARAM 1
//!     PUSH_U8, 2,             // PUSH_U8 2
//!     MUL,                    // MUL
//!     RETURN_VALUE            // RETURN_VALUE
//! ];
//!
//! // Call with parameter: function 0, value 21
//! // Input format: [func_index (u32), param_count (u32), type_id (u8), param bytes...]
//! let input_data = &[
//!     0x00, 0x00, 0x00, 0x00, // function index 0
//!     0x01, 0x00, 0x00, 0x00, // param count 1
//!     0x04,                   // type_id = U64
//!     21, 0, 0, 0, 0, 0, 0, 0, // u64 value 21
//! ];
//! let accounts = &[];
//! let program_id = Pubkey::default();
//! let mut storage = five_vm_mito::StackStorage::new();
//! let result = MitoVM::execute_direct(bytecode, input_data, accounts, &program_id, &mut storage)?;
//! assert_eq!(result, Some(five_vm_mito::Value::U64(42)));
//! # Ok::<(), five_vm_mito::VMError>(())
//! ```

extern crate alloc;

pub mod context;
pub mod error;
pub mod execution;
pub mod handlers;
pub mod lazy_validation;
pub mod macros;
pub mod metadata;
pub mod opcodes;
pub mod resolution;
pub mod stack;
pub mod systems;
pub mod types;
pub mod utils;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod bench_lazy_validation;

pub use context::{ExecutionContext, ExecutionManager};
pub use error::{Result, VMError};
pub use execution::MitoVM;
#[cfg(not(target_os = "solana"))]
pub use execution::VMExecutionContext;
pub use five_protocol::Value;
pub use lazy_validation::{LazyAccountValidator, ValidationStats};
pub use stack::StackStorage;
pub use types::{CallFrame, LocalVariables};
#[cfg(feature = "debug-logs")]
pub use utils::DebugUtils;
pub use utils::{BytecodeUtils, ErrorUtils, ValueRefUtils};

// Re-export pinocchio types
pub use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};

pub use five_protocol::{FIVE_DEPLOY_MAGIC, FIVE_MAGIC};

/// FIVE VM Program ID (placeholder; should match actual deployed program).
pub const FIVE_VM_PROGRAM_ID: [u8; 32] = [
    0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10,
    0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20,
];

/// Stack size for function calls (reduced to fix SBF stack overflow).
pub const STACK_SIZE: usize = 32;

/// Maximum number of local variables allowed globally.
pub const MAX_LOCALS: usize = five_protocol::MAX_LOCALS;

/// Maximum function parameters (limited by parameter array size)
pub const MAX_PARAMETERS: usize = five_protocol::MAX_FUNCTION_PARAMS;

/// Maximum script size in bytes.
pub const MAX_SCRIPT_SIZE: usize = five_protocol::MAX_SCRIPT_SIZE;

/// Function call stack depth (bounded by stack limits).
pub const MAX_CALL_DEPTH: usize = five_protocol::MAX_CALL_DEPTH;

/// Temporary buffer size for byte operations (heap-backed in context).
pub const TEMP_BUFFER_SIZE: usize = five_protocol::TEMP_BUFFER_SIZE; // default 512

pub use five_protocol::*;

/// Legacy enhanced opcodes module for compatibility.
pub mod enhanced_opcodes {
    // Re-export all opcodes from function transport.
    pub use five_protocol::*;

    // Alias for function return compatibility.
    pub use five_protocol::opcodes::RETURN as RET;
}

#[cfg(feature = "debug-logs")]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        pinocchio_log::log!($($arg)*)
    };
}

#[cfg(not(feature = "debug-logs"))]
macro_rules! debug_log {
    ($($arg:tt)*) => {};
}

pub(crate) use debug_log;

macro_rules! error_log {
    ($($arg:tt)*) => {
        let _ = format_args!($($arg)*);
        #[cfg(target_os = "solana")]
        unsafe {
            #[cfg(feature = "debug-logs")]
            {
                pinocchio::log::sol_log("VM ERROR:");
                pinocchio_log::log!($($arg)*);
            }
        }
    };
}

pub(crate) use error_log;
