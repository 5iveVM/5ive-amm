#!/usr/bin/env node

import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';
import {
  deployFiveVmScript,
} from './lib/five-vm-deploy.mjs';
import {
  loadClusterConfig,
  deriveVmAddresses,
  resolveClusterFromEnvOrDefault,
} from './lib/vm-cluster-config.mjs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const tokenTemplateDir = path.join(__dirname, '..', 'five-templates', 'token');

const cluster = process.env.FIVE_VM_CLUSTER || resolveClusterFromEnvOrDefault();
const profile = loadClusterConfig({ cluster });
const derived = deriveVmAddresses(profile);

async function main() {
  const env = {
    rpcUrl: process.env.FIVE_RPC_URL || process.env.RPC_URL || 'http://127.0.0.1:8899',
    fiveProgramId: process.env.FIVE_PROGRAM_ID || profile.programId,
    vmStatePda: process.env.VM_STATE_PDA || derived.vmStatePda,
    keypairPath: process.env.FIVE_KEYPAIR_PATH || path.join(process.env.HOME, '.config', 'solana', 'id.json'),
    artifactPath: path.join(tokenTemplateDir, 'build', 'five-token-template.five'),
  };

  if (process.env.STRICT_VM_CONSTANTS === '1') {
    const parity = spawnSync('node', [
      path.join(__dirname, 'check-vm-constants-parity.mjs'),
      '--rpc-url', env.rpcUrl,
      '--program-id', env.fiveProgramId,
      '--vm-state', env.vmStatePda,
    ], { stdio: 'inherit' });
    if (parity.status !== 0) {
      process.exit(parity.status ?? 1);
    }
  }

  const result = await deployFiveVmScript({ ...env, label: 'token template' });
  console.log(`tokenScriptAccount=${result.scriptAccount}`);
  console.log(`fiveProgramId=${result.fiveProgramId}`);
  console.log(`vmStatePda=${result.vmStatePda}`);
}

main().catch((error) => {
  console.error(error.message || error);
  process.exit(1);
});
