// Bytecode Generator Module
//
// Modular bytecode generation system for the FIVE DSL compiler.
// This module breaks down the previously monolithic bytecode_generator.rs
// into focused, maintainable components.

// Core type definitions and data structures
pub mod types;

// Bytecode opcode utilities and emission functions
pub mod opcodes;

// ABI generation for client-side integration
pub mod abi_generator;

// Fused Opcode optimizations for CU reduction


// Constraint optimization for account validation


// Scope analysis for local variable optimization
pub mod scope_analyzer;

// AST processing and bytecode generation
pub mod ast_generator;

// Constraint enforcement for account validation
pub mod constraint_enforcer;

// Account system for account definitions and field access
pub mod account_system;

// Unified account utilities for consistent account type detection
pub mod account_utils;

// Function dispatch for O(1) function routing
pub mod function_dispatch;

// Import table for Five bytecode imports verification
pub mod import_table;

// Advanced bytecode analysis and disassembly
pub mod bytecode_analyzer;

// Disassembler & diagnostic utilities for bytecode (shared with tests)
pub mod disassembler;

// Compression and size optimization
pub mod compression;

// Performance and runtime optimization
pub mod performance;

// Module merging for multi-file compilation
pub mod module_merger;

// Account indices in bytecode are offset by VM state account.
// Script account is stripped by five-solana (&accounts[1..]).
// VM View: Index 0=VM State, 1=param0, 2=param1
// Formula: account_index = param_index + ACCOUNT_INDEX_OFFSET
// CRITICAL: Must be 1 (see tests/test_account_index_offset.rs)
pub const ACCOUNT_INDEX_OFFSET: u8 = 1;

// CALL opcode emission and patching (public for testing)
pub mod call;
mod header;
mod labels;
mod locals;
mod logging;
mod loops;

// Re-export public types for external use
pub use abi_generator::*;
pub use account_system::*;
pub use account_utils::*;
pub use ast_generator::*;
pub use bytecode_analyzer::*;
pub use call::*;
pub use compression::*;

pub use disassembler::*;
pub use function_dispatch::*;
pub use import_table::*;
pub use module_merger::*;
pub use opcodes::*;
pub use performance::*;
pub use scope_analyzer::*;
pub use types::*;

use crate::ast::AstNode;
use crate::compiler::CompilationMode;
use five_vm_mito::error::VMError;

/// Main bytecode generator that orchestrates the compilation process
/// This is the simplified version that delegates to specialized modules
pub struct DslBytecodeGenerator {
    /// Core bytecode being generated
    bytecode: Vec<u8>,

    /// Current bytecode position for relative jumps
    position: usize,

    /// Function dispatch table for multi-function scripts
    pub(crate) functions: Vec<types::FunctionInfo>,

    /// Symbol table for variable and field resolution
    symbol_table: std::collections::HashMap<String, types::FieldInfo>,

    /// Account type registry for account-related operations
    account_registry: types::AccountRegistry,

    /// Whether to generate function dispatcher
    #[allow(dead_code)]
    generate_dispatcher: bool,

    /// Field counter for symbol table management
    field_counter: u32,

    /// Compilation mode (testing vs deployment)
    compilation_mode: CompilationMode,

    /// Compression optimization flags
    enable_vle_encoding: bool,
    enable_compact_fields: bool,
    enable_instruction_compression: bool,

    /// V2 preview mode for advanced optimizations
    v2_preview: bool,

    /// Optimization level for pattern fusion and advanced optimizations
    optimization_level: crate::compiler::OptimizationLevel,

    /// Interface registry for cross-program invocation support
    interface_registry: Option<crate::interface_registry::InterfaceRegistry>,

    /// Real-time compilation log tracking bytecode generation
    compilation_log: Vec<String>,

    /// Whether to capture diagnostic errors to compilation_log (vs stderr)
    debug_on_error: bool,

    /// Whether to include debug info (function metadata) in bytecode
    pub(crate) include_debug_info: bool,

}

impl DslBytecodeGenerator {
    /// Create a new bytecode generator instance with testing mode
    pub fn new() -> Self {
        Self::with_mode(CompilationMode::Testing)
    }

