/**
 * Type definitions for Five CLI
 * 
 * Comprehensive type system covering WASM integration, CLI operations,
 * compilation pipeline, and VM execution contexts.
 */

// CLI Configuration and Options
export interface CLIConfig {
  rootDir: string;
  verbose?: boolean;
  debug?: boolean;
  wasmDir?: string;
  tempDir?: string;
}

export interface CLIOptions {
  verbose?: boolean;
  debug?: boolean;
  output?: string;
  format?: 'binary' | 'hex' | 'text' | 'json';
  optimize?: boolean;
  target?: 'local' | 'devnet' | 'testnet' | 'mainnet';
  network?: string;
  keypair?: string;
  programId?: string;
  forceLocal?: boolean;
}

// WASM Module Types
export interface WasmModule {
  instance: WebAssembly.Instance;
  module: WebAssembly.Module;
  exports: Record<string, any>;
  memory: WebAssembly.Memory;
}

export interface WasmModuleConfig {
  moduleName: string;
  wasmPath: string;
  jsBindingsPath?: string;
  enableOptimizations?: boolean;
  memorySize?: number;
}

export interface WasmLoadOptions {
  streaming?: boolean;
  enableSIMD?: boolean;
  enableThreads?: boolean;
  importObject?: WebAssembly.Imports;
}

// Five Compiler Types
export interface CompilationOptions {
  sourceFile: string;
  outputFile?: string;
  generateABI?: boolean;
  abiOutputFile?: string;
  optimize?: boolean;
  target?: CompilationTarget;
  includeDebugInfo?: boolean;
  maxBytecodeSize?: number;
  enableVLE?: boolean;
  enableCompression?: boolean;
  includeMetrics?: boolean;
  metricsFormat?: "json" | "csv" | "toml";
  errorFormat?: "terminal" | "json" | "lsp";
  comprehensiveMetrics?: boolean;
  metricsOutput?: string;
}

export type CompilationTarget = 'vm' | 'solana' | 'debug' | 'test';

export interface CompilationResult {
  success: boolean;
  bytecode?: Uint8Array;
  abi?: any;
  metadata?: CompilationMetadata;
  errors?: CompilationError[];
  warnings?: CompilationWarning[];
  disassembly?: string[];
  metrics?: CompilationMetrics;
  metricsReport?: CompilationMetricsReport;
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
}

export interface CompilationError {
  type: 'syntax' | 'semantic' | 'type' | 'runtime';
  message: string;
  line?: number;
  column?: number;
  sourceLocation?: string;
  suggestion?: string;
}

export interface CompilationWarning {
  type: 'performance' | 'deprecation' | 'unused' | 'optimization';
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

// Five VM Types
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
}

export interface AccountInfo {
  pubkey?: string;
  key?: Uint8Array;
  lamports: number;
  data: Uint8Array;
  owner: Uint8Array | string;
  executable: boolean;
  rentEpoch: number;
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

// Bytecode Analysis Types
export interface BytecodeAnalysis {
  instructionCount: number;
  functionCount: number;
  jumpTargets: number[];
  callGraph: CallGraphNode[];
  complexity: ComplexityMetrics;
  optimizationOpportunities: OptimizationSuggestion[];
  securityIssues: SecurityIssue[];
}

export interface CallGraphNode {
  functionName: string;
  callsTo: string[];
  calledBy: string[];
  instructionCount: number;
  complexity: number;
}

export interface ComplexityMetrics {
  cyclomaticComplexity: number;
  nestingDepth: number;
  halsteadComplexity: number;
  maintainabilityIndex: number;
}

export interface OptimizationSuggestion {
  type: 'performance' | 'size' | 'readability';
  location: string;
  description: string;
  estimatedImprovement: string;
  priority: 'low' | 'medium' | 'high';
}

export interface SecurityIssue {
  type: 'vulnerability' | 'warning' | 'info';
  category: string;
  description: string;
  location: string;
  severity: 'low' | 'medium' | 'high' | 'critical';
  recommendation: string;
}

// Deployment Types
export interface DeploymentOptions {
  bytecode: Uint8Array;
  network: 'devnet' | 'testnet' | 'mainnet' | 'local';
  programId?: string;
  upgradeAuthority?: string;
  maxDataSize?: number;
  computeBudget?: number;
  // Five SDK specific options
  scriptAccount?: string;
  extraLamports?: number;
  fiveVMProgramId?: string;
  vmStateAccount?: string;
}

export interface DeploymentResult {
  success: boolean;
  programId?: string;
  transactionId?: string;
  deploymentCost?: number;
  error?: string;
  logs?: string[];
}

// Project Management Types
export interface ProjectConfig {
  name: string;
  version: string;
  description?: string;
  sourceDir: string;
  buildDir: string;
  target: CompilationTarget;
  entryPoint?: string;
  outputArtifactName?: string;
  cluster?: string;
  commitment?: string;
  rpcUrl?: string;
  programId?: string;
  keypairPath?: string;
  optimizations: ProjectOptimizations;
  dependencies: ProjectDependency[];
  multiFileMode?: boolean;
  modules?: Record<string, string[]>;
}

export interface ProjectOptimizations {
  enableVLE: boolean;
  enableCompression: boolean;
  enableRegisterAllocation: boolean;
  enableConstraintOptimization: boolean;
  optimizationLevel: 'production';
}

export interface ProjectDependency {
  name: string;
  version: string;
  type: 'five' | 'wasm' | 'solana';
  path?: string;
}

export interface BuildManifest {
  artifact_path: string;
  abi_path?: string;
  compiler_version?: string;
  source_files: string[];
  target: string;
  timestamp: string;
  hash?: string;
  format: 'five' | 'bin';
  entry_point?: string;
  source_dir?: string;
}

// Error Handling Types
export interface CLIError extends Error {
  code: string;
  exitCode: number;
  category: 'user' | 'system' | 'wasm' | 'network';
  details?: any;
}

export interface ErrorContext {
  command: string;
  arguments: string[];
  options: any;
  environment: {
    nodeVersion: string;
    platform: string;
    arch: string;
  };
}

// Utility Types
export type LogLevel = 'debug' | 'info' | 'warn' | 'error';

export interface Logger {
  debug(message: string, ...args: any[]): void;
  info(message: string, ...args: any[]): void;
  warn(message: string, ...args: any[]): void;
  error(message: string, ...args: any[]): void;
}

export interface ProgressOptions {
  total?: number;
  current?: number;
  message?: string;
  spinner?: boolean;
}

// Command Types
export interface CommandContext {
  config: CLIConfig;
  logger: Logger;
  wasmManager: any; // Will be defined in wasm module
  options: CLIOptions;
}

export interface CommandDefinition {
  name: string;
  description: string;
  aliases?: string[];
  options?: CommandOption[];
  arguments?: CommandArgument[];
  examples?: CommandExample[];
  handler: (args: any[], options: any, context: CommandContext) => Promise<void>;
}

export interface CommandOption {
  flags: string;
  description: string;
  defaultValue?: any;
  choices?: string[];
  required?: boolean;
}

export interface CommandArgument {
  name: string;
  description: string;
  required?: boolean;
  variadic?: boolean;
}

export interface CommandExample {
  command: string;
  description: string;
}
