#!/usr/bin/env node

import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import {
  loadDslBuiltinMatrix,
  repoRootFrom,
} from './lib/dsl-feature-matrix.mjs';

function parseArgs(argv) {
  const args = {
    network: 'localnet',
    resultsDir: '',
    programId: process.env.FIVE_PROGRAM_ID || '',
    vmState: process.env.VM_STATE_PDA || '',
    keypair: process.env.FIVE_KEYPAIR_PATH || path.join(process.env.HOME || '', '.config/solana/id.json'),
  };

  for (let i = 2; i < argv.length; i += 1) {
    const value = argv[i];
    if (value === '--network' && argv[i + 1]) args.network = argv[++i];
    else if (value === '--results-dir' && argv[i + 1]) args.resultsDir = argv[++i];
    else if (value === '--program-id' && argv[i + 1]) args.programId = argv[++i];
    else if (value === '--vm-state' && argv[i + 1]) args.vmState = argv[++i];
    else if (value === '--keypair' && argv[i + 1]) args.keypair = argv[++i];
  }

  return args;
}

function ensureFile(filePath, label) {
  if (!filePath) {
    throw new Error(`missing ${label}`);
  }
  if (!fs.existsSync(filePath)) {
    throw new Error(`${label} not found: ${filePath}`);
  }
}

function loadProbeOutput(filePath) {
  if (!fs.existsSync(filePath)) {
    throw new Error(`probe output not found: ${filePath}`);
  }
  return fs.readFileSync(filePath, 'utf8');
}

function loadProbeArtifact(filePath) {
  if (!fs.existsSync(filePath)) {
    throw new Error(`probe artifact not found: ${filePath}`);
  }
  return JSON.parse(fs.readFileSync(filePath, 'utf8'));
}

function parseCryptoProbe(stdout, stderr) {
  const combined = `${stdout || ''}\n${stderr || ''}`;
  if (combined.includes('SKIP validator_crypto_probe_onchain:')) {
    throw new Error('validator crypto probe skipped');
  }
  const match = combined.match(
    /CRYPTO_PROBE valid_deploy_signature=(\S+) valid_deploy_cu=(\d+) valid_execute_signature=(\S+) valid_execute_cu=(\d+) invalid_execute_signature=(\S+) invalid_execute_cu=(\d+)/
  );
  if (!match) {
    throw new Error('could not parse CRYPTO_PROBE output');
  }
  return {
    validDeploySignature: match[1],
    validDeployCu: Number(match[2]),
    validExecuteSignature: match[3],
    validExecuteCu: Number(match[4]),
    invalidExecuteSignature: match[5],
    invalidExecuteCu: Number(match[6]),
  };
}

function parseStdlibProbe(stdout, stderr) {
  const combined = `${stdout || ''}\n${stderr || ''}`;
  if (combined.includes('SKIP validator_stdlib_time_and_sysvar_onchain:')) {
    throw new Error('validator stdlib probe skipped');
  }
  const match = combined.match(
    /STDLIB_PROBE now_deploy_signature=(\S+) now_deploy_cu=(\d+) now_execute_signature=(\S+) now_execute_cu=(\d+) clock_deploy_signature=(\S+) clock_deploy_cu=(\d+) clock_execute_signature=(\S+) clock_execute_cu=(\d+)/
  );
  if (!match) {
    throw new Error('could not parse STDLIB_PROBE output');
  }
  return {
    nowDeploySignature: match[1],
    nowDeployCu: Number(match[2]),
    nowExecuteSignature: match[3],
    nowExecuteCu: Number(match[4]),
    clockDeploySignature: match[5],
    clockDeployCu: Number(match[6]),
    clockExecuteSignature: match[7],
    clockExecuteCu: Number(match[8]),
  };
}

function parseAccountProbe(stdout, stderr) {
  const combined = `${stdout || ''}\n${stderr || ''}`;
  if (combined.includes('SKIP validator_account_probe_onchain:')) {
    throw new Error('validator account probe skipped');
  }
  const match = combined.match(
    /ACCOUNT_PROBE load_deploy_signature=(\S+) load_deploy_cu=(\d+) load_execute_signature=(\S+) load_execute_cu=(\d+) lamports_deploy_signature=(\S+) lamports_deploy_cu=(\d+) lamports_execute_signature=(\S+) lamports_execute_cu=(\d+) owner_deploy_signature=(\S+) owner_deploy_cu=(\d+) owner_execute_signature=(\S+) owner_execute_cu=(\d+) key_deploy_signature=(\S+) key_deploy_cu=(\d+) key_execute_signature=(\S+) key_execute_cu=(\d+)/
  );
  if (!match) {
    throw new Error('could not parse ACCOUNT_PROBE output');
  }
  return {
    loadDeploySignature: match[1],
    loadDeployCu: Number(match[2]),
    loadExecuteSignature: match[3],
    loadExecuteCu: Number(match[4]),
    lamportsDeploySignature: match[5],
    lamportsDeployCu: Number(match[6]),
    lamportsExecuteSignature: match[7],
    lamportsExecuteCu: Number(match[8]),
    ownerDeploySignature: match[9],
    ownerDeployCu: Number(match[10]),
    ownerExecuteSignature: match[11],
    ownerExecuteCu: Number(match[12]),
    keyDeploySignature: match[13],
    keyDeployCu: Number(match[14]),
    keyExecuteSignature: match[15],
    keyExecuteCu: Number(match[16]),
  };
}

