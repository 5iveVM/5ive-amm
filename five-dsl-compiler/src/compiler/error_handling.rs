// Compiler error handling.

use crate::ast::{AstNode, SourceLocation as AstSourceLocation};
use crate::error::types::{ErrorBuilder, ErrorContext, ErrorSeverity};
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
    let compiler_error =
        convert_vm_error_to_compiler_error(vm_error, category, phase, source, filename);

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
        VMError::UnexpectedEndOfInput => {
            let eof_offset = source.chars().count();
            let (line, column) = position_to_line_col(eof_offset, source);
            let location = SourceLocation::new(line as u32, column as u32, eof_offset)
                .with_file(file_path.clone());

            return ErrorBuilder::new(
                ErrorCode::UNEXPECTED_EOF,
                "unexpected end of file".to_string(),
            )
            .severity(ErrorSeverity::Error)
            .category(category)
            .description(
                "The parser reached the end of the file before the statement or block was complete."
                    .to_string(),
            )
            .location(location)
            .build();
        }
        VMError::InvalidScript => (
            ErrorCode::INVALID_SYNTAX,
            "invalid script syntax - check for syntax errors in accounts, functions, or statements"
                .to_string(),
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
        VMError::UndefinedIdentifierWithContext {
            identifier,
            did_you_mean,
        } => {
            let identifier_text = identifier.to_string();
            let location = find_identifier_location(source, &identifier_text, &file_path);
            let mut context = ErrorContext::new().with_identifier(identifier_text.clone());
            let mut description =
                "This identifier is not declared in the current scope.".to_string();
            if let Some(candidate) = did_you_mean {
                let candidate_str = candidate.to_string();
                context = context.add_data("did_you_mean".to_string(), candidate.to_string());
                if candidate_str.contains(".ctx.") || candidate_str.starts_with("ctx.") {
                    description.push_str(" Account metadata moved under `account.ctx.*`.");
                }
            }

            let mut builder = ErrorBuilder::new(
                ErrorCode::UNDEFINED_VARIABLE,
                format!("cannot find value `{}` in this scope", identifier_text),
            )
            .severity(ErrorSeverity::Error)
            .category(category)
            .description(description)
            .context(context);

            if let Some(loc) = location {
                builder = builder.location(loc);
            }

            return builder.build();
        }
        VMError::DuplicateImport {
            symbol,
            namespace,
            ..
        } => {
            let symbol_text = symbol.to_string();
            let namespace_text = namespace.to_string();
            let location = find_identifier_location(source, &symbol_text, &file_path);
            return build_duplicate_import_error(
                symbol_text,
                namespace_text,
                None,
                location,
                category,
            );
        }
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

pub fn convert_vm_error_to_compiler_error_with_ast(
    vm_error: VMError,
    ast: &AstNode,
    category: ErrorCategory,
    phase: &str,
    source: &str,
    filename: Option<&str>,
) -> CompilerError {
    if let VMError::DuplicateImport {
        ref symbol,
        ref namespace,
        import_ordinal,
    } = vm_error
    {
        let file_path = PathBuf::from(filename.unwrap_or("input.v"));
        let symbol_text = symbol.to_string();
        let namespace_text = namespace.to_string();
        let location = find_import_location_in_ast(ast, import_ordinal as usize, source, &file_path)
            .or_else(|| find_identifier_location(source, &symbol_text, &file_path));

        return build_duplicate_import_error(
            symbol_text,
            namespace_text,
            Some(import_ordinal),
            location,
            category,
        );
    }

    convert_vm_error_to_compiler_error(vm_error, category, phase, source, filename)
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

fn find_identifier_location(
    source: &str,
    identifier: &str,
    file_path: &PathBuf,
) -> Option<SourceLocation> {
    if identifier.is_empty() {
        return None;
    }

    let mut start_index = 0usize;
    while start_index <= source.len() {
        let relative = source[start_index..].find(identifier)?;
        let absolute = start_index + relative;
        let end = absolute + identifier.len();

        let prev = source[..absolute].chars().next_back();
        let next = source[end..].chars().next();
        let starts_at_boundary = prev.map(|c| !is_identifier_char(c)).unwrap_or(true);
        let ends_at_boundary = next.map(|c| !is_identifier_char(c)).unwrap_or(true);

        if starts_at_boundary && ends_at_boundary {
            let (line, column) = position_to_line_col(absolute, source);
            return Some(
                SourceLocation::new(line as u32, column as u32, absolute)
                    .with_file(file_path.clone())
                    .with_length(identifier.chars().count()),
            );
        }

        start_index = end;
    }

    None
}

fn find_import_location_in_ast(
    ast: &AstNode,
    import_ordinal: usize,
    source: &str,
    file_path: &PathBuf,
) -> Option<SourceLocation> {
    let AstNode::Program {
        import_statements, ..
    } = ast
    else {
        return None;
    };

    let AstNode::ImportStatement {
        location: Some(location),
        ..
    } = import_statements.get(import_ordinal)?
    else {
        return None;
    };

    Some(ast_location_to_compiler_location(*location, source, file_path))
}

fn ast_location_to_compiler_location(
    location: AstSourceLocation,
    source: &str,
    file_path: &PathBuf,
) -> SourceLocation {
    let offset = source
        .lines()
        .take(location.line as usize)
        .map(|line| line.chars().count() + 1)
        .sum::<usize>()
        .saturating_add(location.column as usize);

    SourceLocation::new(location.line + 1, location.column + 1, offset)
        .with_length(location.length as usize)
        .with_file(file_path.clone())
}

fn build_duplicate_import_error(
    symbol_text: String,
    namespace_text: String,
    import_ordinal: Option<u32>,
    location: Option<SourceLocation>,
    category: ErrorCategory,
) -> CompilerError {
    let mut context = ErrorContext::new()
        .with_identifier(symbol_text.clone())
        .add_data("namespace".to_string(), namespace_text.clone());

    if let Some(import_ordinal) = import_ordinal {
        context = context.add_data("import_ordinal".to_string(), import_ordinal.to_string());
    }

    let mut builder = ErrorBuilder::new(
        ErrorCode::INVALID_OPERATION,
        format!(
            "duplicate imported {} symbol `{}`",
            namespace_text, symbol_text
        ),
    )
    .severity(ErrorSeverity::Error)
    .category(category)
    .description(
        "Each imported symbol name must be unique within its namespace. Rename one import or use the module path explicitly."
            .to_string(),
    )
    .context(context);

    if let Some(loc) = location {
        builder = builder.location(loc);
    }

    builder.build()
}

fn is_identifier_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
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
    fn test_convert_vm_error_undefined_identifier_with_context() {
        let source = "let amount: u64 = 1;\nreturn ammount;";
        let vm_error = VMError::undefined_identifier("ammount", Some("amount"));

        let compiler_error = convert_vm_error_to_compiler_error(
            vm_error,
            ErrorCategory::Type,
            "type checking",
            source,
            Some("test.v"),
        );

        assert_eq!(compiler_error.code, ErrorCode::UNDEFINED_VARIABLE);
        assert_eq!(
            compiler_error.context.identifier.as_deref(),
            Some("ammount")
        );
        assert_eq!(
            compiler_error
                .context
                .get_data("did_you_mean")
                .map(String::as_str),
            Some("amount")
        );
        assert!(compiler_error.location.is_some());
    }

    #[test]
    fn test_convert_vm_error_undefined_identifier_with_ctx_migration_note() {
        let source = "return payer.key;";
        let vm_error = VMError::undefined_identifier("key", Some("ctx.key"));

        let compiler_error = convert_vm_error_to_compiler_error(
            vm_error,
            ErrorCategory::Type,
            "type checking",
            source,
            Some("test.v"),
        );

        assert_eq!(
            compiler_error
                .context
                .get_data("did_you_mean")
                .map(String::as_str),
            Some("ctx.key")
        );
        let description = compiler_error.description.unwrap_or_default();
        assert!(
            description.contains("account.ctx.*"),
            "expected migration note in description, got: {}",
            description
        );
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
