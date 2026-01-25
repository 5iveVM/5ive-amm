//! Go-to-definition provider for navigation
//!
//! Allows users to jump to function/type definitions via Ctrl+Click or keyboard shortcut.

use lsp_types::{Location, Position, Range, Url};

/// Get the definition location for a symbol at the given position
///
/// # Arguments
/// * `source` - Source code
/// * `line` - 0-indexed line number
/// * `character` - 0-indexed character position
/// * `uri` - File URI
///
/// # Returns
/// Location of the definition if found, None otherwise
pub fn get_definition(
    source: &str,
    line: usize,
    character: usize,
    uri: &Url,
) -> Option<Location> {
    // Extract identifier at cursor position
    let identifier = extract_identifier_at_position(source, line, character)?;

    // Find the definition in source code by searching for definition patterns
    let (def_line, def_char) = find_definition_in_source(source, &identifier)?;

    // Create location pointing to the definition
    Some(Location {
        uri: uri.clone(),
        range: Range {
            start: Position {
                line: def_line as u32,
                character: def_char as u32,
            },
            end: Position {
                line: def_line as u32,
                character: (def_char + identifier.len()) as u32,
            },
        },
    })
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

/// Find the definition location of an identifier in source code
///
/// Searches for definition patterns like:
/// - `pub function name(...)`
/// - `function name(...)`
/// - `account name { ... }`
/// - `pub let name = ...`
fn find_definition_in_source(source: &str, identifier: &str) -> Option<(usize, usize)> {
    let lines: Vec<&str> = source.lines().collect();

    // Search patterns for definitions
    let patterns = vec![
        format!("pub function {}", identifier),  // pub function name
        format!("function {}", identifier),      // function name
        format!("pub {}", identifier),           // pub name (field/account)
        format!("account {}", identifier),       // account name
        format!("let {} ", identifier),          // let name =
        format!("let {} =", identifier),         // let name=
    ];

    for (line_idx, line) in lines.iter().enumerate() {
        for pattern in &patterns {
            if let Some(col) = line.find(pattern) {
                // Extract the identifier position within the pattern
                let identifier_pos = if pattern.contains("pub function") {
                    col + 12  // "pub function ".len()
                } else if pattern.contains("function") {
                    col + 9   // "function ".len()
                } else if pattern.contains("pub ") {
                    col + 4   // "pub ".len()
                } else if pattern.contains("account ") {
                    col + 8   // "account ".len()
                } else if pattern.contains("let ") {
                    col + 4   // "let ".len()
                } else {
                    col
                };

                return Some((line_idx, identifier_pos));
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_extract_identifier_returns_none_on_space() {
        let source = "let x = 5;";
        let identifier = extract_identifier_at_position(source, 0, 3); // At space
        assert_eq!(identifier, None);
    }

    #[test]
    fn test_extract_identifier_at_start() {
        let source = "function my_func() {}";
        let identifier = extract_identifier_at_position(source, 0, 0); // At 'f' in function
        assert_eq!(identifier, Some("function".to_string()));
    }

    #[test]
    fn test_is_identifier_char_letter() {
        assert!(is_identifier_char('a'));
        assert!(is_identifier_char('Z'));
    }

    #[test]
    fn test_is_identifier_char_digit() {
        assert!(is_identifier_char('0'));
        assert!(is_identifier_char('9'));
    }

    #[test]
    fn test_is_identifier_char_underscore() {
        assert!(is_identifier_char('_'));
    }

    #[test]
    fn test_is_identifier_char_space() {
        assert!(!is_identifier_char(' '));
    }

    #[test]
    fn test_is_identifier_char_special() {
        assert!(!is_identifier_char('('));
        assert!(!is_identifier_char(')'));
        assert!(!is_identifier_char('{'));
    }
}
