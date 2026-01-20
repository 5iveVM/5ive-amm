/* tslint:disable */
/* eslint-disable */

export class BytecodeAnalyzer {
  private constructor();
  free(): void;
  [Symbol.dispose](): void;
  /**
   * Analyze bytecode and return instruction breakdown (legacy method for compatibility)
   */
  static analyze(bytecode: Uint8Array): any;
  /**
   * Advanced semantic analysis with full opcode understanding and instruction flow
   * This provides the intelligent analysis that understands what each opcode does
   * and what operands follow each instruction.
   */
  static analyze_semantic(bytecode: Uint8Array): any;
  /**
   * Get detailed information about a specific instruction at an offset
   */
  static analyze_instruction_at(bytecode: Uint8Array, offset: number): any;
  /**
   * Get summary statistics about the bytecode
   */
  static get_bytecode_summary(bytecode: Uint8Array): any;
  /**
   * Get detailed opcode flow analysis - shows execution paths through the bytecode
   */
  static analyze_execution_flow(bytecode: Uint8Array): any;
}

/**
 * Execution result that honestly reports what happened
 */
export enum ExecutionStatus {
  /**
   * All operations completed successfully
   */
  Completed = 0,
  /**
   * Execution stopped because it hit a system program call that cannot be executed in WASM
   */
  StoppedAtSystemCall = 1,
  /**
   * Execution stopped because it hit an INIT_PDA operation that requires real Solana context
   */
  StoppedAtInitPDA = 2,
  /**
   * Execution stopped because it hit an INVOKE operation that requires real RPC
   */
  StoppedAtInvoke = 3,
  /**
   * Execution stopped because it hit an INVOKE_SIGNED operation that requires real RPC
   */
  StoppedAtInvokeSigned = 4,
  /**
   * Execution stopped because compute limit was reached
   */
  ComputeLimitExceeded = 5,
  /**
   * Execution failed due to an error
   */
  Failed = 6,
}

export class FiveVMState {
  private constructor();
  free(): void;
  [Symbol.dispose](): void;
  readonly stack: Array<any>;
  readonly instruction_pointer: number;
  readonly compute_units: bigint;
}

export class FiveVMWasm {
  free(): void;
  [Symbol.dispose](): void;
  /**
   * Create new VM instance with bytecode
   */
  constructor(_bytecode: Uint8Array);
  /**
   * Execute VM with input data and accounts (legacy method)
   */
  execute(input_data: Uint8Array, accounts: Array<any>): any;
  /**
   * Execute VM with partial execution support - stops at system calls
   */
  execute_partial(input_data: Uint8Array, accounts: Array<any>): TestResult;
  /**
   * Get current VM state
   */
  get_state(): any;
  /**
   * Validate bytecode without execution
   */
  static validate_bytecode(bytecode: Uint8Array): boolean;
  /**
   * Get VM constants for JavaScript
   */
  static get_constants(): any;
}

export class ParameterEncoder {
  private constructor();
  free(): void;
  [Symbol.dispose](): void;
  /**
   * Encode function parameters using VLE compression
   * Returns ONLY parameter data - SDK handles discriminator AND function index
   * Each parameter value is VLE-encoded regardless of its declared type for maximum compression
   */
  static encode_execute_vle(_function_index: number, params: Array<any>): Uint8Array;
}

export class TestResult {
  private constructor();
  free(): void;
  [Symbol.dispose](): void;
  /**
   * Compute units consumed
   */
  compute_units_used: bigint;
  /**
   * Final instruction pointer
   */
  instruction_pointer: number;
  /**
   * Which opcode caused the stop (if stopped at system call)
   */
  get stopped_at_opcode(): number | undefined;
  /**
   * Which opcode caused the stop (if stopped at system call)
   */
  set stopped_at_opcode(value: number | null | undefined);
  readonly status: string;
  readonly has_result_value: boolean;
  readonly get_result_value: any;
  readonly final_stack: Array<any>;
  readonly final_memory: Uint8Array;
  readonly final_accounts: Array<any>;
  readonly error_message: string | undefined;
  readonly execution_context: string | undefined;
  readonly stopped_at_opcode_name: string | undefined;
}

