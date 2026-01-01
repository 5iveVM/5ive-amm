//! Error suggestion system
//!
//! This module provides an intelligent suggestion engine that can generate
//! helpful fix suggestions for compiler errors, similar to Rust's compiler.

use crate::error::types::{CompilerError, ErrorCode, SourceLocation};
use std::collections::HashMap;

/// A suggestion for fixing a compiler error
#[derive(Debug, Clone)]
pub struct Suggestion {
    /// The suggestion message
    pub message: String,

    /// Detailed explanation of the suggestion
    pub explanation: Option<String>,

    /// Confidence level of this suggestion (0.0 to 1.0)
    pub confidence: f32,

    /// Type of suggestion
    pub suggestion_type: SuggestionType,

    /// Code fix (if applicable)
    pub code_fix: Option<CodeFix>,

    /// Applicable source location
    pub location: Option<SourceLocation>,
}

impl Suggestion {
    /// Create a new suggestion
    pub fn new(message: String, confidence: f32) -> Self {
        Self {
            message,
            explanation: None,
            confidence,
            suggestion_type: SuggestionType::General,
            code_fix: None,
            location: None,
        }
    }

    /// Add an explanation to the suggestion
    pub fn with_explanation(mut self, explanation: String) -> Self {
        self.explanation = Some(explanation);
        self
    }

    /// Set the suggestion type
    pub fn with_type(mut self, suggestion_type: SuggestionType) -> Self {
        self.suggestion_type = suggestion_type;
        self
    }

    /// Add a code fix
    pub fn with_code_fix(mut self, code_fix: CodeFix) -> Self {
        self.code_fix = Some(code_fix);
        self
    }

    /// Add a source location
    pub fn with_location(mut self, location: SourceLocation) -> Self {
        self.location = Some(location);
        self
    }

    /// Check if this is a high-confidence suggestion
    pub fn is_high_confidence(&self) -> bool {
        self.confidence >= 0.8
    }

    /// Check if this suggestion has a code fix
    pub fn has_code_fix(&self) -> bool {
        self.code_fix.is_some()
    }
}

/// Types of suggestions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SuggestionType {
    /// General suggestion
    General,

    /// Syntax fix suggestion
    SyntaxFix,

    /// Type fix suggestion
    TypeFix,

    /// Missing element suggestion
    Missing,

    /// Replacement suggestion
    Replacement,

    /// Addition suggestion
    Addition,

    /// Removal suggestion
    Removal,
}

/// A code fix that can be applied to resolve an error
#[derive(Debug, Clone)]
pub struct CodeFix {
    /// Description of what the fix does
    pub description: String,

    /// The replacement text
    pub replacement: String,

    /// Location to apply the fix
    pub location: SourceLocation,

    /// Type of fix operation
    pub fix_type: FixType,
}

impl CodeFix {
    /// Create a replacement fix
    pub fn replacement(description: String, replacement: String, location: SourceLocation) -> Self {
        Self {
            description,
            replacement,
            location,
            fix_type: FixType::Replace,
        }
    }

    /// Create an insertion fix
    pub fn insertion(description: String, text: String, location: SourceLocation) -> Self {
        Self {
            description,
            replacement: text,
            location,
            fix_type: FixType::Insert,
        }
    }

    /// Create a deletion fix
    pub fn deletion(description: String, location: SourceLocation) -> Self {
        Self {
            description,
            replacement: String::new(),
            location,
            fix_type: FixType::Delete,
        }
    }
}

/// Types of code fix operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FixType {
    Replace,
    Insert,
    Delete,
}

/// Trait for suggestion rules
///
/// Suggestion rules analyze errors and generate appropriate suggestions.
pub trait SuggestionRule: Send + Sync {
    /// Check if this rule applies to the given error
    fn applies_to(&self, error: &CompilerError) -> bool;

    /// Generate suggestions for the error
    fn generate_suggestions(&self, error: &CompilerError) -> Vec<Suggestion>;

    /// Get the rule name for debugging
    fn name(&self) -> &'static str;

    /// Get the rule priority (higher = more important)
    fn priority(&self) -> u32 {
        100
    }
}

/// Main suggestion engine
///
/// Coordinates multiple suggestion rules to generate helpful error suggestions.
pub struct SuggestionEngine {
    /// Registered suggestion rules
    rules: Vec<Box<dyn SuggestionRule>>,

