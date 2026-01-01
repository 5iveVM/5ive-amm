"use client";

import { useEffect, useState } from "react";
import { GlassCard, GlassHeader } from "@/components/ui/glass-card";
import { X, Folder, FileCode, ChevronRight, ChevronDown, Loader2 } from "lucide-react";
import { useIdeStore, DEFAULT_TOML } from "@/stores/ide-store";
import EXAMPLES from "@/data/examples.json";

interface ScriptFile {
    name: string;
    path: string;
    content?: string;
    additional_files?: Record<string, string>;
}

interface ScriptCategory {
    name: string;
    files: ScriptFile[];
}

interface ScriptBrowserModalProps {
    isOpen: boolean;
    onClose: () => void;
}

export default function ScriptBrowserModal({ isOpen, onClose }: ScriptBrowserModalProps) {
    const { setCode, setFilename, createFile, resetProject } = useIdeStore();
    const [categories, setCategories] = useState<ScriptCategory[]>([]);
    const [loading, setLoading] = useState(true);
    const [expandedCategories, setExpandedCategories] = useState<Record<string, boolean>>({});
    const [loadingFile, setLoadingFile] = useState<string | null>(null);

    useEffect(() => {
        if (isOpen && categories.length === 0) {
            fetchScripts();
        }
    }, [isOpen]);

    // ... inside component ...

    const fetchScripts = async () => {
        setLoading(true);
        try {
            // Simulate network delay for effect
            await new Promise(resolve => setTimeout(resolve, 300));
            setCategories(EXAMPLES.categories);
        } catch (error) {
            console.error("Failed to load scripts:", error);
        } finally {
            setLoading(false);
        }
    };

    const toggleCategory = (categoryName: string) => {
        setExpandedCategories(prev => ({
            ...prev,
            [categoryName]: !prev[categoryName]
        }));
    };

    const loadScript = async (file: ScriptFile) => {
        setLoadingFile(file.path);
        try {
            let content = file.content;

            if (!content) {
                // Fetch from static file
                try {
                    const res = await fetch(`/examples/${file.path}`);
                    if (!res.ok) throw new Error(`Failed to fetch ${file.path}`);
                    content = await res.text();
                } catch (e) {
                    console.error("Error fetching script content:", e);
                    return;
                }
            }

            // Simulate partial loading delay if content was already there
            if (file.content) {
                await new Promise(resolve => setTimeout(resolve, 200));
            }

            // Initialize new project with the script and default config
            const initialFiles: Record<string, string> = {
                [file.name]: content || "",
                'five.toml': DEFAULT_TOML
            };

            // Add additional files if present
            if (file.additional_files) {
                // If additional_files values are paths (strings) and start with something that looks like a path, fetch them
                // Otherwise assume it's content (though our new JSON should be paths)

                await Promise.all(Object.entries(file.additional_files).map(async ([name, val]) => {
                    let fileContent = val;
                    // Check if val is a path (our logic from extract script made it a path relative to examples/ root)
                    // Simple heuristic: if it doesn't contain newlines and ends with .v or .json etc, it's likely a path
                    // Or we just rely on the fact we just migrated them.

                    try {
                        const res = await fetch(`/examples/${val}`);
                        if (res.ok) {
                            fileContent = await res.text();
                        }
                    } catch (e) {
                        console.warn(`Failed to fetch additional file ${name}, using value as content/fallback`);
                    }

                    initialFiles[name] = fileContent;
                }));
            }

            resetProject(initialFiles, file.name);
            onClose();

        } catch (error) {
            console.error("Failed to load script:", error);
        } finally {
            setLoadingFile(null);
        }
    };

    if (!isOpen) return null;

    return (
        <div className="fixed inset-0 z-[60] flex items-center justify-center bg-black/50 backdrop-blur-sm p-4">
            <GlassCard className="w-full max-w-2xl max-h-[80vh] flex flex-col border-rose-pine-hl-high/50 shadow-2xl animate-in zoom-in-95 duration-200">
                <GlassHeader title="Example Scripts" className="bg-rose-pine-base/50">
                    <button onClick={onClose} className="p-1 hover:bg-white/10 rounded-md transition-colors text-rose-pine-subtle hover:text-rose-pine-text">
                        <X size={20} />
                    </button>
                </GlassHeader>

                <div className="flex-1 overflow-y-auto p-4 custom-scrollbar pb-20 md:pb-4">
                    {loading ? (
                        <div className="flex flex-col items-center justify-center py-12 text-rose-pine-subtle gap-3">
                            <Loader2 className="animate-spin" size={24} />
                            <span>Loading library...</span>
                        </div>
                    ) : (
                        <div className="space-y-3">
                            {categories.map((category) => (
                                <div key={category.name} className="overflow-hidden rounded-xl border border-white/10 bg-rose-pine-surface/20 shadow-sm">
                                    <button
                                        onClick={() => toggleCategory(category.name)}
                                        className="w-full flex items-center gap-3 p-4 hover:bg-rose-pine-surface/50 transition-all text-left group"
                                    >
                                        <div className="p-2 rounded-lg bg-rose-pine-overlay/50 group-hover:bg-rose-pine-overlay transition-colors">
                                            {expandedCategories[category.name] ? (
                                                <ChevronDown size={18} className="text-rose-pine-foam" />
                                            ) : (
                                                <ChevronRight size={18} className="text-rose-pine-muted group-hover:text-rose-pine-subtle" />
                                            )}
                                        </div>
                                        <div className="flex flex-col">
                                            <span className="font-bold text-rose-pine-text text-sm md:text-base">{category.name}</span>
                                            <span className="text-xs text-rose-pine-muted">{category.files.length} examples</span>
                                        </div>
                                    </button>

                                    {expandedCategories[category.name] && (
                                        <div className="border-t border-white/5 bg-black/20">
                                            {category.files.map((file) => (
                                                <button
                                                    key={file.path}
                                                    onClick={() => loadScript(file)}
                                                    disabled={loadingFile === file.path}
                                                    className="w-full flex items-center gap-3 px-4 py-3 pl-14 hover:bg-white/5 transition-colors text-left text-sm text-rose-pine-subtle hover:text-rose-pine-text group disabled:opacity-50 border-b border-white/5 last:border-0"
                                                >
                                                    <FileCode size={16} className="group-hover:text-rose-pine-gold transition-colors shrink-0" />
                                                    <span className="truncate font-medium">{file.name}</span>
                                                    {loadingFile === file.path && <Loader2 size={14} className="ml-auto animate-spin" />}
                                                </button>
                                            ))}
                                        </div>
                                    )}
                                </div>
                            ))}
                        </div>
                    )}
                </div>
            </GlassCard>
        </div>
    );
}
