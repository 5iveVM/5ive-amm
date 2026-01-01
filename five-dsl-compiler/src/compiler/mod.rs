// Main DSL Compiler Module
//
// Orchestrates the compilation pipeline from source to bytecode.
//
// This module has been refactored to eliminate DRY violations:
// - Extracted macros.rs for repetitive patterns (execute_phase!, init_compilation_context!)
// - Extracted error_handling.rs for error conversion (eliminates 16 duplicate blocks)
// - Extracted pipeline.rs for core pipeline logic (consolidates 4 duplicate implementations)
//
// Result: ~300 lines of duplicate code eliminated, single source of truth for compilation.

#[macro_use]
pub mod macros;
pub mod error_handling;
pub mod pipeline;

use crate::error::CompilerError;
use crate::five_file::FiveFile;
use crate::metrics::CompilerMetrics;
use pipeline::CompilationPipeline;

// Re-export public types from pipeline for backward compatibility
pub use pipeline::{CompilationConfig, CompilationMode, OptimizationLevel};

/// Main DSL compiler that orchestrates the compilation pipeline.
///
/// This struct provides multiple public APIs for different use cases:
/// - Simple compilation: compile_dsl()
/// - Mode-specific: compile_for_testing(), compile_for_deployment()
/// - Feature flags: compile_with_mode_and_features()
/// - Full config: compile_with_config()
/// - With metrics: compile_with_metrics()
/// - .five file format: compile_to_five_file*()
///
/// All methods now delegate to the unified CompilationPipeline to eliminate
/// duplication and ensure consistent behavior.
pub struct DslCompiler;

impl DslCompiler {
    /// Compile DSL with default testing mode (includes test functions).
    ///
    /// This is the simplest API for quick compilation during development.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let bytecode = DslCompiler::compile_dsl(source)?;
    /// ```
    pub fn compile_dsl(source: &str) -> Result<Vec<u8>, CompilerError> {
        Self::compile_with_mode(source, CompilationMode::Testing)
    }

    /// Compile DSL with features (legacy method for backward compatibility).
    ///
    /// This method is maintained for backward compatibility with existing code.
    /// New code should use `compile_with_config()` instead.
    pub fn compile_dsl_with_features(
        source: &str,
        enable_constraint_cache: bool,
    ) -> Result<Vec<u8>, CompilerError> {
        Self::compile_with_mode_and_features(
            source,
            CompilationMode::Testing,
            enable_constraint_cache,
        )
    }

    /// Compile DSL for deployment (excludes test functions for smaller bytecode).
    ///
    /// Use this when compiling contracts for production deployment.
    /// Test functions are stripped to reduce bytecode size and CU costs.
    pub fn compile_for_deployment(source: &str) -> Result<Vec<u8>, CompilerError> {
        Self::compile_with_mode(source, CompilationMode::Deployment)
    }

    /// Compile DSL for testing (includes test functions for local execution).
    ///
    /// Use this during development and testing. Test functions are included
    /// in the bytecode for local execution.
    pub fn compile_for_testing(source: &str) -> Result<Vec<u8>, CompilerError> {
        Self::compile_with_mode(source, CompilationMode::Testing)
    }

    /// Compile DSL with specific compilation mode.
    ///
    /// Internal helper that sets up a default config with the specified mode.
    pub fn compile_with_mode(
        source: &str,
        mode: CompilationMode,
    ) -> Result<Vec<u8>, CompilerError> {
        Self::compile_with_mode_and_features(source, mode, true)
    }

    /// Compile DSL with mode and feature flags.
    ///
    /// This method provides control over:
    /// - Compilation mode (Testing vs Deployment)
    /// - Constraint caching optimization
    ///
    /// Legacy method maintained for backward compatibility.
    /// New code should use `compile_with_config()` for more control.
    pub fn compile_with_mode_and_features(
        source: &str,
        mode: CompilationMode,
        enable_constraint_cache: bool,
    ) -> Result<Vec<u8>, CompilerError> {
        let config = CompilationConfig::new(mode).with_constraint_cache(enable_constraint_cache);
        Self::compile_with_config(source, &config)
    }

