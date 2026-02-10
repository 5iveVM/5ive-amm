/**
 * Five SDK: client-agnostic library for Five VM.
 */
// ==================== Core SDK Exports ====================
export * from './FiveSDK.js';
export * from './types.js';
export * from './validation/index.js';
export * from './utils/index.js';
export * from './project/toml.js';
export * from './project/config.js';
// Re-export specific sub-modules if needed
export * as accounts from './accounts/index.js';
// export * as compiler from './compiler/index.js'; // Broken: missing relative deps
export * as crypto from './crypto/index.js';
// ==================== Component Exports ====================
export { BytecodeCompiler } from './compiler/BytecodeCompiler.js';
export { ParameterEncoder } from './encoding/ParameterEncoder.js';
// ==================== Metadata and Account Fetching Exports ====================
export { ScriptMetadataParser, MetadataCache } from './metadata/index.js';
// ==================== Crypto Exports ====================
export { PDAUtils, Base58Utils, RentCalculator } from './crypto/index.js';
// ==================== Error Exports ====================
export { FiveSDKError, CompilationSDKError, ExecutionSDKError, DeploymentSDKError, ParameterEncodingError } from './types.js';
// ==================== Validation Exports ====================
export { InputValidator, ValidationError, ValidationErrorType, DEFAULT_VALIDATION_CONFIG, validator, validateInput, Validators } from './validation/index.js';
// ==================== Constants ====================
export { FIVE_VM_PROGRAM_ID } from './types.js';
// ==================== Convenience Functions ====================
/**
 * Quick script compilation helper (client-agnostic)
 */
export const compileScript = async (source, options) => {
    const { FiveSDK } = await import('./FiveSDK.js');
    // Validation is handled in FiveSDK.compile()
    return FiveSDK.compile(source, options);
};
/**
 * Quick local execution helper (perfect for testing)
 */
export const executeLocally = async (bytecode, functionName, parameters = [], options) => {
    const { FiveSDK } = await import('./FiveSDK.js');
    // Validation is handled in FiveSDK.executeLocally()
    return FiveSDK.executeLocally(bytecode, functionName, parameters, options);
};
/**
 * Quick compile and execute helper (one-step testing)
 */
export const compileAndExecuteLocally = async (source, functionName, parameters = [], options) => {
    const { FiveSDK } = await import('./FiveSDK.js');
    // Validation is handled in FiveSDK.compileAndExecuteLocally()
    return FiveSDK.compileAndExecuteLocally(source, functionName, parameters, options);
};
/**
 * Quick account fetching and deserialization helper
 */
export const fetchAccountAndDeserialize = async (accountAddress, connection, options) => {
    const { FiveSDK } = await import('./FiveSDK.js');
    return FiveSDK.fetchAccountAndDeserialize(accountAddress, connection, options);
};
/**
 * Quick batch account fetching helper
 */
export const fetchMultipleAccountsAndDeserialize = async (accountAddresses, connection, options) => {
    const { FiveSDK } = await import('./FiveSDK.js');
    return FiveSDK.fetchMultipleAccountsAndDeserialize(accountAddresses, connection, options);
};
/**
 * Quick instruction-parameter deserialization helper
 */
export const deserializeParameters = async (instructionData, expectedTypes = [], options) => {
    const { FiveSDK } = await import('./FiveSDK.js');
    return FiveSDK.deserializeParameters(instructionData, expectedTypes, options);
};
/**
 * Quick execution with state diff tracking helper
 */
export const executeWithStateDiff = async (scriptAccount, connection, signerKeypair, functionName, parameters = [], options) => {
    const { FiveSDK } = await import('./FiveSDK.js');
    return FiveSDK.executeWithStateDiff(scriptAccount, connection, signerKeypair, functionName, parameters, options);
};
// ==================== Default Export ====================
/**
 * Default export provides the main FiveSDK class
 */
export { FiveSDK as default };
//# sourceMappingURL=index.js.map
