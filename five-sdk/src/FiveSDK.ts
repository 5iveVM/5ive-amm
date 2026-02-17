/**
 * Five SDK client for Five VM scripts.
 */

import {
  FiveSDKConfig,
  FiveScriptSource,
  FiveBytecode,
  CompilationOptions,
  CompilationResult,
  DeploymentOptions,
  SerializedDeployment,
  SerializedExecution,
  FIVE_VM_PROGRAM_ID,
  FiveCompiledFile,
  FiveFunction,
  FunctionNameEntry,
  FeeInformation,
  FiveSDKError,
} from "./types.js";
import { BytecodeCompiler } from "./compiler/BytecodeCompiler.js";
import { ParameterEncoder } from "./encoding/ParameterEncoder.js";
import { PDAUtils, RentCalculator } from "./crypto/index.js";
import {
  ScriptMetadataParser,
  MetadataCache,
  ScriptMetadata,
} from "./metadata/index.js";
import { normalizeAbiFunctions, resolveFunctionIndex } from "./utils/abi.js";
import { validator, Validators } from "./validation/index.js";
import { ProgramIdResolver } from "./config/ProgramIdResolver.js";

import * as Deploy from "./modules/deploy.js";
import * as Execute from "./modules/execute.js";
import * as Fees from "./modules/fees.js";
import * as VMState from "./modules/vm-state.js";
import * as Accounts from "./modules/accounts.js";
import * as StateDiff from "./modules/state-diff.js";
import * as Namespaces from "./modules/namespaces.js";
import * as Admin from "./modules/admin.js";

/**
 * Main Five SDK class - entry point for all Five VM interactions
 */
export class FiveSDK {
  private static compiler: BytecodeCompiler | null = null;
  private static parameterEncoder: ParameterEncoder | null = null;
  private static metadataCache: MetadataCache = new MetadataCache();

  private fiveVMProgramId?: string;
  private debug: boolean;
  private network?: string;

  static admin = {
    generateInitializeVmStateInstruction: Admin.generateInitializeVmStateInstruction,
    generateSetFeesInstruction: Admin.generateSetFeesInstruction,
    generateInitFeeVaultInstruction: Admin.generateInitFeeVaultInstruction,
    generateWithdrawScriptFeesInstruction: Admin.generateWithdrawScriptFeesInstruction,
    initializeVmStateOnSolana: Admin.initializeVmStateOnSolana,
    setFeesOnSolana: Admin.setFeesOnSolana,
    initFeeVaultOnSolana: Admin.initFeeVaultOnSolana,
    withdrawScriptFeesOnSolana: Admin.withdrawScriptFeesOnSolana,
  };

  constructor(config: FiveSDKConfig = {}) {
    // Store the config but resolve at call time
    this.fiveVMProgramId = config.fiveVMProgramId;
    this.debug = config.debug || false;
    this.network = (config as any).network;

    if (this.debug) {
      const resolved = ProgramIdResolver.resolveOptional(this.fiveVMProgramId);
      if (resolved) {
        console.log(
          `[FiveSDK] Initialized with Five VM Program: ${resolved}`,
        );
      } else {
        console.log(
          `[FiveSDK] Initialized (program ID will be resolved at call time)`,
        );
      }
    }
  }

  getConfig(): FiveSDKConfig & { network?: string } {
    return {
      fiveVMProgramId: this.fiveVMProgramId,
      debug: this.debug,
      network: this.network,
    };
  }

  private static async initializeComponents(debug = false) {
    if (!this.compiler) {
      this.compiler = new BytecodeCompiler({ debug });
    }
    if (!this.parameterEncoder) {
      this.parameterEncoder = new ParameterEncoder(debug);
    }
  }

  // ==================== Static Factory Methods ====================

  static create(options: { debug?: boolean; fiveVMProgramId?: string } = {}): FiveSDK {
    return new FiveSDK(options);
  }

  static devnet(options: { debug?: boolean; fiveVMProgramId?: string } = {}): FiveSDK {
    return new FiveSDK({ ...options, network: "devnet" });
  }

  static mainnet(options: { debug?: boolean; fiveVMProgramId?: string } = {}): FiveSDK {
    return new FiveSDK({ ...options, network: "mainnet" });
  }

  static localnet(options: { debug?: boolean; fiveVMProgramId?: string } = {}): FiveSDK {
    return new FiveSDK({ ...options, network: "localnet" });
  }

  // ==================== Program ID Defaults ====================

