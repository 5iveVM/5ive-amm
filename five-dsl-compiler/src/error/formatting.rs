//! Error formatting and display
//!
//! This module provides pluggable error formatters for different output formats
//! including terminal, JSON, and LSP-compatible formatting.

use crate::error::context::{ContextLine, SourceContextExtractor};
use crate::error::types::{CompilerError, ErrorSeverity, RelatedError, SourceLocation};

/// Trait for error formatters
///
/// This allows for pluggable error formatting based on the output context
/// (terminal, JSON, IDE integration, etc.)
pub trait ErrorFormatter {
    /// Format a single compiler error
    fn format_error(&self, error: &CompilerError) -> String;

    /// Format multiple errors as a batch
    fn format_errors(&self, errors: &[CompilerError]) -> String {
        errors
            .iter()
            .map(|e| self.format_error(e))
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    /// Get the formatter name
    fn name(&self) -> &'static str;
}

/// Terminal formatter with colors and rich formatting
///
/// Formats errors for display in a terminal with ANSI color codes
/// and visual formatting similar to the Rust compiler.
pub struct TerminalFormatter {
    /// Whether to use colors in output
    use_colors: bool,

    /// Maximum width for error messages (reserved for future use)
    _max_width: usize,

    /// Source context extractor
    context_extractor: SourceContextExtractor,
}

impl TerminalFormatter {
    /// Create a new terminal formatter
    pub fn new() -> Self {
        Self {
            use_colors: Self::supports_color(),
            _max_width: 120,
            context_extractor: SourceContextExtractor::new(),
        }
    }

    /// Create a terminal formatter with custom settings
    pub fn with_settings(use_colors: bool, max_width: usize) -> Self {
        Self {
            use_colors,
            _max_width: max_width,
            context_extractor: SourceContextExtractor::new(),
        }
    }

    /// Check if the terminal supports colors
    fn supports_color() -> bool {
        // Simple color support detection
        std::env::var("NO_COLOR").is_err()
            && std::env::var("TERM").map(|t| t != "dumb").unwrap_or(true)
    }

    /// Apply color to text if colors are enabled
    fn colorize(&self, text: &str, color: TerminalColor) -> String {
        if self.use_colors {
            format!(
                "{}{}{}",
                color.ansi_code(),
                text,
                TerminalColor::Reset.ansi_code()
            )
        } else {
            text.to_string()
        }
    }

    /// Format the error header (error code, severity, message)
    fn format_header(&self, error: &CompilerError) -> String {
        let severity_color = match error.severity {
            ErrorSeverity::Error => TerminalColor::Red,
            ErrorSeverity::Warning => TerminalColor::Yellow,
            ErrorSeverity::Note => TerminalColor::Blue,
            ErrorSeverity::Help => TerminalColor::Cyan,
        };

        let severity_text = self.colorize(&error.severity.to_string(), severity_color);
        let code_text = self.colorize(&format!("[{}]", error.code), TerminalColor::Bright);

        format!("{}{}: {}", severity_text, code_text, error.message)
    }

    /// Format source location
    fn format_location(&self, location: &SourceLocation) -> String {
        let location_text = format!(" --> {}", location);
        self.colorize(&location_text, TerminalColor::Blue)
    }

    /// Format source context with line numbers and highlighting
    fn format_source_context(&self, source: &str, location: &SourceLocation) -> String {
        let context_result = self.context_extractor.extract_context(source, location);

        let context = match context_result {
            Ok(context) => context,
            Err(_) => return String::new(),
        };

        let mut output = String::new();

        // Calculate the width needed for line numbers
        let max_line_num = context
            .lines
            .iter()
            .map(|l| l.line_number)
            .max()
            .unwrap_or(0);
        let line_num_width = format!("{}", max_line_num).len();

        // Add separator
        output.push_str(&format!(
            "{}  {}\n",
            " ".repeat(line_num_width),
            self.colorize("|", TerminalColor::Blue)
        ));

        // Format each context line
        for line in &context.lines {
            if line.is_error_line {
                // Error line with highlighting
                output.push_str(&self.format_error_line(line, line_num_width, location));

                // Add underline
                output.push_str(&self.format_underline(line_num_width, location, &line.content));
            } else {
                // Regular context line
                let line_num = self.colorize(
                    &format!("{:width$}", line.line_number, width = line_num_width),
                    TerminalColor::Blue,
                );
                let separator = self.colorize(" | ", TerminalColor::Blue);
                output.push_str(&format!("{}{}{}\n", line_num, separator, line.content));
            }
        }

        // Add closing separator
        output.push_str(&format!(
            "{}  {}",
            " ".repeat(line_num_width),
            self.colorize("|", TerminalColor::Blue)
        ));

        output
    }

    /// Format an error line with highlighting
    fn format_error_line(
        &self,
        line: &ContextLine,
        line_num_width: usize,
        _location: &SourceLocation,
    ) -> String {
        let line_num = self.colorize(
            &format!("{:width$}", line.line_number, width = line_num_width),
            TerminalColor::Blue,
        );
        let separator = self.colorize(" | ", TerminalColor::Blue);

        format!("{}{}{}\n", line_num, separator, line.content)
    }