    /// Create a new bytecode generator instance with specific compilation mode
    pub fn with_mode(mode: CompilationMode) -> Self {
        // Enable diagnostic capture by default in testing mode so devs get helpful
        // bytecode diagnostics without needing to opt-in for local development.
        let debug_on_error = matches!(mode, crate::compiler::CompilationMode::Testing);

        Self {
            bytecode: Vec::new(),
            position: 0,
            functions: Vec::new(),
            symbol_table: std::collections::HashMap::new(),
            account_registry: types::AccountRegistry::new(),
            generate_dispatcher: true,
            field_counter: 0,
            compilation_mode: mode,

            // Enable all compression optimizations by default
            enable_vle_encoding: true,
            enable_compact_fields: true,
            enable_instruction_compression: true,

            // V2 preview mode disabled by default
            v2_preview: false,

            // Default optimization level
            optimization_level: crate::compiler::OptimizationLevel::default(),

            // Interface registry starts empty
            interface_registry: None,

            // Compilation log starts empty
            compilation_log: Vec::new(),

            // When true, runtime generation errors capture a disassembly diagnostic
            // and push it into `compilation_log` instead of printing to stderr.
            debug_on_error,

            // Default: include debug info in testing mode
            include_debug_info: matches!(mode, CompilationMode::Testing),
        }
    }

    /// Create a new bytecode generator instance with specific features (legacy)
    pub fn with_features(enable_constraint_cache: bool) -> Self {
        Self::with_mode_and_features(CompilationMode::Testing, enable_constraint_cache)
    }

    /// Create a new bytecode generator instance with mode and features
    pub fn with_mode_and_features(mode: CompilationMode, _enable_constraint_cache: bool) -> Self {
        
        // Configure features as needed
        // Note: constraint cache is now handled by the performance module
        Self::with_mode(mode)
    }

    /// Create a new bytecode generator instance with v2-preview configuration
    pub fn with_v2_preview_config(config: &crate::compiler::CompilationConfig) -> Self {
        let mut generator = Self::with_mode(config.mode);

        // Configure v2-preview features
        if config.v2_preview {
            // Enable v2-preview optimizations
            generator.enable_vle_encoding = true;
            generator.enable_compact_fields = true;
            generator.enable_instruction_compression = true;
            generator.v2_preview = true; // Enable v2-preview mode
        }

        generator
    }

    /// Create a new bytecode generator instance with optimization level configuration
    pub fn with_optimization_config(config: &crate::compiler::CompilationConfig) -> Self {
        let mut generator = Self::with_mode(config.mode);

        // Store optimization level
        generator.optimization_level = config.optimization_level;

        // Production pipeline enforces all advanced optimizations
        generator.enable_vle_encoding = true;
        generator.enable_compact_fields = true;
        generator.enable_instruction_compression = true;
        generator.v2_preview = true;

        // Respect debug info setting from config
        generator.include_debug_info = config.include_debug_info;

        generator
    }

    /// Set the interface registry for cross-program invocation support
    pub fn set_interface_registry(
        &mut self,
        registry: crate::interface_registry::InterfaceRegistry,
    ) {
        self.interface_registry = Some(registry);
    }

    /// Test helper: set the function table for unit tests without requiring full
    /// dispatcher collection. This is published as `pub` for integration tests.
    #[doc(hidden)]
    pub fn set_functions_for_test(&mut self, funcs: Vec<types::FunctionInfo>) {
        self.functions = funcs;
    }

    /// Emit import verification metadata section for Five bytecode imports
    pub fn emit_import_metadata(&mut self, import_table: &ImportTable) -> Result<(), String> {
        // If the import table is empty, emit nothing (forward compatible)
        if import_table.is_empty() {
            return Ok(());
        }

        // Serialize the import table and emit as raw bytes
        let serialized = import_table.serialize();
        self.emit_bytes(&serialized);

        println!(
            "DEBUG: Emitted import verification metadata ({} bytes)",
            serialized.len()
        );

        Ok(())
    }

