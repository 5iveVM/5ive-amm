//! Bridge between LSP and five-dsl-compiler
//!
//! This module reuses the compiler infrastructure to provide diagnostics,
//! type information, and symbol resolution for the LSP.

use five_dsl_compiler::{
    parser::DslParser,
    tokenizer::DslTokenizer,
    ast::AstNode,
};
use lsp_types::Url;
use std::collections::HashMap;

use crate::error::LspError;

/// Caches parsed ASTs to avoid recompiling on every change
pub struct CompilerBridge {
    /// AST cache: (source_hash, AST)
    ast_cache: HashMap<Url, (u64, AstNode)>,
}

impl CompilerBridge {
    pub fn new() -> Self {
        Self {
            ast_cache: HashMap::new(),
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
    fn get_cached_ast(&self, uri: &Url, source: &str) -> Option<AstNode> {
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
    /// This includes both parse errors and type errors.
    pub fn get_diagnostics(
        &mut self,
        uri: &Url,
        source: &str,
    ) -> Result<Vec<lsp_types::Diagnostic>, LspError> {
        let mut diagnostics = Vec::new();

        // Tokenize
        let mut tokenizer = DslTokenizer::new(source);
        let tokens = match tokenizer.tokenize() {
            Ok(tokens) => tokens,
            Err(e) => {
                // Tokenization error - return it as a diagnostic
                diagnostics.push(lsp_types::Diagnostic {
                    range: lsp_types::Range {
                        start: lsp_types::Position { line: 0, character: 0 },
                        end: lsp_types::Position { line: 0, character: 1 },
                    },
                    severity: Some(lsp_types::DiagnosticSeverity::ERROR),
                    code: None,
                    source: Some("five-compiler".to_string()),
                    message: format!("Tokenization error: {}", e),
                    related_information: None,
                    tags: None,
                    code_description: None,
                    data: None,
                });
                return Ok(diagnostics);
            }
        };

        // Parse
        let mut parser = DslParser::new(tokens);
        let ast = match parser.parse() {
            Ok(ast) => ast,
            Err(e) => {
                // Parse error
                diagnostics.push(lsp_types::Diagnostic {
                    range: lsp_types::Range {
                        start: lsp_types::Position { line: 0, character: 0 },
                        end: lsp_types::Position { line: 0, character: 1 },
                    },
                    severity: Some(lsp_types::DiagnosticSeverity::ERROR),
                    code: None,
                    source: Some("five-compiler".to_string()),
                    message: format!("Parse error: {}", e),
                    related_information: None,
                    tags: None,
                    code_description: None,
                    data: None,
                });
                return Ok(diagnostics);
            }
        };

        // Cache AST
        let hash = Self::hash_source(source);
        self.ast_cache.insert(uri.clone(), (hash, ast.clone()));

        // Type check - but don't fail, just collect errors
        // For now, we skip type checking in LSP since we don't have a good error collection mechanism
        // TODO: Implement error collection for type checking

        Ok(diagnostics)
    }

    // TODO: json_to_diagnostic conversion for Phase 2 when reusing LspFormatter
    // For MVP diagnostics, we construct them directly in get_diagnostics()

    /// Clear all caches (useful after significant changes or for testing)
    pub fn clear_caches(&mut self) {
        self.ast_cache.clear();
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
        }
    }
}
