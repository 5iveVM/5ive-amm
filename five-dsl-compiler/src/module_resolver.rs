/// Module Resolution System for Multi-File Compilation
///
/// Handles:
/// - File discovery and module graph construction
/// - Dependency graph analysis
/// - Circular dependency detection
/// - Topological sorting for compilation order
/// - Smart import target discrimination (local, registry, Solana address)

use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
// use five_vm_mito::error::VMError;
use crate::error::ModuleResolutionError;

/// Represents a single module in the project
#[derive(Debug, Clone)]
pub struct ModuleDescriptor {
    /// Module path like "types", "utils::helpers"
    pub module_path: String,
    /// Absolute path to the .v file
    pub file_path: PathBuf,
    /// Loaded source code
    pub source_code: String,
    /// Other modules this depends on
    pub dependencies: Vec<String>,
    /// Whether this is the entry point
    pub is_entry_point: bool,
}

/// Result of analyzing an import statement
#[derive(Debug, Clone, PartialEq)]
pub enum ImportTarget {
    /// Local module: `use types;` or `use utils::helpers;`
    LocalModule {
        module_path: String,
        resolved_file: Option<PathBuf>,
    },
    /// Registry name: `use "token"::{...};`
    RegistryName {
        name: String,
    },
    /// Direct Solana address: `use "HMxPuYGdU7..."::{...};`
    SolanaPubkey {
        address: String,
    },
}

/// Graph of module dependencies
#[derive(Debug)]
pub struct ModuleGraph {
    /// All discovered modules
    modules: HashMap<String, ModuleDescriptor>,
    /// Edges in dependency graph: module -> list of dependencies
    dependency_edges: HashMap<String, Vec<String>>,
    /// Topologically sorted compilation order
    compilation_order: Vec<String>,
}

impl ModuleGraph {
    /// Create a new empty module graph
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
            dependency_edges: HashMap::new(),
            compilation_order: Vec::new(),
        }
    }

    /// Add a module to the graph
    pub fn add_module(&mut self, descriptor: ModuleDescriptor) {
        let module_path = descriptor.module_path.clone();
        self.modules.insert(module_path.clone(), descriptor);
        self.dependency_edges.entry(module_path).or_insert_with(Vec::new);
    }

    /// Add a dependency edge: from_module depends on to_module
    pub fn add_dependency(&mut self, from_module: String, to_module: String) {
        self.dependency_edges
            .entry(from_module)
            .or_default()
            .push(to_module);
    }

    /// Get a module by its path
    pub fn get_module(&self, path: &str) -> Option<&ModuleDescriptor> {
        self.modules.get(path)
    }

    /// Get all modules
    pub fn modules(&self) -> &HashMap<String, ModuleDescriptor> {
        &self.modules
    }

    /// Get compilation order (topologically sorted)
    pub fn compilation_order(&self) -> &[String] {
        &self.compilation_order
    }

    /// Get dependencies for a module
    pub fn get_dependencies(&self, module_path: &str) -> Vec<String> {
        self.dependency_edges
            .get(module_path)
            .cloned()
            .unwrap_or_default()
    }

    /// Detect cycles in the dependency graph using DFS
    pub fn detect_cycles(&self) -> Result<(), ModuleResolutionError> {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for module in self.modules.keys() {
            if !visited.contains(module) {
                self.dfs_cycle_detect(module, &mut visited, &mut rec_stack)?;
            }
        }

        Ok(())
    }

    /// DFS helper for cycle detection
    fn dfs_cycle_detect(
        &self,
        module: &str,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
    ) -> Result<(), ModuleResolutionError> {
        visited.insert(module.to_string());
        rec_stack.insert(module.to_string());

        if let Some(deps) = self.dependency_edges.get(module) {
            for dep in deps {
                if !visited.contains(dep) {
                    self.dfs_cycle_detect(dep, visited, rec_stack)?;
                } else if rec_stack.contains(dep) {
                    return Err(ModuleResolutionError::CircularDependency(module.to_string()));
                }
            }
        }

        rec_stack.remove(module);
        Ok(())
    }

    /// Compute topological sort of modules using Kahn's algorithm
    pub fn compute_compilation_order(&mut self) -> Result<(), ModuleResolutionError> {
        // Check for cycles first
        self.detect_cycles()?;

        // Build in-degree map (count how many dependencies each module has)
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        for module in self.modules.keys() {
            in_degree.insert(module.clone(), 0);
        }

        // Count in-degrees: each edge (from -> deps) means 'from' depends on 'deps'
        for (from, deps) in &self.dependency_edges {
            let count = in_degree.entry(from.clone()).or_insert(0);
            *count = deps.len();
        }

        // Queue of modules with no dependencies (in-degree 0)
        let mut queue: VecDeque<String> = in_degree
            .iter()
            .filter(|(_, &degree)| degree == 0)
            .map(|(module, _)| module.clone())
            .collect();

        let mut order = Vec::new();

        while let Some(module) = queue.pop_front() {
            order.push(module.clone());

            // Find modules that depend on this one and reduce their in-degree
            for (other, deps) in &self.dependency_edges {
                if deps.contains(&module) {
                    let degree = in_degree.entry(other.clone()).or_insert(0);
                    *degree = degree.saturating_sub(1);
                    if *degree == 0 {
                        queue.push_back(other.clone());
                    }
                }
            }
        }

        if order.len() != self.modules.len() {
            return Err(ModuleResolutionError::Generic("Topological sort failed: graph has cycles or unresolvable dependencies".to_string()));
        }

        self.compilation_order = order;
        Ok(())
    }
}

