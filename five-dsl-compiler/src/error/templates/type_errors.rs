//! Type error templates
//!
//! This module contains templates for type-related errors,
//! providing helpful context and conversion suggestions.

use crate::error::templates::{
    ConditionalTemplate, SimpleTemplate, TemplateCondition, TemplateManager,
};

/// Register all type error templates
pub fn register_templates(manager: &mut TemplateManager) {
    register_mismatch_templates(manager);
    register_conversion_templates(manager);
    register_inference_templates(manager);
}

/// Register type mismatch templates
fn register_mismatch_templates(manager: &mut TemplateManager) {
    // Type mismatch with intelligent conversion suggestions
    let type_mismatch_template = ConditionalTemplate::new(
        "type_mismatch",
        "mismatched types: expected `{expected_type}`, found `{actual_type}`".to_string(),
    )
    // Integer promotions
    .add_condition(TemplateCondition::new(
        "mismatched types: expected `{expected_type}`, found `{actual_type}`\n\nhelp: try converting the value:\n  • Use `as {expected_type}` for explicit casting\n  • Use `.into()` if the conversion is safe\n\nExample: `value as {expected_type}` or `value.into()`".to_string(),
        |ctx| {
            matches!(
                (ctx.expected_type.as_deref(), ctx.actual_type.as_deref()),
                (Some("u64"), Some("u32")) | (Some("u64"), Some("u8")) | 
                (Some("u32"), Some("u8")) | (Some("i64"), Some("i32"))
            )
        },
    ))
    // Boolean context
    .add_condition(TemplateCondition::new(
        "mismatched types: expected `{expected_type}`, found `{actual_type}`\n\nhelp: this expression should evaluate to a boolean\n\nNote: Five DSL requires explicit boolean expressions\nTry: `value != 0` or `value == some_value`".to_string(),
        |ctx| {
            ctx.expected_type.as_deref() == Some("bool") && 
            matches!(ctx.actual_type.as_deref(), Some("u64") | Some("u32") | Some("u8"))
        },
    ))
    // String conversions
    .add_condition(TemplateCondition::new(
        "mismatched types: expected `{expected_type}`, found `{actual_type}`\n\nhelp: try converting to string:\n  • Use `.to_string()` method\n  • Use string formatting: `format!(\"{{}}\", value)`".to_string(),
        |ctx| {
            ctx.expected_type.as_deref() == Some("string") &&
            matches!(ctx.actual_type.as_deref(), Some("u64") | Some("u32") | Some("u8"))
        },
    ))
    // Array/slice types
    .add_condition(TemplateCondition::new(
        "mismatched types: expected `{expected_type}`, found `{actual_type}`\n\nhelp: array types must match exactly in Five DSL\n\nNote: consider using slices `&[T]` for flexibility".to_string(),
        |ctx| {
            ctx.expected_type.as_ref().map(|t| t.contains("[")).unwrap_or(false) ||
            ctx.actual_type.as_ref().map(|t| t.contains("[")).unwrap_or(false)
        },
    ));

    manager.register(type_mismatch_template);

    // Function signature mismatches
    let function_signature_template = ConditionalTemplate::new(
        "function_signature_mismatch",
        "function signature mismatch".to_string(),
    )
    .add_condition(TemplateCondition::new(
        "function signature mismatch: wrong parameter count\n\nExpected: {expected_count} parameters\nFound: {actual_count} parameters\n\nhelp: check the function definition and call site".to_string(),
        |ctx| ctx.data.contains_key("expected_count") && ctx.data.contains_key("actual_count"),
    ))
    .add_condition(TemplateCondition::new(
        "function signature mismatch: parameter type error\n\nParameter `{param_name}`: expected `{expected_type}`, found `{actual_type}`\n\nhelp: ensure parameter types match the function definition".to_string(),
        |ctx| ctx.data.contains_key("param_name"),
    ));

    manager.register(function_signature_template);
}

