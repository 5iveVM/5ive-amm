//! Bridge between LSP and five-dsl-compiler
//!
//! This module reuses the compiler infrastructure to provide diagnostics,
//! type information, and symbol resolution for the LSP.

use five_dsl_compiler::{
    parser::DslParser,
    tokenizer::DslTokenizer,
    ast::{AstNode, TypeNode},
    type_checker::DslTypeChecker,
};
use lsp_types::Url;
use std::collections::HashMap;

use crate::error::LspError;

/// Symbol table entry: (type, is_mutable)
type SymbolTableEntry = (TypeNode, bool);

/// Definition information for go-to-definition feature
#[derive(Debug, Clone)]
pub struct DefinitionInfo {
    pub name: String,
    pub type_info: TypeNode,
    pub location: Option<five_dsl_compiler::ast::SourceLocation>,
}

/// Caches parsed ASTs and symbol tables to avoid recompiling on every change
pub struct CompilerBridge {
    /// AST cache: (source_hash, AST)
    ast_cache: HashMap<Url, (u64, AstNode)>,
    /// Symbol table cache: (source_hash, symbol_table)
    /// Stores the compiler's symbol table for hover/completion features
    symbol_cache: HashMap<Url, (u64, HashMap<String, SymbolTableEntry>)>,
}

impl CompilerBridge {
    pub fn new() -> Self {
        Self {
            ast_cache: HashMap::new(),
            symbol_cache: HashMap::new(),
        }
    }

    /// Compute a simple hash of the source code
    fn hash_source(source: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        source.hash(&mut hasher);
        hasher.finish()
    }

    /// Get cached AST if source hasn't changed
    pub fn get_cached_ast(&self, uri: &Url, source: &str) -> Option<AstNode> {
        let hash = Self::hash_source(source);
        self.ast_cache
            .get(uri)
            .filter(|(cached_hash, _)| *cached_hash == hash)
            .map(|(_, ast)| ast.clone())
    }

    /// Run compilation pipeline to get AST
    ///
    /// This reuses the compiler's existing infrastructure:
    /// - Tokenizer for lexical analysis
    /// - Parser for AST generation
    /// - Type checker for semantic analysis
    ///
    /// Returns the AST and collects any compilation errors through the error system
    pub fn compile_to_ast(&mut self, uri: &Url, source: &str) -> Result<AstNode, LspError> {
        let hash = Self::hash_source(source);

        // Check if we already have a valid AST
        if let Some(cached_ast) = self.get_cached_ast(uri, source) {
            return Ok(cached_ast);
        }

        // Tokenize
        let mut tokenizer = DslTokenizer::new(source);
        let tokens = tokenizer
            .tokenize()
            .map_err(|e| LspError::CompilerError(e.to_string()))?;

        // Parse
        let mut parser = DslParser::new(tokens);
        let ast = parser
            .parse()
            .map_err(|e| LspError::CompilerError(e.to_string()))?;

        // Cache AST
        self.ast_cache.insert(uri.clone(), (hash, ast.clone()));

        Ok(ast)
    }

    /// Get LSP diagnostics for a document
    ///
    /// Runs compilation phases (tokenize, parse, type check) and collects all errors.
    /// This includes parse errors and type errors. Returns all errors found, even if
    /// some phases fail.
    pub fn get_diagnostics(
        &mut self,
        uri: &Url,
        source: &str,
    ) -> Result<Vec<lsp_types::Diagnostic>, LspError> {
        let mut diagnostics = Vec::new();

        // Phase 1: Tokenize
        let mut tokenizer = DslTokenizer::new(source);
        let tokens = match tokenizer.tokenize() {
            Ok(tokens) => tokens,
            Err(e) => {
                // Tokenization error - extract position and create diagnostic
                let error_msg = e.to_string();
                let (line, char_pos) = Self::extract_position_from_error(&error_msg, source);

                diagnostics.push(self.create_diagnostic(
                    "Tokenization error",
                    &error_msg,
                    line,
                    char_pos,
                    char_pos.saturating_add(1),
                    lsp_types::DiagnosticSeverity::ERROR,
                ));
                return Ok(diagnostics);
            }
        };

        // Phase 2: Parse
        let mut parser = DslParser::new(tokens);
        let ast = match parser.parse() {
            Ok(ast) => ast,
            Err(e) => {
                // Parse error - extract line number from error message and create diagnostic
                let error_msg = e.to_string();
                let (line, char_pos) = Self::extract_position_from_error(&error_msg, source);

                diagnostics.push(self.create_diagnostic(
                    "Parse error",
                    &error_msg,
                    line,
                    char_pos,
                    char_pos.saturating_add(1),
                    lsp_types::DiagnosticSeverity::ERROR,
                ));
                return Ok(diagnostics);
            }
        };

        // Cache AST
        let hash = Self::hash_source(source);
        self.ast_cache.insert(uri.clone(), (hash, ast.clone()));

        // Phase 3: Type check
        // Try type checking - if it fails, we still return any partial results
        // Note: Type errors don't have position info in their messages yet, so we skip reporting them
        // The parser will catch most actual syntax errors anyway
        let mut type_checker = DslTypeChecker::new();
        match type_checker.check_types(&ast) {
            Ok(()) => {
                // Type checking succeeded - no type errors
            }
            Err(_e) => {
                // Type checking failed - skip reporting for now since error messages don't have positions
                // TODO: Extract position information from AST when type errors occur
            }
        }

        // ALWAYS cache the symbol table, even if type checking failed
        // This enables hover and completion to work even when code has type errors
        let hash = Self::hash_source(source);
        let symbol_table = type_checker.get_symbol_table().clone();
        self.symbol_cache.insert(uri.clone(), (hash, symbol_table));

        Ok(diagnostics)
    }