impl Default for ModuleGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Discovers modules starting from an entry point
pub struct ModuleDiscoverer {
    source_dir: PathBuf,
}

impl ModuleDiscoverer {
    /// Create a new discoverer for a source directory
    pub fn new(source_dir: PathBuf) -> Self {
        Self { source_dir }
    }

    /// Discover all modules starting from an entry point file
    pub fn discover_modules(&self, entry_point: &Path) -> Result<ModuleGraph, ModuleResolutionError> {
        let mut graph = ModuleGraph::new();
        let mut visited = HashSet::new();
        let mut to_process = VecDeque::new();

        // Start with entry point
        to_process.push_back(entry_point.to_path_buf());

        while let Some(file_path) = to_process.pop_front() {
            if visited.contains(&file_path) {
                continue;
            }
            visited.insert(file_path.clone());

            // Load and parse module
            let source_code = std::fs::read_to_string(&file_path)
                .map_err(|_e| ModuleResolutionError::ModuleNotFound {
                    module_path: file_path.to_string_lossy().to_string(),
                    searched_paths: vec![file_path.clone()], // Only one path tried here
                })?;

            let module_path = self.file_path_to_module_path(&file_path)?;

            // Extract dependencies from use statements
            let dependencies = self.extract_dependencies(&source_code);

            let descriptor = ModuleDescriptor {
                module_path: module_path.clone(),
                file_path: file_path.clone(),
                source_code,
                dependencies: dependencies.clone(),
                is_entry_point: file_path == entry_point,
            };

            graph.add_module(descriptor);

            // Queue local module dependencies for discovery
            for dep in dependencies {
                // Only process local modules (not quoted strings)
                if !dep.starts_with('"') {
                    if let Ok(dep_path) = self.module_path_to_file_path(&dep) {
                        if dep_path.exists() && !visited.contains(&dep_path) {
                            to_process.push_back(dep_path);
                        }
                    } else {
                        // Handle error from module_path_to_file_path
                        return Err(ModuleResolutionError::InvalidModulePath(dep));
                    }
                }
            }
        }

        // Build dependency graph and compute order
        for (module_path, descriptor) in graph.modules.clone() {
            for dep in &descriptor.dependencies {
                // Convert local module paths to module names
                if !dep.starts_with('"') {
                    graph.add_dependency(module_path.clone(), dep.clone());
                }
            }
        }

        graph.compute_compilation_order()?;
        Ok(graph)
    }

