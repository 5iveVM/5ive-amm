"use client";

import { useIdeStore } from "@/stores/ide-store";
import { getBaseName, getDirName, isChildOf, SEPARATOR } from "@/utils/file-system";
import {
    ChevronRight,
    ChevronDown,
    FileCode,
    Folder,
    FolderOpen,
    MoreHorizontal,
    FilePlus,
    FolderPlus,
    Trash2,
    Edit2,
    File,
    X,
    AlertCircle,
    Check,
    CornerDownRight,
    Upload,
    Download,
    Hammer,
    Zap, // For .five files
    Braces, // For JSON
    Settings, // For TOML
    Box, // Generic package
    MoreVertical,
    FileText
} from "lucide-react";
import { useState, useMemo, useRef, useEffect } from "react";
import { createPortal } from "react-dom";
import { GlassCard } from "@/components/ui/glass-card";

// Types for Tree Structure
interface TreeNode {
    path: string;
    name: string;
    type: 'file' | 'folder';
    children?: TreeNode[];
}

function buildTree(files: string[]): TreeNode[] {
    const root: TreeNode[] = [];
    const map = new Map<string, TreeNode>();

    const sortedPaths = files.sort();

    const getOrCreateFolder = (path: string): TreeNode => {
        if (path === '') return { path: '', name: 'root', type: 'folder', children: root };

        if (map.has(path)) return map.get(path)!;

        const parentPath = getDirName(path);
        const parentNode = getOrCreateFolder(parentPath);

        const name = getBaseName(path);
        const node: TreeNode = {
            path,
            name,
            type: 'folder',
            children: []
        };

        map.set(path, node);
        parentNode.children!.push(node);
        return node;
    };

    sortedPaths.forEach(path => {
        const dir = getDirName(path);
        const parent = getOrCreateFolder(dir);

        const name = getBaseName(path);
        parent.children!.push({
            path,
            name,
            type: 'file'
        });
    });

    const sortNodes = (nodes: TreeNode[]) => {
        nodes.sort((a, b) => {
            if (a.type !== b.type) return a.type === 'folder' ? -1 : 1;
            return a.name.localeCompare(b.name);
        });
        nodes.forEach(n => {
            if (n.children) sortNodes(n.children);
        });
    };

    sortNodes(root);
    return root;
}

// --- Internal Components ---

interface InlineInputProps {
    initialValue?: string;
    onCommit: (value: string) => void;
    onCancel: () => void;
    placeholder?: string;
    icon?: React.ReactNode;
}

function InlineInput({ initialValue = "", onCommit, onCancel, placeholder, icon }: InlineInputProps) {
    const [value, setValue] = useState(initialValue);
    const inputRef = useRef<HTMLInputElement>(null);

    useEffect(() => {
        inputRef.current?.focus();
        if (initialValue) {
            inputRef.current?.select();
        }
    }, [initialValue]);

    const handleKeyDown = (e: React.KeyboardEvent) => {
        if (e.key === 'Enter') {
            if (value.trim()) onCommit(value.trim());
            else onCancel();
        } else if (e.key === 'Escape') {
            onCancel();
        }
    };

    return (
        <div className="flex items-center gap-1.5 py-1 pr-2 text-xs">
            <span className="opacity-70 w-3.5 flex justify-center">
                <CornerDownRight size={10} className="text-rose-pine-muted" />
            </span>
            <span className="opacity-80 text-rose-pine-iris">
                {icon || <FileCode size={14} />}
            </span>
            <input
                ref={inputRef}
                type="text"
                value={value}
                onChange={(e) => setValue(e.target.value)}
                onKeyDown={handleKeyDown}
                onBlur={() => {
                    if (value.trim()) onCommit(value.trim());
                    else onCancel();
                }}
                className="flex-1 bg-rose-pine-highlight/20 border border-rose-pine-highlight rounded px-1 py-0.5 text-rose-pine-text outline-none min-w-0"
                placeholder={placeholder}
            />
        </div>
    );
}

interface ContextMenuProps {
    x: number;
    y: number;
    items: { label: string; icon?: React.ReactNode; action: () => void; danger?: boolean }[];
    onClose: () => void;
}

