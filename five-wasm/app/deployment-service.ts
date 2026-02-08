/**
 * Deployment service for Stacks VM WASM.
 * Performs real Solana deployments only (no simulation).
 */

import { 
    Connection, 
    Keypair, 
    PublicKey, 
    SystemProgram, 
    Transaction, 
    TransactionInstruction, 
    sendAndConfirmTransaction,
    clusterApiUrl,
    LAMPORTS_PER_SOL,
    ConfirmOptions,
    RpcResponseAndContext,
    SignatureResult
} from '@solana/web3.js';
import { WalletAdapter } from '@solana/wallet-adapter-base';
import * as bs58 from 'bs58';
import { WasmCompilerService } from './wasm-compiler';

/**
 * Supported Solana networks for deployment
 */
export type SolanaNetwork = 'devnet' | 'testnet' | 'mainnet-beta' | 'localnet';

/**
 * Configuration for deployment service
 */
export interface DeploymentConfig {
    /** Target Solana network */
    network: SolanaNetwork;
    /** Custom RPC URL (optional, defaults to public endpoints) */
    rpcUrl?: string;
    /** Commitment level for confirmations */
    commitment?: 'processed' | 'confirmed' | 'finalized';
    /** Transaction confirmation timeout in milliseconds */
    confirmationTimeout?: number;
}

/**
 * Gas estimation for deployment
 */
export interface GasEstimation {
    /** Estimated compute units for deployment */
    computeUnits: number;
    /** Estimated SOL cost for deployment */
    estimatedCost: number;
    /** Rent-exempt minimum balance for script account */
    rentExemptBalance: number;
    /** Transaction fee estimate */
    transactionFee: number;
    /** Total estimated cost in SOL */
    totalCost: number;
}

/**
 * Deployment progress tracking
 */
export interface DeploymentProgress {
    /** Current step being executed */
    step: 'validating' | 'estimating' | 'creating_account' | 'deploying' | 'confirming' | 'completed' | 'failed';
    /** Human-readable description of current step */
    description: string;
    /** Progress percentage (0-100) */
    progress: number;
    /** Transaction signature (if available) */
    signature?: string;
    /** Error message (if failed) */
    error?: string;
}

/**
 * Deployment result
 */
export interface DeploymentResult {
    /** Whether deployment was successful */
    success: boolean;
    /** Deployed script address */
    scriptAddress?: PublicKey;
    /** Transaction signature */
    signature?: string;
    /** Deployment timestamp */
    deployedAt: Date;
    /** Network deployed to */
    network: SolanaNetwork;
    /** Bytecode size */
    bytecodeSize: number;
    /** Gas used */
    gasUsed: number;
    /** SOL cost */
    cost: number;
    /** Error message if failed */
    error?: string;
}

/**
 * Deployment history entry
 */
export interface DeploymentHistoryEntry {
    /** Unique deployment ID */
    id: string;
    /** Script name */
    name: string;
    /** Deployment result */
    result: DeploymentResult;
    /** Original bytecode hash for verification */
    bytecodeHash: string;
    /** Deployer wallet address */
    deployer: string;
}

/**
 * Real Solana deployment service
 */
export class DeploymentService {
    private connection: Connection;
    private config: DeploymentConfig;
    private wasmCompiler: WasmCompilerService;
    private deploymentHistory: DeploymentHistoryEntry[] = [];

    // Default Stacks program IDs by network
    private static readonly PROGRAM_IDS = {
        devnet: new PublicKey('4jFu6SzdXpaz2TJAHrhiBAUp22yX6URtwenD9XLst7uy'),
        testnet: new PublicKey('4jFu6SzdXpaz2TJAHrhiBAUp22yX6URtwenD9XLst7uy'),
        'mainnet-beta': new PublicKey('6h58jYGVnbTGA6dmeXrE5TqQUrSi21KN6ipBrvUAeS8p'),
        localnet: new PublicKey('4jFu6SzdXpaz2TJAHrhiBAUp22yX6URtwenD9XLst7uy')
    };

