//! Document formatting provider
//!
//! Provides basic code formatting for Five DSL documents.
//! Standardizes indentation, spacing, and line breaks.

use lsp_types::{Position, Range, TextEdit};

/// Format an entire document
///
/// Applies consistent indentation and spacing rules.
pub fn format_document(source: &str) -> Vec<TextEdit> {
    let mut edits = Vec::new();
    let lines: Vec<&str> = source.lines().collect();
    let mut current_indent: usize = 0;

    for (line_idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Skip empty lines
        if trimmed.is_empty() {
            continue;
        }

        // Calculate indent level based on brackets
        if trimmed.starts_with('}') || trimmed.starts_with(']') || trimmed.starts_with(')') {
            current_indent = current_indent.saturating_sub(1);
        }

        // Calculate expected indentation (4 spaces per level)
        let expected_indent = "    ".repeat(current_indent);

        // Check if current indentation differs from expected
        let current_indent_str = if line.is_empty() {
            ""
        } else {
            let indent_len = line.len() - line.trim_start().len();
            &line[..indent_len]
        };

        if current_indent_str != expected_indent {
            // Replace current indentation with expected
            let (indent_end, _) = line
                .chars()
                .enumerate()
                .find(|(_, c)| !c.is_whitespace())
                .unwrap_or((line.len(), ' '));

            edits.push(TextEdit {
                range: Range {
                    start: Position {
                        line: line_idx as u32,
                        character: 0,
                    },
                    end: Position {
                        line: line_idx as u32,
                        character: indent_end as u32,
                    },
                },
                new_text: expected_indent.clone(),
            });
        }

        // Update indent for next line based on opening brackets
        if trimmed.ends_with('{') || trimmed.ends_with('[') || trimmed.ends_with('(') {
            current_indent += 1;
        }

        // Special cases for closing brackets
        if (trimmed.starts_with('}') || trimmed.starts_with(']') || trimmed.starts_with(')'))
            && (trimmed.contains('{') || trimmed.contains('[') || trimmed.contains('('))
        {
            // Contains both opening and closing, increase indent on next line
            if trimmed.contains('{') || trimmed.contains('[') || trimmed.contains('(') {
                current_indent += 1;
            }
        }

    }

    edits
}

/// Format a range of code
pub fn format_range(source: &str, start_line: usize, end_line: usize) -> Vec<TextEdit> {
    let lines: Vec<&str> = source.lines().collect();
    let mut edits = Vec::new();
    let mut current_indent: usize = 0;

    // Calculate starting indent from lines before range
    for line in lines.iter().take(start_line) {
        let trimmed = line.trim();
        if trimmed.ends_with('{') || trimmed.ends_with('[') || trimmed.ends_with('(') {
            current_indent += 1;
        }
        if trimmed.starts_with('}') || trimmed.starts_with(']') || trimmed.starts_with(')') {
            current_indent = current_indent.saturating_sub(1);
        }
    }

    // Format the range
    for (idx, line) in lines
        .iter()
        .enumerate()
        .skip(start_line)
        .take(end_line - start_line + 1)
    {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            continue;
        }

        if trimmed.starts_with('}') || trimmed.starts_with(']') || trimmed.starts_with(')') {
            current_indent = current_indent.saturating_sub(1);
        }

        let expected_indent = "    ".repeat(current_indent);
        let current_indent_str = if line.is_empty() {
            ""
        } else {
            let indent_len = line.len() - line.trim_start().len();
            &line[..indent_len]
        };

        if current_indent_str != expected_indent {
            let (indent_end, _) = line
                .chars()
                .enumerate()
                .find(|(_, c)| !c.is_whitespace())
                .unwrap_or((line.len(), ' '));

            edits.push(TextEdit {
                range: Range {
                    start: Position {
                        line: idx as u32,
                        character: 0,
                    },
                    end: Position {
                        line: idx as u32,
                        character: indent_end as u32,
                    },
                },
                new_text: expected_indent,
            });
        }

        if trimmed.ends_with('{') || trimmed.ends_with('[') || trimmed.ends_with('(') {
            current_indent += 1;
        }
    }

    edits
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_simple_function() {
        let source = "pub instruction test() {\nlet x = 5;\n}";
        let edits = format_document(source);

        // Should have edits for indentation
        assert!(!edits.is_empty());
    }

    #[test]
    fn test_format_nested_blocks() {
        let source = "if true {\nif true {\nlet x = 5;\n}\n}";
        let edits = format_document(source);

        // Should fix nested indentation
        assert!(!edits.is_empty());
    }

    #[test]
    fn test_format_already_correct() {
        let source = "pub instruction test() {\n    let x = 5;\n}";
        let edits = format_document(source);

        assert!(edits.is_empty(), "No edits expected for already formatted source");
    }

    #[test]
    fn test_format_range() {
        let source = "pub instruction test() {\nlet x = 5;\nlet y = 10;\n}";
        let edits = format_range(source, 1, 2);

        // Should format only the specified range
        assert!(!edits.is_empty());
    }
}
