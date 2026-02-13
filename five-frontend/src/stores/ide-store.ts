import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { parse as parseToml } from '@iarna/toml';
import type { NetworkType } from '@/lib/network-config';

const MAX_PERSISTED_FILE_BYTES = 200 * 1024;

const filterPersistedFiles = (files: Record<string, string>) => {
    return Object.fromEntries(
        Object.entries(files).filter(([path, content]) => {
            if (path.startsWith('target/')) return false;
            return content.length <= MAX_PERSISTED_FILE_BYTES;
        })
    );
};

export interface LogEntry {
    id: string;
    timestamp: Date;
    message: string;
    type: 'info' | 'success' | 'warning' | 'error' | 'system';
}

export interface VmState {
    stack: string[];
    instructionPointer: number;
    computeUnits: number;
    memory: Uint8Array | null;
}

export interface DeploymentRecord {
    scriptAccount: string;
    programId: string;
    deployedAt: number;
    transactionId?: string;
}

export interface CompilerOptions {
    v2Preview: boolean;
    enhancedErrors: boolean;
    analysisVisible: boolean;
    enableConstraintCache: boolean;
    includeDebugInfo: boolean;
    includeMetrics: boolean;
    optimizationLevel: 'production' | 'debug';
}

// ==================== Workspace Types ====================

export type LinkType = 'inline' | 'external';

export interface WorkspaceState {
    root: string;
    members: string[];
    packages: PackageInfo[];
    activePackage: string | null;
}

export interface PackageInfo {
    name: string;
    path: string;
    entryPoint?: string;
    dependencies: { name: string; link: LinkType }[];
    bytecode?: Uint8Array;
    address?: string;
}

interface IdeState {
    code: string;
    bytecode: Uint8Array | null;
    abi: any | null;
    logs: LogEntry[];
    isCompiling: boolean;
    isExecuting: boolean;
    vmState: VmState;
    currentFilename: string;
    selectedFunctionIndex: number;
    executionParams: any[];
    executionAccounts: string[];

    // Project / File State
    // Project / File State
    files: Record<string, string>; // Map of "path/to/file.five" -> content
    activeFile: string | null;     // Currently focused file path
    openFiles: string[];           // List of paths open in tabs
    expandedFolders: string[];     // List of expanded folder paths

    projectConfig: any | null; // five.toml
    compilerOptions: CompilerOptions;

    // Workspace State
    workspace: WorkspaceState | null;

    // On-Chain State
    deployments: Record<string, DeploymentRecord>; // filename -> deployment info
    isDeploying: boolean;
    isOnChainExecuting: boolean;
    onChainLogs: string[];
    contractAddress: string | null; // Currently selected contract for execution
    rpcEndpoint: string;
    selectedNetwork: NetworkType; // localnet or devnet

    // Cost State
    estimatedCost: number | null;
    estimatedRent: number | null;
    estimatedDeployFee: number | null;
    deployFeeLamports: number | null;
    executeFeeBps: number | null;
    adminAccount: string | null; // Admin account for fee collection
    solPrice: number;
    setEstimatedCost: (cost: number | null) => void;
    setEstimatedRent: (rent: number | null) => void;
    setEstimatedDeployFee: (fee: number | null) => void;
    setFeeConfig: (deployFeeLamports: number | null, executeFeeBps: number | null, adminAccount: string | null) => void;
    setSolPrice: (price: number) => void;

    // Actions
    setCode: (code: string) => void;
    setFilename: (filename: string) => void;
    setBytecode: (bytecode: Uint8Array | null) => void;
    setAbi: (abi: any | null) => void;
    setIsCompiling: (isCompiling: boolean) => void;
    setIsExecuting: (isExecuting: boolean) => void;
    setSelectedFunctionIndex: (index: number) => void;
    setExecutionParams: (params: any[]) => void;
    setExecutionAccounts: (accounts: string[]) => void;
    parseTestParams: () => void;
    appendLog: (message: string, type?: LogEntry['type']) => void;
    clearLogs: () => void;
    updateVmState: (state: Partial<VmState>) => void;
    resetVmState: () => void;

    // File Actions
    // VFS Actions
    openFile: (path: string) => void;
    closeFile: (path: string) => void;
    closeAllFiles: () => void;
    toggleFolder: (path: string) => void;
    createFile: (path: string, content?: string, shouldOpen?: boolean) => void;
    createFolder: (path: string) => void; // Uses a placeholder logic or just UI state
    updateFileContent: (path: string, content: string) => void;
    deleteFile: (path: string) => void;
    renameFile: (oldPath: string, newPath: string) => void;
    setActiveFile: (path: string) => void;
    setProjectConfig: (config: any) => void;
    updateCompilerOptions: (options: Partial<CompilerOptions>) => void;

