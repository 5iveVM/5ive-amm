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

// Client-agnostic SDK - no direct Solana client dependencies
import {
  FiveSDKConfig,
  FiveScript,
  FiveBytecode,
  FiveScriptSource,
  ScriptAccount,
  CompilationOptions,
  CompilationResult,
  DeploymentOptions,
  SerializedDeployment,
  SerializedExecution,
  SerializableAccount,
  SerializedInstruction,
  ExecutionOptions,
  FIVE_VM_PROGRAM_ID,
  FiveSDKError,
  ExecutionSDKError,
  EncodedParameters,
  FiveCompiledFile,
  FiveFunction,
  FunctionNameEntry,
  FeeInformation,
} from "./types.js";
import { BytecodeCompiler } from "./compiler/BytecodeCompiler.js";
import { ParameterEncoder } from "./encoding/ParameterEncoder.js";
import { VLEEncoder } from './lib/vle-encoder.js';
import { PDAUtils, Base58Utils, RentCalculator } from "./crypto/index.js";
import {
  ScriptMetadataParser,
  MetadataCache,
  ScriptMetadata,
} from "./metadata/index.js";
import { normalizeAbiFunctions } from "./utils/abi.js";
import { validator, Validators } from "./validation/index.js";

/**
 * Main Five SDK class - entry point for all Five VM interactions
 * Client-agnostic design: generates serialized transaction data for any Solana client library
 */
export class FiveSDK {
  private static compiler: BytecodeCompiler | null = null;
  private static parameterEncoder: ParameterEncoder | null = null;
  private static metadataCache: MetadataCache = new MetadataCache();

  private fiveVMProgramId: string;
  private debug: boolean;
  private network?: string;

  /**
   * Create a new Five SDK instance (for configuration)
   */
  constructor(config: FiveSDKConfig = {}) {
    this.fiveVMProgramId = config.fiveVMProgramId || FIVE_VM_PROGRAM_ID;
    this.debug = config.debug || false;
    this.network = (config as any).network; // Cast to handle network property

    if (this.debug) {
      console.log(
        `[FiveSDK] Initialized with Five VM Program: ${this.fiveVMProgramId}`,
      );
    }
  }

  /**
   * Get SDK configuration
   */
  getConfig(): FiveSDKConfig & { network?: string } {
    return {
      fiveVMProgramId: this.fiveVMProgramId,
      debug: this.debug,
      network: this.network,
    };
  }

  /**
   * Initialize static components (lazy initialization)
   */
  private static async initializeComponents(debug = false) {
    if (!this.compiler) {
      this.compiler = new BytecodeCompiler({ debug });
    }
    if (!this.parameterEncoder) {
      this.parameterEncoder = new ParameterEncoder(debug);
    }
  }

  /**
   * Poll for transaction confirmation with extended timeout
   * Handles cases where the validator is slow to include transactions
   */
  private static async pollForConfirmation(
    connection: any,
    signature: string,
    commitment: string = "confirmed",
    timeoutMs: number = 120000,
    debug: boolean = false
  ): Promise<{
    success: boolean;
    err?: any;
    error?: string;
  }> {
    const startTime = Date.now();
    const pollIntervalMs = 1000; // Poll every 1 second

    if (debug) {
      console.log(`[FiveSDK] Starting confirmation poll with ${timeoutMs}ms timeout`);
    }

    while (Date.now() - startTime < timeoutMs) {
      try {
        const confirmationStatus = await connection.getSignatureStatus(signature);

        if (debug && (Date.now() - startTime) % 10000 < 1000) {
          console.log(`[FiveSDK] Confirmation status: ${JSON.stringify(confirmationStatus.value)}`);
        }

        if (confirmationStatus.value) {
          // Transaction found in the blockchain
          if (confirmationStatus.value.confirmationStatus === commitment ||
              confirmationStatus.value.confirmations >= 1) {
            if (debug) {
              console.log(
                `[FiveSDK] Transaction confirmed after ${Date.now() - startTime}ms`
              );
            }
            return {
              success: true,
              err: confirmationStatus.value.err,
            };
          }
        }

        // Wait before polling again
        await new Promise(resolve => setTimeout(resolve, pollIntervalMs));
      } catch (error) {
        if (debug) {
          console.log(`[FiveSDK] Polling error: ${error instanceof Error ? error.message : String(error)}`);
        }
        // Continue polling despite errors
        await new Promise(resolve => setTimeout(resolve, pollIntervalMs));
      }
    }

    // Timeout reached
    const elapsed = Date.now() - startTime;
    if (debug) {
      console.log(`[FiveSDK] Confirmation polling timeout after ${elapsed}ms`);
    }

    return {
      success: false,
      error: `Transaction confirmation timeout after ${elapsed}ms. Signature: ${signature}`,
    };
  }


  // ==================== Static Factory Methods ====================

  /**
   * Create SDK instance with default configuration
   */
  static create(
    options: { debug?: boolean; fiveVMProgramId?: string } = {},
  ): FiveSDK {
    return new FiveSDK({
      debug: options.debug || false,
      fiveVMProgramId: options.fiveVMProgramId,
    });
  }

  /**
   * Create SDK instance for devnet
   */
  static devnet(
    options: { debug?: boolean; fiveVMProgramId?: string } = {},
  ): FiveSDK {
    return new FiveSDK({
      debug: options.debug || false,
      fiveVMProgramId: options.fiveVMProgramId,
      network: "devnet",
    });
  }

  /**
   * Create SDK instance for mainnet
   */
  static mainnet(
    options: { debug?: boolean; fiveVMProgramId?: string } = {},
  ): FiveSDK {
    return new FiveSDK({
      debug: options.debug || false,
      fiveVMProgramId: options.fiveVMProgramId,
      network: "mainnet",
    });
  }