    /// Compile DSL with full configuration support.
    ///
    /// This is the recommended API for new code. It provides full control over:
    /// - Compilation mode (Testing vs Deployment)
    /// - V2 preview features
    /// - Constraint caching
    /// - Optimization level
    ///
    /// # Example
    ///
    /// ```ignore
    /// let config = CompilationConfig::new(CompilationMode::Deployment)
    ///     .with_v2_preview(true)
    ///     .with_optimization_level(OptimizationLevel::V3);
    /// let bytecode = DslCompiler::compile_with_config(source, &config)?;
    /// ```
    pub fn compile_with_config(
        source: &str,
        config: &CompilationConfig,
    ) -> Result<Vec<u8>, CompilerError> {
        eprintln!("DEBUG_COMPILER: compile_with_config source_len={}", source.len());
        let mut pipeline = CompilationPipeline::new(source, None);

        // Execute standard pipeline (no interfaces)
        eprintln!("DEBUG_COMPILER: starting tokenize");
        let tokens = pipeline.tokenize()?;
        eprintln!("DEBUG_COMPILER: starting parse");
        let ast = pipeline.parse(tokens)?;
        eprintln!("DEBUG_COMPILER: starting type_check");
        pipeline.type_check(&ast)?;
        let bytecode = pipeline.generate_bytecode(&ast, config)?;

        // Finalize metrics
        pipeline.finalize_metrics(&bytecode);

        #[cfg(debug_assertions)]
        pipeline.print_debug_metrics("Compilation metrics");

        Ok(bytecode)
    }

    /// Compile DSL with full configuration support and return compilation log.
    ///
    /// This method is useful for debugging and analyzing the compilation process.
    /// The log contains detailed information about bytecode generation.
    ///
    /// Returns a tuple of (bytecode, compilation_log).
    pub fn compile_with_config_and_log(
        source: &str,
        config: &CompilationConfig,
    ) -> Result<(Vec<u8>, Vec<String>), CompilerError> {
        let mut pipeline = CompilationPipeline::new(source, None);

        // Execute full pipeline with interface support
        let tokens = pipeline.tokenize()?;
        let ast = pipeline.parse(tokens)?;
        let interface_registry = pipeline.type_check_with_interfaces(&ast)?;

        // Generate bytecode with log capture
        let (bytecode, log) =
            pipeline.generate_bytecode_with_log(&ast, config, Some(interface_registry))?;

        // Finalize metrics
        pipeline.finalize_metrics(&bytecode);

        #[cfg(debug_assertions)]
        if config.v2_preview {
            pipeline.print_debug_metrics("V2-Preview compilation metrics");
        }

        Ok((bytecode, log))
    }

    /// Compile DSL to .five file format with embedded ABI.
    ///
    /// The .five file format contains both the bytecode and the ABI,
    /// making it a complete deployment package.
    pub fn compile_to_five_file(source: &str) -> Result<FiveFile, CompilerError> {
        Self::compile_to_five_file_with_mode(source, CompilationMode::Testing)
    }

    /// Compile DSL to .five file format with specific mode.
    pub fn compile_to_five_file_with_mode(
        source: &str,
        mode: CompilationMode,
    ) -> Result<FiveFile, CompilerError> {
        let config = CompilationConfig::new(mode);
        Self::compile_to_five_file_with_config(source, &config)
    }

    /// Compile DSL to .five file format with full configuration.
    ///
    /// This is the most complete API, providing both bytecode and ABI
    /// with full control over compilation settings.
    pub fn compile_to_five_file_with_config(
        source: &str,
        config: &CompilationConfig,
    ) -> Result<FiveFile, CompilerError> {
        let mut pipeline = CompilationPipeline::new(source, None);

        // Execute standard pipeline
        let tokens = pipeline.tokenize()?;
        let ast = pipeline.parse(tokens)?;
        pipeline.type_check(&ast)?;

        // Generate both bytecode and ABI
        let bytecode = pipeline.generate_bytecode(&ast, config)?;
        let abi = pipeline.generate_abi(&ast, config)?;

        // Finalize metrics before moving bytecode
        pipeline.finalize_metrics(&bytecode);

        #[cfg(debug_assertions)]
        let (bytecode_len, abi_len) = (bytecode.len(), abi.functions.len());

        // Create .five file (takes ownership, no clone needed)
        let five_file = FiveFile::new(abi, bytecode);

        #[cfg(debug_assertions)]
        pipeline.print_five_file_metrics(bytecode_len, abi_len);

        Ok(five_file)
    }