    /// Emit function name metadata section for public functions
    pub fn emit_function_name_metadata(&mut self) -> Result<(), String> {
        use five_protocol::{FunctionNameEntry, VLE};

        // Collect public functions with their indices
        let public_functions = self
            .functions
            .iter()
            .enumerate()
            .filter(|(_, f)| f.is_public)
            .collect::<Vec<_>>();

        let mut names = Vec::new();
        for (index, func) in public_functions {
            names.push(FunctionNameEntry {
                name: func.name.clone(),
                function_index: index as u8,
            });
        }

        if names.len() > u8::MAX as usize {
            return Err("Too many public functions for metadata".to_string());
        }

        // Calculate section size: name_count (u8) + for each entry: name_len (u8) + name bytes
        let mut section_size = 1;
        for name_entry in &names {
            section_size += 1;
            section_size += name_entry.name.len();
        }

        let section_size_u16 = section_size as u16;

        // Emit section_size as VLE u16
        let (size_bytes, bytes) = VLE::encode_u16(section_size_u16);
        self.emit_bytes(&bytes[..size_bytes]);

        // Emit name_count as raw u8 (max 255 entries)
        let name_count_u8 = names.len() as u8;
        let (size, bytes) = VLE::encode_u8(name_count_u8);
        self.emit_bytes(&bytes[..size]);

        // Emit each name
        for name_entry in names {
            // name_len as raw u8 (max 255 characters)
            if name_entry.name.len() > u8::MAX as usize {
                return Err("Function name exceeds maximum length of 255 characters".to_string());
            }
            let name_len_u8 = name_entry.name.len() as u8;
            let (size, bytes) = VLE::encode_u8(name_len_u8);
            self.emit_bytes(&bytes[..size]);

            // name bytes
            self.emit_bytes(name_entry.name.as_bytes());
        }

        Ok(())
    }

    /// Main entry point for bytecode generation
    pub fn generate(&mut self, ast: &AstNode) -> Result<Vec<u8>, VMError> {
        // Clear previous state
        self.reset();

        // Execute the existing generation logic inside a closure so we can capture
        // any error and augment it with disassembly diagnostics before returning.
        let result = self.generate_internal(ast);

        // If an error occurred, augment stderr output with a small disassembly snippet
        if let Err(ref e) = result {
            // Use the disassembler to produce context around the current position.
            // We prefer a modest context window (64 bytes) to avoid overly verbose logs.
            let diag = disassembler::inspect_failure(&self.bytecode, self.position, 64);

            // If debug_on_error is enabled, record the diagnostic into the compilation
            // log for programmatic consumption by tooling. Otherwise fall back to stderr.
            if self.debug_on_error {
                // Keep the diagnostic as a single formatted entry (can be parsed or printed later)
                self.compilation_log
                    .push(format!("BYTECODE DIAGNOSTIC: {:?}\n{}", e, diag));
            } else {
                eprintln!("BYTECODE DIAGNOSTIC: {}\n{}", format!("{:?}", e), diag);
            }
        }

        result
    }

