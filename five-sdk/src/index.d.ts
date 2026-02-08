/**
 * Five SDK: client-agnostic library for Five VM.
 */
export * from './FiveSDK.js';
export * from './types.js';
export * from './validation/index.js';
export * from './utils/index.js';
export * from './project/toml.js';
export * from './project/config.js';
export * as accounts from './accounts/index.js';
export * as crypto from './crypto/index.js';
export type { FiveScriptSource, FiveBytecode, ScriptAccount, FiveFunction, FiveParameter, FiveType, FiveSDKConfig, CompilationOptions, CompilationResult, CompilationError, SerializedInstruction, SerializedDeployment, SerializedExecution, SerializableAccount, EncodedParameter, ParameterEncodingOptions } from './types.js';
export { BytecodeCompiler } from './compiler/BytecodeCompiler.js';
export { ParameterEncoder } from './encoding/ParameterEncoder.js';
export { ScriptMetadataParser, MetadataCache, type ScriptMetadata, type AccountData, type AccountFetcher, type ScriptABI, type FunctionDefinition, type ParameterDefinition } from './metadata/index.js';
export { PDAUtils, Base58Utils, RentCalculator } from './crypto/index.js';
export { FiveSDKError, CompilationSDKError, ExecutionSDKError, DeploymentSDKError, ParameterEncodingError } from './types.js';
export { InputValidator, ValidationError, ValidationErrorType, type ValidationConfig, DEFAULT_VALIDATION_CONFIG, validator, validateInput, Validators } from './validation/index.js';
export { FIVE_VM_PROGRAM_ID } from './types.js';
/**
 * Quick script compilation helper (client-agnostic)
 */
export declare const compileScript: (source: string, options?: import("./types.js").CompilationOptions) => Promise<import("./types.js").CompilationResult>;
/**
 * Quick local execution helper (perfect for testing)
 */
export declare const executeLocally: (bytecode: Uint8Array, functionName: string | number, parameters?: any[], options?: {
    debug?: boolean;
    trace?: boolean;
    computeUnitLimit?: number;
    accounts?: string[];
}) => Promise<{
    success: boolean;
    result?: any;
    logs?: string[];
    computeUnitsUsed?: number;
    executionTime?: number;
    error?: string;
    trace?: any[];
}>;
/**
 * Quick compile and execute helper (one-step testing)
 */
export declare const compileAndExecuteLocally: (source: string, functionName: string | number, parameters?: any[], options?: {
    debug?: boolean;
    trace?: boolean;
    optimize?: boolean;
    computeUnitLimit?: number;
    accounts?: string[];
}) => Promise<{
    success: boolean;
    compilationErrors: import("./types.js").CompilationError[] | undefined;
    error: string;
} | {
    compilation: import("./types.js").CompilationResult;
    bytecodeSize: number;
    functions: import("./types.js").FiveFunction[] | undefined;
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
 * Quick account fetching and VLE deserialization helper
 */
export declare const fetchAccountAndDeserializeVLE: (accountAddress: string, connection: any, options?: {
    debug?: boolean;
    parseMetadata?: boolean;
    validateVLE?: boolean;
}) => Promise<{
    success: boolean;
    accountInfo?: {
        address: string;
        owner: string;
        lamports: number;
        dataLength: number;
    };
    scriptMetadata?: import("./metadata/index.js").ScriptMetadata;
    rawBytecode?: Uint8Array;
    vleData?: {
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
 * Quick batch account fetching helper
 */
export declare const fetchMultipleAccountsAndDeserializeVLE: (accountAddresses: string[], connection: any, options?: {
    debug?: boolean;
    parseMetadata?: boolean;
    validateVLE?: boolean;
    batchSize?: number;
}) => Promise<Map<string, {
    success: boolean;
    accountInfo?: any;
    scriptMetadata?: import("./metadata/index.js").ScriptMetadata;
    rawBytecode?: Uint8Array;
    vleData?: any;
    error?: string;
    logs?: string[];
}>>;
/**
 * Quick VLE parameter deserialization helper
 */
export declare const deserializeVLEParameters: (instructionData: Uint8Array, expectedTypes?: string[], options?: {
    debug?: boolean;
}) => Promise<{
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
 * Quick execution with state diff tracking helper
 */
export declare const executeWithStateDiff: (scriptAccount: string, connection: any, signerKeypair: any, functionName: string | number, parameters?: any[], options?: {
    debug?: boolean;
    network?: string;
    computeUnitLimit?: number;
    trackGlobalFields?: boolean;
    additionalAccounts?: string[];
    includeVMState?: boolean;
}) => Promise<{
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
 * Default export provides the main FiveSDK class
 */
export { FiveSDK as default };
//# sourceMappingURL=index.d.ts.map
