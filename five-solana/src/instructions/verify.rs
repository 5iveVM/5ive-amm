use pinocchio::{
    ProgramResult, program_error::ProgramError,
};

use crate::debug_log;
use five_protocol::{
    opcodes::{self},
    parser::{parse_header, parse_instruction, ParseError},
};

fn map_parse_error(e: ParseError) -> ProgramError {
    match e {
        ParseError::HeaderTooShort => ProgramError::Custom(8102),
        ParseError::InvalidMagic => ProgramError::Custom(8003),
        ParseError::InvalidFunctionCount => ProgramError::Custom(8103),
        ParseError::InstructionOutOfBounds => ProgramError::Custom(8130),
        ParseError::InvalidOpcode => ProgramError::Custom(8107), // Generic invalid opcode
        ParseError::CallTargetOutOfBounds => ProgramError::Custom(8122),
        ParseError::InvalidVLE => ProgramError::Custom(8115),
        ParseError::BytecodeTooShort => ProgramError::Custom(8130),
    }
}

/// Verify bytecode content before deployment
///
/// **Deploy-Time Verification Strategy:**
/// This function performs comprehensive verification of bytecode, enabling
/// trust-based execution at runtime without re-verification:
/// - Header format is valid (magic, features, counts)
/// - All instructions are valid opcodes with proper bounds and arguments
/// - CALL instructions target valid function indices
/// - No incomplete instructions
pub fn verify_bytecode_content(bytecode: &[u8]) -> ProgramResult {
    debug_log!("FIVE: verify_bytecode entry len={}", bytecode.len());
    // Validate bytecode size
    if bytecode.len() > five_protocol::MAX_SCRIPT_SIZE {
        debug_log!("FIVE: bytecode too large");
        return Err(ProgramError::Custom(8101));
    }

    // Use shared parser to validate header and get start offset
    let (header, mut offset) = match parse_header(bytecode) {
        Ok(res) => res,
        Err(e) => {
            debug_log!("FIVE: header parse error: {}", e.message());
            return Err(map_parse_error(e));
        }
    };

    debug_log!("FIVE: counts p={} t={}", header.public_function_count, header.total_function_count);

    // CRITICAL: Validate that at least one public function exists (if functions exist)
    if header.total_function_count > 0 && header.public_function_count == 0 {
        debug_log!("FIVE: pub=0 but total>0");
        return Err(ProgramError::Custom(8104));
    }

    // Validate public_count <= total_count
    if header.public_function_count > header.total_function_count {
        debug_log!("FIVE: pub > total");
        return Err(ProgramError::Custom(8105));
    }

    // Ensure start offset is within bounds
    if offset > bytecode.len() {
        debug_log!("FIVE: start offset OOB");
        return Err(ProgramError::Custom(8106));
    }

    // Iterate and verify all instructions
    while offset < bytecode.len() {
        match parse_instruction(bytecode, offset) {
            Ok((inst, size)) => {
                // Additional Semantic Checks

                // Check CALL target bounds
                if inst.opcode == opcodes::CALL {
                    // For CallInternal, arg1 is the function address (offset)
                    let func_addr = inst.arg1 as usize;
                    if func_addr >= bytecode.len() {
                        debug_log!("FIVE: CALL target OOB");
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
                debug_log!("FIVE: Instruction parse error at {}: {}", offset, e.message());
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
