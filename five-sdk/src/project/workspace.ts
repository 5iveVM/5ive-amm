/**
 * Workspace Configuration and Types
 * 
 * Provides types and utilities for Five workspace support,
 * enabling multi-package projects with cross-bytecode imports.
 */

// ==================== Link Types ====================

/**
 * Dependency link type
 * - inline: Merge into single bytecode (default)
 * - external: Use CALL_EXTERNAL at runtime
 */
export type LinkType = 'inline' | 'external';

// ==================== Workspace Configuration ====================

/**
 * Root workspace configuration from five.toml [workspace] section
 */
export interface WorkspaceConfig {
    /** Member package paths (supports globs like "packages/*") */
    members: string[];
    /** Paths to exclude from workspace */
    exclude?: string[];
    /** Default package settings inherited by members */
    package?: WorkspacePackageDefaults;
    /** Shared workspace dependencies */
    dependencies?: Record<string, WorkspaceDependency>;
}

/**
 * Default settings that can be inherited by workspace members
 */
export interface WorkspacePackageDefaults {
    version?: string;
    authors?: string[];
    edition?: string;
}

/**
 * Workspace-level dependency definition
 */
export interface WorkspaceDependency {
    version?: string;
    path?: string;
    link?: LinkType;
}

// ==================== Package Configuration ====================

/**
 * Package manifest from member five.toml
 */
export interface PackageManifest {
    package: PackageInfo;
    build?: PackageBuildConfig;
    dependencies?: Record<string, PackageDependency>;
    deploy?: PackageDeployConfig;
}

/**
 * Package metadata from [package] section
 */
export interface PackageInfo {
    name: string;
    version: string | { workspace: boolean };
    description?: string;
    authors?: string[];
}

/**
 * Package build configuration from [build] section
 */
export interface PackageBuildConfig {
    source_dir?: string;
    entry_point?: string;
    output?: string;
    output_type?: 'bytecode_account' | 'library';
}

/**
 * Package dependency specification
 */
export interface PackageDependency {
    version?: string;
    path?: string;
    /** Use workspace = true to inherit from workspace deps */
    workspace?: boolean;
    /** Link type: inline (merge) or external (CALL_EXTERNAL) */
    link?: LinkType;
    /** Explicit bytecode address (for deployed dependencies) */
    address?: string;
}

/**
 * Package deployment configuration
 */
export interface PackageDeployConfig {
    /** Deployed bytecode account address */
    address?: string;
    /** PDA seeds for deriving address */
    pda_seeds?: string[];
}

// ==================== Lock File ====================

/**
 * Lock file structure (five.lock)
 */
export interface LockFile {
    version: number;
    packages: LockEntry[];
}

/**
 * Individual package entry in lock file
 */
export interface LockEntry {
    name: string;
    version: string;
    address: string;
    bytecode_hash: string;
    deployed_at?: string;
}

// ==================== Resolved State ====================

/**
 * Resolved workspace state for IDE
 */
export interface WorkspaceState {
    /** Root directory path */
    root: string;
    /** Member package paths */
    members: string[];
    /** Resolved package information */
    packages: ResolvedPackage[];
    /** Lock file data */
    lockFile?: LockFile;
}

/**
 * Resolved package with dependency info
 */
export interface ResolvedPackage {
    name: string;
    path: string;
    entryPoint?: string;
    dependencies: ResolvedDependency[];
    /** Bytecode if compiled */
    bytecode?: Uint8Array;
    /** Deployed address if available */
    address?: string;
}

/**
 * Resolved dependency with link type
 */
export interface ResolvedDependency {
    name: string;
    link: LinkType;
    path?: string;
    address?: string;
}

// ==================== Utilities ====================

/**
 * Parse workspace section from TOML
 */
export function parseWorkspaceConfig(toml: Record<string, any>): WorkspaceConfig | null {
    const ws = toml.workspace;
    if (!ws || !ws.members) return null;

    return {
        members: ws.members,
        exclude: ws.exclude,
        package: ws.package,
        dependencies: ws.dependencies,
    };
}

/**
 * Parse package manifest from TOML
 */
export function parsePackageManifest(toml: Record<string, any>): PackageManifest | null {
    const pkg = toml.package;
    if (!pkg || !pkg.name) return null;

    return {
        package: {
            name: pkg.name,
            version: pkg.version || '0.1.0',
            description: pkg.description,
            authors: pkg.authors,
        },
        build: toml.build,
        dependencies: toml.dependencies,
        deploy: toml.deploy,
    };
}

/**
 * Determine build order from packages (topological sort)
 */
export function getBuildOrder(packages: ResolvedPackage[]): string[] {
    const visited = new Set<string>();
    const order: string[] = [];
    const packageMap = new Map(packages.map(p => [p.name, p]));

    function visit(name: string) {
        if (visited.has(name)) return;
        visited.add(name);

        const pkg = packageMap.get(name);
        if (pkg) {
            for (const dep of pkg.dependencies) {
                visit(dep.name);
            }
        }
        order.push(name);
    }

    for (const pkg of packages) {
        visit(pkg.name);
    }

    return order;
}

/**
 * Check if a file map represents a workspace
 */
export function isWorkspace(files: Record<string, string>): boolean {
    const rootToml = files['five.toml'];
    if (!rootToml) return false;
    return rootToml.includes('[workspace]');
}
