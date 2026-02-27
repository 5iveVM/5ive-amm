import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

export const VALID_LAYERS = new Set([
  'compiler',
  'vm',
  'cli',
  'wasm',
  'lsp',
  'solana_runtime',
  'validator_localnet',
  'validator_devnet_tracked',
]);

export const VALID_PARAMS_SOURCES = new Set(['inline', 'test-params-comment']);
export const VALID_RUNTIME_MODES = new Set(['none', 'generic', 'template_fixture']);
export const VALID_VALIDATOR_MODES = new Set(['none', 'localnet_generic', 'sdk_suite']);

export function repoRootFrom(importMetaUrl) {
  const dir = path.dirname(fileURLToPath(importMetaUrl));
  if (path.basename(dir) === 'lib') {
    return path.resolve(dir, '..', '..');
  }
  return path.resolve(dir, '..');
}

export function matrixPath(repoRoot) {
  return path.join(repoRoot, 'testing', 'dsl-feature-matrix.json');
}

export function canonicalFixtureRoot(repoRoot) {
  return path.join(repoRoot, 'five-cli', 'test-scripts');
}

export function loadDslFeatureMatrix(repoRoot) {
  const file = matrixPath(repoRoot);
  const raw = fs.readFileSync(file, 'utf8');
  const parsed = JSON.parse(raw);
  validateDslFeatureMatrix(parsed, repoRoot);
  return parsed;
}

export function validateDslFeatureMatrix(matrix, repoRoot) {
  if (!matrix || typeof matrix !== 'object') {
    throw new Error('DSL feature matrix must be a JSON object');
  }
  if (!Array.isArray(matrix.categories) || matrix.categories.length === 0) {
    throw new Error('DSL feature matrix categories must be a non-empty array');
  }
  if (!Array.isArray(matrix.scenarios) || matrix.scenarios.length === 0) {
    throw new Error('DSL feature matrix scenarios must be a non-empty array');
  }

  const categoryIds = new Set();
  for (const category of matrix.categories) {
    if (!category?.id || typeof category.id !== 'string') {
      throw new Error('every category must define a string id');
    }
    if (categoryIds.has(category.id)) {
      throw new Error(`duplicate category id: ${category.id}`);
    }
    categoryIds.add(category.id);
    if (!Array.isArray(category.required_layers)) {
      throw new Error(`category ${category.id} must define required_layers`);
    }
    for (const layer of category.required_layers) {
      if (!VALID_LAYERS.has(layer)) {
        throw new Error(`category ${category.id} has invalid required layer: ${layer}`);
      }
    }
  }

  const scenarioIds = new Set();
  for (const scenario of matrix.scenarios) {
    if (!scenario?.id || typeof scenario.id !== 'string') {
      throw new Error('every scenario must define a string id');
    }
    if (scenarioIds.has(scenario.id)) {
      throw new Error(`duplicate scenario id: ${scenario.id}`);
    }
    scenarioIds.add(scenario.id);

    if (!categoryIds.has(scenario.category)) {
      throw new Error(`scenario ${scenario.id} references unknown category: ${scenario.category}`);
    }
    if (!scenario.source || typeof scenario.source !== 'string') {
      throw new Error(`scenario ${scenario.id} must define a source`);
    }
    const sourcePath = resolveScenarioSource(repoRoot, scenario);
    if (!fs.existsSync(sourcePath)) {
      throw new Error(`scenario ${scenario.id} source not found: ${sourcePath}`);
    }
    if (!['positive', 'negative'].includes(scenario.kind)) {
      throw new Error(`scenario ${scenario.id} has invalid kind: ${scenario.kind}`);
    }
    if (!VALID_PARAMS_SOURCES.has(scenario.params_source)) {
      throw new Error(`scenario ${scenario.id} has invalid params_source: ${scenario.params_source}`);
    }
    if (!VALID_RUNTIME_MODES.has(scenario.runtime_mode)) {
      throw new Error(`scenario ${scenario.id} has invalid runtime_mode: ${scenario.runtime_mode}`);
    }
    if (!VALID_VALIDATOR_MODES.has(scenario.validator_mode)) {
      throw new Error(`scenario ${scenario.id} has invalid validator_mode: ${scenario.validator_mode}`);
    }
    if (!scenario.layers || typeof scenario.layers !== 'object') {
      throw new Error(`scenario ${scenario.id} must define layers`);
    }
    for (const [layer, enabled] of Object.entries(scenario.layers)) {
      if (!VALID_LAYERS.has(layer)) {
        throw new Error(`scenario ${scenario.id} uses invalid layer: ${layer}`);
      }
      if (typeof enabled !== 'boolean') {
        throw new Error(`scenario ${scenario.id} layer ${layer} must be boolean`);
      }
    }
    if (scenario.runtime_mode === 'template_fixture') {
      if (!scenario.runtime_fixture || typeof scenario.runtime_fixture !== 'string') {
        throw new Error(`scenario ${scenario.id} must define runtime_fixture for template_fixture mode`);
      }
      const fixturePath = path.join(repoRoot, scenario.runtime_fixture);
      if (!fs.existsSync(fixturePath)) {
        throw new Error(`scenario ${scenario.id} runtime fixture not found: ${fixturePath}`);
      }
    }
    if (scenario.validator_mode === 'sdk_suite' && !scenario.validator_scenario) {
      throw new Error(`scenario ${scenario.id} must define validator_scenario for sdk_suite mode`);
    }
    if (scenario.kind === 'negative' && !scenario.expected_error_contains) {
      throw new Error(`negative scenario ${scenario.id} must define expected_error_contains`);
    }
  }

  for (const category of matrix.categories) {
    const scenarios = matrix.scenarios.filter((scenario) => scenario.category === category.id);
    if (scenarios.length === 0) {
      throw new Error(`category ${category.id} has no scenarios`);
    }
    for (const layer of category.required_layers) {
      const covered = scenarios.some((scenario) => scenario.layers?.[layer] === true);
      if (!covered) {
        throw new Error(`category ${category.id} is missing required layer coverage for ${layer}`);
      }
    }
  }
}

