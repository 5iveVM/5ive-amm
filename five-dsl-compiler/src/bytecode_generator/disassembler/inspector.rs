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
    pool_enabled: bool,
    pool_offset: usize,
    pool_slots: u16,
}

impl BytecodeInspector {
    /// Create a new inspector from bytes.
    pub fn new(bytes: &[u8]) -> Self {
        // Skip the optimized header (10 bytes) and any metadata section
        let (instructions_start, pool_enabled, pool_offset, pool_slots) =
            Self::find_instructions_start(bytes);
        Self {
            bytes: bytes.to_vec(),
            instructions_start,
            pool_enabled,
            pool_offset,
            pool_slots,
        }
    }

    /// Find where instructions start by skipping header and metadata
    fn find_instructions_start(bytes: &[u8]) -> (usize, bool, usize, u16) {
        // Minimum header size is 10 bytes
        if bytes.len() < 10 {
            return (bytes.len(), false, 0, 0);
        }

        // Check for FEATURE_FUNCTION_NAMES at offset [4..8]
        let features = if bytes.len() >= 8 {
            u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]])
        } else {
            0
        };

        const FEATURE_FUNCTION_NAMES: u32 = 1 << 8;

        let mut offset = 10; // After header
        let pool_enabled = (features & five_protocol::FEATURE_CONSTANT_POOL) != 0;
        let mut pool_offset = 0usize;
        let mut pool_slots = 0u16;

        // If metadata is present, skip it
        if (features & FEATURE_FUNCTION_NAMES) != 0 && offset < bytes.len() {
            // Skip metadata section
            // Format: [u16 section_size] [u8 name_count] [u8 name_len, bytes...]*
            if offset + 2 <= bytes.len() {
                let section_size = u16::from_le_bytes([bytes[offset], bytes[offset+1]]);
                offset += 2 + section_size as usize;
            }
        }
        if pool_enabled {
            let desc_size = core::mem::size_of::<five_protocol::ConstantPoolDescriptor>();
            if offset + desc_size <= bytes.len() {
                pool_offset = u32::from_le_bytes([
                    bytes[offset],
                    bytes[offset + 1],
                    bytes[offset + 2],
                    bytes[offset + 3],
                ]) as usize;
                pool_slots = u16::from_le_bytes([bytes[offset + 12], bytes[offset + 13]]);
                let code_offset = pool_offset + pool_slots as usize * 8;
                return (code_offset.min(bytes.len()), pool_enabled, pool_offset, pool_slots);
            }
        }

        (offset.min(bytes.len()), pool_enabled, pool_offset, pool_slots)
    }

    fn read_pool_slot_u64(&self, index: u16) -> Option<u64> {
        if index >= self.pool_slots {
            return None;
        }
        let start = self.pool_offset + index as usize * 8;
        if start + 8 > self.bytes.len() {
            return None;
        }
        let mut buf = [0u8; 8];
        buf.copy_from_slice(&self.bytes[start..start + 8]);
        Some(u64::from_le_bytes(buf))
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

    /// Return the size of the instruction at the given offset.
    /// Returns 1 for unknown instructions to ensure forward progress.
    pub fn instruction_size(bytes: &[u8], offset: usize) -> usize {
        Self::instruction_size_with_pool(bytes, offset, false)
    }

    pub fn instruction_size_with_pool(bytes: &[u8], offset: usize, pool_enabled: bool) -> usize {
        if offset >= bytes.len() {
            return 0;
        }
        let op = bytes[offset];
        if pool_enabled {
            match op {
                opcodes::PUSH_U8
                | opcodes::PUSH_U16
                | opcodes::PUSH_U32
                | opcodes::PUSH_U64
                | opcodes::PUSH_I64
                | opcodes::PUSH_BOOL
                | opcodes::PUSH_PUBKEY
                | opcodes::PUSH_U128
                | opcodes::PUSH_STRING => return 2,
                opcodes::PUSH_U8_W
                | opcodes::PUSH_U16_W
                | opcodes::PUSH_U32_W
                | opcodes::PUSH_U64_W
                | opcodes::PUSH_I64_W
                | opcodes::PUSH_BOOL_W
                | opcodes::PUSH_PUBKEY_W
                | opcodes::PUSH_U128_W
                | opcodes::PUSH_STRING_W => return 3,
                _ => {}
            }
        }
        match op {
            opcodes::PUSH_U8 => 2,
            opcodes::PUSH_U16 => 3,
            opcodes::PUSH_U32 => 5,
            opcodes::PUSH_U64 => 9,
            opcodes::PUSH_I64 => 9,
            opcodes::PUSH_BOOL => 2,
            opcodes::PUSH_PUBKEY => 33,
            opcodes::PUSH_U128 => 17,
            opcodes::PUSH_U8_W
            | opcodes::PUSH_U16_W
            | opcodes::PUSH_U32_W
            | opcodes::PUSH_U64_W
            | opcodes::PUSH_I64_W
            | opcodes::PUSH_BOOL_W
            | opcodes::PUSH_PUBKEY_W
            | opcodes::PUSH_U128_W
            | opcodes::PUSH_STRING_W => 3,

            // Variable length instructions
            opcodes::PUSH_STRING => {
                if offset + 5 <= bytes.len() {
                    if let Some(len) = read_le_u32(bytes, offset + 1) {
                        return 5 + len as usize;
                    }
                }
                1 // Truncated
            }
            opcodes::PUSH_STRING_LITERAL | opcodes::PUSH_ARRAY_LITERAL => {
                if offset + 1 < bytes.len() {
                    let len = bytes[offset+1] as usize;
                    return 2 + len;
                }
                1
            }

            // Fixed size instructions with arguments
            opcodes::LOAD_FIELD | opcodes::STORE_FIELD => 6, // acc(u8) + offset(u32)
            opcodes::LOAD_FIELD_PUBKEY => 6,
            opcodes::STORE_FIELD_ZERO => 6,
            opcodes::REQUIRE_NOT_BOOL => 6,

            opcodes::CALL => {
                if offset + 4 <= bytes.len() {
                    return call_size(bytes, offset);
                }
                1
            }
            opcodes::CALL_EXTERNAL => 5, // opcode + acc(u8) + offset(u16) + param(u8)
            opcodes::CALL_NATIVE => 2,   // id(u8)

            // Jumps
            opcodes::JUMP | opcodes::JUMP_IF | opcodes::JUMP_IF_NOT => 3, // opcode + u16

            // Branch fusion
            opcodes::BR_EQ_U8 => 4, // opcode + val(u8) + offset(u16)
            opcodes::EQ_ZERO_JUMP | opcodes::GT_ZERO_JUMP | opcodes::LT_ZERO_JUMP => 3, // opcode + offset(u16)

            // Local variables (index u8)
            opcodes::SET_LOCAL | opcodes::GET_LOCAL | opcodes::LOAD_PARAM | opcodes::STORE_PARAM
            | opcodes::CLEAR_LOCAL => 2,

            // Account operations
            opcodes::LOAD_ACCOUNT | opcodes::SAVE_ACCOUNT | opcodes::GET_ACCOUNT
            | opcodes::GET_LAMPORTS | opcodes::SET_LAMPORTS | opcodes::GET_DATA
            | opcodes::GET_KEY | opcodes::GET_OWNER
            | opcodes::CHECK_OWNER | opcodes::CHECK_PDA => 2,

            // Constraint operations (u8 arg)
            opcodes::CHECK_SIGNER | opcodes::CHECK_WRITABLE | opcodes::CHECK_INITIALIZED
            | opcodes::CHECK_UNINITIALIZED | opcodes::CHECK_SIGNER_WRITABLE
            | opcodes::REQUIRE_PARAM_GT_ZERO => 2,

            // Array creation (u8 arg)
            opcodes::CREATE_ARRAY => 2,

            // Fused ops
            opcodes::REQUIRE_GTE_U64 | opcodes::FIELD_ADD_PARAM | opcodes::FIELD_SUB_PARAM
            | opcodes::STORE_PARAM_TO_FIELD => 7, // acc(u8) + offset(u32) + param(u8)

            opcodes::REQUIRE_EQ_PUBKEY | opcodes::REQUIRE_EQ_FIELDS => 11, // acc1(u8) + off1(u32) + acc2(u8) + off2(u32)

            opcodes::TRANSFER_DEBIT | opcodes::TRANSFER_CREDIT => 2, // param(u8)
            opcodes::STORE_GLOBAL | opcodes::LOAD_GLOBAL => 3, // offset(u16)

            opcodes::REQUIRE_PARAM_LTE_IMM => 3, // param(u8) + imm(u8)
            opcodes::REQUIRE_FIELD_EQ_IMM => 7, // acc(u8) + off(u32) + imm(u8)

            opcodes::CAST => 2, // type(u8)
            opcodes::LOAD_INPUT => 2, // index(u8)

            // All others are 1 byte (including HALT, RETURN, ADD, SUB, etc.)
            _ => 1,
        }
    }

    /// Find all u64-like pushes and return metadata.
    pub fn find_pushes_u64(&self) -> Vec<PushInfo> {
        let mut out = Vec::new();
        let b = &self.bytes;
        let mut i = self.instructions_start;

        while i < b.len() {
            let op = b[i];
            let size = Self::instruction_size_with_pool(b, i, self.pool_enabled);

            match op {
                opcodes::PUSH_U8 => {
                    if i + 1 < b.len() {
                        if self.pool_enabled {
                            let idx = b[i + 1] as u16;
                            if let Some(val) = self.read_pool_slot_u64(idx) {
                                out.push(PushInfo {
                                    offset: i,
                                    opcode: op,
                                    value: val,
                                    width: 1,
                                });
                            }
                        } else {
                        out.push(PushInfo {
                            offset: i,
                            opcode: op,
                            value: b[i + 1] as u64,
                            width: 1,
                        });
                        }
                    }
                }

                opcodes::PUSH_U16 => {
                    if self.pool_enabled {
                        if i + 1 < b.len() {
                            let idx = b[i + 1] as u16;
                            if let Some(val) = self.read_pool_slot_u64(idx) {
                                out.push(PushInfo {
                                    offset: i,
                                    opcode: op,
                                    value: val,
                                    width: 1,
                                });
                            }
                        }
                    } else if let Some(raw) = read_le_u16(b, i + 1) {
                        out.push(PushInfo {
                            offset: i,
                            opcode: op,
                            value: raw as u64,
                            width: 2,
                        });
                    }
                }

                opcodes::PUSH_U32 => {
                    if self.pool_enabled {
                        if i + 1 < b.len() {
                            let idx = b[i + 1] as u16;
                            if let Some(val) = self.read_pool_slot_u64(idx) {
                                out.push(PushInfo {
                                    offset: i,
                                    opcode: op,
                                    value: val,
                                    width: 1,
                                });
                            }
                        }
                    } else if let Some(raw) = read_le_u32(b, i + 1) {
                        out.push(PushInfo {
                            offset: i,
                            opcode: op,
                            value: raw as u64,
                            width: 4,
                        });
                    }
                }

                opcodes::PUSH_U64 => {
                    if self.pool_enabled {
                        if i + 1 < b.len() {
                            let idx = b[i + 1] as u16;
                            if let Some(val) = self.read_pool_slot_u64(idx) {
                                out.push(PushInfo {
                                    offset: i,
                                    opcode: op,
                                    value: val,
                                    width: 1,
                                });
                            }
                        }
                    } else if let Some(raw) = read_le_u64(b, i + 1) {
                        out.push(PushInfo {
                            offset: i,
                            opcode: op,
                            value: raw,
                            width: 8,
                        });
                    }
                }

                opcodes::PUSH_I64 => {
                    if self.pool_enabled {
                        if i + 1 < b.len() {
                            let idx = b[i + 1] as u16;
                            if let Some(val) = self.read_pool_slot_u64(idx) {
                                out.push(PushInfo {
                                    offset: i,
                                    opcode: op,
                                    value: val,
                                    width: 1,
                                });
                            }
                        }
                    } else if let Some(raw) = read_le_u64(b, i + 1) {
                        out.push(PushInfo {
                            offset: i,
                            opcode: op,
                            value: raw,
                            width: 8,
                        });
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
                    }
                }

                opcodes::PUSH_U128 => {
                    if let Some(low) = read_le_u64(b, i + 1) {
                        out.push(PushInfo {
                            offset: i,
                            opcode: op,
                            value: low,
                            width: 16,
                        });
                    }
                }

                _ => {}
            }

            i += size;
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
            opcodes::PUSH_I64 => {
                if offset + 9 <= b.len() {
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
                // acc(u8) + offset(u32)
                if offset + 6 <= b.len() {
                    let account_index = b[offset + 1];
                    read_le_u32(b, offset + 2).map(|raw| Instruction::LoadField {
                        instr_offset: offset,
                        account_index,
                        field_offset: raw,
                    })
                } else {
                    None
                }
            }
            opcodes::CALL => decode_call_at(b, offset).map(Instruction::Call),
            opcodes::CHECK_SIGNER => {
                if offset + 1 < b.len() {
                    Some(Instruction::CheckSigner {
                        offset,
                        account_index: b[offset + 1],
                    })
                } else {
                    None
                }
            }
            opcodes::CHECK_WRITABLE => {
                if offset + 1 < b.len() {
                    Some(Instruction::CheckWritable {
                        offset,
                        account_index: b[offset + 1],
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
