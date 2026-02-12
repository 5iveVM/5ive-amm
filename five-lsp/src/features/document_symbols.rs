//! Document symbols provider for outline/navigator view
//!
//! Extracts all top-level definitions from the AST for editor outline/navigator views.
//! This is a semantic implementation using the compiler's AST.

use crate::bridge::CompilerBridge;
use five_dsl_compiler::ast::AstNode;
use lsp_types::{DocumentSymbol, SymbolKind, Range, Position};

/// Get document symbols for outline view
///
/// Returns all top-level definitions extracted from the AST for display
/// in the editor's outline panel.
pub fn get_document_symbols(
    bridge: &mut CompilerBridge,
    source: &str,
    uri: &lsp_types::Url,
) -> Vec<DocumentSymbol> {
    // Parse source to AST using compiler
    let ast = match bridge.compile_to_ast(uri, source) {
        Ok(ast) => ast,
        Err(_) => {
            // On parse error, return empty to avoid cascading errors
            // Diagnostics are reported separately
            return Vec::new();
        }
    };

    // Extract symbols from AST
    extract_symbols_from_ast(&ast)
}

/// Extract all symbols from an AST node
fn extract_symbols_from_ast(ast: &AstNode) -> Vec<DocumentSymbol> {
    let mut symbols = Vec::new();

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
        // Add functions (instructions)
        for instr in instruction_definitions {
            if let AstNode::InstructionDefinition {
                name,
                visibility,
                ..
            } = instr
            {
                let location = make_location_range();
                let is_public = match visibility {
                    five_dsl_compiler::ast::Visibility::Public => true,
                    _ => false,
                };

                let detail = if is_public {
                    Some("pub fn".to_string())
                } else {
                    Some("fn".to_string())
                };

                symbols.push(DocumentSymbol {
                    name: name.clone(),
                    detail,
                    kind: SymbolKind::FUNCTION,
                    deprecated: None,
                    range: location.clone(),
                    selection_range: location,
                    children: None,
                    tags: None,
                });
            }
        }

        // Add accounts with nested fields
        for account in account_definitions {
            if let AstNode::AccountDefinition { name, fields, .. } = account {
                let location = make_location_range();
                let mut children = Vec::new();

                // Add fields as nested symbols
                for field in fields {
                    let field_location = make_location_range();
                    let detail = if field.is_mutable {
                        Some("mut field".to_string())
                    } else {
                        Some("field".to_string())
                    };

                    children.push(DocumentSymbol {
                        name: field.name.clone(),
                        detail,
                        kind: SymbolKind::FIELD,
                        deprecated: None,
                        range: field_location.clone(),
                        selection_range: field_location,
                        children: None,
                        tags: None,
                    });
                }

                symbols.push(DocumentSymbol {
                    name: name.clone(),
                    detail: Some("account".to_string()),
                    kind: SymbolKind::STRUCT,
                    deprecated: None,
                    range: location.clone(),
                    selection_range: location,
                    children: if children.is_empty() { None } else { Some(children) },
                    tags: None,
                });
            }
        }

        // Add events
        for event in event_definitions {
            if let AstNode::EventDefinition { name, fields, .. } = event {
                let location = make_location_range();
                let mut children = Vec::new();

                // Add event fields as nested symbols
                for field in fields {
                    let field_location = make_location_range();
                    children.push(DocumentSymbol {
                        name: field.name.clone(),
                        detail: Some("property".to_string()),
                        kind: SymbolKind::PROPERTY,
                        deprecated: None,
                        range: field_location.clone(),
                        selection_range: field_location,
                        children: None,
                        tags: None,
                    });
                }

                symbols.push(DocumentSymbol {
                    name: name.clone(),
                    detail: Some("event".to_string()),
                    kind: SymbolKind::EVENT,
                    deprecated: None,
                    range: location.clone(),
                    selection_range: location,
                    children: if children.is_empty() { None } else { Some(children) },
                    tags: None,
                });
            }
        }

        // Add global fields
        for field in field_definitions {
            if let AstNode::FieldDefinition {
                name,
                is_mutable,
                ..
            } = field
            {
                let location = make_location_range();
                let detail = if *is_mutable {
                    Some("mut field".to_string())
                } else {
                    Some("field".to_string())
                };

                symbols.push(DocumentSymbol {
                    name: name.clone(),
                    detail,
                    kind: SymbolKind::VARIABLE,
                    deprecated: None,
                    range: location.clone(),
                    selection_range: location,
                    children: None,
                    tags: None,
                });
            }
        }

        // Add interfaces
        for interface in interface_definitions {
            if let AstNode::InterfaceDefinition { name, functions, .. } = interface {
                let location = make_location_range();
                let mut children = Vec::new();

                // Add interface functions as nested symbols
                for func in functions {
                    if let AstNode::InterfaceFunction { name: func_name, .. } = func {
                        let func_location = make_location_range();
                        children.push(DocumentSymbol {
                            name: func_name.clone(),
                            detail: Some("method".to_string()),
                            kind: SymbolKind::METHOD,
                            deprecated: None,
                            range: func_location.clone(),
                            selection_range: func_location,
                            children: None,
                            tags: None,
                        });
                    }
                }

                symbols.push(DocumentSymbol {
                    name: name.clone(),
                    detail: Some("interface".to_string()),
                    kind: SymbolKind::INTERFACE,
                    deprecated: None,
                    range: location.clone(),
                    selection_range: location,
                    children: if children.is_empty() { None } else { Some(children) },
                    tags: None,
                });
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
    use super::*;

    #[test]
    fn test_placeholder() {
        // Placeholder test - AST compilation tested in integration tests
        assert!(true);
    }
}
