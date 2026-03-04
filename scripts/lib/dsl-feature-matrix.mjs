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
export const VALID_BUILTIN_CATEGORIES = new Set([
  'time',
  'account_io',
  'program_control',
  'crypto',
  'pda',
  'return_data',
  'logging',
  'memory',
  'sysvar',
]);
export const VALID_BUILTIN_LAYERS = new Set([
  'compiler',
  'bytecode_unit',
  'vm_unit',
  'runtime_unit',
  'cli_matrix',
  'wasm_matrix',
  'lsp_matrix',
  'runtime_matrix',
  'validator_localnet',
]);
export const VALID_BUILTIN_VALIDATOR_MODES = new Set([
  'none',
  'feature_matrix',
  'cli_localnet',
  'cargo_test',
]);
export const VALID_FEATURE_PRIORITIES = new Set(['A', 'B']);
export const VALID_FEATURE_INVENTORY_STATUSES = new Set([
  'uncataloged',
  'matrix_candidate',
  'unit_only',
  'covered',
]);

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

export function builtinMatrixPath(repoRoot) {
  return path.join(repoRoot, 'testing', 'dsl-builtin-matrix.json');
}

export function featureInventoryPath(repoRoot) {
  return path.join(repoRoot, 'testing', 'dsl-feature-inventory.json');
}

export function canonicalFixtureRoot(repoRoot) {
  return path.join(repoRoot, 'five-cli', 'test-scripts');
}

export function collectCanonicalFixtures(repoRoot) {
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
      out.push(path.relative(root, fullPath).split(path.sep).join('/'));
    }
  }

  walk(root);
  out.sort();
  return out;
}

export function loadDslFeatureMatrix(repoRoot) {
  const file = matrixPath(repoRoot);
  const raw = fs.readFileSync(file, 'utf8');
  const parsed = JSON.parse(raw);
  validateDslFeatureMatrix(parsed, repoRoot);
  return parsed;
}

export function loadDslBuiltinMatrix(repoRoot) {
  const file = builtinMatrixPath(repoRoot);
  const raw = fs.readFileSync(file, 'utf8');
  const parsed = JSON.parse(raw);
  validateDslBuiltinMatrix(parsed, repoRoot);
  return parsed;
}

