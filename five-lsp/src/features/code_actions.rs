//! Code actions provider for quick fixes
//!
//! Provides quick fixes for common errors and patterns in Five DSL code.

use lsp_types::{CodeAction, CodeActionKind, Diagnostic, Position, Range, TextEdit, WorkspaceEdit};
use std::collections::HashMap;

/// Get code actions for a diagnostic
///
/// Analyzes the diagnostic message and provides relevant quick fixes.
pub fn get_code_actions(
    _source: &str,
    diagnostic: &Diagnostic,
    _uri: &lsp_types::Url,
) -> Vec<CodeAction> {
    let mut actions = Vec::new();

    let message = diagnostic.message.to_lowercase();

    // Missing visibility modifier
    if message.contains("public") || message.contains("visibility") {
        actions.push(CodeAction {
            title: "Add 'pub' modifier".to_string(),
            kind: Some(CodeActionKind::QUICKFIX),
            diagnostics: Some(vec![diagnostic.clone()]),
            edit: Some(WorkspaceEdit {
                changes: Some(HashMap::new()),
                change_annotations: None,
                document_changes: None,
            }),
            command: None,
            is_preferred: Some(true),
            disabled: None,
            data: None,
        });
    }

    // Mutability issues
    if message.contains("mut") || message.contains("immutable") || message.contains("cannot assign") {
        actions.push(CodeAction {
            title: "Add 'mut' modifier".to_string(),
            kind: Some(CodeActionKind::QUICKFIX),
            diagnostics: Some(vec![diagnostic.clone()]),
            edit: Some(WorkspaceEdit {
                changes: Some(HashMap::new()),
                change_annotations: None,
                document_changes: None,
            }),
            command: None,
            is_preferred: Some(true),
            disabled: None,
            data: None,
        });
    }

    // Type mismatch hints
    if message.contains("type mismatch") || message.contains("expected") {
        actions.push(CodeAction {
            title: "Show type information".to_string(),
            kind: Some(CodeActionKind::QUICKFIX),
            diagnostics: Some(vec![diagnostic.clone()]),
            edit: None,
            command: None,
            is_preferred: None,
            disabled: None,
            data: None,
        });
    }

    // Missing account constraints
    if message.contains("account") || message.contains("constraint") {
        actions.push(CodeAction {
            title: "Add @mut constraint".to_string(),
            kind: Some(CodeActionKind::QUICKFIX),
            diagnostics: Some(vec![diagnostic.clone()]),
            edit: Some(WorkspaceEdit {
                changes: Some(HashMap::new()),
                change_annotations: None,
                document_changes: None,
            }),
            command: None,
            is_preferred: None,
            disabled: None,
            data: None,
        });

        actions.push(CodeAction {
            title: "Add @signer constraint".to_string(),
            kind: Some(CodeActionKind::QUICKFIX),
            diagnostics: Some(vec![diagnostic.clone()]),
            edit: Some(WorkspaceEdit {
                changes: Some(HashMap::new()),
                change_annotations: None,
                document_changes: None,
            }),
            command: None,
            is_preferred: None,
            disabled: None,
            data: None,
        });
    }

    actions
}

/// Quick fix for missing semicolon
pub fn fix_missing_semicolon(source: &str, line: usize, _column: usize) -> Option<TextEdit> {
    let lines: Vec<&str> = source.lines().collect();

    if line >= lines.len() {
        return None;
    }

    let line_str = lines[line];

    // Check if the line ends without a semicolon
    if !line_str.trim().ends_with(';') && !line_str.trim().ends_with('{') {
        return Some(TextEdit {
            range: Range {
                start: Position {
                    line: line as u32,
                    character: line_str.len() as u32,
                },
                end: Position {
                    line: line as u32,
                    character: line_str.len() as u32,
                },
            },
            new_text: ";".to_string(),
        });
    }

    None
}

/// Quick fix for adding visibility modifier
pub fn fix_missing_visibility(source: &str, line: usize) -> Option<TextEdit> {
    let lines: Vec<&str> = source.lines().collect();

    if line >= lines.len() {
        return None;
    }

    let line_str = lines[line];

    // Check if line is a function/struct without 'pub'
    if (line_str.contains("function ") || line_str.contains("account ")) && !line_str.contains("pub ") {
        let trimmed = line_str.trim_start();
        let indent = line_str.len() - trimmed.len();

        return Some(TextEdit {
            range: Range {
                start: Position {
                    line: line as u32,
                    character: indent as u32,
                },
                end: Position {
                    line: line as u32,
                    character: indent as u32,
                },
            },
            new_text: "pub ".to_string(),
        });
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_code_actions_for_visibility() {
        let diagnostic = Diagnostic {
            range: Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: 0,
                    character: 8,
                },
            },
            severity: Some(lsp_types::DiagnosticSeverity::ERROR),
            code: None,
            source: None,
            message: "Missing visibility modifier".to_string(),
            related_information: None,
            tags: None,
            code_description: None,
            data: None,
        };

        let actions = get_code_actions("function test() {}", &diagnostic, &"file:///test.v".parse().unwrap());
        assert!(!actions.is_empty());
        assert!(actions.iter().any(|a| a.title.contains("pub")));
    }

    #[test]
    fn test_fix_missing_semicolon() {
        let source = "let x = 5";
        let fix = fix_missing_semicolon(source, 0, 9);
        assert!(fix.is_some());
        assert_eq!(fix.unwrap().new_text, ";");
    }
}
