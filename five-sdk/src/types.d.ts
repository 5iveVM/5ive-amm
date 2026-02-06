/**
 * Five SDK Core Types
 *
 * Defines the core types and interfaces for the Five SDK with correct terminology:
 * - Five scripts (.v files) compile to bytecode (.bin files)
 * - Script accounts store bytecode on Solana
 * - Five VM Program executes scripts from script accounts
 * - Five VM is the virtual machine that executes bytecode
 */
/**
 * Five VM Program ID - the actual Solana program that executes Five bytecode
 */
export declare const FIVE_VM_PROGRAM_ID = "9MHGM73eszNUtmJS6ypDCESguxWhCBnkUPpTMyLGqURH";
/**
 * Five script source code in .v format
 */
export type FiveScriptSource = string;
/**
 * Compiled Five bytecode in .bin format
 */
export type FiveBytecode = Uint8Array;
/**
 * Source for a module in multi-file compilation
 */
export interface ModuleSource {
    name: string;
    source: string;
    isEntry?: boolean;
}
/**
 * Complete Five compiled file format containing bytecode, ABI, and debug info
 */
export interface FiveCompiledFile {
    /** Compiled bytecode (base64 encoded) */
    bytecode: string;
    /** ABI information for function calls */
    abi: {
        functions: FiveFunction[];
        fields?: any[];
        version?: string;
    };
    /** Human-readable compilation log (disassembly) */
    disassembly?: string[];
    /** Debug information */
    debug?: {
        sourceMap?: string;
        instructionMapping?: any;
        compilationInfo?: any;
    };
    /** File format version */
    version?: string;
}
/**
 * Script account containing Five bytecode (as base58 string)
 */
export type ScriptAccount = string;
/**
 * Five VM execution result
 */
export interface FiveVMResult {
    success: boolean;
    result?: any;
    computeUnitsUsed?: number;
    executionTime?: number;
    logs?: string[];
    status?: string;
    error?: {
        message: string;
        instructionPointer?: number;
        stackTrace?: string[];
    };
}
/**
 * Five script metadata (client-agnostic)
 */
export interface FiveScript {
    /** Script account containing bytecode as base58 string */
    scriptAccount: string;
    /** Compiled bytecode */
    bytecode: FiveBytecode;
    /** Script metadata */
    metadata: {
        name?: string;
        version?: string;
        functions: FiveFunction[];
        abi?: FiveScriptABI;
    };
}
/**
 * Five script ABI definition
 */
export interface FiveScriptABI {
    /** Function definitions */
    functions: FiveFunction[];
    /** Account specifications */
    accounts?: FiveAccountSpec[];
    /** ABI version */
    version?: string;
}
/**
 * Account specification in ABI
 */
export interface FiveAccountSpec {
    /** Account name */
    name: string;
    /** Account type */
    type: "signer" | "writable" | "readonly";
    /** Whether account is optional */
    optional?: boolean;
}
/**
 * Function definition within a Five script
 */
export interface FiveFunction {
    name: string;
    index: number;
    parameters: FiveParameter[];
    returnType?: FiveType;
}
/**
 * Function parameter definition
 */
export interface FiveParameter {
    name: string;
    type: FiveType;
    optional?: boolean;
}
/**
 * Five VM supported types
 */
export type FiveType = "u8" | "u16" | "u32" | "u64" | "i8" | "i16" | "i32" | "i64" | "bool" | "string" | "pubkey" | "bytes" | "array";
/**
 * Client-agnostic serialized instruction data
 */
export interface SerializedInstruction {
    /** Five VM Program ID as base58 string */
    programId: string;
    /** Account metadata for instruction */
    accounts: SerializableAccount[];
    /** Raw instruction data as base64 string */
    data: string;
}
/**
 * Account metadata for serialized instructions
 */
export interface SerializableAccount {
    /** Account public key as base58 string */
    pubkey: string;
    /** Whether this account must sign */
    isSigner: boolean;
    /** Whether this account is writable */
    isWritable: boolean;
}
/**
 * Serialized deployment data
 */
export interface SerializedDeployment {
    /** Deployment instruction data */
    instruction: SerializedInstruction;
    /** Script account (PDA) as base58 string */
    scriptAccount: string;
    /** Required signers as base58 strings */
    requiredSigners: string[];
    /** Estimated deployment cost in lamports */
    estimatedCost: number;
    /** Bytecode size in bytes */
    bytecodeSize: number;
    /** Setup instructions for account creation (optional) */
    setupInstructions?: {
        createScriptAccount?: {
            pda: string;
            seed: string;
            space: number;
            rent: number;
            owner: string;
        };
    };
}
/**
 * Serialized execution data
 */
export interface SerializedExecution {
    /** Execution instruction data */
    instruction: SerializedInstruction;
    /** Script account as base58 string */
    scriptAccount: string;
    /** Encoded parameters data */
    parameters: EncodedParameters;
    /** Required signers as base58 strings */
    requiredSigners: string[];
    /** Estimated compute units */
    estimatedComputeUnits: number;
}
/**
 * Encoded parameter data
 */
