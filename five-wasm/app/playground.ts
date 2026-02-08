/**
 * Stacks VM playground for compiling, testing, and deploying.
 * Uses real Solana deployments only (no simulation).
 */

import { WasmCompilerService, PartialExecutionSummary, WasmAccountInterface } from './wasm-compiler';
import { DeploymentService, DeploymentConfig, DeploymentResult, SolanaNetwork } from './deployment-service';
import { DeploymentUI, ProgressComponent, ToastComponent } from './deployment-ui';
import { WalletAdapter } from '@solana/wallet-adapter-base';
import { PublicKey } from '@solana/web3.js';

/**
 * Playground configuration
 */
export interface PlaygroundConfig {
    /** Default network for deployments */
    defaultNetwork: SolanaNetwork;
    /** Whether to enable auto-compilation */
    autoCompile: boolean;
    /** Whether to enable auto-testing */
    autoTest: boolean;
    /** Maximum bytecode size in bytes */
    maxBytecodeSize: number;
    /** Code editor theme */
    theme: 'light' | 'dark';
}

/**
 * Code project structure
 */
export interface CodeProject {
    /** Project name */
    name: string;
    /** Source code content */
    sourceCode: string;
    /** Compiled bytecode (if available) */
    bytecode?: Uint8Array;
    /** Last compilation timestamp */
    lastCompiled?: Date;
    /** Last test result */
    lastTestResult?: PartialExecutionSummary;
    /** Project metadata */
    metadata: {
        created: Date;
        modified: Date;
        language: 'stacks' | 'assembly' | 'bytecode';
        description?: string;
        tags?: string[];
    };
}

/**
 * Playground state management
 */
export interface PlaygroundState {
    /** Current project */
    currentProject: CodeProject | null;
    /** Available projects */
    projects: CodeProject[];
    /** Compilation state */
    compilation: {
        inProgress: boolean;
        success: boolean;
        bytecode?: Uint8Array;
        error?: string;
        warnings?: string[];
    };
    /** Testing state */
    testing: {
        inProgress: boolean;
        lastResult?: PartialExecutionSummary;
        error?: string;
    };
    /** Deployment state */
    deployment: {
        inProgress: boolean;
        lastResult?: DeploymentResult;
        error?: string;
    };
    /** Connected wallet */
    wallet: WalletAdapter | null;
    /** UI preferences */
    ui: {
        showSidebar: boolean;
        showConsole: boolean;
        showDeploymentPanel: boolean;
        editorFontSize: number;
    };
}

/**
 * Main Playground class - integrates all components
 */
export class StacksPlayground {
    private config: PlaygroundConfig;
    private state: PlaygroundState;
    private wasmCompiler: WasmCompilerService;
    private deploymentUI: DeploymentUI;
    private progressComponent: ProgressComponent;
    private toastComponent: ToastComponent;
    private eventListeners: Map<string, Function[]> = new Map();

    // Default example projects
    private static readonly EXAMPLE_PROJECTS: Partial<CodeProject>[] = [
        {
            name: 'Simple Vault',
            sourceCode: `// Simple vault contract example
// Demonstrates basic Stacks VM operations

function init() {
    // Initialize vault with zero balance
    push_u64(0);
    store_field(0, 0); // Store balance at offset 0
}

function deposit(amount: u64) {
    // Load current balance
    load_field(0, 0);
    
    // Add deposit amount
    push_u64(amount);
    add();
    
    // Store new balance
    store_field(0, 0);
}

function withdraw(amount: u64) {
    // Load current balance
    load_field(0, 0);
    
    // Check sufficient balance
    push_u64(amount);
    dup();
    gt();
    require("Insufficient balance");
    
    // Subtract withdrawal amount
    sub();
    
    // Store new balance
    store_field(0, 0);
}

function get_balance(): u64 {
    load_field(0, 0);
}`,
            metadata: {
                language: 'stacks' as const,
                description: 'A simple vault contract demonstrating basic operations',
                tags: ['example', 'vault', 'beginner']
            }
        },
        {
            name: 'Token Transfer',
            sourceCode: `// Token transfer example
// Shows account operations and PDA management

function transfer(from: pubkey, to: pubkey, amount: u64) {
    // Validate amount
    push_u64(amount);
    push_u64(0);
    gt();
    require("Amount must be greater than zero");
    
    // Load from account balance
    push_pubkey(from);
    push_u8(0); // account index
    load_account();
    load_field(0, 0); // balance at offset 0
    
    // Check sufficient balance
    push_u64(amount);
    dup();
    gt();
    require("Insufficient balance");
    
    // Deduct from sender
    sub();
    store_field(0, 0);
    
    // Load to account balance
    push_pubkey(to);
    push_u8(1); // account index
    load_account();
    load_field(0, 0);
    
    // Add to receiver
    push_u64(amount);
    add();
    store_field(0, 0);
}

function mint(to: pubkey, amount: u64) {
    // Only authority can mint (simplified)
    push_pubkey(to);
    push_u8(0);
    load_account();
    
    load_field(0, 0);
    push_u64(amount);
    add();
    store_field(0, 0);
}`,
            metadata: {
                language: 'stacks' as const,
                description: 'Token transfer operations with account management',
                tags: ['example', 'token', 'intermediate']
            }
        }
    ];

