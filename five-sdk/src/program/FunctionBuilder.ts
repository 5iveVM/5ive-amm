/**
 * FunctionBuilder - Fluent API for building Five VM function calls
 *
 * Provides chainable methods for specifying function parameters:
 * - .accounts() - Map account names to their addresses
 * - .args() - Map data parameters to their values
 * - .instruction() - Generate serialized instruction data
 *
 * Usage:
 * ```typescript
 * const ix = await program
 *   .function('add_amount')
 *   .accounts({ counter: counter1, owner: user1.publicKey })
 *   .args({ amount: 10 })
 *   .instruction();
 * ```
 */

import type {
  FunctionDefinition,
  ParameterDefinition,
  ScriptABI,
} from '../metadata/index.js';
import type { SerializedInstruction } from '../types.js';
import type { FiveProgramOptions } from './FiveProgram.js';
import { AccountResolver } from './AccountResolver.js';
import type { Provider } from '../types.js';

/**
 * FunctionBuilder implements fluent API for function calls
 */
export class FunctionBuilder {
  private functionDef: FunctionDefinition;
  private scriptAccount: string;
  private abi: ScriptABI;
  private options: FiveProgramOptions;
  private accountsMap: Map<string, string> = new Map();
  private argsMap: Map<string, any> = new Map();
  private resolvedAccounts: Set<string> = new Set();

  constructor(
    functionDef: FunctionDefinition,
    scriptAccount: string,
    abi: ScriptABI,
    options: FiveProgramOptions
  ) {
    this.functionDef = functionDef;
    this.scriptAccount = scriptAccount;
    this.abi = abi;
    this.options = options;
  }

  /**
   * Specify accounts for this function call
   * Accepts either base58 strings or PublicKey objects
   *
   * @param accounts - Map of account names to addresses
   * @returns this for method chaining
   */
  accounts(
    accounts: Record<string, string | { toBase58(): string }>
  ): this {
    for (const [name, address] of Object.entries(accounts)) {
      const addressStr =
        typeof address === 'string' ? address : address.toBase58();
      this.accountsMap.set(name, addressStr);
      this.resolvedAccounts.add(addressStr);
    }

    if (this.options.debug) {
      console.log(
        `[FunctionBuilder] Set accounts: ${Array.from(this.accountsMap.keys()).join(', ')}`
      );
    }

    return this;
  }

  /**
   * Specify data parameters for this function call
   *
   * @param args - Map of parameter names to values
   * @returns this for method chaining
   */
  args(args: Record<string, any>): this {
    for (const [name, value] of Object.entries(args)) {
      this.argsMap.set(name, value);
    }

    if (this.options.debug) {
      console.log(
        `[FunctionBuilder] Set args: ${Array.from(this.argsMap.keys()).join(', ')}`
      );
    }

    return this;
  }

  /**
   * Build and return serialized instruction data
   * This is the main method that orchestrates parameter resolution and instruction generation
   *
   * @returns SerializedInstruction ready for transaction building
   */
  async instruction(): Promise<SerializedInstruction> {
    // Validate parameters later, after auto-injection


    // Resolve system accounts (auto-inject when needed)
    const resolver = new AccountResolver(this.options);
    const resolvedSystemAccounts = resolver.resolveSystemAccounts(
      this.functionDef,
      this.accountsMap
    );

    // Merge system accounts into our map
    const systemAccountsList: string[] = [];
    for (const [name, address] of Object.entries(resolvedSystemAccounts)) {
      this.accountsMap.set(name, address);
      this.resolvedAccounts.add(address);
      systemAccountsList.push(address);  // Track for inclusion in accounts list
    }

    // Resolve PDA accounts (auto-derive based on seeds) using a temporary FiveProgram for findAddress.
    const { FiveProgram } = await import('./FiveProgram.js');
    const programForUtil = new FiveProgram(this.scriptAccount, this.abi, this.options);

    const resolvedPdaAccounts = await resolver.resolvePdaAccounts(
      this.abi,
      this.accountsMap,
      programForUtil
    );

    // Merge PDA accounts
    for (const [name, address] of Object.entries(resolvedPdaAccounts)) {
      this.accountsMap.set(name, address);
      this.resolvedAccounts.add(address);
      systemAccountsList.push(address);
    }

    // Now validate that all required parameters are provided (including auto-injected ones)
    this.validateParameters();

    // Merge parameters in ABI order (accounts first, then data)
    const { mergedParams, accountPubkeys } = this.mergeParameters();

    // Append system accounts to the account list (they go at the end)
    const allAccountPubkeys = [...accountPubkeys, ...systemAccountsList];

    // Build account metadata from ABI attributes
    const accountMetadata = this.buildAccountMetadata(
      allAccountPubkeys,
      resolvedSystemAccounts
    );

    if (this.options.debug) {
      console.log(`[FunctionBuilder] Building instruction for function '${this.functionDef.name}'`);
      console.log(`[FunctionBuilder] Merged params:`, mergedParams);
      console.log(`[FunctionBuilder] Account metadata:`, accountMetadata);
    }

    // Call existing SDK method to generate instruction
    // This reuses the proven parameter encoding logic
    const instruction = await this.generateInstructionData(
      mergedParams,
      allAccountPubkeys,
      accountMetadata
    );

    if (this.options.debug) {
      console.log(
        `[FunctionBuilder] Generated instruction:`,
        instruction
      );
    }

    return instruction;
  }

