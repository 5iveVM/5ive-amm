/**
 * AccountResolver - Automatic system account injection
 *
 * Intelligently adds system accounts based on function constraints:
 * - @init constraint → adds SystemProgram
 * - Other constraints may add Rent, Clock, etc. (future)
 *
 * This reduces boilerplate by eliminating the need for developers to
 * manually pass system accounts for every function call.
 */

import type { FunctionDefinition, ParameterDefinition, AccountConstraint, ScriptABI } from '../metadata/index.js';
import type { FiveProgramOptions } from './FiveProgram.js';
import type { FiveProgram } from './FiveProgram.js';

// Known system program IDs (Solana standard)
const SYSTEM_PROGRAM_ID = '11111111111111111111111111111111';

export interface ResolvedSystemAccounts {
  [accountName: string]: string;
}

/**
 * AccountResolver handles automatic injection of system accounts
 */
export class AccountResolver {
  private options: FiveProgramOptions;

  constructor(options: FiveProgramOptions) {
    this.options = options;
  }

  /**
   * Resolve system accounts that should be auto-injected
   * Detects @init constraints and adds SystemProgram if needed
   *
   * @param funcDef - Function definition from ABI
   * @param providedAccounts - Accounts already provided by user
   * @returns Map of system account names to their addresses
   */
  resolveSystemAccounts(
    funcDef: FunctionDefinition,
    providedAccounts: Map<string, string>
  ): ResolvedSystemAccounts {
    const systemAccounts: ResolvedSystemAccounts = {};

    // Check if function has @init constraint
    const hasInit = this.hasInitConstraint(funcDef);

    if (hasInit && !providedAccounts.has('systemProgram')) {
      systemAccounts['systemProgram'] = SYSTEM_PROGRAM_ID;

      if (this.options.debug) {
        console.log(
          `[AccountResolver] Auto-injecting SystemProgram for @init constraint`
        );
      }
    }

    return systemAccounts;
  }

  /**
   * Resolve PDA accounts based on ABI constraints
   *
   * @param abi - Script ABI containing account constraints
   * @param providedAccounts - Currently known accounts (user provided + system)
   * @param program - FiveProgram instance for derivation util
   */
  async resolvePdaAccounts(
    abi: ScriptABI,
    providedAccounts: Map<string, string>,
    program: FiveProgram
  ): Promise<ResolvedSystemAccounts> {
    const pdaAccounts: ResolvedSystemAccounts = {};

    if (!abi.accounts) return pdaAccounts;

    // Filter for PDA constraints
    const pdaConstraints = abi.accounts.filter(acc => acc.type === 'pda');

    for (const constraint of pdaConstraints) {
      if (providedAccounts.has(constraint.name)) {
        continue; // User already provided it
      }

      // Resolve seeds
      const seeds: (string | Uint8Array | Buffer)[] = [];
      let canResolve = true;

      for (const seed of (constraint.seeds || [])) {
        if (seed.startsWith('"') && seed.endsWith('"')) {
          // Static string seed: "my_seed"
          seeds.push(seed.slice(1, -1));
        } else {
          // Dynamic seed: account reference
          const refAddr = providedAccounts.get(seed);
          if (refAddr) {
            seeds.push(refAddr);
          } else {
            // Dependency missing, cannot resolve yet
            // Maybe it's a seed that depends on another PDA?
            // For now, simple single-pass resolution
            canResolve = false;
            break;
          }
        }
      }

      if (canResolve && seeds.length > 0) {
        try {
          const [pda] = await program.findAddress(seeds);
          pdaAccounts[constraint.name] = pda;

          if (this.options.debug) {
            console.log(`[AccountResolver] Auto-derived PDA '${constraint.name}': ${pda}`);
          }
        } catch (e) {
          console.warn(`[AccountResolver] Failed to derive PDA '${constraint.name}':`, e);
        }
      }
    }

    return pdaAccounts;
  }

  /**
   * Check if function has @init constraint on any parameter
   * @init means account creation via CPI
   *
   * @param funcDef - Function definition
   * @returns true if any parameter has @init attribute
   */
  private hasInitConstraint(funcDef: FunctionDefinition): boolean {
    return funcDef.parameters.some((param) => {
      const attributes = param.attributes || [];
      return attributes.includes('init');
    });
  }

  /**
   * Get account metadata from parameter attributes
   * Maps Five DSL attributes to Solana account properties
   *
   * @param param - Parameter definition
   * @returns Object with isSigner and isWritable flags
   */
  getAccountMetadata(param: ParameterDefinition): {
    isSigner: boolean;
    isWritable: boolean;
  } {
    const attributes = param.attributes || [];

    return {
      isSigner: attributes.includes('signer'),
      isWritable: attributes.includes('mut') || attributes.includes('init'),
    };
  }

  /**
   * Validate that all required accounts are provided after resolution
   *
   * @param funcDef - Function definition
   * @param allAccounts - All accounts (user-provided + system-injected)
   * @throws Error if required account is missing
   */
  validateResolvedAccounts(
    funcDef: FunctionDefinition,
    allAccounts: Map<string, string>
  ): void {
    for (const param of funcDef.parameters) {
      if (param.is_account && !allAccounts.has(param.name)) {
        throw new Error(
          `Required account '${param.name}' not provided and not auto-injected`
        );
      }
    }
  }
}
