//! Error registry and configuration.

use crate::error::types::{ErrorCategory, ErrorCode, ErrorSeverity};
use std::collections::HashMap;
use toml;

/// Error registry that manages error definitions and templates.
/// be loaded from external files for easy maintenance and updates.
pub struct ErrorRegistry {
    /// Error definitions indexed by error code
    error_definitions: HashMap<ErrorCode, ErrorDefinition>,

    /// Error categories for organization
    categories: HashMap<String, ErrorCategory>,

    /// Template cache for performance
    template_cache: HashMap<ErrorCode, String>,
}

impl ErrorRegistry {
    /// Create a new empty error registry
    pub fn new() -> Self {
        Self {
            error_definitions: HashMap::new(),
            categories: HashMap::new(),
            template_cache: HashMap::new(),
        }
    }

    /// Clear all registry data
    pub fn clear(&mut self) {
        self.error_definitions.clear();
        self.categories.clear();
        self.template_cache.clear();
    }

    /// Register a new error definition
    pub fn register(&mut self, error: ErrorDefinition) {
        self.error_definitions.insert(error.code, error);
        self.template_cache.clear(); // Invalidate cache
    }

    /// Get an error definition by code
    pub fn get(&self, code: ErrorCode) -> Option<&ErrorDefinition> {
        self.error_definitions.get(&code)
    }

    /// Get all registered error codes
    pub fn get_all_codes(&self) -> Vec<ErrorCode> {
        self.error_definitions.keys().copied().collect()
    }

    /// Get errors by category
    pub fn get_by_category(&self, category: &ErrorCategory) -> Vec<&ErrorDefinition> {
        self.error_definitions
            .values()
            .filter(|def| &def.category == category)
            .collect()
    }

    /// Load error definitions from TOML configuration
    pub fn load_from_config(&mut self, config: &toml::Value) -> Result<(), RegistryError> {
        let errors = config
            .get("errors")
            .ok_or(RegistryError::MissingSection("errors".to_string()))?
            .as_table()
            .ok_or(RegistryError::InvalidFormat(
                "errors section must be a table".to_string(),
            ))?;

        for (code_str, error_config) in errors {
            let error_def = self.parse_error_definition(code_str, error_config)?;
            self.register(error_def);
        }

        Ok(())
    }

