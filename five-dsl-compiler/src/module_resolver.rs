/// Module resolution for multi-file compilation.
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
// use five_vm_mito::error::VMError;
use crate::ast::AstNode;
use crate::config::{DependencyLink, DependencySource, NormalizedDependency, ProjectConfig};
use crate::error::ModuleResolutionError;
use crate::parser::DslParser;
use crate::stdlib_registry::{
    bundled_stdlib_module_path, find_bundled_stdlib_root, STDLIB_DEFAULT_ALIAS,
    STDLIB_PACKAGE_NAME,
};
use crate::tokenizer::DslTokenizer;

/// Represents a single module in the project.
#[derive(Debug, Clone)]
pub struct ModuleDescriptor {
    /// Module path like "types", "utils::helpers".
    pub module_path: String,
    /// Absolute path to the .v file.
    pub file_path: PathBuf,
    /// Loaded source code.
    pub source_code: String,
    /// Other modules this depends on.
    pub dependencies: Vec<String>,
    /// Whether this is the entry point.
    pub is_entry_point: bool,
}

/// Result of analyzing an import statement.
#[derive(Debug, Clone, PartialEq)]
pub enum ImportTarget {
    /// Local module: `use types;` or `use utils::helpers;`
    LocalModule {
        module_path: String,
        resolved_file: Option<PathBuf>,
    },
    /// Registry name: `use "token"::{...};`
    RegistryName { name: String },
    /// Structured scoped namespace target: `@domain/subprogram`.
    ScopedNamespace {
        symbol: char,
        domain: String,
        subprogram: String,
        canonical: String,
    },
    /// Direct Solana address: `use "HMxPuYGdU7..."::{...};`
    SolanaPubkey { address: String },
}

const NAMESPACE_SYMBOLS: [char; 5] = ['!', '@', '#', '$', '%'];

pub fn parse_scoped_namespace(input: &str) -> Option<(char, String, String, String)> {
    let mut chars = input.chars();
    let symbol = chars.next()?;
    if !NAMESPACE_SYMBOLS.contains(&symbol) {
        return None;
    }
    let rest: String = chars.collect();
    let (domain_raw, sub_raw) = rest.split_once('/')?;
    if sub_raw.contains('/') {
        return None;
    }

    let domain = domain_raw.to_ascii_lowercase();
    let subprogram = sub_raw.to_ascii_lowercase();

    let valid_segment = |seg: &str| {
        !seg.is_empty()
            && seg
                .chars()
                .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
    };
    if !valid_segment(&domain) || !valid_segment(&subprogram) {
        return None;
    }

    let canonical = format!("{}{}/{}", symbol, domain, subprogram);
    Some((symbol, domain, subprogram, canonical))
}

/// Graph of module dependencies.
#[derive(Debug)]
pub struct ModuleGraph {
    /// All discovered modules.
    modules: HashMap<String, ModuleDescriptor>,
    /// Edges in dependency graph: module -> list of dependencies.
    dependency_edges: HashMap<String, Vec<String>>,
    /// Topologically sorted compilation order.
    compilation_order: Vec<String>,
}

impl ModuleGraph {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
            dependency_edges: HashMap::new(),
            compilation_order: Vec::new(),
        }
    }

    pub fn add_module(&mut self, descriptor: ModuleDescriptor) {
        let module_path = descriptor.module_path.clone();
        self.modules.insert(module_path.clone(), descriptor);
        self.dependency_edges
            .entry(module_path)
            .or_insert_with(Vec::new);
    }

    pub fn add_dependency(&mut self, from_module: String, to_module: String) {
        self.dependency_edges
            .entry(from_module)
            .or_default()
            .push(to_module);
    }

    pub fn get_module(&self, path: &str) -> Option<&ModuleDescriptor> {
        self.modules.get(path)
    }

    pub fn modules(&self) -> &HashMap<String, ModuleDescriptor> {
        &self.modules
    }

    pub fn compilation_order(&self) -> &[String] {
        &self.compilation_order
    }

    pub fn get_dependencies(&self, module_path: &str) -> Vec<String> {
        self.dependency_edges
            .get(module_path)
            .cloned()
            .unwrap_or_default()
    }

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
                    return Err(ModuleResolutionError::CircularDependency(
                        module.to_string(),
                    ));
                }
            }
        }

        rec_stack.remove(module);
        Ok(())
    }

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
            return Err(ModuleResolutionError::Generic(
                "Topological sort failed: graph has cycles or unresolvable dependencies"
                    .to_string(),
            ));
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

