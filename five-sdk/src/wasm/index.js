/**
 * Five SDK WASM Integration Index
 *
 * Provides access to WASM VM and Compiler classes for the SDK.
 * Includes cross-platform path resolution and resource management.
 */
// Direct re-exports from CLI WASM modules
export { FiveVM } from '../../wasm/vm.js';
export { FiveCompilerWasm } from '../../wasm/compiler.js';
// varint encoder from lib
export { VarintEncoder } from '../../lib/varint-encoder.js';
// Simple WASM utilities (no over-engineering)
//# sourceMappingURL=index.js.map
