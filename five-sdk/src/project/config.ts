import type { ProjectConfig, CompilationTarget } from '../types.js';


/**
 * Parses a raw TOML object into a strict ProjectConfig.
 */
export function parseProjectConfig(parsedToml: Record<string, any>): ProjectConfig {
    const project = parsedToml.project ?? {};
    const build = parsedToml.build ?? {};
    const optimizations = parsedToml.optimizations ?? {};
    const deploy = parsedToml.deploy ?? {};

    const name = project.name ?? 'five-project';
    const target = (project.target ?? 'vm') as CompilationTarget;

    return {
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
        multiFileMode: build.multi_file_mode ?? false,
        optimizations: {
            enableVLE: optimizations.enable_vle ?? true,
            enableCompression: optimizations.enable_compression ?? true,
            enableRegisterAllocation: optimizations.enable_register_allocation ?? true,
            enableConstraintOptimization: optimizations.enable_constraint_optimization ?? true,
            optimizationLevel: 'production'
        },
        dependencies: []
    };
}
