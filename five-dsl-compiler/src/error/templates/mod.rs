//! Error message templates
//!
//! This module provides reusable error message templates that can be used
//! to generate consistent, helpful error messages across the compiler.

pub mod codegen_errors;
pub mod parse_errors;
pub mod type_errors;

use crate::error::types::{CompilerError, ErrorContext};
use std::collections::HashMap;

/// Trait for error templates
///
/// Templates define how errors are presented to users, including message
/// formatting, context extraction, and suggestion generation.
pub trait ErrorTemplate {
    /// Render the error message with the given context
    fn render(&self, context: &ErrorContext) -> String;

    /// Get template placeholders that can be filled
    fn get_placeholders(&self) -> Vec<String>;

    /// Get the template name
    fn name(&self) -> &'static str;

    /// Check if this template applies to the given error
    fn applies_to(&self, error: &CompilerError) -> bool;
}

/// Basic template that uses simple string replacement
pub struct SimpleTemplate {
    name: &'static str,
    template: String,
    placeholders: Vec<String>,
}

impl SimpleTemplate {
    /// Create a new simple template
    pub fn new(name: &'static str, template: String) -> Self {
        let placeholders = extract_placeholders(&template);
        Self {
            name,
            template,
            placeholders,
        }
    }
}

impl ErrorTemplate for SimpleTemplate {
    fn render(&self, context: &ErrorContext) -> String {
        let mut rendered = self.template.clone();

        // Replace common placeholders
        if let Some(expected) = context.expected.first() {
            rendered = rendered.replace("{expected}", expected);
        }
        if let Some(found) = &context.found {
            rendered = rendered.replace("{found}", found);
        }
        if let Some(identifier) = &context.identifier {
            rendered = rendered.replace("{identifier}", identifier);
        }
        if let Some(expected_type) = &context.expected_type {
            rendered = rendered.replace("{expected_type}", expected_type);
        }
        if let Some(actual_type) = &context.actual_type {
            rendered = rendered.replace("{actual_type}", actual_type);
        }

        // Replace custom data placeholders
        for (key, value) in &context.data {
            let placeholder = format!("{{{}}}", key);
            rendered = rendered.replace(&placeholder, value);
        }

        rendered
    }

    fn get_placeholders(&self) -> Vec<String> {
        self.placeholders.clone()
    }

    fn name(&self) -> &'static str {
        self.name
    }

    fn applies_to(&self, _error: &CompilerError) -> bool {
        true // Simple template applies to all errors
    }
}

/// Advanced template that supports conditional rendering
pub struct ConditionalTemplate {
    name: &'static str,
    conditions: Vec<TemplateCondition>,
    default_template: String,
}

impl ConditionalTemplate {
    /// Create a new conditional template
    pub fn new(name: &'static str, default_template: String) -> Self {
        Self {
            name,
            conditions: Vec::new(),
            default_template,
        }
    }

    /// Add a conditional template
    pub fn add_condition(mut self, condition: TemplateCondition) -> Self {
        self.conditions.push(condition);
        self
    }
}

impl ErrorTemplate for ConditionalTemplate {
    fn render(&self, context: &ErrorContext) -> String {
        // Check conditions in order
        for condition in &self.conditions {
            if condition.matches(context) {
                let template = SimpleTemplate::new(self.name, condition.template.clone());
                return template.render(context);
            }
        }

        // Use default template
        let template = SimpleTemplate::new(self.name, self.default_template.clone());
        template.render(context)
    }

    fn get_placeholders(&self) -> Vec<String> {
        let mut placeholders = extract_placeholders(&self.default_template);

        for condition in &self.conditions {
            placeholders.extend(extract_placeholders(&condition.template));
        }

        placeholders.sort();
        placeholders.dedup();
        placeholders
    }

    fn name(&self) -> &'static str {
        self.name
    }

    fn applies_to(&self, _error: &CompilerError) -> bool {
        true
    }
}

/// A condition for conditional templates
pub struct TemplateCondition {
    /// The template to use if this condition matches
    pub template: String,

    /// The condition function
    pub condition: Box<dyn Fn(&ErrorContext) -> bool + Send + Sync>,
}

impl TemplateCondition {
    /// Create a new template condition
    pub fn new<F>(template: String, condition: F) -> Self
    where
        F: Fn(&ErrorContext) -> bool + Send + Sync + 'static,
    {
        Self {
            template,
            condition: Box::new(condition),
        }
    }

    /// Check if this condition matches the given context
    pub fn matches(&self, context: &ErrorContext) -> bool {
        (self.condition)(context)
    }
}

/// Template manager for organizing and selecting templates
pub struct TemplateManager {
    templates: HashMap<String, Box<dyn ErrorTemplate + Send + Sync>>,
}

impl TemplateManager {
    /// Create a new template manager
    pub fn new() -> Self {
        let mut manager = Self {
            templates: HashMap::new(),
        };

        // Register default templates
        manager.register_default_templates();
        manager
    }

    /// Register a template
    pub fn register<T>(&mut self, template: T)
    where
        T: ErrorTemplate + Send + Sync + 'static,
    {
        let name = template.name().to_string();
        self.templates.insert(name, Box::new(template));
    }

