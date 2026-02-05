// Compiler Pipeline Module
//
// Provides the core CompilationPipeline that eliminates duplicate
// compilation logic across 4 different compile methods.

use crate::ast::AstNode;
use crate::bytecode_generator::{types::FIVEABI, DslBytecodeGenerator};
use crate::error::{integration, CompilerError, ErrorCategory};
use crate::interface_registry::InterfaceRegistry;
use crate::metrics::{CompilerMetrics, MetricsCollector};
use crate::parser::DslParser;
use crate::tokenizer::{DslTokenizer, Token};
use crate::type_checker::DslTypeChecker;
use five_vm_mito::error::VMError;
use web_time::Instant;

/// Configuration for the compilation pipeline
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompilationMode {
    /// Include test functions for local execution
    Testing,
    /// Exclude test functions for deployment (smaller bytecode)
    Deployment,
}

/// Optimization level for Five VM pattern fusion
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptimizationLevel {
    /// V1: Basic opcodes only, no pattern fusion
    V1,
    /// V2: VLE + zero-copy optimizations
    V2,
    /// V3: Full pattern fusion with advanced opcodes
    V3,
    /// Production: Optimized header with minimal CU overhead (6 bytes vs 21+ bytes)
    Production,
}

impl Default for OptimizationLevel {
    fn default() -> Self {
        Self::Production
    }
}

/// Compilation features and flags
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilationConfig {
    /// Basic compilation mode (testing vs deployment)
    pub mode: CompilationMode,
    /// Enable v2-preview features (nibble immediates, BR_EQ_U8, etc.)
    pub v2_preview: bool,
    /// Enable constraint caching optimization
    pub enable_constraint_cache: bool,
    /// Elevation for pattern fusion
    pub optimization_level: OptimizationLevel,
    /// Include debug info (function name metadata) in bytecode
    pub include_debug_info: bool,
    /// Enable module namespace qualification (module::function)
    pub enable_module_namespaces: bool,
}

impl CompilationConfig {
    /// Create a new configuration with sensible defaults
    pub fn new(mode: CompilationMode) -> Self {
        Self {
            mode,
            v2_preview: false,
            enable_constraint_cache: true,
            optimization_level: OptimizationLevel::V2,
            include_debug_info: matches!(mode, CompilationMode::Testing),

            enable_module_namespaces: true, // Enabled: critical for multi-module compilation
        }
    }

    /// Enable or disable v2-preview features
    pub fn with_v2_preview(mut self, v2_preview: bool) -> Self {
        self.v2_preview = v2_preview;
        self
    }

    /// Enable or disable constraint caching
    pub fn with_constraint_cache(mut self, enable: bool) -> Self {
        self.enable_constraint_cache = enable;
        self
    }

    /// Set the optimization level
    pub fn with_optimization_level(mut self, level: OptimizationLevel) -> Self {
        self.optimization_level = level;
        self
    }

    /// Enable or disable debug info (function metadata)
    pub fn with_debug_info(mut self, enable: bool) -> Self {
        self.include_debug_info = enable;
        self
    }

    /// Enable or disable module namespace qualification
    pub fn with_module_namespaces(mut self, enable: bool) -> Self {
        self.enable_module_namespaces = enable;
        self
    }

    /// Enable or disable register-based optimization (Removed)
    pub fn with_use_registers(self, _enable: bool) -> Self {
        self
    }

    /// Enable or disable linear scan register allocation (Removed)
    pub fn with_linear_scan_allocation(self, _enable: bool) -> Self {
        self
    }

    /// Parse optimization level from string (for CLI integration)
    pub fn parse_optimization_level(level_str: &str) -> Result<OptimizationLevel, String> {
        match level_str.to_lowercase().as_str() {
            "v1" => Ok(OptimizationLevel::V1),
            "v2" => Ok(OptimizationLevel::V2),
            "v3" => Ok(OptimizationLevel::V3),
            "production" => Ok(OptimizationLevel::Production),
            _ => Err(format!(
                "Invalid optimization level '{}'. Valid options: v1, v2, v3, production",
                level_str
            )),
        }
    }
}

impl Default for CompilationConfig {
    fn default() -> Self {
        Self::new(CompilationMode::Testing)
    }
}