    // Workspace Actions
    detectWorkspace: () => void;
    setActivePackage: (packageName: string | null) => void;
    updatePackageBytecode: (packageName: string, bytecode: Uint8Array) => void;
    syncWorkspaceToLsp: (lspSetDocument: (uri: string, source: string) => Promise<void>) => Promise<void>;

    // On-Chain Actions
    toggleAnalysis: () => void;
    addDeployment: (filename: string, deployment: DeploymentRecord) => void;
    setIsDeploying: (isDeploying: boolean) => void;
    setIsOnChainExecuting: (isOnChainExecuting: boolean) => void;
    setOnChainLogs: (logs: string[]) => void;
    appendOnChainLog: (log: string) => void;
    setContractAddress: (address: string | null) => void;
    setRpcEndpoint: (endpoint: string) => void;
    setSelectedNetwork: (network: NetworkType) => void;
    resetProject: (initialFiles?: Record<string, string>, initialActive?: string) => void;
}

export const DEFAULT_COUNTER_CODE = `// New 5IVE Project
// Start writing your contract here...

account State {
    value: u64;
}

pub initialize(@init state: State) {
    state.value = 0;
    print("Hello 5IVE!");
}

pub main() {
    // Main entry point
}
`;

export const DEFAULT_TOML = `[project]
name = "my_counter_project"
version = "0.1.0"
description = "My 5IVE Project"
target = "vm"

[build]
output_artifact_name = "counter_v1"

[deploy]
network = "devnet"
program_id = "5ive..."`;

