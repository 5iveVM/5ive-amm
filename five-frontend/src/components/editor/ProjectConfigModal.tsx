"use client";

import { GlassCard, GlassHeader } from "@/components/ui/glass-card";
import { X, Save, FileJson } from "lucide-react";
import { useState, useEffect } from "react";
import { useIdeStore } from "@/stores/ide-store";
// Import from the local linked SDK
import { parseToml, parseProjectConfig } from "five-sdk";

interface ProjectConfigModalProps {
    isOpen: boolean;
    onClose: () => void;
}

const DEFAULT_TOML = `[project]
name = "my-counter-project"
version = "0.1.0"
description = "A simple counter contract on 5IVE VM"
target = "vm"

[build]
output_artifact_name = "counter_v1"

[deploy]
network = "devnet"
program_id = "5ive..."
`;

export default function ProjectConfigModal({ isOpen, onClose }: ProjectConfigModalProps) {
    const { projectConfig, setProjectConfig, compilerOptions, updateCompilerOptions } = useIdeStore();
    const [tomlContent, setTomlContent] = useState(DEFAULT_TOML);
    const [parsedConfig, setParsedConfig] = useState<any>(null);
    const [error, setError] = useState<string | null>(null);
    const [activeTab, setActiveTab] = useState<'project' | 'compiler'>('project');

    // Sync store config on mount
    useEffect(() => {
        if (projectConfig) {
            setParsedConfig(projectConfig);
            // TODO: reverse map JSON to TOML if needed, or just keep them separate for now
        }
    }, [projectConfig]);

    const handleParse = () => {
        try {
            const raw = parseToml(tomlContent);
            const config = parseProjectConfig(raw);
            setParsedConfig(config);
            setProjectConfig(config);
            setError(null);
        } catch (e: any) {
            setError(e.message || "Failed to parse TOML");
            setParsedConfig(null);
        }
    };

    if (!isOpen) return null;

    return (
        <div className="fixed inset-0 z-[100] flex items-center justify-center bg-black/50 backdrop-blur-sm p-4">
            <GlassCard variant="heavy" className="w-full max-w-2xl max-h-[85vh] flex flex-col shadow-2xl border-rose-pine-iris/20">
                <GlassHeader title="Project Configuration" className="bg-rose-pine-base/50">
                    <div className="flex items-center gap-4 bg-rose-pine-overlay/30 rounded-lg p-1 ml-4">
                        <button
                            onClick={() => setActiveTab('project')}
                            className={`px-3 py-1 rounded-md text-xs font-medium transition-all ${activeTab === 'project' ? 'bg-rose-pine-iris/20 text-rose-pine-iris shadow-sm' : 'text-rose-pine-subtle hover:text-rose-pine-text'}`}
                        >
                            five.toml
                        </button>
                        <button
                            onClick={() => setActiveTab('compiler')}
                            className={`px-3 py-1 rounded-md text-xs font-medium transition-all ${activeTab === 'compiler' ? 'bg-rose-pine-iris/20 text-rose-pine-iris shadow-sm' : 'text-rose-pine-subtle hover:text-rose-pine-text'}`}
                        >
                            Compiler Settings
                        </button>
                    </div>
                    <button onClick={onClose} className="p-1 hover:bg-white/10 rounded-md transition-colors ml-auto">
                        <X size={18} />
                    </button>
                </GlassHeader>

                <div className="p-6 flex-1 flex flex-col gap-4 overflow-hidden">

                    {activeTab === 'project' ? (
                        <>
                            <p className="text-sm text-rose-pine-muted">
                                Configure your project using <code>five.toml</code>. Shared with CLI.
                            </p>
                            <div className="grid grid-cols-1 md:grid-cols-2 gap-4 flex-1 min-h-0">
                                <div className="flex flex-col gap-2 min-h-0">
                                    <label className="text-xs font-semibold text-rose-pine-subtle uppercase">five.toml</label>
                                    <textarea
                                        className="flex-1 resize-none bg-rose-pine-surface/50 border border-white/10 rounded-lg p-3 font-mono text-sm text-rose-pine-text focus:outline-none focus:border-rose-pine-iris/50"
                                        value={tomlContent}
                                        onChange={(e) => setTomlContent(e.target.value)}
                                        spellCheck={false}
                                    />
                                </div>
                                <div className="flex flex-col gap-2 min-h-0">
                                    <div className="flex justify-between items-center">
                                        <label className="text-xs font-semibold text-rose-pine-subtle uppercase">Parsed Config</label>
                                        <button onClick={handleParse} className="text-xs px-2 py-1 bg-rose-pine-iris/20 text-rose-pine-iris hover:bg-rose-pine-iris/30 rounded flex items-center gap-1 transition-colors">
                                            <FileJson size={12} /> Parse & Save
                                        </button>
                                    </div>
                                    <div className="flex-1 bg-rose-pine-base/60 border border-white/5 rounded-lg p-3 font-mono text-xs text-rose-pine-foam overflow-auto whitespace-pre">
                                        {error ? <span className="text-rose-pine-love">{error}</span> : (parsedConfig ? JSON.stringify(parsedConfig, null, 2) : "// Click Parse to validate...")}
                                    </div>
                                </div>
                            </div>
                        </>
                    ) : (
                        <div className="flex flex-col gap-4 animate-in fade-in duration-200">
                            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                                <GlassCard className="p-4 space-y-3">
                                    <h4 className="text-sm font-medium text-rose-pine-iris mb-2">Language Features</h4>

                                    <label className="flex items-start gap-3 p-2 rounded-md hover:bg-white/5 cursor-pointer">
                                        <input
                                            type="checkbox"
                                            checked={compilerOptions.v2Preview}
                                            onChange={(e) => updateCompilerOptions({ v2Preview: e.target.checked })}
                                            className="mt-1"
                                        />
                                        <div>
                                            <span className="block text-sm font-medium">Enable V2 Preview Features</span>
                                            <span className="text-xs text-rose-pine-muted">Experimental syntax like nested tuples, nibble immediates, and strict types.</span>
                                        </div>
                                    </label>
                                </GlassCard>

                                <GlassCard className="p-4 space-y-3">
                                    <h4 className="text-sm font-medium text-rose-pine-iris mb-2">Optimization & Build</h4>

                                    <label className="flex items-start gap-3 p-2 rounded-md hover:bg-white/5 cursor-pointer">
                                        <input
                                            type="checkbox"
                                            checked={compilerOptions.enableConstraintCache}
                                            onChange={(e) => updateCompilerOptions({ enableConstraintCache: e.target.checked })}
                                            className="mt-1"
                                        />
                                        <div>
                                            <span className="block text-sm font-medium">Constraint Caching</span>
                                            <span className="text-xs text-rose-pine-muted">Increases compile time but reduces runtime CU usage for repeated checks.</span>
                                        </div>
                                    </label>

                                    <div className="p-2">
                                        <span className="block text-sm font-medium mb-1">Optimization Level</span>
                                        <select
                                            value={compilerOptions.optimizationLevel}
                                            onChange={(e) => updateCompilerOptions({ optimizationLevel: e.target.value as any })}
                                            className="w-full bg-rose-pine-base/50 border border-white/10 rounded-md px-2 py-1 text-sm focus:outline-none"
                                        >
                                            <option value="production">Production (Default)</option>
                                            <option value="debug">Debug (No Optimization)</option>
                                        </select>
                                    </div>
                                </GlassCard>

                                <GlassCard className="p-4 space-y-3">
                                    <h4 className="text-sm font-medium text-rose-pine-iris mb-2">Debugging & Metrics</h4>

                                    <label className="flex items-start gap-3 p-2 rounded-md hover:bg-white/5 cursor-pointer">
                                        <input
                                            type="checkbox"
                                            checked={compilerOptions.enhancedErrors}
                                            onChange={(e) => updateCompilerOptions({ enhancedErrors: e.target.checked })}
                                            className="mt-1"
                                        />
                                        <div>
                                            <span className="block text-sm font-medium">Enhanced Error Reporting</span>
                                            <span className="text-xs text-rose-pine-muted">Provides detailed suggestions and code frames for compilation errors.</span>
                                        </div>
                                    </label>

                                    <label className="flex items-start gap-3 p-2 rounded-md hover:bg-white/5 cursor-pointer">
                                        <input
                                            type="checkbox"
                                            checked={compilerOptions.includeMetrics}
                                            onChange={(e) => updateCompilerOptions({ includeMetrics: e.target.checked })}
                                            className="mt-1"
                                        />
                                        <div>
                                            <span className="block text-sm font-medium">Include Metrics</span>
                                            <span className="text-xs text-rose-pine-muted">Embed compilation statistics in the output artifact.</span>
                                        </div>
                                    </label>

                                    <label className="flex items-start gap-3 p-2 rounded-md hover:bg-white/5 cursor-pointer">
                                        <input
                                            type="checkbox"
                                            checked={compilerOptions.includeDebugInfo}
                                            onChange={(e) => updateCompilerOptions({ includeDebugInfo: e.target.checked })}
                                            className="mt-1"
                                        />
                                        <div>
                                            <span className="block text-sm font-medium">Include Function Metadata</span>
                                            <span className="text-xs text-rose-pine-muted">Embed function names for debugging and execution controls.</span>
                                        </div>
                                    </label>
                                </GlassCard>
                            </div>
                        </div>
                    )}
                </div>

                <div className="p-4 border-t border-white/5 bg-rose-pine-base/30 flex justify-end gap-3">
                    <button onClick={onClose} className="px-4 py-2 rounded-lg text-sm hover:bg-white/5 transition-colors">Close</button>
                </div>
            </GlassCard>
        </div>
    );
}
