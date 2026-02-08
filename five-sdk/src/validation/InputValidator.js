/**
 * Five SDK input validation.
 * Protects against common injection and resource abuse cases.
 */
import { FiveSDKError } from '../types.js';
/**
 * Default validation configuration
 */
export const DEFAULT_VALIDATION_CONFIG = {
    maxSourceSize: 1024 * 1024, // 1MB
    maxBytecodeSize: 1024 * 1024, // 1MB
    maxParameters: 256,
    maxParameterSize: 64 * 1024, // 64KB per parameter
    maxPathLength: 1000,
    maxAccounts: 64,
    maxStringLength: 32 * 1024, // 32KB
    maxArrayLength: 10000,
    allowedExtensions: ['.v', '.five', '.bin'],
    allowedPaths: [
        /^[a-zA-Z0-9_\-\.\/]+$/, // Basic safe characters
        /^\.\.?\//, // Relative paths (blocked)
    ]
};
/**
 * Validation error types
 */
export var ValidationErrorType;
(function (ValidationErrorType) {
    ValidationErrorType["INVALID_INPUT"] = "INVALID_INPUT";
    ValidationErrorType["SIZE_EXCEEDED"] = "SIZE_EXCEEDED";
    ValidationErrorType["TYPE_MISMATCH"] = "TYPE_MISMATCH";
    ValidationErrorType["UNSAFE_PATH"] = "UNSAFE_PATH";
    ValidationErrorType["RESOURCE_EXHAUSTION"] = "RESOURCE_EXHAUSTION";
    ValidationErrorType["MALICIOUS_CONTENT"] = "MALICIOUS_CONTENT";
    ValidationErrorType["ENCODING_ERROR"] = "ENCODING_ERROR";
})(ValidationErrorType || (ValidationErrorType = {}));
/**
 * Input validation error
 */
export class ValidationError extends FiveSDKError {
    type;
    field;
    value;
    constructor(message, type, field, value) {
        super(message, 'VALIDATION_ERROR', { type, field, value });
        this.type = type;
        this.field = field;
        this.value = value;
        this.name = 'ValidationError';
    }
}
/**
 * Input validator for Five SDK
 */
