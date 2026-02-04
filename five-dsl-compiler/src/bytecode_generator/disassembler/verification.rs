//! Bytecode Verification Module
//!
//! This module provides verification utilities for compiled bytecode,
//! specifically to detect invalid JUMP targets that can cause runtime errors.
//!
//! Deploy-time verification enables unchecked-execution mode where runtime
//! bounds checks are skipped on bytecode access, assuming all bytecode was
//! verified at deployment time.

use five_protocol::opcodes;

/// Result of bytecode verification
#[derive(Debug, Clone)]
pub struct VerificationResult {
    /// Whether the bytecode is valid
    pub is_valid: bool,
    /// List of verification errors found
    pub errors: Vec<VerificationError>,
    /// Total number of JUMP instructions scanned
    pub jump_count: usize,
    /// Total bytecode length
    pub bytecode_length: usize,
}

/// A single verification error
#[derive(Debug, Clone)]
pub struct VerificationError {
    /// Offset of the instruction in bytecode
    pub offset: usize,
    /// The opcode that has the invalid target
    pub opcode: u8,
    /// The opcode name for display
    pub opcode_name: &'static str,
    /// The invalid target address
    pub target: u16,
    /// Why it's invalid
    pub reason: String,
}

impl std::fmt::Display for VerificationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "0x{:04X}: {} target 0x{:04X} ({}) - {}",
            self.offset, self.opcode_name, self.target, self.target, self.reason
        )
    }
}

impl VerificationResult {
    /// Create a new successful result
    pub fn success(jump_count: usize, bytecode_length: usize) -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            jump_count,
            bytecode_length,
        }
    }

    /// Create a result with errors
    pub fn with_errors(errors: Vec<VerificationError>, jump_count: usize, bytecode_length: usize) -> Self {
        Self {
            is_valid: errors.is_empty(),
            errors,
            jump_count,
            bytecode_length,
        }
    }

    /// Get a formatted summary of errors
    pub fn error_summary(&self) -> String {
        if self.is_valid {
            format!(
                "✓ Bytecode verified: {} bytes, {} JUMP instructions, all valid",
                self.bytecode_length, self.jump_count
            )
        } else {
            let mut summary = format!(
                "✗ Bytecode verification FAILED: {} bytes, {} JUMP instructions, {} errors:\n",
                self.bytecode_length, self.jump_count, self.errors.len()
            );
            for error in &self.errors {
                summary.push_str(&format!("  - {}\n", error));
            }
            summary
        }
    }
}

/// Verifies that all JUMP targets in bytecode are within bounds.
///
/// This function scans the bytecode for JUMP, JUMP_IF, JUMP_IF_NOT, and CALL
/// instructions, extracts their target addresses, and verifies each target
/// is a valid bytecode offset.
///
/// # Arguments
/// * `bytecode` - The compiled bytecode to verify
///
/// # Returns
/// A `VerificationResult` containing validation status and any errors found.
pub fn verify_jump_targets(bytecode: &[u8]) -> VerificationResult {
    let mut errors = Vec::new();
    let mut jump_count = 0;
    let mut offset = 0;

    while offset < bytecode.len() {
        let opcode = bytecode[offset];
        let operand_size = get_operand_size(opcode, &bytecode[offset + 1..]);
        let total_size = 1 + operand_size;

        // Check if instruction is complete
        if offset + total_size > bytecode.len() {
            break;
        }

        // Check JUMP instructions
        if is_jump_instruction(opcode) {
            jump_count += 1;
            if let Some(target) = extract_target(opcode, bytecode, offset) {
                let target_offset = target as usize;
                if target_offset >= bytecode.len() {
                    errors.push(VerificationError {
                        offset,
                        opcode,
                        opcode_name: opcode_name(opcode),
                        target,
                        reason: format!(
                            "target {} is beyond bytecode end {}",
                            target_offset, bytecode.len()
                        ),
                    });
                }
            }
        }

        offset += total_size;
    }

    VerificationResult::with_errors(errors, jump_count, bytecode.len())
}

fn is_jump_instruction(opcode: u8) -> bool {
    matches!(
        opcode,
        opcodes::JUMP
            | opcodes::JUMP_IF
            | opcodes::JUMP_IF_NOT
            | opcodes::EQ_ZERO_JUMP
            | opcodes::GT_ZERO_JUMP
            | opcodes::LT_ZERO_JUMP
            | opcodes::CALL
    )
}

