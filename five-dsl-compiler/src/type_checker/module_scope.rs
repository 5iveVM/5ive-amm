// Module Scope System for Multi-File Compilation
//
// Manages symbol visibility and scoping across multiple modules.
// Provides module-aware symbol resolution with visibility enforcement.

use crate::ast::{TypeNode, Visibility};
use std::collections::HashMap;

/// Information about a symbol in a specific module
#[derive(Debug, Clone)]
pub struct ModuleSymbol {
    /// Type of the symbol
    pub type_info: TypeNode,
    /// Whether the symbol is mutable
    pub is_mutable: bool,
    /// Visibility of the symbol (determines cross-module accessibility)
    pub visibility: Visibility,
}

/// Symbol table for a single module
#[derive(Debug, Clone)]
pub struct ModuleSymbolTable {
    /// Module name
    pub module_name: String,
    /// Local symbols defined in this module
    pub symbols: HashMap<String, ModuleSymbol>,
}

impl ModuleSymbolTable {
    /// Create a new module symbol table
    pub fn new(module_name: String) -> Self {
        Self {
            module_name,
            symbols: HashMap::new(),
        }
    }

    /// Add a symbol to the module scope
    pub fn insert(&mut self, name: String, symbol: ModuleSymbol) {
        self.symbols.insert(name, symbol);
    }

    /// Look up a symbol in the module scope
    pub fn lookup(&self, name: &str) -> Option<&ModuleSymbol> {
        self.symbols.get(name)
    }

    /// Get all symbols in the module
    pub fn iter(&self) -> impl Iterator<Item = (&String, &ModuleSymbol)> {
        self.symbols.iter()
    }

    /// Get symbol count
    pub fn len(&self) -> usize {
        self.symbols.len()
    }

    /// Check if symbol table is empty
    pub fn is_empty(&self) -> bool {
        self.symbols.is_empty()
    }
}

/// Cross-module scope manager
#[derive(Debug, Clone)]
pub struct ModuleScope {
    /// Symbol tables for each module
    module_tables: HashMap<String, ModuleSymbolTable>,
    /// Current module being checked
    current_module: String,
    /// Import relationships (module -> list of imported modules)
    imports: HashMap<String, Vec<String>>,
}

impl ModuleScope {
    /// Create a new module scope
    pub fn new(entry_module: String) -> Self {
        let mut module_tables = HashMap::new();
        module_tables.insert(entry_module.clone(), ModuleSymbolTable::new(entry_module.clone()));

        Self {
            module_tables,
            current_module: entry_module,
            imports: HashMap::new(),
        }
    }

    /// Add a new module to the scope
    pub fn add_module(&mut self, module_name: String) {
        self.module_tables
            .insert(module_name.clone(), ModuleSymbolTable::new(module_name));
    }

    /// Register an import relationship
    pub fn register_import(&mut self, from_module: String, imported_module: String) {
        self.imports
            .entry(from_module)
            .or_default()
            .push(imported_module);
    }

    /// Set the current module context
    pub fn set_current_module(&mut self, module_name: String) {
        self.current_module = module_name;
    }

    /// Get the current module name
    pub fn current_module(&self) -> &str {
        &self.current_module
    }

    /// Add a symbol to the current module's scope
    pub fn add_symbol_to_current(&mut self, name: String, symbol: ModuleSymbol) {
        if let Some(table) = self.module_tables.get_mut(&self.current_module) {
            table.insert(name, symbol);
        }
    }

    /// Look up a symbol respecting visibility rules across modules
    pub fn resolve_symbol(
        &self,
        name: &str,
        requesting_module: &str,
    ) -> Option<ModuleSymbol> {
        // First, check in the requesting module
        if let Some(table) = self.module_tables.get(requesting_module) {
            if let Some(symbol) = table.lookup(name) {
                return Some(symbol.clone());
            }
        }

        // Then, check in imported modules
        if let Some(imported_modules) = self.imports.get(requesting_module) {
            for imported_module in imported_modules {
                if let Some(table) = self.module_tables.get(imported_module) {
                    if let Some(symbol) = table.lookup(name) {
                        // Check visibility: only Public and Internal symbols are importable
                        if symbol.visibility.is_importable() {
                            return Some(symbol.clone());
                        }
                    }
                }
            }
        }

        None
    }

