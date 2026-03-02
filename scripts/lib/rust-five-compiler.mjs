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

export function compileWithRustFiveCompiler(scriptPath) {
  const repoRoot = findRepoRoot(path.dirname(scriptPath));
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), 'five-compile-'));
  const outputPath = path.join(tempDir, `${path.basename(scriptPath, path.extname(scriptPath))}.five`);

  try {
    execFileSync(
      'cargo',
      [
        'run',
        '-q',
        '-p',
        'five-dsl-compiler',
        '--bin',
        'five',
        '--',
        'compile',
        scriptPath,
        '-o',
        outputPath,
        '-m',
        'deployment',
      ],
      {
        cwd: repoRoot,
        stdio: 'pipe',
      }
    );

    return {
      outputPath,
      artifactText: fs.readFileSync(outputPath, 'utf8'),
      bytecode: new Uint8Array(fs.readFileSync(outputPath)),
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
