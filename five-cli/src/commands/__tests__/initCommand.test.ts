import { existsSync, mkdtempSync, readFileSync, rmSync } from 'fs';
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

describe('init command stdlib scaffolding', () => {
  maybeIt('generates entry_point, stdlib docs, and active stdlib imports in starter', () => {
    const cliDist = localCliDist as string;
    const repoRoot = resolveRepoRoot(cliDist);
    const tmpRoot = mkdtempSync(join(tmpdir(), 'five-cli-init-'));
    const projectDir = join(tmpRoot, 'sample');

    try {
      const initResult = spawnSync(
        'node',
        [cliDist, 'init', projectDir, '--no-git'],
        { encoding: 'utf8', cwd: repoRoot }
      );
      expect(initResult.status).toBe(0);

      const toml = readFileSync(join(projectDir, 'five.toml'), 'utf8');
      expect(toml).toContain('schema_version = 1');
      expect(toml).toContain('name = "sample"');
      expect(toml).toContain('entry_point = "src/main.v"');

      const stdlibDoc = readFileSync(join(projectDir, 'docs', 'STDLIB.md'), 'utf8');
      expect(stdlibDoc).toContain('Included modules');

      const main = readFileSync(join(projectDir, 'src/main.v'), 'utf8');
      expect(main).toContain('// use std::builtins;');
      expect(main).toContain('// use std::interfaces::spl_token;');
      expect(main).toContain('pub get_value(counter: Counter) -> u64');
    } finally {
      rmSync(tmpRoot, { recursive: true, force: true });
    }
  });
});
