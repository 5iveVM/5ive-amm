//! Semantic tokens provider for AST-based syntax highlighting
//!
//! Provides semantic highlighting that goes beyond regex patterns, understanding
//! the structure of Five DSL code for more accurate and contextual coloring.

use crate::bridge::CompilerBridge;
use lsp_types::SemanticToken;

/// Semantic token types for Five DSL
pub const SEMANTIC_TOKEN_TYPES: &[&str] = &[
    "function",      // 0: function declarations
    "variable",      // 1: variable declarations
    "type",          // 2: type references
    "keyword",       // 3: language keywords
    "modifier",      // 4: pub, mut, etc.
    "comment",       // 5: comments
    "string",        // 6: string literals
    "number",        // 7: numeric literals
    "account",       // 8: account parameters
    "operator",      // 9: operators
];

/// Semantic token modifiers for Five DSL
pub const SEMANTIC_TOKEN_MODIFIERS: &[&str] = &[
    "declaration",   // 0: declarations
    "definition",    // 1: definitions
    "readonly",      // 2: immutable binding
    "deprecated",    // 3: deprecated items
    "public",        // 4: public visibility
    "mutable",       // 5: mutable binding
];

/// Get semantic tokens for highlighting
///
/// MVP implementation: returns empty tokens for now.
/// In future, walks the AST to extract semantic information about tokens,
/// enabling syntax-aware highlighting that understands the code structure.
pub fn get_semantic_tokens(
    _bridge: &CompilerBridge,
    _source: &str,
    _uri: &lsp_types::Url,
) -> Vec<SemanticToken> {
    // MVP: Return empty tokens
    // Future: Parse AST and extract semantic information
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semantic_token_types_defined() {
        assert_eq!(SEMANTIC_TOKEN_TYPES[0], "function");
        assert_eq!(SEMANTIC_TOKEN_TYPES[1], "variable");
        assert_eq!(SEMANTIC_TOKEN_TYPES[2], "type");
    }

    #[test]
    fn test_semantic_token_modifiers_defined() {
        assert_eq!(SEMANTIC_TOKEN_MODIFIERS[0], "declaration");
        assert_eq!(SEMANTIC_TOKEN_MODIFIERS[1], "definition");
        assert_eq!(SEMANTIC_TOKEN_MODIFIERS[5], "mutable");
    }
}
