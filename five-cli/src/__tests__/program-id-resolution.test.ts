/**
 * Program ID Resolution Integration Tests (strict cluster-config mode)
 */

import { ConfigManager } from '../config/ConfigManager.js';
import { VmClusterConfigResolver } from '../config/VmClusterConfigResolver.js';
import { mkdtemp } from 'fs/promises';
import { join } from 'path';
import { tmpdir } from 'os';

describe('Program ID Resolution - CLI Integration', () => {
  const originalXdgConfigHome = process.env.XDG_CONFIG_HOME;

  const resetConfigManagerSingleton = () => {
    (ConfigManager as any).instance = undefined;
  };

  beforeEach(async () => {
    const isolatedConfigHome = await mkdtemp(join(tmpdir(), 'five-cli-config-'));
    process.env.XDG_CONFIG_HOME = isolatedConfigHome;
    resetConfigManagerSingleton();
  });

  afterEach(() => {
    resetConfigManagerSingleton();
  });

  afterAll(() => {
    if (originalXdgConfigHome !== undefined) {
      process.env.XDG_CONFIG_HOME = originalXdgConfigHome;
    } else {
      delete process.env.XDG_CONFIG_HOME;
    }
    resetConfigManagerSingleton();
  });

  it('resolves from vm constants when no CLI/config override is present', async () => {
    const manager = ConfigManager.getInstance();
    const cfg = await manager.applyOverrides({ target: 'devnet' as any });
    const expected = VmClusterConfigResolver.loadClusterConfig({
      cluster: VmClusterConfigResolver.fromCliTarget(cfg.target as any),
    }).programId;

    const configured = await manager.getProgramId(cfg.target as any);
    const resolved = configured || expected;
    expect(resolved).toBe(expected);
  });

  it('uses per-target config override when set', async () => {
    const manager = ConfigManager.getInstance();
    const override = 'TokenkegQfeZyiNwAJsyFbPVwwQQnmjV7B8B65C7TnP';

    await manager.setProgramId(override, 'devnet');
    const cfg = await manager.applyOverrides({ target: 'devnet' as any });
    const configured = await manager.getProgramId(cfg.target as any);
    expect(configured).toBe(override);
  });

  it('uses explicit CLI override over all defaults', async () => {
    const manager = ConfigManager.getInstance();
    const configId = 'ATokenGPvbdGVqstVQmcLsNZAqeEbtQvvHta7h1Vvta';
    const cliId = '11111111111111111111111111111112';

    await manager.setProgramId(configId, 'devnet');
    const cfg = await manager.applyOverrides({ target: 'devnet' as any });
    const configured = await manager.getProgramId(cfg.target as any);
    const resolved = cliId || configured || VmClusterConfigResolver.loadClusterConfig({
      cluster: VmClusterConfigResolver.fromCliTarget(cfg.target as any),
    }).programId;

    expect(resolved).toBe(cliId);
  });

  it('throws for unsupported target mapping', () => {
    expect(() => VmClusterConfigResolver.fromCliTarget('testnet' as any)).toThrow();
    expect(() => VmClusterConfigResolver.fromCliTarget('wasm' as any)).toThrow();
  });
});