    constructor(config: DeploymentConfig) {
        this.config = {
            commitment: 'confirmed',
            confirmationTimeout: 60000,
            ...config
        };

        // Initialize Solana connection
        const rpcUrl = this.config.rpcUrl || this.getDefaultRpcUrl(config.network);
        this.connection = new Connection(rpcUrl, {
            commitment: this.config.commitment,
            confirmTransactionInitialTimeout: this.config.confirmationTimeout
        });

        // Initialize WASM compiler
        this.wasmCompiler = new WasmCompilerService();
        
        // Load deployment history from localStorage if available
        this.loadDeploymentHistory();
    }

    /**
     * Initialize the deployment service
     */
    async initialize(): Promise<void> {
        await this.wasmCompiler.initialize();
    }

    /**
     * Estimate gas and costs for deployment
     */
    async estimateDeploymentCost(bytecode: Uint8Array): Promise<GasEstimation> {
        // Validate bytecode first
        if (!this.wasmCompiler.validateBytecode(bytecode)) {
            throw new Error('Invalid bytecode format');
        }

        // Calculate space needed for script account
        const scriptDataSize = 48 + bytecode.length; // Header + bytecode
        
        // Get rent-exempt balance
        const rentExemptBalance = await this.connection.getMinimumBalanceForRentExemption(scriptDataSize);
        
        // Estimate transaction fee (typical deployment uses ~10,000 compute units)
        const estimatedComputeUnits = 10000 + Math.floor(bytecode.length / 100);
        const recentBlockhash = await this.connection.getLatestBlockhash();
        const transactionFee = 5000; // 0.000005 SOL typical fee
        
        // Calculate total cost
        const totalCost = (rentExemptBalance + transactionFee) / LAMPORTS_PER_SOL;

        return {
            computeUnits: estimatedComputeUnits,
            estimatedCost: totalCost,
            rentExemptBalance: rentExemptBalance / LAMPORTS_PER_SOL,
            transactionFee: transactionFee / LAMPORTS_PER_SOL,
            totalCost
        };
    }

