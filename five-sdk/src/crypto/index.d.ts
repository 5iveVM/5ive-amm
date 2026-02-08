/**
 * Five SDK crypto utilities.
 */
/**
 * Program Derived Address (PDA) utilities - pure implementation
 */
export declare class PDAUtils {
    private static readonly PDA_MARKER;
    /**
     * Derive script account using seed-based derivation compatible with SystemProgram.createAccountWithSeed
     */
    static deriveScriptAccount(bytecode: Uint8Array, programId?: string): Promise<{
        address: string;
        bump: number;
        seed: string;
    }>;
    /**
     * Pure PDA derivation implementation (Solana-compatible)
     */
    static findProgramAddress(seeds: Buffer[], programId: string): Promise<{
        address: string;
        bump: number;
    }>;
    /**
     * Simplified curve check (placeholder for real ed25519 curve validation)
     */
    private static isOffCurve;
    /**
     * Derive metadata account PDA for script
     */
    static deriveMetadataAccount(scriptAccount: string, programId?: string): Promise<{
        address: string;
        bump: number;
    }>;
    /**
     * Derive user state account PDA
     */
    static deriveUserStateAccount(userPublicKey: string, scriptAccount: string, programId?: string): Promise<{
        address: string;
        bump: number;
    }>;
    /**
     * Derive VM state PDA - temporarily use known correct address
     * TODO: Fix PDA derivation algorithm to match Solana exactly
     */
    static deriveVMStatePDA(programId?: string): Promise<{
        address: string;
        bump: number;
    }>;
    /**
     * Validate that a given address is a valid PDA for the given seeds
     */
    static validatePDA(address: string, seeds: Buffer[], programId: string): Promise<boolean>;
}
/**
 * Base58 encoding/decoding utilities (proper Solana-compatible implementation)
 */
export declare class Base58Utils {
    /**
     * Encode bytes to base58 string
     */
    static encode(bytes: Uint8Array): string;
    /**
     * Decode base58 string to bytes
     */
    static decode(base58String: string): Uint8Array;
    /**
     * Validate base58 string format
     */
    static isValid(base58String: string): boolean;
    /**
     * Generate a random base58 string of specified length
     */
    static random(byteLength?: number): string;
}
/**
 * PublicKey utilities for Solana addresses - pure implementation
 */
export declare class SolanaPublicKeyUtils {
    /**
     * Validate Solana public key format (32 bytes, valid base58)
     */
    static isValid(address: string): boolean;
    /**
     * Normalize and return valid address (throws on invalid)
     */
    static normalize(address: string): string;
    /**
     * Convert address string to bytes (throws on invalid)
     */
    static toBytes(address: string): Uint8Array;
    /**
     * Convert bytes to address string
     */
    static fromBytes(bytes: Uint8Array): string;
    /**
     * Generate random public key for testing
     */
    static random(): string;
    /**
     * Check if two addresses are equal
     */
    static equals(address1: string, address2: string): boolean;
}
/**
 * Rent calculation utilities using Solana rent sysvar
 */
export declare class RentCalculator {
    private static readonly RENT_PER_BYTE_YEAR;
    private static readonly RENT_EXEMPTION_THRESHOLD;
    /**
     * Calculate minimum rent-exempt balance for account size
     */
    static calculateMinimumBalance(accountSize: number): Promise<number>;
    /**
     * Check if balance is rent exempt for given account size
     */
    static isRentExempt(balance: number, accountSize: number): Promise<boolean>;
    /**
     * Calculate minimum rent-exempt balance for account size (legacy method)
     */
    static calculateRentExemption(accountSize: number): number;
    /**
     * Get estimated rent for script account based on bytecode size
     */
    static getScriptAccountRent(bytecodeSize: number): number;
    /**
     * Get estimated rent for user state account
     */
    static getUserStateAccountRent(): number;
    /**
     * Get estimated rent for metadata account
     */
    static getMetadataAccountRent(): number;
    /**
     * Format lamports as SOL string
     */
    static formatSOL(lamports: number): string;
    /**
     * Convert SOL to lamports
     */
    static solToLamports(sol: number): number;
    /**
     * Convert lamports to SOL
     */
    static lamportsToSol(lamports: number): number;
}
/**
 * Hash utilities for cryptographic operations
 */
export declare class HashUtils {
    /**
     * SHA256 hash of data
     */
    static sha256(data: Uint8Array): Promise<Uint8Array>;
    /**
     * Create deterministic seed from multiple inputs
     */
    static createSeed(inputs: (string | Uint8Array)[]): Promise<Uint8Array>;
    /**
     * Generate random bytes for cryptographic use
     */
    static randomBytes(length: number): Promise<Uint8Array>;
}
/**
 * Account validation utilities
 */
export declare class AccountValidator {
    /**
     * Validate account address format
     */
    static validateAddress(address: string): {
        valid: boolean;
        errors: string[];
        normalizedAddress: string | null;
    };
    /**
     * Validate list of account addresses
     */
    static validateAccountList(addresses: string[]): {
        valid: boolean;
        errors: string[];
        validAddresses: string[];
        invalidAddresses: string[];
    };
    /**
     * Validate program ID
     */
    static validateProgramId(programId: string): {
        isValid: boolean;
        error?: string;
    };
    /**
     * Validate script account structure
     */
    static validateScriptAccount(accountData: {
        address: string;
        bytecodeSize?: number;
        rentExempt?: boolean;
    }): {
        isValid: boolean;
        errors: string[];
    };
}
//# sourceMappingURL=index.d.ts.map
