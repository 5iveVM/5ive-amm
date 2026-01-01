//! Core error types for the Five DSL compiler
//!
//! This module defines the fundamental error types used throughout the compiler.
//! These types are designed to be extensible and provide rich context for
//! error reporting and suggestions.

use std::error;
use std::fmt;
use std::path::PathBuf;

/// Unique error code identifier
///
/// Error codes follow the pattern E#### (e.g., E0001, E0308) similar to Rust.
/// Each error code has a corresponding entry in the error registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ErrorCode(pub u32);

impl ErrorCode {
    pub const fn new(code: u32) -> Self {
        Self(code)
    }

    pub fn code(&self) -> u32 {
        self.0
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "E{:04}", self.0)
    }
}

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    Error,
    Warning,
    Note,
    Help,
}

impl fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Error => write!(f, "error"),
            Self::Warning => write!(f, "warning"),
            Self::Note => write!(f, "note"),
            Self::Help => write!(f, "help"),
        }
    }
}

/// Error categories for organization and filtering
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ErrorCategory {
    Syntax,
    Type,
    Semantic,
    Codegen,
    IO,
    Internal,
    Custom(String),
}

impl fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Syntax => write!(f, "syntax"),
            Self::Type => write!(f, "type"),
            Self::Semantic => write!(f, "semantic"),
            Self::Codegen => write!(f, "codegen"),
            Self::IO => write!(f, "io"),
            Self::Internal => write!(f, "internal"),
            Self::Custom(name) => write!(f, "{}", name),
        }
    }
}

/// Main compiler error type
///
/// This is the primary error type used throughout the compiler. It provides
/// rich context including source location, error details, and metadata for
/// generating helpful error messages.
#[derive(Debug, Clone)]
pub struct CompilerError {
    /// Unique error code for this error type
    pub code: ErrorCode,

    /// Error severity level
    pub severity: ErrorSeverity,

    /// Error category for organization
    pub category: ErrorCategory,

    /// Primary error message
    pub message: String,

    /// Detailed error description
    pub description: Option<String>,

    /// Source location where the error occurred
    pub location: Option<SourceLocation>,

    /// Additional context for the error
    pub context: ErrorContext,

    /// Related errors or notes
    pub related: Vec<RelatedError>,
}

impl CompilerError {
    /// Create a new compiler error
    pub fn new(
        code: ErrorCode,
        severity: ErrorSeverity,
        category: ErrorCategory,
        message: String,
    ) -> Self {
        Self {
            code,
            severity,
            category,
            message,
            description: None,
            location: None,
            context: ErrorContext::new(),
            related: Vec::new(),
        }
    }

    /// Add a description to the error
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// Add source location to the error
    pub fn with_location(mut self, location: SourceLocation) -> Self {
        self.location = Some(location);
        self
    }

    /// Add context to the error
    pub fn with_context(mut self, context: ErrorContext) -> Self {
        self.context = context;
        self
    }

    /// Add a related error or note
    pub fn with_related(mut self, related: RelatedError) -> Self {
        self.related.push(related);
        self
    }

    /// Add multiple related errors
    pub fn with_related_errors(mut self, related: Vec<RelatedError>) -> Self {
        self.related.extend(related);
        self
    }

    /// Check if this is an error (vs warning/note)
    pub fn is_error(&self) -> bool {
        matches!(self.severity, ErrorSeverity::Error)
    }

    /// Check if this is a warning
    pub fn is_warning(&self) -> bool {
        matches!(self.severity, ErrorSeverity::Warning)
    }
}

impl fmt::Display for CompilerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)?;

        if let Some(location) = &self.location {
            write!(f, " at {}", location)?;
        }

        if let Some(description) = &self.description {
            write!(f, "\n  {}", description)?;
        }

        Ok(())
    }
}

