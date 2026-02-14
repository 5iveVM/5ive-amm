/**
 * Five SDK Core Types
 *
 * Defines the core types and interfaces for the Five SDK with correct terminology:
 * - Five scripts (.v files) compile to bytecode (.bin files)
 * - Script accounts store bytecode on Solana
 * - Five VM Program executes scripts from script accounts
 * - Five VM is the virtual machine that executes bytecode
 */
// Core types only use minimal imports for base58 string handling
// No direct Solana client library dependencies
// ==================== Core Five VM Types ====================
/**
 * Five VM Program ID - the actual Solana program that executes Five bytecode
 */
// Updated to current localnet deployment
export const FIVE_VM_PROGRAM_ID = "4Qxf3pbCse2veUgZVMiAm3nWqJrYo2pT4suxHKMJdK1d";
// ==================== Error Types ====================
/**
 * Five SDK error types
 */
export class FiveSDKError extends Error {
    code;
    details;
    constructor(message, code, details) {
        super(message);
        this.code = code;
        this.details = details;
        this.name = "FiveSDKError";
    }
}
/**
 * Compilation error
 */
export class CompilationSDKError extends FiveSDKError {
    constructor(message, details) {
        super(message, "COMPILATION_ERROR", details);
        this.name = "CompilationSDKError";
    }
}
/**
 * Execution error
 */
export class ExecutionSDKError extends FiveSDKError {
    constructor(message, details) {
        super(message, "EXECUTION_ERROR", details);
        this.name = "ExecutionSDKError";
    }
}
/**
 * Deployment error
 */
export class DeploymentSDKError extends FiveSDKError {
    constructor(message, details) {
        super(message, "DEPLOYMENT_ERROR", details);
        this.name = "DeploymentSDKError";
    }
}
/**
 * Parameter encoding error
 */
export class ParameterEncodingError extends FiveSDKError {
    constructor(message, details) {
        super(message, "PARAMETER_ENCODING_ERROR", details);
        this.name = "ParameterEncodingError";
    }
}
//# sourceMappingURL=types.js.map