export const useIdeStore = create<IdeState>()(
    persist(
        (set) => ({
            code: DEFAULT_COUNTER_CODE,
            bytecode: null,
            abi: null,
            logs: [],
            isCompiling: false,
            isExecuting: false,
            vmState: {
                stack: [],
                instructionPointer: 0,
                computeUnits: 0,
                memory: null,
            },
            currentFilename: 'counter.v',

            // Project State
            // Project State
            activeFile: 'src/main.v',
            openFiles: ['src/main.v'],
            expandedFolders: ['src'],
            files: {
                'src/main.v': DEFAULT_COUNTER_CODE,
                'five.toml': DEFAULT_TOML
            },
            projectConfig: null,
            compilerOptions: {
                v2Preview: false,
                enhancedErrors: true,
                analysisVisible: false,
                enableConstraintCache: false,
                includeDebugInfo: true,
                includeMetrics: false,
                optimizationLevel: 'production'
            },

            // Workspace State
            workspace: null,

            // On-Chain Initial State
            deployments: {},
            isDeploying: false,
            isOnChainExecuting: false,
            onChainLogs: [],
            contractAddress: null,
            rpcEndpoint: 'http://127.0.0.1:8899', // Default to Localnet
            selectedNetwork: 'localnet' as NetworkType,

            // Execution state
            selectedFunctionIndex: 0,
            executionParams: [],
            executionAccounts: [],

            // Cost State
            estimatedCost: null,
            estimatedRent: null,
            estimatedDeployFee: null,
            deployFeeLamports: null,
            executeFeeBps: null,
            adminAccount: null,
            solPrice: 0,
            setEstimatedCost: (cost) => set({ estimatedCost: cost }),
            setEstimatedRent: (rent) => set({ estimatedRent: rent }),
            setEstimatedDeployFee: (fee) => set({ estimatedDeployFee: fee }),
            setFeeConfig: (deployFeeLamports, executeFeeBps, adminAccount) => set({ deployFeeLamports, executeFeeBps, adminAccount }),
            setSolPrice: (price) => set({ solPrice: price }),

            // Action Implementations
            setRpcEndpoint: (endpoint: string) => set({ rpcEndpoint: endpoint }),
            setSelectedNetwork: (network: NetworkType) => set({ selectedNetwork: network }),


            setCode: (code) => set((state) => ({
                code,
                files: { ...state.files, [state.activeFile!]: code }
            })),

            setFilename: (filename) => set((state) => {
                return { activeFile: filename, currentFilename: filename };
            }),

            setBytecode: (bytecode) => set({ bytecode }),
            setAbi: (abi) => set({ abi }),
            setIsCompiling: (isCompiling) => set({ isCompiling }),
            setIsExecuting: (isExecuting) => set({ isExecuting }),

            setSelectedFunctionIndex: (index) => set({ selectedFunctionIndex: index }),
            setExecutionParams: (params) => set({ executionParams: params }),
            setExecutionAccounts: (accounts) => set({ executionAccounts: accounts }),

            parseTestParams: () => set((state) => {
                // Regex to find @test-params annotation: // @test-params 42 "hello"
                const match = state.code.match(/\/\/\s*@test-params\s+(.+)/);
                if (match && match[1]) {
                    try {
                        // Initial naive parsing - split by space, handle basic types
                        const paramStrings = match[1].trim().split(/\s+/);
                        const params = paramStrings.map(p => {
                            if (p === 'true') return true;
                            if (p === 'false') return false;
                            const num = Number(p);
                            return isNaN(num) ? p.replace(/^"|"$/g, '') : num;
                        });
                        return { executionParams: params };
                    } catch (e) {
                        console.error("Failed to parse test params", e);
                        return {};
                    }
                }
                return {};
            }),

            appendLog: (message, type = 'info') => set((state) => ({
                logs: [
                    ...state.logs,
                    {
                        id: crypto.randomUUID(),
                        timestamp: new Date(),
                        message,
                        type,
                    }
                ]
            })),

            clearLogs: () => set({ logs: [] }),

            updateVmState: (newState) => set((state) => ({
                vmState: { ...state.vmState, ...newState }
            })),

            resetVmState: () => set({
                vmState: {
                    stack: [],
                    instructionPointer: 0,
                    computeUnits: 0,
                    memory: null,
                }
            }),

            // VFS Actions Implementation

            toggleAnalysis: () => set((state) => ({ compilerOptions: { ...state.compilerOptions, analysisVisible: !state.compilerOptions.analysisVisible } })),

            // On-Chain Actions
            addDeployment: (filename, deployment) => set((state) => ({
                deployments: { ...state.deployments, [filename]: deployment },
                contractAddress: deployment.scriptAccount // Auto-select new deployment
            })),
            setIsDeploying: (isDeploying) => set({ isDeploying }),
            setIsOnChainExecuting: (isOnChainExecuting) => set({ isOnChainExecuting }),
            setOnChainLogs: (logs) => set({ onChainLogs: logs }),
            appendOnChainLog: (log) => set((state) => ({ onChainLogs: [...state.onChainLogs, log] })),
            setContractAddress: (address) => set({ contractAddress: address }),

            openFile: (path) => set((state) => {
                if (!state.files[path]) return {}; // File doesn't exist?
                const isOpen = state.openFiles.includes(path);
                return {
                    activeFile: path,
                    openFiles: isOpen ? state.openFiles : [...state.openFiles, path],
                    currentFilename: path,
                    code: state.files[path]
                };
            }),

            closeFile: (path) => set((state) => {
                const newOpenFiles = state.openFiles.filter(p => p !== path);

                // If closing active file, switch to next available
                let newActiveFile = state.activeFile;
                if (state.activeFile === path) {
                    newActiveFile = newOpenFiles.length > 0 ? newOpenFiles[newOpenFiles.length - 1] : null;
                }

                return {
                    openFiles: newOpenFiles,
                    activeFile: newActiveFile,
                    code: newActiveFile ? state.files[newActiveFile] : "",
                    currentFilename: newActiveFile || ""
                };
            }),

            closeAllFiles: () => set({ openFiles: [], activeFile: null, code: "" }),

            toggleFolder: (path) => set((state) => {
                const isExpanded = state.expandedFolders.includes(path);
                return {
                    expandedFolders: isExpanded
                        ? state.expandedFolders.filter(p => p !== path)
                        : [...state.expandedFolders, path]
                };
            }),

            createFile: (path, content = "", shouldOpen = true) => set((state) => ({
                files: { ...state.files, [path]: content },
                activeFile: shouldOpen ? path : state.activeFile,
                openFiles: shouldOpen ? (state.openFiles.includes(path) ? state.openFiles : [...state.openFiles, path]) : state.openFiles,
                code: shouldOpen ? content : state.code,
                currentFilename: shouldOpen ? path : state.currentFilename
            })),

            createFolder: (path) => {
                // In this flat VFS, folders are implicit. 
                // But we can add it to expandedFolders so it shows up if we had strict folder logic.
                // For now, creating a folder usually means preparing UI to create a file INSIDE it.
                // We can just ensure it's expanded.
                set((state) => ({
                    expandedFolders: [...state.expandedFolders, path]
                }));
            },

            updateFileContent: (path, content) => set((state) => {
                const changes: any = {
                    files: { ...state.files, [path]: content },
                    code: path === state.activeFile ? content : state.code
                };

                if (path === 'five.toml' || path === 'Five.toml') {
                    try {
                        const parsed = parseToml(content);
                        changes.projectConfig = parsed;
                    } catch (e) {
                        console.warn("Failed to parse five.toml", e);
                    }
                }

                return changes;
            }),

            deleteFile: (path) => set((state) => {
                // Logic to delete file OR folder (recursive delete)
                // Check if path is a folder (by checking if other files start with path + /)
                const isFolder = Object.keys(state.files).some(k => k.startsWith(path + '/'));

                const newFiles = { ...state.files };
                const pathsToDelete = isFolder
                    ? Object.keys(newFiles).filter(k => k.startsWith(path + '/') || k === path)
                    : [path];

                pathsToDelete.forEach(p => delete newFiles[p]);

                // Close deleted files
                const newOpenFiles = state.openFiles.filter(p => !pathsToDelete.includes(p));

                // Determine new active file
                let newActive = state.activeFile;
                if (state.activeFile && pathsToDelete.includes(state.activeFile)) {
                    newActive = newOpenFiles.length > 0 ? newOpenFiles[newOpenFiles.length - 1] : null;
                }

                return {
                    files: newFiles,
                    openFiles: newOpenFiles,
                    activeFile: newActive,
                    currentFilename: newActive || "",
                    code: newActive ? newFiles[newActive] : ""
                };
            }),

            renameFile: (oldPath, newPath) => set((state) => {
                const newFiles = { ...state.files };
                const pathsToRename = Object.keys(newFiles).filter(k => k === oldPath || k.startsWith(oldPath + '/'));

                pathsToRename.forEach(path => {
                    const suffix = path.slice(oldPath.length);
                    const targetPath = newPath + suffix;
                    newFiles[targetPath] = newFiles[path];
                    delete newFiles[path];
                });

                // Update Open Files list
                const newOpenFiles = state.openFiles.map(p => {
                    if (p === oldPath) return newPath;
                    if (p.startsWith(oldPath + '/')) return newPath + p.slice(oldPath.length);
                    return p;
                });

                // Update Active File
                let newActive = state.activeFile;
                if (state.activeFile === oldPath) newActive = newPath;
                else if (state.activeFile?.startsWith(oldPath + '/')) {
                    newActive = newPath + state.activeFile.slice(oldPath.length);
                }

                return {
                    files: newFiles,
                    openFiles: newOpenFiles,
                    activeFile: newActive,
                    currentFilename: newActive || "", // Keep legacy synced
                };
            }),

            setActiveFile: (path) => set((state) => ({
                activeFile: path,
                currentFilename: path,
                code: state.files[path] || ""
            })),

            setProjectConfig: (config) => set({ projectConfig: config }),

            updateCompilerOptions: (options) => set((state) => ({
                compilerOptions: { ...state.compilerOptions, ...options }
            })),

            // Workspace Actions
            detectWorkspace: () => set((state) => {
                const rootToml = state.files['five.toml'];
                if (!rootToml || !rootToml.includes('[workspace]')) {
                    return { workspace: null };
                }

                // Parse workspace members from TOML content
                const membersMatch = rootToml.match(/members\s*=\s*\[([\s\S]*?)\]/);
                if (!membersMatch) return { workspace: null };

                const membersStr = membersMatch[1];
                const members = membersStr
                    .split(',')
                    .map(m => m.trim().replace(/['"]/g, ''))
                    .filter(m => m.length > 0);

                // Discover packages from file paths
                const packages: PackageInfo[] = [];
                for (const member of members) {
                    const packageTomlPath = `${member}/five.toml`;
                    const packageToml = state.files[packageTomlPath];

                    if (packageToml) {
                        const nameMatch = packageToml.match(/name\s*=\s*["']([^"']+)["']/);
                        const entryMatch = packageToml.match(/entry_point\s*=\s*["']([^"']+)["']/);

                        // Parse dependencies with link type
                        const deps: { name: string; link: LinkType }[] = [];
                        const depsMatch = packageToml.match(/\[dependencies\]([\s\S]*?)(?=\[|$)/);
                        if (depsMatch) {
                            const depLines = depsMatch[1].split('\n');
                            for (const line of depLines) {
                                const depMatch = line.match(/(\w+[-\w]*)\s*=\s*\{.*link\s*=\s*["'](\w+)["']/);
                                if (depMatch) {
                                    deps.push({
                                        name: depMatch[1],
                                        link: depMatch[2] as LinkType
                                    });
                                }
                            }
                        }

                        packages.push({
                            name: nameMatch?.[1] || member.split('/').pop() || member,
                            path: member,
                            entryPoint: entryMatch?.[1],
                            dependencies: deps
                        });
                    }
                }

                return {
                    workspace: {
                        root: '',
                        members,
                        packages,
                        activePackage: packages[0]?.name || null
                    }
                };
            }),

            setActivePackage: (packageName) => set((state) => {
                if (!state.workspace) return {};
                return {
                    workspace: {
                        ...state.workspace,
                        activePackage: packageName
                    }
                };
            }),

            updatePackageBytecode: (packageName, bytecode) => set((state) => {
                if (!state.workspace) return {};
                const packages = state.workspace.packages.map(p =>
                    p.name === packageName ? { ...p, bytecode } : p
                );
                return {
                    workspace: {
                        ...state.workspace,
                        packages
                    }
                };
            }),

            syncWorkspaceToLsp: async (lspSetDocument) => {
                // This is called from within a set() reducer, so we access state via get()
                const state = useIdeStore.getState();
                try {
                    // Notify LSP of all open files
                    for (const filePath of state.openFiles) {
                        const content = state.files[filePath];
                        if (content !== undefined) {
                            const uri = `file:///workspace/${filePath}`;
                            await lspSetDocument(uri, content);
                        }
                    }
                } catch (error) {
                    console.error('[IDE] Error syncing workspace to LSP:', error);
                }
            },

            resetProject: (initialFiles, initialActive) => set((state) => {
                const files = initialFiles || {
                    'src/main.v': DEFAULT_COUNTER_CODE,
                    'five.toml': DEFAULT_TOML
                };
                // Prefer initialActive, then try src/main.v, then just the first file
                const active = initialActive || (files['src/main.v'] ? 'src/main.v' : Object.keys(files)[0]);

                return {
                    files,
                    activeFile: active,
                    openFiles: [active],
                    currentFilename: active,
                    code: files[active] || "",
                    bytecode: null,
                    abi: null,
                    logs: [],
                    vmState: {
                        stack: [],
                        instructionPointer: 0,
                        computeUnits: 0,
                        memory: null,
                    },
                    // Reset Project Config if we are resetting to defaults, otherwise keep current or parse new (not implemented here)
                    // For now, we assume simple reset.
                    deployments: {},
                    isDeploying: false,
                    isExecuting: false,
                    isOnChainExecuting: false,
                    contractAddress: null,
                    estimatedCost: null,
                };
            }),
        }),
        {
            name: 'five-ide-storage',
            version: 2,
            migrate: (persistedState, version) => {
                const state = persistedState as IdeState;
                if (!state?.files) return state as any;

                const files = filterPersistedFiles(state.files);
                const hasActive = state.activeFile && Boolean(files[state.activeFile]);
                const activeFile = hasActive ? state.activeFile : (files['src/main.v'] ? 'src/main.v' : Object.keys(files)[0] || null);
                const openFiles = (state.openFiles || []).filter((path) => Boolean(files[path]));
                if (activeFile && !openFiles.includes(activeFile)) {
                    openFiles.push(activeFile);
                }

                // Ensure workspace packages have bytecode stripped to match partialize type
                const workspace = state.workspace ? {
                    ...state.workspace,
                    packages: state.workspace.packages.map(p => ({ ...p, bytecode: undefined }))
                } : null;

                return {
                    ...state,
                    files,
                    activeFile,
                    openFiles,
                    code: activeFile ? files[activeFile] ?? state.code : state.code,
                    workspace
                };
            },
            partialize: (state) => ({
                files: filterPersistedFiles(state.files),
                activeFile: state.activeFile,
                code: state.code,
                openFiles: state.openFiles,
                expandedFolders: state.expandedFolders,
                projectConfig: state.projectConfig,
                deployments: state.deployments,
                compilerOptions: state.compilerOptions,
                // Persist workspace structure (but not bytecode data)
                workspace: state.workspace ? {
                    ...state.workspace,
                    packages: state.workspace.packages.map(p => ({ ...p, bytecode: undefined }))
                } : null
            }),
        }
    )
);