fn extract_target(opcode: u8, bytecode: &[u8], offset: usize) -> Option<u16> {
    match opcode {
        opcodes::JUMP | opcodes::JUMP_IF | opcodes::JUMP_IF_NOT | opcodes::EQ_ZERO_JUMP
        | opcodes::GT_ZERO_JUMP | opcodes::LT_ZERO_JUMP => {
            // Fixed u16 little-endian encoding
            if offset + 3 <= bytecode.len() {
                Some(u16::from_le_bytes([bytecode[offset + 1], bytecode[offset + 2]]))
            } else {
                None
            }
        }
        opcodes::CALL => {
            // CallInternal: param_count(u8) + function_address(u16)
            if offset + 3 <= bytecode.len() {
                Some(u16::from_le_bytes([bytecode[offset + 2], bytecode[offset + 3]]))
            } else {
                None
            }
        }
        _ => None,
    }
}

fn opcode_name(opcode: u8) -> &'static str {
    match opcode {
        opcodes::JUMP => "JUMP",
        opcodes::JUMP_IF => "JUMP_IF",
        opcodes::JUMP_IF_NOT => "JUMP_IF_NOT",
        opcodes::EQ_ZERO_JUMP => "EQ_ZERO_JUMP",
        opcodes::GT_ZERO_JUMP => "GT_ZERO_JUMP",
        opcodes::LT_ZERO_JUMP => "LT_ZERO_JUMP",
        opcodes::CALL => "CALL",
        _ => "UNKNOWN",
    }
}

/// Get the size of operands for an opcode (not including opcode byte itself)
fn get_operand_size(opcode: u8, remaining: &[u8]) -> usize {
    match opcode {
        // No operand instructions
        opcodes::HALT | opcodes::POP | opcodes::DUP | opcodes::SWAP | opcodes::ADD | opcodes::SUB
        | opcodes::MUL | opcodes::DIV | opcodes::MOD | opcodes::EQ | opcodes::LT | opcodes::GT
        | opcodes::AND | opcodes::OR | opcodes::NOT | opcodes::LOAD_GLOBAL
        | opcodes::INVOKE | opcodes::INVOKE_SIGNED | opcodes::TRANSFER | opcodes::CHECK_OWNER
        | opcodes::ALLOC_LOCALS | opcodes::DEALLOC_LOCALS
        | opcodes::RETURN | opcodes::RETURN_ERROR | opcodes::RETURN_SUCCESS | opcodes::REQUIRE
        | opcodes::CAST => 0,

        // JUMP instructions: u16 offset (fixed)
        opcodes::JUMP | opcodes::JUMP_IF | opcodes::JUMP_IF_NOT | opcodes::EQ_ZERO_JUMP
        | opcodes::GT_ZERO_JUMP | opcodes::LT_ZERO_JUMP => 2,

        // CALL: param_count(1) + address(2) = 3
        opcodes::CALL => 3,

        // Single byte operands (no nibble immediates)
        opcodes::PUSH_0 | opcodes::PUSH_1 | opcodes::PUSH_2 | opcodes::PUSH_3
        | opcodes::GET_LOCAL_0 | opcodes::GET_LOCAL_1 | opcodes::GET_LOCAL_2
        | opcodes::GET_LOCAL_3 | opcodes::SET_LOCAL_0 | opcodes::SET_LOCAL_1
        | opcodes::SET_LOCAL_2 | opcodes::SET_LOCAL_3 | opcodes::LOAD_PARAM_0
        | opcodes::LOAD_PARAM_1 | opcodes::LOAD_PARAM_2 | opcodes::LOAD_PARAM_3 => 0,

        // Single byte operands
        opcodes::PUSH_U8 | opcodes::PUSH_BOOL | opcodes::CHECK_SIGNER | opcodes::CHECK_WRITABLE
        | opcodes::CHECK_INITIALIZED | opcodes::CHECK_UNINITIALIZED | opcodes::SET_LOCAL
        | opcodes::GET_LOCAL | opcodes::LOAD_PARAM | opcodes::STORE_PARAM | opcodes::CAST
        | opcodes::CREATE_ARRAY | opcodes::PUSH_ARRAY_LITERAL
        // Account opcodes taking 1 byte account index
        | opcodes::LOAD_ACCOUNT | opcodes::SAVE_ACCOUNT | opcodes::GET_ACCOUNT
        | opcodes::GET_LAMPORTS | opcodes::SET_LAMPORTS | opcodes::GET_DATA | opcodes::GET_KEY
        | opcodes::GET_OWNER
        // Fused: single byte operands
        | opcodes::CHECK_SIGNER_WRITABLE
        | opcodes::REQUIRE_PARAM_GT_ZERO => 1,

        // Two byte operands
        // (none in current protocol)

        // VLE-encoded operands - need to parse VLE to know actual size
        opcodes::PUSH_U16 => decode_vle_size(remaining, 2),
        opcodes::PUSH_U32 => decode_vle_size(remaining, 4),
        opcodes::PUSH_U64 | opcodes::PUSH_I64 => decode_vle_size(remaining, 9),

        // LOAD_FIELD/STORE_FIELD: account_index(1) + field_offset(VLE)
        opcodes::LOAD_FIELD | opcodes::STORE_FIELD | opcodes::LOAD_FIELD_PUBKEY => {
            1 + decode_vle_size(remaining.get(1..).unwrap_or(&[]), 4)
        }

        // Fused opcodes with acc(u8) + offset(VLE) format
        opcodes::REQUIRE_NOT_BOOL | opcodes::STORE_FIELD_ZERO => {
            1 + decode_vle_size(remaining.get(1..).unwrap_or(&[]), 4)
        }

        // Fused opcodes with acc(u8) + offset(VLE) + param/acc(u8) format
        opcodes::REQUIRE_GTE_U64 | opcodes::FIELD_ADD_PARAM | opcodes::FIELD_SUB_PARAM
        | opcodes::STORE_PARAM_TO_FIELD | opcodes::STORE_KEY_TO_FIELD => {
            2 + decode_vle_size(remaining.get(1..).unwrap_or(&[]), 4)
        }

        // Fused opcodes with acc1(u8) + offset1(VLE) + acc2(u8) + offset2(VLE) format
        opcodes::REQUIRE_EQ_PUBKEY | opcodes::REQUIRE_EQ_FIELDS => {
            let vle1_size = decode_vle_size(remaining.get(1..).unwrap_or(&[]), 4);
            let vle2_size = decode_vle_size(remaining.get(2 + vle1_size..).unwrap_or(&[]), 4);
            2 + vle1_size + vle2_size
        }

        // PUSH_PUBKEY: 32 bytes
        opcodes::PUSH_PUBKEY => 32,

        // PUSH_STRING: VLE length + string bytes
        opcodes::PUSH_STRING => {
            let vle_size = decode_vle_size(remaining, 4);
            if vle_size > 0 && remaining.len() >= vle_size {
                // Parse the VLE length
                let (len, _) = decode_vle_value(&remaining[..vle_size]);
                vle_size + len as usize
            } else {
                1 // Fallback: just skip 1 byte
            }
        }

        // CREATE_TUPLE: element_count(1)
        opcodes::CREATE_TUPLE => 1,

        // Default: assume no operands
        _ => 0,
    }
}

