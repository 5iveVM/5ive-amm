//! Go-to-definition provider for navigation
//!
//! Allows users to jump to function/type definitions via Ctrl+Click or keyboard shortcut.
//!
//! Uses semantic analysis from the compiler's type checker to provide accurate,
//! scope-aware navigation that correctly handles shadowing and nested scopes.

use crate::bridge::CompilerBridge;
use lsp_types::{Location, Position, Range, Url};

/// Get the definition location for a symbol at the given position
///
/// # Arguments
/// * `bridge` - Compiler bridge for semantic analysis
/// * `uri` - File URI
/// * `source` - Source code
/// * `line` - 0-indexed line number
/// * `character` - 0-indexed character position
///
/// # Returns
/// Location of the definition if found, None otherwise
pub fn get_definition(
    bridge: &mut CompilerBridge,
    uri: &Url,
    source: &str,
    line: u32,
    character: u32,
) -> Option<Location> {
    // Extract identifier at cursor position
    let identifier = extract_identifier_at_position(source, line as usize, character as usize)?;

    // Try semantic analysis first
    if let Some(def_info) = bridge.get_definition(uri, source, &identifier) {
        // If we have location info from semantic analysis, use it
        if let Some(loc) = def_info.location {
            return Some(Location {
                uri: uri.clone(),
                range: Range {
                    start: Position {
                        line: loc.line,
                        character: loc.column,
                    },
                    end: Position {
                        line: loc.line,
                        character: loc.column.saturating_add(loc.length),
                    },
                },
            });
        }
        // If we found the symbol but don't have location info, fall back to text search
    }

    // Fallback: Use text-based search when semantic location info is unavailable
    // This is a temporary measure while position tracking is added to the AST
    find_definition_by_text(source, &identifier, uri)
}

/// Fallback text-based definition search
/// Used when semantic analysis finds a symbol but doesn't have position information
/// Prioritizes definitions over uses
fn find_definition_by_text(source: &str, identifier: &str, uri: &Url) -> Option<Location> {
    let lines: Vec<&str> = source.lines().collect();

    // First pass: Look for explicit definitions (higher priority)
    for (line_idx, line) in lines.iter().enumerate() {
        // Skip comments
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") {
            continue;
        }

        // Look for definition patterns in order of specificity
        let definition_patterns = vec![
            format!("pub {}(", identifier),    // pub function(
            format!("pub {} :", identifier),   // pub field :
            format!("mut {} :", identifier),   // mut field :
            format!("let {} ", identifier),    // let variable
            format!("let {}=", identifier),    // let variable=
            format!("{} : ", identifier),      // parameter : (in function signature)
            format!("account {}", identifier), // account definition
        ];

        for pattern in definition_patterns {
            if let Some(pos) = line.find(&pattern) {
                if pos == 0
                    || (pos > 0 && !is_identifier_char(line.chars().nth(pos - 1).unwrap_or(' ')))
                {
                    let identifier_pos = pos + pattern.find(identifier).unwrap_or(0);
                    return Some(Location {
                        uri: uri.clone(),
                        range: Range {
                            start: Position {
                                line: line_idx as u32,
                                character: identifier_pos as u32,
                            },
                            end: Position {
                                line: line_idx as u32,
                                character: (identifier_pos + identifier.len()) as u32,
                            },
                        },
                    });
                }
            }
        }
    }

    // Second pass: Look for any occurrence (fallback)
    for (line_idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") {
            continue;
        }

        let mut search_pos = 0;
        while let Some(pos) = line[search_pos..].find(identifier) {
            let actual_pos = search_pos + pos;

            // Check word boundaries
            let before_ok = actual_pos == 0
                || !is_identifier_char(line.chars().nth(actual_pos - 1).unwrap_or(' '));
            let after_ok = actual_pos + identifier.len() >= line.len()
                || !is_identifier_char(
                    line.chars()
                        .nth(actual_pos + identifier.len())
                        .unwrap_or(' '),
                );

            if before_ok && after_ok {
                return Some(Location {
                    uri: uri.clone(),
                    range: Range {
                        start: Position {
                            line: line_idx as u32,
                            character: actual_pos as u32,
                        },
                        end: Position {
                            line: line_idx as u32,
                            character: (actual_pos + identifier.len()) as u32,
                        },
                    },
                });
            }

            search_pos = actual_pos + identifier.len();
        }
    }

    None
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bridge::CompilerBridge;

    #[test]
    fn test_extract_identifier_simple() {
        let source = "pub increment() {}";
        let identifier = extract_identifier_at_position(source, 0, 5); // At 'increment'
        assert_eq!(identifier, Some("increment".to_string()));
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
        let source = "pub increment() {}";
        let identifier = extract_identifier_at_position(source, 0, 0); // At 'p' in pub
        assert_eq!(identifier, Some("pub".to_string()));
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

    #[test]
    fn test_goto_definition_with_shadowing() {
        // Test that goto-definition respects scope and finds the correct shadowed variable
        let source = r#"mut counter: u64;

pub increment() {
    let counter = 5;
    counter = counter + 1;
}"#;

        let uri = Url::parse("file:///test.v").unwrap();
        let mut bridge = CompilerBridge::new();

        // At the assignment "counter = counter + 1" on line 4,
        // goto-definition of 'counter' should find the local variable on line 3, not global on line 0
        let location = get_definition(&mut bridge, &uri, source, 4, 4);

        assert!(
            location.is_some(),
            "Should find definition for shadowed counter"
        );
        if let Some(loc) = location {
            // Should point to line 3 (the local counter definition)
            assert_eq!(
                loc.range.start.line, 3,
                "Should find local counter on line 3, not global"
            );
        }
    }

    #[test]
    fn test_goto_definition_global_variable() {
        let source = r#"mut total: u64;

pub get_total() -> u64 {
    return total;
}"#;

        let uri = Url::parse("file:///test.v").unwrap();
        let mut bridge = CompilerBridge::new();

        // goto-definition of 'total' in return statement
        let location = get_definition(&mut bridge, &uri, source, 3, 11);

        assert!(
            location.is_some(),
            "Should find definition for global variable"
        );
        if let Some(loc) = location {
            assert_eq!(
                loc.range.start.line, 0,
                "Should find global variable definition"
            );
        }
    }

    #[test]
    fn test_goto_definition_function_parameter() {
        let source = r#"pub increment(value: u64) -> u64 {
    let result = value + 1;
    return result;
}"#;

        let uri = Url::parse("file:///test.v").unwrap();
        let mut bridge = CompilerBridge::new();

        // goto-definition of 'value' in the expression "value + 1"
        let location = get_definition(&mut bridge, &uri, source, 1, 18);

        assert!(
            location.is_some(),
            "Should find definition for function parameter"
        );
        if let Some(loc) = location {
            assert_eq!(
                loc.range.start.line, 0,
                "Should find parameter on function definition line"
            );
        }
    }
}
