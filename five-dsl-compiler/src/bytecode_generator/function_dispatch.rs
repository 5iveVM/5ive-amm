// Function metadata collection for DSL calls.

use super::scope_analyzer;
use super::types::*;
use super::{AccountSystem, OpcodeEmitter};
use super::import_table::ImportTable;
use crate::ast::{AstNode, InstructionParameter, TypeNode};
use crate::ast::ImportItem;
use crate::bytecode_generator::types; // Import the module directly
use crate::config::workspace::{ExportMetadata, LockFile};

use five_vm_mito::error::VMError;
use std::collections::HashMap;

/// Collects function metadata without JUMP_TABLE dispatch.
pub struct FunctionDispatcher {
    /// Function information for metadata collection
    functions: Vec<FunctionInfo>,

    /// Parameter cache for functions
    parameter_cache: HashMap<String, Vec<InstructionParameter>>,

    /// Current function account parameters
    current_function_params: HashMap<String, (String, Vec<String>)>,

    /// Imported functions: function_name -> (account_address, function_list)
    imported_functions: HashMap<String, (String, Option<Vec<String>>)>,

    /// Imported global fields: field_name -> (account_address, field_list)
    /// Now supported with LOAD_EXTERNAL_FIELD opcode for zero-copy access
    imported_fields: HashMap<String, (String, Option<Vec<String>>)>,

    /// Locations in bytecode that need to be patched with function offsets
    /// Map: function_name -> bytecode_offset_of_jump_target
    dispatch_patch_locations: HashMap<String, usize>,

    /// Locations in bytecode that need to be patched with dispatcher jump targets
    /// Vec of (patch_position, target_position) within the code section
    dispatch_jump_patches: Vec<(usize, usize)>,

    /// Public function entry points (function index -> call block offset in code section)
    public_entry_points: Vec<(u8, usize)>,

    /// Import verification table for Five bytecode accounts
    /// Stores both direct addresses and PDA seeds for imported Five bytecode
    import_table: ImportTable,
}

impl FunctionDispatcher {
    fn external_selector(name: &str) -> u16 {
        const OFFSET: u32 = 0x811C9DC5;
        const PRIME: u32 = 0x01000193;
        let mut hash = OFFSET;
        for b in name.as_bytes() {
            hash ^= *b as u32;
            hash = hash.wrapping_mul(PRIME);
        }
        (hash & 0xFFFF) as u16
    }

    fn is_valid_identifier(s: &str) -> bool {
        let mut chars = s.chars();
        let Some(first) = chars.next() else {
            return false;
        };
        if !(first.is_ascii_alphabetic() || first == '_') {
            return false;
        }
        chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
    }

    fn load_lockfile() -> Option<LockFile> {
        let cwd = std::env::current_dir().ok()?;
        let path = cwd.join("five.lock");
        LockFile::load(&path).ok()
    }

    fn resolve_import_item(
        item: &ImportItem,
        exports: Option<&ExportMetadata>,
    ) -> Result<(bool, String, HashMap<String, u16>, Vec<String>), VMError> {
        let classify_unqualified = |name: &str,
                                    exports: Option<&ExportMetadata>|
         -> Result<(bool, String, HashMap<String, u16>, Vec<String>), VMError> {
            if let Some(exports) = exports {
                let method_exists = exports.methods.iter().any(|m| m == name);
                let iface = exports.interfaces.iter().find(|i| i.name == name);

                if method_exists && iface.is_some() {
                    return Err(VMError::InvalidScript);
                }
                if !method_exists && iface.is_none() {
                    return Err(VMError::InvalidScript);
                }
                if let Some(iface) = iface {
                    let mut selectors = HashMap::new();
                    let mut import_names = Vec::new();
                    for (method, callee) in &iface.method_map {
                        selectors.insert(method.clone(), Self::external_selector(callee));
                        import_names.push(callee.clone());
                    }
                    return Ok((true, name.to_string(), selectors, import_names));
                }
            }

            Ok((false, name.to_string(), HashMap::new(), vec![name.to_string()]))
        };

        match item {
            ImportItem::Method(name) => {
                if let Some(exports) = exports {
                    if !exports.methods.iter().any(|m| m == name) {
                        return Err(VMError::InvalidScript);
                    }
                }
                Ok((false, name.clone(), HashMap::new(), vec![name.clone()]))
            }
            ImportItem::Interface(name) => {
                if let Some(exports) = exports {
                    let iface = exports
                        .interfaces
                        .iter()
                        .find(|i| i.name == *name)
                        .ok_or(VMError::InvalidScript)?;
                    let mut selectors = HashMap::new();
                    let mut import_names = Vec::new();
                    for (method, callee) in &iface.method_map {
                        selectors.insert(method.clone(), Self::external_selector(callee));
                        import_names.push(callee.clone());
                    }
                    Ok((true, name.clone(), selectors, import_names))
                } else {
                    Ok((true, name.clone(), HashMap::new(), vec![name.clone()]))
                }
            }
            ImportItem::Unqualified(name) => classify_unqualified(name, exports),
        }
    }

    /// Create a new function dispatcher
    pub fn new() -> Self {
        Self {
            functions: Vec::new(),
            parameter_cache: HashMap::new(),
            current_function_params: HashMap::new(),
            imported_functions: HashMap::new(),
            imported_fields: HashMap::new(),
            dispatch_patch_locations: HashMap::new(),
            dispatch_jump_patches: Vec::new(),
            public_entry_points: Vec::new(),
            import_table: ImportTable::new(),
        }
    }

    // Duplicate `get_functions` removed here. A single canonical `get_functions`
    // implementation lives later in this file and is used by callers.

    /// Check if AST contains PUBLIC functions that need external dispatching
    /// Private functions use direct CALL instructions and don't need dispatching
    pub fn has_callable_functions(&self, ast: &AstNode) -> bool {
        match ast {
            AstNode::Program {
                instruction_definitions,
                init_block,
                ..
            } => {
                println!(
                    "DEBUG: has_callable_functions - instruction_definitions.len() = {}",
                    instruction_definitions.len()
                );
                println!(
                    "DEBUG: has_callable_functions - init_block.is_some() = {}",
                    init_block.is_some()
                );

                // Check for ANY functions (public or private) since we need function dispatch
                // even for scripts with only private functions
                let has_any_functions = !instruction_definitions.is_empty();
                for (i, def) in instruction_definitions.iter().enumerate() {
                    if let AstNode::InstructionDefinition {
                        visibility, name, ..
                    } = def
                    {
                        println!(
                            "DEBUG: instruction_definition[{}] = {} (public: {})",
                            i, name, visibility.is_on_chain_callable()
                        );
                    }
                }

                // Return true if we have any functions OR an init block
                let result = has_any_functions || init_block.is_some();
                println!("DEBUG: has_callable_functions returning {} (has_functions: {}, init_block: {})",
                    result, has_any_functions, init_block.is_some());
                result
            }
            _ => {
                println!(
                    "DEBUG: has_callable_functions - not a Program node: {:?}",
                    ast
                );
                false
            }
        }
    }

