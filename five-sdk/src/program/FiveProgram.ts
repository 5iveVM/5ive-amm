/**
 * FiveProgram - High-level wrapper for Five VM scripts
 *
 * Provides Anchor-style ergonomics while maintaining zero-dependency design.
 * FiveProgram simplifies interaction with Five scripts on-chain by:
 * - Providing fluent API for function calls
 * - Auto-injecting system accounts when needed
 * - Generating TypeScript types from ABI
 * - Handling parameter encoding internally
 *
 * Usage:
 * ```typescript
 * const program = FiveProgram.fromABI(scriptAccount, abi);
 * const ix = await program
 *   .function('increment')
 *   .accounts({ counter: counter1, owner: user1.publicKey })
 *   .instruction();
 * ```
 */

import type {
  ScriptABI,
  FunctionDefinition,
  ParameterDefinition,
  AccountFetcher,
  AccountData,
  ScriptMetadataParser,
} from '../metadata/index.js';
import type { Provider } from '../types.js';
import { FunctionBuilder } from './FunctionBuilder.js';
import { TypeGenerator } from './TypeGenerator.js';
import { ProgramAccount } from './ProgramAccount.js';
import { PublicKey } from '@solana/web3.js';
import { ProgramIdResolver } from '../config/ProgramIdResolver.js';

export interface FiveProgramOptions {
  /** Enable debug logging */
  debug?: boolean;
  /** Optional account fetcher for loading metadata from chain */
  fetcher?: AccountFetcher;
  /** Five VM Program ID (defaults to mainnet/devnet) */
  fiveVMProgramId?: string;
  /** Five VM State PDA account (if not provided, will be derived) */
  vmStateAccount?: string;
  /** Fee receiver account (admin account for transaction fees) */
  feeReceiverAccount?: string;
  /** Wallet/Network Provider for RPC calls */
  provider?: Provider;
}

/**
 * FiveProgram represents a deployed Five script with its ABI
 * Provides high-level API for building function calls
 */
export class FiveProgram {
  private scriptAccount: string;
  private abi: ScriptABI;
  private options: FiveProgramOptions;
  private functionBuilderCache: Map<string, FunctionDefinition> = new Map();

  /**
   * RPC Entry point for calling functions
   * e.g. program.methods.myFunc(params).rpc()
   */
  readonly methods: any;

  constructor(
    scriptAccount: string,
    abi: ScriptABI,
    options?: FiveProgramOptions
  ) {
    this.scriptAccount = scriptAccount;
    this.abi = abi;
    this.options = {
      debug: false,
      ...options,
      // fiveVMProgramId will be resolved at call time via ProgramIdResolver
    };

    // Build cache for quick function lookup
    this.abi.functions.forEach((func) => {
      this.functionBuilderCache.set(func.name, func);
    });

    if (this.options.debug) {
      console.log(
        `[FiveProgram] Initialized with ${this.abi.functions.length} functions`
      );
    }

    // Initialize the methods proxy
    this.methods = new Proxy(this, {
      get: (target, prop) => {
        if (typeof prop === 'string') {
          return (...args: any[]) => {
            const builder = target.function(prop);
            if (args.length === 1 && typeof args[0] === 'object' && !Array.isArray(args[0])) {
              builder.args(args[0]);
            } else if (args.length > 0) {
              throw new Error(
                `FiveProgram.methods.${prop} only supports a single named-arguments object. ` +
                `Use program.function("${prop}").args({...}) for now.`
              );
            }
            return builder;
          }
        }
        return undefined;
      }
    });
  }

  /**
   * Factory method: Create FiveProgram from ABI
   * Used when ABI is already available locally or compiled
   */
  static fromABI(
    scriptAccount: string,
    abi: ScriptABI,
    options?: FiveProgramOptions
  ): FiveProgram {
    return new FiveProgram(scriptAccount, abi, options);
  }

  /**
   * Factory method: Load FiveProgram from deployed script account
   * Fetches script metadata from chain and extracts ABI
   *
   * @param scriptAddress - Script account address
   * @param connection - web3.js Connection object or custom fetcher
   * @param options - SDK options
   * @returns Initialized FiveProgram instance
   */
  static async load(
    scriptAddress: string,
    connection: any, // Connection or custom fetcher
    options: FiveProgramOptions = {}
  ): Promise<FiveProgram> {
    const { ScriptMetadataParser } = await import('../metadata/index.js');
    const { PublicKey } = await import('@solana/web3.js');

    let fetcher: AccountFetcher;

    // Check if connection is a web3.js Connection (has getAccountInfo)
    if (connection && typeof connection.getAccountInfo === 'function') {
      fetcher = {
        getAccountData: async (address: string) => {
          try {
            const pubkey = new PublicKey(address);
            const info = await connection.getAccountInfo(pubkey);
            if (!info) return null;
            return {
              address,
              data: info.data,
              owner: info.owner.toBase58(),
              lamports: info.lamports
            } as AccountData;
          } catch (error) {
            console.warn(`Error fetching account ${address}:`, error);
            return null;
          }
        },
        getMultipleAccountsData: async (addresses: string[]) => {
          // Minimal implementation for getScriptMetadata requirements
          const results = new Map<string, AccountData | null>();
          // getScriptMetadata currently uses getAccountData, but if it used batch:
          try {
            const pubkeys = addresses.map(a => new PublicKey(a));
            const infos = await connection.getMultipleAccountsInfo(pubkeys);
            infos.forEach((info: any, i: number) => {
              const address = addresses[i];
              if (info) {
                results.set(address, {
                  address,
                  data: info.data,
                  owner: info.owner.toBase58(),
                  lamports: info.lamports
                });
              } else {
                results.set(address, null);
              }
            });
          } catch (e) {
            console.warn('Batch fetch failed', e);
          }
          return results;
        }
      };
    } else {
      // Assume it matches AccountFetcher interface
      fetcher = connection as AccountFetcher;
    }

    // Set fetcher in options so ProgramAccount can use it later
    const newOptions = { ...options, fetcher: options.fetcher || fetcher };

    const metadata = await ScriptMetadataParser.getScriptMetadata(fetcher, scriptAddress);
    return new FiveProgram(scriptAddress, metadata.abi, newOptions);
  }

