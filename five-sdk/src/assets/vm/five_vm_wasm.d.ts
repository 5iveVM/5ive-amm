/* tslint:disable */
/* eslint-disable */

/**
 * Bytecode analyzer for WASM
 */
export class BytecodeAnalyzer {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Analyze bytecode and return instruction breakdown (legacy method for compatibility)
     */
    static analyze(bytecode: Uint8Array): any;
    /**
     * Get detailed opcode flow analysis - shows execution paths through the bytecode
     */
    static analyze_execution_flow(bytecode: Uint8Array): any;
    /**
     * Get detailed information about a specific instruction at an offset
     */
    static analyze_instruction_at(bytecode: Uint8Array, offset: number): any;
    /**
     * Advanced semantic analysis with full opcode understanding and instruction flow
     * Performs semantic analysis of bytecode to understand opcode behavior
     * and instruction flow.
     */
    static analyze_semantic(bytecode: Uint8Array): any;
    /**
     * Get summary statistics about the bytecode
     */
    static get_bytecode_summary(bytecode: Uint8Array): any;
}

/**
 * Bytecode Encoding utilities for JavaScript (Fixed Size)
 */
export class BytecodeEncoder {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Decode a u16 value
     * Returns [value, bytes_consumed] or null if invalid
     */
    static decode_u16(bytes: Uint8Array): Array<any> | undefined;
    /**
     * Decode a u32 value
     * Returns [value, bytes_consumed] or null if invalid
     */
    static decode_u32(bytes: Uint8Array): Array<any> | undefined;
    /**
     * Encode a u16 value
     * Returns [size, byte1, byte2]
     */
    static encode_u16(value: number): Array<any>;
    /**
     * Encode a u32 value
     * Returns [size, byte1, byte2, byte3, byte4]
     */
    static encode_u32(value: number): Array<any>;
    /**
     * Calculate encoded size (Always 2 for u16)
     */
    static encoded_size_u16(_value: number): number;
    /**
     * Calculate encoded size (Always 4 for u32)
     */
    static encoded_size_u32(_value: number): number;
}

/**
 * Execution result.
 */
export enum ExecutionStatus {
    /**
     * All operations completed successfully.
     */
    Completed = 0,
    /**
     * Execution stopped because it hit a system program call that cannot be executed in WASM.
     */
    StoppedAtSystemCall = 1,
    /**
     * Execution stopped because it hit an INIT_PDA operation that requires real Solana context.
     */
    StoppedAtInitPDA = 2,
    /**
     * Execution stopped because it hit an INVOKE operation that requires real RPC.
     */
    StoppedAtInvoke = 3,
    /**
     * Execution stopped because it hit an INVOKE_SIGNED operation that requires real RPC.
     */
    StoppedAtInvokeSigned = 4,
    /**
     * Execution stopped because compute limit was reached.
     */
    ComputeLimitExceeded = 5,
    /**
     * Execution failed due to an error.
     */
    Failed = 6,
}

/**
 * JavaScript-compatible VM state representation.
 */
export class FiveVMState {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    readonly compute_units: bigint;
    readonly instruction_pointer: number;
    readonly stack: Array<any>;
}

/**
 * Main WASM VM wrapper.
 */
export class FiveVMWasm {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Execute VM with input data and accounts (legacy method).
     */
    execute(input_data: Uint8Array, accounts: Array<any>): any;
    /**
     * Execute VM with partial execution support - stops at system calls.
     */
    execute_partial(input_data: Uint8Array, accounts: Array<any>): TestResult;
    /**
     * Get VM constants for JavaScript
     */
    static get_constants(): any;
    /**
     * Get current VM state
     */
    get_state(): any;
    /**
     * Create new VM instance with bytecode.
     */
    constructor(_bytecode: Uint8Array);
    /**
     * Validate bytecode without execution
     */
    static validate_bytecode(bytecode: Uint8Array): boolean;
}

/**
 * Parameter encoding utilities using fixed-size encoding and protocol types
 */
export class ParameterEncoder {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Encode function parameters using fixed size encoding
     * Returns ONLY parameter data - SDK handles discriminator AND function index
     */
    static encode_execute(_function_index: number, params: Array<any>): Uint8Array;
}

/**
 * Detailed execution result.
 */
