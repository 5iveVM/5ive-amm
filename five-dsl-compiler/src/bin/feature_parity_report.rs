use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

const ALL_LAYERS: [&str; 8] = [
    "compiler",
    "vm",
    "cli",
    "wasm",
    "lsp",
    "solana_runtime",
    "validator_localnet",
    "validator_devnet_tracked",
];

#[derive(Debug, Clone, Deserialize)]
struct Matrix {
    categories: Vec<Category>,
    scenarios: Vec<Scenario>,
}

#[derive(Debug, Clone, Deserialize)]
struct Category {
    id: String,
    description: String,
    required_layers: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct Scenario {
    category: String,
    kind: String,
    layers: BTreeMap<String, bool>,
}

#[derive(Debug, Clone, Serialize)]
struct CategoryReport {
    category: String,
    description: String,
    scenario_count: usize,
    positive_count: usize,
    negative_count: usize,
    layer_coverage: BTreeMap<String, bool>,
    required_layers: Vec<String>,
    uncataloged_fixtures: usize,
    strength: String,
}

#[derive(Debug, Clone, Serialize)]
struct ParityReport {
    generated_by: String,
    categories: Vec<CategoryReport>,
    summary: Summary,
    uncataloged_fixtures: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct Summary {
    total_categories: usize,
    green: usize,
    yellow: usize,
    red: usize,
}

fn load_matrix(path: &Path) -> Result<Matrix, Box<dyn std::error::Error>> {
    let raw = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&raw)?)
}

fn find_uncataloged_fixtures(root: &Path, tracked: &BTreeSet<String>) -> Vec<String> {
    fn walk(root: &Path, dir: &Path, tracked: &BTreeSet<String>, out: &mut Vec<String>) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(root, &path, tracked, out);
                continue;
            }
            if path.extension().and_then(|ext| ext.to_str()) != Some("v") {
                continue;
            }
            let relative = path
                .strip_prefix(root)
                .expect("fixture relative path")
                .to_string_lossy()
                .replace('\\', "/");
            if !tracked.contains(&relative) {
                out.push(relative);
            }
        }
    }

    let mut out = Vec::new();
    walk(root, root, tracked, &mut out);
    out.sort();
    out
}

fn classify(
    required_layers: &[String],
    layer_coverage: &BTreeMap<String, bool>,
    scenario_count: usize,
) -> String {
    if scenario_count == 0 {
        return "red".to_string();
    }

    let missing_required = required_layers
        .iter()
        .any(|layer| !layer_coverage.get(layer).copied().unwrap_or(false));

    if missing_required {
        "yellow".to_string()
    } else {
        "green".to_string()
    }
}

fn write_markdown(report: &ParityReport, path: &Path) -> std::io::Result<()> {
    let mut md = String::new();
    md.push_str("# 5IVE DSL Feature Parity Matrix\n\n");
    md.push_str("| Category | Scenarios | Positive | Negative | Compiler | VM | CLI | WASM | LSP | Runtime | Localnet | Devnet Tracked | Uncataloged | Strength |\n");
    md.push_str(
        "|---|---:|---:|---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|---:|:---:|\n",
    );

    for category in &report.categories {
        let layer = |name: &str| {
            if category.layer_coverage.get(name).copied().unwrap_or(false) {
                "Y"
            } else {
                "N"
            }
        };
        md.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
            category.category,
            category.scenario_count,
            category.positive_count,
            category.negative_count,
            layer("compiler"),
            layer("vm"),
            layer("cli"),
            layer("wasm"),
            layer("lsp"),
            layer("solana_runtime"),
            layer("validator_localnet"),
            layer("validator_devnet_tracked"),
            category.uncataloged_fixtures,
            category.strength,
        ));
    }

    md.push_str("\n");
    md.push_str(&format!(
        "Summary: total={}, green={}, yellow={}, red={}\n\n",
        report.summary.total_categories,
        report.summary.green,
        report.summary.yellow,
        report.summary.red
    ));

    if !report.uncataloged_fixtures.is_empty() {
        md.push_str("## Uncataloged Fixtures\n\n");
        for fixture in &report.uncataloged_fixtures {
            md.push_str(&format!("- `{}`\n", fixture));
        }
    }

    fs::write(path, md)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let fixture_root = workspace_root.join("five-cli").join("test-scripts");
    let matrix_path = workspace_root
        .join("testing")
        .join("dsl-feature-matrix.json");
    let output_root = workspace_root.join("target").join("feature-parity");
    fs::create_dir_all(&output_root)?;

    let matrix = load_matrix(&matrix_path)?;
    let tracked_sources = {
        let raw = fs::read_to_string(&matrix_path)?;
        let json: serde_json::Value = serde_json::from_str(&raw)?;
        json["scenarios"]
            .as_array()
            .unwrap_or(&Vec::new())
            .iter()
            .filter_map(|scenario| scenario["source"].as_str().map(ToOwned::to_owned))
            .collect::<BTreeSet<_>>()
    };
    let uncataloged = find_uncataloged_fixtures(&fixture_root, &tracked_sources);
    let mut uncataloged_by_category: BTreeMap<String, usize> = BTreeMap::new();
    for fixture in &uncataloged {
        let category = fixture
            .split('/')
            .next()
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| "__root__".to_string());
        *uncataloged_by_category.entry(category).or_insert(0) += 1;
    }

    let mut categories = Vec::new();
    for category in &matrix.categories {
        let scenarios = matrix
            .scenarios
            .iter()
            .filter(|scenario| scenario.category == category.id)
            .collect::<Vec<_>>();

        let mut layer_coverage = BTreeMap::new();
        for layer in ALL_LAYERS {
            let covered = scenarios
                .iter()
                .any(|scenario| scenario.layers.get(layer).copied().unwrap_or(false));
            layer_coverage.insert(layer.to_string(), covered);
        }

        let strength = classify(&category.required_layers, &layer_coverage, scenarios.len());
        categories.push(CategoryReport {
            category: category.id.clone(),
            description: category.description.clone(),
            scenario_count: scenarios.len(),
            positive_count: scenarios
                .iter()
                .filter(|scenario| scenario.kind == "positive")
                .count(),
            negative_count: scenarios
                .iter()
                .filter(|scenario| scenario.kind == "negative")
                .count(),
            layer_coverage,
            required_layers: category.required_layers.clone(),
            uncataloged_fixtures: *uncataloged_by_category.get(&category.id).unwrap_or(&0),
            strength,
        });
    }

    let summary = Summary {
        total_categories: categories.len(),
        green: categories
            .iter()
            .filter(|category| category.strength == "green")
            .count(),
        yellow: categories
            .iter()
            .filter(|category| category.strength == "yellow")
            .count(),
        red: categories
            .iter()
            .filter(|category| category.strength == "red")
            .count(),
    };

    let report = ParityReport {
        generated_by: "five-dsl-compiler/bin/feature_parity_report".to_string(),
        categories,
        summary,
        uncataloged_fixtures: uncataloged,
    };

    let json_path = output_root.join("matrix.json");
    let md_path = output_root.join("matrix.md");
    fs::write(&json_path, serde_json::to_string_pretty(&report)?)?;
    write_markdown(&report, &md_path)?;

    println!("Wrote {}", json_path.display());
    println!("Wrote {}", md_path.display());
    Ok(())
}