    /// Rule-specific configuration
    _config: HashMap<String, SuggestionConfig>,

    /// Maximum number of suggestions to generate per error
    max_suggestions: usize,
}

impl SuggestionEngine {
    /// Create a new suggestion engine
    pub fn new() -> Self {
        let mut engine = Self {
            rules: Vec::new(),
            _config: HashMap::new(),
            max_suggestions: 5,
        };

        // Register default rules
        engine.register_default_rules();
        engine
    }

    /// Set the maximum number of suggestions per error
    pub fn set_max_suggestions(&mut self, max: usize) {
        self.max_suggestions = max;
    }

    /// Add a suggestion rule
    pub fn add_rule(&mut self, rule: Box<dyn SuggestionRule>) {
        self.rules.push(rule);
    }

    /// Generate suggestions for an error
    pub fn generate_suggestions(&self, error: &CompilerError) -> Vec<Suggestion> {
        let mut suggestions = Vec::new();

        // Collect suggestions from all applicable rules
        for rule in &self.rules {
            if rule.applies_to(error) {
                let rule_suggestions = rule.generate_suggestions(error);
                suggestions.extend(rule_suggestions);
            }
        }

        // Sort by confidence and priority
        suggestions.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Limit to max suggestions
        suggestions.truncate(self.max_suggestions);

        suggestions
    }

    /// Register default suggestion rules
    fn register_default_rules(&mut self) {
        self.add_rule(Box::new(MissingSemicolonRule));
        self.add_rule(Box::new(MissingBraceRule));
        self.add_rule(Box::new(UnexpectedTokenRule));
        self.add_rule(Box::new(TypeMismatchRule));
        self.add_rule(Box::new(UndefinedVariableRule));
        self.add_rule(Box::new(MissingReturnTypeRule));
        self.add_rule(Box::new(IncorrectParametersRule));
    }
}

impl Default for SuggestionEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration for suggestion rules
#[derive(Debug, Clone)]
pub struct SuggestionConfig {
    pub enabled: bool,
    pub confidence_threshold: f32,
    pub max_suggestions: usize,
}

impl Default for SuggestionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            confidence_threshold: 0.3,
            max_suggestions: 3,
        }
    }
}

// Default suggestion rules

/// Rule for suggesting missing semicolons
pub struct MissingSemicolonRule;

impl SuggestionRule for MissingSemicolonRule {
    fn applies_to(&self, error: &CompilerError) -> bool {
        error.code == ErrorCode::EXPECTED_TOKEN && error.context.expected.contains(&";".to_string())
    }

    fn generate_suggestions(&self, error: &CompilerError) -> Vec<Suggestion> {
        let mut suggestions = Vec::new();

        if let Some(_found) = &error.context.found {
            let suggestion = Suggestion::new("Did you forget a semicolon?".to_string(), 0.9)
                .with_explanation("Five DSL requires semicolons after statements".to_string())
                .with_type(SuggestionType::Missing);

            // Try to provide a code fix if we have location information
            if let Some(location) = &error.location {
                let code_fix = CodeFix::insertion(
                    "Add semicolon".to_string(),
                    ";".to_string(),
                    location.clone(),
                );
                suggestions.push(suggestion.with_code_fix(code_fix));
            } else {
                suggestions.push(suggestion);
            }
        }

        suggestions
    }

    fn name(&self) -> &'static str {
        "missing_semicolon"
    }

    fn priority(&self) -> u32 {
        200
    }
}

/// Rule for suggesting missing braces
pub struct MissingBraceRule;

impl SuggestionRule for MissingBraceRule {
    fn applies_to(&self, error: &CompilerError) -> bool {
        error.code == ErrorCode::EXPECTED_TOKEN
            && (error.context.expected.contains(&"{".to_string())
                || error.context.expected.contains(&"}".to_string()))
    }

