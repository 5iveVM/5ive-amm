"use client";

import { Editor as MonacoEditor, useMonaco, OnMount } from "@monaco-editor/react";
import { useIdeStore } from "@/stores/ide-store";
import { useThemeStore } from "@/stores/theme-store";
import { useEffect, useState } from "react";


import { GlassCard, GlassHeader } from "@/components/ui/glass-card";
import { Play, Save, Box, Layers } from "lucide-react";
import ProjectConfigModal from "./ProjectConfigModal";
import { WalletMultiButton } from "@solana/wallet-adapter-react-ui";
import { defineMonacoThemes, registerFiveLanguage } from "@/lib/monaco-theme";
import { setupFiveLsp } from "@/lib/monaco-lsp";

export default function GlassEditor() {
    const { code, setCode, currentFilename } = useIdeStore();
    const [mounted, setMounted] = useState(false);
    const [showConfig, setShowConfig] = useState(false);

    // Configure 5IVE DSL syntax highlighting
    const monaco = useMonaco();
    const { theme } = useThemeStore();

    useEffect(() => {
        if (!monaco) return;

        defineMonacoThemes(monaco);
        registerFiveLanguage(monaco);

        monaco.editor.setTheme(theme === 'dark' ? "rose-pine-dark" : "rose-pine-light");
    }, [monaco, theme]);

    const [cursorPosition, setCursorPosition] = useState({ lineNumber: 1, column: 1 });

    const handleEditorDidMount: OnMount = (editor, monacoInstance) => {
        setMounted(true);
        editor.onDidChangeCursorPosition((e) => {
            setCursorPosition({
                lineNumber: e.position.lineNumber,
                column: e.position.column
            });
        });

        // Initialize 5IVE LSP for real-time diagnostics
        setupFiveLsp(monacoInstance).catch((error) => {
            console.error('[GlassEditor] Failed to setup 5IVE LSP:', error);
        });
    };

    // Responsive font size
    const [fontSize, setFontSize] = useState(14);

    useEffect(() => {
        const handleResize = () => {
            setFontSize(window.innerWidth < 768 ? 12 : 14);
        };
        handleResize(); // Init
        window.addEventListener('resize', handleResize);
        return () => window.removeEventListener('resize', handleResize);
    }, []);

    return (
        <>
            {/* Increased opacity for better contrast on mobile */}
            <div className="group h-full w-full flex flex-col min-h-0 overflow-hidden relative bg-rose-pine-base/60 backdrop-blur-xl">
                {/* Header removed for Zen mode */}
                <button
                    onClick={() => setShowConfig(true)}
                    className="absolute top-2 right-4 text-xs p-1.5 rounded-md text-rose-pine-muted hover:text-rose-pine-text transition-colors z-20 opacity-0 group-hover:opacity-100 hover:bg-rose-pine-base/50"
                    title="Project Settings"
                >
                    <Box size={14} />
                </button>

                <div className="flex-1 relative pt-2">
                    <MonacoEditor
                        height="100%"
                        defaultLanguage="five"
                        value={code}
                        theme={theme === 'dark' ? "rose-pine-dark" : "rose-pine-light"}
                        onChange={(value) => setCode(value || "")}
                        onMount={handleEditorDidMount}
                        options={{
                            wordWrap: "off", // Fixed: User requested no wrapping ("text wraps... could be better")
                            minimap: { enabled: false },
                            fontSize: fontSize,
                            fontFamily: "'JetBrains Mono', 'Fira Code', monospace",
                            scrollBeyondLastLine: false,
                            automaticLayout: true,
                            padding: { top: 16 },
                            lineNumbers: "on",
                            lineNumbersMinChars: 3, // Save formatting space
                            glyphMargin: false, // Save formatting space
                            folding: false, // Save formatting space on mobile
                            renderLineHighlight: "all",
                            contextmenu: true,
                            smoothScrolling: true,
                            cursorBlinking: "smooth",
                            cursorSmoothCaretAnimation: "on",
                            stickyScroll: { enabled: false },
                        }}
                        className="bg-transparent"
                    />

                    {!mounted && (
                        <div className="absolute inset-0 flex items-center justify-center text-rose-pine-muted">
                            Initializing Editor...
                        </div>
                    )}
                </div>

                <div className="h-8 border-t border-white/5 bg-rose-pine-base/40 flex items-center px-4 justify-between text-xs text-rose-pine-subtle">
                    <div className="flex items-center gap-4">
                        {/* Clean footer, removed duplicate version info */}
                    </div>
                    <div className="flex items-center gap-4">
                        <span>Ln {cursorPosition.lineNumber}, Col {cursorPosition.column}</span>
                        <span className="hidden md:inline text-rose-pine-muted/50">|</span>
                        <span className="hidden md:inline">Spaces: 4</span>
                        <span className="hidden md:inline text-rose-pine-muted/50">|</span>
                        <span className="hidden md:inline">UTF-8</span>
                        <span className="hidden md:inline text-rose-pine-muted/50">|</span>
                        <span className="font-medium text-rose-pine-iris">5IVE</span>
                    </div>
                </div>
            </div>

            <ProjectConfigModal isOpen={showConfig} onClose={() => setShowConfig(false)} />
        </>
    );
}