function ExplorerContextMenu({ x, y, items, onClose }: ContextMenuProps) {
    const ref = useRef<HTMLDivElement>(null);

    // Ensure menu doesn't go off-screen
    const style: React.CSSProperties = { top: y, left: x };
    if (typeof window !== 'undefined') {
        if (y + 150 > window.innerHeight) style.top = y - 150; // Pop up if near bottom
        if (x + 160 > window.innerWidth) style.left = x - 160;   // Pop left if near right
    }

    useEffect(() => {
        function handleClickOutside(event: MouseEvent) {
            if (ref.current && !ref.current.contains(event.target as Node)) {
                onClose();
            }
        }
        document.addEventListener("mousedown", handleClickOutside);
        return () => document.removeEventListener("mousedown", handleClickOutside);
    }, [onClose]);

    // Use Portal to break out of stacking contexts (z-index wars)
    return createPortal(
        <div
            ref={ref}
            className="fixed z-[9999] min-w-[160px] bg-[#191724]/95 backdrop-blur-xl border border-white/10 rounded-lg shadow-2xl p-1 flex flex-col gap-0.5 animate-in fade-in zoom-in-95 duration-100 ring-1 ring-black/50"
            style={style}
            onContextMenu={(e) => e.preventDefault()}
        >
            {items.map((item, idx) => (
                <button
                    key={idx}
                    onClick={() => { item.action(); onClose(); }}
                    className={`
                        flex items-center gap-2.5 px-2.5 py-1.5 rounded-md text-xs text-left w-full transition-all text-rose-pine-text
                        ${item.danger
                            ? 'hover:bg-rose-pine-love/20 hover:text-rose-pine-love'
                            : 'hover:bg-rose-pine-overlay/50'}
                    `}
                >
                    {item.icon && <span className={item.danger ? 'opacity-100' : 'opacity-70 text-rose-pine-subtle group-hover:text-rose-pine-text'}>{item.icon}</span>}
                    <span className="font-medium">{item.label}</span>
                </button>
            ))}
        </div>,
        document.body
    );
}

interface DeleteModalProps {
    isOpen: boolean;
    itemName: string;
    onConfirm: () => void;
    onCancel: () => void;
}

function DeleteConfirmationModal({ isOpen, itemName, onConfirm, onCancel }: DeleteModalProps) {
    if (!isOpen) return null;

    return createPortal(
        <div className="fixed inset-0 z-[10000] flex items-center justify-center p-4">
            <div className="absolute inset-0 bg-black/60 backdrop-blur-sm animate-in fade-in" onClick={onCancel} />
            <div className="relative w-full max-w-sm bg-[#191724] border border-rose-pine-love/30 rounded-xl shadow-2xl p-6 animate-in zoom-in-95 duration-200">
                <div className="flex flex-col gap-4">
                    <div className="flex items-center gap-3 text-rose-pine-love">
                        <div className="p-2 rounded-full bg-rose-pine-love/10">
                            <AlertCircle size={24} />
                        </div>
                        <h3 className="text-lg font-bold">Delete Item?</h3>
                    </div>

                    <p className="text-sm text-rose-pine-text/80 leading-relaxed">
                        Are you sure you want to delete <span className="font-mono text-rose-pine-love bg-rose-pine-love/5 px-1 py-0.5 rounded border border-rose-pine-love/20">{itemName}</span>?
                        <br />This action cannot be undone.
                    </p>

                    <div className="flex gap-3 justify-end mt-2">
                        <button
                            onClick={onCancel}
                            className="px-4 py-2 rounded-lg text-xs font-medium text-rose-pine-subtle hover:text-rose-pine-text hover:bg-white/5 transition-colors"
                        >
                            Cancel
                        </button>
                        <button
                            onClick={onConfirm}
                            className="px-4 py-2 rounded-lg text-xs font-bold bg-rose-pine-love text-rose-pine-base hover:bg-rose-pine-love/90 transition-colors shadow-lg shadow-rose-pine-love/20 flex items-center gap-2"
                        >
                            <Trash2 size={12} />
                            Delete
                        </button>
                    </div>
                </div>
            </div>
        </div>,
        document.body
    );
}

