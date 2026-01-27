//! Workspace symbols provider for symbol search (Ctrl+T)
//!
//! Enables quick symbol search across the entire workspace,
//! allowing users to jump to any function, variable, account, or type definition.

use lsp_types::{Location, Position, Range, SymbolInformation, SymbolKind, Url};

/// Search for symbols matching a query across the entire workspace
///
/// Returns all matching symbols (functions, variables, accounts, types) in the current file.
/// In a full multi-file implementation, this would search across all files.
pub fn workspace_symbols(
    source: &str,
    query: &str,
    uri: &Url,
) -> Vec<SymbolInformation> {
    let mut symbols = Vec::new();

    if query.is_empty() {
        return symbols;
    }

    let lines: Vec<&str> = source.lines().collect();
    let query_lower = query.to_lowercase();

    for (line_idx, line) in lines.iter().enumerate() {
        // Skip comments
        if line.trim_start().starts_with("//") {
            continue;
        }

        // Find instruction/function definitions
        if let Some(pos) = find_symbol_definition(line, "instruction") {
            if let Some(name) = extract_symbol_name(line, pos + "instruction ".len()) {
                if name.to_lowercase().contains(&query_lower) {
                    symbols.push(SymbolInformation {
                        name: name.clone(),
                        kind: SymbolKind::FUNCTION,
                        location: Location {
                            uri: uri.clone(),
                            range: Range {
                                start: Position {
                                    line: line_idx as u32,
                                    character: (pos + "instruction ".len()) as u32,
                                },
                                end: Position {
                                    line: line_idx as u32,
                                    character: (pos + "instruction ".len() + name.len()) as u32,
                                },
                            },
                        },
                        container_name: None,
                        deprecated: None,
                        tags: None,
                    });
                }
            }
        }

        // Find account definitions
        if let Some(pos) = find_symbol_definition(line, "account") {
            if let Some(name) = extract_symbol_name(line, pos + "account ".len()) {
                if name.to_lowercase().contains(&query_lower) {
                    symbols.push(SymbolInformation {
                        name: name.clone(),
                        kind: SymbolKind::STRUCT,
                        location: Location {
                            uri: uri.clone(),
                            range: Range {
                                start: Position {
                                    line: line_idx as u32,
                                    character: (pos + "account ".len()) as u32,
                                },
                                end: Position {
                                    line: line_idx as u32,
                                    character: (pos + "account ".len() + name.len()) as u32,
                                },
                            },
                        },
                        container_name: None,
                        deprecated: None,
                        tags: None,
                    });
                }
            }
        }

        // Find interface definitions
        if let Some(pos) = find_symbol_definition(line, "interface") {
            if let Some(name) = extract_symbol_name(line, pos + "interface ".len()) {
                if name.to_lowercase().contains(&query_lower) {
                    symbols.push(SymbolInformation {
                        name: name.clone(),
                        kind: SymbolKind::INTERFACE,
                        location: Location {
                            uri: uri.clone(),
                            range: Range {
                                start: Position {
                                    line: line_idx as u32,
                                    character: (pos + "interface ".len()) as u32,
                                },
                                end: Position {
                                    line: line_idx as u32,
                                    character: (pos + "interface ".len() + name.len()) as u32,
                                },
                            },
                        },
                        container_name: None,
                        deprecated: None,
                        tags: None,
                    });
                }
            }
        }

        // Find event definitions
        if let Some(pos) = find_symbol_definition(line, "event") {
            if let Some(name) = extract_symbol_name(line, pos + "event ".len()) {
                if name.to_lowercase().contains(&query_lower) {
                    symbols.push(SymbolInformation {
                        name: name.clone(),
                        kind: SymbolKind::ENUM,
                        location: Location {
                            uri: uri.clone(),
                            range: Range {
                                start: Position {
                                    line: line_idx as u32,
                                    character: (pos + "event ".len()) as u32,
                                },
                                end: Position {
                                    line: line_idx as u32,
                                    character: (pos + "event ".len() + name.len()) as u32,
                                },
                            },
                        },
                        container_name: None,
                        deprecated: None,
                        tags: None,
                    });
                }
            }
        }

        // Find let bindings (variable declarations)
        if let Some(pos) = find_symbol_definition(line, "let") {
            if let Some(name) = extract_symbol_name(line, pos + "let ".len()) {
                if name.to_lowercase().contains(&query_lower) {
                    symbols.push(SymbolInformation {
                        name: name.clone(),
                        kind: SymbolKind::VARIABLE,
                        location: Location {
                            uri: uri.clone(),
                            range: Range {
                                start: Position {
                                    line: line_idx as u32,
                                    character: (pos + "let ".len()) as u32,
                                },
                                end: Position {
                                    line: line_idx as u32,
                                    character: (pos + "let ".len() + name.len()) as u32,
                                },
                            },
                        },
                        container_name: None,
                        deprecated: None,
                        tags: None,
                    });
                }
            }
        }
    }

    symbols
}

