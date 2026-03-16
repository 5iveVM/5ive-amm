pub mod types;

pub mod opcodes;

pub mod abi_generator;

pub mod scope_analyzer;

pub mod ast_generator;

pub mod constraint_enforcer;

pub mod account_system;

pub mod account_utils;

pub mod function_dispatch;

pub mod import_table;

pub mod bytecode_analyzer;

pub mod disassembler;

pub mod performance;

pub mod module_merger;

pub mod constant_pool;

// Account indices are offset by the VM state account (see tests/test_account_index_offset.rs).
pub const ACCOUNT_INDEX_OFFSET: u8 = 1;

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
pub use bytecode_analyzer::AdvancedBytecodeAnalyzer;
pub use call::*;
// pub use compression::*;

pub use constant_pool::*;
pub use function_dispatch::*;
pub use import_table::*;
pub use module_merger::*;
pub use opcodes::*;
pub use performance::*;
pub use scope_analyzer::*;
pub use types::*;

use crate::ast::{AstNode, BlockKind, InstructionParameter, Visibility};
use crate::compiler::CompilationMode;
use std::collections::{HashMap, HashSet};
use five_vm_mito::error::VMError;

/// Main bytecode generator.
pub struct DslBytecodeGenerator {
    bytecode: Vec<u8>,

    header_bytes: Vec<u8>,

    metadata_bytes: Vec<u8>,

    import_metadata_bytes: Vec<u8>,

    header_features: u32,

    position: usize,

    pub(crate) functions: Vec<types::FunctionInfo>,

    symbol_table: std::collections::HashMap<String, types::FieldInfo>,

    account_registry: types::AccountRegistry,

    #[allow(dead_code)]
    generate_dispatcher: bool,

    field_counter: u32,

    compilation_mode: CompilationMode,

    enable_compact_fields: bool,
    enable_instruction_compression: bool,

    v2_preview: bool,

    optimization_level: crate::compiler::OptimizationLevel,
    require_batch_enabled: bool,

    interface_registry: Option<crate::interface_registry::InterfaceRegistry>,

    compilation_log: Vec<String>,

    debug_on_error: bool,

    pub(crate) include_debug_info: bool,

    constant_pool: constant_pool::ConstantPoolBuilder,
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
            header_bytes: Vec::new(),
            metadata_bytes: Vec::new(),
            import_metadata_bytes: Vec::new(),
            header_features: 0,
            position: 0,
            functions: Vec::new(),
            symbol_table: std::collections::HashMap::new(),
            account_registry: types::AccountRegistry::new(),
            generate_dispatcher: true,
            field_counter: 0,
            compilation_mode: mode,

            // Enable all compression optimizations by default
            enable_compact_fields: true,
            enable_instruction_compression: true,

            // V2 preview mode disabled by default
            v2_preview: false,

            // Default optimization level
            optimization_level: crate::compiler::OptimizationLevel::default(),
            require_batch_enabled: true,

            // Interface registry starts empty
            interface_registry: None,

            // Compilation log starts empty
            compilation_log: Vec::new(),

            // When true, runtime generation errors capture a disassembly diagnostic
            // and push it into `compilation_log` instead of printing to stderr.
            debug_on_error,

            // Default: include debug info in testing mode
            include_debug_info: matches!(mode, CompilationMode::Testing),

