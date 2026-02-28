use std::{fs, path::PathBuf};

use five_lsp::CompilerBridge;
use lsp_types::{DiagnosticSeverity, Url};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Matrix {
    scenarios: Vec<Scenario>,
}

#[derive(Debug, Deserialize)]
struct Scenario {
    id: String,
    source: String,
    kind: String,
    expected_error_contains: Option<String>,
    layers: Layers,
}

#[derive(Debug, Deserialize)]
struct Layers {
    lsp: bool,
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn load_matrix() -> Matrix {
    let root = repo_root();
    let path = root.join("testing/dsl-feature-matrix.json");
    let raw = fs::read_to_string(path).expect("read DSL feature matrix");
    serde_json::from_str(&raw).expect("parse DSL feature matrix")
}

fn source_path(root: &std::path::Path, scenario: &Scenario) -> PathBuf {
    root.join("five-cli/test-scripts").join(&scenario.source)
}

fn scenario_uri(scenario: &Scenario) -> Url {
    Url::parse(&format!("file:///dsl-feature-matrix/{}", scenario.id)).expect("valid test URI")
}

#[test]
fn lsp_matrix_scenarios_match_expected_diagnostics() {
    let root = repo_root();
    let matrix = load_matrix();
    let mut bridge = CompilerBridge::new();

    for scenario in matrix
        .scenarios
        .iter()
        .filter(|scenario| scenario.layers.lsp)
    {
        let source = fs::read_to_string(source_path(&root, scenario)).expect("read matrix source");
        let diagnostics = bridge
            .get_diagnostics(&scenario_uri(scenario), &source)
            .unwrap_or_else(|error| {
                panic!("scenario {} diagnostics failed: {}", scenario.id, error)
            });

        match scenario.kind.as_str() {
            "positive" => {
                assert!(
                    diagnostics.is_empty(),
                    "scenario {} should be diagnostic-free, got {:?}",
                    scenario.id,
                    diagnostics
                );
            }
            "negative" => {
                assert!(
                    diagnostics
                        .iter()
                        .any(|diagnostic| diagnostic.severity == Some(DiagnosticSeverity::ERROR)),
                    "scenario {} should emit at least one error diagnostic",
                    scenario.id
                );
                let expected = scenario
                    .expected_error_contains
                    .as_deref()
                    .expect("negative LSP scenario expected_error_contains");
                let messages = diagnostics
                    .iter()
                    .map(|diagnostic| diagnostic.message.as_str())
                    .collect::<Vec<_>>()
                    .join("\n");
                assert!(
                    messages.contains(expected),
                    "scenario {} expected diagnostic containing {:?}, got {}",
                    scenario.id,
                    expected,
                    messages
                );
            }
            other => panic!("unknown scenario kind: {}", other),
        }
    }
}
