import fs from 'fs';
import path from 'path';
import { PublicKey } from '@solana/web3.js';

const VM_STATE_SEED = Buffer.from('vm_state', 'utf-8');
const FEE_VAULT_SEED = Buffer.from([
  0xff, 0x66, 0x69, 0x76, 0x65, 0x5f, 0x76, 0x6d, 0x5f, 0x66, 0x65, 0x65,
  0x5f, 0x76, 0x61, 0x75, 0x6c, 0x74, 0x5f, 0x76, 0x31,
]);
const VALID_CLUSTERS = new Set(['localnet', 'devnet', 'mainnet']);

function resolveDefaultConfigPath(): string {
  if (process.env.FIVE_VM_CONSTANTS_CONFIG) {
    return path.resolve(process.env.FIVE_VM_CONSTANTS_CONFIG);
  }

  const candidates = [
    path.resolve(process.cwd(), 'five-solana/constants.vm.toml'),
    path.resolve(process.cwd(), '../five-solana/constants.vm.toml'),
  ];

  for (const candidate of candidates) {
    if (fs.existsSync(candidate)) {
      return candidate;
    }
  }

  return candidates[0];
}

export type VmCluster = 'localnet' | 'devnet' | 'mainnet';
export type CliTarget = 'local' | 'devnet' | 'testnet' | 'mainnet' | 'wasm';

export interface VmClusterProfile {
  cluster: VmCluster;
  configPath: string;
  programId: string;
  feeVaultShardCount: number;
}

function parseSimpleVmToml(raw: string): { clusters: Record<string, any> } {
  const clusters: Record<string, any> = {};
  let current: string | null = null;
  for (const lineRaw of raw.split('\n')) {
    const line = lineRaw.trim();
    if (!line || line.startsWith('#')) continue;
    const sec = line.match(/^\[clusters\.(localnet|devnet|mainnet)\]$/);
    if (sec) {
      current = sec[1];
      clusters[current] = {};
      continue;
    }
    if (!current) continue;
    const kv = line.match(/^([a-z_]+)\s*=\s*(.+)$/);
    if (!kv) continue;
    const key = kv[1];
    const rawVal = kv[2].trim();
    if (rawVal.startsWith('"') && rawVal.endsWith('"')) {
      clusters[current][key] = rawVal.slice(1, -1);
    } else if (/^\d+$/.test(rawVal)) {
      clusters[current][key] = Number(rawVal);
    } else {
      throw new Error(`Unsupported TOML value: ${line}`);
    }
  }
  return { clusters };
}

export class VmClusterConfigResolver {
  static fromCliTarget(target: CliTarget): VmCluster {
    if (target === 'local') return 'localnet';
    if (target === 'devnet') return 'devnet';
    if (target === 'mainnet') return 'mainnet';
    throw new Error(`Target ${target} has no VM cluster mapping (supported: local|devnet|mainnet)`);
  }

  static resolveClusterFromEnvOrDefault(): VmCluster {
    const cluster = (process.env.FIVE_VM_CLUSTER || 'localnet').trim();
    if (!VALID_CLUSTERS.has(cluster)) {
      throw new Error(`Invalid FIVE_VM_CLUSTER: ${cluster} (expected localnet|devnet|mainnet)`);
    }
    return cluster as VmCluster;
  }

  static getDefaultConfigPath(): string {
    return resolveDefaultConfigPath();
  }

  static loadClusterConfig(input: { cluster?: string; configPath?: string } = {}): VmClusterProfile {
    const cluster = (input.cluster || this.resolveClusterFromEnvOrDefault()).trim();
    if (!VALID_CLUSTERS.has(cluster)) throw new Error(`Invalid cluster: ${cluster}`);
    const configPath = path.resolve(input.configPath || this.getDefaultConfigPath());
    if (!fs.existsSync(configPath)) throw new Error(`VM constants config not found: ${configPath}`);
    const parsed = parseSimpleVmToml(fs.readFileSync(configPath, 'utf-8'));
    const entry = parsed.clusters?.[cluster];
    if (!entry) throw new Error(`Cluster missing in VM constants config: ${cluster}`);
    if (!entry.program_id) throw new Error(`Missing program_id for cluster ${cluster}`);
    if (!Number.isInteger(entry.fee_vault_shard_count) || entry.fee_vault_shard_count < 1) {
      throw new Error(`Invalid fee_vault_shard_count for cluster ${cluster}`);
    }
    return {
      cluster: cluster as VmCluster,
      configPath,
      programId: new PublicKey(entry.program_id).toBase58(),
      feeVaultShardCount: entry.fee_vault_shard_count,
    };
  }

  static deriveVmAddresses(profile: VmClusterProfile) {
    const programPk = new PublicKey(profile.programId);
    const [vmStatePda, vmStateBump] = PublicKey.findProgramAddressSync([VM_STATE_SEED], programPk);
    const feeVaultPdas: Array<{ shardIndex: number; address: string; bump: number }> = [];
    for (let i = 0; i < profile.feeVaultShardCount; i++) {
      const [vault, bump] = PublicKey.findProgramAddressSync([FEE_VAULT_SEED, Buffer.from([i])], programPk);
      feeVaultPdas.push({ shardIndex: i, address: vault.toBase58(), bump });
    }
    return {
      cluster: profile.cluster,
      programId: profile.programId,
      feeVaultShardCount: profile.feeVaultShardCount,
      vmStatePda: vmStatePda.toBase58(),
      vmStateBump,
      feeVaultPdas,
    };
  }
}