    /// Internal generation logic extracted from generate() to reduce complexity
    fn generate_internal(&mut self, ast: &AstNode) -> Result<Vec<u8>, VMError> {
            // Check if we need function dispatch to determine header format
            let mut dispatcher = FunctionDispatcher::new();
            let has_functions = dispatcher.has_callable_functions(ast);

            // Collect function count for OptimizedHeader
            let (public_count, total_count, has_imports) = if has_functions {
                // Pre-collect function information for count
                dispatcher.collect_function_info(ast)?;
                let functions = dispatcher.get_functions();

                // Check if imports exist (for feature flag)
                let has_imports = !dispatcher.get_import_table().is_empty();

                let public_count = functions.iter().filter(|f| f.is_public).count();
                let total_count = functions.len();

                // Validate function counts fit in OptimizedHeader (u8 limit = 255)
                if total_count > 255 {
                    eprintln!(
                        "ERROR: Program has {} functions but OptimizedHeader supports max 255",
                        total_count
                    );
                    eprintln!(
                        "Consider splitting into modules or using a different header format."
                    );
                    return Err(VMError::InvalidScript);
                }

                let public_function_count = public_count as u8;
                let total_function_count = total_count as u8;

                println!(
                    "DEBUG: Collected {} public functions, {} total functions for optimized header",
                    public_function_count, total_function_count
                );

                // CRITICAL: Validate that at least one public function exists
                // This prevents generating bytecode that cannot be called on-chain
                if total_count > 0 && public_count == 0 {
                    eprintln!("ERROR: Script must have at least one public function");
                    eprintln!("All {} functions are internal. Use 'pub fn' to make at least one function callable on-chain.", total_count);
                    eprintln!("Help: Add 'pub' keyword before your main function: 'pub fn test(...) {{ ... }}'");
                    // Temporary debug: return StackError to identify "No Public Functions" case
                    return Err(VMError::StackError);
                }

                // Compiler MUST enforce ordering invariant
                // Public functions at indices 0..(public_count-1)
                // Private functions at indices public_count..(total_count-1)
                // This is validated below in the function emission phase

                // Store functions for metadata emission (copy slice into owned Vec)
                self.functions = functions.to_vec();

                (public_function_count, total_function_count, has_imports)
            } else {
                (0, 0, false)
            };

            // Use OptimizedHeader V2.
            self.emit_optimized_header_v2_with_imports(public_count, total_count, has_imports);

            // Emit function name metadata if there are public functions AND debug info is enabled
            if public_count > 0 && self.include_debug_info {
                self.emit_function_name_metadata()
                    .map_err(|_| VMError::InvalidInstruction)?;
            }

            // Save import table for later emission after main bytecode
            let import_table = dispatcher.get_import_table().clone();

            let mut ast_generator = if has_functions {
                // Use coordinated AST and function dispatcher for multi-function scripts
                // This ensures CALL opcodes are properly coordinated with function indices

                // Process field definitions first to populate symbol table
                self.process_field_definitions(ast)?;

                // Initialize AccountSystem with account definitions from AST
                let mut account_system = AccountSystem::new();
                account_system.process_account_definitions(ast)?;
                // Sync discovered account types into generator-level registry for ABI
                self.account_registry = account_system.get_account_registry().clone();

                let mut scope_analyzer = scope_analyzer::ScopeAnalyzer::new();
                let mut ast_generator =
                    ASTGenerator::with_optimization_level(self.optimization_level);

                // Pass interface registry to AST generator if available
                if let Some(ref interface_registry) = self.interface_registry {
                    ast_generator.set_interface_registry(interface_registry.clone());
                } else {
                    // Fallback: Process interface definitions to populate the interface registry
                    if let AstNode::Program {
                        interface_definitions,
                        ..
                    } = ast
                    {
                        ast_generator.process_interface_definitions(interface_definitions)?;
                    }
                }

                dispatcher.generate_dispatcher(
                    self,
                    ast,
                    &mut account_system,
                    &mut scope_analyzer,
                    &mut ast_generator,
                    &self.symbol_table.clone(),
                )?;

                // No header metadata patching needed.

                ast_generator
            } else {
                // Use direct AST generation for simple scripts
                let mut ast_generator =
                    ASTGenerator::with_optimization_level(self.optimization_level);

                // Pass interface registry to AST generator if available
                if let Some(ref interface_registry) = self.interface_registry {
                    ast_generator.set_interface_registry(interface_registry.clone());
                } else {
                    // Fallback: Process interface definitions to populate the interface registry
                    if let AstNode::Program {
                        interface_definitions,
                        ..
                    } = ast
                    {
                        ast_generator.process_interface_definitions(interface_definitions)?;
                    }
                }

                self.generate_node(ast)?;
                ast_generator
            };

            // Patch all jumps and function calls with their correct offsets
            ast_generator.patch(self)?;

            // Finalize bytecode
            self.finalize_bytecode();

            // CRITICAL: Verify bytecode JUMP targets before deployment
            // This catches register optimization bugs where bytecode structure changes
            // cause JUMP offsets to become invalid (error 8122: CallTargetOutOfBounds)
            let verification_result = disassembler::verify_jump_targets(&self.bytecode);
            if !verification_result.is_valid {
                eprintln!("BYTECODE VERIFICATION FAILED:");
                eprintln!("{}", verification_result.error_summary());
                
                // In debug builds, we panic to make the error highly visible
                #[cfg(debug_assertions)]
                {
                    panic!("Bytecode contains invalid JUMP targets - check disassembler/verification.rs or jumps.rs");
                }
                
                // In release builds, return error
                #[cfg(not(debug_assertions))]
                {
                    return Err(VMError::InvalidInstructionPointer);
                }
            } else {
                eprintln!("BYTECODE VERIFICATION: {} jumps validated, all within {} bytes", 
                    verification_result.jump_count, verification_result.bytecode_length);
            }

            // Emit import verification metadata if imports exist
            self.emit_import_metadata(&import_table)
                .map_err(|_| VMError::InvalidScript)?;

            // Debug: print final bytecode summary to help diagnose missing opcodes in tests
            {
                // Check for presence of REQUIRE opcode in final bytecode
                let contains_require = self
                    .bytecode.contains(&five_protocol::opcodes::REQUIRE);
                eprintln!(
                    "DEBUG: final bytecode len = {}, contains REQUIRE = {}",
                    self.bytecode.len(),
                    contains_require
                );
                // Print a short hexdump (first 256 bytes) to avoid extremely long logs
                let dump_len = std::cmp::min(self.bytecode.len(), 256);
                eprintln!(
                    "DEBUG: final bytecode (first {} bytes) = {:?}",
                    dump_len,
                    &self.bytecode[..dump_len]
                );
            }

            Ok(self.bytecode.clone())
    }

