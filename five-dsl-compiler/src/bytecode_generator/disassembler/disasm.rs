//! Textual bytecode disassembly (human-readable output).

use crate::bytecode_generator::disassembler::call_decoder::*;
use crate::bytecode_generator::disassembler::decoder::*;
use five_protocol::opcodes;

/// Produce a textual disassembly (one line per instruction).
pub fn disassemble(bytes: &[u8]) -> Vec<String> {
    let mut lines = Vec::new();
    
    // Header-aware disassembly: skip magic, features, counts, and metadata.
    let (header, start_pc) = match five_protocol::parse_header(bytes) {
        Ok(res) => res,
        Err(_) => {
            // Fallback for legacy V1 or invalid scripts: just start at 0
            (five_protocol::OptimizedHeader {
                magic: [0; 4],
                features: 0,
                public_function_count: 0,
                total_function_count: 0,
            }, 0)
        }
    };
    let pool_enabled = (header.features & five_protocol::FEATURE_CONSTANT_POOL) != 0;

    if start_pc > 0 {
        lines.push(format!("HEADER: magic=5IVE features=0x{:08X} public={} total={}", 
            header.features, header.public_function_count, header.total_function_count));
        
        if (header.features & five_protocol::FEATURE_FUNCTION_NAMES) != 0 {
            lines.push("METADATA: Function names section skipped".to_string());
        }
    }

    let mut pc = start_pc;
    while pc < bytes.len() {
        let op = bytes[pc];
        match op {
            opcodes::HALT => {
                lines.push(format!("{:04X}: HALT", pc));
                pc += 1;
            }
            opcodes::PUSH_U8 => {
                if pc + 1 < bytes.len() {
                    let idx = bytes[pc + 1];
                    if pool_enabled {
                        lines.push(format!("{:04X}: PUSH_U8 idx={}", pc, idx));
                    } else {
                        lines.push(format!("{:04X}: PUSH_U8 {}", pc, idx));
                    }
                    pc += 2;
                } else {
                    lines.push(format!("{:04X}: PUSH_U8 <truncated>", pc));
                    break;
                }
            }
            opcodes::PUSH_U16 => {
                if pool_enabled {
                    if pc + 1 < bytes.len() {
                        let idx = bytes[pc + 1];
                        lines.push(format!("{:04X}: PUSH_U16 idx={}", pc, idx));
                        pc += 2;
                    } else {
                        lines.push(format!("{:04X}: PUSH_U16 <truncated>", pc));
                        break;
                    }
                } else if pc + 3 <= bytes.len() {
                    let raw = u16::from_le_bytes([bytes[pc + 1], bytes[pc + 2]]);
                    lines.push(format!("{:04X}: PUSH_U16 {}", pc, raw));
                    pc += 3;
                } else {
                    lines.push(format!("{:04X}: PUSH_U16 <truncated>", pc));
                    break;
                }
            }
            opcodes::PUSH_U32 => {
                if pc + 1 < bytes.len() {
                    let idx = bytes[pc + 1];
                    if pool_enabled {
                        lines.push(format!("{:04X}: PUSH_U32 idx={}", pc, idx));
                        pc += 2;
                    } else if pc + 5 <= bytes.len() {
                        let raw = u32::from_le_bytes([bytes[pc + 1], bytes[pc + 2], bytes[pc + 3], bytes[pc + 4]]);
                        lines.push(format!("{:04X}: PUSH_U32 {}", pc, raw));
                        pc += 5;
                    } else {
                        lines.push(format!("{:04X}: PUSH_U32 <truncated>", pc));
                        break;
                    }
                } else {
                    lines.push(format!("{:04X}: PUSH_U32 <truncated>", pc));
                    break;
                }
            }
            opcodes::PUSH_U64 => {
                if pool_enabled {
                    if pc + 1 < bytes.len() {
                        lines.push(format!("{:04X}: PUSH_U64 idx={}", pc, bytes[pc + 1]));
                        pc += 2;
                    } else {
                        lines.push(format!("{:04X}: PUSH_U64 <truncated>", pc));
                        break;
                    }
                } else if pc + 9 <= bytes.len() {
                    if let Some(raw) = read_le_u64(bytes, pc + 1) {
                        lines.push(format!("{:04X}: PUSH_U64 {}", pc, raw));
                        pc += 9;
                    } else {
                        lines.push(format!("{:04X}: PUSH_U64 <truncated>", pc));
                        break;
                    }
                } else {
                    lines.push(format!("{:04X}: PUSH_U64 <truncated>", pc));
                    break;
                }
            }
            opcodes::PUSH_I64 => {
                if pool_enabled {
                    if pc + 1 < bytes.len() {
                        lines.push(format!("{:04X}: PUSH_I64 idx={}", pc, bytes[pc + 1]));
                        pc += 2;
                    } else {
                        lines.push(format!("{:04X}: PUSH_I64 <truncated>", pc));
                        break;
                    }
                } else if pc + 9 <= bytes.len() {
                    if let Some(raw) = read_le_u64(bytes, pc + 1) {
                        lines.push(format!("{:04X}: PUSH_I64 {}", pc, raw as i64));
                        pc += 9;
                    } else {
                        lines.push(format!("{:04X}: PUSH_I64 <truncated>", pc));
                        break;
                    }
                } else {
                    lines.push(format!("{:04X}: PUSH_I64 <truncated>", pc));
                    break;
                }
            }
            opcodes::PUSH_BOOL => {
                if pc + 1 < bytes.len() {
                    let idx = bytes[pc + 1];
                    if pool_enabled {
                        lines.push(format!("{:04X}: PUSH_BOOL idx={}", pc, idx));
                    } else {
                        lines.push(format!("{:04X}: PUSH_BOOL {}", pc, idx));
                    }
                    pc += 2;
                } else {
                    lines.push(format!("{:04X}: PUSH_BOOL <truncated>", pc));
                    break;
                }
            }
            opcodes::PUSH_U128 => {
                if pool_enabled {
                    if pc + 1 < bytes.len() {
                        lines.push(format!("{:04X}: PUSH_U128 idx={}", pc, bytes[pc + 1]));
                        pc += 2;
                    } else {
                        lines.push(format!("{:04X}: PUSH_U128 <truncated>", pc));
                        break;
                    }
                } else if pc + 17 <= bytes.len() {
                    lines.push(format!("{:04X}: PUSH_U128 <16 bytes>", pc));
                    pc += 17;
                } else {
                    lines.push(format!("{:04X}: PUSH_U128 <truncated>", pc));
                    break;
                }
            }
            opcodes::PUSH_PUBKEY => {
                if pool_enabled {
                    if pc + 1 < bytes.len() {
                        lines.push(format!("{:04X}: PUSH_PUBKEY idx={}", pc, bytes[pc + 1]));
                        pc += 2;
                    } else {
                        lines.push(format!("{:04X}: PUSH_PUBKEY <truncated>", pc));
                        break;
                    }
                } else if pc + 33 <= bytes.len() {
                    lines.push(format!("{:04X}: PUSH_PUBKEY <32 bytes>", pc));
                    pc += 33
                } else {
                    lines.push(format!("{:04X}: PUSH_PUBKEY <truncated>", pc));
                    break;
                }
            }
            opcodes::PUSH_STRING => {
                if pool_enabled {
                    if pc + 1 < bytes.len() {
                        lines.push(format!("{:04X}: PUSH_STRING idx={}", pc, bytes[pc + 1]));
                        pc += 2;
                    } else {
                        lines.push(format!("{:04X}: PUSH_STRING <truncated>", pc));
                        break;
                    }
                } else if pc + 5 <= bytes.len() {
                    let len = u32::from_le_bytes([bytes[pc + 1], bytes[pc + 2], bytes[pc + 3], bytes[pc + 4]]);
                    lines.push(format!("{:04X}: PUSH_STRING len={}", pc, len));
                    pc += 5 + len as usize;
                } else {
                    lines.push(format!("{:04X}: PUSH_STRING <truncated>", pc));
                    break;
                }
            }
            opcodes::PUSH_U8_W
            | opcodes::PUSH_U16_W
            | opcodes::PUSH_U32_W
            | opcodes::PUSH_U64_W
            | opcodes::PUSH_I64_W
            | opcodes::PUSH_BOOL_W
            | opcodes::PUSH_U128_W
            | opcodes::PUSH_PUBKEY_W
            | opcodes::PUSH_STRING_W => {
                if pc + 2 < bytes.len() {
                    let idx = u16::from_le_bytes([bytes[pc + 1], bytes[pc + 2]]);
                    lines.push(format!("{:04X}: {} idx={}", pc, five_protocol::opcodes::opcode_name(op), idx));
                    pc += 3;
                } else {
                    lines.push(format!("{:04X}: {} <truncated>", pc, five_protocol::opcodes::opcode_name(op)));
                    break;
                }
            }
            opcodes::CALL => {
                if let Some(call) = decode_call_at(bytes, pc) {
                    let mut s = format!(
                        "{:04X}: CALL param_count={} addr=0x{:04X}",
                        pc, call.param_count, call.function_address
                    );
                    if let Some(ref name) = call.name_metadata {
                        s.push_str(&format!(" name=\"{}\"", name));
                    }
                    lines.push(s);
                    pc += call_size(bytes, pc);
                } else {
                    lines.push(format!("{:04X}: CALL <truncated>", pc));
                    break;
                }
            }
            opcodes::JUMP => {
                if pc + 3 <= bytes.len() {
                    let offset = u16::from_le_bytes([bytes[pc + 1], bytes[pc + 2]]);
                    lines.push(format!("{:04X}: JUMP offset={}", pc, offset));
                    pc += 3;
                } else {
                    lines.push(format!("{:04X}: JUMP <truncated>", pc));
                    break;
                }
            }
            opcodes::JUMP_IF => {
                if pc + 3 <= bytes.len() {
                    let offset = u16::from_le_bytes([bytes[pc + 1], bytes[pc + 2]]);
                    lines.push(format!("{:04X}: JUMP_IF offset={}", pc, offset));
                    pc += 3;
                } else {
                    lines.push(format!("{:04X}: JUMP_IF <truncated>", pc));
                    break;
                }
            }
            opcodes::JUMP_IF_NOT => {
                if pc + 3 <= bytes.len() {
                    let offset = u16::from_le_bytes([bytes[pc + 1], bytes[pc + 2]]);
                    lines.push(format!("{:04X}: JUMP_IF_NOT offset={}", pc, offset));
                    pc += 3;
                } else {
                    lines.push(format!("{:04X}: JUMP_IF_NOT <truncated>", pc));
                    break;
                }
            }
            opcodes::BR_EQ_U8 => {
                if pc + 4 <= bytes.len() {
                    let val = bytes[pc + 1];
                    let offset = u16::from_le_bytes([bytes[pc + 2], bytes[pc + 3]]);
                    lines.push(format!("{:04X}: BR_EQ_U8 val={} offset={}", pc, val, offset));
                    pc += 4;
                } else {
                    lines.push(format!("{:04X}: BR_EQ_U8 <truncated>", pc));
                    break;
                }
            }
            opcodes::LOAD_PARAM => {
                if pc + 1 < bytes.len() {
                    lines.push(format!("{:04X}: LOAD_PARAM idx={}", pc, bytes[pc + 1]));
                    pc += 2;
                } else {
                    lines.push(format!("{:04X}: LOAD_PARAM <truncated>", pc));
                    break;
                }
            }
            // Fused operations (0xC0-0xCF)
            opcodes::REQUIRE_GTE_U64 => {
                // acc(u8) + offset(u32) + param(u8)
                if pc + 6 <= bytes.len() {
                    let acc = bytes[pc + 1];
                    let offset = u32::from_le_bytes([bytes[pc + 2], bytes[pc + 3], bytes[pc + 4], bytes[pc + 5]]);
                    let param = bytes[pc + 6];
                    lines.push(format!("{:04X}: REQUIRE_GTE_U64 acc={} offset={} param={}", pc, acc, offset, param));
                    pc += 7;
                } else {
                    lines.push(format!("{:04X}: REQUIRE_GTE_U64 <truncated>", pc));
                    break;
                }
            }
            opcodes::REQUIRE_NOT_BOOL => {
                // acc(u8) + offset(u32)
                if pc + 6 <= bytes.len() {
                    let acc = bytes[pc + 1];
                    let offset = u32::from_le_bytes([bytes[pc + 2], bytes[pc + 3], bytes[pc + 4], bytes[pc + 5]]);
                    lines.push(format!("{:04X}: REQUIRE_NOT_BOOL acc={} offset={}", pc, acc, offset));
                    pc += 6;
                } else {
                    lines.push(format!("{:04X}: REQUIRE_NOT_BOOL <truncated>", pc));
                    break;
                }
            }
            opcodes::FIELD_ADD_PARAM => {
                // acc(u8) + offset(u32) + param(u8)
                if pc + 7 <= bytes.len() {
                    let acc = bytes[pc + 1];
                    let offset = u32::from_le_bytes([bytes[pc + 2], bytes[pc + 3], bytes[pc + 4], bytes[pc + 5]]);
                    let param = bytes[pc + 6];
                    lines.push(format!("{:04X}: FIELD_ADD_PARAM acc={} offset={} param={}", pc, acc, offset, param));
                    pc += 7;
                } else {
                    lines.push(format!("{:04X}: FIELD_ADD_PARAM <truncated>", pc));
                    break;
                }
            }
            opcodes::FIELD_SUB_PARAM => {
                // acc(u8) + offset(u32) + param(u8)
                if pc + 7 <= bytes.len() {
                    let acc = bytes[pc + 1];
                    let offset = u32::from_le_bytes([bytes[pc + 2], bytes[pc + 3], bytes[pc + 4], bytes[pc + 5]]);
                    let param = bytes[pc + 6];
                    lines.push(format!("{:04X}: FIELD_SUB_PARAM acc={} offset={} param={}", pc, acc, offset, param));
                    pc += 7;
                } else {
                    lines.push(format!("{:04X}: FIELD_SUB_PARAM <truncated>", pc));
                    break;
                }
            }
            opcodes::REQUIRE_PARAM_GT_ZERO => {
                if pc + 1 < bytes.len() {
                    lines.push(format!("{:04X}: REQUIRE_PARAM_GT_ZERO param={}", pc, bytes[pc + 1]));
                    pc += 2;
                } else {
                    lines.push(format!("{:04X}: REQUIRE_PARAM_GT_ZERO <truncated>", pc));
                    break;
                }
            }
            opcodes::REQUIRE_EQ_PUBKEY => {
                // acc1(u8) + offset1(u32) + acc2(u8) + offset2(u32)
                if pc + 11 <= bytes.len() {
                    let acc1 = bytes[pc + 1];
                    let offset1 = u32::from_le_bytes([bytes[pc + 2], bytes[pc + 3], bytes[pc + 4], bytes[pc + 5]]);
                    let acc2 = bytes[pc + 6];
                    let offset2 = u32::from_le_bytes([bytes[pc + 7], bytes[pc + 8], bytes[pc + 9], bytes[pc + 10]]);
                    lines.push(format!("{:04X}: REQUIRE_EQ_PUBKEY acc1={} offset1={} acc2={} offset2={}", pc, acc1, offset1, acc2, offset2));
                    pc += 11;
                } else {
                    lines.push(format!("{:04X}: REQUIRE_EQ_PUBKEY <truncated>", pc));
                    break;
                }
            }
            opcodes::CHECK_SIGNER_WRITABLE => {
                if pc + 1 < bytes.len() {
                    lines.push(format!("{:04X}: CHECK_SIGNER_WRITABLE acc={}", pc, bytes[pc + 1]));
                    pc += 2;
                } else {
                    lines.push(format!("{:04X}: CHECK_SIGNER_WRITABLE <truncated>", pc));
                    break;
                }
            }
            opcodes::STORE_PARAM_TO_FIELD => {
                // acc(u8) + offset(u32) + param(u8)
                if pc + 7 <= bytes.len() {
                    let acc = bytes[pc + 1];
                    let offset = u32::from_le_bytes([bytes[pc + 2], bytes[pc + 3], bytes[pc + 4], bytes[pc + 5]]);
                    let param = bytes[pc + 6];
                    lines.push(format!("{:04X}: STORE_PARAM_TO_FIELD acc={} offset={} param={}", pc, acc, offset, param));
                    pc += 7;
                } else {
                    lines.push(format!("{:04X}: STORE_PARAM_TO_FIELD <truncated>", pc));
                    break;
                }
            }
            opcodes::STORE_FIELD_ZERO => {
                // acc(u8) + offset(u32)
                if pc + 6 <= bytes.len() {
                    let acc = bytes[pc + 1];
                    let offset = u32::from_le_bytes([bytes[pc + 2], bytes[pc + 3], bytes[pc + 4], bytes[pc + 5]]);
                    lines.push(format!("{:04X}: STORE_FIELD_ZERO acc={} offset={}", pc, acc, offset));
                    pc += 6;
                } else {
                    lines.push(format!("{:04X}: STORE_FIELD_ZERO <truncated>", pc));
                    break;
                }
            }
            opcodes::STORE_KEY_TO_FIELD => {
                // acc(u8) + offset(u32) + key_acc(u8)
                if pc + 7 <= bytes.len() {
                    let acc = bytes[pc + 1];
                    let offset = u32::from_le_bytes([bytes[pc + 2], bytes[pc + 3], bytes[pc + 4], bytes[pc + 5]]);
                    let key_acc = bytes[pc + 6];
                    lines.push(format!("{:04X}: STORE_KEY_TO_FIELD acc={} offset={} key_acc={}", pc, acc, offset, key_acc));
                    pc += 7;
                } else {
                    lines.push(format!("{:04X}: STORE_KEY_TO_FIELD <truncated>", pc));
                    break;
                }
            }
            opcodes::REQUIRE_EQ_FIELDS => {
                // acc1(u8) + offset1(u32) + acc2(u8) + offset2(u32)
                if pc + 11 <= bytes.len() {
                    let acc1 = bytes[pc + 1];
                    let offset1 = u32::from_le_bytes([bytes[pc + 2], bytes[pc + 3], bytes[pc + 4], bytes[pc + 5]]);
                    let acc2 = bytes[pc + 6];
                    let offset2 = u32::from_le_bytes([bytes[pc + 7], bytes[pc + 8], bytes[pc + 9], bytes[pc + 10]]);
                    lines.push(format!("{:04X}: REQUIRE_EQ_FIELDS acc1={} offset1={} acc2={} offset2={}", pc, acc1, offset1, acc2, offset2));
                    pc += 11;
                } else {
                    lines.push(format!("{:04X}: REQUIRE_EQ_FIELDS <truncated>", pc));
                    break;
                }
            }
            opcodes::LOAD_FIELD => {
                // acc(u8) + offset(u32)
                if pc + 5 < bytes.len() {
                    let acc = bytes[pc + 1];
                    let offset = u32::from_le_bytes([bytes[pc + 2], bytes[pc + 3], bytes[pc + 4], bytes[pc + 5]]);
                    lines.push(format!("{:04X}: LOAD_FIELD acc={} offset={}", pc, acc, offset));
                    pc += 6;
                } else {
                    lines.push(format!("{:04X}: LOAD_FIELD <truncated>", pc));
                    break;
                }
            }
            opcodes::STORE_FIELD => {
                // acc(u8) + offset(u32)
                if pc + 5 < bytes.len() {
                    let acc = bytes[pc + 1];
                    let offset = u32::from_le_bytes([bytes[pc + 2], bytes[pc + 3], bytes[pc + 4], bytes[pc + 5]]);
                    lines.push(format!(
                        "{:04X}: STORE_FIELD acc={} offset={}",
                        pc, acc, offset
                    ));
                    pc += 6;
                } else {
                    lines.push(format!("{:04X}: STORE_FIELD <truncated>", pc));
                    break;
                }
            }
            opcodes::CHECK_SIGNER => {
                if pc + 1 < bytes.len() {
                    lines.push(format!("{:04X}: CHECK_SIGNER acc={}", pc, bytes[pc + 1]));
                    pc += 2;
                } else {
                    lines.push(format!("{:04X}: CHECK_SIGNER <truncated>", pc));
                    break;
                }
            }
            opcodes::CHECK_WRITABLE => {
                if pc + 1 < bytes.len() {
                    lines.push(format!("{:04X}: CHECK_WRITABLE acc={}", pc, bytes[pc + 1]));
                    pc += 2;
                } else {
                    lines.push(format!("{:04X}: CHECK_WRITABLE <truncated>", pc));
                    break;
                }
            }
            opcodes::GET_KEY => {
                if pc + 1 < bytes.len() {
                    lines.push(format!("{:04X}: GET_KEY acc={}", pc, bytes[pc + 1]));
                    pc += 2;
                } else {
                    lines.push(format!("{:04X}: GET_KEY <truncated>", pc));
                    break;
                }
            }
            opcodes::GET_LOCAL => {
                if pc + 1 < bytes.len() {
                    lines.push(format!("{:04X}: GET_LOCAL idx={}", pc, bytes[pc + 1]));
                    pc += 2;
                } else {
                    lines.push(format!("{:04X}: GET_LOCAL <truncated>", pc));
                    break;
                }
            }
            opcodes::SET_LOCAL => {
                if pc + 1 < bytes.len() {
                    lines.push(format!("{:04X}: SET_LOCAL idx={}", pc, bytes[pc + 1]));
                    pc += 2;
                } else {
                    lines.push(format!("{:04X}: SET_LOCAL <truncated>", pc));
                    break;
                }
            }
            opcodes::EQ_ZERO_JUMP | opcodes::GT_ZERO_JUMP | opcodes::LT_ZERO_JUMP => {
                if pc + 3 <= bytes.len() {
                    let target = u16::from_le_bytes([bytes[pc + 1], bytes[pc + 2]]);
                    lines.push(format!("{:04X}: {} target=0x{:04X}", pc, opcode_name(op), target));
                    pc += 3;
                } else {
                    lines.push(format!("{:04X}: {} <truncated>", pc, opcode_name(op)));
                    break;
                }
            }
            _ => {
                lines.push(format!("{:04X}: {}", pc, opcode_name(op)));
                if let Some(operand_size) = opcodes::operand_size(op, &bytes[pc + 1..], pool_enabled) {
                    let instruction_size = 1 + operand_size;
                    if pc + instruction_size <= bytes.len() {
                        pc += instruction_size;
                    } else {
                        lines.push(format!("{:04X}: {} <truncated>", pc, opcode_name(op)));
                        break;
                    }
                } else {
                    pc += 1;
                }
            }
        }
    }
    lines
}

/// Human-friendly opcode name lookup.
fn opcode_name(op: u8) -> &'static str {
    five_protocol::opcodes::opcode_name(op)
}

/// Alias for disassemble to fix build issues
pub fn get_disassembly(bytes: &[u8]) -> Vec<String> {
    disassemble(bytes)
}

/// Helper to inspect bytecode around a failure (hex dump)
pub fn inspect_failure(bytes: &[u8], pos: usize, context: usize) -> String {
    let start = pos.saturating_sub(context);
    let end = (pos + context).min(bytes.len());
    if start >= end {
        return "Empty bytecode".to_string();
    }
    let chunk = &bytes[start..end];
    let hex: Vec<String> = chunk.iter().map(|b| format!("{:02X}", b)).collect();
    format!("... {} ... (at offset {})", hex.join(" "), pos)
}
