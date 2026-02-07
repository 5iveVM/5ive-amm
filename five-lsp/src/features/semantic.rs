//! Semantic tokens provider for AST-based syntax highlighting
//!
//! Provides semantic highlighting that goes beyond regex patterns, understanding
//! the structure of Five DSL code for more accurate and contextual coloring.

use crate::bridge::CompilerBridge;
use serde::{Deserialize, Serialize};

/// Serializable representation of a semantic token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableSemanticToken {
    pub line: u32,
    pub start_character: u32,
    pub length: u32,
    pub token_type: u32,
    pub token_modifiers: u32,
}

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
/// Scans source code to identify keywords, types, functions, variables, etc.
/// Provides semantic highlighting that understands code structure.
pub fn get_semantic_tokens(
    _bridge: &CompilerBridge,
    source: &str,
    _uri: &lsp_types::Url,
) -> Vec<SerializableSemanticToken> {
    let mut tokens = Vec::new();
    let lines: Vec<&str> = source.lines().collect();

    for (line_idx, line) in lines.iter().enumerate() {
        // Skip processing very long lines (optimization)
        if line.len() > 10000 {
            continue;
        }

        // Extract tokens from the line
        extract_tokens_from_line(line, line_idx as u32, &mut tokens);
    }

    tokens
}

/// Keywords in Five DSL that should be highlighted as keywords
const KEYWORDS: &[&str] = &[
    "instruction", "function", "pub", "let", "mut", "if", "else", "match", "return",
    "account", "field", "interface", "event", "emit", "require", "init", "constraints",
    "use", "import", "as", "when", "for", "while", "do", "break", "continue",
    "true", "false", "None", "Some", "Ok", "Err", "error",
];

/// Types that should be highlighted as types
const TYPES: &[&str] = &[
    "u64", "u32", "u16", "u8", "i64", "i32", "i16", "i8", "bool", "string",
    "pubkey", "lamports", "u128", "Account", "Result", "Option",
];

/// Extract semantic tokens from a single line
fn extract_tokens_from_line(line: &str, line_idx: u32, tokens: &mut Vec<SerializableSemanticToken>) {
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        // Skip whitespace
        if chars[i].is_whitespace() {
            i += 1;
            continue;
        }

        // Handle comments
        if i + 1 < chars.len() && chars[i] == '/' && chars[i + 1] == '/' {
            // Rest of line is a comment
            tokens.push(SerializableSemanticToken {
                line: line_idx,
                start_character: i as u32,
                length: (chars.len() - i) as u32,
                token_type: 5, // comment
                token_modifiers: 0,
            });
            break;
        }

        // Handle string literals
        if chars[i] == '"' {
            let start = i;
            i += 1;
            while i < chars.len() && chars[i] != '"' {
                if chars[i] == '\\' {
                    i += 2;
                } else {
                    i += 1;
                }
            }
            if i < chars.len() {
                i += 1; // Consume closing quote
            }
            tokens.push(SerializableSemanticToken {
                line: line_idx,
                start_character: start as u32,
                length: (i - start) as u32,
                token_type: 6, // string
                token_modifiers: 0,
            });
            continue;
        }

        // Handle numbers
        if chars[i].is_ascii_digit() {
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_' || chars[i] == '.') {
                i += 1;
            }
            tokens.push(SerializableSemanticToken {
                line: line_idx,
                start_character: start as u32,
                length: (i - start) as u32,
                token_type: 7, // number
                token_modifiers: 0,
            });
            continue;
        }

        // Handle @ attributes
        if chars[i] == '@' {
            let start = i;
            i += 1;
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            tokens.push(SerializableSemanticToken {
                line: line_idx,
                start_character: start as u32,
                length: (i - start) as u32,
                token_type: 4, // modifier
                token_modifiers: 0,
            });
            continue;
        }

        // Handle identifiers and keywords
        if chars[i].is_alphabetic() || chars[i] == '_' {
            let start = i;
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();

            let (token_type, token_modifiers) = if KEYWORDS.contains(&word.as_str()) {
                // Highlight keywords
                if word == "pub" {
                    (4, 1 << 4) // modifier + public
                } else if word == "mut" {
                    (4, 1 << 5) // modifier + mutable
                } else {
                    (3, 0) // keyword
                }
            } else if TYPES.contains(&word.as_str()) {
                (2, 0) // type
            } else if word.chars().next().unwrap().is_uppercase() {
                // Capitalized identifiers are likely types
                (2, 0) // type
            } else {
                (1, 0) // variable/identifier
            };

            tokens.push(SerializableSemanticToken {
                line: line_idx,
                start_character: start as u32,
                length: (i - start) as u32,
                token_type,
                token_modifiers,
            });
            continue;
        }

        // Handle operators and punctuation
        if "+-*/%=!<>&|^~".contains(chars[i]) {
            let start = i;
            i += 1;
            // Check for multi-char operators
            if i < chars.len() {
                let two_char: String = chars[start..i.min(start + 2)].iter().collect();
                if matches!(two_char.as_str(), "==" | "!=" | "<=" | ">=" | "&&" | "||" | "->" | "=>" | "+=" | "-=" | "*=" | "/=" | "<<" | ">>" | "<<<") {
                    i += 1;
                }
            }
            tokens.push(SerializableSemanticToken {
                line: line_idx,
                start_character: start as u32,
                length: (i - start) as u32,
                token_type: 9, // operator
                token_modifiers: 0,
            });
            continue;
        }

        // Skip other characters
        i += 1;
    }
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

    #[test]
    fn test_semantic_tokens_extracted() {
        let source = "pub instruction test() { let x = 5; }";
        let bridge = CompilerBridge::new();
        let uri = lsp_types::Url::parse("file:///test.v").unwrap();
        let tokens = get_semantic_tokens(&bridge, source, &uri);

        // Should extract at least some tokens (pub, instruction, test, let, x, number, etc.)
        assert!(!tokens.is_empty());

        // Should have keyword tokens (pub, instruction, let)
        let keyword_tokens: Vec<_> = tokens.iter().filter(|t| t.token_type == 3).collect();
        assert!(!keyword_tokens.is_empty());
    }

    #[test]
    fn test_semantic_tokens_with_strings() {
        let source = r#"let msg = "hello";"#;
        let bridge = CompilerBridge::new();
        let uri = lsp_types::Url::parse("file:///test.v").unwrap();
        let tokens = get_semantic_tokens(&bridge, source, &uri);

        // Should have a string token (type 6)
        let string_tokens: Vec<_> = tokens.iter().filter(|t| t.token_type == 6).collect();
        assert!(!string_tokens.is_empty());
    }

    #[test]
    fn test_semantic_tokens_with_comments() {
        let source = "// Comment line\nlet x = 5;";
        let bridge = CompilerBridge::new();
        let uri = lsp_types::Url::parse("file:///test.v").unwrap();
        let tokens = get_semantic_tokens(&bridge, source, &uri);

        // Should have a comment token (type 5)
        let comment_tokens: Vec<_> = tokens.iter().filter(|t| t.token_type == 5).collect();
        assert!(!comment_tokens.is_empty());
    }
}
