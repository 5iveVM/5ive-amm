//! Configuration System Module
//!
//! Provides project configuration loading and validation
//! with Cargo-style workspace support.

pub mod project_config;
pub mod workspace;

pub use project_config::{
    BuildConfig, DeployConfig, DependencyConfig, OptimizationConfig, ProjectConfig, ProjectInfo,
};

pub use workspace::{
    LinkType, LockEntry, LockFile, OutputType, PackageBuildConfig, PackageConfig,
    PackageDependency, PackageDeployConfig, PackageManifest, VersionSpec, WorkspaceConfig,
    WorkspaceDependency, WorkspacePackageDefaults,
};
