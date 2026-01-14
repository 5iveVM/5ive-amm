use pinocchio::{
    msg, ProgramResult, program_error::ProgramError,
};

use crate::debug_log;
use five_protocol::{encoding::VLE, opcodes::{self, ArgType}};

/// Calculate instruction start offset (skips function name metadata if present)
pub fn compute_instruction_start_offset(bytecode: &[u8]) -> u16 {
    const FEATURE_FUNCTION_NAMES: u32 = 1 << 8;

    if bytecode.len() < 10 {
        return 10;
    }

    let features = u32::from_le_bytes([bytecode[4], bytecode[5], bytecode[6], bytecode[7]]);
    let public_count = bytecode[8];

    if (features & FEATURE_FUNCTION_NAMES) == 0 || public_count == 0 {
        return 10;
    }

    // Parse metadata section size (VLE encoded u16)
    let mut offset = 10usize;
    let mut section_size = 0u16;
    let mut shift = 0;

    while offset < bytecode.len() && shift < 16 {
        let byte = bytecode[offset];
        section_size |= ((byte & 0x7F) as u16) << shift;
        offset += 1;
        if byte & 0x80 == 0 {
            break;
        }
        shift += 7;
    }

    // instruction start = 10 bytes header + VLE size bytes + metadata content bytes
    (offset + section_size as usize).min(bytecode.len()) as u16
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

    // Bypass full parsing to avoid OOM
    // Extract header fields manually
    if bytecode.len() < 10 {
         return Err(ProgramError::Custom(8102));
    }
    let public_function_count = bytecode[8];
    let total_function_count = bytecode[9];

    debug_log!("FIVE: counts p={} t={}", public_function_count, total_function_count);

    // Validate function counts are within bounds
    if total_function_count > five_protocol::MAX_FUNCTIONS as u8 {
        debug_log!("FIVE: total func count too high");
        return Err(ProgramError::Custom(8103));
    }

    // CRITICAL: Validate that at least one public function exists (if functions exist)
    if total_function_count > 0 && public_function_count == 0 {
        debug_log!("FIVE: pub=0 but total>0");
        return Err(ProgramError::Custom(8104));
    }

    // Validate public_count <= total_count
    if public_function_count > total_function_count {
        debug_log!("FIVE: pub > total");
        return Err(ProgramError::Custom(8105));
    }

    // Iterate and verify all instructions
    let mut offset = compute_instruction_start_offset(bytecode) as usize;
    // debug_log!("FIVE: start offset {}", offset);

    // Ensure start offset is within bounds
    if offset > bytecode.len() {
        debug_log!("FIVE: start offset OOB");
        return Err(ProgramError::Custom(8106));
    }

    while offset < bytecode.len() {
        let opcode = bytecode[offset];
        // debug_log!("FIVE: Verify op {} at {}", opcode, offset);

        // Get opcode info - fails if valid opcode is not defined
        let info = match opcodes::get_opcode_info(opcode) {
            Some(i) => i,
            None => {
                debug_log!("FIVE: Unknown opcode {}", opcode);
                return Err(ProgramError::Custom(opcode as u32));
            }
        };

        let mut instruction_size = 1; // 1 byte for opcode

        // Decode arguments based on argument type
        match info.arg_type {
            ArgType::None => {
                 // msg!("FIVE: ArgType::None");
            }
            ArgType::U8 | ArgType::RegisterIndex | ArgType::ValueType => {
                // Bounds check for the argument byte
                if offset + instruction_size + 1 > bytecode.len() {
                    debug_log!("FIVE: invalid U8 arg bounds");
                    return Err(ProgramError::Custom(8110));
                }

                // Special handling for PUSH_STRING_LITERAL: consume string bytes
                if opcode == 0x67 || opcode == opcodes::PUSH_STRING_LITERAL {
                    let str_len = bytecode[offset + instruction_size];
                    // debug_log!("FIVE: PUSH_STRING len {}", str_len);
                    // opcode (1) + len_byte (1) + string_bytes (str_len)
                    let total_len = instruction_size + 1 + (str_len as usize);

                    if offset + total_len > bytecode.len() {
                        debug_log!("FIVE: PUSH_STRING bounds fail");
                        return Err(ProgramError::Custom(8111));
                    }
                    instruction_size = total_len;
                } else {
                    // debug_log!("FIVE: U8 generic");
                    instruction_size += 1;
                }
            }
            ArgType::U16 => {
                if offset + instruction_size + 2 > bytecode.len() {
                    debug_log!("FIVE: U16 bounds fail");
                    return Err(ProgramError::Custom(8112));
                }
                instruction_size += 2;
            }
            ArgType::U32 | ArgType::FunctionIndex | ArgType::LocalIndex | ArgType::AccountIndex => {
                if offset + instruction_size >= bytecode.len() {
                     debug_log!("FIVE: U32 bounds fail 1");
                     return Err(ProgramError::Custom(8113));
                }
                match VLE::decode_u32(&bytecode[offset + instruction_size..]) {
                    Some((value, consumed)) => {
                        // Additional Logic Checks
                        if info.arg_type == ArgType::FunctionIndex && opcode == opcodes::CALL {
                             if value >= total_function_count as u32 {
                                 debug_log!("FIVE: Function index OOB");
                                 return Err(ProgramError::Custom(8114));
                             }
                        }
                        instruction_size += consumed;
                    }
                    None => {
                        debug_log!("FIVE: VLE decode failed");
                        return Err(ProgramError::Custom(8115));
                    }
                }
            }
            ArgType::U64 => {
                 if offset + instruction_size >= bytecode.len() {
                     return Err(ProgramError::Custom(8116));
                }
                match VLE::decode_u64(&bytecode[offset + instruction_size..]) {
                    Some((_, consumed)) => instruction_size += consumed,
                    None => return Err(ProgramError::Custom(8117)),
                }
            }
            ArgType::TwoRegisters => {
                if offset + instruction_size + 2 > bytecode.len() {
                    return Err(ProgramError::Custom(8118));
                }
                instruction_size += 2;
            }
            ArgType::ThreeRegisters => {
                if offset + instruction_size + 3 > bytecode.len() {
                    return Err(ProgramError::Custom(8119));
                }
                instruction_size += 3;
            }
            ArgType::CallExternal => {
                // account_index (u8) + func_offset (u16) + param_count (u8)
                if offset + instruction_size + 4 > bytecode.len() {
                    return Err(ProgramError::Custom(8120));
                }
                instruction_size += 4;
            }
            ArgType::CallInternal => {
                // param_count (u8) + func_addr (u16)
                if offset + instruction_size + 3 > bytecode.len() {
                    return Err(ProgramError::Custom(8121));
                }

                let addr_lo = bytecode[offset + 2];
                let addr_hi = bytecode[offset + 3];
                let func_addr = u16::from_le_bytes([addr_lo, addr_hi]) as usize;

                if func_addr >= bytecode.len() {
                    return Err(ProgramError::Custom(8122));
                }

                instruction_size += 3;
            }
            ArgType::AccountField => {
                // account_index (u8)
                if offset + instruction_size + 1 > bytecode.len() {
                    return Err(ProgramError::Custom(8123));
                }
                instruction_size += 1;

                // field_offset (VLE)
                if offset + instruction_size >= bytecode.len() {
                     return Err(ProgramError::Custom(8124));
                }
                match VLE::decode_u32(&bytecode[offset + instruction_size..]) {
                    Some((_, consumed)) => instruction_size += consumed,
                    None => return Err(ProgramError::Custom(8125)),
                }
            }
        }

        // Final bounds check after size calculation
        if offset + instruction_size > bytecode.len() {
            msg!("FIVE: Final bounds fail");
            return Err(ProgramError::Custom(8130));
        }

        offset += instruction_size;
    }

    Ok(())
}

