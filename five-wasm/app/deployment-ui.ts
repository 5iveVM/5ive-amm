/**
 * Deployment UI components for Stacks VM WASM.
 * Interacts with real Solana networks only.
 */

import { 
    DeploymentService, 
    DeploymentConfig, 
    DeploymentProgress, 
    DeploymentResult, 
    GasEstimation,
    SolanaNetwork,
    DeploymentHistoryEntry,
    DeploymentUtils
} from './deployment-service';
import { WalletAdapter } from '@solana/wallet-adapter-base';
import { PublicKey } from '@solana/web3.js';

/**
 * Deployment configuration UI state
 */
export interface DeploymentUIState {
    /** Currently selected network */
    selectedNetwork: SolanaNetwork;
    /** Custom RPC URL if using custom endpoint */
    customRpcUrl: string;
    /** Whether to use custom RPC URL */
    useCustomRpc: boolean;
    /** Current deployment progress */
    deploymentProgress: DeploymentProgress | null;
    /** Gas estimation for current deployment */
    gasEstimation: GasEstimation | null;
    /** Deployment history */
    deploymentHistory: DeploymentHistoryEntry[];
    /** Error messages */
    error: string | null;
    /** Loading states */
    loading: {
        estimating: boolean;
        deploying: boolean;
        connecting: boolean;
    };
}

/**
 * Network configuration options
 */
export interface NetworkConfig {
    name: SolanaNetwork;
    displayName: string;
    rpcUrl?: string;
    description: string;
    isMainnet: boolean;
}

/**
 * Deployment form data
 */
export interface DeploymentFormData {
    scriptName: string;
    bytecode: Uint8Array;
    network: SolanaNetwork;
    customRpcUrl?: string;
}

/**
 * Main deployment UI controller
 */
export class DeploymentUI {
    private state: DeploymentUIState;
    private deploymentService: DeploymentService | null = null;
    private eventListeners: Map<string, Function[]> = new Map();

    // Network configurations
    public static readonly NETWORKS: NetworkConfig[] = [
        {
            name: 'localnet',
            displayName: 'Local Network',
            rpcUrl: 'http://localhost:8899',
            description: 'Local Solana test validator',
            isMainnet: false
        },
        {
            name: 'devnet',
            displayName: 'Devnet',
            description: 'Solana development network',
            isMainnet: false
        },
        {
            name: 'testnet',
            displayName: 'Testnet',
            description: 'Solana test network',
            isMainnet: false
        },
        {
            name: 'mainnet-beta',
            displayName: 'Mainnet Beta',
            description: 'Solana mainnet (production)',
            isMainnet: true
        }
    ];

    constructor() {
        this.state = {
            selectedNetwork: 'devnet',
            customRpcUrl: '',
            useCustomRpc: false,
            deploymentProgress: null,
            gasEstimation: null,
            deploymentHistory: [],
            error: null,
            loading: {
                estimating: false,
                deploying: false,
                connecting: false
            }
        };
    }

    /**
     * Initialize deployment UI
     */
    async initialize(): Promise<void> {
        await this.updateDeploymentService();
        await this.loadDeploymentHistory();
    }

    /**
     * Set selected network
     */
    async setNetwork(network: SolanaNetwork, customRpcUrl?: string): Promise<void> {
        this.updateState({
            selectedNetwork: network,
            customRpcUrl: customRpcUrl || '',
            useCustomRpc: !!customRpcUrl,
            error: null
        });

        await this.updateDeploymentService();
        this.emit('networkChanged', { network, customRpcUrl });
    }

    /**
     * Estimate deployment costs
     */
    async estimateDeployment(bytecode: Uint8Array): Promise<GasEstimation> {
        if (!this.deploymentService) {
            throw new Error('Deployment service not initialized');
        }

        this.updateState({
            loading: { ...this.state.loading, estimating: true },
            error: null
        });

        try {
            const estimation = await this.deploymentService.estimateDeploymentCost(bytecode);
            
            this.updateState({
                gasEstimation: estimation,
                loading: { ...this.state.loading, estimating: false }
            });

            this.emit('gasEstimated', estimation);
            return estimation;

        } catch (error) {
            const errorMessage = error instanceof Error ? error.message : String(error);
            this.updateState({
                error: errorMessage,
                loading: { ...this.state.loading, estimating: false }
            });
            throw error;
        }
    }

    /**
     * Deploy script to selected network
     */
    async deployScript(
        formData: DeploymentFormData,
        wallet: WalletAdapter
    ): Promise<DeploymentResult> {
        if (!this.deploymentService) {
            throw new Error('Deployment service not initialized');
        }

        if (!wallet.connected || !wallet.publicKey) {
            throw new Error('Wallet not connected');
        }

        this.updateState({
            loading: { ...this.state.loading, deploying: true },
            deploymentProgress: null,
            error: null
        });

        try {
            const result = await this.deploymentService.deployScript(
                formData.scriptName,
                formData.bytecode,
                wallet,
                (progress) => this.handleDeploymentProgress(progress)
            );

            this.updateState({
                loading: { ...this.state.loading, deploying: false },
                deploymentProgress: null
            });

            if (result.success) {
                await this.loadDeploymentHistory();
                this.emit('deploymentSuccess', result);
            } else {
                this.updateState({ error: result.error || 'Deployment failed' });
                this.emit('deploymentError', result);
            }

            return result;

        } catch (error) {
            const errorMessage = error instanceof Error ? error.message : String(error);
            this.updateState({
                error: errorMessage,
                loading: { ...this.state.loading, deploying: false },
                deploymentProgress: null
            });

            this.emit('deploymentError', { error: errorMessage });
            throw error;
        }
    }