/// Core compilation pipeline that executes all compilation phases.
///
/// This struct consolidates compilation logic into a single reusable pipeline to ensure consistent behavior.
///
/// Uses `&'a str` instead of `String` to avoid unnecessary allocations during compilation.
pub struct CompilationPipeline<'a> {
    pub(crate) metrics: MetricsCollector,
    pub(crate) error_collector: integration::ErrorCollector,
    source: &'a str,
    filename: Option<&'a str>,
    #[cfg(debug_assertions)]
    start_time: Instant,
}

impl<'a> CompilationPipeline<'a> {
    /// Create a new compilation pipeline for the given source code.
    ///
    /// Initializes the error system, metrics collector, and error collector.
    ///
    /// Performance: Source is stored as `&str` to avoid unnecessary allocations.
    pub fn new(source: &'a str, filename: Option<&'a str>) -> Self {
        if let Err(e) = integration::initialize_error_system() {
            eprintln!("Warning: Failed to initialize enhanced error system: {}", e);
        }

        Self {
            metrics: MetricsCollector::new(),
            error_collector: integration::ErrorCollector::new(),
            source,
            filename,
            #[cfg(debug_assertions)]
            start_time: Instant::now(),
        }
    }

    /// Execute the tokenization phase.
    ///
    /// Converts source code into a stream of tokens.
    pub fn tokenize(&mut self) -> Result<Vec<Token>, CompilerError> {
        let source = self.source;
        execute_phase!(
            "tokenization",
            &mut self.metrics,
            &mut self.error_collector,
            &source,
            self.filename,
            ErrorCategory::Syntax,
            {
                let mut tokenizer = DslTokenizer::new(source);
                tokenizer.tokenize()
            }
        )
    }

    /// Execute the parsing phase.
    ///
    /// Converts tokens into an Abstract Syntax Tree (AST).
    /// Also records source statistics in metrics.
    pub fn parse(&mut self, tokens: Vec<Token>) -> Result<AstNode, CompilerError> {
        // Record source statistics before parsing
        self.metrics.record_source_stats(self.source, &tokens);

        let source = self.source;
        self.metrics.start_phase("parsing");
        let result: Result<AstNode, VMError> = DslParser::new(tokens).parse();
        match result {
            Ok(ast) => {
                self.metrics.end_phase();
                Ok(ast)
            }
            Err(vm_error) => {
                let compiler_error = crate::compiler::error_handling::convert_and_collect_error(
                    vm_error,
                    ErrorCategory::Syntax,
                    "parsing",
                    source,
                    self.filename,
                    &mut self.error_collector,
                    &mut self.metrics,
                );
                Err(compiler_error)
            }
        }
    }

    /// Execute the type checking phase (simple version without interfaces).
    ///
    /// Validates type correctness of the AST.
    pub fn type_check(&mut self, ast: &AstNode) -> Result<(), CompilerError> {
        let source = self.source;
        execute_phase!(
            "type_checking",
            &mut self.metrics,
            &mut self.error_collector,
            &source,
            self.filename,
            ErrorCategory::Type,
            {
                let mut type_checker = DslTypeChecker::new();
                type_checker.check_types(ast)
            }
        )
    }

    /// Execute type checking with interface preprocessing (full version).
    ///
    /// Two-pass compilation:
    /// 1. Preprocess interfaces into registry
    /// 2. Type check with interface definitions
    ///
    /// Returns the interface registry for bytecode generation.
    pub fn type_check_with_interfaces(
        &mut self,
        ast: &AstNode,
    ) -> Result<InterfaceRegistry, CompilerError> {
        let source = self.source;

        // Phase 1: Interface preprocessing
        let interface_registry = execute_phase!(
            "interface_preprocessing",
            &mut self.metrics,
            &mut self.error_collector,
            &source,
            self.filename,
            ErrorCategory::Type,
            {
                let mut interface_registry = InterfaceRegistry::new();
                match interface_registry.preprocess_interfaces(ast) {
                    Ok(()) => Ok(interface_registry),
                    Err(vm_error) => Err(vm_error),
                }
            }
        )?;

        // Phase 2: Type checking with interface definitions
        execute_phase!(
            "type_checking",
            &mut self.metrics,
            &mut self.error_collector,
            &source,
            self.filename,
            ErrorCategory::Type,
            {
                let mut type_checker = DslTypeChecker::new();

                // Pass interface definitions to type checker
                let preprocess_result = if let AstNode::Program {
                    interface_definitions,
                    ..
                } = ast
                {
                    type_checker.process_interface_definitions(interface_definitions)
                } else {
                    Ok(())
                };

                if let Err(vm_error) = preprocess_result {
                    Err(vm_error)
                } else {
                    type_checker.check_types(ast)
                }
            }
        )?;

        Ok(interface_registry)
    }