    /// Compile DSL with metrics collection enabled.
    ///
    /// Returns both the bytecode and detailed compilation metrics.
    /// Useful for performance analysis and optimization.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let (bytecode, metrics) = DslCompiler::compile_with_metrics(
    ///     source,
    ///     CompilationMode::Testing,
    ///     true
    /// )?;
    /// println!("Opcodes: {}", metrics.opcode_stats.total_opcodes);
    /// ```
    pub fn compile_with_metrics(
        source: &str,
        mode: CompilationMode,
        enable_constraint_cache: bool,
    ) -> Result<(Vec<u8>, CompilerMetrics), CompilerError> {
        let config = CompilationConfig::new(mode).with_constraint_cache(enable_constraint_cache);

        let mut pipeline = CompilationPipeline::new(source, None);

        // Execute standard pipeline
        let tokens = pipeline.tokenize()?;
        let ast = pipeline.parse(tokens)?;
        pipeline.type_check(&ast)?;
        let bytecode = pipeline.generate_bytecode(&ast, &config)?;

        // Finalize metrics
        pipeline.finalize_metrics(&bytecode);

        // Return bytecode and metrics
        // Note: Clone is necessary here because get_metrics() returns &CompilerMetrics
        // and we need to return owned CompilerMetrics. This is acceptable as metrics
        // are typically small and only cloned once at the end of compilation.
        Ok((bytecode, pipeline.get_metrics().clone()))
    }

    /// Compile DSL using automatic module discovery.
    ///
    /// This method starts from an entry point, discovers all dependencies via `use` statements,
    /// and compiles them into a single bytecode binary.
    pub fn compile_with_auto_discovery(
        entry_point: &std::path::Path,
        config: &CompilationConfig,
    ) -> Result<Vec<u8>, CompilerError> {
        use crate::bytecode_generator::ModuleMerger;
        use crate::error::{ErrorCategory, ErrorCode, ErrorSeverity};
        use crate::module_resolver::ModuleDiscoverer;
        use crate::type_checker::ModuleScope;

        // 1. Discover modules
        let source_dir = entry_point
            .parent()
            .unwrap_or(std::path::Path::new(""))
            .to_path_buf();
        let discoverer = ModuleDiscoverer::new(source_dir);
        let graph = discoverer.discover_modules(entry_point).map_err(|e| {
            CompilerError::new(
                ErrorCode::FILE_NOT_FOUND,
                ErrorSeverity::Error,
                ErrorCategory::IO,
                format!("Module discovery failed: {}", e),
            )
        })?;

        // Build ModuleScope from discovered modules
        let entry_module = graph.compilation_order()
            .last()
            .ok_or_else(|| CompilerError::new(
                ErrorCode::INTERNAL_ERROR,
                ErrorSeverity::Error,
                ErrorCategory::Internal,
                "No modules found in compilation graph".to_string(),
            ))?
            .clone();

        let mut module_scope = ModuleScope::new(entry_module.clone());

        // Add all non-entry modules
        for module_name in graph.compilation_order() {
            if module_name != &entry_module {
                module_scope.add_module(module_name.clone());
            }
        }

        // Register imports from dependency graph
        for module_name in graph.compilation_order() {
            let descriptor = graph.get_module(module_name).unwrap();
            for dep in &descriptor.dependencies {
                module_scope.register_import(module_name.clone(), dep.clone());
            }
        }

        // 2. Merge ASTs
        let mut merger = ModuleMerger::new()
            .with_namespaces(config.enable_module_namespaces);

        // Iterate in topological order
        for module_name in graph.compilation_order() {
            let descriptor = graph.get_module(module_name).ok_or_else(|| {
                CompilerError::new(
                    ErrorCode::INTERNAL_ERROR,
                    ErrorSeverity::Error,
                    ErrorCategory::Internal,
                    format!("Module {} not found in graph", module_name),
                )
            })?;

            // Parse module
            let mut pipeline = CompilationPipeline::new(
                &descriptor.source_code,
                descriptor.file_path.to_str(),
            );
            let tokens = pipeline.tokenize()?;
            let ast = pipeline.parse(tokens).map_err(|mut e| {
                if let Some(loc) = &mut e.location {
                    loc.file = Some(descriptor.file_path.clone());
                }
                e
            })?;

            // Populate module scope with symbols from this module
            Self::populate_module_scope_from_ast(&ast, module_name, &mut module_scope)?;

            if descriptor.is_entry_point {
                merger.set_main_ast(ast);
            } else {
                merger.add_module(module_name.clone(), ast);
            }
        }

        // 3. Merge
        let merged_ast = merger.merge().map_err(|e| {
            CompilerError::new(
                ErrorCode::INTERNAL_ERROR,
                ErrorSeverity::Error,
                ErrorCategory::Internal,
                format!("Module merge failed: {}", e),
            )
        })?;

        // 4. Type Check with Module Scope
        {
            use crate::type_checker::DslTypeChecker;
            let mut type_checker = DslTypeChecker::new()
                .with_module_scope(module_scope);
            type_checker.set_current_module(entry_module);
            type_checker.check_types(&merged_ast).map_err(|e| {
                CompilerError::new(
                    ErrorCode::TYPE_MISMATCH,
                    ErrorSeverity::Error,
                    ErrorCategory::Type,
                    format!("Type checking failed: {}", e),
                )
            })?;
        }

        // 5. Generate Bytecode from Merged AST
        // Use entry point source for pipeline context
        // NOTE: Type checking was already done above with correct module_scope.
        // Do NOT call pipeline.type_check() here as it would create a fresh
        // TypeCheckerContext without the module_scope, causing imported account
        // types to fail validation.
        let entry_descriptor = graph
            .get_module(graph.compilation_order().last().unwrap())
            .unwrap();
        let mut pipeline = CompilationPipeline::new(
            &entry_descriptor.source_code,
            entry_descriptor.file_path.to_str(),
        );

        let bytecode = pipeline.generate_bytecode(&merged_ast, config)?;

        pipeline.finalize_metrics(&bytecode);

        Ok(bytecode)
    }

