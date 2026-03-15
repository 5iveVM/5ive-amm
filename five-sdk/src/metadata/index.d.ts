/**
 * Five SDK Script Metadata System
 *
 * Real implementation for parsing script account data and extracting ABI information
 * from deployed Five scripts. This replaces mock implementations with production-ready
 * Solana account data parsing.
 */
/**
 * Account data interface for client-agnostic blockchain interactions
 */
export interface AccountData {
    /** Account address as base58 string */
    address: string;
    /** Account data as byte array */
    data: Uint8Array;
    /** Account owner program ID */
    owner: string;
    /** Account balance in lamports */
    lamports: number;
}
/**
 * Account fetcher interface for retrieving account data
 */
export interface AccountFetcher {
    /** Get single account data */
    getAccountData(address: string): Promise<AccountData | null>;
    /** Get multiple account data in batch */
    getMultipleAccountsData(addresses: string[]): Promise<Map<string, AccountData | null>>;
}
/**
 * Script metadata extracted from deployed script accounts
 */
export interface ScriptMetadata {
    /** Script account address */
    address: string;
    /** Script bytecode */
    bytecode: Uint8Array;
    /** Script ABI with function definitions */
    abi: ScriptABI;
    /** Deploy timestamp */
    deployedAt: number;
    /** Script version */
    version: string;
    /** Authority that deployed the script */
    authority: string;
}
/**
 * Script ABI definition
 */
export interface ScriptABI {
    /** Script name */
    name: string;
    /** Function definitions */
    functions: FunctionDefinition[];
    /** Type definitions */
    types?: TypeDefinition[];
    /** Account constraints */
    accounts?: AccountConstraint[];
}
/**
 * Function definition in script ABI
 */
export interface FunctionDefinition {
    /** Function name */
    name: string;
    /** Function index in bytecode */
    index: number;
    /** Function parameters */
    parameters: ParameterDefinition[];
    /** Return type */
    returnType?: string;
    /** Function visibility */
    visibility: 'public' | 'private';
    /** Documentation */
    docs?: string;
}
/**
 * Parameter definition
 */
export interface ParameterDefinition {
    /** Parameter name */
    name: string;
    /** Parameter type */
    type: string;
    /** Whether this is an account parameter */
    is_account?: boolean;
    /** Compiler-injected parameter not authored in source */
    implicit?: boolean;
    /** Parameter origin */
    source?: 'authored' | 'compiler';
    /** Account attributes (e.g., "mut", "signer", "init") */
    attributes?: string[];
    /** Whether parameter is optional */
    optional?: boolean;
    /** Parameter documentation */
    docs?: string;
}
/**
 * Type definition
 */
export interface TypeDefinition {
    /** Type name */
    name: string;
    /** Type structure */
    structure: 'struct' | 'enum' | 'alias';
    /** Type fields (for structs) */
    fields?: Array<{
        name: string;
        type: string;
    }>;
    /** Type variants (for enums) */
    variants?: Array<{
        name: string;
        value?: number;
    }>;
    /** Alias target (for type aliases) */
    target?: string;
}
/**
 * Account constraint definition
 */
export interface AccountConstraint {
    /** Account name */
    name: string;
    /** Account type */
    type: 'signer' | 'writable' | 'readonly' | 'pda';
    /** Seeds for PDA derivation */
    seeds?: string[];
    /** Required account properties */
    properties?: Record<string, any>;
}
/**
 * Script metadata parser and manager
 */
export declare class ScriptMetadataParser {
    private static readonly SCRIPT_MAGIC;
    private static readonly CURRENT_VERSION;
    private static readonly HEADER_SIZE;
    /**
     * Parse script metadata from account data
     */
    static parseMetadata(accountData: Uint8Array, address: string): ScriptMetadata;
    /**
     * Get script metadata from blockchain using account fetcher
     */
    static getScriptMetadata(accountFetcher: AccountFetcher, scriptAddress: string): Promise<ScriptMetadata>;
    /**
     * Get multiple script metadata entries using account fetcher
     */
    static getMultipleScriptMetadata(accountFetcher: AccountFetcher, scriptAddresses: string[]): Promise<Map<string, ScriptMetadata | null>>;
    /**
     * Extract function signatures from ABI
     */
    static extractFunctionSignatures(abi: ScriptABI): Array<{
        name: string;
        index: number;
        parameters: ParameterDefinition[];
        signature: string;
    }>;
    /**
     * Generate function signature string
     */
    static generateFunctionSignature(func: FunctionDefinition): string;
    /**
     * Validate script ABI structure
     */
    static validateABI(abi: any): {
        valid: boolean;
        errors: string[];
    };
    /**
     * Validate function definition
     */
    private static validateFunction;
    private static readU32;
    private static readU64;
    private static arraysEqual;
}
/**
 * Client-agnostic metadata cache
 */
export declare class MetadataCache {
    private cache;
    private defaultTTL;
    /**
     * Get metadata from cache or fetch
     */
    getMetadata(scriptAddress: string, fetcher: (address: string) => Promise<ScriptMetadata>, ttl?: number): Promise<ScriptMetadata>;
    /**
     * Invalidate cache entry
     */
    invalidate(scriptAddress: string): void;
    /**
     * Clear expired entries
     */
    cleanup(): void;
    /**
     * Get cache statistics
     */
    getStats(): {
        size: number;
        hitRate: number;
        entries: Array<{
            address: string;
            age: number;
            ttl: number;
        }>;
    };
}
//# sourceMappingURL=index.d.ts.map
