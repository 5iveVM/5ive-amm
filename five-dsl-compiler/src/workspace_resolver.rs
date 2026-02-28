use five_vm_mito::error::VMError;
use glob::glob;
/// Workspace Discovery and Resolution
///
/// Provides functionality to discover workspace structure,
/// resolve dependencies, and determine build order.
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::config::workspace::{
    LinkType, LockFile, PackageDependency, PackageManifest, WorkspaceConfig,
};

/// A resolved package in the workspace
#[derive(Debug, Clone)]
pub struct ResolvedPackage {
    /// Package name
    pub name: String,
    /// Path to package directory
    pub path: PathBuf,
    /// Path to package manifest (five.toml)
    pub manifest_path: PathBuf,
    /// Parsed manifest
    pub manifest: PackageManifest,
    /// Resolved dependencies with their link types
    pub dependencies: Vec<ResolvedDependency>,
}

/// A resolved dependency
#[derive(Debug, Clone)]
pub struct ResolvedDependency {
    /// Dependency name
    pub name: String,
    /// Link type (inline or external)
    pub link: LinkType,
    /// Resolved package path (for local dependencies)
    pub package_path: Option<PathBuf>,
    /// Bytecode address (for external dependencies)
    pub address: Option<String>,
}

/// Discovered workspace
#[derive(Debug)]
pub struct Workspace {
    /// Root directory of workspace
    pub root: PathBuf,
    /// Workspace configuration
    pub config: WorkspaceConfig,
    /// Resolved member packages
    pub packages: Vec<ResolvedPackage>,
    /// Lock file (if exists)
    pub lock: LockFile,
}

impl Workspace {
    /// Discover workspace from a starting path
    ///
    /// Walks up directories looking for five.toml with [workspace] section
    pub fn discover(start_path: &Path) -> Result<Option<Self>, WorkspaceError> {
        let mut current = start_path.to_path_buf();

        if current.is_file() {
            current = current
                .parent()
                .ok_or(WorkspaceError::InvalidPath)?
                .to_path_buf();
        }

        while current.as_os_str().len() > 1 {
            let manifest_path = current.join("five.toml");
            if manifest_path.exists() {
                if let Some(config) = Self::try_load_workspace_config(&manifest_path)? {
                    return Self::load(current, config).map(Some);
                }
            }
            current = match current.parent() {
                Some(p) => p.to_path_buf(),
                None => break,
            };
        }

        Ok(None)
    }

    /// Try to load workspace config from five.toml
    fn try_load_workspace_config(path: &Path) -> Result<Option<WorkspaceConfig>, WorkspaceError> {
        let content =
            std::fs::read_to_string(path).map_err(|e| WorkspaceError::IoError(e.to_string()))?;

        // Parse as toml::Value first to check for workspace section
        let value: toml::Value =
            toml::from_str(&content).map_err(|e| WorkspaceError::ParseError(e.to_string()))?;

        if let Some(workspace) = value.get("workspace") {
            let config: WorkspaceConfig = workspace
                .clone()
                .try_into()
                .map_err(|e: toml::de::Error| WorkspaceError::ParseError(e.to_string()))?;
            Ok(Some(config))
        } else {
            Ok(None)
        }
    }

    /// Load workspace from root directory
    pub fn load(root: PathBuf, config: WorkspaceConfig) -> Result<Self, WorkspaceError> {
        let lock_path = root.join("five.lock");
        let lock = LockFile::load(&lock_path).unwrap_or_else(|_| LockFile::new());

        let mut packages = Vec::new();

        // Resolve member paths (with glob support)
        for member_pattern in &config.members {
            let pattern = root.join(member_pattern).display().to_string();

            let paths: Vec<PathBuf> = if member_pattern.contains('*') {
                glob(&pattern)
                    .map_err(|e| WorkspaceError::GlobError(e.to_string()))?
                    .filter_map(Result::ok)
                    .collect()
            } else {
                vec![root.join(member_pattern)]
            };

            for path in paths {
                // Check exclusions
                if let Some(ref excludes) = config.exclude {
                    let relative = path.strip_prefix(&root).unwrap_or(&path);
                    if excludes.iter().any(|e| relative.starts_with(e)) {
                        continue;
                    }
                }

                if path.is_dir() {
                    let manifest_path = path.join("five.toml");
                    if manifest_path.exists() {
                        let package = Self::load_package(&path, &manifest_path, &lock)?;
                        packages.push(package);
                    }
                }
            }
        }

        Ok(Self {
            root,
            config,
            packages,
            lock,
        })
    }

