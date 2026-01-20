"use client";

import { useState } from "react";
import { useIdeStore } from "@/stores/ide-store";
import { TEMPLATES, type Template } from "@/data/templates";
import { FileCode, ChevronDown, Check, Sparkles } from "lucide-react";

interface TemplateSelectorProps {
    className?: string;
}

export default function TemplateSelector({ className }: TemplateSelectorProps) {
    const { createFile, setCode, activeFile, openFile } = useIdeStore();
    const [isOpen, setIsOpen] = useState(false);
    const [selectedTemplate, setSelectedTemplate] = useState<Template | null>(null);

    const handleSelectTemplate = (template: Template) => {
        // Create a new file with the template content
        const filename = `src/${template.id}.v`;
        createFile(filename, template.code, true);
        setCode(template.code);
        setSelectedTemplate(template);
        setIsOpen(false);
    };

    return (
        <div className={`relative ${className || ''}`}>
            <button
                onClick={() => setIsOpen(!isOpen)}
                className="flex items-center gap-2 px-3 py-2 rounded-lg bg-rose-pine-surface/50 border border-white/10 hover:border-rose-pine-iris/30 text-xs text-rose-pine-text transition-all group"
            >
                <Sparkles size={14} className="text-rose-pine-gold" />
                <span>Load Template</span>
                <ChevronDown
                    size={14}
                    className={`text-rose-pine-subtle transition-transform ${isOpen ? 'rotate-180' : ''}`}
                />
            </button>

            {isOpen && (
                <div className="absolute top-full left-0 mt-2 w-64 bg-rose-pine-surface border border-white/10 rounded-xl shadow-xl z-50 overflow-hidden animate-in fade-in slide-in-from-top-2 duration-200">
                    <div className="p-2 border-b border-white/5">
                        <span className="text-[10px] uppercase tracking-wider text-rose-pine-muted font-bold px-2">
                            Quick Start Templates
                        </span>
                    </div>

                    <div className="p-1">
                        {TEMPLATES.map((template) => (
                            <button
                                key={template.id}
                                onClick={() => handleSelectTemplate(template)}
                                className="w-full flex items-start gap-3 p-3 rounded-lg hover:bg-white/5 transition-colors text-left group"
                            >
                                <span className="text-xl">{template.icon}</span>
                                <div className="flex-1 min-w-0">
                                    <div className="flex items-center gap-2">
                                        <span className="text-sm font-medium text-rose-pine-text group-hover:text-rose-pine-foam transition-colors">
                                            {template.name}
                                        </span>
                                        {selectedTemplate?.id === template.id && (
                                            <Check size={12} className="text-emerald-400" />
                                        )}
                                    </div>
                                    <p className="text-[11px] text-rose-pine-subtle truncate">
                                        {template.description}
                                    </p>
                                </div>
                            </button>
                        ))}
                    </div>
                </div>
            )}

            {/* Backdrop */}
            {isOpen && (
                <div
                    className="fixed inset-0 z-40"
                    onClick={() => setIsOpen(false)}
                />
            )}
        </div>
    );
}
