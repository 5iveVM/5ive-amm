import type { ProjectConfig, CompilationTarget } from '../types.js';

const SUPPORTED_SCHEMA_VERSION = 1;

/**
 * Parses a raw TOML object into a strict ProjectConfig.
 */
export function parseProjectConfig(parsedToml: Record<string, any>): ProjectConfig {
    const schemaVersionRaw = parsedToml.schema_version;
    if (schemaVersionRaw === undefined) {
        throw new Error(
            `Missing required top-level 'schema_version'. Add:\nschema_version = ${SUPPORTED_SCHEMA_VERSION}`
        );
    }
    if (!Number.isInteger(schemaVersionRaw) || Number(schemaVersionRaw) <= 0) {
        throw new Error('schema_version must be a positive integer');
    }
    const schemaVersion = Number(schemaVersionRaw);
    if (schemaVersion !== SUPPORTED_SCHEMA_VERSION) {
        throw new Error(
            `Unsupported schema_version=${schemaVersion}. Supported schema_version: ${SUPPORTED_SCHEMA_VERSION}`
        );
    }

    const project = parsedToml.project ?? {};
    const build = parsedToml.build ?? {};
    const optimizations = parsedToml.optimizations ?? {};
    const dependencies = parsedToml.dependencies ?? {};
    const deploy = parsedToml.deploy ?? {};

    const name = project.name ?? 'five-project';
    const target = (project.target ?? 'vm') as CompilationTarget;

    return {
        schemaVersion,
        name,
        version: project.version ?? '0.1.0',
        description: project.description,
        sourceDir: project.source_dir ?? 'src',
        buildDir: project.build_dir ?? 'build',
        target,
        entryPoint: project.entry_point,
        outputArtifactName: build.output_artifact_name ?? name,
        cluster: deploy.cluster ?? deploy.network,
        commitment: deploy.commitment,
        rpcUrl: deploy.rpc_url,
        programId: deploy.program_id,
        keypairPath: deploy.keypair_path,
        optimizations: {
            enableCompression: optimizations.enable_compression ?? true,
            enableConstraintOptimization: optimizations.enable_constraint_optimization ?? true,
            // Public SDK config is locked to the canonical production mode.
            optimizationLevel: 'production'
        },
        dependencies: parseDependencies(dependencies)
    };
}

function parseDependencies(rawDeps: Record<string, any>): ProjectConfig['dependencies'] {
    const out: NonNullable<ProjectConfig['dependencies']> = [];
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
        } as NonNullable<ProjectConfig['dependencies']>[number];

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

        out.push(dependency);
    }

    return out;
}
