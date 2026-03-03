/**
 * Type definitions for Five SDK.
 */

// ==================== Project Configuration ====================

export interface ProjectConfig {
  name: string;
  version: string;
  description?: string;
  sourceDir: string;
  buildDir: string;
  target?: CompilationTarget;
  entryPoint?: string;
  outputArtifactName?: string;
  cluster?: string;
  commitment?: string;
  rpcUrl?: string;
  programId?: string;
  namespaceManager?: string;
  keypairPath?: string;
  optimizations?: ProjectOptimizations;
  dependencies?: ProjectDependency[];
  wasm?: {
    loader?: 'auto' | 'node' | 'bundler';
    modulePaths?: string[];
  };
}

export interface ProjectOptimizations {
  enableCompression?: boolean;
  enableConstraintOptimization?: boolean;
  optimizationLevel?: 'production';
}

export interface ProjectDependency {
  name: string;
  version: string;
  type: 'five' | 'wasm' | 'solana';
  path?: string;
}

// ==================== Compilation Types ====================

export type CompilationTarget = 'vm' | 'solana' | 'debug' | 'test' | 'deployment';

export interface CompilationOptions {
  /** Source file path (if applicable) */
  sourceFile?: string;

  /** Output file path (if applicable) */
  outputFile?: string;

  /** ABI output file path (if applicable) */
  abiOutputFile?: string;

  /** Enable optimizations */
  optimize?: boolean;

  /** Optimization level */
  optimizationLevel?: 'production';

  /** Target environment */
  target?: CompilationTarget;

  /** Include debug info in bytecode */
  includeDebugInfo?: boolean;
  debug?: boolean;

  /** Max allowed bytecode size in bytes */
  maxBytecodeSize?: number;
  maxSize?: number;

  /** Enable general compression */
  enableCompression?: boolean;

  /** Include compilation metrics */
  includeMetrics?: boolean;

  /** Metrics output format */
  metricsFormat?: "json" | "csv" | "toml";

  /** Error output format */
  errorFormat?: "terminal" | "json" | "lsp";

  /** Include comprehensive metrics */
  comprehensiveMetrics?: boolean;

  /** Output file for metrics */
  metricsOutput?: string;

  /** Enable constraint cache */
  enable_constraint_cache?: boolean;

  /** Flat namespace for modules */
  flatNamespace?: boolean;
}

export type FiveScript = FiveScriptSource | FiveBytecode;

export interface CompilationResult {
  success: boolean;
  bytecode?: Uint8Array;
  abi?: any;
  metadata?: CompilationMetadata;
  errors?: CompilationError[];
  warnings?: CompilationWarning[];
  diagnostics?: CompilationError[];
  disassembly?: string[];
  metrics?: CompilationMetrics;
  metricsReport?: CompilationMetricsReport;
  fiveFile?: FiveCompiledFile;
  publicFunctionNames?: string[];
  functionNames?: string[] | FunctionNameEntry[];
  formattedErrorsTerminal?: string;
  formattedErrorsJson?: string;
}

export interface CompilationMetadata {
  sourceFile: string;
  timestamp: string;
  compilerVersion: string;
  target: CompilationTarget;
  optimizations: string[];
  originalSize: number;
  compressedSize: number;
  compressionRatio: number;
  sourceSize?: number; // legacy
  bytecodeSize?: number;
  compilationTime?: number;
  functions?: any[];
}

export interface CompilationError {
  type: string; // 'syntax' | 'semantic' | 'type' | 'runtime' | 'enhanced'
  message: string;
  line?: number;
  column?: number;
  sourceLocation?: string;
  suggestion?: string;

  // Enhanced error fields
  code?: string;
  severity?: string;
  category?: string;
  description?: string;
  location?: any;
  suggestions?: Array<string | {
    message: string;
    explanation?: string;
    confidence?: number;
    codeSuggestion?: string;
  }>;
  sourceLine?: string;
  sourceSnippet?: string;
  rendered?: string;
  raw?: any;
}

export interface CompilationWarning {
  type: string;
  message: string;
  line?: number;
  column?: number;
  sourceLocation?: string;
}

export interface CompilationMetrics {
  compilationTime: number;
  memoryUsed: number;
  optimizationTime: number;
  bytecodeSize: number;
  instructionCount: number;
  functionCount: number;
}

export interface CompilationMetricsReport {
  format: string;
  exported: string;
  detailed?: any;
}

// ==================== VM Execution Types ====================

export interface VMExecutionOptions {
  bytecode: Uint8Array;
  inputData?: Uint8Array;
  accounts?: AccountInfo[];
  maxComputeUnits?: number;
  enableLogging?: boolean;
  enableProfiling?: boolean;
}

export interface VMExecutionResult {
  success: boolean;
  result?: any;
  error?: VMError;
  logs?: string[];
  computeUnitsUsed?: number;
  executionTime?: number;
  memoryUsage?: MemoryUsage;
  profileData?: ProfileData;
  status?: string;
  stoppedAt?: string;
  errorMessage?: string;
}

export interface VMError {
  type: string;
  message: string;
  instructionPointer?: number;
  stackTrace?: string[];
  errorCode?: number;
  details?: any;
}

export interface AccountInfo {
  pubkey?: string;
  key?: Uint8Array;
  lamports: number;
  data: Uint8Array;
  owner?: Uint8Array | string;
  executable?: boolean;
  rentEpoch?: number;
  isWritable?: boolean;
  isSigner?: boolean;
}

export interface MemoryUsage {
  heapUsed: number;
  heapTotal: number;
  external: number;
  arrayBuffers: number;
  rss: number;
}

