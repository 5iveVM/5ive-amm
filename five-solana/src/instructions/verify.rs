use pinocchio::{
    ProgramResult, program_error::ProgramError,
};

use five_protocol::{
    opcodes::{self, get_opcode_info, operand_size},
    parser::{parse_code_bounds, ParseError},
};

fn map_parse_error(e: ParseError) -> ProgramError {
    match e {
        ParseError::HeaderTooShort => ProgramError::Custom(8102),
        ParseError::InvalidMagic => ProgramError::Custom(8003),
        ParseError::InstructionOutOfBounds => ProgramError::Custom(8130),
        ParseError::InvalidOpcode => ProgramError::Custom(8107), // Generic invalid opcode
        ParseError::CallTargetOutOfBounds => ProgramError::Custom(8122),
        ParseError::InvalidFunctionCount => ProgramError::Custom(8103),
        ParseError::BytecodeTooShort => ProgramError::Custom(8130),
    }
}

/// Verify bytecode content before deployment.
pub fn verify_bytecode_content(bytecode: &[u8]) -> ProgramResult {
    // Validate bytecode size
    if bytecode.len() > five_protocol::MAX_SCRIPT_SIZE {
        return Err(ProgramError::Custom(8101));
    }

    // Use shared parser to validate header and get start offset
    let (header, mut offset, code_end) = match parse_code_bounds(bytecode) {
        Ok(res) => res,
        Err(e) => {
            return Err(map_parse_error(e));
        }
    };

    // Validate that at least one public function exists (if functions exist)
    if header.total_function_count > 0 && header.public_function_count == 0 {
        return Err(ProgramError::Custom(8104));
    }

    // Validate public_count <= total_count
    if header.public_function_count > header.total_function_count {
        return Err(ProgramError::Custom(8105));
    }

    // Ensure start offset is within bounds
    if offset > bytecode.len() {
        return Err(ProgramError::Custom(8106));
    }

    let pool_enabled = (header.features & five_protocol::FEATURE_CONSTANT_POOL) != 0;

    // Iterate and verify all instructions using a minimal decode path.
    // We only decode immediate operands for opcodes that need semantic target checks.
    while offset < code_end {
        let opcode = bytecode[offset];
        if get_opcode_info(opcode).is_none() {
            // Preserve existing compatibility behavior: return opcode as custom error.
            return Err(ProgramError::Custom(opcode as u32));
        }

        let remaining = if offset + 1 <= bytecode.len() {
            &bytecode[offset + 1..]
        } else {
            &[]
        };
        let operand_bytes = operand_size(opcode, remaining, pool_enabled)
            .ok_or(ProgramError::Custom(8130))?;
        let next = offset
            .checked_add(1 + operand_bytes)
            .ok_or(ProgramError::Custom(8130))?;
        if next > code_end {
            return Err(ProgramError::Custom(8130));
        }

        // CALL param_count(u8) + function_address(u16)
        if opcode == opcodes::CALL {
            let func_addr = u16::from_le_bytes([bytecode[offset + 2], bytecode[offset + 3]]) as usize;
            if func_addr >= code_end {
                return Err(ProgramError::Custom(8122));
            }
        }

        // Absolute jump targets encoded as u16.
        if matches!(
            opcode,
            opcodes::JUMP
                | opcodes::JUMP_IF
                | opcodes::JUMP_IF_NOT
                | opcodes::EQ_ZERO_JUMP
                | opcodes::GT_ZERO_JUMP
                | opcodes::LT_ZERO_JUMP
                | opcodes::DEC_JUMP_NZ
        ) {
            let target = u16::from_le_bytes([bytecode[offset + 1], bytecode[offset + 2]]) as usize;
            if target >= code_end {
                return Err(ProgramError::Custom(8122));
            }
        }

        // DEC_LOCAL_JUMP_NZ local_index(u8) + target(u16)
        if opcode == opcodes::DEC_LOCAL_JUMP_NZ {
            let target = u16::from_le_bytes([bytecode[offset + 2], bytecode[offset + 3]]) as usize;
            if target >= code_end {
                return Err(ProgramError::Custom(8122));
            }
        }

        // CMP_EQ_JUMP compare(u8) + target(u16)
        if opcode == opcodes::CMP_EQ_JUMP {
            let target = u16::from_le_bytes([bytecode[offset + 2], bytecode[offset + 3]]) as usize;
            if target >= code_end {
                return Err(ProgramError::Custom(8122));
            }
        }

        offset = next;
    }

    Ok(())
}

/// Validate function name metadata format (if present)
#[allow(dead_code)]
pub fn validate_function_metadata(bytecode: &[u8]) -> ProgramResult {
    // This logic is now encapsulated in parse_header.
    // Calling verify_bytecode_content implicitly validates metadata via parse_header.
    // If explicit separate check is needed, we can use parse_header.

    match parse_code_bounds(bytecode) {
        Ok(_) => Ok(()),
        Err(e) => Err(map_parse_error(e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use five_protocol::opcodes;

    fn header(public_count: u8, total_count: u8) -> Vec<u8> {
        let mut out = Vec::with_capacity(10);
        out.extend_from_slice(b"5IVE");
        out.extend_from_slice(&0u32.to_le_bytes()); // features
        out.push(public_count);
        out.push(total_count);
        out
    }

    #[test]
    fn verify_accepts_minimal_halt_script() {
        let mut bytecode = header(1, 1);
        bytecode.push(opcodes::HALT);
        assert!(verify_bytecode_content(&bytecode).is_ok());
    }

    #[test]
    fn verify_rejects_invalid_opcode_with_opcode_code() {
        let mut bytecode = header(1, 1);
        bytecode.push(0x68); // currently unassigned
        let err = verify_bytecode_content(&bytecode).unwrap_err();
        assert_eq!(err, ProgramError::Custom(0x68));
    }

    #[test]
    fn verify_rejects_call_target_out_of_bounds() {
        let mut bytecode = header(1, 1);
        bytecode.push(opcodes::CALL);
        bytecode.push(0); // param_count
        bytecode.extend_from_slice(&(0x7FFFu16).to_le_bytes()); // bad target
        let err = verify_bytecode_content(&bytecode).unwrap_err();
        assert_eq!(err, ProgramError::Custom(8122));
    }

    #[test]
    fn verify_rejects_jump_target_out_of_bounds() {
        let mut bytecode = header(1, 1);
        bytecode.push(opcodes::JUMP);
        bytecode.extend_from_slice(&(0x7FFFu16).to_le_bytes()); // bad target
        let err = verify_bytecode_content(&bytecode).unwrap_err();
        assert_eq!(err, ProgramError::Custom(8122));
    }

    #[test]
    fn verify_rejects_truncated_instruction() {
        let mut bytecode = header(1, 1);
        bytecode.push(opcodes::PUSH_U64); // needs 8 bytes
        let err = verify_bytecode_content(&bytecode).unwrap_err();
        assert_eq!(err, ProgramError::Custom(8130));
    }
}