    fn generate_suggestions(&self, error: &CompilerError) -> Vec<Suggestion> {
        let mut suggestions = Vec::new();

        if error.context.expected.contains(&"{".to_string()) {
            suggestions.push(
                Suggestion::new("Did you forget an opening brace `{`?".to_string(), 0.8)
                    .with_explanation("Functions and blocks require opening braces".to_string())
                    .with_type(SuggestionType::Missing),
            );
        }

        if error.context.expected.contains(&"}".to_string()) {
            suggestions.push(
                Suggestion::new("Did you forget a closing brace `}`?".to_string(), 0.8)
                    .with_explanation("All opened braces must be closed".to_string())
                    .with_type(SuggestionType::Missing),
            );
        }

        suggestions
    }

    fn name(&self) -> &'static str {
        "missing_brace"
    }
}

/// Rule for unexpected token suggestions
pub struct UnexpectedTokenRule;

impl SuggestionRule for UnexpectedTokenRule {
    fn applies_to(&self, error: &CompilerError) -> bool {
        error.code == ErrorCode::UNEXPECTED_TOKEN
    }

    fn generate_suggestions(&self, error: &CompilerError) -> Vec<Suggestion> {
        let mut suggestions = Vec::new();

        if let Some(found) = &error.context.found {
            // Common typos and mistakes
            match found.as_str() {
                "=" => {
                    suggestions.push(
                        Suggestion::new("Did you mean `==` for comparison?".to_string(), 0.7)
                            .with_type(SuggestionType::Replacement),
                    );
                }
                "<<" | ">>" => {
                    suggestions.push(
                        Suggestion::new(
                            "Bitwise operators are not supported in Five DSL".to_string(),
                            0.9,
                        )
                        .with_explanation(
                            "Use arithmetic or logical operators instead".to_string(),
                        ),
                    );
                }
                _ => {
                    suggestions.push(
                        Suggestion::new(format!("Unexpected `{}` token", found), 0.5)
                            .with_explanation("Check the syntax around this location".to_string()),
                    );
                }
            }
        }

        suggestions
    }

    fn name(&self) -> &'static str {
        "unexpected_token"
    }
}

/// Rule for type mismatch suggestions
pub struct TypeMismatchRule;

impl SuggestionRule for TypeMismatchRule {
    fn applies_to(&self, error: &CompilerError) -> bool {
        error.code == ErrorCode::TYPE_MISMATCH
    }

    fn generate_suggestions(&self, error: &CompilerError) -> Vec<Suggestion> {
        let mut suggestions = Vec::new();

        if let (Some(expected), Some(actual)) =
            (&error.context.expected_type, &error.context.actual_type)
        {
            // Suggest common type conversions
            match (expected.as_str(), actual.as_str()) {
                ("u64", "u32") | ("u64", "u8") => {
                    suggestions.push(
                        Suggestion::new(
                            "Try converting to u64 with `.into()` or cast with `as u64`".to_string(),
                            0.8,
                        )
                        .with_type(SuggestionType::TypeFix),
                    );
                }
                ("string", _) => {
                    suggestions.push(
                        Suggestion::new(
                            "Try converting to string with `.to_string()` or string interpolation"
                                .to_string(),
                            0.7,
                        )
                        .with_type(SuggestionType::TypeFix),
                    );
                }
                _ => {
                    suggestions.push(
                        Suggestion::new(
                            format!("Expected type `{}`, but found `{}`", expected, actual),
                            0.6,
                        )
                        .with_explanation(
                            "Check the variable types and function signatures".to_string(),
                        ),
                    );
                }
            }
        }

        suggestions
    }

    fn name(&self) -> &'static str {
        "type_mismatch"
    }
}

/// Rule for undefined variable suggestions
pub struct UndefinedVariableRule;

impl SuggestionRule for UndefinedVariableRule {
    fn applies_to(&self, error: &CompilerError) -> bool {
        error.code == ErrorCode::UNDEFINED_VARIABLE
    }

    fn generate_suggestions(&self, error: &CompilerError) -> Vec<Suggestion> {
        let mut suggestions = Vec::new();

        if let Some(identifier) = &error.context.identifier {
            suggestions.push(
                Suggestion::new(
                    format!("Did you forget to declare variable `{}`?", identifier),
                    0.8,
                )
                .with_explanation("Variables must be declared with `let` before use".to_string())
                .with_type(SuggestionType::Missing),
            );

            // Suggest common typos
            if identifier.len() > 2 {
                suggestions.push(
                    Suggestion::new("Check for typos in the variable name".to_string(), 0.5)
                        .with_type(SuggestionType::General),
                );
            }
        }

        suggestions
    }

