#!/usr/bin/env node

import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import {
  loadDslFeatureMatrix,
  parseEmbeddedJsonFromStdout,
  parseScenarioParams,
  repoRootFrom,
  resolveScenarioSource,
} from './lib/dsl-feature-matrix.mjs';

function parseArgs(argv) {
  const args = {
    network: 'localnet',
    resultsDir: '',
    programId: process.env.FIVE_PROGRAM_ID || '',
    vmState: process.env.VM_STATE_PDA || '',
    keypair: process.env.FIVE_KEYPAIR_PATH || path.join(process.env.HOME || '', '.config/solana/id.json'),
    scenarioIds: [],
  };

  for (let i = 2; i < argv.length; i += 1) {
    const value = argv[i];
    if (value === '--network' && argv[i + 1]) args.network = argv[++i];
    else if (value === '--results-dir' && argv[i + 1]) args.resultsDir = argv[++i];
    else if (value === '--program-id' && argv[i + 1]) args.programId = argv[++i];
    else if (value === '--vm-state' && argv[i + 1]) args.vmState = argv[++i];
    else if (value === '--keypair' && argv[i + 1]) args.keypair = argv[++i];
    else if (value === '--scenario-ids' && argv[i + 1]) {
      args.scenarioIds = argv[++i]
        .split(',')
        .map((entry) => entry.trim())
        .filter(Boolean);
    }
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

function ensureCliBuilt(repoRoot) {
  const cliDir = path.join(repoRoot, 'five-cli');
  const cliEntry = path.join(cliDir, 'dist', 'index.js');
  if (fs.existsSync(cliEntry)) {
    return cliEntry;
  }
  const build = spawnSync('npm', ['run', 'build:js'], {
    cwd: cliDir,
    stdio: 'inherit',
    env: process.env,
  });
  if (build.status !== 0) {
    process.exit(build.status ?? 1);
  }
  return cliEntry;
}

function toReportMarkdown(report) {
  const lines = [];
  lines.push('# DSL Validator Matrix Report');
  lines.push('');
  lines.push(`- Network: ${report.network}`);
  lines.push(`- Program ID: ${report.programId}`);
  lines.push(`- VM State: ${report.vmState || 'n/a'}`);
  lines.push('');
  lines.push('| Scenario | Category | Status | Mode |');
  lines.push('| --- | --- | --- | --- |');
  for (const scenario of report.scenarios) {
    lines.push(`| ${scenario.id} | ${scenario.category} | ${scenario.status} | ${scenario.validatorMode} |`);
  }
  return lines.join('\n');
}

const args = parseArgs(process.argv);
const repoRoot = repoRootFrom(import.meta.url);
const matrix = loadDslFeatureMatrix(repoRoot);
const cliEntry = ensureCliBuilt(repoRoot);
const requireLocalnet = process.env.FIVE_REQUIRE_LOCALNET_MATRIX === '1';

if (args.network === 'localnet') {
  ensureFile(args.keypair, 'validator keypair');
  if (!args.programId) {
    throw new Error('missing validator program id (--program-id or FIVE_PROGRAM_ID)');
  }
  if (requireLocalnet && !args.vmState) {
    throw new Error('missing VM state PDA (--vm-state or VM_STATE_PDA)');
  }
}

const resultsDir =
  args.resultsDir ||
  path.join(
    repoRoot,
    'target',
    'sdk-validator-runs',
    new Date().toISOString().replace(/[:.]/g, '-')
  );
fs.mkdirSync(resultsDir, { recursive: true });

const report = {
  network: args.network,
  programId: args.programId,
  vmState: args.vmState,
  generatedAt: new Date().toISOString(),
  scenarios: [],
};

const validatorLayer =
  args.network === 'devnet' ? 'validator_devnet_tracked' : 'validator_localnet';

const requestedScenarioIds = new Set(args.scenarioIds);
const scenarios = matrix.scenarios.filter((scenario) => {
  if (scenario.layers[validatorLayer] !== true) {
    return false;
  }
  if (requestedScenarioIds.size === 0) {
    return true;
  }
  return requestedScenarioIds.has(scenario.id);
});

if (requestedScenarioIds.size > 0) {
  const foundIds = new Set(scenarios.map((scenario) => scenario.id));
  const missingIds = [...requestedScenarioIds].filter((id) => !foundIds.has(id));
  if (missingIds.length > 0) {
    throw new Error(`unknown or non-validator scenarios requested: ${missingIds.join(', ')}`);
  }
}
const genericScenarios = scenarios.filter((scenario) => scenario.validator_mode === 'localnet_generic');
const suiteScenarios = scenarios.filter((scenario) => scenario.validator_mode === 'sdk_suite');

let failures = 0;

for (const scenario of genericScenarios) {
  const sourcePath = resolveScenarioSource(repoRoot, scenario);
  const params = parseScenarioParams(repoRoot, scenario);
  const target = args.network === 'localnet' ? 'local' : 'devnet';
  const command = [
    cliEntry,
    'deploy-and-execute',
    sourcePath,
    '--target',
    target,
    '--network',
    args.network === 'localnet' ? 'http://127.0.0.1:8899' : 'https://api.devnet.solana.com',
    '--keypair',
    args.keypair,
    '--program-id',
    args.programId,
    '--function',
    String(scenario.function ?? 0),
    '--params',
    JSON.stringify(params),
    '--format',
    'json',
  ];

  const result = spawnSync('node', command, {
    cwd: repoRoot,
    env: process.env,
    encoding: 'utf8',
  });

  if (result.status !== 0) {
    failures += 1;
    report.scenarios.push({
      id: scenario.id,
      category: scenario.category,
      validatorMode: scenario.validator_mode,
      status: 'FAIL',
      error: result.stderr || result.stdout || `exit ${result.status}`,
    });
    continue;
  }

  let payload;
  try {
    payload = parseEmbeddedJsonFromStdout(result.stdout);
  } catch (error) {
    failures += 1;
    report.scenarios.push({
      id: scenario.id,
      category: scenario.category,
      validatorMode: scenario.validator_mode,
      status: 'FAIL',
      error: error.message,
    });
    continue;
  }

  const passed = !!payload.success;
  if (!passed) {
    failures += 1;
  }
  report.scenarios.push({
    id: scenario.id,
    category: scenario.category,
    validatorMode: scenario.validator_mode,
    status: passed ? 'PASS' : 'FAIL',
    result: payload.result,
    transactionId: payload.transactionId || null,
    computeUnitsUsed: payload.computeUnitsUsed ?? null,
  });
}

const suiteNames = [...new Set(suiteScenarios.map((scenario) => scenario.validator_scenario))];
if (suiteNames.length > 0) {
  const suiteResultsDir = path.join(resultsDir, `${args.network}-sdk-suites`);
  const suiteRun = spawnSync(
    'bash',
    [
      path.join(repoRoot, 'scripts', 'run-sdk-validator-suites.sh'),
      '--network',
      args.network,
      '--program-id',
      args.programId,
      '--vm-state',
      args.vmState,
      '--keypair',
      args.keypair,
      '--scenarios',
      suiteNames.join(','),
      '--results-dir',
      suiteResultsDir,
    ],
    {
      cwd: repoRoot,
      env: process.env,
      encoding: 'utf8',
    }
  );

  const suiteReportPath = path.join(suiteResultsDir, 'sdk-validator-report.json');
  const suiteReport = fs.existsSync(suiteReportPath)
    ? JSON.parse(fs.readFileSync(suiteReportPath, 'utf8'))
    : null;

  const byScenario = new Map();
  for (const scenario of suiteReport?.scenarios || []) {
    byScenario.set(scenario.name, scenario);
  }

  for (const scenario of suiteScenarios) {
    const suiteScenario = byScenario.get(scenario.validator_scenario);
    const passed = suiteScenario?.status === 'PASS';
    if (!passed) {
      failures += 1;
    }
    report.scenarios.push({
      id: scenario.id,
      category: scenario.category,
      validatorMode: scenario.validator_mode,
      upstreamScenario: scenario.validator_scenario,
      status: passed ? 'PASS' : 'FAIL',
      exitCode: suiteScenario?.exitCode ?? suiteRun.status ?? 1,
    });
  }
}

const reportJsonPath = path.join(resultsDir, 'dsl-validator-matrix-report.json');
const reportMdPath = path.join(resultsDir, 'dsl-validator-matrix-report.md');
fs.writeFileSync(reportJsonPath, `${JSON.stringify(report, null, 2)}\n`);
fs.writeFileSync(reportMdPath, `${toReportMarkdown(report)}\n`);

console.log(`Validator matrix report written to ${reportJsonPath}`);

if (failures > 0) {
  process.exit(1);
}
