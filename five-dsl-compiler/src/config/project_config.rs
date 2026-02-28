//! Project configuration for multi-file compilation.

use five_vm_mito::error::VMError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProjectConfig {
    pub project: ProjectInfo,
    pub build: BuildConfig,
    #[serde(default)]
    pub optimizations: Option<OptimizationConfig>,
    #[serde(default)]
    pub dependencies: Option<HashMap<String, DependencyConfig>>,
    #[serde(default)]
    pub deploy: Option<DeployConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProjectInfo {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub source_dir: String,
    #[serde(default)]
    pub build_dir: Option<String>,
    #[serde(default)]
    pub target: Option<String>,
    #[serde(default)]
    pub entry_point: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BuildConfig {
    #[serde(default)]
    pub max_bytecode_size: Option<usize>,
    #[serde(default)]
    pub target_compute_units: Option<usize>,
    #[serde(default)]
    pub output_artifact_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OptimizationConfig {
    #[serde(default)]
    pub enable_compression: Option<bool>,
    #[serde(default)]
    pub inline_small_functions: Option<bool>,
    #[serde(default)]
    pub enable_constraint_optimization: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DependencyConfig {
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub git: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DeployConfig {
    #[serde(default)]
    pub network: Option<String>,
    #[serde(default)]
    pub cluster: Option<String>,
    #[serde(default)]
    pub commitment: Option<String>,
    #[serde(default)]
    pub rpc_url: Option<String>,
    #[serde(default)]
    pub program_id: Option<String>,
    #[serde(default)]
    pub keypair_path: Option<String>,
}

impl ProjectConfig {
    pub fn load(path: &Path) -> Result<Self, VMError> {
        let content = std::fs::read_to_string(path).map_err(|_| VMError::InvalidOperation)?;

        toml::from_str(&content).map_err(|_| VMError::InvalidOperation)
    }

    pub fn get_entry_point(&self) -> Option<PathBuf> {
        self.project.entry_point.as_ref().map(PathBuf::from)
    }

    pub fn get_source_dir(&self) -> PathBuf {
        PathBuf::from(self.project.source_dir.as_str())
    }

    pub fn get_build_dir(&self) -> PathBuf {
        self.project
            .build_dir
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("build"))
    }

    pub fn get_max_bytecode_size(&self) -> usize {
        self.build.max_bytecode_size.unwrap_or(1_048_576) // 1MB default
    }

    pub fn get_target_compute_units(&self) -> usize {
        self.build.target_compute_units.unwrap_or(200_000) // 200k default
    }

    pub fn is_compression_enabled(&self) -> bool {
        self.optimizations
            .as_ref()
            .and_then(|o| o.enable_compression)
            .unwrap_or(true)
    }

    pub fn validate(&self) -> Result<(), VMError> {
        // Compiler-owned multi-file flow always compiles from entry point.
        if self.get_entry_point().is_none() {
            return Err(VMError::InvalidOperation);
        }

        // Validate bytecode size
        if self.get_max_bytecode_size() == 0 {
            return Err(VMError::InvalidOperation);
        }

        Ok(())
    }
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            project: ProjectInfo {
                name: "five-project".to_string(),
                version: "0.1.0".to_string(),
                description: None,
                source_dir: "src".to_string(),
                build_dir: Some("build".to_string()),
                target: Some("vm".to_string()),
                entry_point: None,
            },
            build: BuildConfig {
                max_bytecode_size: Some(1_048_576),
                target_compute_units: Some(200_000),
                output_artifact_name: Some("five-project".to_string()),
            },
            optimizations: None,
            dependencies: None,
            deploy: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_config(content: &str) -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("five.toml");
        let mut file = fs::File::create(&config_path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        (temp_dir, config_path)
    }

    #[test]
    fn test_load_basic_config() {
        let content = r#"
[project]
name = "test-project"
version = "0.1.0"
source_dir = "src"
entry_point = "src/main.v"

[build]
max_bytecode_size = 1048576
"#;

        let (_temp, path) = create_test_config(content);
        let config = ProjectConfig::load(&path).unwrap();

        assert_eq!(config.project.name, "test-project");
        assert_eq!(config.project.version, "0.1.0");
    }

    #[test]
    fn test_load_entry_point_config() {
        let content = r#"
[project]
name = "multi-file-project"
version = "0.1.0"
source_dir = "src"
entry_point = "src/main.v"

[build]
max_bytecode_size = 1048576
"#;

        let (_temp, path) = create_test_config(content);
        let config = ProjectConfig::load(&path).unwrap();

        assert_eq!(config.get_entry_point(), Some(PathBuf::from("src/main.v")));
    }

    #[test]
    fn test_validate_without_entry_point_fails() {
        let content = r#"
[project]
name = "invalid-project"
version = "0.1.0"
source_dir = "src"

[build]
max_bytecode_size = 1048576
"#;

        let (_temp, path) = create_test_config(content);
        let config = ProjectConfig::load(&path).unwrap();

        let result = config.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_default_values() {
        let config = ProjectConfig::default();

        assert_eq!(config.get_max_bytecode_size(), 1_048_576);
        assert_eq!(config.get_target_compute_units(), 200_000);
        assert!(config.is_compression_enabled());
    }

    #[test]
    fn test_load_with_optimizations() {
        let content = r#"
[project]
name = "opt-project"
version = "0.1.0"
source_dir = "src"
entry_point = "src/main.v"

[build]
max_bytecode_size = 1048576

[optimizations]
enable_compression = false
"#;

        let (_temp, path) = create_test_config(content);
        let config = ProjectConfig::load(&path).unwrap();

        assert!(!config.is_compression_enabled());
    }
}
