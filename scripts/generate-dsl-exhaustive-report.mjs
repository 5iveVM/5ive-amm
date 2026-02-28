#!/usr/bin/env node

import fs from 'node:fs';
import path from 'node:path';
import {
  canonicalFixtureRoot,
  findUntrackedCanonicalFixtures,
  loadDslBuiltinMatrix,
  loadDslFeatureInventory,
  loadDslFeatureMatrix,
  repoRootFrom,
} from './lib/dsl-feature-matrix.mjs';

function requiredBuiltinLayers(builtin) {
  const required = ['compiler', 'bytecode_unit', 'vm_unit'];
  if (builtin.runtime_applicable) {
    required.push('runtime_unit');
  }
  if (builtin.validator_applicable) {
    required.push('validator_localnet');
  }
  return required;
}

function builtinStatus(builtin, matrixScenarioIds, repoRoot) {
  const requiredLayers = requiredBuiltinLayers(builtin);
  const missingRequiredLayers = requiredLayers.filter((layer) => builtin.layers[layer] !== true);
  const missingUnitSuites = builtin.unit_suites.filter(
    (suitePath) => !fs.existsSync(path.join(repoRoot, suitePath))
  );
  const matrixScenarioFound =
    builtin.matrix_scenario == null ? null : matrixScenarioIds.has(builtin.matrix_scenario);

  let status = 'green';
  if (builtin.layers.compiler !== true || builtin.layers.bytecode_unit !== true || missingUnitSuites.length > 0) {
    status = 'red';
  } else if (missingRequiredLayers.length > 0 || matrixScenarioFound === false || builtin.expected_limitations) {
    status = 'yellow';
  }

  return {
    ...builtin,
    required_layers: requiredLayers,
    missing_required_layers: missingRequiredLayers,
    missing_unit_suites: missingUnitSuites,
    matrix_scenario_found: matrixScenarioFound,
    status,
  };
}

function markdownForBuiltinReport(report) {
  const lines = [];
  lines.push('# Builtin Coverage Matrix');
  lines.push('');
  lines.push(`- Builtins tracked: ${report.summary.total}`);
  lines.push(`- Green: ${report.summary.green}`);
  lines.push(`- Yellow: ${report.summary.yellow}`);
  lines.push(`- Red: ${report.summary.red}`);
  lines.push('');
  lines.push('| Builtin | Group | Compiler | Bytecode | VM | Runtime | CLI | WASM | LSP | Runtime Matrix | Localnet | Status | Limitation |');
  lines.push('| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |');
  for (const builtin of report.builtins) {
    lines.push(
      `| ${builtin.name} | ${builtin.category} | ${builtin.layers.compiler ? 'Y' : 'N'} | ${builtin.layers.bytecode_unit ? 'Y' : 'N'} | ${builtin.layers.vm_unit ? 'Y' : 'N'} | ${builtin.layers.runtime_unit ? 'Y' : 'N'} | ${builtin.layers.cli_matrix ? 'Y' : 'N'} | ${builtin.layers.wasm_matrix ? 'Y' : 'N'} | ${builtin.layers.lsp_matrix ? 'Y' : 'N'} | ${builtin.layers.runtime_matrix ? 'Y' : 'N'} | ${builtin.layers.validator_localnet ? 'Y' : 'N'} | ${builtin.status} | ${builtin.expected_limitations || ''} |`
    );
  }
  return lines.join('\n');
}

function buildFeatureInventoryReport(repoRoot, featureMatrix, inventory) {
  const canonicalRoot = canonicalFixtureRoot(repoRoot);
  const canonicalFixtures = new Set();

  function walk(dir) {
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
      const fullPath = path.join(dir, entry.name);
      if (entry.isDirectory()) {
        walk(fullPath);
        continue;
      }
      if (!entry.name.endsWith('.v')) {
        continue;
      }
      canonicalFixtures.add(path.relative(canonicalRoot, fullPath).split(path.sep).join('/'));
    }
  }

  walk(canonicalRoot);

  const inventoryPaths = new Set(
    inventory.fixtures.map((fixture) =>
      fixture.path.startsWith('__root__/') ? fixture.path.slice('__root__/'.length) : fixture.path
    )
  );
  const trackedScenarioPaths = new Set(featureMatrix.scenarios.map((scenario) => scenario.source));
  const unclassifiedFixtures = [...canonicalFixtures].filter(
    (fixturePath) => !inventoryPaths.has(fixturePath) && !trackedScenarioPaths.has(fixturePath)
  );
  unclassifiedFixtures.sort();

  const countsByStatus = {};
  for (const fixture of inventory.fixtures) {
    countsByStatus[fixture.status] = (countsByStatus[fixture.status] || 0) + 1;
  }

  return {
    generated_at: new Date().toISOString(),
    summary: {
      total_canonical_fixtures: canonicalFixtures.size,
      inventory_entries: inventory.fixtures.length,
      representative_matrix_sources: featureMatrix.scenarios.length,
      counts_by_status: countsByStatus,
      unclassified_fixture_count: unclassifiedFixtures.length,
    },
    families: inventory.feature_families,
    fixtures: inventory.fixtures,
    unclassified_fixtures: unclassifiedFixtures,
    feature_matrix_untracked_fixtures: findUntrackedCanonicalFixtures(repoRoot, featureMatrix),
  };
}

const repoRoot = repoRootFrom(import.meta.url);
const featureMatrix = loadDslFeatureMatrix(repoRoot);
const builtinMatrix = loadDslBuiltinMatrix(repoRoot);
const featureInventory = loadDslFeatureInventory(repoRoot);
const outDir = path.join(repoRoot, 'target', 'feature-parity');
fs.mkdirSync(outDir, { recursive: true });

const matrixScenarioIds = new Set(featureMatrix.scenarios.map((scenario) => scenario.id));
const builtinEntries = builtinMatrix.builtins.map((builtin) =>
  builtinStatus(builtin, matrixScenarioIds, repoRoot)
);

const builtinReport = {
  generated_at: new Date().toISOString(),
  summary: {
    total: builtinEntries.length,
    green: builtinEntries.filter((entry) => entry.status === 'green').length,
    yellow: builtinEntries.filter((entry) => entry.status === 'yellow').length,
    red: builtinEntries.filter((entry) => entry.status === 'red').length,
  },
  builtin_groups: builtinMatrix.builtin_groups,
  builtins: builtinEntries,
};

const featureInventoryReport = buildFeatureInventoryReport(repoRoot, featureMatrix, featureInventory);

fs.writeFileSync(
  path.join(outDir, 'builtin-matrix.json'),
  `${JSON.stringify(builtinReport, null, 2)}\n`
);
fs.writeFileSync(
  path.join(outDir, 'builtin-matrix.md'),
  `${markdownForBuiltinReport(builtinReport)}\n`
);
fs.writeFileSync(
  path.join(outDir, 'feature-inventory.json'),
  `${JSON.stringify(featureInventoryReport, null, 2)}\n`
);

console.log(`Builtin report: ${path.join(outDir, 'builtin-matrix.md')}`);
console.log(`Feature inventory report: ${path.join(outDir, 'feature-inventory.json')}`);
