/**
 * Five SDK Script Metadata System
 * 
 * Real implementation for parsing script account data and extracting ABI information
 * from deployed Five scripts. This replaces mock implementations with production-ready
 * Solana account data parsing.
 */

import { Base58Utils } from '../crypto/index.js';
import { normalizeAbiFunctions } from '../utils/abi.js';

/**
 * Account data interface for client-agnostic blockchain interactions
 */
export interface AccountData {
  /** Account address as base58 string */
  address: string;
  /** Account data as byte array */
  data: Uint8Array;
  /** Account owner program ID */
  owner: string;
  /** Account balance in lamports */
  lamports: number;
}

/**
 * Account fetcher interface for retrieving account data
 */
export interface AccountFetcher {
  /** Get single account data */
  getAccountData(address: string): Promise<AccountData | null>;
  /** Get multiple account data in batch */
  getMultipleAccountsData(addresses: string[]): Promise<Map<string, AccountData | null>>;
}

/**
 * Script metadata extracted from deployed script accounts
 */
export interface ScriptMetadata {
  /** Script account address */
  address: string;
  /** Script bytecode */
  bytecode: Uint8Array;
  /** Script ABI with function definitions */
  abi: ScriptABI;
  /** Deploy timestamp */
  deployedAt: number;
  /** Script version */
  version: string;
  /** Authority that deployed the script */
  authority: string;
}

/**
 * Script ABI definition
 */
export interface ScriptABI {
  /** Script name */
  name: string;
  /** Function definitions */
  functions: FunctionDefinition[];
  /** Type definitions */
  types?: TypeDefinition[];
  /** Account constraints */
  accounts?: AccountConstraint[];
}

/**
 * Function definition in script ABI
 */
export interface FunctionDefinition {
  /** Function name */
  name: string;
  /** Function index in bytecode */
  index: number;
  /** Function parameters */
  parameters: ParameterDefinition[];
  /** Return type */
  returnType?: string;
  /** Function visibility */
  visibility: 'public' | 'private';
  /** Documentation */
  docs?: string;
}

/**
 * Parameter definition
 */
export interface ParameterDefinition {
  /** Parameter name */
  name: string;
  /** Parameter type */
  type: string;
  /** Whether parameter is optional */
  optional?: boolean;
  /** Parameter documentation */
  docs?: string;
}

/**
 * Type definition
 */
export interface TypeDefinition {
  /** Type name */
  name: string;
  /** Type structure */
  structure: 'struct' | 'enum' | 'alias';
  /** Type fields (for structs) */
  fields?: Array<{
    name: string;
    type: string;
  }>;
  /** Type variants (for enums) */
  variants?: Array<{
    name: string;
    value?: number;
  }>;
  /** Alias target (for type aliases) */
  target?: string;
}

/**
 * Account constraint definition
 */
export interface AccountConstraint {
  /** Account name */
  name: string;
  /** Account type */
  type: 'signer' | 'writable' | 'readonly' | 'pda';
  /** Seeds for PDA derivation */
  seeds?: string[];
  /** Required account properties */
  properties?: Record<string, any>;
}

/**
 * Script account data layout
 */
interface ScriptAccountData {
  /** Magic bytes to identify script accounts */
  magic: Uint8Array; // 8 bytes: "FIVE_SCR"
  /** Schema version */
  version: number; // 4 bytes
  /** Deploy timestamp */
  timestamp: number; // 8 bytes (u64)
  /** Authority pubkey */
  authority: Uint8Array; // 32 bytes
  /** Bytecode length */
  bytecodeLength: number; // 4 bytes (u32)
  /** ABI length */
  abiLength: number; // 4 bytes (u32)
  /** Reserved space */
  reserved: Uint8Array; // 8 bytes
  /** Bytecode data */
  bytecode: Uint8Array; // Variable length
  /** ABI data (JSON) */
  abi: Uint8Array; // Variable length
}

/**
 * Script metadata parser and manager
 */
export class ScriptMetadataParser {
  private static readonly SCRIPT_MAGIC = new Uint8Array([
    0x46, 0x49, 0x56, 0x45, 0x5F, 0x53, 0x43, 0x52 // "FIVE_SCR"
  ]);
  
  private static readonly CURRENT_VERSION = 1;
  private static readonly HEADER_SIZE = 64; // Fixed header size

