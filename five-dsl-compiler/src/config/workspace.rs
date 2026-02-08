/// Workspace configuration and management.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use five_vm_mito::error::VMError;

/// Link type for dependencies.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum LinkType {
    /// Merge dependency into single bytecode (current behavior).
    #[default]
    Inline,
    /// Use CALL_EXTERNAL at runtime (separate bytecode accounts).
    External,
}

/// Workspace configuration (root five.toml).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WorkspaceConfig {
    /// Member package paths (supports globs).
    pub members: Vec<String>,
    /// Excluded paths.
    #[serde(default)]
    pub exclude: Option<Vec<String>>,
    /// Default package settings inherited by members.
    #[serde(default)]
    pub package: Option<WorkspacePackageDefaults>,
    /// Shared dependencies.
    #[serde(default)]
    pub dependencies: Option<HashMap<String, WorkspaceDependency>>,
}

/// Default settings that can be inherited by workspace members.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct WorkspacePackageDefaults {
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub authors: Option<Vec<String>>,
    #[serde(default)]
    pub edition: Option<String>,
}

/// Workspace-level dependency definition.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WorkspaceDependency {
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
    /// Default link type for this dependency.
    #[serde(default)]
    pub link: LinkType,
}

/// Package configuration (member five.toml).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PackageConfig {
    pub name: String,
    #[serde(default)]
    pub version: VersionSpec,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub authors: Option<Vec<String>>,
}

/// Version can be explicit or inherited from workspace.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(untagged)]
pub enum VersionSpec {
    #[default]
    Inherit,
    Explicit(String),
    WorkspaceRef { workspace: bool },
}

/// Package build configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PackageBuildConfig {
    #[serde(default = "default_source_dir")]
    pub source_dir: String,
    #[serde(default)]
    pub entry_point: Option<String>,
    #[serde(default)]
    pub output: Option<String>,
    /// Output type: bytecode_account, library, executable.
    #[serde(default)]
    pub output_type: OutputType,
}

fn default_source_dir() -> String {
    "src".to_string()
}

/// Output type for package compilation.
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum OutputType {
    /// Produces a deployable bytecode account.
    #[default]
    BytecodeAccount,
    /// Library that can be linked (inline or external).
    Library,
}

/// Package dependency.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum PackageDependency {
    /// Simple version string.
    Version(String),
    /// Full dependency spec.
    Full {
        #[serde(default)]
        version: Option<String>,
        #[serde(default)]
        path: Option<String>,
        #[serde(default)]
        workspace: Option<bool>,
        /// Link type: inline or external.
        #[serde(default)]
        link: LinkType,
        /// Explicit bytecode address (for deployed dependencies).
        #[serde(default)]
        address: Option<String>,
    },
}

/// Full member package manifest.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PackageManifest {
    pub package: PackageConfig,
    #[serde(default)]
    pub build: Option<PackageBuildConfig>,
    #[serde(default)]
    pub dependencies: Option<HashMap<String, PackageDependency>>,
    #[serde(default)]
    pub deploy: Option<PackageDeployConfig>,
}

/// Package deployment configuration.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct PackageDeployConfig {
    /// Deployed bytecode account address (updated after deployment).
    #[serde(default)]
    pub address: Option<String>,
    /// PDA seeds for deriving address.
    #[serde(default)]
    pub pda_seeds: Option<Vec<String>>,
}

/// Lock file entry for a deployed package.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LockEntry {
    pub name: String,
    pub version: String,
    pub address: String,
    pub bytecode_hash: String,
    pub deployed_at: Option<String>,
}

/// Lock file structure (five.lock).
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct LockFile {
    pub version: u32,
    pub packages: Vec<LockEntry>,
}

impl LockFile {
    pub fn new() -> Self {
        Self {
            version: 1,
            packages: Vec::new(),
        }
    }

    pub fn load(path: &Path) -> Result<Self, VMError> {
        if !path.exists() {
            return Ok(Self::new());
        }
        let content = std::fs::read_to_string(path)
            .map_err(|_| VMError::InvalidOperation)?;
        toml::from_str(&content)
            .map_err(|_| VMError::InvalidOperation)
    }

    pub fn save(&self, path: &Path) -> Result<(), VMError> {
        let content = toml::to_string_pretty(self)
            .map_err(|_| VMError::InvalidOperation)?;
        std::fs::write(path, content)
            .map_err(|_| VMError::InvalidOperation)
    }

    /// Get address for a package
    pub fn get_address(&self, name: &str) -> Option<&str> {
        self.packages
            .iter()
            .find(|p| p.name == name)
            .map(|p| p.address.as_str())
    }

    /// Update or add a package entry
    pub fn update_package(&mut self, entry: LockEntry) {
        if let Some(idx) = self.packages.iter().position(|p| p.name == entry.name) {
            self.packages[idx] = entry;
        } else {
            self.packages.push(entry);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_link_type_default() {
        assert_eq!(LinkType::default(), LinkType::Inline);
    }

    #[test]
    fn test_workspace_config_parse() {
        // The TOML has [workspace] as a section, so we need a wrapper struct
        #[derive(Deserialize)]
        struct WorkspaceToml {
            workspace: WorkspaceConfig,
        }
        
        let toml_str = r#"
[workspace]
members = ["packages/*"]
exclude = ["packages/deprecated"]

[workspace.package]
version = "1.0.0"
"#;
        let parsed: WorkspaceToml = toml::from_str(toml_str).unwrap();
        let config = parsed.workspace;
        assert_eq!(config.members, vec!["packages/*"]);
        assert_eq!(config.exclude, Some(vec!["packages/deprecated".to_string()]));
    }

    #[test]
    fn test_package_dependency_parse() {
        let toml_str = r#"
[package]
name = "test-package"

[dependencies]
math-lib = { path = "../math-lib", link = "external" }
utils = "1.0.0"
"#;
        let manifest: PackageManifest = toml::from_str(toml_str).unwrap();
        assert_eq!(manifest.package.name, "test-package");
    }

    #[test]
    fn test_lock_file() {
        let mut lock = LockFile::new();
        lock.update_package(LockEntry {
            name: "math-lib".to_string(),
            version: "1.0.0".to_string(),
            address: "11111111111111111111111111111111".to_string(),
            bytecode_hash: "abc123".to_string(),
            deployed_at: None,
        });
        
        assert_eq!(lock.get_address("math-lib"), Some("11111111111111111111111111111111"));
        assert_eq!(lock.get_address("unknown"), None);
    }
}