    /**
     * Check network connectivity
     */
    async checkConnectivity(): Promise<boolean> {
        if (!this.deploymentService) {
            return false;
        }

        this.updateState({
            loading: { ...this.state.loading, connecting: true }
        });

        try {
            const connected = await this.deploymentService.checkNetworkConnectivity();
            
            this.updateState({
                loading: { ...this.state.loading, connecting: false },
                error: connected ? null : 'Unable to connect to network'
            });

            this.emit('connectivityChecked', connected);
            return connected;

        } catch (error) {
            this.updateState({
                loading: { ...this.state.loading, connecting: false },
                error: 'Network connectivity check failed'
            });
            return false;
        }
    }

    /**
     * Get current UI state
     */
    getState(): DeploymentUIState {
        return { ...this.state };
    }

    /**
     * Clear deployment history
     */
    async clearHistory(): Promise<void> {
        if (this.deploymentService) {
            this.deploymentService.clearDeploymentHistory();
        }
        
        this.updateState({
            deploymentHistory: []
        });

        this.emit('historyCleared', null);
    }

    /**
     * Get network info
     */
    async getNetworkInfo(): Promise<any> {
        if (!this.deploymentService) {
            throw new Error('Deployment service not initialized');
        }

        return await this.deploymentService.getNetworkInfo();
    }

