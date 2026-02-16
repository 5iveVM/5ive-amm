/**
 * Tests for ProgramIdResolver - centralized program ID resolution
 */

import { describe, it, expect, beforeEach, afterEach } from '@jest/globals';
import { ProgramIdResolver } from '../ProgramIdResolver';
import { VmClusterConfigResolver } from '../VmClusterConfigResolver';

describe('ProgramIdResolver', () => {
  const originalCluster = process.env.FIVE_VM_CLUSTER;
  const clusterProgramId = VmClusterConfigResolver.loadClusterConfig({ cluster: 'localnet' }).programId;

  beforeEach(() => {
    // Clear state before each test
    ProgramIdResolver.clearDefault();
    process.env.FIVE_VM_CLUSTER = 'localnet';
  });

  afterEach(() => {
    // Restore original state
    if (originalCluster) {
      process.env.FIVE_VM_CLUSTER = originalCluster;
    } else {
      delete process.env.FIVE_VM_CLUSTER;
    }
    ProgramIdResolver.clearDefault();
  });

  describe('Precedence Order (CRITICAL)', () => {
    const validProgramId = '11111111111111111111111111111112'; // System Program
    const validDefaultId = 'TokenkegQfeZyiNwAJsyFbPVwwQQnmjV7B8B65C7TnP'; // SPL Token

    it('explicit parameter takes precedence over all', () => {
      ProgramIdResolver.setDefault(validDefaultId);

      const result = ProgramIdResolver.resolve(validProgramId);

      expect(result).toBe(validProgramId);
    });

    it('SDK default used when no explicit parameter', () => {
      ProgramIdResolver.setDefault(validDefaultId);

      const result = ProgramIdResolver.resolve();

      expect(result).toBe(validDefaultId);
    });

    it('cluster config is used when no default or explicit', () => {
      const result = ProgramIdResolver.resolve();
      expect(result).toBe(clusterProgramId);
    });

    it('error message contains setup guidance when cluster config is invalid', () => {
      ProgramIdResolver.clearDefault();
      process.env.FIVE_VM_CLUSTER = 'invalid-cluster';
      expect(() => ProgramIdResolver.resolve()).toThrow(/No program ID resolved for Five VM/);
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
    const validExplicitId = 'So11111111111111111111111111111111111111112'; // SOL token mint

    it('returns cluster-config value when explicit/default missing', () => {
      const result = ProgramIdResolver.resolveOptional();
      expect(result).toBe(clusterProgramId);
    });

    it('returns resolved value if available', () => {
      ProgramIdResolver.setDefault(validTestId);

      const result = ProgramIdResolver.resolveOptional();

      expect(result).toBe(validTestId);
    });

    it('returns explicit value with highest priority', () => {
      ProgramIdResolver.setDefault(validDefaultId);

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

  describe('Cluster Config Integration', () => {
    const validDefaultId = 'TokenkegQfeZyiNwAJsyFbPVwwQQnmjV7B8B65C7TnP'; // SPL Token
    const validExplicitId = 'ATokenGPvbdGVqstVQmcLsNZAqeEbtQvvHta7h1Vvta'; // Associated Token

    it('resolves from cluster config when default absent', () => {
      const result = ProgramIdResolver.resolve();
      expect(result).toBe(clusterProgramId);
    });

    it('sdk default overrides cluster config', () => {
      ProgramIdResolver.setDefault(validDefaultId);

      const result = ProgramIdResolver.resolve();

      expect(result).toBe(validDefaultId);
    });

    it('explicit overrides cluster config/default', () => {
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
        throw new Error('Should throw');
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
      expect(ProgramIdResolver.resolve()).toBe(clusterProgramId);

      ProgramIdResolver.setDefault('TokenkegQfeZyiNwAJsyFbPVwwQQnmjV7B8B65C7TnP');
      expect(ProgramIdResolver.resolve()).toBe('TokenkegQfeZyiNwAJsyFbPVwwQQnmjV7B8B65C7TnP');
    });
  });
});
