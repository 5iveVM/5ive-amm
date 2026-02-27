use std::{fs, path::PathBuf};

use five_dsl_compiler::DslCompiler;
use five_protocol::types;
use five_vm_mito::{stack::StackStorage, MitoVM, Value, FIVE_VM_PROGRAM_ID};
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
    function: Option<u32>,
    params_source: String,
    params: Option<Vec<serde_json::Value>>,
    expected_result: Option<serde_json::Value>,
    layers: Layers,
    requires_accounts: bool,
    requires_cpi: bool,
}

#[derive(Debug, Deserialize)]
struct Layers {
    vm: bool,
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

fn parse_params_from_comment(source: &str) -> Vec<serde_json::Value> {
    let line = source
        .lines()
        .map(str::trim)
        .find(|line| line.contains("@test-params"));

    let Some(line) = line else {
        return Vec::new();
    };

    let params_str = line
        .split("@test-params")
        .nth(1)
        .map(str::trim)
        .unwrap_or("");
    if params_str.is_empty() {
        return Vec::new();
    }
    if params_str.starts_with('[') {
        return serde_json::from_str(params_str).expect("parse inline JSON @test-params");
    }

    params_str
        .split_whitespace()
        .map(|token| {
            if token == "true" {
                serde_json::Value::Bool(true)
            } else if token == "false" {
                serde_json::Value::Bool(false)
            } else if let Ok(number) = token.parse::<u64>() {
                serde_json::Value::Number(number.into())
            } else {
                serde_json::Value::String(token.to_string())
            }
        })
        .collect()
}

fn scenario_params(source: &str, scenario: &Scenario) -> Vec<serde_json::Value> {
    match scenario.params_source.as_str() {
        "inline" => scenario.params.clone().unwrap_or_default(),
        "test-params-comment" => parse_params_from_comment(source),
        other => panic!("unsupported params source: {}", other),
    }
}

fn encode_execute_input(function_index: u32, params: &[serde_json::Value]) -> Vec<u8> {
    let mut input = Vec::new();
    input.extend_from_slice(&function_index.to_le_bytes());
    input.extend_from_slice(&(params.len() as u32).to_le_bytes());

    for param in params {
        match param {
            serde_json::Value::Number(number) => {
                let value = number.as_u64().expect("numeric params must be u64");
                input.push(types::U64);
                input.extend_from_slice(&value.to_le_bytes());
            }
            serde_json::Value::Bool(value) => {
                input.push(types::BOOL);
                let encoded: u32 = if *value { 1 } else { 0 };
                input.extend_from_slice(&encoded.to_le_bytes());
            }
            other => panic!("unsupported VM matrix param: {}", other),
        }
    }

    input
}

fn assert_expected_value(actual: Option<Value>, expected: &serde_json::Value, scenario_id: &str) {
    match expected {
        serde_json::Value::Number(number) => {
            let expected = number.as_u64().expect("expected numeric result must be u64");
            assert_eq!(
                actual,
                Some(Value::U64(expected)),
                "scenario {} returned unexpected VM value",
                scenario_id
            );
        }
        serde_json::Value::Bool(value) => {
            assert_eq!(
                actual,
                Some(Value::Bool(*value)),
                "scenario {} returned unexpected VM value",
                scenario_id
            );
        }
        other => panic!("unsupported expected VM value: {}", other),
    }
}

#[test]
fn vm_matrix_generic_scenarios_execute_successfully() {
    let root = repo_root();
    let matrix = load_matrix();

    for scenario in matrix.scenarios.iter().filter(|scenario| {
        scenario.layers.vm && scenario.kind == "positive" && !scenario.requires_accounts && !scenario.requires_cpi
    }) {
        let source = fs::read_to_string(source_path(&root, scenario)).expect("read matrix source");
        let bytecode = DslCompiler::compile_dsl(&source)
            .unwrap_or_else(|error| panic!("scenario {} failed to compile: {}", scenario.id, error));
        let params = scenario_params(&source, scenario);
        let input = encode_execute_input(scenario.function.unwrap_or(0), &params);
        let mut storage = StackStorage::new();
        let result = MitoVM::execute_direct(&bytecode, &input, &[], &FIVE_VM_PROGRAM_ID, &mut storage)
            .unwrap_or_else(|error| panic!("scenario {} failed VM execution: {:?}", scenario.id, error));

        if let Some(expected) = &scenario.expected_result {
            assert_expected_value(result, expected, &scenario.id);
        }
    }
}