    /**
     * Deploy bytecode to Solana network
     */
    async deployScript(
        name: string,
        bytecode: Uint8Array,
        wallet: WalletAdapter,
        onProgress?: (progress: DeploymentProgress) => void
    ): Promise<DeploymentResult> {
        const startTime = new Date();
        
        try {
            // Step 1: Validate inputs
            this.reportProgress(onProgress, {
                step: 'validating',
                description: 'Validating bytecode and wallet connection',
                progress: 10
            });

            if (!wallet.connected || !wallet.publicKey) {
                throw new Error('Wallet not connected');
            }

            if (!this.wasmCompiler.validateBytecode(bytecode)) {
                throw new Error('Invalid bytecode format');
            }

            // Step 2: Estimate costs
            this.reportProgress(onProgress, {
                step: 'estimating',
                description: 'Estimating deployment costs',
                progress: 20
            });

            const gasEstimation = await this.estimateDeploymentCost(bytecode);

            // Check wallet balance
            const walletBalance = await this.connection.getBalance(wallet.publicKey);
            if (walletBalance < gasEstimation.totalCost * LAMPORTS_PER_SOL) {
                throw new Error(`Insufficient balance. Need ${gasEstimation.totalCost} SOL, have ${walletBalance / LAMPORTS_PER_SOL} SOL`);
            }

            // Step 3: Create script account
            this.reportProgress(onProgress, {
                step: 'creating_account',
                description: 'Creating script account',
                progress: 40
            });

            const scriptKeypair = Keypair.generate();
            const programId = DeploymentService.PROGRAM_IDS[this.config.network];
            
            // Get or create registry PDA
            const [registryPda] = PublicKey.findProgramAddressSync(
                [Buffer.from("registry")],
                programId
            );

            // Build deployment transaction
            const scriptDataSize = 48 + bytecode.length;
            const rentExemptBalance = await this.connection.getMinimumBalanceForRentExemption(scriptDataSize);

            const createAccountIx = SystemProgram.createAccount({
                fromPubkey: wallet.publicKey,
                newAccountPubkey: scriptKeypair.publicKey,
                lamports: rentExemptBalance,
                space: scriptDataSize,
                programId
            });

            // Step 4: Deploy bytecode
            this.reportProgress(onProgress, {
                step: 'deploying',
                description: 'Deploying script to blockchain',
                progress: 60
            });

            // Build deploy instruction data
            const deployData = Buffer.alloc(5 + bytecode.length);
            deployData[0] = 1; // Deploy instruction
            deployData.writeUInt32LE(bytecode.length, 1);
            Buffer.from(bytecode).copy(deployData, 5);

            const deployIx = new TransactionInstruction({
                keys: [
                    { pubkey: scriptKeypair.publicKey, isSigner: false, isWritable: true },
                    { pubkey: registryPda, isSigner: false, isWritable: true },
                    { pubkey: wallet.publicKey, isSigner: true, isWritable: false },
                ],
                programId,
                data: deployData
            });

            // Create and send transaction
            const transaction = new Transaction()
                .add(createAccountIx)
                .add(deployIx);

            const { blockhash } = await this.connection.getLatestBlockhash();
            transaction.recentBlockhash = blockhash;
            transaction.feePayer = wallet.publicKey;

            // Sign with both wallet and script keypair
            const signedTx = await wallet.signTransaction!(transaction);
            signedTx.partialSign(scriptKeypair);

            // Step 5: Send and confirm transaction
            this.reportProgress(onProgress, {
                step: 'confirming',
                description: 'Confirming transaction on blockchain',
                progress: 80
            });

            const signature = await this.connection.sendRawTransaction(signedTx.serialize(), {
                skipPreflight: false,
                preflightCommitment: this.config.commitment
            });

            // Wait for confirmation
            const confirmation = await this.connection.confirmTransaction({
                signature,
                blockhash,
                lastValidBlockHeight: (await this.connection.getLatestBlockhash()).lastValidBlockHeight
            }, this.config.commitment);

            if (confirmation.value.err) {
                throw new Error(`Transaction failed: ${JSON.stringify(confirmation.value.err)}`);
            }

            // Step 6: Deployment completed
            this.reportProgress(onProgress, {
                step: 'completed',
                description: 'Deployment completed successfully',
                progress: 100,
                signature
            });

            const result: DeploymentResult = {
                success: true,
                scriptAddress: scriptKeypair.publicKey,
                signature,
                deployedAt: startTime,
                network: this.config.network,
                bytecodeSize: bytecode.length,
                gasUsed: gasEstimation.computeUnits,
                cost: gasEstimation.totalCost
            };

            // Save to deployment history
            this.saveDeployment(name, result, bytecode, wallet.publicKey.toBase58());

            return result;

        } catch (error) {
            const errorMessage = error instanceof Error ? error.message : String(error);
            
            this.reportProgress(onProgress, {
                step: 'failed',
                description: `Deployment failed: ${errorMessage}`,
                progress: 0,
                error: errorMessage
            });

            return {
                success: false,
                deployedAt: startTime,
                network: this.config.network,
                bytecodeSize: bytecode.length,
                gasUsed: 0,
                cost: 0,
                error: errorMessage
            };
        }
    }

    /**
     * Re-deploy a script from deployment history
     */
    async redeployScript(
        deploymentId: string,
        wallet: WalletAdapter,
        onProgress?: (progress: DeploymentProgress) => void
    ): Promise<DeploymentResult> {
        const historyEntry = this.deploymentHistory.find(entry => entry.id === deploymentId);
        if (!historyEntry) {
            throw new Error(`Deployment with ID ${deploymentId} not found in history`);
        }

        // This would require storing the original bytecode or being able to regenerate it
        // Throw error; requires original bytecode
        throw new Error('Re-deployment requires the original bytecode. Please compile and deploy again.');
    }

    /**
     * Get deployment history
     */
    getDeploymentHistory(): DeploymentHistoryEntry[] {
        return [...this.deploymentHistory];
    }

    /**
     * Clear deployment history
     */
    clearDeploymentHistory(): void {
        this.deploymentHistory = [];
        this.saveDeploymentHistory();
    }

    /**
     * Check network connectivity
     */
    async checkNetworkConnectivity(): Promise<boolean> {
        try {
            await this.connection.getLatestBlockhash();
            return true;
        } catch (error) {
            return false;
        }
    }

