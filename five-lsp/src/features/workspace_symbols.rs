//! Workspace symbols provider for symbol search (Cmd+T / Ctrl+T)
//!
//! Enables quick symbol search across the workspace using semantic analysis.
//! Users can jump to any function, account, event, interface, or field definition.

use crate::bridge::CompilerBridge;
use five_dsl_compiler::ast::AstNode;
use lsp_types::{Location, Position, Range, SymbolInformation, SymbolKind, Url};

fn make_symbol_information(
    name: String,
    kind: SymbolKind,
    location: Location,
) -> SymbolInformation {
    // lsp_types::SymbolInformation still includes deprecated field; we must initialize it.
    #[allow(deprecated)]
    SymbolInformation {
        name,
        kind,
        tags: None,
        deprecated: None,
        location,
        container_name: None,
    }
}

/// Search for symbols matching a query in a file
///
/// Returns all matching symbols extracted from the AST that contain
/// the query string (case-insensitive).
pub fn workspace_symbols(
    bridge: &mut CompilerBridge,
    source: &str,
    query: &str,
    uri: &Url,
) -> Vec<SymbolInformation> {
    if query.is_empty() {
        return Vec::new();
    }

    // Parse source to AST
    let ast = match bridge.compile_to_ast(uri, source) {
        Ok(ast) => ast,
        Err(_) => {
            // On parse error, return empty
            return Vec::new();
        }
    };

    // Extract and filter symbols from AST
    extract_matching_symbols(&ast, query, uri)
}

/// Extract all symbols from AST that match the query
fn extract_matching_symbols(ast: &AstNode, query: &str, uri: &Url) -> Vec<SymbolInformation> {
    let mut symbols = Vec::new();
    let query_lower = query.to_lowercase();

    // Only extract from Program node
    if let AstNode::Program {
        instruction_definitions,
        account_definitions,
        event_definitions,
        field_definitions,
        interface_definitions,
        ..
    } = ast
    {
        // Extract functions (instructions)
        for instr in instruction_definitions {
            if let AstNode::InstructionDefinition { name, .. } = instr {
                if name.to_lowercase().contains(&query_lower) {
                    let location = Location {
                        uri: uri.clone(),
                        range: make_location_range(),
                    };

                    symbols.push(make_symbol_information(
                        name.clone(),
                        SymbolKind::FUNCTION,
                        location,
                    ));
                }
            }
        }

        // Extract accounts
        for account in account_definitions {
            if let AstNode::AccountDefinition { name, fields, .. } = account {
                if name.to_lowercase().contains(&query_lower) {
                    let location = Location {
                        uri: uri.clone(),
                        range: make_location_range(),
                    };
                    symbols.push(make_symbol_information(
                        name.clone(),
                        SymbolKind::STRUCT,
                        location,
                    ));
                }

                // Also search account fields
                for field in fields {
                    if field.name.to_lowercase().contains(&query_lower) {
                        let location = Location {
                            uri: uri.clone(),
                            range: make_location_range(),
                        };
                        symbols.push(make_symbol_information(
                            field.name.clone(),
                            SymbolKind::FIELD,
                            location,
                        ));
                    }
                }
            }
        }

        // Extract events
        for event in event_definitions {
            if let AstNode::EventDefinition { name, fields, .. } = event {
                if name.to_lowercase().contains(&query_lower) {
                    let location = Location {
                        uri: uri.clone(),
                        range: make_location_range(),
                    };
                    symbols.push(make_symbol_information(
                        name.clone(),
                        SymbolKind::EVENT,
                        location,
                    ));
                }

                // Also search event fields
                for field in fields {
                    if field.name.to_lowercase().contains(&query_lower) {
                        let location = Location {
                            uri: uri.clone(),
                            range: make_location_range(),
                        };
                        symbols.push(make_symbol_information(
                            field.name.clone(),
                            SymbolKind::PROPERTY,
                            location,
                        ));
                    }
                }
            }
        }

        // Extract global fields
        for field in field_definitions {
            if let AstNode::FieldDefinition { name, .. } = field {
                if name.to_lowercase().contains(&query_lower) {
                    let location = Location {
                        uri: uri.clone(),
                        range: make_location_range(),
                    };
                    symbols.push(make_symbol_information(
                        name.clone(),
                        SymbolKind::VARIABLE,
                        location,
                    ));
                }
            }
        }

        // Extract interfaces
        for interface in interface_definitions {
            if let AstNode::InterfaceDefinition {
                name, functions, ..
            } = interface
            {
                if name.to_lowercase().contains(&query_lower) {
                    let location = Location {
                        uri: uri.clone(),
                        range: make_location_range(),
                    };
                    symbols.push(make_symbol_information(
                        name.clone(),
                        SymbolKind::INTERFACE,
                        location,
                    ));
                }

                // Also search interface functions
                for func in functions {
                    if let AstNode::InterfaceFunction {
                        name: func_name, ..
                    } = func
                    {
                        if func_name.to_lowercase().contains(&query_lower) {
                            let location = Location {
                                uri: uri.clone(),
                                range: make_location_range(),
                            };
                            symbols.push(make_symbol_information(
                                func_name.clone(),
                                SymbolKind::METHOD,
                                location,
                            ));
                        }
                    }
                }
            }
        }
    }

    symbols
}

/// Create a default location range (placeholder until SourceLocation is available)
fn make_location_range() -> Range {
    Range {
        start: Position {
            line: 0,
            character: 0,
        },
        end: Position {
            line: 0,
            character: 0,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::workspace_symbols;
    use crate::bridge::CompilerBridge;
    use lsp_types::Url;

    fn test_uri(name: &str) -> Url {
        Url::parse(&format!("file:///workspace/{}", name)).expect("valid test uri")
    }

    #[test]
    fn finds_instruction_names_case_insensitively() {
        let mut bridge = CompilerBridge::new();
        let uri = test_uri("instructions.v");
        let source = r#"
transfer() -> u64 {
    return 1;
}

mint_tokens() -> u64 {
    return 2;
}
"#;

        let symbols = workspace_symbols(&mut bridge, source, "TrAnS", &uri);
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "transfer");
    }

    #[test]
    fn finds_account_names_and_fields() {
        let mut bridge = CompilerBridge::new();
        let uri = test_uri("accounts.v");
        let source = r#"
account VaultAccount {
    balance: u64;
    owner: pubkey;
}
"#;

        let account_symbols = workspace_symbols(&mut bridge, source, "vault", &uri);
        assert_eq!(account_symbols.len(), 1);
        assert_eq!(account_symbols[0].name, "VaultAccount");

        let field_symbols = workspace_symbols(&mut bridge, source, "bal", &uri);
        assert_eq!(field_symbols.len(), 1);
        assert_eq!(field_symbols[0].name, "balance");
    }

    #[test]
    fn returns_empty_on_parse_failure() {
        let mut bridge = CompilerBridge::new();
        let uri = test_uri("broken.v");
        let source = "account Broken {";

        let symbols = workspace_symbols(&mut bridge, source, "broken", &uri);
        assert!(symbols.is_empty());
    }
}
