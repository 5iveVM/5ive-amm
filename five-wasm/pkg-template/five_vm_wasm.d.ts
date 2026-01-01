/* tslint:disable */
/* eslint-disable */
/**
* @param {Uint8Array} bytecode
*/
export function validate_bytecode(bytecode: Uint8Array): boolean;
/**
* @returns {any}
*/
export function get_constants(): any;
/**
* @param {any} js_val
* @param {number} value_type
* @returns {any}
*/
export function js_value_to_vm_value(js_val: any, value_type: number): any;

/**
* WASM-compatible account representation for Stacks VM
*/
export class WasmAccount {
  free(): void;
/**
* @param {Uint8Array} key
* @param {Uint8Array} data
* @param {bigint} lamports
* @param {boolean} is_writable
* @param {boolean} is_signer
* @param {Uint8Array} owner
*/
  constructor(key: Uint8Array, data: Uint8Array, lamports: bigint, is_writable: boolean, is_signer: boolean, owner: Uint8Array);
/**
* @returns {Uint8Array}
*/
  readonly key: Uint8Array;
/**
* @returns {Uint8Array}
*/
  data: Uint8Array;
/**
* @returns {bigint}
*/
  lamports: bigint;
/**
* @returns {boolean}
*/
  is_writable: boolean;
/**
* @returns {boolean}
*/
  is_signer: boolean;
/**
* @returns {Uint8Array}
*/
  readonly owner: Uint8Array;
}

/**
* Bytecode analyzer for WASM environment
*/
export class BytecodeAnalyzer {
  free(): void;
/**
* Analyze bytecode and return instruction breakdown
* @param {Uint8Array} bytecode
* @returns {any}
*/
  static analyze(bytecode: Uint8Array): any;
}

/**
* Main WASM VM wrapper providing JavaScript interface to Stacks VM
*/
export class StacksVMWasm {
  free(): void;
/**
* Create new VM instance with bytecode
* @param {Uint8Array} bytecode
*/
  constructor(bytecode: Uint8Array);
/**
* Execute VM with input data and accounts
* @param {Uint8Array} input_data
* @param {any[]} accounts
* @returns {any}
*/
  execute(input_data: Uint8Array, accounts: any[]): any;
/**
* Get current VM state
* @returns {any}
*/
  get_state(): any;
/**
* Validate bytecode without execution
* @param {Uint8Array} bytecode
* @returns {boolean}
*/
  static validate_bytecode(bytecode: Uint8Array): boolean;
/**
* Get VM constants for JavaScript
* @returns {any}
*/
  static get_constants(): any;
}

/**
* JavaScript-compatible VM state representation
*/
export class StacksVMState {
  free(): void;
/**
* @returns {any[]}
*/
  readonly stack: any[];
/**
* @returns {number}
*/
  readonly instruction_pointer: number;
/**
* @returns {bigint}
*/
  readonly compute_units: bigint;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_stacksvmstate_free: (a: number) => void;
  readonly __wbg_get_stacksvmstate_instruction_pointer: (a: number) => number;
  readonly __wbg_get_stacksvmstate_compute_units: (a: number) => number;
  readonly stacksvmstate_stack: (a: number) => number;
  readonly __wbg_wasmaccount_free: (a: number) => void;
  readonly wasmaccount_new: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => number;
  readonly wasmaccount_key: (a: number) => number;
  readonly wasmaccount_data: (a: number) => number;
  readonly wasmaccount_set_data: (a: number, b: number, c: number) => void;
  readonly wasmaccount_owner: (a: number) => number;
  readonly __wbg_get_wasmaccount_lamports: (a: number) => number;
  readonly __wbg_set_wasmaccount_lamports: (a: number, b: number) => void;
  readonly __wbg_get_wasmaccount_is_writable: (a: number) => number;
  readonly __wbg_set_wasmaccount_is_writable: (a: number, b: number) => void;
  readonly __wbg_get_wasmaccount_is_signer: (a: number) => number;
  readonly __wbg_set_wasmaccount_is_signer: (a: number, b: number) => void;
  readonly __wbg_stacksvmwasm_free: (a: number) => void;
  readonly stacksvmwasm_new: (a: number, b: number) => number;
  readonly stacksvmwasm_execute: (a: number, b: number, c: number, d: number) => number;
  readonly stacksvmwasm_get_state: (a: number) => number;
  readonly stacksvmwasm_validate_bytecode: (a: number, b: number) => number;
  readonly stacksvmwasm_get_constants: () => number;
  readonly js_value_to_vm_value: (a: number, b: number) => number;
  readonly __wbg_bytecodeanalyzer_free: (a: number) => void;
  readonly bytecodeanalyzer_analyze: (a: number, b: number) => number;
  readonly main: () => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_exn_store: (a: number) => void;
}

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {InitInput | Promise<InitInput>} module_or_path
*
* @returns {Promise<InitOutput>}
*/
export default function init (module_or_path?: InitInput | Promise<InitInput>): Promise<InitOutput>;