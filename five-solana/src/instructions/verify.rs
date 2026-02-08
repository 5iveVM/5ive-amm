use pinocchio::{
    ProgramResult, program_error::ProgramError,
};

use five_protocol::{
    opcodes::{self},
    parser::{parse_header, parse_instruction, ParseError},
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
    let (header, mut offset) = match parse_header(bytecode) {
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

    // Iterate and verify all instructions
    while offset < bytecode.len() {
        match parse_instruction(bytecode, offset) {
            Ok((inst, size)) => {
                // Additional Semantic Checks

                // Check CALL targets (Internal, External)
                if matches!(inst.opcode, opcodes::CALL | opcodes::CALL_EXTERNAL) {
                    // For CALL, arg1 is the function address (absolute offset)
                    // For CALL_EXTERNAL, arg1 bits 0-23 contain the function offset in external script
                    // We only validate internal targets here.
                    if inst.opcode == opcodes::CALL {
                        let func_addr = inst.arg1 as usize;
                        if func_addr >= bytecode.len() {
                            return Err(ProgramError::Custom(8122));
                        }
                    }
                }

                // Check JUMP target bounds
                if matches!(inst.opcode, 
                    opcodes::JUMP | opcodes::JUMP_IF | opcodes::JUMP_IF_NOT |
                    opcodes::EQ_ZERO_JUMP | opcodes::GT_ZERO_JUMP | opcodes::LT_ZERO_JUMP
                ) {
                    let target = inst.arg1 as usize;
                    if target >= bytecode.len() {
                        return Err(ProgramError::Custom(8122));
                    }
                }

                // Check FunctionIndex bounds (if used by other instructions like CALL_INDIRECT if it existed)
                // Current protocol primarily uses CALL (Internal) or CALL_EXTERNAL

                // Note: parse_instruction handles PUSH_STRING_LITERAL and PUSH_STRING bounds checks
                // by using the correct ArgType and skipping bytes.

                offset += size;
            }
            Err(e) => {
                // If it's InvalidOpcode, we can try to return the opcode as error for compatibility
                if e == ParseError::InvalidOpcode {
                    let opcode = bytecode[offset];
                    return Err(ProgramError::Custom(opcode as u32));
                }
                return Err(map_parse_error(e));
            }
        }
    }

    Ok(())
}

/// Validate function name metadata format (if present)
#[allow(dead_code)]
pub fn validate_function_metadata(bytecode: &[u8]) -> ProgramResult {
    // This logic is now encapsulated in parse_header.
    // Calling verify_bytecode_content implicitly validates metadata via parse_header.
    // If explicit separate check is needed, we can use parse_header.

    match parse_header(bytecode) {
        Ok(_) => Ok(()),
        Err(e) => Err(map_parse_error(e)),
    }
}