    /// Main dispatcher generation orchestrator (simplified - metadata only)
    /// Coordinates with AST generator by populating function index cache
    pub fn generate_dispatcher<T: OpcodeEmitter>(
        &mut self,
        emitter: &mut T,
        ast: &AstNode,
        account_system: &mut AccountSystem,
        scope_analyzer: &mut scope_analyzer::ScopeAnalyzer,
        ast_generator: &mut super::ASTGenerator,
        symbol_table: &HashMap<String, FieldInfo>,
    ) -> Result<(), VMError> {
        if !self.has_callable_functions(ast) {
            return Ok(()); // No functions to dispatch
        }
        // Phase 1: Collect function information (including import statements)
        self.collect_function_info(ast)?;

        // SECURITY: Perform security analysis before code generation
        let mut security_checker = crate::security_rules::SecurityChecker::new();
        security_checker.set_imports(
            self.imported_functions.clone(),
            self.imported_fields.clone(),
        );

        println!("🔒 Five DSL: Running security analysis...");
        match security_checker.analyze_security(ast)? {
            Some(report) => {
                println!(
                    "❌ SECURITY VIOLATION: Compilation blocked due to security rule violations"
                );
                println!("{}", report);
                return Err(VMError::SecurityViolation);
            }
            None => {
                println!("✅ Five DSL: Security analysis passed - no violations detected");
            }
        }

        // Phase 2: Process account definitions
        account_system.process_account_definitions(ast)?;

        if let AstNode::Program {
            instruction_definitions,
            ..
        } = ast
        {
            ast_generator.cache_function_parameter_types(instruction_definitions);
        }

        // Phase 3: Generate function dispatch logic at beginning
        // RESTORED: Dispatch logic is required as MitoVM jumps to start_ip for all functions
        self.generate_function_dispatch_logic(emitter, ast, ast_generator, account_system)?;

        // Phase 4: Generate function bodies
        self.generate_function_bodies(
            emitter,
            ast,
            account_system,
            scope_analyzer,
            ast_generator,
            symbol_table,
        )?;

        // After generating functions, also generate the constraints block (if present).
        // Constraints often reference instruction parameters and global fields; generating
        // them here ensures the AST generator emits the runtime validation opcodes
        // (e.g., REQUIRE) that tests and later stages expect.
        if let AstNode::Program {
            constraints_block: Some(constraints),
            ..
        } = ast
        {
            // constraints is a &Box<AstNode> — pass the underlying AstNode reference
            ast_generator.generate_ast_node(emitter, constraints.as_ref())?;
        }

        Ok(())
    }

    /// Collect function metadata from AST instruction definitions
    pub fn collect_function_info(&mut self, ast: &AstNode) -> Result<(), VMError> {
        self.functions.clear();

        if let AstNode::Program {
            instruction_definitions,
            import_statements,
            init_block,
            ..
        } = ast
        {
            // Add init block as function 0 if present
            if init_block.is_some() {
                self.functions.push(FunctionInfo {
                    name: "__init".to_string(),
                    offset: 0, // Will be patched later
                    parameter_count: 0,
                    is_public: true, // __init is always public (entry point)
                    has_return_type: false, // init blocks don't have return types
                });

                // Cache empty parameters for __init to allow dispatch logic to work
                self.parameter_cache
                    .insert("__init".to_string(), Vec::new());
            }

            // Add instruction definitions as functions with visibility-based ordering
            // Phase 2: Separate public and private functions for proper visibility enforcement
            let mut public_functions = Vec::new();
            let mut private_functions = Vec::new();

            // Separate functions by visibility
            for instruction_def in instruction_definitions {
                if let AstNode::InstructionDefinition { visibility, .. } = instruction_def {
                    if visibility.is_on_chain_callable() {
                        public_functions.push(instruction_def);
                    } else {
                        private_functions.push(instruction_def);
                    }
                }
            }

            // Save counts before consuming the vectors
            let init_count = if init_block.is_some() { 1 } else { 0 };
            let public_count = public_functions.len() + init_count;
            let private_count = private_functions.len();

            // Process public functions first (get indices 0, 1, 2... - externally callable)
            for public_function in public_functions {
                if let AstNode::InstructionDefinition {
                    name,
                    parameters,
                    visibility,
                    return_type,
                    ..
                } = public_function
                {
                    self.functions.push(FunctionInfo {
                        name: name.clone(),
                        offset: 0, // Will be patched later
                        parameter_count: parameters.len() as u8,
                        is_public: visibility.is_on_chain_callable(), // Capture visibility from AST
                        has_return_type: return_type.is_some(), // Check if function has return type
                    });

                    // Cache parameters for later use
                    self.parameter_cache
                        .insert(name.clone(), parameters.clone());
                }
            }

            // ASSERTION: Validate that all public functions were added first
            debug_assert_eq!(
                self.functions.iter().take(public_count).filter(|f| f.is_public).count(),
                public_count,
                "CRITICAL INVARIANT VIOLATION: Not all public functions were added first! \
                 Expected {} public functions at indices 0..{}, but some private functions were mixed in.",
                public_count, public_count - 1
            );

            // Process private functions after (get higher indices - internal only)
            for private_function in private_functions {
                if let AstNode::InstructionDefinition {
                    name,
                    parameters,
                    visibility,
                    return_type,
                    ..
                } = private_function
                {
                    self.functions.push(FunctionInfo {
                        name: name.clone(),
                        offset: 0, // Will be patched later
                        parameter_count: parameters.len() as u8,
                        is_public: visibility.is_on_chain_callable(), // Capture visibility from AST
                        has_return_type: return_type.is_some(), // Check if function has return type
                    });

                    // Cache parameters for later use
                    self.parameter_cache
                        .insert(name.clone(), parameters.clone());
                }
            }

            // FINAL ASSERTION: Validate complete function ordering invariant
            let total_count = self.functions.len();
            debug_assert!(
                public_count + private_count == total_count,
                "Function count mismatch: public({}) + private({}) != total({})",
                public_count,
                private_count,
                total_count
            );
            debug_assert!(
                self.functions
                    .iter()
                    .skip(public_count)
                    .all(|f| !f.is_public),
                "CRITICAL INVARIANT VIOLATION: Found public function at index >= {}! \
                 All private functions must have indices {}..{}",
                public_count,
                public_count,
                total_count - 1
            );

            println!(
                "DEBUG: Function ordering validated - {} public functions (indices 0..{}), {} private functions (indices {}..{})",
                public_count, public_count.saturating_sub(1), private_count, public_count, total_count.saturating_sub(1)
            );

            let lockfile = Self::load_lockfile();

            // Process import statements
            for import_stmt in import_statements {
                if let AstNode::ImportStatement {
                    module_specifier,
                    imported_items,
                } = import_stmt
                {
                    // Extract account address or module path
                    let (account_address, is_external_import, namespace_seed) = match module_specifier {
                        crate::ast::ModuleSpecifier::External(addr) => (addr.clone(), true, None),
                        crate::ast::ModuleSpecifier::Namespace(ns) => {
                            (ns.import_key().to_string(), true, Some(ns.pda_seed_bytes()))
                        }
                        crate::ast::ModuleSpecifier::Local(name) => (name.clone(), false, None),
                        crate::ast::ModuleSpecifier::Nested(path) => (path.join("::"), false, None),
                    };

                    // Store import information for both functions and fields
                    // ARCHITECTURE: Five DSL supports importing both functions and fields
                    // Fields use LOAD_EXTERNAL_FIELD opcode for zero-copy access (read-only)
                    // Functions use CALL_EXTERNAL opcode for external function calls
                    let exports = if is_external_import {
                        lockfile
                            .as_ref()
                            .and_then(|l| l.get_exports(&account_address))
                    } else {
                        None
                    };

                    if let Some(items) = imported_items {
                        // Specific imports: use account::{function_name, field_name}
                        // Store all items as both functions and fields - usage context determines which is used
                        for item in items {
                            let (is_interface, item_name, _selectors, verify_names) =
                                Self::resolve_import_item(item, exports)?;

                            if is_interface {
                                if self.imported_functions.contains_key(&item_name)
                                    || self.imported_fields.contains_key(&item_name)
                                {
                                    return Err(VMError::InvalidScript);
                                }
                                if is_external_import {
                                    for verify_name in verify_names {
                                        if let Some(seed_bytes) = &namespace_seed {
                                            self.import_table.add_import_by_seeds(
                                                vec![seed_bytes.clone()],
                                                verify_name,
                                            );
                                        } else {
                                            self.import_table
                                                .add_import_by_address(&account_address, verify_name);
                                        }
                                    }
                                }
                                continue;
                            }

                            if self.imported_functions.contains_key(&item_name)
                                && self
                                    .imported_functions
                                    .get(&item_name)
                                    .map(|(addr, _)| addr != &account_address)
                                    .unwrap_or(false)
                            {
                                return Err(VMError::InvalidScript);
                            }

                            // Store as imported function for function calls
                            self.imported_functions.insert(
                                item_name.clone(),
                                (account_address.clone(), Some(vec![item_name.clone()])),
                            );

                            // Store as imported field for LOAD_EXTERNAL_FIELD access (read-only)
                            self.imported_fields.insert(
                                item_name.clone(),
                                (account_address.clone(), Some(vec![item_name.clone()])),
                            );

                            // Only external imports are eligible for on-chain import verification metadata.
                            if is_external_import {
                                for verify_name in verify_names {
                                    if let Some(seed_bytes) = &namespace_seed {
                                        self.import_table.add_import_by_seeds(
                                            vec![seed_bytes.clone()],
                                            verify_name,
                                        );
                                    } else {
                                        self.import_table
                                            .add_import_by_address(&account_address, verify_name);
                                    }
                                }
                            }

                            println!("DSL Compiler INFO: Imported '{}' from account {} as function/field", item_name, account_address);
                        }
                    } else {
                        // Import all functions and fields: use account
                        // This is a placeholder - in a real implementation, we'd need to
                        // load the account bytecode and parse its function/field tables
                        self.imported_functions.insert(
                            format!("__import_all_functions_{}", account_address),
                            (account_address.clone(), None),
                        );
                        self.imported_fields.insert(
                            format!("__import_all_fields_{}", account_address),
                            (account_address.clone(), None),
                        );

                        // Only external imports are eligible for on-chain import verification metadata.
                        if is_external_import {
                            if let Some(seed_bytes) = &namespace_seed {
                                self.import_table.add_import_by_seeds(
                                    vec![seed_bytes.clone()],
                                    "import_all".to_string(),
                                );
                            } else {
                                self.import_table
                                    .add_import_by_address(&account_address, "import_all".to_string());
                            }
                        }

                        println!(
                            "DSL Compiler INFO: Imported all functions and fields from account {}",
                            account_address
                        );
                    }
                }
            }
        }

        Ok(())
    }

