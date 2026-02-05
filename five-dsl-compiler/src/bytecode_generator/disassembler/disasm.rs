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
                    lines.push(format!("{:04X}: PUSH_U8 {}", pc, bytes[pc + 1]));
                    pc += 2;
                } else {
                    lines.push(format!("{:04X}: PUSH_U8 <truncated>", pc));
                    break;
                }
            }
            opcodes::PUSH_U64 => {
                if let Some((v, c)) = decode_vle_u128(&bytes[pc + 1..]) {
                    lines.push(format!("{:04X}: PUSH_U64 (vle) {}", pc, v));
                    pc += 1 + c;
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
                if let Some((v, c)) = decode_vle_u128(&bytes[pc + 1..]) {
                    lines.push(format!("{:04X}: PUSH_I64 (vle) {}", pc, v as i128 as i64));
                    pc += 1 + c;
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
                    lines.push(format!("{:04X}: PUSH_BOOL {}", pc, bytes[pc + 1]));
                    pc += 2;
                } else {
                    lines.push(format!("{:04X}: PUSH_BOOL <truncated>", pc));
                    break;
                }
            }
            opcodes::PUSH_PUBKEY => {
                if pc + 33 <= bytes.len() {
                    lines.push(format!("{:04X}: PUSH_PUBKEY <32 bytes>", pc));
                    pc += 33
                } else {
                    lines.push(format!("{:04X}: PUSH_PUBKEY <truncated>", pc));
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
                if pc + 1 < bytes.len() {
                    let after = pc + 1;
                    if let Some((v, c)) = decode_vle_u128(&bytes[after..]) {
                        lines.push(format!("{:04X}: JUMP offset_vle={}", pc, v));
                        pc = after + c;
                    } else {
                        lines.push(format!("{:04X}: JUMP <truncated>", pc));
                        break;
                    }
                } else {
                    lines.push(format!("{:04X}: JUMP <truncated>", pc));
                    break;
                }
            }
            opcodes::JUMP_IF => {
                if pc + 1 < bytes.len() {
                    let after = pc + 1;
                    if let Some((v, c)) = decode_vle_u128(&bytes[after..]) {
                        lines.push(format!("{:04X}: JUMP_IF offset_vle={}", pc, v));
                        pc = after + c;
                    } else {
                        lines.push(format!("{:04X}: JUMP_IF <truncated>", pc));
                        break;
                    }
                } else {
                    lines.push(format!("{:04X}: JUMP_IF <truncated>", pc));
                    break;
                }
            }
            opcodes::JUMP_IF_NOT => {
                if pc + 1 < bytes.len() {
                    let after = pc + 1;
                    if let Some((v, c)) = decode_vle_u128(&bytes[after..]) {
                        lines.push(format!("{:04X}: JUMP_IF_NOT offset_vle={}", pc, v));
                        pc = after + c;
                    } else {
                        lines.push(format!("{:04X}: JUMP_IF_NOT <truncated>", pc));
                        break;
                    }
                } else {
                    lines.push(format!("{:04X}: JUMP_IF_NOT <truncated>", pc));
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
                if pc + 2 <= bytes.len() {
                    let acc = bytes[pc + 1];
                    let after = pc + 2;
                    if let Some((offset, c)) = decode_vle_u128(&bytes[after..]) {
                        if after + c < bytes.len() {
                            let param = bytes[after + c];
                            lines.push(format!("{:04X}: REQUIRE_GTE_U64 acc={} offset_vle={} param={}", pc, acc, offset, param));
                            pc = after + c + 1;
                        } else {
                            lines.push(format!("{:04X}: REQUIRE_GTE_U64 <truncated>", pc));
                            break;
                        }
                    } else {
                        lines.push(format!("{:04X}: REQUIRE_GTE_U64 <truncated>", pc));
                        break;
                    }
                } else {
                    lines.push(format!("{:04X}: REQUIRE_GTE_U64 <truncated>", pc));
                    break;
                }
            }
            opcodes::REQUIRE_NOT_BOOL => {
                if pc + 2 <= bytes.len() {
                    let acc = bytes[pc + 1];
                    let after = pc + 2;
                    if let Some((offset, c)) = decode_vle_u128(&bytes[after..]) {
                        lines.push(format!("{:04X}: REQUIRE_NOT_BOOL acc={} offset_vle={}", pc, acc, offset));
                        pc = after + c;
                    } else {
                        lines.push(format!("{:04X}: REQUIRE_NOT_BOOL <truncated>", pc));
                        break;
                    }
                } else {
                    lines.push(format!("{:04X}: REQUIRE_NOT_BOOL <truncated>", pc));
                    break;
                }
            }
            opcodes::FIELD_ADD_PARAM => {
                if pc + 2 <= bytes.len() {
                    let acc = bytes[pc + 1];
                    let after = pc + 2;
                    if let Some((offset, c)) = decode_vle_u128(&bytes[after..]) {
                        if after + c < bytes.len() {
                            let param = bytes[after + c];
                            lines.push(format!("{:04X}: FIELD_ADD_PARAM acc={} offset_vle={} param={}", pc, acc, offset, param));
                            pc = after + c + 1;
                        } else {
                            lines.push(format!("{:04X}: FIELD_ADD_PARAM <truncated>", pc));
                            break;
                        }
                    } else {
                        lines.push(format!("{:04X}: FIELD_ADD_PARAM <truncated>", pc));
                        break;
                    }
                } else {
                    lines.push(format!("{:04X}: FIELD_ADD_PARAM <truncated>", pc));
                    break;
                }
            }
            opcodes::FIELD_SUB_PARAM => {
                if pc + 2 <= bytes.len() {
                    let acc = bytes[pc + 1];
                    let after = pc + 2;
                    if let Some((offset, c)) = decode_vle_u128(&bytes[after..]) {
                        if after + c < bytes.len() {
                            let param = bytes[after + c];
                            lines.push(format!("{:04X}: FIELD_SUB_PARAM acc={} offset_vle={} param={}", pc, acc, offset, param));
                            pc = after + c + 1;
                        } else {
                            lines.push(format!("{:04X}: FIELD_SUB_PARAM <truncated>", pc));
                            break;
                        }
                    } else {
                        lines.push(format!("{:04X}: FIELD_SUB_PARAM <truncated>", pc));
                        break;
                    }
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
                if pc + 2 <= bytes.len() {
                    let acc1 = bytes[pc + 1];
                    let after1 = pc + 2;
                    if let Some((offset1, c1)) = decode_vle_u128(&bytes[after1..]) {
                        let after2 = after1 + c1;
                        if after2 < bytes.len() {
                            let acc2 = bytes[after2];
                            let after3 = after2 + 1;
                            if let Some((offset2, c2)) = decode_vle_u128(&bytes[after3..]) {
                                lines.push(format!("{:04X}: REQUIRE_EQ_PUBKEY acc1={} offset1_vle={} acc2={} offset2_vle={}", pc, acc1, offset1, acc2, offset2));
                                pc = after3 + c2;
                            } else {
                                lines.push(format!("{:04X}: REQUIRE_EQ_PUBKEY <truncated>", pc));
                                break;
                            }
                        } else {
                            lines.push(format!("{:04X}: REQUIRE_EQ_PUBKEY <truncated>", pc));
                            break;
                        }
                    } else {
                        lines.push(format!("{:04X}: REQUIRE_EQ_PUBKEY <truncated>", pc));
                        break;
                    }
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
                if pc + 2 <= bytes.len() {
                    let acc = bytes[pc + 1];
                    let after = pc + 2;
                    if let Some((offset, c)) = decode_vle_u128(&bytes[after..]) {
                        if after + c < bytes.len() {
                            let param = bytes[after + c];
                            lines.push(format!("{:04X}: STORE_PARAM_TO_FIELD acc={} offset_vle={} param={}", pc, acc, offset, param));
                            pc = after + c + 1;
                        } else {
                            lines.push(format!("{:04X}: STORE_PARAM_TO_FIELD <truncated>", pc));
                            break;
                        }
                    } else {
                        lines.push(format!("{:04X}: STORE_PARAM_TO_FIELD <truncated>", pc));
                        break;
                    }
                } else {
                    lines.push(format!("{:04X}: STORE_PARAM_TO_FIELD <truncated>", pc));
                    break;
                }
            }
            opcodes::STORE_FIELD_ZERO => {
                if pc + 2 <= bytes.len() {
                    let acc = bytes[pc + 1];
                    let after = pc + 2;
                    if let Some((offset, c)) = decode_vle_u128(&bytes[after..]) {
                        lines.push(format!("{:04X}: STORE_FIELD_ZERO acc={} offset_vle={}", pc, acc, offset));
                        pc = after + c;
                    } else {
                        lines.push(format!("{:04X}: STORE_FIELD_ZERO <truncated>", pc));
                        break;
                    }
                } else {
                    lines.push(format!("{:04X}: STORE_FIELD_ZERO <truncated>", pc));
                    break;
                }
            }
            opcodes::STORE_KEY_TO_FIELD => {
                if pc + 2 <= bytes.len() {
                    let acc = bytes[pc + 1];
                    let after = pc + 2;
                    if let Some((offset, c)) = decode_vle_u128(&bytes[after..]) {
                        if after + c < bytes.len() {
                            let key_acc = bytes[after + c];
                            lines.push(format!("{:04X}: STORE_KEY_TO_FIELD acc={} offset_vle={} key_acc={}", pc, acc, offset, key_acc));
                            pc = after + c + 1;
                        } else {
                            lines.push(format!("{:04X}: STORE_KEY_TO_FIELD <truncated>", pc));
                            break;
                        }
                    } else {
                        lines.push(format!("{:04X}: STORE_KEY_TO_FIELD <truncated>", pc));
                        break;
                    }
                } else {
                    lines.push(format!("{:04X}: STORE_KEY_TO_FIELD <truncated>", pc));
                    break;
                }
            }
            opcodes::REQUIRE_EQ_FIELDS => {
                if pc + 2 <= bytes.len() {
                    let acc1 = bytes[pc + 1];
                    let after1 = pc + 2;
                    if let Some((offset1, c1)) = decode_vle_u128(&bytes[after1..]) {
                        let after2 = after1 + c1;
                        if after2 < bytes.len() {
                            let acc2 = bytes[after2];
                            let after3 = after2 + 1;
                            if let Some((offset2, c2)) = decode_vle_u128(&bytes[after3..]) {
                                lines.push(format!("{:04X}: REQUIRE_EQ_FIELDS acc1={} offset1_vle={} acc2={} offset2_vle={}", pc, acc1, offset1, acc2, offset2));
                                pc = after3 + c2;
                            } else {
                                lines.push(format!("{:04X}: REQUIRE_EQ_FIELDS <truncated>", pc));
                                break;
                            }
                        } else {
                            lines.push(format!("{:04X}: REQUIRE_EQ_FIELDS <truncated>", pc));
                            break;
                        }
                    } else {
                        lines.push(format!("{:04X}: REQUIRE_EQ_FIELDS <truncated>", pc));
                        break;
                    }
                } else {
                    lines.push(format!("{:04X}: REQUIRE_EQ_FIELDS <truncated>", pc));
                    break;
                }
            }
            opcodes::LOAD_FIELD => {
                if pc + 2 <= bytes.len() {
                    let acc = bytes[pc + 1];
                    let after = pc + 2;
                    if let Some((v, c)) = decode_vle_u128(&bytes[after..]) {
                        lines.push(format!(
                            "{:04X}: LOAD_FIELD acc={} offset_vle={}",
                            pc, acc, v
                        ));
                        pc = after + c;
                    } else if let Some(raw) = read_le_u32(bytes, after) {
                        lines.push(format!("{:04X}: LOAD_FIELD acc={} offset={}", pc, acc, raw));
                        pc = after + 4;
                    } else {
                        lines.push(format!("{:04X}: LOAD_FIELD <truncated>", pc));
                        break;
                    }
                } else {
                    lines.push(format!("{:04X}: LOAD_FIELD <truncated>", pc));
                    break;
                }
            }
            opcodes::STORE_FIELD => {
                if pc + 2 <= bytes.len() {
                    let acc = bytes[pc + 1];
                    let after = pc + 2;
                    if let Some((v, c)) = decode_vle_u128(&bytes[after..]) {
                        lines.push(format!(
                            "{:04X}: STORE_FIELD acc={} offset_vle={}",
                            pc, acc, v
                        ));
                        pc = after + c
                    } else if let Some(raw) = read_le_u32(bytes, after) {
                        lines.push(format!(
                            "{:04X}: STORE_FIELD acc={} offset={}",
                            pc, acc, raw
                        ));
                        pc = after + 4
                    } else {
                        lines.push(format!("{:04X}: STORE_FIELD <truncated>", pc));
                        break;
                    }
                } else {
                    lines.push(format!("{:04X}: STORE_FIELD <truncated>", pc));
                    break;
                }
            }
            opcodes::CHECK_SIGNER => {
                if let Some((v, c)) = decode_vle_u128(&bytes[pc + 1..]) {
                    lines.push(format!("{:04X}: CHECK_SIGNER acc={}", pc, v));
                    pc += 1 + c;
                } else {
                    lines.push(format!("{:04X}: CHECK_SIGNER <truncated>", pc));
                    break;
                }
            }
            opcodes::CHECK_WRITABLE => {
                if let Some((v, c)) = decode_vle_u128(&bytes[pc + 1..]) {
                    lines.push(format!("{:04X}: CHECK_WRITABLE acc={}", pc, v));
                    pc += 1 + c;
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
            _ => {
                lines.push(format!("{:04X}: {}", pc, opcode_name(op)));
                pc += 1
            }
        }
    }
    lines
}

/// Human-friendly opcode name lookup.
fn opcode_name(op: u8) -> &'static str {
    five_protocol::opcodes::opcode_name(op)
}
