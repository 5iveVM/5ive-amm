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

function syncLocalSdk(repoRoot: string): boolean {
  const result = spawnSync('bash', ['scripts/sync-local-sdk.sh'], {
    encoding: 'utf8',
    cwd: repoRoot
  });
  return result.status === 0;
}

const localCliDist = resolveLocalCliDist();
const repoRoot = localCliDist ? resolveRepoRoot(localCliDist) : null;
const hasWorkingLocalCli = (() => {
  if (!localCliDist) {
    return false;
  }
  const status = spawnSync('node', [localCliDist, '--version'], {
    encoding: 'utf8',
    cwd: repoRoot as string
  }).status;
  return status === 0;
})();
const hasSyncedLocalSdk = hasWorkingLocalCli && repoRoot ? syncLocalSdk(repoRoot) : false;
const maybeIt = hasWorkingLocalCli && hasSyncedLocalSdk ? it : it.skip;

describe('init + stdlib smoke', () => {
  maybeIt('init/build/compile works with bundled stdlib interfaces', () => {
    const tmpRoot = mkdtempSync(join(tmpdir(), 'five-init-stdlib-smoke-'));
    const projectDir = join(tmpRoot, 'app');
    const cliDist = localCliDist as string;
    const localRepoRoot = resolveRepoRoot(cliDist);

    try {
      const initResult = runCli(cliDist, ['init', projectDir, '--no-git'], localRepoRoot);
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
        `script main {
  use std::interfaces::spl_token;
  pub fn run(source: Account, destination: Account, authority: Account) {
    spl_token::transfer(source, destination, authority, 1);
    std::interfaces::spl_token::approve(source, destination, authority, 1);
  }
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

      const legacyPath = join(projectDir, 'src/stdlib-interface-legacy.v');
      writeFileSync(
        legacyPath,
        `script main {
  use std::interfaces::spl_token;
  pub fn run(source: Account, destination: Account, authority: Account) {
    SPLToken.transfer(source, destination, authority, 1);
  }
}
`,
        'utf8'
      );
      const compileLegacyResult = runCli(cliDist, [
        'compile',
        legacyPath,
        '-o',
        join(projectDir, 'build/stdlib-interface-legacy.five')
      ]);
      expect(compileLegacyResult.status).not.toBe(0);
    } finally {
      rmSync(tmpRoot, { recursive: true, force: true });
    }
  });
});
