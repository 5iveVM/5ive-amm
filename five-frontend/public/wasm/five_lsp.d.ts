/* tslint:disable */
/* eslint-disable */

export class FiveLspWasm {
  free(): void;
  [Symbol.dispose](): void;
  /**
   * Create a new LSP instance
   *
   * This initializes the compiler bridge and prepares it for use.
   */
  constructor();
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
   * Clear all caches
   *
   * Useful after large changes or when memory needs to be freed.
   * This forces recompilation on the next analysis.
   */
  clear_caches(): void;
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