  /**
   * Build a Solana Transaction containing this instruction
   */
  async transaction(options: { computeUnits?: number } = {}): Promise<any> {
    const ix = await this.instruction();

    // Import Solana web3 dynamically
    const { Transaction, TransactionInstruction, ComputeBudgetProgram, PublicKey } = await import('@solana/web3.js');

    const tx = new Transaction();

    // add compute budget if requested
    if (options.computeUnits) {
      tx.add(ComputeBudgetProgram.setComputeUnitLimit({ units: options.computeUnits }));
    }

    // construct instruction
    const solanaIx = new TransactionInstruction({
      programId: new PublicKey(ix.programId),
      keys: ix.keys.map(key => ({
        pubkey: new PublicKey(key.pubkey),
        isSigner: key.isSigner,
        isWritable: key.isWritable
      })),
      data: Buffer.from(ix.data, 'base64')
    });

    tx.add(solanaIx);
    return tx;
  }

  /**
   * Send transaction with this instruction (RPC)
   * requires a provider to be set in options
   */
  async rpc(options: {
    signers?: any[],
    skipPreflight?: boolean,
    computeUnits?: number
  } = {}): Promise<string> {
    const provider = this.options.provider;
    if (!provider || !provider.sendAndConfirm) {
      throw new Error("RPC method requires a Provider with sendAndConfirm support");
    }

    const tx = await this.transaction({ computeUnits: options.computeUnits });

    // Send
    const signers = options.signers || [];
    return await provider.sendAndConfirm(tx, signers, {
      skipPreflight: options.skipPreflight
    });
  }



  /**
   * Validate that all required parameters are provided
   * @throws Error if any required parameter is missing
   */
  private validateParameters(): void {
    for (const param of this.functionDef.parameters) {
      if (param.is_account) {
        if (!this.accountsMap.has(param.name)) {
          throw new Error(
            `Missing required account '${param.name}' for function '${this.functionDef.name}'`
          );
        }
      } else {
        if (this.argsMap.get(param.name) === undefined) {
          throw new Error(
            `Missing required argument '${param.name}' for function '${this.functionDef.name}'`
          );
        }
      }
    }
  }

  /**
   * Merge account and data parameters in ABI order
   * Returns both merged array and list of account pubkeys for instruction building
   *
   * @returns Object with mergedParams array and accountPubkeys list
   */
  private mergeParameters(): {
    mergedParams: (string | number | boolean | bigint)[];
    accountPubkeys: string[];
  } {
    const mergedParams: (string | number | boolean | bigint)[] = [];
    const accountPubkeys: string[] = [];

    for (const param of this.functionDef.parameters) {
      if (param.is_account) {
        // Account parameter - must be in accountsMap
        const pubkey = this.accountsMap.get(param.name);
        if (!pubkey) {
          throw new Error(`Missing account '${param.name}'`);
        }
        mergedParams.push(pubkey);
        accountPubkeys.push(pubkey);
      } else {
        // Data parameter - must be in argsMap
        const value = this.argsMap.get(param.name);
        if (value === undefined) {
          throw new Error(`Missing argument '${param.name}'`);
        }
        mergedParams.push(value);
      }
    }

    return { mergedParams, accountPubkeys };
  }