    /// Reset generator state for new compilation
    pub fn reset(&mut self) {
        self.bytecode.clear();
        self.position = 0;
        self.functions.clear();
        self.symbol_table.clear();
        self.account_registry = types::AccountRegistry::new();
        self.field_counter = 0;
        // Keep compilation_mode as it's set during construction
    }

    /// Check if current compilation mode should include test functions
    pub fn should_include_tests(&self) -> bool {
        matches!(self.compilation_mode, CompilationMode::Testing)
    }

    /// Get reference to generated bytecode
    pub fn get_bytecode(&self) -> &Vec<u8> {
        &self.bytecode
    }

    /// Return a textual disassembly of the currently generated bytecode.
    ///
    /// This delegates to the shared disassembler module so callers (CLI, tests,
    /// tooling) get a consistent representation of the bytecode without needing
    /// to re-implement decoding logic.
    pub fn get_disassembly(&self) -> Vec<String> {
        crate::bytecode_generator::disassembler::get_disassembly(&self.bytecode)
    }

    /// Return a structured disassembly (a list of decoded instructions).
    ///
    /// This leverages the shared disassembler/inspector: for each textual
    /// disassembly line we extract the instruction offset (hex) and ask the
    /// inspector to decode that instruction into a structured `Instruction`.
    /// This is intentionally conservative: when decoding fails for a given
    /// offset we push `Instruction::Unknown` so callers always get a
    /// one-to-one list with the textual lines.
    pub fn get_structured_disassembly(
        &self,
    ) -> Vec<crate::bytecode_generator::disassembler::Instruction> {
        use crate::bytecode_generator::disassembler::BytecodeInspector;

        let mut out: Vec<crate::bytecode_generator::disassembler::Instruction> = Vec::new();

        // Get textual disassembly lines (these include offsets like "00A0: ...")
        let lines = crate::bytecode_generator::disassembler::disassemble(&self.bytecode);

        // Create an inspector to decode instructions at offsets
        let inspector = BytecodeInspector::new(&self.bytecode);

        for line in lines {
            // Each disassembly line starts with the offset in hex, like "00A0:".
            // Extract the leading token and parse it as hex.
            let first_token = line.split_whitespace().next().unwrap_or("");
            let hex_offset = first_token.trim_end_matches(':');

            if let Ok(offset) = usize::from_str_radix(hex_offset, 16) {
                // Ask the inspector to decode instruction at this offset.
                if let Some(instr) = inspector.decode_instruction_at(offset) {
                    out.push(instr);
                    continue;
                }
            }

            // Fallback to Unknown when we can't decode
            out.push(crate::bytecode_generator::disassembler::Instruction::Unknown);
        }

        out
    }