export class VLEEncoder {
  private constructor();
  free(): void;
  [Symbol.dispose](): void;
  /**
   * Encode a u32 value using Variable-Length Encoding
   * Returns [size, byte1, byte2, byte3] where size is 1-3
   */
  static encode_u32(value: number): Array<any>;
  /**
   * Encode a u16 value using Variable-Length Encoding
   * Returns [size, byte1, byte2] where size is 1-2
   */
  static encode_u16(value: number): Array<any>;
  /**
   * Decode a u32 value from Variable-Length Encoding
   * Returns [value, bytes_consumed] or null if invalid
   */
  static decode_u32(bytes: Uint8Array): Array<any> | undefined;
  /**
   * Decode a u16 value from Variable-Length Encoding
   * Returns [value, bytes_consumed] or null if invalid
   */
  static decode_u16(bytes: Uint8Array): Array<any> | undefined;
  /**
   * Calculate encoded size without encoding
   */
  static encoded_size_u32(value: number): number;
  /**
   * Calculate encoded size for u16
   */
  static encoded_size_u16(value: number): number;
}

export class WasmAccount {
  free(): void;
  [Symbol.dispose](): void;
  constructor(key: Uint8Array, data: Uint8Array, lamports: bigint, is_writable: boolean, is_signer: boolean, owner: Uint8Array);
  lamports: bigint;
  is_writable: boolean;
  is_signer: boolean;
  readonly key: Uint8Array;
  data: Uint8Array;
  readonly owner: Uint8Array;
}

export class WasmAnalysisResult {
  private constructor();
  free(): void;
  [Symbol.dispose](): void;
  /**
   * Get parsed metrics as JavaScript object
   */
  get_metrics_object(): any;
  /**
   * Whether analysis succeeded
   */
  success: boolean;
  /**
   * Analysis time in milliseconds
   */
  analysis_time: number;
  readonly summary: string;
  readonly metrics: string;
  readonly errors: Array<any>;
}

export class WasmCompilationOptions {
  free(): void;
  [Symbol.dispose](): void;
  /**
   * Create default compilation options
   */
  constructor();
  /**
   * Set compilation mode
   */
  with_mode(mode: string): WasmCompilationOptions;
  /**
   * Set optimization level (production)
   */
  with_optimization_level(level: string): WasmCompilationOptions;
  /**
   * Enable or disable v2-preview features
   */
  with_v2_preview(enabled: boolean): WasmCompilationOptions;
  /**
   * Enable or disable constraint caching optimization
   */
  with_constraint_cache(enabled: boolean): WasmCompilationOptions;
  /**
   * Enable or disable enhanced error reporting
   */
  with_enhanced_errors(enabled: boolean): WasmCompilationOptions;
  /**
   * Set error output format
   */
  with_error_format(format: string): WasmCompilationOptions;
  /**
   * Set source file name for better error reporting
   */
  with_source_file(filename: string): WasmCompilationOptions;
  /**
   * Enable or disable basic metrics collection
   */
  with_metrics(enabled: boolean): WasmCompilationOptions;
  /**
   * Enable or disable comprehensive metrics collection
   */
  with_comprehensive_metrics(enabled: boolean): WasmCompilationOptions;
  /**
   * Set metrics export format
   */
  with_metrics_format(format: string): WasmCompilationOptions;
  /**
   * Enable or disable performance analysis
   */
  with_performance_analysis(enabled: boolean): WasmCompilationOptions;
  /**
   * Enable or disable complexity analysis
   */
  with_complexity_analysis(enabled: boolean): WasmCompilationOptions;
  /**
   * Enable or disable compilation summary
   */
  with_summary(enabled: boolean): WasmCompilationOptions;
  /**
   * Enable or disable verbose output
   */
  with_verbose(enabled: boolean): WasmCompilationOptions;
  /**
   * Enable or disable quiet mode
   */
  with_quiet(enabled: boolean): WasmCompilationOptions;
  /**
   * Set analysis depth level
   */
  with_analysis_depth(depth: string): WasmCompilationOptions;
  /**
   * Set export format
   */
  with_export_format(format: string): WasmCompilationOptions;
  /**
   * Enable or disable debug information
   */
  with_debug_info(enabled: boolean): WasmCompilationOptions;
  /**
   * Enable or disable bytecode compression
   */
  with_compression(enabled: boolean): WasmCompilationOptions;
  /**
   * Enable or disable module namespace qualification
   */
  with_module_namespaces(enabled: boolean): WasmCompilationOptions;
  /**
   * Create production-optimized configuration
   */
  static production_optimized(): WasmCompilationOptions;
  /**
   * Create development-debug configuration
   */
  static development_debug(): WasmCompilationOptions;
  /**
   * Create fast iteration configuration
   */
  static fast_iteration(): WasmCompilationOptions;
  /**
   * Enable v2-preview features (nibble immediates, BR_EQ_U8, etc.)
   */
  v2_preview: boolean;
  /**
   * Enable constraint caching optimization
   */
  enable_constraint_cache: boolean;
  /**
   * Enable enhanced error reporting with suggestions
   */
  enhanced_errors: boolean;
  /**
   * Include basic metrics
   */
  include_metrics: boolean;
  /**
   * Include comprehensive metrics collection
   */
  comprehensive_metrics: boolean;
  /**
   * Include performance analysis
   */
  performance_analysis: boolean;
  /**
   * Include complexity analysis
   */
  complexity_analysis: boolean;
  /**
   * Show compilation summary
   */
  summary: boolean;
  /**
   * Verbose output
   */
  verbose: boolean;
  /**
   * Suppress non-essential output
   */
  quiet: boolean;
  /**
   * Include debug information
   */
  include_debug_info: boolean;
  /**
   * Enable bytecode compression
   */
  compress_output: boolean;
  /**
   * Enable module namespace qualification (module::function)
   */
  enable_module_namespaces: boolean;
  readonly mode: string;
  readonly optimization_level: string;
  readonly error_format: string;
  readonly source_file: string | undefined;
  readonly metrics_format: string;
  readonly analysis_depth: string;
  readonly export_format: string;
}

