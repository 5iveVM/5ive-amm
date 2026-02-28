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
  } else if (target === 'runtime_validator_stdlib_probe_tests::validator_stdlib_time_and_sysvar_onchain') {
    testFile = 'runtime_validator_stdlib_probe_tests';
    testName = 'validator_stdlib_time_and_sysvar_onchain';
    parseOutput = parseStdlibProbe;
  } else {
    throw new Error(`unsupported cargo builtin validator target: ${target}`);
  }

  const run = spawnSync(
    'cargo',
    [
      'test',
      '-q',
      '-p',
      'five',
      '--features',
      'validator-harness',
      '--test',
      testFile,
      testName,
      '--',
      '--ignored',
      '--nocapture',
    ],
    {
      cwd: repoRoot,
      env: {
        ...process.env,
        FIVE_CU_NETWORK: 'localnet',
        FIVE_CU_PROGRAM_ID: args.programId,
        FIVE_CU_PAYER_KEYPAIR: args.keypair,
        FIVE_CU_RPC_URL: 'http://127.0.0.1:8899',
      },
      encoding: 'utf8',
    }
  );

  let probe;
  let groupPassed = run.status === 0;
  let note = '';

  try {
    probe = parseOutput(run.stdout, run.stderr);
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
        (run.status === 0 ? '' : run.stderr || run.stdout || `cargo test exit ${run.status}`),
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
