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
    // Check if we're in a constraint annotation context (after '@')
    if let Some(constraint_suggestions) = try_get_constraint_suggestions(source, line, character) {
        return CompletionList {
            is_incomplete: false,
            items: constraint_suggestions,
        };
    }

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

/// Try to get constraint annotation suggestions if cursor is after '@'
///
/// Returns Some(Vec) if in constraint context, None otherwise
fn try_get_constraint_suggestions(source: &str, line: usize, character: usize) -> Option<Vec<CompletionItem>> {
    let lines: Vec<&str> = source.lines().collect();

    if line >= lines.len() {
        return None;
    }

    let line_str = lines[line];
    let chars: Vec<char> = line_str.chars().collect();

    // Allow character == chars.len() (cursor at end of line)
    if character > chars.len() || character == 0 || chars.is_empty() {
        return None;
    }

    // Look backwards from cursor position to find '@'
    // If character == chars.len(), start from the last character
    let mut pos = if character == chars.len() { character - 1 } else { character - 1 };

    // Skip alphanumeric characters (partial constraint name after @)
    while pos > 0 && chars[pos].is_alphanumeric() {
        pos -= 1;
    }

    // Check if we found '@'
    if chars[pos] == '@' {
        // Extract the partial constraint name after '@'
        let start = pos + 1;
        let end = character.min(chars.len());
        let partial: String = chars[start..end].iter().collect();
        return Some(get_constraint_suggestions(&partial));
    }

    None
}

/// Get constraint annotation suggestions
///
/// Returns completion items for Five DSL constraint annotations
fn get_constraint_suggestions(prefix: &str) -> Vec<CompletionItem> {
    let constraints = vec![
        (
            "@signer",
            "Requires the account to be a signer of the transaction",
            "Required for accounts that must authorize the transaction"
        ),
        (
            "@mut",
            "Marks the account as mutable/writable",
            "Required for accounts that will be modified during execution"
        ),
        (
            "@init",
            "Initializes a new account",
            "Creates and initializes a new account. Syntax: @init(payer=<account>, space=<bytes>)"
        ),
        (
            "@writable",
            "Alias for @mut - marks account as writable",
            "Alternate syntax for @mut constraint"
        ),
    ];

    constraints
        .into_iter()
        .filter(|(name, _, _)| {
            // Filter by prefix, removing '@' for comparison
            let constraint_name = name.strip_prefix('@').unwrap_or(name);
            constraint_name.starts_with(prefix)
        })
        .map(|(name, detail, doc)| CompletionItem {
            label: name.to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some(detail.to_string()),
            documentation: Some(lsp_types::Documentation::String(doc.to_string())),
            insert_text: Some(name.to_string()),
            ..Default::default()
        })
        .collect()
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
    bridge: &CompilerBridge,
    source: &str,
    uri: &lsp_types::Url,
    prefix: &str,
) -> Vec<CompletionItem> {
    let mut suggestions = Vec::new();

    // Get all symbols from the compiled symbol table
    // The bridge maintains a cached symbol table after successful compilation
    if let Some(symbols) = bridge.get_all_symbols(uri, source) {
        suggestions.extend(
            symbols
                .iter()
                .filter(|name| name.starts_with(prefix))
                .map(|name| CompletionItem {
                    label: name.clone(),
                    kind: Some(CompletionItemKind::VARIABLE),
                    detail: Some(format!("Symbol: {}", name)),
                    documentation: Some(lsp_types::Documentation::String(
                        "User-defined symbol from project".to_string(),
                    )),
                    ..Default::default()
                }),
        );
    }

    // Always include common built-in types (in case bridge is unavailable)
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

    #[test]
    fn test_constraint_suggestions_after_at_symbol() {
        let source = "pub transfer(from: account @";
        // Position 28 is at the end, right after '@'
        let suggestions = try_get_constraint_suggestions(source, 0, 28);
        assert!(suggestions.is_some());
        let items = suggestions.unwrap();
        assert!(items.iter().any(|i| i.label == "@signer"));
        assert!(items.iter().any(|i| i.label == "@mut"));
        assert!(items.iter().any(|i| i.label == "@init"));
        assert!(items.iter().any(|i| i.label == "@writable"));
    }

    #[test]
    fn test_constraint_suggestions_partial_match() {
        let source = "pub transfer(from: account @si";
        // Position 30 is at the end, after '@si'
        let suggestions = try_get_constraint_suggestions(source, 0, 30);
        assert!(suggestions.is_some());
        let items = suggestions.unwrap();
        assert!(items.iter().any(|i| i.label == "@signer"));
        assert!(!items.iter().any(|i| i.label == "@mut")); // Should not match 'si'
    }

    #[test]
    fn test_constraint_suggestions_has_documentation() {
        let constraints = get_constraint_suggestions("");
        assert!(!constraints.is_empty());
        for constraint in &constraints {
            assert!(constraint.detail.is_some(), "Constraint {} missing detail", constraint.label);
            assert!(constraint.documentation.is_some(), "Constraint {} missing docs", constraint.label);
        }
    }

    #[test]
    fn test_no_constraint_suggestions_without_at() {
        let source = "pub transfer(from: account mut";
        // Position 31 is after 'mut' but no '@'
        let suggestions = try_get_constraint_suggestions(source, 0, 31);
        assert!(suggestions.is_none());
    }

    #[test]
    fn test_constraint_suggestions_multiple_params() {
        let source = "pub transfer(from: account @signer @mut, to: account @";
        // Position 54 is at the end, after second '@' in 'to' parameter
        let suggestions = try_get_constraint_suggestions(source, 0, 54);
        assert!(suggestions.is_some());
        let items = suggestions.unwrap();
        assert_eq!(items.len(), 4); // All 4 constraints should be suggested
    }
}
