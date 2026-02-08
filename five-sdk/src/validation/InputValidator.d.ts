/**
 * Five SDK input validation.
 * Protects against common injection and resource abuse cases.
 */
import { FiveSDKError } from '../types.js';
/**
 * Input validation configuration
 */
export interface ValidationConfig {
    /** Maximum source code size in bytes */
    maxSourceSize: number;
    /** Maximum bytecode size in bytes */
    maxBytecodeSize: number;
    /** Maximum parameter count */
    maxParameters: number;
    /** Maximum parameter value size */
    maxParameterSize: number;
    /** Maximum file path length */
    maxPathLength: number;
    /** Maximum account count */
    maxAccounts: number;
    /** Maximum string length */
    maxStringLength: number;
    /** Maximum array length */
    maxArrayLength: number;
    /** Allowed file extensions */
    allowedExtensions: string[];
    /** Path whitelist patterns */
    allowedPaths: RegExp[];
}
/**
 * Default validation configuration
 */
export declare const DEFAULT_VALIDATION_CONFIG: ValidationConfig;
/**
 * Validation error types
 */
export declare enum ValidationErrorType {
    INVALID_INPUT = "INVALID_INPUT",
    SIZE_EXCEEDED = "SIZE_EXCEEDED",
    TYPE_MISMATCH = "TYPE_MISMATCH",
    UNSAFE_PATH = "UNSAFE_PATH",
    RESOURCE_EXHAUSTION = "RESOURCE_EXHAUSTION",
    MALICIOUS_CONTENT = "MALICIOUS_CONTENT",
    ENCODING_ERROR = "ENCODING_ERROR"
}
/**
 * Input validation error
 */
export declare class ValidationError extends FiveSDKError {
    type: ValidationErrorType;
    field?: string | undefined;
    value?: any | undefined;
    constructor(message: string, type: ValidationErrorType, field?: string | undefined, value?: any | undefined);
}
/**
 * Input validator for Five SDK
 */
export declare class InputValidator {
    private config;
    constructor(config?: ValidationConfig);
    /**
     * Validate source code input
     */
    validateSourceCode(source: string, context?: string): void;
    /**
     * Validate bytecode input
     */
    validateBytecode(bytecode: Uint8Array, context?: string): void;
    /**
     * Validate file path
     */
    validateFilePath(path: string, context?: string): void;
    /**
     * Validate function parameters
     */
    validateParameters(parameters: any[], context?: string): void;
    /**
     * Validate individual parameter
     */
    validateParameter(parameter: any, context?: string): void;
    /**
     * Validate account addresses
     */
    validateAccounts(accounts: string[], context?: string): void;
    /**
     * Validate Base58 address
     */
    validateBase58Address(address: string, context?: string): void;
    /**
     * Validate function name or index
     */
    validateFunctionReference(functionRef: string | number, context?: string): void;
    /**
     * Validate options object
     */
    validateOptions(options: any, context?: string): void;
    /**
     * Validate string input
     */
    private validateString;
    /**
     * Validate number input (also exposed as public for external use)
     */
    validateNumber(value: number, context?: string): void;
    /**
     * Validate number input (private implementation)
     */
    private validateNumberPrivate;
    /**
     * Validate buffer input
     */
    private validateBuffer;
    /**
     * Validate array input
     */
    private validateArray;
    /**
     * Check for malicious patterns in source code
     */
    private containsMaliciousPatterns;
}
/**
 * Global validator instance
 */
export declare const validator: InputValidator;
/**
 * Validation decorators for class methods
 */
export declare function validateInput(validationFn: (args: any[]) => void): (target: any, propertyName: string, descriptor: PropertyDescriptor) => void;
/**
 * Common validation patterns
 */
export declare const Validators: {
    sourceCode: (source: string) => void;
    bytecode: (bytecode: Uint8Array) => void;
    filePath: (path: string) => void;
    parameters: (params: any[]) => void;
    accounts: (accounts: string[]) => void;
    functionRef: (ref: string | number) => void;
    options: (opts: any) => void;
};
//# sourceMappingURL=InputValidator.d.ts.map