    /**
     * Get current network info
     */
    async getNetworkInfo(): Promise<{
        network: SolanaNetwork;
        blockHeight: number;
        programId: PublicKey;
        connected: boolean;
    }> {
        try {
            const { blockHeight } = await this.connection.getLatestBlockhash();
            return {
                network: this.config.network,
                blockHeight,
                programId: DeploymentService.PROGRAM_IDS[this.config.network],
                connected: true
            };
        } catch (error) {
            return {
                network: this.config.network,
                blockHeight: 0,
                programId: DeploymentService.PROGRAM_IDS[this.config.network],
                connected: false
            };
        }
    }

    /**
     * Get transaction details
     */
    async getTransactionDetails(signature: string): Promise<any> {
        try {
            return await this.connection.getTransaction(signature, {
                commitment: 'confirmed'
            });
        } catch (error) {
            throw new Error(`Failed to fetch transaction details: ${error}`);
        }
    }

    // Private helper methods

    private getDefaultRpcUrl(network: SolanaNetwork): string {
        switch (network) {
            case 'devnet':
                return clusterApiUrl('devnet');
            case 'testnet':
                return clusterApiUrl('testnet');
            case 'mainnet-beta':
                return clusterApiUrl('mainnet-beta');
            case 'localnet':
                return 'http://localhost:8899';
            default:
                throw new Error(`Unknown network: ${network}`);
        }
    }

    private reportProgress(
        onProgress: ((progress: DeploymentProgress) => void) | undefined,
        progress: DeploymentProgress
    ): void {
        if (onProgress) {
            onProgress(progress);
        }
    }

    private saveDeployment(
        name: string,
        result: DeploymentResult,
        bytecode: Uint8Array,
        deployer: string
    ): void {
        const entry: DeploymentHistoryEntry = {
            id: this.generateDeploymentId(),
            name,
            result,
            bytecodeHash: this.hashBytecode(bytecode),
            deployer
        };

        this.deploymentHistory.push(entry);
        this.saveDeploymentHistory();
    }

    private generateDeploymentId(): string {
        return `deploy_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
    }

    private hashBytecode(bytecode: Uint8Array): string {
        // Simple hash of bytecode for verification
        let hash = 0;
        for (let i = 0; i < bytecode.length; i++) {
            hash = ((hash << 5) - hash + bytecode[i]) & 0xffffffff;
        }
        return hash.toString(16);
    }

    private loadDeploymentHistory(): void {
        if (typeof localStorage !== 'undefined') {
            try {
                const stored = localStorage.getItem('stacks_deployment_history');
                if (stored) {
                    this.deploymentHistory = JSON.parse(stored);
                }
            } catch (error) {
                console.warn('Failed to load deployment history:', error);
                this.deploymentHistory = [];
            }
        }
    }

    private saveDeploymentHistory(): void {
        if (typeof localStorage !== 'undefined') {
            try {
                localStorage.setItem('stacks_deployment_history', JSON.stringify(this.deploymentHistory));
            } catch (error) {
                console.warn('Failed to save deployment history:', error);
            }
        }
    }
}

/**
 * Deployment utilities and helpers
 */
export class DeploymentUtils {
    /**
     * Format SOL amount for display
     */
    static formatSol(lamports: number): string {
        return (lamports / LAMPORTS_PER_SOL).toFixed(6) + ' SOL';
    }

    /**
     * Format transaction signature for display
     */
    static formatSignature(signature: string): string {
        return `${signature.substring(0, 8)}...${signature.substring(signature.length - 8)}`;
    }

    /**
     * Get explorer URL for transaction
     */
    static getExplorerUrl(signature: string, network: SolanaNetwork): string {
        const baseUrl = network === 'mainnet-beta' 
            ? 'https://explorer.solana.com'
            : `https://explorer.solana.com?cluster=${network}`;
        return `${baseUrl}/tx/${signature}`;
    }

    /**
     * Get explorer URL for account
     */
    static getAccountExplorerUrl(address: string, network: SolanaNetwork): string {
        const baseUrl = network === 'mainnet-beta' 
            ? 'https://explorer.solana.com'
            : `https://explorer.solana.com?cluster=${network}`;
        return `${baseUrl}/account/${address}`;
    }

    /**
     * Validate Solana address
     */
    static isValidSolanaAddress(address: string): boolean {
        try {
            new PublicKey(address);
            return true;
        } catch {
            return false;
        }
    }
}

// Default export
export default DeploymentService;