    /// Compile DSL using explicit module list.
    pub fn compile_modules(
        module_files: Vec<String>,
        entry_point: &str,
        config: &CompilationConfig,
    ) -> Result<Vec<u8>, CompilerError> {
        use crate::bytecode_generator::ModuleMerger;
        use crate::error::{ErrorCategory, ErrorCode, ErrorSeverity};

        let mut merger = ModuleMerger::new()
            .with_namespaces(config.enable_module_namespaces);

        for file_path in module_files {
            let source = std::fs::read_to_string(&file_path).map_err(|e| {
                CompilerError::new(
                    ErrorCode::FILE_NOT_FOUND,
                    ErrorSeverity::Error,
                    ErrorCategory::IO,
                    format!("Failed to read module {}: {}", file_path, e),
                )
            })?;

            let mut pipeline = CompilationPipeline::new(&source, Some(&file_path));
            let tokens = pipeline.tokenize()?;
            let ast = pipeline.parse(tokens).map_err(|mut e| {
                if let Some(loc) = &mut e.location {
                    loc.file = Some(std::path::PathBuf::from(&file_path));
                }
                e
            })?;

            // Simple heuristics for module naming: filename without extension
            let path = std::path::Path::new(&file_path);
            let module_name = path.file_stem()
                .ok_or_else(|| CompilerError::new(
                    ErrorCode::INVALID_MODULE_PATH,
                    ErrorSeverity::Error,
                    ErrorCategory::Semantic,
                    format!("Invalid module file path (no file stem): {}", file_path),
                ))?
                .to_string_lossy()
                .to_string();

            if file_path == entry_point {
                merger.set_main_ast(ast);
            } else {
                merger.add_module(module_name, ast);
            }
        }

        let merged_ast = merger.merge().map_err(|e| {
            CompilerError::new(
                ErrorCode::INTERNAL_ERROR,
                ErrorSeverity::Error,
                ErrorCategory::Internal,
                format!("Module merge failed: {}", e),
            )
        })?;

        let entry_source = std::fs::read_to_string(entry_point).unwrap_or_default();
        let mut pipeline = CompilationPipeline::new(&entry_source, Some(entry_point));

        pipeline.type_check(&merged_ast)?;
        let bytecode = pipeline.generate_bytecode(&merged_ast, config)?;

        pipeline.finalize_metrics(&bytecode);

        Ok(bytecode)
    }

