/**
 * Five SDK Account System
 * 
 * Client-agnostic account management system with validation, PDA derivation,
 * and account size calculations. Uses serialization instead of direct blockchain calls.
 */

import { PDAUtils, SolanaPublicKeyUtils, RentCalculator, AccountValidator } from '../crypto/index.js';
import { ProgramIdResolver } from '../config/ProgramIdResolver.js';

/**
 * Account data interface (replaces Web3.js AccountInfo)
 */
export interface AccountData {
  /** Account address */
  address: string;
  /** Account data */
  data: Uint8Array;
  /** Account owner program ID */
  owner: string;
  /** Account balance in lamports */
  lamports: number;
}

/**
 * Account types in the Five VM ecosystem
 */
export type AccountType = 
  | 'script'           // Script account containing bytecode
  | 'metadata'         // Script metadata account  
  | 'user_state'       // User state account for script
  | 'system'           // System program
  | 'rent_sysvar'      // Rent sysvar
  | 'clock_sysvar'     // Clock sysvar
  | 'spl_token'        // SPL Token account
  | 'custom';          // Custom account type

/**
 * AccountType enum for test compatibility
 */
export const AccountType = {
  SCRIPT: 'script' as const,
  METADATA: 'metadata' as const,
  USER_STATE: 'user_state' as const,
  SYSTEM: 'system' as const,
  RENT_SYSVAR: 'rent_sysvar' as const,
  CLOCK_SYSVAR: 'clock_sysvar' as const,
  SPL_TOKEN: 'spl_token' as const,
  CUSTOM: 'custom' as const
} as const;

/**
 * Account information with Five VM context
 */
export interface FiveAccount {
  /** Account address */
  address: string;
  /** Account type */
  type: AccountType;
  /** Whether account is signer */
  isSigner: boolean;
  /** Whether account is writable */
  isWritable: boolean;
  /** Whether account is required for execution */
  required?: boolean;
  /** Account owner program ID */
  owner?: string;
  /** Account data size */
  size?: number;
  /** Account lamports balance */
  lamports?: number;
  /** Account data */
  data?: Uint8Array;
  /** PDA derivation info (if applicable) */
  pda?: {
    seeds: Uint8Array[];
    bump: number;
  };
}

/**
 * Account constraints for script execution
 */
export interface AccountConstraints {
  /** Maximum number of accounts allowed */
  maxAccounts?: number;
  /** Maximum total size across all accounts */
  maxTotalSize?: number;
  /** Maximum rent cost allowed */
  maxRentCost?: number;
  /** Required account types that must be present */
  requiredTypes?: AccountType[];
  /** Required signers */
  signers?: string[];
  /** Required writable accounts */
  writableAccounts?: string[];
  /** Required readonly accounts */
  readonlyAccounts?: string[];
  /** Account type constraints */
  typeConstraints?: Map<string, AccountType>;
  /** Minimum rent exemption requirements */
  rentRequirements?: Map<string, number>;
}

/**
 * Account validation result
 */
export interface AccountValidationResult {
  /** Whether validation passed */
  valid: boolean;
  /** Validation errors */
  errors: string[];
  /** Validation warnings */
  warnings: string[];
  /** Estimated costs */
  costs?: {
    rentExemption: number;
    transactionFee: number;
    totalCost: number;
  };
}

/**
 * Account creation parameters
 */
export interface CreateAccountParams {
  /** Account size in bytes */
  size: number;
  /** Owner program ID */
  owner: string;
  /** Whether to make rent-exempt */
  rentExempt: boolean;
  /** Additional lamports beyond rent exemption */
  additionalLamports?: number;
}

/**
 * Solana transaction instruction interface
 */
export interface TransactionInstruction {
  /** Program ID to invoke */
  programId: string;
  /** Account keys and metadata */
  accounts: Array<{
    pubkey: string;
    isSigner: boolean;
    isWritable: boolean;
  }>;
  /** Instruction data */
  data: Uint8Array;
}

/**
 * Account manager for Five VM scripts (serialization-based)
 */
export class FiveAccountManager {
  private programId: string;

  constructor(programId?: string) {
    this.programId = ProgramIdResolver.resolve(programId);
  }

