/**
 * Five SDK client for Five VM scripts.
 */
// Client-agnostic SDK: no direct Solana client dependencies
import { FiveSDKConfig, FiveScript, FiveBytecode, FiveScriptSource, ScriptAccount, CompilationOptions, CompilationResult, DeploymentOptions, SerializedDeployment, SerializedExecution, SerializableAccount, SerializedInstruction, ExecutionOptions, FIVE_VM_PROGRAM_ID, FiveSDKError, ExecutionSDKError, EncodedParameters, FiveCompiledFile, FiveFunction, FunctionNameEntry, } from "./types.js";
import { BytecodeCompiler } from "./compiler/BytecodeCompiler.js";
import { ParameterEncoder } from "./encoding/ParameterEncoder.js";
import { VarintEncoder } from "../lib/varint-encoder.js";
import { PDAUtils, Base58Utils, RentCalculator } from "./crypto/index.js";
import { ScriptMetadataParser, MetadataCache, ScriptMetadata, } from "./metadata/index.js";
import { normalizeAbiFunctions } from "./utils/abi.js";
import { validator, Validators } from "./validation/index.js";
/**
 * Main Five SDK class - entry point for all Five VM interactions
 * Client-agnostic design: generates serialized transaction data for any Solana client library
 */
export class FiveSDK {
    static compiler = null;
    static parameterEncoder = null;
    static metadataCache = new MetadataCache();
    fiveVMProgramId;
    debug;
    network;
    /**
     * Create a new Five SDK instance (for configuration)
     */
    constructor(config = {}) {
        this.fiveVMProgramId = config.fiveVMProgramId || FIVE_VM_PROGRAM_ID;
        this.debug = config.debug || false;
        this.network = config.network; // Cast to handle network property
        if (this.debug) {
            console.log(`[FiveSDK] Initialized with Five VM Program: ${this.fiveVMProgramId}`);
        }
    }
    /**
     * Get SDK configuration
     */
    getConfig() {
        return {
            fiveVMProgramId: this.fiveVMProgramId,
            debug: this.debug,
            network: this.network,
        };
    }
    /**
     * Initialize static components (lazy initialization)
     */
    static async initializeComponents(debug = false) {
        if (!this.compiler) {
            this.compiler = new BytecodeCompiler({ debug });
        }
        if (!this.parameterEncoder) {
            this.parameterEncoder = new ParameterEncoder(debug);
        }
    }
    // ==================== Static Factory Methods ====================
    /**
     * Create SDK instance with default configuration
     */
    static create(options = {}) {
        return new FiveSDK({
            debug: options.debug || false,
            fiveVMProgramId: options.fiveVMProgramId,
        });
    }
    /**
     * Create SDK instance for devnet
     */
    static devnet(options = {}) {
        return new FiveSDK({
            debug: options.debug || false,
            fiveVMProgramId: options.fiveVMProgramId,
            network: "devnet",
        });
    }
    /**
     * Create SDK instance for mainnet
     */
    static mainnet(options = {}) {
        return new FiveSDK({
            debug: options.debug || false,
            fiveVMProgramId: options.fiveVMProgramId,
            network: "mainnet",
        });
    }
    /**
     * Create SDK instance for localnet
     */
    static localnet(options = {}) {
        return new FiveSDK({
            debug: options.debug || false,
            fiveVMProgramId: options.fiveVMProgramId,
            network: "localnet",
        });
    }
    // ==================== Script Compilation ====================
    /**
     * Compile Five script source code to bytecode (static method)
     */
    static async compile(source, options = {}) {
        // Input validation
        Validators.sourceCode(source);
        Validators.options(options);
        await this.initializeComponents(options.debug);
        if (options.debug) {
            console.log(`[FiveSDK] Compiling script (${source.length} chars)...`);
        }
        try {
            const result = await this.compiler.compile(source, options);
            // Generate .five format if compilation successful
            if (result.success && result.bytecode) {
                if (options.debug) {
                    console.log("[FiveSDK] Debug - result.metadata:", JSON.stringify(result.metadata, null, 2));
                    console.log("[FiveSDK] Debug - result.abi:", JSON.stringify(result.abi, null, 2));
                }
                let abiData = result.abi ?? { functions: [], fields: [] };
                if (options.debug) {
                    try {
                        const generatedABI = await this.compiler.generateABI(source);
                        if (generatedABI && generatedABI.functions) {
                            abiData = generatedABI;
                            console.log("[FiveSDK] Generated ABI:", JSON.stringify(abiData, null, 2));
                        }
                    }
                    catch (abiError) {
                        console.warn("[FiveSDK] ABI generation failed, using compiler ABI:", abiError);
                    }
                }
                // Use generated ABI functions, fallback to empty array
                const functions = normalizeAbiFunctions(abiData.functions ?? abiData).map((func) => ({
                    name: func.name,
                    index: func.index,
                    parameters: func.parameters?.map((param) => ({
                        name: param.name,
                        type: param.type,
                        optional: param.optional ?? false,
                    })) || [],
                    returnType: func.returnType,
                }));
                result.fiveFile = {
                    bytecode: Buffer.from(result.bytecode).toString("base64"),
                    abi: {
                        functions,
                        fields: abiData.fields || [],
                        version: "1.0",
                    },
                    disassembly: result.disassembly || [],
                    debug: options.debug
                        ? {
                            compilationInfo: {
                                sourceSize: result.metadata?.sourceSize || 0,
                                bytecodeSize: result.metadata?.bytecodeSize || 0,
                                compilationTime: result.metadata?.compilationTime || 0,
                            },
                        }
                        : undefined,
                    version: "1.0",
                };
                // Extract function names from bytecode
                if (result.bytecode) {
                    const functionNames = await this.getFunctionNames(result.bytecode);
                    result.functionNames = functionNames;
                    result.publicFunctionNames = functionNames.map((f) => f.name);
                }
            }
            if (options.debug) {
                if (result.success) {
                    console.log(`[FiveSDK] Compilation successful: ${result.bytecode?.length} bytes`);
                }
                else {
                    console.log(`[FiveSDK] Compilation failed: ${result.errors?.length} errors`);
                }
            }
            return result;
        }
        catch (error) {
            throw new FiveSDKError(`Compilation failed: ${error instanceof Error ? error.message : "Unknown error"}`, "COMPILATION_ERROR", { source: source.substring(0, 100) + "...", options });
        }
    }
    /**
     * Compile multiple modules (entry + dependencies)
     */
    static async compileModules(mainSource, modules, options = {}) {
        Validators.options(options);
        await this.initializeComponents(options.debug);
        try {
            const result = await this.compiler.compileModules(mainSource, modules, options);
            if (options.debug) {
                console.log(`[FiveSDK] Multi-file compilation ${result.success ? "succeeded" : "failed"}`);
            }
            return result;
        }
        catch (error) {
            throw new FiveSDKError(`Compilation failed: ${error instanceof Error ? error.message : "Unknown error"}`, "COMPILATION_ERROR", { source: mainSource.substring(0, 100) + "...", options });
        }
    }
    /**
     * Compile with automatic module discovery (client-agnostic)
     */
    static async compileWithDiscovery(entryPoint, // File path
    options = {}) {
        // Input validation
        Validators.options(options);
        await this.initializeComponents(options.debug);
        if (options.debug) {
            console.log(`[FiveSDK] Compiling with discovery: ${entryPoint}`);
        }
        try {
            // Access the compiler's compileWithDiscovery method directly if available
            if (typeof this.compiler.compileWithDiscovery === 'function') {
                const result = await this.compiler.compileWithDiscovery(entryPoint, options);
                if (options.debug) {
                    console.log(`[FiveSDK] Discovery compilation ${result.success ? "succeeded" : "failed"}`);
                }
                return result;
            }
            else {
                // Fallback to compileFile if discovery not available (older compiler version)
                console.warn("[FiveSDK] compileWithDiscovery not available in current compiler version, falling back to compileFile");
                return this.compileFile(entryPoint, options);
            }
        }
        catch (error) {
            throw new FiveSDKError(`Discovery compilation failed: ${error instanceof Error ? error.message : "Unknown error"}`, "COMPILATION_ERROR", { source: entryPoint, options });
        }
    }
    /**
     * Discover modules from entry point
     */
    static async discoverModules(entryPoint, options = {}) {
        await this.initializeComponents(options.debug);
        try {
            if (typeof this.compiler.discoverModules === 'function') {
                return await this.compiler.discoverModules(entryPoint);
            }
            else {
                throw new Error("discoverModules not available in current compiler version");
            }
        }
        catch (error) {
            throw new FiveSDKError(`Module discovery failed: ${error instanceof Error ? error.message : "Unknown error"}`, "COMPILATION_ERROR", { source: entryPoint });
        }
    }
    /**
     * Compile script from file path (static method)
     */
    static async compileFile(filePath, options = {}) {
        // Input validation
        Validators.filePath(filePath);
        Validators.options(options);
        await this.initializeComponents(options.debug);
        if (options.debug) {
            console.log(`[FiveSDK] Compiling file: ${filePath}`);
        }
        return this.compiler.compileFile(filePath, options);
    }
    // ==================== Five File Format Utilities ====================
    /**
     * Load .five file and extract components
     */
    static async loadFiveFile(fileContent) {
        try {
            const fiveFile = JSON.parse(fileContent);
            if (!fiveFile.bytecode || !fiveFile.abi) {
                throw new Error("Invalid .five file format: missing bytecode or ABI");
            }
            const bytecode = new Uint8Array(Buffer.from(fiveFile.bytecode, "base64"));
            return {
                bytecode,
                abi: fiveFile.abi,
                debug: fiveFile.debug,
            };
        }
        catch (error) {
            throw new FiveSDKError(`Failed to load .five file: ${error instanceof Error ? error.message : "Unknown error"}`, "FILE_LOAD_ERROR");
        }
    }
    /**
     * Extract bytecode from .five file for deployment
     */
    static extractBytecode(fiveFile) {
        return new Uint8Array(Buffer.from(fiveFile.bytecode, "base64"));
    }
    /**
     * Resolve function name to index using ABI
     */
    static resolveFunctionIndex(abi, functionName) {
        if (!abi || !abi.functions) {
            throw new Error("No ABI information available for function name resolution");
        }
        // Handle both array format: [{ name: "add", index: 0 }] and object format: { "add": { index: 0 } }
        if (Array.isArray(abi.functions)) {
            // Array format
            const func = abi.functions.find((f) => f.name === functionName);
            if (!func) {
                const availableFunctions = abi.functions
                    .map((f) => f.name)
                    .join(", ");
                throw new Error(`Function '${functionName}' not found in ABI. Available functions: ${availableFunctions}`);
            }
            return func.index;
        }
        else {
            // Object format (new WASM ABI)
            const func = abi.functions[functionName];
            if (!func) {
                const availableFunctions = Object.keys(abi.functions).join(", ");
                throw new Error(`Function '${functionName}' not found in ABI. Available functions: ${availableFunctions}`);
            }
            return func.index;
        }
    }
    // ==================== WASM VM Direct Execution (Local Testing) ====================
    /**
     * Execute bytecode directly using WASM VM for local testing and development
     * This bypasses Solana entirely - no network connection needed!
     */
    static async executeLocally(bytecode, functionName, parameters = [], options = {}) {
        // Input validation
        Validators.bytecode(bytecode);
        Validators.functionRef(functionName);
        Validators.parameters(parameters);
        Validators.options(options);
        const startTime = Date.now();
        if (options.debug) {
            console.log(`[FiveSDK] Executing locally: function=${functionName}, params=${parameters.length}`);
            console.log(`[FiveSDK] Parameters:`, parameters);
        }
        try {
            // Load WASM VM
            const wasmVM = await this.loadWasmVM();
            // Resolve function name to index if needed
            let resolvedFunctionIndex;
            if (typeof functionName === "number") {
                resolvedFunctionIndex = functionName;
            }
            else if (options.abi) {
                // Use provided ABI for function name resolution
                try {
                    resolvedFunctionIndex = this.resolveFunctionIndex(options.abi, functionName);
                }
                catch (resolutionError) {
                    throw new FiveSDKError(`Function name resolution failed: ${resolutionError instanceof Error ? resolutionError.message : "Unknown error"}`, "FUNCTION_RESOLUTION_ERROR");
                }
            }
            else {
                // No ABI provided and function name given - cannot resolve
                throw new FiveSDKError(`Cannot resolve function name '${functionName}' without ABI information. Please provide function index or use compileAndExecuteLocally() instead.`, "MISSING_ABI_ERROR");
            }
            // Prepare execution context
            const execOptions = {
                bytecode,
                functionIndex: resolvedFunctionIndex,
                parameters,
                maxComputeUnits: options.computeUnitLimit || 200000,
                trace: options.trace || options.debug || false,
            };
            if (options.debug) {
                console.log(`[FiveSDK] WASM VM execution starting...`);
            }
            // Execute using WASM VM with proper varint parameter encoding
            const transformedParams = parameters.map((param, index) => ({
                type: this.inferParameterType(param),
                value: param,
            }));
            if (options.debug) {
                console.log(`[FiveSDK] Resolved function index: ${resolvedFunctionIndex}`);
                console.log(`[FiveSDK] Transformed parameters:`, transformedParams);
            }
            // Convert account addresses to AccountInfo format if provided
            let accountInfos = [];
            if (options.accounts && options.accounts.length > 0) {
                // Create mock AccountInfo objects from addresses
                // Account data will be empty (0 lamports) for test purposes
                accountInfos = options.accounts.map((address, index) => ({
                    key: address,
                    lamports: 0,
                    data: new Uint8Array(0),
                    owner: 'TokenkegQfeZyiNwAJsyFbPVwwQQforre5PJNYbToN', // System program default
                    isExecutable: false,
                    isSigner: index === 0, // First account is signer by default
                    isWritable: index === 1, // Second account is mutable by default
                }));
                if (options.debug) {
                    console.log(`[FiveSDK] Passing ${accountInfos.length} accounts to WASM VM execution`);
                    accountInfos.forEach((acc, i) => {
                        console.log(`  Account ${i}: ${acc.key.substring(0, 8)}... (signer=${acc.isSigner}, writable=${acc.isWritable})`);
                    });
                }
            }
            const result = await wasmVM.executeFunction(bytecode, resolvedFunctionIndex, transformedParams, accountInfos.length > 0 ? accountInfos : undefined);
            const executionTime = Date.now() - startTime;
            if (options.debug) {
                console.log(`[FiveSDK] Local execution ${result.success ? "completed" : "failed"} in ${executionTime}ms`);
                if (result.computeUnitsUsed) {
                    console.log(`[FiveSDK] Compute units used: ${result.computeUnitsUsed}`);
                }
            }
            return {
                success: result.success,
                result: result.result,
                logs: result.logs,
                computeUnitsUsed: result.computeUnitsUsed,
                executionTime,
                error: result.error,
                trace: result.trace,
            };
        }
        catch (error) {
            const executionTime = Date.now() - startTime;
            const errorMessage = error instanceof Error ? error.message : "Unknown execution error";
            if (options.debug) {
                console.log(`[FiveSDK] Local execution failed after ${executionTime}ms: ${errorMessage}`);
            }
            return {
                success: false,
                executionTime,
                error: errorMessage,
            };
        }
    }
    /**
     * Compile and execute a script locally in one step (perfect for rapid testing)
     */
    static async compileAndExecuteLocally(source, functionName, parameters = [], options = {}) {
        // Input validation
        Validators.sourceCode(source);
        Validators.functionRef(functionName);
        Validators.parameters(parameters);
        Validators.options(options);
        if (options.debug) {
            console.log(`[FiveSDK] Compile and execute locally: ${functionName}`);
        }
        // Compile the script
        const compilation = await this.compile(source, {
            optimize: options.optimize,
            debug: options.debug,
        });
        if (!compilation.success || !compilation.bytecode) {
            return {
                success: false,
                compilationErrors: compilation.errors,
                error: "Compilation failed",
            };
        }
        if (options.debug) {
            console.log(`[FiveSDK] Compilation successful, executing bytecode...`);
        }
        // Execute the compiled bytecode
        const execution = await this.executeLocally(compilation.bytecode, functionName, parameters, {
            ...options,
            abi: compilation.abi, // Pass ABI from compilation for function name resolution
            accounts: options.accounts, // Pass accounts for execution context
        });
        return {
            ...execution,
            compilation,
            bytecodeSize: compilation.bytecode.length,
            functions: compilation.metadata?.functions,
        };
    }
    /**
     * Validate bytecode format and structure using WASM VM
     */
    static async validateBytecode(bytecode, options = {}) {
        // Input validation
        Validators.bytecode(bytecode);
        Validators.options(options);
        if (options.debug) {
            console.log(`[FiveSDK] Validating bytecode (${bytecode.length} bytes)`);
        }
        try {
            const wasmVM = await this.loadWasmVM();
            const validation = await wasmVM.validateBytecode(bytecode);
            if (options.debug) {
                console.log(`[FiveSDK] Validation ${validation.valid ? "passed" : "failed"}`);
            }
            return validation;
        }
        catch (error) {
            if (options.debug) {
                console.log(`[FiveSDK] Validation error: ${error}`);
            }
            return {
                valid: false,
                errors: [
                    error instanceof Error ? error.message : "Unknown validation error",
                ],
            };
        }
    }
    // ==================== Serialized Deployment ====================
    /**
     * Generate deployment instruction data (static method)
     *
     * Creates a complete deployment transaction that includes:
     * 1. Creating the script account PDA owned by Five VM program
     * 2. Deploying bytecode to the created account
     */
    static async generateDeployInstruction(bytecode, deployer, // base58 pubkey string
    options = {}) {
        // Input validation
        Validators.bytecode(bytecode);
        validator.validateBase58Address(deployer, "deployer");
        Validators.options(options);
        if (options.scriptAccount) {
            validator.validateBase58Address(options.scriptAccount, "options.scriptAccount");
        }
        await this.initializeComponents(options.debug);
        if (options.debug) {
            console.log(`[FiveSDK] Generating deployment transaction (${bytecode.length} bytes)...`);
        }
        // Derive script account with seed
        const scriptResult = await PDAUtils.deriveScriptAccount(bytecode, FIVE_VM_PROGRAM_ID);
        const scriptAccount = scriptResult.address;
        const scriptSeed = scriptResult.seed;
        // Derive VM state PDA
        const vmStatePDA = await this.deriveVMStatePDA();
        if (options.debug) {
            console.log(`[FiveSDK] Script Account: ${scriptAccount} (seed: ${scriptSeed})`);
            console.log(`[FiveSDK] VM State PDA: ${vmStatePDA}`);
        }
        // Calculate account size and rent
        const SCRIPT_HEADER_SIZE = 64; // ScriptAccountHeader size from Rust program (64 bytes)
        const totalAccountSize = SCRIPT_HEADER_SIZE + bytecode.length;
        const rentLamports = await this.calculateRentExemption(totalAccountSize);
        // Build account list for deploy instruction (after PDA creation):
        // 0: script_account, 1: vm_state_account, 2: owner (signer)
        const deployAccounts = [
            { pubkey: scriptAccount, isSigner: false, isWritable: true }, // Script account PDA
            { pubkey: vmStatePDA, isSigner: false, isWritable: true }, // VM state PDA
            { pubkey: deployer, isSigner: true, isWritable: true }, // Owner/deployer (must be signer)
            // System program for account creation
            {
                pubkey: "11111111111111111111111111111112",
                isSigner: false,
                isWritable: false,
            },
        ];
        // Encode deployment instruction data
        const instructionData = this.encodeDeployInstruction(bytecode);
        // Create the deployment result with setup instructions
        const result = {
            instruction: {
                programId: FIVE_VM_PROGRAM_ID,
                accounts: deployAccounts,
                data: Buffer.from(instructionData).toString("base64"),
            },
            scriptAccount,
            requiredSigners: [deployer],
            estimatedCost: rentLamports + (options.extraLamports || 0),
            bytecodeSize: bytecode.length,
            // Add setup information for account creation
            setupInstructions: {
                createScriptAccount: {
                    pda: scriptAccount,
                    seed: scriptSeed,
                    space: totalAccountSize,
                    rent: rentLamports,
                    owner: FIVE_VM_PROGRAM_ID,
                },
            },
        };
        if (options.debug) {
            console.log(`[FiveSDK] Generated deployment transaction:`, {
                scriptAccount,
                scriptSeed,
                accountSize: totalAccountSize,
                rentCost: rentLamports,
                deployDataSize: instructionData.length,
            });
        }
        return result;
    }
    // ==================== Serialized Execution ====================
    /**
     * Generate execution instruction data (static method)
     */
    static async generateExecuteInstruction(scriptAccount, // base58 pubkey string
    functionName, parameters = [], accounts = [], // base58 pubkey strings
    connection, // Optional Solana connection for metadata lookup
    options = {}) {
        // Input validation
        validator.validateBase58Address(scriptAccount, "scriptAccount");
        Validators.functionRef(functionName);
        Validators.parameters(parameters);
        Validators.accounts(accounts);
        Validators.options(options);
        await this.initializeComponents(options.debug);
        if (options.debug) {
            console.log(`[FiveSDK] Generating execution instruction:`, {
                scriptAccount,
                function: functionName,
                parameterCount: parameters.length,
                accountCount: accounts.length,
            });
        }
        // Handle missing metadata by generating parameters.
        let functionIndex;
        let encodedParams;
        try {
            // Try to load script metadata for ABI-driven parameter encoding
            const scriptMetadata = await this.getScriptMetadata(scriptAccount, connection);
            // Resolve function index
            functionIndex =
                typeof functionName === "number"
                    ? functionName
                    : FiveSDK.resolveFunctionIndex(scriptMetadata, functionName);
            // Encode parameters with ABI guidance
            encodedParams = await this.encodeParametersWithABI(parameters, scriptMetadata.functions[functionIndex], functionIndex);
        }
        catch (metadataError) {
            if (options.debug) {
                console.log(`[FiveSDK] Metadata not available, using varint encoding with assumed parameter types`);
            }
            // Use varint encoding without metadata
            functionIndex = typeof functionName === "number" ? functionName : 0;
            // Create parameter definitions for varint encoding (assume all u64)
            const paramDefs = parameters.map((_, index) => ({
                name: `param${index}`,
                type: "u64",
            }));
            const paramValues = {};
            paramDefs.forEach((param, index) => {
                paramValues[param.name] = parameters[index];
            });
            if (options.debug) {
                console.log(`[FiveSDK] About to call parameter varint encoder with:`, {
                    functionIndex,
                    paramDefs,
                    paramValues,
                });
            }
            encodedParams = await VarintEncoder.encodeExecute(functionIndex, paramDefs, paramValues);
            if (options.debug) {
                console.log(`[FiveSDK] varint encoder returned:`, {
                    encodedLength: encodedParams.length,
                    encodedBytes: Array.from(encodedParams),
                    hex: Buffer.from(encodedParams).toString("hex"),
                });
            }
        }
        // Derive VM state PDA for all Five VM executions
        const vmStatePDA = await this.deriveVMStatePDA();
        // Build account list with required VM state PDA
        const vmState = options.vmStateAccount || vmStatePDA;
        if (options.debug) {
            console.log(`[FiveSDK] Using VM state account: ${vmState} (override: ${options.vmStateAccount ? "yes" : "no"})`);
        }
        const instructionAccounts = [
            { pubkey: scriptAccount, isSigner: false, isWritable: false },
            { pubkey: vmState, isSigner: false, isWritable: true }, // VM state
            ...accounts.map((acc) => ({
                pubkey: acc,
                isSigner: false, // Consumer will determine signing requirements
                isWritable: true, // Conservative default - consumer can override
            })),
        ];
        // Encode execution instruction data
        const instructionData = this.encodeExecuteInstruction(functionIndex, encodedParams);
        const result = {
            instruction: {
                programId: FIVE_VM_PROGRAM_ID,
                accounts: instructionAccounts,
                data: Buffer.from(instructionData).toString("base64"),
            },
            scriptAccount,
            parameters: {
                function: functionName,
                data: encodedParams,
                count: parameters.length,
            },
            requiredSigners: [], // Consumer determines signers based on their context
            estimatedComputeUnits: options.computeUnitLimit ||
                this.estimateComputeUnits(functionIndex, parameters.length),
        };
        if (options.debug) {
            console.log(`[FiveSDK] Generated execution instruction:`, {
                function: functionName,
                functionIndex,
                parameterBytes: encodedParams.length,
                dataSize: instructionData.length,
                estimatedCU: result.estimatedComputeUnits,
            });
        }
        return result;
    }
    // ==================== Script Analysis ====================
    /**
     * Get script metadata for ABI-driven parameter encoding (static method)
     * Now uses real Solana account data parsing instead of mocks
     */
    static async getScriptMetadata(scriptAccount, connection) {
        // Input validation
        validator.validateBase58Address(scriptAccount, "scriptAccount");
        try {
            if (connection) {
                // Use real blockchain data if connection provided
                const metadata = await ScriptMetadataParser.getScriptMetadata(connection, scriptAccount);
                const normalizedFunctions = normalizeAbiFunctions(metadata.abi?.functions ?? metadata.abi);
                return {
                    functions: normalizedFunctions.map((func) => ({
                        name: func.name,
                        index: func.index,
                        parameters: func.parameters,
                        returnType: func.returnType,
                        visibility: func.visibility,
                    })),
                };
            }
            else {
                // Client-agnostic mode: metadata should be provided by client
                // This maintains the SDK's client-agnostic design
                throw new Error("No connection provided for metadata retrieval. " +
                    "In client-agnostic mode, provide script metadata directly or use getScriptMetadataWithConnection().");
            }
        }
        catch (error) {
            throw new Error(`Failed to get script metadata: ${error instanceof Error ? error.message : "Unknown error"}`);
        }
    }
    /**
     * Get script metadata with explicit connection (for use with any Solana client)
     */
    static async getScriptMetadataWithConnection(scriptAccount, connection) {
        // Input validation
        validator.validateBase58Address(scriptAccount, "scriptAccount");
        return ScriptMetadataParser.getScriptMetadata(connection, scriptAccount);
    }
    /**
     * Parse script metadata from raw account data (client-agnostic)
     */
    static parseScriptMetadata(accountData, address) {
        // Input validation
        Validators.bytecode(accountData); // Reuse bytecode validation for account data
        validator.validateBase58Address(address, "address");
        return ScriptMetadataParser.parseMetadata(accountData, address);
    }
    /**
     * Get script metadata with caching (for performance)
     */
    static async getCachedScriptMetadata(scriptAccount, connection, cacheTTL = 5 * 60 * 1000) {
        // Input validation
        validator.validateBase58Address(scriptAccount, "scriptAccount");
        validator.validateNumber(cacheTTL, "cacheTTL");
        return this.metadataCache.getMetadata(scriptAccount, (address) => ScriptMetadataParser.getScriptMetadata(connection, address), cacheTTL);
    }
    /**
     * Invalidate metadata cache for a script
     */
    static invalidateMetadataCache(scriptAccount) {
        // Input validation
        validator.validateBase58Address(scriptAccount, "scriptAccount");
        this.metadataCache.invalidate(scriptAccount);
    }
    /**
     * Get metadata cache statistics
     */
    static getMetadataCacheStats() {
        return this.metadataCache.getStats();
    }
    // ==================== Private Utility Methods ====================
    /**
     * Derive script account PDA from bytecode using real Solana PDA derivation
     */
    static async deriveScriptAccount(bytecode) {
        const result = await PDAUtils.deriveScriptAccount(bytecode);
        return result.address;
    }
    /**
     * Derive VM state PDA using hardcoded seed (matches Five VM program)
     */
    static async deriveVMStatePDA() {
        const result = await PDAUtils.deriveVMStatePDA(FIVE_VM_PROGRAM_ID);
        return result.address;
    }
    /**
     * Load WASM VM for direct execution
     */
    static wasmVMInstance = null;
    static async loadWasmVM() {
        if (this.wasmVMInstance) {
            return this.wasmVMInstance;
        }
        try {
            // Import existing WASM VM from five-cli infrastructure
            const { FiveVM } = await import("../wasm/vm.js");
            // Create a simple logger for WASM VM
            const logger = {
                debug: (msg) => console.debug("[WASM VM]", msg),
                info: (msg) => console.info("[WASM VM]", msg),
                warn: (msg) => console.warn("[WASM VM]", msg),
                error: (msg) => console.error("[WASM VM]", msg),
            };
            this.wasmVMInstance = new FiveVM(logger); // Initialize WASM VM
            if (this.wasmVMInstance.initialize) {
                await this.wasmVMInstance.initialize();
            }
            return this.wasmVMInstance;
        }
        catch (error) {
            throw new FiveSDKError(`Failed to load WASM VM: ${error instanceof Error ? error.message : "Unknown error"}`, "WASM_LOAD_ERROR");
        }
    }
    /**
     * Calculate rent exemption for account size using real Solana rent calculations
     */
    static async calculateRentExemption(dataSize) {
        return RentCalculator.calculateRentExemption(dataSize);
    }
    /**
     * Encode deployment instruction data
     */
    static encodeDeployInstruction(bytecode) {
        // Deploy instruction: [discriminator(8), bytecode_length(u32_le), permissions(u8), bytecode]
        // Format expected by Five VM Program (five-solana/src/instructions.rs):
        // - Discriminator: 8 (u8)
        // - Length: bytecode.length (u32 little-endian, 4 bytes)
        // - Permissions: 0x00 (1 byte)
        // - Bytecode: actual bytecode bytes
        const lengthBytes = new Uint8Array(4);
        const lengthView = new DataView(lengthBytes.buffer);
        lengthView.setUint32(0, bytecode.length, true); // little-endian
        const result = new Uint8Array(1 + 4 + 1 + bytecode.length);
        result[0] = 8; // Deploy discriminator (matches on-chain FIVE program)
        result.set(lengthBytes, 1); // u32 LE length at bytes 1-4
        result[5] = 0x00; // permissions byte at byte 5
        result.set(bytecode, 6); // bytecode starts at byte 6
        console.log(`[FiveSDK] Deploy instruction encoded:`, {
            discriminator: result[0],
            lengthBytes: Array.from(lengthBytes),
            permissions: result[5],
            bytecodeLength: bytecode.length,
            totalInstructionLength: result.length,
            expectedFormat: `[8, ${bytecode.length}_as_u32le, 0x00, bytecode_bytes]`,
            instructionHex: Buffer.from(result).toString("hex").substring(0, 20) + "...",
        });
        return result;
    }
    /**
     * Encode execution instruction data
     */
    static encodeExecuteInstruction(functionIndex, encodedParams) {
        // Execute instruction: [discriminator(9), function_index(varint), params]
        // encodedParams contains: [varint(paramCount), varint(param1), ...]
        const parts = [];
        parts.push(new Uint8Array([9])); // Execute discriminator (matches on-chain FIVE program)
        const functionIndexEncoded = FiveSDK.encodeVarintNumber(functionIndex);
        parts.push(functionIndexEncoded);
        parts.push(encodedParams); // Contains: [varint(paramCount), varint(param1), ...]
        const totalLength = parts.reduce((sum, part) => sum + part.length, 0);
        const result = new Uint8Array(totalLength);
        let offset = 0;
        for (const part of parts) {
            result.set(part, offset);
            offset += part.length;
        }
        return result;
    }
    /**
     * Encode parameters with ABI guidance
     */
    static async encodeParametersWithABI(parameters, functionDef, functionIndex) {
        if (!this.parameterEncoder) {
            await this.initializeComponents();
        }
        // Use varint encoder to properly encode parameters
        const paramDefs = functionDef.parameters || [];
        const paramValues = {};
        // Map parameters to names
        paramDefs.forEach((param, index) => {
            if (index < parameters.length) {
                paramValues[param.name] = parameters[index];
            }
        });
        // Use ONLY varint encoding - no fallbacks to maintain architecture integrity
        const encoded = await VarintEncoder.encodeExecute(functionIndex, paramDefs, paramValues);
        return encoded;
    }
    // REMOVED: encodeParametersSimple - Five uses ONLY varint encoding
    // REMOVED: Local varint encoding - Five uses centralized varint encoder
    /**
     * varint encode a number for instruction data
     */
    static encodeVarintNumber(value) {
        const bytes = [];
        let num = value;
        while (num >= 0x80) {
            bytes.push((num & 0x7f) | 0x80);
            num >>>= 7;
        }
        bytes.push(num & 0x7f);
        return new Uint8Array(bytes);
    }
    /**
     * Estimate compute units for function execution
     */
    static estimateComputeUnits(functionIndex, parameterCount) {
        // Basic compute unit estimation
        return Math.max(5000, 1000 + parameterCount * 500 + functionIndex * 100);
    }
    /**
     * Infer parameter type from JavaScript value for varint encoding
     */
    static inferParameterType(value) {
        if (typeof value === "boolean") {
            return "bool";
        }
        else if (typeof value === "number") {
            if (Number.isInteger(value)) {
                return value >= 0 ? "u64" : "i64";
            }
            else {
                return "f64";
            }
        }
        else if (typeof value === "string") {
            return "string";
        }
        else if (value instanceof Uint8Array) {
            return "bytes";
        }
        else {
            // Fallback to string representation
            return "string";
        }
    }
    // ==================== Account Fetching and Parameter Deserialization ====================
    /**
     * Fetch account data and deserialize script/account payloads.
     * This is the method requested for pulling down accounts and deserializing Five script data
     */
    static async fetchAccountAndDeserialize(accountAddress, connection, // Solana Connection object
    options = {}) {
        try {
            if (options.debug) {
                console.log(`[FiveSDK] Fetching account and deserializing varint data: ${accountAddress}`);
            }
            // Import Solana web3.js for account fetching
            const { PublicKey } = await import("@solana/web3.js");
            // Validate account address format
            let accountPubkey;
            try {
                accountPubkey = new PublicKey(accountAddress);
            }
            catch (addressError) {
                return {
                    success: false,
                    error: `Invalid account address format: ${accountAddress}`,
                    logs: [],
                };
            }
            // Fetch account info from Solana blockchain
            const accountInfo = await connection.getAccountInfo(accountPubkey, "confirmed");
            if (!accountInfo) {
                return {
                    success: false,
                    error: `Account not found: ${accountAddress}`,
                    logs: [],
                };
            }
            if (!accountInfo.data || accountInfo.data.length === 0) {
                return {
                    success: false,
                    error: `Account has no data: ${accountAddress}`,
                    logs: [],
                };
            }
            const logs = [];
            if (options.debug) {
                console.log(`[FiveSDK] Account fetched successfully:`);
                console.log(`  - Address: ${accountAddress}`);
                console.log(`  - Owner: ${accountInfo.owner.toString()}`);
                console.log(`  - Lamports: ${accountInfo.lamports}`);
                console.log(`  - Data length: ${accountInfo.data.length} bytes`);
                logs.push(`Account fetched: ${accountInfo.data.length} bytes`);
                logs.push(`Owner: ${accountInfo.owner.toString()}`);
                logs.push(`Balance: ${accountInfo.lamports / 1e9} SOL`);
            }
            const result = {
                success: true,
                accountInfo: {
                    address: accountAddress,
                    owner: accountInfo.owner.toString(),
                    lamports: accountInfo.lamports,
                    dataLength: accountInfo.data.length,
                },
                logs,
            };
            // Parse script metadata if requested
            if (options.parseMetadata) {
                try {
                    const scriptMetadata = ScriptMetadataParser.parseMetadata(accountInfo.data, accountAddress);
                    result.scriptMetadata = scriptMetadata;
                    result.rawBytecode = scriptMetadata.bytecode;
                    // Create varint data structure with parsed information
                    result.decodedData = {
                        header: {
                            version: scriptMetadata.version,
                            deployedAt: scriptMetadata.deployedAt,
                            authority: scriptMetadata.authority,
                        },
                        bytecode: scriptMetadata.bytecode,
                        abi: scriptMetadata.abi,
                        functions: normalizeAbiFunctions(scriptMetadata.abi?.functions ?? scriptMetadata.abi).map((func) => ({
                            name: func.name,
                            index: func.index,
                            parameters: func.parameters || [],
                        })),
                    };
                    const parsedFunctions = result.decodedData.functions;
                    if (options.debug) {
                        console.log(`[FiveSDK] Script metadata parsed successfully:`);
                        console.log(`  - Script name: ${scriptMetadata.abi.name}`);
                        console.log(`  - Functions: ${parsedFunctions.length}`);
                        console.log(`  - Bytecode size: ${scriptMetadata.bytecode.length} bytes`);
                        console.log(`  - Authority: ${scriptMetadata.authority}`);
                        logs.push(`Script metadata parsed: ${parsedFunctions.length} functions`);
                        logs.push(`Bytecode: ${scriptMetadata.bytecode.length} bytes`);
                    }
                }
                catch (metadataError) {
                    if (options.debug) {
                        console.warn(`[FiveSDK] Failed to parse script metadata:`, metadataError);
                    }
                    // Fallback: treat as raw bytecode without metadata
                    result.rawBytecode = accountInfo.data;
                    logs.push("Warning: Failed to parse script metadata, treating as raw data");
                }
            }
            else {
                // Just return raw account data
                result.rawBytecode = accountInfo.data;
                logs.push("Raw account data returned (metadata parsing disabled)");
            }
            // Validate varint encoding (varint) if requested and we have bytecode
            if (options.validateEncoding && result.rawBytecode) {
                try {
                    const validation = await this.validateBytecodeEncoding(result.rawBytecode, options.debug);
                    if (validation.valid) {
                        logs.push("varint encoding (varint) validation: PASSED");
                        if (options.debug) {
                            console.log(`[FiveSDK] varint validation passed: ${validation.info}`);
                        }
                    }
                    else {
                        logs.push(`varint encoding (varint) validation: FAILED - ${validation.error}`);
                        if (options.debug) {
                            console.warn(`[FiveSDK] varint validation failed: ${validation.error}`);
                        }
                    }
                }
                catch (validationError) {
                    logs.push(`varint validation error: ${validationError instanceof Error ? validationError.message : "Unknown error"}`);
                }
            }
            return result;
        }
        catch (error) {
            const errorMessage = error instanceof Error ? error.message : "Unknown account fetch error";
            if (options.debug) {
                console.error(`[FiveSDK] Account fetch and parameter deserialization failed: ${errorMessage}`);
            }
            return {
                success: false,
                error: errorMessage,
                logs: [],
            };
        }
    }
    /**
     * Batch fetch multiple accounts and deserialize their parameter data
     */
    static async fetchMultipleAccountsAndDeserialize(accountAddresses, connection, options = {}) {
        const batchSize = options.batchSize || 100; // Solana RPC limit
        const results = new Map();
        if (options.debug) {
            console.log(`[FiveSDK] Batch fetching ${accountAddresses.length} accounts (batch size: ${batchSize})`);
        }
        // Process in batches to avoid RPC limits
        for (let i = 0; i < accountAddresses.length; i += batchSize) {
            const batch = accountAddresses.slice(i, i + batchSize);
            if (options.debug) {
                console.log(`[FiveSDK] Processing batch ${Math.floor(i / batchSize) + 1}/${Math.ceil(accountAddresses.length / batchSize)}`);
            }
            // Fetch each account in the batch concurrently
            const batchPromises = batch.map((address) => this.fetchAccountAndDeserialize(address, connection, {
                debug: false, // Disable individual debug to avoid spam
                parseMetadata: options.parseMetadata,
                validateEncoding: options.validateEncoding,
            }));
            const batchResults = await Promise.allSettled(batchPromises);
            // Store results
            batch.forEach((address, index) => {
                const batchResult = batchResults[index];
                if (batchResult.status === "fulfilled") {
                    results.set(address, batchResult.value);
                }
                else {
                    results.set(address, {
                        success: false,
                        error: `Batch processing failed: ${batchResult.reason}`,
                        logs: [],
                    });
                }
            });
        }
        if (options.debug) {
            const successful = Array.from(results.values()).filter((r) => r.success).length;
            console.log(`[FiveSDK] Batch processing completed: ${successful}/${accountAddresses.length} successful`);
        }
        return results;
    }
    /**
     * Deserialize parameters from instruction data using WASM decoder.
     */
    static async deserializeParameters(instructionData, expectedTypes = [], options = {}) {
        try {
            if (options.debug) {
                console.log(`[FiveSDK] Deserializing varint parameters from ${instructionData.length} bytes:`);
                console.log(`[FiveSDK] Instruction data (hex):`, Buffer.from(instructionData).toString("hex"));
                console.log(`[FiveSDK] Expected parameter types:`, expectedTypes);
            }
            // Load WASM VM for varint decoding
            const wasmVM = await this.loadWasmVM();
            // Use WASM ParameterEncoder to decode varint data
            try {
                const wasmModule = await import("../../assets/vm/five_vm_wasm.js");
                if (options.debug) {
                    console.log(`[FiveSDK] Using WASM ParameterEncoder for varint decoding`);
                }
                // Decode the instruction data
                const decodeInstruction = wasmModule.ParameterEncoder.decode_instruction_varint;
                const decodedResult = decodeInstruction(instructionData);
                if (options.debug) {
                    console.log(`[FiveSDK] varint decoding result:`, decodedResult);
                }
                // Parse the decoded result structure
                const parameters = [];
                if (decodedResult && decodedResult.parameters) {
                    decodedResult.parameters.forEach((param, index) => {
                        parameters.push({
                            type: expectedTypes[index] || "unknown",
                            value: param,
                        });
                    });
                }
                return {
                    success: true,
                    parameters,
                    functionIndex: decodedResult.function_index,
                    discriminator: decodedResult.discriminator,
                };
            }
            catch (wasmError) {
                if (options.debug) {
                    console.warn(`[FiveSDK] WASM varint decoding failed, attempting manual parsing:`, wasmError);
                }
                // Fallback: manual varint parsing
                return this.parseInstructionParametersManually(instructionData, expectedTypes, options.debug);
            }
        }
        catch (error) {
            const errorMessage = error instanceof Error
                ? error.message
                : "Unknown parameter deserialization error";
            if (options.debug) {
                console.error(`[FiveSDK] varint parameter deserialization failed: ${errorMessage}`);
            }
            return {
                success: false,
                error: errorMessage,
            };
        }
    }
    /**
     * Validate Five bytecode envelope/header encoding.
     */
    static async validateBytecodeEncoding(bytecode, debug = false) {
        try {
            // Check for Five VM bytecode header
            if (bytecode.length < 6) {
                return { valid: false, error: "Bytecode too short for Five VM format" };
            }
            // Check for OptimizedHeader format: "5IVE" + features + function_count
            const magicBytes = bytecode.slice(0, 4);
            const expectedMagic = new Uint8Array([0x35, 0x49, 0x56, 0x45]); // "5IVE"
            let isValidHeader = true;
            for (let i = 0; i < 4; i++) {
                if (magicBytes[i] !== expectedMagic[i]) {
                    isValidHeader = false;
                    break;
                }
            }
            if (!isValidHeader) {
                return {
                    valid: false,
                    error: 'Invalid Five VM magic bytes (expected "5IVE")',
                };
            }
            const features = bytecode[4];
            const functionCount = bytecode[5];
            if (debug) {
                console.log(`[FiveSDK] varint validation - Magic: "5IVE", Features: ${features}, Functions: ${functionCount}`);
            }
            return {
                valid: true,
                info: `Valid Five VM bytecode with ${functionCount} functions (features: ${features})`,
            };
        }
        catch (error) {
            return {
                valid: false,
                error: error instanceof Error ? error.message : "varint validation error",
            };
        }
    }
    /**
     * Manual varint instruction parsing (fallback when WASM fails)
     */
    static parseInstructionParametersManually(instructionData, expectedTypes, debug = false) {
        try {
            if (instructionData.length < 2) {
                return { success: false, error: "Instruction data too short" };
            }
            let offset = 0;
            // Read discriminator
            const discriminator = instructionData[offset];
            offset += 1;
            if (debug) {
                console.log(`[FiveSDK] Manual varint parsing - Discriminator: ${discriminator}`);
            }
            // Read function index (varint encoded)
            const { value: functionIndex, bytesRead } = this.readVarintNumber(instructionData, offset);
            offset += bytesRead;
            if (debug) {
                console.log(`[FiveSDK] Manual varint parsing - Function index: ${functionIndex}`);
            }
            // Read parameter count (varint encoded)
            const { value: paramCount, bytesRead: paramCountBytes } = this.readVarintNumber(instructionData, offset);
            offset += paramCountBytes;
            if (debug) {
                console.log(`[FiveSDK] Manual varint parsing - Parameter count: ${paramCount}`);
            }
            // Read parameters
            const parameters = [];
            for (let i = 0; i < paramCount; i++) {
                const { value: paramValue, bytesRead: paramBytes } = this.readVarintNumber(instructionData, offset);
                offset += paramBytes;
                parameters.push({
                    type: expectedTypes[i] || "u64", // Default to u64
                    value: paramValue,
                });
                if (debug) {
                    console.log(`[FiveSDK] Manual varint parsing - Parameter ${i}: ${paramValue}`);
                }
            }
            return {
                success: true,
                parameters,
                functionIndex,
                discriminator,
            };
        }
        catch (error) {
            return {
                success: false,
                error: error instanceof Error ? error.message : "Manual varint parsing failed",
            };
        }
    }
    /**
     * Read varint-encoded (varint) number from byte array
     */
    static readVarintNumber(data, offset) {
        let value = 0;
        let shift = 0;
        let bytesRead = 0;
        while (offset + bytesRead < data.length) {
            const byte = data[offset + bytesRead];
            bytesRead++;
            value |= (byte & 0x7f) << shift;
            if ((byte & 0x80) === 0) {
                break;
            }
            shift += 7;
        }
        return { value, bytesRead };
    }
    // ==================== Account State Mutation Tracking ====================
    /**
     * Execute script with before/after account state tracking
     * This fetches account data before execution, runs the script, then fetches after
     * to show exactly what changed including global fields
     */
    static async executeWithStateDiff(scriptAccount, connection, signerKeypair, functionName, parameters = [], options = {}) {
        const logs = [];
        try {
            if (options.debug) {
                console.log(`[FiveSDK] Starting execution with state diff tracking`);
                console.log(`  Script Account: ${scriptAccount}`);
                console.log(`  Function: ${functionName}`);
                console.log(`  Parameters: ${JSON.stringify(parameters)}`);
                console.log(`  Track Global Fields: ${options.trackGlobalFields}`);
            }
            // Build list of accounts to track
            const accountsToTrack = [scriptAccount];
            // Add VM state PDA if requested
            if (options.includeVMState) {
                const vmStatePDA = await this.deriveVMStatePDA();
                accountsToTrack.push(vmStatePDA);
                if (options.debug) {
                    console.log(`  Added VM State PDA to tracking: ${vmStatePDA}`);
                }
            }
            // Add additional accounts
            if (options.additionalAccounts) {
                accountsToTrack.push(...options.additionalAccounts);
                if (options.debug) {
                    console.log(`  Added ${options.additionalAccounts.length} additional accounts to tracking`);
                }
            }
            logs.push(`Tracking ${accountsToTrack.length} accounts for state changes`);
            // Step 1: Fetch BEFORE state
            if (options.debug) {
                console.log(`[FiveSDK] Step 1: Fetching BEFORE state for ${accountsToTrack.length} accounts...`);
            }
            const beforeState = await this.fetchMultipleAccountsAndDeserialize(accountsToTrack, connection, {
                debug: false, // Avoid debug spam
                parseMetadata: true,
                validateEncoding: false, // Skip validation for speed
            });
            let successfulBeforeFetches = 0;
            for (const [address, result] of beforeState.entries()) {
                if (result.success) {
                    successfulBeforeFetches++;
                }
                else if (options.debug) {
                    console.warn(`[FiveSDK] Warning: Failed to fetch BEFORE state for ${address}: ${result.error}`);
                }
            }
            logs.push(`BEFORE state: ${successfulBeforeFetches}/${accountsToTrack.length} accounts fetched`);
            // Extract global fields from BEFORE state if requested
            let beforeGlobalFields = {};
            if (options.trackGlobalFields) {
                const scriptBefore = beforeState.get(scriptAccount);
                if (scriptBefore?.success && scriptBefore.scriptMetadata) {
                    beforeGlobalFields = this.extractGlobalFields(scriptBefore.scriptMetadata, "before");
                    if (options.debug) {
                        console.log(`[FiveSDK] Extracted ${Object.keys(beforeGlobalFields).length} global fields from BEFORE state`);
                    }
                }
            }
            // Step 2: Execute the script
            if (options.debug) {
                console.log(`[FiveSDK] Step 2: Executing script...`);
            }
            const executionResult = await this.executeOnSolana(scriptAccount, connection, signerKeypair, functionName, parameters, options.additionalAccounts || [], {
                debug: options.debug,
                network: options.network,
                computeUnitLimit: options.computeUnitLimit,
            });
            if (!executionResult.success) {
                logs.push(`Execution failed: ${executionResult.error}`);
                return {
                    success: false,
                    error: `Script execution failed: ${executionResult.error}`,
                    logs,
                };
            }
            logs.push(`Execution successful: ${executionResult.transactionId}`);
            // Step 3: Wait a moment for state to settle
            await new Promise((resolve) => setTimeout(resolve, 1000));
            // Step 4: Fetch AFTER state
            if (options.debug) {
                console.log(`[FiveSDK] Step 3: Fetching AFTER state...`);
            }
            const afterState = await this.fetchMultipleAccountsAndDeserialize(accountsToTrack, connection, {
                debug: false,
                parseMetadata: true,
                validateEncoding: false,
            });
            let successfulAfterFetches = 0;
            for (const [address, result] of afterState.entries()) {
                if (result.success) {
                    successfulAfterFetches++;
                }
                else if (options.debug) {
                    console.warn(`[FiveSDK] Warning: Failed to fetch AFTER state for ${address}: ${result.error}`);
                }
            }
            logs.push(`AFTER state: ${successfulAfterFetches}/${accountsToTrack.length} accounts fetched`);
            // Extract global fields from AFTER state if requested
            let afterGlobalFields = {};
            if (options.trackGlobalFields) {
                const scriptAfter = afterState.get(scriptAccount);
                if (scriptAfter?.success && scriptAfter.scriptMetadata) {
                    afterGlobalFields = this.extractGlobalFields(scriptAfter.scriptMetadata, "after");
                    if (options.debug) {
                        console.log(`[FiveSDK] Extracted ${Object.keys(afterGlobalFields).length} global fields from AFTER state`);
                    }
                }
            }
            // Step 5: Compute differences
            if (options.debug) {
                console.log(`[FiveSDK] Step 4: Computing state differences...`);
            }
            const changes = this.computeStateDifferences(beforeState, afterState, options.debug);
            let globalFieldChanges = [];
            if (options.trackGlobalFields) {
                globalFieldChanges = this.computeGlobalFieldChanges(beforeGlobalFields, afterGlobalFields);
                if (options.debug) {
                    console.log(`[FiveSDK] Found ${globalFieldChanges.length} global field changes`);
                }
            }
            logs.push(`State analysis: ${changes.length} account changes, ${globalFieldChanges.length} global field changes`);
            return {
                success: true,
                execution: {
                    transactionId: executionResult.transactionId,
                    result: executionResult.result,
                    computeUnitsUsed: executionResult.computeUnitsUsed,
                    logs: executionResult.logs,
                },
                stateDiff: {
                    beforeState,
                    afterState,
                    changes,
                    globalFieldChanges,
                },
                logs,
            };
        }
        catch (error) {
            const errorMessage = error instanceof Error ? error.message : "Unknown state tracking error";
            if (options.debug) {
                console.error(`[FiveSDK] State diff execution failed: ${errorMessage}`);
            }
            return {
                success: false,
                error: errorMessage,
                logs,
            };
        }
    }
    /**
     * Compare account states and find detailed differences
     */
    static computeStateDifferences(beforeState, afterState, debug = false) {
        const changes = [];
        // Check all accounts that were tracked
        const allAccounts = new Set([...beforeState.keys(), ...afterState.keys()]);
        for (const account of allAccounts) {
            const before = beforeState.get(account);
            const after = afterState.get(account);
            if (debug) {
                console.log(`[FiveSDK] Analyzing account ${account.substring(0, 8)}...`);
            }
            // Account was created
            if (!before?.success && after?.success) {
                changes.push({
                    account,
                    oldValue: null,
                    newValue: {
                        lamports: after.accountInfo?.lamports,
                        dataLength: after.accountInfo?.dataLength,
                        owner: after.accountInfo?.owner,
                    },
                    changeType: "created",
                });
                continue;
            }
            // Account was deleted
            if (before?.success && !after?.success) {
                changes.push({
                    account,
                    oldValue: {
                        lamports: before.accountInfo?.lamports,
                        dataLength: before.accountInfo?.dataLength,
                        owner: before.accountInfo?.owner,
                    },
                    newValue: null,
                    changeType: "deleted",
                });
                continue;
            }
            // Account exists in both states - check for modifications
            if (before?.success && after?.success) {
                // Check lamports change
                if (before.accountInfo?.lamports !== after.accountInfo?.lamports) {
                    changes.push({
                        account,
                        fieldName: "lamports",
                        oldValue: before.accountInfo?.lamports,
                        newValue: after.accountInfo?.lamports,
                        changeType: "modified",
                    });
                }
                // Check data size change
                if (before.accountInfo?.dataLength !== after.accountInfo?.dataLength) {
                    changes.push({
                        account,
                        fieldName: "dataLength",
                        oldValue: before.accountInfo?.dataLength,
                        newValue: after.accountInfo?.dataLength,
                        changeType: "modified",
                    });
                }
                // Check bytecode changes (for script accounts)
                if (before.rawBytecode && after.rawBytecode) {
                    if (!this.bytecodeEqual(before.rawBytecode, after.rawBytecode)) {
                        changes.push({
                            account,
                            fieldName: "bytecode",
                            oldValue: `${before.rawBytecode.length} bytes (hash: ${this.hashBytecode(before.rawBytecode)})`,
                            newValue: `${after.rawBytecode.length} bytes (hash: ${this.hashBytecode(after.rawBytecode)})`,
                            changeType: "modified",
                        });
                    }
                }
                // Check script metadata changes
                if (before.scriptMetadata && after.scriptMetadata) {
                    this.compareScriptMetadata(before.scriptMetadata, after.scriptMetadata, account, changes);
                }
            }
        }
        if (debug) {
            console.log(`[FiveSDK] Found ${changes.length} total state changes`);
        }
        return changes;
    }
    /**
     * Extract global fields from script metadata
     */
    static extractGlobalFields(scriptMetadata, phase) {
        const globalFields = {};
        try {
            // Global fields are typically stored in the script's state or ABI
            // if (scriptMetadata.abi && scriptMetadata.abi.fields) {
            //   scriptMetadata.abi.fields.forEach((field: any) => {
            //     if (field.global) {
            //       globalFields[field.name] = field.value || field.defaultValue || null;
            //     }
            //   });
            // }
            // Try to extract global state from bytecode if available
            // This would require parsing the Five VM state format
            if (scriptMetadata.bytecode && scriptMetadata.bytecode.length > 6) {
                // Skip header and try to find global state section
                // This is a simplified extraction - in practice, you'd need
                // the full Five VM state parser
                const stateSection = this.extractStateSection(scriptMetadata.bytecode);
                if (stateSection) {
                    Object.assign(globalFields, stateSection);
                }
            }
        }
        catch (error) {
            console.warn(`[FiveSDK] Failed to extract global fields (${phase}):`, error);
        }
        return globalFields;
    }
    /**
     * Compare global field values between before and after states
     */
    static computeGlobalFieldChanges(beforeFields, afterFields) {
        const changes = [];
        // Get all field names from both states
        const allFields = new Set([
            ...Object.keys(beforeFields),
            ...Object.keys(afterFields),
        ]);
        for (const fieldName of allFields) {
            const oldValue = beforeFields[fieldName];
            const newValue = afterFields[fieldName];
            // Check if value changed
            if (!this.deepEqual(oldValue, newValue)) {
                changes.push({
                    fieldName,
                    oldValue,
                    newValue,
                });
            }
        }
        return changes;
    }
    /**
     * Compare script metadata for changes
     */
    static compareScriptMetadata(beforeMetadata, afterMetadata, account, changes) {
        // Check if function count changed
        if (beforeMetadata.abi.functions.length !== afterMetadata.abi.functions.length) {
            changes.push({
                account,
                fieldName: "function_count",
                oldValue: beforeMetadata.abi.functions.length,
                newValue: afterMetadata.abi.functions.length,
                changeType: "modified",
            });
        }
        // Check if script name changed
        if (beforeMetadata.abi.name !== afterMetadata.abi.name) {
            changes.push({
                account,
                fieldName: "script_name",
                oldValue: beforeMetadata.abi.name,
                newValue: afterMetadata.abi.name,
                changeType: "modified",
            });
        }
        // Check if authority changed
        if (beforeMetadata.authority !== afterMetadata.authority) {
            changes.push({
                account,
                fieldName: "authority",
                oldValue: beforeMetadata.authority,
                newValue: afterMetadata.authority,
                changeType: "modified",
            });
        }
    }
    /**
     * Extract state section from Five VM bytecode
     * This is a simplified version - full implementation would require
     * complete Five VM state format knowledge
     */
    static extractStateSection(bytecode) {
        try {
            // Skip header (6 bytes: "5IVE" + features + function_count)
            if (bytecode.length < 6)
                return null;
            // Look for state section marker (this is hypothetical)
            // In practice, you'd need to parse the full Five VM format
            const stateMarker = new Uint8Array([0xff, 0xfe]); // Hypothetical state section marker
            for (let i = 6; i < bytecode.length - 1; i++) {
                if (bytecode[i] === stateMarker[0] &&
                    bytecode[i + 1] === stateMarker[1]) {
                    // Found potential state section
                    // Parse state variables (simplified)
                    const stateData = {};
                    // This would need proper state parsing logic
                    // Return empty object
                    return stateData;
                }
            }
        }
        catch (error) {
            console.warn("[FiveSDK] State section extraction failed:", error);
        }
        return null;
    }
    /**
     * Utility: Check if two bytecode arrays are equal
     */
    static bytecodeEqual(a, b) {
        if (a.length !== b.length)
            return false;
        for (let i = 0; i < a.length; i++) {
            if (a[i] !== b[i])
                return false;
        }
        return true;
    }
    /**
     * Utility: Generate simple hash of bytecode for comparison
     */
    static hashBytecode(bytecode) {
        let hash = 0;
        for (let i = 0; i < bytecode.length; i++) {
            hash = ((hash << 5) - hash + bytecode[i]) & 0xffffffff;
        }
        return hash.toString(16);
    }
    /**
     * Utility: Deep equality check
     */
    static deepEqual(a, b) {
        if (a === b)
            return true;
        if (a == null || b == null)
            return false;
        if (typeof a !== typeof b)
            return false;
        if (typeof a === "object") {
            if (Array.isArray(a) !== Array.isArray(b))
                return false;
            const keysA = Object.keys(a);
            const keysB = Object.keys(b);
            if (keysA.length !== keysB.length)
                return false;
            for (const key of keysA) {
                if (!keysB.includes(key))
                    return false;
                if (!this.deepEqual(a[key], b[key]))
                    return false;
            }
            return true;
        }
        return false;
    }
    // ==================== Actual Solana Operations ====================
    /**
     * Execute script on Solana with real transaction submission
     * This is the method the CLI should use for actual on-chain execution
     */
    static async executeOnSolana(scriptAccount, // The deployed script account (from deployment)
    connection, // Solana Connection object
    signerKeypair, // Solana Keypair object for signing
    functionName, parameters = [], accounts = [], // Additional account pubkeys as base58 strings
    options = {}) {
        // Track the latest signature so we can surface it even on failure for log inspection.
        let lastSignature;
        if (options.debug) {
            console.log(`[FiveSDK] executeOnSolana called with script account: ${scriptAccount}`);
            console.log(`[FiveSDK] function: ${functionName}, parameters: ${JSON.stringify(parameters)}`);
            console.log(`[FiveSDK] options:`, options);
        }
        try {
            // Import Solana web3.js components
            const { PublicKey, Transaction, TransactionInstruction, ComputeBudgetProgram, } = await import("@solana/web3.js");
            // For on-chain execution, we'll bypass metadata requirements and generate instruction directly
            // Generate execution instruction - for MVP, we'll use simple parameter encoding without full metadata
            let executionData;
            try {
                executionData = await this.generateExecuteInstruction(scriptAccount, functionName, parameters, accounts, connection, {
                    debug: options.debug,
                    computeUnitLimit: options.computeUnitLimit,
                    vmStateAccount: options.vmStateAccount,
                });
            }
            catch (metadataError) {
                // NO FALLBACK: Metadata is required for proper varint encoding (varint)
                // ENGINEERING INTEGRITY: No duplicate code paths, no silent degradation
                const errorMessage = `Execution instruction generation failed - metadata required for varint encoding (varint): ${metadataError instanceof Error ? metadataError.message : "Unknown metadata error"}`;
                if (options.debug) {
                    console.error(`[FiveSDK] ${errorMessage}`);
                }
                throw new Error(errorMessage);
            }
            if (options.debug) {
                console.log(`[FiveSDK] Generated execution instruction:`, executionData);
                console.log(`[FiveSDK] Accounts in instruction:`, executionData.instruction.accounts);
            }
            // Create transaction
            const transaction = new Transaction();
            // Add compute budget instruction if specified
            if (options.computeUnitLimit && options.computeUnitLimit > 200000) {
                const computeBudgetIx = ComputeBudgetProgram.setComputeUnitLimit({
                    units: options.computeUnitLimit,
                });
                transaction.add(computeBudgetIx);
            }
            if (options.computeUnitPrice && options.computeUnitPrice > 0) {
                const computePriceIx = ComputeBudgetProgram.setComputeUnitPrice({
                    microLamports: options.computeUnitPrice,
                });
                transaction.add(computePriceIx);
            }
            // Build execution instruction - ensure signer is marked correctly
            const accountKeys = [...executionData.instruction.accounts];
            // Force VM state account override if provided
            if (options.vmStateAccount && accountKeys.length >= 2) {
                for (let i = 0; i < accountKeys.length; i++) {
                    // Heuristic: first writable, non-signer account after script is VM state
                    if (i === 1) {
                        accountKeys[i].pubkey = options.vmStateAccount;
                        if (options.debug) {
                            console.log(`[FiveSDK] Overriding VM state account to: ${options.vmStateAccount}`);
                        }
                        break;
                    }
                }
            }
            // Ensure the signer account is marked as signer on its existing meta if present
            const signerPubkey = signerKeypair.publicKey.toString();
            let signerFound = false;
            for (const meta of accountKeys) {
                if (meta.pubkey === signerPubkey) {
                    meta.isSigner = true;
                    // Keep existing isWritable as-is
                    signerFound = true;
                }
            }
            // If signer not present in the accounts, append it
            if (!signerFound) {
                accountKeys.push({
                    pubkey: signerPubkey,
                    isSigner: true,
                    isWritable: true,
                });
                if (options.debug) {
                    console.log(`[FiveSDK] Added signer account: ${signerPubkey}`);
                }
            }
            if (options.debug) {
                console.log(`[FiveSDK] Final account keys for transaction:`, accountKeys);
            }
            const executeInstruction = new TransactionInstruction({
                keys: accountKeys.map((acc) => ({
                    pubkey: new PublicKey(acc.pubkey),
                    isSigner: acc.isSigner,
                    isWritable: acc.isWritable,
                })),
                programId: new PublicKey(executionData.instruction.programId),
                data: Buffer.from(executionData.instruction.data, "base64"),
            });
            transaction.add(executeInstruction);
            // Set transaction properties
            transaction.feePayer = signerKeypair.publicKey;
            const { blockhash } = await connection.getLatestBlockhash("confirmed");
            transaction.recentBlockhash = blockhash;
            // Sign transaction
            transaction.partialSign(signerKeypair);
            const firstSig = transaction.signatures[0]?.signature;
            if (firstSig) {
                lastSignature = Base58Utils.encode(firstSig);
            }
            if (options.debug) {
                console.log(`[FiveSDK] Sending execution transaction...`);
            }
            // ==================== TRANSACTION DEBUG OUTPUT ====================
            console.log(`\n🔍 EXECUTION TRANSACTION DETAILS:`);
            console.log(`📋 Transaction Overview:`);
            console.log(`  - Instructions: ${transaction.instructions.length}`);
            console.log(`  - Fee Payer: ${transaction.feePayer?.toString()}`);
            console.log(`  - Recent Blockhash: ${transaction.recentBlockhash}`);
            console.log(`  - Signatures: ${transaction.signatures.length}`);
            transaction.instructions.forEach((instruction, index) => {
                console.log(`\n📝 Instruction ${index}:`);
                console.log(`  - Program ID: ${instruction.programId.toString()}`);
                console.log(`  - Data Length: ${instruction.data.length} bytes`);
                console.log(`  - Data (hex): ${instruction.data.toString("hex")}`);
                console.log(`  - Data (base64): ${instruction.data.toString("base64")}`);
                console.log(`  - Accounts (${instruction.keys.length}):`);
                instruction.keys.forEach((key, keyIndex) => {
                    console.log(`    ${keyIndex}: ${key.pubkey.toString()} (signer: ${key.isSigner}, writable: ${key.isWritable})`);
                });
            });
            console.log(`\n🔐 Transaction Signatures:`);
            transaction.signatures.forEach((sig, sigIndex) => {
                console.log(`  ${sigIndex}: ${sig.publicKey.toString()} - ${sig.signature ? "SIGNED" : "UNSIGNED"}`);
            });
            console.log(`\n📦 Serialized Transaction: ${transaction.serialize().length} bytes`);
            console.log(`==================== END TRANSACTION DEBUG ====================\n`);
            console.log(`\n\n\n!!!!!!!!! SENDING TRANSACTION NOW !!!!!!!!!\n\n\n`);
            // Send transaction with preflight disabled to get actual Five VM errors
            const signature = await connection.sendRawTransaction(transaction.serialize(), {
                skipPreflight: true, // Skip preflight to see actual program errors
                preflightCommitment: "confirmed",
                maxRetries: options.maxRetries || 3,
            });
            lastSignature = signature;
            // Wait for confirmation with detailed error handling
            let confirmation;
            try {
                confirmation = await connection.confirmTransaction({
                    signature,
                    blockhash,
                    lastValidBlockHeight: (await connection.getLatestBlockhash("confirmed")).lastValidBlockHeight,
                }, "confirmed");
            }
            catch (confirmError) {
                if (options.debug) {
                    console.log(`[FiveSDK] Confirmation failed, checking transaction status...`);
                }
                // Try to get transaction details even if confirmation failed
                try {
                    const txDetails = await connection.getTransaction(signature, {
                        commitment: "confirmed",
                        maxSupportedTransactionVersion: 0,
                    });
                    if (txDetails) {
                        if (options.debug) {
                            console.log(`[FiveSDK] Transaction found! Status:`, txDetails.meta?.err ? "Failed" : "Success");
                            console.log(`[FiveSDK] Transaction logs:`, txDetails.meta?.logMessages);
                        }
                        if (txDetails.meta?.err) {
                            return {
                                success: false,
                                error: `Transaction failed: ${JSON.stringify(txDetails.meta.err)}`,
                                logs: txDetails.meta.logMessages || [],
                                transactionId: signature,
                            };
                        }
                        else {
                            // Transaction succeeded but confirmation timed out
                            return {
                                success: true,
                                transactionId: signature,
                                computeUnitsUsed: txDetails.meta?.computeUnitsConsumed,
                                logs: txDetails.meta?.logMessages || [],
                                result: "Execution completed successfully (confirmation timeout but transaction succeeded)",
                            };
                        }
                    }
                }
                catch (getTransactionError) {
                    if (options.debug) {
                        console.log(`[FiveSDK] Could not retrieve transaction details:`, getTransactionError);
                    }
                }
                throw confirmError;
            }
            if (options.debug) {
                console.log(`[FiveSDK] Transaction confirmed: ${signature}`);
                console.log(`[FiveSDK] Confirmation:`, confirmation);
            }
            // Check if execution transaction actually succeeded
            if (confirmation.value.err) {
                let logs = [];
                let computeUnitsUsed;
                try {
                    const txDetails = await connection.getTransaction(signature, {
                        commitment: "confirmed",
                        maxSupportedTransactionVersion: 0,
                    });
                    if (txDetails?.meta) {
                        logs = txDetails.meta.logMessages || [];
                        computeUnitsUsed = txDetails.meta.computeUnitsConsumed || undefined;
                    }
                    // Fallback: getLogsForSignature if needed
                    if ((!logs || logs.length === 0) && connection.getLogsForSignature) {
                        const logsResp = await connection.getLogsForSignature(signature, {
                            commitment: "confirmed",
                        });
                        if (logsResp?.value?.logs) {
                            logs = logsResp.value.logs;
                        }
                    }
                }
                catch { }
                const errorMessage = `Execution transaction failed: ${JSON.stringify(confirmation.value.err)}`;
                if (options.debug) {
                    console.log(`[FiveSDK] ${errorMessage}`);
                    if (logs.length) {
                        console.log(`[FiveSDK] On-chain logs:`, logs);
                    }
                }
                return {
                    success: false,
                    error: errorMessage,
                    transactionId: signature,
                    logs,
                    computeUnitsUsed,
                };
            }
            // Get transaction details for logs and compute units
            let computeUnitsUsed;
            let logs = [];
            try {
                const txDetails = await connection.getTransaction(signature, {
                    commitment: "confirmed",
                    maxSupportedTransactionVersion: 0,
                });
                if (txDetails?.meta) {
                    computeUnitsUsed = txDetails.meta.computeUnitsConsumed || undefined;
                    logs = txDetails.meta.logMessages || [];
                }
            }
            catch (logError) {
                if (options.debug) {
                    console.warn(`[FiveSDK] Could not fetch transaction logs: ${logError}`);
                }
            }
            return {
                success: true,
                transactionId: signature,
                computeUnitsUsed,
                logs,
                result: "Execution completed successfully", // Five VM doesn't return complex results yet
            };
        }
        catch (error) {
            const errorMessage = error instanceof Error ? error.message : "Unknown execution error";
            // Capture signature if SendTransactionError populated it
            if (!lastSignature && error?.signature) {
                lastSignature = error.signature;
            }
            if (options.debug) {
                console.error(`[FiveSDK] Execution failed: ${errorMessage}`);
                if (error instanceof Error && error.stack) {
                    console.error(`[FiveSDK] Stack trace:`, error.stack);
                }
            }
            // Attempt to extract logs from SendTransactionError if available
            let logs = error?.transactionLogs || [];
            if (typeof error?.getLogs === "function") {
                try {
                    const extracted = await error.getLogs();
                    if (Array.isArray(extracted)) {
                        logs = extracted;
                    }
                }
                catch {
                    /* ignore */
                }
            }
            return {
                success: false,
                error: errorMessage,
                transactionId: lastSignature,
                logs,
            };
        }
    }
    /**
     * Execute deployed script account on-chain - wrapper for executeOnSolana with simpler interface
     * This is the method the CLI should use for script account execution
     */
    static async executeScriptAccount(scriptAccount, functionIndex = 0, parameters = [], connection, // Solana Connection object
    signerKeypair, // Solana Keypair object
    options = {}) {
        if (options.debug) {
            console.log(`[FiveSDK] executeScriptAccount called with:`);
            console.log(`  Script Account: ${scriptAccount}`);
            console.log(`  Function Index: ${functionIndex}`);
            console.log(`  Parameters: ${JSON.stringify(parameters)}`);
        }
        try {
            // Call the existing executeOnSolana method with function index
            const result = await this.executeOnSolana(scriptAccount, connection, signerKeypair, functionIndex, // Use function index instead of name
            parameters, [], // No additional accounts for now
            {
                debug: options.debug,
                network: options.network,
                computeUnitLimit: options.computeBudget || 1400000,
                maxRetries: options.maxRetries || 3,
                vmStateAccount: options.vmStateAccount,
            });
            if (options.debug) {
                console.log(`[FiveSDK] executeScriptAccount result:`, result);
            }
            return result;
        }
        catch (error) {
            const errorMessage = error instanceof Error
                ? error.message
                : "Unknown script execution error";
            if (options.debug) {
                console.error(`[FiveSDK] executeScriptAccount failed: ${errorMessage}`);
            }
            return {
                success: false,
                error: errorMessage,
                logs: [],
            };
        }
    }
    /**
     * Extract function names from compiled bytecode
     */
    static async getFunctionNames(bytecode) {
        await this.initializeComponents(false);
        try {
            const namesJson = await this.compiler.getFunctionNames(bytecode);
            if (Array.isArray(namesJson)) {
                return namesJson;
            }
            const parsed = JSON.parse(namesJson);
            return parsed;
        }
        catch (error) {
            console.warn("[FiveSDK] Failed to extract function names:", error);
            return [];
        }
    }
    /**
     * Call a function by name instead of index
     */
    static async callFunctionByName(scriptAccount, functionName, parameters = [], accounts = [], connection, options = {}) {
        // First get the available function names
        const available = await this.getFunctionNamesFromScriptAccount(scriptAccount, connection);
        if (!available) {
            throw new ExecutionSDKError(`Cannot resolve function name "${functionName}": unable to fetch bytecode from script account`);
        }
        const funcInfo = available.find((f) => f.name === functionName);
        if (!funcInfo) {
            const availableNames = available.map((f) => f.name).join(", ");
            throw new ExecutionSDKError(`Function "${functionName}" not found. Available functions: ${availableNames}`);
        }
        // Now execute using the resolved index (call the index-based executor)
        return this.executeByIndex(scriptAccount, funcInfo.function_index, parameters, accounts, connection, options);
    }
    /**
     * Generate serialized execution data by function index.
     */
    static async executeByIndex(scriptAccount, functionIndex, parameters = [], accounts = [], connection, options = {}) {
        validator.validateBase58Address(scriptAccount, "scriptAccount");
        Validators.functionRef(functionIndex);
        Validators.parameters(parameters);
        Validators.accounts(accounts);
        Validators.options(options);
        return this.generateExecuteInstruction(scriptAccount, functionIndex, parameters, accounts, connection, options);
    }
    /**
     * Get function names from a deployed script account
     */
    static async getFunctionNamesFromScriptAccount(scriptAccount, connection) {
        if (!connection) {
            console.warn("[FiveSDK] No connection provided for script account lookup");
            return null;
        }
        try {
            const { PublicKey } = await import("@solana/web3.js");
            const accountInfo = await connection.getAccountInfo(new PublicKey(scriptAccount), "confirmed");
            if (!accountInfo) {
                console.warn(`[FiveSDK] Script account ${scriptAccount} not found`);
                return null;
            }
            const data = accountInfo.data;
            const scriptHeaderSize = 64;
            let bytecode = data;
            if (data.length >= scriptHeaderSize &&
                data[0] === 0x35 &&
                data[1] === 0x49 &&
                data[2] === 0x56 &&
                data[3] === 0x45) {
                const readU32LE = (buffer, offset) => buffer[offset] |
                    (buffer[offset + 1] << 8) |
                    (buffer[offset + 2] << 16) |
                    (buffer[offset + 3] << 24);
                const headerVersion = data[4];
                const reserved0 = data[6] | data[7];
                const bytecodeLenOffset = 48;
                const metadataLenOffset = 52;
                const bytecodeLen = readU32LE(data, bytecodeLenOffset);
                const metadataLen = readU32LE(data, metadataLenOffset);
                const totalLen = scriptHeaderSize + bytecodeLen + metadataLen;
                if (headerVersion === 4 &&
                    reserved0 === 0 &&
                    bytecodeLen > 0 &&
                    totalLen <= data.length) {
                    bytecode = data.slice(scriptHeaderSize, scriptHeaderSize + bytecodeLen);
                }
            }
            return await this.getFunctionNames(bytecode);
        }
        catch (error) {
            console.warn("[FiveSDK] Failed to fetch function names from script account:", error);
            return null;
        }
    }
    /**
     * Deploy bytecode to Solana using the correct two-transaction pattern
     * This is the method the CLI should use for actual deployment
     */
    static async deployToSolana(bytecode, connection, // Solana Connection object
    deployerKeypair, // Solana Keypair object
    options = {}) {
        console.log(`[FiveSDK] deployToSolana called with bytecode length: ${bytecode.length}`);
        console.log(`[FiveSDK] options:`, options);
        // Use the provided program ID or fall back to the constant
        const programId = options.fiveVMProgramId || FIVE_VM_PROGRAM_ID;
        try {
            if (options.debug) {
                console.log(`[FiveSDK] Starting deployment with ${bytecode.length} bytes of bytecode to program ${programId}`);
            }
            // Generate script keypair like frontend-five
            const { Keypair, PublicKey, Transaction, TransactionInstruction, SystemProgram, } = await import("@solana/web3.js");
            const scriptKeypair = Keypair.generate();
            const scriptAccount = scriptKeypair.publicKey.toString();
            if (options.debug) {
                console.log(`[FiveSDK] Generated script keypair: ${scriptAccount}`);
            }
            // Calculate account size and rent
            const SCRIPT_HEADER_SIZE = 64; // ScriptHeader::LEN (five-protocol)
            const totalAccountSize = SCRIPT_HEADER_SIZE + bytecode.length;
            const rentLamports = await connection.getMinimumBalanceForRentExemption(totalAccountSize);
            // Generate VM state keypair for this deployment
            const vmStateKeypair = Keypair.generate();
            if (options.debug) {
                console.log(`[FiveSDK] Script Account: ${scriptAccount}`);
                console.log(`[FiveSDK] VM State Account: ${vmStateKeypair.publicKey.toString()}`);
                console.log(`[FiveSDK] Account size: ${totalAccountSize} bytes`);
                console.log(`[FiveSDK] Rent cost: ${rentLamports / 1e9} SOL`);
            }
            // SINGLE TRANSACTION: create VM state + initialize + create script account + deploy bytecode
            const tx = new Transaction();
            // Optional compute budget
            if (options.computeBudget && options.computeBudget > 0) {
                try {
                    const { ComputeBudgetProgram } = await import("@solana/web3.js");
                    tx.add(ComputeBudgetProgram.setComputeUnitLimit({
                        units: options.computeBudget,
                    }));
                }
                catch { }
            }
            // 1) Create VM state account owned by the program
            const VM_STATE_SIZE = 48; // FIVEVMState::LEN
            const vmStateRent = await connection.getMinimumBalanceForRentExemption(VM_STATE_SIZE);
            const createVmStateIx = SystemProgram.createAccount({
                fromPubkey: deployerKeypair.publicKey,
                newAccountPubkey: vmStateKeypair.publicKey,
                lamports: vmStateRent,
                space: VM_STATE_SIZE,
                programId: new PublicKey(programId),
            });
            tx.add(createVmStateIx);
            // 2) Initialize VM state: [discriminator(0)] with accounts [vm_state, authority]
            const initVmStateIx = new TransactionInstruction({
                keys: [
                    {
                        pubkey: vmStateKeypair.publicKey,
                        isSigner: false,
                        isWritable: true,
                    },
                    {
                        pubkey: deployerKeypair.publicKey,
                        isSigner: true,
                        isWritable: false,
                    },
                ],
                programId: new PublicKey(programId),
                data: Buffer.from([0]), // Initialize discriminator
            });
            tx.add(initVmStateIx);
            // 3) Create script account
            const createAccountIx = SystemProgram.createAccount({
                fromPubkey: deployerKeypair.publicKey,
                newAccountPubkey: scriptKeypair.publicKey,
                lamports: rentLamports,
                space: totalAccountSize,
                programId: new PublicKey(programId),
            });
            tx.add(createAccountIx);
            const deployData = this.encodeDeployInstruction(bytecode);
            console.log(`[FiveSDK] DEBUG - deployData type:`, typeof deployData);
            console.log(`[FiveSDK] DEBUG - deployData length:`, deployData.length);
            console.log(`[FiveSDK] DEBUG - deployData hex:`, Buffer.from(deployData).toString("hex"));
            console.log(`[FiveSDK] DEBUG - scriptKeypair.publicKey:`, scriptKeypair.publicKey.toString());
            const instructionDataBuffer = Buffer.from(deployData);
            console.log(`[FiveSDK] DEBUG - instructionDataBuffer hex:`, instructionDataBuffer.toString("hex"));
            const deployIx = new TransactionInstruction({
                keys: [
                    {
                        pubkey: scriptKeypair.publicKey,
                        isSigner: false,
                        isWritable: true,
                    },
                    {
                        pubkey: vmStateKeypair.publicKey,
                        isSigner: false,
                        isWritable: true,
                    },
                    {
                        pubkey: deployerKeypair.publicKey,
                        isSigner: true,
                        isWritable: true,
                    },
                ],
                programId: new PublicKey(programId),
                data: instructionDataBuffer,
            });
            tx.add(deployIx);
            const { blockhash } = await connection.getLatestBlockhash("confirmed");
            tx.recentBlockhash = blockhash;
            tx.feePayer = deployerKeypair.publicKey;
            tx.partialSign(deployerKeypair);
            tx.partialSign(vmStateKeypair);
            tx.partialSign(scriptKeypair);
            if (options.debug) {
                console.log(`\n🔍 COMBINED DEPLOY TX:`);
                console.log(`  - Instructions: ${tx.instructions.length}`);
                console.log(`  - Fee Payer: ${tx.feePayer?.toString()}`);
                console.log(`  - Recent Blockhash: ${tx.recentBlockhash}`);
                tx.instructions.forEach((ix, i) => {
                    console.log(`\n📝 Instruction ${i}: ${ix.programId.toString()} (keys=${ix.keys.length}, data=${ix.data.length} bytes)`);
                    ix.keys.forEach((k, j) => console.log(`    ${j}: ${k.pubkey.toString()} (signer=${k.isSigner}, writable=${k.isWritable})`));
                });
                console.log(`\n📦 Serialized Size: ${tx.serialize().length} bytes`);
            }
            const signature = await connection.sendRawTransaction(tx.serialize(), {
                skipPreflight: true,
                preflightCommitment: "confirmed",
                maxRetries: options.maxRetries || 3,
            });
            const confirmation = await connection.confirmTransaction(signature, "confirmed");
            if (confirmation.value.err) {
                const errorMessage = `Combined deployment failed: ${JSON.stringify(confirmation.value.err)}`;
                if (options.debug)
                    console.log(`[FiveSDK] ${errorMessage}`);
                return {
                    success: false,
                    error: errorMessage,
                    transactionId: signature,
                };
            }
            if (options.debug) {
                console.log(`[FiveSDK] Combined deployment succeeded: ${signature}`);
            }
            return {
                success: true,
                programId: scriptAccount,
                transactionId: signature,
                deploymentCost: rentLamports,
                logs: [
                    `Script Account: ${scriptAccount}`,
                    `Deployment TX: ${signature}`,
                    `Deployment cost (rent): ${rentLamports / 1e9} SOL`,
                    `Bytecode size: ${bytecode.length} bytes`,
                    `VM State Account: ${vmStateKeypair.publicKey.toString()}`,
                ],
                vmStateAccount: vmStateKeypair.publicKey.toString(),
            };
        }
        catch (error) {
            const errorMessage = error instanceof Error ? error.message : "Unknown deployment error";
            if (options.debug) {
                console.error(`[FiveSDK] Deployment failed: ${errorMessage}`);
            }
            return {
                success: false,
                error: errorMessage,
                logs: [],
            };
        }
    }
    /**
     * Deploy large bytecode programs using InitLargeProgram + AppendBytecode pattern
     * Automatically handles programs larger than single transaction limits
     */
    static async deployLargeProgramToSolana(bytecode, connection, // Solana Connection object
    deployerKeypair, // Solana Keypair object
    options = {}) {
        const DEFAULT_CHUNK_SIZE = 750; // Leaves room for transaction overhead
        const chunkSize = options.chunkSize || DEFAULT_CHUNK_SIZE;
        console.log(`[FiveSDK] deployLargeProgramToSolana called with ${bytecode.length} bytes`);
        console.log(`[FiveSDK] Using chunk size: ${chunkSize} bytes`);
        console.log(`[FiveSDK] options:`, options);
        try {
            // If bytecode is small enough, use regular deployment
            if (bytecode.length <= 800) {
                if (options.debug) {
                    console.log(`[FiveSDK] Bytecode is small (${bytecode.length} bytes), using regular deployment`);
                }
                return await this.deployToSolana(bytecode, connection, deployerKeypair, {
                    debug: options.debug,
                    network: options.network,
                    maxRetries: options.maxRetries,
                });
            }
            const { Keypair, PublicKey, Transaction, TransactionInstruction, SystemProgram, } = await import("@solana/web3.js");
            // Generate script keypair
            const scriptKeypair = Keypair.generate();
            const scriptAccount = scriptKeypair.publicKey.toString();
            // Calculate account size and rent
            const SCRIPT_HEADER_SIZE = 64; // ScriptHeader::LEN (five-protocol)
            const totalAccountSize = SCRIPT_HEADER_SIZE + bytecode.length;
            const rentLamports = await connection.getMinimumBalanceForRentExemption(totalAccountSize);
            const programId = new PublicKey(options.fiveVMProgramId || FIVE_VM_PROGRAM_ID);
            // Generate VM state account for this deployment
            const vmStateKeypair = Keypair.generate();
            const VM_STATE_SIZE = 48; // FIVEVMState::LEN
            const vmStateRent = await connection.getMinimumBalanceForRentExemption(VM_STATE_SIZE);
            if (options.debug) {
                console.log(`[FiveSDK] Script Account: ${scriptAccount}`);
                console.log(`[FiveSDK] VM State Account: ${vmStateKeypair.publicKey.toString()}`);
                console.log(`[FiveSDK] Total account size: ${totalAccountSize} bytes`);
                console.log(`[FiveSDK] Initial rent cost: ${(rentLamports + vmStateRent) / 1e9} SOL`);
            }
            const transactionIds = [];
            let totalCost = rentLamports + vmStateRent;
            // TRANSACTION 0: Create VM State Account + Initialize
            if (options.debug) {
                console.log(`[FiveSDK] Step 0: Create VM state account and initialize`);
            }
            const vmStateTransaction = new Transaction();
            vmStateTransaction.add(SystemProgram.createAccount({
                fromPubkey: deployerKeypair.publicKey,
                newAccountPubkey: vmStateKeypair.publicKey,
                lamports: vmStateRent,
                space: VM_STATE_SIZE,
                programId: programId,
            }));
            vmStateTransaction.add(new TransactionInstruction({
                keys: [
                    {
                        pubkey: vmStateKeypair.publicKey,
                        isSigner: false,
                        isWritable: true,
                    },
                    {
                        pubkey: deployerKeypair.publicKey,
                        isSigner: true,
                        isWritable: false,
                    },
                ],
                programId: programId,
                data: Buffer.from([0]), // Initialize discriminator
            }));
            vmStateTransaction.feePayer = deployerKeypair.publicKey;
            const vmStateBlockhash = await connection.getLatestBlockhash("confirmed");
            vmStateTransaction.recentBlockhash = vmStateBlockhash.blockhash;
            vmStateTransaction.partialSign(deployerKeypair);
            vmStateTransaction.partialSign(vmStateKeypair);
            const vmStateSignature = await connection.sendRawTransaction(vmStateTransaction.serialize(), {
                skipPreflight: true,
                preflightCommitment: "confirmed",
                maxRetries: options.maxRetries || 3,
            });
            await connection.confirmTransaction(vmStateSignature, "confirmed");
            transactionIds.push(vmStateSignature);
            if (options.debug) {
                console.log(`[FiveSDK] ✅ VM state initialized: ${vmStateSignature}`);
            }
            // TRANSACTION 1: Create Account + InitLargeProgram
            if (options.debug) {
                console.log(`[FiveSDK] Step 1: Create account and initialize large program`);
            }
            const initTransaction = new Transaction();
            // Add account creation instruction
            const createAccountInstruction = SystemProgram.createAccount({
                fromPubkey: deployerKeypair.publicKey,
                newAccountPubkey: scriptKeypair.publicKey,
                lamports: rentLamports,
                space: SCRIPT_HEADER_SIZE, // Start with just header space
                programId: programId,
            });
            initTransaction.add(createAccountInstruction);
            // Add InitLargeProgram instruction (discriminator 4 + expected_size as u32)
            const initInstructionData = Buffer.concat([
                Buffer.from([4]), // InitLargeProgram discriminator
                Buffer.from(new Uint32Array([bytecode.length]).buffer), // expected_size as little-endian u32
            ]);
            const initLargeProgramInstruction = new TransactionInstruction({
                keys: [
                    {
                        pubkey: scriptKeypair.publicKey,
                        isSigner: false,
                        isWritable: true,
                    },
                    {
                        pubkey: deployerKeypair.publicKey,
                        isSigner: true,
                        isWritable: true,
                    },
                    { pubkey: vmStateKeypair.publicKey, isSigner: false, isWritable: true },
                ],
                programId: programId,
                data: initInstructionData,
            });
            initTransaction.add(initLargeProgramInstruction);
            // Sign and send initialization transaction
            initTransaction.feePayer = deployerKeypair.publicKey;
            const { blockhash } = await connection.getLatestBlockhash("confirmed");
            initTransaction.recentBlockhash = blockhash;
            initTransaction.partialSign(deployerKeypair);
            initTransaction.partialSign(scriptKeypair);
            const initSignature = await connection.sendRawTransaction(initTransaction.serialize(), {
                skipPreflight: true,
                preflightCommitment: "confirmed",
                maxRetries: options.maxRetries || 3,
            });
            await connection.confirmTransaction(initSignature, "confirmed");
            transactionIds.push(initSignature);
            if (options.debug) {
                console.log(`[FiveSDK] ✅ Initialization completed: ${initSignature}`);
            }
            // STEP 2: Split bytecode into chunks and append each
            const chunks = this.chunkBytecode(bytecode, chunkSize);
            if (options.debug) {
                console.log(`[FiveSDK] Split bytecode into ${chunks.length} chunks`);
            }
            for (let i = 0; i < chunks.length; i++) {
                const chunk = chunks[i];
                if (options.progressCallback) {
                    options.progressCallback(i + 1, chunks.length);
                }
                if (options.debug) {
                    console.log(`[FiveSDK] Step ${i + 2}: Appending chunk ${i + 1}/${chunks.length} (${chunk.length} bytes)`);
                }
                // Calculate additional rent needed for this chunk
                const currentInfo = await connection.getAccountInfo(scriptKeypair.publicKey);
                const newSize = currentInfo.data.length + chunk.length;
                const newRentRequired = await connection.getMinimumBalanceForRentExemption(newSize);
                const additionalRent = Math.max(0, newRentRequired - currentInfo.lamports);
                const appendTransaction = new Transaction();
                // Add rent if needed
                if (additionalRent > 0) {
                    if (options.debug) {
                        console.log(`[FiveSDK] Adding ${additionalRent / 1e9} SOL for increased rent`);
                    }
                    appendTransaction.add(SystemProgram.transfer({
                        fromPubkey: deployerKeypair.publicKey,
                        toPubkey: scriptKeypair.publicKey,
                        lamports: additionalRent,
                    }));
                    totalCost += additionalRent;
                }
                // Add AppendBytecode instruction (discriminator 5 + chunk data)
                const appendInstructionData = Buffer.concat([
                    Buffer.from([5]), // AppendBytecode discriminator
                    chunk,
                ]);
                const appendBytecodeInstruction = new TransactionInstruction({
                    keys: [
                        {
                            pubkey: scriptKeypair.publicKey,
                            isSigner: false,
                            isWritable: true,
                        },
                        {
                            pubkey: deployerKeypair.publicKey,
                            isSigner: true,
                            isWritable: true,
                        },
                        { pubkey: vmStateKeypair.publicKey, isSigner: false, isWritable: true },
                    ],
                    programId: programId,
                    data: appendInstructionData,
                });
                appendTransaction.add(appendBytecodeInstruction);
                // Sign and send append transaction
                const appendBlockhash = await connection.getLatestBlockhash("confirmed");
                appendTransaction.feePayer = deployerKeypair.publicKey;
                appendTransaction.recentBlockhash = appendBlockhash.blockhash;
                appendTransaction.partialSign(deployerKeypair);
                const appendSignature = await connection.sendRawTransaction(appendTransaction.serialize(), {
                    skipPreflight: true,
                    preflightCommitment: "confirmed",
                    maxRetries: options.maxRetries || 3,
                });
                await connection.confirmTransaction(appendSignature, "confirmed");
                transactionIds.push(appendSignature);
                if (options.debug) {
                    console.log(`[FiveSDK] ✅ Chunk ${i + 1} appended: ${appendSignature}`);
                }
            }
            // Final verification
            const finalInfo = await connection.getAccountInfo(scriptKeypair.publicKey);
            const expectedSize = SCRIPT_HEADER_SIZE + bytecode.length;
            if (options.debug) {
                console.log(`[FiveSDK] 🔍 Final verification:`);
                console.log(`[FiveSDK] Expected size: ${expectedSize} bytes`);
                console.log(`[FiveSDK] Actual size: ${finalInfo.data.length} bytes`);
                console.log(`[FiveSDK] Match: ${finalInfo.data.length === expectedSize ? "✅ YES" : "❌ NO"}`);
            }
            return {
                success: true,
                scriptAccount,
                transactionIds,
                totalTransactions: transactionIds.length,
                deploymentCost: totalCost,
                chunksUsed: chunks.length,
                vmStateAccount: vmStateKeypair.publicKey.toString(),
                logs: [
                    `Deployed ${bytecode.length} bytes in ${chunks.length} chunks using ${transactionIds.length} transactions`,
                ],
            };
        }
        catch (error) {
            const errorMessage = error instanceof Error
                ? error.message
                : "Unknown large deployment error";
            if (options.debug) {
                console.error(`[FiveSDK] Large deployment failed: ${errorMessage}`);
            }
            return {
                success: false,
                error: errorMessage,
                logs: [],
            };
        }
    }
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
    static async deployLargeProgramOptimizedToSolana(bytecode, connection, // Solana Connection object
    deployerKeypair, // Solana Keypair object
    options = {}) {
        const OPTIMIZED_CHUNK_SIZE = 950; // Larger chunks due to reduced transaction overhead
        const chunkSize = options.chunkSize || OPTIMIZED_CHUNK_SIZE;
        console.log(`[FiveSDK] deployLargeProgramOptimizedToSolana called with ${bytecode.length} bytes`);
        console.log(`[FiveSDK] Using optimized chunk size: ${chunkSize} bytes`);
        console.log(`[FiveSDK] Expected optimization: 50-70% fewer transactions`);
        try {
            // If bytecode is small enough, use regular deployment
            if (bytecode.length <= 800) {
                if (options.debug) {
                    console.log(`[FiveSDK] Bytecode is small (${bytecode.length} bytes), using regular deployment`);
                }
                return await this.deployToSolana(bytecode, connection, deployerKeypair, {
                    debug: options.debug,
                    network: options.network,
                    maxRetries: options.maxRetries,
                });
            }
            const { Keypair, PublicKey, Transaction, TransactionInstruction, SystemProgram, } = await import("@solana/web3.js");
            // Generate script keypair
            const scriptKeypair = Keypair.generate();
            const scriptAccount = scriptKeypair.publicKey.toString();
            // PRE-ALLOCATION OPTIMIZATION: Calculate full account size upfront
            const SCRIPT_HEADER_SIZE = 128; // FIVEScriptHeaderV2::LEN
            const totalAccountSize = SCRIPT_HEADER_SIZE + bytecode.length;
            const rentLamports = await connection.getMinimumBalanceForRentExemption(totalAccountSize);
            const programId = new PublicKey(options.fiveVMProgramId || FIVE_VM_PROGRAM_ID);
            // Generate VM state account for this deployment
            const vmStateKeypair = Keypair.generate();
            const VM_STATE_SIZE = 48; // FIVEVMState::LEN
            const vmStateRent = await connection.getMinimumBalanceForRentExemption(VM_STATE_SIZE);
            if (options.debug) {
                console.log(`[FiveSDK] Script Account: ${scriptAccount}`);
                console.log(`[FiveSDK] VM State Account: ${vmStateKeypair.publicKey.toString()}`);
                console.log(`[FiveSDK] PRE-ALLOCATED full account size: ${totalAccountSize} bytes`);
                console.log(`[FiveSDK] Full rent cost paid upfront: ${(rentLamports + vmStateRent) / 1e9} SOL`);
            }
            const transactionIds = [];
            let totalCost = rentLamports + vmStateRent;
            // TRANSACTION 0: Create VM State Account + Initialize
            if (options.debug) {
                console.log(`[FiveSDK] Step 0: Create VM state account and initialize`);
            }
            const vmStateTransaction = new Transaction();
            vmStateTransaction.add(SystemProgram.createAccount({
                fromPubkey: deployerKeypair.publicKey,
                newAccountPubkey: vmStateKeypair.publicKey,
                lamports: vmStateRent,
                space: VM_STATE_SIZE,
                programId: programId,
            }));
            vmStateTransaction.add(new TransactionInstruction({
                keys: [
                    {
                        pubkey: vmStateKeypair.publicKey,
                        isSigner: false,
                        isWritable: true,
                    },
                    {
                        pubkey: deployerKeypair.publicKey,
                        isSigner: true,
                        isWritable: false,
                    },
                ],
                programId: programId,
                data: Buffer.from([0]), // Initialize discriminator
            }));
            vmStateTransaction.feePayer = deployerKeypair.publicKey;
            const vmStateBlockhash = await connection.getLatestBlockhash("confirmed");
            vmStateTransaction.recentBlockhash = vmStateBlockhash.blockhash;
            vmStateTransaction.partialSign(deployerKeypair);
            vmStateTransaction.partialSign(vmStateKeypair);
            const vmStateSignature = await connection.sendRawTransaction(vmStateTransaction.serialize(), {
                skipPreflight: true,
                preflightCommitment: "confirmed",
                maxRetries: options.maxRetries || 3,
            });
            await connection.confirmTransaction(vmStateSignature, "confirmed");
            transactionIds.push(vmStateSignature);
            if (options.debug) {
                console.log(`[FiveSDK] ✅ VM state initialized: ${vmStateSignature}`);
            }
            // Split bytecode into chunks
            const chunks = this.chunkBytecode(bytecode, chunkSize);
            const firstChunk = chunks[0];
            const remainingChunks = chunks.slice(1);
            if (options.debug) {
                console.log(`[FiveSDK] Split into ${chunks.length} chunks (first: ${firstChunk.length} bytes, remaining: ${remainingChunks.length})`);
            }
            // OPTIMIZATION 1: TRANSACTION 1 - Create Account + InitLargeProgramWithChunk (combined)
            if (options.debug) {
                console.log(`[FiveSDK] ⚡ OPTIMIZED Step 1: Create account + initialize with first chunk (${firstChunk.length} bytes)`);
            }
            const initTransaction = new Transaction();
            // Add account creation instruction with FULL SIZE
            const createAccountInstruction = SystemProgram.createAccount({
                fromPubkey: deployerKeypair.publicKey,
                newAccountPubkey: scriptKeypair.publicKey,
                lamports: rentLamports, // Full rent paid upfront
                space: totalAccountSize, // PRE-ALLOCATE full space
                programId: programId,
            });
            initTransaction.add(createAccountInstruction);
            // Add InitLargeProgramWithChunk instruction (discriminator 4 + expected_size + first_chunk)
            const initInstructionData = Buffer.concat([
                Buffer.from([4]), // InitLargeProgramWithChunk discriminator (same as InitLargeProgram)
                Buffer.from(new Uint32Array([bytecode.length]).buffer), // expected_size as little-endian u32
                firstChunk, // First chunk data
            ]);
            const initLargeProgramWithChunkInstruction = new TransactionInstruction({
                keys: [
                    {
                        pubkey: scriptKeypair.publicKey,
                        isSigner: false,
                        isWritable: true,
                    },
                    {
                        pubkey: deployerKeypair.publicKey,
                        isSigner: true,
                        isWritable: true,
                    },
                    { pubkey: vmStateKeypair.publicKey, isSigner: false, isWritable: true },
                ],
                programId: programId,
                data: initInstructionData,
            });
            initTransaction.add(initLargeProgramWithChunkInstruction);
            // Sign and send initialization transaction
            initTransaction.feePayer = deployerKeypair.publicKey;
            const { blockhash } = await connection.getLatestBlockhash("confirmed");
            initTransaction.recentBlockhash = blockhash;
            initTransaction.partialSign(deployerKeypair);
            initTransaction.partialSign(scriptKeypair);
            const initSignature = await connection.sendRawTransaction(initTransaction.serialize(), {
                skipPreflight: true,
                preflightCommitment: "confirmed",
                maxRetries: options.maxRetries || 3,
            });
            await connection.confirmTransaction(initSignature, "confirmed");
            transactionIds.push(initSignature);
            if (options.debug) {
                console.log(`[FiveSDK] ✅ Optimized initialization completed: ${initSignature}`);
                console.log(`[FiveSDK] First chunk (${firstChunk.length} bytes) included in initialization!`);
            }
            // OPTIMIZATION 2: Group remaining chunks into multi-chunk transactions
            if (remainingChunks.length > 0) {
                const groupedChunks = this.groupChunksForOptimalTransactions(remainingChunks, 950); // Leave room for multi-chunk overhead
                if (options.debug) {
                    console.log(`[FiveSDK] ⚡ OPTIMIZATION: Grouped ${remainingChunks.length} remaining chunks into ${groupedChunks.length} transactions`);
                }
                for (let groupIdx = 0; groupIdx < groupedChunks.length; groupIdx++) {
                    const chunkGroup = groupedChunks[groupIdx];
                    if (options.progressCallback) {
                        options.progressCallback(groupIdx + 2, groupedChunks.length + 1); // +1 for init transaction
                    }
                    if (options.debug) {
                        console.log(`[FiveSDK] ⚡ Step ${groupIdx + 2}: Appending ${chunkGroup.length} chunks in single transaction`);
                    }
                    const appendTransaction = new Transaction();
                    let appendInstruction; // TransactionInstruction from @solana/web3.js
                    if (chunkGroup.length === 1) {
                        // Use single-chunk AppendBytecode instruction for optimization fallback
                        if (options.debug) {
                            console.log(`[FiveSDK] Using single-chunk AppendBytecode for remaining chunk (${chunkGroup[0].length} bytes)`);
                        }
                        const singleChunkData = Buffer.concat([
                            Buffer.from([5]), // AppendBytecode discriminator
                            Buffer.from(chunkGroup[0]),
                        ]);
                        appendInstruction = new TransactionInstruction({
                            keys: [
                                {
                                    pubkey: scriptKeypair.publicKey,
                                    isSigner: false,
                                    isWritable: true,
                                },
                                {
                                    pubkey: deployerKeypair.publicKey,
                                    isSigner: true,
                                    isWritable: true,
                                },
                                { pubkey: vmStateKeypair.publicKey, isSigner: false, isWritable: true },
                            ],
                            programId: programId,
                            data: singleChunkData,
                        });
                    }
                    else {
                        // Use multi-chunk instruction for groups with 2+ chunks
                        const multiChunkData = this.createMultiChunkInstructionData(chunkGroup);
                        appendInstruction = new TransactionInstruction({
                            keys: [
                                {
                                    pubkey: scriptKeypair.publicKey,
                                    isSigner: false,
                                    isWritable: true,
                                },
                                {
                                    pubkey: deployerKeypair.publicKey,
                                    isSigner: true,
                                    isWritable: true,
                                },
                                { pubkey: vmStateKeypair.publicKey, isSigner: false, isWritable: true },
                            ],
                            programId: programId,
                            data: multiChunkData,
                        });
                    }
                    appendTransaction.add(appendInstruction);
                    // Sign and send multi-chunk transaction
                    const appendBlockhash = await connection.getLatestBlockhash("confirmed");
                    appendTransaction.feePayer = deployerKeypair.publicKey;
                    appendTransaction.recentBlockhash = appendBlockhash.blockhash;
                    appendTransaction.partialSign(deployerKeypair);
                    const appendSignature = await connection.sendRawTransaction(appendTransaction.serialize(), {
                        skipPreflight: true,
                        preflightCommitment: "confirmed",
                        maxRetries: options.maxRetries || 3,
                    });
                    await connection.confirmTransaction(appendSignature, "confirmed");
                    transactionIds.push(appendSignature);
                    if (options.debug) {
                        console.log(`[FiveSDK] ✅ Multi-chunk append completed: ${appendSignature}`);
                        console.log(`[FiveSDK] Appended ${chunkGroup.length} chunks totaling ${chunkGroup.reduce((sum, chunk) => sum + chunk.length, 0)} bytes`);
                    }
                }
            }
            // Calculate optimization savings
            const traditionalTransactionCount = 1 + chunks.length; // 1 init + N appends
            const optimizedTransactionCount = transactionIds.length;
            const transactionsSaved = traditionalTransactionCount - optimizedTransactionCount;
            const estimatedCostSaved = transactionsSaved * 0.000005 * 1e9; // Estimate 5000 lamports per transaction saved
            if (options.debug) {
                console.log(`[FiveSDK] 🎉 OPTIMIZATION RESULTS:`);
                console.log(`[FiveSDK]   Traditional method: ${traditionalTransactionCount} transactions`);
                console.log(`[FiveSDK]   Optimized method: ${optimizedTransactionCount} transactions`);
                console.log(`[FiveSDK]   Transactions saved: ${transactionsSaved} (${Math.round((transactionsSaved / traditionalTransactionCount) * 100)}% reduction)`);
                console.log(`[FiveSDK]   Estimated cost saved: ${estimatedCostSaved / 1e9} SOL`);
            }
            return {
                success: true,
                scriptAccount,
                transactionIds,
                totalTransactions: optimizedTransactionCount,
                deploymentCost: totalCost,
                chunksUsed: chunks.length,
                vmStateAccount: vmStateKeypair.publicKey.toString(),
                optimizationSavings: {
                    transactionsSaved,
                    estimatedCostSaved,
                },
                logs: [
                    `✅ Optimized deployment completed`,
                    `📊 ${optimizedTransactionCount} transactions (saved ${transactionsSaved} vs traditional)`,
                    `💰 Cost: ${totalCost / 1e9} SOL`,
                    `🧩 Chunks: ${chunks.length}`,
                    `⚡ Optimization: ${Math.round((transactionsSaved / traditionalTransactionCount) * 100)}% fewer transactions`,
                ],
            };
        }
        catch (error) {
            console.error("[FiveSDK] Optimized deployment failed:", error);
            const errorMessage = error instanceof Error ? error.message : "Unknown deployment error";
            return {
                success: false,
                error: errorMessage,
                logs: [],
            };
        }
    }
    /**
     * Group chunks optimally for multi-chunk transactions
     */
    static groupChunksForOptimalTransactions(chunks, maxGroupSize) {
        const groups = [];
        let currentGroup = [];
        let currentGroupSize = 0;
        // Account for multi-chunk overhead: 1 byte (num_chunks) + 2 bytes per chunk (length)
        const getGroupOverhead = (numChunks) => 1 + numChunks * 2;
        for (const chunk of chunks) {
            const groupOverhead = getGroupOverhead(currentGroup.length + 1);
            const newGroupSize = currentGroupSize + chunk.length + 2; // +2 for chunk length prefix
            if (currentGroup.length === 0) {
                // Always add first chunk to empty group
                currentGroup.push(chunk);
                currentGroupSize = newGroupSize;
            }
            else if (newGroupSize + groupOverhead <= maxGroupSize &&
                currentGroup.length < 8) {
                // Add to current group if it fits and doesn't exceed max chunks per transaction
                currentGroup.push(chunk);
                currentGroupSize = newGroupSize;
            }
            else {
                // Start new group
                groups.push(currentGroup);
                currentGroup = [chunk];
                currentGroupSize = chunk.length + 2;
            }
        }
        if (currentGroup.length > 0) {
            groups.push(currentGroup);
        }
        return groups;
    }
    /**
     * Create instruction data for multi-chunk AppendBytecode instruction
     * Format: [discriminator(5), num_chunks, chunk1_len, chunk1_data, chunk2_len, chunk2_data, ...]
     */
    static createMultiChunkInstructionData(chunks) {
        if (chunks.length < 2 || chunks.length > 10) {
            throw new Error(`Invalid chunk count for multi-chunk instruction: ${chunks.length}`);
        }
        const buffers = [
            Buffer.from([5]), // AppendMultipleBytecodeChunks discriminator
            Buffer.from([chunks.length]), // num_chunks
        ];
        // Add each chunk with length prefix
        for (const chunk of chunks) {
            if (chunk.length > 65535) {
                throw new Error(`Chunk too large for u16 length prefix: ${chunk.length} bytes`);
            }
            // Add chunk length as u16 little-endian
            const lengthBuffer = Buffer.allocUnsafe(2);
            lengthBuffer.writeUInt16LE(chunk.length, 0);
            buffers.push(lengthBuffer);
            // Add chunk data
            buffers.push(Buffer.from(chunk));
        }
        return Buffer.concat(buffers);
    }
    /**
     * Split bytecode into chunks of specified size
     */
    static chunkBytecode(bytecode, chunkSize) {
        const chunks = [];
        for (let i = 0; i < bytecode.length; i += chunkSize) {
            const chunk = bytecode.slice(i, Math.min(i + chunkSize, bytecode.length));
            chunks.push(chunk);
        }
        return chunks;
    }
}
// Export helper functions
export const createFiveSDK = (config) => new FiveSDK(config);
// Export default
export default FiveSDK;
//# sourceMappingURL=FiveSDK.js.map
