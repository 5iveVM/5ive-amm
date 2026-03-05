//! Configuration module.

pub mod project_config;
pub mod workspace;

pub use project_config::{
    BuildConfig, DependencyConfig, DependencyLink, DependencySource, DeployConfig,
    NormalizedDependency, OptimizationConfig, ProjectConfig, ProjectInfo,
};

pub use workspace::{
    LinkType, LockEntry, LockFile, NamespaceBinding, OutputType, PackageBuildConfig, PackageConfig,
    PackageDependency, PackageDeployConfig, PackageManifest, VersionSpec, WorkspaceConfig,
    WorkspaceDependency, WorkspacePackageDefaults,
};