export function loadDslFeatureInventory(repoRoot) {
  const file = featureInventoryPath(repoRoot);
  const raw = fs.readFileSync(file, 'utf8');
  const parsed = JSON.parse(raw);
  validateDslFeatureInventory(parsed, repoRoot);
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

export function validateDslBuiltinMatrix(matrix, repoRoot) {
  if (!matrix || typeof matrix !== 'object') {
    throw new Error('DSL builtin matrix must be a JSON object');
  }
  if (!Array.isArray(matrix.builtin_groups) || matrix.builtin_groups.length === 0) {
    throw new Error('DSL builtin matrix builtin_groups must be a non-empty array');
  }
  if (!Array.isArray(matrix.builtins) || matrix.builtins.length === 0) {
    throw new Error('DSL builtin matrix builtins must be a non-empty array');
  }

  const groupIds = new Set();
  for (const group of matrix.builtin_groups) {
    if (!group?.id || typeof group.id !== 'string') {
      throw new Error('every builtin group must define a string id');
    }
    if (groupIds.has(group.id)) {
      throw new Error(`duplicate builtin group id: ${group.id}`);
    }
    if (!VALID_BUILTIN_CATEGORIES.has(group.id)) {
      throw new Error(`builtin group ${group.id} has invalid category id`);
    }
    if (!group.description || typeof group.description !== 'string') {
      throw new Error(`builtin group ${group.id} must define a description`);
    }
    groupIds.add(group.id);
  }

  const builtinIds = new Set();
  const builtinNames = new Set();
  for (const builtin of matrix.builtins) {
    if (!builtin?.id || typeof builtin.id !== 'string') {
      throw new Error('every builtin must define a string id');
    }
    if (builtinIds.has(builtin.id)) {
      throw new Error(`duplicate builtin id: ${builtin.id}`);
    }
    builtinIds.add(builtin.id);

    if (!builtin.name || typeof builtin.name !== 'string') {
      throw new Error(`builtin ${builtin.id} must define a name`);
    }
    if (builtinNames.has(builtin.name)) {
      throw new Error(`duplicate builtin name: ${builtin.name}`);
    }
    builtinNames.add(builtin.name);

    if (!builtin.module || typeof builtin.module !== 'string') {
      throw new Error(`builtin ${builtin.id} must define a module`);
    }
    if (!groupIds.has(builtin.category)) {
      throw new Error(`builtin ${builtin.id} references unknown category ${builtin.category}`);
    }
    if (!builtin.wrapper_of || typeof builtin.wrapper_of !== 'string') {
      throw new Error(`builtin ${builtin.id} must define wrapper_of`);
    }
    if (!Number.isInteger(builtin.arity) || builtin.arity < 0) {
      throw new Error(`builtin ${builtin.id} must define a non-negative integer arity`);
    }

    for (const flag of [
      'requires_accounts',
      'requires_runtime_buffers',
      'requires_runtime_sysvar',
      'requires_signature_material',
      'runtime_applicable',
      'validator_applicable',
    ]) {
      if (typeof builtin[flag] !== 'boolean') {
        throw new Error(`builtin ${builtin.id} flag ${flag} must be boolean`);
      }
    }

    if (!builtin.layers || typeof builtin.layers !== 'object') {
      throw new Error(`builtin ${builtin.id} must define layers`);
    }
    for (const [layer, enabled] of Object.entries(builtin.layers)) {
      if (!VALID_BUILTIN_LAYERS.has(layer)) {
        throw new Error(`builtin ${builtin.id} uses invalid layer ${layer}`);
      }
      if (typeof enabled !== 'boolean') {
        throw new Error(`builtin ${builtin.id} layer ${layer} must be boolean`);
      }
    }

    if (!Array.isArray(builtin.unit_suites) || builtin.unit_suites.length === 0) {
      throw new Error(`builtin ${builtin.id} must define unit_suites`);
    }
    for (const suite of builtin.unit_suites) {
      if (typeof suite !== 'string' || suite.length === 0) {
        throw new Error(`builtin ${builtin.id} unit_suites entries must be strings`);
      }
    }

    if (builtin.matrix_scenario != null && typeof builtin.matrix_scenario !== 'string') {
      throw new Error(`builtin ${builtin.id} matrix_scenario must be a string or null`);
    }
    if (builtin.validator_mode != null && !VALID_BUILTIN_VALIDATOR_MODES.has(builtin.validator_mode)) {
      throw new Error(
        `builtin ${builtin.id} has invalid validator_mode ${builtin.validator_mode}`
      );
    }
    if (builtin.validator_target != null && typeof builtin.validator_target !== 'string') {
      throw new Error(`builtin ${builtin.id} validator_target must be a string or null`);
    }
    if (builtin.layers.validator_localnet === true) {
      if (!builtin.validator_mode || builtin.validator_mode === 'none') {
        throw new Error(
          `builtin ${builtin.id} enables validator_localnet without validator_mode`
        );
      }
      if (!builtin.validator_target) {
        throw new Error(
          `builtin ${builtin.id} enables validator_localnet without validator_target`
        );
      }
    }
    if (
      builtin.expected_limitations != null &&
      typeof builtin.expected_limitations !== 'string'
    ) {
      throw new Error(`builtin ${builtin.id} expected_limitations must be a string or null`);
    }
  }

  if (repoRoot) {
    const stdlibBuiltins = loadStdlibBuiltinNames(repoRoot);
    for (const builtinName of stdlibBuiltins) {
      if (!builtinNames.has(builtinName)) {
        throw new Error(`stdlib builtin ${builtinName} is missing from dsl-builtin-matrix.json`);
      }
    }
    for (const builtinName of builtinNames) {
      if (!stdlibBuiltins.has(builtinName)) {
        throw new Error(`builtin matrix entry ${builtinName} is not exported by five-stdlib/std/builtins.v`);
      }
    }
  }
}

export function validateDslFeatureInventory(inventory, repoRoot) {
  if (!inventory || typeof inventory !== 'object') {
    throw new Error('DSL feature inventory must be a JSON object');
  }
  if (!Array.isArray(inventory.feature_families) || inventory.feature_families.length === 0) {
    throw new Error('DSL feature inventory feature_families must be a non-empty array');
  }
  if (!Array.isArray(inventory.fixtures) || inventory.fixtures.length === 0) {
    throw new Error('DSL feature inventory fixtures must be a non-empty array');
  }

  const familyIds = new Set();
  for (const family of inventory.feature_families) {
    if (!family?.id || typeof family.id !== 'string') {
      throw new Error('every feature family must define a string id');
    }
    if (familyIds.has(family.id)) {
      throw new Error(`duplicate feature family id: ${family.id}`);
    }
    familyIds.add(family.id);
    if (!family.description || typeof family.description !== 'string') {
      throw new Error(`feature family ${family.id} must define a description`);
    }
    if (!family.owned_by_category || typeof family.owned_by_category !== 'string') {
      throw new Error(`feature family ${family.id} must define owned_by_category`);
    }
    if (!VALID_FEATURE_PRIORITIES.has(family.priority)) {
      throw new Error(`feature family ${family.id} has invalid priority ${family.priority}`);
    }
    if (!Array.isArray(family.required_layers) || family.required_layers.length === 0) {
      throw new Error(`feature family ${family.id} must define required_layers`);
    }
    for (const layer of family.required_layers) {
      if (!VALID_LAYERS.has(layer)) {
        throw new Error(`feature family ${family.id} has invalid required layer ${layer}`);
      }
    }
    if (!Number.isInteger(family.phase) || family.phase < 1) {
      throw new Error(`feature family ${family.id} must define a positive integer phase`);
    }
  }

  const fixturePaths = new Set();
  for (const fixture of inventory.fixtures) {
    if (!fixture?.path || typeof fixture.path !== 'string') {
      throw new Error('every feature inventory fixture must define a path');
    }
    if (fixturePaths.has(fixture.path)) {
      throw new Error(`duplicate feature inventory fixture path: ${fixture.path}`);
    }
    fixturePaths.add(fixture.path);
    if (!familyIds.has(fixture.family)) {
      throw new Error(
        `feature inventory fixture ${fixture.path} references unknown family ${fixture.family}`
      );
    }
    if (!VALID_FEATURE_INVENTORY_STATUSES.has(fixture.status)) {
      throw new Error(
        `feature inventory fixture ${fixture.path} has invalid status ${fixture.status}`
      );
    }
    if (!fixture.coverage_notes || typeof fixture.coverage_notes !== 'string') {
      throw new Error(`feature inventory fixture ${fixture.path} must define coverage_notes`);
    }
    if (!fixture.preferred_runner || typeof fixture.preferred_runner !== 'string') {
      throw new Error(`feature inventory fixture ${fixture.path} must define preferred_runner`);
    }
    const resolvedPath = resolveInventoryFixturePath(repoRoot, fixture.path);
    if (!fs.existsSync(resolvedPath)) {
      throw new Error(
        `feature inventory fixture ${fixture.path} not found at ${resolvedPath}`
      );
    }
  }
}

export function resolveScenarioSource(repoRoot, scenario) {
  return path.join(canonicalFixtureRoot(repoRoot), scenario.source);
}

export function resolveInventoryFixturePath(repoRoot, fixturePath) {
  const normalized = fixturePath.split(path.sep).join('/');
  if (normalized.startsWith('__root__/')) {
    return path.join(canonicalFixtureRoot(repoRoot), normalized.slice('__root__/'.length));
  }
  return path.join(canonicalFixtureRoot(repoRoot), normalized);
}

export function loadStdlibBuiltinNames(repoRoot) {
  const stdlibPath = path.join(repoRoot, 'five-stdlib', 'std', 'builtins.v');
  const content = fs.readFileSync(stdlibPath, 'utf8');
  const names = new Set();
  for (const match of content.matchAll(/pub\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(/g)) {
    names.add(match[1]);
  }
  return names;
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
  return collectCanonicalFixtures(repoRoot).filter((fixture) => !tracked.has(fixture));
}

export function findUnclassifiedCanonicalFixtures(repoRoot, matrix, inventory) {
  const tracked = new Set(matrix.scenarios.map((scenario) => scenario.source));
  const classified = new Set(
    inventory.fixtures.map((fixture) =>
      fixture.path.startsWith('__root__/') ? fixture.path.slice('__root__/'.length) : fixture.path
    )
  );

  return collectCanonicalFixtures(repoRoot).filter(
    (fixture) => !tracked.has(fixture) && !classified.has(fixture)
  );
}

export function assertNoUnclassifiedCanonicalFixtures(repoRoot, matrix, inventory) {
  const unclassified = findUnclassifiedCanonicalFixtures(repoRoot, matrix, inventory);
  if (unclassified.length > 0) {
    throw new Error(
      `canonical DSL fixtures must be tracked by dsl-feature-matrix.json or dsl-feature-inventory.json; missing entries: ${unclassified.join(', ')}`
    );
  }
}

export function parseEmbeddedJsonFromStdout(stdout) {
  const marker = 'JSON Output';
  const markerIndex = stdout.lastIndexOf(marker);
  const rawCandidate = markerIndex >= 0 ? stdout.slice(markerIndex + marker.length) : stdout;
  const candidate = rawCandidate.replace(/\u001b\[[0-9;]*m/g, '');
  const startIndexes = [];
  for (let i = 0; i < candidate.length; i += 1) {
    if (candidate[i] === '{' && (i === 0 || candidate[i - 1] === '\n')) {
      startIndexes.push(i);
    }
  }

  for (let i = startIndexes.length - 1; i >= 0; i -= 1) {
    const start = startIndexes[i];
    let depth = 0;
    let inString = false;
    let escaped = false;

    for (let cursor = start; cursor < candidate.length; cursor += 1) {
      const char = candidate[cursor];

      if (inString) {
        if (escaped) {
          escaped = false;
        } else if (char === '\\') {
          escaped = true;
        } else if (char === '"') {
          inString = false;
        }
        continue;
      }

      if (char === '"') {
        inString = true;
        continue;
      }

      if (char === '{') {
        depth += 1;
        continue;
      }

      if (char === '}') {
        depth -= 1;
        if (depth === 0) {
          const objectSource = candidate.slice(start, cursor + 1);
          try {
            return JSON.parse(objectSource);
          } catch {
            break;
          }
        }
      }
    }
  }

  throw new Error('unable to locate JSON payload in command output');
}