    /// Parse a single error definition from configuration
    fn parse_error_definition(
        &self,
        code_str: &str,
        config: &toml::Value,
    ) -> Result<ErrorDefinition, RegistryError> {
        // Parse error code (e.g., "E0001" -> ErrorCode(1))
        let code = self.parse_error_code(code_str)?;

        let config_table = config.as_table().ok_or_else(|| {
            RegistryError::InvalidFormat(format!("Error {} must be a table", code_str))
        })?;

        // Required fields
        let title = config_table
            .get("title")
            .and_then(|v| v.as_str())
            .ok_or_else(|| RegistryError::MissingField(code_str.to_string(), "title".to_string()))?
            .to_string();

        let category_str = config_table
            .get("category")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                RegistryError::MissingField(code_str.to_string(), "category".to_string())
            })?;

        let category = self.parse_category(category_str)?;

        // Optional fields
        let description = config_table
            .get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let severity = config_table
            .get("severity")
            .and_then(|v| v.as_str())
            .map(|s| self.parse_severity(s))
            .transpose()?
            .unwrap_or(ErrorSeverity::Error);

        let help = config_table
            .get("help")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // Parse template
        let template = config_table
            .get("template")
            .and_then(|v| v.as_str())
            .unwrap_or(&title)
            .to_string();

        // Parse suggestions
        let suggestions = self.parse_suggestions(config_table)?;

        // Parse examples
        let examples = self.parse_examples(config_table)?;

        Ok(ErrorDefinition {
            code,
            title,
            description,
            category,
            severity,
            template,
            help,
            suggestions,
            examples,
        })
    }

    /// Parse error code string (e.g., "E0001" -> ErrorCode(1))
    fn parse_error_code(&self, code_str: &str) -> Result<ErrorCode, RegistryError> {
        if !code_str.starts_with('E') {
            return Err(RegistryError::InvalidErrorCode(code_str.to_string()));
        }

        let number_str = &code_str[1..];
        let number: u32 = number_str
            .parse()
            .map_err(|_| RegistryError::InvalidErrorCode(code_str.to_string()))?;

        Ok(ErrorCode::new(number))
    }

    /// Parse error category
    fn parse_category(&self, category_str: &str) -> Result<ErrorCategory, RegistryError> {
        match category_str {
            "syntax" => Ok(ErrorCategory::Syntax),
            "type" => Ok(ErrorCategory::Type),
            "semantic" => Ok(ErrorCategory::Semantic),
            "codegen" => Ok(ErrorCategory::Codegen),
            "io" => Ok(ErrorCategory::IO),
            "internal" => Ok(ErrorCategory::Internal),
            _ => Ok(ErrorCategory::Custom(category_str.to_string())),
        }
    }

    /// Parse error severity
    fn parse_severity(&self, severity_str: &str) -> Result<ErrorSeverity, RegistryError> {
        match severity_str {
            "error" => Ok(ErrorSeverity::Error),
            "warning" => Ok(ErrorSeverity::Warning),
            "note" => Ok(ErrorSeverity::Note),
            "help" => Ok(ErrorSeverity::Help),
            _ => Err(RegistryError::InvalidSeverity(severity_str.to_string())),
        }
    }

    /// Parse suggestions array
    fn parse_suggestions(
        &self,
        config: &toml::Table,
    ) -> Result<Vec<SuggestionTemplate>, RegistryError> {
        let Some(suggestions_value) = config.get("suggestions") else {
            return Ok(Vec::new());
        };

        let suggestions_array = suggestions_value.as_array().ok_or_else(|| {
            RegistryError::InvalidFormat("suggestions must be an array".to_string())
        })?;

        let mut suggestions = Vec::new();
        for (i, suggestion_value) in suggestions_array.iter().enumerate() {
            let suggestion_table = suggestion_value.as_table().ok_or_else(|| {
                RegistryError::InvalidFormat(format!("suggestion {} must be a table", i))
            })?;

            let condition = suggestion_table
                .get("condition")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let text = suggestion_table
                .get("text")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    RegistryError::MissingField(format!("suggestion_{}", i), "text".to_string())
                })?
                .to_string();

            let fix = suggestion_table
                .get("fix")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            suggestions.push(SuggestionTemplate {
                condition,
                text,
                fix,
            });
        }

        Ok(suggestions)
    }

    /// Parse examples
    fn parse_examples(&self, config: &toml::Table) -> Result<Vec<ErrorExample>, RegistryError> {
        let Some(examples_value) = config.get("examples") else {
            return Ok(Vec::new());
        };

        let examples_table = examples_value
            .as_table()
            .ok_or_else(|| RegistryError::InvalidFormat("examples must be a table".to_string()))?;

        let mut examples = Vec::new();

        if let (Some(before), Some(after)) = (
            examples_table.get("before").and_then(|v| v.as_str()),
            examples_table.get("after").and_then(|v| v.as_str()),
        ) {
            examples.push(ErrorExample {
                before: before.to_string(),
                after: after.to_string(),
                description: examples_table
                    .get("description")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
            });
        }

        Ok(examples)
    }
}

impl Default for ErrorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Error definition.
#[derive(Debug, Clone)]
pub struct ErrorDefinition {
    /// Error code
    pub code: ErrorCode,

    /// Short error title/message
    pub title: String,

    /// Detailed description
    pub description: Option<String>,

    /// Error category
    pub category: ErrorCategory,

    /// Error severity
    pub severity: ErrorSeverity,

    /// Message template with placeholders
    pub template: String,