export class WasmCompilationResult {
  private constructor();
  free(): void;
  [Symbol.dispose](): void;
  get_formatted_errors_terminal(): string;
  get_formatted_errors_json(): string;
  /**
   * Get all errors formatted as terminal output
   */
  format_all_terminal(): string;
  /**
   * Get all errors as JSON array
   */
  format_all_json(): string;
  /**
   * Get parsed metrics as JavaScript object
   */
  get_metrics_object(): any;
  /**
   * Get fully detailed metrics regardless of export format
   */
  get_metrics_detailed(): any;
  /**
   * Whether compilation succeeded
   */
  success: boolean;
  /**
   * Size of generated bytecode
   */
  bytecode_size: number;
  /**
   * Compilation time in milliseconds
   */
  compilation_time: number;
  /**
   * Total error count
   */
  error_count: number;
  /**
   * Total warning count
   */
  warning_count: number;
  readonly bytecode: Uint8Array | undefined;
  readonly abi: any;
  readonly warnings: Array<any>;
  readonly errors: Array<any>;
  readonly compiler_errors: WasmCompilerError[];
  readonly disassembly: Array<any>;
  readonly metrics: string;
  readonly metrics_format: string;
}

export class WasmCompilationWithMetrics {
  private constructor();
  free(): void;
  [Symbol.dispose](): void;
  /**
   * Get parsed metrics as JavaScript object
   */
  get_metrics_object(): any;
  /**
   * Whether compilation succeeded
   */
  success: boolean;
  /**
   * Size of generated bytecode
   */
  bytecode_size: number;
  /**
   * Compilation time in milliseconds
   */
  compilation_time: number;
  readonly bytecode: Uint8Array | undefined;
  readonly warnings: Array<any>;
  readonly errors: Array<any>;
  readonly metrics: string;
}

export class WasmCompilerError {
  private constructor();
  free(): void;
  [Symbol.dispose](): void;
  /**
   * Get formatted error message (terminal style)
   * Get formatted error message (terminal style)
   */
  format_terminal(): string;
  /**
   * Get error as JSON string
   */
  format_json(): string;
  readonly code: string;
  readonly line: any;
  readonly column: any;
  readonly severity: string;
  readonly category: string;
  readonly message: string;
  readonly description: string | undefined;
  readonly location: WasmSourceLocation | undefined;
  readonly suggestions: WasmSuggestion[];
  readonly source_line: string | undefined;
}

export class WasmEnhancedCompilationResult {
  private constructor();
  free(): void;
  [Symbol.dispose](): void;
  /**
   * Get all errors formatted as terminal output
   */
  format_all_terminal(): string;
  /**
   * Get all errors as JSON array
   */
  format_all_json(): string;
  /**
   * Whether compilation succeeded
   */
  success: boolean;
  /**
   * Size of generated bytecode
   */
  bytecode_size: number;
  /**
   * Compilation time in milliseconds
   */
  compilation_time: number;
  /**
   * Total error count
   */
  error_count: number;
  /**
   * Total warning count
   */
  warning_count: number;
  readonly compiler_errors: WasmCompilerError[];
}

