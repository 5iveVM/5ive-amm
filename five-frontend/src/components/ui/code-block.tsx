"use client";

import { useThemeStore } from "@/stores/theme-store";
import { cn } from "@/lib/utils";
import { Check, Copy, Terminal } from "lucide-react";
import { useState } from "react";

interface CodeBlockProps {
    code: string;
    filename?: string;
    language?: string;
}

// Simple regex-based syntax highlighter for TypeScript/JS
const highlightCode = (code: string, language?: string) => {
    if (language !== "typescript" && language !== "javascript") return code;

    // Split by common delimiters but keep them for reconstruction
    const tokens = code.split(/(\s+|[(){}[\];,]|\/\/.*$)/gm);

    return tokens.map((token, i) => {
        // Keywords
        if (/^(import|from|const|let|var|function|async|await|return|if|else|new|export|default|interface|type)$/.test(token)) {
            return <span key={i} className="text-rose-pine-iris font-semibold">{token}</span>; // Iris/Purple
        }
        // Types / Classes (Starts with Uppercase)
        if (/^[A-Z][a-zA-Z0-9_]*$/.test(token)) {
            return <span key={i} className="text-rose-pine-gold">{token}</span>; // Gold
        }
        // Strings
        if (/^["'`].*["'`]$/.test(token)) {
            return <span key={i} className="text-rose-pine-rose">{token}</span>; // Rose
        }
        // Numbers
        if (/^\d+$/.test(token)) {
            return <span key={i} className="text-rose-pine-love">{token}</span>; // Love/Red
        }
        // Comments
        if (/^\/\//.test(token)) {
            return <span key={i} className="text-rose-pine-muted italic">{token}</span>; // Muted
        }

        return token;
    });
};

export function CodeBlock({ code, filename, language }: CodeBlockProps) {
    const { theme } = useThemeStore();
    const [copied, setCopied] = useState(false);

    const handleCopy = async () => {
        await navigator.clipboard.writeText(code);
        setCopied(true);
        setTimeout(() => setCopied(false), 2000);
    };

    const isTerminal = language === "shell" || language === "bash";

    // Terminal style override: always dark background
    const containerClasses = isTerminal
        ? "bg-rose-pine-base border-rose-pine-hl-low/20 text-rose-pine-text" // Darker bg, brighter text
        : theme === "dark"
            ? "bg-black/40 border-white/5 text-rose-pine-text"
            : "bg-rose-pine-surface/50 border-rose-pine-hl-low/40 text-rose-pine-text";

    return (
        <div className={cn(
            "rounded-lg overflow-hidden border transition-all group relative",
            containerClasses
        )}>
            {/* Header - Only render if filename exists */}
            {filename && (
                <div className={cn(
                    "px-4 py-2 text-xs font-mono border-b flex items-center justify-between select-none",
                    isTerminal
                        ? "bg-white/5 border-white/5 text-rose-pine-muted"
                        : theme === "dark"
                            ? "bg-white/5 border-white/5 text-rose-pine-muted"
                            : "bg-rose-pine-overlay/5 border-rose-pine-hl-low/40 text-rose-pine-subtle"
                )}>
                    <div className="flex items-center gap-2">
                        {isTerminal ? <Terminal size={12} /> : (
                            <div className="flex gap-1.5 mr-2">
                                <div className="w-2 h-2 rounded-full bg-rose-pine-muted/20" />
                                <div className="w-2 h-2 rounded-full bg-rose-pine-muted/20" />
                                <div className="w-2 h-2 rounded-full bg-rose-pine-muted/20" />
                            </div>
                        )}
                        {filename}
                    </div>
                </div>
            )}

            {/* Copy Button - Headerless mode (Floating) or Header mode (In header) */}
            {filename ? (
                <div className="absolute top-2 right-2">
                    <button
                        onClick={handleCopy}
                        className="opacity-0 group-hover:opacity-100 transition-opacity p-1.5 hover:bg-white/10 rounded-md text-rose-pine-subtle hover:text-rose-pine-text"
                        title="Copy to clipboard"
                    >
                        {copied ? <Check size={14} className="text-emerald-400" /> : <Copy size={14} />}
                    </button>
                </div>
            ) : (
                <button
                    onClick={handleCopy}
                    className="absolute top-3 right-3 opacity-0 group-hover:opacity-100 transition-all p-2 bg-rose-pine-overlay/20 hover:bg-rose-pine-overlay/40 backdrop-blur-sm rounded-md text-rose-pine-subtle hover:text-rose-pine-text z-10"
                    title="Copy to clipboard"
                >
                    {copied ? <Check size={14} className="text-emerald-400" /> : <Copy size={14} />}
                </button>
            )}

            {/* Code Content */}
            <div className={cn("overflow-x-auto", filename ? "p-4" : "p-5")}>
                <pre className="font-mono text-sm leading-relaxed">
                    {isTerminal ? (
                        <div className="flex flex-col gap-1">
                            {code.split('\n').filter(Boolean).map((line, i) => (
                                <div key={i} className="flex gap-3">
                                    <span className="text-rose-pine-subtle/50 select-none">$</span>
                                    <span>{line}</span>
                                </div>
                            ))}
                        </div>
                    ) : (
                        <code>
                            {highlightCode(code, language)}
                        </code>
                    )}
                </pre>
            </div>
        </div>
    );
}
