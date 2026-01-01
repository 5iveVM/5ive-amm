//! High-level bytecode inspection queries for tests and diagnostics.

use crate::bytecode_generator::disassembler::call_decoder::*;
use crate::bytecode_generator::disassembler::decoder::*;
use crate::bytecode_generator::disassembler::types::*;
use five_protocol::opcodes;

/// Lightweight bytecode inspector used by tests and diagnostics.
///
/// It provides higher-level queries (contains push of value X, list CALLs, etc).
#[derive(Debug, Clone)]
pub struct BytecodeInspector {
    bytes: Vec<u8>,
    instructions_start: usize,
}

impl BytecodeInspector {
    /// Create a new inspector from bytes.
    pub fn new(bytes: &[u8]) -> Self {
        // Skip the optimized header (10 bytes) and any metadata section
        let instructions_start = Self::find_instructions_start(bytes);
        Self {
            bytes: bytes.to_vec(),
            instructions_start,
        }
    }

    /// Find where instructions start by skipping header and metadata
    fn find_instructions_start(bytes: &[u8]) -> usize {
        // Minimum header size is 10 bytes
        if bytes.len() < 10 {
            return bytes.len();
        }

        // Check for FEATURE_FUNCTION_NAMES at offset [4..8]
        let features = if bytes.len() >= 8 {
            u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]])
        } else {
            0
        };

        const FEATURE_FUNCTION_NAMES: u32 = 1 << 8;

        let mut offset = 10; // After header

        // If metadata is present, skip it
        if (features & FEATURE_FUNCTION_NAMES) != 0 && offset < bytes.len() {
            // Skip metadata section
            // Format: [VLE u16 section_size] [u8 name_count] [u8 name_len, bytes...]*
            // Try to decode the section size
            if let Some((section_size, bytes_read)) = Self::decode_vle_u16(&bytes[offset..]) {
                offset += bytes_read + section_size as usize;
            }
        }

        offset.min(bytes.len())
    }

    /// Simple VLE u16 decoder
    fn decode_vle_u16(bytes: &[u8]) -> Option<(u16, usize)> {
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

    /// Return true if the given raw opcode exists anywhere in the bytecode.
    pub fn contains_opcode(&self, opcode: u8) -> bool {
        self.bytes.contains(&opcode)
    }

    /// Return true if there is any CALL opcode present.
    pub fn contains_call(&self) -> bool {
        self.contains_opcode(opcodes::CALL)
    }

    /// Return true if any push of the given u64 value exists (tolerant to encodings).
    pub fn contains_push_u64(&self, value: u64) -> bool {
        self.find_pushes_u64().iter().any(|p| p.value == value)
    }

    /// Find all u64-like pushes and return metadata.
    ///
    /// This is intentionally forgiving: it tries VLE decoding then falls back to
    /// fixed-width immediates where the opcode historically permitted them.
    pub fn find_pushes_u64(&self) -> Vec<PushInfo> {
        let mut out = Vec::new();
        let b = &self.bytes;
        let mut i = self.instructions_start;

        while i < b.len() {
            let op = b[i];
            match op {
                opcodes::PUSH_U8 => {
                    if i + 1 < b.len() {
                        out.push(PushInfo {
                            offset: i,
                            opcode: op,
                            value: b[i + 1] as u64,
                            width: 1,
                        });
                        i += 2;
                    } else {
                        break;
                    }
                }

                opcodes::PUSH_U16 | opcodes::PUSH_U32 | opcodes::PUSH_U64 => {
                    if let Some((v, c)) = decode_vle_u128(&b[i + 1..]) {
                        out.push(PushInfo {
                            offset: i,
                            opcode: op,
                            value: v as u64,
                            width: c,
                        });
                        i += 1 + c;
                    } else {
                        // fallback fixed-width
                        match op {
                            opcodes::PUSH_U16 => {
                                if i + 3 <= b.len() {
                                    if let Some(raw) = read_le_u16(b, i + 1) {
                                        out.push(PushInfo {
                                            offset: i,
                                            opcode: op,
                                            value: raw as u64,
                                            width: 2,
                                        });
                                        i += 3;
                                    } else {
                                        break;
                                    }
                                } else {
                                    break;
                                }
                            }
                            opcodes::PUSH_U32 => {
                                if i + 5 <= b.len() {
                                    if let Some(raw) = read_le_u32(b, i + 1) {
                                        out.push(PushInfo {
                                            offset: i,
                                            opcode: op,
                                            value: raw as u64,
                                            width: 4,
                                        });
                                        i += 5;
                                    } else {
                                        break;
                                    }
                                } else {
                                    break;
                                }
                            }
                            opcodes::PUSH_U64 => {
                                if i + 9 <= b.len() {
                                    if let Some(raw) = read_le_u64(b, i + 1) {
                                        out.push(PushInfo {
                                            offset: i,
                                            opcode: op,
                                            value: raw,
                                            width: 8,
                                        });
                                        i += 9;
                                    } else {
                                        break;
                                    }
                                } else {
                                    break;
                                }
                            }
                            _ => i += 1,
                        }
                    }
                }

                opcodes::PUSH_I64 => {
                    if let Some((v, c)) = decode_vle_u128(&b[i + 1..]) {
                        let as_i = v as i128 as i64;
                        out.push(PushInfo {
                            offset: i,
                            opcode: op,
                            value: as_i as u64,
                            width: c,
                        });
                        i += 1 + c;
                    } else if i + 9 <= b.len() {
                        if let Some(raw) = read_le_u64(b, i + 1) {
                            out.push(PushInfo {
                                offset: i,
                                opcode: op,
                                value: raw,
                                width: 8,
                            });
                            i += 9;
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }

                opcodes::PUSH_BOOL => {
                    if i + 1 < b.len() {
                        out.push(PushInfo {
                            offset: i,
                            opcode: op,
                            value: b[i + 1] as u64,
                            width: 1,
                        });
                        i += 2;
                    } else {
                        break;
                    }
                }

                opcodes::PUSH_PUBKEY => {
                    if i + 33 <= b.len() {
                        i += 33;
                    } else {
                        break;
                    }
                }

                opcodes::PUSH_U128 => {
                    if i + 17 <= b.len() {
                        if let Some(low) = read_le_u64(b, i + 1) {
                            out.push(PushInfo {
                                offset: i,
                                opcode: op,
                                value: low,
                                width: 16,
                            });
                            i += 17;
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }

                opcodes::PUSH_STRING
                | opcodes::PUSH_STRING_LITERAL
                | opcodes::PUSH_ARRAY_LITERAL => {
                    if let Some((len, c)) = decode_vle_u128(&b[i + 1..]) {
                        let start = i + 1 + c;
                        if start + (len as usize) <= b.len() {
                            i = start + (len as usize);
                            continue;
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }

                opcodes::LOAD_FIELD | opcodes::STORE_FIELD => {
                    if i + 2 <= b.len() {
                        let after = i + 2;
                        if let Some((_v, c)) = decode_vle_u128(&b[after..]) {
                            i = after + c;
                        } else if after + 4 <= b.len() {
                            i = after + 4;
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }

                opcodes::CALL => {
                    if i + 4 <= b.len() {
                        // skip param + addr; skip optional metadata heuristically
                        i += call_size(b, i);
                    } else {
                        break;
                    }
                }

                opcodes::JUMP | opcodes::JUMP_IF | opcodes::JUMP_IF_NOT | opcodes::BR_EQ_U8 => {
                    if op == opcodes::BR_EQ_U8 {
                        if i + 2 <= b.len() {
                            if let Some((_v, c)) = decode_vle_u128(&b[i + 2..]) {
                                i += 2 + c;
                            } else {
                                break;
                            }
                        } else {
                            break;
                        }
                    } else if let Some((_v, c)) = decode_vle_u128(&b[i + 1..]) {
                        i += 1 + c;
                    } else {
                        break;
                    }
                }

                _ => {
                    i += 1;
                }
            }
        }

        out
    }

    /// Find CALLs with metadata.
    pub fn find_calls(&self) -> Vec<CallSite> {
        let mut out = Vec::new();
        let b = &self.bytes;
        let mut i = 0usize;
        while i < b.len() {
            let op = b[i];
            if op == opcodes::CALL {
                if let Some(call) = decode_call_at(b, i) {
                    let advance = call_size(b, i);
                    out.push(call);
                    i += advance;
                    continue;
                } else {
                    break;
                }
            }
            i += 1;
        }
        out
    }

    /// Decode a single instruction at offset into a structured `Instruction`.
    /// Returns `None` if the offset is out of bounds or the encoding is clearly truncated.
    pub fn decode_instruction_at(&self, offset: usize) -> Option<Instruction> {
        let b = &self.bytes;
        if offset >= b.len() {
            return None;
        }
        let op = b[offset];
        match op {
            opcodes::PUSH_U8 => {
                if offset + 1 < b.len() {
                    Some(Instruction::PushU64(PushInfo {
                        offset,
                        opcode: op,
                        value: b[offset + 1] as u64,
                        width: 1,
                    }))
                } else {
                    None
                }
            }
            opcodes::PUSH_U16 | opcodes::PUSH_U32 | opcodes::PUSH_U64 => {
                if let Some((v, c)) = decode_vle_u128(&b[offset + 1..]) {
                    Some(Instruction::PushU64(PushInfo {
                        offset,
                        opcode: op,
                        value: v as u64,
                        width: c,
                    }))
                } else {
                    match op {
                        opcodes::PUSH_U16 => read_le_u16(b, offset + 1).map(|raw| {
                            Instruction::PushU64(PushInfo {
                                offset,
                                opcode: op,
                                value: raw as u64,
                                width: 2,
                            })
                        }),
                        opcodes::PUSH_U32 => read_le_u32(b, offset + 1).map(|raw| {
                            Instruction::PushU64(PushInfo {
                                offset,
                                opcode: op,
                                value: raw as u64,
                                width: 4,
                            })
                        }),
                        opcodes::PUSH_U64 => read_le_u64(b, offset + 1).map(|raw| {
                            Instruction::PushU64(PushInfo {
                                offset,
                                opcode: op,
                                value: raw,
                                width: 8,
                            })
                        }),
                        _ => None,
                    }
                }
            }
            opcodes::PUSH_I64 => {
                if let Some((v, c)) = decode_vle_u128(&b[offset + 1..]) {
                    let as_i = v as i128 as i64;
                    Some(Instruction::PushU64(PushInfo {
                        offset,
                        opcode: op,
                        value: as_i as u64,
                        width: c,
                    }))
                } else if offset + 9 <= b.len() {
                    read_le_u64(b, offset + 1).map(|raw| {
                        Instruction::PushU64(PushInfo {
                            offset,
                            opcode: op,
                            value: raw,
                            width: 8,
                        })
                    })
                } else {
                    None
                }
            }
            opcodes::PUSH_BOOL => {
                if offset + 1 < b.len() {
                    Some(Instruction::PushU64(PushInfo {
                        offset,
                        opcode: op,
                        value: b[offset + 1] as u64,
                        width: 1,
                    }))
                } else {
                    None
                }
            }
            opcodes::SET_LOCAL => {
                if offset + 1 < b.len() {
                    Some(Instruction::SetLocal {
                        offset,
                        index: b[offset + 1],
                    })
                } else {
                    None
                }
            }
            opcodes::GET_LOCAL => {
                if offset + 1 < b.len() {
                    Some(Instruction::GetLocal {
                        offset,
                        index: b[offset + 1],
                    })
                } else {
                    None
                }
            }
            opcodes::ALLOC_LOCALS => Some(Instruction::AllocLocals { offset }),
            opcodes::DEALLOC_LOCALS => Some(Instruction::DeallocLocals { offset }),
            opcodes::LOAD_FIELD | opcodes::STORE_FIELD => {
                if offset + 2 <= b.len() {
                    let account_index = b[offset + 1];
                    let after = offset + 2;
                    if let Some((v, _)) = decode_vle_u128(&b[after..]) {
                        Some(Instruction::LoadField {
                            instr_offset: offset,
                            account_index,
                            field_offset: v as u32,
                        })
                    } else { read_le_u32(b, after).map(|raw| Instruction::LoadField {
                            instr_offset: offset,
                            account_index,
                            field_offset: raw,
                        }) }
                } else {
                    None
                }
            }
            opcodes::CALL => decode_call_at(b, offset).map(Instruction::Call),
            opcodes::CHECK_SIGNER => {
                if let Some((v, _)) = decode_vle_u128(&b[offset + 1..]) {
                    Some(Instruction::CheckSigner {
                        offset,
                        account_index: v as u8,
                    })
                } else {
                    None
                }
            }
            opcodes::CHECK_WRITABLE => {
                if let Some((v, _)) = decode_vle_u128(&b[offset + 1..]) {
                    Some(Instruction::CheckWritable {
                        offset,
                        account_index: v as u8,
                    })
                } else {
                    None
                }
            }
            opcodes::GET_KEY => {
                if offset + 1 < b.len() {
                    Some(Instruction::GetKey {
                        offset,
                        account_index: b[offset + 1],
                    })
                } else {
                    None
                }
            }
            _ => Some(Instruction::Opcode(op)),
        }
    }
}
