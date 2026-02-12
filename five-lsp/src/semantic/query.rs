//! Position-based AST Query Engine
//!
//! Provides utilities for finding AST nodes at cursor positions,
//! extracting symbols, and navigating the AST for LSP features.

use five_dsl_compiler::ast::{AstNode, SourceLocation};

/// Find the AST node at a given position (line, column)
///
/// This performs a depth-first search to find the deepest (most specific)
/// node that contains the given position.
///
/// # Arguments
/// * `ast` - The root AST node (usually a Program)
/// * `line` - 0-indexed line number
/// * `column` - 0-indexed character offset
///
/// # Returns
/// The deepest AST node containing the position, or None if not found
pub fn ast_node_at_position(ast: &AstNode, line: u32, column: u32) -> Option<&AstNode> {
    // Helper function to recursively find the node
    fn find_node<'a>(node: &'a AstNode, line: u32, column: u32) -> Option<&'a AstNode> {
        // Check if this node has a location that contains the position
        let location = node_location(node)?;
        if !location.contains(line, column) {
            return None;
        }

        // Try to find a more specific child node
        let children = get_children(node);
        for child in children {
            if let Some(found) = find_node(child, line, column) {
                return Some(found);
            }
        }

        // No child found, return this node
        Some(node)
    }

    find_node(ast, line, column)
}

/// Extract the symbol name under the cursor
///
/// This is a text-based heuristic for quickly finding the identifier
/// at a given position without full AST traversal.
///
/// # Arguments
/// * `source` - The source code
/// * `line` - 0-indexed line number
/// * `column` - 0-indexed character offset
///
/// # Returns
/// The identifier at the position, or None if not on an identifier
pub fn symbol_under_cursor(source: &str, line: u32, column: u32) -> Option<String> {
    let lines: Vec<&str> = source.lines().collect();
    let line_text = lines.get(line as usize)?;

    // Find the start and end of the identifier at the column
    let chars: Vec<char> = line_text.chars().collect();
    if column as usize >= chars.len() {
        return None;
    }

    let char_at_pos = chars[column as usize];
    if !is_identifier_char(char_at_pos) {
        return None;
    }

    // Find the start of the identifier
    let mut start = column as usize;
    while start > 0 && is_identifier_char(chars[start - 1]) {
        start -= 1;
    }

    // Find the end of the identifier
    let mut end = column as usize + 1;
    while end < chars.len() && is_identifier_char(chars[end]) {
        end += 1;
    }

    Some(chars[start..end].iter().collect())
}

/// Check if a character is valid in an identifier
fn is_identifier_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Get the source location of an AST node
///
/// Extracts the location information from various AST node types.
fn node_location(node: &AstNode) -> Option<SourceLocation> {
    match node {
        // Literals and expressions - most have location field
        AstNode::StringLiteral { location, .. } => Some(*location),
        AstNode::BinaryExpression { location, .. } => Some(*location),
        AstNode::UnaryExpression { location, .. } => Some(*location),
        // Statements
        AstNode::IfStatement { location, .. } => Some(*location),
        AstNode::ReturnStatement { .. } => None, // Not all statements have location
        AstNode::ExpressionStatement { .. } => None,
        // Declarations
        AstNode::LetStatement { .. } => None,
        AstNode::FunctionDefinition { .. } => None,
        AstNode::FieldDefinition { .. } => None,
        AstNode::AccountDefinition { .. } => None,
        AstNode::InterfaceDefinition { .. } => None,
        AstNode::EventDefinition { .. } => None,
        AstNode::UseStatement { .. } => None,
        AstNode::InitBlock { .. } => None,
        AstNode::ConstraintsBlock { .. } => None,
        _ => None,
    }
}

/// Get all child nodes of an AST node
///
/// Returns a vector of references to child nodes for traversal.
fn get_children(node: &AstNode) -> Vec<&AstNode> {
    let mut children = Vec::new();

    match node {
        AstNode::Program {
            field_definitions,
            instruction_definitions,
            event_definitions,
            account_definitions,
            interface_definitions,
            use_statements,
            ..
        } => {
            children.extend(field_definitions.iter());
            children.extend(instruction_definitions.iter());
            children.extend(event_definitions.iter());
            children.extend(account_definitions.iter());
            children.extend(interface_definitions.iter());
            children.extend(use_statements.iter());
        }
        AstNode::InstructionDefinition { body, .. } => {
            if let Some(statements) = body {
                children.extend(statements.iter());
            }
        }
        AstNode::IfStatement {
            condition,
            body,
            alternate,
            ..
        } => {
            children.push(condition.as_ref());
            children.extend(body.iter());
            if let Some(alt_stmts) = alternate {
                children.extend(alt_stmts.iter());
            }
        }
        AstNode::BinaryExpression { left, right, .. } => {
            children.push(left.as_ref());
            children.push(right.as_ref());
        }
        AstNode::UnaryExpression { operand, .. } => {
            children.push(operand.as_ref());
        }
        AstNode::LetStatement { value, .. } => {
            if let Some(val) = value {
                children.push(val.as_ref());
            }
        }
        AstNode::ReturnStatement { value, .. } => {
            if let Some(val) = value {
                children.push(val.as_ref());
            }
        }
        AstNode::ExpressionStatement { expression, .. } => {
            children.push(expression.as_ref());
        }
        _ => {}
    }

    children
}

/// Find the enclosing function node for a given position
///
/// Useful for signature help and context-aware completion.
pub fn enclosing_function(ast: &AstNode, line: u32, column: u32) -> Option<&AstNode> {
    fn find_enclosing<'a>(node: &'a AstNode, line: u32, column: u32) -> Option<&'a AstNode> {
        match node {
            AstNode::FunctionDefinition { location, body, .. } => {
                if location.contains(line, column) {
                    // Check if we're inside the body
                    if let Some(statements) = body {
                        for stmt in statements {
                            if let Some(found) = find_enclosing(stmt, line, column) {
                                return Some(found);
                            }
                        }
                    }
                    return Some(node);
                }
            }
            _ => {
                for child in get_children(node) {
                    if let Some(found) = find_enclosing(child, line, column) {
                        return Some(found);
                    }
                }
            }
        }
        None
    }

    find_enclosing(ast, line, column)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_under_cursor() {
        let source = "let myVar = 5;\nlet x = myVar + 1;";

        // Test on "myVar" in first line
        assert_eq!(symbol_under_cursor(source, 0, 4), Some("myVar".to_string()));
        assert_eq!(symbol_under_cursor(source, 0, 5), Some("myVar".to_string()));
        assert_eq!(symbol_under_cursor(source, 0, 8), Some("myVar".to_string()));

        // Test on "myVar" in second line
        assert_eq!(symbol_under_cursor(source, 1, 8), Some("myVar".to_string()));

        // Test on whitespace
        assert_eq!(symbol_under_cursor(source, 0, 0), Some("let".to_string()));
        assert_eq!(symbol_under_cursor(source, 0, 3), None); // Space

        // Test on number
        assert_eq!(symbol_under_cursor(source, 0, 12), Some("5".to_string()));
    }

    #[test]
    fn test_is_identifier_char() {
        assert!(is_identifier_char('a'));
        assert!(is_identifier_char('Z'));
        assert!(is_identifier_char('_'));
        assert!(is_identifier_char('0'));
        assert!(!is_identifier_char(' '));
        assert!(!is_identifier_char('='));
        assert!(!is_identifier_char('+'));
    }
}
