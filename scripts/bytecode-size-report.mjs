#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import { spawnSync } from 'node:child_process';

const ROOT = process.cwd();
const REBUILD = process.argv.includes('--rebuild');
const JSON_OUT = process.argv.includes('--json');
const BASELINE_PATH = path.join(ROOT, 'scripts', 'bytecode-baseline.json');

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
  const rel = path.relative(ROOT, projectDir);
  const result = spawnSync(
    'node',
    ['five-cli/dist/index.js', 'build', '--project', projectDir],
    { cwd: ROOT, stdio: 'pipe', encoding: 'utf8' },
  );
  if (result.status !== 0) {
    const stderr = (result.stderr || '').trim();
    const stdout = (result.stdout || '').trim();
    throw new Error(
      `build failed for ${rel}\n${stderr || stdout || 'unknown error'}`,
    );
  }
}

function readArtifact(projectDir) {
  const manifestPath = path.join(projectDir, '.five', 'build.json');
  const project = path.basename(projectDir);
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
    const raw = fs.readFileSync(fallback, 'utf8');
    const artifact = JSON.parse(raw);
    const bytecode = Buffer.from(artifact.bytecode || '', 'base64');
    const abi = artifact.abi || {};
    const functions = Array.isArray(abi.functions) ? abi.functions : [];
    return {
      project,
      artifactPath: path.relative(ROOT, fallback),
      target: 'unknown',
      bytecodeBytes: bytecode.length,
      opcodeBuckets: analyzeOpcodeBuckets(bytecode),
      abiBytes: Buffer.byteLength(JSON.stringify(abi), 'utf8'),
      functionCount: functions.length,
      totalFileBytes: fs.statSync(fallback).size,
    };
  }

  const manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf8'));
  const artifactPath = path.isAbsolute(manifest.artifact_path)
    ? manifest.artifact_path
    : path.join(projectDir, manifest.artifact_path);
  if (!fs.existsSync(artifactPath)) {
    const fallback = findFallbackArtifact();
    if (fallback) {
      const rawFallback = fs.readFileSync(fallback, 'utf8');
      const artifactFallback = JSON.parse(rawFallback);
      const bytecodeFallback = Buffer.from(artifactFallback.bytecode || '', 'base64');
      const abiFallback = artifactFallback.abi || {};
      const functionsFallback = Array.isArray(abiFallback.functions)
        ? abiFallback.functions
        : [];
      return {
        project,
        artifactPath: path.relative(ROOT, fallback),
        target: manifest.target || 'unknown',
        bytecodeBytes: bytecodeFallback.length,
        opcodeBuckets: analyzeOpcodeBuckets(bytecodeFallback),
        abiBytes: Buffer.byteLength(JSON.stringify(abiFallback), 'utf8'),
        functionCount: functionsFallback.length,
        totalFileBytes: fs.statSync(fallback).size,
      };
    }
    return {
      project,
      missing: true,
      reason: `missing artifact at ${artifactPath}`,
    };
  }

  const raw = fs.readFileSync(artifactPath, 'utf8');
  const artifact = JSON.parse(raw);
  const bytecode = Buffer.from(artifact.bytecode || '', 'base64');
  const abi = artifact.abi || {};
  const abiBytes = Buffer.byteLength(JSON.stringify(abi), 'utf8');
  const functions = Array.isArray(abi.functions) ? abi.functions : [];

  return {
    project,
    artifactPath: path.relative(ROOT, artifactPath),
    target: manifest.target || 'unknown',
    bytecodeBytes: bytecode.length,
    opcodeBuckets: analyzeOpcodeBuckets(bytecode),
    abiBytes,
    functionCount: functions.length,
    totalFileBytes: fs.statSync(artifactPath).size,
  };
}

function printTable(rows) {
  const header =
    'project'.padEnd(20) +
    'bytecode'.padStart(10) +
    '  ' +
    'abi'.padStart(8) +
    '  ' +
    'funcs'.padStart(6) +
    '  ' +
    'target'.padStart(7) +
    '  ' +
    'field/br/local/imm'.padStart(20);
  console.log(header);
  console.log('-'.repeat(header.length));
  for (const row of rows) {
    if (row.missing) {
      console.log(`${row.project.padEnd(20)}MISSING  ${row.reason}`);
      continue;
    }
    console.log(
      row.project.padEnd(20) +
        String(row.bytecodeBytes).padStart(10) +
        '  ' +
        String(row.abiBytes).padStart(8) +
        '  ' +
        String(row.functionCount).padStart(6) +
        '  ' +
        String(row.target).padStart(7) +
        '  ' +
        `${row.opcodeBuckets.field_ops}/${row.opcodeBuckets.branch_ops}/${row.opcodeBuckets.call_local_ops}/${row.opcodeBuckets.immediate_ops}`.padStart(20),
    );
  }
}

function main() {
  const projects = discoverProjects();
  if (projects.length === 0) {
    throw new Error('no 5ive-* projects found');
  }

  if (REBUILD) {
    for (const dir of projects) {
      buildProject(dir);
    }
  }

  const rows = projects.map(readArtifact);
  const complete = rows
    .filter((r) => !r.missing)
    .sort((a, b) => b.bytecodeBytes - a.bytecodeBytes);

  const topBytecode = complete.slice(0, 5).map((r) => ({
    project: r.project,
    bytes: r.bytecodeBytes,
  }));
  const topAbi = [...complete]
    .sort((a, b) => b.abiBytes - a.abiBytes)
    .slice(0, 5)
    .map((r) => ({ project: r.project, bytes: r.abiBytes }));
  const baseline = fs.existsSync(BASELINE_PATH)
    ? JSON.parse(fs.readFileSync(BASELINE_PATH, 'utf8'))
    : [];
  const baselineByProject = new Map((baseline || []).map((b) => [b.project, b]));

  if (JSON_OUT) {
    console.log(JSON.stringify({ rows, topBytecode, topAbi }, null, 2));
    return;
  }

  printTable(rows);
  console.log('\nTop bytecode offenders:');
  for (const item of topBytecode) {
    console.log(`- ${item.project}: ${item.bytes} bytes`);
  }
  console.log('\nTop ABI payload offenders:');
  for (const item of topAbi) {
    console.log(`- ${item.project}: ${item.bytes} bytes`);
  }
  console.log('\nBucket deltas vs baseline (field/branch/local/imm):');
  for (const row of complete) {
    const base = baselineByProject.get(row.project);
    if (!base?.opcodeBuckets) {
      console.log(`- ${row.project}: ${row.opcodeBuckets.field_ops}/${row.opcodeBuckets.branch_ops}/${row.opcodeBuckets.call_local_ops}/${row.opcodeBuckets.immediate_ops}`);
      continue;
    }
    const d = {
      field: row.opcodeBuckets.field_ops - base.opcodeBuckets.field_ops,
      branch: row.opcodeBuckets.branch_ops - base.opcodeBuckets.branch_ops,
      local: row.opcodeBuckets.call_local_ops - base.opcodeBuckets.call_local_ops,
      imm: row.opcodeBuckets.immediate_ops - base.opcodeBuckets.immediate_ops,
    };
    console.log(`- ${row.project}: ${d.field >= 0 ? '+' : ''}${d.field}/${d.branch >= 0 ? '+' : ''}${d.branch}/${d.local >= 0 ? '+' : ''}${d.local}/${d.imm >= 0 ? '+' : ''}${d.imm}`);
  }
}

main();
