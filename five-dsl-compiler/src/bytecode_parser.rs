//! Bytecode parser utilities for external tooling.

use five_protocol::opcodes;
use std::borrow::Cow;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{self, Display};

/// Function call information extracted from bytecode.
#[derive(Debug, Clone)]
pub struct CallInfo<'a> {
    pub position: usize,
    pub param_count: u8,
    pub function_address: u16,
    pub function_name: Option<Cow<'a, str>>,
}

/// Bytecode parsing results containing extracted metadata.
#[derive(Debug, Clone)]
pub struct BytecodeMetadata<'a> {
    /// All function calls found in the bytecode with their metadata
    pub function_calls: Vec<CallInfo<'a>>,
    /// Function name table built from first occurrences
    pub name_table: Vec<Cow<'a, str>>,
    /// Total bytecode size
    pub bytecode_size: usize,
}

/// Errors that can occur while parsing bytecode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BytecodeParseError {
    /// Bytecode ended before a complete CALL instruction was read
    IncompleteCallInstruction,
    /// Name reference marker encountered without a following index
    IncompleteNameReference,
    /// Function name length exceeds remaining bytecode
    IncompleteFunctionName,
    /// Function name bytes were not valid UTF-8
    InvalidUtf8FunctionName,
}

impl Display for BytecodeParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BytecodeParseError::IncompleteCallInstruction => {
                write!(f, "Incomplete CALL instruction")
            }
            BytecodeParseError::IncompleteNameReference => {
                write!(f, "Incomplete name reference in CALL")
            }
            BytecodeParseError::IncompleteFunctionName => {
                write!(f, "Incomplete function name in CALL")
            }
            BytecodeParseError::InvalidUtf8FunctionName => {
                write!(f, "Invalid UTF-8 in function name")
            }
        }
    }
}

impl Error for BytecodeParseError {}

/// Fast bytecode parser for extracting function call metadata.
pub struct BytecodeParser;