  /**
   * Set the default program ID for all SDK instances and operations
   * Useful when deploying to a known program ID across your application
   * @param programId - Solana public key (base58 encoded)
   */
  static setDefaultProgramId(programId: string): void {
    ProgramIdResolver.setDefault(programId);
  }

  /**
   * Get the currently set default program ID
   * @returns The default program ID, or undefined if not set
   */
  static getDefaultProgramId(): string | undefined {
    return ProgramIdResolver.getDefault();
  }

  // ==================== Namespaces ====================

  static canonicalizeNamespace(value: string) {
    return Namespaces.canonicalizeScopedNamespace(value);
  }

  static namespaceSeedBytes(value: string): Uint8Array {
    return Namespaces.namespaceSeedBytes(value);
  }

  static resolveNamespaceFromLockfile(value: string, lockfile: any): string | undefined {
    return Namespaces.resolveNamespaceFromLockfile(value, lockfile);
  }

  static async deriveNamespaceAccounts(value: string, fiveVMProgramId: string) {
    return Namespaces.deriveNamespaceAccounts(value, fiveVMProgramId);
  }

  static async registerNamespaceTldOnChain(
    namespaceValue: string,
    options: {
      managerScriptAccount: string;
      connection: any;
      signerKeypair: any;
      fiveVMProgramId?: string;
      debug?: boolean;
    },
  ) {
    return Namespaces.registerNamespaceTldOnChain(namespaceValue, options);
  }

  static async bindNamespaceOnChain(
    namespaceValue: string,
    scriptAccount: string,
    options: {
      managerScriptAccount: string;
      connection: any;
      signerKeypair: any;
      fiveVMProgramId?: string;
      debug?: boolean;
    },
  ) {
    return Namespaces.bindNamespaceOnChain(namespaceValue, scriptAccount, options);
  }

  static async resolveNamespaceOnChain(
    namespaceValue: string,
    options: {
      managerScriptAccount: string;
      connection: any;
      signerKeypair: any;
      fiveVMProgramId?: string;
      debug?: boolean;
    },
  ) {
    return Namespaces.resolveNamespaceOnChain(namespaceValue, options);
  }

  // ==================== Script Compilation ====================