    /// Get reference to function information
    pub fn get_functions(&self) -> &Vec<types::FunctionInfo> {
        &self.functions
    }

    /// Get reference to symbol table
    pub fn get_symbol_table(&self) -> &std::collections::HashMap<String, types::FieldInfo> {
        &self.symbol_table
    }

    /// Get reference to account registry
    pub fn get_account_registry(&self) -> &types::AccountRegistry {
        &self.account_registry
    }

    /// Process field definitions to populate the symbol table
    fn process_field_definitions(&mut self, ast: &AstNode) -> Result<(), VMError> {
        if let AstNode::Program {
            field_definitions, ..
        } = ast
        {
            for field_def in field_definitions {
                if let AstNode::FieldDefinition {
                    name,
                    field_type,
                    is_mutable,
                    ..
                } = field_def
                {
                    let field_info = types::FieldInfo {
                        offset: self.field_counter,
                        field_type: self.type_node_to_string(field_type),
                        is_mutable: *is_mutable, // Respect the mutability from the AST
                        is_optional: false,
                        is_parameter: false,
                    };
                    self.symbol_table.insert(name.clone(), field_info);
                    self.field_counter += 1;
                    println!(
                        "DEBUG: Added field '{}' to symbol table at offset {}",
                        name,
                        self.field_counter - 1
                    );
                }
            }
        }
        Ok(())
    }

    /// Convert TypeNode to string representation
    fn type_node_to_string(&self, type_node: &crate::ast::TypeNode) -> String {
        use crate::ast::TypeNode;

        match type_node {
            TypeNode::Primitive(name) => name.clone(),
            TypeNode::Generic { base, .. } => base.clone(),
            TypeNode::Named(name) => name.clone(),
            TypeNode::Account => "Account".to_string(),
            _ => "unknown".to_string(),
        }
    }

    /// Finalize bytecode generation
    fn finalize_bytecode(&mut self) {
        // Add any final opcodes or padding if needed
        // For now, just ensure we end with a HALT
        if !self.bytecode.is_empty()
            && self.bytecode[self.bytecode.len() - 1] != five_protocol::opcodes::HALT
        {
            self.emit_opcode(five_protocol::opcodes::HALT);
            self.log_opcode("HALT", "End program execution");
        }
    }

    /// Generate bytecode for an AST node using the AST generator
    fn generate_node(&mut self, node: &AstNode) -> Result<(), VMError> {
        println!(
            "DEBUG: generate_node called with node type: {:?}",
            std::mem::discriminant(node)
        );
        let mut ast_generator = ASTGenerator::with_optimization_level(self.optimization_level);

        // Initialize and configure AccountSystem with account definitions from AST
        let mut account_system = AccountSystem::new();
        account_system.process_account_definitions(node)?;

        // Set the account system in the AST generator for proper field offset resolution
        ast_generator.set_account_system(account_system);

        // Copy existing symbol table to AST generator to maintain state
        ast_generator.set_symbol_table(self.symbol_table.clone());

        println!("DEBUG: About to call ast_generator.generate_ast_node - this should trigger Program node processing");
        ast_generator.generate_ast_node(self, node)?;

        // Copy back the symbol table and field counter for other modules
        // This ensures state consistency across the generation process
        self.symbol_table = ast_generator.clone_symbol_table();
        Ok(())
    }

    // Duplicate functions removed - now using function_dispatch module
}

impl DslBytecodeGenerator {
    /// Enable or disable bytecode diagnostic capture on generation errors.
    ///
    /// When enabled, generation-time diagnostics will be captured into
    /// `compilation_log` instead of being printed to stderr.
    pub fn set_debug_on_error(&mut self, enabled: bool) {
        self.debug_on_error = enabled;
    }

    /// Append an arbitrary entry to the compilation log (thread-local to this generator).
    pub fn push_compilation_log(&mut self, entry: String) {
        self.compilation_log.push(entry);
    }

}

impl Default for DslBytecodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}
