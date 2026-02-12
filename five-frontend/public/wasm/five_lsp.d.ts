/* tslint:disable */
/* eslint-disable */

/**
 * WASM wrapper for the Five LSP compiler bridge
 *
 * This is the main entry point for WASM clients. It wraps the Rust
 * CompilerBridge and exposes it to JavaScript via wasm-bindgen.
 */
export class FiveLspWasm {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Clear all caches
     *
     * Useful after large changes or when memory needs to be freed.
     * This forces recompilation on the next analysis.
     */
    clear_caches(): void;
    /**
     * Find all references to a symbol at the given position
     *
     * # Arguments
     * * `uri` - File URI (e.g., "file:///test.v")
     * * `source` - The source code
     * * `line` - 0-indexed line number
     * * `character` - 0-indexed character position
     *
     * # Returns
     * A JSON string containing an array of Locations where the symbol is referenced
     *
     * # Example
     * ```typescript
     * const lsp = FiveLspWasm.new();
     * const result = lsp.find_references('file:///test.v', 'let x = 5; let y = x;', 0, 4);
     * const references = JSON.parse(result);  // Array of Location objects
     * ```
     */
    find_references(uri: string, source: string, line: number, character: number): string;
    /**
     * Get code actions for a diagnostic
     *
     * Provides quick fix suggestions for a diagnostic at the given position.
     */
    get_code_actions(uri: string, source: string, diagnostic_json: string): string;
    /**
     * Get completion suggestions at the given position
     *
     * # Arguments
     * * `uri` - File URI (e.g., "file:///test.v")
     * * `source` - The source code
     * * `line` - 0-indexed line number
     * * `character` - 0-indexed character position
     *
     * # Returns
     * A JSON string containing CompletionList with suggestions
     *
     * # Example
     * ```typescript
     * const lsp = FiveLspWasm.new();
     * const result = lsp.get_completions('file:///test.v', 'let x = ', 0, 8);
     * const completions = JSON.parse(result);
     * ```
     */
    get_completions(uri: string, source: string, line: number, character: number): string;
    /**
     * Get go-to-definition location for a symbol at the given position
     *
     * # Arguments
     * * `uri` - File URI (e.g., "file:///test.v")
     * * `source` - The source code
     * * `line` - 0-indexed line number
     * * `character` - 0-indexed character position
     *
     * # Returns
     * A JSON string containing Location if definition found, null otherwise
     *
     * # Example
     * ```typescript
     * const lsp = FiveLspWasm.new();
     * const result = lsp.get_definition('file:///test.v', 'function foo() {}', 0, 9);
     * const location = result ? JSON.parse(result) : null;
     * ```
     */
    get_definition(uri: string, source: string, line: number, character: number): string | undefined;
    /**
     * Get diagnostics for a Five DSL file
     *
     * # Arguments
     * * `uri` - File URI (e.g., "file:///test.v")
     * * `source` - The source code to analyze
     *
     * # Returns
     * A JSON string containing an array of diagnostics, or an error message
     *
     * # Example
     * ```typescript
     * const lsp = FiveLspWasm.new();
     * const result = lsp.get_diagnostics('file:///test.v', 'init { let x = 5; }');
     * const diagnostics = JSON.parse(result);
     * ```
     */
    get_diagnostics(uri: string, source: string): string;
    /**
     * Get document symbols for outline view
     *
     * Returns all top-level definitions (functions, variables, accounts) for
     * display in the editor's outline/navigator panel.
     */
    get_document_symbols(uri: string, source: string): string;
    /**
     * Get hover information for a symbol at the given position
     *
     * # Arguments
     * * `uri` - File URI (e.g., "file:///test.v")
     * * `source` - The source code
     * * `line` - 0-indexed line number
     * * `character` - 0-indexed character position
     *
     * # Returns
     * A JSON string containing hover information, or error message
     *
     * # Example
     * ```typescript
     * const lsp = FiveLspWasm.new();
     * const result = lsp.get_hover('file:///test.v', 'let x = 5;', 0, 4);
     * const hover = result ? JSON.parse(result) : null;
     * ```
     */
    get_hover(uri: string, source: string, line: number, character: number): string | undefined;
    /**
     * Get semantic tokens for syntax highlighting
     *
     * Returns an array of semantic tokens for AST-based syntax highlighting.
     * Provides more accurate highlighting than regex-based approaches.
     */
    get_semantic_tokens(uri: string, source: string): string;
    /**
     * Create a new LSP instance
     *
     * This initializes the compiler bridge and prepares it for use.
     */
    constructor();
    /**
     * Prepare a rename operation
     *
     * Validates that a symbol at the given position can be renamed and returns its name.
     */
    prepare_rename(source: string, line: number, character: number): string | undefined;
    /**
     * Rename a symbol across all occurrences
     *
     * Performs a safe rename of a symbol, updating all references to it.
     */
    rename(uri: string, source: string, line: number, character: number, new_name: string): string | undefined;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_fivelspwasm_free: (a: number, b: number) => void;
    readonly fivelspwasm_new: () => number;
    readonly fivelspwasm_get_diagnostics: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly fivelspwasm_get_hover: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
    readonly fivelspwasm_get_completions: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
    readonly fivelspwasm_get_definition: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
    readonly fivelspwasm_find_references: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
    readonly fivelspwasm_get_semantic_tokens: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly fivelspwasm_get_document_symbols: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly fivelspwasm_get_code_actions: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
    readonly fivelspwasm_prepare_rename: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly fivelspwasm_rename: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number) => void;
    readonly fivelspwasm_clear_caches: (a: number) => void;
    readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
    readonly __wbindgen_export: (a: number, b: number) => number;
    readonly __wbindgen_export2: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_export3: (a: number, b: number, c: number) => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