function toMarkdown(report) {
  const lines = [];
  lines.push('# Builtin Validator Localnet Report');
  lines.push('');
  lines.push(`- Network: ${report.network}`);
  lines.push(`- Program ID: ${report.programId}`);
  lines.push(`- VM State: ${report.vmState || 'n/a'}`);
  lines.push('');
  lines.push('| Builtin | Status | Mode | Target | Signature | CU | Notes |');
  lines.push('| --- | --- | --- | --- | --- | --- | --- |');
  for (const builtin of report.builtins) {
    const signature =
      builtin.transactionId ||
      builtin.validExecuteSignature ||
      builtin.validDeploySignature ||
      '';
    const cu =
      builtin.computeUnitsUsed ??
      builtin.validExecuteCu ??
      builtin.validDeployCu ??
      '';
    lines.push(
      `| ${builtin.name} | ${builtin.status} | ${builtin.validatorMode} | ${builtin.validatorTarget} | ${signature} | ${cu} | ${builtin.note || ''} |`
    );
  }
  return lines.join('\n');
}

const args = parseArgs(process.argv);
if (args.network !== 'localnet') {
  throw new Error('builtin validator matrix currently supports only --network localnet');
}

ensureFile(args.keypair, 'validator keypair');
if (!args.programId) {
  throw new Error('missing validator program id (--program-id or FIVE_PROGRAM_ID)');
}
if (!args.vmState) {
  throw new Error('missing VM state PDA (--vm-state or VM_STATE_PDA)');
}

const repoRoot = repoRootFrom(import.meta.url);
const builtinMatrix = loadDslBuiltinMatrix(repoRoot);
const resultsDir =
  args.resultsDir ||
  path.join(
    repoRoot,
    'target',
    'sdk-validator-runs',
    new Date().toISOString().replace(/[:.]/g, '-'),
    'builtin-localnet'
  );
fs.mkdirSync(resultsDir, { recursive: true });

const report = {
  generatedAt: new Date().toISOString(),
  network: args.network,
  programId: args.programId,
  vmState: args.vmState,
  builtins: [],
};

let failures = 0;

const validatorBuiltins = builtinMatrix.builtins.filter(
  (builtin) => builtin.layers.validator_localnet === true
);
const featureMatrixBuiltins = validatorBuiltins.filter(
  (builtin) => builtin.validator_mode === 'feature_matrix'
);
const cargoBuiltins = validatorBuiltins.filter(
  (builtin) => builtin.validator_mode === 'cargo_test'
);

if (featureMatrixBuiltins.length > 0) {
  const scenarioIds = [...new Set(featureMatrixBuiltins.map((builtin) => builtin.validator_target))];
  const featureResultsDir = path.join(resultsDir, 'feature-matrix');
  const run = spawnSync(
    'node',
    [
      path.join(repoRoot, 'scripts', 'run-dsl-validator-matrix.mjs'),
      '--network',
      args.network,
      '--program-id',
      args.programId,
      '--vm-state',
      args.vmState,
      '--keypair',
      args.keypair,
      '--results-dir',
      featureResultsDir,
      '--scenario-ids',
      scenarioIds.join(','),
    ],
    {
      cwd: repoRoot,
      env: process.env,
      encoding: 'utf8',
      maxBuffer: 10 * 1024 * 1024,
    }
  );

  const featureReportPath = path.join(featureResultsDir, 'dsl-validator-matrix-report.json');
  const featureReport = fs.existsSync(featureReportPath)
    ? JSON.parse(fs.readFileSync(featureReportPath, 'utf8'))
    : { scenarios: [] };
  const scenariosById = new Map(
    (featureReport.scenarios || []).map((scenario) => [scenario.id, scenario])
  );

  for (const builtin of featureMatrixBuiltins) {
    const scenario = scenariosById.get(builtin.validator_target);
    const passed = scenario?.status === 'PASS';
    if (!passed) {
      failures += 1;
    }
    report.builtins.push({
      id: builtin.id,
      name: builtin.name,
      validatorMode: builtin.validator_mode,
      validatorTarget: builtin.validator_target,
      status: passed ? 'PASS' : 'FAIL',
      transactionId: scenario?.transactionId ?? null,
      computeUnitsUsed: scenario?.computeUnitsUsed ?? null,
      note: passed ? '' : scenario?.error || run.stderr || run.stdout || 'feature matrix validator run failed',
    });
  }
}

const cargoGroups = new Map();
for (const builtin of cargoBuiltins) {
  const group = cargoGroups.get(builtin.validator_target) || [];
  group.push(builtin);
  cargoGroups.set(builtin.validator_target, group);
}

