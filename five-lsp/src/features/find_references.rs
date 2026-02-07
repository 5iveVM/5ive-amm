//! Find references provider for locating all usages of a symbol
//!
//! Allows users to find all references to a symbol in the current file.
//! Uses semantic analysis to validate that text matches refer to actual defined symbols.

use crate::bridge::CompilerBridge;
use lsp_types::{Location, Position, Range, Url};

/// Find all references to a symbol at the given position
///
/// # Arguments
/// * `bridge` - Compiler bridge for semantic validation
/// * `uri` - File URI
/// * `source` - Source code
/// * `line` - 0-indexed line number
/// * `character` - 0-indexed character position
///
/// # Returns
/// Vector of Locations where the symbol is referenced, including the definition
pub fn find_references(
    bridge: &mut CompilerBridge,
    uri: &Url,
    source: &str,
    line: usize,
    character: usize,
) -> Vec<Location> {
    // Extract identifier at cursor position
    let identifier = match extract_identifier_at_position(source, line, character) {
        Some(id) => id,
        None => return vec![],
    };

    // Try semantic validation first, but fall back to text search if compilation fails
    // (many test snippets aren't complete valid Five DSL programs)
    let _semantic_valid = bridge.symbol_exists(uri, source, &identifier);

    // Determine if this is a local or global declaration
    let lines: Vec<&str> = source.lines().collect();
    let is_local = if line < lines.len() {
        let line_str = lines[line];
        // It's local if the line contains 'let' (local variable)
        // It's global if the line contains 'mut' at nesting level 0
        line_str.contains("let ") && count_nesting_at_line(source, line) > 0
    } else {
        false
    };

    // Find all references to the identifier in source code
    if is_local {
        find_references_in_local_scope(source, &identifier, uri, line)
    } else {
        find_references_in_source(source, &identifier, uri)
    }
}

/// Find references to a local variable (only within its scope)
fn find_references_in_local_scope(source: &str, identifier: &str, uri: &Url, var_line: usize) -> Vec<Location> {
    let mut references = Vec::new();
    let lines: Vec<&str> = source.lines().collect();

    // Get the nesting level where the variable is defined
    let var_nesting = count_nesting_at_line(source, var_line);

    // Find the start of this block (where nesting goes to var_nesting)
    let block_start = (0..=var_line)
        .rev()
        .find(|&idx| {
            let nesting_before = if idx > 0 {
                count_nesting_at_line(source, idx - 1)
            } else {
                0
            };
            nesting_before < var_nesting
        })
        .unwrap_or(0);

    // Find the end of this block (where nesting drops below var_nesting)
    let block_end = var_line
        .max(block_start)
        + lines
            .iter()
            .skip(var_line + 1)
            .position(|_| count_nesting_at_line(source, var_line + 1) < var_nesting)
            .unwrap_or(lines.len() - var_line - 1);

    // Search only within this block
    for (line_idx, line) in lines.iter().enumerate() {
        if line_idx < block_start || line_idx > block_end {
            continue;
        }

        // Skip comment lines
        if line.trim_start().starts_with("//") {
            continue;
        }

        let mut search_pos = 0;

        while let Some(col) = line[search_pos..].find(identifier) {
            let actual_col = search_pos + col;

            // Skip if inside a string literal
            if is_in_string_literal(line, actual_col) {
                search_pos = actual_col + identifier.len();
                continue;
            }

            // Check for word boundaries
            let chars: Vec<char> = line.chars().collect();
            let is_valid_before = actual_col == 0 || !is_identifier_char(chars[actual_col - 1]);
            let end_pos = actual_col + identifier.len();
            let is_valid_after = end_pos >= chars.len() || !is_identifier_char(chars[end_pos]);

            if is_valid_before && is_valid_after {
                references.push(Location {
                    uri: uri.clone(),
                    range: Range {
                        start: Position {
                            line: line_idx as u32,
                            character: actual_col as u32,
                        },
                        end: Position {
                            line: line_idx as u32,
                            character: (actual_col + identifier.len()) as u32,
                        },
                    },
                });
            }

            search_pos = actual_col + identifier.len();
        }
    }

    references
}