export interface ProfileData {
  instructionCounts: Map<string, number>;
  functionCallCounts: Map<string, number>;
  hotspots: Array<{
    instruction: string;
    count: number;
    percentage: number;
  }>;
}

// ==================== Common Utilities ====================

export interface Logger {
  debug(message: string, ...args: any[]): void;
  info(message: string, ...args: any[]): void;
  warn(message: string, ...args: any[]): void;
  error(message: string, ...args: any[]): void;
}

export interface CLIError extends Error {
  code: string;
  exitCode?: number;
  category: string;
  details?: any;
}

// ==================== Legacy SDK Types (for compatibility) ====================

/** @deprecated Use ProgramIdResolver with explicit config or cluster config instead. */
export const FIVE_VM_PROGRAM_ID = "4Qxf3pbCse2veUgZVMiAm3nWqJrYo2pT4suxHKMJdK1d";

export interface FiveSDKConfig {
  network?: string;
  connection?: any;
  payer?: any;
  confirmTransactionInitialTimeout?: number;
  debug?: boolean;
  fiveVMProgramId?: string;
}

export interface FiveScriptSource {
  filename: string;
  content: string;
}

export type FiveBytecode = Uint8Array;

export type ScriptAccount = SerializableAccount;

export interface SerializableAccount {
  pubkey: string;
  data: string; // base64
  lamports: number;
  owner: string;
  executable: boolean;
  isSigner?: boolean;
  isWritable?: boolean;
}

export interface DeploymentOptions {
  network?: string;
  payer?: any;
  space?: number;
  programId?: string;
  scriptAccount?: any;
  scriptSeed?: string;
  extraLamports?: number;
  debug?: boolean;
  permissions?: number;
  adminAccount?: string; // Admin account for fee collection
  estimateFees?: boolean; // Fee estimation (true by default when connection provided, false to disable)
  fiveVMProgramId?: string; // Custom Five VM Program ID
  exportMetadata?: {
    methods?: string[];
    interfaces?: Array<{
      name: string;
      methodMap?: Record<string, string>;
    }>;
  };
  namespace?: string;
}

export interface FeeInformation {
  // Deprecated for flat-lamport VM fees; kept for compatibility.
  feeBps: number;
  basisLamports: number;          // Rent or tx fee basis
  feeLamports: number;            // Flat VM fee in lamports
  totalEstimatedCost: number;     // Basis + fee
  costBreakdown: {
    basis: string;                // SOL formatted
    fee: string;                  // SOL formatted
    total: string;                // SOL formatted
  };
}

export interface SerializedDeployment {
  programId: string;
  signature?: string;
  instruction?: any;
  scriptAccount?: string; // Pubkey
  requiredSigners?: string[];
  estimatedCost?: number;
  bytecodeSize?: number;
  setupInstructions?: any;
  adminAccount?: string;
  feeInformation?: FeeInformation;  // Fee estimation data
}

export interface SerializedExecution {
  transactionId?: string;
  result?: any;
  logs?: string[];
  estimatedComputeUnits?: number;
  instruction?: any;
  scriptAccount?: string; // Pubkey
  parameters?: any;
  requiredSigners?: string[];
  adminAccount?: string;
  feeInformation?: FeeInformation;  // Fee estimation data
}

export interface SerializedInstruction {
  programId: string;
  keys: Array<{ pubkey: string; isSigner: boolean; isWritable: boolean }>;
  data: string; // base64
}

export interface ExecutionOptions extends VMExecutionOptions {
  // Alias or extension
}

export class FiveSDKError extends Error {
  code: string;
  details?: any;

  constructor(message: string, code: string, details?: any) {
    super(message);
    this.code = code;
    this.details = details;
    this.name = 'FiveSDKError';
  }
}

export class ExecutionSDKError extends FiveSDKError {
  constructor(message: string, details?: any) {
    super(message, 'EXECUTION_ERROR', details);
    this.name = 'ExecutionSDKError';
  }
}

export class CompilationSDKError extends FiveSDKError {
  constructor(message: string, details?: any) {
    super(message, 'COMPILATION_ERROR', details);
    this.name = 'CompilationSDKError';
  }
}

export interface EncodedParameters {
  data: Uint8Array;
}

export interface EncodedParameter {
  type: string | number;
  value: any;
}

export interface ParameterEncodingOptions {
  none?: boolean;
  strict?: boolean;
}

export class ParameterEncodingError extends Error {
  details?: any;
  constructor(message: string, details?: any) {
    super(message);
    this.details = details;
    this.name = 'ParameterEncodingError';
  }
}

export interface FiveCompiledFile {
  filename: string;
  bytecode: string; // Base64 encoded
  metadata?: CompilationMetadata;
  abi?: any;
  debug?: any;
  metrics?: any;
  disassembly?: string[];
  version: string;
}

export interface FiveFunction {
  name: string;
  index?: number;
  parameters: FiveParameter[];
  returnType?: FiveType;
}

export interface FiveParameter {
  name: string;
  type: FiveType;
  param_type?: FiveType;
  optional?: boolean;
  is_account?: boolean;
  isAccount?: boolean;
  attributes?: string[];
}

export type FiveType = string; // Placeholder for now

export interface FunctionNameEntry {
  name: string;
  index: number;
  function_index?: number;
}

// ==================== Provider & Wallet Types ====================

export interface Provider {
  connection: any; // Connection from @solana/web3.js
  publicKey?: { toBase58(): string };
  sendAndConfirm?: (
    tx: any, // Transaction | VersionedTransaction
    signers?: any[],
    options?: any
  ) => Promise<string>;
  simulate?: (
    tx: any,
    signers?: any[],
    options?: any
  ) => Promise<any>;
}
