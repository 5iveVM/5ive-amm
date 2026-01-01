"use client";

import { Editor as MonacoEditor, useMonaco, OnMount } from "@monaco-editor/react";
import { useThemeStore } from "@/stores/theme-store";
import { useEffect, useState } from "react";
import { GlassCard } from "@/components/ui/glass-card";
import { Play, Loader2, ChevronRight, Terminal, Hammer } from "lucide-react";
import { useFiveWasm } from "@/hooks/useFiveWasm";
import { cn } from "@/lib/utils";
import { defineMonacoThemes, registerFiveLanguage } from "@/lib/monaco-theme";

interface DocsEditorProps {
    code: string;
    filename?: string;
    height?: string;
}

export default function DocsEditor({ code, filename = "example.five", height = "300px" }: DocsEditorProps) {
    const [mounted, setMounted] = useState(false);
    const [isRunning, setIsRunning] = useState(false);
    const [logs, setLogs] = useState<string[]>([]);
    const [showOutput, setShowOutput] = useState(false);

    // New Hook Integration
    const { isReady, compile, execute } = useFiveWasm();

    const monaco = useMonaco();
    const { theme } = useThemeStore();

    // Register themes and language once
    useEffect(() => {
        if (!monaco) return;

        defineMonacoThemes(monaco);
        registerFiveLanguage(monaco);

        monaco.editor.setTheme(theme === 'dark' ? "rose-pine-dark" : "rose-pine-light");
    }, [monaco, theme]);

    const handleEditorDidMount: OnMount = (editor, monacoInstance) => {
        setMounted(true);
    };

    const handleRun = async () => {
        if (!isReady) return;
        setIsRunning(true);
        setShowOutput(true);
        setLogs(["Compiling..."]);

        try {
            // 1. Compile
            const compileResult = await compile(code);

            if (!compileResult.success) {
                setLogs(prev => [...prev, "Compilation failed:", compileResult.error || "Unknown error"]);
                setIsRunning(false);
                return;
            }

            setLogs(prev => [...prev, "Compilation successful."]);

            if (compileResult.bytecode) {
                setLogs(prev => [...prev, `Bytecode generated (${compileResult.bytecode!.length} bytes).`]);
            } else {
                setLogs(prev => [...prev, "Warning: No bytecode output."]);
            }

        } catch (e) {
            setLogs(prev => [...prev, `Detailed Error: ${e}`]);
        } finally {
            setIsRunning(false);
        }
    };

    return (
        <GlassCard className="relative overflow-hidden border-rose-pine-hl-low/50 bg-black/40 flex flex-col">
            <div className="flex items-center justify-between px-4 py-2 border-b border-white/5 bg-white/5 shrink-0">
                <div className="flex items-center gap-3">
                    <div className="flex gap-1.5">
                        <div className="w-2.5 h-2.5 rounded-full bg-rose-pine-love/50" />
                        <div className="w-2.5 h-2.5 rounded-full bg-rose-pine-gold/50" />
                        <div className="w-2.5 h-2.5 rounded-full bg-rose-pine-foam/50" />
                    </div>
                    <span className="text-xs text-rose-pine-muted font-mono">{filename}</span>
                </div>

                <div className="flex items-center gap-2">
                    {/* Run Button */}
                    <button
                        onClick={handleRun}
                        disabled={!isReady || isRunning}
                        className={cn(
                            "flex items-center gap-1.5 px-2.5 py-1 rounded-md text-[10px] font-bold uppercase tracking-wider transition-all",
                            !isReady
                                ? "opacity-50 cursor-not-allowed bg-white/5 text-rose-pine-muted"
                                : isRunning
                                    ? "bg-rose-pine-surface text-rose-pine-subtle cursor-wait"
                                    : "bg-rose-pine-love/10 text-rose-pine-love hover:bg-rose-pine-love/20 hover:scale-105 active:scale-95"
                        )}
                    >
                        {isRunning ? <Loader2 size={12} className="animate-spin" /> : <Hammer size={10} fill="currentColor" />}
                        {isRunning ? "Building..." : "Build"}
                    </button>
                </div>
            </div>

            <div style={{ height }} className="relative shrink-0">
                <MonacoEditor
                    height="100%"
                    defaultLanguage="five"
                    value={code}
                    onMount={handleEditorDidMount}
                    options={{
                        readOnly: true,
                        domReadOnly: true,
                        minimap: { enabled: false },
                        scrollBeyondLastLine: false,
                        automaticLayout: true,
                        padding: { top: 16, bottom: 16 },
                        lineNumbers: "on",
                        glyphMargin: false,
                        folding: false,
                        renderLineHighlight: "none",
                        contextmenu: false,
                        fontFamily: "'JetBrains Mono', 'Fira Code', monospace",
                        fontSize: 13,
                        scrollbar: {
                            vertical: 'hidden',
                            horizontal: 'auto',
                            useShadows: false
                        },
                        overviewRulerLanes: 0,
                        hideCursorInOverviewRuler: true,
                        matchBrackets: "never",
                    }}
                    className="bg-transparent"
                />
                {!mounted && (
                    <div className="absolute inset-0 flex items-center justify-center text-rose-pine-muted text-sm">
                        Loading code...
                    </div>
                )}
            </div>

            {/* Output Console (Collapsible) */}
            {showOutput && (
                <div className="border-t border-white/10 bg-black/60 p-3 font-mono text-xs">
                    <div
                        className="flex items-center gap-2 text-rose-pine-subtle mb-2 cursor-pointer hover:text-rose-pine-text transition-colors select-none"
                        onClick={() => setShowOutput(false)}
                    >
                        <ChevronRight size={12} className="rotate-90" />
                        <span className="font-bold uppercase tracking-wider text-[10px]">Console Output</span>
                    </div>
                    <div className="space-y-1 max-h-32 overflow-y-auto pl-1">
                        {logs.length === 0 ? (
                            <span className="text-rose-pine-muted opacity-50 italic">Ready to execute...</span>
                        ) : (
                            logs.map((log, i) => (
                                <div key={i} className="flex gap-2">
                                    <span className="text-rose-pine-iris">➜</span>
                                    <span className={cn(
                                        "break-all",
                                        log.includes("Error") || log.includes("failed") ? "text-rose-pine-love" :
                                            log.includes("successful") ? "text-rose-pine-foam" : "text-rose-pine-text"
                                    )}>
                                        {log}
                                    </span>
                                </div>
                            ))
                        )}
                    </div>
                </div>
            )}
        </GlassCard>
    );
}
