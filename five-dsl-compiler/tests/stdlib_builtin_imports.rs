use std::{collections::BTreeSet, fs, path::PathBuf};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct BuiltinMatrix {
    builtins: Vec<BuiltinEntry>,
}

#[derive(Debug, Deserialize)]
struct BuiltinEntry {
    name: String,
    layers: BuiltinLayers,
    unit_suites: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct BuiltinLayers {
    compiler: bool,
    bytecode_unit: bool,
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn load_builtin_matrix() -> BuiltinMatrix {
    let path = repo_root().join("testing/dsl-builtin-matrix.json");
    let raw = fs::read_to_string(path).expect("read builtin matrix");
    serde_json::from_str(&raw).expect("parse builtin matrix")
}

fn load_stdlib_builtin_names() -> BTreeSet<String> {
    let path = repo_root().join("five-stdlib/std/builtins.v");
    let raw = fs::read_to_string(path).expect("read stdlib builtins");
    raw.match_indices("pub ")
        .filter_map(|(offset, _)| {
            let rest = &raw[offset + 4..];
            let name: String = rest
                .chars()
                .take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '_')
                .collect();
            if name.is_empty() {
                None
            } else {
                Some(name)
            }
        })
        .collect()
}

#[test]
fn builtin_inventory_matches_stdlib_exports() {
    let matrix = load_builtin_matrix();
    let matrix_names: BTreeSet<String> = matrix
        .builtins
        .iter()
        .map(|builtin| builtin.name.clone())
        .collect();
    let stdlib_names = load_stdlib_builtin_names();

    assert_eq!(
        matrix_names.len(),
        29,
        "expected 29 stdlib builtin wrappers in inventory"
    );
    assert_eq!(
        matrix_names, stdlib_names,
        "builtin inventory must exactly match five-stdlib/std/builtins.v exports"
    );
}

#[test]
fn every_builtin_has_compiler_and_bytecode_ownership() {
    let matrix = load_builtin_matrix();

    for builtin in matrix.builtins {
        assert!(
            builtin.layers.compiler,
            "builtin {} must be compiler-tracked",
            builtin.name
        );
        assert!(
            builtin.layers.bytecode_unit,
            "builtin {} must be bytecode-unit tracked",
            builtin.name
        );
        assert!(
            builtin
                .unit_suites
                .iter()
                .any(|suite| suite.contains("five-dsl-compiler/")),
            "builtin {} must name at least one compiler-owned suite",
            builtin.name
        );
    }
}