    /// Convert file path to module path
    fn file_path_to_module_path(&self, file_path: &Path) -> Result<String, ModuleResolutionError> {
        let relative = file_path
            .strip_prefix(&self.source_dir)
            .map_err(|e| ModuleResolutionError::InvalidModulePath(e.to_string()))?;

        let module_path = relative
            .with_extension("")
            .components()
            .map(|c| c.as_os_str().to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join("::");

        // Handle mod.v files specially
        if module_path.ends_with("::mod") {
            Ok(module_path.trim_end_matches("::mod").to_string())
        } else {
            Ok(module_path)
        }
    }

    /// Convert module path to file path
    fn module_path_to_file_path(&self, module_path: &str) -> Result<PathBuf, ModuleResolutionError> {
        let path_parts: Vec<&str> = module_path.split("::").collect();
        let mut file_path = self.source_dir.clone();

        // Try file first, then directory/mod.v
        for (i, part) in path_parts.iter().enumerate() {
            file_path.push(part);
            if i == path_parts.len() - 1 {
                // Last component: try both .v and /mod.v
                file_path.set_extension("v");
                return Ok(file_path);
            }
        }

        Ok(file_path)
    }

    /// Extract use statement dependencies from source code
    fn extract_dependencies(&self, source_code: &str) -> Vec<String> {
        let mut dependencies = Vec::new();

        // Search for use statements anywhere in the source, not just at line start
        let mut remaining = source_code;
        while let Some(pos) = remaining.find("use ") {
            // Make sure "use" is preceded by whitespace or punctuation (not part of identifier)
            if pos > 0 {
                let prev_char = remaining.chars().nth(pos - 1).unwrap();
                if prev_char.is_alphanumeric() || prev_char == '_' {
                    // "use" is part of a larger identifier, skip it
                    remaining = &remaining[pos + 4..];
                    continue;
                }
            }

            // Extract the text after "use "
            let after_use = &remaining[pos + 4..];

            // Find the end of the use statement (semicolon)
            if let Some(semicolon_pos) = after_use.find(';') {
                let use_text = &after_use[..semicolon_pos];

                // Parse: use identifier or use "string"
                if let Some(import_target) = self.parse_use_statement(use_text) {
                    dependencies.push(import_target);
                }
            }

            // Move past this occurrence
            remaining = &remaining[pos + 4..];
        }

        dependencies
    }

    /// Parse a use statement and extract the import target
    /// The input should be the text between "use " and ";" (no semicolon included)
    fn parse_use_statement(&self, rest: &str) -> Option<String> {
        let rest = rest.trim();

        // Handle quoted strings: use "token"::{...}
        if rest.starts_with('"') {
            if let Some(end_quote) = rest[1..].find('"') {
                let content = &rest[1..=end_quote];
                return Some(format!("\"{}\"", content));
            }
        }

        // Handle unquoted identifiers: use types or use utils::helpers
        if let Some(brace) = rest.find('{') {
            let import_path = rest[..brace].trim();
            if !import_path.is_empty() && import_path.chars().next().unwrap().is_alphabetic() {
                return Some(import_path.to_string());
            }
        } else if !rest.is_empty() && rest.chars().next().unwrap().is_alphabetic() {
            // Simple identifier or path like "lib" or "utils::helpers"
            return Some(rest.to_string());
        }

        None
    }
}

/// Detect the type of import target
pub fn detect_import_target(input: &str) -> ImportTarget {
    // Check if it's a quoted string
    if input.starts_with('"') && input.ends_with('"') {
        let inner = &input[1..input.len() - 1];

        // Check if it's a valid Solana base58 pubkey
        if is_valid_solana_pubkey(inner) {
            return ImportTarget::SolanaPubkey {
                address: inner.to_string(),
            };
        }

        // Otherwise treat as registry name
        return ImportTarget::RegistryName {
            name: inner.to_string(),
        };
    }

    // Unquoted: must be local module
    ImportTarget::LocalModule {
        module_path: input.to_string(),
        resolved_file: None,
    }
}

/// Check if a string is a valid Solana base58 pubkey
pub fn is_valid_solana_pubkey(s: &str) -> bool {
    // Solana pubkeys are 32-58 base58 characters
    if s.len() < 32 || s.len() > 58 {
        return false;
    }

    s.chars().all(is_base58_char)
}

/// Check if a character is valid base58
fn is_base58_char(c: char) -> bool {
    matches!(c,
        '1'..='9' | 'A'..='H' | 'J'..='N' | 'P'..='Z' | 'a'..='k' | 'm'..='z'
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_solana_pubkey_detection() {
        // Valid pubkey (44 chars, all base58)
        assert!(is_valid_solana_pubkey(
            "HMxPuYGdU7qT3MZqPHM8bCp7g5P2uGkk3qHvB7LhPp4"
        ));

        // Too short
        assert!(!is_valid_solana_pubkey("short"));

        // Invalid characters (contains O, I, l)
        assert!(!is_valid_solana_pubkey(
            "HMxPuYGdU7qT3MZqPHM8bCp7g5P2uGkk3qHvB7LhPpOOO"
        ));
    }

    #[test]
    fn test_import_target_detection() {
        // Local module
        let target = detect_import_target("types");
        assert!(matches!(target, ImportTarget::LocalModule { .. }));

        // Local with path
        let target = detect_import_target("utils::helpers");
        assert!(matches!(target, ImportTarget::LocalModule { .. }));

        // Registry name
        let target = detect_import_target("\"token\"");
        assert!(matches!(target, ImportTarget::RegistryName { .. }));

        // Solana address
        let target = detect_import_target("\"HMxPuYGdU7qT3MZqPHM8bCp7g5P2uGkk3qHvB7LhPp4\"");
        assert!(matches!(target, ImportTarget::SolanaPubkey { .. }));
    }

    #[test]
    fn test_module_graph_topological_sort() {
        let mut graph = ModuleGraph::new();

        // Add modules
        graph.add_module(ModuleDescriptor {
            module_path: "main".to_string(),
            file_path: PathBuf::from("main.v"),
            source_code: String::new(),
            dependencies: vec!["types".to_string(), "utils".to_string()],
            is_entry_point: true,
        });

        graph.add_module(ModuleDescriptor {
            module_path: "types".to_string(),
            file_path: PathBuf::from("types.v"),
            source_code: String::new(),
            dependencies: vec![],
            is_entry_point: false,
        });

        graph.add_module(ModuleDescriptor {
            module_path: "utils".to_string(),
            file_path: PathBuf::from("utils.v"),
            source_code: String::new(),
            dependencies: vec!["types".to_string()],
            is_entry_point: false,
        });

        // Add edges
        graph.add_dependency("main".to_string(), "types".to_string());
        graph.add_dependency("main".to_string(), "utils".to_string());
        graph.add_dependency("utils".to_string(), "types".to_string());

        graph.compute_compilation_order().unwrap();

        let order = graph.compilation_order();
        // types must come before utils, utils must come before main
        let types_pos = order.iter().position(|m| m == "types").unwrap();
        let utils_pos = order.iter().position(|m| m == "utils").unwrap();
        let main_pos = order.iter().position(|m| m == "main").unwrap();

        assert!(types_pos < utils_pos);
        assert!(utils_pos < main_pos);
    }

    #[test]
    fn test_cycle_detection() {
        let mut graph = ModuleGraph::new();

        // Add modules with cycle
        graph.add_module(ModuleDescriptor {
            module_path: "a".to_string(),
            file_path: PathBuf::from("a.v"),
            source_code: String::new(),
            dependencies: vec!["b".to_string()],
            is_entry_point: true,
        });

        graph.add_module(ModuleDescriptor {
            module_path: "b".to_string(),
            file_path: PathBuf::from("b.v"),
            source_code: String::new(),
            dependencies: vec!["a".to_string()],
            is_entry_point: false,
        });

        graph.add_dependency("a".to_string(), "b".to_string());
        graph.add_dependency("b".to_string(), "a".to_string());

        let result = graph.detect_cycles();
        assert!(result.is_err());
    }
}
