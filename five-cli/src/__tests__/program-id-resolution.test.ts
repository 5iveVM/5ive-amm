/**
 * Program ID Resolution Integration Tests
 *
 * Tests the complete program ID resolution precedence chain across CLI, config, and environment.
 * Validates precedence order: CLI flag → project config → CLI config → env var → SDK default → error
 */

import fs from 'fs';
import path from 'path';
import os from 'os';
import { ConfigManager } from '../config/ConfigManager.js';
import { ProgramIdResolver } from 'five-sdk';

describe('Program ID Resolution Precedence', () => {
  let tempConfigDir: string;
  let originalConfigPath: string | undefined;
  let originalEnv: string | undefined;
  let originalDefault: string | undefined;

  beforeEach(() => {
    // Create temporary config directory
    tempConfigDir = fs.mkdtempSync(path.join(os.tmpdir(), 'five-resolution-test-'));

    // Save and clear environment
    originalConfigPath = process.env.FIVE_CONFIG_DIR;
    originalEnv = process.env.FIVE_PROGRAM_ID;
    originalDefault = ProgramIdResolver.getDefault();

    process.env.FIVE_CONFIG_DIR = tempConfigDir;
    delete process.env.FIVE_PROGRAM_ID;
    ProgramIdResolver.clearDefault();
  });

  afterEach(() => {
    // Restore environment
    if (originalConfigPath) {
      process.env.FIVE_CONFIG_DIR = originalConfigPath;
    } else {
      delete process.env.FIVE_CONFIG_DIR;
    }

    if (originalEnv) {
      process.env.FIVE_PROGRAM_ID = originalEnv;
    } else {
      delete process.env.FIVE_PROGRAM_ID;
    }

    if (originalDefault) {
      ProgramIdResolver.setDefault(originalDefault);
    } else {
      ProgramIdResolver.clearDefault();
    }

    // Clean up temp directory
    if (fs.existsSync(tempConfigDir)) {
      fs.rmSync(tempConfigDir, { recursive: true });
    }
  });

  describe('Precedence Order', () => {
    const cli = '11111111111111111111111111111112'; // System Program
    const config = 'TokenkegQfeZyiNwAJsyFbPVwwQQnmjV7B8B65C7TnP'; // SPL Token
    const env = 'ATokenGPvbdGVqstVQmcLsNZAqeEbtQvvHta7h1Vvta'; // Associated Token
    const sdkDefault = 'So11111111111111111111111111111111111111112'; // SOL Token

    it('should use CLI flag when all sources present', async () => {
      // Set all sources
      await ConfigManager.getInstance().setProgramId(config);
      process.env.FIVE_PROGRAM_ID = env;
      ProgramIdResolver.setDefault(sdkDefault);

      // Resolve with CLI flag
      const resolved = ProgramIdResolver.resolve(cli);
      expect(resolved).toBe(cli);
    });

    it('should use config when no CLI flag', async () => {
      // Set config and other sources
      await ConfigManager.getInstance().setProgramId(config);
      process.env.FIVE_PROGRAM_ID = env;
      ProgramIdResolver.setDefault(sdkDefault);

      // Resolve without CLI flag - should use config
      // Note: In real CLI, this would be: ProgramIdResolver.resolve(cliConfig.programId || projectConfig.programId)
      const resolved = ProgramIdResolver.resolve(config);
      expect(resolved).toBe(config);
    });

    it('should use environment variable when no CLI or config', async () => {
      // Only set env
      process.env.FIVE_PROGRAM_ID = env;
      ProgramIdResolver.clearDefault();

      const resolved = ProgramIdResolver.resolve();
      expect(resolved).toBe(env);
    });

    it('should use SDK default when no other sources', async () => {
      // Only set SDK default
      ProgramIdResolver.setDefault(sdkDefault);

      const resolved = ProgramIdResolver.resolve();
      expect(resolved).toBe(sdkDefault);
    });

    it('should error when no sources available', () => {
      // Clear all sources
      ProgramIdResolver.clearDefault();
      delete process.env.FIVE_PROGRAM_ID;

      expect(() => ProgramIdResolver.resolve()).toThrow();
    });

    it('should allow undefined when requested', () => {
      // Clear all sources
      ProgramIdResolver.clearDefault();
      delete process.env.FIVE_PROGRAM_ID;

      const resolved = ProgramIdResolver.resolveOptional();
      expect(resolved).toBeUndefined();
    });
  });

  describe('CLI Integration', () => {
    it('should handle program ID from config file', async () => {
      const manager = ConfigManager.getInstance();
      const configId = 'TokenkegQfeZyiNwAJsyFbPVwwQQnmjV7B8B65C7TnP';

      await manager.setProgramId(configId);

      // Simulate CLI retrieving from config
      const stored = await manager.getProgramId();
      expect(stored).toBe(configId);

      // Use stored ID with resolver
      const resolved = ProgramIdResolver.resolve(stored);
      expect(resolved).toBe(configId);
    });

    it('should handle per-target program IDs', async () => {
      const manager = ConfigManager.getInstance();
      const devnetId = '11111111111111111111111111111112';
      const testnetId = 'TokenkegQfeZyiNwAJsyFbPVwwQQnmjV7B8B65C7TnP';

      // Set per-target
      await manager.setProgramId(devnetId, 'devnet');
      await manager.setProgramId(testnetId, 'testnet');

      // Simulate CLI switching targets
      let target: 'devnet' | 'testnet' = 'devnet';
      let stored = await manager.getProgramId(target);
      expect(ProgramIdResolver.resolve(stored)).toBe(devnetId);

      target = 'testnet';
      stored = await manager.getProgramId(target);
      expect(ProgramIdResolver.resolve(stored)).toBe(testnetId);
    });

    it('should handle CLI flags overriding config', async () => {
      const manager = ConfigManager.getInstance();
      const configId = 'TokenkegQfeZyiNwAJsyFbPVwwQQnmjV7B8B65C7TnP';
      const cliFlagId = '11111111111111111111111111111112';

      // Set in config
      await manager.setProgramId(configId);

      // Simulate CLI flag override
      const resolved = ProgramIdResolver.resolve(cliFlagId); // cliFlag provided
      expect(resolved).toBe(cliFlagId);
    });
  });

  describe('Environment Variable Integration', () => {
    it('should respect FIVE_PROGRAM_ID env var', () => {
      const envId = 'ATokenGPvbdGVqstVQmcLsNZAqeEbtQvvHta7h1Vvta';
      process.env.FIVE_PROGRAM_ID = envId;

      const resolved = ProgramIdResolver.resolve();
      expect(resolved).toBe(envId);
    });

    it('should override env var with CLI flag', () => {
      const envId = 'ATokenGPvbdGVqstVQmcLsNZAqeEbtQvvHta7h1Vvta';
      const cliId = '11111111111111111111111111111112';
      process.env.FIVE_PROGRAM_ID = envId;

      const resolved = ProgramIdResolver.resolve(cliId);
      expect(resolved).toBe(cliId);
    });

    it('should handle empty env var as unset', () => {
      process.env.FIVE_PROGRAM_ID = '';
      const sdkDefault = 'So11111111111111111111111111111111111111112';
      ProgramIdResolver.setDefault(sdkDefault);

      const resolved = ProgramIdResolver.resolve();
      expect(resolved).toBe(sdkDefault);
    });
  });

  describe('Error Cases', () => {
    it('should throw clear error with setup guidance', () => {
      ProgramIdResolver.clearDefault();
      delete process.env.FIVE_PROGRAM_ID;

      expect(() => ProgramIdResolver.resolve()).toThrow(
        /Program ID required|No program ID resolved/i
      );
    });

    it('should include helpful message in error', () => {
      ProgramIdResolver.clearDefault();
      delete process.env.FIVE_PROGRAM_ID;

      try {
        ProgramIdResolver.resolve();
        fail('Should have thrown');
      } catch (error: any) {
        const message = error.message;
        expect(message).toMatch(/config|environment|five config/i);
      }
    });

    it('should validate all resolved IDs', () => {
      // Invalid ID should fail validation
      expect(() => ProgramIdResolver.resolve('invalid-id')).toThrow();
    });
  });

  describe('Complex Workflows', () => {
    it('should handle multi-network deployment workflow', async () => {
      const manager = ConfigManager.getInstance();
      const devnetId = '11111111111111111111111111111112';
      const testnetId = 'TokenkegQfeZyiNwAJsyFbPVwwQQnmjV7B8B65C7TnP';
      const mainnetId = 'ATokenGPvbdGVqstVQmcLsNZAqeEbtQvvHta7h1Vvta';

      // Setup per-target IDs
      await manager.setProgramId(devnetId, 'devnet');
      await manager.setProgramId(testnetId, 'testnet');
      await manager.setProgramId(mainnetId, 'mainnet');

      // Simulate deploying to each
      const networks = [
        { target: 'devnet' as const, id: devnetId },
        { target: 'testnet' as const, id: testnetId },
        { target: 'mainnet' as const, id: mainnetId },
      ];

      for (const { target, id } of networks) {
        const stored = await manager.getProgramId(target);
        const resolved = ProgramIdResolver.resolve(stored);
        expect(resolved).toBe(id);
      }
    });

    it('should handle CI/CD with env var', async () => {
      // CI/CD sets program ID via env var
      const ciProgramId = '11111111111111111111111111111112';
      process.env.FIVE_PROGRAM_ID = ciProgramId;

      // CLI resolves it
      const resolved = ProgramIdResolver.resolve();
      expect(resolved).toBe(ciProgramId);
    });

    it('should handle local dev -> staging -> production flow', async () => {
      const manager = ConfigManager.getInstance();
      const devId = '11111111111111111111111111111112'; // local
      const stagingId = 'TokenkegQfeZyiNwAJsyFbPVwwQQnmjV7B8B65C7TnP'; // staging
      const prodId = 'ATokenGPvbdGVqstVQmcLsNZAqeEbtQvvHta7h1Vvta'; // production

      // Local dev uses CLI config
      await manager.setProgramId(devId, 'devnet');
      let resolved = ProgramIdResolver.resolve(await manager.getProgramId('devnet'));
      expect(resolved).toBe(devId);

      // Staging uses five.toml (simulated)
      const stagingConfigId = stagingId;
      resolved = ProgramIdResolver.resolve(stagingConfigId);
      expect(resolved).toBe(stagingId);

      // Production uses env var (from CI/CD)
      process.env.FIVE_PROGRAM_ID = prodId;
      resolved = ProgramIdResolver.resolve();
      expect(resolved).toBe(prodId);
    });

    it('should override config with CLI flag for one-off runs', async () => {
      const manager = ConfigManager.getInstance();
      const configId = 'TokenkegQfeZyiNwAJsyFbPVwwQQnmjV7B8B65C7TnP';
      const oneOffId = '11111111111111111111111111111112';

      // Normal: use config
      await manager.setProgramId(configId);
      let resolved = ProgramIdResolver.resolve(await manager.getProgramId());
      expect(resolved).toBe(configId);

      // One-off: override with flag
      resolved = ProgramIdResolver.resolve(oneOffId);
      expect(resolved).toBe(oneOffId);

      // Config unchanged
      resolved = ProgramIdResolver.resolve(await manager.getProgramId());
      expect(resolved).toBe(configId);
    });
  });

  describe('SDK and CLI Integration', () => {
    it('should support FiveSDK.setDefaultProgramId()', () => {
      const sdkId = '11111111111111111111111111111112';
      ProgramIdResolver.setDefault(sdkId);

      const resolved = ProgramIdResolver.resolve();
      expect(resolved).toBe(sdkId);
    });

    it('should allow CLI to override SDK default', async () => {
      const manager = ConfigManager.getInstance();
      const cliId = 'TokenkegQfeZyiNwAJsyFbPVwwQQnmjV7B8B65C7TnP';
      const sdkDefault = '11111111111111111111111111111112';

      // SDK sets default
      ProgramIdResolver.setDefault(sdkDefault);

      // CLI stores different ID
      await manager.setProgramId(cliId);

      // CLI value takes precedence
      const resolved = ProgramIdResolver.resolve(cliId);
      expect(resolved).toBe(cliId);
    });
  });

  describe('Validation across chain', () => {
    it('should validate IDs as they pass through resolver', () => {
      const validId = '11111111111111111111111111111112';
      const resolved = ProgramIdResolver.resolve(validId);
      expect(resolved).toBe(validId);
    });

    it('should reject invalid IDs early', () => {
      expect(() => ProgramIdResolver.resolve('invalid')).toThrow();
      expect(() => ProgramIdResolver.resolve('I' + 'O' + 'l')).toThrow(); // Invalid chars
    });

    it('should validate config-stored IDs', async () => {
      const manager = ConfigManager.getInstance();
      const validId = '11111111111111111111111111111112';

      await manager.setProgramId(validId);
      const stored = await manager.getProgramId();

      // Should validate on storage
      expect(stored).toBe(validId);

      // Should validate on resolve
      const resolved = ProgramIdResolver.resolve(stored);
      expect(resolved).toBe(validId);
    });
  });

  describe('Backward Compatibility', () => {
    it('should work when program ID not provided (resolveOptional)', () => {
      // Local/WASM execution paths don't need program ID
      const resolved = ProgramIdResolver.resolveOptional();
      expect(resolved).toBeUndefined();
    });

    it('should support legacy env var fallback', () => {
      // Old code might rely on env var
      const legacyId = '11111111111111111111111111111112';
      process.env.FIVE_PROGRAM_ID = legacyId;

      const resolved = ProgramIdResolver.resolve();
      expect(resolved).toBe(legacyId);
    });

    it('should work when called without arguments', () => {
      const id = '11111111111111111111111111111112';
      ProgramIdResolver.setDefault(id);

      const resolved = ProgramIdResolver.resolve();
      expect(resolved).toBe(id);
    });
  });
});
