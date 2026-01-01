//! Initialization and configuration methods for ASTGenerator
//!
//! This module contains constructors and configuration methods for setting up
//! the AST generator with various options and state management.

use super::super::account_system::AccountSystem;
use super::types::ASTGenerator;
use crate::ast::AstNode;
use crate::ast::TypeNode;
use crate::type_checker::{InterfaceInfo, InterfaceMethod, InterfaceSerializer};
use five_vm_mito::error::VMError;
use std::collections::HashMap;

impl ASTGenerator {
    /// Internal constructor with configurable v2_preview flag
    ///
    /// This eliminates duplication between the three public constructors.
    fn new_internal(v2_preview: bool) -> Self {
        Self {
            global_symbol_table: HashMap::new(),
            local_symbol_table: HashMap::new(),
            type_cache: HashMap::new(),
            expression_depth: 0,
            loop_stack: Vec::new(),
            field_counter: 0,
            account_system: None,
            current_function_context: None,
            current_function_return_type: None,
            jump_patches: Vec::new(),
            br_eq_u8_patches: Vec::new(),
            function_patches: Vec::new(),
            function_positions: HashMap::new(),
            label_positions: HashMap::new(),
            label_counter: 0,
            interface_registry: HashMap::new(),
            v2_preview,
            // Resource tracking initialization
            max_locals_used: 0,
            max_stack_depth_seen: 0,
            current_call_depth: 0,
            max_call_depth_seen: 1, // At least one level (entry function)
            string_literals_count: 0,
            estimated_temp_usage: 64, // Default minimum
            function_call_count: 0,
            name_deduplication: super::super::types::NameDeduplication::new(),
            precomputed_allocations: None,
            // External imports for CALL_EXTERNAL generation
            external_imports: HashMap::new(),
        }
    }

    /// Create a new AST generator with V1 optimizations (default)
    pub fn new() -> Self {
        Self::new_internal(false)
    }

    /// Create a new AST generator with v2-preview mode
    pub fn with_v2_preview(v2_preview: bool) -> Self {
        Self::new_internal(v2_preview)
    }

    /// Create a new AST generator with specific optimization level
    pub fn with_optimization_level(optimization_level: crate::compiler::OptimizationLevel) -> Self {
        use crate::compiler::OptimizationLevel;

        let v2_preview = match optimization_level {
            OptimizationLevel::Production => true,
            _ => false,
        };

        Self::new_internal(v2_preview)
    }

    /// Reset the generator state for new compilation
    pub fn reset(&mut self) {
        self.global_symbol_table.clear();
        self.local_symbol_table.clear();
        self.type_cache.clear();
        self.expression_depth = 0;
        self.loop_stack.clear();
        self.field_counter = 0;
        self.account_system = None;
        self.jump_patches.clear();
        self.br_eq_u8_patches.clear();
        self.function_patches.clear();
        self.function_positions.clear();
        self.label_positions.clear();
        self.label_counter = 0;
        self.interface_registry.clear();
        self.name_deduplication = super::super::types::NameDeduplication::new();
        self.external_imports.clear();
        // Reset resource tracking
        self.reset_resource_tracking();
    }

    /// Set the account system for proper field offset resolution
    pub fn set_account_system(&mut self, account_system: AccountSystem) {
        self.account_system = Some(account_system);
    }

    /// Set precomputed variable allocations from ScopeAnalyzer
    pub fn set_precomputed_allocations(&mut self, allocations: HashMap<String, usize>) {
        self.precomputed_allocations = Some(allocations);
    }

    /// Set the function dispatcher for handling imported functions
    pub fn set_function_dispatcher(
        &mut self,
        _dispatcher: super::super::function_dispatch::FunctionDispatcher,
    ) {
        // Function dispatcher removed per user request
    }

    /// Register an external import for CALL_EXTERNAL generation
    /// 
    /// When a function from this module is called, CALL_EXTERNAL will be emitted
    /// instead of a regular CALL opcode.
    pub fn register_external_import(
        &mut self,
        module_name: String,
        account_index: u8,
        functions: HashMap<String, u16>,
    ) {
        use super::types::ExternalImport;
        self.external_imports.insert(
            module_name.clone(),
            ExternalImport {
                module_name,
                account_index,
                functions,
            },
        );
    }

    /// Check if a module is registered as external import
    pub fn is_external_import(&self, module_name: &str) -> bool {
        self.external_imports.contains_key(module_name)
    }

    /// Get external import info for a module
    pub fn get_external_import(&self, module_name: &str) -> Option<&super::types::ExternalImport> {
        self.external_imports.get(module_name)
    }

    /// Process interface definitions and populate the registry
    pub fn process_interface_definitions(
        &mut self,
        interface_definitions: &[AstNode],
    ) -> Result<(), VMError> {
        for interface_def in interface_definitions {
            if let AstNode::InterfaceDefinition {
                name,
                program_id,
                serializer,
                functions,
            } = interface_def
            {
                let mut methods = HashMap::new();

                for function_def in functions {
                    if let AstNode::InterfaceFunction {
                        name: method_name,
                        parameters,
                        return_type,
                        discriminator,
                        discriminator_bytes,
                    } = function_def
                    {
                        // Convert InstructionParameter to TypeNode for storage
                        let param_types: Vec<TypeNode> = parameters
                            .iter()
                            .map(|param| param.param_type.clone())
                            .collect();

                        let return_type_node = return_type.as_ref().map(|rt| (**rt).clone());

                        methods.insert(
                            method_name.clone(),
                            InterfaceMethod {
                                discriminator: discriminator.unwrap_or(0), // Default to 0 if no discriminator
                                discriminator_bytes: discriminator_bytes.clone(),
                                parameters: param_types,
                                return_type: return_type_node,
                            },
                        );
                    }
                }

                let interface_info = InterfaceInfo {
                    program_id: program_id.clone().unwrap_or_default(), // Default to empty if no program ID
                    serializer: match serializer
                        .as_ref()
                        .map(|s| s.as_str())
                    {
                        None => InterfaceSerializer::Raw,
                        Some("borsh") => InterfaceSerializer::Borsh,
                        Some("bincode") => InterfaceSerializer::Bincode,
                        Some("raw") => InterfaceSerializer::Raw,
                        Some(_) => return Err(VMError::InvalidOperation),
                    },
                    methods,
                };

                self.interface_registry.insert(name.clone(), interface_info);
            }
        }
        Ok(())
    }

    /// Get interface information by name
    pub fn get_interface_info(&self, interface_name: &str) -> Option<&InterfaceInfo> {
        self.interface_registry.get(interface_name)
    }

    /// Set the current function context (used for init block special handling)
    pub fn set_function_context(&mut self, function_name: Option<String>) {
        self.current_function_context = function_name;
    }

    /// Set the interface registry from the centralized interface management system
    pub fn set_interface_registry(
        &mut self,
        registry: crate::interface_registry::InterfaceRegistry,
    ) {
        // Copy interface information from centralized registry to AST generator registry
        for (name, interface_info) in registry.get_all_interfaces() {
            self.interface_registry
                .insert(name.clone(), interface_info.clone());
        }
    }
}