  /**
   * Get function builder for a specific function
   * Creates a fluent interface for building function calls
   *
   * @param functionName - Name of the function to call
   * @returns FunctionBuilder for chaining .accounts().args().instruction()
   * @throws Error if function not found in ABI
   */
  function(functionName: string): FunctionBuilder {
    const funcDef = this.functionBuilderCache.get(functionName);
    if (!funcDef) {
      const available = Array.from(this.functionBuilderCache.keys());
      throw new Error(
        `Function '${functionName}' not found in ABI. Available: ${available.join(', ')}`
      );
    }

    return new FunctionBuilder(
      funcDef,
      this.scriptAccount,
      this.abi,
      this.options
    );
  }

  /**
   * Get an account handler for fetching and decoding state
   *
   * @param structName - Name of the struct definition in ABI (e.g., "Counter")
   * @returns ProgramAccount instance
   */
  account(structName: string): ProgramAccount {
    return new ProgramAccount(
      structName,
      this.abi,
      this.options.fetcher
    );
  }

  /**
   * Get all available function names from ABI
   */
  getFunctions(): string[] {
    return this.abi.functions.map((func) => func.name);
  }

  /**
   * Get function definition by name
   */
  getFunction(name: string): FunctionDefinition | undefined {
    return this.functionBuilderCache.get(name);
  }

  /**
   * Get all function definitions
   */
  getAllFunctions(): FunctionDefinition[] {
    return this.abi.functions;
  }

  /**
   * Generate TypeScript types from ABI
   * Creates type-safe interfaces for function parameters
   *
   * @returns TypeScript interface as string
   */
  generateTypes(): string {
    const typeGenerator = new TypeGenerator(this.abi, {
      scriptName: this.abi.name || 'Script',
      debug: this.options.debug,
    });
    return typeGenerator.generate();
  }

  /**
   * Get script account address
   */
  getScriptAccount(): string {
    return this.scriptAccount;
  }

  /**
   * Get Five VM Program ID with consistent resolver precedence
   */
  getFiveVMProgramId(): string {
    return ProgramIdResolver.resolve(this.options.fiveVMProgramId);
  }

  /**
   * Derive a Program Derived Address (PDA)
   *
   * Runtime-compatible behavior: the script account is always prepended as the
   * first seed so derived addresses match VM script-scoped PDA domains.
   *
   * @param seeds - User seeds (strings, public keys in base58, or buffers)
   * @param programId - Optional program ID (defaults to Five VM)
   * @returns [address, bump]
   */
  async findAddress(
    seeds: (string | Uint8Array | Buffer)[],
    programId?: string
  ): Promise<[string, number]> {
    const { PublicKey } = await import('@solana/web3.js');

    // Convert seeds to Buffers
    const bufferSeeds = seeds.map(seed => {
      if (seed instanceof Uint8Array || (typeof Buffer !== 'undefined' && Buffer.isBuffer(seed))) {
        return seed;
      }
      if (typeof seed === 'string') {
        // Check if it looks like a Pubkey (Base58, 32 bytes approx)
        // Heuristic: if it decodes to 32 bytes, treat as pubkey, else string literal
        try {
          const pk = new PublicKey(seed);
          return pk.toBuffer();
        } catch {
          return Buffer.from(seed);
        }
      }
      throw new Error(`Unsupported seed type: ${typeof seed}`);
    });

    const pid = new PublicKey(programId || this.getFiveVMProgramId());
    const scriptPk: any = new PublicKey(this.scriptAccount);
    const scriptSeed: Uint8Array =
      typeof scriptPk.toBuffer === 'function'
        ? scriptPk.toBuffer()
        : typeof scriptPk.toBytes === 'function'
          ? scriptPk.toBytes()
          : (() => { throw new Error('PublicKey implementation missing toBuffer/toBytes'); })();

    // @ts-ignore - PublicKey.findProgramAddress return type mismatch in some versions
    const [addr, bump] = PublicKey.findProgramAddressSync([scriptSeed, ...bufferSeeds], pid);

    return [addr.toBase58(), bump];
  }

  /**
   * Get the ABI
   */
  getABI(): ScriptABI {
    return this.abi;
  }

  /**
   * Get options
   */
  getOptions(): FiveProgramOptions {
    return this.options;
  }

  /**
   * Get VM State Account
   */
  getVMStateAccount(): string | undefined {
    return this.options.vmStateAccount;
  }

  /**
   * Set VM State Account
   */
  setVMStateAccount(account: string): this {
    this.options.vmStateAccount = account;
    return this;
  }

  /**
   * Get Fee Receiver Account
   */
  getFeeReceiverAccount(): string | undefined {
    return this.options.feeReceiverAccount;
  }

  /**
   * Set Fee Receiver Account
   */
  setFeeReceiverAccount(account: string): this {
    this.options.feeReceiverAccount = account;
    return this;
  }
}