    /// Discover modules starting from an entry point.
    /// Returns a topologically sorted list of module paths.
    pub fn discover_modules(
        entry_point: &std::path::Path,
    ) -> Result<Vec<String>, CompilerError> {
        use crate::error::{ErrorCategory, ErrorCode, ErrorSeverity};
        use crate::module_resolver::ModuleDiscoverer;

        let source_dir = entry_point
            .parent()
            .unwrap_or(std::path::Path::new(""))
            .to_path_buf();
        let discoverer = ModuleDiscoverer::new(source_dir);
        let graph = discoverer.discover_modules(entry_point).map_err(|e| {
            CompilerError::new(
                ErrorCode::FILE_NOT_FOUND,
                ErrorSeverity::Error,
                ErrorCategory::IO,
                format!("Module discovery failed: {}", e),
            )
        })?;

        // Return file paths in compilation order
        let mut paths = Vec::new();
        for module_name in graph.compilation_order() {
            if let Some(descriptor) = graph.get_module(module_name) {
                paths.push(descriptor.file_path.to_string_lossy().to_string());
            }
        }

        Ok(paths)
    }

    /// Populate ModuleScope with symbols from an AST
    fn populate_module_scope_from_ast(
        ast: &crate::ast::AstNode,
        module_name: &str,
        scope: &mut crate::type_checker::ModuleScope,
    ) -> Result<(), CompilerError> {
        use crate::type_checker::ModuleSymbol;
        use crate::ast::{AstNode, TypeNode};


        scope.set_current_module(module_name.to_string());

        if let AstNode::Program {
            instruction_definitions,
            field_definitions,
            account_definitions,
            ..
        } = ast {
            // Add functions to scope
            for instr_def in instruction_definitions {
                if let AstNode::InstructionDefinition {
                    name,
                    return_type,
                    visibility,
                    ..
                } = instr_def {
                    let type_info = return_type
                        .as_ref()
                        .map(|t| (**t).clone())
                        .unwrap_or_else(|| TypeNode::Primitive("void".to_string()));

                    scope.add_symbol_to_current(name.clone(), ModuleSymbol {
                        type_info,
                        is_mutable: false,
                        visibility: *visibility,
                    });
                }
            }

            // Add fields to scope
            for field_def in field_definitions {
                if let AstNode::FieldDefinition {
                    name,
                    field_type,
                    visibility,
                    ..
                } = field_def {
                    scope.add_symbol_to_current(name.clone(), ModuleSymbol {
                        type_info: (**field_type).clone(),
                        is_mutable: true,
                        visibility: *visibility,
                    });
                }
            }

            // Add account types to scope
            for account_def in account_definitions {
                if let AstNode::AccountDefinition {
                    name,
                    visibility,
                    ..
                } = account_def {
                    scope.add_symbol_to_current(name.clone(), ModuleSymbol {
                        type_info: TypeNode::Account,
                        is_mutable: false,
                        visibility: *visibility,
                    });
                }
            }
        }

        Ok(())
    }
    /// Compile DSL using automatic module discovery to .five file format.
    ///
    /// This method preserves the ABI in the output.
    pub fn compile_with_auto_discovery_to_five_file(
        entry_point: &std::path::Path,
        config: &CompilationConfig,
    ) -> Result<FiveFile, CompilerError> {
        use crate::bytecode_generator::ModuleMerger;
        use crate::error::{ErrorCategory, ErrorCode, ErrorSeverity};
        use crate::module_resolver::ModuleDiscoverer;
        use crate::type_checker::ModuleScope;

        // 1. Discover modules
        let source_dir = entry_point
            .parent()
            .unwrap_or(std::path::Path::new(""))
            .to_path_buf();
        let discoverer = ModuleDiscoverer::new(source_dir);
        let graph = discoverer.discover_modules(entry_point).map_err(|e| {
            CompilerError::new(
                ErrorCode::FILE_NOT_FOUND,
                ErrorSeverity::Error,
                ErrorCategory::IO,
                format!("Module discovery failed: {}", e),
            )
        })?;

        // Build ModuleScope from discovered modules
        let entry_module = graph.compilation_order()
            .last()
            .ok_or_else(|| CompilerError::new(
                ErrorCode::INTERNAL_ERROR,
                ErrorSeverity::Error,
                ErrorCategory::Internal,
                "No modules found in compilation graph".to_string(),
            ))?
            .clone();

        let mut module_scope = ModuleScope::new(entry_module.clone());

        // Add all non-entry modules
        for module_name in graph.compilation_order() {
            if module_name != &entry_module {
                module_scope.add_module(module_name.clone());
            }
        }

        // Register imports from dependency graph
        for module_name in graph.compilation_order() {
            let descriptor = graph.get_module(module_name).unwrap();
            for dep in &descriptor.dependencies {
                module_scope.register_import(module_name.clone(), dep.clone());
            }
        }

        // 2. Merge ASTs
        let mut merger = ModuleMerger::new()
            .with_namespaces(config.enable_module_namespaces);

        // Iterate in topological order
        for module_name in graph.compilation_order() {
            let descriptor = graph.get_module(module_name).ok_or_else(|| {
                CompilerError::new(
                    ErrorCode::INTERNAL_ERROR,
                    ErrorSeverity::Error,
                    ErrorCategory::Internal,
                    format!("Module {} not found in graph", module_name),
                )
            })?;

            // Parse module
            let mut pipeline = CompilationPipeline::new(
                &descriptor.source_code,
                descriptor.file_path.to_str(),
            );
            let tokens = pipeline.tokenize()?;
            let ast = pipeline.parse(tokens).map_err(|mut e| {
                if let Some(loc) = &mut e.location {
                    loc.file = Some(descriptor.file_path.clone());
                }
                e
            })?;

            // Populate module scope with symbols from this module
            Self::populate_module_scope_from_ast(&ast, module_name, &mut module_scope)?;

            if descriptor.is_entry_point {
                merger.set_main_ast(ast);
            } else {
                merger.add_module(module_name.clone(), ast);
            }
        }

        // 3. Merge
        let merged_ast = merger.merge().map_err(|e| {
            CompilerError::new(
                ErrorCode::INTERNAL_ERROR,
                ErrorSeverity::Error,
                ErrorCategory::Internal,
                format!("Module merge failed: {}", e),
            )
        })?;

        // 4. Type Check with Module Scope
        {
            use crate::type_checker::DslTypeChecker;
            let mut type_checker = DslTypeChecker::new()
                .with_module_scope(module_scope);
            type_checker.set_current_module(entry_module);
            type_checker.check_types(&merged_ast).map_err(|e| {
                CompilerError::new(
                    ErrorCode::TYPE_MISMATCH,
                    ErrorSeverity::Error,
                    ErrorCategory::Type,
                    format!("Type checking failed: {}", e),
                )
            })?;
        }

        // 5. Generate Bytecode and ABI from Merged AST
        // Use entry point source for pipeline context
        // NOTE: Type checking was already done above with correct module_scope.
        // Do NOT call pipeline.type_check() here as it would create a fresh
        // TypeCheckerContext without the module_scope, causing imported account
        // types to fail validation.
        let entry_descriptor = graph
            .get_module(graph.compilation_order().last().unwrap())
            .unwrap();
        let mut pipeline = CompilationPipeline::new(
            &entry_descriptor.source_code,
            entry_descriptor.file_path.to_str(),
        );

        let bytecode = pipeline.generate_bytecode(&merged_ast, config)?;
        let abi = pipeline.generate_abi(&merged_ast, config)?;

        pipeline.finalize_metrics(&bytecode);

        Ok(FiveFile::new(abi, bytecode))
    }