    /// Check if a function is imported and get external call info
    pub fn get_imported_function(
        &self,
        function_name: &str,
    ) -> Option<&(String, Option<Vec<String>>)> {
        self.imported_functions.get(function_name)
    }

    /// Check if a global field is imported and get external access info
    pub fn get_imported_field(&self, field_name: &str) -> Option<&(String, Option<Vec<String>>)> {
        self.imported_fields.get(field_name)
    }

    /// Get all imported functions for account index resolution
    pub fn get_all_imported_functions(&self) -> &HashMap<String, (String, Option<Vec<String>>)> {
        &self.imported_functions
    }

    /// Get all imported fields for account index resolution
    pub fn get_all_imported_fields(&self) -> &HashMap<String, (String, Option<Vec<String>>)> {
        &self.imported_fields
    }

    /// Get the import verification table for Five bytecode accounts
    /// NEW: Returns reference to ImportTable containing address/PDA seed metadata
    pub fn get_import_table(&self) -> &ImportTable {
        &self.import_table
    }

    /// Public function entry points for compact entry-table metadata emission.
    pub fn get_public_entry_points(&self) -> &[(u8, usize)] {
        &self.public_entry_points
    }

    /// Generate function dispatch logic at the beginning of bytecode
    /// Implements a jump table to route execution to the correct function based on index
    /// Generate function dispatch logic at the beginning of bytecode
    /// Implements a jump table to route execution to the correct function based on index
    /// Generate function dispatch logic at the beginning of bytecode
    /// Implements a jump table to route execution to the correct function based on index
    #[allow(dead_code)]
    fn generate_function_dispatch_logic<T: OpcodeEmitter>(
        &mut self,
        emitter: &mut T,
        ast: &AstNode,
        _ast_generator: &mut super::ASTGenerator,
        account_system: &AccountSystem,
    ) -> Result<(), VMError> {
        // Only generate dispatcher if we have functions to dispatch
        if !self.has_callable_functions(ast) {
            return Ok(());
        }

        println!("DEBUG: Generating function dispatch logic (Jump Table)");

        // 1. Generate Dispatcher Preamble (Jump Table)
        // Checks function index (param 0) and jumps to the corresponding "Call Block"
        
        // We need to track where the jump offsets are so we can patch them 
        // to point to the Call Blocks we'll generate next.
        let mut jump_patch_locations: Vec<(usize, String, usize)> = Vec::new();

        for (i, function) in self.functions.iter().enumerate() {
            // Load function index from parameter 0 using nibble immediate to avoid LOAD_PARAM 0 rejection
            emitter.emit_opcode(five_protocol::opcodes::LOAD_PARAM_0);

            // Fast fused compare+jump path for public dispatcher checks.
            emitter.emit_opcode(five_protocol::opcodes::CMP_EQ_JUMP);
            emitter.emit_u8(i as u8);

            let patch_pos = emitter.get_position();
            jump_patch_locations.push((i, function.name.clone(), patch_pos));
            emitter.emit_u16(0xFFFF); // Placeholder absolute offset to Call Block
        }
        
        println!("DEBUG: Finished Checks Loop. Position: {}", emitter.get_position());

        // If no match found (and not init), halt or error
        // Default behavior: just halt/return if no function matches
        emitter.emit_opcode(five_protocol::opcodes::HALT);

        println!("DEBUG: Emitting Call Blocks. Position: {}", emitter.get_position());

        // 2. Generate Call Blocks
        // Each block jumps directly to the function (not CALL, to avoid call depth issues)
        self.public_entry_points.clear();
        for (function_index, name, patch_pos) in jump_patch_locations {
            let function = self.functions.iter().find(|f| f.name == name)
                .ok_or(VMError::InvalidScript)?;
            
            // Patch the JUMP_IF to point here (start of Call Block)
            let call_block_start = emitter.get_position();
            println!("DEBUG: Patching {} at {} to point to Call Block at {}", name, patch_pos, call_block_start);
            self.dispatch_jump_patches.push((patch_pos, call_block_start));
            if function.is_public {
                self.public_entry_points
                    .push((function_index as u8, call_block_start));

                // Public entry invocations already have parameters parsed into the
                // VM parameter frame by EXECUTE payload decoding. Jump directly to
                // the function body to avoid redundant stack shuffling + CALL frame setup.
                emitter.emit_opcode(five_protocol::opcodes::JUMP);
                let jump_offset_pos = emitter.get_position();
                self.dispatch_patch_locations
                    .insert(function.name.clone(), jump_offset_pos);
                emitter.emit_u16(0xFFFF); // Placeholder for function offset
                continue;
            }

            // For functions with parameters, we need to move them from the input parameters
            // Jump directly to the function body to preserve parameter indexing
            // in the original execution context.
            
            // Retrieve parameters from cache
            let function_parameters = &self.parameter_cache[&name];

            // Stack mode: LOAD_PARAM + CALL.
            let mut actual_data_count: u8 = 0;
            let mut data_param_index: u8 = 0;

            for param in function_parameters.iter() {
                let is_account = super::account_utils::is_account_parameter(
                    &param.param_type,
                    &param.attributes,
                    Some(account_system.get_account_registry()),
                );
                if is_account { continue; }

                actual_data_count += 1;
                data_param_index += 1;

                // Use optimized opcodes if possible
                match data_param_index {
                    1 => emitter.emit_opcode(five_protocol::opcodes::LOAD_PARAM_1),
                    2 => emitter.emit_opcode(five_protocol::opcodes::LOAD_PARAM_2),
                    3 => emitter.emit_opcode(five_protocol::opcodes::LOAD_PARAM_3),
                    _ => {
                        emitter.emit_opcode(five_protocol::opcodes::LOAD_PARAM);
                        emitter.emit_u8(data_param_index);
                    }
                }
            }

            emitter.emit_opcode(five_protocol::opcodes::CALL);
            emitter.emit_u8(actual_data_count);

            let call_offset_pos = emitter.get_position();
            self.dispatch_patch_locations.insert(function.name.clone(), call_offset_pos);
            emitter.emit_u16(0xFFFF); // Placeholder for function offset

            // HALT after return
            emitter.emit_opcode(five_protocol::opcodes::HALT);

        }

        Ok(())
    }

