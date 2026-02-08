//! Source context extraction.

use crate::error::types::{ErrorContext, SourceLocation};

/// Extracts source code context around error locations.
pub struct SourceContextExtractor {
    /// Number of lines to show before the error line
    pub lines_before: usize,

    /// Number of lines to show after the error line
    pub lines_after: usize,

    /// Maximum line length before truncation
    pub max_line_length: usize,
}

impl SourceContextExtractor {
    /// Create a new context extractor with default settings
    pub fn new() -> Self {
        Self {
            lines_before: 2,
            lines_after: 2,
            max_line_length: 120,
        }
    }

    /// Create a context extractor with custom settings
    pub fn with_settings(lines_before: usize, lines_after: usize, max_line_length: usize) -> Self {
        Self {
            lines_before,
            lines_after,
            max_line_length,
        }
    }

    /// Extract context from source code at the given location
    pub fn extract_context(
        &self,
        source: &str,
        location: &SourceLocation,
    ) -> Result<SourceContext, ContextError> {
        let lines: Vec<&str> = source.lines().collect();

        if lines.is_empty() {
            return Ok(SourceContext::empty());
        }

        let error_line_idx = (location.line as usize).saturating_sub(1);
        if error_line_idx >= lines.len() {
            return Err(ContextError::LineOutOfRange {
                line: location.line,
                total_lines: lines.len(),
            });
        }

        let start_line = error_line_idx.saturating_sub(self.lines_before);
        let end_line = (error_line_idx + self.lines_after + 1).min(lines.len());

        let mut context_lines = Vec::new();
        for (i, line) in lines[start_line..end_line].iter().enumerate() {
            let line_number = start_line + i + 1;
            let is_error_line = line_number == location.line as usize;

            let line_content = if line.len() > self.max_line_length {
                format!("{}...", &line[..self.max_line_length - 3])
            } else {
                line.to_string()
            };

            context_lines.push(ContextLine {
                line_number,
                content: line_content,
                is_error_line,
            });
        }

        // Extract the specific error span within the line
        let error_line = lines[error_line_idx];
        let error_span = self.extract_error_span(error_line, location)?;

        Ok(SourceContext {
            lines: context_lines,
            error_line_number: location.line,
            error_column: location.column,
            error_span,
            file_path: location.file.clone(),
        })
    }

    /// Extract the specific error span within a line
    fn extract_error_span(
        &self,
        line: &str,
        location: &SourceLocation,
    ) -> Result<ErrorSpan, ContextError> {
        let chars: Vec<char> = line.chars().collect();
        let start_col = (location.column as usize).saturating_sub(1);

        if start_col >= chars.len() {
            return Err(ContextError::ColumnOutOfRange {
                column: location.column,
                line_length: chars.len(),
            });
        }

        let end_col = (start_col + location.length).min(chars.len());

        let before: String = chars[..start_col].iter().collect();
        let span: String = chars[start_col..end_col].iter().collect();
        let after: String = chars[end_col..].iter().collect();

        Ok(ErrorSpan {
            before,
            span,
            after,
            start_column: location.column,
            end_column: start_col as u32 + location.length as u32,
        })
    }

    /// Create an underline string for highlighting the error
    pub fn create_underline(&self, location: &SourceLocation, line: &str) -> String {
        let chars: Vec<char> = line.chars().collect();
        let start_col = (location.column as usize).saturating_sub(1);
        let mut underline = String::new();

        // Add spaces for columns before the error
        for i in 0..start_col {
            if i < chars.len() && chars[i] == '\t' {
                underline.push('\t');
            } else {
                underline.push(' ');
            }
        }

        // Add underline characters for the error span
        let underline_length = if location.length > 0 {
            location.length
        } else {
            1
        };
        for _ in 0..underline_length {
            underline.push('^');
        }

        underline
    }
}

impl Default for SourceContextExtractor {
    fn default() -> Self {
        Self::new()
    }
}

/// Source context around an error location
#[derive(Debug, Clone)]
pub struct SourceContext {
    /// Lines of source code around the error
    pub lines: Vec<ContextLine>,

    /// Line number where the error occurred
    pub error_line_number: u32,

    /// Column number where the error occurred
    pub error_column: u32,

    /// Detailed error span information
    pub error_span: ErrorSpan,

    /// File path (if available)
    pub file_path: Option<std::path::PathBuf>,
}

impl SourceContext {
    /// Create an empty source context
    pub fn empty() -> Self {
        Self {
            lines: Vec::new(),
            error_line_number: 0,
            error_column: 0,
            error_span: ErrorSpan::empty(),
            file_path: None,
        }
    }

    /// Get the error line content
    pub fn error_line(&self) -> Option<&ContextLine> {
        self.lines.iter().find(|line| line.is_error_line)
    }

    /// Get lines before the error
    pub fn lines_before(&self) -> Vec<&ContextLine> {
        self.lines
            .iter()
            .take_while(|line| !line.is_error_line)
            .collect()
    }

    /// Get lines after the error
    pub fn lines_after(&self) -> Vec<&ContextLine> {
        self.lines
            .iter()
            .skip_while(|line| !line.is_error_line)
            .skip(1) // Skip the error line itself
            .collect()
    }
}

