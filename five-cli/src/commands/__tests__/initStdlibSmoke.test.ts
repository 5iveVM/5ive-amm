import { existsSync, mkdtempSync, writeFileSync, rmSync } from 'fs';
import { join, resolve, dirname } from 'path';
import { tmpdir } from 'os';
import { spawnSync } from 'child_process';

function resolveLocalCliDist(): string | null {
  const fromPkgRoot = resolve(process.cwd(), 'dist/index.js');
  if (existsSync(fromPkgRoot)) {
    return fromPkgRoot;
  }
  const fromRepoRoot = resolve(process.cwd(), 'five-cli/dist/index.js');
  if (existsSync(fromRepoRoot)) {
    return fromRepoRoot;
  }
  return null;
}

function runCli(cliDist: string, args: string[], cwd?: string) {
  return spawnSync('node', [cliDist, ...args], {
    encoding: 'utf8',
    cwd
  });
}

function resolveRepoRoot(cliDist: string): string {
  return dirname(dirname(dirname(cliDist)));
}

const localCliDist = resolveLocalCliDist();
const hasWorkingLocalCli = (() => {
  if (!localCliDist) {
    return false;
  }
  const status = spawnSync('node', [localCliDist, '--version'], {
    encoding: 'utf8',
    cwd: resolveRepoRoot(localCliDist)
  }).status;
  return status === 0;
})();
const maybeIt = hasWorkingLocalCli ? it : it.skip;

describe('init + stdlib smoke', () => {
  maybeIt('init/build/compile works with bundled stdlib interfaces', () => {
    const tmpRoot = mkdtempSync(join(tmpdir(), 'five-init-stdlib-smoke-'));
    const projectDir = join(tmpRoot, 'app');
    const cliDist = localCliDist as string;
    const repoRoot = resolveRepoRoot(cliDist);

    try {
      const initResult = runCli(cliDist, ['init', projectDir, '--no-git'], repoRoot);
      expect(initResult.status).toBe(0);

      const buildResult = runCli(cliDist, ['build', '--project', projectDir]);
      expect(`${buildResult.stdout}\n${buildResult.stderr}`).not.toContain(
        'Missing required project.entry_point'
      );
      expect(buildResult.status).toBe(0);

      const compileMainResult = runCli(cliDist, [
        'compile',
        join(projectDir, 'src/main.v'),
        '-o',
        join(projectDir, 'build/main-single.five')
      ]);
      expect(compileMainResult.status).toBe(0);

      const smokePath = join(projectDir, 'src/stdlib-interface-smoke.v');
      writeFileSync(
        smokePath,
        `pub smoke() -> u64 {
  return 1;
}
`,
        'utf8'
      );

      const compileSmokeResult = runCli(cliDist, [
        'compile',
        smokePath,
        '-o',
        join(projectDir, 'build/stdlib-interface-smoke.five')
      ]);
      expect(compileSmokeResult.status).toBe(0);
    } finally {
      rmSync(tmpRoot, { recursive: true, force: true });
    }
  });
});
