/**
 * Five SDK Account System
 *
 * Client-agnostic account management system with validation, PDA derivation,
 * and account size calculations. Uses serialization instead of direct blockchain calls.
 */
/**
 * Account data interface (replaces Web3.js AccountInfo)
 */
export interface AccountData {
    /** Account address */
    address: string;
    /** Account data */
    data: Uint8Array;
    /** Account owner program ID */
    owner: string;
    /** Account balance in lamports */
    lamports: number;
}
/**
 * Account types in the Five VM ecosystem
 */
export type AccountType = 'script' | 'metadata' | 'user_state' | 'system' | 'rent_sysvar' | 'clock_sysvar' | 'spl_token' | 'custom';
/**
 * AccountType enum for test compatibility
 */
export declare const AccountType: {
    readonly SCRIPT: "script";
    readonly METADATA: "metadata";
    readonly USER_STATE: "user_state";
    readonly SYSTEM: "system";
    readonly RENT_SYSVAR: "rent_sysvar";
    readonly CLOCK_SYSVAR: "clock_sysvar";
    readonly SPL_TOKEN: "spl_token";
    readonly CUSTOM: "custom";
};
/**
 * Account information with Five VM context
 */
export interface FiveAccount {
    /** Account address */
    address: string;
    /** Account type */
    type: AccountType;
    /** Whether account is signer */
    isSigner: boolean;
    /** Whether account is writable */
    isWritable: boolean;
    /** Whether account is required for execution */
    required?: boolean;
    /** Account owner program ID */
    owner?: string;
    /** Account data size */
    size?: number;
    /** Account lamports balance */
    lamports?: number;
    /** Account data */
    data?: Uint8Array;
    /** PDA derivation info (if applicable) */
    pda?: {
        seeds: Uint8Array[];
        bump: number;
    };
}
/**
 * Account constraints for script execution
 */
export interface AccountConstraints {
    /** Maximum number of accounts allowed */
    maxAccounts?: number;
    /** Maximum total size across all accounts */
    maxTotalSize?: number;
    /** Maximum rent cost allowed */
    maxRentCost?: number;
    /** Required account types that must be present */
    requiredTypes?: AccountType[];
    /** Required signers */
    signers?: string[];
    /** Required writable accounts */
    writableAccounts?: string[];
    /** Required readonly accounts */
    readonlyAccounts?: string[];
    /** Account type constraints */
    typeConstraints?: Map<string, AccountType>;
    /** Minimum rent exemption requirements */
    rentRequirements?: Map<string, number>;
}
/**
 * Account validation result
 */
export interface AccountValidationResult {
    /** Whether validation passed */
    valid: boolean;
    /** Validation errors */
    errors: string[];
    /** Validation warnings */
    warnings: string[];
    /** Estimated costs */
    costs?: {
        rentExemption: number;
        transactionFee: number;
        totalCost: number;
    };
}
/**
 * Account creation parameters
 */
export interface CreateAccountParams {
    /** Account size in bytes */
    size: number;
    /** Owner program ID */
    owner: string;
    /** Whether to make rent-exempt */
    rentExempt: boolean;
    /** Additional lamports beyond rent exemption */
    additionalLamports?: number;
}
/**
 * Solana transaction instruction interface
 */
export interface TransactionInstruction {
    /** Program ID to invoke */
    programId: string;
    /** Account keys and metadata */
    accounts: Array<{
        pubkey: string;
        isSigner: boolean;
        isWritable: boolean;
    }>;
    /** Instruction data */
    data: Uint8Array;
}
/**
 * Account manager for Five VM scripts (serialization-based)
 */
export declare class FiveAccountManager {
    private programId;
    constructor(programId?: string);
    /**
     * Encode System Program CreateAccount instruction
     */
    private encodeCreateAccountInstruction;
    /**
     * Create script account PDA and return serialized instruction
     */
    createScriptAccount(bytecode: Uint8Array, payerAddress: string): Promise<{
        address: string;
        bump: number;
        createInstruction: TransactionInstruction;
        rentLamports: number;
    }>;
    /**
     * Create metadata account for script
     */
    createMetadataAccount(scriptAccount: string, payerAddress: string): Promise<{
        address: string;
        bump: number;
        createInstruction: TransactionInstruction;
        rentLamports: number;
    }>;
    /**
     * Create user state account for script interaction
     */
    createUserStateAccount(userPublicKey: string, scriptAccount: string): Promise<{
        address: string;
        bump: number;
        createInstruction: any;
        rentLamports: number;
    }>;
    /**
     * Validate account constraints for script execution
     */
    validateAccountConstraints(accounts: FiveAccount[], constraints: AccountConstraints): Promise<AccountValidationResult>;
    /**
     * Get account info using client-agnostic account fetcher interface
     */
    getAccountInfo(address: string, accountFetcher?: any): Promise<FiveAccount | null>;
    /**
     * Get multiple account infos in batch using client-agnostic interface
     */
    getMultipleAccountInfos(addresses: string[], accountFetcher?: any): Promise<Map<string, FiveAccount | null>>;
    /**
     * Check if accounts exist and are properly initialized
     */
    validateAccountsExist(addresses: string[]): Promise<{
        existing: string[];
        missing: string[];
        invalid: string[];
    }>;
    /**
     * Calculate total costs for account creation
     */
    calculateAccountCreationCosts(accounts: Array<{
        type: AccountType;
        size: number;
    }>): Promise<{
        rentExemption: number;
        transactionFees: number;
        total: number;
        breakdown: Array<{
            type: AccountType;
            size: number;
            rent: number;
        }>;
    }>;
    /**
     * Build standard account list for script execution
     */
    buildExecutionAccounts(scriptAccount: string, userAccount: string, additionalAccounts?: Array<{
        address: string;
        isSigner: boolean;
        isWritable: boolean;
    }>): FiveAccount[];
    private determineAccountTypeFromData;
}
/**
 * Account utilities for client-agnostic operations
 */
export declare class AccountUtils {
    /**
     * Build serializable account list (client-agnostic)
     */
    static buildSerializableAccounts(accounts: FiveAccount[]): Array<{
        pubkey: string;
        isSigner: boolean;
        isWritable: boolean;
    }>;
    /**
     * Deduplicate account list while preserving most permissive permissions
     */
    static deduplicateAccounts(accounts: FiveAccount[]): FiveAccount[];
    /**
     * Sort accounts by standard Solana conventions
     */
    static sortAccounts(accounts: FiveAccount[]): FiveAccount[];
    /**
     * Validate account list structure and compute statistics
     */
    static validateAccountList(accounts: FiveAccount[]): {
        valid: boolean;
        errors: string[];
        totalSize: number;
        requiredAccounts: FiveAccount[];
        optionalAccounts: FiveAccount[];
    };
    /**
     * Filter accounts by type
     */
    static filterAccountsByType(accounts: FiveAccount[], type: AccountType): FiveAccount[];
    /**
     * Calculate total size of accounts
     */
    static calculateTotalSize(accounts: FiveAccount[]): number;
}
//# sourceMappingURL=index.d.ts.map