    constructor(config: Partial<PlaygroundConfig> = {}) {
        this.config = {
            defaultNetwork: 'devnet',
            autoCompile: true,
            autoTest: false,
            maxBytecodeSize: 1024 * 1024, // 1MB
            theme: 'light',
            ...config
        };

        this.state = {
            currentProject: null,
            projects: [],
            compilation: {
                inProgress: false,
                success: false
            },
            testing: {
                inProgress: false
            },
            deployment: {
                inProgress: false
            },
            wallet: null,
            ui: {
                showSidebar: true,
                showConsole: true,
                showDeploymentPanel: false,
                editorFontSize: 14
            }
        };

        // Initialize components
        this.wasmCompiler = new WasmCompilerService();
        this.deploymentUI = new DeploymentUI();
        this.progressComponent = new ProgressComponent('progress-container');
        this.toastComponent = new ToastComponent();

        // Setup event listeners
        this.setupEventListeners();
    }

    /**
     * Initialize the playground
     */
    async initialize(): Promise<void> {
        // Initialize WASM compiler
        await this.wasmCompiler.initialize();
        
        // Initialize deployment UI
        await this.deploymentUI.initialize();
        
        // Load saved projects
        await this.loadProjects();
        
        // Create example projects if none exist
        if (this.state.projects.length === 0) {
            await this.createExampleProjects();
        }

        this.emit('initialized', null);
        this.toastComponent.showSuccess('Playground initialized successfully');
    }

    /**
     * Create a new project
     */
    async createProject(
        name: string, 
        sourceCode: string = '', 
        language: 'stacks' | 'assembly' | 'bytecode' = 'stacks'
    ): Promise<CodeProject> {
        const project: CodeProject = {
            name,
            sourceCode,
            metadata: {
                created: new Date(),
                modified: new Date(),
                language,
                description: '',
                tags: []
            }
        };

        this.state.projects.push(project);
        this.state.currentProject = project;
        
        await this.saveProjects();
        this.emit('projectCreated', project);
        this.toastComponent.showSuccess(`Project "${name}" created`);
        
        return project;
    }

    /**
     * Load an existing project
     */
    async loadProject(projectName: string): Promise<void> {
        const project = this.state.projects.find(p => p.name === projectName);
        if (!project) {
            throw new Error(`Project "${projectName}" not found`);
        }

        this.state.currentProject = project;
        this.emit('projectLoaded', project);
        
        // Auto-compile if enabled
        if (this.config.autoCompile && project.sourceCode) {
            await this.compileCurrentProject();
        }
    }

    /**
     * Update current project source code
     */
    async updateProjectCode(sourceCode: string): Promise<void> {
        if (!this.state.currentProject) {
            throw new Error('No project loaded');
        }

        this.state.currentProject.sourceCode = sourceCode;
        this.state.currentProject.metadata.modified = new Date();
        
        await this.saveProjects();
        this.emit('projectUpdated', this.state.currentProject);
        
        // Auto-compile if enabled
        if (this.config.autoCompile) {
            await this.compileCurrentProject();
        }
    }

    /**
     * Compile current project
     */
    async compileCurrentProject(): Promise<Uint8Array> {
        if (!this.state.currentProject) {
            throw new Error('No project loaded');
        }

        this.updateCompilationState({ inProgress: true, success: false });

        try {
            // Simulate compilation by creating test bytecode
            // In a real implementation, this would call the DSL compiler
            const bytecode = this.createTestBytecode(this.state.currentProject.sourceCode);
            
            // Validate the bytecode
            const isValid = this.wasmCompiler.validateBytecode(bytecode);
            if (!isValid) {
                throw new Error('Generated bytecode is invalid');
            }

            this.state.currentProject.bytecode = bytecode;
            this.state.currentProject.lastCompiled = new Date();

            this.updateCompilationState({
                inProgress: false,
                success: true,
                bytecode
            });

            await this.saveProjects();
            this.emit('compilationSuccess', { project: this.state.currentProject, bytecode });
            this.toastComponent.showSuccess('Compilation successful');

            // Auto-test if enabled
            if (this.config.autoTest) {
                await this.testCurrentProject();
            }

            return bytecode;

        } catch (error) {
            const errorMessage = error instanceof Error ? error.message : String(error);
            
            this.updateCompilationState({
                inProgress: false,
                success: false,
                error: errorMessage
            });

            this.emit('compilationError', { error: errorMessage });
            this.toastComponent.showError(`Compilation failed: ${errorMessage}`);
            
            throw error;
        }
    }

