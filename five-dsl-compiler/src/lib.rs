// DSL compiler library.

pub mod ast;
pub mod bytecode_generator;
pub mod bytecode_parser;
pub mod compiler;
pub mod config;
pub mod disassembler;
pub mod error;
pub mod five_file;
pub mod import_discovery;
pub mod interface_serializer;
pub mod interface_registry;
pub mod metrics;
pub mod module_resolver;
pub mod parser;
pub mod workspace_resolver;
#[cfg(feature = "security-audit")]
pub mod security_rules;
#[cfg(not(feature = "security-audit"))]
pub mod security_rules {
    use crate::ast::AstNode;
    use five_vm_mito::error::VMError;
    use std::collections::HashMap;
    use std::vec::Vec;

    #[derive(Debug, Clone)]
    pub enum SecurityViolation {}

    #[derive(Debug)]
    pub struct SecurityError;

    #[derive(Debug, Default)]
    pub struct SecurityChecker;

    impl SecurityChecker {
        pub fn new() -> Self {
            Self
        }

        #[allow(clippy::type_complexity)]
        pub fn set_imports(
            &mut self,
            _functions: HashMap<String, (String, Option<Vec<String>>)>,
            _fields: HashMap<String, (String, Option<Vec<String>>)>,
        ) {
        }

        pub fn analyze_security(&mut self, _ast: &AstNode) -> Result<Option<String>, VMError> {
            Ok(None)
        }
    }

    pub fn validate_import_security(_imports: &[AstNode]) -> Result<(), VMError> {
        Ok(())
    }
}
pub mod tokenizer;
pub mod type_checker;

// Re-export public API
pub use ast::{
    AstNode, ErrorVariant, EventFieldAssignment, MatchArm, StructField, StructLiteralField,
    SwitchCase, TypeNode, Visibility,
};
pub use bytecode_generator::{DslBytecodeGenerator, FieldInfo};
pub use bytecode_parser::{BytecodeMetadata, BytecodeParseError, BytecodeParser, CallInfo};
pub use compiler::{CompilationConfig, CompilationMode, DslCompiler};
pub use config::{
    BuildConfig, DeployConfig, DependencyConfig, OptimizationConfig, ProjectConfig, ProjectInfo,
    LinkType, LockFile, PackageManifest, WorkspaceConfig,
};
pub use five_file::FiveFile;
pub use import_discovery::{
    resolve_import_statement, DiscoveredFunction, DiscoveredInterface, ImportDiscovery,
};
pub use interface_registry::InterfaceRegistry;
pub use metrics::{export_metrics, CompilerMetrics, ExportFormat, MetricsCollector};
pub use module_resolver::{
    detect_import_target, is_valid_solana_pubkey, ImportTarget, ModuleDescriptor, ModuleDiscoverer,
    ModuleGraph,
};
pub use parser::DslParser;
pub use security_rules::{
    validate_import_security, SecurityChecker, SecurityError, SecurityViolation,
};
pub use tokenizer::{DslTokenizer, Token};
pub use type_checker::{DslTypeChecker, ModuleScope, ModuleSymbol, ModuleSymbolTable};
pub use bytecode_generator::ModuleMerger;
pub use ast::{BlockKind, InstructionParameter};
