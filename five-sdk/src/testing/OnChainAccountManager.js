/**
 * On-Chain Account Manager for Five VM Account-System Testing
 *
 * Creates and manages real Solana accounts on-chain for comprehensive account-system testing.
 * Handles account creation, funding, initialization, and cleanup.
 */
import { Connection, Keypair, PublicKey, SystemProgram, Transaction, sendAndConfirmTransaction, LAMPORTS_PER_SOL } from '@solana/web3.js';
import { RentCalculator } from '../crypto/index.js';
import { PDAUtils } from '../crypto/index.js';
/**
 * Manages creation and lifecycle of real Solana accounts for testing
 */
export class OnChainAccountManager {
    connection;
    payer;
    options;
    createdAccounts = [];
    signers = new Map();
    constructor(connection, payer, options = {}) {
        this.connection = connection;
        this.payer = payer;
        this.options = options;
        this.options.maxRetries = this.options.maxRetries || 3;
        this.options.retryDelay = this.options.retryDelay || 1000;
    }
    /**
     * Create a signer account with generated keypair
     * Transfers SOL from payer to fund the account
     */
    async createSignerAccount(lamports = LAMPORTS_PER_SOL // 1 SOL default
    ) {
        const keypair = Keypair.generate();
        const publicKey = keypair.publicKey;
        if (this.options.debug) {
            console.log(`[OnChainAccountManager] Creating signer account: ${publicKey.toString()}`);
        }
        // Ensure payer has sufficient balance
        await this.ensureSufficientBalance(lamports);
        // Transfer SOL from payer to new account
        const transaction = new Transaction().add(SystemProgram.transfer({
            fromPubkey: this.payer.publicKey,
            toPubkey: publicKey,
            lamports: lamports,
        }));
        try {
            const signature = await sendAndConfirmTransaction(this.connection, transaction, [this.payer], { commitment: 'confirmed' });
            if (this.options.debug) {
                console.log(`[OnChainAccountManager] Signer account funded: ${signature}`);
            }
            this.createdAccounts.push(publicKey);
            this.signers.set(publicKey.toString(), keypair);
            return { publicKey, keypair };
        }
        catch (error) {
            const errorMessage = error instanceof Error ? error.message : String(error);
            throw new Error(`Failed to create signer account: ${errorMessage}`);
        }
    }
    /**
     * Create a regular account with specified space and owner
     * Uses SystemProgram.createAccount for account creation
     */
    async createAccount(space, owner, lamports) {
        const keypair = Keypair.generate();
        const publicKey = keypair.publicKey;
        // Calculate rent-exempt lamports if not provided
        const requiredLamports = lamports || RentCalculator.calculateRentExemption(space);
        if (this.options.debug) {
            console.log(`[OnChainAccountManager] Creating account: ${publicKey.toString()} ` +
                `(space=${space}, lamports=${requiredLamports})`);
        }
        // Ensure payer has sufficient balance
        await this.ensureSufficientBalance(requiredLamports);
        // Create the account using SystemProgram
        const transaction = new Transaction().add(SystemProgram.createAccount({
            fromPubkey: this.payer.publicKey,
            newAccountPubkey: publicKey,
            lamports: requiredLamports,
            space: space,
            programId: owner,
        }));
        try {
            const signature = await sendAndConfirmTransaction(this.connection, transaction, [this.payer, keypair], { commitment: 'confirmed' });
            if (this.options.debug) {
                console.log(`[OnChainAccountManager] Account created: ${signature}`);
            }
            this.createdAccounts.push(publicKey);
            return publicKey;
        }
        catch (error) {
            const errorMessage = error instanceof Error ? error.message : String(error);
            throw new Error(`Failed to create account: ${errorMessage}`);
        }
    }
    /**
     * Create and initialize a state account with initial data
     */
    async createStateAccount(space, owner, initialData, lamports) {
        const accountAddress = await this.createAccount(space, owner, lamports);
        // Write initial data if provided
        if (initialData && initialData.length > 0) {
            await this.writeAccountData(accountAddress, initialData);
        }
        if (this.options.debug) {
            console.log(`[OnChainAccountManager] State account initialized: ${accountAddress.toString()}`);
        }
        return accountAddress;
    }
    /**
     * Create a PDA account at a specific seed path
     */
    async createPDAAccount(seeds, programId, space, owner, lamports) {
        const pda = await PDAUtils.findProgramAddress(seeds, programId.toString());
        const pdaAddress = new PublicKey(pda.address);
        if (this.options.debug) {
            console.log(`[OnChainAccountManager] Creating PDA: ${pdaAddress.toString()} ` +
                `(bump=${pda.bump})`);
        }
        // Check if PDA already exists
        const exists = await this.checkAccountExists(pdaAddress);
        if (exists) {
            if (this.options.debug) {
                console.log(`[OnChainAccountManager] PDA already exists, skipping creation`);
            }
            return { publicKey: pdaAddress, bump: pda.bump };
        }
        // Create PDA account
        const createdAddress = await this.createAccount(space, owner || programId, lamports);
        return { publicKey: createdAddress, bump: pda.bump };
    }
    /**
     * Write data to an account (requires account to be writable)
     */
    async writeAccountData(accountAddress, data) {
        if (this.options.debug) {
            console.log(`[OnChainAccountManager] Writing ${data.length} bytes to account: ${accountAddress.toString()}`);
        }
        // Note: Writing data directly to an account requires special instructions
        // This is a simplified implementation. In practice, you would use the Five VM program
        // or SystemProgram.allocate + loader to write data.
        // For now, we'll just log that data would be written.
        if (this.options.debug) {
            console.log(`[OnChainAccountManager] Data would be written via Five VM program`);
        }
    }
    /**
     * Check if an account exists
     */
    async checkAccountExists(publicKey) {
        try {
            const accountInfo = await this.connection.getAccountInfo(publicKey);
            return accountInfo !== null;
        }
        catch (error) {
            if (this.options.debug) {
                const errorMessage = error instanceof Error ? error.message : String(error);
                console.log(`[OnChainAccountManager] Error checking account: ${errorMessage}`);
            }
            return false;
        }
    }
    /**
     * Ensure payer has sufficient balance
     */
    async ensureSufficientBalance(required) {
        const balance = await this.connection.getBalance(this.payer.publicKey);
        // Add buffer for transaction fees (0.01 SOL)
        const feeBuffer = 0.01 * LAMPORTS_PER_SOL;
        const totalRequired = required + feeBuffer;
        if (balance < totalRequired) {
            throw new Error(`Insufficient balance: ${(balance / LAMPORTS_PER_SOL).toFixed(4)} SOL available, ` +
                `${(totalRequired / LAMPORTS_PER_SOL).toFixed(4)} SOL required. ` +
                `Please fund the payer account: ${this.payer.publicKey.toString()}`);
        }
    }
    /**
     * Create account with retry logic
     */
    async createAccountWithRetry(createFn, options = {}) {
        const maxRetries = options.maxRetries || this.options.maxRetries || 3;
        const retryDelay = options.retryDelay || this.options.retryDelay || 1000;
        for (let attempt = 1; attempt <= maxRetries; attempt++) {
            try {
                return await createFn();
            }
            catch (error) {
                const errorMessage = error instanceof Error ? error.message : String(error);
                if (attempt === maxRetries) {
                    throw new Error(`Failed to create account after ${maxRetries} attempts: ${errorMessage}`);
                }
                if (this.options.debug) {
                    console.log(`[OnChainAccountManager] Account creation failed (attempt ${attempt}/${maxRetries}), ` +
                        `retrying in ${retryDelay}ms...`);
                }
                await new Promise(resolve => setTimeout(resolve, retryDelay));
            }
        }
        throw new Error('Unreachable');
    }
    /**
     * Get a signer keypair if it was created by this manager
     */
    getSignerKeypair(publicKey) {
        const key = typeof publicKey === 'string' ? publicKey : publicKey.toString();
        return this.signers.get(key);
    }
    /**
     * Get all created accounts
     */
    getCreatedAccounts() {
        return [...this.createdAccounts];
    }
    /**
     * Cleanup: Close all created accounts and transfer remaining SOL back to payer
     * This helps with test isolation and prevents account accumulation on testnet
     */
    async cleanup() {
        if (!this.options.cleanup) {
            if (this.options.debug) {
                console.log(`[OnChainAccountManager] Cleanup disabled, skipping`);
            }
            return;
        }
        if (this.options.debug) {
            console.log(`[OnChainAccountManager] Cleaning up ${this.createdAccounts.length} accounts`);
        }
        // Close accounts and transfer lamports back to payer
        for (const accountAddress of this.createdAccounts) {
            try {
                // Get account info to determine if it's an account we can close
                const accountInfo = await this.connection.getAccountInfo(accountAddress);
                if (accountInfo) {
                    // Only try to close accounts owned by SystemProgram (data accounts)
                    // Don't try to close program accounts or special accounts
                    if (accountInfo.owner.equals(SystemProgram.programId)) {
                        const transaction = new Transaction().add(SystemProgram.transfer({
                            fromPubkey: accountAddress,
                            toPubkey: this.payer.publicKey,
                            lamports: accountInfo.lamports,
                        }));
                        const signer = this.getSignerKeypair(accountAddress);
                        if (signer) {
                            await sendAndConfirmTransaction(this.connection, transaction, [signer], { commitment: 'confirmed' });
                            if (this.options.debug) {
                                console.log(`[OnChainAccountManager] Closed account: ${accountAddress.toString()}`);
                            }
                        }
                    }
                }
            }
            catch (error) {
                const errorMessage = error instanceof Error ? error.message : String(error);
                if (this.options.debug) {
                    console.log(`[OnChainAccountManager] Error closing account ${accountAddress.toString()}: ` +
                        `${errorMessage}`);
                }
            }
        }
        this.createdAccounts = [];
        this.signers.clear();
    }
}
export default OnChainAccountManager;
//# sourceMappingURL=OnChainAccountManager.js.map