impl error::Error for CompilerError {
    fn description(&self) -> &str {
        &self.message
    }

    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

/// Error type specific to module resolution issues.
#[derive(Debug, Clone)]
pub enum ModuleResolutionError {
    CircularDependency(String),
    ModuleNotFound { module_path: String, searched_paths: Vec<PathBuf> },
    InvalidModulePath(String),
    IoError(String), // For underlying std::io::Error
    Generic(String), // Fallback for other issues
}

impl fmt::Display for ModuleResolutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModuleResolutionError::CircularDependency(path) => {
                write!(f, "Circular dependency detected involving module: {}", path)
            }
            ModuleResolutionError::ModuleNotFound { module_path, searched_paths } => {
                write!(f, "Module '{}' not found. Searched paths: {:?}", module_path, searched_paths)
            }
            ModuleResolutionError::InvalidModulePath(path) => {
                write!(f, "Invalid module path format: {}", path)
            }
            ModuleResolutionError::IoError(msg) => write!(f, "IO error during module resolution: {}", msg),
            ModuleResolutionError::Generic(msg) => write!(f, "Module resolution error: {}", msg),
        }
    }
}

impl error::Error for ModuleResolutionError {}

impl From<std::io::Error> for ModuleResolutionError {
    fn from(err: std::io::Error) -> Self {
        ModuleResolutionError::IoError(err.to_string())
    }
}

/// Related error or note
///
/// Used to provide additional context or related information for the main error.
#[derive(Debug, Clone)]
pub struct RelatedError {
    pub severity: ErrorSeverity,
    pub message: String,
    pub location: Option<SourceLocation>,
}

impl RelatedError {
    pub fn note(message: String) -> Self {
        Self {
            severity: ErrorSeverity::Note,
            message,
            location: None,
        }
    }

    pub fn help(message: String) -> Self {
        Self {
            severity: ErrorSeverity::Help,
            message,
            location: None,
        }
    }

    pub fn with_location(mut self, location: SourceLocation) -> Self {
        self.location = Some(location);
        self
    }
}

/// Source location information
///
/// Represents a position in the source code where an error occurred.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceLocation {
    /// File path (if available)
    pub file: Option<PathBuf>,

    /// Line number (1-based)
    pub line: u32,

    /// Column number (1-based)
    pub column: u32,

    /// Character offset in the source (0-based)
    pub offset: usize,

    /// Length of the error span
    pub length: usize,
}

impl SourceLocation {
    pub fn new(line: u32, column: u32, offset: usize) -> Self {
        Self {
            file: None,
            line,
            column,
            offset,
            length: 1,
        }
    }

    pub fn with_file(mut self, file: PathBuf) -> Self {
        self.file = Some(file);
        self
    }

    pub fn with_length(mut self, length: usize) -> Self {
        self.length = length;
        self
    }

    /// Create a span from start to end location
    pub fn span_to(&self, end: &SourceLocation) -> Self {
        Self {
            file: self.file.clone(),
            line: self.line,
            column: self.column,
            offset: self.offset,
            length: end.offset + end.length - self.offset,
        }
    }
}

impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(file) = &self.file {
            write!(f, "{}:{}:{}", file.display(), self.line, self.column)
        } else {
            write!(f, "{}:{}", self.line, self.column)
        }
    }
}

/// Additional context for errors
///
/// Provides rich context information that can be used for error formatting
/// and suggestion generation.
#[derive(Debug, Clone, Default)]
pub struct ErrorContext {
    /// Source code snippet around the error
    pub source_snippet: Option<String>,

    /// Full source line containing the error
    pub source_line: Option<String>,

    /// Expected tokens/values (for parse errors)
    pub expected: Vec<String>,

    /// Actual token/value found
    pub found: Option<String>,

    /// Variable/function name being referenced
    pub identifier: Option<String>,

    /// Type information (for type errors)
    pub expected_type: Option<String>,
    pub actual_type: Option<String>,

    /// Custom context data
    pub data: std::collections::HashMap<String, String>,
}

