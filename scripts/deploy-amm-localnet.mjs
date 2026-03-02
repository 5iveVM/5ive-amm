#!/usr/bin/env node

import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';
import { loadClusterConfig, deriveVmAddresses } from './lib/vm-cluster-config.mjs';
import { deployFiveVmScript, loadExplicitDeployEnv } from './lib/five-vm-deploy.mjs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const CYAN = '\x1b[36m';
const GREEN = '\x1b[32m';
const RED = '\x1b[31m';
const NC = '\x1b[0m';

const cluster = process.env.FIVE_VM_CLUSTER || 'localnet';
const profile = loadClusterConfig({ cluster });
const derived = deriveVmAddresses(profile);

async function main() {
  const env = loadExplicitDeployEnv(path.join(__dirname, '..', '5ive-amm', 'build', '5ive-amm.five'));

  console.log(`${CYAN}═══════════════════════════════════════════════════════════${NC}`);
  console.log(`${CYAN}5ive-amm - Five VM Deployment${NC}`);
  console.log(`${CYAN}═══════════════════════════════════════════════════════════${NC}\n`);
  console.log(`${CYAN}▶ Configuration${NC}`);
  console.log(`  Cluster: ${cluster}`);
  console.log(`  RPC URL: ${env.rpcUrl}`);
  console.log(`  Five Program: ${env.fiveProgramId}`);
  console.log(`  VM State: ${env.vmStatePda}\n`);

  if (process.env.STRICT_VM_CONSTANTS === '1') {
    const strictCheck = path.join(__dirname, 'check-vm-constants-parity.mjs');
    const parity = spawnSync(
      'node',
      [
        strictCheck,
        '--rpc-url',
        env.rpcUrl,
        '--program-id',
        env.fiveProgramId,
        '--vm-state',
        env.vmStatePda,
      ],
      { stdio: 'inherit' },
    );
    if (parity.status !== 0) {
      process.exit(parity.status ?? 1);
    }
  }

  const result = await deployFiveVmScript({
    ...env,
    feeVaultAccount: derived.feeVaultPdas[0].address,
    label: '5ive-amm',
  });

  console.log(`${CYAN}═══════════════════════════════════════════════════════════${NC}`);
  console.log(`${GREEN}✓ Deployment Complete${NC}\n`);
  console.log(`  Script Account: ${result.scriptAccount}`);
  console.log(`  Deploy Signature: ${result.signature}`);
  console.log(`  Executable Bytecode: ${result.bytecodeLength} bytes`);
  console.log(`  Deploy Metadata: ${result.metadataLength} bytes`);
  console.log(`  Import Metadata Preserved: ${result.hadImportMetadata ? 'yes' : 'no'}\n`);
  console.log(`${CYAN}Export For User Journeys${NC}`);
  console.log(`  export FIVE_AMM_SCRIPT_ACCOUNT=${result.scriptAccount}`);
  console.log(`  export FIVE_VM_STATE_PDA=${result.vmStatePda}`);
}

main().catch((error) => {
  console.error(`${RED}${error.message || error}${NC}`);
  process.exit(1);
});
