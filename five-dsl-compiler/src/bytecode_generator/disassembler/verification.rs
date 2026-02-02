//! Bytecode Verification Module
//!
//! This module provides verification utilities for compiled bytecode,
//! specifically to detect invalid JUMP targets that can cause runtime errors.
//!
//! The primary use case is catching register optimization bugs where bytecode
//! structure changes cause JUMP offsets to become invalid.

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
///
/// # Example
/// ```ignore
/// let result = verify_jump_targets(&bytecode);
/// if !result.is_valid {
///     eprintln!("{}", result.error_summary());
/// }
/// ```
pub fn verify_jump_targets(bytecode: &[u8]) -> VerificationResult {
    let mut errors = Vec::new();
    let mut jump_count = 0;
    let bytecode_len = bytecode.len();

    // Use the same instructions start logic as BytecodeInspector
    let mut offset = find_instructions_start(bytecode);
    
    while offset < bytecode_len {
        let opcode = bytecode[offset];

        match opcode {
            // JUMP instructions use fixed u16 offset (despite protocol comment saying VLE)
            // The VM's control_flow.rs uses ctx.fetch_u16() for JUMP/JUMP_IF/JUMP_IF_NOT
            opcodes::JUMP | opcodes::JUMP_IF | opcodes::JUMP_IF_NOT => {
                jump_count += 1;
                
                // Need at least 2 more bytes for the u16 target
                if offset + 2 >= bytecode_len {
                    errors.push(VerificationError {
                        offset,
                        opcode,
                        opcode_name: opcode_name(opcode),
                        target: 0,
                        reason: "Truncated: missing jump offset bytes".to_string(),
                    });
                    break;
                }

                // Read u16 target (little-endian)
                let target = u16::from_le_bytes([bytecode[offset + 1], bytecode[offset + 2]]);

                // Validate target is within bytecode bounds
                if target as usize >= bytecode_len {
                    errors.push(VerificationError {
                        offset,
                        opcode,
                        opcode_name: opcode_name(opcode),
                        target,
                        reason: format!(
                            "Out of bounds: target {} >= bytecode length {} ({}% overflow)",
                            target,
                            bytecode_len,
                            (target as usize * 100) / bytecode_len
                        ),
                    });
                }

                offset += 3; // opcode + u16
            }

            // CALL instruction: param_count(u8) + function_address(u16 fixed)
            opcodes::CALL => {
                jump_count += 1;

                if offset + 4 > bytecode_len {
                    errors.push(VerificationError {
                        offset,
                        opcode,
                        opcode_name: "CALL",
                        target: 0,
                        reason: "Truncated: missing CALL operands".to_string(),
                    });
                    break;
                }

                // Skip param_count (1 byte), read function_address (2 bytes, fixed u16)
                let target = u16::from_le_bytes([bytecode[offset + 2], bytecode[offset + 3]]);

                if target as usize >= bytecode_len {
                    errors.push(VerificationError {
                        offset,
                        opcode,
                        opcode_name: "CALL",
                        target,
                        reason: format!(
                            "Out of bounds: target {} >= bytecode length {} ({}% overflow)",
                            target,
                            bytecode_len,
                            (target as usize * 100) / bytecode_len
                        ),
                    });
                }

                // Use call_size helper to properly skip the CALL with potential metadata
                offset += super::call_decoder::call_size(bytecode, offset);
            }

            // CALL_REG instruction (register mode): function_address(u16 fixed)
            // Format: opcode(1) + function_address(u16)
            opcodes::CALL_REG => {
                jump_count += 1;

                if offset + 3 > bytecode_len {
                    errors.push(VerificationError {
                        offset,
                        opcode,
                        opcode_name: "CALL_REG",
                        target: 0,
                        reason: "Truncated: missing CALL_REG function address".to_string(),
                    });
                    break;
                }

                // Read function_address (2 bytes, fixed u16)
                let target = u16::from_le_bytes([bytecode[offset + 1], bytecode[offset + 2]]);

                if target as usize >= bytecode_len {
                    errors.push(VerificationError {
                        offset,
                        opcode,
                        opcode_name: "CALL_REG",
                        target,
                        reason: format!(
                            "Out of bounds: target {} >= bytecode length {} ({}% overflow)",
                            target,
                            bytecode_len,
                            (target as usize * 100) / bytecode_len
                        ),
                    });
                }

                offset += 3; // opcode + u16
            }

            // BR_EQ_U8: compare_value(u8) + vle_offset
            opcodes::BR_EQ_U8 => {
                jump_count += 1;
                // Skip: opcode(1) + compare_value(1) + VLE offset
                if offset + 2 < bytecode_len {
                    let vle_size = decode_vle_size(&bytecode[offset + 2..], 9);
                    offset += 2 + vle_size;
                } else {
                    break;
                }
            }

            // CALL_EXTERNAL: account_index(1) + offset(2) + param_count(1)
            opcodes::CALL_EXTERNAL => {
                offset += 5;
            }

            // Handle other opcodes - use get_operand_size
            _ => {
                offset += 1 + get_operand_size(opcode, bytecode.get(offset + 1..).unwrap_or(&[]));
            }
        }
    }

    VerificationResult::with_errors(errors, jump_count, bytecode_len)
}