impl ErrorContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_source_snippet(mut self, snippet: String) -> Self {
        self.source_snippet = Some(snippet);
        self
    }

    pub fn with_source_line(mut self, line: String) -> Self {
        self.source_line = Some(line);
        self
    }

    pub fn with_expected(mut self, expected: Vec<String>) -> Self {
        self.expected = expected;
        self
    }

    pub fn with_found(mut self, found: String) -> Self {
        self.found = Some(found);
        self
    }

    pub fn with_identifier(mut self, identifier: String) -> Self {
        self.identifier = Some(identifier);
        self
    }

    pub fn with_types(mut self, expected: String, actual: String) -> Self {
        self.expected_type = Some(expected);
        self.actual_type = Some(actual);
        self
    }

    pub fn with_data(mut self, data: std::collections::HashMap<String, String>) -> Self {
        self.data = data;
        self
    }

    pub fn add_data(mut self, key: String, value: String) -> Self {
        self.data.insert(key, value);
        self
    }

    /// Get context data by key
    pub fn get_data(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }
}

/// Error builder for convenient error construction
pub struct ErrorBuilder {
    error: CompilerError,
}

impl ErrorBuilder {
    pub fn new(code: ErrorCode, message: String) -> Self {
        Self {
            error: CompilerError::new(code, ErrorSeverity::Error, ErrorCategory::Internal, message),
        }
    }

    pub fn severity(mut self, severity: ErrorSeverity) -> Self {
        self.error.severity = severity;
        self
    }

    pub fn category(mut self, category: ErrorCategory) -> Self {
        self.error.category = category;
        self
    }

    pub fn description(mut self, description: String) -> Self {
        self.error.description = Some(description);
        self
    }

    pub fn location(mut self, location: SourceLocation) -> Self {
        self.error.location = Some(location);
        self
    }

    pub fn context(mut self, context: ErrorContext) -> Self {
        self.error.context = context;
        self
    }

    pub fn related(mut self, related: RelatedError) -> Self {
        self.error.related.push(related);
        self
    }

    pub fn build(self) -> CompilerError {
        self.error
    }
}

// Common error codes
impl ErrorCode {
    // Syntax errors (0000-0999)
    pub const EXPECTED_TOKEN: ErrorCode = ErrorCode::new(1);
    pub const UNEXPECTED_EOF: ErrorCode = ErrorCode::new(2);
    pub const UNEXPECTED_TOKEN: ErrorCode = ErrorCode::new(3);
    pub const INVALID_SYNTAX: ErrorCode = ErrorCode::new(4);
    pub const UNMATCHED_DELIMITER: ErrorCode = ErrorCode::new(10);
    pub const MISSING_IDENTIFIER: ErrorCode = ErrorCode::new(11);

    // Type errors (1000-1999)
    pub const TYPE_MISMATCH: ErrorCode = ErrorCode::new(1000);
    pub const CANNOT_INFER_TYPE: ErrorCode = ErrorCode::new(1001);
    pub const INVALID_CONVERSION: ErrorCode = ErrorCode::new(1002);
    pub const FUNCTION_SIGNATURE_MISMATCH: ErrorCode = ErrorCode::new(1003);
    pub const ARITHMETIC_TYPE_MISMATCH: ErrorCode = ErrorCode::new(1004);
    pub const COMPARISON_TYPE_MISMATCH: ErrorCode = ErrorCode::new(1005);
    pub const LOGICAL_TYPE_MISMATCH: ErrorCode = ErrorCode::new(1006);
    pub const ARRAY_INDEX_TYPE_MISMATCH: ErrorCode = ErrorCode::new(1007);
    pub const ASSIGNMENT_TYPE_MISMATCH: ErrorCode = ErrorCode::new(1008);
    pub const LITERAL_OVERFLOW: ErrorCode = ErrorCode::new(1010);

