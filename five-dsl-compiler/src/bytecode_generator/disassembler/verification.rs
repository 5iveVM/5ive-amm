//! Bytecode Verification Module
//!
//! This module provides verification utilities for compiled bytecode,
//! specifically to detect invalid JUMP targets that can cause runtime errors.
//!
//! The primary use case is catching bytecode structure or patching bugs where
//! JUMP offsets become invalid.

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

    let (features, start_offset) = match five_protocol::parse_header(bytecode) {
        Ok((header, start)) => (header.features, start),
        Err(_) => (0, 0),
    };
    let pool_enabled = (features & five_protocol::FEATURE_CONSTANT_POOL) != 0;
    let mut offset = start_offset;

    // If constant pool is enabled, cap scan length at the end of the code section
    let mut scan_len = bytecode_len;
    if pool_enabled {
        let metadata_end = find_instructions_start(bytecode);
        let desc_size = core::mem::size_of::<five_protocol::ConstantPoolDescriptor>();
        if metadata_end + desc_size <= bytecode_len {
            let base = metadata_end;
            let pool_offset = u32::from_le_bytes([
                bytecode[base],
                bytecode[base + 1],
                bytecode[base + 2],
                bytecode[base + 3],
            ]) as usize;
            let string_blob_offset = u32::from_le_bytes([
                bytecode[base + 4],
                bytecode[base + 5],
                bytecode[base + 6],
                bytecode[base + 7],
            ]) as usize;
            let string_blob_len = u32::from_le_bytes([
                bytecode[base + 8],
                bytecode[base + 9],
                bytecode[base + 10],
                bytecode[base + 11],
            ]) as usize;
            let pool_slots = u16::from_le_bytes([bytecode[base + 12], bytecode[base + 13]]) as usize;
            let code_offset = pool_offset + pool_slots * 8;
            let code_end = if string_blob_len > 0 {
                string_blob_offset
            } else {
                string_blob_offset.max(code_offset)
            };
            if code_end > 0 && code_end <= bytecode_len {
                scan_len = code_end;
            }
        }
    }
    
    while offset < scan_len {
        let opcode = bytecode[offset];

        match opcode {
            // JUMP instructions use fixed u16 offset
            opcodes::JUMP | opcodes::JUMP_IF | opcodes::JUMP_IF_NOT => {
                jump_count += 1;
                
                // Need at least 2 more bytes for the u16 target
                if offset + 2 >= scan_len {
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
                if target as usize >= scan_len {
                    errors.push(VerificationError {
                        offset,
                        opcode,
                        opcode_name: opcode_name(opcode),
                        target,
                        reason: format!(
                            "Out of bounds: target {} >= bytecode length {} ({}% overflow)",
                            target,
                            scan_len,
                            (target as usize * 100) / scan_len
                        ),
                    });
                }

                offset += 3; // opcode + u16
            }

            // CALL instruction: param_count(u8) + function_address(u16 fixed)
            opcodes::CALL => {
                jump_count += 1;

                if offset + 4 > scan_len {
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

                if target as usize >= scan_len {
                    errors.push(VerificationError {
                        offset,
                        opcode,
                        opcode_name: "CALL",
                        target,
                        reason: format!(
                            "Out of bounds: target {} >= bytecode length {} ({}% overflow)",
                            target,
                            scan_len,
                            (target as usize * 100) / scan_len
                        ),
                    });
                }

                // CALL is fixed-width in current bytecode format.
                offset += 4;
            }

            // REQUIRE_EQ_PUBKEY: reject legacy key sentinel offsets (0x3FFF).
            opcodes::REQUIRE_EQ_PUBKEY => {
                if offset + 11 > scan_len {
                    break;
                }
                let offset1 = u32::from_le_bytes([
                    bytecode[offset + 2],
                    bytecode[offset + 3],
                    bytecode[offset + 4],
                    bytecode[offset + 5],
                ]);
                let offset2 = u32::from_le_bytes([
                    bytecode[offset + 7],
                    bytecode[offset + 8],
                    bytecode[offset + 9],
                    bytecode[offset + 10],
                ]);
                if offset1 == 0x3FFF || offset2 == 0x3FFF {
                    errors.push(VerificationError {
                        offset,
                        opcode,
                        opcode_name: "REQUIRE_EQ_PUBKEY",
                        target: 0x3FFF,
                        reason: "Legacy sentinel offset 0x3FFF is unsupported".to_string(),
                    });
                }
                offset += 11;
            }


            // BR_EQ_U8: compare_value(u8) + offset(u16)
            opcodes::BR_EQ_U8 => {
                // Opcode(1) + Val(1) + Offset(2) = 4
                offset += 4;
            }

            // CMP_EQ_JUMP: compare_value(u8) + absolute_target(u16)
            opcodes::CMP_EQ_JUMP => {
                if offset + 4 > scan_len {
                    break;
                }
                let target = u16::from_le_bytes([bytecode[offset + 2], bytecode[offset + 3]]);
                jump_count += 1;
                if target as usize >= scan_len {
                    errors.push(VerificationError {
                        offset,
                        opcode,
                        opcode_name: "CMP_EQ_JUMP",
                        target,
                        reason: format!(
                            "Out of bounds: target {} >= bytecode length {}",
                            target, scan_len
                        ),
                    });
                }
                offset += 4;
            }

            // DEC_JUMP_NZ: absolute_target(u16)
            opcodes::DEC_JUMP_NZ => {
                if offset + 3 > scan_len {
                    break;
                }
                let target = u16::from_le_bytes([bytecode[offset + 1], bytecode[offset + 2]]);
                jump_count += 1;
                if target as usize >= scan_len {
                    errors.push(VerificationError {
                        offset,
                        opcode,
                        opcode_name: "DEC_JUMP_NZ",
                        target,
                        reason: format!(
                            "Out of bounds: target {} >= bytecode length {}",
                            target, scan_len
                        ),
                    });
                }
                offset += 3;
            }

            // CALL_EXTERNAL: account_index(1) + offset(2) + param_count(1)
            opcodes::CALL_EXTERNAL => {
                offset += 5;
            }

            // Handle other opcodes - use get_operand_size
            _ => {
                let remaining = bytecode.get(offset + 1..).unwrap_or(&[]);
                let operand_size = opcodes::operand_size(opcode, remaining, pool_enabled).unwrap_or(0);
                offset += 1 + operand_size;
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
        // Format: [u16 section_size] [u8 name_count] [u8 name_len, bytes...]*
        if offset + 2 <= bytes.len() {
            let section_size = u16::from_le_bytes([bytes[offset], bytes[offset+1]]);
            offset += 2 + section_size as usize;
        }
    }

    offset.min(bytes.len())
}

/// Get human-readable opcode name
fn opcode_name(opcode: u8) -> &'static str {
    opcodes::opcode_name(opcode)
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

    #[test]
    fn test_rejects_legacy_pubkey_sentinel() {
        let bytecode = vec![
            opcodes::REQUIRE_EQ_PUBKEY,
            1, // acc1
            0x00, 0x00, 0x00, 0x00, // offset1
            2, // acc2
            0xFF, 0x3F, 0x00, 0x00, // offset2 = 0x3FFF (legacy sentinel)
            opcodes::HALT,
        ];
        let result = verify_jump_targets(&bytecode);
        assert!(!result.is_valid);
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].opcode_name, "REQUIRE_EQ_PUBKEY");
        assert_eq!(result.errors[0].target, 0x3FFF);
    }

    #[test]
    fn test_call_is_fixed_width_for_scanning() {
        let bytecode = vec![
            opcodes::CALL,
            0x00,       // param_count
            0x09, 0x00, // target=9
            0xFF, 0x3F, // metadata-like bytes that must be treated as payload bytes
            opcodes::JUMP_IF,
            0x09, 0x00, // valid jump target
            opcodes::HALT,
        ];
        let result = verify_jump_targets(&bytecode);
        assert!(result.is_valid, "{}", result.error_summary());
    }
}