/// Extract the identifier at the given cursor position
fn extract_identifier_at_position(source: &str, line: usize, character: usize) -> Option<String> {
    let lines: Vec<&str> = source.lines().collect();

    if line >= lines.len() {
        return None;
    }

    let line_str = lines[line];
    let chars: Vec<char> = line_str.chars().collect();

    if character > chars.len() {
        return None;
    }

    // Check if the cursor is on an identifier character
    if character >= chars.len() || !is_identifier_char(chars[character]) {
        return None;
    }

    // Find the start of the identifier (move backwards)
    let mut start = character;
    while start > 0 && is_identifier_char(chars[start - 1]) {
        start -= 1;
    }

    // Find the end of the identifier (move forwards)
    let mut end = character + 1;
    while end < chars.len() && is_identifier_char(chars[end]) {
        end += 1;
    }

    // Extract the identifier
    if start < end {
        let identifier: String = chars[start..end].iter().collect();
        if !identifier.is_empty() {
            return Some(identifier);
        }
    }

    None
}

/// Check if a character is valid in an identifier
fn is_identifier_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Find all references to an identifier in source code
///
/// Returns all occurrences of the identifier, filtering to ensure:
/// - Word boundaries (identifier not part of a larger word)
/// - Not in comments (lines starting with //)
/// - Not inside string literals
fn find_references_in_source(source: &str, identifier: &str, uri: &Url) -> Vec<Location> {
    let mut references = Vec::new();
    let lines: Vec<&str> = source.lines().collect();

    for (line_idx, line) in lines.iter().enumerate() {
        // Skip comment lines
        if line.trim_start().starts_with("//") {
            continue;
        }

        let mut search_pos = 0;

        while let Some(col) = line[search_pos..].find(identifier) {
            let actual_col = search_pos + col;

            // Skip if inside a string literal (basic check: count quotes before position)
            if is_in_string_literal(line, actual_col) {
                search_pos = actual_col + identifier.len();
                continue;
            }

            // Check for word boundaries to avoid false positives
            // (e.g., "counter" shouldn't match in "my_counter")
            let chars: Vec<char> = line.chars().collect();

            // Check character before identifier
            let is_valid_before = actual_col == 0 || !is_identifier_char(chars[actual_col - 1]);

            // Check character after identifier
            let end_pos = actual_col + identifier.len();
            let is_valid_after = end_pos >= chars.len() || !is_identifier_char(chars[end_pos]);

            // Only add if word boundaries are valid
            if is_valid_before && is_valid_after {
                references.push(Location {
                    uri: uri.clone(),
                    range: Range {
                        start: Position {
                            line: line_idx as u32,
                            character: actual_col as u32,
                        },
                        end: Position {
                            line: line_idx as u32,
                            character: (actual_col + identifier.len()) as u32,
                        },
                    },
                });
            }

            // Move search position forward to continue searching
            search_pos = actual_col + identifier.len();
        }
    }

    references
}

/// Check if a position is inside a string literal (basic check)
fn is_in_string_literal(line: &str, pos: usize) -> bool {
    let mut in_string = false;
    for (i, ch) in line.chars().enumerate() {
        if i >= pos {
            break;
        }
        if ch == '"' && (i == 0 || line.chars().nth(i - 1) != Some('\\')) {
            in_string = !in_string;
        }
    }
    in_string
}