  static async compile(
    source: FiveScriptSource | string,
    options: CompilationOptions & { debug?: boolean } = {},
  ): Promise<CompilationResult> {
    const sourceContent = typeof source === 'string' ? source : source.content;
    const sourceFilename = typeof source === 'string' ? 'unknown.v' : source.filename || 'unknown.v';

    Validators.sourceCode(sourceContent);
    Validators.options(options);

    await this.initializeComponents(options.debug);

    try {
      const result = await this.compiler!.compile(source, options);

      if (result.success && result.bytecode) {
        let abiData: any = result.abi ?? { functions: [], fields: [] };
        if (options.debug) {
          try {
            const generatedABI = await this.compiler!.generateABI(source);
            if (generatedABI && generatedABI.functions) {
              abiData = generatedABI;
            }
          } catch (abiError) {
            console.warn("[FiveSDK] ABI generation failed:", abiError);
          }
        }

        const functions = normalizeAbiFunctions(abiData.functions ?? abiData)
          .map<FiveFunction>((func) => ({
            name: func.name,
            index: func.index,
            parameters: func.parameters?.map((param) => ({
              name: param.name,
              type: param.type as any,
              param_type: param.param_type as any,
              optional: param.optional ?? false,
              is_account: param.is_account ?? param.isAccount ?? false,
              isAccount: param.isAccount ?? param.is_account ?? false,
              attributes: Array.isArray(param.attributes) ? [...param.attributes] : [],
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
          metrics: options.includeMetrics ? result.metrics : undefined,
          version: "1.0",
        };

        if (result.bytecode) {
          const functionNames = await this.getFunctionNames(result.bytecode);
          result.functionNames = functionNames;
          result.publicFunctionNames = functionNames.map((f) => f.name);
        }
      }

      return result;
    } catch (error) {
      throw new FiveSDKError(
        `Compilation failed: ${error instanceof Error ? error.message : "Unknown error"}`,
        "COMPILATION_ERROR",
      );
    }
  }

  static async compileModules(
    mainSource: FiveScriptSource | string,
    modules: Array<{ name: string; source: string }>,
    options: CompilationOptions & { debug?: boolean } = {},
  ): Promise<CompilationResult> {
    const mainSourceObj = typeof mainSource === 'string' ? { content: mainSource, filename: 'main.v' } : mainSource;
    Validators.options(options);
    await this.initializeComponents(options.debug);
    if (!this.compiler) throw new FiveSDKError("Compiler not initialized", "COMPILER_ERROR");
    return this.compiler.compileModules(mainSourceObj, modules, options);
  }

  static async compileWithDiscovery(
    entryPoint: string,
    options: CompilationOptions & { debug?: boolean } = {},
  ): Promise<CompilationResult> {
    Validators.options(options);
    await this.initializeComponents(options.debug);
    if (typeof (this.compiler as any).compileWithDiscovery === 'function') {
      return (this.compiler as any).compileWithDiscovery(entryPoint, options);
    }
    return this.compileFile(entryPoint, options);
  }

  static async discoverModules(
    entryPoint: string,
    options: { debug?: boolean } = {}
  ): Promise<string[]> {
    await this.initializeComponents(options.debug);
    if (typeof (this.compiler as any).discoverModules === 'function') {
      return (this.compiler as any).discoverModules(entryPoint);
    }
    throw new Error("discoverModules not available in current compiler version");
  }

  static async compileFile(
    filePath: string,
    options: CompilationOptions & { debug?: boolean } = {},
  ): Promise<CompilationResult> {
    Validators.filePath(filePath);
    Validators.options(options);
    await this.initializeComponents(options.debug);
    return this.compiler!.compileFile(filePath, options);
  }

  // ==================== Five File Format Utilities ====================

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
      return { bytecode, abi: fiveFile.abi, debug: fiveFile.debug };
    } catch (error) {
      throw new FiveSDKError(
        `Failed to load .five file: ${error instanceof Error ? error.message : "Unknown error"}`,
        "FILE_LOAD_ERROR",
      );
    }
  }

  static extractBytecode(fiveFile: FiveCompiledFile): FiveBytecode {
    return new Uint8Array(Buffer.from(fiveFile.bytecode, "base64"));
  }

  static resolveFunctionIndex(abi: any, functionName: string): number {
    return resolveFunctionIndex(abi, functionName);
  }

  // ==================== WASM VM Direct Execution ====================

  static async executeLocally(
    bytecode: FiveBytecode,
    functionName: string | number,
    parameters: any[] = [],
    options: {
      debug?: boolean;
      trace?: boolean;
      computeUnitLimit?: number;
      abi?: any;
      accounts?: string[];
    } = {},
  ): Promise<any> {
    return Execute.executeLocally(bytecode, functionName, parameters, options);
  }

  static async execute(
    source: FiveScriptSource | string,
    functionName: string | number,
    parameters: any[] = [],
    options: {
      debug?: boolean;
      trace?: boolean;
      optimize?: boolean;
      computeUnitLimit?: number;
      vmStateAccount?: string;
      accounts?: string[];
    } = {},
  ) {
    await this.initializeComponents(options.debug);
    return Execute.execute(this.compiler!, source, functionName, parameters, options);
  }

  static async validateBytecode(
    bytecode: FiveBytecode,
    options: { debug?: boolean } = {},
  ): Promise<{
    valid: boolean;
    errors?: string[];
    metadata?: any;
    functions?: any[];
  }> {
    Validators.bytecode(bytecode);
    Validators.options(options);
    await this.initializeComponents(options.debug);

    // Using compile's validateSource? No, validateBytecode logic is in FiveSDK using loadWasmVM.
    // I moved loadWasmVM to instance.ts but validateBytecode logic was in FiveSDK.
    // I should have moved validateBytecode logic to execute.ts or validation/index.ts or a new module.
    // I forgot to move `validateBytecode`. It was in `FiveSDK.ts`.
    // I can invoke `loadWasmVM` here and call it.

    try {
      const { loadWasmVM } = await import("./wasm/instance.js");
      const wasmVM = await loadWasmVM();
      return await wasmVM.validateBytecode(bytecode);
    } catch (error) {
      return {
        valid: false,
        errors: [error instanceof Error ? error.message : "Unknown validation error"],
      };
    }
  }

  // ==================== Deployment ====================

  static async generateDeployInstruction(
    bytecode: FiveBytecode,
    deployer: string,
    options: DeploymentOptions & { debug?: boolean } = {},
    connection?: any,
  ): Promise<SerializedDeployment> {
    return Deploy.generateDeployInstruction(bytecode, deployer, options, connection);
  }

  // ==================== Execution ====================

  static async generateExecuteInstruction(
    scriptAccount: string,
    functionName: string | number,
    parameters: any[] = [],
    accounts: string[] = [],
    connection?: any,
    options: {
      debug?: boolean;
      computeUnitLimit?: number;
      vmStateAccount?: string;
      fiveVMProgramId?: string;
      abi?: any;
      adminAccount?: string;
      estimateFees?: boolean;
      accountMetadata?: Map<string, { isSigner: boolean; isWritable: boolean; isSystemAccount?: boolean }>;
    } = {},
  ): Promise<SerializedExecution> {
    return Execute.generateExecuteInstruction(
      scriptAccount,
      functionName,
      parameters,
      accounts,
      connection,
      options
    );
  }

  // ==================== VM State & Fees ====================

  static async getVMState(connection: any, fiveVMProgramId?: string) {
    return VMState.getVMState(connection, fiveVMProgramId);
  }

  static async getFees(connection: any, fiveVMProgramId?: string) {
    return Fees.getFees(connection, fiveVMProgramId);
  }

  static async calculateDeployFee(
    bytecodeSize: number,
    connection?: any,
    fiveVMProgramId?: string,
  ): Promise<FeeInformation> {
    return Fees.calculateDeployFee(bytecodeSize, connection, fiveVMProgramId);
  }

  static async calculateExecuteFee(
    connection?: any,
    fiveVMProgramId?: string,
  ): Promise<FeeInformation> {
    return Fees.calculateExecuteFee(connection, fiveVMProgramId);
  }

  static async getFeeInformation(
    bytecodeSize: number,
    connection?: any,
    fiveVMProgramId?: string,
  ) {
    return Fees.getFeeInformation(bytecodeSize, connection, fiveVMProgramId);
  }

  // ==================== Script Analysis ====================

  static async getScriptMetadata(
    scriptAccount: string,
    connection?: any,
  ): Promise<{ functions: any[] }> {
    validator.validateBase58Address(scriptAccount, "scriptAccount");
    if (!connection) throw new Error("No connection provided");
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

  static async getScriptMetadataWithConnection(
    scriptAccount: string,
    connection: any,
  ): Promise<ScriptMetadata> {
    validator.validateBase58Address(scriptAccount, "scriptAccount");
    return ScriptMetadataParser.getScriptMetadata(connection, scriptAccount);
  }

  static parseScriptMetadata(
    accountData: Uint8Array,
    address: string,
  ): ScriptMetadata {
    Validators.bytecode(accountData);
    validator.validateBase58Address(address, "address");
    return ScriptMetadataParser.parseMetadata(accountData, address);
  }

  static async getCachedScriptMetadata(
    scriptAccount: string,
    connection: any,
    cacheTTL: number = 5 * 60 * 1000,
  ): Promise<ScriptMetadata> {
    validator.validateBase58Address(scriptAccount, "scriptAccount");
    validator.validateNumber(cacheTTL, "cacheTTL");
    return this.metadataCache.getMetadata(
      scriptAccount,
      (address) => ScriptMetadataParser.getScriptMetadata(connection, address),
      cacheTTL,
    );
  }

  static invalidateMetadataCache(scriptAccount: string): void {
    validator.validateBase58Address(scriptAccount, "scriptAccount");
    this.metadataCache.invalidate(scriptAccount);
  }

  static getMetadataCacheStats(): any {
    return this.metadataCache.getStats();
  }

  // ==================== Utilities ====================

  static async executeOnSolana(
    scriptAccount: string,
    connection: any,
    signerKeypair: any,
    functionName: string | number,
    parameters: any[] = [],
    accounts: string[] = [],
    options: any = {},
  ) {
    return Execute.executeOnSolana(
      scriptAccount,
      connection,
      signerKeypair,
      functionName,
      parameters,
      accounts,
      options,
    );
  }

  static async executeScriptAccount(
    scriptAccount: string,
    functionIndex: number = 0,
    parameters: any[] = [],
    connection: any,
    signerKeypair: any,
    options: any = {},
  ) {
    return Execute.executeScriptAccount(
      scriptAccount,
      functionIndex,
      parameters,
      connection,
      signerKeypair,
      options,
    );
  }

  static async getFunctionNames(
    bytecode: FiveBytecode,
  ): Promise<FunctionNameEntry[]> {
    await this.initializeComponents(false);
    try {
      const namesJson = await this.compiler!.getFunctionNames(bytecode);
      if (Array.isArray(namesJson)) return namesJson as FunctionNameEntry[];
      return JSON.parse(namesJson as any) as FunctionNameEntry[];
    } catch (error) {
      console.warn("[FiveSDK] Failed to extract function names:", error);
      return [];
    }
  }

  static async callFunctionByName(
    scriptAccount: string,
    functionName: string,
    parameters: any[] = [],
    accounts: string[] = [],
    connection?: any,
    options: any = {},
  ): Promise<SerializedExecution> {
    const available = await this.getFunctionNamesFromScriptAccount(
      scriptAccount,
      connection,
    );
    if (!available) {
      throw new Error(`Cannot resolve function name "${functionName}"`);
    }
    const funcInfo = available.find((f) => f.name === functionName);
    if (!funcInfo) {
      throw new Error(`Function "${functionName}" not found`);
    }
    return this.executeByIndex(
      scriptAccount,
      funcInfo.function_index,
      parameters,
      accounts,
      connection,
      options,
    );
  }

  static async executeByIndex(
    scriptAccount: string,
    functionIndex: number,
    parameters: any[] = [],
    accounts: string[] = [],
    connection?: any,
    options: any = {},
  ): Promise<SerializedExecution> {
    return this.generateExecuteInstruction(
      scriptAccount,
      functionIndex,
      parameters,
      accounts,
      connection,
      options,
    );
  }

  static async getFunctionNamesFromScriptAccount(
    scriptAccount: string,
    connection?: any,
  ): Promise<FunctionNameEntry[] | null> {
    if (!connection) return null;
    try {
      const { PublicKey } = await import("@solana/web3.js");
      const accountInfo = await connection.getAccountInfo(new PublicKey(scriptAccount), "confirmed");
      if (!accountInfo) return null;

      const data = accountInfo.data;
      // Extract bytecode from account data.

      const scriptHeaderSize = 64;
      let bytecode = data;
      if (data.length >= scriptHeaderSize && data[0] === 0x35 && data[1] === 0x49 && data[2] === 0x56 && data[3] === 0x45) {
         const readU32LE = (buffer: Uint8Array, offset: number) => buffer[offset] | (buffer[offset + 1] << 8) | (buffer[offset + 2] << 16) | (buffer[offset + 3] << 24);
         const bytecodeLen = readU32LE(data, 48);
         const metadataLen = readU32LE(data, 52);
         const bytecodeStart = scriptHeaderSize + metadataLen;
         if (bytecodeLen > 0 && (bytecodeStart + bytecodeLen) <= data.length) {
             bytecode = data.slice(bytecodeStart, bytecodeStart + bytecodeLen);
         }
      }
      return await this.getFunctionNames(bytecode);
    } catch (e) {
      return null;
    }
  }

  static async createDeploymentTransaction(
    bytecode: FiveBytecode,
    connection: any,
    deployerPublicKey: any,
    options: any = {},
  ) {
    return Deploy.createDeploymentTransaction(bytecode, connection, deployerPublicKey, options);
  }

  static async deployToSolana(
    bytecode: FiveBytecode,
    connection: any,
    deployerKeypair: any,
    options: any = {},
  ) {
    return Deploy.deployToSolana(bytecode, connection, deployerKeypair, options);
  }

  static async deployLargeProgramToSolana(
    bytecode: FiveBytecode,
    connection: any,
    deployerKeypair: any,
    options: any = {},
  ) {
    return Deploy.deployLargeProgramToSolana(bytecode, connection, deployerKeypair, options);
  }

  static async deployLargeProgramOptimizedToSolana(
    bytecode: FiveBytecode,
    connection: any,
    deployerKeypair: any,
    options: any = {},
  ) {
    return Deploy.deployLargeProgramOptimizedToSolana(bytecode, connection, deployerKeypair, options);
  }

  static async fetchAccountAndDeserialize(
    accountAddress: string,
    connection: any,
    options: any = {},
  ) {
    return Accounts.fetchAccountAndDeserialize(accountAddress, connection, options);
  }

  static async fetchMultipleAccountsAndDeserialize(
    accountAddresses: string[],
    connection: any,
    options: any = {},
  ) {
    return Accounts.fetchMultipleAccountsAndDeserialize(accountAddresses, connection, options);
  }

  static async deserializeParameters(
    instructionData: Uint8Array,
    expectedTypes: string[] = [],
    options: any = {},
  ) {
    return Accounts.deserializeParameters(instructionData, expectedTypes, options);
  }

  static async validateBytecodeEncoding(
    bytecode: Uint8Array,
    debug: boolean = false,
  ) {
    return Accounts.validateBytecodeEncoding(bytecode, debug);
  }

  static async executeWithStateDiff(
    scriptAccount: string,
    connection: any,
    signerKeypair: any,
    functionName: string | number,
    parameters: any[] = [],
    options: any = {},
  ) {
    return StateDiff.executeWithStateDiff(
      scriptAccount,
      connection,
      signerKeypair,
      functionName,
      parameters,
      options,
    );
  }
}

// Export helper functions
export const createFiveSDK = (config?: FiveSDKConfig) => new FiveSDK(config);

// Export default
export default FiveSDK;
