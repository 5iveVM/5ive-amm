//! Code actions provider for quick fixes
//!
//! Provides quick fixes for common errors and patterns in Five DSL code.

use lsp_types::{CodeAction, CodeActionKind, Diagnostic, Position, Range, TextEdit, WorkspaceEdit};
use std::collections::HashMap;

/// Get code actions for a diagnostic
///
/// Analyzes the diagnostic message and provides relevant quick fixes.
/// Offers semantic-aware suggestions for Five DSL errors.
pub fn get_code_actions(
    source: &str,
    diagnostic: &Diagnostic,
    uri: &lsp_types::Url,
) -> Vec<CodeAction> {
    let mut actions = Vec::new();

    let message = diagnostic.message.to_lowercase();
    let line_num = diagnostic.range.start.line as usize;

    // Missing visibility modifier - prepend 'pub ' at start of line
    if message.contains("public") || message.contains("visibility") {
        if let Some(edit) = fix_missing_visibility(source, line_num) {
            actions.push(code_action_from_edit(
                "Add 'pub' modifier",
                edit,
                uri.clone(),
                diagnostic.clone(),
                true,
            ));
        }
    }

    // Mutability issues - prepend 'mut ' before identifier
    if message.contains("mut") || message.contains("immutable") || message.contains("cannot assign")
    {
        if let Some(edit) = fix_missing_mutability(source, line_num, &diagnostic.range) {
            actions.push(code_action_from_edit(
                "Add 'mut' modifier",
                edit,
                uri.clone(),
                diagnostic.clone(),
                true,
            ));
        }
    }

    // Type mismatch hints - informational only, no fix available
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

    // Missing account constraints - add constraint after parameter name
    if message.contains("account") || message.contains("constraint") {
        if let Some(edit) = fix_missing_account_constraint(source, line_num, "@mut") {
            actions.push(code_action_from_edit(
                "Add @mut constraint",
                edit,
                uri.clone(),
                diagnostic.clone(),
                false,
            ));
        }

        if let Some(edit) = fix_missing_account_constraint(source, line_num, "@signer") {
            actions.push(code_action_from_edit(
                "Add @signer constraint",
                edit,
                uri.clone(),
                diagnostic.clone(),
                false,
            ));
        }
    }

    // Missing semicolon errors
    if message.contains("semicolon") || message.contains("expected `;`") {
        if let Some(edit) =
            fix_missing_semicolon(source, line_num, diagnostic.range.start.character as usize)
        {
            actions.push(code_action_from_edit(
                "Add semicolon",
                edit,
                uri.clone(),
                diagnostic.clone(),
                true,
            ));
        }
    }

    // Unused variable warnings
    if message.contains("unused") || message.contains("not used") {
        actions.push(CodeAction {
            title: "Remove unused variable".to_string(),
            kind: Some(CodeActionKind::QUICKFIX),
            diagnostics: Some(vec![diagnostic.clone()]),
            edit: None,
            command: None,
            is_preferred: None,
            disabled: None,
            data: None,
        });
    }

    actions
}

/// Helper to create a CodeAction from a TextEdit
fn code_action_from_edit(
    title: &str,
    edit: TextEdit,
    uri: lsp_types::Url,
    diagnostic: Diagnostic,
    is_preferred: bool,
) -> CodeAction {
    let mut changes = HashMap::new();
    changes.insert(uri, vec![edit]);

    CodeAction {
        title: title.to_string(),
        kind: Some(CodeActionKind::QUICKFIX),
        diagnostics: Some(vec![diagnostic]),
        edit: Some(WorkspaceEdit {
            changes: Some(changes),
            change_annotations: None,
            document_changes: None,
        }),
        command: None,
        is_preferred: Some(is_preferred),
        disabled: None,
        data: None,
    }
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
    if (line_str.contains("function ") || line_str.contains("account "))
        && !line_str.contains("pub ")
    {
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

/// Quick fix for adding mutability modifier
///
/// Inserts 'mut ' before the variable name on the specified line.
/// Looks for 'let ' keyword and adds 'mut ' after it.
pub fn fix_missing_mutability(source: &str, line: usize, _range: &Range) -> Option<TextEdit> {
    let lines: Vec<&str> = source.lines().collect();

    if line >= lines.len() {
        return None;
    }

    let line_str = lines[line];

    // Find 'let ' keyword and insert 'mut ' after it
    if let Some(let_pos) = line_str.find("let ") {
        let insert_pos = let_pos + 4; // Length of "let "

        return Some(TextEdit {
            range: Range {
                start: Position {
                    line: line as u32,
                    character: insert_pos as u32,
                },
                end: Position {
                    line: line as u32,
                    character: insert_pos as u32,
                },
            },
            new_text: "mut ".to_string(),
        });
    }

    None
}

/// Quick fix for adding account constraints
///
/// Appends a constraint (e.g., '@mut', '@signer') after the account parameter.
pub fn fix_missing_account_constraint(
    source: &str,
    line: usize,
    constraint: &str,
) -> Option<TextEdit> {
    let lines: Vec<&str> = source.lines().collect();

    if line >= lines.len() {
        return None;
    }

    let line_str = lines[line];

    // Look for 'account ' keyword - constraint should be added after the type name
    if let Some(account_pos) = line_str.find("account ") {
        // Find the next space or colon after 'account ', which marks the end of the account parameter name
        let search_start = account_pos + 8; // Length of "account "
        let remaining = &line_str[search_start..];

        if let Some(end_pos) = remaining.find(|c| c == ' ' || c == ',' || c == ')') {
            let insert_pos = search_start + end_pos;

            return Some(TextEdit {
                range: Range {
                    start: Position {
                        line: line as u32,
                        character: insert_pos as u32,
                    },
                    end: Position {
                        line: line as u32,
                        character: insert_pos as u32,
                    },
                },
                new_text: format!(" {}", constraint),
            });
        }
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

        let uri: lsp_types::Url = "file:///test.v".parse().unwrap();
        let actions = get_code_actions("function test() {}", &diagnostic, &uri);
        assert!(!actions.is_empty());
        assert!(actions.iter().any(|a| a.title.contains("pub")));
        // Verify the action has actual edits now
        assert!(actions[0].edit.is_some());
    }

    #[test]
    fn test_fix_missing_semicolon() {
        let source = "let x = 5";
        let fix = fix_missing_semicolon(source, 0, 9);
        assert!(fix.is_some());
        assert_eq!(fix.unwrap().new_text, ";");
    }

    #[test]
    fn test_fix_missing_visibility() {
        let source = "function test() {}";
        let fix = fix_missing_visibility(source, 0);
        assert!(fix.is_some());
        let edit = fix.unwrap();
        assert_eq!(edit.new_text, "pub ");
        assert_eq!(edit.range.start.character, 0);
    }

    #[test]
    fn test_fix_missing_mutability() {
        let source = "let x = 5;";
        let range = Range {
            start: Position {
                line: 0,
                character: 4,
            },
            end: Position {
                line: 0,
                character: 5,
            },
        };
        let fix = fix_missing_mutability(source, 0, &range);
        assert!(fix.is_some());
        let edit = fix.unwrap();
        assert_eq!(edit.new_text, "mut ");
        assert_eq!(edit.range.start.character, 4); // After "let "
    }

    #[test]
    fn test_fix_missing_account_constraint() {
        let source = "account counter)";
        let fix = fix_missing_account_constraint(source, 0, "@mut");
        assert!(fix.is_some());
        let edit = fix.unwrap();
        assert_eq!(edit.new_text, " @mut");
    }
}