  /**
   * Parse script metadata from account data
   */
  static parseMetadata(accountData: Uint8Array, address: string): ScriptMetadata {
    if (accountData.length < this.HEADER_SIZE) {
      throw new Error(`Invalid script account: data too small (${accountData.length} bytes, minimum ${this.HEADER_SIZE})`);
    }

    let offset = 0;

    // Parse header
    const magic = accountData.slice(offset, offset + 8);
    offset += 8;

    if (!this.arraysEqual(magic, this.SCRIPT_MAGIC)) {
      throw new Error('Invalid script account: magic bytes mismatch');
    }

    const version = this.readU32(accountData, offset);
    offset += 4;

    if (version > this.CURRENT_VERSION) {
      throw new Error(`Unsupported script version: ${version} (max supported: ${this.CURRENT_VERSION})`);
    }

    const timestamp = this.readU64(accountData, offset);
    offset += 8;

    const authority = accountData.slice(offset, offset + 32);
    offset += 32;

    const bytecodeLength = this.readU32(accountData, offset);
    offset += 4;

    const abiLength = this.readU32(accountData, offset);
    offset += 4;

    // Skip reserved space
    offset += 8;

    // Validate data lengths
    const expectedSize = this.HEADER_SIZE + bytecodeLength + abiLength;
    if (accountData.length < expectedSize) {
      throw new Error(`Invalid script account: expected ${expectedSize} bytes, got ${accountData.length}`);
    }

    // Extract bytecode
    const bytecode = accountData.slice(offset, offset + bytecodeLength);
    offset += bytecodeLength;

    // Extract and parse ABI
    const abiData = accountData.slice(offset, offset + abiLength);
    const abiJson = new TextDecoder().decode(abiData);
    
    let abi: ScriptABI;
    try {
      abi = JSON.parse(abiJson);
    } catch (error) {
      throw new Error(`Invalid ABI JSON: ${error instanceof Error ? error.message : 'Parse error'}`);
    }

    return {
      address,
      bytecode,
      abi,
      deployedAt: timestamp,
      version: version.toString(),
      authority: Base58Utils.encode(authority)
    };
  }