/// A single line of source context
#[derive(Debug, Clone)]
pub struct ContextLine {
    pub line_number: usize,
    pub content: String,
    pub is_error_line: bool,
}

impl ContextLine {
    /// Get the line number as a formatted string with consistent width
    pub fn formatted_line_number(&self, width: usize) -> String {
        format!("{:width$}", self.line_number, width = width)
    }
}

/// Detailed information about the error span within a line
#[derive(Debug, Clone)]
pub struct ErrorSpan {
    /// Text before the error span
    pub before: String,

    /// The actual error span text
    pub span: String,

    /// Text after the error span
    pub after: String,

    /// Starting column of the error (1-based)
    pub start_column: u32,

    /// Ending column of the error (1-based)
    pub end_column: u32,
}

impl ErrorSpan {
    fn empty() -> Self {
        Self {
            before: String::new(),
            span: String::new(),
            after: String::new(),
            start_column: 0,
            end_column: 0,
        }
    }

    /// Get the full line content
    pub fn full_line(&self) -> String {
        format!("{}{}{}", self.before, self.span, self.after)
    }

    /// Get the length of the error span
    pub fn span_length(&self) -> usize {
        self.span.chars().count()
    }
}

/// Context extraction errors
#[derive(Debug, Clone)]
pub enum ContextError {
    LineOutOfRange { line: u32, total_lines: usize },
    ColumnOutOfRange { column: u32, line_length: usize },
}

impl std::fmt::Display for ContextError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LineOutOfRange { line, total_lines } => {
                write!(
                    f,
                    "Line {} is out of range (source has {} lines)",
                    line, total_lines
                )
            }
            Self::ColumnOutOfRange {
                column,
                line_length,
            } => {
                write!(
                    f,
                    "Column {} is out of range (line has {} characters)",
                    column, line_length
                )
            }
        }
    }
}

impl std::error::Error for ContextError {}

/// Convenience function to extract context and populate ErrorContext
pub fn extract_error_context(
    source: &str,
    location: &SourceLocation,
) -> Result<ErrorContext, ContextError> {
    let extractor = SourceContextExtractor::new();
    let context = extractor.extract_context(source, location)?;

    let mut error_context = ErrorContext::new();

    // Set source line if available
    if let Some(error_line) = context.error_line() {
        error_context = error_context.with_source_line(error_line.content.clone());
    }

    // Create source snippet
    let snippet = context
        .lines
        .iter()
        .map(|line| {
            if line.is_error_line {
                format!("{}  {}", line.formatted_line_number(3), line.content)
            } else {
                format!("{}  {}", line.formatted_line_number(3), line.content)
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    error_context = error_context.with_source_snippet(snippet);

    Ok(error_context)
}

/// Extract context for multiple related locations
pub fn extract_multi_location_context(
    source: &str,
    locations: &[SourceLocation],
) -> Result<Vec<ErrorContext>, ContextError> {
    locations
        .iter()
        .map(|loc| extract_error_context(source, loc))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_context_extraction() {
        let source = r#"script Test {
    balance: u64;
    
    test() {
        let x = invalid_syntax;
        balance = 100;
    }
}"#;

        let location = SourceLocation::new(5, 17, 0)
            .with_length(14)
            .with_file(PathBuf::from("test.v"));

        let extractor = SourceContextExtractor::new();
        let context = extractor.extract_context(source, &location).unwrap();

        assert_eq!(context.error_line_number, 5);
        assert_eq!(context.error_column, 17);
        assert!(context.lines.len() > 1);

        let error_line = context.error_line().unwrap();
        assert!(error_line.content.contains("invalid_syntax"));
        assert!(error_line.is_error_line);
    }

    #[test]
    fn test_underline_creation() {
        let source = "    let x = invalid_syntax;";
        let location = SourceLocation::new(1, 13, 0).with_length(14);

        let extractor = SourceContextExtractor::new();
        let underline = extractor.create_underline(&location, source);

        assert_eq!(underline, "            ^^^^^^^^^^^^^^");
    }

    #[test]
    fn test_error_span_extraction() {
        let line = "let x = invalid_syntax;";
        let location = SourceLocation::new(1, 9, 0).with_length(14);

        let extractor = SourceContextExtractor::new();
        let span = extractor.extract_error_span(line, &location).unwrap();

        assert_eq!(span.before, "let x = ");
        assert_eq!(span.span, "invalid_syntax");
        assert_eq!(span.after, ";");
    }

    #[test]
    fn test_empty_source() {
        let source = "";
        let location = SourceLocation::new(1, 1, 0);

        let extractor = SourceContextExtractor::new();
        let context = extractor.extract_context(source, &location).unwrap();

        assert_eq!(context.lines.len(), 0);
    }

    #[test]
    fn test_line_out_of_range() {
        let source = "single line";
        let location = SourceLocation::new(5, 1, 0); // Line 5 doesn't exist

        let extractor = SourceContextExtractor::new();
        let result = extractor.extract_context(source, &location);

        assert!(matches!(result, Err(ContextError::LineOutOfRange { .. })));
    }
}
