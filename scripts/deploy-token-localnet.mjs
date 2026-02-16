#!/usr/bin/env node

/**
 * Deploy token template to localnet using the robust chunked deploy flow.
 * This avoids transaction-size limits from single-transaction deploys.
 */

import path from 'path';
import { fileURLToPath } from 'url';
import { spawnSync } from 'child_process';
import { loadClusterConfig, deriveVmAddresses, resolveClusterFromEnvOrDefault } from './lib/vm-cluster-config.mjs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const tokenTemplateDir = path.join(__dirname, '..', 'five-templates', 'token');
const deployScript = path.join(tokenTemplateDir, 'deploy-to-five-vm.mjs');

const cluster = process.env.FIVE_VM_CLUSTER || resolveClusterFromEnvOrDefault();
const profile = loadClusterConfig({ cluster });
const derived = deriveVmAddresses(profile);

const env = {
  ...process.env,
  RPC_URL: process.env.RPC_URL || 'http://127.0.0.1:8899',
  FIVE_PROGRAM_ID: process.env.FIVE_PROGRAM_ID || profile.programId,
  VM_STATE_PDA: process.env.VM_STATE_PDA || derived.vmStatePda,
};

if (process.env.STRICT_VM_CONSTANTS === '1') {
  const parity = spawnSync('node', [
    path.join(__dirname, 'check-vm-constants-parity.mjs'),
    '--rpc-url', env.RPC_URL,
    '--program-id', env.FIVE_PROGRAM_ID,
    '--vm-state', env.VM_STATE_PDA,
  ], { stdio: 'inherit' });
  if (parity.status !== 0) {
    process.exit(parity.status ?? 1);
  }
}

const result = spawnSync('node', [deployScript, 'Token'], {
  cwd: tokenTemplateDir,
  env,
  stdio: 'inherit',
});

if (result.status !== 0) {
  process.exit(result.status ?? 1);
}
