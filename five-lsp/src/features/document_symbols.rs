//! Document symbols provider for outline/navigator view
//!
//! Provides a list of all top-level definitions (functions, variables)
//! in the document for quick navigation.

use crate::bridge::CompilerBridge;
use lsp_types::{DocumentSymbol, SymbolKind, Range, Position};

/// Get document symbols for outline view
///
/// Returns all top-level functions and variables that can be
/// displayed in the editor's outline panel for quick navigation.
pub fn get_document_symbols(
    _bridge: &CompilerBridge,
    source: &str,
    _uri: &lsp_types::Url,
) -> Vec<DocumentSymbol> {
    let mut symbols = Vec::new();

    // Extract symbols by searching source code directly
    // This is an MVP implementation; in future, could enhance with AST parsing
    let lines: Vec<&str> = source.lines().collect();

    for (line_idx, line) in lines.iter().enumerate() {
        // Find function declarations
        if let Some(col) = line.find("function ") {
            if let Some(name_end) = line[col + 9..].find('(') {
                let name = line[col + 9..col + 9 + name_end].trim().to_string();
                if !name.is_empty() {
                    let range = Range {
                        start: Position {
                            line: line_idx as u32,
                            character: col as u32,
                        },
                        end: Position {
                            line: line_idx as u32,
                            character: (col + 8 + name_end) as u32,
                        },
                    };
                    #[allow(deprecated)]
                    {
                        symbols.push(DocumentSymbol {
                            name,
                            detail: Some("function".to_string()),
                            kind: SymbolKind::FUNCTION,
                            deprecated: None,
                            range,
                            selection_range: range,
                            children: None,
                            tags: None,
                        });
                    }
                }
            }
        }

        // Find variable declarations
        if let Some(col) = line.find("let ") {
            if let Some(var_end) = line[col + 4..].find(|c: char| c == '=' || c == ';' || c == ' ') {
                let name = line[col + 4..col + 4 + var_end].trim().to_string();
                if !name.is_empty() && is_valid_identifier(&name) {
                    let range = Range {
                        start: Position {
                            line: line_idx as u32,
                            character: col as u32,
                        },
                        end: Position {
                            line: line_idx as u32,
                            character: (col + 3 + var_end) as u32,
                        },
                    };
                    #[allow(deprecated)]
                    {
                        symbols.push(DocumentSymbol {
                            name,
                            detail: Some("variable".to_string()),
                            kind: SymbolKind::VARIABLE,
                            deprecated: None,
                            range,
                            selection_range: range,
                            children: None,
                            tags: None,
                        });
                    }
                }
            }
        }

        // Find init block
        if line.contains("init ") {
            if let Some(col) = line.find("init") {
                let range = Range {
                    start: Position {
                        line: line_idx as u32,
                        character: col as u32,
                    },
                    end: Position {
                        line: line_idx as u32,
                        character: (col + 4) as u32,
                    },
                };
                #[allow(deprecated)]
                {
                    symbols.push(DocumentSymbol {
                        name: "init".to_string(),
                        detail: Some("initialization block".to_string()),
                        kind: SymbolKind::CONSTRUCTOR,
                        deprecated: None,
                        range,
                        selection_range: range,
                        children: None,
                        tags: None,
                    });
                }
            }
        }
    }

    symbols
}

/// Check if a string is a valid identifier
fn is_valid_identifier(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    let first_char = name.chars().next().unwrap();
    if !first_char.is_alphabetic() && first_char != '_' {
        return false;
    }
    name.chars().skip(1).all(|c| c.is_alphanumeric() || c == '_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_identifier() {
        assert!(is_valid_identifier("my_func"));
        assert!(is_valid_identifier("_private"));
        assert!(is_valid_identifier("Counter"));
        assert!(!is_valid_identifier("123invalid"));
        assert!(!is_valid_identifier("my-var"));
    }
}