    /// Patch the dispatch logic with actual function offsets
    pub fn patch_dispatch_logic_with_base(
        &self,
        emitter: &mut impl OpcodeEmitter,
        base_offset: usize,
    ) -> Result<(), VMError> {
        for (patch_pos, target_pos) in &self.dispatch_jump_patches {
            let absolute_target = target_pos
                .checked_add(base_offset)
                .ok_or(VMError::InvalidInstructionPointer)?;
            if absolute_target > five_protocol::MAX_U16_ADDRESS {
                return Err(VMError::InvalidInstructionPointer);
            }
            emitter.patch_u16(*patch_pos, absolute_target as u16);
        }

        for (name, patch_pos) in &self.dispatch_patch_locations {
            let function = self.functions.iter().find(|f| f.name == *name)
                .ok_or(VMError::InvalidScript)?;

            let absolute_target = function
                .offset
                .checked_add(base_offset)
                .ok_or(VMError::InvalidInstructionPointer)?;
            if absolute_target > five_protocol::MAX_U16_ADDRESS {
                return Err(VMError::InvalidFunctionIndex);
            }
            emitter.patch_u16(*patch_pos, absolute_target as u16);
        }
        Ok(())
    }

    /// Generate bytecode for all user-defined instruction definitions.
    /// Note: Functions are now called directly via dispatch logic or AST generator patterns.
    fn generate_function_bodies<T: OpcodeEmitter>(
        &mut self,
        emitter: &mut T,
        ast: &AstNode,
        account_system: &mut AccountSystem,
        scope_analyzer: &mut scope_analyzer::ScopeAnalyzer,
        ast_generator: &mut super::ASTGenerator,
        symbol_table: &HashMap<String, FieldInfo>,
    ) -> Result<(), VMError> {
        if let AstNode::Program {
            instruction_definitions,
            import_statements,
            init_block,
            ..
        } = ast
        {
            // Register external imports for AST function-call emission.
            ast_generator.external_imports.clear();
            let lockfile = Self::load_lockfile();
            let mut external_import_index: u8 = 0;
            for import_stmt in import_statements {
                let AstNode::ImportStatement {
                    module_specifier,
                    imported_items,
                } = import_stmt
                else {
                    continue;
                };
                let (address, _namespace_seed, local_module_alias, local_module_path, is_external_import) = match module_specifier {
                    crate::ast::ModuleSpecifier::External(address) => {
                        (address.clone(), None, None, None, true)
                    }
                    crate::ast::ModuleSpecifier::Namespace(ns) => {
                        (ns.import_key().to_string(), Some(ns.pda_seed_bytes()), None, None, true)
                    }
                    crate::ast::ModuleSpecifier::Local(name) => {
                        (name.clone(), None, Some(name.clone()), Some(name.clone()), false)
                    }
                    crate::ast::ModuleSpecifier::Nested(path) if !path.is_empty() => {
                        let full = path.join("::");
                        let alias = path[path.len() - 1].clone();
                        (full.clone(), None, Some(alias), Some(full), false)
                    }
                    _ => continue,
                };

                let exports = lockfile
                    .as_ref()
                    .and_then(|l| l.get_exports(&address));
                let mut selectors = HashMap::new();
                let mut allow_any_function = imported_items.is_none();
                if let Some(items) = imported_items {
                    for item in items {
                        let (is_interface, item_name, interface_selectors, _) =
                            Self::resolve_import_item(item, exports)?;
                        if is_interface {
                            if let Some(alias) = &local_module_alias {
                                ast_generator.register_module_interface_alias(
                                    alias.clone(),
                                    item_name.clone(),
                                );
                            }
                            if let Some(full) = &local_module_path {
                                ast_generator.register_module_interface_alias(
                                    full.clone(),
                                    item_name.clone(),
                                );
                            }
                            if is_external_import {
                                let allow_any = interface_selectors.is_empty();
                                ast_generator.register_external_import(
                                    item_name,
                                    external_import_index,
                                    allow_any,
                                    interface_selectors,
                                );
                            }
                        } else {
                            if is_external_import {
                                selectors.insert(item_name.clone(), Self::external_selector(&item_name));
                            }
                        }
                    }
                    if selectors.is_empty() {
                        allow_any_function = true;
                    }
                }

                if is_external_import {
                    let mut keys = Vec::new();
                    if Self::is_valid_identifier(&address) {
                        keys.push(address.clone());
                    }
                    keys.push(format!("ext{}", external_import_index));

                    for key in keys {
                        ast_generator.register_external_import(
                            key,
                            external_import_index,
                            allow_any_function,
                            selectors.clone(),
                        );
                    }
                }

                if let (Some(alias), Some(full_path)) = (&local_module_alias, &local_module_path) {
                    if let Some(interface_name) =
                        ast_generator.find_interface_for_module_alias(alias)
                    {
                        ast_generator
                            .register_module_interface_alias(alias.clone(), interface_name.clone());
                        ast_generator
                            .register_module_interface_alias(full_path.clone(), interface_name);
                    }
                }

                if is_external_import {
                    external_import_index = external_import_index.saturating_add(1);
                }
            }

            // Generate init block first if present
            if let Some(init) = init_block {
                let init_offset = emitter.get_position();
                self.update_function_offset("__init", init_offset)?;

                // Record function position in AST generator for CALL patching
                ast_generator.record_function_position(emitter, "__init".to_string());

                // Generate init block body (init blocks are always void)
                self.generate_single_function_body(
                    emitter,
                    "__init",
                    &[],
                    &None,
                    init,
                    account_system,
                    scope_analyzer,
                    ast_generator,
                    symbol_table,
                )?;
            }

            // Generate instruction definition bodies
            // Sort functions: public functions first, then private functions
            // Phase 2: Visibility-based ordering for proper function dispatch
            let mut sorted_functions: Vec<&AstNode> = instruction_definitions.iter().collect();
            sorted_functions.sort_by(|a, b| {
                let (a_is_public, a_params) = if let AstNode::InstructionDefinition {
                    visibility,
                    parameters,
                    ..
                } = a
                {
                    (visibility.is_on_chain_callable(), parameters.len())
                } else {
                    (false, 999)
                };
                let (b_is_public, b_params) = if let AstNode::InstructionDefinition {
                    visibility,
                    parameters,
                    ..
                } = b
                {
                    (visibility.is_on_chain_callable(), parameters.len())
                } else {
                    (false, 999)
                };

                // Primary sort: public functions first (true sorts before false)
                match b_is_public.cmp(&a_is_public) {
                    std::cmp::Ordering::Equal => {
                        // Secondary sort: within same visibility, fewer parameters first
                        a_params.cmp(&b_params)
                    }
                    other => other,
                }
            });

            for instruction_def in sorted_functions {
                if let AstNode::InstructionDefinition {
                    name,
                    parameters,
                    return_type,
                    body,
                    ..
                } = instruction_def
                {
                    println!("DEBUG: Processing instruction definition: {}", name);
                    // Function offset will be recorded inside generate_single_function_body
                    // before ALLOC_LOCALS is emitted

                    // Record function position in AST generator for CALL patching
                    ast_generator.record_function_position(emitter, name.clone());

                    // Generate function body with parameters and return type
                    println!("DEBUG: About to generate function body for: {}", name);
                    self.generate_single_function_body(
                        emitter,
                        name,
                        parameters,
                        return_type,
                        body,
                        account_system,
                        scope_analyzer,
                        ast_generator,
                        symbol_table,
                    )?;
                    println!("DEBUG: Completed function body generation for: {}", name);
                }
            }
        }

        Ok(())
    }

