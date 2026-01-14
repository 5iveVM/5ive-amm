//! Canonical Parser for Five Bytecode
//!
//! This module provides a shared bytecode parser that interprets the optimized header,
//! VLE-encoded immediates, and validates CALL targets and instruction bounds.
//! Used by both compilers and on-chain verifiers to ensure consistent parsing.

use crate::encoding::VLE;
use crate::opcodes::{get_opcode_info, ArgType};
use crate::OptimizedHeader;
use crate::{FunctionNameEntry, FunctionNameMetadata};
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

/// Parsed bytecode result containing header and instructions with validation
#[derive(Debug, Clone)]
pub struct ParsedBytecode<'a> {
    pub header: OptimizedHeader,
    pub instructions: alloc::vec::Vec<ParsedInstruction>,
    pub errors: alloc::vec::Vec<ParseError>,
    pub total_size: usize,
    pub bytecode: &'a [u8],
}

/// Parsed script result for optimized bytecode with metadata sections
#[derive(Debug, Clone)]
pub struct ParsedScript {
    pub header: OptimizedHeader,
    pub function_names: Option<FunctionNameMetadata>,
    pub instructions: Vec<ParsedInstruction>,
    pub bytecode_start: usize,
}

/// Parsed instruction with decoded arguments
#[derive(Debug, Clone, Copy)]
pub struct ParsedInstruction {
    pub offset: usize,
    pub opcode: u8,
    pub arg1: u32,
    pub arg2: u32,
    pub size: usize,
}

/// Parser error types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseError {
    InvalidMagic,
    HeaderTooShort,
    BytecodeTooShort,
    InvalidOpcode,
    InvalidVLE,
    InstructionOutOfBounds,
    CallTargetOutOfBounds,
    InvalidFunctionCount,
    // Add more as needed
}

impl ParseError {
    /// Get error message
    pub fn message(&self) -> &'static str {
        match self {
            ParseError::InvalidMagic => "Invalid magic number",
            ParseError::HeaderTooShort => "Bytecode too short for header",
            ParseError::BytecodeTooShort => "Bytecode too short for instruction",
            ParseError::InvalidOpcode => "Invalid opcode",
            ParseError::InvalidVLE => "Invalid VLE encoding",
            ParseError::InstructionOutOfBounds => "Instruction out of bounds",
            ParseError::CallTargetOutOfBounds => "CALL target out of bounds",
            ParseError::InvalidFunctionCount => "Invalid function count",
        }
    }
}

/// Parse header and return basic info + instruction start offset
///
/// This function validates the header magic, basic fields, and skips any metadata
/// sections (like function names) to find where instructions actually begin.
/// It uses 0 allocations.
pub fn parse_header(bytecode: &[u8]) -> Result<(OptimizedHeader, usize), ParseError> {
    if bytecode.len() < crate::FIVE_HEADER_OPTIMIZED_SIZE {
        return Err(ParseError::HeaderTooShort);
    }

    // Parse header
    let magic = [bytecode[0], bytecode[1], bytecode[2], bytecode[3]];
    if magic != *b"5IVE" {
        return Err(ParseError::InvalidMagic);
    }

    // Read features as little-endian u32 from bytes 4..8 and counts from bytes 8/9
    let features = u32::from_le_bytes([bytecode[4], bytecode[5], bytecode[6], bytecode[7]]);

    let header = OptimizedHeader {
        magic,
        features,
        public_function_count: bytecode[8],
        total_function_count: bytecode[9],
    };

    // Validate function counts
    if header.total_function_count > crate::MAX_FUNCTIONS as u8 {
        return Err(ParseError::InvalidFunctionCount);
    }

    // Calculate instruction start offset (skip metadata)
    let mut offset = crate::FIVE_HEADER_OPTIMIZED_SIZE;

    // Skip function names metadata if present
    if (header.features & crate::FEATURE_FUNCTION_NAMES) != 0 {
        // Read section size (VLE u16)
        if offset >= bytecode.len() {
             // Incomplete metadata
             // If we just have the header and claim metadata but no bytes, that's technically OOB for metadata
             return Ok((header, offset)); // Let caller decide if OOB matter
        }

        match VLE::decode_u16(&bytecode[offset..]) {
            Some((section_size, consumed)) => {
                offset += consumed;
                // Skip the section content
                offset += section_size as usize;
            }
            None => return Err(ParseError::InvalidVLE),
        }
    }

    // Ensure start offset is within bounds (or exactly at end if empty script)
    if offset > bytecode.len() {
        return Err(ParseError::HeaderTooShort); // Metadata claimed to be larger than bytecode
    }

    Ok((header, offset))
}

