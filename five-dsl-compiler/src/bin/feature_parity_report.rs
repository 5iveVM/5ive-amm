use serde::Serialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize)]
struct CategoryReport {
    category: String,
    fixtures: usize,
    compiler_tests: Vec<String>,
    stage_coverage: StageCoverage,
    strength: String,
}

#[derive(Debug, Clone, Serialize)]
struct StageCoverage {
    syntax: bool,
    type_semantics: bool,
    bytecode: bool,
    runtime: bool,
}

#[derive(Debug, Clone, Serialize)]
struct ParityReport {
    generated_by: String,
    categories: Vec<CategoryReport>,
    summary: Summary,
}

#[derive(Debug, Clone, Serialize)]
struct Summary {
    total_categories: usize,
    green: usize,
    yellow: usize,
    red: usize,
}

fn list_fixture_categories(root: &Path) -> BTreeMap<String, usize> {
    let mut out = BTreeMap::new();
    let entries = match fs::read_dir(root) {
        Ok(e) => e,
        Err(_) => return out,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        let mut count = 0usize;
        if let Ok(files) = fs::read_dir(&path) {
            for f in files.flatten() {
                let fp = f.path();
                if fp.is_file() {
                    if let Some(ext) = fp.extension().and_then(|e| e.to_str()) {
                        if ext == "five" || ext == "v" {
                            count += 1;
                        }
                    }
                }
            }
        }
        out.insert(name, count);
    }
    out
}

fn compiler_test_mapping() -> BTreeMap<String, Vec<String>> {
    let mut map = BTreeMap::new();
    map.insert(
        "01-language-basics".to_string(),
        vec![
            "test_compiler_features".to_string(),
            "test_function_dispatch".to_string(),
            "test_scope_analyzer".to_string(),
        ],
    );
    map.insert(
        "02-operators-expressions".to_string(),
        vec![
            "test_bytecode_snapshots".to_string(),
            "golden_bytecode".to_string(),
            "lib::test_multiplication_precedence_over_addition_correct".to_string(),
        ],
    );
    map.insert(
        "03-control-flow".to_string(),
        vec![
            "test_compiler_features::test_if_statement_compilation".to_string(),
            "lib::test_nested_control_flow".to_string(),
            "lib::test_else_if_chain_compiles".to_string(),
        ],
    );
    map.insert(
        "04-account-system".to_string(),
        vec![
            "test_constraints".to_string(),
            "test_init_constraint_bytecode".to_string(),
            "lending_regression_field_limit".to_string(),
        ],
    );
    map.insert(
        "05-blockchain-integration".to_string(),
        vec![
            "cpi_compile_regression_tests".to_string(),
            "cpi_unit_tests".to_string(),
            "protocol_alignment_tests".to_string(),
        ],
    );
    map.insert(
        "06-advanced-features".to_string(),
        vec![
            "lending_regression_u128_fields".to_string(),
            "lib::test_generic_type_option_result".to_string(),
            "lib::test_array_types_rust_and_ts_style".to_string(),
        ],
    );
    map.insert(
        "07-error-system".to_string(),
        vec![
            "diagnostics_reserved_keyword_function".to_string(),
            "error::templates".to_string(),
        ],
    );
    map.insert(
        "08-match-expressions".to_string(),
        vec![
            "lib::test_match_expression_parsing".to_string(),
            "lib::test_match_expression_with_guard".to_string(),
        ],
    );
    map.insert(
        "10-featured-examples".to_string(),
        vec![
            "template_modernization_regression_tests".to_string(),
            "test_bytecode_benchmarks".to_string(),
        ],
    );
    map.insert(
        "11-token-examples".to_string(),
        vec![
            "cpi_compile_regression_tests".to_string(),
            "template_modernization_regression_tests::token_template_import_contract_regression"
                .to_string(),
        ],
    );
    map
}

fn stage_coverage_for(category: &str, fixtures: usize) -> StageCoverage {
    let runtime = matches!(
        category,
        "04-account-system"
            | "05-blockchain-integration"
            | "10-featured-examples"
            | "11-token-examples"
    );
    StageCoverage {
        syntax: fixtures > 0,
        type_semantics: fixtures > 0,
        bytecode: fixtures > 0,
        runtime,
    }
}

fn classify_strength(coverage: &StageCoverage) -> String {
    match (coverage.syntax, coverage.type_semantics, coverage.bytecode, coverage.runtime) {
        (true, true, true, true) => "green".to_string(),
        (true, true, true, false) => "yellow".to_string(),
        _ => "red".to_string(),
    }
}

fn write_markdown(report: &ParityReport, path: &Path) -> std::io::Result<()> {
    let mut md = String::new();
    md.push_str("# 5IVE DSL Feature Parity Matrix\n\n");
    md.push_str("| Category | Fixtures | Syntax | Type | Bytecode | Runtime | Strength | Compiler Tests |\n");
    md.push_str("|---|---:|:---:|:---:|:---:|:---:|:---:|---|\n");
    for c in &report.categories {
        let tests = c.compiler_tests.join(", ");
        md.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} |\n",
            c.category,
            c.fixtures,
            if c.stage_coverage.syntax { "Y" } else { "N" },
            if c.stage_coverage.type_semantics { "Y" } else { "N" },
            if c.stage_coverage.bytecode { "Y" } else { "N" },
            if c.stage_coverage.runtime { "Y" } else { "N" },
            c.strength,
            tests
        ));
    }
    md.push_str("\n");
    md.push_str(&format!(
        "Summary: total={}, green={}, yellow={}, red={}\n",
        report.summary.total_categories, report.summary.green, report.summary.yellow, report.summary.red
    ));
    fs::write(path, md)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let fixture_root = workspace_root.join("five-cli").join("test-scripts");
    let output_root = workspace_root.join("target").join("feature-parity");
    fs::create_dir_all(&output_root)?;

    let fixtures = list_fixture_categories(&fixture_root);
    let test_map = compiler_test_mapping();

    let mut categories = Vec::new();
    for (category, fixture_count) in fixtures {
        let tests = test_map.get(&category).cloned().unwrap_or_default();
        let coverage = stage_coverage_for(&category, fixture_count);
        let strength = classify_strength(&coverage);
        categories.push(CategoryReport {
            category,
            fixtures: fixture_count,
            compiler_tests: tests,
            stage_coverage: coverage,
            strength,
        });
    }

    let summary = Summary {
        total_categories: categories.len(),
        green: categories.iter().filter(|c| c.strength == "green").count(),
        yellow: categories.iter().filter(|c| c.strength == "yellow").count(),
        red: categories.iter().filter(|c| c.strength == "red").count(),
    };

    let report = ParityReport {
        generated_by: "five-dsl-compiler/bin/feature_parity_report".to_string(),
        categories,
        summary,
    };

    let json_path = output_root.join("matrix.json");
    let md_path = output_root.join("matrix.md");
    fs::write(&json_path, serde_json::to_string_pretty(&report)?)?;
    write_markdown(&report, &md_path)?;

    println!("Wrote {}", json_path.display());
    println!("Wrote {}", md_path.display());
    Ok(())
}