impl BytecodeParser {
    /// Parse bytecode and extract function call metadata.
    pub fn parse_function_calls<'a>(
        bytecode: &'a [u8],
    ) -> Result<BytecodeMetadata<'a>, BytecodeParseError> {
        let mut calls = Vec::new();
        let mut name_table: Vec<Cow<'a, str>> = Vec::new();
        let (features, start_offset) = match five_protocol::parse_header(bytecode) {
            Ok((header, start)) => (header.features, start),
            Err(_) => {
                if bytecode.len() >= 4 && &bytecode[0..4] == b"5IVE" {
                    (0, 4)
                } else {
                    (0, 0)
                }
            }
        };

        let mut position = start_offset;

        while position < bytecode.len() {
            let opcode = bytecode[position];
            position += 1;

            match opcode {
                opcodes::CALL => {
                    if position + 2 >= bytecode.len() {
                        return Err(BytecodeParseError::IncompleteCallInstruction);
                    }

                    // Read required parameters
                    let param_count = bytecode[position];
                    position += 1;

                    let function_address =
                        u16::from_le_bytes([bytecode[position], bytecode[position + 1]]);
                    position += 2;

                    // Check for optional function name metadata
                    let function_name = if position < bytecode.len() {
                        let name_len = bytecode[position];
                        position += 1;

                        if name_len == 0xFF {
                            // Name reference - read index
                            if position >= bytecode.len() {
                                return Err(BytecodeParseError::IncompleteNameReference);
                            }
                            let name_index = bytecode[position] as usize;
                            position += 1;

                            // Look up name from table
                            name_table.get(name_index).cloned()
                        } else {
                            // Inline name - read string
                            if position + name_len as usize > bytecode.len() {
                                return Err(BytecodeParseError::IncompleteFunctionName);
                            }

                            let name_bytes = &bytecode[position..position + name_len as usize];
                            position += name_len as usize;

                            let name = std::str::from_utf8(name_bytes)
                                .map_err(|_| BytecodeParseError::InvalidUtf8FunctionName)?;

                            // Add to name table for future references
                            name_table.push(Cow::Borrowed(name));
                            Some(Cow::Borrowed(name))
                        }
                    } else {
                        None // No metadata present
                    };

                    calls.push(CallInfo {
                        position: position - 3, // Position of CALL opcode
                        param_count,
                        function_address,
                        function_name,
                    });
                }
                _ => {
                    // Skip other opcodes by advancing past their operands
                    position += Self::get_operand_size(opcode, bytecode, position, features);
                }
            }
        }

        Ok(BytecodeMetadata {
            function_calls: calls,
            name_table,
            bytecode_size: bytecode.len(),
        })
    }

    /// Get the operand size for a given opcode (simplified version for parsing)
    fn get_operand_size(opcode: u8, bytecode: &[u8], position: usize, features: u32) -> usize {
        if (features & five_protocol::FEATURE_CONSTANT_POOL) != 0 {
            match opcode {
                opcodes::PUSH_U8
                | opcodes::PUSH_U16
                | opcodes::PUSH_U32
                | opcodes::PUSH_U64
                | opcodes::PUSH_I64
                | opcodes::PUSH_BOOL
                | opcodes::PUSH_PUBKEY
                | opcodes::PUSH_U128
                | opcodes::PUSH_STRING => return 1, // pool index (u8)
                opcodes::PUSH_U8_W
                | opcodes::PUSH_U16_W
                | opcodes::PUSH_U32_W
                | opcodes::PUSH_U64_W
                | opcodes::PUSH_I64_W
                | opcodes::PUSH_BOOL_W
                | opcodes::PUSH_PUBKEY_W
                | opcodes::PUSH_U128_W
                | opcodes::PUSH_STRING_W => return 2, // pool index (u16)
                _ => {}
            }
        }

        match opcode {
            opcodes::PUSH_U8 | opcodes::PUSH_BOOL => 1,
            opcodes::PUSH_U64 | opcodes::PUSH_I64 => 8,
            opcodes::PUSH_PUBKEY => 32,
            opcodes::PUSH_U128 => 16,
            opcodes::LOAD_FIELD | opcodes::STORE_FIELD => 4,
            opcodes::JUMP | opcodes::JUMP_IF_NOT | opcodes::JUMP_IF => 2,
            opcodes::BR_EQ_U8 => 3,
            opcodes::PUSH_STRING => {
                if position + 4 <= bytecode.len() {
                    let len = u32::from_le_bytes([
                        bytecode[position],
                        bytecode[position + 1],
                        bytecode[position + 2],
                        bytecode[position + 3],
                    ]) as usize;
                    4 + len
                } else {
                    0
                }
            }
            // CALL is handled specially above
            _ => 0,
        }
    }

    /// Extract function interface information for ecosystem composability
    pub fn extract_function_interfaces<'a>(
        bytecode: &'a [u8],
    ) -> Result<HashMap<Cow<'a, str>, u16>, BytecodeParseError> {
        let metadata = Self::parse_function_calls(bytecode)?;
        let mut interfaces: HashMap<Cow<'a, str>, u16> = HashMap::new();

        for call in metadata.function_calls {
            if let Some(name) = call.function_name {
                interfaces.insert(name, call.function_address);
            }
        }

        Ok(interfaces)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::borrow::Cow;

    #[test]
    fn test_parse_call_with_name() {
        // CALL with embedded name: CALL(0x90) param_count(1) addr(0x0100) name_len(4) "test"
        let bytecode = vec![
            0x90, // CALL
            0x01, // param_count
            0x00, 0x01, // function_address (little-endian)
            0x04, // name_len
            b't', b'e', b's', b't', // name bytes
        ];

        let result = BytecodeParser::parse_function_calls(&bytecode).unwrap();
        assert_eq!(result.function_calls.len(), 1);
        assert_eq!(
            result.function_calls[0].function_name,
            Some(Cow::Borrowed("test"))
        );
        assert_eq!(result.function_calls[0].param_count, 1);
        assert_eq!(result.function_calls[0].function_address, 0x0100);
    }

    #[test]
    fn test_parse_call_with_name_ref() {
        // Two CALLs: first with name, second with reference
        let bytecode = vec![
            0x90, // CALL
            0x01, // param_count
            0x00, 0x01, // function_address
            0x04, // name_len
            b't', b'e', b's', b't', // name bytes
            0x90, // CALL
            0x02, // param_count
            0x00, 0x02, // function_address
            0xFF, // name reference marker
            0x00, // name index (references first occurrence)
        ];

        let result = BytecodeParser::parse_function_calls(&bytecode).unwrap();
        assert_eq!(result.function_calls.len(), 2);
        assert_eq!(
            result.function_calls[0].function_name,
            Some(Cow::Borrowed("test"))
        );
        assert_eq!(
            result.function_calls[1].function_name,
            Some(Cow::Borrowed("test"))
        );
        assert_eq!(result.name_table, vec![Cow::Borrowed("test")]);
    }
}