/// Parse bytecode and return parsed metadata with validation errors
pub fn parse_bytecode(bytecode: &[u8]) -> ParsedBytecode<'_> {
    let mut instructions = alloc::vec::Vec::new();
    let mut errors = alloc::vec::Vec::new();

    let (header, start_offset) = match parse_header(bytecode) {
        Ok(res) => res,
        Err(e) => {
            errors.push(e);
             return ParsedBytecode {
                header: OptimizedHeader {
                    magic: [0u8; 4],
                    features: 0,
                    public_function_count: 0,
                    total_function_count: 0,
                },
                instructions,
                errors,
                total_size: bytecode.len(),
                bytecode,
            };
        }
    };

    // Parse instructions
    let mut offset = start_offset;
    while offset < bytecode.len() {
        match parse_instruction(bytecode, offset) {
            Ok((inst, size)) => {
                // Validate CALL targets (arg1 is function address/offset)
                if inst.opcode == crate::opcodes::CALL && inst.arg1 as usize >= bytecode.len() {
                    errors.push(ParseError::CallTargetOutOfBounds);
                }
                // Additional validations can be added here
                instructions.push(inst);
                offset += size;
            }
            Err(err) => {
                errors.push(err);
                // On error, skip to next byte to continue parsing
                offset += 1;
            }
        }
    }

    ParsedBytecode {
        header,
        instructions,
        errors,
        total_size: bytecode.len(),
        bytecode,
    }
}