    /// Get a template by name
    pub fn get(&self, name: &str) -> Option<&(dyn ErrorTemplate + Send + Sync)> {
        self.templates.get(name).map(|t| t.as_ref())
    }

    /// Find the best template for an error
    pub fn find_template(
        &self,
        error: &CompilerError,
    ) -> Option<&(dyn ErrorTemplate + Send + Sync)> {
        // Try to find a specific template for this error
        for template in self.templates.values() {
            if template.applies_to(error) {
                return Some(template.as_ref());
            }
        }

        // Fall back to default template
        self.get("default")
    }

    /// Render an error using the best available template
    pub fn render_error(&self, error: &CompilerError) -> String {
        if let Some(template) = self.find_template(error) {
            template.render(&error.context)
        } else {
            // Fallback rendering
            format!("{}: {}", error.code, error.message)
        }
    }

    /// Register default templates
    fn register_default_templates(&mut self) {
        // Default template
        self.register(SimpleTemplate::new("default", "{message}".to_string()));

        // Expected token template
        let expected_token_template = ConditionalTemplate::new(
            "expected_token",
            "expected `{expected}`, found `{found}`".to_string(),
        )
        .add_condition(TemplateCondition::new(
            "did you forget a semicolon?".to_string(),
            |ctx| ctx.expected.contains(&";".to_string()),
        ))
        .add_condition(TemplateCondition::new(
            "did you forget a closing brace `}`?".to_string(),
            |ctx| ctx.expected.contains(&"}".to_string()),
        ));

        self.register(expected_token_template);

        // Type mismatch template
        let type_mismatch_template = ConditionalTemplate::new(
            "type_mismatch",
            "mismatched types: expected `{expected_type}`, found `{actual_type}`".to_string(),
        )
        .add_condition(TemplateCondition::new(
            "mismatched types: expected `{expected_type}`, found `{actual_type}`\n\nhelp: try converting with `.into()` or cast with `as {expected_type}`".to_string(),
            |ctx| {
                matches!(
                    (ctx.expected_type.as_deref(), ctx.actual_type.as_deref()),
                    (Some("u64"), Some("u32")) | (Some("u64"), Some("u8"))
                )
            },
        ));

        self.register(type_mismatch_template);

        // Undefined variable template
        self.register(SimpleTemplate::new(
            "undefined_variable",
            "cannot find value `{identifier}` in this scope\n\nhelp: did you forget to declare it with `let`?".to_string(),
        ));
    }
}

impl Default for TemplateManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract placeholders from a template string
fn extract_placeholders(template: &str) -> Vec<String> {
    let mut placeholders = Vec::new();
    let mut chars = template.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '{' {
            let mut placeholder = String::new();

            // Read until closing brace
            while let Some(&next_ch) = chars.peek() {
                if next_ch == '}' {
                    chars.next(); // consume the '}'
                    break;
                }
                placeholder.push(chars.next().unwrap());
            }

            if !placeholder.is_empty() {
                placeholders.push(placeholder);
            }
        }
    }

    placeholders.sort();
    placeholders.dedup();
    placeholders
}

/// Create common error templates
pub fn create_common_templates() -> TemplateManager {
    let mut manager = TemplateManager::new();

    // Add parse error templates
    parse_errors::register_templates(&mut manager);

    // Add type error templates
    type_errors::register_templates(&mut manager);

    // Add codegen error templates
    codegen_errors::register_templates(&mut manager);

    manager
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::types::{ErrorBuilder, ErrorCode};

    #[test]
    fn test_simple_template() {
        let template =
            SimpleTemplate::new("test", "expected `{expected}`, found `{found}`".to_string());

        let context = ErrorContext::new()
            .with_expected(vec![";".to_string()])
            .with_found("}".to_string());

        let rendered = template.render(&context);
        assert_eq!(rendered, "expected `;`, found `}`");
    }

    #[test]
    fn test_placeholder_extraction() {
        let placeholders = extract_placeholders("expected `{expected}`, found `{found}`");
        assert_eq!(placeholders, vec!["expected", "found"]);

        let placeholders = extract_placeholders("no placeholders here");
        assert!(placeholders.is_empty());

        let placeholders = extract_placeholders("{duplicate} and {duplicate}");
        assert_eq!(placeholders, vec!["duplicate"]);
    }

    #[test]
    fn test_conditional_template() {
        let template = ConditionalTemplate::new("conditional_test", "default message".to_string())
            .add_condition(TemplateCondition::new(
                "semicolon message".to_string(),
                |ctx| ctx.expected.contains(&";".to_string()),
            ));

        // Test condition match
        let context_semicolon = ErrorContext::new().with_expected(vec![";".to_string()]);
        let rendered = template.render(&context_semicolon);
        assert_eq!(rendered, "semicolon message");

        // Test default
        let context_default = ErrorContext::new().with_expected(vec!["}".to_string()]);
        let rendered = template.render(&context_default);
        assert_eq!(rendered, "default message");
    }

    #[test]
    fn test_template_manager() {
        let manager = TemplateManager::new();

        let template = manager.get("default").unwrap();
        assert_eq!(template.name(), "default");

        let error = ErrorBuilder::new(ErrorCode::EXPECTED_TOKEN, "test error".to_string()).build();

        let rendered = manager.render_error(&error);
        assert!(!rendered.is_empty());
    }
}