#[derive(Debug, Clone)]
struct DependencyContext {
    project_root: PathBuf,
    dependencies: HashMap<String, NormalizedDependency>,
    stdlib_root: Option<PathBuf>,
}

#[derive(Debug, Clone)]
struct ResolvedDependencyModule {
    module_path: String,
    file_path: PathBuf,
    source_code: String,
}

impl ModuleDiscoverer {
    /// Create a new discoverer for a source directory
    pub fn new(source_dir: PathBuf) -> Self {
        Self { source_dir }
    }

    /// Discover all modules starting from an entry point file
    pub fn discover_modules(
        &self,
        entry_point: &Path,
    ) -> Result<ModuleGraph, ModuleResolutionError> {
        let dep_ctx = self.load_dependency_context()?;
        let mut graph = ModuleGraph::new();
        let mut visited_files = HashSet::new();
        let mut to_process = VecDeque::new();

        // Start with entry point
        to_process.push_back(entry_point.to_path_buf());

        while let Some(file_path) = to_process.pop_front() {
            if visited_files.contains(&file_path) {
                continue;
            }
            visited_files.insert(file_path.clone());

            // Load and parse module
            let source_code = std::fs::read_to_string(&file_path).map_err(|_e| {
                ModuleResolutionError::ModuleNotFound {
                    module_path: file_path.to_string_lossy().to_string(),
                    searched_paths: vec![file_path.clone()], // Only one path tried here
                }
            })?;

            if self.source_mentions_std_import(&source_code) {
                if let Some(std_dep) = dep_ctx.dependencies.get(STDLIB_DEFAULT_ALIAS) {
                    if std_dep.package != STDLIB_PACKAGE_NAME {
                        return Err(ModuleResolutionError::Generic(format!(
                            "Dependency '{}' must map to package '{}'",
                            STDLIB_DEFAULT_ALIAS, STDLIB_PACKAGE_NAME
                        )));
                    }
                    if std_dep.source != DependencySource::Bundled
                        || std_dep.link != DependencyLink::Inline
                    {
                        return Err(ModuleResolutionError::Generic(
                            "Dependency 'std' is declared but not enabled for this phase. Use source='bundled' and link='inline'.".to_string(),
                        ));
                    }
                } else if dep_ctx.stdlib_root.is_none() {
                    return Err(ModuleResolutionError::Generic(
                        "Missing required std dependency. Add to five.toml:\n[dependencies]\nstd = { package = \"@5ive/std\", version = \"0.1.0\", source = \"bundled\", link = \"inline\" }".to_string(),
                    ));
                }
            }

            let module_path = self.file_path_to_module_path(&file_path)?;

            // Extract dependencies from use/import statements
            let dependencies = self.extract_dependencies(&source_code, &dep_ctx);

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
                if dep.starts_with('"') {
                    continue;
                }

                if (dep == STDLIB_DEFAULT_ALIAS || dep.starts_with("std::"))
                    && !dep_ctx.dependencies.contains_key(STDLIB_DEFAULT_ALIAS)
                    && dep_ctx.stdlib_root.is_none()
                {
                    return Err(ModuleResolutionError::Generic(
                        "Missing required std dependency. Add to five.toml:\n[dependencies]\nstd = { package = \"@5ive/std\", version = \"0.1.0\", source = \"bundled\", link = \"inline\" }".to_string(),
                    ));
                }

                if graph.get_module(&dep).is_some() {
                    continue;
                }

                if let Some(resolved) = self.resolve_dependency_module(&dep, &dep_ctx)? {
                    let dep_deps = self.extract_dependencies(&resolved.source_code, &dep_ctx);
                    graph.add_module(ModuleDescriptor {
                        module_path: resolved.module_path.clone(),
                        file_path: resolved.file_path,
                        source_code: resolved.source_code,
                        dependencies: dep_deps.clone(),
                        is_entry_point: false,
                    });
                    continue;
                }

                if let Ok(dep_path) = self.module_path_to_file_path(&dep) {
                    if dep_path.exists() && !visited_files.contains(&dep_path) {
                        to_process.push_back(dep_path);
                    } else if !dep_path.exists() {
                        return Err(ModuleResolutionError::ModuleNotFound {
                            module_path: dep.clone(),
                            searched_paths: vec![dep_path],
                        });
                    }
                } else {
                    // Handle error from module_path_to_file_path
                    return Err(ModuleResolutionError::InvalidModulePath(dep));
                }
            }
        }

