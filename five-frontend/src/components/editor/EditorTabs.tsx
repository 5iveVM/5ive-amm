"use client";

import { useIdeStore } from "@/stores/ide-store";
import { getBaseName } from "@/utils/file-system";
import { X } from "lucide-react";

export default function EditorTabs() {
    const {
        openFiles,
        activeFile,
        setActiveFile,
        closeFile,
        files
    } = useIdeStore();

    if (openFiles.length === 0) {
        return null;
    }

    return (
        <div className="flex items-center gap-2 overflow-x-auto no-scrollbar py-1">
            {openFiles.map((path) => {
                const isActive = path === activeFile;
                const name = getBaseName(path);

                return (
                    <div
                        key={path}
                        className={`
                            group flex items-center gap-2 px-4 py-1.5 rounded-full text-xs cursor-pointer select-none transition-all duration-300 relative border
                            ${isActive
                                ? 'bg-rose-pine-iris/10 border-rose-pine-iris/30 text-rose-pine-text font-medium shadow-[0_0_10px_rgba(196,167,231,0.1)]'
                                : 'bg-rose-pine-surface/40 border-white/5 text-rose-pine-muted hover:bg-rose-pine-surface/60 hover:text-rose-pine-subtle hover:border-white/10'
                            }
                        `}
                        onClick={() => setActiveFile(path)}
                    >
                        {/* File Icon (Simple) */}
                        <span className={`opacity-80 ${isActive ? 'text-rose-pine-iris' : 'text-rose-pine-muted'}`}>
                            {path.endsWith('.five') || path.endsWith('.v') ? '⚡' : '#'}
                        </span>

                        <span className="truncate max-w-[150px]">{name}</span>

                        {/* Close Button */}
                        <button
                            onClick={(e) => {
                                e.stopPropagation();
                                closeFile(path);
                            }}
                            className={`p-0.5 rounded-full opacity-0 group-hover:opacity-100 transition-all hover:bg-rose-pine-love/20 hover:text-rose-pine-love ${isActive ? 'text-rose-pine-subtle' : 'text-rose-pine-muted'} ml-1`}
                        >
                            <X size={12} />
                        </button>
                    </div>
                );
            })}
        </div>
    );
}
