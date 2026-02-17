//! Document symbols provider for outline/navigator view
//!
//! Extracts all top-level definitions from the AST for editor outline/navigator views.
//! This is a semantic implementation using the compiler's AST.

use crate::bridge::CompilerBridge;
use five_dsl_compiler::ast::AstNode;
use lsp_types::{DocumentSymbol, Position, Range, SymbolKind};

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

                symbols.push(make_symbol(
                    name.clone(),
                    detail,
                    SymbolKind::FUNCTION,
                    location.clone(),
                    location,
                    None,
                ));
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

                    children.push(make_symbol(
                        field.name.clone(),
                        detail,
                        SymbolKind::FIELD,
                        field_location.clone(),
                        field_location,
                        None,
                    ));
                }

                symbols.push(make_symbol(
                    name.clone(),
                    Some("account".to_string()),
                    SymbolKind::STRUCT,
                    location.clone(),
                    location,
                    if children.is_empty() { None } else { Some(children) },
                ));
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
                    children.push(make_symbol(
                        field.name.clone(),
                        Some("property".to_string()),
                        SymbolKind::PROPERTY,
                        field_location.clone(),
                        field_location,
                        None,
                    ));
                }

                symbols.push(make_symbol(
                    name.clone(),
                    Some("event".to_string()),
                    SymbolKind::EVENT,
                    location.clone(),
                    location,
                    if children.is_empty() { None } else { Some(children) },
                ));
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

                symbols.push(make_symbol(
                    name.clone(),
                    detail,
                    SymbolKind::VARIABLE,
                    location.clone(),
                    location,
                    None,
                ));
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
                        children.push(make_symbol(
                            func_name.clone(),
                            Some("method".to_string()),
                            SymbolKind::METHOD,
                            func_location.clone(),
                            func_location,
                            None,
                        ));
                    }
                }

                symbols.push(make_symbol(
                    name.clone(),
                    Some("interface".to_string()),
                    SymbolKind::INTERFACE,
                    location.clone(),
                    location,
                    if children.is_empty() { None } else { Some(children) },
                ));
            }
        }
    }

    symbols
}

fn make_symbol(
    name: String,
    detail: Option<String>,
    kind: SymbolKind,
    range: Range,
    selection_range: Range,
    children: Option<Vec<DocumentSymbol>>,
) -> DocumentSymbol {
    #[allow(deprecated)]
    DocumentSymbol {
        name,
        detail,
        kind,
        range,
        selection_range,
        children,
        deprecated: None,
        tags: None,
    }
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
    #[test]
    fn test_placeholder() {
        // Placeholder test - AST compilation tested in integration tests
        assert!(true);
    }
}
