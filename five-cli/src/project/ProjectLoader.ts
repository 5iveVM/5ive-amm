import { readFile, access, mkdir, writeFile } from 'fs/promises';
import { dirname, isAbsolute, join, resolve } from 'path';
import { createHash } from 'crypto';
import { parse as parseToml } from '@iarna/toml';

import { BuildManifest, CompilationTarget, ProjectConfig } from '../types.js';

export interface LoadedProjectConfig {
  config: ProjectConfig;
  configPath: string;
  rootDir: string;
}

/**
 * Load project configuration, discovering five.toml if needed.
 */
export async function loadProjectConfig(
  projectPath?: string,
  cwd: string = process.cwd()
): Promise<LoadedProjectConfig | null> {
  const configPath = await findConfigPath(projectPath, cwd);
  if (!configPath) {
    return null;
  }

  const fileContent = await readFile(configPath, 'utf8');
  let parsed: any;
  try {
    parsed = parseToml(fileContent);
  } catch (e) {
    throw new Error(`Failed to parse five.toml at ${configPath}: ${e}`);
  }

  const rootDir = dirname(configPath);

  const project = parsed.project ?? {};
  const build = parsed.build ?? {};
  const optimizations = parsed.optimizations ?? {};
  const deploy = parsed.deploy ?? {};
  const modules = parsed.modules ?? {};

  const name = project.name ?? 'five-project';
  const target = (project.target ?? 'vm') as CompilationTarget;

  const config: ProjectConfig = {
    name,
    version: project.version ?? '0.1.0',
    description: project.description,
    sourceDir: project.source_dir ?? 'src',
    buildDir: project.build_dir ?? 'build',
    target,
    entryPoint: project.entry_point ?? build.entry_point,
    outputArtifactName: build.output_artifact_name ?? name,
    cluster: deploy.cluster ?? deploy.network,
    commitment: deploy.commitment,
    rpcUrl: deploy.rpc_url,
    programId: deploy.program_id,
    keypairPath: deploy.keypair_path,
    multiFileMode: build.multi_file_mode ?? false,
    optimizations: {
      enableVLE: optimizations.enable_vle ?? true,
      enableCompression: optimizations.enable_compression ?? true,
      enableConstraintOptimization: optimizations.enable_constraint_optimization ?? true,
      optimizationLevel: 'production'
    },
    dependencies: [],
    modules: modules as Record<string, string[]>
  };

  return { config, configPath, rootDir };
}

/**
 * Discover five.toml path starting from cwd or explicit project path.
 */
export async function findConfigPath(
  projectPath?: string,
  cwd: string = process.cwd()
): Promise<string | null> {
  if (projectPath) {
    const candidate = isAbsolute(projectPath) ? projectPath : resolve(cwd, projectPath);
    // If a directory is provided, look for five.toml inside
    try {
      await access(candidate);
      const asToml = candidate.endsWith('.toml') ? candidate : join(candidate, 'five.toml');
      try {
        await access(asToml);
        return asToml;
      } catch {
        // fallthrough to upward search from candidate dir
        return await searchUpwards(candidate);
      }
    } catch {
      return null;
    }
  }

  return await searchUpwards(cwd);
}

async function searchUpwards(startDir: string): Promise<string | null> {
  let current = resolve(startDir);
  while (true) {
    const candidate = join(current, 'five.toml');
    try {
      await access(candidate);
      return candidate;
    } catch {
      // continue
    }

    const parent = dirname(current);
    if (parent === current) {
      break;
    }
    current = parent;
  }
  return null;
}

/**
 * Load build manifest if present.
 */
export async function loadBuildManifest(rootDir: string): Promise<BuildManifest | null> {
  const manifestPath = join(rootDir, '.five', 'build.json');
  try {
    const content = await readFile(manifestPath, 'utf8');
    return JSON.parse(content) as BuildManifest;
  } catch {
    return null;
  }
}

/**
 * Write build manifest to .five/build.json.
 */
export async function writeBuildManifest(rootDir: string, manifest: BuildManifest): Promise<string> {
  const manifestDir = join(rootDir, '.five');
  await mkdir(manifestDir, { recursive: true });
  const manifestPath = join(manifestDir, 'build.json');
  await writeFile(manifestPath, JSON.stringify(manifest, null, 2));
  return manifestPath;
}

/**
 * Compute SHA-256 hash of a buffer.
 */
export function computeHash(data: Buffer | Uint8Array): string {
  const hash = createHash('sha256');
  hash.update(data);
  return hash.digest('hex');
}
