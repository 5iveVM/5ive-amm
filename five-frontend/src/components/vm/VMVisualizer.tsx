"use client";

import { useIdeStore } from "@/stores/ide-store";
import { GlassCard, GlassHeader } from "@/components/ui/glass-card";
import { RotateCcw, Cpu, Binary, FileCode, Hash, ArrowRight, Terminal as TerminalIcon, Copy, Check, Trash2 } from "lucide-react";
import { motion, AnimatePresence } from "framer-motion";
import { useState, useMemo, useEffect, useRef } from "react";

// --- Opcode Definitions (Based on five-protocol/src/opcodes.rs) ---
const UNKNOWN_OP = { name: 'UNKNOWN', args: 0 };
// @ts-ignore - Index signature mismatch is fine for this map
const OPCODE_MAP: Record<number, { name: string, args: string }> = {
    // Control Flow
    0x00: { name: 'HALT', args: 'none' },
    0x01: { name: 'JUMP', args: 'u16_fixed' },
    0x02: { name: 'JUMP_IF', args: 'u16_fixed' },
    0x03: { name: 'JUMP_IF_NOT', args: 'u16_fixed' },
    0x04: { name: 'REQUIRE', args: 'none' },
    0x05: { name: 'ASSERT', args: 'none' },
    0x06: { name: 'RETURN', args: 'none' },
    0x07: { name: 'RETURN_VALUE', args: 'none' },
    0x08: { name: 'NOP', args: 'none' },
    0x09: { name: 'BR_EQ_U8', args: 'br_eq_u8' }, // u8 val + u16 offset

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
    0x19: { name: 'PUSH_U16', args: 'u16_fixed' },
    0x1A: { name: 'PUSH_U32', args: 'u32_fixed' },
    0x1B: { name: 'PUSH_U64', args: 'u64_fixed' },
    0x1C: { name: 'PUSH_I64', args: 'u64_fixed' }, // i64 as u64
    0x1D: { name: 'PUSH_BOOL', args: 'u8' },
    0x1E: { name: 'PUSH_PUBKEY', args: 'pubkey' },
    0x1F: { name: 'PUSH_U128', args: 'u128' },
    0x67: { name: 'PUSH_STRING', args: 'string_fixed' }, // u32 len + bytes

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
    0x3B: { name: 'ROTATE_LEFT', args: 'none' },
    0x3C: { name: 'ROTATE_RIGHT', args: 'none' },

    // Byte Manipulation
    0x3D: { name: 'BYTE_SWAP_16', args: 'none' },
    0x3E: { name: 'BYTE_SWAP_32', args: 'none' },
    0x3F: { name: 'BYTE_SWAP_64', args: 'none' },

    // Memory
    0x40: { name: 'STORE', args: 'u32_fixed' },
    0x41: { name: 'LOAD', args: 'u32_fixed' },
    0x42: { name: 'STORE_FIELD', args: 'account_field' },
    0x43: { name: 'LOAD_FIELD', args: 'account_field' },
    0x44: { name: 'LOAD_INPUT', args: 'u8' },
    0x45: { name: 'STORE_GLOBAL', args: 'u16_fixed' },
    0x46: { name: 'LOAD_GLOBAL', args: 'u16_fixed' },
    0x47: { name: 'LOAD_EXTERNAL_FIELD', args: 'none' },

    // Account
    0x50: { name: 'CREATE_ACCOUNT', args: 'none' },
    0x51: { name: 'LOAD_ACCOUNT', args: 'u32_fixed' },
    0x52: { name: 'SAVE_ACCOUNT', args: 'u32_fixed' },
    0x53: { name: 'GET_ACCOUNT', args: 'u32_fixed' },
    0x54: { name: 'GET_LAMPORTS', args: 'u32_fixed' },
    0x55: { name: 'SET_LAMPORTS', args: 'u32_fixed' },
    0x56: { name: 'GET_DATA', args: 'u32_fixed' },
    0x57: { name: 'GET_KEY', args: 'u32_fixed' },
    0x58: { name: 'GET_OWNER', args: 'u32_fixed' },
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

    // Constraints
    0x70: { name: 'CHECK_SIGNER', args: 'u32_fixed' },
    0x71: { name: 'CHECK_WRITABLE', args: 'u32_fixed' },
    0x72: { name: 'CHECK_OWNER', args: 'u32_fixed' },
    0x73: { name: 'CHECK_INITIALIZED', args: 'u32_fixed' },
    0x74: { name: 'CHECK_PDA', args: 'u32_fixed' },
    0x75: { name: 'CHECK_UNINITIALIZED', args: 'u32_fixed' },
    0x76: { name: 'CHECK_DEDUPE_TABLE', args: 'none' },
    0x77: { name: 'CHECK_CACHED', args: 'none' },
    0x78: { name: 'CHECK_COMPLEXITY_GROUP', args: 'none' },
    0x79: { name: 'CHECK_DEDUPE_MASK', args: 'none' },

    // System / Function
    0x80: { name: 'INVOKE', args: 'none' },
    0x81: { name: 'INVOKE_SIGNED', args: 'none' },
    0x82: { name: 'GET_CLOCK', args: 'none' },
    0x83: { name: 'GET_RENT', args: 'none' },
    0x84: { name: 'INIT_ACCOUNT', args: 'u32_fixed' },
    0x85: { name: 'INIT_PDA_ACCOUNT', args: 'u32_fixed' },
    0x86: { name: 'DERIVE_PDA', args: 'none' },
    0x87: { name: 'FIND_PDA', args: 'none' },
    0x88: { name: 'DERIVE_PDA_PARAMS', args: 'none' },
    0x89: { name: 'FIND_PDA_PARAMS', args: 'none' },

    0x90: { name: 'CALL', args: 'u32_fixed' },
    0x91: { name: 'CALL_EXTERNAL', args: 'call_external' },
    0x92: { name: 'CALL_NATIVE', args: 'none' },
    0x93: { name: 'PREPARE_CALL', args: 'none' },
    0x94: { name: 'FINISH_CALL', args: 'none' },

    // Locals
    0xA0: { name: 'ALLOC_LOCALS', args: 'none' },
    0xA1: { name: 'DEALLOC_LOCALS', args: 'none' },
    0xA2: { name: 'SET_LOCAL', args: 'u8' },
    0xA3: { name: 'GET_LOCAL', args: 'u8' },
    0xA4: { name: 'CLEAR_LOCAL', args: 'u32_fixed' },
    0xA5: { name: 'LOAD_PARAM', args: 'u8' },
    0xA6: { name: 'STORE_PARAM', args: 'u8' },
    0xA7: { name: 'WRITE_DATA', args: 'none' },
    0xA8: { name: 'DATA_LEN', args: 'none' },
    0xA9: { name: 'EMIT_EVENT', args: 'none' },
    0xAA: { name: 'LOG_DATA', args: 'none' },
    0xAB: { name: 'GET_SIGNER_KEY', args: 'none' },
    0xAF: { name: 'CAST', args: 'u8' },

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
    0xE7: { name: 'EQ_ZERO_JUMP', args: 'u16_fixed' },
    0xE8: { name: 'TRANSFER_DEBIT', args: 'u8' },
    0xE9: { name: 'TRANSFER_CREDIT', args: 'u8' },
    0xEA: { name: 'RETURN_SUCCESS', args: 'none' },
    0xEB: { name: 'RETURN_ERROR', args: 'none' },
    0xEC: { name: 'GT_ZERO_JUMP', args: 'u16_fixed' },
    0xED: { name: 'LT_ZERO_JUMP', args: 'u16_fixed' },
    0xF7: { name: 'BULK_LOAD_FIELD_N', args: 'u8' },
};

