/**
 * Five SDK WASM Integration Index
 * 
 * Provides access to WASM VM and Compiler classes for the SDK.
 * Includes cross-platform path resolution and resource management.
 */

// Direct re-exports from CLI WASM modules
export { FiveVM } from './vm.js';
export { FiveCompiler } from './compiler/index.js';

// Bytecode encoder from lib
export { BytecodeEncoder } from '../lib/bytecode-encoder.js';

// Simple WASM utilities (no over-engineering)