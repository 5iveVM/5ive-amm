#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import { spawnSync } from 'node:child_process';

const ROOT = process.cwd();
const REBUILD = process.argv.includes('--rebuild');
const WRITE_BASELINE = process.argv.includes('--write-baseline');
const BASELINE_PATH = path.join(ROOT, 'scripts', 'bytecode-baseline.json');
const ALLOW_GROWTH = process.argv.includes('--allow-growth');

function analyzeOpcodeBuckets(bytecode) {
  const fieldOps = new Set([0x43, 0x44, 0x49, 0x4a, 0x4b, 0x4c, 0xe5, 0xe6, 0xe8, 0xe9, 0xea, 0xeb]);
  const branchOps = new Set([0x01, 0x02, 0x03, 0x0d, 0x0e, 0x0f, 0xec, 0xed, 0xee, 0xef]);
  const callLocalOps = new Set([0x05, 0x10, 0x11, 0x9b, 0x9c, 0xa0, 0xa1, 0xd0, 0xd1, 0xd2, 0xd3, 0xd4, 0xd5, 0xd6, 0xd7]);
  const immediateOps = new Set([0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4a, 0x4b, 0x4c]);
  let field_ops = 0;
  let branch_ops = 0;
  let call_local_ops = 0;
  let immediate_ops = 0;
  for (const op of bytecode) {
    if (fieldOps.has(op)) field_ops += 1;
    if (branchOps.has(op)) branch_ops += 1;
    if (callLocalOps.has(op)) call_local_ops += 1;
    if (immediateOps.has(op)) immediate_ops += 1;
  }
  return { field_ops, branch_ops, call_local_ops, immediate_ops };
}

function discoverProjects() {
  return fs
    .readdirSync(ROOT, { withFileTypes: true })
    .filter((d) => d.isDirectory() && d.name.startsWith('5ive-'))
    .map((d) => path.join(ROOT, d.name))
    .filter((dir) => fs.existsSync(path.join(dir, 'five.toml')))
    .sort();
}

function buildProject(projectDir) {
  const result = spawnSync(
    'node',
    ['five-cli/dist/index.js', 'build', '--project', projectDir],
    { cwd: ROOT, stdio: 'pipe', encoding: 'utf8' },
  );
  if (result.status !== 0) {
    throw new Error(
      `build failed for ${path.relative(ROOT, projectDir)}: ${result.stderr || result.stdout}`,
    );
  }
}

