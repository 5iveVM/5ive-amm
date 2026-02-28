//! Bytecode disassembler and verification utilities.

pub mod call_decoder;
pub mod decoder;
pub mod disasm;
pub mod inspector;
pub mod types;
pub mod verification;

pub use call_decoder::*;
pub use decoder::{read_byte, read_le_u16, read_le_u32, read_le_u64, read_utf8_string};
pub use disasm::{disassemble, get_disassembly, inspect_failure};
pub use inspector::BytecodeInspector;
pub use types::*;
pub use verification::{verify_jump_targets, VerificationError, VerificationResult};
