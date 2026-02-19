/**
 * Five SDK - Unified client library for Five VM scripts
 *
 * Provides a standardized way to interact with Five scripts deployed on Solana.
 * Key concepts:
 * - Five scripts (.v) compile to bytecode (.bin)
 * - Bytecode is deployed to script accounts on Solana
 * - Five VM Program executes scripts from script accounts
 * - This SDK abstracts the complexity while maintaining performance
 */
import { FiveSDKConfig, FiveBytecode, FiveScriptSource, CompilationOptions, CompilationResult, DeploymentOptions, SerializedDeployment, SerializedExecution, FiveCompiledFile, FiveFunction, FunctionNameEntry } from "./types.js";
import { ScriptMetadata } from "./metadata/index.js";
/**
 * Main Five SDK class - entry point for all Five VM interactions
 * Client-agnostic design: generates serialized transaction data for any Solana client library
 */
export declare class FiveSDK {
    private static compiler;
    private static parameterEncoder;
    private static metadataCache;
    private fiveVMProgramId;
    private debug;
    private network?;
    /**
     * Create a new Five SDK instance (for configuration)
     */
    constructor(config?: FiveSDKConfig);
    /**
     * Get SDK configuration
     */
    getConfig(): FiveSDKConfig & {
        network?: string;
    };
    /**
     * Initialize static components (lazy initialization)
     */
    private static initializeComponents;
    /**
     * Create SDK instance with default configuration
     */
    static create(options?: {
        debug?: boolean;
        fiveVMProgramId?: string;
    }): FiveSDK;
    /**
     * Create SDK instance for devnet
     */
    static devnet(options?: {
        debug?: boolean;
        fiveVMProgramId?: string;
    }): FiveSDK;
    /**
     * Create SDK instance for mainnet
     */
    static mainnet(options?: {
        debug?: boolean;
        fiveVMProgramId?: string;
    }): FiveSDK;
    /**
     * Create SDK instance for localnet
     */
    static localnet(options?: {
        debug?: boolean;
        fiveVMProgramId?: string;
    }): FiveSDK;
    /**
     * Compile Five script source code to bytecode (static method)
     */
    static compile(source: FiveScriptSource, options?: CompilationOptions & {
        debug?: boolean;
    }): Promise<CompilationResult>;
    /**
     * Compile a project via five.toml entry-point discovery
     */
    static compileProject(projectPath?: string, options?: CompilationOptions & {
        debug?: boolean;
    }): Promise<CompilationResult>;
    /**
     * Compile with automatic module discovery (client-agnostic)
     */
    static compileWithDiscovery(entryPoint: string, // File path
    options?: CompilationOptions & {
        debug?: boolean;
    }): Promise<CompilationResult>;
    /**
     * Discover modules from entry point
     */
    static discoverModules(entryPoint: string, options?: {
        debug?: boolean;
    }): Promise<string[]>;
    /**
     * Compile script from file path (static method)
     */
    static compileFile(filePath: string, options?: CompilationOptions & {
        debug?: boolean;
    }): Promise<CompilationResult>;
    /**
     * Load .five file and extract components
     */
    static loadFiveFile(fileContent: string): Promise<{
        bytecode: FiveBytecode;
        abi: any;
        debug?: any;
    }>;
    /**
     * Extract bytecode from .five file for deployment
     */
    static extractBytecode(fiveFile: FiveCompiledFile): FiveBytecode;
    /**
     * Resolve function name to index using ABI
     */
    static resolveFunctionIndex(abi: any, functionName: string): number;
    /**
     * Execute bytecode directly using WASM VM for local testing and development
     * This bypasses Solana entirely - no network connection needed!
     */
    static executeLocally(bytecode: FiveBytecode, functionName: string | number, parameters?: any[], options?: {
        debug?: boolean;
        trace?: boolean;
        computeUnitLimit?: number;
        abi?: any;
        accounts?: string[];
    }): Promise<{
        success: boolean;
        result?: any;
        logs?: string[];
        computeUnitsUsed?: number;
        executionTime?: number;
        error?: string;
        trace?: any[];
    }>;
    /**
     * Compile and execute a script locally in one step (perfect for rapid testing)
     */
    static compileAndExecuteLocally(source: FiveScriptSource, functionName: string | number, parameters?: any[], options?: {
        debug?: boolean;
        trace?: boolean;
        optimize?: boolean;
        computeUnitLimit?: number;
        accounts?: string[];
    }): Promise<{
        success: boolean;
        compilationErrors: import("./types.js").CompilationError[] | undefined;
        error: string;
    } | {
        compilation: CompilationResult;
        bytecodeSize: number;
        functions: FiveFunction[] | undefined;
        success: boolean;
        result?: any;
        logs?: string[];
        computeUnitsUsed?: number;
        executionTime?: number;
        error?: string;
        trace?: any[];
        compilationErrors?: never;
    }>;
    /**
     * Validate bytecode format and structure using WASM VM
     */
    static validateBytecode(bytecode: FiveBytecode, options?: {
        debug?: boolean;
    }): Promise<{
        valid: boolean;
        errors?: string[];
        metadata?: any;
        functions?: any[];
    }>;
    /**
     * Generate deployment instruction data (static method)
     *
     * Creates a complete deployment transaction that includes:
     * 1. Creating the script account PDA owned by Five VM program
     * 2. Deploying bytecode to the created account
     */
    static generateDeployInstruction(bytecode: FiveBytecode, deployer: string, // base58 pubkey string
    options?: DeploymentOptions & {
        debug?: boolean;
    }): Promise<SerializedDeployment>;
    /**
     * Generate execution instruction data (static method)
     */
    static generateExecuteInstruction(scriptAccount: string, // base58 pubkey string
    functionName: string | number, parameters?: any[], accounts?: string[], // base58 pubkey strings
    connection?: any, // Optional Solana connection for metadata lookup
    options?: {
        debug?: boolean;
        computeUnitLimit?: number;
        vmStateAccount?: string;
    }): Promise<SerializedExecution>;
    /**
     * Get script metadata for ABI-driven parameter encoding (static method)
     * Now uses real Solana account data parsing instead of mocks
     */
    static getScriptMetadata(scriptAccount: string, connection?: any): Promise<{
        functions: any[];
    }>;
    /**
     * Get script metadata with explicit connection (for use with any Solana client)
     */
    static getScriptMetadataWithConnection(scriptAccount: string, connection: any): Promise<ScriptMetadata>;
    /**
     * Parse script metadata from raw account data (client-agnostic)
     */
    static parseScriptMetadata(accountData: Uint8Array, address: string): ScriptMetadata;
    /**
     * Get script metadata with caching (for performance)
     */
    static getCachedScriptMetadata(scriptAccount: string, connection: any, cacheTTL?: number): Promise<ScriptMetadata>;
    /**
     * Invalidate metadata cache for a script
     */
    static invalidateMetadataCache(scriptAccount: string): void;
    /**
     * Get metadata cache statistics
     */
    static getMetadataCacheStats(): any;
    /**
     * Derive script account PDA from bytecode using real Solana PDA derivation
     */
    private static deriveScriptAccount;
    /**
     * Derive VM state PDA using hardcoded seed (matches Five VM program)
     */
    private static deriveVMStatePDA;
    /**
     * Load WASM VM for direct execution
     */
    private static wasmVMInstance;
    private static loadWasmVM;
    /**
     * Calculate rent exemption for account size using real Solana rent calculations
     */
    private static calculateRentExemption;
    /**
     * Encode deployment instruction data
     */
    private static encodeDeployInstruction;
    /**
     * Encode execution instruction data
     */
    private static encodeExecuteInstruction;
    /**
     * Encode parameters with ABI guidance
     */
    private static encodeParametersWithABI;
    /**
     * Varint encode a number for instruction data
     */
    private static encodeVarintNumber;
    /**
     * Estimate compute units for function execution
     */
    private static estimateComputeUnits;
    /**
     * Infer parameter type from JavaScript value for varint encoding (varint)
     */
    private static inferParameterType;
    /**
     * Fetch account data and deserialize script/account payloads.
     * Canonical neutral entrypoint.
     */
    static fetchAccountAndDeserialize(accountAddress: string, connection: any, options?: {
        debug?: boolean;
        parseMetadata?: boolean;
        validateEncoding?: boolean;
    }): Promise<{
        success: boolean;
        accountInfo?: {
            address: string;
            owner: string;
            lamports: number;
            dataLength: number;
        };
        scriptMetadata?: ScriptMetadata;
        rawBytecode?: Uint8Array;
        decodedData?: {
            header: any;
            bytecode: Uint8Array;
            abi?: any;
            functions?: Array<{
                name: string;
                index: number;
                parameters: any[];
            }>;
        };
        error?: string;
        logs?: string[];
    }>;
    /**
     * Batch fetch multiple accounts and deserialize script/account payloads.
     * Canonical neutral entrypoint.
     */
    static fetchMultipleAccountsAndDeserialize(accountAddresses: string[], connection: any, options?: {
        debug?: boolean;
        parseMetadata?: boolean;
        validateEncoding?: boolean;
        batchSize?: number;
    }): Promise<Map<string, {
        success: boolean;
        accountInfo?: any;
        scriptMetadata?: ScriptMetadata;
        rawBytecode?: Uint8Array;
        decodedData?: any;
        error?: string;
        logs?: string[];
    }>>;
    /**
     * Deserialize instruction parameters from instruction data.
     * Canonical neutral entrypoint.
     */
    static deserializeParameters(instructionData: Uint8Array, expectedTypes?: string[], options?: {
        debug?: boolean;
    }): Promise<{
        success: boolean;
        parameters?: Array<{
            type: string;
            value: any;
        }>;
        functionIndex?: number;
        discriminator?: number;
        error?: string;
    }>;
    /**
     * Validate Five bytecode envelope/header encoding.
     * Canonical neutral entrypoint.
     */
    static validateBytecodeEncoding(bytecode: Uint8Array, debug?: boolean): Promise<{
        valid: boolean;
        info?: string;
        error?: string;
    }>;
    private static parseInstructionParametersManually;
    private static readVarintNumber;
    /**
     * Execute script with before/after account state tracking
     * This fetches account data before execution, runs the script, then fetches after
     * to show exactly what changed including global fields
     */
    static executeWithStateDiff(scriptAccount: string, connection: any, signerKeypair: any, functionName: string | number, parameters?: any[], options?: {
        debug?: boolean;
        network?: string;
        computeUnitLimit?: number;
        trackGlobalFields?: boolean;
        additionalAccounts?: string[];
        includeVMState?: boolean;
    }): Promise<{
        success: boolean;
        execution?: {
            transactionId?: string;
            result?: any;
            computeUnitsUsed?: number;
            logs?: string[];
        };
        stateDiff?: {
            beforeState: Map<string, any>;
            afterState: Map<string, any>;
            changes: Array<{
                account: string;
                fieldName?: string;
                oldValue: any;
                newValue: any;
                changeType: "created" | "modified" | "deleted";
            }>;
            globalFieldChanges?: Array<{
                fieldName: string;
                oldValue: any;
                newValue: any;
            }>;
        };
        error?: string;
        logs?: string[];
    }>;
    /**
     * Compare account states and find detailed differences
     */
    private static computeStateDifferences;
    /**
     * Extract global fields from script metadata
     */
    private static extractGlobalFields;
    /**
     * Compare global field values between before and after states
     */
    private static computeGlobalFieldChanges;
    /**
     * Compare script metadata for changes
     */
    private static compareScriptMetadata;
    /**
     * Extract state section from Five VM bytecode
     * This is a simplified version - full implementation would require
     * complete Five VM state format knowledge
     */
    private static extractStateSection;
    /**
     * Utility: Check if two bytecode arrays are equal
     */
    private static bytecodeEqual;
    /**
     * Utility: Generate simple hash of bytecode for comparison
     */
    private static hashBytecode;
    /**
     * Utility: Deep equality check
     */
    private static deepEqual;
    /**
     * Execute script on Solana with real transaction submission
     * This is the method the CLI should use for actual on-chain execution
     */
    static executeOnSolana(scriptAccount: string, // The deployed script account (from deployment)
    connection: any, // Solana Connection object
    signerKeypair: any, // Solana Keypair object for signing
    functionName: string | number, parameters?: any[], accounts?: string[], // Additional account pubkeys as base58 strings
    options?: {
        debug?: boolean;
        network?: string;
        computeUnitLimit?: number;
        computeUnitPrice?: number;
        maxRetries?: number;
        skipPreflight?: boolean;
        vmStateAccount?: string;
    }): Promise<{
        success: boolean;
        result?: any;
        transactionId?: string;
        computeUnitsUsed?: number;
        cost?: number;
        error?: string;
        logs?: string[];
    }>;
    /**
     * Execute deployed script account on-chain - wrapper for executeOnSolana with simpler interface
     * This is the method the CLI should use for script account execution
     */
    static executeScriptAccount(scriptAccount: string, functionIndex: number | undefined, parameters: any[] | undefined, connection: any, // Solana Connection object
    signerKeypair: any, // Solana Keypair object
    options?: {
        debug?: boolean;
        network?: string;
        computeBudget?: number;
        maxRetries?: number;
        vmStateAccount?: string;
    }): Promise<{
        success: boolean;
        result?: any;
        transactionId?: string;
        computeUnitsUsed?: number;
        cost?: number;
        error?: string;
        logs?: string[];
    }>;
    /**
     * Extract function names from compiled bytecode
     */
    static getFunctionNames(bytecode: FiveBytecode): Promise<FunctionNameEntry[]>;
    /**
     * Call a function by name instead of index
     */
    static callFunctionByName(scriptAccount: string, functionName: string, parameters?: any[], accounts?: string[], connection?: any, options?: {
        debug?: boolean;
        computeUnitLimit?: number;
        vmStateAccount?: string;
    }): Promise<SerializedExecution>;
    /**
     * Generate serialized execution data by function index.
     */
    static executeByIndex(scriptAccount: string, functionIndex: number, parameters?: any[], accounts?: string[], connection?: any, options?: {
        debug?: boolean;
        computeUnitLimit?: number;
        vmStateAccount?: string;
    }): Promise<SerializedExecution>;
    /**
     * Get function names from a deployed script account
     */
    static getFunctionNamesFromScriptAccount(scriptAccount: string, connection?: any): Promise<FunctionNameEntry[] | null>;
    /**
     * Deploy bytecode to Solana using the correct two-transaction pattern
     * This is the method the CLI should use for actual deployment
     */
    static deployToSolana(bytecode: FiveBytecode, connection: any, // Solana Connection object
    deployerKeypair: any, // Solana Keypair object
    options?: {
        debug?: boolean;
        network?: string;
        computeBudget?: number;
        maxRetries?: number;
        fiveVMProgramId?: string;
    }): Promise<{
        success: boolean;
        programId?: string;
        transactionId?: string;
        deploymentCost?: number;
        error?: string;
        logs?: string[];
        vmStateAccount?: string;
    }>;
    /**
     * Deploy large bytecode programs using InitLargeProgram + AppendBytecode pattern
     * Automatically handles programs larger than single transaction limits
     */
    static deployLargeProgramToSolana(bytecode: FiveBytecode, connection: any, // Solana Connection object
    deployerKeypair: any, // Solana Keypair object
    options?: {
        chunkSize?: number;
        debug?: boolean;
        network?: string;
        maxRetries?: number;
        fiveVMProgramId?: string;
        progressCallback?: (chunk: number, total: number) => void;
    }): Promise<{
        success: boolean;
        scriptAccount?: string;
        transactionIds?: string[];
        totalTransactions?: number;
        deploymentCost?: number;
        chunksUsed?: number;
        vmStateAccount?: string;
        error?: string;
        logs?: string[];
    }>;
    /**
     * Deploy large program to Solana with OPTIMIZED instruction combining (50-70% fewer transactions)
     *
     * This uses the new optimized Five VM instructions:
     * - InitLargeProgramWithChunk (discriminator 4) - combines initialization with first chunk
     * - AppendMultipleBytecodeChunks (discriminator 5) - appends multiple chunks per transaction
     *
     * Benefits:
     * - Reduces transaction count by 50-70%
     * - Lower deployment costs due to fewer transactions
     * - Faster deployment due to fewer network round-trips
     * - Pre-allocates full account space to eliminate rent transfers
     */
    static deployLargeProgramOptimizedToSolana(bytecode: FiveBytecode, connection: any, // Solana Connection object
    deployerKeypair: any, // Solana Keypair object
    options?: {
        chunkSize?: number;
        debug?: boolean;
        network?: string;
        maxRetries?: number;
        fiveVMProgramId?: string;
        progressCallback?: (transaction: number, total: number) => void;
    }): Promise<{
        success: boolean;
        scriptAccount?: string;
        transactionIds?: string[];
        totalTransactions?: number;
        deploymentCost?: number;
        chunksUsed?: number;
        vmStateAccount?: string;
        optimizationSavings?: {
            transactionsSaved: number;
            estimatedCostSaved: number;
        };
        error?: string;
        logs?: string[];
    }>;
    /**
     * Group chunks optimally for multi-chunk transactions
     */
    private static groupChunksForOptimalTransactions;
    /**
     * Create instruction data for multi-chunk AppendBytecode instruction
     * Format: [discriminator(5), num_chunks, chunk1_len, chunk1_data, chunk2_len, chunk2_data, ...]
     */
    private static createMultiChunkInstructionData;
    /**
     * Split bytecode into chunks of specified size
     */
    private static chunkBytecode;
}
export declare const createFiveSDK: (config?: FiveSDKConfig) => FiveSDK;
export default FiveSDK;
//# sourceMappingURL=FiveSDK.d.ts.map