    // Semantic errors (2000-2999)
    pub const UNDEFINED_VARIABLE: ErrorCode = ErrorCode::new(2000);
    pub const VARIABLE_ALREADY_DEFINED: ErrorCode = ErrorCode::new(2001);
    pub const FUNCTION_ALREADY_DEFINED: ErrorCode = ErrorCode::new(2002);
    pub const UNDEFINED_FIELD: ErrorCode = ErrorCode::new(2003);
    pub const IMMUTABLE_ASSIGNMENT: ErrorCode = ErrorCode::new(2004);
    pub const INVALID_OPERATION: ErrorCode = ErrorCode::new(2005);
    pub const CIRCULAR_DEPENDENCY: ErrorCode = ErrorCode::new(2006);
    pub const INVALID_MODULE_PATH: ErrorCode = ErrorCode::new(2007);
    pub const UNREACHABLE_CODE: ErrorCode = ErrorCode::new(2010);
    pub const UNUSED_VARIABLE: ErrorCode = ErrorCode::new(2011);

    // Codegen errors (3000-3999)
    pub const STACK_OVERFLOW_CODEGEN: ErrorCode = ErrorCode::new(3000);
    pub const REGISTER_ALLOCATION_FAILED: ErrorCode = ErrorCode::new(3001);
    pub const BYTECODE_SIZE_LIMIT: ErrorCode = ErrorCode::new(3002);
    pub const INVALID_OPCODE: ErrorCode = ErrorCode::new(3010);

    // Runtime errors (4000-4999)
    pub const STACK_UNDERFLOW: ErrorCode = ErrorCode::new(4000);
    pub const DIVISION_BY_ZERO: ErrorCode = ErrorCode::new(4001);
    pub const INTEGER_OVERFLOW: ErrorCode = ErrorCode::new(4002);
    pub const ARRAY_INDEX_OUT_OF_BOUNDS: ErrorCode = ErrorCode::new(4003);
    pub const COMPUTE_UNIT_LIMIT: ErrorCode = ErrorCode::new(4010);
    pub const STACK_DEPTH_LIMIT: ErrorCode = ErrorCode::new(4011);

    // IO errors (5000-5999)
    pub const FILE_NOT_FOUND: ErrorCode = ErrorCode::new(5000);
    pub const PERMISSION_DENIED: ErrorCode = ErrorCode::new(5001);
    pub const IO_ERROR: ErrorCode = ErrorCode::new(5002);
    pub const INVALID_CONFIGURATION: ErrorCode = ErrorCode::new(5010);

    // Internal errors (9000-9999)
    pub const INTERNAL_ERROR: ErrorCode = ErrorCode::new(9000);
    pub const NOT_IMPLEMENTED: ErrorCode = ErrorCode::new(9001);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_display() {
        let code = ErrorCode::new(1);
        assert_eq!(format!("{}", code), "E0001");

        let code = ErrorCode::new(1000);
        assert_eq!(format!("{}", code), "E1000");
    }

    #[test]
    fn test_error_builder() {
        let error = ErrorBuilder::new(ErrorCode::EXPECTED_TOKEN, "test error".to_string())
            .severity(ErrorSeverity::Warning)
            .category(ErrorCategory::Syntax)
            .description("A test error".to_string())
            .build();

        assert_eq!(error.code, ErrorCode::EXPECTED_TOKEN);
        assert_eq!(error.severity, ErrorSeverity::Warning);
        assert_eq!(error.category, ErrorCategory::Syntax);
        assert_eq!(error.message, "test error");
        assert_eq!(error.description.unwrap(), "A test error");
    }

    #[test]
    fn test_source_location() {
        let loc = SourceLocation::new(10, 5, 100).with_length(3);

        assert_eq!(loc.line, 10);
        assert_eq!(loc.column, 5);
        assert_eq!(loc.offset, 100);
        assert_eq!(loc.length, 3);
    }

    #[test]
    fn test_error_context() {
        let context = ErrorContext::new()
            .with_expected(vec!["identifier".to_string(), "literal".to_string()])
            .with_found("operator".to_string())
            .add_data("phase".to_string(), "parsing".to_string());

        assert_eq!(context.expected.len(), 2);
        assert_eq!(context.found.as_ref().unwrap(), "operator");
        assert_eq!(context.get_data("phase").unwrap(), "parsing");
    }
}