    /// Check if a symbol is on-chain callable from the current module
    pub fn is_on_chain_callable(&self, name: &str) -> bool {
        if let Some(table) = self.module_tables.get(&self.current_module) {
            if let Some(symbol) = table.lookup(name) {
                return symbol.visibility.is_on_chain_callable();
            }
        }
        false
    }

    /// Get the symbol table for a module
    pub fn get_module_table(&self, module_name: &str) -> Option<&ModuleSymbolTable> {
        self.module_tables.get(module_name)
    }

    /// Get all module names
    pub fn module_names(&self) -> impl Iterator<Item = &String> {
        self.module_tables.keys()
    }

    /// Check if a module exists
    pub fn has_module(&self, module_name: &str) -> bool {
        self.module_tables.contains_key(module_name)
    }

    /// Check if a module imports another
    pub fn imports_module(&self, from_module: &str, imported: &str) -> bool {
        self.imports
            .get(from_module)
            .map(|imports| imports.contains(&imported.to_string()))
            .unwrap_or(false)
    }

    /// Get all imported modules for a given module
    pub fn get_imports(&self, module_name: &str) -> Option<&Vec<String>> {
        self.imports.get(module_name)
    }
}

impl Default for ModuleScope {
    fn default() -> Self {
        Self::new("main".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_symbol_table_creation() {
        let table = ModuleSymbolTable::new("test_module".to_string());
        assert_eq!(table.module_name, "test_module");
        assert!(table.is_empty());
        assert_eq!(table.len(), 0);
    }

    #[test]
    fn test_insert_and_lookup_symbols() {
        let mut table = ModuleSymbolTable::new("test_module".to_string());

        let symbol = ModuleSymbol {
            type_info: TypeNode::Primitive("u64".to_string()),
            is_mutable: true,
            visibility: Visibility::Public,
        };

        table.insert("x".to_string(), symbol.clone());
        assert_eq!(table.len(), 1);

        let looked_up = table.lookup("x").unwrap();
        assert_eq!(looked_up.visibility, Visibility::Public);
        assert!(looked_up.is_mutable);
    }

    #[test]
    fn test_module_scope_creation() {
        let scope = ModuleScope::new("main".to_string());
        assert_eq!(scope.current_module(), "main");
        assert!(scope.has_module("main"));
    }

    #[test]
    fn test_add_module() {
        let mut scope = ModuleScope::new("main".to_string());
        scope.add_module("helper".to_string());

        assert!(scope.has_module("main"));
        assert!(scope.has_module("helper"));
    }

    #[test]
    fn test_register_import() {
        let mut scope = ModuleScope::new("main".to_string());
        scope.add_module("helper".to_string());
        scope.register_import("main".to_string(), "helper".to_string());

        assert!(scope.imports_module("main", "helper"));
    }

    #[test]
    fn test_resolve_symbol_local() {
        let mut scope = ModuleScope::new("main".to_string());

        let symbol = ModuleSymbol {
            type_info: TypeNode::Primitive("u64".to_string()),
            is_mutable: false,
            visibility: Visibility::Public,
        };

        scope.add_symbol_to_current("x".to_string(), symbol);

        let resolved = scope.resolve_symbol("x", "main");
        assert!(resolved.is_some());
    }

    #[test]
    fn test_resolve_symbol_imported() {
        let mut scope = ModuleScope::new("main".to_string());
        scope.add_module("helper".to_string());
        scope.register_import("main".to_string(), "helper".to_string());

        // Add symbol to helper module
        scope.set_current_module("helper".to_string());
        let symbol = ModuleSymbol {
            type_info: TypeNode::Primitive("u64".to_string()),
            is_mutable: false,
            visibility: Visibility::Public,
        };
        scope.add_symbol_to_current("helper_fn".to_string(), symbol);

        // Resolve from main
        let resolved = scope.resolve_symbol("helper_fn", "main");
        assert!(resolved.is_some());
    }

    #[test]
    fn test_visibility_enforcement_public() {
        let mut scope = ModuleScope::new("main".to_string());
        scope.add_module("helper".to_string());
        scope.register_import("main".to_string(), "helper".to_string());

        scope.set_current_module("helper".to_string());
        let symbol = ModuleSymbol {
            type_info: TypeNode::Primitive("u64".to_string()),
            is_mutable: false,
            visibility: Visibility::Public,
        };
        scope.add_symbol_to_current("public_fn".to_string(), symbol);

        // Public symbols should be importable
        let resolved = scope.resolve_symbol("public_fn", "main");
        assert!(resolved.is_some());
    }

    #[test]
    fn test_visibility_enforcement_internal() {
        let mut scope = ModuleScope::new("main".to_string());
        scope.add_module("helper".to_string());
        scope.register_import("main".to_string(), "helper".to_string());

        scope.set_current_module("helper".to_string());
        let symbol = ModuleSymbol {
            type_info: TypeNode::Primitive("u64".to_string()),
            is_mutable: false,
            visibility: Visibility::Internal,
        };
        scope.add_symbol_to_current("internal_fn".to_string(), symbol);

        // Internal symbols should be importable
        let resolved = scope.resolve_symbol("internal_fn", "main");
        assert!(resolved.is_some());
    }

    #[test]
    fn test_visibility_enforcement_private() {
        let mut scope = ModuleScope::new("main".to_string());
        scope.add_module("helper".to_string());
        scope.register_import("main".to_string(), "helper".to_string());

        scope.set_current_module("helper".to_string());
        let symbol = ModuleSymbol {
            type_info: TypeNode::Primitive("u64".to_string()),
            is_mutable: false,
            visibility: Visibility::Private,
        };
        scope.add_symbol_to_current("private_fn".to_string(), symbol);

        // Private symbols should NOT be importable
        let resolved = scope.resolve_symbol("private_fn", "main");
        assert!(resolved.is_none());
    }

    #[test]
    fn test_on_chain_callable_public() {
        let mut scope = ModuleScope::new("main".to_string());

        let symbol = ModuleSymbol {
            type_info: TypeNode::Primitive("u64".to_string()),
            is_mutable: false,
            visibility: Visibility::Public,
        };

        scope.add_symbol_to_current("public_fn".to_string(), symbol);

        // Public functions are on-chain callable
        assert!(scope.is_on_chain_callable("public_fn"));
    }

    #[test]
    fn test_on_chain_callable_internal() {
        let mut scope = ModuleScope::new("main".to_string());

        let symbol = ModuleSymbol {
            type_info: TypeNode::Primitive("u64".to_string()),
            is_mutable: false,
            visibility: Visibility::Internal,
        };

        scope.add_symbol_to_current("internal_fn".to_string(), symbol);

        // Internal functions are NOT on-chain callable
        assert!(!scope.is_on_chain_callable("internal_fn"));
    }

    #[test]
    fn test_no_import_no_access() {
        let mut scope = ModuleScope::new("main".to_string());
        scope.add_module("helper".to_string());
        // Note: NOT registering import

        scope.set_current_module("helper".to_string());
        let symbol = ModuleSymbol {
            type_info: TypeNode::Primitive("u64".to_string()),
            is_mutable: false,
            visibility: Visibility::Public,
        };
        scope.add_symbol_to_current("helper_fn".to_string(), symbol);

        // Without explicit import, should not be able to resolve
        let resolved = scope.resolve_symbol("helper_fn", "main");
        assert!(resolved.is_none());
    }

    #[test]
    fn test_get_module_table() {
        let scope = ModuleScope::new("main".to_string());

        let table = scope.get_module_table("main");
        assert!(table.is_some());

        let table = scope.get_module_table("nonexistent");
        assert!(table.is_none());
    }

    #[test]
    fn test_get_all_imports() {
        let mut scope = ModuleScope::new("main".to_string());
        scope.add_module("helper1".to_string());
        scope.add_module("helper2".to_string());
        scope.register_import("main".to_string(), "helper1".to_string());
        scope.register_import("main".to_string(), "helper2".to_string());

        let imports = scope.get_imports("main");
        assert!(imports.is_some());
        assert_eq!(imports.unwrap().len(), 2);
    }
}