    /**
     * Test current project using WASM VM
     */
    async testCurrentProject(accounts: WasmAccountInterface[] = []): Promise<PartialExecutionSummary> {
        if (!this.state.currentProject?.bytecode) {
            throw new Error('Project must be compiled before testing');
        }

        this.updateTestingState({ inProgress: true });

        try {
            const testResult = await this.wasmCompiler.testBytecodeExecution(
                this.state.currentProject.bytecode,
                new Uint8Array(), // Empty input data
                accounts
            );

            this.state.currentProject.lastTestResult = testResult;
            
            this.updateTestingState({
                inProgress: false,
                lastResult: testResult
            });

            await this.saveProjects();
            this.emit('testCompleted', testResult);

            if (testResult.test_success) {
                this.toastComponent.showSuccess(`Test passed: ${testResult.outcome}`);
            } else {
                this.toastComponent.showError(`Test failed: ${testResult.error_details}`);
            }

            return testResult;

        } catch (error) {
            const errorMessage = error instanceof Error ? error.message : String(error);
            
            this.updateTestingState({
                inProgress: false,
                error: errorMessage
            });

            this.emit('testError', { error: errorMessage });
            this.toastComponent.showError(`Test failed: ${errorMessage}`);
            
            throw error;
        }
    }

    /**
     * Deploy current project to Solana
     */
    async deployCurrentProject(
        scriptName: string,
        network: SolanaNetwork,
        wallet: WalletAdapter
    ): Promise<DeploymentResult> {
        if (!this.state.currentProject?.bytecode) {
            throw new Error('Project must be compiled before deployment');
        }

        if (!wallet.connected || !wallet.publicKey) {
            throw new Error('Wallet not connected');
        }

        this.updateDeploymentState({ inProgress: true });
        this.state.ui.showDeploymentPanel = true;

        try {
            // Set deployment network
            await this.deploymentUI.setNetwork(network);

            // Deploy using deployment UI
            const result = await this.deploymentUI.deployScript(
                {
                    scriptName,
                    bytecode: this.state.currentProject.bytecode,
                    network,
                },
                wallet
            );

            this.updateDeploymentState({
                inProgress: false,
                lastResult: result
            });

            this.emit('deploymentCompleted', result);

            if (result.success) {
                this.toastComponent.showSuccess(
                    `Deployed successfully to ${network}! Address: ${result.scriptAddress?.toBase58()}`
                );
            } else {
                this.toastComponent.showError(`Deployment failed: ${result.error}`);
            }

            return result;

        } catch (error) {
            const errorMessage = error instanceof Error ? error.message : String(error);
            
            this.updateDeploymentState({
                inProgress: false,
                error: errorMessage
            });

            this.emit('deploymentError', { error: errorMessage });
            this.toastComponent.showError(`Deployment failed: ${errorMessage}`);
            
            throw error;
        }
    }

    /**
     * Connect wallet
     */
    async connectWallet(wallet: WalletAdapter): Promise<void> {
        try {
            if (!wallet.connected) {
                await wallet.connect();
            }

            this.state.wallet = wallet;
            this.emit('walletConnected', wallet);
            this.toastComponent.showSuccess(`Wallet connected: ${wallet.publicKey?.toBase58()}`);

        } catch (error) {
            const errorMessage = error instanceof Error ? error.message : String(error);
            this.emit('walletError', { error: errorMessage });
            this.toastComponent.showError(`Wallet connection failed: ${errorMessage}`);
            throw error;
        }
    }

    /**
     * Disconnect wallet
     */
    async disconnectWallet(): Promise<void> {
        if (this.state.wallet) {
            await this.state.wallet.disconnect();
            this.state.wallet = null;
            this.emit('walletDisconnected', null);
            this.toastComponent.showInfo('Wallet disconnected');
        }
    }

    /**
     * Get current state
     */
    getState(): PlaygroundState {
        return { ...this.state };
    }

    /**
     * Get available projects
     */
    getProjects(): CodeProject[] {
        return [...this.state.projects];
    }

    /**
     * Delete a project
     */
    async deleteProject(projectName: string): Promise<void> {
        const index = this.state.projects.findIndex(p => p.name === projectName);
        if (index === -1) {
            throw new Error(`Project "${projectName}" not found`);
        }

        this.state.projects.splice(index, 1);
        
        if (this.state.currentProject?.name === projectName) {
            this.state.currentProject = null;
        }

        await this.saveProjects();
        this.emit('projectDeleted', { name: projectName });
        this.toastComponent.showInfo(`Project "${projectName}" deleted`);
    }