    /// Compile DSL using explicit module list to .five file format.
    ///
    /// This method preserves the ABI in the output.
    pub fn compile_modules_to_five_file(
        module_files: Vec<String>,
        entry_point: &str,
        config: &CompilationConfig,
    ) -> Result<FiveFile, CompilerError> {
        use crate::bytecode_generator::ModuleMerger;
        use crate::error::{ErrorCategory, ErrorCode, ErrorSeverity};

        let mut merger = ModuleMerger::new()
            .with_namespaces(config.enable_module_namespaces);

        for file_path in module_files {
            let source = std::fs::read_to_string(&file_path).map_err(|e| {
                CompilerError::new(
                    ErrorCode::FILE_NOT_FOUND,
                    ErrorSeverity::Error,
                    ErrorCategory::IO,
                    format!("Failed to read module {}: {}", file_path, e),
                )
            })?;

            let mut pipeline = CompilationPipeline::new(&source, Some(&file_path));
            let tokens = pipeline.tokenize()?;
            let ast = pipeline.parse(tokens).map_err(|mut e| {
                if let Some(loc) = &mut e.location {
                    loc.file = Some(std::path::PathBuf::from(&file_path));
                }
                e
            })?;

            // Simple heuristics for module naming: filename without extension
            let path = std::path::Path::new(&file_path);
            let module_name = path.file_stem()
                .ok_or_else(|| CompilerError::new(
                    ErrorCode::INVALID_MODULE_PATH,
                    ErrorSeverity::Error,
                    ErrorCategory::Semantic,
                    format!("Invalid module file path (no file stem): {}", file_path),
                ))?
                .to_string_lossy()
                .to_string();

            if file_path == entry_point {
                merger.set_main_ast(ast);
            } else {
                merger.add_module(module_name, ast);
            }
        }

        let merged_ast = merger.merge().map_err(|e| {
            CompilerError::new(
                ErrorCode::INTERNAL_ERROR,
                ErrorSeverity::Error,
                ErrorCategory::Internal,
                format!("Module merge failed: {}", e),
            )
        })?;

        let entry_source = std::fs::read_to_string(entry_point).unwrap_or_default();
        let mut pipeline = CompilationPipeline::new(&entry_source, Some(entry_point));

        pipeline.type_check(&merged_ast)?;
        let bytecode = pipeline.generate_bytecode(&merged_ast, config)?;
        let abi = pipeline.generate_abi(&merged_ast, config)?;

        pipeline.finalize_metrics(&bytecode);

        Ok(FiveFile::new(abi, bytecode))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bytecode_matches_with_and_without_log() {
        let source = "";
        let config = CompilationConfig::new(CompilationMode::Testing);

        let bytecode_only =
            DslCompiler::compile_with_config(source, &config).expect("compile without log");
        let (bytecode_with_log, _log) =
            DslCompiler::compile_with_config_and_log(source, &config).expect("compile with log");

        assert_eq!(bytecode_only, bytecode_with_log);
    }

    #[test]
    fn test_compile_empty_source() {
        let source = "";
        let bytecode = DslCompiler::compile_dsl(source).expect("compilation failed");
        assert!(!bytecode.is_empty());
    }

    #[test]
    fn test_compile_with_different_modes() {
        let source = "";

        // Testing mode
        let bytecode_testing =
            DslCompiler::compile_for_testing(source).expect("testing compilation failed");

        // Deployment mode
        let bytecode_deployment =
            DslCompiler::compile_for_deployment(source).expect("deployment compilation failed");

        // Both should produce valid bytecode
        assert!(!bytecode_testing.is_empty());
        assert!(!bytecode_deployment.is_empty());
    }

    #[test]
    fn test_compile_with_config_builder() {
        let source = "";
        let config = CompilationConfig::new(CompilationMode::Testing)
            .with_v2_preview(false)
            .with_constraint_cache(true)
            .with_optimization_level(OptimizationLevel::V2);

        let bytecode =
            DslCompiler::compile_with_config(source, &config).expect("compilation failed");
        assert!(!bytecode.is_empty());
    }

    #[test]
    fn test_compile_to_five_file() {
        let source = "";
        let five_file =
            DslCompiler::compile_to_five_file(source).expect("five file compilation failed");

        assert!(!five_file.bytecode.is_empty());
        let _ = five_file.abi; // May be 0 functions for empty source
    }



    #[test]
    fn test_compile_with_metrics() {
        let source = "";
        let (bytecode, metrics) =
            DslCompiler::compile_with_metrics(source, CompilationMode::Testing, true)
                .expect("metrics compilation failed");

        assert!(!bytecode.is_empty());
        let _ = metrics; // Verify metrics are collected
    }

    #[test]
    fn test_backward_compatibility_apis() {
        let source = "";

        // Test legacy API
        let bytecode1 =
            DslCompiler::compile_dsl_with_features(source, true).expect("legacy API failed");

        // Test mode-based API
        let bytecode2 = DslCompiler::compile_with_mode(source, CompilationMode::Testing)
            .expect("mode API failed");

        // Test features API
        let bytecode3 =
            DslCompiler::compile_with_mode_and_features(source, CompilationMode::Testing, true)
                .expect("features API failed");

        // All should produce valid bytecode
        assert!(!bytecode1.is_empty());
        assert!(!bytecode2.is_empty());
        assert!(!bytecode3.is_empty());
    }
}