    /**
     * Get transaction details
     */
    async getTransactionDetails(signature: string): Promise<any> {
        if (!this.deploymentService) {
            throw new Error('Deployment service not initialized');
        }

        return await this.deploymentService.getTransactionDetails(signature);
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

    /**
     * Format utilities for UI display
     */
    static readonly Utils = {
        formatSol: DeploymentUtils.formatSol,
        formatSignature: DeploymentUtils.formatSignature,
        getExplorerUrl: DeploymentUtils.getExplorerUrl,
        getAccountExplorerUrl: DeploymentUtils.getAccountExplorerUrl,
        isValidSolanaAddress: DeploymentUtils.isValidSolanaAddress,

        /**
         * Format deployment progress for display
         */
        formatProgress(progress: DeploymentProgress): string {
            return `${progress.description} (${progress.progress}%)`;
        },

        /**
         * Format deployment result for display
         */
        formatDeploymentResult(result: DeploymentResult): string {
            if (result.success) {
                return `✅ Deployed to ${result.scriptAddress?.toBase58()} (${DeploymentUtils.formatSol(result.cost * 1e9)})`;
            } else {
                return `❌ Deployment failed: ${result.error}`;
            }
        },

        /**
         * Get network display info
         */
        getNetworkDisplay(network: SolanaNetwork): NetworkConfig | undefined {
            return DeploymentUI.NETWORKS.find(n => n.name === network);
        },

        /**
         * Validate deployment form
         */
        validateDeploymentForm(formData: DeploymentFormData): string | null {
            if (!formData.scriptName.trim()) {
                return 'Script name is required';
            }
            
            if (formData.scriptName.length > 50) {
                return 'Script name must be 50 characters or less';
            }

            if (!formData.bytecode || formData.bytecode.length === 0) {
                return 'Bytecode is required';
            }

            if (formData.bytecode.length > 1024 * 1024) { // 1MB limit
                return 'Bytecode too large (max 1MB)';
            }

            return null;
        }
    };

    // Private methods

    private async updateDeploymentService(): Promise<void> {
        const config: DeploymentConfig = {
            network: this.state.selectedNetwork,
            rpcUrl: this.state.useCustomRpc ? this.state.customRpcUrl : undefined,
            commitment: 'confirmed'
        };

        this.deploymentService = new DeploymentService(config);
        await this.deploymentService.initialize();
    }

    private async loadDeploymentHistory(): Promise<void> {
        if (this.deploymentService) {
            const history = this.deploymentService.getDeploymentHistory();
            this.updateState({ deploymentHistory: history });
        }
    }

    private handleDeploymentProgress(progress: DeploymentProgress): void {
        this.updateState({ deploymentProgress: progress });
        this.emit('deploymentProgress', progress);
    }

    private updateState(updates: Partial<DeploymentUIState>): void {
        this.state = { ...this.state, ...updates };
        this.emit('stateChanged', this.state);
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

/**
 * Progress component for displaying deployment status
 */
export class ProgressComponent {
    private element: HTMLElement;
    private progress: DeploymentProgress | null = null;

    constructor(containerId: string) {
        const container = document.getElementById(containerId);
        if (!container) {
            throw new Error(`Container element with ID '${containerId}' not found`);
        }

        this.element = this.createProgressElement();
        container.appendChild(this.element);
    }

    /**
     * Update progress display
     */
    updateProgress(progress: DeploymentProgress | null): void {
        this.progress = progress;
        this.render();
    }

    /**
     * Show error state
     */
    showError(error: string): void {
        this.progress = {
            step: 'failed',
            description: error,
            progress: 0,
            error
        };
        this.render();
    }

    /**
     * Clear progress display
     */
    clear(): void {
        this.progress = null;
        this.render();
    }

    private createProgressElement(): HTMLElement {
        const div = document.createElement('div');
        div.className = 'deployment-progress';
        div.style.cssText = `
            border: 1px solid #ddd;
            border-radius: 8px;
            padding: 16px;
            margin: 16px 0;
            background: #f9f9f9;
            display: none;
        `;
        return div;
    }

    private render(): void {
        if (!this.progress) {
            this.element.style.display = 'none';
            return;
        }

        this.element.style.display = 'block';
        
        const isError = this.progress.step === 'failed';
        const isComplete = this.progress.step === 'completed';
        
        this.element.style.borderColor = isError ? '#f56565' : isComplete ? '#48bb78' : '#4299e1';
        this.element.style.backgroundColor = isError ? '#fed7d7' : isComplete ? '#c6f6d5' : '#ebf8ff';

        this.element.innerHTML = `
            <div style="display: flex; align-items: center; margin-bottom: 8px;">
                <div style="font-weight: bold; color: ${isError ? '#c53030' : isComplete ? '#2f855a' : '#2b6cb0'};">
                    ${this.getStepIcon()} ${this.progress.description}
                </div>
                ${this.progress.signature ? `
                    <a href="${DeploymentUtils.getExplorerUrl(this.progress.signature, 'devnet')}" 
                       target="_blank" 
                       style="margin-left: auto; color: #2b6cb0; text-decoration: none; font-size: 12px;">
                        View Transaction ↗
                    </a>
                ` : ''}
            </div>
            <div style="background: #fff; border-radius: 4px; height: 8px; overflow: hidden;">
                <div style="
                    background: ${isError ? '#f56565' : isComplete ? '#48bb78' : '#4299e1'};
                    height: 100%;
                    width: ${this.progress.progress}%;
                    transition: width 0.3s ease;
                "></div>
            </div>
        `;
    }

    private getStepIcon(): string {
        if (!this.progress) return '';
        
        switch (this.progress.step) {
            case 'completed': return '✅';
            case 'failed': return '❌';
            case 'validating': return '🔍';
            case 'estimating': return '📊';
            case 'creating_account': return '🏗️';
            case 'deploying': return '🚀';
            case 'confirming': return '⏳';
            default: return '⚙️';
        }
    }
}

/**
 * Toast notification component for deployment events
 */
export class ToastComponent {
    private container: HTMLElement;

    constructor() {
        this.container = this.createToastContainer();
        document.body.appendChild(this.container);
    }

    /**
     * Show success toast
     */
    showSuccess(message: string, duration: number = 5000): void {
        this.showToast(message, 'success', duration);
    }

    /**
     * Show error toast
     */
    showError(message: string, duration: number = 8000): void {
        this.showToast(message, 'error', duration);
    }

    /**
     * Show info toast
     */
    showInfo(message: string, duration: number = 5000): void {
        this.showToast(message, 'info', duration);
    }

    private createToastContainer(): HTMLElement {
        const container = document.createElement('div');
        container.style.cssText = `
            position: fixed;
            top: 20px;
            right: 20px;
            z-index: 10000;
            pointer-events: none;
        `;
        return container;
    }

    private showToast(message: string, type: 'success' | 'error' | 'info', duration: number): void {
        const toast = document.createElement('div');
        
        const colors = {
            success: { bg: '#c6f6d5', border: '#48bb78', text: '#2f855a' },
            error: { bg: '#fed7d7', border: '#f56565', text: '#c53030' },
            info: { bg: '#ebf8ff', border: '#4299e1', text: '#2b6cb0' }
        };

        const color = colors[type];
        
        toast.style.cssText = `
            background: ${color.bg};
            border: 1px solid ${color.border};
            color: ${color.text};
            padding: 12px 16px;
            border-radius: 8px;
            margin-bottom: 8px;
            max-width: 300px;
            box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
            pointer-events: auto;
            transform: translateX(100%);
            transition: transform 0.3s ease;
            font-size: 14px;
            font-weight: 500;
        `;

        toast.textContent = message;
        this.container.appendChild(toast);

        // Animate in
        setTimeout(() => {
            toast.style.transform = 'translateX(0)';
        }, 10);

        // Auto-remove
        setTimeout(() => {
            toast.style.transform = 'translateX(100%)';
            setTimeout(() => {
                if (toast.parentNode) {
                    toast.parentNode.removeChild(toast);
                }
            }, 300);
        }, duration);
    }
}

// Default export
export default DeploymentUI;