export class TestResult {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Compute units consumed.
     */
    compute_units_used: bigint;
    /**
     * Final instruction pointer.
     */
    instruction_pointer: number;
    /**
     * Which opcode caused the stop (if stopped at system call).
     */
    get stopped_at_opcode(): number | undefined;
    /**
     * Which opcode caused the stop (if stopped at system call).
     */
    set stopped_at_opcode(value: number | null | undefined);
    readonly error_message: string | undefined;
    readonly execution_context: string | undefined;
    readonly final_accounts: Array<any>;
    readonly final_memory: Uint8Array;
    readonly final_stack: Array<any>;
    readonly get_result_value: any;
    readonly has_result_value: boolean;
    readonly status: string;
    readonly stopped_at_opcode_name: string | undefined;
}

/**
 * JavaScript-compatible account representation.
 */
export class WasmAccount {
    free(): void;
    [Symbol.dispose](): void;
    constructor(key: Uint8Array, data: Uint8Array, lamports: bigint, is_writable: boolean, is_signer: boolean, owner: Uint8Array);
    is_signer: boolean;
    is_writable: boolean;
    lamports: bigint;
    data: Uint8Array;
    readonly key: Uint8Array;
    readonly owner: Uint8Array;
}

/**
 * WASM source analysis result
 */
export class WasmAnalysisResult {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Get parsed metrics as JavaScript object
     */
    get_metrics_object(): any;
    /**
     * Analysis time in milliseconds
     */
    analysis_time: number;
    /**
     * Whether analysis succeeded
     */
    success: boolean;
    readonly errors: Array<any>;
    readonly metrics: string;
    readonly summary: string;
}

/**
 * Compilation options for enhanced error reporting and formatting
 */
export class WasmCompilationOptions {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Create development-debug configuration
     */
    static development_debug(): WasmCompilationOptions;
    /**
     * Create fast iteration configuration
     */
    static fast_iteration(): WasmCompilationOptions;
    /**
     * Create default compilation options
     */
    constructor();
    /**
     * Create production-optimized configuration
     */
    static production_optimized(): WasmCompilationOptions;
    /**
     * Set analysis depth level
     */
    with_analysis_depth(depth: string): WasmCompilationOptions;
    /**
     * Enable or disable complexity analysis
     */
    with_complexity_analysis(enabled: boolean): WasmCompilationOptions;
    /**
     * Enable or disable comprehensive metrics collection
     */
    with_comprehensive_metrics(enabled: boolean): WasmCompilationOptions;
    /**
     * Enable or disable bytecode compression
     */
    with_compression(enabled: boolean): WasmCompilationOptions;
    /**
     * Enable or disable constraint caching optimization
     */
    with_constraint_cache(enabled: boolean): WasmCompilationOptions;
    /**
     * Enable or disable debug information
     */
    with_debug_info(enabled: boolean): WasmCompilationOptions;
    /**
     * Enable or disable REQUIRE_BATCH lowering.
     */
    with_disable_require_batch(enabled: boolean): WasmCompilationOptions;
    /**
     * Enable or disable enhanced error reporting
     */
    with_enhanced_errors(enabled: boolean): WasmCompilationOptions;
    /**
     * Set error output format
     */
    with_error_format(format: string): WasmCompilationOptions;
    /**
     * Set export format
     */
    with_export_format(format: string): WasmCompilationOptions;
    /**
     * Enable or disable basic metrics collection
     */
    with_metrics(enabled: boolean): WasmCompilationOptions;
    /**
     * Set metrics export format
     */
    with_metrics_format(format: string): WasmCompilationOptions;
    /**
     * Set compilation mode
     */
    with_mode(mode: string): WasmCompilationOptions;
    /**
     * Enable or disable module namespace qualification
     */
    with_module_namespaces(enabled: boolean): WasmCompilationOptions;
    /**
     * Set optimization level (production)
     */
    with_optimization_level(level: string): WasmCompilationOptions;
    /**
     * Enable or disable performance analysis
     */
    with_performance_analysis(enabled: boolean): WasmCompilationOptions;
    /**
     * Enable or disable quiet mode
     */
    with_quiet(enabled: boolean): WasmCompilationOptions;
    /**
     * Set source file name for better error reporting
     */
    with_source_file(filename: string): WasmCompilationOptions;
    /**
     * Enable or disable compilation summary
     */
    with_summary(enabled: boolean): WasmCompilationOptions;
    /**
     * Enable or disable v2-preview features
     */
    with_v2_preview(enabled: boolean): WasmCompilationOptions;
    /**
     * Enable or disable verbose output
     */
    with_verbose(enabled: boolean): WasmCompilationOptions;
    /**
     * Include complexity analysis
     */
    complexity_analysis: boolean;
    /**
     * Include comprehensive metrics collection
     */
    comprehensive_metrics: boolean;
    /**
     * Enable bytecode compression
     */
    compress_output: boolean;
    /**
     * Disable REQUIRE_BATCH lowering in compiler pipeline.
     */
    disable_require_batch: boolean;
    /**
     * Enable constraint caching optimization
     */
    enable_constraint_cache: boolean;
    /**
     * Enable module namespace qualification (module::function)
     */
    enable_module_namespaces: boolean;
    /**
     * Enable enhanced error reporting with suggestions
     */
    enhanced_errors: boolean;
    /**
     * Include debug information
     */
    include_debug_info: boolean;
    /**
     * Include basic metrics
     */
    include_metrics: boolean;
    /**
     * Include performance analysis
     */
    performance_analysis: boolean;
    /**
     * Suppress non-essential output
     */
    quiet: boolean;
    /**
     * Show compilation summary
     */
    summary: boolean;
    /**
     * Enable v2-preview features (nibble immediates, BR_EQ_U8, etc.)
     */
    v2_preview: boolean;
    /**
     * Verbose output
     */
    verbose: boolean;
    readonly analysis_depth: string;
    readonly error_format: string;
    readonly export_format: string;
    readonly metrics_format: string;
    readonly mode: string;
    readonly optimization_level: string;
    readonly source_file: string | undefined;
}