export class InputValidator {
    config;
    constructor(config = DEFAULT_VALIDATION_CONFIG) {
        this.config = config;
    }
    /**
     * Validate source code input
     */
    validateSourceCode(source, context = 'source') {
        this.validateString(source, context, this.config.maxSourceSize);
        // Validate encoding (must be valid UTF-8)
        try {
            new TextEncoder().encode(source);
        }
        catch (error) {
            throw new ValidationError(`Source code contains invalid encoding`, ValidationErrorType.ENCODING_ERROR, context);
        }
    }
    /**
     * Validate bytecode input
     */
    validateBytecode(bytecode, context = 'bytecode') {
        this.validateBuffer(bytecode, context, this.config.maxBytecodeSize);
        // Basic bytecode structure validation
        if (bytecode.length < 8) {
            throw new ValidationError(`Bytecode too small: ${bytecode.length} bytes (minimum 8)`, ValidationErrorType.SIZE_EXCEEDED, context, bytecode.length);
        }
    }
    /**
     * Validate file path
     */
    validateFilePath(path, context = 'filePath') {
        this.validateString(path, context, this.config.maxPathLength);
        // Check for path traversal attacks
        if (path.includes('..') || path.includes('~') || path.startsWith('/')) {
            throw new ValidationError(`Unsafe file path: ${path}`, ValidationErrorType.UNSAFE_PATH, context, path);
        }
        // Validate allowed paths
        const isAllowed = this.config.allowedPaths.some(pattern => pattern.test(path));
        if (!isAllowed) {
            throw new ValidationError(`File path not allowed: ${path}`, ValidationErrorType.UNSAFE_PATH, context, path);
        }
        // Validate file extension
        const extension = path.substring(path.lastIndexOf('.'));
        if (extension && !this.config.allowedExtensions.includes(extension)) {
            throw new ValidationError(`File extension not allowed: ${extension}`, ValidationErrorType.UNSAFE_PATH, context, extension);
        }
    }
    /**
     * Validate function parameters
     */
    validateParameters(parameters, context = 'parameters') {
        if (!Array.isArray(parameters)) {
            throw new ValidationError(`Parameters must be an array`, ValidationErrorType.TYPE_MISMATCH, context, typeof parameters);
        }
        if (parameters.length > this.config.maxParameters) {
            throw new ValidationError(`Too many parameters: ${parameters.length} (max ${this.config.maxParameters})`, ValidationErrorType.SIZE_EXCEEDED, context, parameters.length);
        }
        parameters.forEach((param, index) => {
            this.validateParameter(param, `${context}[${index}]`);
        });
    }
    /**
     * Validate individual parameter
     */
    validateParameter(parameter, context = 'parameter') {
        if (parameter === null || parameter === undefined) {
            return; // Allow null/undefined parameters
        }
        const type = typeof parameter;
        switch (type) {
            case 'string':
                this.validateString(parameter, context, this.config.maxParameterSize);
                break;
            case 'number':
                this.validateNumberPrivate(parameter, context);
                break;
            case 'boolean':
                // Boolean is always valid
                break;
            case 'object':
                if (Array.isArray(parameter)) {
                    this.validateArray(parameter, context);
                }
                else if (parameter instanceof Uint8Array) {
                    this.validateBuffer(parameter, context, this.config.maxParameterSize);
                }
                else {
                    throw new ValidationError(`Unsupported parameter type: ${type}`, ValidationErrorType.TYPE_MISMATCH, context, type);
                }
                break;
            default:
                throw new ValidationError(`Unsupported parameter type: ${type}`, ValidationErrorType.TYPE_MISMATCH, context, type);
        }
    }
    /**
     * Validate account addresses
     */
    validateAccounts(accounts, context = 'accounts') {
        if (!Array.isArray(accounts)) {
            throw new ValidationError(`Accounts must be an array`, ValidationErrorType.TYPE_MISMATCH, context, typeof accounts);
        }
        if (accounts.length > this.config.maxAccounts) {
            throw new ValidationError(`Too many accounts: ${accounts.length} (max ${this.config.maxAccounts})`, ValidationErrorType.SIZE_EXCEEDED, context, accounts.length);
        }
        accounts.forEach((account, index) => {
            this.validateBase58Address(account, `${context}[${index}]`);
        });
    }
    /**
     * Validate Base58 address
     */
    validateBase58Address(address, context = 'address') {
        this.validateString(address, context, 100); // Solana addresses are ~44 chars
        // Solana address length validation (typically 32-44 characters)
        if (address.length < 32 || address.length > 44) {
            throw new ValidationError(`Invalid address length: ${address.length} (expected 32-44 characters)`, ValidationErrorType.INVALID_INPUT, context, address.length);
        }
        // Basic Base58 format validation (after length check)
        const base58Regex = /^[1-9A-HJ-NP-Za-km-z]+$/;
        if (!base58Regex.test(address)) {
            throw new ValidationError(`Invalid Base58 address format: ${address}`, ValidationErrorType.INVALID_INPUT, context, address);
        }
    }
    /**
     * Validate function name or index
     */
    validateFunctionReference(functionRef, context = 'function') {
        if (typeof functionRef === 'number') {
            this.validateNumberPrivate(functionRef, context);
            if (functionRef < 0 || !Number.isInteger(functionRef)) {
                throw new ValidationError(`Function index must be a non-negative integer: ${functionRef}`, ValidationErrorType.INVALID_INPUT, context, functionRef);
            }
        }
        else if (typeof functionRef === 'string') {
            this.validateString(functionRef, context, 256);
            // Function name validation (alphanumeric + underscore)
            const functionNameRegex = /^[a-zA-Z_][a-zA-Z0-9_]*$/;
            if (!functionNameRegex.test(functionRef)) {
                throw new ValidationError(`Invalid function name format: ${functionRef}`, ValidationErrorType.INVALID_INPUT, context, functionRef);
            }
        }
        else {
            throw new ValidationError(`Function reference must be string or number`, ValidationErrorType.TYPE_MISMATCH, context, typeof functionRef);
        }
    }
    /**
     * Validate options object
     */
    validateOptions(options, context = 'options') {
        if (options === null || options === undefined) {
            return; // Options are optional
        }
        if (typeof options !== 'object' || Array.isArray(options)) {
            throw new ValidationError(`Options must be an object`, ValidationErrorType.TYPE_MISMATCH, context, typeof options);
        }
        // Validate specific option fields
        if ('debug' in options && options.debug !== undefined && typeof options.debug !== 'boolean') {
            throw new ValidationError(`Options.debug must be boolean`, ValidationErrorType.TYPE_MISMATCH, `${context}.debug`, typeof options.debug);
        }
        if ('computeUnitLimit' in options && options.computeUnitLimit !== undefined) {
            this.validateNumberPrivate(options.computeUnitLimit, `${context}.computeUnitLimit`);
        }
        if ('maxSize' in options && options.maxSize !== undefined) {
            this.validateNumberPrivate(options.maxSize, `${context}.maxSize`);
        }
    }
    // ==================== Private Helper Methods ====================
    /**
     * Validate string input
     */
    validateString(value, context, maxLength) {
        if (typeof value !== 'string') {
            throw new ValidationError(`Expected string but got ${typeof value}`, ValidationErrorType.TYPE_MISMATCH, context, typeof value);
        }
        if (value.length > maxLength) {
            throw new ValidationError(`String too long: ${value.length} characters (max ${maxLength})`, ValidationErrorType.SIZE_EXCEEDED, context, value.length);
        }
    }
    /**
     * Validate number input (also exposed as public for external use)
     */
    validateNumber(value, context = 'number') {
        this.validateNumberPrivate(value, context);
    }
    /**
     * Validate number input (private implementation)
     */
    validateNumberPrivate(value, context) {
        if (typeof value !== 'number') {
            throw new ValidationError(`Expected number but got ${typeof value}`, ValidationErrorType.TYPE_MISMATCH, context, typeof value);
        }
        if (!Number.isFinite(value)) {
            throw new ValidationError(`Number must be finite: ${value}`, ValidationErrorType.INVALID_INPUT, context, value);
        }
    }
    /**
     * Validate buffer input
     */
    validateBuffer(buffer, context, maxSize) {
        if (!(buffer instanceof Uint8Array)) {
            throw new ValidationError(`Expected Uint8Array but got ${buffer?.constructor?.name || typeof buffer}`, ValidationErrorType.TYPE_MISMATCH, context, buffer?.constructor?.name || typeof buffer);
        }
        if (buffer.length > maxSize) {
            throw new ValidationError(`Buffer too large: ${buffer.length} bytes (max ${maxSize})`, ValidationErrorType.SIZE_EXCEEDED, context, buffer.length);
        }
    }
    /**
     * Validate array input
     */
    validateArray(array, context) {
        if (array.length > this.config.maxArrayLength) {
            throw new ValidationError(`Array too long: ${array.length} elements (max ${this.config.maxArrayLength})`, ValidationErrorType.SIZE_EXCEEDED, context, array.length);
        }
        array.forEach((item, index) => {
            this.validateParameter(item, `${context}[${index}]`);
        });
    }
    /**
     * Check for malicious patterns in source code
     */
    containsMaliciousPatterns(source) {
        const maliciousPatterns = [
            // Script injection patterns
            /<script/i,
            /javascript:/i,
            /vbscript:/i,
            /onload=/i,
            /onerror=/i,
            // File system access patterns
            /\.\.\/\.\.\//,
            /\/etc\/passwd/i,
            /\/proc\//i,
            /\\windows\\system32/i,
            // Network access patterns
            /fetch\(/i,
            /XMLHttpRequest/i,
            /require\(/i,
            /import\(/i,
            // Dangerous functions
            /eval\(/i,
            /Function\(/i,
            /setTimeout\(/i,
            /setInterval\(/i,
        ];
        return maliciousPatterns.some(pattern => pattern.test(source));
    }
}
/**
 * Global validator instance
 */
export const validator = new InputValidator();
/**
 * Validation decorators for class methods
 */
export function validateInput(validationFn) {
    return function (target, propertyName, descriptor) {
        const method = descriptor.value;
        descriptor.value = function (...args) {
            validationFn(args);
            return method.apply(this, args);
        };
    };
}
/**
 * Common validation patterns
 */
export const Validators = {
    sourceCode: (source) => validator.validateSourceCode(source),
    bytecode: (bytecode) => validator.validateBytecode(bytecode),
    filePath: (path) => validator.validateFilePath(path),
    parameters: (params) => validator.validateParameters(params),
    accounts: (accounts) => validator.validateAccounts(accounts),
    functionRef: (ref) => validator.validateFunctionReference(ref),
    options: (opts) => validator.validateOptions(opts)
};
//# sourceMappingURL=InputValidator.js.map