    fn name(&self) -> &'static str {
        "undefined_variable"
    }
}

/// Rule for missing return type suggestions
pub struct MissingReturnTypeRule;

impl SuggestionRule for MissingReturnTypeRule {
    fn applies_to(&self, error: &CompilerError) -> bool {
        error.message.contains("return type") && error.code == ErrorCode::INVALID_SYNTAX
    }

    fn generate_suggestions(&self, _error: &CompilerError) -> Vec<Suggestion> {
        vec![
            Suggestion::new("Did you forget to specify a return type?".to_string(), 0.8)
                .with_explanation("Functions that return values need `-> ReturnType`".to_string())
                .with_type(SuggestionType::Missing),
        ]
    }

    fn name(&self) -> &'static str {
        "missing_return_type"
    }
}

/// Rule for incorrect parameter suggestions
pub struct IncorrectParametersRule;

impl SuggestionRule for IncorrectParametersRule {
    fn applies_to(&self, error: &CompilerError) -> bool {
        error.message.contains("parameter") || error.message.contains("argument")
    }

    fn generate_suggestions(&self, _error: &CompilerError) -> Vec<Suggestion> {
        vec![Suggestion::new(
            "Check the function signature and parameter types".to_string(),
            0.7,
        )
        .with_explanation(
            "Ensure parameter names and types match the function definition".to_string(),
        )
        .with_type(SuggestionType::General)]
    }

    fn name(&self) -> &'static str {
        "incorrect_parameters"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::types::{ErrorBuilder, ErrorContext, SourceLocation};

    #[test]
    fn test_suggestion_creation() {
        let suggestion = Suggestion::new("Test suggestion".to_string(), 0.8)
            .with_explanation("Test explanation".to_string())
            .with_type(SuggestionType::SyntaxFix);

        assert_eq!(suggestion.message, "Test suggestion");
        assert_eq!(suggestion.confidence, 0.8);
        assert!(suggestion.is_high_confidence());
        assert_eq!(suggestion.suggestion_type, SuggestionType::SyntaxFix);
    }

    #[test]
    fn test_suggestion_engine() {
        let engine = SuggestionEngine::new();

        let error = ErrorBuilder::new(
            ErrorCode::EXPECTED_TOKEN,
            "expected `;`, found `}`".to_string(),
        )
        .context(
            ErrorContext::new()
                .with_expected(vec![";".to_string()])
                .with_found("}".to_string()),
        )
        .build();

        let suggestions = engine.generate_suggestions(&error);
        assert!(!suggestions.is_empty());

        // Should suggest missing semicolon
        assert!(suggestions.iter().any(|s| s.message.contains("semicolon")));
    }

    #[test]
    fn test_missing_semicolon_rule() {
        let rule = MissingSemicolonRule;

        let error = ErrorBuilder::new(ErrorCode::EXPECTED_TOKEN, "expected `;`".to_string())
            .context(
                ErrorContext::new()
                    .with_expected(vec![";".to_string()])
                    .with_found("}".to_string()),
            )
            .build();

        assert!(rule.applies_to(&error));

        let suggestions = rule.generate_suggestions(&error);
        assert!(!suggestions.is_empty());
        assert!(suggestions[0].message.contains("semicolon"));
        assert!(suggestions[0].confidence > 0.8);
    }

    #[test]
    fn test_type_mismatch_rule() {
        let rule = TypeMismatchRule;

        let error = ErrorBuilder::new(ErrorCode::TYPE_MISMATCH, "type mismatch".to_string())
            .context(ErrorContext::new().with_types("u64".to_string(), "u32".to_string()))
            .build();

        assert!(rule.applies_to(&error));

        let suggestions = rule.generate_suggestions(&error);
        assert!(!suggestions.is_empty());
        assert!(suggestions[0].message.contains("convert"));
    }

    #[test]
    fn test_code_fix() {
        let location = SourceLocation::new(5, 10, 100);
        let fix = CodeFix::replacement("Add semicolon".to_string(), ";".to_string(), location);

        assert_eq!(fix.description, "Add semicolon");
        assert_eq!(fix.replacement, ";");
        assert_eq!(fix.fix_type, FixType::Replace);
    }
}