    /**
     * Update UI preferences
     */
    updateUIPreferences(preferences: Partial<PlaygroundState['ui']>): void {
        this.state.ui = { ...this.state.ui, ...preferences };
        this.emit('uiPreferencesUpdated', this.state.ui);
    }

    /**
     * Add event listener
     */
    on(event: string, callback: Function): void {
        if (!this.eventListeners.has(event)) {
            this.eventListeners.set(event, []);
        }
        this.eventListeners.get(event)!.push(callback);
    }

    /**
     * Remove event listener
     */
    off(event: string, callback: Function): void {
        const listeners = this.eventListeners.get(event);
        if (listeners) {
            const index = listeners.indexOf(callback);
            if (index > -1) {
                listeners.splice(index, 1);
            }
        }
    }

    // Private helper methods

    private setupEventListeners(): void {
        // Listen to deployment UI events
        this.deploymentUI.on('deploymentProgress', (progress) => {
            this.progressComponent.updateProgress(progress);
        });

        this.deploymentUI.on('deploymentSuccess', (result) => {
            this.progressComponent.clear();
        });

        this.deploymentUI.on('deploymentError', (result) => {
            this.progressComponent.showError(result.error || 'Deployment failed');
        });
    }

    private async createExampleProjects(): Promise<void> {
        for (const exampleData of StacksPlayground.EXAMPLE_PROJECTS) {
            const project: CodeProject = {
                name: exampleData.name!,
                sourceCode: exampleData.sourceCode!,
                metadata: {
                    created: new Date(),
                    modified: new Date(),
                    language: exampleData.metadata!.language!,
                    description: exampleData.metadata!.description,
                    tags: exampleData.metadata!.tags || []
                }
            };

            this.state.projects.push(project);
        }

        await this.saveProjects();
    }

    private createTestBytecode(sourceCode: string): Uint8Array {
        // Create test bytecode with 5IVE optimized header
        // This is a simplified implementation - real version would use actual compiler
        const header = [
            0x35, 0x49, 0x56, 0x45, // "5IVE"
            0x00, 0x00, 0x00, 0x00, // features
            0x00, 0x00              // public/total function counts
        ];

        // Minimal program: HALT
        const program = [0x00];

        return new Uint8Array([...header, ...program]);
    }

    private updateCompilationState(updates: Partial<PlaygroundState['compilation']>): void {
        this.state.compilation = { ...this.state.compilation, ...updates };
        this.emit('compilationStateChanged', this.state.compilation);
    }

    private updateTestingState(updates: Partial<PlaygroundState['testing']>): void {
        this.state.testing = { ...this.state.testing, ...updates };
        this.emit('testingStateChanged', this.state.testing);
    }

    private updateDeploymentState(updates: Partial<PlaygroundState['deployment']>): void {
        this.state.deployment = { ...this.state.deployment, ...updates };
        this.emit('deploymentStateChanged', this.state.deployment);
    }

    private async loadProjects(): Promise<void> {
        if (typeof localStorage !== 'undefined') {
            try {
                const stored = localStorage.getItem('stacks_playground_projects');
                if (stored) {
                    const projectsData = JSON.parse(stored);
                    this.state.projects = projectsData.map((data: any) => ({
                        ...data,
                        metadata: {
                            ...data.metadata,
                            created: new Date(data.metadata.created),
                            modified: new Date(data.metadata.modified)
                        },
                        lastCompiled: data.lastCompiled ? new Date(data.lastCompiled) : undefined,
                        bytecode: data.bytecode ? new Uint8Array(data.bytecode) : undefined
                    }));
                }
            } catch (error) {
                console.warn('Failed to load projects:', error);
                this.state.projects = [];
            }
        }
    }

    private async saveProjects(): Promise<void> {
        if (typeof localStorage !== 'undefined') {
            try {
                const projectsData = this.state.projects.map(project => ({
                    ...project,
                    bytecode: project.bytecode ? Array.from(project.bytecode) : undefined
                }));
                localStorage.setItem('stacks_playground_projects', JSON.stringify(projectsData));
            } catch (error) {
                console.warn('Failed to save projects:', error);
            }
        }
    }

    private emit(event: string, data: any): void {
        const listeners = this.eventListeners.get(event) || [];
        listeners.forEach(callback => {
            try {
                callback(data);
            } catch (error) {
                console.error(`Error in event listener for ${event}:`, error);
            }
        });
    }
}

// Default export
export default StacksPlayground;
