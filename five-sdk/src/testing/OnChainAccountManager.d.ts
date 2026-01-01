/**
 * On-Chain Account Manager for Five VM Account-System Testing
 *
 * Creates and manages real Solana accounts on-chain for comprehensive account-system testing.
 * Handles account creation, funding, initialization, and cleanup.
 */
import { Connection, Keypair, PublicKey } from '@solana/web3.js';
export interface OnChainAccountManagerOptions {
    debug?: boolean;
    cleanup?: boolean;
    maxRetries?: number;
    retryDelay?: number;
}
/**
 * Manages creation and lifecycle of real Solana accounts for testing
 */
export declare class OnChainAccountManager {
    private connection;
    private payer;
    private options;
    private createdAccounts;
    private signers;
    constructor(connection: Connection, payer: Keypair, options?: OnChainAccountManagerOptions);
    /**
     * Create a signer account with generated keypair
     * Transfers SOL from payer to fund the account
     */
    createSignerAccount(lamports?: number): Promise<{
        publicKey: PublicKey;
        keypair: Keypair;
    }>;
    /**
     * Create a regular account with specified space and owner
     * Uses SystemProgram.createAccount for account creation
     */
    createAccount(space: number, owner: PublicKey, lamports?: number): Promise<PublicKey>;
    /**
     * Create and initialize a state account with initial data
     */
    createStateAccount(space: number, owner: PublicKey, initialData?: Uint8Array, lamports?: number): Promise<PublicKey>;
    /**
     * Create a PDA account at a specific seed path
     */
    createPDAAccount(seeds: Buffer[], programId: PublicKey, space: number, owner?: PublicKey, lamports?: number): Promise<{
        publicKey: PublicKey;
        bump: number;
    }>;
    /**
     * Write data to an account (requires account to be writable)
     */
    writeAccountData(accountAddress: PublicKey, data: Uint8Array): Promise<void>;
    /**
     * Check if an account exists
     */
    checkAccountExists(publicKey: PublicKey): Promise<boolean>;
    /**
     * Ensure payer has sufficient balance
     */
    ensureSufficientBalance(required: number): Promise<void>;
    /**
     * Create account with retry logic
     */
    createAccountWithRetry(createFn: () => Promise<PublicKey>, options?: {
        maxRetries?: number;
        retryDelay?: number;
    }): Promise<PublicKey>;
    /**
     * Get a signer keypair if it was created by this manager
     */
    getSignerKeypair(publicKey: PublicKey | string): Keypair | undefined;
    /**
     * Get all created accounts
     */
    getCreatedAccounts(): PublicKey[];
    /**
     * Cleanup: Close all created accounts and transfer remaining SOL back to payer
     * This helps with test isolation and prevents account accumulation on testnet
     */
    cleanup(): Promise<void>;
}
export default OnChainAccountManager;
//# sourceMappingURL=OnChainAccountManager.d.ts.map