  /**
   * Get script metadata from blockchain using account fetcher
   */
  static async getScriptMetadata(
    accountFetcher: AccountFetcher,
    scriptAddress: string
  ): Promise<ScriptMetadata> {
    try {
      // Validate address format (basic base58 check)
      if (!scriptAddress || scriptAddress.length < 32 || scriptAddress.length > 44) {
        throw new Error(`Invalid script address format: ${scriptAddress}`);
      }

      // Fetch account data
      const accountData = await accountFetcher.getAccountData(scriptAddress);

      if (!accountData) {
        throw new Error(`Script account not found: ${scriptAddress}`);
      }

      if (!accountData.data || accountData.data.length === 0) {
        throw new Error(`Script account has no data: ${scriptAddress}`);
      }

      // Parse metadata
      return this.parseMetadata(accountData.data, scriptAddress);
      
    } catch (error) {
      throw new Error(`Failed to get script metadata for ${scriptAddress}: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }

  /**
   * Get multiple script metadata entries using account fetcher
   */
  static async getMultipleScriptMetadata(
    accountFetcher: AccountFetcher,
    scriptAddresses: string[]
  ): Promise<Map<string, ScriptMetadata | null>> {
    const results = new Map<string, ScriptMetadata | null>();

    // Validate addresses (basic format check)
    const validAddresses: string[] = [];
    for (const address of scriptAddresses) {
      if (address && address.length >= 32 && address.length <= 44) {
        validAddresses.push(address);
      } else {
        results.set(address, null);
      }
    }

    if (validAddresses.length === 0) {
      return results;
    }

    try {
      // Batch fetch account data
      const accountDataMap = await accountFetcher.getMultipleAccountsData(validAddresses);
      
      // Parse metadata for each account
      for (const address of validAddresses) {
        const accountData = accountDataMap.get(address);

        if (!accountData || !accountData.data || accountData.data.length === 0) {
          results.set(address, null);
          continue;
        }

        try {
          const metadata = this.parseMetadata(accountData.data, address);
          results.set(address, metadata);
        } catch (error) {
          console.warn(`Failed to parse metadata for ${address}:`, error);
          results.set(address, null);
        }
      }
    } catch (error) {
      throw new Error(`Batch metadata fetch failed: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }

    return results;
  }

  /**
   * Extract function signatures from ABI
   */
  static extractFunctionSignatures(abi: ScriptABI): Array<{
    name: string;
    index: number;
    parameters: ParameterDefinition[];
    signature: string;
  }> {
    const functions = normalizeAbiFunctions(abi.functions ?? (abi as any)).map<FunctionDefinition>(
      (func) => ({
        name: func.name,
        index: func.index,
        parameters: func.parameters,
        returnType: func.returnType,
        visibility: func.visibility ?? 'public',
      }),
    );

    return functions.map(func => ({
      name: func.name,
      index: func.index,
      parameters: func.parameters,
      signature: this.generateFunctionSignature(func)
    }));
  }

  /**
   * Generate function signature string
   */
  static generateFunctionSignature(func: FunctionDefinition): string {
    const paramStrings = func.parameters.map(param => 
      `${param.name}: ${param.type}${param.optional ? '?' : ''}`
    );
    const returnType = func.returnType ? ` -> ${func.returnType}` : '';
    return `${func.name}(${paramStrings.join(', ')})${returnType}`;
  }

  /**
   * Validate script ABI structure
   */
  static validateABI(abi: any): { valid: boolean; errors: string[] } {
    const errors: string[] = [];

    if (!abi || typeof abi !== 'object') {
      errors.push('ABI must be an object');
      return { valid: false, errors };
    }

    if (typeof abi.name !== 'string' || abi.name.length === 0) {
      errors.push('ABI must have a non-empty name');
    }

    const functions = normalizeAbiFunctions(abi.functions ?? (abi as any));

    if (functions.length === 0) {
      errors.push('ABI must have at least one function');
    }

    for (let i = 0; i < functions.length; i++) {
      const func = functions[i];
      const funcErrors = this.validateFunction(func, i);
      errors.push(...funcErrors);
    }

    return {
      valid: errors.length === 0,
      errors
    };
  }

  /**
   * Validate function definition
   */
  private static validateFunction(func: any, index: number): string[] {
    const errors: string[] = [];
    const prefix = `Function ${index}`;

    if (typeof func.name !== 'string' || func.name.length === 0) {
      errors.push(`${prefix}: must have a non-empty name`);
    }

    if (typeof func.index !== 'number' || func.index < 0) {
      errors.push(`${prefix}: must have a non-negative index`);
    }

    if (!Array.isArray(func.parameters)) {
      errors.push(`${prefix}: must have a parameters array`);
    }

    if (func.visibility && !['public', 'private'].includes(func.visibility)) {
      errors.push(`${prefix}: visibility must be 'public' or 'private'`);
    }

    return errors;
  }

  // Utility methods for binary data parsing

  private static readU32(data: Uint8Array, offset: number): number {
    return (
      data[offset] |
      (data[offset + 1] << 8) |
      (data[offset + 2] << 16) |
      (data[offset + 3] << 24)
    ) >>> 0; // Convert to unsigned
  }

  private static readU64(data: Uint8Array, offset: number): number {
    // Read as two 32-bit values and combine (JavaScript limitation for large numbers)
    const low = this.readU32(data, offset);
    const high = this.readU32(data, offset + 4);
    return low + (high * 0x100000000);
  }

  private static arraysEqual(a: Uint8Array, b: Uint8Array): boolean {
    if (a.length !== b.length) return false;
    for (let i = 0; i < a.length; i++) {
      if (a[i] !== b[i]) return false;
    }
    return true;
  }
}

/**
 * Client-agnostic metadata cache
 */
export class MetadataCache {
  private cache = new Map<string, {
    metadata: ScriptMetadata;
    timestamp: number;
    ttl: number;
  }>();

  private defaultTTL = 5 * 60 * 1000; // 5 minutes

  /**
   * Get metadata from cache or fetch
   */
  async getMetadata(
    scriptAddress: string,
    fetcher: (address: string) => Promise<ScriptMetadata>,
    ttl: number = this.defaultTTL
  ): Promise<ScriptMetadata> {
    const now = Date.now();
    const cached = this.cache.get(scriptAddress);

    if (cached && (now - cached.timestamp) < cached.ttl) {
      return cached.metadata;
    }

    // Fetch fresh metadata
    const metadata = await fetcher(scriptAddress);
    
    // Cache the result
    this.cache.set(scriptAddress, {
      metadata,
      timestamp: now,
      ttl
    });

    return metadata;
  }

  /**
   * Invalidate cache entry
   */
  invalidate(scriptAddress: string): void {
    this.cache.delete(scriptAddress);
  }

  /**
   * Clear expired entries
   */
  cleanup(): void {
    const now = Date.now();
    for (const [address, entry] of this.cache.entries()) {
      if ((now - entry.timestamp) >= entry.ttl) {
        this.cache.delete(address);
      }
    }
  }

  /**
   * Get cache statistics
   */
  getStats(): {
    size: number;
    hitRate: number;
    entries: Array<{
      address: string;
      age: number;
      ttl: number;
    }>;
  } {
    const now = Date.now();
    return {
      size: this.cache.size,
      hitRate: 0, // Would need to track hits/misses
      entries: Array.from(this.cache.entries()).map(([address, entry]) => ({
        address,
        age: now - entry.timestamp,
        ttl: entry.ttl
      }))
    };
  }
}
