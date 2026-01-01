/**
 * Account Meta Generator for Five VM Testing
 *
 * Generates AccountMeta structures for testing account system scripts
 * based on ABI requirements and constraint types (@signer, @mut, @init)
 */
/**
 * Account constraint types from Five VM
 */
export interface AccountConstraints {
    name: string;
    writable: boolean;
    signer: boolean;
    init?: boolean;
}
/**
 * Generated account metadata for testing
 */
export interface GeneratedAccountMeta {
    pubkey: string;
    isSigner: boolean;
    isWritable: boolean;
    keypair?: {
        publicKey: string;
        secretKey: Uint8Array;
    };
}
/**
 * Test account generation context
 */
export interface TestAccountContext {
    script: string;
    functionIndex: number;
    accounts: GeneratedAccountMeta[];
    stateData?: Map<string, any>;
}
/**
 * Account Meta Generator for Five VM test scripts
 */
export declare class AccountMetaGenerator {
    private static accountCache;
    private static stateDataCache;
    /**
     * Generate AccountMeta array from ABI function definition
     */
    static generateAccountsForFunction(abi: any, functionName: string, options?: {
        reuseAccounts?: boolean;
        generateStateData?: boolean;
        debug?: boolean;
    }): Promise<TestAccountContext>;
    /**
     * Generate single AccountMeta from account specification
     */
    private static generateAccountMeta;
    /**
     * Generate signer account with keypair
     */
    private static generateSignerAccount;
    /**
     * Generate regular (non-signer) account
     */
    private static generateRegularAccount;
    /**
     * Check if account is a state account that needs data
     */
    private static isStateAccount;
    /**
     * Generate mock state data for state accounts
     */
    private static generateStateData;
    /**
     * Format accounts for Five CLI execution
     */
    static formatAccountsForCLI(context: TestAccountContext): {
        accountsParam: string;
        keypairsNeeded: Array<{
            name: string;
            keypair: any;
        }>;
    };
    /**
     * Generate accounts from .five file
     */
    static generateFromFiveFile(fiveFilePath: string, functionName?: string, options?: {
        reuseAccounts?: boolean;
        generateStateData?: boolean;
        debug?: boolean;
    }): Promise<TestAccountContext>;
    /**
     * Clear account cache (useful for testing)
     */
    static clearCache(): void;
    /**
     * Get account cache statistics
     */
    static getCacheStats(): {
        accountsCached: number;
        stateDataCached: number;
    };
}
/**
 * Utility functions for account management
 */
export declare class AccountTestUtils {
    /**
     * Create test accounts for common constraint patterns
     */
    static createStandardTestAccounts(): Promise<{
        payer: GeneratedAccountMeta;
        authority: GeneratedAccountMeta;
        state: GeneratedAccountMeta;
        readonly: GeneratedAccountMeta;
    }>;
    /**
     * Validate account constraints match requirements
     */
    static validateAccountConstraints(accounts: GeneratedAccountMeta[], requirements: AccountConstraints[]): {
        valid: boolean;
        errors: string[];
    };
}
//# sourceMappingURL=AccountMetaGenerator.d.ts.map