/// Parse a single instruction at the given offset
pub fn parse_instruction(
    bytecode: &[u8],
    offset: usize,
) -> Result<(ParsedInstruction, usize), ParseError> {
    if offset >= bytecode.len() {
        return Err(ParseError::InstructionOutOfBounds);
    }

    let opcode = bytecode[offset];
    let info = get_opcode_info(opcode);
    if info.is_none() {
        return Err(ParseError::InvalidOpcode);
    }

    let arg_type = info.unwrap().arg_type;

    let mut arg1 = 0u32;
    let mut arg2 = 0u32;
    let mut total_size = 1; // opcode size

    // Decode arg1 based on arg_type
    match arg_type {
        ArgType::None => {}
        ArgType::U8 => {
            if offset + total_size >= bytecode.len() {
                return Err(ParseError::InstructionOutOfBounds);
            }
            arg1 = bytecode[offset + total_size] as u32;
            total_size += 1;

            // Special handling for PUSH_STRING_LITERAL (0x66)
            // ArgType::U8 consumes length. We must also skip the string bytes.
            if opcode == crate::opcodes::PUSH_STRING_LITERAL {
                let str_len = arg1 as usize;
                if offset + total_size + str_len > bytecode.len() {
                    return Err(ParseError::InstructionOutOfBounds);
                }
                total_size += str_len;
            }
        }
        ArgType::U16 => {
            if offset + total_size + 1 >= bytecode.len() {
                return Err(ParseError::InstructionOutOfBounds);
            }
            let bytes = [
                bytecode[offset + total_size],
                bytecode[offset + total_size + 1],
            ];
            arg1 = u16::from_le_bytes(bytes) as u32;
            total_size += 2;
        }
        ArgType::U32 => match VLE::decode_u32(&bytecode[offset + total_size..]) {
            Some((value, consumed)) => {
                arg1 = value;
                total_size += consumed;

                // Special handling for PUSH_STRING (0x67)
                // ArgType::U32 consumes VLE length. We must also skip the string bytes.
                if opcode == crate::opcodes::PUSH_STRING {
                    let str_len = arg1 as usize;
                    if offset + total_size + str_len > bytecode.len() {
                        return Err(ParseError::InstructionOutOfBounds);
                    }
                    total_size += str_len;
                }
            }
            None => {
                return Err(ParseError::InvalidVLE);
            }
        },
        ArgType::FunctionIndex => match VLE::decode_u32(&bytecode[offset + total_size..]) {
            Some((value, consumed)) => {
                arg1 = value;
                total_size += consumed;
            }
            None => {
                return Err(ParseError::InvalidVLE);
            }
        },
        ArgType::LocalIndex => match VLE::decode_u32(&bytecode[offset + total_size..]) {
            Some((value, consumed)) => {
                arg1 = value;
                total_size += consumed;
            }
            None => {
                return Err(ParseError::InvalidVLE);
            }
        },
        ArgType::AccountIndex => match VLE::decode_u32(&bytecode[offset + total_size..]) {
            Some((value, consumed)) => {
                arg1 = value;
                total_size += consumed;
            }
            None => {
                return Err(ParseError::InvalidVLE);
            }
        },
        ArgType::U64 => {
            match VLE::decode_u64(&bytecode[offset + total_size..]) {
                Some((value, consumed)) => {
                    arg1 = value as u32; // Truncate for now
                    if value > u32::MAX as u64 {
                        return Err(ParseError::InvalidVLE);
                    }
                    total_size += consumed;
                }
                None => return Err(ParseError::InvalidVLE),
            }
        }
        ArgType::ValueType => {
            if offset + total_size >= bytecode.len() {
                return Err(ParseError::InstructionOutOfBounds);
            }
            arg1 = bytecode[offset + total_size] as u32;
            total_size += 1;
        }
        ArgType::TwoRegisters => {
            if offset + total_size + 1 >= bytecode.len() {
                return Err(ParseError::InstructionOutOfBounds);
            }
            arg1 = bytecode[offset + total_size] as u32;
            arg2 = bytecode[offset + total_size + 1] as u32;
            total_size += 2;
        }
        ArgType::ThreeRegisters => {
            if offset + total_size + 2 >= bytecode.len() {
                return Err(ParseError::InstructionOutOfBounds);
            }
            arg1 = bytecode[offset + total_size] as u32;
            arg2 = ((bytecode[offset + total_size + 1] as u32) << 8)
                | bytecode[offset + total_size + 2] as u32;
            total_size += 3;
        }
        ArgType::RegisterIndex => {
            if offset + total_size >= bytecode.len() {
                return Err(ParseError::InstructionOutOfBounds);
            }
            arg1 = bytecode[offset + total_size] as u32;
            total_size += 1;
        }
        ArgType::CallExternal => {
            if offset + total_size + 3 >= bytecode.len() {
                return Err(ParseError::InstructionOutOfBounds);
            }
            // Consumes 4 bytes: account_index (u8) + func_offset (u16) + param_count (u8)
            let account_idx = bytecode[offset + total_size] as u32;
            let offset_bytes = [
                bytecode[offset + total_size + 1],
                bytecode[offset + total_size + 2],
            ];
            let func_offset = u16::from_le_bytes(offset_bytes) as u32;
            let param_count = bytecode[offset + total_size + 3] as u32;

            arg1 = (account_idx << 24) | func_offset;
            arg2 = param_count;
            total_size += 4;
        }
        ArgType::CallInternal => {
            if offset + total_size + 2 >= bytecode.len() {
                return Err(ParseError::InstructionOutOfBounds);
            }
            // Consumes 3 bytes: param_count (u8) + function_address (u16)
            let param_count = bytecode[offset + total_size] as u32;
            let addr_bytes = [
                bytecode[offset + total_size + 1],
                bytecode[offset + total_size + 2],
            ];
            let func_addr = u16::from_le_bytes(addr_bytes) as u32;

            arg1 = func_addr;
            arg2 = param_count;
            total_size += 3;
        }
        ArgType::AccountField => {
            if offset + total_size >= bytecode.len() {
                return Err(ParseError::InstructionOutOfBounds);
            }
            arg1 = bytecode[offset + total_size] as u32; // account_index
            total_size += 1;

            match VLE::decode_u32(&bytecode[offset + total_size..]) {
                Some((value, consumed)) => {
                    arg2 = value; // field_offset
                    total_size += consumed;
                }
                None => {
                    return Err(ParseError::InvalidVLE);
                }
            }
        }
    }

    // Check bounds
    if offset + total_size > bytecode.len() {
        return Err(ParseError::InstructionOutOfBounds);
    }

    Ok((
        ParsedInstruction {
            offset,
            opcode,
            arg1,
            arg2,
            size: total_size,
        },
        total_size,
    ))
}