/**
 * WASM compilation result - unified with enhanced error support
 */
export class WasmCompilationResult {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Get all errors as JSON array
     */
    format_all_json(): string;
    /**
     * Get all errors formatted as terminal output
     */
    format_all_terminal(): string;
    get_formatted_errors_json(): string;
    get_formatted_errors_terminal(): string;
    /**
     * Get fully detailed metrics regardless of export format
     */
    get_metrics_detailed(): any;
    /**
     * Get parsed metrics as JavaScript object
     */
    get_metrics_object(): any;
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
     * Whether compilation succeeded
     */
    success: boolean;
    /**
     * Total warning count
     */
    warning_count: number;
    readonly abi: any;
    readonly bytecode: Uint8Array | undefined;
    readonly compiler_errors: WasmCompilerError[];
    readonly disassembly: Array<any>;
    readonly errors: Array<any>;
    readonly metrics: string;
    readonly metrics_format: string;
    readonly warnings: Array<any>;
}

/**
 * WASM compilation result with comprehensive metrics
 */
export class WasmCompilationWithMetrics {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Get parsed metrics as JavaScript object
     */
    get_metrics_object(): any;
    /**
     * Size of generated bytecode
     */
    bytecode_size: number;
    /**
     * Compilation time in milliseconds
     */
    compilation_time: number;
    /**
     * Whether compilation succeeded
     */
    success: boolean;
    readonly bytecode: Uint8Array | undefined;
    readonly errors: Array<any>;
    readonly metrics: string;
    readonly warnings: Array<any>;
}

/**
 * Enhanced compiler error for WASM
 */
export class WasmCompilerError {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Get error as JSON string
     */
    format_json(): string;
    /**
     * Get formatted error message (terminal style)
     * Get formatted error message (terminal style)
     */
    format_terminal(): string;
    readonly category: string;
    readonly code: string;
    readonly column: any;
    readonly description: string | undefined;
    readonly line: any;
    readonly location: WasmSourceLocation | undefined;
    readonly message: string;
    readonly severity: string;
    readonly source_line: string | undefined;
    readonly suggestions: WasmSuggestion[];
}

/**
 * Enhanced compilation result with rich error information
 */
export class WasmEnhancedCompilationResult {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Get all errors as JSON array
     */
    format_all_json(): string;
    /**
     * Get all errors formatted as terminal output
     */
    format_all_terminal(): string;
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
     * Whether compilation succeeded
     */
    success: boolean;
    /**
     * Total warning count
     */
    warning_count: number;
    readonly compiler_errors: WasmCompilerError[];
}

