//! Hover provider for type information
//!
//! Shows type information and mutability when hovering over symbols.

use crate::bridge::CompilerBridge;
use lsp_types::{Hover, HoverContents, MarkedString, Position, Range, Url};

/// Get hover information for a symbol at the given position
///
/// # Arguments
/// * `bridge` - Compiler bridge with cached AST and type context
/// * `source` - Source code
/// * `position` - Cursor position (0-indexed line and character)
/// * `uri` - File URI
///
/// # Returns
/// Option<Hover> if a symbol was found at the position, None otherwise
pub fn get_hover(
    bridge: &CompilerBridge,
    source: &str,
    position: Position,
    uri: &Url,
) -> Option<Hover> {
    // Convert Position to 0-indexed (Position from LSP is 0-indexed already)
    let line = position.line as usize;
    let character = position.character as usize;

    // Get the identifier at the cursor position
    let identifier = extract_identifier_at_position(source, line, character)?;

    // Try to resolve the symbol type
    let (type_info, is_mutable) = bridge.resolve_symbol(uri, source, &identifier)?;

    // Format type as string
    let type_string = format_type_node(&type_info);
    let mut hover_text = type_string;

    // Add mutability indicator
    if is_mutable {
        hover_text.push_str(" (mutable)");
    }

    // Create hover with markdown formatting
    let hover = Hover {
        contents: HoverContents::Scalar(MarkedString::String(format!(
            "```five\n{}\n```",
            hover_text
        ))),
        range: Some(Range {
            start: Position {
                line: position.line,
                character: (character - identifier.len()) as u32,
            },
            end: Position {
                line: position.line,
                character: character as u32,
            },
        }),
    };

    Some(hover)
}

/// Extract the identifier at the given cursor position
///
/// Handles extracting the full word under the cursor from the source code.
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
    // If not, we can't extract an identifier at this position
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

/// Format a TypeNode as a human-readable string
fn format_type_node(type_node: &five_dsl_compiler::ast::TypeNode) -> String {
    use five_dsl_compiler::ast::TypeNode;

    match type_node {
        TypeNode::Primitive(name) => name.clone(),
        TypeNode::Generic { base, args } => {
            let arg_strs: Vec<String> = args.iter().map(format_type_node).collect();
            format!("{}<{}>", base, arg_strs.join(", "))
        }
        TypeNode::Array { element_type, size } => {
            let elem_str = format_type_node(element_type);
            match size {
                Some(s) => format!("[{}; {}]", elem_str, s),
                None => format!("[{}]", elem_str),
            }
        }
        TypeNode::Tuple { elements } => {
            let elem_strs: Vec<String> = elements.iter().map(format_type_node).collect();
            format!("({})", elem_strs.join(", "))
        }
        TypeNode::Struct { fields } => {
            // For structs, show field count (full fields would be too verbose)
            format!("struct {{ {} fields }}", fields.len())
        }
        TypeNode::Sized { base_type, size } => {
            format!("{}<{}>", base_type, size)
        }
        TypeNode::Union { types } => {
            let type_strs: Vec<String> = types.iter().map(format_type_node).collect();
            type_strs.join(" | ")
        }
        TypeNode::Account => "account".to_string(),
        TypeNode::Named(name) => name.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_identifier_simple() {
        let source = "let x = 5;";
        // Position 4 is where 'x' is
        let identifier = extract_identifier_at_position(source, 0, 4);
        assert_eq!(identifier, Some("x".to_string()));
    }

    #[test]
    fn test_extract_identifier_multichar() {
        let source = "let counter = 0;";
        // Position 4 is within 'counter'
        let identifier = extract_identifier_at_position(source, 0, 4);
        assert_eq!(identifier, Some("counter".to_string()));
    }

    #[test]
    fn test_extract_identifier_at_end() {
        let source = "let x = 5;";
        // Position 4 is at 'x'
        let identifier = extract_identifier_at_position(source, 0, 4);
        assert_eq!(identifier, Some("x".to_string()));
    }

    #[test]
    fn test_extract_identifier_returns_none_on_space() {
        let source = "let x = 5;";
        // Position 5 is a space after 'x'
        let identifier = extract_identifier_at_position(source, 0, 5);
        assert_eq!(identifier, None);
    }

    #[test]
    fn test_format_primitive_type() {
        let type_node = five_dsl_compiler::ast::TypeNode::Primitive("u64".to_string());
        assert_eq!(format_type_node(&type_node), "u64");
    }

    #[test]
    fn test_format_generic_type() {
        let type_node = five_dsl_compiler::ast::TypeNode::Generic {
            base: "Option".to_string(),
            args: vec![five_dsl_compiler::ast::TypeNode::Primitive(
                "u64".to_string(),
            )],
        };
        assert_eq!(format_type_node(&type_node), "Option<u64>");
    }

    #[test]
    fn test_format_sized_type() {
        let type_node = five_dsl_compiler::ast::TypeNode::Sized {
            base_type: "string".to_string(),
            size: 32,
        };
        assert_eq!(format_type_node(&type_node), "string<32>");
    }

    #[test]
    fn test_format_account_type() {
        let type_node = five_dsl_compiler::ast::TypeNode::Account;
        assert_eq!(format_type_node(&type_node), "account");
    }

    #[test]
    fn test_format_named_type() {
        let type_node = five_dsl_compiler::ast::TypeNode::Named("MyCustomType".to_string());
        assert_eq!(format_type_node(&type_node), "MyCustomType");
    }

    #[test]
    fn test_format_array_type() {
        let type_node = five_dsl_compiler::ast::TypeNode::Array {
            element_type: Box::new(five_dsl_compiler::ast::TypeNode::Primitive(
                "u64".to_string(),
            )),
            size: Some(10),
        };
        assert_eq!(format_type_node(&type_node), "[u64; 10]");
    }
}