            constant_pool: constant_pool::ConstantPoolBuilder::new(),
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
            generator.enable_compact_fields = true;
            generator.enable_instruction_compression = true;
            generator.v2_preview = true; // Enable v2-preview mode
        }
        generator.require_batch_enabled = !config.disable_require_batch;
        generator.include_debug_info = config.include_debug_info;

        generator
    }

    /// Create a new bytecode generator instance with optimization level configuration
    pub fn with_optimization_config(config: &crate::compiler::CompilationConfig) -> Self {
        let mut generator = Self::with_mode(config.mode);

        // Store optimization level
        generator.optimization_level = config.optimization_level;

        // Production pipeline enforces all advanced optimizations
        generator.enable_compact_fields = true;
        generator.enable_instruction_compression = true;
        generator.v2_preview = true;

        // Respect debug info setting from config
        generator.include_debug_info = config.include_debug_info;
        generator.require_batch_enabled = !config.disable_require_batch;

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
        let serialized = import_table.serialize()?;
        self.import_metadata_bytes = serialized;

        println!(
            "DEBUG: Emitted import verification metadata ({} bytes)",
            self.import_metadata_bytes.len()
        );

        Ok(())
    }

    /// Emit function name metadata section for public functions
    pub fn emit_function_name_metadata(&mut self) -> Result<(), String> {
        use five_protocol::FunctionNameEntry;

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

        let mut out = Vec::new();

        // Emit section_size as fixed u16
        out.extend_from_slice(&section_size_u16.to_le_bytes());

        // Emit name_count as raw u8 (max 255 entries)
        let name_count_u8 = names.len() as u8;
        out.push(name_count_u8);

        // Emit each name
        for name_entry in names {
            if name_entry.name.len() > u8::MAX as usize {
                return Err("Function name exceeds maximum length of 255 characters".to_string());
            }
            let name_len_u8 = name_entry.name.len() as u8;
            out.push(name_len_u8);
            out.extend_from_slice(name_entry.name.as_bytes());
        }

        self.metadata_bytes = out;

        Ok(())
    }

    /// Emit compact public entry table metadata.
    /// Format: [section_size:u16][public_entry_count:u8][entry_offset:u16 * count]
    /// Offsets are relative to the start of the code section.
    pub fn emit_public_entry_table_metadata(
        &mut self,
        entries: &[(u8, usize)],
    ) -> Result<(), String> {
        if entries.is_empty() {
            return Ok(());
        }
        if entries.len() > u8::MAX as usize {
            return Err("Too many public entries for metadata".to_string());
        }

        let mut sorted = entries.to_vec();
        sorted.sort_by_key(|(idx, _)| *idx);

        let count = sorted.len() as u8;
        let section_size = 1usize + (count as usize) * 2;
        let section_size_u16 = u16::try_from(section_size)
            .map_err(|_| "Public entry section too large".to_string())?;

        self.metadata_bytes
            .extend_from_slice(&section_size_u16.to_le_bytes());
        self.metadata_bytes.push(count);
        for (_, offset) in sorted {
            let rel = u16::try_from(offset)
                .map_err(|_| "Public entry offset exceeds u16 range".to_string())?;
            self.metadata_bytes.extend_from_slice(&rel.to_le_bytes());
        }

        self.header_features |= five_protocol::FEATURE_PUBLIC_ENTRY_TABLE;
        self.header_features |= five_protocol::FEATURE_FAST_DISPATCH_TABLE;
        self.header_features |= five_protocol::FEATURE_FUSED_BRANCH_OPS;
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
        let optimized_ast = self.inline_private_helpers(ast);
        let ast = &optimized_ast;

        // Check if we need function dispatch to determine header format
        let mut dispatcher = FunctionDispatcher::new();
        let has_functions = dispatcher.has_callable_functions(ast);

        // Collect function counts for the canonical v1 bytecode header.
        let (public_count, total_count, has_imports) = if has_functions {
            // Pre-collect function information for count
            dispatcher.collect_function_info(ast)?;
            let functions = dispatcher.get_functions();

            // Check if imports exist (for feature flag)
            let has_imports = !dispatcher.get_import_table().is_empty();

            let public_count = functions.iter().filter(|f| f.is_public).count();
            let total_count = functions.len();

            // Validate function counts fit in the v1 bytecode header (u8 limit = 255)
            if total_count > 255 {
                eprintln!(
                    "ERROR: Program has {} functions but ScriptBytecodeHeaderV1 supports max 255",
                    total_count
                );
                eprintln!("Consider splitting into modules or using a different header format.");
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

        // Use the canonical ScriptBytecodeHeaderV1.
        self.emit_script_bytecode_header_v1_with_imports(public_count, total_count, has_imports);

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
            let mut ast_generator = ASTGenerator::with_optimization_level(self.optimization_level);
            ast_generator.set_require_batch_enabled(self.require_batch_enabled);

            // Seed interface registry from type-checking pipeline when available.
            if let Some(ref interface_registry) = self.interface_registry {
                ast_generator.set_interface_registry(interface_registry.clone());
            }
            // Always merge interfaces present in the merged AST to ensure imported
            // stdlib/module interfaces are available during bytecode lowering.
            if let AstNode::Program {
                interface_definitions,
                ..
            } = ast
            {
                ast_generator.process_interface_definitions(interface_definitions)?;
            }

            dispatcher.generate_dispatcher(
                self,
                ast,
                &mut account_system,
                &mut scope_analyzer,
                &mut ast_generator,
                &self.symbol_table.clone(),
            )?;

            if public_count > 0 {
                self.emit_public_entry_table_metadata(dispatcher.get_public_entry_points())
                    .map_err(|_| VMError::InvalidInstruction)?;
            }

            // No header metadata patching needed.

            ast_generator
        } else {
            // Use direct AST generation for simple scripts
            let mut ast_generator = ASTGenerator::with_optimization_level(self.optimization_level);
            ast_generator.set_require_batch_enabled(self.require_batch_enabled);

            // Seed interface registry from type-checking pipeline when available.
            if let Some(ref interface_registry) = self.interface_registry {
                ast_generator.set_interface_registry(interface_registry.clone());
            }
            // Always merge interfaces present in the merged AST to ensure imported
            // stdlib/module interfaces are available during bytecode lowering.
            if let AstNode::Program {
                interface_definitions,
                ..
            } = ast
            {
                ast_generator.process_interface_definitions(interface_definitions)?;
            }

            self.generate_node(ast)?;
            ast_generator
        };

        // Finalize instruction stream (code only)
        self.finalize_bytecode();

        // Emit import verification metadata if imports exist (appended after string blob)
        self.emit_import_metadata(&import_table)
            .map_err(|_| VMError::InvalidScript)?;

        // Compute layout offsets
        let desc_size = core::mem::size_of::<five_protocol::ConstantPoolDescriptor>();
        let header_len = self.header_bytes.len();
        let metadata_len = self.metadata_bytes.len();
        let base_offset = header_len + metadata_len + desc_size;
        let pool_offset = (base_offset + 7) & !7; // 8-byte alignment
        let padding_len = pool_offset - base_offset;

        let pool_slots = self.constant_pool.pool_slots();
        let pool_size = pool_slots as usize * 8;
        let code_offset = pool_offset + pool_size;

        // Patch dispatcher jump/call offsets with absolute base
        dispatcher.patch_dispatch_logic_with_base(self, code_offset)?;

        // Patch all jumps and function calls with their correct offsets (absolute)
        ast_generator.patch_with_base(self, code_offset)?;
        self.optimize_bytecode_post_lowering(code_offset)?;

        let string_blob = self.constant_pool.string_blob();
        let string_blob_offset = code_offset + self.bytecode.len();
        let string_blob_len = string_blob.len();

        // Update header features to include constant pool flags
        self.header_features |= five_protocol::FEATURE_CONSTANT_POOL;
        if string_blob_len > 0 {
            self.header_features |= five_protocol::FEATURE_CONSTANT_POOL_STRINGS;
        }
        let has_compact_immediates = self.bytecode.iter().any(|opcode| {
            matches!(
                *opcode,
                five_protocol::opcodes::LOAD_FIELD_S
                    | five_protocol::opcodes::STORE_FIELD_S
                    | five_protocol::opcodes::LOAD_FIELD_PUBKEY_S
                    | five_protocol::opcodes::STORE_FIELD_ZERO_S
                    | five_protocol::opcodes::JUMP_S8
                    | five_protocol::opcodes::JUMP_IF_NOT_S8
                    | five_protocol::opcodes::BR_EQ_U8_S8
            )
        });
        if has_compact_immediates {
            self.header_features |= five_protocol::FEATURE_COMPACT_IMMEDIATES;
        }
        let feature_bytes = self.header_features.to_le_bytes();
        if self.header_bytes.len() >= 8 {
            self.header_bytes[4..8].copy_from_slice(&feature_bytes);
        }

        // Build descriptor bytes
        let desc = five_protocol::ConstantPoolDescriptor {
            pool_offset: pool_offset as u32,
            string_blob_offset: string_blob_offset as u32,
            string_blob_len: string_blob_len as u32,
            pool_slots,
            reserved: 0,
        };
        let mut desc_bytes = Vec::with_capacity(desc_size);
        desc_bytes.extend_from_slice(&desc.pool_offset.to_le_bytes());
        desc_bytes.extend_from_slice(&desc.string_blob_offset.to_le_bytes());
        desc_bytes.extend_from_slice(&desc.string_blob_len.to_le_bytes());
        desc_bytes.extend_from_slice(&desc.pool_slots.to_le_bytes());
        desc_bytes.extend_from_slice(&desc.reserved.to_le_bytes());

        // Assemble final bytecode
        let mut final_bytecode = Vec::new();
        final_bytecode.extend_from_slice(&self.header_bytes);
        final_bytecode.extend_from_slice(&self.metadata_bytes);
        final_bytecode.extend_from_slice(&desc_bytes);
        if padding_len > 0 {
            final_bytecode.extend_from_slice(&vec![0u8; padding_len]);
        }
        final_bytecode.extend_from_slice(&self.constant_pool.pool_bytes());
        final_bytecode.extend_from_slice(&self.bytecode);
        final_bytecode.extend_from_slice(string_blob);

        // CRITICAL: Verify bytecode JUMP targets before deployment
        // Import verification metadata is appended after executable code and may contain
        // arbitrary bytes that look like opcodes, so validate only the executable region.
        let verification_result = disassembler::verify_jump_targets(&final_bytecode);
        if !verification_result.is_valid {
            eprintln!("BYTECODE VERIFICATION FAILED:");
            eprintln!("{}", verification_result.error_summary());
            #[cfg(debug_assertions)]
            {
                panic!("Bytecode contains invalid JUMP targets - check disassembler/verification.rs or jumps.rs");
            }
            #[cfg(not(debug_assertions))]
            {
                return Err(VMError::InvalidInstructionPointer);
            }
        } else {
            eprintln!(
                "BYTECODE VERIFICATION: {} jumps validated, all within {} bytes",
                verification_result.jump_count, verification_result.bytecode_length
            );
        }

        final_bytecode.extend_from_slice(&self.import_metadata_bytes);
        self.bytecode = final_bytecode;

        // Debug: print final bytecode summary to help diagnose missing opcodes in tests
        {
            let contains_require = self.bytecode.contains(&five_protocol::opcodes::REQUIRE);
            eprintln!(
                "DEBUG: final bytecode len = {}, contains REQUIRE = {}",
                self.bytecode.len(),
                contains_require
            );
            let dump_len = std::cmp::min(self.bytecode.len(), 256);
            eprintln!(
                "DEBUG: final bytecode (first {} bytes) = {:?}",
                dump_len,
                &self.bytecode[..dump_len]
            );
        }

        Ok(self.bytecode.clone())
    }

    fn inline_private_helpers(&self, ast: &AstNode) -> AstNode {
        if matches!(
            std::env::var("FIVE_DISABLE_INLINE_OPT")
                .ok()
                .as_deref()
                .map(str::to_ascii_lowercase)
                .as_deref(),
            Some("1") | Some("true") | Some("yes") | Some("on")
        ) {
            return ast.clone();
        }

        #[derive(Clone)]
        struct InlineCandidate {
            parameters: Vec<InstructionParameter>,
            body: AstNode,
        }

        fn ast_weight(node: &AstNode) -> usize {
            match node {
                AstNode::Block { statements, .. } => {
                    1 + statements.iter().map(ast_weight).sum::<usize>()
                }
                AstNode::IfStatement {
                    condition,
                    then_branch,
                    else_branch,
                } => {
                    1 + ast_weight(condition)
                        + ast_weight(then_branch)
                        + else_branch.as_ref().map(|n| ast_weight(n)).unwrap_or(0)
                }
                AstNode::FunctionCall { args, .. } => {
                    1 + args.iter().map(ast_weight).sum::<usize>()
                }
                AstNode::Assignment { value, .. } => 1 + ast_weight(value),
                AstNode::LetStatement { value, .. } => 1 + ast_weight(value),
                AstNode::RequireStatement { condition } => 1 + ast_weight(condition),
                AstNode::FieldAssignment { object, value, .. } => {
                    1 + ast_weight(object) + ast_weight(value)
                }
                AstNode::ReturnStatement { value } => {
                    1 + value.as_ref().map(|v| ast_weight(v)).unwrap_or(0)
                }
                AstNode::WhileLoop { condition, body } => 1 + ast_weight(condition) + ast_weight(body),
                AstNode::DoWhileLoop { body, condition } => 1 + ast_weight(body) + ast_weight(condition),
                AstNode::ForLoop {
                    init,
                    condition,
                    update,
                    body,
                } => {
                    1 + init.as_ref().map(|n| ast_weight(n)).unwrap_or(0)
                        + condition.as_ref().map(|n| ast_weight(n)).unwrap_or(0)
                        + update.as_ref().map(|n| ast_weight(n)).unwrap_or(0)
                        + ast_weight(body)
                }
                AstNode::ForInLoop { iterable, body, .. }
                | AstNode::ForOfLoop { iterable, body, .. } => 1 + ast_weight(iterable) + ast_weight(body),
                _ => 1,
            }
        }

        fn body_is_safe_inline_candidate(node: &AstNode) -> bool {
            fn has_disallowed(n: &AstNode) -> bool {
                match n {
                    AstNode::FunctionCall { .. }
                    | AstNode::IfStatement { .. }
                    | AstNode::MatchExpression { .. }
                    | AstNode::WhileLoop { .. }
                    | AstNode::DoWhileLoop { .. }
                    | AstNode::ForLoop { .. }
                    | AstNode::ForInLoop { .. }
                    | AstNode::ForOfLoop { .. }
                    | AstNode::SwitchStatement { .. }
                    | AstNode::BreakStatement { .. }
                    | AstNode::ContinueStatement { .. }
                    | AstNode::ReturnStatement { .. } => true,
                    AstNode::Block { statements, .. } => statements.iter().any(has_disallowed),
                    AstNode::Assignment { value, .. } => has_disallowed(value),
                    AstNode::FieldAssignment { object, value, .. } => {
                        has_disallowed(object) || has_disallowed(value)
                    }
                    AstNode::RequireStatement { condition } => has_disallowed(condition),
                    AstNode::MethodCall { object, args, .. } => {
                        has_disallowed(object) || args.iter().any(has_disallowed)
                    }
                    AstNode::LetStatement { value, .. } => has_disallowed(value),
                    AstNode::TupleDestructuring { value, .. } => has_disallowed(value),
                    AstNode::TupleAssignment { targets, value } => {
                        targets.iter().any(has_disallowed) || has_disallowed(value)
                    }
                    AstNode::FieldAccess { object, .. } => has_disallowed(object),
                    AstNode::BinaryExpression { left, right, .. } => {
                        has_disallowed(left) || has_disallowed(right)
                    }
                    AstNode::UnaryExpression { operand, .. } => has_disallowed(operand),
                    AstNode::ArrayLiteral { elements } | AstNode::TupleLiteral { elements } => {
                        elements.iter().any(has_disallowed)
                    }
                    AstNode::ArrayAccess { array, index } => {
                        has_disallowed(array) || has_disallowed(index)
                    }
                    AstNode::Cast { value, target_type } => {
                        has_disallowed(value) || has_disallowed(target_type)
                    }
                    AstNode::ErrorPropagation { expression } => has_disallowed(expression),
                    AstNode::TemplateLiteral { parts } => parts.iter().any(has_disallowed),
                    AstNode::StructLiteral { fields } => {
                        fields.iter().any(|f| has_disallowed(&f.value))
                    }
                    AstNode::EmitStatement { fields, .. } => {
                        fields.iter().any(|f| has_disallowed(&f.value))
                    }
                    _ => false,
                }
            }

            let stmt_count_ok = match node {
                AstNode::Block { statements, .. } => statements.len() <= 8,
                _ => false,
            };
            stmt_count_ok && ast_weight(node) <= 64 && !has_disallowed(node)
        }

        fn count_calls(node: &AstNode, private_names: &HashSet<String>, counts: &mut HashMap<String, usize>) {
            match node {
                AstNode::FunctionCall { name, args } => {
                    if private_names.contains(name) {
                        *counts.entry(name.clone()).or_insert(0) += 1;
                    }
                    for arg in args {
                        count_calls(arg, private_names, counts);
                    }
                }
                AstNode::Block { statements, .. } => {
                    for stmt in statements {
                        count_calls(stmt, private_names, counts);
                    }
                }
                AstNode::Assignment { value, .. } => count_calls(value, private_names, counts),
                AstNode::FieldAssignment { object, value, .. } => {
                    count_calls(object, private_names, counts);
                    count_calls(value, private_names, counts);
                }
                AstNode::RequireStatement { condition } => {
                    count_calls(condition, private_names, counts)
                }
                AstNode::MethodCall { object, args, .. } => {
                    count_calls(object, private_names, counts);
                    for arg in args {
                        count_calls(arg, private_names, counts);
                    }
                }
                AstNode::LetStatement { value, .. } => count_calls(value, private_names, counts),
                AstNode::TupleDestructuring { value, .. } => count_calls(value, private_names, counts),
                AstNode::TupleAssignment { targets, value } => {
                    for target in targets {
                        count_calls(target, private_names, counts);
                    }
                    count_calls(value, private_names, counts);
                }
                AstNode::IfStatement {
                    condition,
                    then_branch,
                    else_branch,
                } => {
                    count_calls(condition, private_names, counts);
                    count_calls(then_branch, private_names, counts);
                    if let Some(else_branch) = else_branch {
                        count_calls(else_branch, private_names, counts);
                    }
                }
                AstNode::MatchExpression { expression, arms } => {
                    count_calls(expression, private_names, counts);
                    for arm in arms {
                        count_calls(&arm.pattern, private_names, counts);
                        if let Some(guard) = &arm.guard {
                            count_calls(guard, private_names, counts);
                        }
                        count_calls(&arm.body, private_names, counts);
                    }
                }
                AstNode::ReturnStatement { value } => {
                    if let Some(value) = value {
                        count_calls(value, private_names, counts);
                    }
                }
                AstNode::ForLoop {
                    init,
                    condition,
                    update,
                    body,
                } => {
                    if let Some(init) = init {
                        count_calls(init, private_names, counts);
                    }
                    if let Some(condition) = condition {
                        count_calls(condition, private_names, counts);
                    }
                    if let Some(update) = update {
                        count_calls(update, private_names, counts);
                    }
                    count_calls(body, private_names, counts);
                }
                AstNode::ForInLoop { iterable, body, .. }
                | AstNode::ForOfLoop { iterable, body, .. } => {
                    count_calls(iterable, private_names, counts);
                    count_calls(body, private_names, counts);
                }
                AstNode::WhileLoop { condition, body } => {
                    count_calls(condition, private_names, counts);
                    count_calls(body, private_names, counts);
                }
                AstNode::DoWhileLoop { body, condition } => {
                    count_calls(body, private_names, counts);
                    count_calls(condition, private_names, counts);
                }
                AstNode::SwitchStatement {
                    discriminant,
                    cases,
                    default_case,
                } => {
                    count_calls(discriminant, private_names, counts);
                    for case in cases {
                        count_calls(&case.pattern, private_names, counts);
                        for stmt in &case.body {
                            count_calls(stmt, private_names, counts);
                        }
                    }
                    if let Some(default_case) = default_case {
                        count_calls(default_case, private_names, counts);
                    }
                }
                AstNode::ArrowFunction { body, .. } => count_calls(body, private_names, counts),
                AstNode::StructLiteral { fields } => {
                    for field in fields {
                        count_calls(&field.value, private_names, counts);
                    }
                }
                AstNode::ArrayLiteral { elements } | AstNode::TupleLiteral { elements } => {
                    for element in elements {
                        count_calls(element, private_names, counts);
                    }
                }
                AstNode::FieldAccess { object, .. } => count_calls(object, private_names, counts),
                AstNode::Cast { value, target_type } => {
                    count_calls(value, private_names, counts);
                    count_calls(target_type, private_names, counts);
                }
                AstNode::TupleAccess { object, .. } => count_calls(object, private_names, counts),
                AstNode::ArrayAccess { array, index } => {
                    count_calls(array, private_names, counts);
                    count_calls(index, private_names, counts);
                }
                AstNode::ErrorPropagation { expression } => {
                    count_calls(expression, private_names, counts)
                }
                AstNode::TemplateLiteral { parts } => {
                    for part in parts {
                        count_calls(part, private_names, counts);
                    }
                }
                AstNode::UnaryExpression { operand, .. } => {
                    count_calls(operand, private_names, counts)
                }
                AstNode::BinaryExpression { left, right, .. } => {
                    count_calls(left, private_names, counts);
                    count_calls(right, private_names, counts);
                }
                _ => {}
            }
        }

        fn rewrite_node(
            node: &AstNode,
            candidates: &HashMap<String, InlineCandidate>,
            statement_ctx: bool,
        ) -> AstNode {
            if statement_ctx {
                if let AstNode::FunctionCall { name, args } = node {
                    if let Some(candidate) = candidates.get(name) {
                        if candidate.parameters.len() == args.len() {
                            let mut statements = Vec::with_capacity(
                                candidate.parameters.len() + 1,
                            );
                            for (param, arg) in candidate.parameters.iter().zip(args.iter()) {
                                statements.push(AstNode::LetStatement {
                                    name: param.name.clone(),
                                    type_annotation: None,
                                    is_mutable: false,
                                    value: Box::new(rewrite_node(arg, candidates, false)),
                                });
                            }
                            match &candidate.body {
                                AstNode::Block {
                                    statements: body_statements,
                                    ..
                                } => {
                                    for stmt in body_statements {
                                        statements.push(rewrite_node(stmt, candidates, true));
                                    }
                                }
                                other => statements.push(rewrite_node(other, candidates, true)),
                            }
                            return AstNode::Block {
                                statements,
                                kind: BlockKind::Regular,
                            };
                        }
                    }
                }
            }

            match node {
                AstNode::Program {
                    program_name,
                    field_definitions,
                    instruction_definitions,
                    event_definitions,
                    account_definitions,
                    type_definitions,
                    interface_definitions,
                    import_statements,
                    init_block,
                    constraints_block,
                } => AstNode::Program {
                    program_name: program_name.clone(),
                    field_definitions: field_definitions.clone(),
                    instruction_definitions: instruction_definitions
                        .iter()
                        .map(|n| rewrite_node(n, candidates, false))
                        .collect(),
                    event_definitions: event_definitions.clone(),
                    account_definitions: account_definitions.clone(),
                    type_definitions: type_definitions.clone(),
                    interface_definitions: interface_definitions.clone(),
                    import_statements: import_statements.clone(),
                    init_block: init_block
                        .as_ref()
                        .map(|n| Box::new(rewrite_node(n, candidates, true))),
                    constraints_block: constraints_block
                        .as_ref()
                        .map(|n| Box::new(rewrite_node(n, candidates, true))),
                },
                AstNode::Block { statements, kind } => AstNode::Block {
                    statements: statements
                        .iter()
                        .map(|stmt| rewrite_node(stmt, candidates, true))
                        .collect(),
                    kind: kind.clone(),
                },
                AstNode::InstructionDefinition {
                    name,
                    parameters,
                    return_type,
                    body,
                    visibility,
                    is_public,
                } => AstNode::InstructionDefinition {
                    name: name.clone(),
                    parameters: parameters.clone(),
                    return_type: return_type.clone(),
                    body: Box::new(rewrite_node(body, candidates, true)),
                    visibility: *visibility,
                    is_public: *is_public,
                },
                AstNode::Assignment { target, value } => AstNode::Assignment {
                    target: target.clone(),
                    value: Box::new(rewrite_node(value, candidates, false)),
                },
                AstNode::FieldAssignment { object, field, value } => AstNode::FieldAssignment {
                    object: Box::new(rewrite_node(object, candidates, false)),
                    field: field.clone(),
                    value: Box::new(rewrite_node(value, candidates, false)),
                },
                AstNode::RequireStatement { condition } => AstNode::RequireStatement {
                    condition: Box::new(rewrite_node(condition, candidates, false)),
                },
                AstNode::MethodCall { object, method, args } => AstNode::MethodCall {
                    object: Box::new(rewrite_node(object, candidates, false)),
                    method: method.clone(),
                    args: args
                        .iter()
                        .map(|arg| rewrite_node(arg, candidates, false))
                        .collect(),
                },
                AstNode::LetStatement {
                    name,
                    type_annotation,
                    is_mutable,
                    value,
                } => AstNode::LetStatement {
                    name: name.clone(),
                    type_annotation: type_annotation.clone(),
                    is_mutable: *is_mutable,
                    value: Box::new(rewrite_node(value, candidates, false)),
                },
                AstNode::TupleDestructuring { targets, value } => AstNode::TupleDestructuring {
                    targets: targets.clone(),
                    value: Box::new(rewrite_node(value, candidates, false)),
                },
                AstNode::TupleAssignment { targets, value } => AstNode::TupleAssignment {
                    targets: targets
                        .iter()
                        .map(|n| rewrite_node(n, candidates, false))
                        .collect(),
                    value: Box::new(rewrite_node(value, candidates, false)),
                },
                AstNode::IfStatement {
                    condition,
                    then_branch,
                    else_branch,
                } => AstNode::IfStatement {
                    condition: Box::new(rewrite_node(condition, candidates, false)),
                    then_branch: Box::new(rewrite_node(then_branch, candidates, true)),
                    else_branch: else_branch
                        .as_ref()
                        .map(|n| Box::new(rewrite_node(n, candidates, true))),
                },
                AstNode::ReturnStatement { value } => AstNode::ReturnStatement {
                    value: value
                        .as_ref()
                        .map(|v| Box::new(rewrite_node(v, candidates, false))),
                },
                AstNode::MatchExpression { expression, arms } => AstNode::MatchExpression {
                    expression: Box::new(rewrite_node(expression, candidates, false)),
                    arms: arms
                        .iter()
                        .map(|arm| crate::ast::MatchArm {
                            pattern: Box::new(rewrite_node(&arm.pattern, candidates, false)),
                            guard: arm
                                .guard
                                .as_ref()
                                .map(|g| Box::new(rewrite_node(g, candidates, false))),
                            body: Box::new(rewrite_node(&arm.body, candidates, true)),
                        })
                        .collect(),
                },
                AstNode::ForLoop {
                    init,
                    condition,
                    update,
                    body,
                } => AstNode::ForLoop {
                    init: init
                        .as_ref()
                        .map(|n| Box::new(rewrite_node(n, candidates, true))),
                    condition: condition
                        .as_ref()
                        .map(|n| Box::new(rewrite_node(n, candidates, false))),
                    update: update
                        .as_ref()
                        .map(|n| Box::new(rewrite_node(n, candidates, true))),
                    body: Box::new(rewrite_node(body, candidates, true)),
                },
                AstNode::ForInLoop {
                    variable,
                    iterable,
                    body,
                } => AstNode::ForInLoop {
                    variable: variable.clone(),
                    iterable: Box::new(rewrite_node(iterable, candidates, false)),
                    body: Box::new(rewrite_node(body, candidates, true)),
                },
                AstNode::ForOfLoop {
                    variable,
                    iterable,
                    body,
                } => AstNode::ForOfLoop {
                    variable: variable.clone(),
                    iterable: Box::new(rewrite_node(iterable, candidates, false)),
                    body: Box::new(rewrite_node(body, candidates, true)),
                },
                AstNode::WhileLoop { condition, body } => AstNode::WhileLoop {
                    condition: Box::new(rewrite_node(condition, candidates, false)),
                    body: Box::new(rewrite_node(body, candidates, true)),
                },
                AstNode::DoWhileLoop { body, condition } => AstNode::DoWhileLoop {
                    body: Box::new(rewrite_node(body, candidates, true)),
                    condition: Box::new(rewrite_node(condition, candidates, false)),
                },
                AstNode::SwitchStatement {
                    discriminant,
                    cases,
                    default_case,
                } => AstNode::SwitchStatement {
                    discriminant: Box::new(rewrite_node(discriminant, candidates, false)),
                    cases: cases
                        .iter()
                        .map(|case| crate::ast::SwitchCase {
                            pattern: Box::new(rewrite_node(&case.pattern, candidates, false)),
                            body: case
                                .body
                                .iter()
                                .map(|n| rewrite_node(n, candidates, true))
                                .collect(),
                        })
                        .collect(),
                    default_case: default_case
                        .as_ref()
                        .map(|n| Box::new(rewrite_node(n, candidates, true))),
                },
                AstNode::FunctionCall { name, args } => AstNode::FunctionCall {
                    name: name.clone(),
                    args: args
                        .iter()
                        .map(|arg| rewrite_node(arg, candidates, false))
                        .collect(),
                },
                AstNode::FieldAccess { object, field } => AstNode::FieldAccess {
                    object: Box::new(rewrite_node(object, candidates, false)),
                    field: field.clone(),
                },
                AstNode::Cast { value, target_type } => AstNode::Cast {
                    value: Box::new(rewrite_node(value, candidates, false)),
                    target_type: Box::new(rewrite_node(target_type, candidates, false)),
                },
                AstNode::TupleAccess { object, index } => AstNode::TupleAccess {
                    object: Box::new(rewrite_node(object, candidates, false)),
                    index: *index,
                },
                AstNode::ArrayAccess { array, index } => AstNode::ArrayAccess {
                    array: Box::new(rewrite_node(array, candidates, false)),
                    index: Box::new(rewrite_node(index, candidates, false)),
                },
                AstNode::ErrorPropagation { expression } => AstNode::ErrorPropagation {
                    expression: Box::new(rewrite_node(expression, candidates, false)),
                },
                AstNode::TemplateLiteral { parts } => AstNode::TemplateLiteral {
                    parts: parts
                        .iter()
                        .map(|p| rewrite_node(p, candidates, false))
                        .collect(),
                },
                AstNode::UnaryExpression { operator, operand } => AstNode::UnaryExpression {
                    operator: operator.clone(),
                    operand: Box::new(rewrite_node(operand, candidates, false)),
                },
                AstNode::BinaryExpression {
                    operator,
                    left,
                    right,
                } => AstNode::BinaryExpression {
                    operator: operator.clone(),
                    left: Box::new(rewrite_node(left, candidates, false)),
                    right: Box::new(rewrite_node(right, candidates, false)),
                },
                AstNode::StructLiteral { fields } => AstNode::StructLiteral {
                    fields: fields
                        .iter()
                        .map(|f| crate::ast::StructLiteralField {
                            field_name: f.field_name.clone(),
                            value: Box::new(rewrite_node(&f.value, candidates, false)),
                        })
                        .collect(),
                },
                AstNode::ArrayLiteral { elements } => AstNode::ArrayLiteral {
                    elements: elements
                        .iter()
                        .map(|e| rewrite_node(e, candidates, false))
                        .collect(),
                },
                AstNode::TupleLiteral { elements } => AstNode::TupleLiteral {
                    elements: elements
                        .iter()
                        .map(|e| rewrite_node(e, candidates, false))
                        .collect(),
                },
                AstNode::EmitStatement { event_name, fields } => AstNode::EmitStatement {
                    event_name: event_name.clone(),
                    fields: fields
                        .iter()
                        .map(|f| crate::ast::EventFieldAssignment {
                            field_name: f.field_name.clone(),
                            value: Box::new(rewrite_node(&f.value, candidates, false)),
                        })
                        .collect(),
                },
                _ => node.clone(),
            }
        }

        let AstNode::Program {
            program_name,
            field_definitions,
            instruction_definitions,
            event_definitions,
            account_definitions,
            type_definitions,
            interface_definitions,
            import_statements,
            init_block,
            constraints_block,
        } = ast
        else {
            return ast.clone();
        };

        let mut private_candidates: HashMap<String, InlineCandidate> = HashMap::new();
        let mut private_names: HashSet<String> = HashSet::new();

        for definition in instruction_definitions {
            if let AstNode::InstructionDefinition {
                name,
                parameters,
                return_type,
                body,
                visibility,
                ..
            } = definition
            {
                if *visibility == Visibility::Public || name.contains("::") || return_type.is_some() {
                    continue;
                }
                if body_is_safe_inline_candidate(body) {
                    private_names.insert(name.clone());
                    private_candidates.insert(
                        name.clone(),
                        InlineCandidate {
                            parameters: parameters.clone(),
                            body: (**body).clone(),
                        },
                    );
                }
            }
        }

        if private_candidates.is_empty() {
            return ast.clone();
        }

        let mut call_counts: HashMap<String, usize> = HashMap::new();
        for definition in instruction_definitions {
            count_calls(definition, &private_names, &mut call_counts);
        }
        if let Some(init_block) = init_block {
            count_calls(init_block, &private_names, &mut call_counts);
        }
        if let Some(constraints_block) = constraints_block {
            count_calls(constraints_block, &private_names, &mut call_counts);
        }

        private_candidates.retain(|name, _| call_counts.get(name).copied().unwrap_or(0) == 1);
        if private_candidates.is_empty() {
            return ast.clone();
        }

        let mut rewritten_instruction_defs: Vec<AstNode> = instruction_definitions
            .iter()
            .map(|node| rewrite_node(node, &private_candidates, false))
            .collect();

        rewritten_instruction_defs.retain(|definition| {
            if let AstNode::InstructionDefinition {
                name, visibility, ..
            } = definition
            {
                !(*visibility != Visibility::Public && private_candidates.contains_key(name))
            } else {
                true
            }
        });

        AstNode::Program {
            program_name: program_name.clone(),
            field_definitions: field_definitions.clone(),
            instruction_definitions: rewritten_instruction_defs,
            event_definitions: event_definitions.clone(),
            account_definitions: account_definitions.clone(),
            type_definitions: type_definitions.clone(),
            interface_definitions: interface_definitions.clone(),
            import_statements: import_statements.clone(),
            init_block: init_block
                .as_ref()
                .map(|n| Box::new(rewrite_node(n, &private_candidates, true))),
            constraints_block: constraints_block
                .as_ref()
                .map(|n| Box::new(rewrite_node(n, &private_candidates, true))),
        }
    }

    fn optimize_bytecode_post_lowering(&mut self, code_offset: usize) -> Result<(), VMError> {
        if matches!(
            std::env::var("FIVE_DISABLE_BYTECODE_PEEPHOLE")
                .ok()
                .as_deref()
                .map(str::to_ascii_lowercase)
                .as_deref(),
            Some("1") | Some("true") | Some("yes") | Some("on")
        ) {
            return Ok(());
        }

        use crate::bytecode_generator::disassembler::BytecodeInspector;
        use five_protocol::opcodes;

        #[derive(Clone)]
        enum BranchKind {
            Jump { target_abs: i32 },
            JumpIfNot { target_abs: i32 },
            BrEqU8 { compare: u8, target_abs: i32 },
        }

        #[derive(Clone)]
        struct InstructionRewrite {
            old_start: usize,
            old_size: usize,
            bytes: Vec<u8>,
            branch: Option<BranchKind>,
            removed: bool,
        }

        fn read_u16_le(bytes: &[u8], offset: usize) -> Option<u16> {
            if offset + 1 >= bytes.len() {
                return None;
            }
            Some(u16::from_le_bytes([bytes[offset], bytes[offset + 1]]))
        }

        let original = self.bytecode.clone();
        if original.is_empty() {
            return Ok(());
        }

        let code_offset_i32 = code_offset as i32;
        let original_len = original.len();
        let code_end_abs = code_offset_i32 + original_len as i32;
        let mut instructions: Vec<InstructionRewrite> = Vec::new();

        let mut pc = 0usize;
        while pc < original_len {
            let size = BytecodeInspector::instruction_size(&original, pc);
            if size == 0 || pc + size > original_len {
                break;
            }

            let opcode = original[pc];
            let mut bytes = original[pc..pc + size].to_vec();

            if opcode == opcodes::PUSH_U8 && size >= 2 {
                bytes = match original[pc + 1] {
                    0 => vec![opcodes::PUSH_0],
                    1 => vec![opcodes::PUSH_1],
                    2 => vec![opcodes::PUSH_2],
                    3 => vec![opcodes::PUSH_3],
                    _ => bytes,
                };
            } else if opcode == opcodes::GET_LOCAL && size >= 2 {
                bytes = match original[pc + 1] {
                    0 => vec![opcodes::GET_LOCAL_0],
                    1 => vec![opcodes::GET_LOCAL_1],
                    2 => vec![opcodes::GET_LOCAL_2],
                    3 => vec![opcodes::GET_LOCAL_3],
                    _ => bytes,
                };
            } else if opcode == opcodes::SET_LOCAL && size >= 2 {
                bytes = match original[pc + 1] {
                    0 => vec![opcodes::SET_LOCAL_0],
                    1 => vec![opcodes::SET_LOCAL_1],
                    2 => vec![opcodes::SET_LOCAL_2],
                    3 => vec![opcodes::SET_LOCAL_3],
                    _ => bytes,
                };
            }

            let branch = match opcode {
                opcodes::JUMP => read_u16_le(&original, pc + 1)
                    .map(|target| BranchKind::Jump { target_abs: target as i32 }),
                opcodes::JUMP_IF_NOT => read_u16_le(&original, pc + 1)
                    .map(|target| BranchKind::JumpIfNot { target_abs: target as i32 }),
                opcodes::JUMP_S8 => {
                    if size >= 2 {
                        let rel = original[pc + 1] as i8 as i32;
                        Some(BranchKind::Jump {
                            target_abs: code_offset_i32 + pc as i32 + 2 + rel,
                        })
                    } else {
                        None
                    }
                }
                opcodes::JUMP_IF_NOT_S8 => {
                    if size >= 2 {
                        let rel = original[pc + 1] as i8 as i32;
                        Some(BranchKind::JumpIfNot {
                            target_abs: code_offset_i32 + pc as i32 + 2 + rel,
                        })
                    } else {
                        None
                    }
                }
                opcodes::BR_EQ_U8 => {
                    if size >= 4 {
                        let compare = original[pc + 1];
                        let rel = read_u16_le(&original, pc + 2)
                            .map(|raw| raw as i16 as i32)
                            .unwrap_or(0);
                        Some(BranchKind::BrEqU8 {
                            compare,
                            target_abs: code_offset_i32 + pc as i32 + 4 + rel,
                        })
                    } else {
                        None
                    }
                }
                opcodes::BR_EQ_U8_S8 => {
                    if size >= 3 {
                        let compare = original[pc + 1];
                        let rel = original[pc + 2] as i8 as i32;
                        Some(BranchKind::BrEqU8 {
                            compare,
                            target_abs: code_offset_i32 + pc as i32 + 3 + rel,
                        })
                    } else {
                        None
                    }
                }
                _ => None,
            };

            let mut removed = false;
            if let Some(BranchKind::Jump { target_abs }) = &branch {
                let next_abs = code_offset_i32 + pc as i32 + size as i32;
                if *target_abs == next_abs {
                    removed = true;
                }
            }

            instructions.push(InstructionRewrite {
                old_start: pc,
                old_size: size,
                bytes,
                branch,
                removed,
            });

            pc += size;
        }

        if instructions.is_empty() {
            return Ok(());
        }

        for _ in 0..10 {
            let mut changed = false;
            let mut old_to_new: HashMap<usize, usize> = HashMap::new();
            let mut new_pc = 0usize;
            for inst in &instructions {
                old_to_new.insert(inst.old_start, new_pc);
                if !inst.removed {
                    new_pc += inst.bytes.len();
                }
            }
            let new_code_end_abs = code_offset_i32 + new_pc as i32;

            for inst in &mut instructions {
                if inst.removed {
                    continue;
                }
                let Some(branch) = &inst.branch else {
                    continue;
                };
                let Some(&inst_new_start) = old_to_new.get(&inst.old_start) else {
                    continue;
                };

                let old_target_abs = match branch {
                    BranchKind::Jump { target_abs }
                    | BranchKind::JumpIfNot { target_abs }
                    | BranchKind::BrEqU8 { target_abs, .. } => *target_abs,
                };

                let mut translated_target_abs = old_target_abs;
                if old_target_abs >= code_offset_i32 && old_target_abs < code_end_abs {
                    let old_rel = (old_target_abs - code_offset_i32) as usize;
                    if let Some(new_rel) = old_to_new.get(&old_rel) {
                        translated_target_abs = code_offset_i32 + *new_rel as i32;
                    }
                } else if old_target_abs == code_end_abs {
                    translated_target_abs = new_code_end_abs;
                }

                match branch {
                    BranchKind::Jump { .. } => {
                        let rel = translated_target_abs - (code_offset_i32 + inst_new_start as i32 + 2);
                        let new_bytes = if (i8::MIN as i32..=i8::MAX as i32).contains(&rel) {
                            vec![opcodes::JUMP_S8, rel as i8 as u8]
                        } else {
                            let target_u16 = translated_target_abs as u16;
                            vec![
                                opcodes::JUMP,
                                (target_u16 & 0xFF) as u8,
                                (target_u16 >> 8) as u8,
                            ]
                        };
                        if new_bytes != inst.bytes {
                            inst.bytes = new_bytes;
                            changed = true;
                        }
                        if inst.bytes.len() == 3 {
                            let next_abs = code_offset_i32 + inst.old_start as i32 + inst.old_size as i32;
                            if translated_target_abs == next_abs {
                                inst.removed = true;
                                changed = true;
                            }
                        }
                    }
                    BranchKind::JumpIfNot { .. } => {
                        let rel = translated_target_abs - (code_offset_i32 + inst_new_start as i32 + 2);
                        let new_bytes = if (i8::MIN as i32..=i8::MAX as i32).contains(&rel) {
                            vec![opcodes::JUMP_IF_NOT_S8, rel as i8 as u8]
                        } else {
                            let target_u16 = translated_target_abs as u16;
                            vec![
                                opcodes::JUMP_IF_NOT,
                                (target_u16 & 0xFF) as u8,
                                (target_u16 >> 8) as u8,
                            ]
                        };
                        if new_bytes != inst.bytes {
                            inst.bytes = new_bytes;
                            changed = true;
                        }
                    }
                    BranchKind::BrEqU8 { compare, .. } => {
                        let rel8 = translated_target_abs - (code_offset_i32 + inst_new_start as i32 + 3);
                        let new_bytes = if (i8::MIN as i32..=i8::MAX as i32).contains(&rel8) {
                            vec![opcodes::BR_EQ_U8_S8, *compare, rel8 as i8 as u8]
                        } else {
                            let rel16 = translated_target_abs - (code_offset_i32 + inst_new_start as i32 + 4);
                            if (i16::MIN as i32..=i16::MAX as i32).contains(&rel16) {
                                let rel_u16 = rel16 as i16 as u16;
                                vec![
                                    opcodes::BR_EQ_U8,
                                    *compare,
                                    (rel_u16 & 0xFF) as u8,
                                    (rel_u16 >> 8) as u8,
                                ]
                            } else {
                                inst.bytes.clone()
                            }
                        };
                        if new_bytes != inst.bytes {
                            inst.bytes = new_bytes;
                            changed = true;
                        }
                    }
                }
            }

            if !changed {
                break;
            }
        }

        let mut optimized = Vec::with_capacity(original_len);
        for inst in &instructions {
            if !inst.removed {
                optimized.extend_from_slice(&inst.bytes);
            }
        }

        if !Self::validate_optimized_control_flow(&optimized, code_offset) {
            self.bytecode = original;
            self.position = self.bytecode.len();
            return Ok(());
        }

        self.bytecode = optimized;
        self.position = self.bytecode.len();
        Ok(())
    }

    fn validate_optimized_control_flow(code: &[u8], code_offset: usize) -> bool {
        use crate::bytecode_generator::disassembler::BytecodeInspector;
        use five_protocol::opcodes;

        let mut starts = HashSet::new();
        let mut pc = 0usize;
        while pc < code.len() {
            let size = BytecodeInspector::instruction_size(code, pc);
            if size == 0 || pc + size > code.len() {
                return false;
            }
            starts.insert(code_offset as i32 + pc as i32);
            pc += size;
        }

        let code_start = code_offset as i32;
        let code_end = code_start + code.len() as i32;
        let mut pc = 0usize;
        while pc < code.len() {
            let size = BytecodeInspector::instruction_size(code, pc);
            let op = code[pc];
            let target = match op {
                opcodes::JUMP | opcodes::JUMP_IF_NOT => {
                    if pc + 2 >= code.len() {
                        return false;
                    }
                    Some(u16::from_le_bytes([code[pc + 1], code[pc + 2]]) as i32)
                }
                opcodes::JUMP_S8 | opcodes::JUMP_IF_NOT_S8 => {
                    if pc + 1 >= code.len() {
                        return false;
                    }
                    let rel = code[pc + 1] as i8 as i32;
                    Some(code_start + pc as i32 + 2 + rel)
                }
                opcodes::BR_EQ_U8 => {
                    if pc + 3 >= code.len() {
                        return false;
                    }
                    let rel = u16::from_le_bytes([code[pc + 2], code[pc + 3]]) as i16 as i32;
                    Some(code_start + pc as i32 + 4 + rel)
                }
                opcodes::BR_EQ_U8_S8 => {
                    if pc + 2 >= code.len() {
                        return false;
                    }
                    let rel = code[pc + 2] as i8 as i32;
                    Some(code_start + pc as i32 + 3 + rel)
                }
                _ => None,
            };

            if let Some(target) = target {
                if target < code_start || target >= code_end || !starts.contains(&target) {
                    return false;
                }
            }
            pc += size;
        }

        true
    }

    /// Reset generator state for new compilation
    pub fn reset(&mut self) {
        self.bytecode.clear();
        self.header_bytes.clear();
        self.metadata_bytes.clear();
        self.import_metadata_bytes.clear();
        self.header_features = 0;
        self.position = 0;
        self.functions.clear();
        self.symbol_table.clear();
        self.account_registry = types::AccountRegistry::new();
        self.field_counter = 0;
        self.constant_pool = constant_pool::ConstantPoolBuilder::new();
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

    /// Get reference to generated function-name metadata section
    pub fn get_metadata_bytes(&self) -> &[u8] {
        &self.metadata_bytes
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
        use crate::bytecode_generator::disassembler::BytecodeInspector;

        if self.bytecode.is_empty() {
            self.emit_opcode(five_protocol::opcodes::HALT);
            self.log_opcode("HALT", "End program execution");
            return;
        }

        let mut i = 0;
        let mut last_op = None;

        while i < self.bytecode.len() {
            let op = self.bytecode[i];
            last_op = Some(op);
            let size = BytecodeInspector::instruction_size(&self.bytecode, i);
            if size == 0 {
                break;
            }
            i += size;
        }

        if last_op != Some(five_protocol::opcodes::HALT) {
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
        ast_generator.set_require_batch_enabled(self.require_batch_enabled);

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
