#!/usr/bin/env node

import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import {
  loadDslBuiltinMatrix,
  loadDslFeatureMatrix,
  parseEmbeddedJsonFromStdout,
  parseScenarioParams,
  repoRootFrom,
  resolveScenarioSource,
} from './lib/dsl-feature-matrix.mjs';

const repoRoot = repoRootFrom(import.meta.url);
const cliDir = path.join(repoRoot, 'five-cli');
const cliEntry = path.join(cliDir, 'dist', 'index.js');

function ensureCliBuilt() {
  if (fs.existsSync(cliEntry)) {
    return;
  }
  const build = spawnSync('npm', ['run', 'build:js'], {
    cwd: cliDir,
    stdio: 'inherit',
    env: process.env,
  });
  if (build.status !== 0) {
    process.exit(build.status ?? 1);
  }
}

function valuesEqual(expected, actual) {
  if (typeof expected === 'number' && typeof actual === 'number') {
    return Number(expected) === Number(actual);
  }
  return JSON.stringify(expected) === JSON.stringify(actual);
}

ensureCliBuilt();

const featureMatrix = loadDslFeatureMatrix(repoRoot);
const builtinMatrix = loadDslBuiltinMatrix(repoRoot);
const scenariosById = new Map(featureMatrix.scenarios.map((scenario) => [scenario.id, scenario]));

const builtinTargets = builtinMatrix.builtins.filter(
  (builtin) => builtin.layers.cli_matrix === true && typeof builtin.matrix_scenario === 'string'
);

if (builtinTargets.length === 0) {
  console.log('No CLI-eligible builtin matrix scenarios are declared yet.');
  process.exit(0);
}

let failures = 0;
for (const builtin of builtinTargets) {
  const scenario = scenariosById.get(builtin.matrix_scenario);
  if (!scenario) {
    failures += 1;
    console.error(`[FAIL] ${builtin.name}: missing matrix scenario ${builtin.matrix_scenario}`);
    continue;
  }
  if (
    scenario.kind !== 'positive' ||
    scenario.layers.cli !== true ||
    scenario.requires_accounts ||
    scenario.requires_cpi
  ) {
    failures += 1;
    console.error(
      `[FAIL] ${builtin.name}: scenario ${scenario.id} is not eligible for CLI local execution`
    );
    continue;
  }

  const sourcePath = resolveScenarioSource(repoRoot, scenario);
  const params = parseScenarioParams(repoRoot, scenario);
  const args = [
    cliEntry,
    'execute',
    sourcePath,
    '--local',
    '--function',
    String(scenario.function ?? 0),
    '--params',
    JSON.stringify(params),
  ];

  const result = spawnSync('node', args, {
    cwd: repoRoot,
    env: process.env,
    encoding: 'utf8',
  });

  if (result.status !== 0) {
    failures += 1;
    console.error(`[FAIL] ${builtin.name}: command exited with ${result.status}`);
    if (result.stdout) console.error(result.stdout);
    if (result.stderr) console.error(result.stderr);
    continue;
  }

  let payload;
  try {
    payload = parseEmbeddedJsonFromStdout(result.stdout);
  } catch (error) {
    failures += 1;
    console.error(`[FAIL] ${builtin.name}: ${error.message}`);
    continue;
  }

  if (!payload.success) {
    failures += 1;
    console.error(`[FAIL] ${builtin.name}: execution returned success=false`);
    continue;
  }

  if (
    Object.prototype.hasOwnProperty.call(scenario, 'expected_result') &&
    !valuesEqual(scenario.expected_result, payload.result)
  ) {
    failures += 1;
    console.error(
      `[FAIL] ${builtin.name}: expected ${JSON.stringify(scenario.expected_result)}, got ${JSON.stringify(payload.result)}`
    );
    continue;
  }

  console.log(`[PASS] ${builtin.name} via ${scenario.id}`);
}

if (failures > 0) {
  process.exit(1);
}

console.log(`Builtin CLI matrix passed (${builtinTargets.length} builtin(s)).`);