    /// Help text
    pub help: Option<String>,

    /// Suggestion templates
    pub suggestions: Vec<SuggestionTemplate>,

    /// Code examples
    pub examples: Vec<ErrorExample>,
}

impl ErrorDefinition {
    /// Create a new error definition
    pub fn new(code: ErrorCode, title: String, category: ErrorCategory) -> Self {
        Self {
            code,
            title: title.clone(),
            description: None,
            category,
            severity: ErrorSeverity::Error,
            template: title,
            help: None,
            suggestions: Vec::new(),
            examples: Vec::new(),
        }
    }

    /// Render the error message with context
    pub fn render_message(&self, context: &HashMap<String, String>) -> String {
        let mut message = self.template.clone();

        // Replace placeholders in template
        for (key, value) in context {
            let placeholder = format!("{{{}}}", key);
            message = message.replace(&placeholder, value);
        }

        message
    }

    /// Get applicable suggestions based on context
    pub fn get_applicable_suggestions(
        &self,
        context: &HashMap<String, String>,
    ) -> Vec<&SuggestionTemplate> {
        self.suggestions
            .iter()
            .filter(|suggestion| {
                // If no condition, suggestion always applies
                let Some(condition) = &suggestion.condition else {
                    return true;
                };

                // Check if condition matches context
                context
                    .get("condition")
                    .map(|ctx_condition| ctx_condition == condition)
                    .unwrap_or(false)
            })
            .collect()
    }
}

/// Template for error suggestions
#[derive(Debug, Clone)]
pub struct SuggestionTemplate {
    /// Condition when this suggestion applies
    pub condition: Option<String>,

    /// Suggestion text
    pub text: String,

    /// Optional code fix
    pub fix: Option<String>,
}

/// Error example with before/after code
#[derive(Debug, Clone)]
pub struct ErrorExample {
    /// Code that causes the error
    pub before: String,

    /// Corrected code
    pub after: String,

    /// Optional description
    pub description: Option<String>,
}

/// Registry errors
#[derive(Debug, Clone, PartialEq)]
pub enum RegistryError {
    MissingSection(String),
    MissingField(String, String),
    InvalidFormat(String),
    InvalidErrorCode(String),
    InvalidSeverity(String),
    ParseError(String),
}

impl std::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingSection(section) => {
                write!(f, "Missing section: {}", section)
            }
            Self::MissingField(context, field) => {
                write!(f, "Missing field '{}' in {}", field, context)
            }
            Self::InvalidFormat(msg) => {
                write!(f, "Invalid format: {}", msg)
            }
            Self::InvalidErrorCode(code) => {
                write!(f, "Invalid error code: {}", code)
            }
            Self::InvalidSeverity(severity) => {
                write!(f, "Invalid severity: {}", severity)
            }
            Self::ParseError(msg) => {
                write!(f, "Parse error: {}", msg)
            }
        }
    }
}

impl std::error::Error for RegistryError {}

/// Helper functions for working with the registry

/// Load error registry from TOML file
pub fn load_registry_from_file(
    path: &std::path::Path,
) -> Result<ErrorRegistry, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    load_registry_from_string(&content)
}

/// Load error registry from TOML string
pub fn load_registry_from_string(
    content: &str,
) -> Result<ErrorRegistry, Box<dyn std::error::Error>> {
    let config: toml::Value = toml::from_str(content)?;
    let mut registry = ErrorRegistry::new();
    registry.load_from_config(&config)?;
    Ok(registry)
}

