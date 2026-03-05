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

const SUPPORTED_SCHEMA_VERSION = 1;

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
  const dependencies = parsed.dependencies ?? {};
  const deploy = parsed.deploy ?? {};
  const schemaVersionRaw = parsed.schema_version;
  if (schemaVersionRaw === undefined) {
    throw new Error(
      `Invalid five.toml at ${configPath}: missing required top-level 'schema_version'. Add:\nschema_version = ${SUPPORTED_SCHEMA_VERSION}`
    );
  }
  if (!Number.isInteger(schemaVersionRaw) || schemaVersionRaw <= 0) {
    throw new Error(
      `Invalid five.toml at ${configPath}: schema_version must be a positive integer`
    );
  }
  const schemaVersion = Number(schemaVersionRaw);
  if (schemaVersion !== SUPPORTED_SCHEMA_VERSION) {
    throw new Error(
      `Unsupported five.toml schema_version=${schemaVersion} at ${configPath}. Supported schema_version: ${SUPPORTED_SCHEMA_VERSION}`
    );
  }

  const name = project.name ?? 'five-project';
  const target = (project.target ?? 'vm') as CompilationTarget;

  const config: ProjectConfig = {
    schemaVersion,
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
    namespaceManager: deploy.namespace_manager ?? deploy.namespace_manager_script,
    keypairPath: deploy.keypair_path,
    optimizations: {
      enableCompression: optimizations.enable_compression ?? true,
      enableConstraintOptimization: optimizations.enable_constraint_optimization ?? true,
      optimizationLevel: 'production'
    },
    dependencies: parseDependencies(dependencies)
  };

  return { config, configPath, rootDir };
}

function parseDependencies(rawDeps: Record<string, any>): ProjectConfig['dependencies'] {
  const dependencies: ProjectConfig['dependencies'] = [];
  const seenNamespaces = new Set<string>();
  const seenAddresses = new Set<string>();
  const seenMoatTargets = new Set<string>();

  for (const [alias, rawValue] of Object.entries(rawDeps ?? {})) {
    const value = (rawValue ?? {}) as Record<string, any>;
    const dependency = {
      alias,
      package: String(value.package ?? ''),
      version: value.version !== undefined ? String(value.version) : undefined,
      source: String(value.source ?? ''),
      link: String(value.link ?? ''),
      path: value.path !== undefined ? String(value.path) : undefined,
      namespace: value.namespace !== undefined ? String(value.namespace) : undefined,
      address: value.address !== undefined ? String(value.address) : undefined,
      moatAccount: value.moat_account !== undefined ? String(value.moat_account) : undefined,
      module: value.module !== undefined ? String(value.module) : undefined,
      pin: value.pin !== undefined ? String(value.pin) : undefined,
      cluster: value.cluster !== undefined ? String(value.cluster) : undefined,
    } as ProjectConfig['dependencies'][number];

    if (!dependency.package) {
      throw new Error(`Invalid dependency '${alias}': missing required field 'package'`);
    }
    if (
      dependency.source !== 'bundled' &&
      dependency.source !== 'path' &&
      dependency.source !== 'namespace' &&
      dependency.source !== 'address' &&
      dependency.source !== 'moat'
    ) {
      throw new Error(
        `Invalid dependency '${alias}': source must be one of bundled|path|namespace|address|moat`
      );
    }
    if (dependency.link !== 'inline' && dependency.link !== 'external') {
      throw new Error(`Invalid dependency '${alias}': link must be one of inline|external`);
    }

    const hasPath = Boolean(dependency.path);
    const hasNamespace = Boolean(dependency.namespace);
    const hasAddress = Boolean(dependency.address);
    const hasMoatAccount = Boolean(dependency.moatAccount);
    const hasModule = Boolean(dependency.module);

    if (dependency.source === 'path' && !hasPath) {
      throw new Error(`Invalid dependency '${alias}': source=path requires 'path'`);
    }
    if (dependency.source === 'namespace' && !hasNamespace) {
      throw new Error(`Invalid dependency '${alias}': source=namespace requires 'namespace'`);
    }
    if (dependency.source === 'address' && !hasAddress) {
      throw new Error(`Invalid dependency '${alias}': source=address requires 'address'`);
    }
    if (dependency.source === 'moat' && (!hasMoatAccount || !hasModule)) {
      throw new Error(
        `Invalid dependency '${alias}': source=moat requires 'moat_account' and 'module'`
      );
    }
    if (dependency.source !== 'path' && hasPath) {
      throw new Error(`Invalid dependency '${alias}': 'path' is only valid for source=path`);
    }
    if (dependency.source !== 'namespace' && hasNamespace) {
      throw new Error(
        `Invalid dependency '${alias}': 'namespace' is only valid for source=namespace`
      );
    }
    if (dependency.source !== 'address' && hasAddress) {
      throw new Error(`Invalid dependency '${alias}': 'address' is only valid for source=address`);
    }
    if (dependency.source !== 'moat' && hasMoatAccount) {
      throw new Error(
        `Invalid dependency '${alias}': 'moat_account' is only valid for source=moat`
      );
    }
    if (dependency.source !== 'moat' && hasModule) {
      throw new Error(
        `Invalid dependency '${alias}': 'module' is only valid for source=moat`
      );
    }

    if (dependency.source === 'bundled' || dependency.source === 'path') {
      if (dependency.link !== 'inline') {
        throw new Error(
          `Invalid dependency '${alias}': source=${dependency.source} currently requires link=inline`
        );
      }
    } else if (dependency.link !== 'external') {
      throw new Error(
        `Invalid dependency '${alias}': source=${dependency.source} currently requires link=external`
      );
    }

    if (dependency.namespace) {
      if (seenNamespaces.has(dependency.namespace)) {
        throw new Error(`Invalid dependencies: duplicate namespace '${dependency.namespace}'`);
      }
      seenNamespaces.add(dependency.namespace);
    }
    if (dependency.address) {
      if (seenAddresses.has(dependency.address)) {
        throw new Error(`Invalid dependencies: duplicate address '${dependency.address}'`);
      }
      seenAddresses.add(dependency.address);
    }
    if (dependency.moatAccount && dependency.module) {
      const moatTarget = `${dependency.moatAccount}::${dependency.module}`;
      if (seenMoatTargets.has(moatTarget)) {
        throw new Error(`Invalid dependencies: duplicate moat target '${moatTarget}'`);
      }
      seenMoatTargets.add(moatTarget);
    }

    dependencies.push(dependency);
  }

  return dependencies;
}

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

export async function loadBuildManifest(rootDir: string): Promise<BuildManifest | null> {
  const manifestPath = join(rootDir, '.five', 'build.json');
  try {
    const content = await readFile(manifestPath, 'utf8');
    return JSON.parse(content) as BuildManifest;
  } catch {
    return null;
  }
}

export async function writeBuildManifest(rootDir: string, manifest: BuildManifest): Promise<string> {
  const manifestDir = join(rootDir, '.five');
  await mkdir(manifestDir, { recursive: true });
  const manifestPath = join(manifestDir, 'build.json');
  await writeFile(manifestPath, JSON.stringify(manifest, null, 2));
  return manifestPath;
}

export function computeHash(data: Buffer | Uint8Array): string {
  const hash = createHash('sha256');
  hash.update(data);
  return hash.digest('hex');
}
