//! Inlay hints provider for type annotations
//!
//! Shows inferred types and parameter names as hints inline in the editor,
//! helping users understand the types of expressions and parameters.

use lsp_types::{InlayHint, InlayHintKind, InlayHintLabel, Position};

/// Get inlay hints for a line
///
/// Returns type annotations and parameter hints for better code understanding.
pub fn get_inlay_hints(source: &str, line: usize) -> Vec<InlayHint> {
    let lines: Vec<&str> = source.lines().collect();

    if line >= lines.len() {
        return vec![];
    }

    let line_str = lines[line];
    let mut hints = Vec::new();

    // Find let statements and add type hints
    find_let_statements(line_str, line as u32, &mut hints);

    // Find function calls and add parameter hints
    find_function_calls(line_str, line as u32, &mut hints);

    // Find return statements and add type hints
    find_return_statements(line_str, line as u32, &mut hints);

    hints
}

/// Find let statements and suggest type hints
fn find_let_statements(line: &str, line_num: u32, hints: &mut Vec<InlayHint>) {
    if let Some(let_pos) = line.find("let ") {
        // Find the identifier after "let"
        let after_let = &line[let_pos + 4..];
        if let Some((name_end, _)) = after_let
            .chars()
            .enumerate()
            .find(|(_, c)| *c == '=' || *c == ':' || *c == ';')
        {
            let var_name = after_let[..name_end].trim();

            // Check if type annotation already exists
            if !line[let_pos..].contains(':')
                || line[let_pos..].find(':').unwrap()
                    > line[let_pos..].find('=').unwrap_or(usize::MAX)
            {
                // Add hint after variable name
                hints.push(InlayHint {
                    position: Position {
                        line: line_num,
                        character: (let_pos + 4 + name_end) as u32,
                    },
                    label: InlayHintLabel::String(": unknown".to_string()),
                    kind: Some(InlayHintKind::TYPE),
                    text_edits: None,
                    tooltip: Some(lsp_types::InlayHintTooltip::String(format!(
                        "Inferred type for `{}` (explicit annotation recommended)",
                        var_name
                    ))),
                    padding_left: Some(true),
                    padding_right: Some(false),
                    data: None,
                });
            }
        }
    }
}

/// Find function calls and suggest parameter name hints
fn find_function_calls(line: &str, line_num: u32, hints: &mut Vec<InlayHint>) {
    let mut pos = 0;

    while let Some(paren_pos) = line[pos..].find('(') {
        let actual_pos = pos + paren_pos;

        // Find function name before parenthesis
        if let Some(func_name) = extract_function_name_at(&line[..actual_pos]) {
            // Add parameter hints based on function
            add_parameter_hints(&func_name, line_num, actual_pos as u32, hints);
        }

        pos = actual_pos + 1;
    }
}

/// Find return statements and add type hints
fn find_return_statements(line: &str, line_num: u32, hints: &mut Vec<InlayHint>) {
    if let Some(return_pos) = line.find("return ") {
        let after_return = &line[return_pos + 7..];

        // Find the end of the expression
        if let Some(end_pos) = after_return.find(';').or_else(|| Some(after_return.len())) {
            let expr = after_return[..end_pos].trim();

            if !expr.is_empty() {
                hints.push(InlayHint {
                    position: Position {
                        line: line_num,
                        character: (return_pos + 7 + end_pos) as u32,
                    },
                    label: InlayHintLabel::String(" -> unknown".to_string()),
                    kind: Some(InlayHintKind::TYPE),
                    text_edits: None,
                    tooltip: Some(lsp_types::InlayHintTooltip::String(
                        "Return type".to_string(),
                    )),
                    padding_left: Some(true),
                    padding_right: Some(false),
                    data: None,
                });
            }
        }
    }
}

/// Extract function name before an opening parenthesis
fn extract_function_name_at(text: &str) -> Option<String> {
    let trimmed = text.trim_end();
    if trimmed.is_empty() {
        return None;
    }

    let chars: Vec<char> = trimmed.chars().collect();
    let end = chars.len();
    let mut start = end;

    while start > 0 && (chars[start - 1].is_alphanumeric() || chars[start - 1] == '_') {
        start -= 1;
    }

    if start < end {
        Some(chars[start..end].iter().collect())
    } else {
        None
    }
}

/// Add parameter name hints for known functions
fn add_parameter_hints(func_name: &str, line_num: u32, paren_pos: u32, hints: &mut Vec<InlayHint>) {
    let params = match func_name {
        "require" => vec!["condition"],
        "return" => vec!["value"],
        "emit" => vec!["event", "fields"],
        "let" => vec!["name", "value"],
        _ => return, // Only add hints for known functions
    };

    if params.is_empty() {
        return;
    }

    // Add hint for first parameter
    hints.push(InlayHint {
        position: Position {
            line: line_num,
            character: paren_pos + 1,
        },
        label: InlayHintLabel::String(format!("{}: ", params[0])),
        kind: Some(InlayHintKind::PARAMETER),
        text_edits: None,
        tooltip: Some(lsp_types::InlayHintTooltip::String(format!(
            "Parameter: {}",
            params[0]
        ))),
        padding_left: Some(false),
        padding_right: Some(false),
        data: None,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_inlay_hints_for_let() {
        let source = "let x = 5;";
        let hints = get_inlay_hints(source, 0);

        // Should have at least one hint for the let statement
        assert!(!hints.is_empty());
    }

    #[test]
    fn test_extract_function_name_at() {
        assert_eq!(
            extract_function_name_at("my_func"),
            Some("my_func".to_string())
        );
        assert_eq!(
            extract_function_name_at("some_call.method"),
            Some("method".to_string())
        );
    }

    #[test]
    fn test_hints_skip_already_annotated() {
        let source = "let x: u64 = 5;";
        let hints = get_inlay_hints(source, 0);

        // Should skip adding type hint if annotation already exists
        assert!(hints.is_empty());
    }

    #[test]
    fn test_parameter_hints_for_require() {
        let mut hints = Vec::new();
        add_parameter_hints("require", 0, 8, &mut hints);

        assert!(!hints.is_empty());
        match &hints[0].label {
            InlayHintLabel::String(s) => assert!(s.contains("condition")),
            InlayHintLabel::LabelParts(_) => panic!("Expected string label"),
        }
    }

    #[test]
    fn test_return_statement_hints() {
        let source = "return x;";
        let hints = get_inlay_hints(source, 0);

        // Should have hints for return statement
        assert!(!hints.is_empty());
    }
}
