/**
 * Five SDK WASM Integration Index
 * 
 * Provides access to WASM VM and Compiler classes for the SDK.
 * Includes cross-platform path resolution and resource management.
 */

// Direct re-exports from CLI WASM modules
export { FiveVM } from './vm.js';
export { FiveCompiler } from './compiler.js';

// VLE encoder from lib
export { VLEEncoder } from '../lib/vle-encoder.js';

// Simple WASM utilities (no over-engineering)