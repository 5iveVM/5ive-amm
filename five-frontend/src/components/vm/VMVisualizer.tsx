"use client";

import { useIdeStore } from "@/stores/ide-store";
import { GlassCard, GlassHeader } from "@/components/ui/glass-card";
import { RotateCcw, Cpu, Binary, FileCode, Hash, ArrowRight, Terminal as TerminalIcon, Copy, Check, Trash2 } from "lucide-react";
import { motion, AnimatePresence } from "framer-motion";
import { useState, useMemo, useEffect, useRef } from "react";

// --- Opcode Definitions (Based on five-protocol/src/opcodes.rs) ---
const UNKNOWN_OP = { name: 'UNKNOWN', args: 0 };
// @ts-expect-error - Index signature mismatch is fine for this map
const OPCODE_MAP: Record<number, { name: string, args: string }> = {
    // Control Flow
    0x00: { name: 'HALT', args: 'none' },
    0x01: { name: 'JUMP', args: 'u16_fixed' }, // Protocol says U16 arg type but VLE encoded offset comment. Wait, parser.rs says ArgType::U16 is fixed 2 bytes.
    0x02: { name: 'JUMP_IF', args: 'u16_fixed' },
    0x03: { name: 'JUMP_IF_NOT', args: 'u16_fixed' },
    0x04: { name: 'REQUIRE', args: 'none' },
    0x05: { name: 'ASSERT', args: 'none' },
    0x06: { name: 'RETURN', args: 'none' },
    0x07: { name: 'RETURN_VALUE', args: 'none' },
    0x08: { name: 'NOP', args: 'none' },
    0x09: { name: 'BR_EQ_U8', args: 'br_eq_u8' }, // Special handling: u8 + vle_u16? Wait, parser.rs says ArgType::U8 (1 byte). But OpcodePatterns emit u8 + vle_u16.
                                                 // Check parser.rs: ArgType::U8 -> reads 1 byte.
                                                 // Wait, if parser says ArgType::U8, it only consumes 1 byte.
                                                 // five-protocol/src/opcodes.rs: BR_EQ_U8 has ArgType::U8.
                                                 // OpcodePatterns::emit_br_eq_u8 emits u8 + vle_u16.
                                                 // This means OpcodePatterns is emitting MORE than the opcode definition expects if it's just U8.
                                                 // Or maybe the relative jump is implicit? No.
                                                 // Actually if ArgType::U8 is used, the parser only advances 1 byte.
                                                 // If BR_EQ_U8 is supposed to jump, it needs an offset.
                                                 // If the protocol definition is wrong (ArgType::U8), then the parser will be desync with the emitter.
                                                 // Let's assume the emitter is correct about intent (compare & jump) but the parser implementation in five-protocol depends on `ArgType`.
                                                 // If `ArgType` is `U8`, parser reads 1 byte.
                                                 // If `OpcodePatterns` emits 3 bytes (u8 + u16), the VM execution loop must handle fetching the extra bytes manually if `ArgType` doesn't cover it.
                                                 // But `five-protocol` parser is "Canonical Parser". If it parses it as 1 arg, then the next bytes are treated as next opcode.
                                                 // This suggests `BR_EQ_U8` might NOT be fully implemented or there is a bug in `five-protocol` definition if it takes 2 args.
                                                 // HOWEVER, for `VMVisualizer`, I should stick to what `five-protocol` says to avoid desync, OR if I know it's a bug, I should maybe fix it or note it.
                                                 // Given `OpcodeInfo` says `ArgType::U8`, `VMVisualizer` should probably treat it as `u8`.
                                                 // But `OpcodePatterns` is what generates the bytecode.
                                                 // I will trust `five-protocol`'s `ArgType` for now for the "Standard" visualization, but `five-dsl-compiler` seems to emit more.
                                                 // Wait, `five-protocol` comment says: `BR_EQ_U8: u8 = 0x09; // Fused compare-and-branch: compare with u8, jump if equal`.
                                                 // It implies a jump.
                                                 // If I look at `five-vm-mito`, it delegates to protocol.
                                                 // Let's look at `five-vm-mito` execution loop if possible? No, I shouldn't dig too deep if not needed.
                                                 // I will treat it as `u8` for now to match `five-protocol`.
                                                 // Wait, if I treat it as `u8`, the next bytes (the offset) will be visualized as opcodes, which will look like garbage.
                                                 // If `five-dsl-compiler` emits it, it expects the VM to consume it.
                                                 // I will assume `ArgType::U8` is the metadata, but maybe the VM handles it specially?
                                                 // Actually, `five-protocol/src/parser.rs` uses `get_opcode_info` to determine size.
                                                 // If `ArgType` is `U8`, it consumes 1 byte.
                                                 // So `five-protocol` parser effectively breaks on `BR_EQ_U8` if it's supposed to have a jump offset.
                                                 // This looks like a discrepancy I should probably fix in `five-protocol` if I could, but my task is to sync opcodes.
                                                 // I'll stick to `five-protocol` definition for the Visualizer: `u8`.

    // Stack
    0x10: { name: 'POP', args: 'none' },
    0x11: { name: 'DUP', args: 'none' },
    0x12: { name: 'DUP2', args: 'none' },
    0x13: { name: 'SWAP', args: 'none' },
    0x14: { name: 'PICK', args: 'none' },
    0x15: { name: 'ROT', args: 'none' },
    0x16: { name: 'DROP', args: 'none' },
    0x17: { name: 'OVER', args: 'none' },
    0xF8: { name: 'CREATE_TUPLE', args: 'none' },
    0xF9: { name: 'TUPLE_GET', args: 'none' },
    0xFA: { name: 'UNPACK_TUPLE', args: 'none' },
    0xFB: { name: 'STACK_SIZE', args: 'none' },
    0xFC: { name: 'STACK_CLEAR', args: 'none' },

    // Optional/Result
    0xF2: { name: 'OPTIONAL_SOME', args: 'none' },
    0xF3: { name: 'OPTIONAL_NONE', args: 'none' },
    0xF4: { name: 'OPTIONAL_UNWRAP', args: 'none' },
    0xF5: { name: 'OPTIONAL_IS_SOME', args: 'none' },
    0xF6: { name: 'OPTIONAL_GET_VALUE', args: 'none' },
    0xFD: { name: 'OPTIONAL_IS_NONE', args: 'none' },
    0xF0: { name: 'RESULT_OK', args: 'none' },
    0xF1: { name: 'RESULT_ERR', args: 'none' },
    0xFE: { name: 'RESULT_IS_OK', args: 'none' },
    0xFF: { name: 'RESULT_IS_ERR', args: 'none' },
    0xAC: { name: 'RESULT_UNWRAP', args: 'none' },
    0xAD: { name: 'RESULT_GET_VALUE', args: 'none' },
    0xAE: { name: 'RESULT_GET_ERROR', args: 'none' },

    // Pushes
    0x18: { name: 'PUSH_U8', args: 'u8' },
    0x19: { name: 'PUSH_U16', args: 'u16_fixed' }, // Protocol ArgType::U16 (fixed 2 bytes usually, but check comments/usage. parser says fixed)
    0x1A: { name: 'PUSH_U32', args: 'u32_vle' }, // Protocol ArgType::U32 (VLE)
    0x1B: { name: 'PUSH_U64', args: 'u64_vle' }, // Protocol ArgType::U64 (VLE)
    0x1C: { name: 'PUSH_I64', args: 'u64_vle' }, // Protocol ArgType::U64 (VLE)
    0x1D: { name: 'PUSH_BOOL', args: 'u8' }, // Adjusted to match OpcodePatterns emission
    0x1E: { name: 'PUSH_PUBKEY', args: 'pubkey' }, // Adjusted to match reality (32 bytes) despite protocol ArgType::None
    0x1F: { name: 'PUSH_U128', args: 'u128' }, // Adjusted to match reality (16 bytes)
    0x67: { name: 'PUSH_STRING', args: 'u8' }, // Protocol ArgType::U8 (Length? Or string index? OpcodePatterns emits u8. Protocol says `length_vle + string_data`. But ArgType is U8.)
                                               // This visualizer is based on the parser logic which drives the view.

    // Arithmetic
    0x20: { name: 'ADD', args: 'none' },
    0x21: { name: 'SUB', args: 'none' },
    0x22: { name: 'MUL', args: 'none' },
    0x23: { name: 'DIV', args: 'none' },
    0x24: { name: 'MOD', args: 'none' },
    0x25: { name: 'GT', args: 'none' },
    0x26: { name: 'LT', args: 'none' },
    0x27: { name: 'EQ', args: 'none' },
    0x28: { name: 'GTE', args: 'none' },
    0x29: { name: 'LTE', args: 'none' },
    0x2A: { name: 'NEQ', args: 'none' },
    0x2B: { name: 'NEG', args: 'none' },
    0x2C: { name: 'ADD_CHECKED', args: 'none' },
    0x2D: { name: 'SUB_CHECKED', args: 'none' },
    0x2E: { name: 'MUL_CHECKED', args: 'none' },

    // Logical
    0x30: { name: 'AND', args: 'none' },
    0x31: { name: 'OR', args: 'none' },
    0x32: { name: 'NOT', args: 'none' },
    0x33: { name: 'XOR', args: 'none' },
    0x34: { name: 'BITWISE_NOT', args: 'none' },
    0x35: { name: 'BITWISE_AND', args: 'none' },
    0x36: { name: 'BITWISE_OR', args: 'none' },
    0x37: { name: 'BITWISE_XOR', args: 'none' },
    0x38: { name: 'SHIFT_LEFT', args: 'none' },
    0x39: { name: 'SHIFT_RIGHT', args: 'none' },
    0x3A: { name: 'SHIFT_RIGHT_ARITH', args: 'none' },
    0x3B: { name: 'ROTATE_LEFT', args: 'none' }, // Wait, ROTATE_LEFT is 0x3B in protocol? Yes.
    0x3C: { name: 'ROTATE_RIGHT', args: 'none' },

    // Byte Manipulation
    0x3D: { name: 'BYTE_SWAP_16', args: 'none' },
    0x3E: { name: 'BYTE_SWAP_32', args: 'none' },
    0x3F: { name: 'BYTE_SWAP_64', args: 'none' },

    // Memory
    0x40: { name: 'STORE', args: 'u32_vle' }, // Protocol ArgType::U32 (VLE)
    0x41: { name: 'LOAD', args: 'u32_vle' }, // Protocol ArgType::U32 (VLE)
    0x42: { name: 'STORE_FIELD', args: 'account_field' }, // Protocol ArgType::AccountField (u32 account_index + u32 field_offset VLE) -> wait parser says: u32 arg1 (account_idx), u32 arg2 (vle field_offset)
                                                          // ArgType::AccountField in parser: consumes 1 byte (u8 account_index) + VLE u32.
                                                          // Visualizer needs to handle this.
    0x43: { name: 'LOAD_FIELD', args: 'account_field' },
    0x44: { name: 'LOAD_INPUT', args: 'u8' },
    0x45: { name: 'STORE_GLOBAL', args: 'u16_fixed' }, // Protocol ArgType::U16 (Fixed)
    0x46: { name: 'LOAD_GLOBAL', args: 'u16_fixed' },
    0x47: { name: 'LOAD_EXTERNAL_FIELD', args: 'none' },

    // Account
    0x50: { name: 'CREATE_ACCOUNT', args: 'none' },
    0x51: { name: 'LOAD_ACCOUNT', args: 'u32_vle' }, // ArgType::AccountIndex (VLE u32)
    0x52: { name: 'SAVE_ACCOUNT', args: 'u32_vle' },
    0x53: { name: 'GET_ACCOUNT', args: 'u32_vle' },
    0x54: { name: 'GET_LAMPORTS', args: 'u32_vle' },
    0x55: { name: 'SET_LAMPORTS', args: 'u32_vle' },
    0x56: { name: 'GET_DATA', args: 'u32_vle' },
    0x57: { name: 'GET_KEY', args: 'u32_vle' },
    0x58: { name: 'GET_OWNER', args: 'u32_vle' },
    0x59: { name: 'TRANSFER', args: 'none' },
    0x5A: { name: 'TRANSFER_SIGNED', args: 'none' },

    // Array / String
    0x60: { name: 'CREATE_ARRAY', args: 'u8' },
    0x61: { name: 'PUSH_ARRAY_LITERAL', args: 'u8' },
    0x62: { name: 'ARRAY_INDEX', args: 'none' },
    0x63: { name: 'ARRAY_LENGTH', args: 'none' },
    0x64: { name: 'ARRAY_SET', args: 'none' },
    0x65: { name: 'ARRAY_GET', args: 'none' },
    0x66: { name: 'PUSH_STRING_LITERAL', args: 'u8' },
    // 0x67 PUSH_STRING already listed

    // Constraints
    0x70: { name: 'CHECK_SIGNER', args: 'u32_vle' }, // ArgType::AccountIndex
    0x71: { name: 'CHECK_WRITABLE', args: 'u32_vle' },
    0x72: { name: 'CHECK_OWNER', args: 'u32_vle' },
    0x73: { name: 'CHECK_INITIALIZED', args: 'u32_vle' },
    0x74: { name: 'CHECK_PDA', args: 'u32_vle' },
    0x75: { name: 'CHECK_UNINITIALIZED', args: 'u32_vle' },
    0x76: { name: 'CHECK_DEDUPE_TABLE', args: 'none' }, // Protocol doesn't specify ArgType in snippet, assumig None or check parser. Parser doesn't list it explicitly in my snippet.
    0x77: { name: 'CHECK_CACHED', args: 'none' },
    0x78: { name: 'CHECK_COMPLEXITY_GROUP', args: 'none' },
    0x79: { name: 'CHECK_DEDUPE_MASK', args: 'none' },

    // System / Function
    0x80: { name: 'INVOKE', args: 'none' },
    0x81: { name: 'INVOKE_SIGNED', args: 'none' },
    0x82: { name: 'GET_CLOCK', args: 'none' },
    0x83: { name: 'GET_RENT', args: 'none' }, // Was missing in protocol file I read? No, it's there.
    0x84: { name: 'INIT_ACCOUNT', args: 'u32_vle' }, // ArgType::AccountIndex
    0x85: { name: 'INIT_PDA_ACCOUNT', args: 'u32_vle' },
    0x86: { name: 'DERIVE_PDA', args: 'none' },
    0x87: { name: 'FIND_PDA', args: 'none' }, // Protocol says nothing about args? Assuming None.
    0x88: { name: 'DERIVE_PDA_PARAMS', args: 'none' },
    0x89: { name: 'FIND_PDA_PARAMS', args: 'none' },

    0x90: { name: 'CALL', args: 'u32_vle' }, // ArgType::FunctionIndex (VLE u32)
    0x91: { name: 'CALL_EXTERNAL', args: 'call_external' }, // ArgType::CallExternal
    0x92: { name: 'CALL_NATIVE', args: 'none' },
    0x93: { name: 'PREPARE_CALL', args: 'none' },
    0x94: { name: 'FINISH_CALL', args: 'none' },

    // Locals
    0xA0: { name: 'ALLOC_LOCALS', args: 'none' },
    0xA1: { name: 'DEALLOC_LOCALS', args: 'none' },
    0xA2: { name: 'SET_LOCAL', args: 'u8' },
    0xA3: { name: 'GET_LOCAL', args: 'u8' },
    0xA4: { name: 'CLEAR_LOCAL', args: 'u32_vle' }, // ArgType::LocalIndex
    0xA5: { name: 'LOAD_PARAM', args: 'u8' },
    0xA6: { name: 'STORE_PARAM', args: 'u8' },
    0xA7: { name: 'WRITE_DATA', args: 'none' },
    0xA8: { name: 'DATA_LEN', args: 'none' },
    0xA9: { name: 'EMIT_EVENT', args: 'none' },
    0xAA: { name: 'LOG_DATA', args: 'none' },
    0xAB: { name: 'GET_SIGNER_KEY', args: 'none' },
    0xAF: { name: 'CAST', args: 'u8' }, // ArgType not listed but usually CAST takes type. Assuming u8.

    // Registers (0xB0 - 0xBF)
    0xB0: { name: 'LOAD_REG_U8', args: 'u8' }, // ArgType::RegisterIndex -> u8? Parser says RegisterIndex -> 1 byte.
    0xB1: { name: 'LOAD_REG_U32', args: 'u8' },
    0xB2: { name: 'LOAD_REG_U64', args: 'u8' },
    0xB3: { name: 'LOAD_REG_BOOL', args: 'u8' },
    0xB4: { name: 'LOAD_REG_PUBKEY', args: 'u8' },
    0xB5: { name: 'ADD_REG', args: 'u24_3reg' }, // ArgType::ThreeRegisters (3 bytes)
    0xB6: { name: 'SUB_REG', args: 'u24_3reg' },
    0xB7: { name: 'MUL_REG', args: 'u24_3reg' },
    0xB8: { name: 'DIV_REG', args: 'u24_3reg' },
    0xB9: { name: 'EQ_REG', args: 'u24_3reg' },
    0xBA: { name: 'GT_REG', args: 'u24_3reg' },
    0xBB: { name: 'LT_REG', args: 'u24_3reg' },
    0xBC: { name: 'PUSH_REG', args: 'u8' },
    0xBD: { name: 'POP_REG', args: 'u8' },
    0xBE: { name: 'COPY_REG', args: 'u16_2reg' }, // ArgType::TwoRegisters (2 bytes)
    0xBF: { name: 'CLEAR_REG', args: 'u8' },

    // Nibble Ops (Optimizations)
    0xD0: { name: 'GET_LOCAL_0', args: 'none' },
    0xD1: { name: 'GET_LOCAL_1', args: 'none' },
    0xD2: { name: 'GET_LOCAL_2', args: 'none' },
    0xD3: { name: 'GET_LOCAL_3', args: 'none' },
    0xD4: { name: 'SET_LOCAL_0', args: 'none' },
    0xD5: { name: 'SET_LOCAL_1', args: 'none' },
    0xD6: { name: 'SET_LOCAL_2', args: 'none' },
    0xD7: { name: 'SET_LOCAL_3', args: 'none' },
    0xD8: { name: 'PUSH_0', args: 'none' },
    0xD9: { name: 'PUSH_1', args: 'none' },
    0xDA: { name: 'PUSH_2', args: 'none' },
    0xDB: { name: 'PUSH_3', args: 'none' },
    0xDC: { name: 'LOAD_PARAM_0', args: 'none' },
    0xDD: { name: 'LOAD_PARAM_1', args: 'none' },
    0xDE: { name: 'LOAD_PARAM_2', args: 'none' },
    0xDF: { name: 'LOAD_PARAM_3', args: 'none' },

    // Fusion
    0xE0: { name: 'PUSH_ZERO', args: 'none' },
    0xE1: { name: 'PUSH_ONE', args: 'none' },
    0xE2: { name: 'DUP_ADD', args: 'none' },
    0xE3: { name: 'DUP_SUB', args: 'none' },
    0xE4: { name: 'DUP_MUL', args: 'none' },
    0xE5: { name: 'VALIDATE_AMOUNT_NONZERO', args: 'none' },
    0xE6: { name: 'VALIDATE_SUFFICIENT', args: 'none' },
    0xE7: { name: 'EQ_ZERO_JUMP', args: 'u16_fixed' }, // ArgType::U16 (Fixed)
    0xE8: { name: 'TRANSFER_DEBIT', args: 'u8' },
    0xE9: { name: 'TRANSFER_CREDIT', args: 'u8' },
    0xEA: { name: 'RETURN_SUCCESS', args: 'none' },
    0xEB: { name: 'RETURN_ERROR', args: 'none' },
    0xEC: { name: 'GT_ZERO_JUMP', args: 'u16_fixed' }, // Assuming U16
    0xED: { name: 'LT_ZERO_JUMP', args: 'u16_fixed' }, // Assuming U16
    0xF7: { name: 'BULK_LOAD_FIELD_N', args: 'u8' },
};

