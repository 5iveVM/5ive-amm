//! Error system integration module
//!
//! This module provides high-level integration functions for the error system,
//! making it easy to use the enhanced error capabilities throughout the compiler.

use crate::error::{
    formatting::{JsonFormatter, LspFormatter, TerminalFormatter},
    registry::load_registry_from_string,
    templates::create_common_templates,
    types::{CompilerError, ErrorBuilder, ErrorCategory, ErrorCode, ErrorSeverity, SourceLocation},
    ErrorSystem,
};
use std::sync::LazyLock;

/// Global error system instance
static GLOBAL_ERROR_SYSTEM: LazyLock<std::sync::RwLock<ErrorSystem>> =
    LazyLock::new(|| std::sync::RwLock::new(ErrorSystem::with_default_config()));

/// Initialize the global error system with default configuration
pub fn initialize_error_system() -> Result<(), Box<dyn std::error::Error>> {
    let mut system = GLOBAL_ERROR_SYSTEM.write().unwrap_or_else(|e| {
        eprintln!("Global error system poisoned during init: {e}; using default");
        let mut guard = e.into_inner();
        *guard = ErrorSystem::default();
        guard
    });

    // Load default error registry
    let default_config = include_str!("default_errors.toml");
    let registry = load_registry_from_string(default_config)?;
    system.set_registry(registry);

    // Set up template manager
    let template_manager = create_common_templates();
    system.set_template_manager(template_manager);

    Ok(())
}

/// Get the global error system (read-only)
pub fn get_error_system() -> std::sync::RwLockReadGuard<'static, ErrorSystem> {
    GLOBAL_ERROR_SYSTEM.read().unwrap_or_else(|e| {
        eprintln!("Global error system read lock poisoned: {e}; resetting to default");
        {
            let mut writer = GLOBAL_ERROR_SYSTEM.write().unwrap_or_else(|e2| {
                eprintln!("Failed to acquire write lock during reset: {e2}");
                e2.into_inner()
            });
            *writer = ErrorSystem::default();
        }
        GLOBAL_ERROR_SYSTEM.read().unwrap_or_else(|e3| {
            eprintln!("Read lock poisoned after reset: {e3}; using last state");
            e3.into_inner()
        })
    })
}

/// Get the global error system (mutable)  
pub fn get_error_system_mut() -> std::sync::RwLockWriteGuard<'static, ErrorSystem> {
    GLOBAL_ERROR_SYSTEM.write().unwrap_or_else(|e| {
        eprintln!("Global error system write lock poisoned: {e}; returning default");
        let mut guard = e.into_inner();
        *guard = ErrorSystem::default();
        guard
    })
}

/// Format an error using a provided error system
pub fn format_error(system: &ErrorSystem, error: &CompilerError) -> String {
    system.format_error(error)
}

/// Format multiple errors using a provided error system
pub fn format_errors(system: &ErrorSystem, errors: &[CompilerError]) -> String {
    system.format_errors(errors)
}

/// Generate suggestions for an error using a provided error system
pub fn generate_suggestions(
    system: &ErrorSystem,
    error: &CompilerError,
) -> Vec<crate::error::suggestions::Suggestion> {
    system.generate_suggestions(error)
}

/// Set the formatter for an error system
pub fn set_formatter(system: &mut ErrorSystem, formatter_name: &str) -> Result<(), String> {
    match formatter_name {
        "terminal" => system.set_formatter(TerminalFormatter::new()),
        "json" => system.set_formatter(JsonFormatter::new()),
        "json-pretty" => system.set_formatter(JsonFormatter::pretty()),
        "lsp" => system.set_formatter(LspFormatter::new()),
        _ => return Err(format!("Unknown formatter: {}", formatter_name)),
    }

    Ok(())
}

/// Quick error creation functions for common scenarios

/// Create a parse error with rich context
pub fn parse_error(
    code: ErrorCode,
    message: String,
    location: Option<SourceLocation>,
    expected: Option<Vec<String>>,
    found: Option<String>,
) -> CompilerError {
    let mut builder = ErrorBuilder::new(code, message)
        .severity(ErrorSeverity::Error)
        .category(ErrorCategory::Syntax);

    if let Some(loc) = location {
        builder = builder.location(loc);
    }

    let mut context = crate::error::types::ErrorContext::new();
    if let Some(exp) = expected {
        context = context.with_expected(exp);
    }
    if let Some(fnd) = found {
        context = context.with_found(fnd);
    }

    builder.context(context).build()
}