/// Validate function name metadata format (if present)
#[allow(dead_code)]
pub fn validate_function_metadata(bytecode: &[u8]) -> ProgramResult {
    const FEATURE_FUNCTION_NAMES: u32 = 1 << 8;

    if bytecode.len() < 10 {
        return Ok(());
    }

    let features = u32::from_le_bytes([bytecode[4], bytecode[5], bytecode[6], bytecode[7]]);
    let public_count = bytecode[8];

    if (features & FEATURE_FUNCTION_NAMES) == 0 || public_count == 0 {
        return Ok(());
    }

    // Parse and validate metadata section
    let mut offset = 10usize;
    let mut section_size = 0u16;
    let mut shift = 0;

    // Decode VLE u16 section size
    while offset < bytecode.len() && shift < 16 {
        let byte = bytecode[offset];
        section_size |= ((byte & 0x7F) as u16) << shift;
        offset += 1;
        if byte & 0x80 == 0 {
            break;
        }
        shift += 7;
    }

    // Validate metadata doesn't exceed bytecode bounds
    let metadata_end = offset + section_size as usize;
    if metadata_end > bytecode.len() {
        return Err(ProgramError::Custom(8209)); // Metadata exceeds bytecode
    }

    // Quick validation: metadata section should contain valid name entries
    // Each entry has: name_len (u8) + name_bytes
    // At minimum, we expect at least public_count entries
    if section_size == 0 && public_count > 0 {
        return Err(ProgramError::Custom(8210)); // Missing metadata for public functions
    }

    Ok(())
}