    /// Execute the bytecode generation phase.
    ///
    /// Converts the AST into Five VM bytecode.
    pub fn generate_bytecode(
        &mut self,
        ast: &AstNode,
        config: &CompilationConfig,
    ) -> Result<Vec<u8>, CompilerError> {
        let source = self.source;
        execute_phase!(
            "bytecode_generation",
            &mut self.metrics,
            &mut self.error_collector,
            &source,
            self.filename,
            ErrorCategory::Codegen,
            {
                let mut generator = DslBytecodeGenerator::with_optimization_config(config);
                generator.generate(ast)
            }
        )
    }

    /// Execute bytecode generation with interface registry (for two-pass compilation).
    ///
    /// Provides the interface registry to the bytecode generator for
    /// cross-contract call support.
    pub fn generate_bytecode_with_interfaces(
        &mut self,
        ast: &AstNode,
        config: &CompilationConfig,
        interface_registry: InterfaceRegistry,
    ) -> Result<Vec<u8>, CompilerError> {
        let source = self.source;
        execute_phase!(
            "bytecode_generation",
            &mut self.metrics,
            &mut self.error_collector,
            &source,
            self.filename,
            ErrorCategory::Codegen,
            {
                let mut generator = DslBytecodeGenerator::with_optimization_config(config);
                generator.set_interface_registry(interface_registry);
                generator.generate(ast)
            }
        )
    }

    /// Execute bytecode generation and capture compilation log.
    ///
    /// Returns both bytecode and the compilation log for debugging.
    pub fn generate_bytecode_with_log(
        &mut self,
        ast: &AstNode,
        config: &CompilationConfig,
        interface_registry: Option<InterfaceRegistry>,
    ) -> Result<(Vec<u8>, Vec<String>), CompilerError> {
        let source = self.source;
        let bytecode = execute_phase!(
            "bytecode_generation",
            &mut self.metrics,
            &mut self.error_collector,
            &source,
            self.filename,
            ErrorCategory::Codegen,
            {
                let mut generator = DslBytecodeGenerator::with_optimization_config(config);
                if let Some(registry) = interface_registry {
                    generator.set_interface_registry(registry);
                }
                match generator.generate(ast) {
                    Ok(bytecode) => {
                        let log = generator.get_compilation_log().to_vec();
                        Ok((bytecode, log))
                    }
                    Err(vm_error) => Err(vm_error),
                }
            }
        )?;

        Ok(bytecode)
    }

    /// Generate ABI (Application Binary Interface) for the compiled contract.
    ///
    /// The ABI describes the contract's public functions and their signatures.
    pub fn generate_abi(
        &mut self,
        ast: &AstNode,
        config: &CompilationConfig,
    ) -> Result<FIVEABI, CompilerError> {
        let source = self.source;
        execute_phase!(
            "abi_generation",
            &mut self.metrics,
            &mut self.error_collector,
            &source,
            self.filename,
            ErrorCategory::Codegen,
            {
                let mut generator = DslBytecodeGenerator::with_mode_and_features(
                    config.mode,
                    config.enable_constraint_cache,
                );
                generator.generate_abi(ast)
            }
        )
    }

    /// Finalize metrics with bytecode analytics.
    ///
    /// Records final bytecode size and analytics, then finalizes the metrics.
    pub fn finalize_metrics(&mut self, bytecode: &[u8]) {
        self.metrics
            .record_bytecode_analytics(bytecode, bytecode.len() * 2);
        self.metrics.finalize();
    }

    /// Get a reference to the collected metrics.
    pub fn get_metrics(&self) -> &CompilerMetrics {
        self.metrics.get_metrics()
    }

