import { mkdtemp, writeFile, mkdir } from 'fs/promises';
import { join, resolve } from 'path';
import { tmpdir } from 'os';

import { loadProjectConfig, writeBuildManifest, loadBuildManifest } from '../ProjectLoader.js';

describe('ProjectLoader', () => {
  const createTempDir = async () => {
    return await mkdtemp(join(tmpdir(), 'five-project-'));
  };

  const writeConfig = async (dir: string, content: string) => {
    await writeFile(join(dir, 'five.toml'), content);
  };

  it('loads project config with defaults and paths', async () => {
    const dir = await createTempDir();
    await writeConfig(
      dir,
      `
schema_version = 1

[project]
name = "demo"
version = "0.1.0"
source_dir = "src"
build_dir = "build"
entry_point = "src/main.v"
target = "vm"

[build]
output_artifact_name = "demo-artifact"
`
    );

    const loaded = await loadProjectConfig(undefined, dir);
    expect(loaded).not.toBeNull();
    if (!loaded) return;

    expect(loaded.config.name).toBe('demo');
    expect(loaded.config.entryPoint).toBe('src/main.v');
    expect(loaded.config.buildDir).toBe('build');
    expect(loaded.config.outputArtifactName).toBe('demo-artifact');
  });

  it('writes and reads manifest in .five/build.json', async () => {
    const dir = await createTempDir();
    await mkdir(resolve(dir, '.five'), { recursive: true });

    const manifest = {
      artifact_path: 'build/demo.five',
      abi_path: 'build/demo.abi.json',
      compiler_version: 'test',
      source_files: ['src/main.v'],
      target: 'vm',
      timestamp: new Date().toISOString(),
      hash: 'abc123',
      format: 'five' as const,
      entry_point: 'src/main.v',
      source_dir: 'src'
    };

    await writeBuildManifest(dir, manifest);
    const loaded = await loadBuildManifest(dir);
    expect(loaded).toEqual(manifest);
  });

  it('discovers five.toml from parent directories', async () => {
    const root = await createTempDir();
    await writeConfig(
      root,
      `
schema_version = 1

[project]
name = "root-project"
version = "0.1.0"
source_dir = "src"
build_dir = "build"
target = "vm"
`
    );

    const child = join(root, 'nested', 'deep');
    await mkdir(child, { recursive: true });

    const loaded = await loadProjectConfig(undefined, child);
    expect(loaded).not.toBeNull();
    if (!loaded) return;
    expect(loaded.config.name).toBe('root-project');
  });

  it('honors explicit project path argument', async () => {
    const dir = await createTempDir();
    await writeConfig(
      dir,
      `
schema_version = 1

[project]
name = "explicit-project"
version = "0.1.0"
source_dir = "src"
build_dir = "build"
target = "vm"
`
    );

    const loaded = await loadProjectConfig(dir, tmpdir());
    expect(loaded).not.toBeNull();
    if (!loaded) return;
    expect(loaded.config.name).toBe('explicit-project');
  });

  it('parses dependency aliases from five.toml', async () => {
    const dir = await createTempDir();
    await writeConfig(
      dir,
      `
schema_version = 1

[project]
name = "deps-project"
version = "0.1.0"
source_dir = "src"
build_dir = "build"
target = "vm"

[dependencies]
std = { package = "@5ive/std", version = "0.1.0", source = "bundled", link = "inline" }
local_math = { package = "math-lib", source = "path", path = "../math-lib", link = "inline" }
`
    );

    const loaded = await loadProjectConfig(undefined, dir);
    expect(loaded).not.toBeNull();
    if (!loaded) return;

    expect(loaded.config.dependencies).toEqual([
      {
        alias: 'std',
        package: '@5ive/std',
        version: '0.1.0',
        source: 'bundled',
        link: 'inline',
        path: undefined,
        namespace: undefined,
        address: undefined,
        moatAccount: undefined,
        module: undefined,
        pin: undefined,
        cluster: undefined,
      },
      {
        alias: 'local_math',
        package: 'math-lib',
        version: undefined,
        source: 'path',
        link: 'inline',
        path: '../math-lib',
        namespace: undefined,
        address: undefined,
        moatAccount: undefined,
        module: undefined,
        pin: undefined,
        cluster: undefined,
      },
    ]);
  });

  it('fails when dependency source-specific fields are missing', async () => {
    const dir = await createTempDir();
    await writeConfig(
      dir,
      `
schema_version = 1

[project]
name = "invalid-deps-project"
version = "0.1.0"
source_dir = "src"
build_dir = "build"
target = "vm"

[dependencies]
broken = { package = "math-lib", source = "path", link = "inline" }
`
    );

    await expect(loadProjectConfig(undefined, dir)).rejects.toThrow(
      "Invalid dependency 'broken': source=path requires 'path'"
    );
  });

  it('fails when schema_version is missing', async () => {
    const dir = await createTempDir();
    await writeConfig(
      dir,
      `
[project]
name = "missing-schema"
version = "0.1.0"
source_dir = "src"
build_dir = "build"
target = "vm"
`
    );

    await expect(loadProjectConfig(undefined, dir)).rejects.toThrow(
      "missing required top-level 'schema_version'"
    );
  });
});
