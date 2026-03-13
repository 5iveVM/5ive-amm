import fs from 'fs';
import os from 'os';
import path from 'path';
import { execFileSync } from 'child_process';

function findRepoRoot(startDir) {
  let current = startDir;
  while (true) {
    if (fs.existsSync(path.join(current, 'Cargo.toml')) && fs.existsSync(path.join(current, 'five-dsl-compiler'))) {
      return current;
    }
    const parent = path.dirname(current);
    if (parent === current) {
      throw new Error(`Could not locate repo root from ${startDir}`);
    }
    current = parent;
  }
}

export function compileWithRustFiveCompiler(scriptPath, options = {}) {
  const repoRoot = findRepoRoot(path.dirname(scriptPath));
  const cliPath = path.join(repoRoot, 'five-cli', 'dist', 'index.js');
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), 'five-compile-'));
  const srcDir = path.join(tempDir, 'src');
  const entryPath = path.join(srcDir, 'main.v');
  let outputPath = path.join(tempDir, 'build', 'main.five');

  try {
    fs.mkdirSync(srcDir, { recursive: true });
    fs.copyFileSync(scriptPath, entryPath);
    fs.writeFileSync(
      path.join(tempDir, 'five.toml'),
      `schema_version = 1

[project]
name = "temp-compile"
version = "0.1.0"
source_dir = "src"
build_dir = "build"
entry_point = "src/main.v"
target = "vm"

[dependencies]
std = { package = "@5ive/std", version = "0.1.0", source = "bundled", link = "inline" }
`,
      'utf8',
    );

    const env = { ...process.env };
    if (options.disableRequireBatch) {
      env.FIVE_DISABLE_REQUIRE_BATCH = '1';
    }

    execFileSync('node', [cliPath, 'build', '--project', tempDir], {
      cwd: repoRoot,
      stdio: 'pipe',
      env,
    });

    if (!fs.existsSync(outputPath)) {
      const buildDir = path.join(tempDir, 'build');
      const candidates = fs.existsSync(buildDir)
        ? fs
            .readdirSync(buildDir)
            .filter((name) => name.endsWith('.five'))
            .map((name) => path.join(buildDir, name))
        : [];
      if (candidates.length === 0) {
        throw new Error(`No .five artifact produced in ${buildDir}`);
      }
      outputPath = candidates[0];
    }

    const artifactBuffer = fs.readFileSync(outputPath);
    const artifactText = artifactBuffer.toString('utf8');
    let bytecode = new Uint8Array(artifactBuffer);

    try {
      const parsed = JSON.parse(artifactText);
      if (parsed && typeof parsed.bytecode === 'string') {
        bytecode = new Uint8Array(Buffer.from(parsed.bytecode, 'base64'));
      }
    } catch {
      // Non-JSON artifacts are treated as raw bytecode bytes.
    }

    return {
      outputPath,
      artifactText,
      bytecode,
    };
  } catch (error) {
    const stderr = error?.stderr ? String(error.stderr) : '';
    const stdout = error?.stdout ? String(error.stdout) : '';
    const detail = [stderr, stdout].filter(Boolean).join('\n').trim();
    throw new Error(detail || error?.message || 'Rust compiler invocation failed');
  } finally {
    try {
      fs.rmSync(tempDir, { recursive: true, force: true });
    } catch {
      // ignore cleanup failures
    }
  }
}