/**
 * WASM DSL Compiler for client-side compilation
 */
export class WasmFiveCompiler {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Get detailed analysis of source code
     */
    analyze_source(source: string): WasmAnalysisResult;
    /**
     * Get detailed analysis of source code with compilation mode selection
     */
    analyze_source_mode(source: string, mode: string): WasmAnalysisResult;
    /**
     * Unified compilation method with enhanced error reporting and metrics
     */
    compile(source: string, options: WasmCompilationOptions): WasmCompilationResult;
    /**
     * Compile multi-file project with explicit module list
     */
    compileModules(module_files: any, entry_point: string, options: WasmCompilationOptions): WasmCompilationResult;
    /**
     * Compile multi-file project with automatic discovery
     */
    compileMultiWithDiscovery(entry_point: string, options: WasmCompilationOptions): WasmCompilationResult;
    /**
     * Multi-file compilation using module merger (main source + modules)
     */
    compile_multi(main_source: string, modules: any, options: WasmCompilationOptions): WasmCompilationResult;
    /**
     * Compile DSL and generate both bytecode and ABI
     */
    compile_with_abi(source: string): any;
    /**
     * Discover modules starting from an entry point
     */
    discoverModules(entry_point: string): any;
    /**
     * Extract function name metadata from compiled bytecode
     * Returns a list of discovered functions in the bytecode
     */
    extractFunctionMetadata(bytecode: Uint8Array): any;
    /**
     * Extract account definitions from DSL source code
     */
    extract_account_definitions(source: string): any;
    /**
     * Extract function signatures with account parameters
     */
    extract_function_signatures(source: string): any;
    /**
     * Format an error message using the native terminal formatter
     * This provides rich Rust-style error output with source context and colors
     */
    format_error_terminal(message: string, code: string, severity: string, line: number, column: number, _source: string): string;
    /**
     * Generate ABI from DSL source code for function calls
     */
    generate_abi(source: string): any;
    /**
     * Get compiler statistics
     */
    get_compiler_stats(): any;
    /**
     * Get comprehensive compiler statistics including which opcodes are used vs unused
     */
    get_opcode_analysis(source: string): any;
    /**
     * Get opcode usage statistics from compilation
     */
    get_opcode_usage(source: string): any;
    /**
     * Create a new WASM compiler instance
     */
    constructor();
    /**
     * Optimize bytecode
     */
    optimize_bytecode(bytecode: Uint8Array): Uint8Array;
    /**
     * Parse DSL source code and return AST information
     */
    parse_dsl(source: string): any;
    /**
     * Type-check parsed AST
     */
    type_check(_ast_json: string): any;
    /**
     * Validate account constraints against function parameters
     */
    validate_account_constraints(source: string, function_name: string, accounts_json: string): any;
    /**
     * Validate DSL syntax without full compilation
     */
    validate_syntax(source: string): any;
}

/**
 * WASM-exposed metrics collector wrapper
 */
export class WasmMetricsCollector {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * End the current compilation phase
     */
    end_phase(): void;
    /**
     * Export metrics in the requested format
     */
    export(format: string): string;
    /**
     * Finalize metrics collection
     */
    finalize(): void;
    /**
     * Get metrics as a JS object for programmatic use
     */
    get_metrics_object(): any;
    constructor();
    /**
     * Reset the collector for a new compilation
     */
    reset(): void;
    /**
     * Start timing a compilation phase
     */
    start_phase(phase_name: string): void;
}

/**
 * Enhanced source location for WASM
 */
export class WasmSourceLocation {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Column number (1-based)
     */
    column: number;
    /**
     * Length of the relevant text
     */
    length: number;
    /**
     * Line number (1-based)
     */
    line: number;
    /**
     * Byte offset in source
     */
    offset: number;
    readonly file: string | undefined;
}

/**
 * Enhanced error suggestion for WASM
 */
export class WasmSuggestion {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Confidence score (0.0 to 1.0)
     */
    confidence: number;
    readonly code_suggestion: string | undefined;
    readonly explanation: string | undefined;
    readonly message: string;
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