    /// Generate a single function body with scope analysis and parameter handling
    fn generate_single_function_body<T: OpcodeEmitter>(
        &mut self,
        emitter: &mut T,
        function_name: &str,
        parameters: &[InstructionParameter],
        return_type: &Option<Box<TypeNode>>, // Changed from &Option<TypeNode>
        body: &AstNode,
        account_system: &mut AccountSystem,
        scope_analyzer: &mut scope_analyzer::ScopeAnalyzer,
        ast_generator: &mut super::ASTGenerator,
        symbol_table: &HashMap<String, FieldInfo>,
    ) -> Result<(), VMError> {
        // Begin function scope analysis
        scope_analyzer.begin_function(function_name.to_string())?;

        // Set up account parameter tracking
        self.setup_account_parameters(parameters, account_system)?;

        // Add parameters to scope analysis
        for param in parameters {
            let param_type = self.type_node_to_string(&param.param_type);
            scope_analyzer.declare_variable(&param.name, &param_type, true)?;
        }

        // Analyze function scope
        scope_analyzer.analyze_node(body)?;

        // Generate parameter loading code
        // ... (existing logging code omitted for brevity)
        self.generate_parameter_loading(emitter, parameters)?;

        // Enforce account constraints (@signer, @has_one, etc.)
        // This must run after parameters are loaded (though current implementation uses account indices directly)
        super::constraint_enforcer::emit_constraint_checks(
            emitter,
            parameters,
            account_system.get_account_registry(),
        )?;

        // Record function start position BEFORE ALLOC_LOCALS
        // The CALL instruction should jump to the start of the function, including ALLOC_LOCALS
        let function_offset = emitter.get_position();
        self.update_function_offset(&function_name, function_offset)?;

        // Connect ScopeAnalyzer optimizations to ASTGenerator
        let allocations_vec = scope_analyzer.optimize_local_slots(function_name)?;
        let mut allocations_map = HashMap::new();
        let mut max_local_index: i32 = -1;
        for (name, slot) in allocations_vec {
            if slot as i32 > max_local_index {
                max_local_index = slot as i32;
            }
            allocations_map.insert(name, slot);
        }

        // Emit ALLOC_LOCALS to reserve space for local variables
        // If max_local_index is -1, we need 0 locals. Otherwise max_index + 1.
        let required_locals = (max_local_index + 1) as u8;
        if required_locals > 0 {
             // 0xA0 is ALLOC_LOCALS in five_protocol::opcodes
             // We can use the constant if imported, or just use the opcode value with a comment
             // ALLOC_LOCALS is imported via `use super::opcodes::*;`? No, `use five_protocol::opcodes::*;` is not fully visible here?
             // function_dispatch.rs imports `five_protocol` types but maybe not all opcodes.
             // It uses `five_protocol::opcodes::LOAD_PARAM` etc explicitly.
             emitter.emit_opcode(five_protocol::opcodes::ALLOC_LOCALS);
             emitter.emit_u8(required_locals);
             println!("DEBUG: Emitting ALLOC_LOCALS {} for function {}", required_locals, function_name);
        }
        
        // Setup AST generator
        ast_generator.set_precomputed_allocations(allocations_map);
        ast_generator.field_counter = required_locals as u32; // Reset local slot counter to next available slot (or keep it if we want to reuse?)
        // Actually field_counter should probably track the count to avoid overwriting if new locals are added dynamically (e.g. temps)
        // But ScopeAnalyzer should have caught all variables. Temps (like in array literal) use field_counter.
        // So field_counter should start at required_locals.

        ast_generator.local_symbol_table.clear(); // Clear previous locals

        // Set up AST generator similar to the working direct path
        ast_generator.set_symbol_table(symbol_table.clone());

        // Use the shared AccountSystem that already has account type registrations
        ast_generator.set_account_system(account_system.clone());

        // Set the function context for proper init block handling
        ast_generator.set_function_context(Some(function_name.to_string()));

        // Set function parameters for payer resolution in @init constraints
        ast_generator.current_function_parameters = Some(parameters.to_vec());

        // Add function parameters to the main AST generator's symbol table
        // CRITICAL FIX: Dual indexing strategy for VM's separate arrays:
        // - accounts[] array: accessed by STORE_FIELD, GET_KEY - uses 0-based account-only index
        // - params[] array: accessed by LOAD_PARAM - uses 1-based unified index
        //
        // For account params: offset = account position (0-based among accounts only)
        // For data params: offset = position in params array (1-based in bytecode semantics)

        // CALL-BASED DISPATCH indexing:
        // After CALL, the new frame has pushed data params at indices 1..N.
        // The function body uses LOAD_PARAM with DATA-ONLY indices.
        //
        // - Account params: offset = position in accounts[] (0-based), is_parameter=false
        // - Data params: offset = data_counter (0-based), is_parameter=true
        //   LOAD_PARAM uses (offset+1) to access new frame's params array
        //
        // Example for init_mint(mint_account, authority, freeze_auth, decimals, ...):
        // - mint_account: account_offset=0, is_parameter=false
        // - authority: account_offset=1, is_parameter=false
        // - freeze_authority: offset=0, is_parameter=true → LOAD_PARAM 1
        // - decimals: offset=1, is_parameter=true → LOAD_PARAM 2  ← Correct!
        
        let mut account_counter: u32 = 0;
        let mut data_counter: u32 = 0;

        // Count data parameters to ensure field_counter starts after them
        let total_data_params: u32 = parameters
            .iter()
            .filter(|p| !super::account_utils::is_account_parameter(
                &p.param_type,
                &p.attributes,
                Some(account_system.get_account_registry())
            ))
            .count() as u32;

        for (_index, param) in parameters.iter().enumerate() {
            let param_type = self.type_node_to_string(&param.param_type);

            let is_account = super::account_utils::is_account_parameter(
                &param.param_type,
                &param.attributes,
                Some(account_system.get_account_registry())
            );

            // Determine offset and access pattern
            let (offset, is_parameter) = if is_account {
                // Accounts: use account-specific index for STORE_FIELD/GET_KEY
                let acc_off = account_counter;
                account_counter += 1;
                (acc_off, false)
            } else {
                // Data params: use DATA-ONLY counter
                // After CALL, new frame has data params at consecutive indices 1..N
                let data_off = data_counter;
                data_counter += 1;
                (data_off, true)
            };

            let field_info = super::types::FieldInfo {
                offset,
                field_type: param_type,
                // Implicit mutability: @init implies mutable, or explicit @mut
                is_mutable: param.is_init || param.attributes.iter().any(|a| a.name == "mut"),
                is_optional: param.is_optional,
                is_parameter,       // Account params use account access, data params use LOAD_PARAM
            };
            ast_generator.add_parameter_to_symbol_table(param.name.clone(), field_info);
        }

        // CRITICAL FIX: Ensure field_counter doesn't collide with data parameters
        // Data parameters are allocated offsets 0..(total_data_params-1)
        // So new locals should start at total_data_params
        ast_generator.field_counter = std::cmp::max(ast_generator.field_counter, total_data_params);

        // Add function start label for jumps
        // Note: Function offset already recorded before ALLOC_LOCALS
        ast_generator.record_function_position(emitter, function_name.to_string());

        // Generate account initialization sequences AFTER adding all parameters to the symbol table.
        // This ensures that seeds for one account can reference other account parameters.
        for (index, param) in parameters.iter().enumerate() {
            ast_generator.generate_init_account_sequence(emitter, param, index)?;
        }

        ast_generator.emit_pda_param_setup(emitter, parameters)?;

        // Inject @requires(condition) checks
        for param in parameters {
            for attr in &param.attributes {
                if attr.name == "requires" {
                    if let Some(condition) = attr.args.first() {
                        println!("DEBUG: Generating require statement for condition in function_dispatch: {:?}", condition);
                        ast_generator.generate_ast_node(emitter, &AstNode::RequireStatement { 
                            condition: Box::new(condition.clone()) 
                        })?;
                    }
                }
            }
        }

        println!(
            "DEBUG: About to generate AST node for function: {}",
            function_name
        );
        match ast_generator.generate_ast_node(emitter, body) {
            Ok(()) => {
                println!(
                    "DEBUG: Completed AST node generation for function: {}",
                    function_name
                );
            }
            Err(e) => {
                println!(
                    "ERROR: AST generation failed for function {}: {:?}",
                    function_name, e
                );
                return Err(e);
            }
        }

        // Emit RETURN opcode only for void functions
        // Functions with return types should use RETURN_VALUE from their explicit return statements
        if return_type.is_none() {
            println!(
                "DEBUG: Emitting RETURN opcode for void function: {}",
                function_name
            );
            // Emit RETURN for void functions (dispatcher now uses HALT, no need for zero-push)
            emitter.emit_opcode(five_protocol::opcodes::RETURN);
        } else {
            println!(
                "DEBUG: Function {} has return type, using RETURN_VALUE from explicit return statement",
                function_name
            );
        }

        // Clear function context to avoid leaking to other functions
        ast_generator.set_function_context(None);
        ast_generator.current_function_parameters = None;
        ast_generator.set_precomputed_allocations(HashMap::new()); // Clear/Reset allocations

        // End function scope analysis
        scope_analyzer.end_function()?;

        Ok(())
    }

