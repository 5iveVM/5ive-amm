use std::{fs, path::PathBuf};

use five_dsl_compiler::DslCompiler;
use five_protocol::FIVE_MAGIC;
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
    bytecode_assertions: Option<BytecodeAssertions>,
    layers: Layers,
}

#[derive(Debug, Deserialize)]
struct BytecodeAssertions {
    must_start_with_magic: Option<bool>,
    min_length: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct Layers {
    compiler: bool,
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

fn scenario_source_path(root: &std::path::Path, scenario: &Scenario) -> PathBuf {
    root.join("five-cli/test-scripts").join(&scenario.source)
}

#[test]
fn compiler_matrix_scenarios_match_expectations() {
    let root = repo_root();
    let matrix = load_matrix();

    for scenario in matrix.scenarios.iter().filter(|scenario| scenario.layers.compiler) {
        let source_path = scenario_source_path(&root, scenario);
        let source = fs::read_to_string(&source_path)
            .unwrap_or_else(|error| panic!("failed reading {}: {}", source_path.display(), error));
        let compiled = DslCompiler::compile_dsl(&source);

        match scenario.kind.as_str() {
            "positive" => {
                let bytecode = compiled.unwrap_or_else(|error| {
                    panic!("scenario {} should compile, got {}", scenario.id, error)
                });
                if let Some(assertions) = &scenario.bytecode_assertions {
                    if assertions.must_start_with_magic.unwrap_or(false) {
                        assert!(
                            bytecode.starts_with(&FIVE_MAGIC),
                            "scenario {} missing FIVE magic header",
                            scenario.id
                        );
                    }
                    if let Some(min_length) = assertions.min_length {
                        assert!(
                            bytecode.len() >= min_length,
                            "scenario {} bytecode shorter than {} bytes",
                            scenario.id,
                            min_length
                        );
                    }
                }
            }
            "negative" => {
                let error = compiled.expect_err("negative compiler scenario should fail");
                let message = error.to_string();
                let expected = scenario
                    .expected_error_contains
                    .as_deref()
                    .expect("negative scenario expected_error_contains");
                assert!(
                    message.contains(expected),
                    "scenario {} expected error containing {:?}, got {}",
                    scenario.id,
                    expected,
                    message
                );
            }
            other => panic!("unknown scenario kind: {}", other),
        }
    }
}
