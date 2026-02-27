//! Pretty-printing and formatting functions for decoded instructions.

use crate::bytecode_generator::disassembler::disasm::disassemble;
use crate::bytecode_generator::disassembler::types::*;

/// Pretty-print a single decoded instruction into a human-friendly string.
pub fn pretty_instruction(instr: &Instruction) -> String {
    match instr {
        Instruction::PushU64(p) => format!(
            "{:04X}: {} value={} width={}",
            p.offset,
            opcode_name(p.opcode),
            p.value,
            p.width
        ),
        Instruction::Call(c) => {
            if let Some(ref name) = c.name_metadata {
                format!(
                    "{:04X}: CALL param_count={} addr=0x{:04X} name={}",
                    c.offset, c.param_count, c.function_address, name
                )
            } else {
                format!(
                    "{:04X}: CALL param_count={} addr=0x{:04X}",
                    c.offset, c.param_count, c.function_address
                )
            }
        }
        Instruction::SetLocal { offset, index } => format!("{:04X}: SET_LOCAL {}", offset, index),
        Instruction::GetLocal { offset, index } => format!("{:04X}: GET_LOCAL {}", offset, index),
        Instruction::AllocLocals { offset } => format!("{:04X}: ALLOC_LOCALS", offset),
        Instruction::DeallocLocals { offset } => format!("{:04X}: DEALLOC_LOCALS", offset),
        Instruction::LoadField {
            instr_offset,
            account_index,
            field_offset,
        } => format!(
            "{:04X}: LOAD_FIELD account_index={} offset={}",
            instr_offset, account_index, field_offset
        ),
        Instruction::StoreField {
            instr_offset,
            account_index,
            field_offset,
        } => format!(
            "{:04X}: STORE_FIELD account_index={} offset={}",
            instr_offset, account_index, field_offset
        ),
        Instruction::CallNative { offset, syscall_id } => {
            format!("{:04X}: CALL_NATIVE {}", offset, syscall_id)
        }
        Instruction::Invoke { offset } => format!("{:04X}: INVOKE", offset),
        Instruction::InvokeSigned { offset } => format!("{:04X}: INVOKE_SIGNED", offset),
        Instruction::PushStringLiteral { offset, len } => {
            format!("{:04X}: PUSH_STRING_LITERAL len={}", offset, len)
        }
        Instruction::PushArrayLiteral { offset, len } => {
            format!("{:04X}: PUSH_ARRAY_LITERAL len={}", offset, len)
        }
        Instruction::CheckSigner {
            offset,
            account_index,
        } => format!(
            "{:04X}: CHECK_SIGNER account_index={}",
            offset, account_index
        ),
        Instruction::CheckWritable {
            offset,
            account_index,
        } => format!(
            "{:04X}: CHECK_WRITABLE account_index={}",
            offset, account_index
        ),
        Instruction::GetKey {
            offset,
            account_index,
        } => format!("{:04X}: GET_KEY account_index={}", offset, account_index),
        Instruction::Opcode(op) => format!("OPCODE 0x{:02X}", op),
        Instruction::Unknown => "UNKNOWN".to_string(),
    }
}

/// Convenience wrapper returning a textual disassembly.
pub fn get_disassembly(bytes: &[u8]) -> Vec<String> {
    disassemble(bytes)
}

/// Human-friendly opcode name lookup.
fn opcode_name(op: u8) -> &'static str {
    five_protocol::opcodes::opcode_name(op)
}