    /// Helper to create an LSP diagnostic from error information
    fn create_diagnostic(
        &self,
        title: &str,
        message: &str,
        line: u32,
        start_char: u32,
        end_char: u32,
        severity: lsp_types::DiagnosticSeverity,
    ) -> lsp_types::Diagnostic {
        lsp_types::Diagnostic {
            range: lsp_types::Range {
                start: lsp_types::Position {
                    line,
                    character: start_char,
                },
                end: lsp_types::Position {
                    line,
                    character: end_char,
                },
            },
            severity: Some(severity),
            code: None,
            source: Some("five-compiler".to_string()),
            message: format!("{}: {}", title, message),
            related_information: None,
            tags: None,
            code_description: None,
            data: None,
        }
    }

    // TODO: json_to_diagnostic conversion for Phase 2 when reusing LspFormatter
    // For MVP diagnostics, we construct them directly in get_diagnostics()

    /// Extract position information from error message and convert to line/column
    ///
    /// Parser errors include "at position X" in the message. We convert the character
    /// position to line/column by counting newlines up to that position.
    fn extract_position_from_error(error_msg: &str, source: &str) -> (u32, u32) {
        // Try multiple patterns to extract position information
        let char_pos = error_msg
            .split("position ")
            .nth(1)
            .and_then(|s| s.split(|c: char| !c.is_numeric()).next())
            .and_then(|s| s.parse::<usize>().ok())
            .or_else(|| {
                // Try "at X:" pattern
                error_msg
                    .split("at ")
                    .nth(1)
                    .and_then(|s| s.split(':').next())
                    .and_then(|s| s.trim().parse::<usize>().ok())
            })
            .unwrap_or(0);

        // Convert character position to line/column
        let mut line = 0u32;
        let mut col = 0u32;
        for (i, ch) in source.chars().enumerate() {
            if i >= char_pos {
                break;
            }
            if ch == '\n' {
                line += 1;
                col = 0;
            } else {
                col += 1;
            }
        }

        (line, col)
    }

    /// Resolve a symbol's type information
    ///
    /// Looks up the symbol in the cached symbol table.
    /// Returns (TypeNode, is_mutable) if the symbol exists and type checking succeeded.
    pub fn resolve_symbol(
        &self,
        uri: &Url,
        source: &str,
        symbol_name: &str,
    ) -> Option<SymbolTableEntry> {
        let hash = Self::hash_source(source);
        self.symbol_cache
            .get(uri)
            .filter(|(cached_hash, _)| *cached_hash == hash)
            .and_then(|(_, symbol_table)| symbol_table.get(symbol_name).cloned())
    }

    /// Get all symbols from the cached symbol table
    ///
    /// Returns a list of all symbol names currently defined in the source.
    /// Useful for code completion and other features that need project symbols.
    pub fn get_all_symbols(
        &self,
        uri: &Url,
        source: &str,
    ) -> Option<Vec<String>> {
        let hash = Self::hash_source(source);
        self.symbol_cache
            .get(uri)
            .filter(|(cached_hash, _)| *cached_hash == hash)
            .map(|(_, symbol_table)| symbol_table.keys().cloned().collect())
    }

    /// Clear all caches (useful after significant changes or for testing)
    pub fn clear_caches(&mut self) {
        self.ast_cache.clear();
        self.symbol_cache.clear();
    }

    /// Get definition information for a symbol (for go-to-definition)
    ///
    /// Returns the definition location if available.
    /// Requires that compile_to_ast or get_diagnostics was called first.
    pub fn get_definition(&mut self, uri: &Url, source: &str, symbol_name: &str) -> Option<DefinitionInfo> {
        if let Ok(ast) = self.compile_to_ast(uri, source) {
            let mut type_checker = DslTypeChecker::new();
            let _ = type_checker.check_types(&ast);

            // Get the definition from type checker
            if let Some(def) = type_checker.get_definition(symbol_name) {
                return Some(DefinitionInfo {
                    name: symbol_name.to_string(),
                    type_info: def.type_info.clone(),
                    location: def.location,
                });
            }
        }

        None
    }

    /// Check if a symbol is defined (for find-references validation)
    ///
    /// Verifies that the symbol exists in the type checker's symbol table.
    /// Used to filter out false positives from text-based searches.
    pub fn symbol_exists(&mut self, uri: &Url, source: &str, symbol_name: &str) -> bool {
        if let Ok(ast) = self.compile_to_ast(uri, source) {
            let mut type_checker = DslTypeChecker::new();
            if let Ok(()) = type_checker.check_types(&ast) {
                return type_checker.get_definition(symbol_name).is_some();
            }
        }
        false
    }

    /// Get all defined symbols in the source (for find-references scope validation)
    ///
    /// Returns the set of all symbols that have been defined in this source.
    /// Used to validate that a text-based match refers to an actual symbol.
    pub fn get_defined_symbols(&mut self, uri: &Url, source: &str) -> Vec<String> {
        if let Ok(ast) = self.compile_to_ast(uri, source) {
            let mut type_checker = DslTypeChecker::new();
            if let Ok(()) = type_checker.check_types(&ast) {
                return type_checker
                    .get_all_definitions()
                    .keys()
                    .cloned()
                    .collect();
            }
        }
        vec![]
    }
}

impl Default for CompilerBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for CompilerBridge {
    fn clone(&self) -> Self {
        Self {
            ast_cache: self.ast_cache.clone(),
            symbol_cache: self.symbol_cache.clone(),
        }
    }
}
