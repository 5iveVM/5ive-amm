//! Textual bytecode disassembly (human-readable output).

use crate::bytecode_generator::disassembler::call_decoder::*;
use crate::bytecode_generator::disassembler::decoder::*;
use five_protocol::opcodes;

/// Produce a textual disassembly (one line per instruction).
pub fn disassemble(bytes: &[u8]) -> Vec<String> {
    let mut lines = Vec::new();
    let mut pc = 0usize;
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