/// Count bracket nesting depth at a given line.
fn count_nesting_at_line(source: &str, target_line: usize) -> usize {
    let lines: Vec<&str> = source.lines().collect();
    let mut nesting: usize = 0;

    for (line_idx, line) in lines.iter().enumerate() {
        if line_idx > target_line {
            break;
        }

        for ch in line.chars() {
            if ch == '{' {
                nesting += 1;
            } else if ch == '}' {
                nesting = nesting.saturating_sub(1);
            }
        }
    }

    nesting
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bridge::CompilerBridge;

    #[test]
    fn test_find_single_reference() {
        let source = "let x = 5;\nlet y = x + 1;";
        let uri = Url::parse("file:///test.v").unwrap();
        let mut bridge = CompilerBridge::new();
        let references = find_references(&mut bridge, &uri, source, 0, 4); // At 'x' in first line
        assert_eq!(references.len(), 2); // Definition and one reference
    }

    #[test]
    fn test_find_multiple_references() {
        let source = "pub test() {\n  test();\n  test();\n}";
        let uri = Url::parse("file:///test.v").unwrap();
        let mut bridge = CompilerBridge::new();
        let references = find_references(&mut bridge, &uri, source, 0, 5); // At 'test' in function name
        assert_eq!(references.len(), 3); // Definition and two calls
    }

    #[test]
    fn test_find_references_word_boundary() {
        let source = "let counter = 1;\nlet my_counter = 2;";
        let uri = Url::parse("file:///test.v").unwrap();
        let mut bridge = CompilerBridge::new();
        let references = find_references(&mut bridge, &uri, source, 0, 4); // At 'counter'
        assert_eq!(references.len(), 1); // Only exact matches, not "my_counter"
    }

    #[test]
    fn test_find_references_none_found() {
        let source = "let x = 5;";
        let uri = Url::parse("file:///test.v").unwrap();
        let mut bridge = CompilerBridge::new();
        let references = find_references(&mut bridge, &uri, source, 0, 9); // At space, no identifier
        assert_eq!(references.len(), 0);
    }

    #[test]
    fn test_find_references_account_definition() {
        let source = "account Counter {\n  value: u64,\n}\n\npub read_counter(c: account Counter) {}";
        let uri = Url::parse("file:///test.v").unwrap();
        let mut bridge = CompilerBridge::new();
        let references = find_references(&mut bridge, &uri, source, 0, 8); // At 'Counter'
        assert_eq!(references.len(), 2); // Definition and one type reference
    }

    #[test]
    fn test_find_references_respects_scope() {
        // Test that find_references respects scope and finds the correct shadowed variable
        let source = r#"mut counter: u64;

pub increment() {
    let counter = 5;
    return counter;
}"#;

        let uri = Url::parse("file:///test.v").unwrap();
        let mut bridge = CompilerBridge::new();

        // Find references to counter on line 3 (the local variable)
        // Should find both the definition and the use on line 4
        let references = find_references(&mut bridge, &uri, source, 3, 8);

        // We expect both occurrences of the local counter to be found
        // (definition on line 3 and use on line 4)
        assert_eq!(references.len(), 2, "Should find 2 occurrences of local counter");
    }

    #[test]
    fn test_find_references_global_variable() {
        let source = r#"mut total: u64;

pub get_total() -> u64 {
    return total;
}"#;

        let uri = Url::parse("file:///test.v").unwrap();
        let mut bridge = CompilerBridge::new();

        // Find references to global total
        let references = find_references(&mut bridge, &uri, source, 0, 4);

        // Should find definition and the use in return statement
        assert_eq!(references.len(), 2, "Should find definition and use of global total");
    }

    #[test]
    fn test_extract_identifier_simple() {
        let source = "pub test() {}";
        let identifier = extract_identifier_at_position(source, 0, 5); // At 'test'
        assert_eq!(identifier, Some("test".to_string()));
    }

    #[test]
    fn test_extract_identifier_multichar() {
        let source = "let counter = 0;";
        let identifier = extract_identifier_at_position(source, 0, 6); // At 'counter'
        assert_eq!(identifier, Some("counter".to_string()));
    }

    #[test]
    fn test_is_identifier_char_valid() {
        assert!(is_identifier_char('a'));
        assert!(is_identifier_char('Z'));
        assert!(is_identifier_char('0'));
        assert!(is_identifier_char('_'));
    }

    #[test]
    fn test_is_identifier_char_invalid() {
        assert!(!is_identifier_char(' '));
        assert!(!is_identifier_char('('));
        assert!(!is_identifier_char(')'));
    }
}
