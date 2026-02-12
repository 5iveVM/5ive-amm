//! WASM API Contract Tests
//!
//! Validates that all WASM methods adhere to the contract defined in LSP_CONTRACT.md

use five_lsp::wasm::FiveLspWasm;
use lsp_types::{Diagnostic, Hover, Location, WorkspaceEdit};

#[test]
fn test_workspace_edit_json_consistency() {
    // Test that WorkspaceEdit serialization is consistent
    let mut lsp = FiveLspWasm::new();
    let uri = "file:///test.v";
    let source = "let oldName: u64 = 5;\nlet x = oldName + 1;";

    // Attempt rename
    let result = lsp.rename(uri, source, 0, 4, "newName").unwrap();

    if let Some(edit_json) = result {
        // Verify it deserializes to valid WorkspaceEdit
        let edit: WorkspaceEdit = serde_json::from_str(&edit_json)
            .expect("WorkspaceEdit should deserialize correctly");

        // Verify structure: should have either 'changes' or 'documentChanges'
        assert!(
            edit.changes.is_some() || edit.document_changes.is_some(),
            "WorkspaceEdit must have either changes or documentChanges"
        );

        // If changes exist, verify URI format
        if let Some(changes) = &edit.changes {
            for (uri_str, text_edits) in changes {
                assert!(
                    uri_str.starts_with("file://"),
                    "URI in WorkspaceEdit must use file:// scheme"
                );
                assert!(!text_edits.is_empty(), "Text edits should not be empty");

                // Verify all ranges are valid (start <= end)
                for edit in text_edits {
                    assert!(
                        edit.range.start.line <= edit.range.end.line,
                        "Range start line must be <= end line"
                    );
                    if edit.range.start.line == edit.range.end.line {
                        assert!(
                            edit.range.start.character <= edit.range.end.character,
                            "Range start character must be <= end character on same line"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn test_diagnostic_json_roundtrip() {
    let mut lsp = FiveLspWasm::new();
    let uri = "file:///test.v";
    let source = "let x: u64 = \"invalid\";"; // Type error

    let diagnostics_json = lsp.get_diagnostics(uri, source).unwrap();

    // Deserialize to verify JSON format
    let diagnostics: Vec<Diagnostic> = serde_json::from_str(&diagnostics_json)
        .expect("Diagnostics should deserialize correctly");

    // Verify all diagnostics have valid ranges
    for diagnostic in &diagnostics {
        assert!(
            diagnostic.range.start.line <= diagnostic.range.end.line,
            "Diagnostic range must be valid"
        );
        assert!(!diagnostic.message.is_empty(), "Diagnostic message should not be empty");
    }

    // Re-serialize and verify it matches
    let re_serialized = serde_json::to_string(&diagnostics).unwrap();
    let re_parsed: Vec<Diagnostic> = serde_json::from_str(&re_serialized).unwrap();
    assert_eq!(
        diagnostics.len(),
        re_parsed.len(),
        "Round-trip should preserve diagnostic count"
    );
}

#[test]
fn test_hover_json_contract() {
    let mut lsp = FiveLspWasm::new();
    let uri = "file:///test.v";
    let source = "let x: u64 = 5;";

    let hover_result = lsp.get_hover(uri, source, 0, 4).unwrap();

    if let Some(hover_json) = hover_result {
        // Verify it deserializes to valid Hover
        let hover: Hover = serde_json::from_str(&hover_json)
            .expect("Hover should deserialize correctly");

        // Verify contents exist
        match &hover.contents {
            lsp_types::HoverContents::Scalar(markup) => {
                assert!(!markup.value.is_empty(), "Hover content should not be empty");
            }
            lsp_types::HoverContents::Array(items) => {
                assert!(!items.is_empty(), "Hover array should not be empty");
            }
            lsp_types::HoverContents::Markup(markup) => {
                assert!(!markup.value.is_empty(), "Hover markup should not be empty");
            }
        }
    }
}

#[test]
fn test_definition_json_contract() {
    let mut lsp = FiveLspWasm::new();
    let uri = "file:///test.v";
    let source = "let x: u64 = 5;\nlet y = x + 1;";

    let definition_result = lsp.get_definition(uri, source, 1, 8).unwrap();

    if let Some(definition_json) = definition_result {
        // Verify it deserializes to valid Location
        let location: Location = serde_json::from_str(&definition_json)
            .expect("Location should deserialize correctly");

        // Verify URI format
        assert!(
            location.uri.as_str().starts_with("file://"),
            "Location URI must use file:// scheme"
        );

        // Verify range is valid
        assert!(
            location.range.start.line <= location.range.end.line,
            "Location range must be valid"
        );
    }
}

#[test]
fn test_prepare_rename_with_uri_parameter() {
    let lsp = FiveLspWasm::new();
    let uri = "file:///test.v";
    let source = "let oldName: u64 = 5;";

    // Test with valid URI
    let result = lsp.prepare_rename(uri, source, 0, 4).unwrap();
    assert!(result.is_some(), "prepare_rename should find symbol");
    assert_eq!(result.unwrap(), "oldName", "Should return correct identifier");

    // Test with invalid position
    let result = lsp.prepare_rename(uri, source, 0, 0).unwrap();
    assert!(
        result.is_none() || result.unwrap() == "let",
        "prepare_rename at keyword position"
    );
}

#[test]
fn test_invalid_uri_error() {
    let mut lsp = FiveLspWasm::new();
    let source = "let x: u64 = 5;";

    // Test various invalid URIs
    let invalid_uris = vec![
        "not-a-uri",
        "http://example.com/test.v", // Wrong scheme
        "file://test.v",             // Missing authority (should be ///)
        "",
    ];

    for invalid_uri in invalid_uris {
        let result = lsp.get_diagnostics(invalid_uri, source);
        assert!(result.is_err(), "Invalid URI '{}' should produce error", invalid_uri);

        if let Err(err) = result {
            let err_str = format!("{:?}", err);
            assert!(
                err_str.contains("Invalid URI") || err_str.contains("uri"),
                "Error message should mention URI: {}",
                err_str
            );
        }
    }
}

#[test]
fn test_empty_source_edge_case() {
    let mut lsp = FiveLspWasm::new();
    let uri = "file:///test.v";
    let source = "";

    // get_diagnostics should handle empty source gracefully
    let diagnostics_json = lsp.get_diagnostics(uri, source).unwrap();
    let diagnostics: Vec<Diagnostic> = serde_json::from_str(&diagnostics_json).unwrap();
    // Empty source may have diagnostics (e.g., "empty file" warning) or be empty
    // The key is it shouldn't panic

    // get_hover on empty source
    let hover = lsp.get_hover(uri, source, 0, 0).unwrap();
    assert!(hover.is_none(), "Hover on empty source should return None");

    // get_definition on empty source
    let def = lsp.get_definition(uri, source, 0, 0).unwrap();
    assert!(def.is_none(), "Definition on empty source should return None");
}

#[test]
fn test_position_out_of_bounds() {
    let mut lsp = FiveLspWasm::new();
    let uri = "file:///test.v";
    let source = "let x: u64 = 5;"; // Single line

    // Test position beyond EOF
    let hover = lsp.get_hover(uri, source, 999, 999).unwrap();
    assert!(hover.is_none(), "Position beyond EOF should return None");

    let def = lsp.get_definition(uri, source, 999, 999).unwrap();
    assert!(def.is_none(), "Position beyond EOF should return None for definition");
}

#[test]
fn test_clear_caches_idempotent() {
    let mut lsp = FiveLspWasm::new();
    let uri = "file:///test.v";
    let source = "let x: u64 = 5;";

    // Get diagnostics to populate cache
    let _ = lsp.get_diagnostics(uri, source).unwrap();

    // Clear caches multiple times (should not panic)
    lsp.clear_caches();
    lsp.clear_caches();
    lsp.clear_caches();

    // Should still work after clearing
    let diagnostics_json = lsp.get_diagnostics(uri, source).unwrap();
    let _diagnostics: Vec<Diagnostic> = serde_json::from_str(&diagnostics_json).unwrap();
}
