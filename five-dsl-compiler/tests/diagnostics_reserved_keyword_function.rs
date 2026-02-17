//! Regression test for Issue 5: Parser Diagnostic Error - pub init(...)
//!
//! Tests that:
//! 1. Reserved keywords like 'init' cannot be used as function names
//! 2. Error message explicitly reports keyword reservation
//! 3. The reserved-keyword diagnostic is distinguishable from generic parse failures
//! 4. This affects all reserved keywords (fn, let, if, pub, init, etc.)
//! 5. Some keywords like 'account' work in parameter contexts but not function names

use five_dsl_compiler::compiler::DslCompiler;

#[test]
fn test_init_keyword_as_function_name_fails() {
    // 'init' is a reserved keyword and cannot be used as function name
    let dsl = r#"
pub init() {
    // ...
}

pub main() {
}
"#;

    let result = DslCompiler::compile_dsl(dsl);

    // Should fail - 'init' is reserved
    assert!(result.is_err(), "'init' is a reserved keyword");

    if let Err(e) = result {
        let msg = format!("{:?}", e);
        assert!(
            msg.contains("reserved keyword"),
            "expected reserved-keyword diagnostic, got: {msg}"
        );
        assert!(
            msg.contains("'init'"),
            "expected offending keyword in diagnostic, got: {msg}"
        );
    }
}

#[test]
fn test_fn_keyword_as_function_name_fails() {
    // 'fn' is also a reserved keyword
    let dsl = r#"
pub fn() {
    // ...
}

pub main() {
}
"#;

    let result = DslCompiler::compile_dsl(dsl);

    // Should fail - 'fn' is reserved
    assert!(result.is_err(), "'fn' is a reserved keyword");
}

#[test]
fn test_let_keyword_as_function_name_fails() {
    // 'let' is also a reserved keyword
    let dsl = r#"
pub let() {
    // ...
}

pub main() {
}
"#;

    let result = DslCompiler::compile_dsl(dsl);

    // Should fail - 'let' is reserved
    assert!(result.is_err(), "'let' is a reserved keyword");
}

#[test]
fn test_if_keyword_as_function_name_fails() {
    // 'if' is also a reserved keyword
    let dsl = r#"
pub if() {
    // ...
}

pub main() {
}
"#;

    let result = DslCompiler::compile_dsl(dsl);

    // Should fail - 'if' is reserved
    assert!(result.is_err(), "'if' is a reserved keyword");
}

#[test]
fn test_return_keyword_as_function_name_fails() {
    // 'return' is also a reserved keyword
    let dsl = r#"
pub return() {
    // ...
}

pub main() {
}
"#;

    let result = DslCompiler::compile_dsl(dsl);

    // Should fail - 'return' is reserved
    assert!(result.is_err(), "'return' is a reserved keyword");
}

#[test]
fn test_valid_function_name_works() {
    // Non-reserved identifiers should work fine
    let dsl = r#"
pub initialize() {
    // ...
}

pub main() {
}
"#;

    let result = DslCompiler::compile_dsl(dsl);

    // Should succeed - 'initialize' is a valid identifier
    assert!(
        result.is_ok(),
        "Non-reserved identifiers should work as function names"
    );
}

#[test]
fn test_account_keyword_in_parameter_works() {
    // Interesting case: 'account' keyword works in parameter context
    let dsl = r#"
pub transfer(from: account @mut, to: account @mut) {
    // 'account' can be used as a parameter type
}

pub main() {
}
"#;

    let result = DslCompiler::compile_dsl(dsl);

    // Should succeed - 'account' is allowed in parameter context
    assert!(
        result.is_ok(),
        "'account' keyword should work in parameter types"
    );
}

#[test]
fn test_account_keyword_as_function_name_fails() {
    // But 'account' still cannot be used as function name
    let dsl = r#"
pub account() {
    // ...
}

pub main() {
}
"#;

    let result = DslCompiler::compile_dsl(dsl);

    // Should fail - 'account' cannot be function name
    assert!(
        result.is_err(),
        "'account' keyword should not work as function name"
    );
}

#[test]
fn test_error_message_quality() {
    // Reserved keyword failures should produce targeted diagnostics.
    let dsl = r#"
pub init() {
}

pub main() {
}
"#;

    let result = DslCompiler::compile_dsl(dsl);

    let err = result.expect_err("reserved keyword must fail");
    let msg = format!("{:?}", err);
    assert!(msg.contains("reserved keyword"), "diagnostic should mention reserved keyword: {msg}");
    assert!(msg.contains("'init'"), "diagnostic should include offending token: {msg}");
}

#[test]
fn test_parser_error_applies_to_all_keywords() {
    // Reserved-keyword diagnostics should be specific, not generic identifier failures.
    let dsl = r#"
pub pub() { }
pub main() { }
"#;

    let result = DslCompiler::compile_dsl(dsl);

    assert!(result.is_err(), "All keywords should fail");
    let msg = format!("{:?}", result.unwrap_err());
    assert!(
        msg.contains("reserved keyword"),
        "all reserved keywords should report reserved-keyword diagnostic: {msg}"
    );
}

#[test]
fn test_identifier_error_message_different_contexts() {
    // The error "expected identifier" appears in many contexts
    // It's used for missing tokens, not just keyword conflicts
    // This makes the error message less useful
    let dsl = r#"
pub func( {  // Missing identifier, then missing tokens
}

pub main() {
}
"#;

    let result = DslCompiler::compile_dsl(dsl);

    // This also produces parse errors about expecting identifier
    // But for a different reason (token sequence)
    assert!(result.is_err());
    eprintln!("Generic 'expected identifier' error masks multiple problems");
}

#[test]
fn test_workaround_rename_with_suffix() {
    // Current workaround: rename reserved keywords by adding suffix
    let dsl = r#"
pub init_function() {
    // Renamed from 'init' to 'init_function'
}

pub fn_implementation() {
    // Renamed from 'fn' to 'fn_implementation'
}

pub main() {
}
"#;

    let result = DslCompiler::compile_dsl(dsl);

    // Should succeed - simple renaming is a valid workaround
    assert!(
        result.is_ok(),
        "Renaming with suffix is the current workaround"
    );
}