/// Create a parse error with enhanced context and suggestions
pub fn parse_error_with_context(
    code: ErrorCode,
    message: String,
    location: Option<SourceLocation>,
    expected: Option<Vec<String>>,
    found: Option<String>,
    suggestions: Vec<String>,
) -> CompilerError {
    let mut builder = ErrorBuilder::new(code, message)
        .severity(ErrorSeverity::Error)
        .category(ErrorCategory::Syntax);

    if let Some(loc) = location {
        builder = builder.location(loc);
    }

    let mut context = crate::error::types::ErrorContext::new();
    if let Some(exp) = expected {
        // Pass the full expected vector
        if !exp.is_empty() {
            context = context.with_expected(exp);
        }
    }
    if let Some(fnd) = found {
        context = context.with_found(fnd);
    }

    // Add suggestions to the error
    let mut error = builder.context(context).build();

    // Use the suggestion system to add helpful suggestions
    if let Ok(system) = crate::error::ERROR_SYSTEM.read() {
        let generated_suggestions = system.suggestion_engine.generate_suggestions(&error);

        // Combine manual suggestions with generated ones
        let mut all_suggestions = suggestions
            .into_iter()
            .map(|s| crate::error::suggestions::Suggestion {
                message: s,
                confidence: 0.8,
                explanation: None,
                suggestion_type: crate::error::suggestions::SuggestionType::General,
                code_fix: None,
                location: None,
            })
            .collect::<Vec<_>>();

        all_suggestions.extend(generated_suggestions);

        // Apply suggestions to the error (this would require modifying the error structure)
        // Include in the description.
        if !all_suggestions.is_empty() {
            let suggestion_text = all_suggestions
                .iter()
                .take(3) // Limit to top 3 suggestions
                .map(|s| format!("  • {}", s.message))
                .collect::<Vec<_>>()
                .join("\n");

            error.description = Some(format!(
                "{}\n\nSuggestions:\n{}",
                error.description.unwrap_or_default(),
                suggestion_text
            ));
        }
    }

    error
}

/// Create a type error with type information
pub fn type_error(
    code: ErrorCode,
    message: String,
    location: Option<SourceLocation>,
    expected_type: Option<String>,
    actual_type: Option<String>,
) -> CompilerError {
    let mut builder = ErrorBuilder::new(code, message)
        .severity(ErrorSeverity::Error)
        .category(ErrorCategory::Type);

    if let Some(loc) = location {
        builder = builder.location(loc);
    }

    let mut context = crate::error::types::ErrorContext::new();
    if let (Some(expected), Some(actual)) = (expected_type, actual_type) {
        context = context.with_types(expected, actual);
    }

    builder.context(context).build()
}

/// Create a semantic error with identifier context
pub fn semantic_error(
    code: ErrorCode,
    message: String,
    location: Option<SourceLocation>,
    identifier: Option<String>,
) -> CompilerError {
    let mut builder = ErrorBuilder::new(code, message)
        .severity(ErrorSeverity::Error)
        .category(ErrorCategory::Semantic);

    if let Some(loc) = location {
        builder = builder.location(loc);
    }

    let mut context = crate::error::types::ErrorContext::new();
    if let Some(id) = identifier {
        context = context.with_identifier(id);
    }

    builder.context(context).build()
}

/// Create a codegen error with optional runtime context
pub fn codegen_error(
    code: ErrorCode,
    message: String,
    location: Option<SourceLocation>,
    context_data: Option<std::collections::HashMap<String, String>>,
) -> CompilerError {
    let mut builder = ErrorBuilder::new(code, message)
        .severity(ErrorSeverity::Error)
        .category(ErrorCategory::Codegen);

    if let Some(loc) = location {
        builder = builder.location(loc);
    }

    let mut context = crate::error::types::ErrorContext::new();
    if let Some(data) = context_data {
        context = context.with_data(data);
    }

    builder.context(context).build()
}

/// Create a warning with optional suggestion
pub fn warning(
    code: ErrorCode,
    message: String,
    location: Option<SourceLocation>,
    help: Option<String>,
) -> CompilerError {
    let mut builder = ErrorBuilder::new(code, message)
        .severity(ErrorSeverity::Warning)
        .category(ErrorCategory::Semantic);

    if let Some(loc) = location {
        builder = builder.location(loc);
    }

    if let Some(help_text) = help {
        let related = crate::error::types::RelatedError::help(help_text);
        builder = builder.related(related);
    }

    builder.build()
}

/// Utility functions for error collection and reporting