  /**
   * Encode System Program CreateAccount instruction
   */
  private encodeCreateAccountInstruction(params: CreateAccountParams): Uint8Array {
    // Encoding for CreateAccount instruction
    // In a real implementation, this would use proper Solana instruction encoding
    const buffer = new ArrayBuffer(32);
    const view = new DataView(buffer);
    
    // Instruction discriminator for CreateAccount (0)
    view.setUint32(0, 0, true);
    // Account size
    view.setUint32(4, params.size, true);
    // Rent lamports (calculated)
    const rentLamports = params.rentExempt ? RentCalculator.calculateRentExemption(params.size) : 0;
    view.setBigUint64(8, BigInt(rentLamports), true);
    
    // Owner program ID would be encoded here in real implementation
    // Return the basic instruction data
    return new Uint8Array(buffer);
  }

  /**
   * Create script account PDA and return serialized instruction
   */
  async createScriptAccount(bytecode: Uint8Array, payerAddress: string): Promise<{
    address: string;
    bump: number;
    createInstruction: TransactionInstruction;
    rentLamports: number;
  }> {
    const pda = await PDAUtils.deriveScriptAccount(bytecode, this.programId);
    const rentLamports = RentCalculator.getScriptAccountRent(bytecode.length);

    // Create serialized instruction for System Program CreateAccount
    const createInstruction: TransactionInstruction = {
      programId: '11111111111111111111111111111112', // System Program
      accounts: [
        { pubkey: payerAddress, isSigner: true, isWritable: true },
        { pubkey: pda.address, isSigner: false, isWritable: true }
      ],
      data: this.encodeCreateAccountInstruction({
          size: bytecode.length + 256, // Bytecode + metadata
          owner: this.programId,
          rentExempt: true
        })
    };

    return {
      address: pda.address,
      bump: pda.bump,
      createInstruction,
      rentLamports
    };
  }

  /**
   * Create metadata account for script
   */
  async createMetadataAccount(scriptAccount: string, payerAddress: string): Promise<{
    address: string;
    bump: number;
    createInstruction: TransactionInstruction;
    rentLamports: number;
  }> {
    const pda = await PDAUtils.deriveMetadataAccount(scriptAccount, this.programId);
    const rentLamports = RentCalculator.getMetadataAccountRent();

    const createInstruction: TransactionInstruction = {
      programId: '11111111111111111111111111111112', // System Program
      accounts: [
        { pubkey: payerAddress, isSigner: true, isWritable: true },
        { pubkey: pda.address, isSigner: false, isWritable: true }
      ],
      data: this.encodeCreateAccountInstruction({
        size: 1024, // 1KB for metadata
        owner: this.programId,
        rentExempt: true
      })
    };

    return {
      address: pda.address,
      bump: pda.bump,
      createInstruction,
      rentLamports
    };
  }

  /**
   * Create user state account for script interaction
   */
  async createUserStateAccount(
    userPublicKey: string,
    scriptAccount: string
  ): Promise<{
    address: string;
    bump: number;
    createInstruction: any;
    rentLamports: number;
  }> {
    const pda = await PDAUtils.deriveUserStateAccount(
      userPublicKey,
      scriptAccount,
      this.programId
    );
    const rentLamports = RentCalculator.getUserStateAccountRent();

    return {
      address: pda.address,
      bump: pda.bump,
      createInstruction: {
        programId: '11111111111111111111111111111112', // System Program
        accounts: [
          { pubkey: pda.address, isSigner: false, isWritable: true },
          { pubkey: userPublicKey, isSigner: true, isWritable: true },
          { pubkey: this.programId, isSigner: false, isWritable: false }
        ],
        data: this.encodeCreateAccountInstruction({
          size: 512, // 512 bytes for user state
          owner: this.programId,
          rentExempt: true
        })
      },
      rentLamports
    };
  }

