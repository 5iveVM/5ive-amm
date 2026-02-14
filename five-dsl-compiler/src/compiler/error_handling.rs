// Compiler error handling.

use crate::error::types::{ErrorBuilder, ErrorSeverity};
use crate::error::{integration, CompilerError, ErrorCategory, ErrorCode, SourceLocation};
use crate::metrics::MetricsCollector;
use five_vm_mito::error::VMError;
use std::path::PathBuf;

/// Convert VMError to CompilerError and collect it in error collector and metrics.
///
/// This is the central error handling function that eliminates 16 duplicate
/// error handling blocks throughout the compiler.
///
/// # Arguments
///
/// * `vm_error` - The VMError from tokenization/parsing/type checking/codegen
/// * `category` - ErrorCategory (Syntax, Type, Codegen, etc.)
/// * `phase` - Phase name for error reporting ("tokenization", "parsing", etc.)
/// * `source` - Source code for context extraction
/// * `error_collector` - Error collector to record the error
/// * `metrics` - Metrics collector to record the error
///
/// # Returns
///
/// Returns the CompilerError for propagation to caller
pub fn convert_and_collect_error(
    vm_error: VMError,
    category: ErrorCategory,
    phase: &str,
    source: &str,
    filename: Option<&str>,
    error_collector: &mut integration::ErrorCollector,
    metrics: &mut MetricsCollector,
) -> CompilerError {
    let compiler_error = convert_vm_error_to_compiler_error(vm_error, category, phase, source, filename);

    // Format error message before moving into collector
    let error_msg = format!("{}", compiler_error);

    // Record error in collector (takes ownership, clone necessary)
    // Note: error_collector.error() requires ownership of CompilerError,
    // and we also need to return it, so clone is unavoidable here.
    // This is acceptable as errors are uncommon and small.
    error_collector.error(compiler_error.clone());

    // Record simplified error in metrics
    metrics.record_error(&error_msg, phase);

    compiler_error
}

/// Convert VMError to enhanced CompilerError with context.
///
/// This is the core conversion function moved from compiler/mod.rs to eliminate
/// duplication. It provides detailed error messages with source locations.
///
/// # Arguments
///
/// * `vm_error` - The VMError to convert
/// * `category` - ErrorCategory for the resulting CompilerError
/// * `phase` - Compilation phase for context
/// * `source` - Source code for line/column calculation
///
/// # Returns
///
/// Returns a fully-formed CompilerError with context
pub fn convert_vm_error_to_compiler_error(
    vm_error: VMError,
    category: ErrorCategory,
    _phase: &str,
    source: &str,
    filename: Option<&str>,
) -> CompilerError {
    let file_path = PathBuf::from(filename.unwrap_or("input.v"));
    let (error_code, message) = match &vm_error {
        VMError::ParseError {
            expected,
            found,
            position,
        } => {
            let location = SourceLocation::new(
                position_to_line_col(*position, source).0 as u32,
                position_to_line_col(*position, source).1 as u32,
                *position,
            )
            .with_file(file_path);

            // Improve error message when expected is empty or generic
            let message = if expected.is_empty() {
                // Provide more context when expected is not specified
                if found.contains("type '") {
                    let type_hint = if found.contains("pubkey") {
                        "\n\nNote: `pubkey` is a valid type, but may be used in an invalid context."
                    } else if found.starts_with("type '") {
                        "\n\nNote: This appears to be a type that's not valid in the current context."
                    } else {
                        ""
                    };
                    format!("unexpected `{}` token at this location{}", found, type_hint)
                } else {
                    format!("unexpected `{}` token at this location", found)
                }
            } else if expected.len() < 20 && expected.contains("TokenKind") {
                // TokenKind debug format is not user-friendly
                format!("expected a different token, found `{}`", found)
            } else {
                format!("expected `{}`, found `{}`", expected, found)
            };

            let error = integration::parse_error(
                ErrorCode::EXPECTED_TOKEN,
                message,
                Some(location),
                Some(vec![expected.to_string()]),
                Some(found.to_string()),
            );
            return error;
        }
        VMError::UnexpectedToken => (ErrorCode::UNEXPECTED_TOKEN, "unexpected token".to_string()),
        VMError::UnexpectedEndOfInput => (
            ErrorCode::UNEXPECTED_EOF,
            "unexpected end of file".to_string(),
        ),
        VMError::InvalidScript => (
            ErrorCode::INVALID_SYNTAX,
            "invalid script syntax - check for syntax errors in accounts, functions, or statements".to_string(),
        ),
        VMError::TypeMismatch => (
            ErrorCode::TYPE_MISMATCH,
            "type mismatch in expression or assignment".to_string(),
        ),
        VMError::StackError => (
            ErrorCode::STACK_OVERFLOW_CODEGEN,
            "stack error during compilation".to_string(),
        ),
        // Semantic errors
        VMError::UndefinedIdentifier => (
            ErrorCode::UNDEFINED_VARIABLE,
            "undefined variable or identifier".to_string(),
        ),
        VMError::UndefinedField => (
            ErrorCode::UNDEFINED_FIELD,
            "undefined field access".to_string(),
        ),
        VMError::ImmutableField => (
            ErrorCode::IMMUTABLE_ASSIGNMENT,
            "assignment to immutable field".to_string(),
        ),
        VMError::InvalidOperation => (
            ErrorCode::INVALID_OPERATION,
            "invalid operation or method call".to_string(),
        ),
        VMError::InvalidParameterCount => (
            ErrorCode::FUNCTION_SIGNATURE_MISMATCH,
            "incorrect number of function parameters".to_string(),
        ),
        VMError::IndexOutOfBounds => (
            ErrorCode::ARRAY_INDEX_OUT_OF_BOUNDS,
            "array index out of bounds".to_string(),
        ),
        VMError::DivisionByZero => (ErrorCode::DIVISION_BY_ZERO, "division by zero".to_string()),
        VMError::CallStackOverflow => (
            ErrorCode::STACK_OVERFLOW_CODEGEN,
            "call stack overflow".to_string(),
        ),
        VMError::CallStackUnderflow => (
            ErrorCode::STACK_UNDERFLOW,
            "call stack underflow".to_string(),
        ),
        _ => (ErrorCode::INVALID_SYNTAX, format!("{:?}", vm_error)),
    };

    ErrorBuilder::new(error_code, message)
        .severity(ErrorSeverity::Error)
        .category(category)
        .build()
}