/// Error collector for gathering multiple errors during compilation
pub struct ErrorCollector {
    errors: Vec<CompilerError>,
    warnings: Vec<CompilerError>,
    max_errors: usize,
}

impl ErrorCollector {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
            max_errors: 100,
        }
    }

    pub fn with_max_errors(max_errors: usize) -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
            max_errors,
        }
    }

    /// Add an error to the collector
    pub fn error(&mut self, error: CompilerError) {
        if error.is_error() {
            if self.errors.len() < self.max_errors {
                self.errors.push(error);
            }
        } else if error.is_warning() {
            self.warnings.push(error);
        }
    }

    /// Add multiple errors
    pub fn errors(&mut self, errors: Vec<CompilerError>) {
        for error in errors {
            self.error(error);
        }
    }

    /// Check if there are any errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Check if there are any warnings
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    /// Get error count
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    /// Get warning count  
    pub fn warning_count(&self) -> usize {
        self.warnings.len()
    }

    /// Get all errors
    pub fn get_errors(&self) -> &[CompilerError] {
        &self.errors
    }

    /// Get all warnings
    pub fn get_warnings(&self) -> &[CompilerError] {
        &self.warnings
    }

    /// Get all errors and warnings combined
    pub fn get_all(&self) -> Vec<&CompilerError> {
        let mut all = Vec::new();
        all.extend(self.errors.iter());
        all.extend(self.warnings.iter());
        all
    }

    /// Format all collected errors and warnings
    pub fn format_all(&self, system: &ErrorSystem) -> String {
        let all_errors: Vec<CompilerError> = self.get_all().into_iter().cloned().collect();
        format_errors(system, &all_errors)
    }

    /// Clear all collected errors and warnings
    pub fn clear(&mut self) {
        self.errors.clear();
        self.warnings.clear();
    }
}

impl Default for ErrorCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Result type for compiler operations that can produce multiple errors
pub type CompilerResult<T> = Result<T, ErrorCollector>;

/// Extension trait for Results to easily convert to CompilerResult
pub trait IntoCompilerResult<T> {
    fn into_compiler_result(self) -> CompilerResult<T>;
}

impl<T> IntoCompilerResult<T> for Result<T, CompilerError> {
    fn into_compiler_result(self) -> CompilerResult<T> {
        match self {
            Ok(value) => Ok(value),
            Err(error) => {
                let mut collector = ErrorCollector::new();
                collector.error(error);
                Err(collector)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::types::ErrorCode;

    #[test]
    fn test_error_system_initialization() {
        initialize_error_system().expect("Failed to initialize error system");

        let system = get_error_system();
        assert!(system.has_registry());
    }

    #[test]
    fn test_parse_error_creation() {
        let error = parse_error(
            ErrorCode::EXPECTED_TOKEN,
            "expected `;`, found `}`".to_string(),
            Some(SourceLocation::new(5, 10, 100)),
            Some(vec![";".to_string()]),
            Some("}".to_string()),
        );

        assert_eq!(error.code, ErrorCode::EXPECTED_TOKEN);
        assert_eq!(error.category, ErrorCategory::Syntax);
        assert!(error.location.is_some());
        assert!(error.context.expected.contains(&";".to_string()));
    }

    #[test]
    fn test_error_collector() {
        let mut collector = ErrorCollector::new();

        let error1 = parse_error(
            ErrorCode::EXPECTED_TOKEN,
            "error 1".to_string(),
            None,
            None,
            None,
        );

        let warning1 = warning(
            ErrorCode::UNUSED_VARIABLE,
            "warning 1".to_string(),
            None,
            Some("Consider removing this variable".to_string()),
        );

        collector.error(error1);
        collector.error(warning1);

        assert_eq!(collector.error_count(), 1);
        assert_eq!(collector.warning_count(), 1);
        assert!(collector.has_errors());
        assert!(collector.has_warnings());
    }

    #[test]
    fn test_formatter_switching() {
        initialize_error_system().expect("Failed to initialize");
        {
            let mut sys = get_error_system_mut();
            assert!(set_formatter(&mut sys, "terminal").is_ok());
        }
        {
            let mut sys = get_error_system_mut();
            assert!(set_formatter(&mut sys, "json").is_ok());
        }
        {
            let mut sys = get_error_system_mut();
            assert!(set_formatter(&mut sys, "lsp").is_ok());
        }
        {
            let mut sys = get_error_system_mut();
            assert!(set_formatter(&mut sys, "invalid").is_err());
        }
    }
}