    /// Load a single package
    fn load_package(
        path: &Path,
        manifest_path: &Path,
        lock: &LockFile,
    ) -> Result<ResolvedPackage, WorkspaceError> {
        let content = std::fs::read_to_string(manifest_path)
            .map_err(|e| WorkspaceError::IoError(e.to_string()))?;

        let manifest: PackageManifest = toml::from_str(&content).map_err(|e| {
            WorkspaceError::ParseError(format!(
                "Failed to parse {}: {}",
                manifest_path.display(),
                e
            ))
        })?;

        // Resolve dependencies
        let mut dependencies = Vec::new();
        if let Some(ref deps) = manifest.dependencies {
            for (name, dep) in deps {
                let resolved = Self::resolve_dependency(name, dep, path, lock)?;
                dependencies.push(resolved);
            }
        }

        Ok(ResolvedPackage {
            name: manifest.package.name.clone(),
            path: path.to_path_buf(),
            manifest_path: manifest_path.to_path_buf(),
            manifest,
            dependencies,
        })
    }

    /// Resolve a single dependency
    fn resolve_dependency(
        name: &str,
        dep: &PackageDependency,
        package_path: &Path,
        lock: &LockFile,
    ) -> Result<ResolvedDependency, WorkspaceError> {
        match dep {
            PackageDependency::Version(_version) => {
                // Version-only dependency - look up in lock file
                Ok(ResolvedDependency {
                    name: name.to_string(),
                    link: LinkType::Inline,
                    package_path: None,
                    address: lock.get_address(name).map(String::from),
                })
            }
            PackageDependency::Full {
                path,
                link,
                address,
                ..
            } => {
                let package_path = path.as_ref().map(|p| package_path.join(p));

                let resolved_address = address
                    .clone()
                    .or_else(|| lock.get_address(name).map(String::from));

                Ok(ResolvedDependency {
                    name: name.to_string(),
                    link: link.clone(),
                    package_path,
                    address: resolved_address,
                })
            }
        }
    }

    /// Get build order (topological sort by dependencies)
    pub fn build_order(&self) -> Result<Vec<&ResolvedPackage>, WorkspaceError> {
        let mut order = Vec::new();
        let mut visited = HashSet::new();
        let mut in_progress = HashSet::new();

        // Build name-to-package map
        let package_map: HashMap<&str, &ResolvedPackage> =
            self.packages.iter().map(|p| (p.name.as_str(), p)).collect();

        for package in &self.packages {
            self.visit_package(
                package,
                &package_map,
                &mut visited,
                &mut in_progress,
                &mut order,
            )?;
        }

        Ok(order)
    }

    fn visit_package<'a>(
        &'a self,
        package: &'a ResolvedPackage,
        package_map: &HashMap<&str, &'a ResolvedPackage>,
        visited: &mut HashSet<String>,
        in_progress: &mut HashSet<String>,
        order: &mut Vec<&'a ResolvedPackage>,
    ) -> Result<(), WorkspaceError> {
        if visited.contains(&package.name) {
            return Ok(());
        }

        if in_progress.contains(&package.name) {
            return Err(WorkspaceError::CyclicDependency(package.name.clone()));
        }

        in_progress.insert(package.name.clone());

        // Visit dependencies first
        for dep in &package.dependencies {
            if let Some(dep_package) = package_map.get(dep.name.as_str()) {
                self.visit_package(dep_package, package_map, visited, in_progress, order)?;
            }
        }

        in_progress.remove(&package.name);
        visited.insert(package.name.clone());
        order.push(package);

        Ok(())
    }

    /// Get packages with external link dependencies
    pub fn packages_with_external_deps(&self) -> Vec<&ResolvedPackage> {
        self.packages
            .iter()
            .filter(|p| p.dependencies.iter().any(|d| d.link == LinkType::External))
            .collect()
    }

    /// Save lock file
    pub fn save_lock(&self) -> Result<(), WorkspaceError> {
        self.lock
            .save(&self.root.join("five.lock"))
            .map_err(|_| WorkspaceError::IoError("Failed to save lock file".to_string()))
    }
}

/// Workspace error types
#[derive(Debug, Clone)]
pub enum WorkspaceError {
    InvalidPath,
    IoError(String),
    ParseError(String),
    GlobError(String),
    CyclicDependency(String),
    PackageNotFound(String),
}

impl std::fmt::Display for WorkspaceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidPath => write!(f, "Invalid path"),
            Self::IoError(e) => write!(f, "IO error: {}", e),
            Self::ParseError(e) => write!(f, "Parse error: {}", e),
            Self::GlobError(e) => write!(f, "Glob pattern error: {}", e),
            Self::CyclicDependency(name) => write!(f, "Cyclic dependency detected: {}", name),
            Self::PackageNotFound(name) => write!(f, "Package not found: {}", name),
        }
    }
}

impl std::error::Error for WorkspaceError {}

impl From<WorkspaceError> for VMError {
    fn from(_: WorkspaceError) -> Self {
        VMError::InvalidOperation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_discovery_no_workspace() {
        let temp = std::env::temp_dir();
        let result = Workspace::discover(&temp);
        assert!(result.is_ok());
        // Temp dir shouldn't have a workspace
    }
}
