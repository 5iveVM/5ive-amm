//! Find references provider for locating all usages of a symbol
//!
//! Allows users to find all references to a symbol in the current file.

use lsp_types::{Location, Position, Range, Url};

/// Find all references to a symbol at the given position
///
/// # Arguments
/// * `source` - Source code
/// * `line` - 0-indexed line number
/// * `character` - 0-indexed character position
/// * `uri` - File URI
///
/// # Returns
/// Vector of Locations where the symbol is referenced, including the definition
pub fn find_references(
    source: &str,
    line: usize,
    character: usize,
    uri: &Url,
) -> Vec<Location> {
    // Extract identifier at cursor position
    let identifier = match extract_identifier_at_position(source, line, character) {
        Some(id) => id,
        None => return vec![],
    };

    // Find all references to the identifier in source code
    find_references_in_source(source, &identifier, uri)
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
/// Returns all occurrences of the identifier, filtering to ensure word boundaries
/// (i.e., the identifier is not part of a larger word).
fn find_references_in_source(source: &str, identifier: &str, uri: &Url) -> Vec<Location> {
    let mut references = Vec::new();
    let lines: Vec<&str> = source.lines().collect();

    for (line_idx, line) in lines.iter().enumerate() {
        let mut search_pos = 0;

        while let Some(col) = line[search_pos..].find(identifier) {
            let actual_col = search_pos + col;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_single_reference() {
        let source = "let x = 5;\nlet y = x + 1;";
        let uri = Url::parse("file:///test.v").unwrap();
        let references = find_references(source, 0, 4, &uri); // At 'x' in first line
        assert_eq!(references.len(), 2); // Definition and one reference
    }

    #[test]
    fn test_find_multiple_references() {
        let source = "function test() {\n  test();\n  test();\n}";
        let uri = Url::parse("file:///test.v").unwrap();
        let references = find_references(source, 0, 9, &uri); // At 'test' in function name
        assert_eq!(references.len(), 3); // Definition and two calls
    }

    #[test]
    fn test_find_references_word_boundary() {
        let source = "let counter = 1;\nlet my_counter = 2;";
        let uri = Url::parse("file:///test.v").unwrap();
        let references = find_references(source, 0, 4, &uri); // At 'counter'
        assert_eq!(references.len(), 1); // Only exact matches, not "my_counter"
    }

    #[test]
    fn test_find_references_none_found() {
        let source = "let x = 5;";
        let uri = Url::parse("file:///test.v").unwrap();
        let references = find_references(source, 0, 9, &uri); // At space, no identifier
        assert_eq!(references.len(), 0);
    }

    #[test]
    fn test_find_references_account_definition() {
        let source = "account Counter {\n  value: u64,\n}\n\npub read_counter(c: account Counter) {}";
        let uri = Url::parse("file:///test.v").unwrap();
        let references = find_references(source, 0, 8, &uri); // At 'Counter'
        assert_eq!(references.len(), 2); // Definition and one type reference
    }

    #[test]
    fn test_extract_identifier_simple() {
        let source = "function my_func() {}";
        let identifier = extract_identifier_at_position(source, 0, 11); // At 'my_func'
        assert_eq!(identifier, Some("my_func".to_string()));
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
