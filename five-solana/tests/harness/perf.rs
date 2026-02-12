use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CuMetrics {
    pub deploy: u64,
    pub execute: u64,
    pub total: u64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BaselineSnapshot {
    pub commit: String,
    pub tests: BTreeMap<String, CuMetrics>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AllowlistEntry {
    pub owner: String,
    pub rationale: String,
    pub expires_commit: String,
    #[serde(default)]
    pub fields: Vec<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AllowlistSnapshot {
    pub commit: String,
    #[serde(default)]
    pub tests: BTreeMap<String, AllowlistEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Regression {
    pub field: &'static str,
    pub baseline: u64,
    pub current: u64,
}

#[inline]
pub fn print_bench_line(family: &str, opcode: &str, variant: &str, metrics: &CuMetrics) {
    println!(
        "BENCH family={} opcode={} variant={} deploy={} execute={} total={}",
        family, opcode, variant, metrics.deploy, metrics.execute, metrics.total
    );
}

#[inline]
pub fn print_scenario_line(name: &str, execute_units: u64, total_units: u64) {
    println!(
        "SCENARIO name={} execute={} total={}",
        name, execute_units, total_units
    );
}

pub fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

pub fn baseline_file_path(repo_root: &Path, commit: &str) -> PathBuf {
    repo_root
        .join("five-solana/tests/benchmarks/baseline")
        .join(format!("{}.json", commit))
}

pub fn allowlist_file_path(repo_root: &Path, commit: &str) -> PathBuf {
    repo_root
        .join("five-solana/tests/benchmarks/allowlist")
        .join(format!("{}.json", commit))
}

pub fn load_baseline(path: &Path) -> Option<BaselineSnapshot> {
    let raw = fs::read_to_string(path).ok()?;
    serde_json::from_str(&raw).ok()
}

pub fn load_allowlist(path: &Path) -> Option<AllowlistSnapshot> {
    let raw = fs::read_to_string(path).ok()?;
    serde_json::from_str(&raw).ok()
}

pub fn compare_cu(baseline: &CuMetrics, current: &CuMetrics) -> Vec<Regression> {
    let mut out = Vec::new();
    if current.deploy > baseline.deploy {
        out.push(Regression {
            field: "deploy",
            baseline: baseline.deploy,
            current: current.deploy,
        });
    }
    if current.execute > baseline.execute {
        out.push(Regression {
            field: "execute",
            baseline: baseline.execute,
            current: current.execute,
        });
    }
    if current.total > baseline.total {
        out.push(Regression {
            field: "total",
            baseline: baseline.total,
            current: current.total,
        });
    }
    out
}

pub fn assert_no_regression(test_name: &str, current: &CuMetrics) {
    let commit = std::env::var("FIVE_BENCH_BASELINE_COMMIT").unwrap_or_else(|_| "local".to_string());
    let root = repo_root();
    let baseline_path = baseline_file_path(&root, &commit);
    let allowlist_path = allowlist_file_path(&root, &commit);

    let Some(baseline) = load_baseline(&baseline_path) else {
        println!(
            "BENCH baseline_missing commit={} path={} test={}",
            commit,
            baseline_path.display(),
            test_name
        );
        return;
    };

    let Some(expected) = baseline.tests.get(test_name) else {
        println!(
            "BENCH baseline_entry_missing commit={} test={} path={}",
            commit,
            test_name,
            baseline_path.display()
        );
        return;
    };

    let regressions = compare_cu(expected, current);
    if regressions.is_empty() {
        return;
    }

    let allowlist = load_allowlist(&allowlist_path);
    let allow = allowlist
        .as_ref()
        .and_then(|a| a.tests.get(test_name));

    let mut blocked: Vec<&Regression> = Vec::new();
    for regression in &regressions {
        let allowed = allow
            .map(|entry| {
                if entry.fields.is_empty() {
                    true
                } else {
                    entry.fields.iter().any(|field| field == regression.field)
                }
            })
            .unwrap_or(false);
        if !allowed {
            blocked.push(regression);
        }
    }

    if blocked.is_empty() {
        if let Some(entry) = allow {
            println!(
                "BENCH allowlisted test={} owner={} rationale={} expires_commit={} path={}",
                test_name,
                entry.owner,
                entry.rationale,
                entry.expires_commit,
                allowlist_path.display()
            );
        }
        return;
    }

    let details = blocked
        .iter()
        .map(|r| format!("{}:{}->{}", r.field, r.baseline, r.current))
        .collect::<Vec<_>>()
        .join(", ");

    panic!(
        "CU regression detected for {} against {} ({}): {}",
        test_name,
        commit,
        baseline_path.display(),
        details
    );
}