export class WasmFiveCompiler {
  free(): void;
  [Symbol.dispose](): void;
  /**
   * Create a new WASM compiler instance
   */
  constructor();
  /**
   * Format an error message using the native terminal formatter
   * This provides rich Rust-style error output with source context and colors
   */
  format_error_terminal(message: string, code: string, severity: string, line: number, column: number, _source: string): string;
  /**
   * Unified compilation method with enhanced error reporting and metrics
   */
  compile(source: string, options: WasmCompilationOptions): WasmCompilationResult;
  /**
   * Compile multi-file project with automatic discovery
   */
  compileMultiWithDiscovery(entry_point: string, options: WasmCompilationOptions): WasmCompilationResult;
  /**
   * Discover modules starting from an entry point
   */
  discoverModules(entry_point: string): any;
  /**
   * Compile multi-file project with explicit module list
   */
  compileModules(module_files: any, entry_point: string, options: WasmCompilationOptions): WasmCompilationResult;
  /**
   * Extract function name metadata from compiled bytecode
   * Returns a list of discovered functions in the bytecode
   */
  extractFunctionMetadata(bytecode: Uint8Array): any;
  /**
   * Multi-file compilation using module merger (main source + modules)
   */
  compile_multi(main_source: string, modules: any, options: WasmCompilationOptions): WasmCompilationResult;
  /**
   * Get detailed analysis of source code
   */
  analyze_source(source: string): WasmAnalysisResult;
  /**
   * Get opcode usage statistics from compilation
   */
  get_opcode_usage(source: string): any;
  /**
   * Get comprehensive compiler statistics including which opcodes are used vs unused
   */
  get_opcode_analysis(source: string): any;
  /**
   * Get detailed analysis of source code with compilation mode selection
   */
  analyze_source_mode(source: string, mode: string): WasmAnalysisResult;
  /**
   * Parse DSL source code and return AST information
   */
  parse_dsl(source: string): any;
  /**
   * Type-check parsed AST
   */
  type_check(_ast_json: string): any;
  /**
   * Optimize bytecode
   */
  optimize_bytecode(bytecode: Uint8Array): Uint8Array;
  /**
   * Extract account definitions from DSL source code
   */
  extract_account_definitions(source: string): any;
  /**
   * Extract function signatures with account parameters
   */
  extract_function_signatures(source: string): any;
  /**
   * Validate account constraints against function parameters
   */
  validate_account_constraints(source: string, function_name: string, accounts_json: string): any;
  /**
   * Get compiler statistics
   */
  get_compiler_stats(): any;
  /**
   * Generate ABI from DSL source code for function calls
   */
  generate_abi(source: string): any;
  /**
   * Compile DSL and generate both bytecode and ABI
   */
  compile_with_abi(source: string): any;
  /**
   * Validate DSL syntax without full compilation
   */
  validate_syntax(source: string): any;
}

export class WasmMetricsCollector {
  free(): void;
  [Symbol.dispose](): void;
  constructor();
  /**
   * Start timing a compilation phase
   */
  start_phase(phase_name: string): void;
  /**
   * End the current compilation phase
   */
  end_phase(): void;
  /**
   * Finalize metrics collection
   */
  finalize(): void;
  /**
   * Reset the collector for a new compilation
   */
  reset(): void;
  /**
   * Export metrics in the requested format
   */
  export(format: string): string;
  /**
   * Get metrics as a JS object for programmatic use
   */
  get_metrics_object(): any;
}

export class WasmSourceLocation {
  private constructor();
  free(): void;
  [Symbol.dispose](): void;
  /**
   * Line number (1-based)
   */
  line: number;
  /**
   * Column number (1-based)
   */
  column: number;
  /**
   * Byte offset in source
   */
  offset: number;
  /**
   * Length of the relevant text
   */
  length: number;
  readonly file: string | undefined;
}

export class WasmSuggestion {
  private constructor();
  free(): void;
  [Symbol.dispose](): void;
  /**
   * Confidence score (0.0 to 1.0)
   */
  confidence: number;
  readonly message: string;
  readonly explanation: string | undefined;
  readonly code_suggestion: string | undefined;
}

/**
 * Get function names from bytecode as a JS value (array of objects)
 *
 * This function avoids constructing `FunctionNameInfo` JS instances and instead
 * marshals the parsed metadata directly into a serde-friendly structure and
 * returns a `JsValue` via `JsValue::from_serde`.
 */
export function get_function_names(bytecode: Uint8Array): any;

/**
 * Get the count of public functions from bytecode header
 */
export function get_public_function_count(bytecode: Uint8Array): number;

/**
 * Get information about the WASM compiler capabilities
 */
export function get_wasm_compiler_info(): any;

/**
 * Helper function to convert JS value to VM Value
 */
export function js_value_to_vm_value(js_val: any, value_type: number): any;

export function log_to_console(message: string): void;

/**
 * Parse function names from bytecode metadata
 *
 * Returns a JS value which is a JSON string encoding an array of objects:
 * [ { "name": "...", "function_index": N }, ... ]
 * We serialize via serde_json and return the JSON string as a `JsValue` to
 * avoid complex JS object construction in Rust/WASM glue.
 */
export function parse_function_names(bytecode: Uint8Array): any;

/**
 * Utility: Validate optimized headers and mirror bytecode back to JS callers
 */
export function wrap_with_script_header(bytecode: Uint8Array): Uint8Array;