/// Find a symbol definition keyword at the start of a line
fn find_symbol_definition(line: &str, keyword: &str) -> Option<usize> {
    let trimmed = line.trim_start();
    let indent = line.len() - trimmed.len();

    if let Some(pos) = trimmed.find(keyword) {
        if pos == 0 {
            // Verify it's a keyword (not part of a larger word)
            let after = keyword.len();
            if after >= trimmed.len() || trimmed.chars().nth(after).unwrap().is_whitespace() {
                return Some(indent);
            }
        }
    }
    None
}

/// Extract a symbol name that follows a position
fn extract_symbol_name(line: &str, start: usize) -> Option<String> {
    if start >= line.len() {
        return None;
    }

    let chars: Vec<char> = line.chars().collect();
    let mut pos = start;

    // Skip whitespace
    while pos < chars.len() && chars[pos].is_whitespace() {
        pos += 1;
    }

    if pos >= chars.len() {
        return None;
    }

    // Extract identifier
    let name_start = pos;
    while pos < chars.len() && (chars[pos].is_alphanumeric() || chars[pos] == '_') {
        pos += 1;
    }

    if name_start < pos {
        Some(chars[name_start..pos].iter().collect())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_function_symbols() {
        let source = "pub instruction my_function() {}\ninstruction helper() {}";
        let uri = Url::parse("file:///test.v").unwrap();
        let symbols = workspace_symbols(source, "function", &uri);

        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "my_function");
        assert_eq!(symbols[0].kind, SymbolKind::FUNCTION);
    }

    #[test]
    fn test_find_account_symbols() {
        let source = "account Counter { value: u64 }";
        let uri = Url::parse("file:///test.v").unwrap();
        let symbols = workspace_symbols(source, "counter", &uri);

        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "Counter");
        assert_eq!(symbols[0].kind, SymbolKind::STRUCT);
    }

    #[test]
    fn test_find_variable_symbols() {
        let source = "let counter = 0;\nlet helper = 5;";
        let uri = Url::parse("file:///test.v").unwrap();
        let symbols = workspace_symbols(source, "counter", &uri);

        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "counter");
        assert_eq!(symbols[0].kind, SymbolKind::VARIABLE);
    }

    #[test]
    fn test_empty_query_returns_empty() {
        let source = "instruction test() {}";
        let uri = Url::parse("file:///test.v").unwrap();
        let symbols = workspace_symbols(source, "", &uri);

        assert_eq!(symbols.len(), 0);
    }

    #[test]
    fn test_case_insensitive_search() {
        let source = "instruction MyFunction() {}";
        let uri = Url::parse("file:///test.v").unwrap();

        // Search with lowercase
        let symbols = workspace_symbols(source, "myfunction", &uri);
        assert_eq!(symbols.len(), 1);

        // Search with uppercase
        let symbols = workspace_symbols(source, "MYFUNCTION", &uri);
        assert_eq!(symbols.len(), 1);
    }
}
