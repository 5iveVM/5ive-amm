/**
 * Account Meta Generator for Five VM Testing
 * 
 * Generates AccountMeta structures for testing account system scripts
 * based on ABI requirements and constraint types (@signer, @mut, @init)
 */

import { SolanaPublicKeyUtils, Base58Utils } from '../crypto/index.js';

/**
 * Account constraint types from Five VM
 */
export interface AccountConstraints {
  name: string;
  writable: boolean;
  signer: boolean;
  init?: boolean; // Derived from account parameter patterns
}

/**
 * Generated account metadata for testing
 */
export interface GeneratedAccountMeta {
  pubkey: string;
  isSigner: boolean;
  isWritable: boolean;
  keypair?: {
    publicKey: string;
    secretKey: Uint8Array;
  };
}

/**
 * Test account generation context
 */
export interface TestAccountContext {
  script: string;
  functionIndex: number;
  accounts: GeneratedAccountMeta[];
  stateData?: Map<string, any>;
}

/**
 * Account Meta Generator for Five VM test scripts
 */
export class AccountMetaGenerator {
  private static accountCache = new Map<string, GeneratedAccountMeta>();
  private static stateDataCache = new Map<string, any>();

  /**
   * Generate AccountMeta array from ABI function definition
   */
  static async generateAccountsForFunction(
    abi: any,
    functionName: string,
    options: {
      reuseAccounts?: boolean;
      generateStateData?: boolean;
      debug?: boolean;
    } = {}
  ): Promise<TestAccountContext> {
    const func = abi.functions?.[functionName];
    if (!func) {
      throw new Error(`Function '${functionName}' not found in ABI`);
    }

    const accountSpecs = func.accounts || [];
    const accounts: GeneratedAccountMeta[] = [];
    const stateData = new Map<string, any>();

    if (options.debug) {
      console.log(`[AccountMetaGenerator] Generating accounts for ${functionName}:`);
      console.log(`  - Found ${accountSpecs.length} account requirements`);
    }

    for (const accountSpec of accountSpecs) {
      const accountMeta = await this.generateAccountMeta(
        accountSpec,
        options
      );
      
      accounts.push(accountMeta);

      // Generate state data for state accounts
      if (options.generateStateData && this.isStateAccount(accountSpec)) {
        const state = this.generateStateData(accountSpec);
        stateData.set(accountSpec.name, state);
      }

      if (options.debug) {
        console.log(`  - ${accountSpec.name}: ${accountMeta.pubkey} (signer: ${accountMeta.isSigner}, writable: ${accountMeta.isWritable})`);
      }
    }

    return {
      script: functionName,
      functionIndex: func.index,
      accounts,
      stateData: stateData.size > 0 ? stateData : undefined
    };
  }

  /**
   * Generate single AccountMeta from account specification
   */
  private static async generateAccountMeta(
    accountSpec: AccountConstraints,
    options: {
      reuseAccounts?: boolean;
      debug?: boolean;
    }
  ): Promise<GeneratedAccountMeta> {
    // Check cache for reusable accounts
    const cacheKey = `${accountSpec.name}_${accountSpec.signer}_${accountSpec.writable}`;
    if (options.reuseAccounts && this.accountCache.has(cacheKey)) {
      const cached = this.accountCache.get(cacheKey)!;
      if (options.debug) {
        console.log(`    [Cache Hit] Reusing account for ${accountSpec.name}: ${cached.pubkey}`);
      }
      return cached;
    }

    let accountMeta: GeneratedAccountMeta;

    if (accountSpec.signer) {
      // Generate new keypair for signer accounts
      accountMeta = await this.generateSignerAccount(accountSpec, options);
    } else {
      // Generate regular account for non-signer accounts
      accountMeta = await this.generateRegularAccount(accountSpec, options);
    }

    // Cache for reuse
    if (options.reuseAccounts) {
      this.accountCache.set(cacheKey, accountMeta);
    }

    return accountMeta;
  }

  /**
   * Generate signer account with keypair
   */
  private static async generateSignerAccount(
    accountSpec: AccountConstraints,
    options: { debug?: boolean }
  ): Promise<GeneratedAccountMeta> {
    // Generate new keypair for signer
    const crypto = await import('crypto');
    const keypair = crypto.generateKeyPairSync('ed25519', {
      publicKeyEncoding: { type: 'spki', format: 'der' },
      privateKeyEncoding: { type: 'pkcs8', format: 'der' }
    });

    // Extract raw public key (32 bytes) from SPKI DER encoding
    // SPKI format for ed25519: 12-byte header + 32-byte key
    const publicKeyDer = keypair.publicKey as unknown as Buffer;
    const publicKeyRaw = publicKeyDer.slice(-32); // Last 32 bytes are the raw key
    const publicKey = Base58Utils.encode(new Uint8Array(publicKeyRaw));

    // Extract private key (64 bytes from PKCS8: 32-byte seed + 32-byte public key)
    const privateKeyDer = keypair.privateKey as unknown as Buffer;
    const secretKey = new Uint8Array(privateKeyDer.slice(-64, -32)); // Extract the seed

    if (options.debug) {
      console.log(`    [Signer] Generated keypair for ${accountSpec.name}: ${publicKey}`);
    }

    return {
      pubkey: publicKey,
      isSigner: true,
      isWritable: accountSpec.writable,
      keypair: {
        publicKey,
        secretKey
      }
    };
  }