function parseEntryPubFunctions(entryFilePath) {
  if (!fs.existsSync(entryFilePath)) return new Set();
  const src = fs.readFileSync(entryFilePath, 'utf8');
  const regex = /^\s*pub\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(/gm;
  const names = new Set();
  for (let m = regex.exec(src); m; m = regex.exec(src)) {
    names.add(m[1]);
  }
  return names;
}

function collectProject(projectDir) {
  const project = path.basename(projectDir);
  const manifestPath = path.join(projectDir, '.five', 'build.json');
  const findFallbackArtifact = () => {
    const buildDir = path.join(projectDir, 'build');
    if (!fs.existsSync(buildDir)) return null;
    const candidates = fs
      .readdirSync(buildDir)
      .filter((name) => name.endsWith('.five'))
      .map((name) => path.join(buildDir, name))
      .sort();
    return candidates[0] || null;
  };

  if (!fs.existsSync(manifestPath)) {
    const fallback = findFallbackArtifact();
    if (!fallback) {
      return { project, missing: true, reason: 'missing .five/build.json and build/*.five' };
    }
    const artifact = JSON.parse(fs.readFileSync(fallback, 'utf8'));
    const abi = artifact.abi || {};
    const functions = Array.isArray(abi.functions) ? abi.functions : [];
    const exported = functions.map((f) => String(f?.name || ''));
    return {
      project,
      bytecodeBytes: Buffer.from(artifact.bytecode || '', 'base64').length,
      opcodeBuckets: analyzeOpcodeBuckets(Buffer.from(artifact.bytecode || '', 'base64')),
      abiBytes: Buffer.byteLength(JSON.stringify(abi), 'utf8'),
      functionCount: exported.length,
      exported,
      unexpected: exported.filter((name) => name.includes('::')),
      entryPoint: 'unknown',
      artifactPath: path.relative(ROOT, fallback),
    };
  }

  const manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf8'));
  const artifactPath = path.isAbsolute(manifest.artifact_path)
    ? manifest.artifact_path
    : path.join(projectDir, manifest.artifact_path);
  if (!fs.existsSync(artifactPath)) {
    const fallback = findFallbackArtifact();
    if (!fallback) {
      return { project, missing: true, reason: `missing artifact: ${artifactPath}` };
    }
    const artifact = JSON.parse(fs.readFileSync(fallback, 'utf8'));
    const abi = artifact.abi || {};
    const functions = Array.isArray(abi.functions) ? abi.functions : [];
    const exported = functions.map((f) => String(f?.name || ''));
    const entryPoint = manifest.entry_point || 'src/main.v';
    const entryFile = path.join(projectDir, entryPoint);
    const expectedPub = parseEntryPubFunctions(entryFile);
    const unexpected = exported.filter(
      (name) => name.includes('::') || (name !== '__init' && !expectedPub.has(name)),
    );
    return {
      project,
      bytecodeBytes: Buffer.from(artifact.bytecode || '', 'base64').length,
      opcodeBuckets: analyzeOpcodeBuckets(Buffer.from(artifact.bytecode || '', 'base64')),
      abiBytes: Buffer.byteLength(JSON.stringify(abi), 'utf8'),
      functionCount: exported.length,
      exported,
      unexpected,
      entryPoint,
      artifactPath: path.relative(ROOT, fallback),
    };
  }

  const entryPoint = manifest.entry_point || 'src/main.v';
  const entryFile = path.join(projectDir, entryPoint);
  const expectedPub = parseEntryPubFunctions(entryFile);

  const artifact = JSON.parse(fs.readFileSync(artifactPath, 'utf8'));
  const abi = artifact.abi || {};
  const functions = Array.isArray(abi.functions) ? abi.functions : [];
  const exported = functions.map((f) => String(f?.name || ''));
  const unexpected = exported.filter(
    (name) => name.includes('::') || (name !== '__init' && !expectedPub.has(name)),
  );

  return {
    project,
    bytecodeBytes: Buffer.from(artifact.bytecode || '', 'base64').length,
    opcodeBuckets: analyzeOpcodeBuckets(Buffer.from(artifact.bytecode || '', 'base64')),
    abiBytes: Buffer.byteLength(JSON.stringify(abi), 'utf8'),
    functionCount: exported.length,
    exported,
    unexpected,
    entryPoint,
    artifactPath: path.relative(ROOT, artifactPath),
  };
}

function loadBaseline() {
  if (!fs.existsSync(BASELINE_PATH)) return null;
  return JSON.parse(fs.readFileSync(BASELINE_PATH, 'utf8'));
}

function writeBaseline(rows) {
  const baseline = rows
    .filter((r) => !r.missing)
    .map((r) => ({
      project: r.project,
      bytecodeBytes: r.bytecodeBytes,
      abiBytes: r.abiBytes,
      functionCount: r.functionCount,
      exported: r.exported,
      opcodeBuckets: r.opcodeBuckets,
    }))
    .sort((a, b) => a.project.localeCompare(b.project));
  fs.writeFileSync(BASELINE_PATH, `${JSON.stringify(baseline, null, 2)}\n`);
  console.log(`Wrote baseline: ${path.relative(ROOT, BASELINE_PATH)}`);
}

function main() {
  const projects = discoverProjects();
  if (REBUILD) {
    for (const projectDir of projects) {
      buildProject(projectDir);
    }
  }

  const rows = projects.map(collectProject);
  const baseline = loadBaseline();
  const baselineByProject = new Map((baseline || []).map((b) => [b.project, b]));

  const missing = rows.filter((r) => r.missing);
  if (missing.length > 0) {
    for (const m of missing) {
      console.error(`[missing] ${m.project}: ${m.reason}`);
    }
    process.exit(1);
  }

  const offenders = rows.filter((r) => r.unexpected.length > 0);
  const growth = [];
  for (const row of [...rows].sort((a, b) => b.bytecodeBytes - a.bytecodeBytes)) {
    const base = baselineByProject.get(row.project);
    const delta = base ? row.bytecodeBytes - base.bytecodeBytes : 0;
    const deltaText = base ? ` (delta ${delta >= 0 ? '+' : ''}${delta})` : '';
    if (base && delta > 0) {
      growth.push({ project: row.project, delta });
    }
    const bucketDelta = base?.opcodeBuckets
      ? {
          field: row.opcodeBuckets.field_ops - base.opcodeBuckets.field_ops,
          branch: row.opcodeBuckets.branch_ops - base.opcodeBuckets.branch_ops,
          local: row.opcodeBuckets.call_local_ops - base.opcodeBuckets.call_local_ops,
          imm: row.opcodeBuckets.immediate_ops - base.opcodeBuckets.immediate_ops,
        }
      : null;
    const bucketText = bucketDelta
      ? `${bucketDelta.field >= 0 ? '+' : ''}${bucketDelta.field}/${bucketDelta.branch >= 0 ? '+' : ''}${bucketDelta.branch}/${bucketDelta.local >= 0 ? '+' : ''}${bucketDelta.local}/${bucketDelta.imm >= 0 ? '+' : ''}${bucketDelta.imm}`
      : `${row.opcodeBuckets.field_ops}/${row.opcodeBuckets.branch_ops}/${row.opcodeBuckets.call_local_ops}/${row.opcodeBuckets.immediate_ops}`;
    console.log(
      `${row.project}: bytecode=${row.bytecodeBytes}${deltaText}, abi=${row.abiBytes}, funcs=${row.functionCount}` +
        `, buckets(field/branch/local/imm)=${bucketText}`,
    );
  }

  if (WRITE_BASELINE) {
    writeBaseline(rows);
  }

  if (offenders.length > 0) {
    console.error('\nUnexpected ABI exports detected:');
    for (const row of offenders) {
      console.error(`- ${row.project}: ${row.unexpected.join(', ')}`);
    }
    process.exit(1);
  }

  if (!ALLOW_GROWTH && growth.length > 0) {
    console.error('\nUnexpected bytecode growth detected:');
    for (const item of growth) {
      console.error(`- ${item.project}: +${item.delta} bytes`);
    }
    console.error('\nUpdate baseline intentionally with: node scripts/check-bytecode-abi-surface.mjs --rebuild --write-baseline');
    process.exit(1);
  }

  console.log('\nABI surface check passed (exports align with entrypoint pub functions).');
}

main();