  /**
   * Validate account constraints for script execution
   */
  async validateAccountConstraints(
    accounts: FiveAccount[],
    constraints: AccountConstraints
  ): Promise<AccountValidationResult> {
    const errors: string[] = [];
    const warnings: string[] = [];
    let totalRentCost = 0;

    // Validate signers (if specified)
    if (constraints.signers) {
      const providedSigners = accounts.filter(acc => acc.isSigner).map(acc => acc.address);
      for (const requiredSigner of constraints.signers) {
        if (!providedSigners.includes(requiredSigner)) {
          errors.push(`Missing required signer: ${requiredSigner}`);
        }
      }
    }

    // Validate writable accounts (if specified)
    if (constraints.writableAccounts) {
      const providedWritable = accounts.filter(acc => acc.isWritable).map(acc => acc.address);
      for (const requiredWritable of constraints.writableAccounts) {
        if (!providedWritable.includes(requiredWritable)) {
          errors.push(`Missing required writable account: ${requiredWritable}`);
        }
      }
    }

    // Validate readonly accounts (if specified)
    if (constraints.readonlyAccounts) {
      const providedReadonly = accounts.filter(acc => !acc.isWritable).map(acc => acc.address);
      for (const requiredReadonly of constraints.readonlyAccounts) {
        if (!providedReadonly.includes(requiredReadonly)) {
          errors.push(`Missing required readonly account: ${requiredReadonly}`);
        }
      }
    }

    // Validate account types (if specified)
    if (constraints.typeConstraints) {
      for (const [address, expectedType] of constraints.typeConstraints) {
        const account = accounts.find(acc => acc.address === address);
        if (!account) {
          errors.push(`Missing account for type constraint: ${address}`);
          continue;
        }

        if (account.type !== expectedType) {
          errors.push(`Account ${address} has type ${account.type}, expected ${expectedType}`);
        }
      }
    }

    // Validate rent requirements (if specified)
    if (constraints.rentRequirements) {
        for (const [address, requiredRent] of constraints.rentRequirements) {
          const account = accounts.find(acc => acc.address === address);
          if (!account) {
            continue; // Already handled above
          }

          if (account.lamports !== undefined && account.lamports < requiredRent) {
            errors.push(`Account ${address} has ${account.lamports} lamports, needs ${requiredRent} for rent exemption`);
          }

          totalRentCost += requiredRent;
        }
    }

    // Validate maximum accounts constraint
    if (constraints.maxAccounts !== undefined && accounts.length > constraints.maxAccounts) {
      errors.push(`Too many accounts: ${accounts.length}, maximum allowed: ${constraints.maxAccounts}`);
    }

    // Validate maximum total size constraint
    if (constraints.maxTotalSize !== undefined) {
      const totalSize = accounts.reduce((sum, acc) => sum + (acc.size || 0), 0);
      if (totalSize > constraints.maxTotalSize) {
        errors.push(`Total account size ${totalSize} exceeds maximum: ${constraints.maxTotalSize}`);
      }
    }

    // Validate required types constraint
    if (constraints.requiredTypes) {
      const providedTypes = new Set(accounts.map(acc => acc.type));
      for (const requiredType of constraints.requiredTypes) {
        if (!providedTypes.has(requiredType)) {
          errors.push(`Missing required account type: ${requiredType}`);
        }
      }
    }

    // Calculate costs for all accounts
    const accountSizes = accounts.map(account => ({
      type: account.type,
      size: account.size || 0
    }));
    const costs = await this.calculateAccountCreationCosts(accountSizes);
    totalRentCost = costs.rentExemption;

    // Validate maximum rent cost constraint
    if (constraints.maxRentCost !== undefined && totalRentCost > constraints.maxRentCost) {
      errors.push(`Total rent cost ${totalRentCost} exceeds maximum: ${constraints.maxRentCost}`);
    }

    // Validate account addresses
    for (const account of accounts) {
      const addressValidation = AccountValidator.validateAddress(account.address);
      if (!addressValidation.valid) {
        errors.push(`Invalid account address ${account.address}: ${addressValidation.errors.join(', ')}`);
      }
    }

    const valid = errors.length === 0;
    const result: AccountValidationResult = {
      valid,
      errors,
      warnings
    };

    if (valid) {
      result.costs = {
        rentExemption: costs.rentExemption,
        transactionFee: costs.transactionFees,
        totalCost: costs.total
      };
    }

    return result;
  }

  /**
   * Get account info using client-agnostic account fetcher interface
   */
  async getAccountInfo(address: string, accountFetcher?: any): Promise<FiveAccount | null> {
    if (!accountFetcher) {
      throw new Error('Account fetcher required for blockchain operations. Use client-agnostic account fetcher interface.');
    }

    try {
      const accountData = await accountFetcher.getAccountData(address);

      if (!accountData) {
        return null;
      }

      return {
        address,
        type: this.determineAccountTypeFromData(accountData, address),
        isSigner: false, // Cannot determine from account info alone
        isWritable: false, // Cannot determine from account info alone
        owner: accountData.owner,
        size: accountData.data.length,
        lamports: accountData.lamports,
        data: accountData.data
      };
    } catch (error) {
      console.warn(`Failed to get account info for ${address}:`, error);
      return null;
    }
  }

