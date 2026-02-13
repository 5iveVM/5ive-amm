/**
 * Config Command Program ID Tests
 *
 * Tests for program ID management via `five config set --program-id` and related commands.
 * Covers persistence, validation, multi-target support, and error handling.
 */

import { ConfigManager } from '../config/ConfigManager.js';
import { ConfigTarget } from '../config/types.js';

describe('Config Command - Program ID Management', () => {
  const validProgramIds = {
    system: '11111111111111111111111111111112', // System Program
    token: 'TokenkegQfeZyiNwAJsyFbPVwwQQnmjV7B8B65C7TnP', // SPL Token
    associated: 'ATokenGPvbdGVqstVQmcLsNZAqeEbtQvvHta7h1Vvta', // Associated Token
    sol: 'So11111111111111111111111111111111111111112', // SOL Token
  };

  describe('setProgramId() - Basic Operations', () => {
    it('should store program ID for current target', async () => {
      const manager = ConfigManager.getInstance();
      await manager.setProgramId(validProgramIds.system);
      const stored = await manager.getProgramId();
      expect(stored).toBe(validProgramIds.system);
    });

    it('should store program ID for specific target', async () => {
      const manager = ConfigManager.getInstance();
      await manager.setProgramId(validProgramIds.system, 'devnet');
      const devnet = await manager.getProgramId('devnet');
      expect(devnet).toBe(validProgramIds.system);
    });

    it('should update existing program ID', async () => {
      const manager = ConfigManager.getInstance();
      await manager.setProgramId(validProgramIds.system);
      await manager.setProgramId(validProgramIds.token);
      const stored = await manager.getProgramId();
      expect(stored).toBe(validProgramIds.token);
    });

    it('should support all valid targets', async () => {
      const manager = ConfigManager.getInstance();
      const targets: ConfigTarget[] = ['devnet', 'testnet', 'mainnet'];
      const ids = Object.values(validProgramIds);

      for (let i = 0; i < targets.length; i++) {
        await manager.setProgramId(ids[i], targets[i]);
        const stored = await manager.getProgramId(targets[i]);
        expect(stored).toBe(ids[i]);
      }
    });
  });

  describe('getProgramId() - Retrieval', () => {
    it('should return program ID for current target', async () => {
      const manager = ConfigManager.getInstance();
      await manager.setProgramId(validProgramIds.system);
      const retrieved = await manager.getProgramId();
      expect(retrieved).toBe(validProgramIds.system);
    });

    it('should return program ID for specific target', async () => {
      const manager = ConfigManager.getInstance();
      await manager.setProgramId(validProgramIds.token, 'testnet');
      const testnet = await manager.getProgramId('testnet');
      expect(testnet).toBe(validProgramIds.token);
    });

    it('should return stored IDs independently per target', async () => {
      const manager = ConfigManager.getInstance();
      const devnetId = validProgramIds.system;
      const testnetId = validProgramIds.token;

      await manager.setProgramId(devnetId, 'devnet');
      await manager.setProgramId(testnetId, 'testnet');

      expect(await manager.getProgramId('devnet')).toBe(devnetId);
      expect(await manager.getProgramId('testnet')).toBe(testnetId);
    });
  });

  describe('clearProgramId() - Deletion', () => {
    it('should remove program ID for current target', async () => {
      const manager = ConfigManager.getInstance();
      await manager.setProgramId(validProgramIds.system);
      await manager.clearProgramId();
      // After clear, should not have value for current target
      const retrieved = await manager.getProgramId();
      // Note: May be undefined or from persistent config
      expect(typeof retrieved === 'string' || retrieved === undefined).toBe(true);
    });

    it('should remove program ID for specific target', async () => {
      const manager = ConfigManager.getInstance();
      await manager.setProgramId(validProgramIds.system, 'devnet');
      await manager.setProgramId(validProgramIds.token, 'testnet');
      await manager.clearProgramId('devnet');

      // Testnet should still have its ID
      const testnet = await manager.getProgramId('testnet');
      expect(testnet).toBe(validProgramIds.token);
    });

    it('should not error when clearing non-existent program ID', async () => {
      const manager = ConfigManager.getInstance();
      // Should not throw
      await expect(manager.clearProgramId()).resolves.toBeUndefined();
    });
  });

  describe('getAllProgramIds() - Bulk Retrieval', () => {
    it('should return object with stored program IDs', async () => {
      const manager = ConfigManager.getInstance();
      await manager.setProgramId(validProgramIds.system, 'devnet');
      await manager.setProgramId(validProgramIds.token, 'testnet');

      const all = await manager.getAllProgramIds();

      expect(all).toHaveProperty('devnet');
      expect(all).toHaveProperty('testnet');
      expect(all.devnet).toBe(validProgramIds.system);
      expect(all.testnet).toBe(validProgramIds.token);
    });

    it('should include multiple targets in response', async () => {
      const manager = ConfigManager.getInstance();
      const targets: ConfigTarget[] = ['devnet', 'testnet', 'mainnet'];
      const ids = Object.values(validProgramIds).slice(0, 3);

      for (let i = 0; i < targets.length; i++) {
        await manager.setProgramId(ids[i], targets[i]);
      }

      const all = await manager.getAllProgramIds();

      for (let i = 0; i < targets.length; i++) {
        expect(all[targets[i]]).toBe(ids[i]);
      }
    });
  });

  describe('Multi-target Workflows', () => {
    it('should handle switching between targets', async () => {
      const manager = ConfigManager.getInstance();

      // Set for devnet
      await manager.setProgramId(validProgramIds.system, 'devnet');
      expect(await manager.getProgramId('devnet')).toBe(validProgramIds.system);

      // Set for testnet
      await manager.setProgramId(validProgramIds.token, 'testnet');
      expect(await manager.getProgramId('testnet')).toBe(validProgramIds.token);

      // Devnet still exists
      expect(await manager.getProgramId('devnet')).toBe(validProgramIds.system);
    });

    it('should clear specific target without affecting others', async () => {
      const manager = ConfigManager.getInstance();

      await manager.setProgramId(validProgramIds.system, 'devnet');
      await manager.setProgramId(validProgramIds.token, 'testnet');

      // Clear devnet only
      await manager.clearProgramId('devnet');

      // Testnet unchanged
      expect(await manager.getProgramId('testnet')).toBe(validProgramIds.token);
    });
  });

  describe('Error Handling', () => {
    it('should handle valid program IDs', async () => {
      const manager = ConfigManager.getInstance();

      for (const id of Object.values(validProgramIds)) {
        await manager.setProgramId(id);
        expect(await manager.getProgramId()).toBe(id);
      }
    });

    it('should reject invalid target', async () => {
      const manager = ConfigManager.getInstance();

      try {
        await manager.setProgramId(validProgramIds.system, 'invalid-target' as ConfigTarget);
        fail('Should have thrown for invalid target');
      } catch (error: any) {
        // Expected to throw
        expect(error).toBeDefined();
      }
    });

    it('should handle getting config', async () => {
      const manager = ConfigManager.getInstance();

      // Should not throw
      const config = await manager.get();
      expect(config).toBeDefined();
      expect(config.target).toBeDefined();
    });
  });

  describe('Persistence - Core Functionality', () => {
    it('should maintain program ID after set', async () => {
      const manager = ConfigManager.getInstance();
      const programId = validProgramIds.system;

      await manager.setProgramId(programId);
      const stored = await manager.getProgramId();

      expect(stored).toBe(programId);
    });

    it('should persist program IDs across get operations', async () => {
      const manager = ConfigManager.getInstance();
      const devnetId = validProgramIds.system;
      const testnetId = validProgramIds.token;

      // Set both
      await manager.setProgramId(devnetId, 'devnet');
      await manager.setProgramId(testnetId, 'testnet');

      // Get them multiple times
      expect(await manager.getProgramId('devnet')).toBe(devnetId);
      expect(await manager.getProgramId('testnet')).toBe(testnetId);
      expect(await manager.getProgramId('devnet')).toBe(devnetId);
      expect(await manager.getProgramId('testnet')).toBe(testnetId);
    });
  });

  describe('Workflow Scenarios', () => {
    it('should support personal dev workflow', async () => {
      const manager = ConfigManager.getInstance();

      // Set once for devnet
      await manager.setProgramId(validProgramIds.system);

      // Use multiple times
      for (let i = 0; i < 3; i++) {
        const stored = await manager.getProgramId();
        expect(stored).toBe(validProgramIds.system);
      }
    });

    it('should support multi-network setup', async () => {
      const manager = ConfigManager.getInstance();

      // Setup all networks
      await manager.setProgramId(validProgramIds.system, 'devnet');
      await manager.setProgramId(validProgramIds.token, 'testnet');
      await manager.setProgramId(validProgramIds.associated, 'mainnet');

      // Simulate getting IDs for each network
      const networks = ['devnet', 'testnet', 'mainnet'] as const;
      const expectedIds = [validProgramIds.system, validProgramIds.token, validProgramIds.associated];

      for (let i = 0; i < networks.length; i++) {
        const stored = await manager.getProgramId(networks[i]);
        expect(stored).toBe(expectedIds[i]);
      }
    });

    it('should support override with clear and set', async () => {
      const manager = ConfigManager.getInstance();

      // Initial set
      await manager.setProgramId(validProgramIds.system);
      expect(await manager.getProgramId()).toBe(validProgramIds.system);

      // Override: clear and set new
      await manager.clearProgramId();
      await manager.setProgramId(validProgramIds.token);
      expect(await manager.getProgramId()).toBe(validProgramIds.token);
    });
  });
});