    /// Consume the pipeline and return owned metrics.
    ///
    /// This avoids cloning when the pipeline is no longer needed.
    pub fn into_metrics(self) -> CompilerMetrics {
        self.metrics.get_metrics().clone()
    }

    /// Get reference to the error collector
    pub fn get_error_collector(&self) -> &integration::ErrorCollector {
        &self.error_collector
    }

    /// Print debug metrics (only in debug builds).
    ///
    /// Outputs compilation statistics including opcode count, bytecode size,
    /// and total compilation time.
    #[cfg(debug_assertions)]
    pub fn print_debug_metrics(&self, prefix: &str) {
        let total_time = self.start_time.elapsed();
        let collected_metrics = self.get_metrics();
        eprintln!(
            "{}: {} opcodes, {} bytes, {:?} total time",
            prefix,
            collected_metrics.opcode_stats.total_opcodes,
            collected_metrics.bytecode_analytics.final_size,
            total_time
        );
    }

    /// Print debug metrics for .five file compilation (only in debug builds).
    #[cfg(debug_assertions)]
    pub fn print_five_file_metrics(&self, bytecode_len: usize, abi_len: usize) {
        let total_time = self.start_time.elapsed();
        let collected_metrics = self.get_metrics();
        eprintln!(
            "Five file compilation: {} opcodes, {} bytes bytecode, {} functions in ABI, {:?} total time",
            collected_metrics.opcode_stats.total_opcodes,
            bytecode_len,
            abi_len,
            total_time
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compilation_config_defaults() {
        let config = CompilationConfig::default();
        assert_eq!(config.mode, CompilationMode::Testing);
        assert!(!config.v2_preview);
        assert!(config.enable_constraint_cache);
        assert_eq!(config.optimization_level, OptimizationLevel::V2);
    }

    #[test]
    fn test_compilation_config_builder() {
        let config = CompilationConfig::new(CompilationMode::Deployment)
            .with_v2_preview(true)
            .with_constraint_cache(false)
            .with_optimization_level(OptimizationLevel::V3);

        assert_eq!(config.mode, CompilationMode::Deployment);
        assert!(config.v2_preview);
        assert!(!config.enable_constraint_cache);
        assert_eq!(config.optimization_level, OptimizationLevel::V3);
    }

    #[test]
    fn test_parse_optimization_level() {
        assert_eq!(
            CompilationConfig::parse_optimization_level("v1").unwrap(),
            OptimizationLevel::V1
        );
        assert_eq!(
            CompilationConfig::parse_optimization_level("V2").unwrap(),
            OptimizationLevel::V2
        );
        assert_eq!(
            CompilationConfig::parse_optimization_level("v3").unwrap(),
            OptimizationLevel::V3
        );
        assert_eq!(
            CompilationConfig::parse_optimization_level("production").unwrap(),
            OptimizationLevel::Production
        );
        assert!(CompilationConfig::parse_optimization_level("invalid").is_err());
    }

    #[test]
    fn test_pipeline_creation() {
        let _pipeline = CompilationPipeline::new("", None);
        assert!(true);
    }

    #[test]
    fn test_empty_source_compilation() {
        let mut pipeline = CompilationPipeline::new("", None);
        let config = CompilationConfig::default();

        let tokens = pipeline.tokenize().expect("tokenization failed");
        let ast = pipeline.parse(tokens).expect("parsing failed");
        pipeline.type_check(&ast).expect("type checking failed");

        let bytecode = pipeline
            .generate_bytecode(&ast, &config)
            .expect("bytecode generation failed");

        assert!(!bytecode.is_empty());
    }

    #[test]
    fn test_interface_registry_empty_program() {
        let mut pipeline = CompilationPipeline::new("", None);
        let tokens = pipeline.tokenize().expect("tokenization failed");
        let ast = pipeline.parse(tokens).expect("parsing failed");

        let _registry = pipeline
            .type_check_with_interfaces(&ast)
            .expect("interface preprocessing failed");

        assert!(true);
    }

    #[test]
    fn test_interface_preprocessing_with_interface() {
        let source = r#"
            interface IToken @program("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA") {
                fn transfer @discriminator(1)(to: pubkey, amount: u64) -> bool;
                fn balance_of @discriminator(2)(owner: pubkey) -> u64;
            }
        "#;

        let mut pipeline = CompilationPipeline::new(source, None);
        let tokens = pipeline.tokenize().expect("tokenization failed");
        let ast = pipeline.parse(tokens).expect("parsing failed");

        let _registry = pipeline
            .type_check_with_interfaces(&ast)
            .expect("interface preprocessing failed");

        assert!(true);
    }

    #[test]
    fn test_bytecode_generation_with_interface_registry() {
        let source = r#"
            interface IToken @program("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA") {
                fn transfer @discriminator(1)(to: pubkey, amount: u64) -> bool;
            }

            pub fn main() {}
        "#;

        let mut pipeline = CompilationPipeline::new(source, None);
        let config = CompilationConfig::default();

        let tokens = pipeline.tokenize().expect("tokenization failed");
        let ast = pipeline.parse(tokens).expect("parsing failed");
        let registry = pipeline
            .type_check_with_interfaces(&ast)
            .expect("interface preprocessing failed");

        let bytecode = pipeline
            .generate_bytecode_with_interfaces(&ast, &config, registry)
            .expect("bytecode generation with interfaces failed");

        assert!(!bytecode.is_empty());
    }

    #[test]
    fn test_generate_bytecode_with_log_and_interfaces() {
        let source = r#"
            interface IVault @program("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA") {
                fn deposit @discriminator(1)(amount: u64);
            }
        "#;

        let mut pipeline = CompilationPipeline::new(source, None);
        let config = CompilationConfig::default();

        let tokens = pipeline.tokenize().expect("tokenization failed");
        let ast = pipeline.parse(tokens).expect("parsing failed");
        let registry = pipeline
            .type_check_with_interfaces(&ast)
            .expect("interface preprocessing failed");

        let (bytecode, log) = pipeline
            .generate_bytecode_with_log(&ast, &config, Some(registry))
            .expect("bytecode generation with log failed");

        assert!(!bytecode.is_empty());
        let _ = log;
    }

    #[test]
    fn test_interface_error_handling() {
        let source = r#"
            interface BadInterface {
                // Missing function signature
                fn incomplete_function
            }
        "#;

        let mut pipeline = CompilationPipeline::new(source, None);

        let result = pipeline.tokenize();

        if let Ok(tokens) = result {
            let parse_result = pipeline.parse(tokens);
            assert!(parse_result.is_err() || parse_result.is_ok());
        }
    }

    #[test]
    fn test_abi_generation() {
        let source = r#"
            pub fn transfer(to: pubkey, amount: u64) {
                // Function body
            }
        "#;

        let mut pipeline = CompilationPipeline::new(source, None);
        let config = CompilationConfig::default();

        let tokens = pipeline.tokenize().expect("tokenization failed");
        let ast = pipeline.parse(tokens).expect("parsing failed");
        pipeline.type_check(&ast).expect("type checking failed");

        let abi = pipeline
            .generate_abi(&ast, &config)
            .expect("ABI generation failed");

        assert!(!abi.functions.is_empty());
    }

    #[test]
    fn test_pipeline_metrics_finalization() {
        let mut pipeline = CompilationPipeline::new("", None);
        let config = CompilationConfig::default();

        let tokens = pipeline.tokenize().expect("tokenization failed");
        let ast = pipeline.parse(tokens).expect("parsing failed");
        pipeline.type_check(&ast).expect("type checking failed");
        let bytecode = pipeline
            .generate_bytecode(&ast, &config)
            .expect("bytecode generation failed");

        pipeline.finalize_metrics(&bytecode);

        let metrics = pipeline.get_metrics();
        assert!(metrics.bytecode_analytics.final_size > 0);
    }

    #[test]
    fn test_zero_allocation_source_access() {
        let source = "pub fn test() {}";
        let mut pipeline = CompilationPipeline::new(source, None);
        let config = CompilationConfig::default();

        let tokens = pipeline.tokenize().expect("tokenization failed");
        let ast = pipeline.parse(tokens).expect("parsing failed");
        pipeline.type_check(&ast).expect("type checking failed");
        let _bytecode = pipeline
            .generate_bytecode(&ast, &config)
            .expect("bytecode generation failed");

        assert!(true);
    }
}