/// Register type conversion templates
fn register_conversion_templates(manager: &mut TemplateManager) {
    // Invalid conversion
    manager.register(SimpleTemplate::new(
        "invalid_conversion",
        "cannot convert `{from_type}` to `{to_type}`\n\nhelp: these types are not compatible for automatic conversion\nNote: you may need explicit casting or a conversion function".to_string(),
    ));

    // Lossy conversion warning
    manager.register(SimpleTemplate::new(
        "lossy_conversion",
        "potentially lossy conversion from `{from_type}` to `{to_type}`\n\nhelp: use explicit casting `as {to_type}` if this is intentional\nNote: this conversion may lose precision or overflow".to_string(),
    ));

    // Overflow in literal
    manager.register(SimpleTemplate::new(
        "literal_overflow",
        "literal `{literal}` does not fit in type `{target_type}`\n\nhelp: try using a larger type or check the value range\n\nType ranges:\n  • u8: 0 to 255\n  • u32: 0 to 4,294,967,295\n  • u64: 0 to 18,446,744,073,709,551,615".to_string(),
    ));
}

/// Register type inference templates  
fn register_inference_templates(manager: &mut TemplateManager) {
    // Cannot infer type
    let cannot_infer_template = ConditionalTemplate::new(
        "cannot_infer_type",
        "cannot infer type for `{identifier}`".to_string(),
    )
    .add_condition(TemplateCondition::new(
        "cannot infer type for `{identifier}`\n\nhelp: add an explicit type annotation:\n  let {identifier}: SomeType = value;\n\nOR provide more context so the type can be inferred".to_string(),
        |ctx| ctx.identifier.is_some(),
    ))
    .add_condition(TemplateCondition::new(
        "cannot infer type for expression\n\nhelp: the type of this expression is ambiguous\nTry adding type annotations or providing more context".to_string(),
        |ctx| ctx.identifier.is_none(),
    ));

    manager.register(cannot_infer_template);

    // Ambiguous type
    manager.register(SimpleTemplate::new(
        "ambiguous_type",
        "type annotations needed: ambiguous type for `{identifier}`\n\nhelp: multiple types are possible here\nBe more specific with type annotations or context".to_string(),
    ));

    // Recursive type
    manager.register(SimpleTemplate::new(
        "recursive_type",
        "recursive type `{type_name}` has infinite size\n\nhelp: recursive types must be behind a pointer or reference\nTry using `Box<T>` or similar indirection".to_string(),
    ));

    // Missing type annotation
    manager.register(SimpleTemplate::new(
        "missing_type_annotation",
        "missing type annotation for `{identifier}`\n\nhelp: Five DSL requires explicit types in this context\n\nExample:\n  let {identifier}: u64 = value;".to_string(),
    ));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::types::ErrorContext;

    #[test]
    fn test_type_template_registration() {
        let mut manager = TemplateManager::new();
        register_templates(&mut manager);

        assert!(manager.get("type_mismatch").is_some());
        assert!(manager.get("cannot_infer_type").is_some());
        assert!(manager.get("invalid_conversion").is_some());
    }

    #[test]
    fn test_type_mismatch_suggestions() {
        let mut manager = TemplateManager::new();
        register_templates(&mut manager);

        let template = manager.get("type_mismatch").unwrap();

        // Test integer conversion suggestion
        let context = ErrorContext::new().with_types("u64".to_string(), "u32".to_string());

        let rendered = template.render(&context);
        assert!(rendered.contains("as u64"));
        assert!(rendered.contains(".into()"));
    }

    #[test]
    fn test_boolean_context_suggestion() {
        let mut manager = TemplateManager::new();
        register_templates(&mut manager);

        let template = manager.get("type_mismatch").unwrap();

        // Test boolean context
        let context = ErrorContext::new().with_types("bool".to_string(), "u64".to_string());

        let rendered = template.render(&context);
        assert!(rendered.contains("boolean"));
        assert!(rendered.contains("!= 0"));
    }

    #[test]
    fn test_string_conversion_suggestion() {
        let mut manager = TemplateManager::new();
        register_templates(&mut manager);

        let template = manager.get("type_mismatch").unwrap();

        // Test string conversion
        let context = ErrorContext::new().with_types("string".to_string(), "u64".to_string());

        let rendered = template.render(&context);
        assert!(rendered.contains(".to_string()"));
        assert!(rendered.contains("format!"));
    }

    #[test]
    fn test_type_inference_template() {
        let mut manager = TemplateManager::new();
        register_templates(&mut manager);

        let template = manager.get("cannot_infer_type").unwrap();

        // Test with identifier
        let context = ErrorContext::new().with_identifier("value".to_string());

        let rendered = template.render(&context);
        assert!(rendered.contains("let value: SomeType"));
        assert!(rendered.contains("type annotation"));
    }
}