/// Create a basic registry with common error definitions
pub fn create_default_registry() -> ErrorRegistry {
    let mut registry = ErrorRegistry::new();

    // Register common errors
    registry.register(ErrorDefinition::new(
        ErrorCode::EXPECTED_TOKEN,
        "expected `{expected}`, found `{found}`".to_string(),
        ErrorCategory::Syntax,
    ));

    registry.register(ErrorDefinition::new(
        ErrorCode::UNEXPECTED_EOF,
        "unexpected end of file".to_string(),
        ErrorCategory::Syntax,
    ));

    registry.register(ErrorDefinition::new(
        ErrorCode::TYPE_MISMATCH,
        "mismatched types: expected `{expected}`, found `{actual}`".to_string(),
        ErrorCategory::Type,
    ));

    registry.register(ErrorDefinition::new(
        ErrorCode::UNDEFINED_VARIABLE,
        "cannot find value `{identifier}` in this scope".to_string(),
        ErrorCategory::Semantic,
    ));

    registry
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_registry() {
        let mut registry = ErrorRegistry::new();

        let error_def = ErrorDefinition::new(
            ErrorCode::EXPECTED_TOKEN,
            "expected token".to_string(),
            ErrorCategory::Syntax,
        );

        registry.register(error_def);

        let retrieved = registry.get(ErrorCode::EXPECTED_TOKEN).unwrap();
        assert_eq!(retrieved.code, ErrorCode::EXPECTED_TOKEN);
        assert_eq!(retrieved.title, "expected token");
        assert_eq!(retrieved.category, ErrorCategory::Syntax);
    }

    #[test]
    fn test_error_code_parsing() {
        let registry = ErrorRegistry::new();

        assert_eq!(
            registry.parse_error_code("E0001").unwrap(),
            ErrorCode::new(1)
        );
        assert_eq!(
            registry.parse_error_code("E1000").unwrap(),
            ErrorCode::new(1000)
        );

        assert!(registry.parse_error_code("1000").is_err());
        assert!(registry.parse_error_code("EINVALID").is_err());
    }

    #[test]
    fn test_category_parsing() {
        let registry = ErrorRegistry::new();

        assert_eq!(
            registry.parse_category("syntax").unwrap(),
            ErrorCategory::Syntax
        );
        assert_eq!(
            registry.parse_category("type").unwrap(),
            ErrorCategory::Type
        );
        assert_eq!(
            registry.parse_category("custom").unwrap(),
            ErrorCategory::Custom("custom".to_string())
        );
    }

    #[test]
    fn test_message_rendering() {
        let error_def = ErrorDefinition::new(
            ErrorCode::EXPECTED_TOKEN,
            "expected `{expected}`, found `{found}`".to_string(),
            ErrorCategory::Syntax,
        );

        let mut context = HashMap::new();
        context.insert("expected".to_string(), ";".to_string());
        context.insert("found".to_string(), "}".to_string());

        let rendered = error_def.render_message(&context);
        assert_eq!(rendered, "expected `;`, found `}`");
    }

    #[test]
    fn test_toml_config_loading() {
        let config_str = r#"
        [errors.E0001]
        title = "expected `{expected}`, found `{found}`"
        category = "syntax"
        description = "The parser expected a specific token"
        
        [[errors.E0001.suggestions]]
        condition = "missing_semicolon"
        text = "Did you forget a semicolon?"
        fix = "Add `;` at the end"
        "#;

        let config: toml::Value = toml::from_str(config_str).unwrap();
        let mut registry = ErrorRegistry::new();
        registry.load_from_config(&config).unwrap();

        let error_def = registry.get(ErrorCode::new(1)).unwrap();
        assert_eq!(error_def.title, "expected `{expected}`, found `{found}`");
        assert_eq!(error_def.category, ErrorCategory::Syntax);
        assert_eq!(error_def.suggestions.len(), 1);
        assert_eq!(error_def.suggestions[0].text, "Did you forget a semicolon?");
    }

    #[test]
    fn test_default_registry() {
        let registry = create_default_registry();

        assert!(registry.get(ErrorCode::EXPECTED_TOKEN).is_some());
        assert!(registry.get(ErrorCode::TYPE_MISMATCH).is_some());
        assert!(registry.get(ErrorCode::UNDEFINED_VARIABLE).is_some());
    }
}
