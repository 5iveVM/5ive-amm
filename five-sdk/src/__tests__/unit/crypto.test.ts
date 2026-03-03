/**
 * Five SDK Crypto Operations Unit Tests
 * 
 * REAL TESTING - NO MOCKS: Tests actual Solana crypto operations including PDA derivation,
 * base58 encoding, rent calculations, and address validation using real Solana libraries.
 * 
 * Adheres to CLAUDE.md Rule #3: NO MOCKS OR FAKE CODE - Tests must only pass with real implementations
 */

import { describe, it, expect, beforeEach } from '@jest/globals';
import { PublicKey } from '@solana/web3.js';
import bs58 from 'bs58';
import { 
  PDAUtils, 
  Base58Utils, 
  RentCalculator, 
  SolanaPublicKeyUtils,
  AccountValidator,
  HashUtils
} from '../../crypto/index.js';

describe('Five SDK Crypto Operations - Real Implementation Tests', () => {
  // Use real, valid Solana addresses for testing
  const FIVE_PROGRAM_ID = '7wVkyXsUiRcZtAHGZcXbTPCXnE7DBP6juN35H6FEUUZo';
  const VALID_BASE58_ADDRESS = '11111111111111111111111111111112';
  const ANOTHER_VALID_ADDRESS = 'TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA'; // SPL Token Program

  describe('PDAUtils - Real PDA Derivation', () => {
    describe('deriveScriptAccount', () => {
      it('should derive real PDA for script account with actual bytecode', async () => {
        const bytecode = new Uint8Array([0x46, 0x49, 0x56, 0x45, 0x01, 0x02, 0x03]); // "FIVE" + data
        
        const result = await PDAUtils.deriveScriptAccount(bytecode, VALID_BASE58_ADDRESS, FIVE_PROGRAM_ID);
        
        // Verify result structure
        expect(result).toHaveProperty('address');
        expect(result).toHaveProperty('bump');
        expect(typeof result.address).toBe('string');
        expect(typeof result.bump).toBe('number');
        
        // Verify address is valid base58
        expect(() => new PublicKey(result.address)).not.toThrow();
        
        // Verify bump is valid (0-255)
        expect(result.bump).toBeGreaterThanOrEqual(0);
        expect(result.bump).toBeLessThanOrEqual(255);
        
        // Verify deterministic - same bytecode should produce same result
        const result2 = await PDAUtils.deriveScriptAccount(bytecode, VALID_BASE58_ADDRESS, FIVE_PROGRAM_ID);
        expect(result.address).toBe(result2.address);
        expect(result.bump).toBe(result2.bump);
      });

      it('should produce different script accounts for different bytecode', async () => {
        const bytecode1 = new Uint8Array([1, 2, 3, 4, 5]);
        const bytecode2 = new Uint8Array([5, 4, 3, 2, 1]);
        
        const result1 = await PDAUtils.deriveScriptAccount(bytecode1, VALID_BASE58_ADDRESS, FIVE_PROGRAM_ID);
        const result2 = await PDAUtils.deriveScriptAccount(bytecode2, VALID_BASE58_ADDRESS, FIVE_PROGRAM_ID);
        
        // Different bytecode should produce different PDAs
        expect(result1.address).not.toBe(result2.address);
      });

      it('should produce different script accounts for different base public keys', async () => {
        const bytecode = new Uint8Array([7, 7, 7, 7, 7]);

        const result1 = await PDAUtils.deriveScriptAccount(bytecode, VALID_BASE58_ADDRESS, FIVE_PROGRAM_ID);
        const result2 = await PDAUtils.deriveScriptAccount(bytecode, ANOTHER_VALID_ADDRESS, FIVE_PROGRAM_ID);

        expect(result1.address).not.toBe(result2.address);
      });

      it('should match PublicKey.createWithSeed exactly', async () => {
        const bytecode = new Uint8Array([9, 8, 7, 6, 5, 4]);
        const result = await PDAUtils.deriveScriptAccount(bytecode, VALID_BASE58_ADDRESS, FIVE_PROGRAM_ID);

        const expected = await PublicKey.createWithSeed(
          new PublicKey(VALID_BASE58_ADDRESS),
          result.seed,
          new PublicKey(FIVE_PROGRAM_ID)
        );

        expect(result.address).toBe(expected.toBase58());
      });

      it('should handle empty bytecode gracefully', async () => {
        const bytecode = new Uint8Array(0);
        
        const result = await PDAUtils.deriveScriptAccount(bytecode, VALID_BASE58_ADDRESS, FIVE_PROGRAM_ID);
        
        expect(result).toHaveProperty('address');
        expect(result).toHaveProperty('bump');
        expect(() => new PublicKey(result.address)).not.toThrow();
      });
    });

    describe('deriveMetadataAccount', () => {
      it('should derive real metadata PDA for valid script account', async () => {
        const scriptAccount = VALID_BASE58_ADDRESS;
        
        const result = await PDAUtils.deriveMetadataAccount(scriptAccount, FIVE_PROGRAM_ID);
        
        expect(result).toHaveProperty('address');
        expect(result).toHaveProperty('bump');
        expect(() => new PublicKey(result.address)).not.toThrow();
        expect(result.bump).toBeGreaterThanOrEqual(0);
        expect(result.bump).toBeLessThanOrEqual(255);
      });

      it('should be deterministic for same script account', async () => {
        const scriptAccount = VALID_BASE58_ADDRESS;
        
        const result1 = await PDAUtils.deriveMetadataAccount(scriptAccount, FIVE_PROGRAM_ID);
        const result2 = await PDAUtils.deriveMetadataAccount(scriptAccount, FIVE_PROGRAM_ID);
        
        expect(result1.address).toBe(result2.address);
        expect(result1.bump).toBe(result2.bump);
      });
    });

    describe('deriveUserStateAccount', () => {
      it('should derive real user state PDA', async () => {
        const userPubkey = VALID_BASE58_ADDRESS;
        const scriptAccount = ANOTHER_VALID_ADDRESS;
        
        const result = await PDAUtils.deriveUserStateAccount(userPubkey, scriptAccount, FIVE_PROGRAM_ID);
        
        expect(result).toHaveProperty('address');
        expect(result).toHaveProperty('bump');
        expect(() => new PublicKey(result.address)).not.toThrow();
        expect(result.bump).toBeGreaterThanOrEqual(0);
        expect(result.bump).toBeLessThanOrEqual(255);
      });
    });

    describe('findProgramAddress', () => {
      it('should match Solana Web3 implementation for known edge cases (regression test)', async () => {
        // Known seed that caused a mismatch when isOffCurve was incorrect
        const seedString = 'test-seed-0';
        const programId = 'J99pDwVh1PqcxyBGKRvPKk8MUvW8V8KF6TmVEavKnzaF';

        const pdaUtilsResult = await PDAUtils.findProgramAddress(
          [Buffer.from(seedString)],
          programId
        );

        const [web3Address, web3Bump] = PublicKey.findProgramAddressSync(
          [Buffer.from(seedString)],
          new PublicKey(programId)
        );

        expect(pdaUtilsResult.address).toBe(web3Address.toBase58());
        expect(pdaUtilsResult.bump).toBe(web3Bump);
      });
    });
  });

  describe('Base58Utils - Real Base58 Operations', () => {
    describe('encode', () => {
      it('should encode data to valid base58', () => {
        const data = new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8]);
        
        const result = Base58Utils.encode(data);
        
        expect(typeof result).toBe('string');
        expect(result.length).toBeGreaterThan(0);
        
        // Verify it's valid base58 by decoding it back
        const decoded = bs58.decode(result);
        expect(new Uint8Array(decoded)).toEqual(data);
      });

      it('should handle empty data', () => {
        const data = new Uint8Array(0);
        const result = Base58Utils.encode(data);
        expect(result).toBe('');
      });

      it('should produce consistent results', () => {
        const data = new Uint8Array([42, 100, 200]);
        
        const result1 = Base58Utils.encode(data);
        const result2 = Base58Utils.encode(data);
        
        expect(result1).toBe(result2);
      });
    });

    describe('decode', () => {
      it('should decode valid base58 strings', () => {
        const originalData = new Uint8Array([1, 2, 3, 4, 5]);
        const encoded = bs58.encode(originalData);
        
        const result = Base58Utils.decode(encoded);
        
        expect(result).toEqual(originalData);
      });

      it('should handle empty string', () => {
        const result = Base58Utils.decode('');
        expect(result).toEqual(new Uint8Array(0));
      });

      it('should throw on invalid base58 characters', () => {
        expect(() => Base58Utils.decode('0OIl')).toThrow('Invalid base58');
      });
    });

    describe('isValid', () => {
      it('should return true for valid base58 strings', () => {
        const validBase58 = bs58.encode(new Uint8Array([1, 2, 3, 4]));
        const result = Base58Utils.isValid(validBase58);
        expect(result).toBe(true);
      });

      it('should return false for invalid base58 strings', () => {
        expect(Base58Utils.isValid('0OIl')).toBe(false);
        expect(Base58Utils.isValid('invalid+chars')).toBe(false);
        expect(Base58Utils.isValid('')).toBe(true); // Empty string is technically valid
      });
    });

    describe('random', () => {
      it('should generate random base58 strings of correct length', () => {
        const length = 8;
        const result = Base58Utils.random(length);
        
        expect(typeof result).toBe('string');
        expect(Base58Utils.isValid(result)).toBe(true);
        
        // Decode to verify byte length
        const decoded = Base58Utils.decode(result);
        expect(decoded.length).toBe(length);
      });

      it('should generate different results on consecutive calls', () => {
        const result1 = Base58Utils.random(4);
        const result2 = Base58Utils.random(4);
        
        // While theoretically possible to be equal, extremely unlikely
        expect(result1).not.toBe(result2);
      });
    });
  });

  describe('SolanaPublicKeyUtils - Real Solana Address Validation', () => {
    describe('isValid', () => {
      it('should validate real Solana addresses correctly', () => {
        expect(SolanaPublicKeyUtils.isValid(VALID_BASE58_ADDRESS)).toBe(true);
        expect(SolanaPublicKeyUtils.isValid(ANOTHER_VALID_ADDRESS)).toBe(true);
        expect(SolanaPublicKeyUtils.isValid(FIVE_PROGRAM_ID)).toBe(true);
      });

      it('should reject invalid addresses', () => {
        expect(SolanaPublicKeyUtils.isValid('invalid-address')).toBe(false);
        expect(SolanaPublicKeyUtils.isValid('0OIl')).toBe(false);
        expect(SolanaPublicKeyUtils.isValid('')).toBe(false);
        expect(SolanaPublicKeyUtils.isValid('too-long-to-be-a-valid-solana-address-because-it-exceeds-the-maximum-length')).toBe(false);
      });

      it('should handle edge cases', () => {
        // Real edge case testing with actual base58 validation
        expect(SolanaPublicKeyUtils.isValid('1'.repeat(44))).toBe(false); // Not valid base58/checksum
        expect(SolanaPublicKeyUtils.isValid('1'.repeat(43))).toBe(false); // Too short
        expect(SolanaPublicKeyUtils.isValid('1'.repeat(45))).toBe(false); // Too long
        
        // Test with valid length but invalid characters
        expect(SolanaPublicKeyUtils.isValid('0'.repeat(44))).toBe(false); // '0' not in base58
        expect(SolanaPublicKeyUtils.isValid('O'.repeat(44))).toBe(false); // 'O' not in base58
      });
    });

    describe('normalize', () => {
      it('should return valid addresses unchanged', () => {
        expect(SolanaPublicKeyUtils.normalize(VALID_BASE58_ADDRESS)).toBe(VALID_BASE58_ADDRESS);
        expect(SolanaPublicKeyUtils.normalize(ANOTHER_VALID_ADDRESS)).toBe(ANOTHER_VALID_ADDRESS);
      });

      it('should throw on invalid addresses', () => {
        expect(() => SolanaPublicKeyUtils.normalize('invalid')).toThrow('Invalid Solana address');
      });
    });
  });

  describe('RentCalculator - Real Solana Rent Calculations', () => {
    describe('calculateMinimumBalance', () => {
      it('should calculate realistic rent exemption amounts', async () => {
        const dataSize = 1000; // 1KB
        const result = await RentCalculator.calculateMinimumBalance(dataSize);
        
        expect(typeof result).toBe('number');
        expect(result).toBeGreaterThan(0);
        
        // Should be reasonable rent amount (not too high, not too low)
        expect(result).toBeGreaterThan(1000000); // > 0.001 SOL
        expect(result).toBeLessThan(100000000); // < 0.1 SOL
      });

      it('should scale with data size', async () => {
        const smallSize = 100;
        const largeSize = 10000;
        
        const smallRent = await RentCalculator.calculateMinimumBalance(smallSize);
        const largeRent = await RentCalculator.calculateMinimumBalance(largeSize);
        
        expect(largeRent).toBeGreaterThan(smallRent);
      });

      it('should handle zero size', async () => {
        const result = await RentCalculator.calculateMinimumBalance(0);
        expect(result).toBeGreaterThan(0); // Minimum rent even for 0 bytes
      });
    });

    describe('isRentExempt', () => {
      it('should correctly identify rent-exempt amounts', async () => {
        const dataSize = 500;
        const rentExemptAmount = await RentCalculator.calculateMinimumBalance(dataSize);
        
        expect(await RentCalculator.isRentExempt(rentExemptAmount, dataSize)).toBe(true);
        expect(await RentCalculator.isRentExempt(rentExemptAmount - 1, dataSize)).toBe(false);
        expect(await RentCalculator.isRentExempt(rentExemptAmount + 1000, dataSize)).toBe(true);
      });
    });
  });

  describe('HashUtils - Real Hash Operations', () => {
    describe('sha256', () => {
      it('should compute real SHA256 hashes', async () => {
        const data = new Uint8Array([1, 2, 3, 4, 5]);
        const result = await HashUtils.sha256(data);
        
        expect(result).toBeInstanceOf(Uint8Array);
        expect(result.length).toBe(32); // SHA256 produces 32 bytes
        
        // Verify deterministic
        const result2 = await HashUtils.sha256(data);
        expect(result).toEqual(result2);
      });

      it('should produce different hashes for different inputs', async () => {
        const data1 = new Uint8Array([1, 2, 3]);
        const data2 = new Uint8Array([3, 2, 1]);
        
        const hash1 = await HashUtils.sha256(data1);
        const hash2 = await HashUtils.sha256(data2);
        
        expect(hash1).not.toEqual(hash2);
      });

      it('should handle empty data', async () => {
        const result = await HashUtils.sha256(new Uint8Array(0));
        expect(result).toBeInstanceOf(Uint8Array);
        expect(result.length).toBe(32);
      });
    });

    describe('createSeed', () => {
      it('should create deterministic seeds from inputs', async () => {
        const inputs = ['test', 'seed', 'data'];
        const result = await HashUtils.createSeed(inputs);
        
        expect(result).toBeInstanceOf(Uint8Array);
        expect(result.length).toBe(32);
        
        // Verify deterministic
        const result2 = await HashUtils.createSeed(inputs);
        expect(result).toEqual(result2);
      });

      it('should produce different seeds for different inputs', async () => {
        const inputs1 = ['test', 'seed'];
        const inputs2 = ['test', 'different'];
        
        const seed1 = await HashUtils.createSeed(inputs1);
        const seed2 = await HashUtils.createSeed(inputs2);
        
        expect(seed1).not.toEqual(seed2);
      });
    });

    describe('randomBytes', () => {
      it('should generate random bytes of correct length', async () => {
        const length = 16;
        const result = await HashUtils.randomBytes(length);
        
        expect(result).toBeInstanceOf(Uint8Array);
        expect(result.length).toBe(length);
      });

      it('should generate different results on consecutive calls', async () => {
        const result1 = await HashUtils.randomBytes(8);
        const result2 = await HashUtils.randomBytes(8);
        
        // Extremely unlikely to be equal
        expect(result1).not.toEqual(result2);
      });
    });
  });

  describe('AccountValidator - Real Account Validation', () => {
    describe('validateAddress', () => {
      it('should validate real Solana addresses', () => {
        const result = AccountValidator.validateAddress(VALID_BASE58_ADDRESS);
        
        expect(result.valid).toBe(true);
        expect(result.errors).toHaveLength(0);
        expect(result.normalizedAddress).toBe(VALID_BASE58_ADDRESS);
      });

      it('should reject invalid addresses with detailed errors', () => {
        const result = AccountValidator.validateAddress('invalid-address');
        
        expect(result.valid).toBe(false);
        expect(result.errors.length).toBeGreaterThan(0);
        expect(result.errors[0]).toContain('Invalid Solana address');
        expect(result.normalizedAddress).toBeNull();
      });
    });

    describe('validateAccountList', () => {
      it('should validate lists of real addresses', () => {
        const addresses = [VALID_BASE58_ADDRESS, ANOTHER_VALID_ADDRESS, FIVE_PROGRAM_ID];
        const result = AccountValidator.validateAccountList(addresses);
        
        expect(result.valid).toBe(true);
        expect(result.errors).toHaveLength(0);
        expect(result.validAddresses).toEqual(addresses);
        expect(result.invalidAddresses).toHaveLength(0);
      });

      it('should identify mixed valid/invalid addresses', () => {
        const addresses = [VALID_BASE58_ADDRESS, 'invalid', ANOTHER_VALID_ADDRESS, 'also-invalid'];
        const result = AccountValidator.validateAccountList(addresses);
        
        expect(result.valid).toBe(false);
        expect(result.errors.length).toBeGreaterThan(0);
        expect(result.validAddresses).toEqual([VALID_BASE58_ADDRESS, ANOTHER_VALID_ADDRESS]);
        expect(result.invalidAddresses).toEqual(['invalid', 'also-invalid']);
      });
    });
  });
});