/// Parse function name metadata section
/// Returns the metadata and the offset after the metadata section
fn parse_function_names(
    bytecode: &[u8],
    offset: &mut usize,
) -> Result<(FunctionNameMetadata, usize), String> {
    use crate::VLE;

    let _start_offset = *offset;

    // Read section size
    let (section_size, bytes_read) =
        VLE::decode_u16(&bytecode[*offset..]).ok_or("Invalid VLE for section_size")?;
    *offset += bytes_read;

    // Read name count (u8, always 1 byte)
    let (name_count, bytes_read) =
        VLE::decode_u8(&bytecode[*offset..]).ok_or("Invalid VLE for name_count")?;
    *offset += bytes_read;

    let mut names = Vec::with_capacity(name_count as usize);

    for idx in 0..name_count {
        // Read name length (u8, always 1 byte)
        let (name_len, bytes_read) =
            VLE::decode_u8(&bytecode[*offset..]).ok_or("Invalid VLE for name_len")?;
        *offset += bytes_read;

        // Read name bytes
        let name_bytes = &bytecode[*offset..*offset + name_len as usize];
        *offset += name_len as usize;

        let name = String::from_utf8(name_bytes.to_vec())
            .map_err(|_| "Invalid UTF-8 in function name".to_string())?;

        names.push(FunctionNameEntry {
            name,
            function_index: idx,
        });
    }

    Ok((
        FunctionNameMetadata {
            section_size,
            names,
        },
        *offset,
    ))
}

