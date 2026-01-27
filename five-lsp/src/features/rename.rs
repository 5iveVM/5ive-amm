//! Rename refactoring provider
//!
//! Enables safe renaming of symbols across all their usages in the document.

use lsp_types::{Range, Position, WorkspaceEdit, TextEdit};
use std::collections::HashMap;

/// Prepare a rename operation
///
/// Returns the current name of the symbol if it can be renamed.
pub fn prepare_rename(source: &str, line: usize, character: usize) -> Option<String> {
    // Extract the identifier at the position
    let lines: Vec<&str> = source.lines().collect();

    if line >= lines.len() {
        return None;
    }

    let line_str = lines[line];
    let chars: Vec<char> = line_str.chars().collect();

    if character > chars.len() {
        return None;
    }

    // Check if cursor is on an identifier character
    if character >= chars.len() || !is_identifier_char(chars[character]) {
        return None;
    }

    // Find the start of the identifier
    let mut start = character;
    while start > 0 && is_identifier_char(chars[start - 1]) {
        start -= 1;
    }

    // Find the end of the identifier
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

/// Rename a symbol across all its usages
///
/// Finds all references to a symbol and replaces them with the new name.
pub fn rename(
    source: &str,
    line: usize,
    character: usize,
    new_name: &str,
    uri: &lsp_types::Url,
) -> Option<WorkspaceEdit> {
    // Get the current name
    let old_name = prepare_rename(source, line, character)?;

    // Validate new name
    if !is_valid_identifier(new_name) {
        return None;
    }

    // Find all occurrences of the old name
    let lines: Vec<&str> = source.lines().collect();
    let mut text_edits = Vec::new();

    for (line_idx, line_str) in lines.iter().enumerate() {
        // Skip comment lines
        if line_str.trim_start().starts_with("//") {
            continue;
        }

        let mut search_pos = 0;

        while let Some(col) = line_str[search_pos..].find(&old_name) {
            let actual_col = search_pos + col;

            // Skip if inside a string literal
            if is_in_string_literal(line_str, actual_col) {
                search_pos = actual_col + old_name.len();
                continue;
            }

            let chars: Vec<char> = line_str.chars().collect();

            // Check word boundaries
            let is_valid_before = actual_col == 0 || !is_identifier_char(chars[actual_col - 1]);
            let end_pos = actual_col + old_name.len();
            let is_valid_after = end_pos >= chars.len() || !is_identifier_char(chars[end_pos]);

            // Only replace if word boundaries are valid
            if is_valid_before && is_valid_after {
                text_edits.push(TextEdit {
                    range: Range {
                        start: Position {
                            line: line_idx as u32,
                            character: actual_col as u32,
                        },
                        end: Position {
                            line: line_idx as u32,
                            character: (actual_col + old_name.len()) as u32,
                        },
                    },
                    new_text: new_name.to_string(),
                });
            }

            // Move search position forward
            search_pos = actual_col + old_name.len();
        }
    }

    // Build workspace edit
    let mut changes = HashMap::new();
    changes.insert(uri.clone(), text_edits);

    Some(WorkspaceEdit {
        changes: Some(changes),
        change_annotations: None,
        document_changes: None,
    })
}

/// Check if a character is valid in an identifier
fn is_identifier_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
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

/// Check if a string is a valid identifier
fn is_valid_identifier(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    // Must start with letter or underscore
    let first_char = name.chars().next().unwrap();
    if !first_char.is_alphabetic() && first_char != '_' {
        return false;
    }

    // Rest must be alphanumeric or underscore
    name.chars().skip(1).all(|c| c.is_alphanumeric() || c == '_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prepare_rename() {
        let source = "let counter = 5;";
        let name = prepare_rename(source, 0, 6); // At 'counter'
        assert_eq!(name, Some("counter".to_string()));
    }

    #[test]
    fn test_prepare_rename_space_returns_none() {
        let source = "let counter = 5;";
        let name = prepare_rename(source, 0, 3); // At space
        assert_eq!(name, None);
    }

    #[test]
    fn test_is_valid_identifier() {
        assert!(is_valid_identifier("my_var"));
        assert!(is_valid_identifier("_private"));
        assert!(is_valid_identifier("Counter"));
        assert!(!is_valid_identifier("123invalid"));
        assert!(!is_valid_identifier("my-var"));
        assert!(!is_valid_identifier(""));
    }

    #[test]
    fn test_rename_single_occurrence() {
        let source = "let x = 5;";
        let uri = "file:///test.v".parse().unwrap();
        let edit = rename(source, 0, 4, "y", &uri);
        assert!(edit.is_some());
        let edit = edit.unwrap();
        assert!(edit.changes.is_some());
    }

    #[test]
    fn test_rename_multiple_occurrences() {
        let source = "let x = 5;\nlet y = x + 1;";
        let uri = "file:///test.v".parse().unwrap();
        let edit = rename(source, 0, 4, "z", &uri);
        assert!(edit.is_some());
        let edit = edit.unwrap();
        let changes = edit.changes.unwrap();
        let edits = changes.get(&uri).unwrap();
        assert_eq!(edits.len(), 2); // Two occurrences of 'x'
    }

    #[test]
    fn test_rename_word_boundary_respected() {
        let source = "let counter = 1;\nlet my_counter = 2;";
        let uri = "file:///test.v".parse().unwrap();
        let edit = rename(source, 0, 4, "x", &uri); // Rename first 'counter'
        assert!(edit.is_some());
        let edit = edit.unwrap();
        let changes = edit.changes.unwrap();
        let edits = changes.get(&uri).unwrap();
        assert_eq!(edits.len(), 1); // Only the first 'counter', not part of 'my_counter'
    }
}