    /// Format underline for error highlighting
    fn format_underline(
        &self,
        line_num_width: usize,
        location: &SourceLocation,
        line: &str,
    ) -> String {
        let underline = self.context_extractor.create_underline(location, line);
        let underline_colored = self.colorize(&underline, TerminalColor::Red);
        let separator = self.colorize(" | ", TerminalColor::Blue);

        format!(
            "{}  {}{}\n",
            " ".repeat(line_num_width),
            separator,
            underline_colored
        )
    }

    /// Format related errors and notes
    fn format_related(&self, related: &[RelatedError]) -> String {
        if related.is_empty() {
            return String::new();
        }

        let mut output = String::new();
        for rel in related {
            let severity_color = match rel.severity {
                ErrorSeverity::Error => TerminalColor::Red,
                ErrorSeverity::Warning => TerminalColor::Yellow,
                ErrorSeverity::Note => TerminalColor::Blue,
                ErrorSeverity::Help => TerminalColor::Cyan,
            };

            let severity_text = self.colorize(&rel.severity.to_string(), severity_color);
            output.push_str(&format!("{}: {}\n", severity_text, rel.message));

            if let Some(location) = &rel.location {
                output.push_str(&self.format_location(location));
                output.push('\n');
            }
        }

        output
    }
}

impl ErrorFormatter for TerminalFormatter {
    fn format_error(&self, error: &CompilerError) -> String {
        let mut output = String::new();

        // Error header
        output.push_str(&self.format_header(error));
        output.push('\n');

        // Location
        if let Some(location) = &error.location {
            output.push_str(&self.format_location(location));
            output.push('\n');

            // Source context (if we can extract it)
            if let Some(source_line) = &error.context.source_line {
                let mock_source = source_line; // In real usage, we'd have the full source
                output.push_str(&self.format_source_context(mock_source, location));
                output.push('\n');
            }
        }

        // Description
        if let Some(description) = &error.description {
            output.push('\n');
            output.push_str(description);
            output.push('\n');
        }

        // Related errors and notes
        let related_output = self.format_related(&error.related);
        if !related_output.is_empty() {
            output.push('\n');
            output.push_str(&related_output);
        }

        output
    }

    fn name(&self) -> &'static str {
        "terminal"
    }
}

impl Default for TerminalFormatter {
    fn default() -> Self {
        Self::new()
    }
}

/// JSON formatter for programmatic consumption
///
/// Formats errors as JSON objects for integration with IDEs,
/// build tools, and other programmatic consumers.
pub struct JsonFormatter {
    /// Whether to pretty-print JSON
    pretty: bool,
}

impl JsonFormatter {
    pub fn new() -> Self {
        Self { pretty: false }
    }

    pub fn pretty() -> Self {
        Self { pretty: true }
    }
}

impl ErrorFormatter for JsonFormatter {
    fn format_error(&self, error: &CompilerError) -> String {
        let json_error = JsonError {
            code: format!("{}", error.code),
            severity: error.severity,
            category: format!("{}", error.category),
            message: error.message.clone(),
            description: error.description.clone(),
            location: error.location.as_ref().map(|loc| JsonLocation {
                file: loc.file.as_ref().map(|f| f.display().to_string()),
                line: loc.line,
                column: loc.column,
                offset: loc.offset,
                length: loc.length,
            }),
            related: error
                .related
                .iter()
                .map(|rel| JsonRelatedError {
                    severity: rel.severity,
                    message: rel.message.clone(),
                    location: rel.location.as_ref().map(|loc| JsonLocation {
                        file: loc.file.as_ref().map(|f| f.display().to_string()),
                        line: loc.line,
                        column: loc.column,
                        offset: loc.offset,
                        length: loc.length,
                    }),
                })
                .collect(),
        };

        if self.pretty {
            serde_json::to_string_pretty(&json_error).unwrap_or_else(|_| "{}".to_string())
        } else {
            serde_json::to_string(&json_error).unwrap_or_else(|_| "{}".to_string())
        }
    }

    fn name(&self) -> &'static str {
        "json"
    }
}

impl Default for JsonFormatter {
    fn default() -> Self {
        Self::new()
    }
}

/// LSP (Language Server Protocol) formatter
///
/// Formats errors for integration with LSP-compatible editors
/// following the LSP diagnostic specification.
pub struct LspFormatter;

impl LspFormatter {
    pub fn new() -> Self {
        Self
    }
}

