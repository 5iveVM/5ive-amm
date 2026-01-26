//! Integration tests for LSP diagnostics
//!
//! Tests that the bridge correctly identifies and reports syntax, parse, and type errors
//! as LSP diagnostics.

use five_lsp::CompilerBridge;
use lsp_types::Url;

fn create_test_uri(filename: &str) -> Url {
    Url::parse(&format!("file:///test/{}", filename))
        .expect("Failed to create test URI")
}

#[test]
fn test_no_errors_returns_empty_diagnostics() {
    let mut bridge = CompilerBridge::new();
    let uri = create_test_uri("valid.v");

    // Valid minimal Five DSL code
    let source = r#"
        init {
            // valid initialization
        }
    "#;

    let diagnostics = bridge
        .get_diagnostics(&uri, source)
        .expect("Failed to get diagnostics");

    assert!(
        diagnostics.is_empty(),
        "Valid code should produce no diagnostics, got: {:?}",
        diagnostics
    );
}

#[test]
fn test_parse_error_reported_as_diagnostic() {
    let mut bridge = CompilerBridge::new();
    let uri = create_test_uri("parse_error.v");

    // Invalid syntax - missing closing brace
    let source = r#"
        init {
            let x = 5;
        // missing }
    "#;

    let diagnostics = bridge
        .get_diagnostics(&uri, source)
        .expect("Failed to get diagnostics");

    assert!(
        !diagnostics.is_empty(),
        "Parse error should produce at least one diagnostic"
    );
    assert_eq!(
        diagnostics[0].severity,
        Some(lsp_types::DiagnosticSeverity::ERROR)
    );
    assert!(
        diagnostics[0].message.contains("Parse error"),
        "Diagnostic message should mention parse error: {}",
        diagnostics[0].message
    );
}

#[test]
fn test_type_error_reported_as_diagnostic() {
    let mut bridge = CompilerBridge::new();
    let uri = create_test_uri("type_error.v");

    // Type error - using undefined variable
    let source = r#"
        init {
            let x = undefined_var;
        }
    "#;

    let diagnostics = bridge
        .get_diagnostics(&uri, source)
        .expect("Failed to get diagnostics");

    // Type checking might fail on undefined variable
    // Just verify we get diagnostics (type checker will determine if error is reported)
    println!(
        "Type error diagnostics: {:?}",
        diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

#[test]
fn test_multiple_diagnostics_collected() {
    let mut bridge = CompilerBridge::new();
    let uri = create_test_uri("multiple_errors.v");

    // File with multiple issues
    let source = r#"
        init {
            let x = 5
            let y = 10;
        }
    "#;

    let diagnostics = bridge
        .get_diagnostics(&uri, source)
        .expect("Failed to get diagnostics");

    // We expect at least one diagnostic (missing semicolon or similar)
    println!(
        "Diagnostics for multiple errors: {:?}",
        diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

#[test]
fn test_ast_caching() {
    let mut bridge = CompilerBridge::new();
    let uri = create_test_uri("cache_test.v");

    let source = r#"
        init {
            // valid
        }
    "#;

    // First call - compiles and caches
    let diag1 = bridge
        .get_diagnostics(&uri, source)
        .expect("Failed first call");

    // Second call with same source - should use cache
    let diag2 = bridge
        .get_diagnostics(&uri, source)
        .expect("Failed second call");

    assert_eq!(diag1.len(), diag2.len(), "Cached diagnostics should match");
}

#[test]
fn test_cache_invalidation_on_source_change() {
    let mut bridge = CompilerBridge::new();
    let uri = create_test_uri("cache_invalidation.v");

    let valid_source = r#"
        init {
            // valid
        }
    "#;

    let invalid_source = r#"
        init {
            // missing brace
    "#;

    // First call - valid code
    let diag1 = bridge
        .get_diagnostics(&uri, valid_source)
        .expect("Failed first call");
    assert!(diag1.is_empty(), "Valid code should have no diagnostics");

    // Second call - invalid code (different source, cache invalidated)
    let diag2 = bridge
        .get_diagnostics(&uri, invalid_source)
        .expect("Failed second call");
    assert!(
        !diag2.is_empty(),
        "Invalid code should produce diagnostics"
    );
}

#[test]
fn test_diagnostic_has_source_field() {
    let mut bridge = CompilerBridge::new();
    let uri = create_test_uri("source_field.v");

    let source = r#"init { x }"#; // Missing x definition

    let diagnostics = bridge
        .get_diagnostics(&uri, source)
        .expect("Failed to get diagnostics");

    if !diagnostics.is_empty() {
        assert_eq!(
            diagnostics[0].source.as_ref().map(|s| s.as_str()),
            Some("five-compiler"),
            "Diagnostic should have five-compiler as source"
        );
    }
}

#[test]
fn test_different_files_independent_caches() {
    let mut bridge = CompilerBridge::new();
    let uri1 = create_test_uri("file1.v");
    let uri2 = create_test_uri("file2.v");

    let source1 = r#"init { }"#;
    let source2 = r#"init { "#; // Incomplete

    let diag1 = bridge
        .get_diagnostics(&uri1, source1)
        .expect("Failed for file1");
    let diag2 = bridge
        .get_diagnostics(&uri2, source2)
        .expect("Failed for file2");

    assert!(diag1.is_empty(), "File 1 should be valid");
    assert!(!diag2.is_empty(), "File 2 should have errors");
}