// --- Main Components ---

interface FileNodeProps {
    node: TreeNode;
    level: number;
    onContextMenu: (e: React.MouseEvent, node: TreeNode) => void;
    renamingPath: string | null;
    onRenameCommit: (path: string, newName: string) => void;
    onRenameCancel: () => void;
    isCreating: { type: 'file' | 'folder', parentPath: string } | null;
    onCreateCommit: (name: string) => void;
    onCreateCancel: () => void;
}

// Helper to get icon for file type
function getFileIcon(name: string) {
    if (name.endsWith('.five') || name.endsWith('.v')) return <Zap size={14} className="text-[#f6c177]" />; // Gold for 5IVE
    if (name.endsWith('.json')) return <Braces size={14} className="text-[#9ccfd8]" />; // Foam for JSON
    if (name.endsWith('.toml')) return <Settings size={14} className="text-[#c4a7e7]" />; // Iris for Config
    if (name.endsWith('.md')) return <FileText size={14} className="text-[#eb6f92]" />; // Love for Docs
    return <FileCode size={14} className="text-rose-pine-subtle" />;
}

function FileNode({
    node,
    level,
    onContextMenu,
    renamingPath,
    onRenameCommit,
    onRenameCancel,
    isCreating,
    onCreateCommit,
    onCreateCancel
}: FileNodeProps) {
    const {
        activeFile,
        openFile,
        expandedFolders,
        toggleFolder,
        deleteFile
    } = useIdeStore();

    const isExpanded = expandedFolders.includes(node.path);
    const isActive = activeFile === node.path;
    const isRenaming = renamingPath === node.path;

    // Check if we are creating a child inside this folder
    const isCreatingChild = isCreating && isCreating.parentPath === node.path;

    // Indentation: 12px per level + 12px base padding
    const paddingLeft = level * 12 + 12;

    const handleClick = (e: React.MouseEvent) => {
        e.stopPropagation();
        if (node.type === 'folder') {
            toggleFolder(node.path);
        } else {
            openFile(node.path);
        }
    };

    const handleContextMenu = (e: React.MouseEvent) => {
        e.preventDefault();
        e.stopPropagation();
        onContextMenu(e, node);
    };

    if (isRenaming) {
        return (
            <div style={{ paddingLeft: `${paddingLeft - 12}px` }} className="py-1">
                <InlineInput
                    initialValue={node.name}
                    onCommit={(newName) => onRenameCommit(node.path, newName)}
                    onCancel={onRenameCancel}
                    icon={node.type === 'folder' ? <Folder size={14} /> : <FileCode size={14} />}
                />
            </div>
        );
    }

    return (
        <div className="relative">
            {/* Indentation Guide (Vertical Line) */}
            {level > 0 && (
                <div
                    className="absolute top-0 bottom-0 w-px bg-white/5"
                    style={{ left: `${(level * 12) + 6}px` }}
                />
            )}

            <div
                className={`
                    group flex items-center gap-1.5 cursor-pointer transition-all duration-150 select-none relative
                    md:h-7 h-10 pr-2
                    ${isActive
                        ? 'bg-white/5 text-rose-pine-text'
                        : 'text-rose-pine-subtle hover:bg-white/5 hover:text-rose-pine-text'}
                `}
                style={{ paddingLeft: `${paddingLeft}px` }}
                onClick={handleClick}
                onContextMenu={handleContextMenu}
            >
                {/* Active Border Marker */}
                {isActive && (
                    <div className="absolute left-0 top-0 bottom-0 w-[2px] bg-rose-pine-iris" />
                )}

                {/* Icon */}
                <span className="opacity-70 shrink-0 flex items-center justify-center w-4">
                    {node.type === 'folder' ? (
                        isExpanded ? <ChevronDown size={14} /> : <ChevronRight size={14} />
                    ) : (
                        // Placeholder for alignment if no arrow needed (files)
                        <span className="w-3.5" />
                    )}
                </span>

                <span className={`shrink-0 flex items-center ${node.type === 'folder' ? 'text-rose-pine-iris' : ''}`}>
                    {node.type === 'folder' ? (
                        isExpanded ? <FolderOpen size={14} /> : <Folder size={14} />
                    ) : (
                        getFileIcon(node.name)
                    )}
                </span>

                <span className="truncate flex-1 text-[13px] md:text-xs font-medium tracking-tight opacity-90">{node.name}</span>

                {/* Quick Actions (Hover) */}
                <div className="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                    {/* Only show delete on hover for utility */}
                    <button
                        className="p-1 hover:bg-rose-pine-love/20 hover:text-rose-pine-love rounded text-rose-pine-muted transition-colors"
                        onClick={(e) => {
                            e.stopPropagation();
                            onContextMenu(e, node); // Trigger context menu for now, or direct delete
                        }}
                    >
                        <MoreHorizontal size={12} />
                    </button>
                </div>
            </div>

            {/* Children and Creation Input */}
            {node.type === 'folder' && isExpanded && (
                <div>
                    {/* Existing Children */}
                    {node.children?.map(child => (
                        <FileNode
                            key={child.path}
                            node={child}
                            level={level + 1}
                            onContextMenu={onContextMenu}
                            renamingPath={renamingPath}
                            onRenameCommit={onRenameCommit}
                            onRenameCancel={onRenameCancel}
                            isCreating={isCreating}
                            onCreateCommit={onCreateCommit}
                            onCreateCancel={onCreateCancel}
                        />
                    ))}

                    {/* Creation Input Slot */}
                    {isCreatingChild && (
                        <div style={{ paddingLeft: `${paddingLeft + 12}px` }} className="py-1"> {/* Indent one level deeper */}
                            <InlineInput
                                onCommit={onCreateCommit}
                                onCancel={onCreateCancel}
                                placeholder={isCreating.type === 'folder' ? "Folder name..." : "File name..."}
                                icon={isCreating.type === 'folder' ? <Folder size={14} /> : <FileCode size={14} />}
                            />
                        </div>
                    )}
                </div>
            )}
        </div>
    );
}

