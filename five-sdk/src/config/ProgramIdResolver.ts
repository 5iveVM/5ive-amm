/**
 * Centralized program ID resolution for Five SDK.
 * Implements consistent precedence: explicit → default → env → baked → error
 */

import { validator } from '../validation/index.js';

/**
 * Baked program ID injected at release time (set by scripts/set-default-program-id.sh)
 * Empty string by default; overridden in npm published packages.
 */
export const FIVE_BAKED_PROGRAM_ID = '4Qxf3pbCse2veUgZVMiAm3nWqJrYo2pT4suxHKMJdK1d';

/**
 * Centralized resolver for program IDs across all SDK operations.
 * Ensures consistent validation and error messaging.
 */
export class ProgramIdResolver {
  private static defaultProgramId: string | undefined;

  /**
   * Set the default program ID for the entire SDK.
   * Used when no explicit program ID is provided.
   * @param programId - Solana public key (base58 encoded)
   * @throws {ValidationError} - If programId is not a valid Solana address
   */
  static setDefault(programId: string): void {
    validator.validateBase58Address(programId, 'defaultProgramId');
    this.defaultProgramId = programId;
  }

  /**
   * Get the currently set default program ID.
   * @returns The default program ID, or undefined if not set
   */
  static getDefault(): string | undefined {
    return this.defaultProgramId;
  }

  /**
   * Resolve program ID with consistent precedence.
   * Order: explicit → instance default → env (FIVE_PROGRAM_ID) → baked → error
   *
   * @param explicit - Explicit program ID (highest priority)
   * @param options - Resolution options
   * @returns Resolved program ID (validated)
   * @throws {Error} - If no valid program ID resolves and allowUndefined is false
   *
   * @example
   * ```typescript
   * // Will use explicit if provided, fall back through chain
   * const programId = ProgramIdResolver.resolve(options.fiveVMProgramId);
   *
   * // For local/WASM paths that don't need program ID
   * const optional = ProgramIdResolver.resolveOptional(options.fiveVMProgramId);
   * ```
   */
  static resolve(
    explicit?: string,
    options?: { allowUndefined?: boolean }
  ): string {
    // Precedence: explicit → default → env → baked
    // Note: Must check for explicit empty string differently from undefined
    let resolved: string | undefined;

    if (explicit) {
      resolved = explicit;
    } else if (this.defaultProgramId) {
      resolved = this.defaultProgramId;
    } else if (process.env.FIVE_PROGRAM_ID) {
      resolved = process.env.FIVE_PROGRAM_ID;
    } else if (FIVE_BAKED_PROGRAM_ID) {
      resolved = FIVE_BAKED_PROGRAM_ID;
    }

    if (!resolved && !options?.allowUndefined) {
      throw new Error(
        `No program ID resolved for Five VM. ` +
        `Set via one of: ` +
        `(1) explicit call parameter, ` +
        `(2) FiveSDK.setDefaultProgramId(), ` +
        `(3) FIVE_PROGRAM_ID environment variable, ` +
        `(4) released package default. ` +
        `For setup guidance, see: https://docs.five.build/cli/program-id-setup`
      );
    }

    if (resolved) {
      validator.validateBase58Address(resolved, 'programId');
    }
    return resolved || '';
  }

  /**
   * Resolve program ID but allow undefined return.
   * Used for local/WASM execution paths that don't require a program ID.
   *
   * @param explicit - Explicit program ID (highest priority)
   * @returns Resolved program ID (validated) or undefined if no resolution possible
   */
  static resolveOptional(explicit?: string): string | undefined {
    try {
      const result = this.resolve(explicit, { allowUndefined: true });
      return result || undefined;
    } catch {
      return undefined;
    }
  }

  /**
   * Clear the default program ID.
   * Useful for testing or resetting to clean state.
   */
  static clearDefault(): void {
    this.defaultProgramId = undefined;
  }
}
