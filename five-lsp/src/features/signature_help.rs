//! Signature help provider for parameter hints
//!
//! Shows function signatures and parameter information as users type,
//! helping them understand what arguments a function expects.

use crate::bridge::CompilerBridge;
use lsp_types::{ParameterInformation, ParameterLabel, SignatureHelp, SignatureInformation};

/// Get signature help at cursor position
///
/// Returns function signature with parameter info when cursor is inside a function call.
pub fn get_signature_help(
    _bridge: &CompilerBridge,
    source: &str,
    line: usize,
    character: usize,
) -> Option<SignatureHelp> {
    let lines: Vec<&str> = source.lines().collect();

    if line >= lines.len() {
        return None;
    }

    let line_str = lines[line];
    if character > line_str.len() {
        return None;
    }

    // Find the opening parenthesis before cursor
    let before_cursor = &line_str[..character];
    let paren_pos = before_cursor.rfind('(')?;

    // Extract the function name before the parenthesis
    let func_area = &before_cursor[..paren_pos];
    let func_name = extract_function_name(func_area)?;

    // Count commas to find current parameter index
    let inside_parens = &line_str[paren_pos + 1..character];
    let param_index = inside_parens.chars().filter(|&c| c == ',').count();

    // Get signature for this function
    get_function_signature(&func_name, param_index)
}

/// Extract function name from text preceding an opening parenthesis
fn extract_function_name(text: &str) -> Option<String> {
    let trimmed = text.trim_end();
    if trimmed.is_empty() {
        return None;
    }

    // Find the start of the identifier (move backwards)
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

/// Get signature information for a known function
fn get_function_signature(func_name: &str, active_param: usize) -> Option<SignatureHelp> {
    let signatures = match func_name {
        "instruction" | "pub" => {
            vec![SignatureInformation {
                label: "instruction name(params) -> return_type".to_string(),
                documentation: Some(lsp_types::Documentation::String(
                    "Defines a public on-chain callable function".to_string(),
                )),
                parameters: Some(vec![
                    ParameterInformation {
                        label: ParameterLabel::Simple("name: String".to_string()),
                        documentation: None,
                    },
                    ParameterInformation {
                        label: ParameterLabel::Simple("params: [Parameter]".to_string()),
                        documentation: None,
                    },
                ]),
                active_parameter: Some(active_param as u32),
            }]
        }
        "let" => {
            vec![SignatureInformation {
                label: "let name [: type] = value;".to_string(),
                documentation: Some(lsp_types::Documentation::String(
                    "Variable binding - creates a new variable".to_string(),
                )),
                parameters: Some(vec![
                    ParameterInformation {
                        label: ParameterLabel::Simple("name: String".to_string()),
                        documentation: None,
                    },
                    ParameterInformation {
                        label: ParameterLabel::Simple("type: TypeAnnotation (optional)".to_string()),
                        documentation: None,
                    },
                    ParameterInformation {
                        label: ParameterLabel::Simple("value: Expression".to_string()),
                        documentation: None,
                    },
                ]),
                active_parameter: Some(active_param as u32),
            }]
        }
        "if" => {
            vec![SignatureInformation {
                label: "if condition { then_branch } [else { else_branch }]".to_string(),
                documentation: Some(lsp_types::Documentation::String(
                    "Conditional execution - runs code if condition is true".to_string(),
                )),
                parameters: Some(vec![
                    ParameterInformation {
                        label: ParameterLabel::Simple("condition: bool".to_string()),
                        documentation: None,
                    },
                    ParameterInformation {
                        label: ParameterLabel::Simple("then_branch: Block".to_string()),
                        documentation: None,
                    },
                    ParameterInformation {
                        label: ParameterLabel::Simple("else_branch: Block (optional)".to_string()),
                        documentation: None,
                    },
                ]),
                active_parameter: Some(active_param as u32),
            }]
        }
        "require" => {
            vec![SignatureInformation {
                label: "require(condition);".to_string(),
                documentation: Some(lsp_types::Documentation::String(
                    "Assert a condition - fails if false".to_string(),
                )),
                parameters: Some(vec![ParameterInformation {
                    label: ParameterLabel::Simple("condition: bool".to_string()),
                    documentation: None,
                }]),
                active_parameter: Some(active_param as u32),
            }]
        }
        "return" => {
            vec![SignatureInformation {
                label: "return [value];".to_string(),
                documentation: Some(lsp_types::Documentation::String(
                    "Return from function with optional value".to_string(),
                )),
                parameters: Some(vec![ParameterInformation {
                    label: ParameterLabel::Simple("value: Expression (optional)".to_string()),
                    documentation: None,
                }]),
                active_parameter: Some(active_param as u32),
            }]
        }
        "emit" => {
            vec![SignatureInformation {
                label: "emit EventName { field: value, ... };".to_string(),
                documentation: Some(lsp_types::Documentation::String(
                    "Emit an event with fields".to_string(),
                )),
                parameters: Some(vec![
                    ParameterInformation {
                        label: ParameterLabel::Simple("event_name: String".to_string()),
                        documentation: None,
                    },
                    ParameterInformation {
                        label: ParameterLabel::Simple("fields: FieldAssignment".to_string()),
                        documentation: None,
                    },
                ]),
                active_parameter: Some(active_param as u32),
            }]
        }
        _ => {
            // Generic function signature for unknown functions
            vec![SignatureInformation {
                label: format!("{}(...)", func_name),
                documentation: Some(lsp_types::Documentation::String(
                    "Function call".to_string(),
                )),
                parameters: Some(vec![ParameterInformation {
                    label: ParameterLabel::Simple("params: [Expression]".to_string()),
                    documentation: None,
                }]),
                active_parameter: Some(active_param as u32),
            }]
        }
    };

    Some(SignatureHelp {
        signatures,
        active_signature: Some(0),
        active_parameter: Some(active_param as u32),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_function_name() {
        assert_eq!(extract_function_name("my_func"), Some("my_func".to_string()));
        assert_eq!(extract_function_name("  test"), Some("test".to_string()));
        assert_eq!(
            extract_function_name("some.method"),
            Some("method".to_string())
        );
    }

    #[test]
    fn test_signature_for_let() {
        let sig = get_function_signature("let", 0);
        assert!(sig.is_some());
        let sig = sig.unwrap();
        assert_eq!(sig.signatures.len(), 1);
        assert!(sig.signatures[0]
            .label
            .contains("let name [: type] = value"));
    }

    #[test]
    fn test_signature_for_if() {
        let sig = get_function_signature("if", 1);
        assert!(sig.is_some());
        let sig = sig.unwrap();
        assert_eq!(sig.signatures[0].active_parameter, Some(1));
    }

    #[test]
    fn test_signature_for_require() {
        let sig = get_function_signature("require", 0);
        assert!(sig.is_some());
        let sig = sig.unwrap();
        assert!(sig.signatures[0].label.contains("require"));
    }

    #[test]
    fn test_get_signature_help_at_cursor() {
        let source = "let x = my_func(";
        let bridge = CompilerBridge::new();
        let help = get_signature_help(&bridge, source, 0, source.len());

        // Should find generic function signature
        assert!(help.is_some());
    }
}