/// Find where instructions start by skipping header and metadata
/// (Same logic as BytecodeInspector::find_instructions_start)
fn find_instructions_start(bytes: &[u8]) -> usize {
    // Check for 5IVE magic at start (was STKS in older versions)
    if bytes.len() < 10 || &bytes[0..4] != b"5IVE" {
        // No header - raw bytecode starts at 0
        return 0;
    }

    // Check for FEATURE_FUNCTION_NAMES at offset [4..8]
    let features = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);

    const FEATURE_FUNCTION_NAMES: u32 = 1 << 8;

    let mut offset = 10; // After header

    // If metadata is present, skip it
    if (features & FEATURE_FUNCTION_NAMES) != 0 && offset < bytes.len() {
        // Skip metadata section
        // Format: [VLE u16 section_size] [u8 name_count] [u8 name_len, bytes...]*
        if let Some((section_size, bytes_read)) = decode_vle_u16_simple(&bytes[offset..]) {
            offset += bytes_read + section_size as usize;
        }
    }

    offset.min(bytes.len())
}

/// Simple VLE u16 decoder for header parsing
fn decode_vle_u16_simple(bytes: &[u8]) -> Option<(u16, usize)> {
    if bytes.is_empty() {
        return None;
    }

    let first = bytes[0];
    if first < 128 {
        Some((first as u16, 1))
    } else if bytes.len() >= 2 {
        let value = ((first & 0x7F) as u16) | ((bytes[1] as u16) << 7);
        Some((value, 2))
    } else {
        None
    }
}

/// Decode VLE u128 (same as in decoder.rs but local for verification)
fn decode_vle_u128(bytes: &[u8]) -> Option<(u128, usize)> {
    let mut value: u128 = 0;
    let mut shift = 0;
    for (i, &b) in bytes.iter().enumerate() {
        value |= ((b & 0x7F) as u128) << shift;
        if b & 0x80 == 0 {
            return Some((value, i + 1));
        }
        shift += 7;
        if shift > 127 {
            break;
        }
    }
    // If we got here without a terminator, return what we have
    if !bytes.is_empty() {
        Some((value, bytes.len()))
    } else {
        None
    }
}

/// Get human-readable opcode name
fn opcode_name(opcode: u8) -> &'static str {
    match opcode {
        opcodes::JUMP => "JUMP",
        opcodes::JUMP_IF => "JUMP_IF",
        opcodes::JUMP_IF_NOT => "JUMP_IF_NOT",
        opcodes::CALL => "CALL",
        opcodes::BR_EQ_U8 => "BR_EQ_U8",
        opcodes::CALL_EXTERNAL => "CALL_EXTERNAL",
        _ => "UNKNOWN",
    }
}

