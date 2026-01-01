// Type definitions for the type checker

use crate::ast::{StructField, TypeNode};
use crate::type_checker::ModuleScope;
use std::collections::HashMap;

/// Interface method information for bytecode generation
#[derive(Debug, Clone)]
pub struct InterfaceMethod {
    pub discriminator: u8,
    pub discriminator_bytes: Option<Vec<u8>>,
    pub parameters: Vec<TypeNode>,
    pub return_type: Option<TypeNode>,
}

/// Serializer options for interface-based CPI calls
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InterfaceSerializer {
    Raw,
    Borsh,
    Bincode,
}

/// Interface definition information
#[derive(Debug, Clone)]
pub struct InterfaceInfo {
    pub program_id: String,
    pub serializer: InterfaceSerializer,
    pub methods: HashMap<String, InterfaceMethod>,
}

/// Type checker context for .five DSL
pub struct TypeCheckerContext {
    pub symbol_table: HashMap<String, (TypeNode, bool)>, // Store (type, is_mutable)
    pub(crate) account_definitions: HashMap<String, Vec<StructField>>,
    pub(crate) interface_registry: HashMap<String, InterfaceInfo>,
    /// Tracks which account parameters are writable (@mut) for the current function
    pub(crate) current_writable_accounts: Option<std::collections::HashSet<String>>,
    /// Current function name for diagnostics
    pub(crate) current_function: Option<String>,
    /// Map of user-defined function return types for type inference
    pub(crate) function_return_types: HashMap<String, Option<TypeNode>>,
    /// Module scope for multi-module type checking (optional)
    pub(crate) module_scope: Option<ModuleScope>,
    /// Current module being type-checked (for multi-module support)
    pub(crate) current_module: Option<String>,
}

impl Default for TypeCheckerContext {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeCheckerContext {
    pub fn new() -> Self {
        Self {
            symbol_table: HashMap::new(),
            account_definitions: HashMap::new(),
            interface_registry: HashMap::new(),
            current_writable_accounts: None,
            current_function: None,
            function_return_types: HashMap::new(),
            module_scope: None,
            current_module: None,
        }
    }

    /// Create a new type checker context with multi-module support
    pub fn with_module_scope(mut self, module_scope: ModuleScope) -> Self {
        self.module_scope = Some(module_scope);
        self
    }

    /// Set the current module being type-checked
    pub fn set_current_module(&mut self, module_name: String) {
        self.current_module = Some(module_name.clone());
        if let Some(ref mut scope) = self.module_scope {
            scope.set_current_module(module_name);
        }
    }

    /// Add a symbol to the current module scope (if multi-module mode)
    pub fn add_to_module_scope(&mut self, name: String, type_info: TypeNode, is_mutable: bool, visibility: crate::ast::Visibility) {
        if let Some(ref mut scope) = self.module_scope {
            let symbol = super::ModuleSymbol {
                type_info,
                is_mutable,
                visibility,
            };
            scope.add_symbol_to_current(name, symbol);
        }
    }

    /// Resolve a symbol using module scope if available, with mutability info
    pub fn resolve_symbol(&self, name: &str) -> Option<(TypeNode, bool)> {
        if let Some(ref scope) = self.module_scope {
            if let Some(current_module) = &self.current_module {
                if let Some(symbol) = scope.resolve_symbol(name, current_module) {
                    return Some((symbol.type_info.clone(), symbol.is_mutable));
                }
            }
        }
        // Fall back to local symbol table
        self.symbol_table.get(name).map(|(t, m)| (t.clone(), *m))
    }

    /// Resolve a symbol using module scope if available (type only, no mutability)
    pub fn resolve_with_module_scope(&self, name: &str) -> Option<TypeNode> {
        self.resolve_symbol(name).map(|(t, _)| t)
    }

    /// Check if a symbol is on-chain callable (for multi-module support)
    pub fn is_on_chain_callable_symbol(&self, name: &str) -> bool {
        if let Some(ref scope) = self.module_scope {
            if let Some(current_module) = &self.current_module {
                return scope.resolve_symbol(name, current_module)
                    .map(|s| s.visibility.is_on_chain_callable())
                    .unwrap_or(false);
            }
        }
        false
    }

    /// Get the symbol table (for testing)
    pub fn get_symbol_table(&self) -> &std::collections::HashMap<String, (TypeNode, bool)> {
        &self.symbol_table
    }

    /// Get the current module name (for testing)
    pub fn get_current_module(&self) -> Option<&str> {
        self.current_module.as_deref()
    }

    /// Check if module scope is active (for testing)
    pub fn has_module_scope(&self) -> bool {
        self.module_scope.is_some()
    }

    /// Get mutable reference to module scope (for testing)
    pub fn get_module_scope_mut(&mut self) -> Option<&mut ModuleScope> {
        self.module_scope.as_mut()
    }
}