impl ErrorFormatter for LspFormatter {
    fn format_error(&self, error: &CompilerError) -> String {
        let diagnostic = LspDiagnostic {
            range: error.location.as_ref().map(|loc| LspRange {
                start: LspPosition {
                    line: loc.line.saturating_sub(1), // LSP uses 0-based line numbers
                    character: loc.column.saturating_sub(1), // LSP uses 0-based column numbers
                },
                end: LspPosition {
                    line: loc.line.saturating_sub(1),
                    character: (loc.column + loc.length as u32).saturating_sub(1),
                },
            }),
            severity: match error.severity {
                ErrorSeverity::Error => 1,
                ErrorSeverity::Warning => 2,
                ErrorSeverity::Note => 3,
                ErrorSeverity::Help => 4,
            },
            code: Some(format!("{}", error.code)),
            source: Some("five-compiler".to_string()),
            message: error.message.clone(),
            related_information: error
                .related
                .iter()
                .filter_map(|rel| {
                    rel.location.as_ref().map(|loc| LspRelatedInformation {
                        location: LspLocation {
                            uri: loc
                                .file
                                .as_ref()
                                .map(|f| format!("file://{}", f.display()))
                                .unwrap_or_else(|| "file://unknown".to_string()),
                            range: LspRange {
                                start: LspPosition {
                                    line: loc.line.saturating_sub(1),
                                    character: loc.column.saturating_sub(1),
                                },
                                end: LspPosition {
                                    line: loc.line.saturating_sub(1),
                                    character: (loc.column + loc.length as u32).saturating_sub(1),
                                },
                            },
                        },
                        message: rel.message.clone(),
                    })
                })
                .collect(),
        };

        serde_json::to_string(&diagnostic).unwrap_or_else(|_| "{}".to_string())
    }

    fn name(&self) -> &'static str {
        "lsp"
    }
}

impl Default for LspFormatter {
    fn default() -> Self {
        Self::new()
    }
}

/// Terminal colors
#[derive(Debug, Clone, Copy)]
enum TerminalColor {
    Red,
    Yellow,
    Blue,
    Cyan,
    Bright,
    Reset,
}

impl TerminalColor {
    fn ansi_code(self) -> &'static str {
        match self {
            Self::Red => "\x1b[31m",
            Self::Yellow => "\x1b[33m",
            Self::Blue => "\x1b[34m",
            Self::Cyan => "\x1b[36m",
            Self::Bright => "\x1b[1m",
            Self::Reset => "\x1b[0m",
        }
    }
}

// JSON serialization structures
#[derive(serde::Serialize)]
struct JsonError {
    code: String,
    severity: ErrorSeverity,
    category: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    location: Option<JsonLocation>,
    related: Vec<JsonRelatedError>,
}

#[derive(serde::Serialize)]
struct JsonLocation {
    #[serde(skip_serializing_if = "Option::is_none")]
    file: Option<String>,
    line: u32,
    column: u32,
    offset: usize,
    length: usize,
}

#[derive(serde::Serialize)]
struct JsonRelatedError {
    severity: ErrorSeverity,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    location: Option<JsonLocation>,
}

// LSP serialization structures
#[derive(serde::Serialize)]
struct LspDiagnostic {
    #[serde(skip_serializing_if = "Option::is_none")]
    range: Option<LspRange>,
    severity: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source: Option<String>,
    message: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    related_information: Vec<LspRelatedInformation>,
}

#[derive(serde::Serialize)]
struct LspRange {
    start: LspPosition,
    end: LspPosition,
}

#[derive(serde::Serialize)]
struct LspPosition {
    line: u32,
    character: u32,
}

#[derive(serde::Serialize)]
struct LspLocation {
    uri: String,
    range: LspRange,
}

#[derive(serde::Serialize)]
struct LspRelatedInformation {
    location: LspLocation,
    message: String,
}

// Implement Serialize for ErrorSeverity
impl serde::Serialize for ErrorSeverity {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::types::{ErrorBuilder, ErrorCategory, ErrorCode};
    use std::path::PathBuf;

    #[test]
    fn test_terminal_formatter() {
        let error = ErrorBuilder::new(
            ErrorCode::EXPECTED_TOKEN,
            "expected `;`, found `}`".to_string(),
        )
        .category(ErrorCategory::Syntax)
        .location(SourceLocation::new(5, 17, 100).with_file(PathBuf::from("test.v")))
        .build();

        let formatter = TerminalFormatter::new();
        let output = formatter.format_error(&error);

        assert!(output.contains("error"));
        assert!(output.contains("E0001"));
        assert!(output.contains("expected `;`, found `}`"));
        assert!(output.contains("test.v:5:17"));
    }

    #[test]
    fn test_json_formatter() {
        let error = ErrorBuilder::new(ErrorCode::TYPE_MISMATCH, "type mismatch".to_string())
            .category(ErrorCategory::Type)
            .build();

        let formatter = JsonFormatter::new();
        let output = formatter.format_error(&error);

        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["code"], "E1000");
        assert_eq!(parsed["message"], "type mismatch");
        assert_eq!(parsed["severity"], "error");
    }

    #[test]
    fn test_lsp_formatter() {
        let error = ErrorBuilder::new(
            ErrorCode::UNDEFINED_VARIABLE,
            "undefined variable".to_string(),
        )
        .location(SourceLocation::new(3, 10, 50))
        .build();

        let formatter = LspFormatter::new();
        let output = formatter.format_error(&error);

        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["severity"], 1); // Error
        assert_eq!(parsed["message"], "undefined variable");
        assert_eq!(parsed["range"]["start"]["line"], 2); // LSP uses 0-based
        assert_eq!(parsed["range"]["start"]["character"], 9);
    }
}