/// Get operand size for an opcode (simplified for verification purposes)
fn get_operand_size(opcode: u8, remaining: &[u8]) -> usize {
    match opcode {
        // No operands
        opcodes::HALT | opcodes::RETURN | opcodes::RETURN_VALUE |
        opcodes::POP | opcodes::DUP | opcodes::DUP2 | opcodes::SWAP |
        opcodes::ADD | opcodes::SUB | opcodes::MUL | opcodes::DIV | opcodes::MOD |
        opcodes::EQ | opcodes::NEQ | opcodes::GT | opcodes::LT | opcodes::GTE | opcodes::LTE |
        opcodes::AND | opcodes::OR | opcodes::NOT | opcodes::REQUIRE | opcodes::ASSERT |
        opcodes::ADD_CHECKED | opcodes::SUB_CHECKED | opcodes::MUL_CHECKED |
        opcodes::PUSH_ZERO | opcodes::PUSH_ONE | opcodes::PUSH_0 | opcodes::PUSH_1 |
        opcodes::PUSH_2 | opcodes::PUSH_3 |
        opcodes::GET_LOCAL_0 | opcodes::GET_LOCAL_1 | opcodes::GET_LOCAL_2 | opcodes::GET_LOCAL_3 |
        opcodes::SET_LOCAL_0 | opcodes::SET_LOCAL_1 | opcodes::SET_LOCAL_2 | opcodes::SET_LOCAL_3 |
        opcodes::LOAD_PARAM_0 | opcodes::LOAD_PARAM_1 | opcodes::LOAD_PARAM_2 | opcodes::LOAD_PARAM_3 => 0,

        // Single byte operands
        opcodes::PUSH_U8 | opcodes::PUSH_BOOL |
        opcodes::CHECK_SIGNER | opcodes::CHECK_WRITABLE | opcodes::CHECK_INITIALIZED |
        opcodes::CHECK_UNINITIALIZED |
        opcodes::SET_LOCAL | opcodes::GET_LOCAL | opcodes::LOAD_PARAM | opcodes::STORE_PARAM |
        opcodes::ALLOC_LOCALS | opcodes::DEALLOC_LOCALS |
        opcodes::PUSH_REG | opcodes::POP_REG | opcodes::CLEAR_REG | opcodes::CAST |
        opcodes::CREATE_ARRAY | opcodes::PUSH_ARRAY_LITERAL |
        // Account opcodes taking 1 byte account index
        opcodes::LOAD_ACCOUNT | opcodes::SAVE_ACCOUNT | opcodes::GET_ACCOUNT | 
        opcodes::GET_LAMPORTS | opcodes::SET_LAMPORTS | opcodes::GET_DATA | 
        opcodes::GET_KEY | opcodes::GET_OWNER |
        // Fused: single byte operands
        opcodes::CHECK_SIGNER_WRITABLE |  // acc(u8)
        opcodes::REQUIRE_PARAM_GT_ZERO => 1, // param(u8)

        // Two byte operands
        opcodes::COPY_REG |
        // Fused: two u8 operands
        opcodes::REQUIRE_GTE_REG => 2, // src1(u8) + src2(u8)

        // Three byte operands (dest, src1, src2)
        opcodes::ADD_REG | opcodes::SUB_REG | opcodes::MUL_REG | opcodes::DIV_REG |
        opcodes::EQ_REG | opcodes::GT_REG | opcodes::LT_REG => 3,

        // LOAD_REG variants: reg(1) + immediate value
        opcodes::LOAD_REG_U8 => 2,   // reg + u8
        opcodes::LOAD_REG_BOOL => 2, // reg + bool
        opcodes::LOAD_REG_U32 => { // reg + u32 (VLE)
            1 + decode_vle_size(remaining.get(1..).unwrap_or(&[]), 4)
        }
        opcodes::LOAD_REG_U64 => { // reg + u64 (VLE)
            1 + decode_vle_size(remaining.get(1..).unwrap_or(&[]), 9)
        }
        opcodes::LOAD_REG_PUBKEY => 33, // reg + 32 bytes

        // VLE-encoded operands - need to parse VLE to know actual size
        opcodes::PUSH_U16 => decode_vle_size(remaining, 2),
        opcodes::PUSH_U32 => decode_vle_size(remaining, 4),
        opcodes::PUSH_U64 | opcodes::PUSH_I64 => decode_vle_size(remaining, 9),

        // LOAD_FIELD/STORE_FIELD: account_index(1) + field_offset(VLE)
        opcodes::LOAD_FIELD | opcodes::STORE_FIELD | opcodes::LOAD_FIELD_PUBKEY => {
            1 + decode_vle_size(remaining.get(1..).unwrap_or(&[]), 4)
        }

        // Fused opcodes with acc(u8) + offset(VLE) format
        opcodes::REQUIRE_NOT_BOOL |  // acc(u8) offset(VLE)
        opcodes::STORE_FIELD_ZERO => {  // acc(u8) offset(VLE)
            1 + decode_vle_size(remaining.get(1..).unwrap_or(&[]), 4)
        }

        // Fused opcodes with acc(u8) + offset(VLE) + param/acc(u8) format
        opcodes::REQUIRE_GTE_U64 |     // acc(u8) offset(VLE) param(u8)
        opcodes::FIELD_ADD_PARAM |     // acc(u8) offset(VLE) param(u8)
        opcodes::FIELD_SUB_PARAM |     // acc(u8) offset(VLE) param(u8)
        opcodes::STORE_PARAM_TO_FIELD | // acc(u8) offset(VLE) param(u8)
        opcodes::STORE_KEY_TO_FIELD |  // acc(u8) offset(VLE) key_acc(u8)
        opcodes::ADD_FIELD_REG |       // acc(u8) offset(VLE) reg(u8)
        opcodes::SUB_FIELD_REG => {    // acc(u8) offset(VLE) reg(u8)
            2 + decode_vle_size(remaining.get(1..).unwrap_or(&[]), 4)
        }

        // Fused opcodes with reg(u8) + acc(u8) + offset(VLE) format
        opcodes::LOAD_FIELD_REG |      // reg(u8) acc(u8) offset(VLE) 
        opcodes::STORE_FIELD_REG => {  // reg(u8) acc(u8) offset(VLE)
            2 + decode_vle_size(remaining.get(2..).unwrap_or(&[]), 4)
        }

        // Fused opcodes with acc1(u8) + offset1(VLE) + acc2(u8) + offset2(VLE) format
        opcodes::REQUIRE_EQ_PUBKEY |   // acc1(u8) offset1(VLE) acc2(u8) offset2(VLE)
        opcodes::REQUIRE_EQ_FIELDS => {  // acc1(u8) offset1(VLE) acc2(u8) offset2(VLE)
            let vle1_size = decode_vle_size(remaining.get(1..).unwrap_or(&[]), 4);
            let vle2_size = decode_vle_size(remaining.get(2 + vle1_size..).unwrap_or(&[]), 4);
            2 + vle1_size + vle2_size
        }

        // PUSH_PUBKEY: 32 bytes
        opcodes::PUSH_PUBKEY => 32,

        // PUSH_U128: 16 bytes
        opcodes::PUSH_U128 => 16,

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