/// Convert byte position to line and column for error reporting.
///
/// Scans through the source code to find the line and column corresponding
/// to a byte position. Used for error location reporting.
///
/// # Arguments
///
/// * `position` - Byte position in source
/// * `source` - Source code string
///
/// # Returns
///
/// Returns `(line, column)` tuple (1-indexed)
pub fn position_to_line_col(position: usize, source: &str) -> (usize, usize) {
    let mut line = 1;
    let mut col = 1;

    for (i, ch) in source.chars().enumerate() {
        if i >= position {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }

    (line, col)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_to_line_col_first_line() {
        let source = "hello world";
        assert_eq!(position_to_line_col(0, source), (1, 1));
        assert_eq!(position_to_line_col(6, source), (1, 7));
    }

    #[test]
    fn test_position_to_line_col_multiple_lines() {
        let source = "line 1\nline 2\nline 3";
        assert_eq!(position_to_line_col(0, source), (1, 1)); // 'l' in line 1
        assert_eq!(position_to_line_col(7, source), (2, 1)); // 'l' in line 2
        assert_eq!(position_to_line_col(14, source), (3, 1)); // 'l' in line 3
    }

    #[test]
    fn test_position_to_line_col_end_of_file() {
        let source = "a\nb\nc";
        assert_eq!(position_to_line_col(100, source), (3, 2)); // Past EOF
    }

    #[test]
    fn test_convert_vm_error_undefined_identifier() {
        let source = "let x = y";
        let vm_error = VMError::UndefinedIdentifier;

        let compiler_error = convert_vm_error_to_compiler_error(
            vm_error,
            ErrorCategory::Type,
            "type checking",
            source,
            None,
        );

        assert!(compiler_error.to_string().contains("undefined variable"));
    }

    #[test]
    fn test_convert_vm_error_type_mismatch() {
        let source = "let x: u64 = \"hello\"";
        let vm_error = VMError::TypeMismatch;

        let compiler_error = convert_vm_error_to_compiler_error(
            vm_error,
            ErrorCategory::Type,
            "type checking",
            source,
            None,
        );

        assert!(compiler_error.to_string().contains("type mismatch"));
    }

    #[test]
    fn test_convert_and_collect_error() {
        let source = "invalid code";
        let vm_error = VMError::InvalidScript;
        let mut error_collector = integration::ErrorCollector::new();
        let mut metrics = MetricsCollector::new();

        let _compiler_error = convert_and_collect_error(
            vm_error,
            ErrorCategory::Syntax,
            "parsing",
            source,
            None,
            &mut error_collector,
            &mut metrics,
        );

        // Error should be recorded in both collectors
        let collected_metrics = metrics.get_metrics();
        assert!(
            !collected_metrics.error_patterns.error_frequency.is_empty(),
            "error frequency should record the converted error"
        );
    }
}
