#!/usr/bin/env node

import fs from 'node:fs';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { deriveVmAddresses, loadClusterConfig } from './lib/vm-cluster-config.mjs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const ROOT_DIR = path.resolve(__dirname, '..');
const CLI_PATH = path.join(ROOT_DIR, 'five-cli', 'dist', 'index.js');

const targets = [
  {
    key: 'token',
    envName: 'FIVE_TOKEN_SCRIPT_ACCOUNT',
    // SDK validator token scenarios are authored against the template ABI/contract.
    projectDir: path.join(ROOT_DIR, 'five-templates', 'token'),
  },
  {
    key: 'amm',
    envName: 'FIVE_AMM_SCRIPT_ACCOUNT',
    projectDir: path.join(ROOT_DIR, '5ive-amm'),
  },
  {
    key: 'lending',
    envName: 'FIVE_LENDING_SCRIPT_ACCOUNT',
    projectDir: path.join(ROOT_DIR, '5ive-lending'),
  },
  {
    key: 'lendingOracle',
    envName: 'FIVE_LENDING_ORACLE_SCRIPT_ACCOUNT',
    // No 5ive-* oracle helper project exists yet; keep template helper for now.
    projectDir: path.join(ROOT_DIR, 'five-templates', 'lending-oracle-helper'),
  },
];

function parseArgs(argv) {
  return {
    shell: argv.includes('--shell'),
  };
}

function runNode(args, cwd) {
  const result = spawnSync(process.execPath, args, {
    cwd,
    encoding: 'utf-8',
    env: process.env,
  });

  if (result.status !== 0) {
    const details = [result.stdout, result.stderr].filter(Boolean).join('\n').trim();
    throw new Error(details || `Command failed: node ${args.join(' ')}`);
  }

  return result.stdout;
}

function parseJsonFromMixedOutput(raw) {
  const start = raw.indexOf('{');
  const end = raw.lastIndexOf('}');
  if (start === -1 || end === -1 || end < start) {
    throw new Error('Command did not emit JSON output');
  }
  return JSON.parse(raw.slice(start, end + 1));
}

function buildProject(projectDir) {
  runNode([CLI_PATH, 'build', '--project', projectDir], projectDir);

  const manifestPath = path.join(projectDir, '.five', 'build.json');
  const manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf-8'));
  const artifactPath = path.isAbsolute(manifest.artifact_path)
    ? manifest.artifact_path
    : path.join(projectDir, manifest.artifact_path);

  return { manifest, artifactPath };
}

function deployArtifact(projectDir, artifactPath, rpcUrl, keypairPath, fiveVmProgramId, vmStatePda) {
  const args = [
    CLI_PATH,
    'deploy',
    artifactPath,
    '--project',
    projectDir,
    '--format',
    'json',
    '--target',
    'local',
    '--network',
    rpcUrl,
    '--keypair',
    keypairPath,
  ];

  if (fiveVmProgramId) {
    args.push('--program-id', fiveVmProgramId);
  }
  if (vmStatePda) {
    args.push('--vm-state-account', vmStatePda);
  }

  const stdout = runNode(args, projectDir);
  const parsed = parseJsonFromMixedOutput(stdout);
  const scriptAccount = parsed.scriptAccount || parsed.programId;
  if (!parsed.success || !scriptAccount) {
    throw new Error(parsed.error || `Deployment failed for ${artifactPath}`);
  }

  return {
    ...parsed,
    scriptAccount,
  };
}

function main() {
  const args = parseArgs(process.argv.slice(2));
  const profile = loadClusterConfig({ cluster: 'localnet' });
  const derived = deriveVmAddresses(profile);

  const rpcUrl = process.env.FIVE_RPC_URL || process.env.RPC_URL || profile.rpcUrl || 'http://127.0.0.1:8899';
  const fiveVmProgramId = process.env.FIVE_PROGRAM_ID || profile.programId;
  const vmStatePda = process.env.VM_STATE_PDA || derived.vmStatePda;
  const keypairPath = process.env.FIVE_KEYPAIR_PATH || path.join(process.env.HOME, '.config', 'solana', 'id.json');

  const accounts = {};
  const deployments = [];

  for (const target of targets) {
    const { artifactPath } = buildProject(target.projectDir);
    const deployment = deployArtifact(
      target.projectDir,
      artifactPath,
      rpcUrl,
      keypairPath,
      fiveVmProgramId,
      vmStatePda,
    );

    accounts[target.key] = deployment.scriptAccount;
    deployments.push({
      envName: target.envName,
      projectDir: target.projectDir,
      artifactPath,
      scriptAccount: deployment.scriptAccount,
      transactionId: deployment.transactionId || null,
      deploymentMode: deployment.deploymentMode || null,
      totalTransactions: deployment.totalTransactions ?? null,
    });
  }

  const output = {
    network: 'localnet',
    rpcUrl,
    fiveVmProgramId,
    vmStatePda,
    keypairPath,
    accounts,
    deployments,
  };

  console.log(JSON.stringify(output, null, 2));

  if (args.shell) {
    console.log(`export FIVE_RPC_URL=${rpcUrl}`);
    console.log(`export FIVE_PROGRAM_ID=${fiveVmProgramId}`);
    console.log(`export VM_STATE_PDA=${vmStatePda}`);
    console.log(`export FIVE_KEYPAIR_PATH=${keypairPath}`);
    for (const target of targets) {
      console.log(`export ${target.envName}=${accounts[target.key]}`);
    }
  }
}

try {
  main();
} catch (error) {
  console.error(error.message || error);
  process.exit(1);
}
