/**
 * Five SDK - Client-Agnostic TypeScript/JavaScript library for Five VM
 * 
 * Minimal build for Frontend integration.
 */

// ==================== Core SDK Exports ====================

export * from './FiveSDK.js';
export * from './types.js';
// export * from './validation/index.js'; // Might rely on broken things
export * from './lib/bytecode-encoder.js';
export * from './encoding/ParameterEncoder.js'; // Added missing export
export * from './crypto/index.js'; // Added missing export (for PDAUtils)
export * from './project/toml.js';
export * from './project/config.js';
export * from './project/workspace.js';
export * from './wasm/vm.js';
export * from './wasm/compiler/index.js';
export * from './wasm/loader.js';
export * from './testing/index.js';

// ==================== FiveProgram High-Level API ====================

export * from './program/index.js';
export * from './modules/namespaces.js';

// ==================== Program ID Resolution ====================

export { ProgramIdResolver, FIVE_BAKED_PROGRAM_ID } from './config/ProgramIdResolver.js';

// ==================== Constants ====================

export { FIVE_VM_PROGRAM_ID } from './types.js';

// Default export disabled for minimal build
// export { FiveSDK as default };
