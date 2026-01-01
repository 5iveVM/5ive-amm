"use client";

import { useIdeStore } from "@/stores/ide-store";
import { GlassCard, GlassHeader } from "@/components/ui/glass-card";
import { RotateCcw, Cpu, Binary, FileCode, Hash, ArrowRight, Terminal as TerminalIcon, Copy, Check, Trash2 } from "lucide-react";
import { motion, AnimatePresence } from "framer-motion";
import { useState, useMemo, useEffect, useRef } from "react";

// --- Opcode Definitions (Subset based on protocol) ---
const UNKNOWN_OP = { name: 'UNKNOWN', args: 0 };
// @ts-ignore - Index signature mismatch is fine for this map
const OPCODE_MAP: Record<number, { name: string, args: string }> = {
    // Control Flow
    0x00: { name: 'HALT', args: 'none' },
    0x01: { name: 'JUMP', args: 'u16' },
    0x02: { name: 'JUMP_IF', args: 'u16' },
    0x03: { name: 'JUMP_IF_NOT', args: 'u16' },
    0x04: { name: 'REQUIRE', args: 'none' },
    0x05: { name: 'ASSERT', args: 'none' },
    0x06: { name: 'RETURN', args: 'none' },
    0x07: { name: 'RETURN_VALUE', args: 'none' },

    // Stack
    0x10: { name: 'POP', args: 'none' },
    0x11: { name: 'DUP', args: 'none' },
    0x12: { name: 'DUP2', args: 'none' },
    0x13: { name: 'SWAP', args: 'none' },
    0x14: { name: 'PICK', args: 'none' },
    0x15: { name: 'ROT', args: 'none' },
    0x16: { name: 'DROP', args: 'none' },
    0x17: { name: 'OVER', args: 'none' },

    // Pushes
    0x18: { name: 'PUSH_U8', args: 'u8' },
    0x19: { name: 'PUSH_U16', args: 'vle' },
    0x1A: { name: 'PUSH_U32', args: 'vle' },
    0x1B: { name: 'PUSH_U64', args: 'vle' },
    0x1C: { name: 'PUSH_I64', args: 'vle' },
    0x1D: { name: 'PUSH_BOOL', args: 'u8' },
    0x1E: { name: 'PUSH_PUBKEY', args: 'none' },
    0x1F: { name: 'PUSH_U128', args: 'none' },

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

    // Logical
    0x30: { name: 'AND', args: 'none' },
    0x31: { name: 'OR', args: 'none' },
    0x32: { name: 'NOT', args: 'none' },

    // Memory
    0x40: { name: 'STORE', args: 'none' },
    0x41: { name: 'LOAD', args: 'none' },
    0x42: { name: 'STORE_FIELD', args: 'vle' },
    0x43: { name: 'LOAD_FIELD', args: 'vle' },

    // Account
    0x50: { name: 'CREATE_ACCOUNT', args: 'none' },
    0x53: { name: 'GET_ACCOUNT', args: 'none' },
    0x56: { name: 'GET_DATA', args: 'none' },

    // System / Function
    0x80: { name: 'INVOKE', args: 'none' },
    0x90: { name: 'CALL', args: 'vle_fn' },

    // Locals
    0xA2: { name: 'SET_LOCAL', args: 'u8' },
    0xA3: { name: 'GET_LOCAL', args: 'u8' },
    0xA5: { name: 'LOAD_PARAM', args: 'u8' },
    0xA6: { name: 'STORE_PARAM', args: 'u8' },
    0xA9: { name: 'EMIT_EVENT', args: 'none' },
    0xAA: { name: 'LOG_DATA', args: 'none' },

    // Nibble Ops (Optimizations)
    0xD0: { name: 'GET_LOCAL_0', args: 'none' },
    0xD1: { name: 'GET_LOCAL_1', args: 'none' },
    0xD4: { name: 'SET_LOCAL_0', args: 'none' },
    0xD5: { name: 'SET_LOCAL_1', args: 'none' },
    0xD8: { name: 'PUSH_0', args: 'none' },
    0xD9: { name: 'PUSH_1', args: 'none' },

    // Fusion
    0xE0: { name: 'PUSH_ZERO', args: 'none' },
    0xE1: { name: 'PUSH_ONE', args: 'none' },
    0xE2: { name: 'DUP_ADD', args: 'none' },
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
                } else if (opDef.args === 'u16') {
                    // Attempt VLE decode for Jumps as per protocol
                    const { value, length } = decodeVLE(bytecode, pc);
                    argsStr = `+${value} (0x${value.toString(16)})`;
                    pc += length;
                } else if (opDef.args === 'vle' || opDef.args === 'vle_fn') {
                    const { value, length } = decodeVLE(bytecode, pc);
                    argsStr = `#${value}`;
                    pc += length;
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
