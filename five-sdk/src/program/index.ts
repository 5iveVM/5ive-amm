/**
 * Five SDK Program Module
 *
 * High-level wrapper for Five VM scripts with Anchor-style ergonomics.
 * Provides simple, type-safe API for building function calls.
 *
 * Usage:
 * ```typescript
 * import { FiveProgram } from '@five-vm/sdk';
 *
 * const program = FiveProgram.fromABI(scriptAccount, abi);
 * const ix = await program
 *   .function('increment')
 *   .accounts({ counter: counter1, owner: user1 })
 *   .instruction();
 * ```
 */

export { FiveProgram } from './FiveProgram.js';
export type { FiveProgramOptions } from './FiveProgram.js';

export { FunctionBuilder } from './FunctionBuilder.js';

export { AccountResolver } from './AccountResolver.js';
export type { ResolvedSystemAccounts } from './AccountResolver.js';

export { TypeGenerator } from './TypeGenerator.js';
export type { TypeGeneratorOptions } from './TypeGenerator.js';
export * from './FiveProgram.js';
export * from './FunctionBuilder.js';
export * from './ProgramAccount.js';
export * from './TypeGenerator.js';
export * from './AccountResolver.js';
export * from './SessionManager.js';
