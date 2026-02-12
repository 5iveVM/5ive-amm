//! Position-based AST Query Engine
//!
//! Provides utilities for finding AST nodes at cursor positions,
//! extracting symbols, and navigating the AST for LSP features.
//!
//! Note: This module is intentionally simplified to work with the current
//! AST structure. Full AST traversal is deferred pending AST API updates.

use five_dsl_compiler::ast::AstNode;

/// Extract the symbol name under the cursor
///
/// This is a text-based heuristic for quickly finding the identifier
/// at a given position without full AST traversal.
///
/// # Arguments
/// * `source` - The source code
/// * `line` - 0-indexed line number
/// * `column` - 0-indexed character offset
///
/// # Returns
/// The identifier at the position, or None if not on an identifier
pub fn symbol_under_cursor(source: &str, line: u32, column: u32) -> Option<String> {
    let lines: Vec<&str> = source.lines().collect();
    let line_text = lines.get(line as usize)?;

    // Find the start and end of the identifier at the column
    let chars: Vec<char> = line_text.chars().collect();
    if column as usize >= chars.len() {
        return None;
    }

    let char_at_pos = chars[column as usize];
    if !is_identifier_char(char_at_pos) {
        return None;
    }

    // Find the start of the identifier
    let mut start = column as usize;
    while start > 0 && is_identifier_char(chars[start - 1]) {
        start -= 1;
    }

    // Find the end of the identifier
    let mut end = column as usize + 1;
    while end < chars.len() && is_identifier_char(chars[end]) {
        end += 1;
    }

    Some(chars[start..end].iter().collect())
}

/// Check if a character is valid in an identifier
fn is_identifier_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Find the AST node at a given position (line, column)
///
/// TODO: Implement full AST traversal once AST location info is available
pub fn ast_node_at_position(_ast: &AstNode, _line: u32, _column: u32) -> Option<&AstNode> {
    None
}

/// Find the enclosing function node for a given position
///
/// TODO: Implement function lookup once AST structure is stable
pub fn enclosing_function(_ast: &AstNode, _line: u32, _column: u32) -> Option<&AstNode> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_under_cursor() {
        let source = "let myVar = 5;\nlet x = myVar + 1;";

        // Test on "myVar" in first line
        assert_eq!(symbol_under_cursor(source, 0, 4), Some("myVar".to_string()));
        assert_eq!(symbol_under_cursor(source, 0, 5), Some("myVar".to_string()));
        assert_eq!(symbol_under_cursor(source, 0, 8), Some("myVar".to_string()));

        // Test on "myVar" in second line
        assert_eq!(symbol_under_cursor(source, 1, 8), Some("myVar".to_string()));

        // Test on whitespace
        assert_eq!(symbol_under_cursor(source, 0, 0), Some("let".to_string()));
        assert_eq!(symbol_under_cursor(source, 0, 3), None); // Space

        // Test on number
        assert_eq!(symbol_under_cursor(source, 0, 12), Some("5".to_string()));
    }

    #[test]
    fn test_is_identifier_char() {
        assert!(is_identifier_char('a'));
        assert!(is_identifier_char('Z'));
        assert!(is_identifier_char('_'));
        assert!(is_identifier_char('0'));
        assert!(!is_identifier_char(' '));
        assert!(!is_identifier_char('='));
        assert!(!is_identifier_char('+'));
    }
}
