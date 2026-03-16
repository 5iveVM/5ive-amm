import { readFile } from 'fs/promises';
import { existsSync } from 'fs';
import path from 'path';

const CLI_ROOT = existsSync(path.join(process.cwd(), 'templates'))
  ? process.cwd()
  : path.resolve(process.cwd(), 'five-cli');

describe('init-generated templates', () => {
  it('keeps AGENTS self-contained for the public build and local test path', async () => {
    const agents = await readFile(
      path.join(CLI_ROOT, 'templates', 'AGENTS.md'),
      'utf8',
    );

    expect(agents).toContain('This file is the complete minimum contract for building and locally validating a 5IVE project.');
    expect(agents).toContain('## 0) Policy Overrides (2026-03)');
    expect(agents).toContain('Default account serializer is `raw`.');
    expect(agents).toContain('the installed `5ive` CLI, bundled SDK, and bundled compiler are the supported toolchain');
    expect(agents).toContain('Node.js `>=18`');
    expect(agents).toContain('docs/STDLIB.md');
    expect(agents).toContain('Build with `5ive build` as the preferred project build command.');
    expect(agents).toContain('Project builds require `project.entry_point` in `five.toml`.');
    expect(agents).toContain('Run local tests with `5ive test --sdk-runner`.');
    expect(agents).not.toContain('Rust compiler');
  });

  it('keeps AGENTS reference aligned with the public CLI/SDK path', async () => {
    const reference = await readFile(
      path.join(CLI_ROOT, 'templates', 'AGENTS_REFERENCE.md'),
      'utf8',
    );

    expect(reference).toContain('the installed CLI/SDK behavior is authoritative');
    expect(reference).toContain('maintainer-only compiler workflows');
    expect(reference).toContain('## Policy Overrides (2026-03)');
    expect(reference).toContain('Default account serializer is `raw`.');
    expect(reference).toContain('spl_token::SPLToken::transfer');
    expect(reference).not.toContain('ExampleProgram.do_thing(...)');
  });

  it('keeps the AGENTS checklist anchored to the public CLI compiler path', async () => {
    const checklist = await readFile(
      path.join(CLI_ROOT, 'templates', 'AGENTS_CHECKLIST.md'),
      'utf8',
    );

    expect(checklist).toContain('the installed `5ive` CLI / bundled SDK compiler as the public validation path');
    expect(checklist).toContain('module_alias::Interface::method(...)');
    expect(checklist).toContain('standardize on `0`');
  });

  it('uses current account metadata syntax in the generated basic starter', async () => {
    const initSource = await readFile(
      path.join(CLI_ROOT, 'src', 'commands', 'init.ts'),
      'utf8',
    );

    expect(initSource).toContain('counter.authority = authority.ctx.key;');
    expect(initSource).toContain('require(counter.authority == authority.ctx.key);');
    expect(initSource).not.toContain('counter.authority = authority.key;');
    expect(initSource).not.toContain('require(counter.authority == authority.key);');
  });

  it('keeps account testing guide examples on .ctx.key syntax', async () => {
    const guide = await readFile(
      path.join(CLI_ROOT, 'docs', 'ACCOUNT_TESTING_GUIDE.md'),
      'utf8',
    );

    expect(guide).toContain('require(authority.ctx.key != from.ctx.key);');
    expect(guide).toContain('require(from.ctx.key != to.ctx.key);');
    expect(guide).not.toContain('require(authority.key != from.key);');
    expect(guide).not.toContain('require(from.key != to.key);');
  });

  it('does not ship dead pre-generated template ABI or bytecode artifacts', () => {
    const removedArtifacts = [
      path.join(CLI_ROOT, 'templates', 'amm.abi.json'),
      path.join(CLI_ROOT, 'templates', 'counter.abi.json'),
      path.join(CLI_ROOT, 'templates', 'counter.five'),
      path.join(CLI_ROOT, 'templates', 'escrow.abi.json'),
      path.join(CLI_ROOT, 'templates', 'hello_world.five'),
      path.join(CLI_ROOT, 'templates', 'multisig.abi.json'),
      path.join(CLI_ROOT, 'templates', 'nft.abi.json'),
      path.join(CLI_ROOT, 'templates', 'spl-token.abi.json'),
      path.join(CLI_ROOT, 'templates', 'spl-token.five'),
      path.join(CLI_ROOT, 'templates', 'token.five'),
      path.join(CLI_ROOT, 'templates', 'vault.abi.json'),
    ];

    for (const artifact of removedArtifacts) {
      expect(existsSync(artifact)).toBe(false);
    }
  });
});
