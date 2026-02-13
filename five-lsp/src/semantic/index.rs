//! Workspace-wide Semantic Index
//!
//! Maintains a cache of ASTs, symbol tables, and cross-file references
//! for all files in the workspace. Provides fast lookup for LSP queries.

use five_dsl_compiler::ast::{AstNode, SourceLocation};
use five_dsl_compiler::type_checker::module_scope::ModuleSymbolTable;
use lsp_types::{Location, SymbolInformation, SymbolKind, Url};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

/// Symbol definition with location
#[derive(Debug, Clone)]
pub struct SymbolDefinition {
    pub name: String,
    pub uri: String,
    pub location: SourceLocation,
    pub kind: SymbolKind,
}

/// Semantic index for workspace-wide symbol tracking
pub struct SemanticIndex {
    /// Per-file AST cache (URI → (AST, content hash))
    ast_cache: HashMap<String, (AstNode, u64)>,

    /// Per-file symbol tables (URI → ModuleSymbolTable)
    symbol_tables: HashMap<String, ModuleSymbolTable>,

    /// Definition map: symbol name → [(uri, location)]
    definitions: HashMap<String, Vec<SymbolDefinition>>,

    /// Reference map: symbol name → [(uri, location)]
    references: HashMap<String, Vec<(String, SourceLocation)>>,
}

impl SemanticIndex {
    /// Create a new semantic index
    pub fn new() -> Self {
        Self {
            ast_cache: HashMap::new(),
            symbol_tables: HashMap::new(),
            definitions: HashMap::new(),
            references: HashMap::new(),
        }
    }

    /// Update a file in the index
    ///
    /// Parses the source, extracts symbols, and updates the index.
    /// Uses content hashing to avoid redundant work.
    pub fn update_file(&mut self, uri: &str, source: &str) -> Result<(), String> {
        // Calculate content hash
        let content_hash = calculate_hash(source);

        // Check if file has changed
        if let Some((_, existing_hash)) = self.ast_cache.get(uri) {
            if *existing_hash == content_hash {
                // Content unchanged, skip update
                return Ok(());
            }
        }

        // TODO: Parse AST from source using compiler
        // For now, we'll need to integrate with the compiler bridge
        // This is where we'd call: CompilerBridge::parse(source)

        // Placeholder: Mark as needing implementation
        Err("AST parsing not yet integrated".to_string())
    }

    /// Get the AST for a file
    pub fn get_ast(&self, uri: &str) -> Option<&AstNode> {
        self.ast_cache.get(uri).map(|(ast, _)| ast)
    }

    /// Get the symbol table for a file
    pub fn get_symbol_table(&self, uri: &str) -> Option<&ModuleSymbolTable> {
        self.symbol_tables.get(uri)
    }

    /// Find the definition of a symbol
    ///
    /// Returns the location where the symbol is defined.
    pub fn find_definition(&self, symbol_name: &str) -> Option<Location> {
        let defs = self.definitions.get(symbol_name)?;
        let first_def = defs.first()?;

        let uri = Url::parse(&first_def.uri).ok()?;
        let range = location_to_lsp_range(first_def.location);

        Some(Location { uri, range })
    }

    /// Find all references to a symbol
    ///
    /// Returns all locations where the symbol is used.
    pub fn find_references(&self, symbol_name: &str) -> Vec<Location> {
        let mut locations = Vec::new();

        // Add definition as a reference
        if let Some(defs) = self.definitions.get(symbol_name) {
            for def in defs {
                if let Ok(uri) = Url::parse(&def.uri) {
                    let range = location_to_lsp_range(def.location);
                    locations.push(Location { uri, range });
                }
            }
        }

        // Add all references
        if let Some(refs) = self.references.get(symbol_name) {
            for (uri_str, location) in refs {
                if let Ok(uri) = Url::parse(uri_str) {
                    let range = location_to_lsp_range(*location);
                    locations.push(Location { uri, range });
                }
            }
        }

        locations
    }

    /// Get all symbols in the workspace matching a query
    ///
    /// Used for workspace symbol search (Ctrl+T in editors).
    pub fn workspace_symbols(&self, query: &str) -> Vec<SymbolInformation> {
        let mut symbols = Vec::new();
        let query_lower = query.to_lowercase();

        for (symbol_name, defs) in &self.definitions {
            // Filter by query (case-insensitive substring match)
            if !symbol_name.to_lowercase().contains(&query_lower) {
                continue;
            }

            for def in defs {
                if let Ok(uri) = Url::parse(&def.uri) {
                    let range = location_to_lsp_range(def.location);

                    #[allow(deprecated)]
                    symbols.push(SymbolInformation {
                        name: def.name.clone(),
                        kind: def.kind,
                        tags: None,
                        deprecated: None,
                        location: Location { uri, range },
                        container_name: None,
                    });
                }
            }
        }

        symbols
    }

    /// Clear all cached data
    pub fn clear(&mut self) {
        self.ast_cache.clear();
        self.symbol_tables.clear();
        self.definitions.clear();
        self.references.clear();
    }

    /// Get the number of indexed files
    pub fn file_count(&self) -> usize {
        self.ast_cache.len()
    }
}

impl Default for SemanticIndex {
    fn default() -> Self {
        Self::new()
    }
}

/// Calculate a hash for content change detection
fn calculate_hash(content: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    hasher.finish()
}

/// Convert SourceLocation to LSP Range
fn location_to_lsp_range(location: SourceLocation) -> lsp_types::Range {
    lsp_types::Range {
        start: lsp_types::Position {
            line: location.line,
            character: location.column,
        },
        end: lsp_types::Position {
            line: location.line,
            character: location.column + location.length,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_hash_stability() {
        let source1 = "let x = 5;";
        let source2 = "let x = 5;";
        let source3 = "let y = 5;";

        let hash1 = calculate_hash(source1);
        let hash2 = calculate_hash(source2);
        let hash3 = calculate_hash(source3);

        assert_eq!(hash1, hash2, "Same content should have same hash");
        assert_ne!(hash1, hash3, "Different content should have different hash");
    }

    #[test]
    fn test_location_to_lsp_range() {
        let location = SourceLocation::new(10, 5, 8);
        let range = location_to_lsp_range(location);

        assert_eq!(range.start.line, 10);
        assert_eq!(range.start.character, 5);
        assert_eq!(range.end.line, 10);
        assert_eq!(range.end.character, 13); // 5 + 8
    }

    #[test]
    fn test_semantic_index_creation() {
        let index = SemanticIndex::new();
        assert_eq!(index.file_count(), 0);
    }

    #[test]
    fn test_semantic_index_clear() {
        let mut index = SemanticIndex::new();
        // Would add files here if update_file was implemented
        index.clear();
        assert_eq!(index.file_count(), 0);
    }
}
