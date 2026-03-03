import { existsSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'fs';
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

function runLocalCli(cliDist: string, args: string[], cwd?: string) {
  return spawnSync('node', [cliDist, ...args], {
    encoding: 'utf8',
    cwd,
  });
}

function hasGlobalFiveCli(): boolean {
  return spawnSync('5ive', ['--version'], {
    encoding: 'utf8',
  }).status === 0;
}

function runGlobalCli(args: string[], cwd?: string) {
  return spawnSync('5ive', args, {
    encoding: 'utf8',
    cwd,
  });
}

const localCliDist = resolveLocalCliDist();
const hasWorkingLocalCliDist =
  process.env.RUN_CLI_DIST_SMOKE === '1' &&
  !!localCliDist &&
  runLocalCli(localCliDist, ['--version'], resolveRepoRoot(localCliDist)).status === 0;

const hasWorkingGlobalCli =
  process.env.RUN_GLOBAL_CLI_SMOKE === '1' &&
  hasGlobalFiveCli();

const maybeLocalIt = hasWorkingLocalCliDist ? it : it.skip;
const maybeGlobalIt = hasWorkingGlobalCli ? it : it.skip;

describe('init + stdlib smoke', () => {
  maybeLocalIt('built local CLI dist supports the current scaffold user journey', () => {
    const tmpRoot = mkdtempSync(join(tmpdir(), 'five-cli-dist-smoke-'));
    const projectDir = join(tmpRoot, 'app');
    const cliDist = localCliDist as string;
    const repoRoot = resolveRepoRoot(cliDist);

    try {
      const initResult = runLocalCli(cliDist, ['init', projectDir, '--no-git'], repoRoot);
      expect(initResult.status).toBe(0);

      const toml = readFileSync(join(projectDir, 'five.toml'), 'utf8');
      expect(toml).toContain('entry_point = "src/main.v"');

      const main = readFileSync(join(projectDir, 'src/main.v'), 'utf8');
      expect(main).toContain('authority.ctx.key');

      const buildResult = runLocalCli(cliDist, ['build', '--project', projectDir], repoRoot);
      expect(buildResult.status).toBe(0);

      const builtArtifact = JSON.parse(
        readFileSync(join(projectDir, 'build/main.five'), 'utf8'),
      );
      expect(typeof builtArtifact.bytecode).toBe('string');
      expect(builtArtifact.bytecode.length).toBeGreaterThan(0);
      expect(Array.isArray(builtArtifact.abi?.functions)).toBe(true);
      expect(
        builtArtifact.abi.functions.some((fn: { name?: string }) => fn.name === 'increment'),
      ).toBe(true);

      const smokePath = join(projectDir, 'src/current-dsl-smoke.v');
      writeFileSync(
        smokePath,
        `account Counter {
  authority: pubkey;
  seen: u64;
}

use std::interfaces::spl_token;

pub init_counter(counter: Counter @mut, authority: account @signer) {
  counter.authority = authority.ctx.key;
  counter.seen = authority.ctx.lamports;
}

pub transfer_one(
  source: account @mut,
  destination: account @mut,
  authority: account @signer
) {
  spl_token::transfer(source, destination, authority, 1);
}
`,
        'utf8',
      );

      const compileResult = runLocalCli(
        cliDist,
        ['compile', smokePath, '-o', join(projectDir, 'build/current-dsl-smoke.five')],
        repoRoot,
      );
      expect(compileResult.status).toBe(0);

      const compiledArtifact = JSON.parse(
        readFileSync(join(projectDir, 'build/current-dsl-smoke.five'), 'utf8'),
      );
      expect(typeof compiledArtifact.bytecode).toBe('string');
      expect(compiledArtifact.bytecode.length).toBeGreaterThan(0);
      expect(Array.isArray(compiledArtifact.abi?.functions)).toBe(true);
      expect(
        compiledArtifact.abi.functions.some((fn: { name?: string }) => fn.name === 'transfer_one'),
      ).toBe(true);
    } finally {
      rmSync(tmpRoot, { recursive: true, force: true });
    }
  });

  maybeGlobalIt('installed global 5ive CLI matches the current scaffold flow', () => {
    const tmpRoot = mkdtempSync(join(tmpdir(), 'five-global-cli-smoke-'));
    const projectDir = join(tmpRoot, 'app');

    try {
      const initResult = runGlobalCli(['init', projectDir, '--no-git']);
      expect(initResult.status).toBe(0);

      const buildResult = runGlobalCli(['build', '--project', projectDir]);
      expect(buildResult.status).toBe(0);
    } finally {
      rmSync(tmpRoot, { recursive: true, force: true });
    }
  });
});
