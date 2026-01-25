//! Completion provider for code suggestions
//!
//! Provides intelligent code suggestions including:
//! - Keywords (function, let, if, pub, etc.)
//! - Variables and functions in scope
//! - Type names
//! - Context-aware filtering

use crate::bridge::CompilerBridge;
use lsp_types::{CompletionItem, CompletionItemKind, CompletionList};
use std::collections::HashSet;

/// Get completion suggestions at the given position
///
/// # Arguments
/// * `bridge` - Compiler bridge with cached symbol table
/// * `source` - Source code
/// * `line` - 0-indexed line number
/// * `character` - 0-indexed character position
/// * `uri` - File URI
///
/// # Returns
/// CompletionList with suggestions, or empty list if no context
pub fn get_completions(
    bridge: &CompilerBridge,
    source: &str,
    line: usize,
    character: usize,
    uri: &lsp_types::Url,
) -> CompletionList {
    // Extract the word being completed
    let word = extract_word_at_position(source, line, character);

    // Get all suggestions
    let mut suggestions = Vec::new();

    // Add keywords
    suggestions.extend(get_keyword_suggestions(&word));

    // Add symbols from symbol table
    suggestions.extend(get_symbol_suggestions(bridge, source, uri, &word));

    // Add type names
    suggestions.extend(get_type_suggestions(&word));

    // Remove duplicates while preserving order
    let mut seen = HashSet::new();
    suggestions.retain(|item| seen.insert(item.label.clone()));

    CompletionList {
        is_incomplete: false,
        items: suggestions,
    }
}

/// Extract the partial word being completed at the cursor position
fn extract_word_at_position(source: &str, line: usize, character: usize) -> String {
    let lines: Vec<&str> = source.lines().collect();

    if line >= lines.len() {
        return String::new();
    }

    let line_str = lines[line];
    let chars: Vec<char> = line_str.chars().collect();

    if character > chars.len() {
        return String::new();
    }

    // Find the start of the word (move backwards from cursor)
    let mut start = character;
    while start > 0 && is_completion_char(chars[start - 1]) {
        start -= 1;
    }

    // Extract the word
    let word: String = chars[start..character].iter().collect();
    word
}

/// Check if a character can be part of a completion word
fn is_completion_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Get keyword suggestions
fn get_keyword_suggestions(prefix: &str) -> Vec<CompletionItem> {
    let keywords = vec![
        ("function", "Define a function"),
        ("pub", "Public visibility"),
        ("let", "Variable binding"),
        ("mut", "Mutable binding"),
        ("if", "Conditional statement"),
        ("else", "Else clause"),
        ("for", "For loop"),
        ("while", "While loop"),
        ("return", "Return from function"),
        ("init", "Initialization block"),
        ("match", "Pattern matching"),
        ("struct", "Define a struct"),
        ("account", "Account parameter"),
        ("true", "Boolean true"),
        ("false", "Boolean false"),
    ];

    keywords
        .into_iter()
        .filter(|(name, _)| name.starts_with(prefix))
        .map(|(name, doc)| CompletionItem {
            label: name.to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some(format!("{}: {}", name, doc)),
            documentation: Some(lsp_types::Documentation::String(doc.to_string())),
            ..Default::default()
        })
        .collect()
}

/// Get symbol suggestions from the symbol table
fn get_symbol_suggestions(
    _bridge: &CompilerBridge,
    _source: &str,
    _uri: &lsp_types::Url,
    prefix: &str,
) -> Vec<CompletionItem> {
    let mut suggestions = Vec::new();

    // Try to get symbols from symbol table
    // Note: This is a simplified implementation that shows all symbols
    // In the future, we could filter by scope and context
    // TODO: Integrate with bridge.resolve_symbol() to get actual project symbols

    // For now, we'll suggest common types and built-ins
    let common_types = vec![
        ("u64", "Unsigned 64-bit integer", CompletionItemKind::TYPE_PARAMETER),
        ("u32", "Unsigned 32-bit integer", CompletionItemKind::TYPE_PARAMETER),
        ("u8", "Unsigned 8-bit integer", CompletionItemKind::TYPE_PARAMETER),
        ("bool", "Boolean type", CompletionItemKind::TYPE_PARAMETER),
        ("string", "String type", CompletionItemKind::TYPE_PARAMETER),
        ("pubkey", "Solana public key", CompletionItemKind::TYPE_PARAMETER),
    ];

    suggestions.extend(
        common_types
            .into_iter()
            .filter(|(name, _, _)| name.starts_with(prefix))
            .map(|(name, doc, kind)| CompletionItem {
                label: name.to_string(),
                kind: Some(kind),
                detail: Some(doc.to_string()),
                documentation: Some(lsp_types::Documentation::String(doc.to_string())),
                ..Default::default()
            }),
    );

    suggestions
}

/// Get type name suggestions
fn get_type_suggestions(prefix: &str) -> Vec<CompletionItem> {
    vec![
        ("Option", "Optional value", CompletionItemKind::TYPE_PARAMETER),
        ("Result", "Result type", CompletionItemKind::TYPE_PARAMETER),
        ("Vec", "Vector/Array type", CompletionItemKind::TYPE_PARAMETER),
    ]
    .into_iter()
    .filter(|(name, _, _)| name.starts_with(prefix))
    .map(|(name, doc, kind)| CompletionItem {
        label: name.to_string(),
        kind: Some(kind),
        detail: Some(doc.to_string()),
        documentation: Some(lsp_types::Documentation::String(doc.to_string())),
        ..Default::default()
    })
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_word_simple() {
        let source = "let x = 5;";
        // Position 6 is after "let x"
        let word = extract_word_at_position(source, 0, 6);
        assert_eq!(word, "");
    }

    #[test]
    fn test_extract_word_partial() {
        let source = "function";
        // Position 4 is after "func"
        let word = extract_word_at_position(source, 0, 4);
        assert_eq!(word, "func");
    }

    #[test]
    fn test_extract_word_at_space_returns_previous_word() {
        let source = "let x = 5;";
        // Position 5 is the space after 'x'
        // The function correctly walks back to find 'x'
        let word = extract_word_at_position(source, 0, 5);
        assert_eq!(word, "x");
    }

    #[test]
    fn test_keyword_filtering() {
        let keywords = get_keyword_suggestions("fu");
        assert!(keywords.iter().any(|k| k.label == "function"));
        assert!(!keywords.iter().any(|k| k.label == "let"));
    }

    #[test]
    fn test_keyword_function_has_details() {
        let keywords = get_keyword_suggestions("fun");
        let function_keyword = keywords.iter().find(|k| k.label == "function");
        assert!(function_keyword.is_some());
        assert!(function_keyword.unwrap().detail.is_some());
        assert!(function_keyword.unwrap().documentation.is_some());
    }

    #[test]
    fn test_type_suggestions_option() {
        let types = get_type_suggestions("Opt");
        assert!(types.iter().any(|t| t.label == "Option"));
    }

    #[test]
    fn test_no_duplicates_in_keyword_suggestions() {
        let keywords = get_keyword_suggestions("");
        let labels: Vec<_> = keywords.iter().map(|k| k.label.clone()).collect();
        assert_eq!(labels.len(), labels.iter().collect::<std::collections::HashSet<_>>().len());
    }
}