  /**
   * Create SDK instance for localnet
   */
  static localnet(
    options: { debug?: boolean; fiveVMProgramId?: string } = {},
  ): FiveSDK {
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
  static async compile(
    source: FiveScriptSource | string,
    options: CompilationOptions & { debug?: boolean } = {},
  ): Promise<CompilationResult> {
    const sourceContent = typeof source === 'string' ? source : source.content;
    const sourceFilename = typeof source === 'string' ? 'unknown.v' : source.filename || 'unknown.v';
    const sourceLength = sourceContent.length;

    // Input validation
    Validators.sourceCode(sourceContent);
    Validators.options(options);

    await this.initializeComponents(options.debug);

    if (options.debug) {
      console.log(`[FiveSDK] Compiling script (${sourceLength} chars)...`);
    }

    try {
      const result = await this.compiler!.compile(source, options);

      // Generate .five format if compilation successful
      if (result.success && result.bytecode) {
        if (options.debug) {
          console.log(
            "[FiveSDK] Debug - result.metadata:",
            JSON.stringify(result.metadata, null, 2),
          );
          console.log(
            "[FiveSDK] Debug - result.abi:",
            JSON.stringify(result.abi, null, 2),
          );
        }

        let abiData: any = result.abi ?? { functions: [], fields: [] };
        if (options.debug) {
          try {
            const generatedABI = await this.compiler!.generateABI(source);
            if (generatedABI && generatedABI.functions) {
              abiData = generatedABI;
              console.log(
                "[FiveSDK] Generated ABI:",
                JSON.stringify(abiData, null, 2),
              );
            }
          } catch (abiError) {
            console.warn(
              "[FiveSDK] ABI generation failed, using compiler ABI:",
              abiError,
            );
          }
        }

        // Use generated ABI functions, fallback to empty array
        const functions = normalizeAbiFunctions(
          abiData.functions ?? abiData,
        ).map<FiveFunction>((func) => ({
          name: func.name,
          index: func.index,
          parameters:
            func.parameters?.map((param) => ({
              name: param.name,
              type: param.type as any,
              optional: param.optional ?? false,
            })) || [],
          returnType: func.returnType as any,
        }));

        result.fiveFile = {
          filename: sourceFilename,
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
          metrics: options.includeMetrics ? {
            compilationTime: result.metadata?.compilationTime || 0,
            memoryUsed: 0,
            optimizationTime: 0,
            bytecodeSize: result.bytecode?.length || 0,
            instructionCount: 0,
            functionCount: 0
          } : undefined,
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
          console.log(
            `[FiveSDK] Compilation successful: ${result.bytecode?.length} bytes`,
          );
        } else {
          console.log(
            `[FiveSDK] Compilation failed: ${result.errors?.length} errors`,
          );
        }
      }

      return result;
    } catch (error) {
      throw new FiveSDKError(
        `Compilation failed: ${error instanceof Error ? error.message : "Unknown error"}`,
        "COMPILATION_ERROR",
        // The original instruction was to replace the context object with a const declaration.
        // This would lead to a syntax error. To maintain syntactic correctness as per instructions,
        // and assuming the intent was to add this line for context, it's placed as a string.
        // If the intent was to execute this line, it would need to be outside the FiveSDKError constructor.
        `const nameMatch = source.content.substring(0, 500).match(/program\\s+([a-zA-Z0-9_]+)/);`
      );
    }
  }

  /**
   * Compile multiple modules (entry + dependencies)
   */
  static async compileModules(
    mainSource: FiveScriptSource | string,
    modules: Array<{ name: string; source: string }>,
    options: CompilationOptions & { debug?: boolean } = {},
  ): Promise<CompilationResult> {
    const mainSourceObj = typeof mainSource === 'string' ? { content: mainSource, filename: 'main.v' } : mainSource;

    try {
      Validators.options(options);
      await this.initializeComponents(options.debug);

      if (options.debug) {
        console.log(`[FiveSDK] Compiling modules for: ${mainSourceObj.filename}`);
      }

      if (!this.compiler) {
        throw new FiveSDKError("Compiler not initialized", "COMPILER_ERROR");
      }

      const result = await this.compiler.compileModules(mainSourceObj, modules, options);

      if (options.debug) {
        if (result.success) {
          console.log(
            `[FiveSDK] Module compilation successful: ${result.bytecode?.length} bytes`,
          );
        } else {
          console.log(
            `[FiveSDK] Module compilation failed: ${result.errors?.length} errors`,
          );
        }
      }

      return result;
    } catch (error) {
      throw new FiveSDKError(
        `Compilation failed: ${error instanceof Error ? error.message : "Unknown error"}`,
        "COMPILATION_ERROR",
        { source: mainSourceObj.content.substring(0, 100) + "...", options },
      );
    }
  }

  /**
   * Compile with automatic module discovery (client-agnostic)
   */
  static async compileWithDiscovery(
    entryPoint: string, // File path
    options: CompilationOptions & { debug?: boolean } = {},
  ): Promise<CompilationResult> {
    // Input validation
    Validators.options(options);

    await this.initializeComponents(options.debug);

    if (options.debug) {
      console.log(`[FiveSDK] Compiling with discovery: ${entryPoint}`);
    }

    try {
      // Access the compiler's compileWithDiscovery method directly if available
      if (typeof (this.compiler as any).compileWithDiscovery === 'function') {
        const result = await (this.compiler as any).compileWithDiscovery(entryPoint, options);

        if (options.debug) {
          console.log(`[FiveSDK] Discovery compilation ${result.success ? "succeeded" : "failed"}`);
        }

        return result;
      } else {
        // Fallback to compileFile if discovery not available (older compiler version)
        console.warn("[FiveSDK] compileWithDiscovery not available in current compiler version, falling back to compileFile");
        return this.compileFile(entryPoint, options);
      }
    } catch (error) {
      throw new FiveSDKError(
        `Discovery compilation failed: ${error instanceof Error ? error.message : "Unknown error"}`,
        "COMPILATION_ERROR",
        { source: entryPoint, options },
      );
    }
  }

  /**
   * Discover modules from entry point
   */
  static async discoverModules(
    entryPoint: string,
    options: { debug?: boolean } = {}
  ): Promise<string[]> {
    await this.initializeComponents(options.debug);

    try {
      if (typeof (this.compiler as any).discoverModules === 'function') {
        return await (this.compiler as any).discoverModules(entryPoint);
      } else {
        throw new Error("discoverModules not available in current compiler version");
      }
    } catch (error) {
      throw new FiveSDKError(
        `Module discovery failed: ${error instanceof Error ? error.message : "Unknown error"}`,
        "COMPILATION_ERROR",
        { source: entryPoint },
      );
    }
  }

  /**
   * Compile script from file path (static method)
   */
  static async compileFile(
    filePath: string,
    options: CompilationOptions & { debug?: boolean } = {},
  ): Promise<CompilationResult> {
    // Input validation
    Validators.filePath(filePath);
    Validators.options(options);

    await this.initializeComponents(options.debug);

    if (options.debug) {
      console.log(`[FiveSDK] Compiling file: ${filePath}`);
    }

    return this.compiler!.compileFile(filePath, options);
  }

  // ==================== Five File Format Utilities ====================

  /**
   * Load .five file and extract components
   */
  static async loadFiveFile(fileContent: string): Promise<{
    bytecode: FiveBytecode;
    abi: any;
    debug?: any;
  }> {
    try {
      const fiveFile: FiveCompiledFile = JSON.parse(fileContent);

      if (!fiveFile.bytecode || !fiveFile.abi) {
        throw new Error("Invalid .five file format: missing bytecode or ABI");
      }

      const bytecode = new Uint8Array(Buffer.from(fiveFile.bytecode as any, "base64"));

      return {
        bytecode,
        abi: fiveFile.abi,
        debug: fiveFile.debug,
      };
    } catch (error) {
      throw new FiveSDKError(
        `Failed to load .five file: ${error instanceof Error ? error.message : "Unknown error"}`,
        "FILE_LOAD_ERROR",
      );
    }
  }

  /**
   * Extract bytecode from .five file for deployment
   */
  static extractBytecode(fiveFile: FiveCompiledFile): FiveBytecode {
    return new Uint8Array(Buffer.from(fiveFile.bytecode, "base64"));
  }

  /**
   * Resolve function name to index using ABI
   */
  static resolveFunctionIndex(abi: any, functionName: string): number {
    if (!abi || !abi.functions) {
      throw new Error(
        "No ABI information available for function name resolution",
      );
    }

    // Handle both array format: [{ name: "add", index: 0 }] and object format: { "add": { index: 0 } }
    if (Array.isArray(abi.functions)) {
      // Array format (legacy)
      const func = abi.functions.find((f: any) => f.name === functionName);
      if (!func) {
        const availableFunctions = abi.functions
          .map((f: any) => f.name)
          .join(", ");
        throw new Error(
          `Function '${functionName}' not found in ABI. Available functions: ${availableFunctions}`,
        );
      }
      return func.index;
    } else {
      // Object format (new WASM ABI)
      const func = abi.functions[functionName];
      if (!func) {
        const availableFunctions = Object.keys(abi.functions).join(", ");
        throw new Error(
          `Function '${functionName}' not found in ABI. Available functions: ${availableFunctions}`,
        );
      }
      return func.index;
    }
  }

  // ==================== WASM VM Direct Execution (Local Testing) ====================

  /**
   * Execute bytecode directly using WASM VM for local testing and development
   * This bypasses Solana entirely - no network connection needed!
   */
  static async executeLocally(
    bytecode: FiveBytecode,
    functionName: string | number,
    parameters: any[] = [],
    options: {
      debug?: boolean;
      trace?: boolean;
      computeUnitLimit?: number;
      abi?: any; // Optional ABI for function name resolution
      accounts?: string[]; // Account addresses for execution context
    } = {},
  ): Promise<{
    success: boolean;
    result?: any;
    logs?: string[];
    computeUnitsUsed?: number;
    executionTime?: number;
    error?: string;
    trace?: any[];
  }> {
    // Input validation
    Validators.bytecode(bytecode);
    Validators.functionRef(functionName);
    Validators.parameters(parameters);
    Validators.options(options);

    const startTime = Date.now();

    if (options.debug) {
      console.log(
        `[FiveSDK] Executing locally: function=${functionName}, params=${parameters.length}`,
      );
      console.log(`[FiveSDK] Parameters:`, parameters);
    }

    try {
      // Load WASM VM
      const wasmVM = await this.loadWasmVM();

      // Resolve function name to index if needed
      let resolvedFunctionIndex: number;
      if (typeof functionName === "number") {
        resolvedFunctionIndex = functionName;
      } else if (options.abi) {
        // Use provided ABI for function name resolution
        try {
          resolvedFunctionIndex = this.resolveFunctionIndex(
            options.abi,
            functionName,
          );
        } catch (resolutionError) {
          throw new FiveSDKError(
            `Function name resolution failed: ${resolutionError instanceof Error ? resolutionError.message : "Unknown error"}`,
            "FUNCTION_RESOLUTION_ERROR",
          );
        }
      } else {
        // No ABI provided and function name given - cannot resolve
        throw new FiveSDKError(
          `Cannot resolve function name '${functionName}' without ABI information. Please provide function index or use compileAndExecuteLocally() instead.`,
          "MISSING_ABI_ERROR",
        );
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

      // Execute using WASM VM with proper VLE parameter encoding
      const transformedParams = parameters.map((param, index) => ({
        type: this.inferParameterType(param),
        value: param,
      }));

      if (options.debug) {
        console.log(
          `[FiveSDK] Resolved function index: ${resolvedFunctionIndex}`,
        );
        console.log(`[FiveSDK] Transformed parameters:`, transformedParams);
      }

      // Convert account addresses to AccountInfo format if provided
      let accountInfos: any[] = [];
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
          console.log(
            `[FiveSDK] Passing ${accountInfos.length} accounts to WASM VM execution`
          );
          accountInfos.forEach((acc, i) => {
            console.log(
              `  Account ${i}: ${acc.key.substring(0, 8)}... (signer=${acc.isSigner}, writable=${acc.isWritable})`
            );
          });
        }
      }

      const result = await wasmVM.executeFunction(
        bytecode,
        resolvedFunctionIndex,
        transformedParams,
        accountInfos.length > 0 ? accountInfos : undefined
      );

      const executionTime = Date.now() - startTime;

      if (options.debug) {
        console.log(
          `[FiveSDK] Local execution ${result.success ? "completed" : "failed"} in ${executionTime}ms`,
        );
        if (result.computeUnitsUsed) {
          console.log(
            `[FiveSDK] Compute units used: ${result.computeUnitsUsed}`,
          );
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
    } catch (error) {
      const executionTime = Date.now() - startTime;
      const errorMessage =
        error instanceof Error ? error.message : "Unknown execution error";

      if (options.debug) {
        console.log(
          `[FiveSDK] Local execution failed after ${executionTime}ms: ${errorMessage}`,
        );
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
  static async execute(
    source: FiveScriptSource | string,
    functionName: string | number,
    parameters: any[] = [],
    options: {
      debug?: boolean;
      trace?: boolean; // Add trace
      optimize?: boolean;
      computeUnitLimit?: number;
      vmStateAccount?: string;
      accounts?: string[];
    } = {},
  ) {
    const sourceContent = typeof source === 'string' ? source : source.content;

    // Input validation
    Validators.sourceCode(sourceContent);
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
    // Execute the compiled bytecode
    const execution = await this.executeLocally(
      compilation.bytecode,
      functionName,
      parameters,
      {
        debug: options.debug,
        trace: options.trace,
        computeUnitLimit: options.computeUnitLimit,
        accounts: options.accounts,
        abi: compilation.abi, // Pass ABI from compilation for function name resolution
      },
    );

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
  static async validateBytecode(
    bytecode: FiveBytecode,
    options: { debug?: boolean } = {},
  ): Promise<{
    valid: boolean;
    errors?: string[];
    metadata?: any;
    functions?: any[];
  }> {
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
        console.log(
          `[FiveSDK] Validation ${validation.valid ? "passed" : "failed"}`,
        );
      }

      return validation;
    } catch (error) {
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
   * 3. Optionally estimating deployment fees (automatic if connection provided)
   *
   * Fee estimation is enabled by default when a connection is provided.
   * Pass estimateFees: false to disable fee calculation.
   */
  static async generateDeployInstruction(
    bytecode: FiveBytecode,
    deployer: string, // base58 pubkey string
    options: DeploymentOptions & { debug?: boolean } = {},
    connection?: any, // Optional connection for fee estimation
  ): Promise<SerializedDeployment> {
    // Input validation
    Validators.bytecode(bytecode);
    validator.validateBase58Address(deployer, "deployer");
    Validators.options(options);
    if (options.scriptAccount) {
      validator.validateBase58Address(
        options.scriptAccount,
        "options.scriptAccount",
      );
    }

    await this.initializeComponents(options.debug);

    if (options.debug) {
      console.log(
        `[FiveSDK] Generating deployment transaction (${bytecode.length} bytes)...`,
      );
    }

    // Derive script account with seed
    const scriptResult = await PDAUtils.deriveScriptAccount(
      bytecode,
      FIVE_VM_PROGRAM_ID,
    );
    const scriptAccount = scriptResult.address;
    const scriptSeed = scriptResult.seed;

    // Derive VM state PDA
    const vmStatePDA = await this.deriveVMStatePDA();

    if (options.debug) {
      console.log(
        `[FiveSDK] Script Account: ${scriptAccount} (seed: ${scriptSeed})`,
      );
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

    // Add admin account if provided (required if deploy fees are enabled)
    if (options.adminAccount) {
      deployAccounts.push({
        pubkey: options.adminAccount,
        isSigner: false,
        isWritable: true,
      });
    }

    // Encode deployment instruction data
    const instructionData = this.encodeDeployInstruction(bytecode, options.permissions || 0);

    // Create the deployment result with setup instructions
    const result: SerializedDeployment = {
      programId: FIVE_VM_PROGRAM_ID, // Added top-level consistency
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
      adminAccount: options.adminAccount,
    };

    if (options.debug) {
      console.log(`[FiveSDK] Generated deployment transaction:`, {
        scriptAccount,
        scriptSeed,
        accountSize: totalAccountSize,
        rentCost: rentLamports,
        deployDataSize: instructionData.length,
        adminAccount: options.adminAccount,
      });
    }

    // Automatically calculate and include fee information if connection provided
    // User can explicitly disable with estimateFees: false
    const shouldEstimateFees = options.estimateFees !== false && connection;

    if (shouldEstimateFees) {
      try {
        const deployFee = await this.calculateDeployFee(
          bytecode.length,
          connection,
          options.fiveVMProgramId || FIVE_VM_PROGRAM_ID,
        );
        result.feeInformation = deployFee;

        if (options.debug) {
          console.log(`[FiveSDK] Deploy fee estimate:`, deployFee);
        }
      } catch (error) {
        if (options.debug) {
          console.warn(
            `[FiveSDK] Could not estimate deploy fees:`,
            error instanceof Error ? error.message : "Unknown error",
          );
        }
      }
    }

    return result;
  }

  // ==================== Serialized Execution ====================

  /**
   * Generate execution instruction data (static method)
   *
   * Fee estimation is enabled by default when a connection is provided.
   * Pass estimateFees: false in options to disable fee calculation.
   */
  static async generateExecuteInstruction(
    scriptAccount: string, // base58 pubkey string
    functionName: string | number,
    parameters: any[] = [],
    accounts: string[] = [], // base58 pubkey strings
    connection?: any, // Optional Solana connection for metadata lookup
    options: {
      debug?: boolean;
      computeUnitLimit?: number;
      vmStateAccount?: string;
      fiveVMProgramId?: string;
      abi?: any; // Optional ABI for parameter encoding
      adminAccount?: string; // Admin account for fee collection
      estimateFees?: boolean; // Request fee estimation
    } = {},
  ): Promise<SerializedExecution> {
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

    // Handle missing metadata gracefully - generate parameters for VLE encoding
    let functionIndex: number;
    let encodedParams: Uint8Array;
    let actualParamCount: number = 0;
    let funcDef: any = null;

    try {
      // Use provided ABI if available, otherwise try to load from chain
      let scriptMetadata = options.abi;

      if (!scriptMetadata) {
        // Try to load script metadata for ABI-driven parameter encoding
        scriptMetadata = await this.getScriptMetadata(
          scriptAccount,
          connection,
        );
      } else if (options.debug) {
        console.log(`[FiveSDK] Using provided ABI with ${Array.isArray(scriptMetadata.functions) ? scriptMetadata.functions.length : Object.keys(scriptMetadata.functions || {}).length} functions`);
      }

      // Normalize ABI functions to array format if needed
      if (Array.isArray(scriptMetadata.functions)) {
        // ABI is already in array format, keep it as is
      } else if (typeof scriptMetadata.functions === 'object' && scriptMetadata.functions !== null) {
        // Convert object format to array format
        scriptMetadata.functions = Object.entries(scriptMetadata.functions).map(([name, func]: [string, any]) => ({
          name,
          ...(func || {}),
        }));
      }

      // Resolve function index
      functionIndex =
        typeof functionName === "number"
          ? functionName
          : FiveSDK.resolveFunctionIndex(scriptMetadata, functionName);

      // Get function definition - handle array format
      funcDef = Array.isArray(scriptMetadata.functions)
        ? scriptMetadata.functions.find((f: any) => f.index === functionIndex)
        : scriptMetadata.functions[functionIndex];

      // Encode all parameters (including accounts) with ABI guidance
      const paramDefs = (funcDef.parameters || []);
      actualParamCount = paramDefs.length;
      encodedParams = await this.encodeParametersWithABI(
        parameters,
        funcDef,
        functionIndex,
        accounts,
        options,
      );
    } catch (metadataError) {
      if (options.debug) {
        console.log(
          `[FiveSDK] Metadata not available, using VLE encoding with assumed parameter types`,
        );
        console.log(`[FiveSDK] ABI processing error:`, metadataError);
      }

      // GRACEFUL HANDLING: Use VLE encoding without metadata
      functionIndex = typeof functionName === "number" ? functionName : 0;

      // Create parameter definitions for VLE encoding (assume all u64)
      const paramDefs = parameters.map((_, index) => ({
        name: `param${index}`,
        type: "u64",
      }));

      const paramValues: Record<string, any> = {};
      paramDefs.forEach((param, index) => {
        paramValues[param.name] = parameters[index];
      });

      if (options.debug) {
        console.log(
          `[FiveSDK] About to call VLEEncoder.encodeExecuteVLE with:`,
          {
            functionIndex,
            paramDefs,
            paramValues,
          },
        );
      }

      actualParamCount = paramDefs.length;
      encodedParams = await VLEEncoder.encodeExecuteVLE(
        functionIndex,
        paramDefs,
        paramValues,
        true,
        options,
      );

      if (options.debug) {
        console.log(`[FiveSDK] VLE encoder returned:`, {
          encodedLength: encodedParams.length,
          encodedBytes: Array.from(encodedParams),
          hex: Buffer.from(encodedParams).toString("hex"),
        });
      }
    }

    // Derive VM state PDA - required for all Five VM executions
    const vmStatePDA = await this.deriveVMStatePDA();

    // Build account list with required VM state PDA
    const vmState = options.vmStateAccount || vmStatePDA;
    if (options.debug) {
      console.log(
        `[FiveSDK] Using VM state account: ${vmState} (override: ${options.vmStateAccount ? "yes" : "no"})`,
      );
    }

    // Auto-resolve admin account from VM state if not provided but connection exists
    let adminAccount = options.adminAccount;
    if (!adminAccount && connection) {
      try {
        // Use the override or derive it using PDAUtils to support custom program ID
        let vmStateAddress = options.vmStateAccount;
        if (!vmStateAddress) {
          const pda = await PDAUtils.deriveVMStatePDA(options.fiveVMProgramId || FIVE_VM_PROGRAM_ID);
          vmStateAddress = pda.address;
        }

        // Dynamically import PublicKey
        const { PublicKey } = await import("@solana/web3.js");
        const info = await connection.getAccountInfo(new PublicKey(vmStateAddress));

        if (info) {
          const data = new Uint8Array(info.data);
          // Authority is first 32 bytes (based on getVMState logic)
          if (data.length >= 32) {
            const authorityPubkey = new PublicKey(data.slice(0, 32));
            adminAccount = authorityPubkey.toBase58();

            if (options.debug) {
              console.log(`[FiveSDK] Resolved admin (fee recipient) from on-chain state: ${adminAccount}`);
            }
          }
        } else if (options.debug) {
          console.warn(`[FiveSDK] VM State account not found at ${vmStateAddress}`);
        }
      } catch (error) {
        if (options.debug) {
          console.warn(`[FiveSDK] Failed to resolve admin account from VM state:`, error);
        }
      }
    }

    // Resolve proper account attributes (signer/writable) using ABI if available
    const instructionAccounts = [
      { pubkey: scriptAccount, isSigner: false, isWritable: false },
      { pubkey: vmState, isSigner: false, isWritable: true }, // VM state (required!)
    ];

    // Build map of pubkey strings to their metadata from ABI
    const abiAccountMetadata = new Map<string, { isSigner: boolean; isWritable: boolean }>();

    if (funcDef && funcDef.parameters) {
      // First pass: detect if there's an @init constraint and find the payer
      let hasInit = false;
      let payerPubkey: string | undefined;
      for (let i = 0; i < funcDef.parameters.length; i++) {
        const param = funcDef.parameters[i];
        if (param.is_account || param.isAccount) {
          const attributes = param.attributes || [];
          if (attributes.includes('init')) {
            hasInit = true;
            // Find the payer - typically the @signer in an @init context
            for (let j = 0; j < funcDef.parameters.length; j++) {
              const payerParam = funcDef.parameters[j];
              if (
                i !== j &&
                (payerParam.is_account || payerParam.isAccount) &&
                (payerParam.attributes || []).includes('signer')
              ) {
                const payerValue = parameters[j];
                payerPubkey = payerValue?.toString();
                if (options.debug) {
                  console.log(`[FiveSDK] Detected @init on '${param.name}', found payer: ${payerParam.name} = ${payerPubkey}`);
                }
                break;
              }
            }
            break;
          }
        }
      }

      // Second pass: build metadata
      funcDef.parameters.forEach((param: any, paramIndex: number) => {
        if (param.is_account || param.isAccount) {
          const value = parameters[paramIndex];
          const pubkey = value?.toString();
          if (pubkey) {
            const attributes = param.attributes || [];
            const isSigner = attributes.includes('signer');
            const isWritable = attributes.includes('mut') ||
                              attributes.includes('init') ||
                              // If this is the payer for @init, it must be writable
                              (hasInit && pubkey === payerPubkey);

            if (options.debug) {
              console.log(`[FiveSDK] ABI Metadata for param '${param.name}': pubkey=${pubkey}, signer=${isSigner}, writable=${isWritable}`);
            }

            const existing = abiAccountMetadata.get(pubkey) || { isSigner: false, isWritable: false };
            abiAccountMetadata.set(pubkey, {
              isSigner: existing.isSigner || isSigner,
              isWritable: existing.isWritable || isWritable
            });
          }
        }
      });
    }

    // Add user provided accounts with metadata merged from ABI
    const userInstructionAccounts = accounts.map((acc, index) => {
      const metadata = abiAccountMetadata.get(acc);
      const isSigner = metadata ? metadata.isSigner : (index === 0 && adminAccount ? true : false);
      const isWritable = metadata ? metadata.isWritable : true;

      if (options.debug) {
        console.log(`[FiveSDK] Instruction Account [${instructionAccounts.length + index}]: pubkey=${acc}, signer=${isSigner}, writable=${isWritable} (via ${metadata ? 'ABI' : 'Fallback'})`);
      }

      return {
        pubkey: acc,
        isSigner,
        isWritable
      };
    });

    instructionAccounts.push(...userInstructionAccounts);

    // Add admin account if resolved (required for fee collection)
    if (adminAccount) {
      // Ensure admin isn't already added. If it is, ensure it's writable
      const existingAdminIdx = instructionAccounts.findIndex(a => a.pubkey === adminAccount);
      if (existingAdminIdx === -1) {
        instructionAccounts.push({
          pubkey: adminAccount,
          isSigner: false,
          isWritable: true,
        });
      } else {
        instructionAccounts[existingAdminIdx].isWritable = true;
      }
    }

    // Encode execution instruction data
    const instructionData = this.encodeExecuteInstruction(
      functionIndex,
      encodedParams,
      actualParamCount,
    );

    const result: SerializedExecution = {
      instruction: {
        programId: options.fiveVMProgramId || FIVE_VM_PROGRAM_ID,
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
      estimatedComputeUnits:
        options.computeUnitLimit ||
        this.estimateComputeUnits(functionIndex, parameters.length),
      adminAccount: adminAccount,
    };

    if (options.debug) {
      console.log(`[FiveSDK] Generated execution instruction:`, {
        function: functionName,
        functionIndex,
        parameterBytes: encodedParams.length,
        dataSize: instructionData.length,
        estimatedCU: result.estimatedComputeUnits,
        adminAccount: options.adminAccount,
      });
    }

    // Automatically calculate and include fee information if connection provided
    // User can explicitly disable with estimateFees: false
    const shouldEstimateFees = options.estimateFees !== false && connection;

    if (shouldEstimateFees) {
      try {
        const executeFee = await this.calculateExecuteFee(
          connection,
          options.fiveVMProgramId || FIVE_VM_PROGRAM_ID,
        );
        result.feeInformation = executeFee;

        if (options.debug) {
          console.log(`[FiveSDK] Execute fee estimate:`, executeFee);
        }
      } catch (error) {
        if (options.debug) {
          console.warn(
            `[FiveSDK] Could not estimate execute fees:`,
            error instanceof Error ? error.message : "Unknown error",
          );
        }
      }
    }

    return result;
  }

  // ==================== VM State & Fees ====================

  /**
   * Fetch VM state from the blockchain
   */
  static async getVMState(connection: any, fiveVMProgramId?: string): Promise<{
    authority: string;
    scriptCount: number;
    deployFeeBps: number;
    executeFeeBps: number;
    isInitialized: boolean;
  }> {
    const programId = fiveVMProgramId || FIVE_VM_PROGRAM_ID;
    const vmStatePDA = await PDAUtils.deriveVMStatePDA(programId);

    let accountData: Uint8Array;
    try {
      if (typeof connection.getAccountInfo === 'function') {
        // Dynamically import PublicKey for web3.js compatibility
        let pubkey: any = vmStatePDA.address;
        try {
          const { PublicKey } = await import("@solana/web3.js");
          pubkey = new PublicKey(vmStatePDA.address);
        } catch { }

        const info = await connection.getAccountInfo(pubkey);
        if (!info) throw new Error("VM State account not found");
        accountData = new Uint8Array(info.data);
      } else if (typeof connection.getAccountData === 'function') {
        const info = await connection.getAccountData(vmStatePDA.address);
        if (!info) throw new Error("VM State account not found");
        accountData = new Uint8Array(info.data);
      } else {
        throw new Error("Invalid connection object: must support getAccountInfo or getAccountData");
      }

      if (accountData.length < 56) throw new Error(`VM State account data too small: expected 56, got ${accountData.length}`);

      const authority = Base58Utils.encode(accountData.slice(0, 32));
      const view = new DataView(accountData.buffer, accountData.byteOffset, accountData.byteLength);

      return {
        authority,
        scriptCount: Number(view.getBigUint64(32, true)),
        deployFeeBps: view.getUint32(40, true),
        executeFeeBps: view.getUint32(44, true),
        isInitialized: accountData[48] === 1
      };
    } catch (error) {
      throw new Error(`Failed to fetch VM state: ${error instanceof Error ? error.message : "Unknown error"}`);
    }
  }

  /**
   * Get current VM fees and admin account
   */
  static async getFees(connection: any, fiveVMProgramId?: string): Promise<{
    deployFeeBps: number;
    executeFeeBps: number;
    adminAccount: string | null;
  }> {
    try {
      const state = await this.getVMState(connection, fiveVMProgramId);
      return {
        deployFeeBps: state.deployFeeBps,
        executeFeeBps: state.executeFeeBps,
        adminAccount: state.authority
      };
    } catch (error) {
      // Graceful fallback for offline usage or uninitialized network
      return {
        deployFeeBps: 0,
        executeFeeBps: 0,
        adminAccount: null
      };
    }
  }

  /**
   * Calculate deployment fee based on bytecode size and current fee configuration
   * Fee = (rentExemption * deployFeeBps) / 10000
   */
  static async calculateDeployFee(
    bytecodeSize: number,
    connection?: any,
    fiveVMProgramId?: string,
  ): Promise<FeeInformation> {
    try {
      // Calculate rent exemption for the account
      // Account size = ScriptAccountHeader (64 bytes) + bytecode size
      const accountSize = 64 + bytecodeSize;
      const rentLamports = RentCalculator.calculateRentExemption(accountSize);

      // Get current fee configuration
      const vmState = await this.getVMState(connection, fiveVMProgramId);
      const deployFeeBps = vmState.deployFeeBps;

      // Calculate fee: (rentLamports * bps) / 10000
      const feeLamports = Math.floor((rentLamports * deployFeeBps) / 10000);

      return {
        feeBps: deployFeeBps,
        basisLamports: rentLamports,
        feeLamports,
        totalEstimatedCost: rentLamports + feeLamports,
        costBreakdown: {
          basis: RentCalculator.formatSOL(rentLamports),
          fee: RentCalculator.formatSOL(feeLamports),
          total: RentCalculator.formatSOL(rentLamports + feeLamports),
        },
      };
    } catch (error) {
      // Fallback: assume no fees if unable to fetch VM state
      const accountSize = 64 + bytecodeSize;
      const rentLamports = RentCalculator.calculateRentExemption(accountSize);

      return {
        feeBps: 0,
        basisLamports: rentLamports,
        feeLamports: 0,
        totalEstimatedCost: rentLamports,
        costBreakdown: {
          basis: RentCalculator.formatSOL(rentLamports),
          fee: "0 SOL",
          total: RentCalculator.formatSOL(rentLamports),
        },
      };
    }
  }

  /**
   * Calculate execution fee based on current fee configuration
   * Fee = (standardTxFee * executeFeeBps) / 10000
   * Standard transaction fee basis is 5000 lamports
   */
  static async calculateExecuteFee(
    connection?: any,
    fiveVMProgramId?: string,
  ): Promise<FeeInformation> {
    const STANDARD_TX_FEE = 5000; // lamports, matches on-chain constant

    try {
      // Get current fee configuration
      const vmState = await this.getVMState(connection, fiveVMProgramId);
      const executeFeeBps = vmState.executeFeeBps;

      // Calculate fee: (5000 * bps) / 10000
      const feeLamports = Math.floor((STANDARD_TX_FEE * executeFeeBps) / 10000);

      return {
        feeBps: executeFeeBps,
        basisLamports: STANDARD_TX_FEE,
        feeLamports,
        totalEstimatedCost: STANDARD_TX_FEE + feeLamports,
        costBreakdown: {
          basis: RentCalculator.formatSOL(STANDARD_TX_FEE),
          fee: RentCalculator.formatSOL(feeLamports),
          total: RentCalculator.formatSOL(STANDARD_TX_FEE + feeLamports),
        },
      };
    } catch (error) {
      // Fallback: assume no fees if unable to fetch VM state
      return {
        feeBps: 0,
        basisLamports: 5000,
        feeLamports: 0,
        totalEstimatedCost: 5000,
        costBreakdown: {
          basis: "0.000005 SOL",
          fee: "0 SOL",
          total: "0.000005 SOL",
        },
      };
    }
  }

  /**
   * Get comprehensive fee information for display
   * Combines deploy and execute fee calculations with admin account info
   */
  static async getFeeInformation(
    bytecodeSize: number,
    connection?: any,
    fiveVMProgramId?: string,
  ): Promise<{
    deploy: FeeInformation;
    execute: FeeInformation;
    adminAccount: string | null;
    feesEnabled: boolean;
  }> {
    try {
      const [deployFee, executeFee, vmState] = await Promise.all([
        this.calculateDeployFee(bytecodeSize, connection, fiveVMProgramId),
        this.calculateExecuteFee(connection, fiveVMProgramId),
        this.getVMState(connection, fiveVMProgramId),
      ]);

      const feesEnabled = vmState.deployFeeBps > 0 || vmState.executeFeeBps > 0;

      return {
        deploy: deployFee,
        execute: executeFee,
        adminAccount: vmState.authority,
        feesEnabled,
      };
    } catch (error) {
      // Fallback with zero fees
      const deployFee = await this.calculateDeployFee(
        bytecodeSize,
        connection,
        fiveVMProgramId,
      );
      const executeFee = await this.calculateExecuteFee(connection, fiveVMProgramId);

      return {
        deploy: deployFee,
        execute: executeFee,
        adminAccount: null,
        feesEnabled: false,
      };
    }
  }

  /**
   * Helper method to calculate fee from basis points
   * Private static method used internally
   */
  private static calculateFeeFromBps(amount: number, bps: number): number {
    return Math.floor((amount * bps) / 10000);
  }

  // ==================== Script Analysis ====================

  /**
   * Get script metadata for ABI-driven parameter encoding (static method)
   * Now uses real Solana account data parsing instead of mocks
   */
  static async getScriptMetadata(
    scriptAccount: string,
    connection?: any, // Optional connection for direct blockchain access
  ): Promise<{ functions: any[] }> {
    // Input validation
    validator.validateBase58Address(scriptAccount, "scriptAccount");

    try {
      if (connection) {
        // Use real blockchain data if connection provided
        const metadata = await ScriptMetadataParser.getScriptMetadata(
          connection,
          scriptAccount,
        );
        const normalizedFunctions = normalizeAbiFunctions(
          metadata.abi?.functions ?? metadata.abi,
        );
        return {
          functions: normalizedFunctions.map((func) => ({
            name: func.name,
            index: func.index,
            parameters: func.parameters,
            returnType: func.returnType,
            visibility: func.visibility,
          })),
        };
      } else {
        // Client-agnostic mode: metadata should be provided by client
        // This maintains the SDK's client-agnostic design
        throw new Error(
          "No connection provided for metadata retrieval. " +
          "In client-agnostic mode, provide script metadata directly or use getScriptMetadataWithConnection().",
        );
      }
    } catch (error) {
      throw new Error(
        `Failed to get script metadata: ${error instanceof Error ? error.message : "Unknown error"}`,
      );
    }
  }

  /**
   * Get script metadata with explicit connection (for use with any Solana client)
   */
  static async getScriptMetadataWithConnection(
    scriptAccount: string,
    connection: any,
  ): Promise<ScriptMetadata> {
    // Input validation
    validator.validateBase58Address(scriptAccount, "scriptAccount");

    return ScriptMetadataParser.getScriptMetadata(connection, scriptAccount);
  }

  /**
   * Parse script metadata from raw account data (client-agnostic)
   */
  static parseScriptMetadata(
    accountData: Uint8Array,
    address: string,
  ): ScriptMetadata {
    // Input validation
    Validators.bytecode(accountData); // Reuse bytecode validation for account data
    validator.validateBase58Address(address, "address");

    return ScriptMetadataParser.parseMetadata(accountData, address);
  }

  /**
   * Get script metadata with caching (for performance)
   */
  static async getCachedScriptMetadata(
    scriptAccount: string,
    connection: any,
    cacheTTL: number = 5 * 60 * 1000, // 5 minutes default
  ): Promise<ScriptMetadata> {
    // Input validation
    validator.validateBase58Address(scriptAccount, "scriptAccount");
    validator.validateNumber(cacheTTL, "cacheTTL");

    return this.metadataCache.getMetadata(
      scriptAccount,
      (address) => ScriptMetadataParser.getScriptMetadata(connection, address),
      cacheTTL,
    );
  }

  /**
   * Invalidate metadata cache for a script
   */
  static invalidateMetadataCache(scriptAccount: string): void {
    // Input validation
    validator.validateBase58Address(scriptAccount, "scriptAccount");

    this.metadataCache.invalidate(scriptAccount);
  }

  /**
   * Get metadata cache statistics
   */
  static getMetadataCacheStats(): any {
    return this.metadataCache.getStats();
  }

  // ==================== Private Utility Methods ====================

  /**
   * Derive script account PDA from bytecode using real Solana PDA derivation
   */
  private static async deriveScriptAccount(
    bytecode: FiveBytecode,
  ): Promise<string> {
    const result = await PDAUtils.deriveScriptAccount(bytecode);
    return result.address;
  }

  /**
   * Derive VM state PDA using hardcoded seed (matches Five VM program)
   */
  private static async deriveVMStatePDA(): Promise<string> {
    const result = await PDAUtils.deriveVMStatePDA(FIVE_VM_PROGRAM_ID);
    return result.address;
  }

  /**
   * Load WASM VM for direct execution
   */
  private static wasmVMInstance: any = null;

  private static async loadWasmVM(): Promise<any> {
    if (this.wasmVMInstance) {
      return this.wasmVMInstance;
    }

    try {
      // Import existing WASM VM from five-cli infrastructure
      const { FiveVM } = await import('./wasm/vm.js');

      // Create a simple logger for WASM VM
      const logger = {
        debug: (msg: string) => console.debug("[WASM VM]", msg),
        info: (msg: string) => console.info("[WASM VM]", msg),
        warn: (msg: string) => console.warn("[WASM VM]", msg),
        error: (msg: string) => console.error("[WASM VM]", msg),
      };
      this.wasmVMInstance = new FiveVM(logger); // Initialize WASM VM
      if (this.wasmVMInstance.initialize) {
        await this.wasmVMInstance.initialize();
      }

      return this.wasmVMInstance;
    } catch (error) {
      throw new FiveSDKError(
        `Failed to load WASM VM: ${error instanceof Error ? error.message : "Unknown error"}`,
        "WASM_LOAD_ERROR",
      );
    }
  }

  /**
   * Calculate rent exemption for account size using real Solana rent calculations
   */
  private static async calculateRentExemption(
    dataSize: number,
  ): Promise<number> {
    return RentCalculator.calculateRentExemption(dataSize);
  }

  /**
   * Encode deployment instruction data
   */
  private static encodeDeployInstruction(
    bytecode: FiveBytecode,
    permissions: number = 0
  ): Uint8Array {
    // Deploy instruction: [discriminator(8), bytecode_length(u32_le), permissions(u8), bytecode]
    // Format expected by Five VM Program (five-solana/src/instructions.rs):
    // - Discriminator: 8 (u8)
    // - Length: bytecode.length (u32 little-endian, 4 bytes)
    // - Permissions: 0x00 (1 byte)
    // - Bytecode: actual bytecode bytes

    // IMPORTANT: Use Buffer.writeUInt32LE() for proper u32 LE encoding
    // DataView with Buffer.allocUnsafe().buffer doesn't work correctly in Node.js
    const lengthBuffer = Buffer.allocUnsafe(4);
    lengthBuffer.writeUInt32LE(bytecode.length, 0);

    const result = new Uint8Array(1 + 4 + 1 + bytecode.length);
    result[0] = 8; // Deploy discriminator (matches on-chain FIVE program)
    result.set(new Uint8Array(lengthBuffer), 1); // u32 LE length at bytes 1-4
    result[5] = permissions; // permissions byte at byte 5
    result.set(bytecode, 6); // bytecode starts at byte 6

    console.log(`[FiveSDK] Deploy instruction encoded:`, {
      discriminator: result[0],
      lengthBytes: Array.from(new Uint8Array(lengthBuffer)),
      permissions: result[5],
      bytecodeLength: bytecode.length,
      totalInstructionLength: result.length,
      expectedFormat: `[8, ${bytecode.length}_as_u32le, 0x${permissions.toString(16).padStart(2, '0')}, bytecode_bytes]`,
      instructionHex:
        Buffer.from(result).toString("hex").substring(0, 20) + "...",
    });

    return result;
  }

  /**
   * Encode execution instruction data
   */
  private static encodeExecuteInstruction(
    functionIndex: number,
    encodedParams: Uint8Array,
    paramCount: number,
  ): Uint8Array {
    // Execute instruction format: [discriminator(9), VLE(func_idx), VLE(param_count), params...]
    // NOTE: For typed params, encodedParams starts with [VLE(128), VLE(count), typed_params...]
    // In that case, the sentinel (128) IS the param_count, so we should NOT add another one.

    const TYPED_PARAM_SENTINEL = 128;
    const isTypedParams = encodedParams.length > 0 && encodedParams[0] === TYPED_PARAM_SENTINEL;

    const parts = [];
    parts.push(new Uint8Array([9])); // Execute discriminator
    parts.push(FiveSDK.encodeVLENumber(functionIndex)); // VLE function index

    // SKIP: Prepend paramCount header if not already present
    // Typed parameters already include their own header (Sentinel + Count)
    if (isTypedParams) {
      // For typed params, encodedParams already contains [VLE(128), VLE(count), ...]
      // Don't add paramCount - the sentinel acts as the marker
      parts.push(encodedParams);
    } else {
      // For simple VLE params, add param count then raw params
      parts.push(FiveSDK.encodeVLENumber(paramCount)); // VLE param count
      parts.push(encodedParams); // Raw parameter data
    }

    const totalLength = parts.reduce((sum, part) => sum + part.length, 0);
    const result = new Uint8Array(totalLength);
    let resultOffset = 0;

    for (const part of parts) {
      result.set(part, resultOffset);
      resultOffset += part.length;
    }
    return result;
  }

  /**
   * Encode parameters with ABI guidance
   */
  private static async encodeParametersWithABI(
    parameters: any[],
    functionDef: any,
    functionIndex: number,
    accounts: string[] = [],
    options: any = {},
  ): Promise<Uint8Array> {
    if (!this.parameterEncoder) {
      await this.initializeComponents();
    }

    const isAccountParam = (param: any): boolean => {
      if (!param) {
        return false;
      }
      if (param.isAccount || param.is_account) {
        return true;
      }
      const type = (param.type || param.param_type || '').toString().trim().toLowerCase();
      // DSL types that map to accounts
      return type === 'account' || type === 'mint' || type === 'tokenaccount';
    };

    const isPubkeyParam = (param: any): boolean => {
      if (!param) {
        return false;
      }
      const type = (param.type || param.param_type || '').toString().trim().toLowerCase();
      return type === 'pubkey';
    };

    // Do not filter out account parameters anymore - Five VM now supports them
    const paramDefs = (functionDef.parameters || []);

    // Helper to find account index
    const getAccountIndex = (pubkeyOrIdx: any): number => {
      if (typeof pubkeyOrIdx === 'number') return pubkeyOrIdx;

      // Handle PublicKey objects - try multiple conversion methods
      let pubkeyStr: string;
      if (typeof pubkeyOrIdx === 'string') {
        pubkeyStr = pubkeyOrIdx;
      } else if (pubkeyOrIdx && typeof pubkeyOrIdx.toBase58 === 'function') {
        pubkeyStr = pubkeyOrIdx.toBase58();
      } else if (pubkeyOrIdx && typeof pubkeyOrIdx.toString === 'function') {
        pubkeyStr = pubkeyOrIdx.toString();
      } else {
        throw new Error(`Invalid account parameter: cannot convert to pubkey string: ${JSON.stringify(pubkeyOrIdx)}`);
      }

      // In execute instructions, indices 0 and 1 are Script and VM State
      // providedAccounts starts at index 2
      const idx = accounts.indexOf(pubkeyStr);
      if (idx !== -1) {
        if (options.debug) {
          console.log(`[FiveSDK] Mapped account ${pubkeyStr.slice(0, 8)}... to index ${idx + 2}`);
        }
        return idx + 2;
      }

      // CRITICAL: Don't silently fall back to 0 - this causes VM errors
      throw new Error(`Account parameter ${pubkeyStr.slice(0, 8)}... not found in accounts array. Available accounts: ${accounts.map((a: string) => a.slice(0, 8) + '...').join(', ')}`);
    };

    // Validate parameter count
    if (parameters.length !== paramDefs.length) {
      console.warn(
        `[FiveSDK] Parameter validation warning: Function '${functionDef.name}' expects ${paramDefs.length} parameters (including accounts), but received ${parameters.length}.`,
        {
          expected: paramDefs.map((p: any) => p.name),
          receivedCount: parameters.length
        }
      );
    }

    const paramValues: Record<string, any> = {};

    // Map parameters to names
    paramDefs.forEach((param: any, index: number) => {
      if (index < parameters.length) {
        let value = parameters[index];
        if (isAccountParam(param)) {
          // Convert PublicKey objects to base58 strings first
          let accountPubkey: string | null = null;
          if (value && typeof value === 'object' && typeof value.toBase58 === 'function') {
            accountPubkey = value.toBase58();
          } else if (typeof value === 'string') {
            accountPubkey = value;
          } else if (typeof value === 'number') {
            // If already an index, get the pubkey from accounts array
            if (value >= 0 && value < accounts.length) {
              accountPubkey = accounts[value];
            } else {
              throw new Error(`Account index ${value} out of bounds. Available accounts: ${accounts.length}`);
            }
          }

          // Find the account index in the accounts array
          if (accountPubkey) {
            const accountIndex = accounts.indexOf(accountPubkey);
            if (accountIndex >= 0) {
              // Pass the account index (not the pubkey string)
              // The VLEEncoder will wrap it with metadata so WASM knows it's an account
              // Add +2 offset to account for script account and VM state PDA that come first in transaction
              value = accountIndex + 2;
              if (options.debug) {
                console.log(`[FiveSDK] Parameter ${index} (${param.name}) is account type, mapped to transaction index: ${value}`);
              }
            } else {
              throw new Error(`Account ${accountPubkey} not found in accounts array`);
            }
          }
        } else if (isPubkeyParam(param)) {
          // Convert PublicKey objects to base58 strings for WASM encoder
          if (value && typeof value === 'object' && typeof value.toBase58 === 'function') {
            value = value.toBase58();
          }
        }
        paramValues[param.name] = value;
      }
    });

    // Use ONLY VLE encoding - no fallbacks to maintain architecture integrity
    const encoded = await VLEEncoder.encodeExecuteVLE(
      functionIndex,
      paramDefs,
      paramValues,
      true,
      options,
    );
    return encoded;
  }

  // REMOVED: encodeParametersSimple - Five uses ONLY VLE encoding

  // REMOVED: Local VLE encoding - Five uses centralized VLE encoder

  /**
   * VLE encode a number for instruction data
   */
  private static encodeVLENumber(value: number): Uint8Array {
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
  private static estimateComputeUnits(
    functionIndex: number,
    parameterCount: number,
  ): number {
    // Basic compute unit estimation
    return Math.max(5000, 1000 + parameterCount * 500 + functionIndex * 100);
  }

  /**
   * Infer parameter type from JavaScript value for VLE encoding
   */
  private static inferParameterType(value: any): string {
    if (typeof value === "boolean") {
      return "bool";
    } else if (typeof value === "number") {
      if (Number.isInteger(value)) {
        return value >= 0 ? "u64" : "i64";
      } else {
        return "f64";
      }
    } else if (typeof value === "string") {
      return "string";
    } else if (value instanceof Uint8Array) {
      return "bytes";
    } else {
      // Fallback to string representation
      return "string";
    }
  }

  // ==================== Account Fetching and VLE Deserialization ====================

  /**
   * Fetch account data and deserialize VLE-encoded script data
   * This is the method requested for pulling down accounts and deserializing Five script data
   */
  static async fetchAccountAndDeserializeVLE(
    accountAddress: string,
    connection: any, // Solana Connection object
    options: {
      debug?: boolean;
      parseMetadata?: boolean; // Parse full script metadata or just raw data
      validateVLE?: boolean; // Validate VLE encoding format
    } = {},
  ): Promise<{
    success: boolean;
    accountInfo?: {
      address: string;
      owner: string;
      lamports: number;
      dataLength: number;
    };
    scriptMetadata?: ScriptMetadata;
    rawBytecode?: Uint8Array;
    vleData?: {
      header: any;
      bytecode: Uint8Array;
      abi?: any;
      functions?: Array<{ name: string; index: number; parameters: any[] }>;
    };
    error?: string;
    logs?: string[];
  }> {
    try {
      if (options.debug) {
        console.log(
          `[FiveSDK] Fetching account and deserializing VLE data: ${accountAddress}`,
        );
      }

      // Import Solana web3.js for account fetching
      const { PublicKey } = await import("@solana/web3.js");

      // Validate account address format
      let accountPubkey: any;
      try {
        accountPubkey = new PublicKey(accountAddress);
      } catch (addressError) {
        return {
          success: false,
          error: `Invalid account address format: ${accountAddress}`,
          logs: [],
        };
      }

      // Fetch account info from Solana blockchain
      const accountInfo = await connection.getAccountInfo(
        accountPubkey,
        "confirmed",
      );

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

      const logs: string[] = [];

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

      const result: any = {
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
          const scriptMetadata = ScriptMetadataParser.parseMetadata(
            accountInfo.data,
            accountAddress,
          );
          result.scriptMetadata = scriptMetadata;
          result.rawBytecode = scriptMetadata.bytecode;

          // Create VLE data structure with parsed information
          result.vleData = {
            header: {
              version: scriptMetadata.version,
              deployedAt: scriptMetadata.deployedAt,
              authority: scriptMetadata.authority,
            },
            bytecode: scriptMetadata.bytecode,
            abi: scriptMetadata.abi,
            functions: normalizeAbiFunctions(
              scriptMetadata.abi?.functions ?? scriptMetadata.abi,
            ).map((func: any) => ({
              name: func.name,
              index: func.index,
              parameters: func.parameters || [],
            })),
          };
          const parsedFunctions = result.vleData.functions;

          if (options.debug) {
            console.log(`[FiveSDK] Script metadata parsed successfully:`);
            console.log(`  - Script name: ${scriptMetadata.abi.name}`);
            console.log(
              `  - Functions: ${parsedFunctions.length}`,
            );
            console.log(
              `  - Bytecode size: ${scriptMetadata.bytecode.length} bytes`,
            );
            console.log(`  - Authority: ${scriptMetadata.authority}`);

            logs.push(
              `Script metadata parsed: ${parsedFunctions.length} functions`,
            );
            logs.push(`Bytecode: ${scriptMetadata.bytecode.length} bytes`);
          }
        } catch (metadataError) {
          if (options.debug) {
            console.warn(
              `[FiveSDK] Failed to parse script metadata:`,
              metadataError,
            );
          }

          // Fallback: treat as raw bytecode without metadata
          result.rawBytecode = accountInfo.data;
          logs.push(
            "Warning: Failed to parse script metadata, treating as raw data",
          );
        }
      } else {
        // Just return raw account data
        result.rawBytecode = accountInfo.data;
        logs.push("Raw account data returned (metadata parsing disabled)");
      }

      // Validate VLE encoding if requested and we have bytecode
      if (options.validateVLE && result.rawBytecode) {
        try {
          const validation = await this.validateVLEEncoding(
            result.rawBytecode,
            options.debug,
          );
          if (validation.valid) {
            logs.push("VLE encoding validation: PASSED");
            if (options.debug) {
              console.log(
                `[FiveSDK] VLE validation passed: ${validation.info}`,
              );
            }
          } else {
            logs.push(`VLE encoding validation: FAILED - ${validation.error}`);
            if (options.debug) {
              console.warn(
                `[FiveSDK] VLE validation failed: ${validation.error}`,
              );
            }
          }
        } catch (vleError) {
          logs.push(
            `VLE validation error: ${vleError instanceof Error ? vleError.message : "Unknown error"}`,
          );
        }
      }

      return result;
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : "Unknown account fetch error";

      if (options.debug) {
        console.error(
          `[FiveSDK] Account fetch and VLE deserialization failed: ${errorMessage}`,
        );
      }

      return {
        success: false,
        error: errorMessage,
        logs: [],
      };
    }
  }

  /**
   * Batch fetch multiple accounts and deserialize their VLE data
   */
  static async fetchMultipleAccountsAndDeserializeVLE(
    accountAddresses: string[],
    connection: any,
    options: {
      debug?: boolean;
      parseMetadata?: boolean;
      validateVLE?: boolean;
      batchSize?: number; // Solana RPC batch limit
    } = {},
  ): Promise<
    Map<
      string,
      {
        success: boolean;
        accountInfo?: any;
        scriptMetadata?: ScriptMetadata;
        rawBytecode?: Uint8Array;
        vleData?: any;
        error?: string;
        logs?: string[];
      }
    >
  > {
    const batchSize = options.batchSize || 100; // Solana RPC limit
    const results = new Map();

    if (options.debug) {
      console.log(
        `[FiveSDK] Batch fetching ${accountAddresses.length} accounts (batch size: ${batchSize})`,
      );
    }

    // Process in batches to avoid RPC limits
    for (let i = 0; i < accountAddresses.length; i += batchSize) {
      const batch = accountAddresses.slice(i, i + batchSize);

      if (options.debug) {
        console.log(
          `[FiveSDK] Processing batch ${Math.floor(i / batchSize) + 1}/${Math.ceil(accountAddresses.length / batchSize)}`,
        );
      }

      // Fetch each account in the batch concurrently
      const batchPromises = batch.map((address) =>
        this.fetchAccountAndDeserializeVLE(address, connection, {
          debug: false, // Disable individual debug to avoid spam
          parseMetadata: options.parseMetadata,
          validateVLE: options.validateVLE,
        }),
      );

      const batchResults = await Promise.allSettled(batchPromises);

      // Store results
      batch.forEach((address, index) => {
        const batchResult = batchResults[index];
        if (batchResult.status === "fulfilled") {
          results.set(address, batchResult.value);
        } else {
          results.set(address, {
            success: false,
            error: `Batch processing failed: ${batchResult.reason}`,
            logs: [],
          });
        }
      });
    }

    if (options.debug) {
      const successful = Array.from(results.values()).filter(
        (r) => r.success,
      ).length;
      console.log(
        `[FiveSDK] Batch processing completed: ${successful}/${accountAddresses.length} successful`,
      );
    }

    return results;
  }

  /**
   * Deserialize VLE-encoded parameters from instruction data using WASM decoder
   */
  static async deserializeVLEParameters(
    instructionData: Uint8Array,
    expectedTypes: string[] = [],
    options: { debug?: boolean } = {},
  ): Promise<{
    success: boolean;
    parameters?: Array<{ type: string; value: any }>;
    functionIndex?: number;
    discriminator?: number;
    error?: string;
  }> {
    try {
      if (options.debug) {
        console.log(
          `[FiveSDK] Deserializing VLE parameters from ${instructionData.length} bytes:`,
        );
        console.log(
          `[FiveSDK] Instruction data (hex):`,
          Buffer.from(instructionData).toString("hex"),
        );
        console.log(`[FiveSDK] Expected parameter types:`, expectedTypes);
      }

      // Load WASM VM for VLE decoding
      const wasmVM = await this.loadWasmVM();

      // Use WASM ParameterEncoder to decode VLE data
      try {
        const wasmModule = await import(
          "../assets/vm/five_vm_wasm.js" as string
        );

        if (options.debug) {
          console.log(`[FiveSDK] Using WASM ParameterEncoder for VLE decoding`);
        }

        // Decode the instruction data
        const decodedResult =
          wasmModule.ParameterEncoder.decode_vle_instruction(instructionData);

        if (options.debug) {
          console.log(`[FiveSDK] VLE decoding result:`, decodedResult);
        }

        // Parse the decoded result structure
        const parameters: Array<{ type: string; value: any }> = [];

        if (decodedResult && decodedResult.parameters) {
          decodedResult.parameters.forEach((param: any, index: number) => {
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
      } catch (wasmError) {
        if (options.debug) {
          console.warn(
            `[FiveSDK] WASM VLE decoding failed, attempting manual parsing:`,
            wasmError,
          );
        }

        // Fallback: manual VLE parsing
        return this.parseVLEInstructionManually(
          instructionData,
          expectedTypes,
          options.debug,
        );
      }
    } catch (error) {
      const errorMessage =
        error instanceof Error
          ? error.message
          : "Unknown VLE deserialization error";

      if (options.debug) {
        console.error(
          `[FiveSDK] VLE parameter deserialization failed: ${errorMessage}`,
        );
      }

      return {
        success: false,
        error: errorMessage,
      };
    }
  }

  /**
   * Validate VLE encoding format in bytecode
   */
  private static async validateVLEEncoding(
    bytecode: Uint8Array,
    debug: boolean = false,
  ): Promise<{ valid: boolean; error?: string; info?: string }> {
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
        console.log(
          `[FiveSDK] VLE validation - Magic: "5IVE", Features: ${features}, Functions: ${functionCount}`,
        );
      }

      return {
        valid: true,
        info: `Valid Five VM bytecode with ${functionCount} functions (features: ${features})`,
      };
    } catch (error) {
      return {
        valid: false,
        error: error instanceof Error ? error.message : "VLE validation error",
      };
    }
  }

  /**
   * Manual VLE instruction parsing (fallback when WASM fails)
   */
  private static parseVLEInstructionManually(
    instructionData: Uint8Array,
    expectedTypes: string[],
    debug: boolean = false,
  ): {
    success: boolean;
    parameters?: Array<{ type: string; value: any }>;
    functionIndex?: number;
    discriminator?: number;
    error?: string;
  } {
    try {
      if (instructionData.length < 2) {
        return { success: false, error: "Instruction data too short" };
      }

      let offset = 0;

      // Read discriminator
      const discriminator = instructionData[offset];
      offset += 1;

      if (debug) {
        console.log(
          `[FiveSDK] Manual VLE parsing - Discriminator: ${discriminator}`,
        );
      }

      // Read function index (VLE encoded)
      const { value: functionIndex, bytesRead } = this.readVLENumber(
        instructionData,
        offset,
      );
      offset += bytesRead;

      if (debug) {
        console.log(
          `[FiveSDK] Manual VLE parsing - Function index: ${functionIndex}`,
        );
      }

      // Read parameter count (VLE encoded)
      const { value: paramCount, bytesRead: paramCountBytes } =
        this.readVLENumber(instructionData, offset);
      offset += paramCountBytes;

      if (debug) {
        console.log(
          `[FiveSDK] Manual VLE parsing - Parameter count: ${paramCount}`,
        );
      }

      // Read parameters
      const parameters: Array<{ type: string; value: any }> = [];

      for (let i = 0; i < paramCount; i++) {
        const { value: paramValue, bytesRead: paramBytes } = this.readVLENumber(
          instructionData,
          offset,
        );
        offset += paramBytes;

        parameters.push({
          type: expectedTypes[i] || "u64", // Default to u64
          value: paramValue,
        });

        if (debug) {
          console.log(
            `[FiveSDK] Manual VLE parsing - Parameter ${i}: ${paramValue}`,
          );
        }
      }

      return {
        success: true,
        parameters,
        functionIndex,
        discriminator,
      };
    } catch (error) {
      return {
        success: false,
        error:
          error instanceof Error ? error.message : "Manual VLE parsing failed",
      };
    }
  }

  /**
   * Read VLE-encoded number from byte array
   */
  private static readVLENumber(
    data: Uint8Array,
    offset: number,
  ): { value: number; bytesRead: number } {
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
  static async executeWithStateDiff(
    scriptAccount: string,
    connection: any,
    signerKeypair: any,
    functionName: string | number,
    parameters: any[] = [],
    options: {
      debug?: boolean;
      network?: string;
      computeUnitLimit?: number;
      trackGlobalFields?: boolean; // Track global variables changes
      additionalAccounts?: string[]; // Additional accounts to track
      includeVMState?: boolean; // Include VM state PDA in tracking
    } = {},
  ): Promise<{
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
  }> {
    const logs: string[] = [];

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
          console.log(
            `  Added ${options.additionalAccounts.length} additional accounts to tracking`,
          );
        }
      }

      logs.push(
        `Tracking ${accountsToTrack.length} accounts for state changes`,
      );

      // Step 1: Fetch BEFORE state
      if (options.debug) {
        console.log(
          `[FiveSDK] Step 1: Fetching BEFORE state for ${accountsToTrack.length} accounts...`,
        );
      }

      const beforeState = await this.fetchMultipleAccountsAndDeserializeVLE(
        accountsToTrack,
        connection,
        {
          debug: false, // Avoid debug spam
          parseMetadata: true,
          validateVLE: false, // Skip validation for speed
        },
      );

      let successfulBeforeFetches = 0;
      for (const [address, result] of beforeState.entries()) {
        if (result.success) {
          successfulBeforeFetches++;
        } else if (options.debug) {
          console.warn(
            `[FiveSDK] Warning: Failed to fetch BEFORE state for ${address}: ${result.error}`,
          );
        }
      }

      logs.push(
        `BEFORE state: ${successfulBeforeFetches}/${accountsToTrack.length} accounts fetched`,
      );

      // Extract global fields from BEFORE state if requested
      let beforeGlobalFields: Record<string, any> = {};
      if (options.trackGlobalFields) {
        const scriptBefore = beforeState.get(scriptAccount);
        if (scriptBefore?.success && scriptBefore.scriptMetadata) {
          beforeGlobalFields = this.extractGlobalFields(
            scriptBefore.scriptMetadata,
            "before",
          );
          if (options.debug) {
            console.log(
              `[FiveSDK] Extracted ${Object.keys(beforeGlobalFields).length} global fields from BEFORE state`,
            );
          }
        }
      }

      // Step 2: Execute the script
      if (options.debug) {
        console.log(`[FiveSDK] Step 2: Executing script...`);
      }

      const executionResult = await this.executeOnSolana(
        scriptAccount,
        connection,
        signerKeypair,
        functionName,
        parameters,
        options.additionalAccounts || [],
        {
          debug: options.debug,
          network: options.network,
          computeUnitLimit: options.computeUnitLimit,
        },
      );

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

      const afterState = await this.fetchMultipleAccountsAndDeserializeVLE(
        accountsToTrack,
        connection,
        {
          debug: false,
          parseMetadata: true,
          validateVLE: false,
        },
      );

      let successfulAfterFetches = 0;
      for (const [address, result] of afterState.entries()) {
        if (result.success) {
          successfulAfterFetches++;
        } else if (options.debug) {
          console.warn(
            `[FiveSDK] Warning: Failed to fetch AFTER state for ${address}: ${result.error}`,
          );
        }
      }

      logs.push(
        `AFTER state: ${successfulAfterFetches}/${accountsToTrack.length} accounts fetched`,
      );

      // Extract global fields from AFTER state if requested
      let afterGlobalFields: Record<string, any> = {};
      if (options.trackGlobalFields) {
        const scriptAfter = afterState.get(scriptAccount);
        if (scriptAfter?.success && scriptAfter.scriptMetadata) {
          afterGlobalFields = this.extractGlobalFields(
            scriptAfter.scriptMetadata,
            "after",
          );
          if (options.debug) {
            console.log(
              `[FiveSDK] Extracted ${Object.keys(afterGlobalFields).length} global fields from AFTER state`,
            );
          }
        }
      }

      // Step 5: Compute differences
      if (options.debug) {
        console.log(`[FiveSDK] Step 4: Computing state differences...`);
      }

      const changes = this.computeStateDifferences(
        beforeState,
        afterState,
        options.debug,
      );
      let globalFieldChanges: Array<{
        fieldName: string;
        oldValue: any;
        newValue: any;
      }> = [];

      if (options.trackGlobalFields) {
        globalFieldChanges = this.computeGlobalFieldChanges(
          beforeGlobalFields,
          afterGlobalFields,
        );
        if (options.debug) {
          console.log(
            `[FiveSDK] Found ${globalFieldChanges.length} global field changes`,
          );
        }
      }

      logs.push(
        `State analysis: ${changes.length} account changes, ${globalFieldChanges.length} global field changes`,
      );

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
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : "Unknown state tracking error";

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
  private static computeStateDifferences(
    beforeState: Map<string, any>,
    afterState: Map<string, any>,
    debug: boolean = false,
  ): Array<{
    account: string;
    fieldName?: string;
    oldValue: any;
    newValue: any;
    changeType: "created" | "modified" | "deleted";
  }> {
    const changes: Array<{
      account: string;
      fieldName?: string;
      oldValue: any;
      newValue: any;
      changeType: "created" | "modified" | "deleted";
    }> = [];

    // Check all accounts that were tracked
    const allAccounts = new Set([...beforeState.keys(), ...afterState.keys()]);

    for (const account of allAccounts) {
      const before = beforeState.get(account);
      const after = afterState.get(account);

      if (debug) {
        console.log(
          `[FiveSDK] Analyzing account ${account.substring(0, 8)}...`,
        );
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
          this.compareScriptMetadata(
            before.scriptMetadata,
            after.scriptMetadata,
            account,
            changes,
          );
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
  private static extractGlobalFields(
    scriptMetadata: ScriptMetadata,
    phase: "before" | "after",
  ): Record<string, any> {
    const globalFields: Record<string, any> = {};

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
    } catch (error) {
      console.warn(
        `[FiveSDK] Failed to extract global fields (${phase}):`,
        error,
      );
    }

    return globalFields;
  }

  /**
   * Compare global field values between before and after states
   */
  private static computeGlobalFieldChanges(
    beforeFields: Record<string, any>,
    afterFields: Record<string, any>,
  ): Array<{ fieldName: string; oldValue: any; newValue: any }> {
    const changes: Array<{ fieldName: string; oldValue: any; newValue: any }> =
      [];

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
  private static compareScriptMetadata(
    beforeMetadata: ScriptMetadata,
    afterMetadata: ScriptMetadata,
    account: string,
    changes: Array<{
      account: string;
      fieldName?: string;
      oldValue: any;
      newValue: any;
      changeType: "created" | "modified" | "deleted";
    }>,
  ): void {
    // Check if function count changed
    if (
      beforeMetadata.abi.functions.length !== afterMetadata.abi.functions.length
    ) {
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
  private static extractStateSection(
    bytecode: Uint8Array,
  ): Record<string, any> | null {
    try {
      // Skip header (6 bytes: "5IVE" + features + function_count)
      if (bytecode.length < 6) return null;

      // Look for state section marker (this is hypothetical)
      // In practice, you'd need to parse the full Five VM format
      const stateMarker = new Uint8Array([0xff, 0xfe]); // Hypothetical state section marker

      for (let i = 6; i < bytecode.length - 1; i++) {
        if (
          bytecode[i] === stateMarker[0] &&
          bytecode[i + 1] === stateMarker[1]
        ) {
          // Found potential state section
          // Parse state variables (simplified)
          const stateData: Record<string, any> = {};

          // This would need proper state parsing logic
          // For now, return empty object
          return stateData;
        }
      }
    } catch (error) {
      console.warn("[FiveSDK] State section extraction failed:", error);
    }

    return null;
  }

  /**
   * Utility: Check if two bytecode arrays are equal
   */
  private static bytecodeEqual(a: Uint8Array, b: Uint8Array): boolean {
    if (a.length !== b.length) return false;
    for (let i = 0; i < a.length; i++) {
      if (a[i] !== b[i]) return false;
    }
    return true;
  }

  /**
   * Utility: Generate simple hash of bytecode for comparison
   */
  private static hashBytecode(bytecode: Uint8Array): string {
    let hash = 0;
    for (let i = 0; i < bytecode.length; i++) {
      hash = ((hash << 5) - hash + bytecode[i]) & 0xffffffff;
    }
    return hash.toString(16);
  }

  /**
   * Utility: Deep equality check
   */
  private static deepEqual(a: any, b: any): boolean {
    if (a === b) return true;
    if (a == null || b == null) return false;
    if (typeof a !== typeof b) return false;

    if (typeof a === "object") {
      if (Array.isArray(a) !== Array.isArray(b)) return false;

      const keysA = Object.keys(a);
      const keysB = Object.keys(b);

      if (keysA.length !== keysB.length) return false;

      for (const key of keysA) {
        if (!keysB.includes(key)) return false;
        if (!this.deepEqual(a[key], b[key])) return false;
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
  static async executeOnSolana(
    scriptAccount: string, // The deployed script account (from deployment)
    connection: any, // Solana Connection object
    signerKeypair: any, // Solana Keypair object for signing
    functionName: string | number,
    parameters: any[] = [],
    accounts: string[] = [], // Additional account pubkeys as base58 strings
    options: {
      debug?: boolean;
      network?: string;
      computeUnitLimit?: number;
      computeUnitPrice?: number;
      maxRetries?: number;
      vmStateAccount?: string;
      fiveVMProgramId?: string;
      abi?: any; // Optional ABI for parameter encoding
    } = {},
  ): Promise<{
    success: boolean;
    result?: any;
    transactionId?: string;
    computeUnitsUsed?: number;
    cost?: number;
    error?: string;
    logs?: string[];
  }> {
    // Track the latest signature so we can surface it even on failure for log inspection.
    let lastSignature: string | undefined;

    if (options.debug) {
      console.log(
        `[FiveSDK] executeOnSolana called with script account: ${scriptAccount}`,
      );
      console.log(
        `[FiveSDK] function: ${functionName}, parameters: ${JSON.stringify(parameters)}`,
      );
      console.log(`[FiveSDK] options:`, options);
    }

    try {
      // Import Solana web3.js components
      const {
        PublicKey,
        Transaction,
        TransactionInstruction,
        ComputeBudgetProgram,
      } = await import("@solana/web3.js");

      // For on-chain execution, we'll bypass metadata requirements and generate instruction directly
      // Generate execution instruction - for MVP, we'll use simple parameter encoding without full metadata
      let executionData;
      try {
        executionData = await this.generateExecuteInstruction(
          scriptAccount,
          functionName,
          parameters,
          accounts,
          connection,
          {
            debug: options.debug,
            computeUnitLimit: options.computeUnitLimit,
            vmStateAccount: options.vmStateAccount,
            fiveVMProgramId: options.fiveVMProgramId,
            abi: options.abi,
          },
        );
      } catch (metadataError) {
        // NO FALLBACK: Metadata is required for proper VLE encoding
        // ENGINEERING INTEGRITY: No duplicate code paths, no silent degradation
        const errorMessage = `Execution instruction generation failed - metadata required for VLE encoding: ${metadataError instanceof Error ? metadataError.message : "Unknown metadata error"}`;
        if (options.debug) {
          console.error(`[FiveSDK] ${errorMessage}`);
        }
        throw new Error(errorMessage);
      }

      if (options.debug) {
        console.log(
          `[FiveSDK] Generated execution instruction:`,
          executionData,
        );
        console.log(
          `[FiveSDK] Accounts in instruction:`,
          executionData.instruction.accounts,
        );
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
              console.log(
                `[FiveSDK] Overriding VM state account to: ${options.vmStateAccount}`,
              );
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
        console.log(
          `[FiveSDK] Final account keys for transaction:`,
          accountKeys,
        );
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
        console.log(
          `  - Data (base64): ${instruction.data.toString("base64")}`,
        );
        console.log(`  - Accounts (${instruction.keys.length}):`);

        instruction.keys.forEach((key, keyIndex) => {
          console.log(
            `    ${keyIndex}: ${key.pubkey.toString()} (signer: ${key.isSigner}, writable: ${key.isWritable})`,
          );
        });
      });

      console.log(`\n🔐 Transaction Signatures:`);
      transaction.signatures.forEach((sig, sigIndex) => {
        console.log(
          `  ${sigIndex}: ${sig.publicKey.toString()} - ${sig.signature ? "SIGNED" : "UNSIGNED"}`,
        );
      });

      console.log(
        `\n📦 Serialized Transaction: ${transaction.serialize().length} bytes`,
      );
      console.log(
        `==================== END TRANSACTION DEBUG ====================\n`,
      );

      console.log(`\n\n\n!!!!!!!!! SENDING TRANSACTION NOW !!!!!!!!!\n\n\n`);

      // Send transaction with preflight disabled to get actual Five VM errors
      const signature = await connection.sendRawTransaction(
        transaction.serialize(),
        {
          skipPreflight: true, // Skip preflight to see actual program errors
          preflightCommitment: "confirmed",
          maxRetries: options.maxRetries || 3,
        },
      );
      lastSignature = signature;

      // Wait for confirmation with detailed error handling
      let confirmation;
      try {
        confirmation = await connection.confirmTransaction(
          {
            signature,
            blockhash,
            lastValidBlockHeight: (
              await connection.getLatestBlockhash("confirmed")
            ).lastValidBlockHeight,
          },
          "confirmed",
        );
      } catch (confirmError) {
        if (options.debug) {
          console.log(
            `[FiveSDK] Confirmation failed, checking transaction status...`,
          );
        }

        // Try to get transaction details even if confirmation failed
        try {
          const txDetails = await connection.getTransaction(signature, {
            commitment: "confirmed",
            maxSupportedTransactionVersion: 0,
          });

          if (txDetails) {
            if (options.debug) {
              console.log(
                `[FiveSDK] Transaction found! Status:`,
                txDetails.meta?.err ? "Failed" : "Success",
              );
              console.log(
                `[FiveSDK] Transaction logs:`,
                txDetails.meta?.logMessages,
              );
            }

            if (txDetails.meta?.err) {
              return {
                success: false,
                error: `Transaction failed: ${JSON.stringify(txDetails.meta.err)}`,
                logs: txDetails.meta.logMessages || [],
                transactionId: signature,
              };
            } else {
              // Transaction succeeded but confirmation timed out
              return {
                success: true,
                transactionId: signature,
                computeUnitsUsed: txDetails.meta?.computeUnitsConsumed,
                logs: txDetails.meta?.logMessages || [],
                result:
                  "Execution completed successfully (confirmation timeout but transaction succeeded)",
              };
            }
          }
        } catch (getTransactionError) {
          if (options.debug) {
            console.log(
              `[FiveSDK] Could not retrieve transaction details:`,
              getTransactionError,
            );
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
        let logs: string[] = [];
        let computeUnitsUsed: number | undefined;
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
            } as any);
            if (logsResp?.value?.logs) {
              logs = logsResp.value.logs;
            }
          }
        } catch { }

        const errorMessage = `Execution transaction failed: ${JSON.stringify(
          confirmation.value.err,
        )}`;
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
      let computeUnitsUsed: number | undefined;
      let logs: string[] = [];

      try {
        const txDetails = await connection.getTransaction(signature, {
          commitment: "confirmed",
          maxSupportedTransactionVersion: 0,
        });

        if (txDetails?.meta) {
          computeUnitsUsed = txDetails.meta.computeUnitsConsumed || undefined;
          logs = txDetails.meta.logMessages || [];
        }
      } catch (logError) {
        if (options.debug) {
          console.warn(
            `[FiveSDK] Could not fetch transaction logs: ${logError}`,
          );
        }
      }

      return {
        success: true,
        transactionId: signature,
        computeUnitsUsed,
        logs,
        result: "Execution completed successfully", // Five VM doesn't return complex results yet
      };
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : "Unknown execution error";

      // Capture signature if SendTransactionError populated it
      if (!lastSignature && (error as any)?.signature) {
        lastSignature = (error as any).signature;
      }

      if (options.debug) {
        console.error(`[FiveSDK] Execution failed: ${errorMessage}`);
        if (error instanceof Error && error.stack) {
          console.error(`[FiveSDK] Stack trace:`, error.stack);
        }
      }

      // Attempt to extract logs from SendTransactionError if available
      let logs: string[] = (error as any)?.transactionLogs || [];
      if (typeof (error as any)?.getLogs === "function") {
        try {
          const extracted = await (error as any).getLogs();
          if (Array.isArray(extracted)) {
            logs = extracted;
          }
        } catch {
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
  static async executeScriptAccount(
    scriptAccount: string,
    functionIndex: number = 0,
    parameters: any[] = [],
    connection: any, // Solana Connection object
    signerKeypair: any, // Solana Keypair object
    options: {
      debug?: boolean;
      network?: string;
      computeBudget?: number;
      maxRetries?: number;
      vmStateAccount?: string;
      fiveVMProgramId?: string;
    } = {},
  ): Promise<{
    success: boolean;
    result?: any;
    transactionId?: string;
    computeUnitsUsed?: number;
    cost?: number;
    error?: string;
    logs?: string[];
  }> {
    if (options.debug) {
      console.log(`[FiveSDK] executeScriptAccount called with:`);
      console.log(`  Script Account: ${scriptAccount}`);
      console.log(`  Function Index: ${functionIndex}`);
      console.log(`  Parameters: ${JSON.stringify(parameters)}`);
    }

    try {
      // Call the existing executeOnSolana method with function index
      const result = await this.executeOnSolana(
        scriptAccount,
        connection,
        signerKeypair,
        functionIndex, // Use function index instead of name
        parameters,
        [], // No additional accounts for now
        {
          debug: options.debug,
          network: options.network,
          computeUnitLimit: options.computeBudget || 1400000,
          maxRetries: options.maxRetries || 3,
          vmStateAccount: options.vmStateAccount,
          fiveVMProgramId: options.fiveVMProgramId,
        },
      );

      if (options.debug) {
        console.log(`[FiveSDK] executeScriptAccount result:`, result);
      }

      return result;
    } catch (error) {
      const errorMessage =
        error instanceof Error
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
  static async getFunctionNames(
    bytecode: FiveBytecode,
  ): Promise<FunctionNameEntry[]> {
    await this.initializeComponents(false);

    try {
      const namesJson = await this.compiler!.getFunctionNames(bytecode);
      if (Array.isArray(namesJson)) {
        return namesJson as FunctionNameEntry[];
      }
      const parsed = JSON.parse(namesJson as any) as FunctionNameEntry[];
      return parsed;
    } catch (error) {
      console.warn("[FiveSDK] Failed to extract function names:", error);
      return [];
    }
  }

  /**
   * Call a function by name instead of index
   */
  static async callFunctionByName(
    scriptAccount: string,
    functionName: string,
    parameters: any[] = [],
    accounts: string[] = [],
    connection?: any,
    options: {
      debug?: boolean;
      computeUnitLimit?: number;
      vmStateAccount?: string;
    } = {},
  ): Promise<SerializedExecution> {
    // First get the available function names
    const available = await this.getFunctionNamesFromScriptAccount(
      scriptAccount,
      connection,
    );
    if (!available) {
      throw new ExecutionSDKError(
        `Cannot resolve function name "${functionName}": unable to fetch bytecode from script account`,
      );
    }

    const funcInfo = available.find((f) => f.name === functionName);
    if (!funcInfo) {
      const availableNames = available.map((f) => f.name).join(", ");
      throw new ExecutionSDKError(
        `Function "${functionName}" not found. Available functions: ${availableNames}`,
      );
    }

    // Now execute using the resolved index (call the index-based executor)
    return this.executeByIndex(
      scriptAccount,
      funcInfo.function_index,
      parameters,
      accounts,
      connection,
      options,
    );
  }

  /**
   * Generate serialized execution data by function index.
   */
  static async executeByIndex(
    scriptAccount: string,
    functionIndex: number,
    parameters: any[] = [],
    accounts: string[] = [],
    connection?: any,
    options: {
      debug?: boolean;
      computeUnitLimit?: number;
      vmStateAccount?: string;
    } = {},
  ): Promise<SerializedExecution> {
    validator.validateBase58Address(scriptAccount, "scriptAccount");
    Validators.functionRef(functionIndex);
    Validators.parameters(parameters);
    Validators.accounts(accounts);
    Validators.options(options);

    return this.generateExecuteInstruction(
      scriptAccount,
      functionIndex,
      parameters,
      accounts,
      connection,
      options,
    );
  }

  /**
   * Get function names from a deployed script account
   */
  static async getFunctionNamesFromScriptAccount(
    scriptAccount: string,
    connection?: any,
  ): Promise<FunctionNameEntry[] | null> {
    if (!connection) {
      console.warn(
        "[FiveSDK] No connection provided for script account lookup",
      );
      return null;
    }

    try {
      const { PublicKey } = await import("@solana/web3.js");
      const accountInfo = await connection.getAccountInfo(
        new PublicKey(scriptAccount),
        "confirmed",
      );
      if (!accountInfo) {
        console.warn(`[FiveSDK] Script account ${scriptAccount} not found`);
        return null;
      }

      const data = accountInfo.data;
      const scriptHeaderSize = 64;
      let bytecode = data;

      if (
        data.length >= scriptHeaderSize &&
        data[0] === 0x35 &&
        data[1] === 0x49 &&
        data[2] === 0x56 &&
        data[3] === 0x45
      ) {
        const readU32LE = (buffer: Uint8Array, offset: number): number =>
          buffer[offset] |
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

        if (
          headerVersion === 4 &&
          reserved0 === 0 &&
          bytecodeLen > 0 &&
          totalLen <= data.length
        ) {
          bytecode = data.slice(
            scriptHeaderSize,
            scriptHeaderSize + bytecodeLen,
          );
        }
      }

      return await this.getFunctionNames(bytecode);
    } catch (error) {
      console.warn(
        "[FiveSDK] Failed to fetch function names from script account:",
        error,
      );
      return null;
    }
  }

  /**
   * Deploy bytecode to Solana using the correct two-transaction pattern
   * This is the method the CLI should use for actual deployment
   */
  /**
   * Create a deployment transaction (for frontend use)
   * Generates necessary keypairs and builds the transaction, signing it with the generated keys.
   * The caller (frontend) must add the fee payer's signature and send it.
   */
  static async createDeploymentTransaction(
    bytecode: FiveBytecode,
    connection: any,
    deployerPublicKey: any, // PublicKey
    options: {
      debug?: boolean;
      fiveVMProgramId?: string;
      computeBudget?: number;
    } = {},
  ): Promise<{
    transaction: any;
    scriptKeypair: any;
    vmStateKeypair: any;
    programId: string;
    rentLamports: number;
  }> {
    const {
      Keypair,
      PublicKey,
      Transaction,
      TransactionInstruction,
      SystemProgram,
      ComputeBudgetProgram,
    } = await import("@solana/web3.js");

    const programIdStr = options.fiveVMProgramId || FIVE_VM_PROGRAM_ID;
    const programId = new PublicKey(programIdStr);

    // Generate script keypair
    const scriptKeypair = Keypair.generate();
    const scriptAccount = scriptKeypair.publicKey.toString();

    // Calculate account size and rent
    const SCRIPT_HEADER_SIZE = 64; // ScriptHeader::LEN
    const totalAccountSize = SCRIPT_HEADER_SIZE + bytecode.length;
    const rentLamports = await connection.getMinimumBalanceForRentExemption(totalAccountSize);

    // Generate VM state keypair
    const vmStateKeypair = Keypair.generate();
    const VM_STATE_SIZE = 56; // FIVEVMState::LEN
    const vmStateRent = await connection.getMinimumBalanceForRentExemption(VM_STATE_SIZE);

    if (options.debug) {
      console.log(`[FiveSDK] Preparing deployment transaction:`);
      console.log(`  - Script Account: ${scriptAccount}`);
      console.log(`  - VM State Account: ${vmStateKeypair.publicKey.toString()}`);
      console.log(`  - Deployer: ${deployerPublicKey.toString()}`);
    }

    const tx = new Transaction();

    // Add compute budget if requested
    if (options.computeBudget && options.computeBudget > 0) {
      tx.add(
        ComputeBudgetProgram.setComputeUnitLimit({
          units: options.computeBudget,
        }),
      );
    }

    // 1. Create VM State Account
    tx.add(
      SystemProgram.createAccount({
        fromPubkey: deployerPublicKey,
        newAccountPubkey: vmStateKeypair.publicKey,
        lamports: vmStateRent,
        space: VM_STATE_SIZE,
        programId: programId,
      }),
    );

    // 2. Initialize VM State
    tx.add(
      new TransactionInstruction({
        keys: [
          { pubkey: vmStateKeypair.publicKey, isSigner: false, isWritable: true },
          { pubkey: deployerPublicKey, isSigner: true, isWritable: false },
        ],
        programId: programId,
        data: Buffer.from([0]), // Initialize discriminator
      }),
    );

    // 3. Create Script Account
    tx.add(
      SystemProgram.createAccount({
        fromPubkey: deployerPublicKey,
        newAccountPubkey: scriptKeypair.publicKey,
        lamports: rentLamports,
        space: totalAccountSize,
        programId: programId,
      }),
    );

    // 4. Deploy Instruction
    const deployData = this.encodeDeployInstruction(bytecode);
    tx.add(
      new TransactionInstruction({
        keys: [
          { pubkey: scriptKeypair.publicKey, isSigner: false, isWritable: true },
          { pubkey: vmStateKeypair.publicKey, isSigner: false, isWritable: true },
          { pubkey: deployerPublicKey, isSigner: true, isWritable: true },
        ],
        programId: programId,
        data: Buffer.from(deployData),
      }),
    );

    const { blockhash } = await connection.getLatestBlockhash("confirmed");
    tx.recentBlockhash = blockhash;
    tx.feePayer = deployerPublicKey;

    // Partial sign with generated keys
    tx.partialSign(scriptKeypair);
    tx.partialSign(vmStateKeypair);

    return {
      transaction: tx,
      scriptKeypair,
      vmStateKeypair,
      programId: scriptAccount,
      rentLamports,
    };
  }

  static async deployToSolana(
    bytecode: FiveBytecode,
    connection: any, // Solana Connection object
    deployerKeypair: any, // Solana Keypair object
    options: {
      debug?: boolean;
      network?: string;
      computeBudget?: number;
      maxRetries?: number;
      fiveVMProgramId?: string;
      vmStateAccount?: string;
    } = {},
  ): Promise<{
    success: boolean;
    programId?: string;
    transactionId?: string;
    deploymentCost?: number;
    error?: string;
    logs?: string[];
    vmStateAccount?: string;
  }> {
    console.log(
      `[FiveSDK] deployToSolana called with bytecode length: ${bytecode.length}`,
    );
    console.log(`[FiveSDK] options:`, options);

    // Use the provided program ID or fall back to the constant
    const programId = options.fiveVMProgramId || FIVE_VM_PROGRAM_ID;

    try {
      if (options.debug) {
        console.log(
          `[FiveSDK] Starting deployment with ${bytecode.length} bytes of bytecode to program ${programId}`,
        );
      }

      // Generate script keypair like frontend-five
      const {
        Keypair,
        PublicKey,
        Transaction,
        TransactionInstruction,
        SystemProgram,
      } = await import("@solana/web3.js");
      const scriptKeypair = Keypair.generate();
      const scriptAccount = scriptKeypair.publicKey.toString();

      if (options.debug) {
        console.log(`[FiveSDK] Generated script keypair: ${scriptAccount}`);
      }

      // Calculate account size and rent
      const SCRIPT_HEADER_SIZE = 64; // ScriptHeader::LEN (five-protocol)
      const totalAccountSize = SCRIPT_HEADER_SIZE + bytecode.length;
      const rentLamports =
        await connection.getMinimumBalanceForRentExemption(totalAccountSize);

      // Generate VM state keypair for this deployment OR reuse
      let vmStatePubkey: any;
      let vmStateKeypair: any;
      let vmStateRent = 0;
      const VM_STATE_SIZE = 56; // FIVEVMState::LEN

      if (options.vmStateAccount) {
        vmStatePubkey = new PublicKey(options.vmStateAccount);
        if (options.debug) {
          console.log(`[FiveSDK] Reuse VM State: ${vmStatePubkey.toString()}`);
        }
      } else {
        vmStateKeypair = Keypair.generate();
        vmStatePubkey = vmStateKeypair.publicKey;
        vmStateRent = await connection.getMinimumBalanceForRentExemption(VM_STATE_SIZE);
      }

      if (options.debug) {
        console.log(`[FiveSDK] Script Account: ${scriptAccount}`);
        console.log(`[FiveSDK] VM State Account: ${vmStatePubkey.toString()}`);
        console.log(`[FiveSDK] Account size: ${totalAccountSize} bytes`);
        console.log(`[FiveSDK] Rent cost: ${((rentLamports + vmStateRent) / 1e9)} SOL`);
      }

      // SINGLE TRANSACTION: create VM state + initialize + create script account + deploy bytecode
      const tx = new Transaction();

      // Optional compute budget
      if (options.computeBudget && options.computeBudget > 0) {
        try {
          const { ComputeBudgetProgram } = await import("@solana/web3.js");
          tx.add(
            ComputeBudgetProgram.setComputeUnitLimit({
              units: options.computeBudget,
            }),
          );
        } catch { }
      }

      if (!options.vmStateAccount) {
        // 1) Create VM state account owned by the program
        const createVmStateIx = SystemProgram.createAccount({
          fromPubkey: deployerKeypair.publicKey,
          newAccountPubkey: vmStatePubkey,
          lamports: vmStateRent,
          space: VM_STATE_SIZE,
          programId: new PublicKey(programId),
        });
        tx.add(createVmStateIx);

        // 2) Initialize VM state: [discriminator(0)] with accounts [vm_state, authority]
        const initVmStateIx = new TransactionInstruction({
          keys: [
            {
              pubkey: vmStatePubkey,
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
      }

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
      console.log(
        `[FiveSDK] DEBUG - deployData hex:`,
        Buffer.from(deployData).toString("hex"),
      );
      console.log(
        `[FiveSDK] DEBUG - scriptKeypair.publicKey:`,
        scriptKeypair.publicKey.toString(),
      );

      const instructionDataBuffer = Buffer.from(deployData);
      console.log(
        `[FiveSDK] DEBUG - instructionDataBuffer hex:`,
        instructionDataBuffer.toString("hex"),
      );

      const deployIx = new TransactionInstruction({
        keys: [
          {
            pubkey: scriptKeypair.publicKey,
            isSigner: false,
            isWritable: true,
          },
          {
            pubkey: vmStatePubkey,
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
      if (!options.vmStateAccount) {
        tx.partialSign(vmStateKeypair);
      }
      tx.partialSign(scriptKeypair);

      if (options.debug) {
        console.log(`\n🔍 COMBINED DEPLOY TX:`);
        console.log(`  - Instructions: ${tx.instructions.length}`);
        console.log(`  - Fee Payer: ${tx.feePayer?.toString()}`);
        console.log(`  - Recent Blockhash: ${tx.recentBlockhash}`);
        tx.instructions.forEach((ix, i) => {
          console.log(
            `\n📝 Instruction ${i}: ${ix.programId.toString()} (keys=${ix.keys.length}, data=${ix.data.length} bytes)`,
          );
          ix.keys.forEach((k, j) =>
            console.log(
              `    ${j}: ${k.pubkey.toString()} (signer=${k.isSigner}, writable=${k.isWritable})`,
            ),
          );
        });
        console.log(`\n📦 Serialized Size: ${tx.serialize().length} bytes`);
      }

      const txSerialized = tx.serialize();
      if (options.debug) {
        console.log(`[FiveSDK] Transaction serialized: ${txSerialized.length} bytes`);
      }

      const signature = await connection.sendRawTransaction(txSerialized, {
        skipPreflight: true,
        preflightCommitment: "confirmed",
        maxRetries: options.maxRetries || 3,
      });

      if (options.debug) {
        console.log(`[FiveSDK] sendRawTransaction completed, returned signature: ${signature}`);
      }

      // Custom confirmation polling with extended timeout (120 seconds)
      // The default confirmTransaction has a 30s timeout which may be too short
      const confirmationResult = await this.pollForConfirmation(
        connection,
        signature,
        "confirmed",
        120000, // 120 second timeout
        options.debug
      );

      if (!confirmationResult.success) {
        const errorMessage = `Deployment confirmation failed: ${confirmationResult.error || "Unknown error"}`;
        if (options.debug) console.log(`[FiveSDK] ${errorMessage}`);
        return {
          success: false,
          error: errorMessage,
          transactionId: signature,
        };
      }

      if (confirmationResult.err) {
        const errorMessage = `Combined deployment failed: ${JSON.stringify(confirmationResult.err)}`;
        if (options.debug) console.log(`[FiveSDK] ${errorMessage}`);
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
          `VM State Account: ${vmStatePubkey.toString()}`,
        ],
        vmStateAccount: vmStatePubkey.toString(),
      };
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : "Unknown deployment error";

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
  static async deployLargeProgramToSolana(
    bytecode: FiveBytecode,
    connection: any, // Solana Connection object
    deployerKeypair: any, // Solana Keypair object
    options: {
      chunkSize?: number; // Default: 750 bytes
      debug?: boolean;
      network?: string;
      maxRetries?: number;
      fiveVMProgramId?: string;
      progressCallback?: (chunk: number, total: number) => void;
      vmStateAccount?: string;
    } = {},
  ): Promise<{
    success: boolean;
    scriptAccount?: string;
    transactionIds?: string[];
    totalTransactions?: number;
    deploymentCost?: number;
    chunksUsed?: number;
    vmStateAccount?: string;
    error?: string;
    logs?: string[];
  }> {
    const DEFAULT_CHUNK_SIZE = 500; // Leaves room for transaction overhead
    const chunkSize = options.chunkSize || DEFAULT_CHUNK_SIZE;

    console.log(
      `[FiveSDK] deployLargeProgramToSolana called with ${bytecode.length} bytes`,
    );
    console.log(`[FiveSDK] Using chunk size: ${chunkSize} bytes`);
    console.log(`[FiveSDK] options:`, options);

    try {
      // If bytecode is small enough, use regular deployment
      if (bytecode.length <= 800) {
        if (options.debug) {
          console.log(
            `[FiveSDK] Bytecode is small (${bytecode.length} bytes), using regular deployment`,
          );
        }
        return await this.deployToSolana(
          bytecode,
          connection,
          deployerKeypair,
          {
            debug: options.debug,
            network: options.network,
            maxRetries: options.maxRetries,
            fiveVMProgramId: options.fiveVMProgramId,
            vmStateAccount: options.vmStateAccount,
          },
        );
      }

      const {
        Keypair,
        PublicKey,
        Transaction,
        TransactionInstruction,
        SystemProgram,
      } = await import("@solana/web3.js");

      // Generate script keypair
      const scriptKeypair = Keypair.generate();
      const scriptAccount = scriptKeypair.publicKey.toString();

      // Calculate account size and rent
      const SCRIPT_HEADER_SIZE = 64; // ScriptHeader::LEN (five-protocol)
      const totalAccountSize = SCRIPT_HEADER_SIZE + bytecode.length;
      const rentLamports =
        await connection.getMinimumBalanceForRentExemption(totalAccountSize);

      const programId = options.fiveVMProgramId ? new PublicKey(options.fiveVMProgramId) : new PublicKey(FIVE_VM_PROGRAM_ID);

      // Handle VM state account (reuse or create)
      let vmStatePubkey: any;
      let vmStateKeypair: any;
      let vmStateRent = 0;
      const VM_STATE_SIZE = 56; // FIVEVMState::LEN

      if (options.vmStateAccount) {
        vmStatePubkey = new PublicKey(options.vmStateAccount);
        if (options.debug) {
          console.log(`[FiveSDK] Reuse existing VM State: ${vmStatePubkey.toString()}`);
        }
      } else {
        // Generate NEW VM state account
        vmStateKeypair = Keypair.generate();
        vmStatePubkey = vmStateKeypair.publicKey;
        vmStateRent = await connection.getMinimumBalanceForRentExemption(VM_STATE_SIZE);
      }

      if (options.debug) {
        console.log(`[FiveSDK] Script Account: ${scriptAccount}`);
        console.log(
          `[FiveSDK] VM State Account: ${vmStatePubkey.toString()}`,
        );
        console.log(`[FiveSDK] Total account size: ${totalAccountSize} bytes`);
        console.log(
          `[FiveSDK] Initial rent cost: ${(rentLamports + vmStateRent) / 1e9} SOL`,
        );
      }

      const transactionIds: string[] = [];
      let totalCost = rentLamports + vmStateRent;

      // TRANSACTION 0: Create VM State Account + Initialize (ONLY IF CREATING NEW)
      if (!options.vmStateAccount) {
        if (options.debug) {
          console.log(
            `[FiveSDK] Step 0: Create VM state account and initialize`,
          );
        }

        const vmStateTransaction = new Transaction();
        vmStateTransaction.add(
          SystemProgram.createAccount({
            fromPubkey: deployerKeypair.publicKey,
            newAccountPubkey: vmStateKeypair.publicKey,
            lamports: vmStateRent,
            space: VM_STATE_SIZE,
            programId: programId,
          }),
        );
        vmStateTransaction.add(
          new TransactionInstruction({
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
          }),
        );
        vmStateTransaction.feePayer = deployerKeypair.publicKey;
        const vmStateBlockhash =
          await connection.getLatestBlockhash("confirmed");
        vmStateTransaction.recentBlockhash = vmStateBlockhash.blockhash;
        vmStateTransaction.partialSign(deployerKeypair);
        vmStateTransaction.partialSign(vmStateKeypair);

        const vmStateSignature = await connection.sendRawTransaction(
          vmStateTransaction.serialize(),
          {
            skipPreflight: true,
            preflightCommitment: "confirmed",
            maxRetries: options.maxRetries || 3,
          },
        );

        await connection.confirmTransaction(vmStateSignature, "confirmed");
        transactionIds.push(vmStateSignature);

        if (options.debug) {
          console.log(
            `[FiveSDK] ✅ VM state initialized: ${vmStateSignature}`,
          );
        }
      }

      // TRANSACTION 1: Create Account + InitLargeProgram
      if (options.debug) {
        console.log(
          `[FiveSDK] Step 1: Create account and initialize large program`,
        );
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
          {
            pubkey: vmStatePubkey,
            isSigner: false,
            isWritable: true,
          },
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

      const initSignature = await connection.sendRawTransaction(
        initTransaction.serialize(),
        {
          skipPreflight: true,
          preflightCommitment: "confirmed",
          maxRetries: options.maxRetries || 3,
        },
      );

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
          console.log(
            `[FiveSDK] Step ${i + 2}: Appending chunk ${i + 1}/${chunks.length} (${chunk.length} bytes)`,
          );
        }

        // Calculate additional rent needed for this chunk
        let currentInfo = await connection.getAccountInfo(
          scriptKeypair.publicKey,
        );

        // Retry logic for account info if null (eventual consistency)
        if (!currentInfo) {
          if (options.debug) console.log(`[FiveSDK] Account info null, retrying...`);
          await new Promise(resolve => setTimeout(resolve, 1000));
          currentInfo = await connection.getAccountInfo(scriptKeypair.publicKey);
          if (!currentInfo) throw new Error("Script account not found after initialization");
        }
        const newSize = currentInfo.data.length + chunk.length;
        const newRentRequired =
          await connection.getMinimumBalanceForRentExemption(newSize);
        const additionalRent = Math.max(
          0,
          newRentRequired - currentInfo.lamports,
        );

        const appendTransaction = new Transaction();

        // Add rent if needed
        if (additionalRent > 0) {
          if (options.debug) {
            console.log(
              `[FiveSDK] Adding ${additionalRent / 1e9} SOL for increased rent`,
            );
          }
          appendTransaction.add(
            SystemProgram.transfer({
              fromPubkey: deployerKeypair.publicKey,
              toPubkey: scriptKeypair.publicKey,
              lamports: additionalRent,
            }),
          );
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
            {
              pubkey: vmStatePubkey,
              isSigner: false,
              isWritable: true,
            },
          ],
          programId: programId,
          data: appendInstructionData,
        });
        appendTransaction.add(appendBytecodeInstruction);

        // Sign and send append transaction
        const appendBlockhash =
          await connection.getLatestBlockhash("confirmed");
        appendTransaction.feePayer = deployerKeypair.publicKey;
        appendTransaction.recentBlockhash = appendBlockhash.blockhash;
        appendTransaction.partialSign(deployerKeypair);

        const appendSignature = await connection.sendRawTransaction(
          appendTransaction.serialize(),
          {
            skipPreflight: true,
            preflightCommitment: "confirmed",
            maxRetries: options.maxRetries || 3,
          },
        );

        await connection.confirmTransaction(appendSignature, "confirmed");
        transactionIds.push(appendSignature);

        if (options.debug) {
          console.log(
            `[FiveSDK] ✅ Chunk ${i + 1} appended: ${appendSignature}`,
          );
        }
      }

      // Final verification
      const finalInfo = await connection.getAccountInfo(
        scriptKeypair.publicKey,
      );
      const expectedSize = SCRIPT_HEADER_SIZE + bytecode.length;

      if (options.debug) {
        console.log(`[FiveSDK] 🔍 Final verification:`);
        console.log(`[FiveSDK] Expected size: ${expectedSize} bytes`);
        console.log(`[FiveSDK] Actual size: ${finalInfo.data.length} bytes`);
        console.log(
          `[FiveSDK] Match: ${finalInfo.data.length === expectedSize ? "✅ YES" : "❌ NO"}`,
        );
      }

      return {
        success: true,
        scriptAccount,
        transactionIds,
        totalTransactions: transactionIds.length,
        deploymentCost: totalCost,
        chunksUsed: chunks.length,
        vmStateAccount: vmStatePubkey.toString(),
        logs: [
          `Deployed ${bytecode.length} bytes in ${chunks.length} chunks using ${transactionIds.length} transactions`,
        ],
      };
    } catch (error) {
      const errorMessage =
        error instanceof Error
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
  static async deployLargeProgramOptimizedToSolana(
    bytecode: FiveBytecode,
    connection: any, // Solana Connection object
    deployerKeypair: any, // Solana Keypair object
    options: {
      chunkSize?: number; // Default: 950 bytes (optimized for lower transaction overhead)
      debug?: boolean;
      network?: string;
      maxRetries?: number;
      fiveVMProgramId?: string;
      progressCallback?: (transaction: number, total: number) => void;
    } = {},
  ): Promise<{
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
  }> {
    const OPTIMIZED_CHUNK_SIZE = 500; // Larger chunks due to reduced transaction overhead
    const chunkSize = options.chunkSize || OPTIMIZED_CHUNK_SIZE;

    console.log(
      `[FiveSDK] deployLargeProgramOptimizedToSolana called with ${bytecode.length} bytes`,
    );
    console.log(`[FiveSDK] Using optimized chunk size: ${chunkSize} bytes`);
    console.log(`[FiveSDK] Expected optimization: 50-70% fewer transactions`);

    try {
      // If bytecode is small enough, use regular deployment
      if (bytecode.length <= 800) {
        if (options.debug) {
          console.log(
            `[FiveSDK] Bytecode is small (${bytecode.length} bytes), using regular deployment`,
          );
        }
        return await this.deployToSolana(
          bytecode,
          connection,
          deployerKeypair,
          {
            debug: options.debug,
            network: options.network,
            maxRetries: options.maxRetries,
          },
        );
      }

      const {
        Keypair,
        PublicKey,
        Transaction,
        TransactionInstruction,
        SystemProgram,
      } = await import("@solana/web3.js");

      // Generate script keypair
      const scriptKeypair = Keypair.generate();
      const scriptAccount = scriptKeypair.publicKey.toString();

      // PRE-ALLOCATION OPTIMIZATION: Calculate full account size upfront
      const SCRIPT_HEADER_SIZE = 128; // FIVEScriptHeaderV2::LEN
      const totalAccountSize = SCRIPT_HEADER_SIZE + bytecode.length;
      const rentLamports =
        await connection.getMinimumBalanceForRentExemption(totalAccountSize);

      const programId = new PublicKey(
        options.fiveVMProgramId || FIVE_VM_PROGRAM_ID,
      );

      // Generate VM state account for this deployment
      const vmStateKeypair = Keypair.generate();
      const VM_STATE_SIZE = 48; // FIVEVMState::LEN
      const vmStateRent =
        await connection.getMinimumBalanceForRentExemption(VM_STATE_SIZE);

      if (options.debug) {
        console.log(`[FiveSDK] Script Account: ${scriptAccount}`);
        console.log(
          `[FiveSDK] VM State Account: ${vmStateKeypair.publicKey.toString()}`,
        );
        console.log(
          `[FiveSDK] PRE-ALLOCATED full account size: ${totalAccountSize} bytes`,
        );
        console.log(
          `[FiveSDK] Full rent cost paid upfront: ${(rentLamports + vmStateRent) / 1e9} SOL`,
        );
      }

      const transactionIds: string[] = [];
      let totalCost = rentLamports + vmStateRent;

      // TRANSACTION 0: Create VM State Account + Initialize
      if (options.debug) {
        console.log(
          `[FiveSDK] Step 0: Create VM state account and initialize`,
        );
      }

      const vmStateTransaction = new Transaction();
      vmStateTransaction.add(
        SystemProgram.createAccount({
          fromPubkey: deployerKeypair.publicKey,
          newAccountPubkey: vmStateKeypair.publicKey,
          lamports: vmStateRent,
          space: VM_STATE_SIZE,
          programId: programId,
        }),
      );
      vmStateTransaction.add(
        new TransactionInstruction({
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
        }),
      );
      vmStateTransaction.feePayer = deployerKeypair.publicKey;
      const vmStateBlockhash =
        await connection.getLatestBlockhash("confirmed");
      vmStateTransaction.recentBlockhash = vmStateBlockhash.blockhash;
      vmStateTransaction.partialSign(deployerKeypair);
      vmStateTransaction.partialSign(vmStateKeypair);

      const vmStateSignature = await connection.sendRawTransaction(
        vmStateTransaction.serialize(),
        {
          skipPreflight: true,
          preflightCommitment: "confirmed",
          maxRetries: options.maxRetries || 3,
        },
      );
      await connection.confirmTransaction(vmStateSignature, "confirmed");
      transactionIds.push(vmStateSignature);

      if (options.debug) {
        console.log(
          `[FiveSDK] ✅ VM state initialized: ${vmStateSignature}`,
        );
      }

      // Split bytecode into chunks
      const chunks = this.chunkBytecode(bytecode, chunkSize);
      const firstChunk = chunks[0];
      const remainingChunks = chunks.slice(1);

      if (options.debug) {
        console.log(
          `[FiveSDK] Split into ${chunks.length} chunks (first: ${firstChunk.length} bytes, remaining: ${remainingChunks.length})`,
        );
      }

      // OPTIMIZATION 1: TRANSACTION 1 - Create Account + InitLargeProgramWithChunk (combined)
      if (options.debug) {
        console.log(
          `[FiveSDK] ⚡ OPTIMIZED Step 1: Create account + initialize with first chunk (${firstChunk.length} bytes)`,
        );
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
          {
            pubkey: vmStateKeypair.publicKey,
            isSigner: false,
            isWritable: true,
          },
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

      const initSignature = await connection.sendRawTransaction(
        initTransaction.serialize(),
        {
          skipPreflight: true,
          preflightCommitment: "confirmed",
          maxRetries: options.maxRetries || 3,
        },
      );

      await connection.confirmTransaction(initSignature, "confirmed");
      transactionIds.push(initSignature);

      if (options.debug) {
        console.log(
          `[FiveSDK] ✅ Optimized initialization completed: ${initSignature}`,
        );
        console.log(
          `[FiveSDK] First chunk (${firstChunk.length} bytes) included in initialization!`,
        );
      }

      // OPTIMIZATION 2: Group remaining chunks into multi-chunk transactions
      if (remainingChunks.length > 0) {
        const groupedChunks = this.groupChunksForOptimalTransactions(
          remainingChunks,
          500,
        ); // Leave room for multi-chunk overhead

        if (options.debug) {
          console.log(
            `[FiveSDK] ⚡ OPTIMIZATION: Grouped ${remainingChunks.length} remaining chunks into ${groupedChunks.length} transactions`,
          );
        }

        for (let groupIdx = 0; groupIdx < groupedChunks.length; groupIdx++) {
          const chunkGroup = groupedChunks[groupIdx];

          if (options.progressCallback) {
            options.progressCallback(groupIdx + 2, groupedChunks.length + 1); // +1 for init transaction
          }

          if (options.debug) {
            console.log(
              `[FiveSDK] ⚡ Step ${groupIdx + 2}: Appending ${chunkGroup.length} chunks in single transaction`,
            );
          }

          const appendTransaction = new Transaction();

          let appendInstruction: any; // TransactionInstruction from @solana/web3.js

          if (chunkGroup.length === 1) {
            // Use single-chunk AppendBytecode instruction for optimization fallback
            if (options.debug) {
              console.log(
                `[FiveSDK] Using single-chunk AppendBytecode for remaining chunk (${chunkGroup[0].length} bytes)`,
              );
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
                {
                  pubkey: vmStateKeypair.publicKey,
                  isSigner: false,
                  isWritable: true,
                },
              ],
              programId: programId,
              data: singleChunkData,
            });
          } else {
            // Use multi-chunk instruction for groups with 2+ chunks
            const multiChunkData =
              this.createMultiChunkInstructionData(chunkGroup);

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
                {
                  pubkey: vmStateKeypair.publicKey,
                  isSigner: false,
                  isWritable: true,
                },
              ],
              programId: programId,
              data: multiChunkData,
            });
          }

          appendTransaction.add(appendInstruction);

          // Sign and send multi-chunk transaction
          const appendBlockhash =
            await connection.getLatestBlockhash("confirmed");
          appendTransaction.feePayer = deployerKeypair.publicKey;
          appendTransaction.recentBlockhash = appendBlockhash.blockhash;
          appendTransaction.partialSign(deployerKeypair);

          const appendSignature = await connection.sendRawTransaction(
            appendTransaction.serialize(),
            {
              skipPreflight: true,
              preflightCommitment: "confirmed",
              maxRetries: options.maxRetries || 3,
            },
          );

          await connection.confirmTransaction(appendSignature, "confirmed");
          transactionIds.push(appendSignature);

          if (options.debug) {
            console.log(
              `[FiveSDK] ✅ Multi-chunk append completed: ${appendSignature}`,
            );
            console.log(
              `[FiveSDK] Appended ${chunkGroup.length} chunks totaling ${chunkGroup.reduce((sum, chunk) => sum + chunk.length, 0)} bytes`,
            );
          }
        }
      }

      // Explicitly finalize the script to ensure upload_mode is cleared
      if (options.debug) {
        console.log(`[FiveSDK] Sending FinalizeScript instruction to complete deployment`);
      }
      const finalizeTransaction = new Transaction();
      finalizeTransaction.add(
        new TransactionInstruction({
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
          ],
          programId: programId,
          data: Buffer.from([7]), // FinalizeScript discriminator
        }),
      );
      finalizeTransaction.feePayer = deployerKeypair.publicKey;
      const finalizeBlockhash = await connection.getLatestBlockhash("confirmed");
      finalizeTransaction.recentBlockhash = finalizeBlockhash.blockhash;
      finalizeTransaction.partialSign(deployerKeypair);

      const finalizeSignature = await connection.sendRawTransaction(
        finalizeTransaction.serialize(),
        {
          skipPreflight: true,
          preflightCommitment: "confirmed",
          maxRetries: options.maxRetries || 3,
        },
      );
      // Use custom polling for finalize to handle validator latency
      const finalizeConfirmation = await this.pollForConfirmation(
        connection,
        finalizeSignature,
        "confirmed",
        120000, // 120 second timeout
        options.debug
      );
      if (!finalizeConfirmation.success) {
        console.error(`[FiveSDK] FinalizeScript confirmation failed: ${finalizeConfirmation.error}`);
      }
      transactionIds.push(finalizeSignature);
      if (options.debug) {
        console.log(`[FiveSDK] ✅ FinalizeScript completed: ${finalizeSignature}`);
      }

      // Calculate optimization savings
      const traditionalTransactionCount = 1 + chunks.length; // 1 init + N appends
      const optimizedTransactionCount = transactionIds.length;
      const transactionsSaved =
        traditionalTransactionCount - optimizedTransactionCount;
      const estimatedCostSaved = transactionsSaved * 0.000005 * 1e9; // Estimate 5000 lamports per transaction saved

      if (options.debug) {
        console.log(`[FiveSDK] 🎉 OPTIMIZATION RESULTS:`);
        console.log(
          `[FiveSDK]   Traditional method: ${traditionalTransactionCount} transactions`,
        );
        console.log(
          `[FiveSDK]   Optimized method: ${optimizedTransactionCount} transactions`,
        );
        console.log(
          `[FiveSDK]   Transactions saved: ${transactionsSaved} (${Math.round((transactionsSaved / traditionalTransactionCount) * 100)}% reduction)`,
        );
        console.log(
          `[FiveSDK]   Estimated cost saved: ${estimatedCostSaved / 1e9} SOL`,
        );
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
    } catch (error: any) {
      console.error("[FiveSDK] Optimized deployment failed:", error);

      const errorMessage =
        error instanceof Error ? error.message : "Unknown deployment error";

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
  private static groupChunksForOptimalTransactions(
    chunks: Uint8Array[],
    maxGroupSize: number,
  ): Uint8Array[][] {
    const groups: Uint8Array[][] = [];
    let currentGroup: Uint8Array[] = [];
    let currentGroupSize = 0;

    // Account for multi-chunk overhead: 1 byte (num_chunks) + 2 bytes per chunk (length)
    const getGroupOverhead = (numChunks: number) => 1 + numChunks * 2;

    for (const chunk of chunks) {
      const groupOverhead = getGroupOverhead(currentGroup.length + 1);
      const newGroupSize = currentGroupSize + chunk.length + 2; // +2 for chunk length prefix

      if (currentGroup.length === 0) {
        // Always add first chunk to empty group
        currentGroup.push(chunk);
        currentGroupSize = newGroupSize;
      } else if (
        newGroupSize + groupOverhead <= maxGroupSize &&
        currentGroup.length < 8
      ) {
        // Add to current group if it fits and doesn't exceed max chunks per transaction
        currentGroup.push(chunk);
        currentGroupSize = newGroupSize;
      } else {
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
   * Format: [discriminator(5), chunk1_data + chunk2_data + ...]
   */
  private static createMultiChunkInstructionData(chunks: Uint8Array[]): Buffer {
    if (chunks.length < 2 || chunks.length > 10) {
      throw new Error(
        `Invalid chunk count for multi-chunk instruction: ${chunks.length}`,
      );
    }

    // Just concatenate all chunks with discriminator 5
    // This allows the on-chain program (which only sees bytes) to append them as a single stream
    const buffers: Buffer[] = [
      Buffer.from([5]), // AppendBytecode discriminator
    ];

    for (const chunk of chunks) {
      buffers.push(Buffer.from(chunk));
    }

    return Buffer.concat(buffers);
  }

  /**
   * Split bytecode into chunks of specified size
   */
  private static chunkBytecode(
    bytecode: Uint8Array,
    chunkSize: number,
  ): Uint8Array[] {
    const chunks: Uint8Array[] = [];
    for (let i = 0; i < bytecode.length; i += chunkSize) {
      const chunk = bytecode.slice(i, Math.min(i + chunkSize, bytecode.length));
      chunks.push(chunk);
    }
    return chunks;
  }
}

// Export helper functions
export const createFiveSDK = (config?: FiveSDKConfig) => new FiveSDK(config);

// Export default
export default FiveSDK;