  /**
   * Generate regular (non-signer) account
   */
  private static async generateRegularAccount(
    accountSpec: AccountConstraints,
    options: { debug?: boolean }
  ): Promise<GeneratedAccountMeta> {
    // Generate random address for non-signer accounts
    const randomAddress = SolanaPublicKeyUtils.random();

    if (options.debug) {
      console.log(`    [Regular] Generated address for ${accountSpec.name}: ${randomAddress}`);
    }

    return {
      pubkey: randomAddress,
      isSigner: false,
      isWritable: accountSpec.writable
    };
  }

  /**
   * Check if account is a state account that needs data
   */
  private static isStateAccount(accountSpec: AccountConstraints): boolean {
    // Common patterns for state accounts
    const statePatterns = [
      'state',
      'account',
      'data',
      'storage'
    ];
    
    const name = accountSpec.name.toLowerCase();
    return statePatterns.some(pattern => name.includes(pattern));
  }

  /**
   * Generate mock state data for state accounts
   */
  private static generateStateData(accountSpec: AccountConstraints): any {
    // Generate appropriate state data based on account name
    const name = accountSpec.name.toLowerCase();
    
    if (name.includes('state')) {
      return {
        count: 42,
        total_operations: 0,
        admin: SolanaPublicKeyUtils.random(),
        created_accounts: 0,
        modification_count: 0
      };
    }
    
    if (name.includes('init')) {
      return {
        created_accounts: 0,
        admin: SolanaPublicKeyUtils.random()
      };
    }
    
    if (name.includes('mut')) {
      return {
        modification_count: 0
      };
    }
    
    // Default state data
    return {
      value: 42,
      owner: SolanaPublicKeyUtils.random()
    };
  }

  /**
   * Format accounts for Five CLI execution
   */
  static formatAccountsForCLI(context: TestAccountContext): {
    accountsParam: string;
    keypairsNeeded: Array<{ name: string; keypair: any }>;
  } {
    const accounts = context.accounts.map(acc => acc.pubkey).join(',');
    const keypairs = context.accounts
      .filter(acc => acc.keypair)
      .map(acc => ({
        name: acc.pubkey,
        keypair: acc.keypair!
      }));

    return {
      accountsParam: accounts,
      keypairsNeeded: keypairs
    };
  }

  /**
   * Generate accounts from .five file
   */
  static async generateFromFiveFile(
    fiveFilePath: string,
    functionName: string = 'test',
    options: {
      reuseAccounts?: boolean;
      generateStateData?: boolean;
      debug?: boolean;
    } = {}
  ): Promise<TestAccountContext> {
    const fs = await import('fs');
    const path = await import('path');
    
    if (!fs.existsSync(fiveFilePath)) {
      throw new Error(`Five file not found: ${fiveFilePath}`);
    }

    const fiveData = JSON.parse(fs.readFileSync(fiveFilePath, 'utf8'));
    const abi = fiveData.abi;

    if (!abi || !abi.functions) {
      throw new Error(`Invalid Five file: missing ABI or functions in ${fiveFilePath}`);
    }

    return this.generateAccountsForFunction(abi, functionName, options);
  }

  /**
   * Clear account cache (useful for testing)
   */
  static clearCache(): void {
    this.accountCache.clear();
    this.stateDataCache.clear();
  }

  /**
   * Get account cache statistics
   */
  static getCacheStats(): {
    accountsCached: number;
    stateDataCached: number;
  } {
    return {
      accountsCached: this.accountCache.size,
      stateDataCached: this.stateDataCache.size
    };
  }
}

/**
 * Utility functions for account management
 */
export class AccountTestUtils {
  /**
   * Create test accounts for common constraint patterns
   */
  static async createStandardTestAccounts(): Promise<{
    payer: GeneratedAccountMeta;
    authority: GeneratedAccountMeta;
    state: GeneratedAccountMeta;
    readonly: GeneratedAccountMeta;
  }> {
    const generator = AccountMetaGenerator;
    
    const [payer, authority, state, readonly] = await Promise.all([
      generator['generateSignerAccount']({ name: 'payer', signer: true, writable: true }, {}),
      generator['generateSignerAccount']({ name: 'authority', signer: true, writable: false }, {}),
      generator['generateRegularAccount']({ name: 'state', signer: false, writable: true }, {}),
      generator['generateRegularAccount']({ name: 'readonly', signer: false, writable: false }, {})
    ]);

    return { payer, authority, state, readonly };
  }

  /**
   * Validate account constraints match requirements
   */
  static validateAccountConstraints(
    accounts: GeneratedAccountMeta[],
    requirements: AccountConstraints[]
  ): {
    valid: boolean;
    errors: string[];
  } {
    const errors: string[] = [];

    if (accounts.length !== requirements.length) {
      errors.push(`Account count mismatch: expected ${requirements.length}, got ${accounts.length}`);
      return { valid: false, errors };
    }

    for (let i = 0; i < requirements.length; i++) {
      const account = accounts[i];
      const requirement = requirements[i];

      if (account.isSigner !== requirement.signer) {
        errors.push(`Account ${i} (${requirement.name}): signer mismatch - expected ${requirement.signer}, got ${account.isSigner}`);
      }

      if (account.isWritable !== requirement.writable) {
        errors.push(`Account ${i} (${requirement.name}): writable mismatch - expected ${requirement.writable}, got ${account.isWritable}`);
      }

      if (requirement.signer && !account.keypair) {
        errors.push(`Account ${i} (${requirement.name}): signer account missing keypair`);
      }
    }

    return {
      valid: errors.length === 0,
      errors
    };
  }
}