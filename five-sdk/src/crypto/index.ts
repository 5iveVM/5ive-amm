/**
 * Five SDK crypto utilities.
 */

import bs58 from 'bs58';
import { createHash, randomBytes } from 'crypto';
import { PublicKey } from '@solana/web3.js';

/**
 * Program Derived Address (PDA) utilities - pure implementation
 */
export class PDAUtils {
  private static readonly PDA_MARKER = Buffer.from('ProgramDerivedAddress');

  /**
   * Derive script account using seed-based derivation compatible with SystemProgram.createAccountWithSeed
   * @param bytecode - The bytecode to derive address for
   * @param basePublicKey - The deployer/base public key used for createWithSeed
   * @param programId - The Five VM program ID (required - no default to enforce explicit configuration)
   */
  static async deriveScriptAccount(
    bytecode: Uint8Array,
    basePublicKey: string,
    programId: string
  ): Promise<{
    address: string;
    bump: number;
    seed: string;
  }> {
    try {
      const deployerKey = new PublicKey(basePublicKey);
      const programKey = new PublicKey(programId);
      const seed = createHash('sha256')
        .update(Buffer.from(bytecode))
        .digest('hex')
        .slice(0, 32);
      const address = await PublicKey.createWithSeed(deployerKey, seed, programKey);

      return {
        address: address.toBase58(),
        bump: 0, // Seed-based accounts don't use bumps
        seed: seed
      };
    } catch (error) {
      throw new Error(`Failed to derive script account: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }

  /**
   * Pure PDA derivation implementation (Solana-compatible)
   */
  static async findProgramAddress(
    seeds: Buffer[],
    programId: string
  ): Promise<{ address: string; bump: number }> {
    const crypto = await import('crypto');
    const programIdBytes = Base58Utils.decode(programId);

    // Try the full Solana bump range from 255 down to 0.
    for (let bump = 255; bump >= 0; bump--) {
      const seedsWithBump = [...seeds, Buffer.from([bump])];

      // Create the hash input
      let hashInput = Buffer.alloc(0);
      for (const seed of seedsWithBump) {
        hashInput = Buffer.concat([hashInput, seed]);
      }
      hashInput = Buffer.concat([hashInput, Buffer.from(programIdBytes), this.PDA_MARKER]);

      // Hash and check if it's on curve (simplified check)
      const hash = crypto.createHash('sha256').update(hashInput).digest();

      // Basic curve check (simplified - real Solana checks ed25519 curve)
      if (this.isOffCurve(hash)) {
        return {
          address: Base58Utils.encode(new Uint8Array(hash)),
          bump
        };
      }
    }

    throw new Error('Unable to find valid program address');
  }

  /**
   * Check if hash is off the Ed25519 curve (valid PDA)
   */
  private static isOffCurve(hash: Buffer): boolean {
    return !PublicKey.isOnCurve(hash);
  }

  /**
   * Derive metadata account PDA for script
   * @param scriptAccount - The script account address
   * @param programId - The Five VM program ID (required - no default to enforce explicit configuration)
   */
  static async deriveMetadataAccount(
    scriptAccount: string,
    programId: string
  ): Promise<{
    address: string;
    bump: number;
  }> {
    try {
      const scriptAccountBytes = Base58Utils.decode(scriptAccount);
      const result = await this.findProgramAddress(
        [Buffer.from(scriptAccountBytes), Buffer.from('metadata', 'utf8')],
        programId
      );

      return result;
    } catch (error) {
      throw new Error(`Failed to derive metadata account PDA: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }

  /**
   * Derive user state account PDA
   * @param userPublicKey - The user's public key
   * @param scriptAccount - The script account address
   * @param programId - The Five VM program ID (required - no default to enforce explicit configuration)
   */
  static async deriveUserStateAccount(
    userPublicKey: string,
    scriptAccount: string,
    programId: string
  ): Promise<{
    address: string;
    bump: number;
  }> {
    try {
      const userBytes = Base58Utils.decode(userPublicKey);
      const scriptBytes = Base58Utils.decode(scriptAccount);

      const result = await this.findProgramAddress(
        [
          Buffer.from(userBytes),
          Buffer.from(scriptBytes),
          Buffer.from('state', 'utf8')
        ],
        programId
      );

      return result;
    } catch (error) {
      throw new Error(`Failed to derive user state account PDA: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }

  /**
   * Derive VM state PDA for the given program ID
   * @param programId - The Five VM program ID (required - no default to enforce explicit configuration)
   */
  static async deriveVMStatePDA(
    programId: string
  ): Promise<{
    address: string;
    bump: number;
  }> {
    try {
      // Use algorithmic derivation; no hardcoded PDA
      console.log(`[deriveVMStatePDA] Deriving VM state PDA algorithmically`);
      return await this.findProgramAddress(
        [Buffer.from('vm_state', 'utf8')],
        programId
      );
    } catch (error) {
      throw new Error(`Failed to derive VM state PDA: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }

  /**
   * Validate that a given address is a valid PDA for the given seeds
   */
  static async validatePDA(
    address: string,
    seeds: Buffer[],
    programId: string
  ): Promise<boolean> {
    try {
      const expectedResult = await this.findProgramAddress(seeds, programId);
      return expectedResult.address === address;
    } catch {
      return false;
    }
  }
}

/**
 * Base58 encoding/decoding utilities (proper Solana-compatible implementation)
 */
export class Base58Utils {
  /**
   * Encode bytes to base58 string
   */
  static encode(bytes: Uint8Array): string {
    return bs58.encode(bytes);
  }

  /**
   * Decode base58 string to bytes
   */
  static decode(base58String: string): Uint8Array {
    try {
      return new Uint8Array(bs58.decode(base58String));
    } catch (error) {
      throw new Error(`Invalid base58 string: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }

  /**
   * Validate base58 string format
   */
  static isValid(base58String: string): boolean {
    try {
      bs58.decode(base58String);
      return true;
    } catch {
      return false;
    }
  }

  /**
   * Generate a random base58 string of specified length
   */
  static random(byteLength: number = 32): string {
    const bytes = randomBytes(byteLength);
    return bs58.encode(bytes);
  }
}

/**
 * PublicKey utilities for Solana addresses - pure implementation
 */
export class SolanaPublicKeyUtils {
  /**
   * Validate Solana public key format (32 bytes, valid base58)
   */
  static isValid(address: string): boolean {
    try {
      const decoded = Base58Utils.decode(address);
      // Solana addresses are 32 bytes
      return decoded.length === 32;
    } catch {
      return false;
    }
  }

  /**
   * Normalize and return valid address (throws on invalid)
   */
  static normalize(address: string): string {
    if (!this.isValid(address)) {
      throw new Error(`Invalid Solana address: ${address}`);
    }
    // Re-encode to ensure consistent format
    return Base58Utils.encode(Base58Utils.decode(address));
  }

  /**
   * Convert address string to bytes (throws on invalid)
   */
  static toBytes(address: string): Uint8Array {
    if (!this.isValid(address)) {
      throw new Error(`Invalid Solana address: ${address}`);
    }
    return Base58Utils.decode(address);
  }

  /**
   * Convert bytes to address string
   */
  static fromBytes(bytes: Uint8Array): string {
    if (bytes.length !== 32) {
      throw new Error(`Invalid public key bytes: expected 32 bytes, got ${bytes.length}`);
    }
    return Base58Utils.encode(bytes);
  }

  /**
   * Generate random public key for testing
   */
  static random(): string {
    return Base58Utils.random(32);
  }

  /**
   * Check if two addresses are equal
   */
  static equals(address1: string, address2: string): boolean {
    try {
      const normalized1 = this.normalize(address1);
      const normalized2 = this.normalize(address2);
      return normalized1 === normalized2;
    } catch {
      return false;
    }
  }
}

/**
 * Rent calculation utilities using Solana rent sysvar
 */
export class RentCalculator {
  // Solana rent-exempt minimum (updated 2024 values)
  private static readonly RENT_PER_BYTE_YEAR = 3480; // lamports per byte per year
  private static readonly RENT_EXEMPTION_THRESHOLD = 2 * 365 * 24 * 60 * 60; // 2 years in seconds

  /**
   * Calculate minimum rent-exempt balance for account size
   */
  static async calculateMinimumBalance(accountSize: number): Promise<number> {
    // Account header size (32 bytes for owner + 8 bytes for lamports + other metadata)
    const totalSize = accountSize + 128; // Account overhead

    // Calculate rent exemption (simplified calculation)
    const rentPerYear = totalSize * this.RENT_PER_BYTE_YEAR;
    const rentExemption = Math.ceil((rentPerYear * this.RENT_EXEMPTION_THRESHOLD) / (365 * 24 * 60 * 60));

    return rentExemption;
  }

  /**
   * Check if balance is rent exempt for given account size
   */
  static async isRentExempt(balance: number, accountSize: number): Promise<boolean> {
    const minimumBalance = await this.calculateMinimumBalance(accountSize);
    return balance >= minimumBalance;
  }

  /**
   * Calculate minimum rent-exempt balance for account size (legacy method)
   */
  static calculateRentExemption(accountSize: number): number {
    // Account header size (32 bytes for owner + 8 bytes for lamports + other metadata)
    const totalSize = accountSize + 128; // Account overhead

    // Calculate rent exemption (simplified calculation)
    const rentPerYear = totalSize * this.RENT_PER_BYTE_YEAR;
    const rentExemption = Math.ceil((rentPerYear * this.RENT_EXEMPTION_THRESHOLD) / (365 * 24 * 60 * 60));

    return rentExemption;
  }

  /**
   * Query rent from RPC when possible and fall back to local estimation otherwise.
   */
  static async calculateRentExemptionWithConnection(
    accountSize: number,
    connection?: {
      getMinimumBalanceForRentExemption?: (size: number) => Promise<number>;
    }
  ): Promise<number> {
    if (connection?.getMinimumBalanceForRentExemption) {
      try {
        return await connection.getMinimumBalanceForRentExemption(accountSize);
      } catch {
        // Fall back to local estimation below.
      }
    }
    return this.calculateRentExemption(accountSize);
  }

  /**
   * Get estimated rent for script account based on bytecode size
   */
  static getScriptAccountRent(bytecodeSize: number): number {
    // Script account includes: bytecode + metadata + ABI info
    const metadataSize = 256; // Estimated metadata size
    const totalAccountSize = bytecodeSize + metadataSize;

    return this.calculateRentExemption(totalAccountSize);
  }

  /**
   * Get estimated rent for user state account
   */
  static getUserStateAccountRent(): number {
    // User state accounts are typically small (256-512 bytes)
    const stateAccountSize = 512;
    return this.calculateRentExemption(stateAccountSize);
  }

  /**
   * Get estimated rent for metadata account
   */
  static getMetadataAccountRent(): number {
    // Metadata accounts store ABI and function signatures
    const metadataAccountSize = 1024; // 1KB for metadata
    return this.calculateRentExemption(metadataAccountSize);
  }

  /**
   * Format lamports as SOL string
   */
  static formatSOL(lamports: number): string {
    const sol = lamports / 1e9;
    return `${sol.toFixed(9)} SOL`;
  }

  /**
   * Convert SOL to lamports
   */
  static solToLamports(sol: number): number {
    return Math.floor(sol * 1e9);
  }

  /**
   * Convert lamports to SOL
   */
  static lamportsToSol(lamports: number): number {
    return lamports / 1e9;
  }
}

/**
 * Hash utilities for cryptographic operations
 */
export class HashUtils {
  /**
   * SHA256 hash of data
   */
  static async sha256(data: Uint8Array): Promise<Uint8Array> {
    const crypto = await import('crypto');
    return new Uint8Array(crypto.createHash('sha256').update(data).digest());
  }

  /**
   * Create deterministic seed from multiple inputs
   */
  static async createSeed(inputs: (string | Uint8Array)[]): Promise<Uint8Array> {
    const crypto = await import('crypto');
    const hash = crypto.createHash('sha256');

    for (const input of inputs) {
      if (typeof input === 'string') {
        hash.update(Buffer.from(input, 'utf8'));
      } else {
        hash.update(input);
      }
    }

    return new Uint8Array(hash.digest());
  }

  /**
   * Generate random bytes for cryptographic use
   */
  static async randomBytes(length: number): Promise<Uint8Array> {
    const crypto = await import('crypto');
    return new Uint8Array(crypto.randomBytes(length));
  }
}

/**
 * Account validation utilities
 */
export class AccountValidator {
  /**
   * Validate account address format
   */
  static validateAddress(address: string): {
    valid: boolean;
    errors: string[];
    normalizedAddress: string | null;
  } {
    try {
      const normalizedAddress = SolanaPublicKeyUtils.normalize(address);
      return {
        valid: true,
        errors: [],
        normalizedAddress
      };
    } catch (error) {
      return {
        valid: false,
        errors: [`Invalid Solana address: ${error instanceof Error ? error.message : 'Unknown error'}`],
        normalizedAddress: null
      };
    }
  }

  /**
   * Validate list of account addresses
   */
  static validateAccountList(addresses: string[]): {
    valid: boolean;
    errors: string[];
    validAddresses: string[];
    invalidAddresses: string[];
  } {
    const errors: string[] = [];
    const validAddresses: string[] = [];
    const invalidAddresses: string[] = [];

    for (const address of addresses) {
      const validation = this.validateAddress(address);
      if (validation.valid) {
        validAddresses.push(address);
      } else {
        invalidAddresses.push(address);
        errors.push(...validation.errors);
      }
    }

    return {
      valid: invalidAddresses.length === 0,
      errors,
      validAddresses,
      invalidAddresses
    };
  }

  /**
   * Validate program ID
   */
  static validateProgramId(programId: string): {
    isValid: boolean;
    error?: string;
  } {
    const addressValidation = this.validateAddress(programId);
    if (!addressValidation.valid) {
      return {
        isValid: false,
        error: addressValidation.errors[0] || 'Invalid program ID'
      };
    }

    // Additional program ID validation could go here
    // (e.g., checking if it's a known program)

    return { isValid: true };
  }

  /**
   * Validate script account structure
   */
  static validateScriptAccount(accountData: {
    address: string;
    bytecodeSize?: number;
    rentExempt?: boolean;
  }): {
    isValid: boolean;
    errors: string[];
  } {
    const errors: string[] = [];

    // Validate address
    const addressValidation = this.validateAddress(accountData.address);
    if (!addressValidation.valid) {
      errors.push(...addressValidation.errors);
    }

    // Validate bytecode size
    if (accountData.bytecodeSize !== undefined) {
      if (accountData.bytecodeSize <= 0) {
        errors.push('Bytecode size must be positive');
      }
      if (accountData.bytecodeSize > 10 * 1024 * 1024) { // 10MB limit
        errors.push('Bytecode size exceeds maximum limit');
      }
    }

    return {
      isValid: errors.length === 0,
      errors
    };
  }
}