for (const [target, builtins] of cargoGroups.entries()) {
  let testFile;
  let testName;
  let parseOutput;

  if (target === 'runtime_validator_crypto_probe_tests::validator_crypto_probe_onchain') {
    testFile = 'runtime_validator_crypto_probe_tests';
    testName = 'validator_crypto_probe_onchain';
    parseOutput = parseCryptoProbe;
  } else if (target === 'runtime_validator_account_probe_tests::validator_account_probe_onchain') {
    testFile = 'runtime_validator_account_probe_tests';
    testName = 'validator_account_probe_onchain';
    parseOutput = parseAccountProbe;
  } else if (target === 'runtime_validator_stdlib_probe_tests::validator_stdlib_time_and_sysvar_onchain') {
    testFile = 'runtime_validator_stdlib_probe_tests';
    testName = 'validator_stdlib_time_and_sysvar_onchain';
    parseOutput = parseStdlibProbe;
  } else {
    throw new Error(`unsupported cargo builtin validator target: ${target}`);
  }

  const probeArtifactPath = path.join(resultsDir, `${testFile}-${testName}.json`);
  const probeOutputPath = path.join(resultsDir, `${testFile}-${testName}.log`);
  let probeStdout = '';
  let probeStderr = '';
  let probe;
  let groupPassed = true;
  let note = '';

  try {
    if (fs.existsSync(probeArtifactPath)) {
      probe = loadProbeArtifact(probeArtifactPath);
    } else {
      probeStdout = loadProbeOutput(probeOutputPath);
      probe = parseOutput(probeStdout, probeStderr);
    }
  } catch (error) {
    groupPassed = false;
    note = error.message;
  }

  for (const builtin of builtins) {
    let entry = {
      id: builtin.id,
      name: builtin.name,
      validatorMode: builtin.validator_mode,
      validatorTarget: builtin.validator_target,
      status: groupPassed ? 'PASS' : 'FAIL',
      note:
        note ||
        (!groupPassed ? probeStdout || probeStderr || 'missing or invalid probe log' : ''),
    };

    if (builtin.name === 'verify_ed25519_instruction') {
      entry = {
        ...entry,
        validDeploySignature: probe?.validDeploySignature ?? null,
        validDeployCu: probe?.validDeployCu ?? null,
        validExecuteSignature: probe?.validExecuteSignature ?? null,
        validExecuteCu: probe?.validExecuteCu ?? null,
        invalidExecuteSignature: probe?.invalidExecuteSignature ?? null,
        invalidExecuteCu: probe?.invalidExecuteCu ?? null,
      };
    } else if (builtin.name === 'now_seconds') {
      entry = {
        ...entry,
        transactionId: probe?.nowExecuteSignature ?? null,
        computeUnitsUsed: probe?.nowExecuteCu ?? null,
        deploySignature: probe?.nowDeploySignature ?? null,
        deployComputeUnitsUsed: probe?.nowDeployCu ?? null,
      };
    } else if (builtin.name === 'clock_sysvar') {
      entry = {
        ...entry,
        transactionId: probe?.clockExecuteSignature ?? null,
        computeUnitsUsed: probe?.clockExecuteCu ?? null,
        deploySignature: probe?.clockDeploySignature ?? null,
        deployComputeUnitsUsed: probe?.clockDeployCu ?? null,
      };
    } else if (builtin.name === 'load_account_u64_word') {
      entry = {
        ...entry,
        transactionId: probe?.loadExecuteSignature ?? null,
        computeUnitsUsed: probe?.loadExecuteCu ?? null,
        deploySignature: probe?.loadDeploySignature ?? null,
        deployComputeUnitsUsed: probe?.loadDeployCu ?? null,
        note:
          entry.note ||
          'Shared validator probe also exercises account.ctx.lamports, account.ctx.owner, and account.ctx.key.',
      };
    }

    if (!groupPassed) {
      failures += 1;
    }

    report.builtins.push(entry);
  }
}

const jsonPath = path.join(resultsDir, 'builtin-validator-localnet.json');
const mdPath = path.join(resultsDir, 'builtin-validator-localnet.md');
const parityJsonPath = path.join(repoRoot, 'target', 'feature-parity', 'builtin-validator-localnet.json');
const parityMdPath = path.join(repoRoot, 'target', 'feature-parity', 'builtin-validator-localnet.md');
fs.mkdirSync(path.dirname(parityJsonPath), { recursive: true });
fs.writeFileSync(jsonPath, `${JSON.stringify(report, null, 2)}\n`);
fs.writeFileSync(mdPath, `${toMarkdown(report)}\n`);
fs.writeFileSync(parityJsonPath, `${JSON.stringify(report, null, 2)}\n`);
fs.writeFileSync(parityMdPath, `${toMarkdown(report)}\n`);

console.log(`Builtin validator report written to ${jsonPath}`);

if (failures > 0) {
  process.exit(1);
}
