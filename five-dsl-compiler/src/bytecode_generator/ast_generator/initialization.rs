//! ASTGenerator initialization and configuration.

use super::super::account_system::AccountSystem;
use super::types::ASTGenerator;
use crate::ast::AstNode;
use crate::ast::TypeNode;
use crate::type_checker::{InterfaceInfo, InterfaceMethod, InterfaceSerializer};
use five_vm_mito::error::VMError;
use sha2::Digest;

use std::collections::HashMap;

impl ASTGenerator {
    /// Internal constructor with configurable v2_preview flag.
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
            current_function_parameters: None,
            current_function_return_type: None,
            function_parameter_types: HashMap::new(),
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
            module_interface_aliases: HashMap::new(),
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
        self.current_function_parameters = None;
        self.function_parameter_types.clear();
        self.jump_patches.clear();
        self.br_eq_u8_patches.clear();
        self.function_patches.clear();
        self.function_positions.clear();
        self.label_positions.clear();
        self.label_counter = 0;
        self.interface_registry.clear();
        self.name_deduplication = super::super::types::NameDeduplication::new();
        self.external_imports.clear();
        self.module_interface_aliases.clear();
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

    pub(crate) fn cache_function_parameter_types(
        &mut self,
        instruction_definitions: &[AstNode],
    ) {
        self.function_parameter_types.clear();
        for instruction_def in instruction_definitions {
            if let AstNode::InstructionDefinition {
                name,
                parameters,
                ..
            } = instruction_def
            {
                self.function_parameter_types.insert(
                    name.clone(),
                    parameters
                        .iter()
                        .map(|param| param.param_type.clone())
                        .collect(),
                );
            }
        }
    }

    /// Register an external import for CALL_EXTERNAL generation
    /// 
    /// When a function from this module is called, CALL_EXTERNAL will be emitted
    /// instead of a regular CALL opcode.
    pub fn register_external_import(
        &mut self,
        module_name: String,
        account_index: u8,
        allow_any_function: bool,
        functions: HashMap<String, u16>,
    ) {
        use super::types::ExternalImport;
        self.external_imports.insert(
            module_name.clone(),
            ExternalImport {
                module_name,
                account_index,
                allow_any_function,
                functions,
            },
        );
    }

    pub fn register_module_interface_alias(
        &mut self,
        module_namespace: String,
        interface_name: String,
    ) {
        self.module_interface_aliases
            .insert(module_namespace, interface_name);
    }

    pub fn find_interface_for_module_alias(&self, alias: &str) -> Option<String> {
        let mut matches: Vec<String> = self
            .interface_registry
            .keys()
            .filter(|name| {
                let snake = Self::to_snake_case(name);
                snake == alias || name.as_str() == alias
            })
            .cloned()
            .collect();

        if matches.len() == 1 {
            return matches.pop();
        }
        None
    }

    fn to_snake_case(name: &str) -> String {
        let mut out = String::new();
        let chars: Vec<char> = name.chars().collect();
        for (i, ch) in chars.iter().enumerate() {
            let ch = *ch;
            if ch.is_uppercase() {
                let prev = if i > 0 { Some(chars[i - 1]) } else { None };
                let next = chars.get(i + 1).copied();
                let needs_sep = (i != 0
                    && prev.map(|p| p.is_lowercase() || p.is_ascii_digit()).unwrap_or(false))
                    || (i != 0
                        && prev.map(|p| p.is_uppercase()).unwrap_or(false)
                        && next.map(|n| n.is_lowercase()).unwrap_or(false));
                if needs_sep {
                    out.push('_');
                }
                for lower in ch.to_lowercase() {
                    out.push(lower);
                }
            } else {
                out.push(ch);
            }
        }
        out
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
                is_anchor,
                functions,
            } = interface_def
            {
                let mut methods = HashMap::new();
                let serializer_hint = serializer.clone();

                for function_def in functions {
                    if let AstNode::InterfaceFunction {
                        name: method_name,
                        parameters,
                        return_type,
                        discriminator,
                        discriminator_bytes,
                        is_anchor: is_method_anchor,
                    } = function_def
                    {
                        let method_anchor = *is_anchor || *is_method_anchor;
                        // Convert InstructionParameter to TypeNode for storage
                        let param_types: Vec<TypeNode> = parameters
                            .iter()
                            .map(|param| param.param_type.clone())
                            .collect();

                        let return_type_node = return_type.as_ref().map(|rt| (**rt).clone());

                        let (discriminator_val, discriminator_bytes_val) = if let Some(bytes) = discriminator_bytes {
                            (discriminator.unwrap_or(0), Some(bytes.clone()))
                        } else if let Some(disc) = discriminator {
                            (*disc, None)
                        } else if method_anchor {
                            let preimage = format!("global:{}", method_name);
                            let mut hasher = sha2::Sha256::new();
                            hasher.update(preimage.as_bytes());
                            let result = hasher.finalize();
                            (0, Some(result[..8].to_vec()))
                        } else {
                            (0, None)
                        };

                        methods.insert(
                            method_name.clone(),
                            InterfaceMethod {
                                discriminator: discriminator_val,
                                discriminator_bytes: discriminator_bytes_val,
                                is_anchor: method_anchor,
                                parameters: param_types,
                                return_type: return_type_node,
                            },
                        );
                    }
                }

                let has_anchor_methods = methods.values().any(|m| m.is_anchor);
                let anchor_mode = *is_anchor || has_anchor_methods;

                let interface_info = InterfaceInfo {
                    program_id: program_id.clone().unwrap_or_default(), // Default to empty if no program ID
                    serializer: match serializer_hint.as_deref() {
                        None => {
                            if anchor_mode {
                                InterfaceSerializer::Borsh
                            } else {
                                InterfaceSerializer::Bincode
                            }
                        }
                        Some("borsh") => InterfaceSerializer::Borsh,
                        Some("bincode") => InterfaceSerializer::Bincode,
                        Some("raw") => InterfaceSerializer::Raw,
                        Some(_) => return Err(VMError::InvalidOperation),
                    },
                    is_anchor: anchor_mode,
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
