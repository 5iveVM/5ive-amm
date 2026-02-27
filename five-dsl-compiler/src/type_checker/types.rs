// Type definitions for the type checker

use crate::ast::{InstructionParameter, StructField, TypeNode, SourceLocation};
use crate::type_checker::ModuleScope;
use five_vm_mito::error::VMError;
use std::collections::{HashMap, HashSet};

/// Information about where a symbol is defined
#[derive(Debug, Clone)]
pub struct SymbolDefinition {
    pub type_info: TypeNode,
    pub is_mutable: bool,
    pub location: Option<SourceLocation>,  // Where this symbol was defined
}

/// Interface method information for bytecode generation
#[derive(Debug, Clone)]
pub struct InterfaceMethod {
    pub discriminator: u8,
    pub discriminator_bytes: Option<Vec<u8>>,
    pub is_anchor: bool,
    pub parameters: Vec<InstructionParameter>,
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
    pub is_anchor: bool,
    pub methods: HashMap<String, InterfaceMethod>,
}

/// Type checker context for .five DSL
pub struct TypeCheckerContext {
    pub symbol_table: HashMap<String, (TypeNode, bool)>, // Store (type, is_mutable)
    /// Symbol definitions with source location for go-to-definition and hover
    pub(crate) symbol_definitions: HashMap<String, SymbolDefinition>,
    pub(crate) account_definitions: HashMap<String, Vec<StructField>>,
    pub(crate) interface_registry: HashMap<String, InterfaceInfo>,
    /// Tracks which account parameters are writable (@mut) for the current function
    pub(crate) current_writable_accounts: Option<std::collections::HashSet<String>>,
    /// Current function name for diagnostics
    pub(crate) current_function: Option<String>,
    /// Full current function parameter metadata for constraint-aware validation.
    pub(crate) current_function_parameters: Option<Vec<InstructionParameter>>,
    /// Map of user-defined function return types for type inference
    pub(crate) function_return_types: HashMap<String, Option<TypeNode>>,
    /// Module scope for multi-module type checking (optional)
    pub(crate) module_scope: Option<ModuleScope>,
    /// Current module being type-checked (for multi-module support)
    pub(crate) current_module: Option<String>,
    /// Imported external interface namespace symbols from use/import statements.
    pub(crate) imported_external_interfaces: HashSet<String>,
    /// Canonical module alias/full-path -> interface name mapping used for module-qualified CPI calls.
    pub(crate) interface_module_aliases: HashMap<String, String>,
    /// Canonical imported module alias -> full module path (for diagnostics/suggestions).
    pub(crate) imported_module_aliases: HashMap<String, String>,
    /// Account parameter names that have seeded @init and therefore expose `account.ctx.bump`.
    pub(crate) init_bump_accounts: HashSet<String>,
    /// Account parameter names that have @init and therefore expose `account.ctx.space`.
    pub(crate) init_space_accounts: HashSet<String>,
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
            symbol_definitions: HashMap::new(),
            account_definitions: HashMap::new(),
            interface_registry: HashMap::new(),
            current_writable_accounts: None,
            current_function: None,
            current_function_parameters: None,
            function_return_types: HashMap::new(),
            module_scope: None,
            current_module: None,
            imported_external_interfaces: HashSet::new(),
            interface_module_aliases: HashMap::new(),
            imported_module_aliases: HashMap::new(),
            init_bump_accounts: HashSet::new(),
            init_space_accounts: HashSet::new(),
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

    /// Record where a symbol was defined (for go-to-definition)
    pub fn record_definition(&mut self, name: String, type_info: TypeNode, is_mutable: bool, location: Option<SourceLocation>) {
        self.symbol_definitions.insert(name, SymbolDefinition {
            type_info,
            is_mutable,
            location,
        });
    }

    /// Get definition information for a symbol (includes source location)
    pub fn get_definition(&self, name: &str) -> Option<&SymbolDefinition> {
        self.symbol_definitions.get(name)
    }

    /// Get all symbol definitions (for workspace symbol search)
    pub fn get_all_definitions(&self) -> &HashMap<String, SymbolDefinition> {
        &self.symbol_definitions
    }

    /// Get mutable reference to module scope (for testing)
    pub fn get_module_scope_mut(&mut self) -> Option<&mut ModuleScope> {
        self.module_scope.as_mut()
    }

    /// Build a rich undefined-identifier VM error with nearest-match context.
    pub(crate) fn undefined_identifier_error(&self, name: &str) -> VMError {
        let candidate = self.closest_identifier_candidate(name);
        VMError::undefined_identifier(name, candidate.as_deref())
    }

    fn closest_identifier_candidate(&self, target: &str) -> Option<String> {
        if target.is_empty() {
            return None;
        }

        let mut best: Option<(usize, usize, String)> = None;
        let max_distance = if target.len() <= 4 { 1 } else { 2 };

        for candidate in self
            .symbol_table
            .keys()
            .map(String::as_str)
            .chain(self.interface_registry.keys().map(String::as_str))
            .chain(self.imported_external_interfaces.iter().map(String::as_str))
            .chain(self.interface_module_aliases.keys().map(String::as_str))
            .chain(self.imported_module_aliases.keys().map(String::as_str))
            .chain(self.function_return_types.keys().map(String::as_str))
        {
            if candidate == target {
                continue;
            }

            let distance = levenshtein_distance(target, candidate);
            if distance > max_distance {
                continue;
            }

            let len_delta = target.len().abs_diff(candidate.len());
            match &best {
                Some((best_distance, best_len_delta, best_name)) => {
                    if distance < *best_distance
                        || (distance == *best_distance
                            && (len_delta < *best_len_delta
                                || (len_delta == *best_len_delta
                                    && candidate < best_name.as_str())))
                    {
                        best = Some((distance, len_delta, candidate.to_string()));
                    }
                }
                None => {
                    best = Some((distance, len_delta, candidate.to_string()));
                }
            }
        }

        best.map(|(_, _, name)| name)
    }
}

fn levenshtein_distance(a: &str, b: &str) -> usize {
    if a == b {
        return 0;
    }
    if a.is_empty() {
        return b.chars().count();
    }
    if b.is_empty() {
        return a.chars().count();
    }

    let b_chars: Vec<char> = b.chars().collect();
    let mut prev_row: Vec<usize> = (0..=b_chars.len()).collect();
    let mut curr_row = vec![0; b_chars.len() + 1];

    for (i, a_char) in a.chars().enumerate() {
        curr_row[0] = i + 1;
        for (j, b_char) in b_chars.iter().enumerate() {
            let substitution_cost = if a_char == *b_char { 0 } else { 1 };
            curr_row[j + 1] = (prev_row[j + 1] + 1)
                .min(curr_row[j] + 1)
                .min(prev_row[j] + substitution_cost);
        }
        prev_row.clone_from(&curr_row);
    }

    prev_row[b_chars.len()]
}