    /// Set up account parameter tracking for field access
    fn setup_account_parameters(
        &mut self,
        parameters: &[InstructionParameter],
        account_system: &mut AccountSystem,
    ) -> Result<(), VMError> {
        self.current_function_params.clear();

        for param in parameters {
            let type_name = self.type_node_to_string(&param.param_type);

            if account_system.is_account_type(&type_name) {
                self.current_function_params
                    .insert(param.name.clone(), (type_name, param.attributes.iter().map(|a| a.name.clone()).collect()));
            }
        }

        Ok(())
    }

    /// Generate parameter loading code - FIXED to avoid unnecessary local variable storage
    fn generate_parameter_loading<T: OpcodeEmitter>(
        &self,
        _emitter: &mut T,
        parameters: &[InstructionParameter],
    ) -> Result<(), VMError> {
        println!(
            "DEBUG: generate_parameter_loading called with {} parameters",
            parameters.len()
        );

        // ARCHITECTURAL FIX: Don't generate any parameter loading opcodes here
        // Parameters are already available via ctx.parameters[] in MitoVM
        // The AST generator will emit LOAD_PARAM directly when parameters are accessed

        // REMOVED: ALLOC_LOCALS, LOAD_PARAM, SET_LOCAL sequence
        // This was causing inefficient LOAD_PARAM → SET_LOCAL → GET_LOCAL chain

        println!("DEBUG: generate_parameter_loading completed (no opcodes emitted - direct access model)");
        Ok(())
    }

