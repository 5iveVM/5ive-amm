//! Disassembler and diagnostic utilities for FIVE DSL bytecode.
//!
//! This module provides:
//! - A small textual disassembler (`disassemble`) for human-readable output.
//! - A structured decoder (`Instruction`, `decode_instruction_at`) for tooling.
//! - A simple inspector (`BytecodeInspector`) with high-level queries used by tests.
//!
//! The implementation aims to be defensive: it never panics on truncated input
//! and prefers conservative advances when it cannot fully decode an operand.

// Core type definitions
pub mod types;
pub use types::*;

// Low-level decoding utilities
pub mod decoder;
pub use decoder::{
    decode_vle_u128, read_byte, read_le_u16, read_le_u32, read_le_u64, read_utf8_string,
};

// CALL instruction decoding (shared)
mod call_decoder;

// High-level inspection queries
pub mod inspector;
pub use inspector::BytecodeInspector;

// Textual disassembly
pub mod disasm;
pub use disasm::disassemble;

// Pretty-printing and formatting
pub mod pretty;
pub use pretty::{get_disassembly, pretty_instruction};

// Diagnostic utilities
pub mod diagnostics;
pub use diagnostics::inspect_failure;

// Bytecode verification (JUMP target validation)
pub mod verification;
pub use verification::{verify_jump_targets, VerificationResult, VerificationError};

// Macro definitions (for internal use)
#[allow(unused_macros)]
pub mod macros;

/// Re-export opcode constants for convenient use by callers.
pub use five_protocol::opcodes;