  /**
   * Get multiple account infos in batch using client-agnostic interface
   */
  async getMultipleAccountInfos(addresses: string[], accountFetcher?: any): Promise<Map<string, FiveAccount | null>> {
    if (!accountFetcher) {
      throw new Error('Account fetcher required for blockchain operations. Use client-agnostic account fetcher interface.');
    }

    const results = new Map<string, FiveAccount | null>();

    // Validate addresses first
    const validAddresses: string[] = [];
    
    for (const address of addresses) {
      if (SolanaPublicKeyUtils.isValid(address)) {
        validAddresses.push(address);
      } else {
        // Invalid address - set to null
        results.set(address, null);
      }
    }

    if (validAddresses.length === 0) {
      return results;
    }

    try {
      const accountsData = await accountFetcher.getMultipleAccountsData(validAddresses);

      for (const address of validAddresses) {
        const accountData = accountsData.get(address);

        if (!accountData) {
          results.set(address, null);
          continue;
        }

        results.set(address, {
          address,
          type: this.determineAccountTypeFromData(accountData, address),
          isSigner: false,
          isWritable: false,
          owner: accountData.owner,
          size: accountData.data.length,
          lamports: accountData.lamports,
          data: accountData.data
        });
      }
    } catch (error) {
      // Fallback to individual requests
      for (const address of addresses) {
        const accountInfo = await this.getAccountInfo(address, accountFetcher);
        results.set(address, accountInfo);
      }
    }

    return results;
  }

  /**
   * Check if accounts exist and are properly initialized
   */
  async validateAccountsExist(addresses: string[]): Promise<{
    existing: string[];
    missing: string[];
    invalid: string[];
  }> {
    const existing: string[] = [];
    const missing: string[] = [];
    const invalid: string[] = [];

    for (const address of addresses) {
      if (!SolanaPublicKeyUtils.isValid(address)) {
        invalid.push(address);
        continue;
      }

      const accountInfo = await this.getAccountInfo(address);
      if (accountInfo) {
        existing.push(address);
      } else {
        missing.push(address);
      }
    }

    return { existing, missing, invalid };
  }

  /**
   * Calculate total costs for account creation
   */
  async calculateAccountCreationCosts(
    accounts: Array<{
      type: AccountType;
      size: number;
    }>
  ): Promise<{
    rentExemption: number;
    transactionFees: number;
    total: number;
    breakdown: Array<{
      type: AccountType;
      size: number;
      rent: number;
    }>;
  }> {
    let totalRent = 0;
    const breakdown: Array<{ type: AccountType; size: number; rent: number }> = [];

    for (const account of accounts) {
      const rent = RentCalculator.calculateRentExemption(account.size);
      totalRent += rent;
      breakdown.push({
        type: account.type,
        size: account.size,
        rent
      });
    }

    const transactionFees = 5000 * accounts.length; // Base fee per account creation
    const total = totalRent + transactionFees;

    return {
      rentExemption: totalRent,
      transactionFees,
      total,
      breakdown
    };
  }

  /**
   * Build standard account list for script execution
   */
  buildExecutionAccounts(
    scriptAccount: string,
    userAccount: string,
    additionalAccounts: Array<{
      address: string;
      isSigner: boolean;
      isWritable: boolean;
    }> = []
  ): FiveAccount[] {
    const accounts: FiveAccount[] = [
      {
        address: scriptAccount,
        type: 'script',
        isSigner: false,
        isWritable: false
      },
      {
        address: userAccount,
        type: 'custom',
        isSigner: true,
        isWritable: true
      },
      {
        address: this.programId,
        type: 'custom',
        isSigner: false,
        isWritable: false
      },
      {
        address: '11111111111111111111111111111112', // System Program
        type: 'system',
        isSigner: false,
        isWritable: false
      }
    ];

    // Add additional accounts
    for (const account of additionalAccounts) {
      accounts.push({
        address: account.address,
        type: 'custom',
        isSigner: account.isSigner,
        isWritable: account.isWritable
      });
    }

    return accounts;
  }