    /// Update function offset for a specific function.
    /// Note: Offset patching is no longer needed - JUMP_TABLE complexity removed.
    /// Functions are now handled directly by the AST generator or called via CALL opcodes.
    fn update_function_offset(
        &mut self,
        function_name: &str,
        offset: usize,
    ) -> Result<(), VMError> {
        for function in &mut self.functions {
            if function.name == function_name {
                function.offset = offset;
                return Ok(());
            }
        }
        Err(VMError::InvalidScript) // Function not found
    }

    /// Get function information
    pub fn get_function_info(&self, function_name: &str) -> Option<&FunctionInfo> {
        self.functions.iter().find(|f| f.name == function_name)
    }

    /// Get all functions
    pub fn get_functions(&self) -> &[FunctionInfo] {
        &self.functions
    }

    /// Get current function parameters
    pub fn get_current_function_params(&self) -> &HashMap<String, (String, Vec<String>)> {
        &self.current_function_params
    }

    /// Check if a parameter is an account parameter
    pub fn is_account_parameter(&self, param_name: &str) -> bool {
        self.current_function_params.contains_key(param_name)
    }

    /// Get account type for parameter
    pub fn get_account_type(&self, param_name: &str) -> Option<&str> {
        self.current_function_params
            .get(param_name)
            .map(|(account_type, _)| account_type.as_str())
    }

    /// Helper: Convert TypeNode to string representation
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

    /// Generate optimization report (simplified for metadata-only approach)
    pub fn generate_optimization_report(&self) -> String {
        let mut report = String::new();
        report.push_str("Function Metadata Report\n");
        report.push_str("========================\n\n");

        report.push_str(&format!("Total functions: {}\n", self.functions.len()));
        report.push_str("Dispatch method: Direct function calls (JUMP_TABLE removed)\n");

        for (index, function) in self.functions.iter().enumerate() {
            report.push_str(&format!(
                "\nFunction {}: {}\n  Offset: {}\n  Parameters: {}\n",
                index, function.name, function.offset, function.parameter_count
            ));

            if let Some(params) = self.parameter_cache.get(&function.name) {
                for param in params {
                    let param_type = self.type_node_to_string(&param.param_type);
                    report.push_str(&format!(
                        "    - {} ({}): {:?}\n",
                        param.name, param_type, param.attributes
                    ));
                }
            }
        }

        report
    }
}

impl Default for FunctionDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Extension methods for the main DslBytecodeGenerator
impl super::DslBytecodeGenerator {
    /// Generate functions using the function dispatcher
    pub fn generate_with_function_dispatcher(
        &mut self,
        ast: &AstNode,
        symbol_table: &mut std::collections::HashMap<String, types::FieldInfo>,
    ) -> Result<(), VMError> {
        let mut dispatcher = FunctionDispatcher::new();
        let mut account_system = AccountSystem::new();
        let mut scope_analyzer = scope_analyzer::ScopeAnalyzer::new();
        // v2_preview disabled by default for production stability
        // Use ASTGenerator::with_v2_preview(true) to enable field access optimizations
        let mut ast_generator = super::ASTGenerator::new();

        if dispatcher.has_callable_functions(ast) {
            dispatcher.generate_dispatcher(
                self,
                ast,
                &mut account_system,
                &mut scope_analyzer,
                &mut ast_generator,
                symbol_table,
            )?;
        }

        Ok(())
    }