export interface EncodedParameters {
    /** Function name or index */
    function: string | number;
    /** Raw encoded parameter bytes */
    data: Uint8Array;
    /** Parameter count */
    count: number;
}
/**
 * Five SDK configuration options (client-agnostic)
 */
export interface FiveSDKConfig {
    /** Five VM Program ID as base58 string (defaults to standard program) */
    fiveVMProgramId?: string;
    /** Enable debug logging */
    debug?: boolean;
    /** Solana network identifier (e.g., 'devnet', 'mainnet', 'localnet') */
    network?: string;
}
/**
 * Bytecode compilation options
 */
export interface CompilationOptions {
    /** Enable optimizations */
    optimize?: boolean;
    /** Target platform */
    target?: "vm" | "solana" | "debug";
    /** Include debug information */
    debug?: boolean;
    /** Maximum bytecode size */
    maxSize?: number;
    /** Optimization level for Five VM (only production mode) */
    optimizationLevel?: "production";
    /** Include metrics export */
    includeMetrics?: boolean;
    /** Metrics export format */
    metricsFormat?: "json" | "csv" | "toml";
    /** Error formatting mode */
    errorFormat?: "terminal" | "json" | "lsp";
    /** Enable comprehensive metrics collection */
    comprehensiveMetrics?: boolean;
    /** Optional metrics output path (handled by CLI) */
    metricsOutput?: string;
}
/**
 * Compilation result
 */
export interface CompilationResult {
    success: boolean;
    bytecode?: FiveBytecode;
    abi?: any;
    fiveFile?: FiveCompiledFile;
    disassembly?: string[];
    functionNames?: FunctionNameEntry[];
    publicFunctionNames?: string[];
    metadata?: {
        sourceSize: number;
        bytecodeSize: number;
        functions: FiveFunction[];
        compilationTime: number;
    };
    errors?: CompilationError[];
    metricsReport?: CompilationMetricsReport;
}
/**
 * Compilation error details
 */
export interface CompilationError {
    message: string;
    line?: number;
    column?: number;
    severity: "error" | "warning" | "info";
}
export interface CompilationMetricsReport {
    format: string;
    exported: string;
    detailed?: any;
}
/**
 * Script deployment options (client-agnostic)
 */
export interface DeploymentOptions {
    /** Custom script account as base58 string (optional - will generate PDA if not provided) */
    scriptAccount?: string;
    /** Additional rent lamports */
    extraLamports?: number;
}
/**
 * Script deployment result (deprecated - use SerializedDeployment)
 */
export interface DeploymentResult {
    success: boolean;
    scriptAccount: string;
    transactionSignature?: string;
    deploymentCost: number;
    bytecodeSize: number;
    logs?: string[];
    error?: string;
}
/**
 * Script execution options (client-agnostic)
 */
export interface ExecutionOptions {
    /** Function to execute (name or index) */
    function?: string | number;
    /** Function parameters */
    parameters?: any[];
    /** Account public keys as base58 strings */
    accounts?: string[];
    /** Maximum compute units */
    computeUnitLimit?: number;
    /** Additional compute unit price */
    computeUnitPrice?: number;
    /** Enable execution tracing */
    trace?: boolean;
}
/**
 * Encoded parameter for Five VM
 */
export interface EncodedParameter {
    type: number;
    value: any;
}
/**
 * Parameter encoding options
 */
export interface ParameterEncodingOptions {
    /** Function signature for type validation */
    functionSignature?: FiveFunction;
    /** ABI for type resolution */
    abi?: any;
    /** Enable strict type checking */
    strict?: boolean;
}
/**
 * Five SDK error types
 */
export declare class FiveSDKError extends Error {
    code: string;
    details?: any | undefined;
    constructor(message: string, code: string, details?: any | undefined);
}
/**
 * Compilation error
 */
export declare class CompilationSDKError extends FiveSDKError {
    constructor(message: string, details?: any);
}
/**
 * Execution error
 */
export declare class ExecutionSDKError extends FiveSDKError {
    constructor(message: string, details?: any);
}
/**
 * Deployment error
 */
export declare class DeploymentSDKError extends FiveSDKError {
    constructor(message: string, details?: any);
}
/**
 * Function name entry in metadata
 */
export interface FunctionNameEntry {
    /** Function name */
    name: string;
    /** Function index */
    function_index: number;
}
/**
 * Function name metadata section
 */
export interface FunctionNameMetadata {
    /** Section size in bytes */
    section_size: number;
    /** Array of function name entries */
    names: FunctionNameEntry[];
}
/**
 * Function name info for WASM export
 */
export interface FunctionNameInfo {
    /** Function name */
    name: string;
    /** Function index */
    index: number;
}
/**
 * Extended script metadata with function names
 */
export interface ExtendedScriptMetadata {
    /** Original metadata */
    metadata: any;
    /** Function names extracted from bytecode */
    functionNames?: FunctionNameEntry[];
}
/**
 * Parameter encoding error
 */
export declare class ParameterEncodingError extends FiveSDKError {
    constructor(message: string, details?: any);
}
//# sourceMappingURL=types.d.ts.map