  /**
   * Build account metadata (isSigner, isWritable) from ABI attributes
   *
   * @param accountPubkeys - List of account pubkeys in order
   * @param systemAccounts - System accounts that were auto-injected
   * @returns Map of pubkey to metadata
   */
  private buildAccountMetadata(
    accountPubkeys: string[],
    systemAccounts: Record<string, string>
  ): Map<
    string,
    { isSigner: boolean; isWritable: boolean; isSystemAccount?: boolean }
  > {
    const metadata = new Map<
      string,
      { isSigner: boolean; isWritable: boolean; isSystemAccount?: boolean }
    >();

    // First pass: identify if there's an @init constraint and find the payer
    let hasInit = false;
    let payerPubkey: string | undefined;
    for (const param of this.functionDef.parameters) {
      if (param.is_account) {
        const attributes = param.attributes || [];
        if (attributes.includes('init')) {
          hasInit = true;
          if (this.options.debug) {
            console.log(`[FunctionBuilder] Detected @init constraint on parameter: ${param.name}`);
          }
          // Find the payer - typically the @signer in an @init context
          // Check if there's another account marked @signer for the payer
          for (const payerParam of this.functionDef.parameters) {
            if (
              payerParam.is_account &&
              payerParam !== param &&
              (payerParam.attributes || []).includes('signer')
            ) {
              payerPubkey = this.accountsMap.get(payerParam.name);
              if (this.options.debug) {
                console.log(`[FunctionBuilder] Found payer for @init: ${payerParam.name} = ${payerPubkey}`);
              }
              break;
            }
          }
          break;
        }
      }
    }

    // Build metadata for function parameters
    for (const param of this.functionDef.parameters) {
      if (param.is_account) {
        const pubkey = this.accountsMap.get(param.name);
        if (!pubkey) continue;

        const attributes = param.attributes || [];
        const isWritable =
          attributes.includes('mut') ||
          attributes.includes('init') ||
          // If this is the payer for @init, it must be writable
          (hasInit && pubkey === payerPubkey);

        const entry = {
          isSigner: attributes.includes('signer'),
          isWritable,
        };

        if (this.options.debug) {
          console.log(`[FunctionBuilder] Account metadata for ${param.name}: pubkey=${pubkey}, isSigner=${entry.isSigner}, isWritable=${entry.isWritable}, isPayer=${hasInit && pubkey === payerPubkey}`);
        }

        metadata.set(pubkey, entry);
      }
    }

    // Mark system accounts
    for (const [name, address] of Object.entries(systemAccounts)) {
      if (!metadata.has(address)) {
        metadata.set(address, {
          isSigner: false,
          isWritable: false,
          isSystemAccount: true,
        });
      }
    }

    return metadata;
  }

  /**
   * Generate instruction data using serialization
   * Integrates with FiveSDK.generateExecuteInstruction() for parameter encoding
   *
   * @param mergedParams - Parameters in ABI order
   * @param accountList - List of all account pubkeys (function params + system accounts)
   * @param accountMetadata - Account metadata (isSigner, isWritable)
   * @returns SerializedInstruction
   */
  private async generateInstructionData(
    mergedParams: (string | number | boolean | bigint)[],
    accountList: string[],
    accountMetadata: Map<
      string,
      { isSigner: boolean; isWritable: boolean; isSystemAccount?: boolean }
    >
  ): Promise<SerializedInstruction> {
    // Account list is already passed in

    // Dynamically import FiveSDK to avoid circular dependencies
    const { FiveSDK } = await import('../FiveSDK.js');

    // Call the SDK's generateExecuteInstruction method
    // This handles VLE encoding and parameter validation
    const executionResult = await FiveSDK.generateExecuteInstruction(
      this.scriptAccount,
      this.functionDef.index,  // Use function index directly
      mergedParams,            // All parameters in merged order
      accountList,             // Account pubkey list
      undefined,               // No connection needed - we have ABI
      {
        debug: this.options.debug,
        abi: this.abi,        // Pass ABI for parameter encoding
        fiveVMProgramId: this.options.fiveVMProgramId,
        vmStateAccount: this.options.vmStateAccount,
        adminAccount: this.options.feeReceiverAccount,
      }
    );

    // Map SDK's instruction format (with 'accounts') to SerializedInstruction format (with 'keys')
    const sdkInstruction = executionResult.instruction;

    // Convert accounts array to keys array with proper naming
    const keys = (sdkInstruction.accounts || []).map((acc: any) => {
      // Handle both string and PublicKey-like objects
      let pubkeyStr: string;
      if (typeof acc.pubkey === 'string') {
        pubkeyStr = acc.pubkey;
      } else if (acc.pubkey && typeof acc.pubkey.toBase58 === 'function') {
        pubkeyStr = acc.pubkey.toBase58();
      } else {
        pubkeyStr = String(acc.pubkey);
      }

      return {
        pubkey: pubkeyStr,
        isSigner: acc.isSigner,
        isWritable: acc.isWritable,
      };
    });

    // Return the properly formatted serialized instruction
    const instruction: SerializedInstruction = {
      programId: sdkInstruction.programId,
      keys,
      data: sdkInstruction.data,
    };

    return instruction;
  }

  /**
   * Get the function definition
   */
  getFunctionDef(): FunctionDefinition {
    return this.functionDef;
  }

  /**
   * Get accounts that have been set
   */
  getAccounts(): Record<string, string> {
    return Object.fromEntries(this.accountsMap);
  }

  /**
   * Get arguments that have been set
   */
  getArgs(): Record<string, any> {
    return Object.fromEntries(this.argsMap);
  }
}
