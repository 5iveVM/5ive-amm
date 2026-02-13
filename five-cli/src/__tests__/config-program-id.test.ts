/**
 * Config Command Program ID Tests
 *
 * Tests for program ID management via `five config set --program-id` and related commands.
 * Covers persistence, validation, multi-target support, and error handling.
 */

import fs from 'fs';
import path from 'path';
import os from 'os';
import { ConfigManager } from '../config/ConfigManager.js';
import { FiveConfig, ConfigTarget } from '../config/types.js';

describe('Config Command - Program ID Management', () => {
  let tempConfigDir: string;
  let originalConfigPath: string | undefined;

  beforeEach(() => {
    // Create temporary config directory for tests
    tempConfigDir = fs.mkdtempSync(path.join(os.tmpdir(), 'five-config-test-'));

    // Save original config path if it exists
    originalConfigPath = process.env.FIVE_CONFIG_DIR;
    process.env.FIVE_CONFIG_DIR = tempConfigDir;
  });

  afterEach(() => {
    // Restore original config path
    if (originalConfigPath) {
      process.env.FIVE_CONFIG_DIR = originalConfigPath;
    } else {
      delete process.env.FIVE_CONFIG_DIR;
    }

    // Clean up temp directory
    if (fs.existsSync(tempConfigDir)) {
      fs.rmSync(tempConfigDir, { recursive: true });
    }
  });

  describe('setProgramId()', () => {
    it('should store program ID for current target', async () => {
      const manager = ConfigManager.getInstance();
      const programId = '11111111111111111111111111111112'; // System Program

      await manager.setProgramId(programId);

      const retrieved = await manager.getProgramId();
      expect(retrieved).toBe(programId);
    });

    it('should store program ID for specific target', async () => {
      const manager = ConfigManager.getInstance();
      const devnetId = '11111111111111111111111111111112';
      const testnetId = 'TokenkegQfeZyiNwAJsyFbPVwwQQnmjV7B8B65C7TnP';

      await manager.setProgramId(devnetId, 'devnet');
      await manager.setProgramId(testnetId, 'testnet');

      const devnet = await manager.getProgramId('devnet');
      const testnet = await manager.getProgramId('testnet');

      expect(devnet).toBe(devnetId);
      expect(testnet).toBe(testnetId);
    });

    it('should persist program ID across instances', async () => {
      const manager1 = ConfigManager.getInstance();
      const programId = 'ATokenGPvbdGVqstVQmcLsNZAqeEbtQvvHta7h1Vvta';

      await manager1.setProgramId(programId);

      // Create new instance to verify persistence
      const manager2 = ConfigManager.getInstance();
      const retrieved = await manager2.getProgramId();

      expect(retrieved).toBe(programId);
    });

    it('should validate Solana base58 format', async () => {
      const manager = ConfigManager.getInstance();

      // Invalid formats should be rejected
      const invalidIds = [
        'invalid-id',
        'toolongexampleidthatwillexceed44charactersincluding0OIl',
        '0OIl123456789', // Contains invalid characters
        'short', // Too short
      ];

      for (const invalidId of invalidIds) {
        await expect(manager.setProgramId(invalidId)).rejects.toThrow();
      }
    });

    it('should update existing program ID', async () => {
      const manager = ConfigManager.getInstance();
      const id1 = '11111111111111111111111111111112';
      const id2 = 'TokenkegQfeZyiNwAJsyFbPVwwQQnmjV7B8B65C7TnP';

      await manager.setProgramId(id1);
      let retrieved = await manager.getProgramId();
      expect(retrieved).toBe(id1);

      await manager.setProgramId(id2);
      retrieved = await manager.getProgramId();
      expect(retrieved).toBe(id2);
    });

    it('should support all valid targets', async () => {
      const manager = ConfigManager.getInstance();
      const targets: ConfigTarget[] = ['wasm', 'local', 'devnet', 'testnet', 'mainnet'];
      const programIds: Record<string, string> = {
        wasm: '11111111111111111111111111111112',
        local: 'TokenkegQfeZyiNwAJsyFbPVwwQQnmjV7B8B65C7TnP',
        devnet: 'ATokenGPvbdGVqstVQmcLsNZAqeEbtQvvHta7h1Vvta',
        testnet: 'So11111111111111111111111111111111111111112',
        mainnet: 'AjJVHdYu7ASTWCDoNiZtNrEY2wnELYsZNf5s2pHJQPdt',
      };

      for (const target of targets) {
        await manager.setProgramId(programIds[target], target);
        const retrieved = await manager.getProgramId(target);
        expect(retrieved).toBe(programIds[target]);
      }
    });
  });

  describe('getProgramId()', () => {
    it('should return undefined when not set', async () => {
      const manager = ConfigManager.getInstance();
      const result = await manager.getProgramId();
      expect(result).toBeUndefined();
    });

    it('should return program ID for current target', async () => {
      const manager = ConfigManager.getInstance();
      const programId = '11111111111111111111111111111112';

      await manager.setProgramId(programId);
      const retrieved = await manager.getProgramId();

      expect(retrieved).toBe(programId);
    });

    it('should return program ID for specific target', async () => {
      const manager = ConfigManager.getInstance();
      const devnetId = '11111111111111111111111111111112';
      const testnetId = 'TokenkegQfeZyiNwAJsyFbPVwwQQnmjV7B8B65C7TnP';

      await manager.setProgramId(devnetId, 'devnet');
      await manager.setProgramId(testnetId, 'testnet');

      const devnet = await manager.getProgramId('devnet');
      const testnet = await manager.getProgramId('testnet');

      expect(devnet).toBe(devnetId);
      expect(testnet).toBe(testnetId);
    });

    it('should return undefined for target with no program ID', async () => {
      const manager = ConfigManager.getInstance();
      const result = await manager.getProgramId('testnet');
      expect(result).toBeUndefined();
    });
  });

  describe('clearProgramId()', () => {
    it('should remove program ID for current target', async () => {
      const manager = ConfigManager.getInstance();
      const programId = '11111111111111111111111111111112';

      await manager.setProgramId(programId);
      expect(await manager.getProgramId()).toBe(programId);

      await manager.clearProgramId();
      expect(await manager.getProgramId()).toBeUndefined();
    });

    it('should remove program ID for specific target', async () => {
      const manager = ConfigManager.getInstance();
      const devnetId = '11111111111111111111111111111112';
      const testnetId = 'TokenkegQfeZyiNwAJsyFbPVwwQQnmjV7B8B65C7TnP';

      await manager.setProgramId(devnetId, 'devnet');
      await manager.setProgramId(testnetId, 'testnet');

      await manager.clearProgramId('devnet');

      expect(await manager.getProgramId('devnet')).toBeUndefined();
      expect(await manager.getProgramId('testnet')).toBe(testnetId);
    });

    it('should not error when clearing non-existent program ID', async () => {
      const manager = ConfigManager.getInstance();
      await expect(manager.clearProgramId()).resolves.toBeUndefined();
    });

    it('should persist after clear', async () => {
      const manager1 = ConfigManager.getInstance();
      const programId = '11111111111111111111111111111112';

      await manager1.setProgramId(programId);
      await manager1.clearProgramId();

      const manager2 = ConfigManager.getInstance();
      const retrieved = await manager2.getProgramId();
      expect(retrieved).toBeUndefined();
    });
  });

  describe('getAllProgramIds()', () => {
    it('should return empty object when none set', async () => {
      const manager = ConfigManager.getInstance();
      const all = await manager.getAllProgramIds();

      expect(all).toEqual({});
    });

    it('should return all stored program IDs', async () => {
      const manager = ConfigManager.getInstance();
      const devnetId = '11111111111111111111111111111112';
      const testnetId = 'TokenkegQfeZyiNwAJsyFbPVwwQQnmjV7B8B65C7TnP';
      const mainnetId = 'ATokenGPvbdGVqstVQmcLsNZAqeEbtQvvHta7h1Vvta';

      await manager.setProgramId(devnetId, 'devnet');
      await manager.setProgramId(testnetId, 'testnet');
      await manager.setProgramId(mainnetId, 'mainnet');

      const all = await manager.getAllProgramIds();

      expect(all).toEqual({
        devnet: devnetId,
        testnet: testnetId,
        mainnet: mainnetId,
      });
    });

    it('should only include targets with program IDs', async () => {
      const manager = ConfigManager.getInstance();
      const devnetId = '11111111111111111111111111111112';

      await manager.setProgramId(devnetId, 'devnet');

      const all = await manager.getAllProgramIds();

      expect(all).toEqual({
        devnet: devnetId,
      });
      expect(all.testnet).toBeUndefined();
      expect(all.mainnet).toBeUndefined();
    });

    it('should persist across instances', async () => {
      const manager1 = ConfigManager.getInstance();
      const devnetId = '11111111111111111111111111111112';
      const testnetId = 'TokenkegQfeZyiNwAJsyFbPVwwQQnmjV7B8B65C7TnP';

      await manager1.setProgramId(devnetId, 'devnet');
      await manager1.setProgramId(testnetId, 'testnet');

      const manager2 = ConfigManager.getInstance();
      const all = await manager2.getAllProgramIds();

      expect(all).toEqual({
        devnet: devnetId,
        testnet: testnetId,
      });
    });
  });

  describe('Multi-target workflows', () => {
    it('should handle setting different IDs per target', async () => {
      const manager = ConfigManager.getInstance();
      const ids = {
        devnet: '11111111111111111111111111111112',
        testnet: 'TokenkegQfeZyiNwAJsyFbPVwwQQnmjV7B8B65C7TnP',
        mainnet: 'ATokenGPvbdGVqstVQmcLsNZAqeEbtQvvHta7h1Vvta',
      };

      // Set all
      for (const [target, id] of Object.entries(ids)) {
        await manager.setProgramId(id, target as ConfigTarget);
      }

      // Verify all independently
      for (const [target, id] of Object.entries(ids)) {
        const retrieved = await manager.getProgramId(target as ConfigTarget);
        expect(retrieved).toBe(id);
      }

      // Verify all together
      const all = await manager.getAllProgramIds();
      expect(all).toEqual(ids);
    });

    it('should allow switching between targets', async () => {
      const manager = ConfigManager.getInstance();
      const devnetId = '11111111111111111111111111111112';
      const testnetId = 'TokenkegQfeZyiNwAJsyFbPVwwQQnmjV7B8B65C7TnP';

      // Set devnet
      await manager.setProgramId(devnetId, 'devnet');
      let current = await manager.getProgramId('devnet');
      expect(current).toBe(devnetId);

      // Set testnet
      await manager.setProgramId(testnetId, 'testnet');
      current = await manager.getProgramId('testnet');
      expect(current).toBe(testnetId);

      // Devnet still exists
      const devnet = await manager.getProgramId('devnet');
      expect(devnet).toBe(devnetId);
    });

    it('should handle clearing specific target without affecting others', async () => {
      const manager = ConfigManager.getInstance();
      const devnetId = '11111111111111111111111111111112';
      const testnetId = 'TokenkegQfeZyiNwAJsyFbPVwwQQnmjV7B8B65C7TnP';

      await manager.setProgramId(devnetId, 'devnet');
      await manager.setProgramId(testnetId, 'testnet');

      // Clear devnet
      await manager.clearProgramId('devnet');

      // Devnet cleared
      expect(await manager.getProgramId('devnet')).toBeUndefined();

      // Testnet unchanged
      expect(await manager.getProgramId('testnet')).toBe(testnetId);
    });
  });

  describe('Config file persistence', () => {
    it('should save to config file', async () => {
      const manager = ConfigManager.getInstance();
      const programId = '11111111111111111111111111111112';

      await manager.setProgramId(programId);

      // Check config file exists and contains program ID
      const configPath = path.join(tempConfigDir, 'config.json');
      expect(fs.existsSync(configPath)).toBe(true);

      const configData = JSON.parse(fs.readFileSync(configPath, 'utf-8'));
      expect(configData.programIds).toBeDefined();
      expect(configData.programIds.devnet).toBe(programId);
    });

    it('should load from saved config file', async () => {
      const manager1 = ConfigManager.getInstance();
      const programId = '11111111111111111111111111111112';

      await manager1.setProgramId(programId, 'devnet');

      // Create new instance (simulates new CLI invocation)
      const manager2 = ConfigManager.getInstance();
      const retrieved = await manager2.getProgramId('devnet');

      expect(retrieved).toBe(programId);
    });

    it('should preserve other config fields when updating program ID', async () => {
      const manager = ConfigManager.getInstance();
      const config = await manager.get();

      // Modify another field
      config.showConfig = true;

      // Set program ID
      await manager.setProgramId('11111111111111111111111111111112');

      // Verify other field preserved
      const updated = await manager.get();
      expect(updated.showConfig).toBe(true);
    });
  });

  describe('Error handling', () => {
    it('should reject invalid Solana pubkey', async () => {
      const manager = ConfigManager.getInstance();

      await expect(manager.setProgramId('not-a-valid-key')).rejects.toThrow();
      await expect(manager.setProgramId('00000000000000000000000000000000')).rejects.toThrow();
      await expect(manager.setProgramId('I0l')).rejects.toThrow(); // Invalid chars
    });

    it('should reject invalid target', async () => {
      const manager = ConfigManager.getInstance();
      const validId = '11111111111111111111111111111112';

      await expect(manager.setProgramId(validId, 'invalid-target' as ConfigTarget)).rejects.toThrow();
      await expect(manager.getProgramId('invalid-target' as ConfigTarget)).rejects.toThrow();
      await expect(manager.clearProgramId('invalid-target' as ConfigTarget)).rejects.toThrow();
    });

    it('should handle missing config directory gracefully', async () => {
      const manager = ConfigManager.getInstance();

      // Should create directory if needed
      await manager.setProgramId('11111111111111111111111111111112');

      const configPath = path.join(tempConfigDir, 'config.json');
      expect(fs.existsSync(configPath)).toBe(true);
    });
  });
});
