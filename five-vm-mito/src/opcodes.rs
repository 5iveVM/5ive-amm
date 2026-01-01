//! Opcode helpers for MitoVM that delegate to five-protocol (single source of truth).

// Re-export ArgType and all opcode constants from protocol
pub use five_protocol::opcodes::*;
pub use five_protocol::ArgType;

/// Get human-readable name for an opcode (delegates to protocol table)
#[inline]
pub fn opcode_name(opcode: u8) -> &'static str {
    five_protocol::opcodes::opcode_name(opcode)
}

/// Validate opcode (delegates to protocol table)
#[inline]
pub fn is_valid_opcode(opcode: u8) -> bool {
    five_protocol::opcodes::is_valid_opcode(opcode)
}

/// Get expected compute unit cost for an opcode (delegates to protocol table)
#[inline]
pub fn opcode_cu_cost(opcode: u8) -> u8 {
    five_protocol::opcodes::opcode_compute_cost(opcode)
}

/// Get argument type for an opcode (reads from protocol table)
#[inline]
pub fn opcode_arg_type(opcode: u8) -> ArgType {
    match five_protocol::opcodes::get_opcode_info(opcode) {
        Some(info) => info.arg_type,
        None => ArgType::None,
    }
}

/// Enhanced instruction format documentation (kept for developer reference)
pub const ENHANCED_INSTRUCTION_FORMATS: &str = r#"
Enhanced MitoVM Instruction Formats:

CALL format:
  CALL u16_offset
  
RET format:
  RET (no args)
  
WRITE_DATA format:
  WRITE_DATA (pops: AccountData, u64_value)
  
Enhanced PUSH format:
  PUSH_PUBKEY [32 bytes]
"#;