  // Private helper methods

  private determineAccountTypeFromData(accountData: AccountData, address: string): AccountType {
    const owner = accountData.owner;

    // System accounts
    if (owner === '11111111111111111111111111111112') {
      return 'system';
    }

    // Five VM accounts
    if (owner === this.programId) {
      if (accountData.data.length > 1000) {
        return 'script'; // Likely contains bytecode
      } else {
        return 'metadata'; // Likely metadata or state
      }
    }

    // SPL Token accounts
    if (owner === 'TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA') {
      return 'spl_token';
    }

    // Sysvar accounts
    if (address === 'SysvarRent111111111111111111111111111111111') {
      return 'rent_sysvar';
    }
    if (address === 'SysvarC1ock11111111111111111111111111111111') {
      return 'clock_sysvar';
    }

    return 'custom';
  }

}

/**
 * Account utilities for client-agnostic operations
 */
export class AccountUtils {
  /**
   * Build serializable account list (client-agnostic)
   */
  static buildSerializableAccounts(accounts: FiveAccount[]): Array<{
    pubkey: string;
    isSigner: boolean;
    isWritable: boolean;
  }> {
    return accounts.map(account => ({
      pubkey: account.address,
      isSigner: account.isSigner,
      isWritable: account.isWritable
    }));
  }


  /**
   * Deduplicate account list while preserving most permissive permissions
   */
  static deduplicateAccounts(accounts: FiveAccount[]): FiveAccount[] {
    const accountMap = new Map<string, FiveAccount>();

    for (const account of accounts) {
      const existing = accountMap.get(account.address);
      
      if (!existing) {
        accountMap.set(account.address, { ...account });
      } else {
        // Keep most permissive permissions
        existing.isSigner = existing.isSigner || account.isSigner;
        existing.isWritable = existing.isWritable || account.isWritable;
      }
    }

    return Array.from(accountMap.values());
  }

  /**
   * Sort accounts by standard Solana conventions
   */
  static sortAccounts(accounts: FiveAccount[]): FiveAccount[] {
    return accounts.sort((a, b) => {
      // Signers first
      if (a.isSigner !== b.isSigner) {
        return b.isSigner ? 1 : -1;
      }

      // Writable accounts next
      if (a.isWritable !== b.isWritable) {
        return b.isWritable ? 1 : -1;
      }

      // Alphabetical by address
      return a.address.localeCompare(b.address);
    });
  }

  /**
   * Validate account list structure and compute statistics
   */
  static validateAccountList(accounts: FiveAccount[]): {
    valid: boolean;
    errors: string[];
    totalSize: number;
    requiredAccounts: FiveAccount[];
    optionalAccounts: FiveAccount[];
  } {
    const errors: string[] = [];
    const requiredAccounts: FiveAccount[] = [];
    const optionalAccounts: FiveAccount[] = [];
    let totalSize = 0;

    if (!Array.isArray(accounts)) {
      errors.push('Accounts must be an array');
      return { 
        valid: false, 
        errors, 
        totalSize: 0, 
        requiredAccounts: [], 
        optionalAccounts: [] 
      };
    }

    for (let i = 0; i < accounts.length; i++) {
      const account = accounts[i];
      const prefix = `Account ${i}`;

      // Validate address
      if (!account.address || !SolanaPublicKeyUtils.isValid(account.address)) {
        errors.push(`${prefix}: Invalid address`);
      }

      // Validate size
      if (account.size !== undefined) {
        if (account.size < 0) {
          errors.push(`${prefix}: size must be positive`);
        } else {
          totalSize += account.size;
        }
      }

      // Categorize by required/optional
      if (account.required) {
        requiredAccounts.push(account);
      } else {
        optionalAccounts.push(account);
      }
    }

    return {
      valid: errors.length === 0,
      errors,
      totalSize,
      requiredAccounts,
      optionalAccounts
    };
  }

  /**
   * Filter accounts by type
   */
  static filterAccountsByType(accounts: FiveAccount[], type: AccountType): FiveAccount[] {
    return accounts.filter(account => account.type === type);
  }

  /**
   * Calculate total size of accounts
   */
  static calculateTotalSize(accounts: FiveAccount[]): number {
    return accounts.reduce((total, account) => total + (account.size || 0), 0);
  }
}
