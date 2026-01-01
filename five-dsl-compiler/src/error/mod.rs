//! Five VM Enhanced Error System
//!
//! This module provides a comprehensive, maintainable error handling system
//! for the Five DSL compiler, designed for easy updates and extensions.
//!
//! ## Architecture Overview
//!
//! The error system is built around several core components:
//! - **Error Types**: Structured error definitions with rich context
//! - **Error Registry**: Centralized registry of all error codes and templates
//! - **Formatters**: Pluggable error display formatters (terminal, JSON, LSP)
//! - **Suggestions**: Intelligent suggestion engine for error fixes
//! - **Templates**: Configurable error message templates
//!
//! ## Design Principles
//!
//! - **Modular**: Each component can be updated independently
//! - **Configurable**: Error messages can be updated without recompilation
//! - **Extensible**: New error types and formatters can be added easily
//! - **Maintainable**: Clear separation of concerns and plugin architecture

pub mod context;
pub mod formatting;
pub mod integration;
pub mod registry;
pub mod suggestions;
pub mod templates;
pub mod types;

// Public API re-exports for easy access
pub use context::*;
pub use formatting::*;
pub use integration::*;
pub use registry::*;
pub use suggestions::*;
pub use templates::*;
pub use types::*;

use std::sync::LazyLock;
use toml;

/// Global error system instance
///
/// This provides a singleton access point to the error system, initialized
/// with default configuration. Can be reloaded with custom configuration.
pub static ERROR_SYSTEM: LazyLock<std::sync::RwLock<ErrorSystem>> = LazyLock::new(|| {
    let system = ErrorSystem::with_default_config();
    std::sync::RwLock::new(system)
});

/// Main error system coordinator
///
/// This is the primary interface for the error system, coordinating between
/// the registry, formatters, and suggestion engine.
pub struct ErrorSystem {
    registry: ErrorRegistry,
    formatter: Box<dyn ErrorFormatter + Send + Sync>,
    suggestion_engine: SuggestionEngine,
}

impl ErrorSystem {
    /// Create a new error system with default configuration
    pub fn new() -> Self {
        Self {
            registry: ErrorRegistry::new(),
            formatter: Box::new(TerminalFormatter::new()),
            suggestion_engine: SuggestionEngine::new(),
        }
    }

    /// Create error system with default configuration loaded from embedded config
    pub fn with_default_config() -> Self {
        let mut system = Self::new();

        // Load default error configuration
        let default_config = include_str!("default_errors.toml");
        if let Err(e) = system.load_config(default_config) {
            eprintln!("Warning: Failed to load default error config: {}", e);
        }

        system
    }

    /// Load error configuration from TOML string
    pub fn load_config(&mut self, config_str: &str) -> Result<(), Box<dyn std::error::Error>> {
        let config: toml::Value = toml::from_str(config_str)?;
        self.registry.load_from_config(&config)?;
        Ok(())
    }

    /// Reload configuration (useful for development hot-reload)
    pub fn reload_config(&mut self, config_str: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.registry.clear();
        self.load_config(config_str)
    }

    /// Set the error formatter
    pub fn set_formatter<F>(&mut self, formatter: F)
    where
        F: ErrorFormatter + Send + Sync + 'static,
    {
        self.formatter = Box::new(formatter);
    }

    /// Format an error for display
    pub fn format_error(&self, error: &CompilerError) -> String {
        self.formatter.format_error(error)
    }

    /// Generate suggestions for an error
    pub fn generate_suggestions(&self, error: &CompilerError) -> Vec<Suggestion> {
        self.suggestion_engine.generate_suggestions(error)
    }

    /// Create a comprehensive error report with context and suggestions
    pub fn create_error_report(&self, error: &CompilerError) -> ErrorReport {
        let suggestions = self.generate_suggestions(error);
        let formatted = self.format_error(error);

        ErrorReport {
            error: error.clone(),
            formatted_message: formatted,
            suggestions,
        }
    }

    /// Add a custom suggestion rule
    pub fn add_suggestion_rule<R>(&mut self, rule: R)
    where
        R: SuggestionRule + Send + Sync + 'static,
    {
        self.suggestion_engine.add_rule(Box::new(rule));
    }

    /// Format multiple errors as a batch
    pub fn format_errors(&self, errors: &[CompilerError]) -> String {
        self.formatter.format_errors(errors)
    }

    /// Set the error registry
    pub fn set_registry(&mut self, registry: ErrorRegistry) {
        self.registry = registry;
    }

    /// Set the template manager
    pub fn set_template_manager(&mut self, _template_manager: TemplateManager) {
        // Template manager integration will be implemented later
        // For now, this is a placeholder to maintain the interface
    }

    /// Check if the error system has a registry loaded
    pub fn has_registry(&self) -> bool {
        !self.registry.get_all_codes().is_empty()
    }
}

impl Default for ErrorSystem {
    fn default() -> Self {
        Self::with_default_config()
    }
}

/// Comprehensive error report
#[derive(Debug, Clone)]
pub struct ErrorReport {
    pub error: CompilerError,
    pub formatted_message: String,
    pub suggestions: Vec<Suggestion>,
}

/// Convenience function to access the global error system
pub fn error_system() -> std::sync::RwLockReadGuard<'static, ErrorSystem> {
    ERROR_SYSTEM.read().unwrap_or_else(|e| {
        eprintln!("Global error system poisoned: {e}; resetting to default");
        {
            let mut writer = ERROR_SYSTEM.write().unwrap_or_else(|e2| {
                eprintln!("Failed to acquire write lock during reset: {e2}");
                e2.into_inner()
            });
            *writer = ErrorSystem::default();
        }
        ERROR_SYSTEM.read().unwrap_or_else(|e3| {
            eprintln!("Read lock poisoned after reset: {e3}; using last state");
            e3.into_inner()
        })
    })
}

/// Convenience function to access the global error system mutably
pub fn error_system_mut() -> std::sync::RwLockWriteGuard<'static, ErrorSystem> {
    ERROR_SYSTEM.write().unwrap_or_else(|e| {
        eprintln!("Global error system poisoned: {e}; returning default");
        let mut guard = e.into_inner();
        *guard = ErrorSystem::default();
        guard
    })
}

/// Format an error using an error system
pub fn format_error(system: &ErrorSystem, error: &CompilerError) -> String {
    system.format_error(error)
}

/// Generate suggestions using an error system
pub fn suggest_fixes(system: &ErrorSystem, error: &CompilerError) -> Vec<Suggestion> {
    system.generate_suggestions(error)
}

/// Create a complete error report using an error system
pub fn create_error_report(system: &ErrorSystem, error: &CompilerError) -> ErrorReport {
    system.create_error_report(error)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_system_initialization() {
        let _system = ErrorSystem::new();

        // Should initialize without panicking
        assert!(true);
    }

    #[test]
    fn test_global_error_system_access() {
        // Should be able to access global error system
        let _system = error_system();
        assert!(true);
    }

    #[test]
    fn test_config_loading() {
        let mut system = ErrorSystem::new();

        let test_config = r#"
        [errors.E0001]
        category = "test"
        title = "Test error"
        description = "A test error for validation"
        "#;

        system
            .load_config(test_config)
            .expect("Should load test config");
    }
}