/// Decode VLE size from bytes (returns actual byte count used)
fn decode_vle_size(bytes: &[u8], max_size: usize) -> usize {
    let mut size = 0;
    for (i, &b) in bytes.iter().take(max_size).enumerate() {
        size = i + 1;
        if b & 0x80 == 0 {
            break;
        }
    }
    size
}

/// Decode VLE value and return (value, bytes_consumed)
fn decode_vle_value(bytes: &[u8]) -> (u64, usize) {
    let mut value: u64 = 0;
    let mut shift = 0;
    for (i, &b) in bytes.iter().enumerate() {
        value |= ((b & 0x7F) as u64) << shift;
        if b & 0x80 == 0 {
            return (value, i + 1);
        }
        shift += 7;
        if shift > 63 {
            break;
        }
    }
    (value, bytes.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_bytecode() {
        // Simple valid bytecode: HALT (no header)
        let bytecode = vec![opcodes::HALT];
        let result = verify_jump_targets(&bytecode);
        assert!(result.is_valid);
        assert_eq!(result.errors.len(), 0);
    }

    #[test]
    fn test_invalid_jump_target() {
        // JUMP to offset 1000 using fixed u16 in small bytecode
        // 1000 in little-endian u16: 0xE8, 0x03
        let bytecode = vec![
            opcodes::JUMP,
            0xE8, 0x03, // 1000 in little-endian u16
            opcodes::HALT,
        ];
        let result = verify_jump_targets(&bytecode);
        // This should fail - target 1000 is out of 4-byte bytecode
        assert!(!result.is_valid);
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].target, 1000);
    }

    #[test]
    fn test_valid_jump_target() {
        // JUMP to offset 3 (valid) using fixed u16 encoding
        let bytecode = vec![
            opcodes::JUMP,
            0x03, 0x00, // 3 in little-endian u16
            opcodes::HALT, // at offset 3 - valid target
        ];
        let result = verify_jump_targets(&bytecode);
        assert!(result.is_valid);
        assert_eq!(result.jump_count, 1);
    }

    #[test]
    fn test_valid_jump_edge_case() {
        // JUMP to last valid offset
        let bytecode = vec![
            opcodes::JUMP,
            0x03, 0x00, // 3 in little-endian u16 - points to the HALT
            opcodes::HALT, // at offset 3
        ];
        let result = verify_jump_targets(&bytecode);
        assert!(result.is_valid);
    }
}