/// Parse optimized bytecode with metadata sections
pub fn parse_optimized_bytecode(bytecode: &[u8]) -> Result<ParsedScript, String> {
    let (header, start_offset) = parse_header(bytecode).map_err(|e| e.message().to_string())?;

    let (function_names, bytecode_start) = if (header.features & crate::FEATURE_FUNCTION_NAMES) != 0
    {
        // If parse_header returned start_offset that already skipped metadata, we might need to backtrack
        // if we want to extract names.
        // However, parse_header logic simply skips it.
        // If we want names, we have to parse them.
        let mut offset = crate::FIVE_HEADER_OPTIMIZED_SIZE;
        let (metadata, final_offset) = parse_function_names(bytecode, &mut offset)?;
        (Some(metadata), final_offset)
    } else {
        (None, start_offset)
    };

    let mut instructions = Vec::new();
    let mut offset = bytecode_start;
    while offset < bytecode.len() {
        match parse_instruction(bytecode, offset) {
            Ok((inst, size)) => {
                // Validate CALL targets (arg1 is function address/offset)
                if inst.opcode == crate::opcodes::CALL
                    && inst.arg1 as usize >= bytecode.len()
                {
                    return Err("CALL target out of bounds".to_string());
                }
                instructions.push(inst);
                offset += size;
            }
            Err(e) => return Err(format!("Parse error at offset {}: {:?}", offset, e)),
        }
    }

    Ok(ParsedScript {
        header,
        function_names,
        instructions,
        bytecode_start,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bytecode_builder::BytecodeBuilder;
    use crate::encoding::VLE;
    use crate::opcodes::*;
    use crate::{FunctionNameEntry, FEATURE_FUNCTION_NAMES};

    #[test]
    fn test_parse_valid_header() {
        let bytecode = [
            b'5', b'I', b'V', b'E', // magic [0..4]
            0, 0, 0, 0,    // features [4..8] (4-byte u32)
            1,    // public_functions [8]
            2,    // total_functions [9]
            HALT, // instruction [10]
        ];
        let parsed = parse_bytecode(&bytecode);
        assert!(parsed.errors.is_empty());
        assert_eq!(parsed.header.magic, *b"5IVE");
        assert_eq!(parsed.header.public_function_count, 1);
        assert_eq!(parsed.header.total_function_count, 2);
        assert_eq!(parsed.instructions.len(), 1);
        assert_eq!(parsed.instructions[0].opcode, HALT);
    }

    #[test]
    fn test_parse_invalid_magic() {
        let bytecode = [
            b'B', b'A', b'D', b'X', // invalid magic [0..4]
            0, 0, 0, 0, // features [4..8]
            1, 2,    // counts [8..10]
            HALT, // instruction [10]
        ];
        let parsed = parse_bytecode(&bytecode);
        assert_eq!(parsed.errors.len(), 1);
        assert!(matches!(parsed.errors[0], ParseError::InvalidMagic));
    }

    #[test]
    fn test_instruction_bounds() {
        let bytecode = [
            b'5', b'I', b'V', b'E', // magic [0..4]
            0, 0, 0, 0, // features [4..8]
            1, 2,       // counts [8..10]
            PUSH_U8, // opcode [10] but no arg following
        ];
        let parsed = parse_bytecode(&bytecode);
        assert_eq!(
            parsed.errors.len(),
            1,
            "Expected 1 error, got {:?}",
            parsed.errors
        );
        // PUSH_U8 requires 1 argument byte, which is missing
        assert!(matches!(
            parsed.errors[0],
            ParseError::InstructionOutOfBounds
        ));
    }

    #[test]
    fn test_parse_optimized_bytecode_without_function_names() {
        let mut builder = BytecodeBuilder::new();
        builder.emit_header(1, 2);
        builder.emit_u8(HALT);

        let bytecode = builder.build();
        let parsed = parse_optimized_bytecode(&bytecode).unwrap();

        assert_eq!(parsed.header.magic, *b"5IVE");
        assert_eq!(parsed.header.public_function_count, 1);
        assert_eq!(parsed.header.total_function_count, 2);
        assert!(parsed.function_names.is_none());
        assert_eq!(parsed.instructions.len(), 1);
        assert_eq!(parsed.instructions[0].opcode, HALT);
    }

    #[test]
    fn test_parse_optimized_bytecode_with_function_names() {
        let mut builder = BytecodeBuilder::new();
        builder.emit_header(2, 2);
        // Patch features field (u32 at position 4) to set FEATURE_FUNCTION_NAMES
        builder
            .patch_u32(4, FEATURE_FUNCTION_NAMES)
            .expect("patch feature");

        // Emit function name metadata section
        let names = Vec::from([
            FunctionNameEntry {
                name: "func1".to_string(),
                function_index: 0,
            },
            FunctionNameEntry {
                name: "func2".to_string(),
                function_index: 1,
            },
        ]);

        // Calculate section size
        let mut section_size = 0;
        section_size += 1; // name_count (u8, always 1 byte)
        for name_entry in &names {
            section_size += 1; // name_len (u8, always 1 byte)
            section_size += name_entry.name.len();
        }
        let section_size_u16 = section_size as u16;
        let (size_bytes, bytes) = VLE::encode_u16(section_size_u16);
        builder.emit_bytes(&bytes[..size_bytes]);

        // Emit name_count as VLE u8
        let name_count_u8 = names.len() as u8;
        let (size, bytes) = VLE::encode_u8(name_count_u8);
        builder.emit_bytes(&bytes[..size]);

        // Emit each name
        for name_entry in names {
            let name_len_u8 = name_entry.name.len() as u8;
            let (size, bytes) = VLE::encode_u8(name_len_u8);
            builder.emit_bytes(&bytes[..size]);
            builder.emit_bytes(name_entry.name.as_bytes());
        }

        // Emit instruction
        builder.emit_u8(HALT);

        let bytecode = builder.build();
        let parsed = parse_optimized_bytecode(&bytecode).unwrap();

        assert_eq!(parsed.header.magic, *b"5IVE");
        assert_eq!(parsed.header.public_function_count, 2);
        assert_eq!(parsed.header.total_function_count, 2);
        assert!(parsed.function_names.is_some());

        let metadata = parsed.function_names.as_ref().unwrap();
        assert_eq!(metadata.names.len(), 2);
        assert_eq!(metadata.names[0].name, "func1");
        assert_eq!(metadata.names[0].function_index, 0);
        assert_eq!(metadata.names[1].name, "func2");
        assert_eq!(metadata.names[1].function_index, 1);

        assert_eq!(parsed.instructions.len(), 1);
        assert_eq!(parsed.instructions[0].opcode, HALT);
    }

    #[test]
    fn test_parse_optimized_bytecode_invalid_magic() {
        // Need at least 10 bytes for header: 4 (magic) + 4 (features) + 1 (public_count) + 1 (total_count)
        let bytecode = Vec::from([b'B', b'A', b'D', b'X', 0, 0, 0, 0, 1, 2]);
        let result = parse_optimized_bytecode(&bytecode);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid magic number".to_string());
    }

    #[test]
    fn test_parse_optimized_bytecode_too_short() {
        let bytecode = Vec::from([b'5', b'I', b'V']); // Too short for header
        let result = parse_optimized_bytecode(&bytecode);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Bytecode too short for header".to_string()
        );
    }
}