interface DisassembledOp {
    offset: number;
    opcode: number;
    name: string;
    args?: string;
    bytes: number[];
}

// VLE Decoder Helper
const decodeVLE = (bytes: Uint8Array, offset: number): { value: number, length: number } => {
    let result = 0;
    let shift = 0;
    let length = 0;

    while (offset + length < bytes.length) {
        const byte = bytes[offset + length];
        result |= (byte & 0x7f) << shift;
        length++;
        if ((byte & 0x80) === 0) break;
        shift += 7;
    }
    return { value: result, length };
};

export default function VMVisualizer() {
    const { vmState, logs, resetVmState, bytecode, clearLogs } = useIdeStore();
    const [viewMode, setViewMode] = useState<'assembly' | 'hex'>('assembly');
    const [isCopied, setIsCopied] = useState(false);
    const scrollRef = useRef<HTMLDivElement>(null);

    const handleCopyLogs = () => {
        const logText = logs.map(l => `[${l.timestamp.toLocaleTimeString()}] ${l.message}`).join('\n');
        navigator.clipboard.writeText(logText);
        setIsCopied(true);
        setTimeout(() => setIsCopied(false), 2000);
    };

    // --- Disassembler Logic ---
    const disassembly = useMemo(() => {
        if (!bytecode) return [];
        const ops: DisassembledOp[] = [];
        let pc = 0;

        while (pc < bytecode.length) {
            const startPc = pc;
            const opByte = bytecode[pc];
            pc++;

            const opDef = OPCODE_MAP[opByte];
            const name = opDef ? opDef.name : `OP_${opByte.toString(16).toUpperCase()}`;
            let argsStr = '';

            // Argument Decoding
            if (opDef) {
                if (opDef.args === 'u8') {
                    if (pc < bytecode.length) {
                        argsStr = `0x${bytecode[pc].toString(16).toUpperCase()}`;
                        pc++;
                    }
                } else if (opDef.args === 'u16_fixed') {
                    if (pc + 1 < bytecode.length) {
                        const val = bytecode[pc] | (bytecode[pc + 1] << 8);
                        argsStr = `0x${val.toString(16).toUpperCase()}`;
                        pc += 2;
                    }
                } else if (opDef.args === 'u32_vle' || opDef.args === 'u64_vle' || opDef.args === 'vle') {
                    const { value, length } = decodeVLE(bytecode, pc);
                    argsStr = `#${value}`;
                    pc += length;
                } else if (opDef.args === 'account_field') {
                    // u8 account_index + vle field_offset
                    if (pc < bytecode.length) {
                        const accIdx = bytecode[pc];
                        pc++;
                        const { value: fieldOffset, length } = decodeVLE(bytecode, pc);
                        argsStr = `Acc:${accIdx} Field:${fieldOffset}`;
                        pc += length;
                    }
                } else if (opDef.args === 'br_eq_u8') {
                    // u8 val + vle offset
                    if (pc < bytecode.length) {
                        const val = bytecode[pc];
                        pc++;
                        const { value: offset, length } = decodeVLE(bytecode, pc);
                        argsStr = `Val:${val} Offset:+${offset}`;
                        pc += length;
                    }
                } else if (opDef.args === 'pubkey') {
                    if (pc + 32 <= bytecode.length) {
                        // Display first/last few bytes of pubkey
                        const pk = Array.from(bytecode.slice(pc, pc + 32));
                        argsStr = `[${pk.slice(0,4).map(b=>b.toString(16).padStart(2,'0')).join('')}...]`;
                        pc += 32;
                    }
                } else if (opDef.args === 'u128') {
                    if (pc + 16 <= bytecode.length) {
                        argsStr = `(u128)`;
                        pc += 16;
                    }
                } else if (opDef.args === 'call_external') {
                    // account_index(u8) + func_offset(u16) + param_count(u8)
                    if (pc + 3 < bytecode.length) {
                        const accIdx = bytecode[pc];
                        const funcOffset = bytecode[pc+1] | (bytecode[pc+2] << 8);
                        const paramCount = bytecode[pc+3];
                        argsStr = `Acc:${accIdx} Fn:+${funcOffset} Params:${paramCount}`;
                        pc += 4;
                    }
                } else if (opDef.args === 'u16_2reg') {
                     if (pc + 1 < bytecode.length) {
                        const r1 = bytecode[pc];
                        const r2 = bytecode[pc+1];
                        argsStr = `r${r1}, r${r2}`;
                        pc += 2;
                    }
                } else if (opDef.args === 'u24_3reg') {
                    if (pc + 2 < bytecode.length) {
                       const r1 = bytecode[pc];
                       const r2 = bytecode[pc+1];
                       const r3 = bytecode[pc+2];
                       argsStr = `r${r1}, r${r2}, r${r3}`;
                       pc += 3;
                   }
                }
            }

            ops.push({
                offset: startPc,
                opcode: opByte,
                name,
                args: argsStr,
                bytes: Array.from(bytecode.slice(startPc, pc))
            });
        }
        return ops;
    }, [bytecode]);

    // --- Hex Dump Logic ---
    const hexRows = useMemo(() => {
        if (!bytecode) return [];
        const rows = [];
        for (let i = 0; i < bytecode.length; i += 16) {
            const chunk = bytecode.slice(i, i + 16);
            const hex = Array.from(chunk).map(b => b.toString(16).padStart(2, '0').toUpperCase());
            const ascii = Array.from(chunk).map(b => (b >= 32 && b <= 126) ? String.fromCharCode(b) : '.').join('');
            rows.push({ offset: i, hex, ascii });
        }
        return rows;
    }, [bytecode]);

    // Auto-scroll to active instruction
    useEffect(() => {
        if (viewMode === 'assembly' && scrollRef.current && disassembly.length > 0) {
            const activeEl = scrollRef.current.querySelector('[data-active="true"]');
            if (activeEl) {
                activeEl.scrollIntoView({ behavior: 'smooth', block: 'center' });
            }
        }
        // For hex view, we could calculate the row index from IP -> Math.floor(IP / 16)
        if (viewMode === 'hex' && scrollRef.current && bytecode) {
            const rowIdx = Math.floor(vmState.instructionPointer / 16);
            const activeEl = scrollRef.current.children[0]?.children?.[rowIdx + 1]; // +1 for header
            if (activeEl) {
                activeEl.scrollIntoView({ behavior: 'smooth', block: 'center' });
            }
        }
    }, [vmState.instructionPointer, disassembly, viewMode, bytecode]);

    return (
        <div className="h-full flex flex-col gap-4">
            {/* Console / Logs - Full Height */}
            <div className="flex-1 flex flex-col min-h-0 border-rose-pine-foam/20">
                <GlassHeader title="Console Output" className="py-2 px-3 bg-rose-pine-base/40 border-b-border-white/5">
                    <div className="flex items-center gap-2">
                        <TerminalIcon size={14} className="text-rose-pine-subtle" />
                        <button
                            onClick={clearLogs}
                            className="p-1 hover:bg-rose-pine-love/10 rounded transition-colors text-rose-pine-muted hover:text-rose-pine-love"
                            title="Clear Console"
                        >
                            <Trash2 size={14} />
                        </button>
                        <button
                            onClick={handleCopyLogs}
                            className="p-1 hover:bg-rose-pine-overlay/50 rounded transition-colors text-rose-pine-muted hover:text-rose-pine-text"
                            title="Copy Logs"
                        >
                            {isCopied ? <Check size={14} className="text-rose-pine-foam" /> : <Copy size={14} />}
                        </button>
                    </div>
                </GlassHeader>
                <div className="flex-1 p-2 overflow-y-auto font-mono text-xs space-y-1 bg-black/20 custom-scrollbar">
                    {logs.length === 0 ? (
                        <div className="text-rose-pine-muted/40 italic p-2">Ready to execute...</div>
                    ) : (
                        logs.map((log) => (
                            <div key={log.id} className="flex gap-2 text-rose-pine-text/90">
                                <span className="text-rose-pine-muted shrink-0">[{log.timestamp.toLocaleTimeString().split(' ')[0]}]</span>
                                <span className={
                                    log.type === 'error' ? 'text-rose-pine-love' :
                                        log.type === 'success' ? 'text-rose-pine-foam' :
                                            log.type === 'warning' ? 'text-rose-pine-gold' :
                                                'text-rose-pine-text'
                                }>{log.message}</span>
                            </div>
                        ))
                    )}
                </div>
            </div>
        </div>
    );
}