export default function ProjectExplorer({ onCompile }: { onCompile?: (path: string) => void }) {
    const { files, createFile, deleteFile, renameFile, expandedFolders, toggleFolder } = useIdeStore();

    const tree = useMemo(() => buildTree(Object.keys(files)), [files]);

    // UI State
    const [contextMenu, setContextMenu] = useState<{ x: number, y: number, path: string, type: 'file' | 'folder' } | null>(null);
    const [renamingPath, setRenamingPath] = useState<string | null>(null);
    const [isCreating, setIsCreating] = useState<{ type: 'file' | 'folder', parentPath: string } | null>(null);
    const [itemToDelete, setItemToDelete] = useState<{ path: string, name: string } | null>(null);

    // Handlers
    const handleNodeContextMenu = (e: React.MouseEvent, node: TreeNode) => {
        setContextMenu({
            x: e.clientX,
            y: e.clientY,
            path: node.path,
            type: node.type
        });
    };

    const handleRenameCommit = (path: string, newName: string) => {
        if (!newName || newName === getBaseName(path)) {
            setRenamingPath(null);
            return;
        }

        const dir = getDirName(path);
        const newPath = dir ? `${dir}/${newName}` : newName;

        if (files[newPath]) {
            alert('File already exists!');
            return;
        }

        renameFile(path, newPath);
        setRenamingPath(null);
    };

    const handleCreateCommit = (name: string) => {
        if (!isCreating) return;
        const { type, parentPath } = isCreating;

        const path = parentPath ? `${parentPath}/${name}` : name;

        if (files[path]) {
            alert('File already exists!');
            return;
        }

        if (type === 'file') {
            createFile(path, "// New file");
        } else {
            createFile(`${path}/.keep`, "");
        }
        setIsCreating(null);
    };

    const handleDeleteConfirm = () => {
        if (itemToDelete) {
            deleteFile(itemToDelete.path);
            setItemToDelete(null);
        }
    };

    // Context Menu Actions
    const menuItems = useMemo(() => {
        if (!contextMenu) return [];

        const items = [];

        if (contextMenu.type === 'folder') {
            items.push(
                { label: 'New File', icon: <FilePlus size={14} />, action: () => setIsCreating({ type: 'file', parentPath: contextMenu.path }) },
                { label: 'New Folder', icon: <FolderPlus size={14} />, action: () => setIsCreating({ type: 'folder', parentPath: contextMenu.path }) },
                { label: 'Rename', icon: <Edit2 size={14} />, action: () => setRenamingPath(contextMenu.path) },
            );
        } else {
            // File Actions
            // Add Compile for .v / .five files
            if (contextMenu.path.endsWith('.v') || contextMenu.path.endsWith('.five')) {
                items.push(
                    { label: 'Compile', icon: <Hammer size={14} />, action: () => onCompile?.(contextMenu.path) }
                );
            }

            items.push(
                { label: 'Rename', icon: <Edit2 size={14} />, action: () => setRenamingPath(contextMenu.path) },
            );
        }

        items.push({
            label: 'Delete',
            icon: <Trash2 size={14} />,
            danger: true,
            action: () => setItemToDelete({ path: contextMenu.path, name: getBaseName(contextMenu.path) })
        });

        return items;
    }, [contextMenu, onCompile]);


    const handleRootNewFile = () => {
        setIsCreating({ type: 'file', parentPath: '' });
    };

    const handleRootNewFolder = () => {
        setIsCreating({ type: 'folder', parentPath: '' });
    };

    return (
        <>
            <div className="h-full flex flex-col bg-transparent" onContextMenu={(e) => {
                if (e.target === e.currentTarget) {
                    e.preventDefault();
                    setContextMenu({ x: e.clientX, y: e.clientY, path: '', type: 'folder' });
                }
            }}>
                {/* Header / Actions */}
                <div className="flex items-center justify-between h-9 px-3 border-b border-white/5 bg-transparent">
                    <span className="text-[11px] font-bold text-rose-pine-muted uppercase tracking-widest pl-1">Explorer</span>
                    <div className="flex gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity duration-300">
                        {/* Action buttons appear on hover of the container - wait, applying group to main/parent in page.tsx might be needed, or just always show but slightly dimmed */}
                        <button onClick={handleRootNewFile} className="p-1.5 hover:bg-white/10 rounded-md text-rose-pine-muted hover:text-rose-pine-text transition-colors" title="New File">
                            <FilePlus size={14} />
                        </button>
                        <button onClick={handleRootNewFolder} className="p-1.5 hover:bg-white/10 rounded-md text-rose-pine-muted hover:text-rose-pine-text transition-colors" title="New Folder">
                            <FolderPlus size={14} />
                        </button>
                    </div>
                </div>

                {/* Tree */}
                <div className="flex-1 overflow-y-auto custom-scrollbar py-2" onClick={() => setContextMenu(null)}>
                    {isCreating && isCreating.parentPath === '' && (
                        <div className="pl-3">
                            <InlineInput
                                onCommit={handleCreateCommit}
                                onCancel={() => setIsCreating(null)}
                                placeholder={isCreating.type === 'folder' ? "Folder name..." : "File name..."}
                                icon={isCreating.type === 'folder' ? <Folder size={14} /> : <FileCode size={14} />}
                            />
                        </div>
                    )}

                    {tree.map(node => (
                        <FileNode
                            key={node.path}
                            node={node}
                            level={0}
                            onContextMenu={handleNodeContextMenu}
                            renamingPath={renamingPath}
                            onRenameCommit={handleRenameCommit}
                            onRenameCancel={() => setRenamingPath(null)}
                            isCreating={isCreating}
                            onCreateCommit={handleCreateCommit}
                            onCreateCancel={() => setIsCreating(null)}
                        />
                    ))}
                </div>
            </div>

            {contextMenu && (
                <ExplorerContextMenu
                    x={contextMenu.x}
                    y={contextMenu.y}
                    items={menuItems}
                    onClose={() => setContextMenu(null)}
                />
            )}

            <DeleteConfirmationModal
                isOpen={!!itemToDelete}
                itemName={itemToDelete?.name || ''}
                onConfirm={handleDeleteConfirm}
                onCancel={() => setItemToDelete(null)}
            />
        </>
    );
}
