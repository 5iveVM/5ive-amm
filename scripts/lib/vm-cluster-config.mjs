import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import web3 from '../../five-cli/node_modules/@solana/web3.js/lib/index.cjs.js';

const { PublicKey } = web3;
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const VM_STATE_SEED = Buffer.from('vm_state', 'utf-8');
const FEE_VAULT_SEED = Buffer.from([
  0xff, 0x66, 0x69, 0x76, 0x65, 0x5f, 0x76, 0x6d, 0x5f, 0x66, 0x65, 0x65,
  0x5f, 0x76, 0x61, 0x75, 0x6c, 0x74, 0x5f, 0x76, 0x31,
]);

const CLUSTERS = new Set(['localnet', 'devnet', 'mainnet']);

export function resolveClusterFromEnvOrDefault() {
  const raw = (process.env.FIVE_VM_CLUSTER || 'localnet').trim();
  if (!CLUSTERS.has(raw)) {
    throw new Error(`Invalid FIVE_VM_CLUSTER: ${raw} (expected localnet|devnet|mainnet)`);
  }
  return raw;
}

function parseSimpleVmToml(raw) {
  const clusters = {};
  let current = null;
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

export function loadClusterConfig({ cluster, configPath } = {}) {
  const resolvedCluster = cluster || resolveClusterFromEnvOrDefault();
  if (!CLUSTERS.has(resolvedCluster)) {
    throw new Error(`Invalid cluster: ${resolvedCluster}`);
  }
  const cfgPath = path.resolve(
    configPath || process.env.FIVE_VM_CONSTANTS_CONFIG || path.join(__dirname, '..', '..', 'five-solana', 'constants.vm.toml'),
  );
  const raw = fs.readFileSync(cfgPath, 'utf-8');
  const parsed = parseSimpleVmToml(raw);
  const entry = parsed.clusters?.[resolvedCluster];
  if (!entry) throw new Error(`Cluster missing in config: ${resolvedCluster}`);
  if (!entry.program_id) throw new Error(`Missing program_id for cluster: ${resolvedCluster}`);
  if (!Number.isInteger(entry.fee_vault_shard_count) || entry.fee_vault_shard_count < 1) {
    throw new Error(`Invalid fee_vault_shard_count for cluster: ${resolvedCluster}`);
  }
  const programId = new PublicKey(entry.program_id);
  return {
    cluster: resolvedCluster,
    configPath: cfgPath,
    programId: programId.toBase58(),
    feeVaultShardCount: entry.fee_vault_shard_count,
  };
}

export function deriveVmAddresses(profile) {
  const programId = new PublicKey(profile.programId);
  const [vmState, vmStateBump] = PublicKey.findProgramAddressSync([VM_STATE_SEED], programId);
  const feeVaultPdas = [];
  for (let i = 0; i < profile.feeVaultShardCount; i++) {
    const [vault, bump] = PublicKey.findProgramAddressSync([FEE_VAULT_SEED, Buffer.from([i])], programId);
    feeVaultPdas.push({ shardIndex: i, address: vault.toBase58(), bump });
  }
  return {
    cluster: profile.cluster,
    programId: profile.programId,
    feeVaultShardCount: profile.feeVaultShardCount,
    vmStatePda: vmState.toBase58(),
    vmStateBump,
    feeVaultPdas,
  };
}