interface DisassembledOp {
    offset: number;
    opcode: number;
    name: string;
    args?: string;
    bytes: number[];
}

// Helpers for decoding fixed-size little-endian integers
const decodeU16 = (bytes: Uint8Array, offset: number): number => {
    return bytes[offset] | (bytes[offset + 1] << 8);
};

const decodeU32 = (bytes: Uint8Array, offset: number): number => {
    return (
        bytes[offset] |
        (bytes[offset + 1] << 8) |
        (bytes[offset + 2] << 16) |
        (bytes[offset + 3] << 24)
    ) >>> 0; // Ensure unsigned
};

const decodeU64 = (bytes: Uint8Array, offset: number): bigint => {
    const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
    return view.getBigUint64(offset, true); // true = little endian
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
                        const val = decodeU16(bytecode, pc);
                        argsStr = `0x${val.toString(16).toUpperCase()}`;
                        pc += 2;
                    }
                } else if (opDef.args === 'u32_fixed') {
                    if (pc + 3 < bytecode.length) {
                        const val = decodeU32(bytecode, pc);
                        argsStr = `#${val}`;
                        pc += 4;
                    }
                } else if (opDef.args === 'u64_fixed') {
                     if (pc + 7 < bytecode.length) {
                         const val = decodeU64(bytecode, pc);
                         argsStr = `#${val}`;
                         pc += 8;
                     }
                } else if (opDef.args === 'account_field') {
                    // u8 account_index + u32 field_offset
                    if (pc + 4 < bytecode.length) {
                        const accIdx = bytecode[pc];
                        pc++;
                        const fieldOffset = decodeU32(bytecode, pc);
                        argsStr = `Acc:${accIdx} Field:${fieldOffset}`;
                        pc += 4;
                    }
                } else if (opDef.args === 'br_eq_u8') {
                    // u8 val + u16 offset
                    if (pc + 2 < bytecode.length) {
                        const val = bytecode[pc];
                        pc++;
                        const offset = decodeU16(bytecode, pc);
                        argsStr = `Val:${val} Offset:+${offset}`;
                        pc += 2;
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
                        const funcOffset = decodeU16(bytecode, pc+1);
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
                } else if (opDef.args === 'string_fixed') {
                    // u32 length + bytes
                    if (pc + 3 < bytecode.length) {
                        const len = decodeU32(bytecode, pc);
                        pc += 4;
                        if (pc + len <= bytecode.length) {
                             const strBytes = bytecode.slice(pc, pc + len);
                             const str = new TextDecoder().decode(strBytes);
                             // Truncate long strings for display
                             argsStr = `"${str.length > 20 ? str.substring(0, 17) + '...' : str}"`;
                             pc += len;
                        } else {
                            argsStr = `<TRUNCATED STRING len=${len}>`;
                        }
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