export function resolveScenarioSource(repoRoot, scenario) {
  return path.join(canonicalFixtureRoot(repoRoot), scenario.source);
}

export function parseScenarioParams(repoRoot, scenario) {
  if (scenario.params_source === 'inline') {
    return Array.isArray(scenario.params) ? scenario.params : [];
  }

  const content = fs.readFileSync(resolveScenarioSource(repoRoot, scenario), 'utf8');
  const line = content
    .split('\n')
    .map((entry) => entry.trim())
    .find((entry) => entry.includes('@test-params'));

  if (!line) {
    return [];
  }

  const match = line.match(/@test-params(?:\s+(.*))?$/);
  const paramsStr = (match?.[1] || '').trim();
  if (!paramsStr) {
    return [];
  }
  if (paramsStr.startsWith('[')) {
    const parsed = JSON.parse(paramsStr);
    return Array.isArray(parsed) ? parsed : [];
  }
  return paramsStr
    .split(/\s+/)
    .filter(Boolean)
    .map(parseMatrixToken);
}

export function parseMatrixToken(token) {
  if (
    (token.startsWith('"') && token.endsWith('"')) ||
    (token.startsWith("'") && token.endsWith("'"))
  ) {
    return token.slice(1, -1);
  }
  if (token === 'true') return true;
  if (token === 'false') return false;
  const asNumber = Number(token);
  if (!Number.isNaN(asNumber)) {
    return asNumber;
  }
  return token;
}

export function findUntrackedCanonicalFixtures(repoRoot, matrix) {
  const tracked = new Set(matrix.scenarios.map((scenario) => scenario.source));
  const root = canonicalFixtureRoot(repoRoot);
  const out = [];

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
      const relative = path.relative(root, fullPath).split(path.sep).join('/');
      if (!tracked.has(relative)) {
        out.push(relative);
      }
    }
  }

  walk(root);
  out.sort();
  return out;
}

export function parseEmbeddedJsonFromStdout(stdout) {
  const marker = 'JSON Output';
  const markerIndex = stdout.lastIndexOf(marker);
  const candidate = markerIndex >= 0 ? stdout.slice(markerIndex + marker.length) : stdout;
  const braceIndex = candidate.indexOf('{');
  if (braceIndex < 0) {
    throw new Error('unable to locate JSON payload in command output');
  }
  return JSON.parse(candidate.slice(braceIndex).trim());
}
