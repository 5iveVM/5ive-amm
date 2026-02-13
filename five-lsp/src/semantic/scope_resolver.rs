//! Scope Resolution for Symbol Lookup
//!
//! Provides scope-aware symbol resolution with shadowing support.
//! Reuses the compiler's ModuleScope for cross-module visibility.

use five_dsl_compiler::ast::{AstNode, Visibility};
use five_dsl_compiler::type_checker::module_scope::ModuleScope;
use std::collections::HashMap;

/// Scope chain entry representing a lexical scope level
#[derive(Debug, Clone)]
pub struct ScopeLevel {
    /// Symbols defined at this scope level
    pub symbols: HashMap<String, SymbolInfo>,
    /// Scope type (global, function, block)
    pub scope_type: ScopeType,
}

/// Type of lexical scope
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeType {
    /// Module-level scope (global variables, functions)
    Global,
    /// Function scope (parameters, local variables)
    Function,
    /// Block scope (if/while/for bodies)
    Block,
}

/// Information about a symbol in a scope
#[derive(Debug, Clone)]
pub struct SymbolInfo {
    /// Symbol name
    pub name: String,
    /// Type of the symbol (if known)
    pub type_name: Option<String>,
    /// Whether the symbol is mutable
    pub is_mutable: bool,
    /// Visibility modifier
    pub visibility: Visibility,
}

/// Scope resolver for finding symbols with shadowing support
pub struct ScopeResolver {
    /// Stack of scope levels (innermost last)
    scope_stack: Vec<ScopeLevel>,
}

impl ScopeResolver {
    /// Create a new scope resolver
    pub fn new() -> Self {
        Self {
            scope_stack: vec![ScopeLevel {
                symbols: HashMap::new(),
                scope_type: ScopeType::Global,
            }],
        }
    }

    /// Push a new scope level
    pub fn push_scope(&mut self, scope_type: ScopeType) {
        self.scope_stack.push(ScopeLevel {
            symbols: HashMap::new(),
            scope_type,
        });
    }

    /// Pop the current scope level
    pub fn pop_scope(&mut self) {
        if self.scope_stack.len() > 1 {
            self.scope_stack.pop();
        }
    }

    /// Add a symbol to the current scope
    pub fn add_symbol(&mut self, name: String, info: SymbolInfo) {
        if let Some(scope) = self.scope_stack.last_mut() {
            scope.symbols.insert(name, info);
        }
    }

    /// Look up a symbol, respecting shadowing (innermost scope wins)
    pub fn lookup(&self, name: &str) -> Option<&SymbolInfo> {
        // Search from innermost to outermost scope
        for scope in self.scope_stack.iter().rev() {
            if let Some(symbol) = scope.symbols.get(name) {
                return Some(symbol);
            }
        }
        None
    }

    /// Get all symbols visible at the current scope
    pub fn visible_symbols(&self) -> HashMap<String, SymbolInfo> {
        let mut symbols = HashMap::new();

        // Collect symbols from outermost to innermost (inner shadows outer)
        for scope in self.scope_stack.iter() {
            for (name, info) in &scope.symbols {
                symbols.insert(name.clone(), info.clone());
            }
        }

        symbols
    }

    /// Build scope chain from AST at a given position
    ///
    /// Walks the AST from root to the node at position, building the
    /// scope stack along the way.
    pub fn build_scope_at_position(
        _ast: &AstNode,
        _line: u32,
        _column: u32,
        module_scope: &ModuleScope,
    ) -> Self {
        let mut resolver = Self::new();

        // Add module-level symbols from module_scope
        let current_module_name = module_scope.current_module();
        if let Some(current_table) = module_scope.get_module_table(current_module_name) {
            for (name, symbol) in current_table.iter() {
                resolver.add_symbol(
                    name.clone(),
                    SymbolInfo {
                        name: name.clone(),
                        type_name: Some(format!("{:?}", symbol.type_info)), // Simplified: just format the type
                        is_mutable: symbol.is_mutable,
                        visibility: symbol.visibility,
                    },
                );
            }
        }

        // TODO: Implement AST walking for function and block scopes
        // The AST structure has changed since this was written, so we defer implementation
        resolver
    }
}

impl Default for ScopeResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope_resolver_shadowing() {
        let mut resolver = ScopeResolver::new();

        // Add global symbol
        resolver.add_symbol(
            "x".to_string(),
            SymbolInfo {
                name: "x".to_string(),
                type_name: Some("u64".to_string()),
                is_mutable: false,
                visibility: Visibility::Public,
            },
        );

        // Lookup in global scope
        assert!(resolver.lookup("x").is_some());
        assert_eq!(resolver.lookup("x").unwrap().type_name.as_deref(), Some("u64"));

        // Push function scope and add shadowing symbol
        resolver.push_scope(ScopeType::Function);
        resolver.add_symbol(
            "x".to_string(),
            SymbolInfo {
                name: "x".to_string(),
                type_name: Some("string".to_string()),
                is_mutable: true,
                visibility: Visibility::Private,
            },
        );

        // Inner scope shadows outer
        assert_eq!(resolver.lookup("x").unwrap().type_name.as_deref(), Some("string"));
        assert!(resolver.lookup("x").unwrap().is_mutable);

        // Pop scope - back to global
        resolver.pop_scope();
        assert_eq!(resolver.lookup("x").unwrap().type_name.as_deref(), Some("u64"));
        assert!(!resolver.lookup("x").unwrap().is_mutable);
    }

    #[test]
    fn test_visible_symbols() {
        let mut resolver = ScopeResolver::new();

        resolver.add_symbol(
            "global".to_string(),
            SymbolInfo {
                name: "global".to_string(),
                type_name: Some("u64".to_string()),
                is_mutable: false,
                visibility: Visibility::Public,
            },
        );

        resolver.push_scope(ScopeType::Function);
        resolver.add_symbol(
            "local".to_string(),
            SymbolInfo {
                name: "local".to_string(),
                type_name: Some("string".to_string()),
                is_mutable: true,
                visibility: Visibility::Private,
            },
        );

        let symbols = resolver.visible_symbols();
        assert_eq!(symbols.len(), 2);
        assert!(symbols.contains_key("global"));
        assert!(symbols.contains_key("local"));
    }
}
