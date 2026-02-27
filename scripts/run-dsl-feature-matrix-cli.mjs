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

function parseTextExecutionResult(stdout) {
  const localSectionIndex = stdout.lastIndexOf('LOCAL EXECUTION');
  const localSection = localSectionIndex >= 0 ? stdout.slice(localSectionIndex) : stdout;
  const success =
    localSection.includes('LOCAL EXECUTION') && localSection.includes('OK Execution succeeded');
  if (!success) {
    return { success: false };
  }

  const resultMatch = localSection.match(/Result:\s+([^\n]+)/);
  if (!resultMatch) {
    return { success: true, result: undefined };
  }

  const raw = resultMatch[1].trim();
  if (/^-?\d+$/.test(raw)) {
    return { success: true, result: Number(raw) };
  }
  if (raw === 'true' || raw === 'false') {
    return { success: true, result: raw === 'true' };
  }
  if (
    (raw.startsWith('"') && raw.endsWith('"')) ||
    (raw.startsWith("'") && raw.endsWith("'"))
  ) {
    return { success: true, result: raw.slice(1, -1) };
  }
  return { success: true, result: raw };
}

ensureCliBuilt();
const matrix = loadDslFeatureMatrix(repoRoot);
const scenarios = matrix.scenarios.filter(
  (scenario) =>
    scenario.kind === 'positive' &&
    scenario.layers.cli === true &&
    scenario.requires_accounts === false &&
    scenario.requires_cpi === false
);

let failures = 0;
for (const scenario of scenarios) {
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

  let payload = parseTextExecutionResult(result.stdout);
  if (!payload.success && result.stdout) {
    try {
      payload = parseEmbeddedJsonFromStdout(result.stdout);
    } catch {
      // Fall through to command failure handling below.
    }
  }

  if (!payload.success && result.status !== 0) {
    failures += 1;
    console.error(`[FAIL] ${scenario.id}: command exited with ${result.status}`);
    if (result.stdout) console.error(result.stdout);
    if (result.stderr) console.error(result.stderr);
    continue;
  }

  if (!payload.success) {
    failures += 1;
    console.error(`[FAIL] ${scenario.id}: execution returned success=false`);
    if (result.stdout) console.error(result.stdout);
    if (result.stderr) console.error(result.stderr);
    continue;
  }

  if (Object.prototype.hasOwnProperty.call(scenario, 'expected_result')) {
    if (!valuesEqual(scenario.expected_result, payload.result)) {
      failures += 1;
      console.error(
        `[FAIL] ${scenario.id}: expected result ${JSON.stringify(
          scenario.expected_result
        )}, got ${JSON.stringify(payload.result)}`
      );
      continue;
    }
  }

  console.log(`[PASS] ${scenario.id}`);
}

if (failures > 0) {
  process.exit(1);
}

console.log(`CLI feature matrix passed (${scenarios.length} scenario(s)).`);
