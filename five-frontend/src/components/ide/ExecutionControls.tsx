"use client";

import { useIdeStore } from "@/stores/ide-store";
import { ChevronDown, Sparkles, Loader2, Play, Globe, Cpu, Coins, Terminal, Dices, Copy, Check } from "lucide-react";
import { cn } from "@/lib/utils";
import { LAMPORTS_PER_SOL, Keypair } from "@solana/web3.js";
import { useState, useEffect } from "react";

/**
 * Execution Controls Component
 * 
 * Redesigned for improved DevEX:
 * 1. Segmented Control for Mode Switching
 * 2. Visual Hierarchy (Config -> Inputs -> Actions)
 * 3. Integrated Deployment Cost Widget
 * 4. Structured Inputs with Helper Tools (Random Keypair, etc.)
 */
interface ExecutionControlsProps {
    onRun: () => void;
    isExecuting: boolean;
    isOnChain: boolean;
    onToggleMode: (isOnChain: boolean) => void;
    estimatedCost: number | null;
    solPrice: number;
}

export default function ExecutionControls({
    onRun,
    isExecuting,
    isOnChain,
    onToggleMode,
    estimatedCost,
    solPrice
}: ExecutionControlsProps) {
    const {
        abi,
        selectedFunctionIndex,
        setSelectedFunctionIndex,
        executionParams,
        setExecutionParams,
        executionAccounts,
        setExecutionAccounts,
        parseTestParams,
        bytecode,
        estimatedRent,
        estimatedDeployFee,
        deployFeeLamports,
    } = useIdeStore();

    // Local state to track copied status for keypair generation
    const [copiedIndex, setCopiedIndex] = useState<{ type: 'param' | 'account', index: number } | null>(null);

    // If no ABI/Bytecode, show empty state or basic message
    const functions = abi?.functions || [];
    const functionList = Array.isArray(functions) ? functions : Object.values(functions);
    const selectedFunction = functionList.find((f: any) => f.index === selectedFunctionIndex) || functionList[0];

    // Helper to format parameter types
    const formatType = (type: any): string => {
        if (!type) return 'u64';
        if (typeof type === 'string') return type;
        if (typeof type === 'object') {
            const keys = Object.keys(type);
            if (keys.length > 0) {
                const key = keys[0];
                const value = type[key];
                if (value === null || (Array.isArray(value) && value.length === 0)) return key;
                if (key === 'Primitive') return value;
                if (key === 'Named') return value;
                return key;
            }
        }
        return JSON.stringify(type);
    };

    // Helper to determine input type
    const getInputType = (paramType: any): 'text' | 'number' | 'bool' | 'pubkey' => {
        const typeStr = formatType(paramType);
        if (['u8', 'u16', 'u32', 'u64', 'i8', 'i16', 'i32', 'i64'].includes(typeStr)) return 'number';
        if (typeStr === 'bool') return 'bool';
        if (['pubkey', 'account', 'signer'].includes(typeStr) || typeStr.includes('Account')) return 'pubkey';
        return 'text';
    };

    // Initialize params array if length doesn't match
    useEffect(() => {
        if (selectedFunction) {
            // Ensure params array is big enough
            if (selectedFunction.parameters && executionParams.length !== selectedFunction.parameters.length) {
                // Don't auto-reset blindly, but maybe we should if completely empty
            }
            // Ensure accounts array is big enough
            if (selectedFunction.accounts && executionAccounts.length !== selectedFunction.accounts.length) {
                // Logic handled in update helpers to expand
            }
        }
    }, [selectedFunction, executionParams.length, executionAccounts.length]);


    const updateParam = (index: number, value: any) => {
        const newParams = [...executionParams];
        // Ensure array is big enough
        while (newParams.length <= index) newParams.push("");
        newParams[index] = value;
        setExecutionParams(newParams);
    };

    const updateAccount = (index: number, value: string) => {
        const newAccounts = [...executionAccounts];
        while (newAccounts.length <= index) newAccounts.push("");
        newAccounts[index] = value;
        setExecutionAccounts(newAccounts);
    };

    const generateKeypairParam = (index: number) => {
        const kp = Keypair.generate();
        updateParam(index, kp.publicKey.toBase58());
    };

    const generateKeypairAccount = (index: number) => {
        const kp = Keypair.generate();
        updateAccount(index, kp.publicKey.toBase58());
    };

    const copyToClipboard = (text: string, type: 'param' | 'account', index: number) => {
        navigator.clipboard.writeText(text);
        setCopiedIndex({ type, index });
        setTimeout(() => setCopiedIndex(null), 1500);
    }

    return (
        <div className="flex flex-col gap-0 min-h-0">
            {/* 1. Configuration Section (Top Pinned) */}
            <div className="p-4 border-b border-white/5 space-y-4">
                {/* Segmented Control for Mode */}
                <div className="bg-black/20 p-1 rounded-lg border border-white/5 flex relative">
                    <button
                        onClick={() => onToggleMode(false)}
                        className={cn(
                            "flex-1 flex items-center justify-center gap-2 py-1.5 px-3 rounded-md text-xs font-medium transition-all duration-200",
                            !isOnChain
                                ? "bg-rose-pine-surface text-rose-pine-text shadow-sm"
                                : "text-rose-pine-subtle hover:text-rose-pine-text hover:bg-white/5"
                        )}
                    >
                        <Cpu size={14} />
                        Simulator
                    </button>
                    <button
                        onClick={() => onToggleMode(true)}
                        className={cn(
                            "flex-1 flex items-center justify-center gap-2 py-1.5 px-3 rounded-md text-xs font-medium transition-all duration-200",
                            isOnChain
                                ? "bg-rose-pine-surface text-rose-pine-text shadow-sm"
                                : "text-rose-pine-subtle hover:text-rose-pine-text hover:bg-white/5"
                        )}
                    >
                        <Globe size={14} />
                        On-Chain
                    </button>
                </div>

                {/* Deployment Cost Widget */}
                {estimatedCost !== null && (
                    <div className="flex items-center gap-3 p-3 rounded-lg bg-emerald-500/5 border border-emerald-500/10 animate-in fade-in slide-in-from-top-1">
                        <div className="p-2 rounded bg-emerald-500/10 text-emerald-400">
                            <Coins size={16} />
                        </div>
                        <div className="flex flex-col min-w-0">
                            <span className="text-[10px] text-emerald-500/70 py-0.5 font-medium uppercase tracking-wider leading-none">
                                Estimated Cost (Rent + Deploy Fee)
                            </span>
                            <div className="flex items-baseline gap-1.5">
                                <span className="text-sm font-mono font-bold text-emerald-400 leading-tight">
                                    ◎ {(estimatedCost / LAMPORTS_PER_SOL).toFixed(5)}
                                </span>
                                <span className="text-xs text-emerald-500/50">
                                    (${((estimatedCost / LAMPORTS_PER_SOL) * solPrice).toFixed(4)})
                                </span>
                            </div>
                            {estimatedRent !== null && estimatedDeployFee !== null && (
                                <div className="text-[10px] text-emerald-500/50 font-mono mt-1">
                                    rent ◎ {(estimatedRent / LAMPORTS_PER_SOL).toFixed(5)} + fee ◎ {(estimatedDeployFee / LAMPORTS_PER_SOL).toFixed(5)}
                                    {deployFeeLamports !== null ? ` (${deployFeeLamports.toLocaleString()} lamports)` : ""}
                                </div>
                            )}
                        </div>
                    </div>
                )}
            </div>

            {/* 2. Inputs Section (Scrollable) */}
            <div className="flex-1 overflow-y-auto p-4 space-y-6">

                {/* Function Selector */}
                <div className="space-y-2">
                    <div className="flex items-center justify-between">
                        <label className="text-xs text-rose-pine-subtle font-medium uppercase tracking-wider">
                            Function
                        </label>
                        <button
                            onClick={parseTestParams}
                            disabled={!bytecode}
                            className="text-[10px] px-2 py-1 hover:bg-rose-pine-iris/10 text-rose-pine-iris hover:text-rose-pine-iris transition-colors rounded flex items-center gap-1 opacity-70 hover:opacity-100 disabled:opacity-30"
                            title="Auto-fill from //@test-params"
                        >
                            <Sparkles size={10} />
                            Auto-Fill
                        </button>
                    </div>

                    <div className="relative group">
                        <select
                            className="w-full appearance-none bg-rose-pine-surface/10 border border-white/5 rounded-lg py-2.5 pl-3 pr-10 text-sm text-rose-pine-text focus:outline-none focus:border-rose-pine-iris/50 focus:bg-rose-pine-surface/20 transition-all font-mono disabled:opacity-50"
                            value={selectedFunctionIndex}
                            onChange={(e) => {
                                setSelectedFunctionIndex(Number(e.target.value));
                                setExecutionParams([]); // Clear params on function switch
                                setExecutionAccounts([]); // Clear accounts on function switch
                            }}
                            disabled={!bytecode}
                        >
                            {functionList.map((f: any) => (
                                <option key={f.index} value={f.index}>
                                    {f.name}
                                </option>
                            ))}
                            {functionList.length === 0 && (
                                <option value={0}>Main (Default)</option>
                            )}
                        </select>
                        <div className="absolute right-3 top-1/2 -translate-y-1/2 text-rose-pine-subtle pointer-events-none group-hover:text-rose-pine-text transition-colors">
                            <ChevronDown size={14} />
                        </div>
                    </div>
                </div>

                {/* Structured Inputs */}
                <div className="space-y-6">
                    {/* ACCOUNTS (Separate Section if exist) */}
                    {functionList.length > 0 && selectedFunction?.accounts?.length > 0 && (
                        <div className="space-y-3">
                            <div className="flex items-center justify-between">
                                <label className="text-xs text-rose-pine-subtle font-medium uppercase tracking-wider">
                                    Accounts
                                </label>
                                <span className="text-[10px] text-rose-pine-muted bg-white/5 px-1.5 py-px rounded">Context</span>
                            </div>

                            {selectedFunction.accounts.map((acc: any, i: number) => {
                                const currentValue = executionAccounts[i] ?? "";
                                const displayType = acc.signer ? 'Signer' : (acc.writable ? 'Writable' : 'Read-Only');

                                return (
                                    <div key={`acc-${i}`} className="space-y-1.5 animate-in fade-in slide-in-from-left-2 duration-300">
                                        <div className="flex items-center justify-between">
                                            <div className="flex items-center gap-2">
                                                <span className="text-xs font-medium text-rose-pine-text">{acc.name}</span>
                                                <span className={cn(
                                                    "text-[10px] px-1.5 py-px rounded font-mono",
                                                    acc.signer ? "bg-amber-500/10 text-amber-500" :
                                                        acc.writable ? "bg-rose-pine-iris/10 text-rose-pine-iris" : "bg-white/5 text-rose-pine-muted"
                                                )}>
                                                    {displayType}
                                                </span>
                                            </div>
                                            <button
                                                onClick={() => generateKeypairAccount(i)}
                                                className="text-[10px] flex items-center gap-1 text-rose-pine-iris hover:text-rose-pine-foam transition-colors"
                                                title="Generate Random Keypair"
                                            >
                                                <Dices size={12} />
                                                Random
                                            </button>
                                        </div>
                                        <div className="relative">
                                            <input
                                                type="text"
                                                placeholder={`Enter Public Key for ${acc.name}...`}
                                                className="w-full bg-black/20 border border-white/5 rounded-lg py-2 pl-3 pr-8 text-sm font-mono text-rose-pine-text focus:outline-none focus:border-rose-pine-iris/50 focus:bg-black/30 transition-all placeholder:text-white/10"
                                                value={currentValue}
                                                onChange={(e) => updateAccount(i, e.target.value)}
                                            />
                                            {currentValue && (
                                                <button
                                                    onClick={() => copyToClipboard(currentValue, 'account', i)}
                                                    className="absolute right-2 top-1/2 -translate-y-1/2 text-rose-pine-subtle hover:text-rose-pine-text transition-colors"
                                                >
                                                    {(copiedIndex?.type === 'account' && copiedIndex.index === i) ? <Check size={14} className="text-emerald-500" /> : <Copy size={14} />}
                                                </button>
                                            )}
                                        </div>
                                    </div>
                                );
                            })}
                        </div>
                    )}


                    {/* PARAMETERS */}
                    <div className="space-y-3">
                        {selectedFunction?.parameters?.length > 0 && (
                            <label className="text-xs text-rose-pine-subtle font-medium uppercase tracking-wider">
                                Parameters
                            </label>
                        )}

                        {functionList.length > 0 && selectedFunction?.parameters?.map((p: any, i: number) => {
                            const inputType = getInputType(p.type || p.param_type);
                            const displayType = formatType(p.type || p.param_type);
                            const currentValue = executionParams[i] ?? "";

                            return (
                                <div key={`param-${i}`} className="space-y-1.5 animate-in fade-in slide-in-from-left-2 duration-300" style={{ animationDelay: `${i * 50}ms` }}>
                                    <div className="flex items-center justify-between">
                                        <div className="flex items-center gap-2">
                                            <span className="text-xs font-medium text-rose-pine-text">{p.name}</span>
                                            <span className="text-[10px] px-1.5 py-px rounded bg-white/5 text-rose-pine-muted font-mono">
                                                {displayType}
                                            </span>
                                        </div>
                                        {/* Action Buttons based on type */}
                                        {inputType === 'pubkey' && (
                                            <button
                                                onClick={() => generateKeypairParam(i)}
                                                className="text-[10px] flex items-center gap-1 text-rose-pine-iris hover:text-rose-pine-foam transition-colors"
                                                title="Generate Random Keypair"
                                            >
                                                <Dices size={12} />
                                                Random
                                            </button>
                                        )}
                                    </div>

                                    <div className="relative">
                                        {inputType === 'bool' ? (
                                            <div className="relative">
                                                <select
                                                    className="w-full appearance-none bg-black/20 border border-white/5 rounded-lg py-2 pl-3 pr-8 text-sm font-mono text-rose-pine-text focus:outline-none focus:border-rose-pine-iris/50 focus:bg-black/30 transition-all"
                                                    value={currentValue.toString()}
                                                    onChange={(e) => updateParam(i, e.target.value === 'true')}
                                                >
                                                    <option value="true">true</option>
                                                    <option value="false">false</option>
                                                </select>
                                                <ChevronDown className="absolute right-3 top-1/2 -translate-y-1/2 text-rose-pine-subtle pointer-events-none" size={14} />
                                            </div>
                                        ) : (
                                            <div className="relative">
                                                <input
                                                    type={inputType === 'number' ? 'number' : 'text'}
                                                    placeholder={`Enter ${displayType}...`}
                                                    className="w-full bg-black/20 border border-white/5 rounded-lg py-2 pl-3 pr-8 text-sm font-mono text-rose-pine-text focus:outline-none focus:border-rose-pine-iris/50 focus:bg-black/30 transition-all placeholder:text-white/10"
                                                    value={currentValue}
                                                    onChange={(e) => {
                                                        let val: any = e.target.value;
                                                        if (inputType === 'number' && val !== '') {
                                                            const num = Number(val);
                                                            if (!isNaN(num)) val = num;
                                                        }
                                                        updateParam(i, val);
                                                    }}
                                                />
                                                {/* Copy Button for Pubkeys */}
                                                {inputType === 'pubkey' && currentValue && (
                                                    <button
                                                        onClick={() => copyToClipboard(currentValue.toString(), 'param', i)}
                                                        className="absolute right-2 top-1/2 -translate-y-1/2 text-rose-pine-subtle hover:text-rose-pine-text transition-colors"
                                                    >
                                                        {(copiedIndex?.type === 'param' && copiedIndex.index === i) ? <Check size={14} className="text-emerald-500" /> : <Copy size={14} />}
                                                    </button>
                                                )}
                                            </div>
                                        )}
                                    </div>
                                </div>
                            );
                        })}

                        {(!selectedFunction?.parameters || selectedFunction.parameters.length === 0) && (!selectedFunction?.accounts || selectedFunction.accounts.length === 0) && (
                            <div className="py-8 text-center bg-black/10 rounded-lg border border-white/5 border-dashed">
                                <span className="text-xs text-rose-pine-subtle opacity-70">
                                    This function takes no parameters.
                                </span>
                            </div>
                        )}
                    </div>
                </div>
            </div>

            {/* 3. Actions Section (Bottom Pinned) */}
            <div className="p-4 border-t border-white/5 bg-rose-pine-surface/20">
                <button
                    onClick={onRun}
                    disabled={isExecuting || !bytecode}
                    className={cn(
                        "w-full flex items-center justify-center gap-2 py-2.5 rounded-lg text-sm font-medium transition-all duration-300 shadow-lg hover:shadow-xl hover:scale-[1.02] active:scale-[0.98]",
                        isExecuting
                            ? "bg-rose-pine-surface text-rose-pine-subtle cursor-wait"
                            : isOnChain
                                ? "bg-gradient-to-r from-emerald-500 to-emerald-600 hover:from-emerald-400 hover:to-emerald-500 text-white shadow-emerald-500/20"
                                : "bg-gradient-to-r from-rose-pine-iris to-rose-pine-foam hover:from-rose-pine-iris/90 hover:to-rose-pine-foam/90 text-white shadow-rose-pine-iris/20"
                    )}
                >
                    {isExecuting ? (
                        <>
                            <Loader2 size={16} className="animate-spin" />
                            <span>Executing...</span>
                        </>
                    ) : (
                        <>
                            <Play size={16} fill="currentColor" className="opacity-90" />
                            <span>{isOnChain ? "Execute On-Chain" : "Run Simulation"}</span>
                        </>
                    )}
                </button>
            </div>
        </div>
    );
}
