"use client";

import { Editor as MonacoEditor, useMonaco, OnMount } from "@monaco-editor/react";
import { useThemeStore } from "@/stores/theme-store";
import { useEffect, useState } from "react";
import { GlassCard } from "@/components/ui/glass-card";
import { Loader2, Hammer } from "lucide-react";
import { useFiveWasm } from "@/hooks/useFiveWasm";
import { cn } from "@/lib/utils";
import { defineMonacoThemes, registerFiveLanguage } from "@/lib/monaco-theme";

interface DocsEditorProps {
    code: string;
    filename?: string;
    height?: string;
    language?: string;
    showRunControls?: boolean;
}

export default function DocsEditor({
    code,
    filename = "example.five",
    height = "300px",
    language = "five",
    showRunControls = true
}: DocsEditorProps) {
    const [mounted, setMounted] = useState(false);
    const [isRunning, setIsRunning] = useState(false);
    const [status, setStatus] = useState("Ready");
    const [statusKind, setStatusKind] = useState<"idle" | "running" | "success" | "error">("idle");

    // State for the code to allow editing while syncing with props
    const [editorCode, setEditorCode] = useState(code);

    // Sync local state when the prop changes (e.g. user selects a different example)
    useEffect(() => {
        setEditorCode(code);
    }, [code]);

    // New Hook Integration
    const { isReady, compile } = useFiveWasm();

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
        // Define themes immediately at mount time to prevent default "vs" light flash
        defineMonacoThemes(monacoInstance);
        registerFiveLanguage(monacoInstance);
        monacoInstance.editor.setTheme(theme === 'dark' ? "rose-pine-dark" : "rose-pine-light");
    };

    const handleBuild = async () => {
        if (!isReady) return;
        setIsRunning(true);
        setStatusKind("running");
        setStatus("Building...");

        try {
            // 1. Compile
            const compileResult = await compile(editorCode);

            if (!compileResult.success) {
                setStatusKind("error");
                setStatus("Build failed");
                setIsRunning(false);
                return;
            }

            if (compileResult.bytecode) {
                setStatusKind("success");
                setStatus(`Built • ${compileResult.bytecode.length} bytes`);
            } else {
                setStatusKind("success");
                setStatus("Build successful");
            }

        } catch (e: any) {
            setStatusKind("error");
            setStatus("System Error");
        } finally {
            setIsRunning(false);
        }
    };

    return (
        <GlassCard className={cn(
            "relative overflow-hidden flex flex-col transition-all duration-300",
            theme === "dark"
                ? "border-rose-pine-hl-low/50 bg-black/40"
                : "border-rose-pine-hl-med/40 bg-white/95 shadow-sm"
        )}>
            <div className={cn(
                "flex items-center justify-between gap-3 px-4 py-2.5 border-b shrink-0",
                theme === "dark"
                    ? "border-white/5 bg-white/5"
                    : "border-rose-pine-hl-low/40 bg-rose-pine-surface/35"
            )}>
                <div className="flex min-w-0 items-center gap-3">
                    <div className="flex gap-1.5">
                        <div className="w-2.5 h-2.5 rounded-full bg-rose-pine-love/50" />
                        <div className="w-2.5 h-2.5 rounded-full bg-rose-pine-gold/50" />
                        <div className="w-2.5 h-2.5 rounded-full bg-rose-pine-foam/50" />
                    </div>
                    <span className="truncate text-xs text-rose-pine-muted font-mono">{filename}</span>
                </div>

                {showRunControls && (
                    <div className="flex min-w-0 items-center gap-2">
                        <span
                            className={cn(
                                "hidden md:inline truncate text-[11px] font-medium",
                                statusKind === "error" ? "text-rose-pine-love" :
                                    statusKind === "success" ? "text-rose-pine-foam" :
                                        statusKind === "running" ? "text-rose-pine-gold" :
                                            "text-rose-pine-muted"
                            )}
                            aria-live="polite"
                        >
                            {status}
                        </span>
                        <button
                            onClick={handleBuild}
                            disabled={!isReady || isRunning}
                            className={cn(
                                "shrink-0 flex items-center gap-1.5 px-3 py-1.5 rounded-md text-[11px] font-bold uppercase tracking-wider transition-all",
                                !isReady
                                    ? "opacity-50 cursor-not-allowed bg-white/5 text-rose-pine-muted"
                                    : isRunning
                                        ? "bg-rose-pine-surface text-rose-pine-subtle cursor-wait"
                                        : "bg-rose-pine-love text-rose-pine-base hover:bg-rose-pine-love/90 hover:scale-105 active:scale-95 shadow-lg shadow-rose-pine-love/20"
                            )}
                        >
                            {isRunning ? <Loader2 size={12} className="animate-spin" /> : <Hammer size={10} fill="currentColor" />}
                            {isRunning ? "Building..." : "Build"}
                        </button>
                    </div>
                )}
            </div>

            <div style={{ height }} className="relative shrink-0 group">
                <MonacoEditor
                    height="100%"
                    defaultLanguage={language}
                    value={editorCode}
                    onChange={(val) => setEditorCode(val || "")}
                    theme={theme === 'dark' ? "rose-pine-dark" : "rose-pine-light"}
                    onMount={handleEditorDidMount}
                    options={{
                        readOnly: false,
                        domReadOnly: false,
                        minimap: { enabled: false },
                        scrollBeyondLastLine: false,
                        automaticLayout: true,
                        padding: { top: 16, bottom: 16 },
                        lineNumbers: "on",
                        glyphMargin: false,
                        folding: false,
                        renderLineHighlight: "all",
                        contextmenu: true,
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
        </GlassCard>
    );
}
