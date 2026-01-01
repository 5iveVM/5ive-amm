"use client";

import { useEffect, useState } from "react";
import { Folder, FileCode, ChevronRight, ChevronDown, Loader2 } from "lucide-react";
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

interface ScriptBrowserProps {
    onLoad?: () => void;
}

export default function ScriptBrowser({ onLoad }: ScriptBrowserProps) {
    const { resetProject } = useIdeStore();
    const [categories, setCategories] = useState<ScriptCategory[]>([]);
    const [loading, setLoading] = useState(true);
    const [expandedCategories, setExpandedCategories] = useState<Record<string, boolean>>({});
    const [loadingFile, setLoadingFile] = useState<string | null>(null);

    useEffect(() => {
        fetchScripts();
    }, []);

    const fetchScripts = async () => {
        setLoading(true);
        try {
            // Simulate network delay for effect
            await new Promise(resolve => setTimeout(resolve, 300));
            setCategories(EXAMPLES.categories);

            // Auto-expand first category
            if (EXAMPLES.categories.length > 0) {
                setExpandedCategories({ [EXAMPLES.categories[0].name]: true });
            }
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
            // Find content from static data if available
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

            // Simulate loading
            if (file.content) {
                await new Promise(resolve => setTimeout(resolve, 200));
            }

            // Generate a safe project name from the file name
            const projectName = file.name.replace(/\.v$/, '').replace(/[^a-zA-Z0-9_]/g, '_').toLowerCase();
            const projectFileName = `src/${file.name}`; // Always put in src/

            // Generate a unique TOML config for this project
            const projectToml = `[project]
name = "${projectName}"
version = "0.1.0"
description = "Example: ${file.name}"
target = "vm"

[build]
output_artifact_name = "${projectName}"

[deploy]
network = "devnet"
program_id = "5ive..."`;

            // Initialize new project with the script in src/ and custom config
            const initialFiles: Record<string, string> = {
                [projectFileName]: content || "",
                'five.toml': projectToml
            };

            // Add additional files if present (handling src/ prefix if needed)
            if (file.additional_files) {
                await Promise.all(Object.entries(file.additional_files).map(async ([name, val]) => {
                    let fileContent = val;
                    try {
                        // Try to fetch if it looks like a path (no newlines, ends in .v/.json etc)
                        // Since we know our data structure, we can just fetch
                        const res = await fetch(`/examples/${val}`);
                        if (res.ok) {
                            fileContent = await res.text();
                        }
                    } catch (e) {
                        console.warn(`Failed to fetch additional file ${name}`);
                    }

                    // If additional file has no path prefix, put it in src/
                    const path = name.includes('/') ? name : `src/${name}`;
                    initialFiles[path] = fileContent;
                }));
            }

            resetProject(initialFiles, projectFileName);

            if (onLoad) onLoad();

        } catch (error) {
            console.error("Failed to load script:", error);
        } finally {
            setLoadingFile(null);
        }
    };

    return (
        <div className="flex-1 overflow-y-auto custom-scrollbar h-full">
            {loading ? (
                <div className="flex flex-col items-center justify-center py-12 text-rose-pine-subtle gap-3">
                    <Loader2 className="animate-spin" size={24} />
                    <span>Loading examples...</span>
                </div>
            ) : (
                <div className="space-y-1 p-2">
                    {categories.map((category) => (
                        <div key={category.name} className="overflow-hidden rounded-lg">
                            <button
                                onClick={() => toggleCategory(category.name)}
                                className="w-full flex items-center gap-2 p-2 hover:bg-white/5 transition-all text-left group rounded-lg"
                            >
                                <div className="text-rose-pine-subtle group-hover:text-rose-pine-text transition-colors">
                                    {expandedCategories[category.name] ? (
                                        <ChevronDown size={14} />
                                    ) : (
                                        <ChevronRight size={14} />
                                    )}
                                </div>
                                <div className="flex flex-col">
                                    <span className="font-medium text-rose-pine-text text-sm">{category.name}</span>
                                </div>
                            </button>

                            {expandedCategories[category.name] && (
                                <div className="ml-2 pl-2 border-l border-white/5 space-y-0.5 mt-1">
                                    {category.files.map((file) => (
                                        <button
                                            key={file.path}
                                            onClick={() => loadScript(file)}
                                            disabled={loadingFile === file.path}
                                            className="w-full flex items-center gap-2 px-3 py-2 rounded-md hover:bg-white/5 transition-colors text-left text-xs text-rose-pine-subtle hover:text-rose-pine-text group disabled:opacity-50"
                                        >
                                            <FileCode size={14} className="group-hover:text-rose-pine-gold transition-colors shrink-0" />
                                            <span className="truncate">{file.name}</span>
                                            {loadingFile === file.path && <Loader2 size={12} className="ml-auto animate-spin" />}
                                        </button>
                                    ))}
                                </div>
                            )}
                        </div>
                    ))}
                </div>
            )}
        </div>
    );
}
