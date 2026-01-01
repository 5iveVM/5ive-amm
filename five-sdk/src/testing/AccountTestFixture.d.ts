/**
 * Account Test Fixture Framework for Five VM
 *
 * Provides a reusable, composable pattern for testing Five account-system scripts.
 * Builders use this to:
 * 1. Define test accounts with specific constraints
 * 2. Initialize account state before test execution
 * 3. Validate constraints locally before on-chain testing
 * 4. Reuse common account setups across multiple tests
 *
 * Example:
 * ```
 * const fixture = new AccountTestFixture()
 *   .addSignerAccount('payer')
 *   .addStateAccount('counter', { value: 0 })
 *   .addMutableAccount('target')
 *   .build();
 * ```
 */
import { GeneratedAccountMeta } from './AccountMetaGenerator.js';
import { Connection, Keypair, PublicKey } from '@solana/web3.js';
/**
 * Account state initialization template
 * Builders provide this to set initial account data
 */
export interface AccountStateTemplate {
    [fieldName: string]: any;
}
/**
 * Account specification for fixture building
 * Describes what kind of account is needed
 */
export interface FixtureAccountSpec {
    name: string;
    type: 'signer' | 'mutable' | 'readonly' | 'state' | 'init';
    state?: AccountStateTemplate;
    description?: string;
}
/**
 * Build options for fixture compilation
 */
export interface BuildOptions {
    debug?: boolean;
    mode?: 'local' | 'onchain';
    connection?: Connection;
    payer?: Keypair;
    cleanup?: boolean;
    fiveVMProgramId?: PublicKey;
}
/**
 * Compiled fixture ready for test execution
 * Contains all accounts and their metadata
 */
export interface CompiledFixture {
    accounts: GeneratedAccountMeta[];
    accountsByName: Map<string, GeneratedAccountMeta>;
    stateData: Map<string, AccountStateTemplate>;
    specs: FixtureAccountSpec[];
    metadata: {
        signerCount: number;
        mutableCount: number;
        readonlyCount: number;
        stateCount: number;
    };
    cleanup?: () => Promise<void>;
}
/**
 * Validation result from constraint checking
 */
export interface ConstraintValidationResult {
    valid: boolean;
    errors: string[];
    warnings: string[];
}
/**
 * Test execution context with bound accounts
 */
export interface AccountExecutionContext {
    fixture: CompiledFixture;
    functionName: string;
    parameters: any[];
    accountAddresses: string[];
    keypairs: Map<string, Uint8Array>;
}
/**
 * Account Test Fixture Builder
 * Fluent API for constructing test account setups
 */
export declare class AccountTestFixture {
    private specs;
    private stateTemplates;
    /**
     * Add a signer account (@signer constraint)
     * Used for transaction signers, authority checks
     */
    addSignerAccount(name: string, options?: {
        description?: string;
    }): this;
    /**
     * Add a mutable account (@mut constraint)
     * Can be modified by the script
     */
    addMutableAccount(name: string, state?: AccountStateTemplate, options?: {
        description?: string;
    }): this;
    /**
     * Add a read-only account
     * Cannot be modified by the script
     */
    addReadOnlyAccount(name: string, state?: AccountStateTemplate, options?: {
        description?: string;
    }): this;
    /**
     * Add a state account (@mut state: StateType)
     * Typically for program state storage
     */
    addStateAccount(name: string, state?: AccountStateTemplate, options?: {
        description?: string;
    }): this;
    /**
     * Add an initialization account (@init constraint)
     * For account creation patterns
     */
    addInitAccount(name: string, options?: {
        description?: string;
    }): this;
    /**
     * Add multiple accounts in one call
     * Useful for standard patterns
     */
    addPattern(pattern: 'authorization' | 'state-mutation' | 'batch-operation'): this;
    /**
     * Compile fixture into accounts ready for execution
     * Supports both local (synthetic) and on-chain (real) account modes
     */
    build(options?: BuildOptions): Promise<CompiledFixture>;
    /**
     * Build fixture with local synthetic accounts (existing behavior)
     */
    private buildLocal;
    /**
     * Build fixture with real on-chain accounts
     */
    private buildOnChain;
    /**
     * Create individual account based on spec
     */
    private createAccount;
    /**
     * Validate fixture against ABI constraints
     */
    validateAgainstABI(abiFunction: any): ConstraintValidationResult;
    /**
     * Get summary of fixture specs for debugging
     */
    getSummary(): string;
}
/**
 * Predefined fixture templates for common patterns
 * Builders can extend these for their specific needs
 */
export declare class FixtureTemplates {
    /**
     * Simple state mutation pattern
     * For scripts that increment counters, track modifications, etc.
     */
    static stateCounter(): AccountTestFixture;
    /**
     * Authorization pattern
     * For scripts that check permissions via @signer
     */
    static authorization(): AccountTestFixture;
    /**
     * Account creation pattern
     * For scripts using @init constraint
     */
    static accountCreation(): AccountTestFixture;
    /**
     * Multi-account transaction pattern
     * For scripts that operate on multiple mutable accounts
     */
    static batchOperation(): AccountTestFixture;
    /**
     * Complex authorization pattern
     * For multi-signature or advanced permission schemes
     */
    static multiSigPattern(): AccountTestFixture;
    /**
     * PDA pattern
     * For scripts that use Program Derived Addresses
     */
    static pdaPattern(): AccountTestFixture;
}
/**
 * Test execution builder
 * Combines fixture with test parameters for execution
 */
export declare class AccountTestExecutor {
    /**
     * Bind fixture accounts to a test execution
     */
    static bindFixture(fixture: CompiledFixture, functionName: string, parameters?: any[]): AccountExecutionContext;
    /**
     * Validate execution context before running
     */
    static validateContext(context: AccountExecutionContext): ConstraintValidationResult;
    /**
     * Get human-readable execution summary
     */
    static getSummary(context: AccountExecutionContext): string;
}
//# sourceMappingURL=AccountTestFixture.d.ts.map