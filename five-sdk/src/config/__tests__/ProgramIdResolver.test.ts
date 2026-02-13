/**
 * Tests for ProgramIdResolver - centralized program ID resolution
 */

import { describe, it, expect, beforeEach, afterEach } from '@jest/globals';
import { ProgramIdResolver } from '../ProgramIdResolver';

describe('ProgramIdResolver', () => {
  // Store original env for restoration
  const originalEnv = process.env.FIVE_PROGRAM_ID;

  beforeEach(() => {
    // Clear state before each test
    ProgramIdResolver.clearDefault();
    delete process.env.FIVE_PROGRAM_ID;
  });

  afterEach(() => {
    // Restore original state
    if (originalEnv) {
      process.env.FIVE_PROGRAM_ID = originalEnv;
    } else {
      delete process.env.FIVE_PROGRAM_ID;
    }
    ProgramIdResolver.clearDefault();
  });

  describe('Precedence Order (CRITICAL)', () => {
    const validProgramId = '11111111111111111111111111111112'; // System Program
    const validDefaultId = 'TokenkegQfeZyiNwAJsyFbPVwwQQnmjV7B8B65C7TnP'; // SPL Token
    const validEnvId = 'ATokenGPvbdGVqstVQmcLsNZAqeEbtQvvHta7h1Vvta'; // Associated Token

    it('explicit parameter takes precedence over all', () => {
      ProgramIdResolver.setDefault(validDefaultId);
      process.env.FIVE_PROGRAM_ID = validEnvId;

      const result = ProgramIdResolver.resolve(validProgramId);

      expect(result).toBe(validProgramId);
    });

    it('SDK default used when no explicit parameter', () => {
      ProgramIdResolver.setDefault(validDefaultId);

      const result = ProgramIdResolver.resolve();

      expect(result).toBe(validDefaultId);
    });

    it('environment variable used when no default or explicit', () => {
      process.env.FIVE_PROGRAM_ID = validEnvId;

      const result = ProgramIdResolver.resolve();

      expect(result).toBe(validEnvId);
    });

    it('throws error when no resolution possible', () => {
      ProgramIdResolver.clearDefault();
      delete process.env.FIVE_PROGRAM_ID;

      expect(() => ProgramIdResolver.resolve()).toThrow(/No program ID resolved/);
    });

    it('error message contains setup guidance', () => {
      ProgramIdResolver.clearDefault();
      delete process.env.FIVE_PROGRAM_ID;

      try {
        ProgramIdResolver.resolve();
        fail('Should have thrown');
      } catch (error: any) {
        expect(error.message).toContain('explicit call parameter');
        expect(error.message).toContain('FiveSDK.setDefaultProgramId()');
        expect(error.message).toContain('FIVE_PROGRAM_ID');
        expect(error.message).toContain('docs.five.build');
      }
    });
  });

  describe('Validation (CRITICAL)', () => {
    it('rejects invalid Solana pubkey format', () => {
      expect(() => {
        ProgramIdResolver.resolve('invalid_format_xyz');
      }).toThrow(/Invalid/);
    });

    it('rejects too-short base58 string', () => {
      expect(() => {
        ProgramIdResolver.resolve('11111');
      }).toThrow(/Invalid address length/);
    });

    it('rejects non-base58 characters', () => {
      expect(() => {
        ProgramIdResolver.resolve('OOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOO'); // O is invalid base58
      }).toThrow(/Invalid Base58/);
    });

    it('accepts valid Solana pubkey (32 byte System Program)', () => {
      const result = ProgramIdResolver.resolve('11111111111111111111111111111112');
      expect(result).toBe('11111111111111111111111111111112');
    });

    it('validates setDefault() input', () => {
      expect(() => {
        ProgramIdResolver.setDefault('invalid_format');
      }).toThrow();
    });

    it('accepts valid pubkey in setDefault()', () => {
      const validId = '11111111111111111111111111111112';
      ProgramIdResolver.setDefault(validId);
      expect(ProgramIdResolver.getDefault()).toBe(validId);
    });
  });

  describe('Optional Resolution', () => {
    const validTestId = '11111111111111111111111111111112'; // System Program
    const validDefaultId = 'TokenkegQfeZyiNwAJsyFbPVwwQQnmjV7B8B65C7TnP'; // SPL Token
    const validEnvId = 'ATokenGPvbdGVqstVQmcLsNZAqeEbtQvvHta7h1Vvta'; // Associated Token
    const validExplicitId = 'So11111111111111111111111111111111111111112'; // SOL token mint

    it('returns undefined when no resolution possible', () => {
      ProgramIdResolver.clearDefault();
      delete process.env.FIVE_PROGRAM_ID;

      const result = ProgramIdResolver.resolveOptional();

      expect(result).toBeUndefined();
    });

    it('returns resolved value if available', () => {
      ProgramIdResolver.setDefault(validTestId);

      const result = ProgramIdResolver.resolveOptional();

      expect(result).toBe(validTestId);
    });

    it('returns explicit value with highest priority', () => {
      ProgramIdResolver.setDefault(validDefaultId);
      process.env.FIVE_PROGRAM_ID = validEnvId;

      const result = ProgramIdResolver.resolveOptional(validExplicitId);

      expect(result).toBe(validExplicitId);
    });
  });

  describe('SDK Default Management', () => {
    it('getDefault returns undefined initially', () => {
      expect(ProgramIdResolver.getDefault()).toBeUndefined();
    });

    it('setDefault stores value', () => {
      const testId = '11111111111111111111111111111112';
      ProgramIdResolver.setDefault(testId);

      expect(ProgramIdResolver.getDefault()).toBe(testId);
    });

    it('clearDefault removes stored default', () => {
      ProgramIdResolver.setDefault('11111111111111111111111111111112');
      ProgramIdResolver.clearDefault();

      expect(ProgramIdResolver.getDefault()).toBeUndefined();
    });

    it('default persists across multiple resolve calls', () => {
      const testId = '11111111111111111111111111111112';
      ProgramIdResolver.setDefault(testId);

      const result1 = ProgramIdResolver.resolve();
      const result2 = ProgramIdResolver.resolve();

      expect(result1).toBe(testId);
      expect(result2).toBe(testId);
    });
  });

  describe('Environment Variable Integration', () => {
    const validSystemId = '11111111111111111111111111111112'; // System Program
    const validEnvId = 'TokenkegQfeZyiNwAJsyFbPVwwQQnmjV7B8B65C7TnP'; // SPL Token
    const validExplicitId = 'ATokenGPvbdGVqstVQmcLsNZAqeEbtQvvHta7h1Vvta'; // Associated Token

    it('respects FIVE_PROGRAM_ID env var', () => {
      process.env.FIVE_PROGRAM_ID = validSystemId;

      const result = ProgramIdResolver.resolve();

      expect(result).toBe(validSystemId);
    });

    it('env var overrides missing default', () => {
      process.env.FIVE_PROGRAM_ID = validEnvId;

      const result = ProgramIdResolver.resolve();

      expect(result).toBe(validEnvId);
    });

    it('explicit overrides env var', () => {
      process.env.FIVE_PROGRAM_ID = validEnvId;

      const result = ProgramIdResolver.resolve(validExplicitId);

      expect(result).toBe(validExplicitId);
    });
  });

  describe('Edge Cases', () => {
    const validDefaultId = 'TokenkegQfeZyiNwAJsyFbPVwwQQnmjV7B8B65C7TnP'; // SPL Token

    it('handles empty string explicitly', () => {
      // Empty string is falsy, should fall through to default
      ProgramIdResolver.setDefault(validDefaultId);

      const result = ProgramIdResolver.resolve('');

      expect(result).toBe(validDefaultId);
    });

    it('handles null explicitly', () => {
      ProgramIdResolver.setDefault(validDefaultId);

      const result = ProgramIdResolver.resolve(null as any);

      expect(result).toBe(validDefaultId);
    });

    it('handles undefined explicitly', () => {
      ProgramIdResolver.setDefault(validDefaultId);

      const result = ProgramIdResolver.resolve(undefined);

      expect(result).toBe(validDefaultId);
    });

    it('allows whitespace in validation error message', () => {
      try {
        ProgramIdResolver.resolve('  spaces  ');
        fail('Should throw');
      } catch (error: any) {
        expect(error.message).toBeDefined();
      }
    });
  });

  describe('Real Solana Program IDs', () => {
    const realProgramIds = [
      '11111111111111111111111111111112', // System Program
      'TokenkegQfeZyiNwAJsyFbPVwwQQnmjV7B8B65C7TnP', // SPL Token Program
      'ATokenGPvbdGVqstVQmcLsNZAqeEbtQvvHta7h1Vvta', // Associated Token Program
    ];

    realProgramIds.forEach((programId) => {
      it(`accepts real Solana program ID: ${programId.slice(0, 10)}...`, () => {
        const result = ProgramIdResolver.resolve(programId);
        expect(result).toBe(programId);
      });
    });
  });

  describe('Multiple Setups and Teardowns', () => {
    it('supports multiple setDefault calls', () => {
      const id1 = '11111111111111111111111111111112';
      const id2 = 'TokenkegQfeZyiNwAJsyFbPVwwQQnmjV7B8B65C7TnP';

      ProgramIdResolver.setDefault(id1);
      expect(ProgramIdResolver.resolve()).toBe(id1);

      ProgramIdResolver.setDefault(id2);
      expect(ProgramIdResolver.resolve()).toBe(id2);
    });

    it('can reset and reconfigure', () => {
      ProgramIdResolver.setDefault('11111111111111111111111111111112');
      ProgramIdResolver.clearDefault();
      delete process.env.FIVE_PROGRAM_ID;

      expect(() => ProgramIdResolver.resolve()).toThrow();

      ProgramIdResolver.setDefault('TokenkegQfeZyiNwAJsyFbPVwwQQnmjV7B8B65C7TnP');
      expect(ProgramIdResolver.resolve()).toBe('TokenkegQfeZyiNwAJsyFbPVwwQQnmjV7B8B65C7TnP');
    });
  });
});
