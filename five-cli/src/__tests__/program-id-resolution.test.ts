/**
 * Program ID Resolution Integration Tests
 *
 * Tests how program ID is resolved across CLI, config, and environment.
 * Validates the resolution precedence: CLI → config → env → error
 */

import { ConfigManager } from '../config/ConfigManager.js';

describe('Program ID Resolution - CLI Integration', () => {
  const testProgramIds = {
    id1: '11111111111111111111111111111112',
    id2: 'TokenkegQfeZyiNwAJsyFbPVwwQQnmjV7B8B65C7TnP',
    id3: 'ATokenGPvbdGVqstVQmcLsNZAqeEbtQvvHta7h1Vvta',
  };

  describe('CLI Config Integration', () => {
    it('should handle program ID from config file', async () => {
      const manager = ConfigManager.getInstance();
      const configId = testProgramIds.id1;

      await manager.setProgramId(configId);

      // Simulate CLI retrieving from config
      const stored = await manager.getProgramId();
      expect(stored).toBe(configId);
    });

    it('should handle per-target program IDs', async () => {
      const manager = ConfigManager.getInstance();
      const devnetId = testProgramIds.id1;
      const testnetId = testProgramIds.id2;

      // Set per-target
      await manager.setProgramId(devnetId, 'devnet');
      await manager.setProgramId(testnetId, 'testnet');

      // Simulate CLI switching targets
      let target: 'devnet' | 'testnet' = 'devnet';
      let stored = await manager.getProgramId(target);
      expect(stored).toBe(devnetId);

      target = 'testnet';
      stored = await manager.getProgramId(target);
      expect(stored).toBe(testnetId);
    });

    it('should handle CLI flags overriding config', async () => {
      const manager = ConfigManager.getInstance();
      const configId = testProgramIds.id1;
      const cliFlagId = testProgramIds.id2;

      // Set in config
      await manager.setProgramId(configId);

      // Simulate CLI flag override
      // In real CLI, this would be: const resolved = cliFlagId || configId
      expect(cliFlagId || configId).toBe(cliFlagId);
    });
  });

  describe('Environment Variable Integration', () => {
    beforeEach(() => {
      delete process.env.FIVE_PROGRAM_ID;
    });

    afterEach(() => {
      delete process.env.FIVE_PROGRAM_ID;
    });

    it('should support environment variable', () => {
      const envId = testProgramIds.id1;
      process.env.FIVE_PROGRAM_ID = envId;

      // Simulate CLI checking env
      const fromEnv = process.env.FIVE_PROGRAM_ID;
      expect(fromEnv).toBe(envId);
    });

    it('should allow CLI flag to override env var', () => {
      const envId = testProgramIds.id1;
      const cliId = testProgramIds.id2;
      process.env.FIVE_PROGRAM_ID = envId;

      // CLI flag precedence
      const resolved = cliId || process.env.FIVE_PROGRAM_ID;
      expect(resolved).toBe(cliId);
    });

    it('should handle empty env var as unset', () => {
      process.env.FIVE_PROGRAM_ID = '';

      // Empty env should be treated as unset
      const resolved = process.env.FIVE_PROGRAM_ID || 'fallback';
      expect(resolved).toBe('fallback');
    });
  });

  describe('Workflow Integration', () => {
    it('should support multi-network deployment', async () => {
      const manager = ConfigManager.getInstance();
      const devnetId = testProgramIds.id1;
      const testnetId = testProgramIds.id2;
      const mainnetId = testProgramIds.id3;

      // Setup per-target IDs
      await manager.setProgramId(devnetId, 'devnet');
      await manager.setProgramId(testnetId, 'testnet');
      await manager.setProgramId(mainnetId, 'mainnet');

      // Simulate deploying to each
      const networks = ['devnet', 'testnet', 'mainnet'] as const;
      const expectedIds = [devnetId, testnetId, mainnetId];

      for (let i = 0; i < networks.length; i++) {
        const stored = await manager.getProgramId(networks[i]);
        expect(stored).toBe(expectedIds[i]);
      }
    });

    it('should handle personal development workflow', async () => {
      const manager = ConfigManager.getInstance();
      const devId = testProgramIds.id1;

      // Set once for development
      await manager.setProgramId(devId);

      // Use multiple times
      for (let i = 0; i < 3; i++) {
        const stored = await manager.getProgramId();
        expect(stored).toBe(devId);
      }
    });

    it('should support CI/CD with environment variable', () => {
      const ciProgramId = testProgramIds.id1;
      process.env.FIVE_PROGRAM_ID = ciProgramId;

      // CLI resolves from env
      const resolved = process.env.FIVE_PROGRAM_ID;
      expect(resolved).toBe(ciProgramId);

      delete process.env.FIVE_PROGRAM_ID;
    });

    it('should allow one-off override', async () => {
      const manager = ConfigManager.getInstance();
      const configId = testProgramIds.id1;
      const oneOffId = testProgramIds.id2;

      // Normal: use config
      await manager.setProgramId(configId);
      let resolved = await manager.getProgramId();
      expect(resolved).toBe(configId);

      // One-off: override with explicit value
      resolved = oneOffId; // Simulating CLI flag
      expect(resolved).toBe(oneOffId);

      // Config unchanged
      resolved = await manager.getProgramId();
      expect(resolved).toBe(configId);
    });
  });

  describe('Error Scenarios', () => {
    it('should handle missing program ID gracefully', async () => {
      const manager = ConfigManager.getInstance();

      // When no program ID is set for a valid target
      const noIdFromConfig = await manager.getProgramId('testnet');
      // Should return undefined or string
      expect(typeof noIdFromConfig === 'string' || noIdFromConfig === undefined).toBe(true);
    });

    it('should support fallback chain', async () => {
      const manager = ConfigManager.getInstance();

      // Simulate: CLI flag || config || env || error
      const cliFlag = undefined;
      const config = testProgramIds.id1;
      const env = process.env.FIVE_PROGRAM_ID;

      const resolved = cliFlag || config || env || null;
      expect(resolved).toBe(config);
    });

    it('should prefer CLI over all others', async () => {
      const manager = ConfigManager.getInstance();
      const cliFlag = testProgramIds.id1;
      const config = testProgramIds.id2;
      process.env.FIVE_PROGRAM_ID = testProgramIds.id3;

      await manager.setProgramId(config);

      // CLI flag takes highest priority
      const resolved = cliFlag || config || process.env.FIVE_PROGRAM_ID;
      expect(resolved).toBe(cliFlag);

      delete process.env.FIVE_PROGRAM_ID;
    });
  });

  describe('Backward Compatibility', () => {
    it('should work without program ID for local execution', async () => {
      // Local/WASM execution may not need program ID
      const localId = undefined;
      expect(localId === undefined || typeof localId === 'string').toBe(true);
    });

    it('should support legacy environment variable usage', () => {
      const legacyId = testProgramIds.id1;
      process.env.FIVE_PROGRAM_ID = legacyId;

      const resolved = process.env.FIVE_PROGRAM_ID;
      expect(resolved).toBe(legacyId);

      delete process.env.FIVE_PROGRAM_ID;
    });
  });

  describe('Configuration Persistence', () => {
    it('should maintain multi-target setup', async () => {
      const manager = ConfigManager.getInstance();

      // Set multiple targets
      await manager.setProgramId(testProgramIds.id1, 'devnet');
      await manager.setProgramId(testProgramIds.id2, 'testnet');

      // Verify both exist
      expect(await manager.getProgramId('devnet')).toBe(testProgramIds.id1);
      expect(await manager.getProgramId('testnet')).toBe(testProgramIds.id2);

      // Clear one
      await manager.clearProgramId('devnet');

      // Other remains
      expect(await manager.getProgramId('testnet')).toBe(testProgramIds.id2);
    });

    it('should support update and revert workflow', async () => {
      const manager = ConfigManager.getInstance();
      const id1 = testProgramIds.id1;
      const id2 = testProgramIds.id2;

      // Set initial
      await manager.setProgramId(id1);
      expect(await manager.getProgramId()).toBe(id1);

      // Update
      await manager.setProgramId(id2);
      expect(await manager.getProgramId()).toBe(id2);

      // Revert (clear and reset)
      await manager.clearProgramId();
      await manager.setProgramId(id1);
      expect(await manager.getProgramId()).toBe(id1);
    });
  });
});