    /// Check if AST has callable functions
    pub fn has_callable_functions(&self, ast: &AstNode) -> bool {
        let dispatcher = FunctionDispatcher::new();
        dispatcher.has_callable_functions(ast)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{AstNode, BlockKind, InstructionParameter, TypeNode};
    use crate::config::workspace::{ExportMetadata, InterfaceExport};

    #[test]
    fn test_function_dispatcher_creation() {
        let dispatcher = FunctionDispatcher::new();
        assert_eq!(dispatcher.functions.len(), 0);
    }

    #[test]
    fn test_callable_functions_detection() {
        let dispatcher = FunctionDispatcher::new();

        // Program with instruction definitions
        let ast_with_functions = AstNode::Program {
            program_name: "test".to_string(),
            field_definitions: vec![],
            instruction_definitions: vec![AstNode::InstructionDefinition {
                name: "test_func".to_string(),
                visibility: crate::Visibility::Public,
                is_public: true,
                parameters: vec![],
                return_type: None,
                body: Box::new(AstNode::Block {
                    statements: vec![],
                    kind: BlockKind::Regular,
                }),
            }],
            init_block: None,
            constraints_block: None,
            event_definitions: vec![],
            account_definitions: vec![],
            interface_definitions: vec![],
            import_statements: vec![],
        };

        assert!(dispatcher.has_callable_functions(&ast_with_functions));

        // Program without functions
        let ast_without_functions = AstNode::Program {
            program_name: "test".to_string(),
            field_definitions: vec![],
            instruction_definitions: vec![],
            init_block: None,
            constraints_block: None,
            event_definitions: vec![],
            account_definitions: vec![],
            interface_definitions: vec![],
            import_statements: vec![],
        };

        assert!(!dispatcher.has_callable_functions(&ast_without_functions));
    }

    #[test]
    fn test_function_info_collection() {
        let mut dispatcher = FunctionDispatcher::new();

        let ast = AstNode::Program {
            program_name: "test".to_string(),
            field_definitions: vec![],
            instruction_definitions: vec![
                AstNode::InstructionDefinition {
                    name: "func1".to_string(),
                    visibility: crate::Visibility::Public,
                    is_public: true,
                    parameters: vec![InstructionParameter {
                        name: "param1".to_string(),
                        param_type: TypeNode::Primitive("u64".to_string()),
                        is_optional: false,
                        default_value: None,
                        attributes: vec![],
                        is_init: false,
                        init_config: None,
                    pda_config: None,
                    }],
                    return_type: None,
                    body: Box::new(AstNode::Block {
                        statements: vec![],
                        kind: BlockKind::Regular,
                    }),
                },
                AstNode::InstructionDefinition {
                    name: "func2".to_string(),
                    visibility: crate::Visibility::Public,
                    is_public: true,
                    parameters: vec![],
                    return_type: None,
                    body: Box::new(AstNode::Block {
                        statements: vec![],
                        kind: BlockKind::Regular,
                    }),
                },
            ],
            init_block: Some(Box::new(AstNode::Block {
                statements: vec![],
                kind: BlockKind::Init,
            })),
            constraints_block: None,
            event_definitions: vec![],
            account_definitions: vec![],
            interface_definitions: vec![],
            import_statements: vec![],
        };

        dispatcher.collect_function_info(&ast).unwrap();

        assert_eq!(dispatcher.functions.len(), 3); // init + 2 functions
        assert_eq!(dispatcher.functions[0].name, "__init");
        assert_eq!(dispatcher.functions[1].name, "func1");
        assert_eq!(dispatcher.functions[2].name, "func2");
        assert_eq!(dispatcher.functions[1].parameter_count, 1);
        assert_eq!(dispatcher.functions[2].parameter_count, 0);
    }

    #[test]
    fn test_account_parameter_detection() {
        let mut dispatcher = FunctionDispatcher::new();
        let mut account_system = AccountSystem::new();

        let parameters = vec![
            InstructionParameter {
                name: "signer".to_string(),
                param_type: TypeNode::Primitive("Account".to_string()),
                is_optional: false,
                default_value: None,
                attributes: vec![crate::ast::Attribute { name: "signer".to_string(), args: vec![] }],
                is_init: false,
                init_config: None,
                    pda_config: None,
            },
            InstructionParameter {
                name: "amount".to_string(),
                param_type: TypeNode::Primitive("u64".to_string()),
                is_optional: false,
                default_value: None,
                attributes: vec![],
                is_init: false,
                init_config: None,
                    pda_config: None,
            },
        ];

        dispatcher
            .setup_account_parameters(&parameters, &mut account_system)
            .unwrap();

        assert!(dispatcher.is_account_parameter("signer"));
        assert!(!dispatcher.is_account_parameter("amount"));
        assert_eq!(dispatcher.get_account_type("signer"), Some("Account"));
    }

    /// Test import_table is initialized and populated correctly
    #[test]
    fn test_import_table_initialization() {
        let dispatcher = FunctionDispatcher::new();
        let table = dispatcher.get_import_table();

        // Empty table should be empty
        assert!(table.is_empty());
        assert_eq!(table.len(), 0);
    }

    /// Test import_table is populated when imports are registered
    #[test]
    fn test_import_table_populated_on_import() {
        let mut dispatcher = FunctionDispatcher::new();

        // Simulate import registration via populate_import_table method
        // We'll directly add an import entry to the import_table
        let address = "11111111111111111111111111111111";
        dispatcher.import_table.add_import_by_address(address, "external_func".to_string());

        let table = dispatcher.get_import_table();
        assert!(!table.is_empty());
        assert_eq!(table.len(), 1);

        // Verify serialization works
        let serialized = table.serialize().expect("serialize");
        assert!(!serialized.is_empty());
        assert_eq!(serialized[0], 1); // import_count = 1
    }

    /// Test multiple imports in import_table
    #[test]
    fn test_import_table_multiple_entries() {
        let mut dispatcher = FunctionDispatcher::new();

        // Add multiple imports
        dispatcher.import_table.add_import_by_address(
            "11111111111111111111111111111111",
            "func1".to_string(),
        );
        dispatcher.import_table.add_import_by_address(
            "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "func2".to_string(),
        );
        dispatcher.import_table.add_import_by_seeds(
            vec![b"vault".to_vec(), b"user".to_vec()],
            "pda_func".to_string(),
        );

        let table = dispatcher.get_import_table();
        assert_eq!(table.len(), 3);

        // Verify serialization includes all 3 imports
        let serialized = table.serialize().expect("serialize");
        assert_eq!(serialized[0], 3); // import_count = 3
    }

    #[test]
    fn test_unqualified_import_classifies_to_interface_when_exported() {
        let exports = ExportMetadata {
            methods: vec!["transfer".to_string()],
            interfaces: vec![InterfaceExport {
                name: "TokenOps".to_string(),
                method_map: std::collections::HashMap::from([(
                    "transfer".to_string(),
                    "transfer_checked".to_string(),
                )]),
            }],
        };

        let item = ImportItem::Unqualified("TokenOps".to_string());
        let (is_interface, name, selectors, verify_names) =
            FunctionDispatcher::resolve_import_item(&item, Some(&exports)).unwrap();

        assert!(is_interface);
        assert_eq!(name, "TokenOps");
        assert_eq!(selectors.get("transfer"), Some(&FunctionDispatcher::external_selector("transfer_checked")));
        assert_eq!(verify_names, vec!["transfer_checked".to_string()]);
    }

    #[test]
    fn test_unqualified_import_rejects_ambiguous_symbol() {
        let exports = ExportMetadata {
            methods: vec!["TokenOps".to_string()],
            interfaces: vec![InterfaceExport {
                name: "TokenOps".to_string(),
                method_map: std::collections::HashMap::new(),
            }],
        };
        let item = ImportItem::Unqualified("TokenOps".to_string());
        assert!(FunctionDispatcher::resolve_import_item(&item, Some(&exports)).is_err());
    }

    #[test]
    fn test_unqualified_import_rejects_missing_symbol_when_exports_known() {
        let exports = ExportMetadata {
            methods: vec!["transfer".to_string()],
            interfaces: vec![InterfaceExport {
                name: "TokenOps".to_string(),
                method_map: std::collections::HashMap::new(),
            }],
        };
        let item = ImportItem::Unqualified("missing".to_string());
        assert!(FunctionDispatcher::resolve_import_item(&item, Some(&exports)).is_err());
    }

    #[test]
    fn test_explicit_method_or_interface_rejects_type_mismatch_against_exports() {
        let exports = ExportMetadata {
            methods: vec!["transfer".to_string()],
            interfaces: vec![InterfaceExport {
                name: "TokenOps".to_string(),
                method_map: std::collections::HashMap::new(),
            }],
        };

        let method_item = ImportItem::Method("TokenOps".to_string());
        assert!(FunctionDispatcher::resolve_import_item(&method_item, Some(&exports)).is_err());

        let interface_item = ImportItem::Interface("transfer".to_string());
        assert!(FunctionDispatcher::resolve_import_item(&interface_item, Some(&exports)).is_err());
    }

    /// Test import_table with PDA seeds
    #[test]
    fn test_import_table_pda_seeds() {
        let mut dispatcher = FunctionDispatcher::new();

        let seeds = vec![b"vault".to_vec(), b"user".to_vec()];
        dispatcher.import_table.add_import_by_seeds(seeds, "get_vault".to_string());

        let table = dispatcher.get_import_table();
        assert_eq!(table.len(), 1);

        // Verify serialization format
        let serialized = table.serialize().expect("serialize");
        assert_eq!(serialized[0], 1); // import_count = 1
        assert_eq!(serialized[1], 1); // import_type = 1 (PDA seeds)
        assert_eq!(serialized[2], 2); // seed_count = 2
    }
}
