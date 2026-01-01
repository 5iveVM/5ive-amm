//! Parse error templates
//!
//! This module contains templates for parsing-related errors,
//! providing helpful context and suggestions for syntax issues.

use crate::error::templates::{
    ConditionalTemplate, SimpleTemplate, TemplateCondition, TemplateManager,
};

/// Register all parse error templates
pub fn register_templates(manager: &mut TemplateManager) {
    register_token_templates(manager);
    register_syntax_templates(manager);
    register_structure_templates(manager);
}

/// Register token-related error templates
fn register_token_templates(manager: &mut TemplateManager) {
    // Expected token template with intelligent suggestions
    let expected_token_template = ConditionalTemplate::new(
        "expected_token",
        "expected `{expected}`, found `{found}`".to_string(),
    )
    .add_condition(TemplateCondition::new(
        "expected `{expected}`, found `{found}`\n\nhelp: did you forget a semicolon after the statement?".to_string(),
        |ctx| ctx.expected.contains(&";".to_string()),
    ))
    .add_condition(TemplateCondition::new(
        "expected `{expected}`, found `{found}`\n\nhelp: did you forget a closing brace `}`?".to_string(),
        |ctx| ctx.expected.contains(&"}".to_string()),
    ))
    .add_condition(TemplateCondition::new(
        "expected `{expected}`, found `{found}`\n\nhelp: did you forget a closing parenthesis `)`?".to_string(),
        |ctx| ctx.expected.contains(&")".to_string()),
    ))
    .add_condition(TemplateCondition::new(
        "expected `{expected}`, found `{found}`\n\nhelp: did you forget to open a block with `{`?".to_string(),
        |ctx| ctx.expected.contains(&"{".to_string()),
    ));

    manager.register(expected_token_template);

    // Unexpected token template
    let unexpected_token_template = ConditionalTemplate::new(
        "unexpected_token",
        "unexpected token `{found}`".to_string(),
    )
    .add_condition(TemplateCondition::new(
        "unexpected token `{found}`\n\nhelp: did you mean `==` for comparison instead of `=`?".to_string(),
        |ctx| ctx.found.as_ref().map(|f| f == "=").unwrap_or(false),
    ))
    .add_condition(TemplateCondition::new(
        "unexpected token `{found}`\n\nnote: Five DSL doesn't support bitwise operators\nhelp: use arithmetic or logical operators instead".to_string(),
        |ctx| {
            ctx.found.as_ref()
                .map(|f| matches!(f.as_str(), "<<" | ">>" | "&" | "|" | "^"))
                .unwrap_or(false)
        },
    ));

    manager.register(unexpected_token_template);

    // EOF template
    manager.register(SimpleTemplate::new(
        "unexpected_eof",
        "unexpected end of file\n\nhelp: this file contains an incomplete construct".to_string(),
    ));
}

/// Register syntax-related error templates
fn register_syntax_templates(manager: &mut TemplateManager) {
    // Invalid syntax template with context-aware suggestions
    let invalid_syntax_template = ConditionalTemplate::new(
        "invalid_syntax",
        "invalid syntax: {message}".to_string(),
    )
    .add_condition(TemplateCondition::new(
        "invalid function signature\n\nhelp: function signatures should be `fn name(param: type) -> return_type`\n\nExample:\n  fn calculate(x: u64, y: u64) -> u64".to_string(),
        |ctx| ctx.data.get("context").map(|c| c.contains("function")).unwrap_or(false),
    ))
    .add_condition(TemplateCondition::new(
        "invalid variable declaration\n\nhelp: variables should be declared with `let name: type = value`\n\nExample:\n  let balance: u64 = 100;".to_string(),
        |ctx| ctx.data.get("context").map(|c| c.contains("variable")).unwrap_or(false),
    ))
    .add_condition(TemplateCondition::new(
        "invalid script structure\n\nhelp: scripts should be defined as:\n  script ScriptName {\n      // fields and functions\n  }".to_string(),
        |ctx| ctx.data.get("context").map(|c| c.contains("script")).unwrap_or(false),
    ));

    manager.register(invalid_syntax_template);

    // Missing elements
    manager.register(SimpleTemplate::new(
        "missing_identifier",
        "expected identifier, found `{found}`\n\nhelp: identifiers must start with a letter or underscore".to_string(),
    ));

    manager.register(SimpleTemplate::new(
        "missing_expression",
        "expected expression, found `{found}`\n\nhelp: this position requires a value or expression".to_string(),
    ));
}

/// Register structure-related error templates
fn register_structure_templates(manager: &mut TemplateManager) {
    // Unmatched delimiters
    let unmatched_delimiter_template = ConditionalTemplate::new(
        "unmatched_delimiter",
        "unmatched delimiter".to_string(),
    )
    .add_condition(TemplateCondition::new(
        "unmatched `{`\n\nhelp: every opening brace `{` must have a corresponding closing brace `}`".to_string(),
        |ctx| ctx.data.get("delimiter").map(|d| d == "{").unwrap_or(false),
    ))
    .add_condition(TemplateCondition::new(
        "unmatched `(`\n\nhelp: every opening parenthesis `(` must have a corresponding closing parenthesis `)`".to_string(),
        |ctx| ctx.data.get("delimiter").map(|d| d == "(").unwrap_or(false),
    ))
    .add_condition(TemplateCondition::new(
        "unmatched `[`\n\nhelp: every opening bracket `[` must have a corresponding closing bracket `]`".to_string(),
        |ctx| ctx.data.get("delimiter").map(|d| d == "[").unwrap_or(false),
    ));

    manager.register(unmatched_delimiter_template);

    // Nested structure errors
    manager.register(SimpleTemplate::new(
        "invalid_nesting",
        "invalid nesting: {message}\n\nhelp: check that all blocks are properly opened and closed"
            .to_string(),
    ));

    // Incomplete constructs
    manager.register(SimpleTemplate::new(
        "incomplete_construct",
        "incomplete {construct}\n\nhelp: this {construct} is missing required elements".to_string(),
    ));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::types::ErrorContext;

    #[test]
    fn test_parse_template_registration() {
        let mut manager = TemplateManager::new();
        register_templates(&mut manager);

        assert!(manager.get("expected_token").is_some());
        assert!(manager.get("unexpected_token").is_some());
        assert!(manager.get("invalid_syntax").is_some());
    }

    #[test]
    fn test_expected_token_template() {
        let mut manager = TemplateManager::new();
        register_templates(&mut manager);

        let template = manager.get("expected_token").unwrap();

        // Test semicolon suggestion
        let context = ErrorContext::new()
            .with_expected(vec![";".to_string()])
            .with_found("}".to_string());

        let rendered = template.render(&context);
        assert!(rendered.contains("semicolon"));
        assert!(rendered.contains("expected `;`, found `}`"));
    }

    #[test]
    fn test_unexpected_token_suggestions() {
        let mut manager = TemplateManager::new();
        register_templates(&mut manager);

        let template = manager.get("unexpected_token").unwrap();

        // Test assignment vs comparison
        let context = ErrorContext::new().with_found("=".to_string());
        let rendered = template.render(&context);
        assert!(rendered.contains("=="));
        assert!(rendered.contains("comparison"));
    }
}