        // Build dependency graph and compute order
        for (module_path, descriptor) in graph.modules.clone() {
            for dep in &descriptor.dependencies {
                // Convert local module paths to module names
                if !dep.starts_with('"') && graph.modules.contains_key(dep) {
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
    fn module_path_to_file_path(
        &self,
        module_path: &str,
    ) -> Result<PathBuf, ModuleResolutionError> {
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

    /// Extract use/import dependencies from source code.
    fn extract_dependencies(&self, source_code: &str, dep_ctx: &DependencyContext) -> Vec<String> {
        let mut tokenizer = DslTokenizer::new(source_code);
        let mut out = Vec::new();
        let mut parsed_with_ast = false;
        if let Ok(tokens) = tokenizer.tokenize() {
            if let Ok(ast) = DslParser::new(tokens).parse() {
                parsed_with_ast = true;
                if let AstNode::Program {
                    import_statements, ..
                } = ast
                {
                    for import in import_statements {
                        if let AstNode::ImportStatement {
                            module_specifier,
                            imported_items,
                            ..
                        } = import
                        {
                            match module_specifier {
                                crate::ast::ModuleSpecifier::Local(name) => out.push(name),
                                crate::ast::ModuleSpecifier::Nested(path) => {
                                    if let Some(items) = imported_items {
                                        if !items.is_empty() {
                                            out.push(path.join("::"));
                                            continue;
                                        }
                                    }

                                    let full = path.join("::");
                                    let full_is_module =
                                        self.dependency_module_exists(&full, dep_ctx);

                                    if full_is_module {
                                        out.push(full);
                                        continue;
                                    }

                                    if path.len() > 1 {
                                        let parent = path[..path.len() - 1].join("::");
                                        let parent_is_module =
                                            self.dependency_module_exists(&parent, dep_ctx);
                                        if parent_is_module {
                                            out.push(parent);
                                            continue;
                                        }
                                    }

                                    out.push(full);
                                }
                                crate::ast::ModuleSpecifier::External(addr) => {
                                    out.push(format!("\"{}\"", addr))
                                }
                                crate::ast::ModuleSpecifier::Namespace(ns) => {
                                    out.push(format!("\"{}\"", ns.import_key()))
                                }
                            }
                        }
                    }
                }
            }
        }

        if !parsed_with_ast {
            out.extend(self.extract_use_paths_from_source(source_code));
        }

        out.sort();
        out.dedup();
        out
    }

    fn extract_use_paths_from_source(&self, source_code: &str) -> Vec<String> {
        let mut out = Vec::new();
        for raw_line in source_code.lines() {
            let line = raw_line.trim();
            let (prefix, body) = if let Some(rest) = line.strip_prefix("pub use ") {
                ("pub_use", rest)
            } else if let Some(rest) = line.strip_prefix("pub import ") {
                ("pub_import", rest)
            } else if let Some(rest) = line.strip_prefix("use ") {
                ("use", rest)
            } else if let Some(rest) = line.strip_prefix("import ") {
                ("import", rest)
            } else {
                continue;
            };

            if prefix.is_empty() {
                continue;
            }

            let mut target = body.split(';').next().unwrap_or("").trim();
            if target.is_empty() {
                continue;
            }
            if let Some((lhs, _)) = target.split_once(" as ") {
                target = lhs.trim();
            }
            if let Some((lhs, _)) = target.split_once("::{") {
                target = lhs.trim();
            }
            if target.is_empty() {
                continue;
            }
            out.push(target.to_string());
        }
        out
    }

    fn source_mentions_std_import(&self, source_code: &str) -> bool {
        self.extract_use_paths_from_source(source_code)
            .iter()
            .any(|path| path == STDLIB_DEFAULT_ALIAS || path.starts_with("std::"))
    }

    fn dependency_module_exists(&self, module_path: &str, dep_ctx: &DependencyContext) -> bool {
        if self
            .module_path_to_file_path(module_path)
            .map(|p| p.exists())
            .unwrap_or(false)
        {
            return true;
        }

        matches!(
            self.resolve_dependency_module(module_path, dep_ctx),
            Ok(Some(_))
        )
    }

    fn find_project_config_path(&self) -> Option<PathBuf> {
        let mut cursor = Some(self.source_dir.clone());
        while let Some(dir) = cursor {
            let candidate = dir.join("five.toml");
            if candidate.exists() {
                return Some(candidate);
            }
            cursor = dir.parent().map(|p| p.to_path_buf());
        }
        None
    }

    fn load_dependency_context(&self) -> Result<DependencyContext, ModuleResolutionError> {
        let Some(config_path) = self.find_project_config_path() else {
            let project_root = self.source_dir.clone();
            return Ok(DependencyContext {
                project_root: project_root.clone(),
                dependencies: HashMap::new(),
                stdlib_root: find_bundled_stdlib_root(&project_root),
            });
        };

        let project_root = config_path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| self.source_dir.clone());
        let config = ProjectConfig::load(&config_path).map_err(|_| {
            ModuleResolutionError::Generic(format!(
                "Failed to parse project configuration at {}",
                config_path.display()
            ))
        })?;
        let dependencies = config.normalized_dependencies().map_err(|_| {
            ModuleResolutionError::Generic(format!(
                "Invalid dependency configuration in {}",
                config_path.display()
            ))
        })?;
        let stdlib_root = find_bundled_stdlib_root(&project_root);

        Ok(DependencyContext {
            project_root,
            dependencies,
            stdlib_root,
        })
    }

    fn resolve_dependency_module(
        &self,
        module_path: &str,
        dep_ctx: &DependencyContext,
    ) -> Result<Option<ResolvedDependencyModule>, ModuleResolutionError> {
        let mut parts = module_path.split("::");
        let alias = match parts.next() {
            Some(value) if !value.is_empty() => value,
            _ => return Ok(None),
        };

        let tail = parts.collect::<Vec<_>>().join("::");

        if alias == STDLIB_DEFAULT_ALIAS && !dep_ctx.dependencies.contains_key(alias) {
            let stdlib_root = dep_ctx.stdlib_root.as_ref().ok_or_else(|| {
                ModuleResolutionError::Generic(
                    "Unable to resolve bundled stdlib root. Set FIVE_STDLIB_ROOT or use five-cli bundled assets."
                        .to_string(),
                )
            })?;
            let effective_tail = if tail.is_empty() {
                "prelude".to_string()
            } else {
                tail.clone()
            };
            let module_file = bundled_stdlib_module_path(stdlib_root, &effective_tail);
            let source_code =
                std::fs::read_to_string(&module_file).map_err(|_| ModuleResolutionError::ModuleNotFound {
                    module_path: module_path.to_string(),
                    searched_paths: vec![module_file.clone()],
                })?;
            return Ok(Some(ResolvedDependencyModule {
                module_path: module_path.to_string(),
                file_path: module_file,
                source_code,
            }));
        }

        let Some(dep) = dep_ctx.dependencies.get(alias) else {
            return Ok(None);
        };

        match (dep.source, dep.link) {
            (DependencySource::Bundled, DependencyLink::Inline) => {
                if dep.package != STDLIB_PACKAGE_NAME {
                    return Err(ModuleResolutionError::Generic(format!(
                        "Bundled dependency '{}' is unsupported (only package '{}' is currently supported)",
                        alias, STDLIB_PACKAGE_NAME
                    )));
                }
                let stdlib_root = dep_ctx.stdlib_root.as_ref().ok_or_else(|| {
                    ModuleResolutionError::Generic(
                        "Unable to resolve bundled stdlib root. Set FIVE_STDLIB_ROOT or use five-cli bundled assets."
                            .to_string(),
                    )
                })?;
                let effective_tail = if tail.is_empty() {
                    "prelude".to_string()
                } else {
                    tail
                };
                let module_file = bundled_stdlib_module_path(stdlib_root, &effective_tail);
                let source_code = std::fs::read_to_string(&module_file).map_err(|_| {
                    ModuleResolutionError::ModuleNotFound {
                        module_path: module_path.to_string(),
                        searched_paths: vec![module_file.clone()],
                    }
                })?;
                Ok(Some(ResolvedDependencyModule {
                    module_path: module_path.to_string(),
                    file_path: module_file,
                    source_code,
                }))
            }
            (DependencySource::Path, DependencyLink::Inline) => {
                let dep_path = dep.path.as_ref().ok_or_else(|| {
                    ModuleResolutionError::Generic(format!(
                        "Dependency '{}' with source=path is missing 'path'",
                        alias
                    ))
                })?;
                let dep_root = {
                    let candidate = PathBuf::from(dep_path);
                    if candidate.is_absolute() {
                        candidate
                    } else {
                        dep_ctx.project_root.join(candidate)
                    }
                };

                let dep_config_path = dep_root.join("five.toml");
                let dep_project = if dep_config_path.exists() {
                    Some(ProjectConfig::load(&dep_config_path).map_err(|_| {
                        ModuleResolutionError::Generic(format!(
                            "Failed to parse dependency project configuration at {}",
                            dep_config_path.display()
                        ))
                    })?)
                } else {
                    None
                };

                let module_file = if tail.is_empty() {
                    let Some(dep_project) = dep_project.as_ref() else {
                        return Err(ModuleResolutionError::Generic(format!(
                            "Dependency '{}' imported without module tail requires a five.toml entry_point",
                            alias
                        )));
                    };
                    let entry_point = dep_project.get_entry_point().ok_or_else(|| {
                        ModuleResolutionError::Generic(format!(
                            "Dependency '{}' imported without module tail requires entry_point",
                            alias
                        ))
                    })?;
                    dep_root.join(entry_point)
                } else {
                    let dep_source_dir = dep_project
                        .as_ref()
                        .map(|cfg| cfg.get_source_dir())
                        .unwrap_or_else(|| PathBuf::from("src"));
                    dep_root
                        .join(dep_source_dir)
                        .join(tail.replace("::", "/"))
                        .with_extension("v")
                };

                let source_code = std::fs::read_to_string(&module_file).map_err(|_| {
                    ModuleResolutionError::ModuleNotFound {
                        module_path: module_path.to_string(),
                        searched_paths: vec![module_file.clone()],
                    }
                })?;

                Ok(Some(ResolvedDependencyModule {
                    module_path: module_path.to_string(),
                    file_path: module_file,
                    source_code,
                }))
            }
            (DependencySource::Namespace, DependencyLink::External)
            | (DependencySource::Address, DependencyLink::External)
            | (DependencySource::Moat, DependencyLink::External) => {
                Err(ModuleResolutionError::Generic(format!(
                    "Dependency '{}' is declared with source='{}' and link='{}', but external dependency resolution is not enabled yet",
                    alias,
                    match dep.source {
                        DependencySource::Bundled => "bundled",
                        DependencySource::Path => "path",
                        DependencySource::Namespace => "namespace",
                        DependencySource::Address => "address",
                        DependencySource::Moat => "moat",
                    },
                    match dep.link {
                        DependencyLink::Inline => "inline",
                        DependencyLink::External => "external",
                    }
                )))
            }
            _ => Err(ModuleResolutionError::Generic(format!(
                "Dependency '{}' has unsupported source/link combination",
                alias
            ))),
        }
    }
}

pub fn canonical_module_name(
    path: &Path,
    source_root: &Path,
) -> Result<String, ModuleResolutionError> {
    let relative = path
        .strip_prefix(source_root)
        .map_err(|e| ModuleResolutionError::InvalidModulePath(e.to_string()))?;

    let name = relative
        .with_extension("")
        .components()
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join("::");

    if name.ends_with("::mod") {
        Ok(name.trim_end_matches("::mod").to_string())
    } else {
        Ok(name)
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

        if let Some((symbol, domain, subprogram, canonical)) = parse_scoped_namespace(inner) {
            return ImportTarget::ScopedNamespace {
                symbol,
                domain,
                subprogram,
                canonical,
            };
        }

        // Otherwise treat as registry name
        return ImportTarget::RegistryName {
            name: inner.to_string(),
        };
    }

    if let Some((symbol, domain, subprogram, canonical)) = parse_scoped_namespace(input) {
        return ImportTarget::ScopedNamespace {
            symbol,
            domain,
            subprogram,
            canonical,
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

        // Scoped namespace (bare)
        let target = detect_import_target("@5ive-tech/program");
        assert!(matches!(target, ImportTarget::ScopedNamespace { .. }));

        // Scoped namespace (quoted)
        let target = detect_import_target("\"@5ive-tech/program\"");
        assert!(matches!(target, ImportTarget::ScopedNamespace { .. }));
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
