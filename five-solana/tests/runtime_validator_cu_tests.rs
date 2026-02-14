mod harness;

use std::path::PathBuf;

use harness::validator::{
    run_external_burst_non_cpi, run_external_interface_mapping_non_cpi, run_external_non_cpi,
    run_fixture_scenario, Network, ScenarioRunResult, ValidatorHarness,
};

// Env contract:
// - FIVE_CU_NETWORK=localnet|devnet
// - FIVE_CU_PAYER_KEYPAIR=/path/to/id.json
// - FIVE_CU_PROGRAM_ID=<predeployed five program id>
// - FIVE_CU_RPC_URL=<optional>
// - FIVE_CU_SCENARIOS=<optional comma-separated list>
// - FIVE_CU_RESULTS_FILE=<optional explicit output path>
// - FIVE_CU_DEVNET_OPT_IN=1 (required only for devnet runs)

const DEFAULT_SCENARIOS: [&str; 6] = [
    "token_full_e2e",
    "external_non_cpi",
    "external_interface_mapping_non_cpi",
    "external_burst_non_cpi",
    "memory_string_heavy",
    "arithmetic_intensive",
];

fn parse_scenarios() -> Vec<String> {
    if let Ok(raw) = std::env::var("FIVE_CU_SCENARIOS") {
        return raw
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();
    }
    DEFAULT_SCENARIOS.iter().map(|s| s.to_string()).collect()
}

fn fixture_for(name: &str, repo_root: &std::path::Path) -> Option<PathBuf> {
    match name {
        "token_full_e2e" => Some(repo_root.join("five-templates/token/runtime-fixtures/init_mint.json")),
        "memory_string_heavy" => Some(repo_root.join("five-templates/token/runtime-fixtures/init_mint.json")),
        "arithmetic_intensive" => {
            Some(repo_root.join("five-templates/arithmetic-bench/runtime-fixtures/arithmetic_heavy.json"))
        }
        _ => None,
    }
}

fn print_summary_line(result: &ScenarioRunResult) {
    let execute: u64 = result.step_results.iter().map(|s| s.units).sum();
    println!(
        "BPF_CU validator scenario={} deploy={} execute={} total={} steps={}",
        result.name,
        result.deploy_units,
        execute,
        result.total_units,
        result.step_results.len()
    );
}

#[test]
#[ignore = "requires running validator or devnet config"]
fn validator_cu_orchestrator() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");

    let mut harness = ValidatorHarness::from_env().unwrap_or_else(|e| panic!("validator harness init failed: {}", e));

    if harness.network == Network::Devnet {
        if std::env::var("FIVE_CU_DEVNET_OPT_IN").ok().as_deref() != Some("1") {
            panic!(
                "devnet run blocked: set FIVE_CU_DEVNET_OPT_IN=1 to run validator CU scenarios on devnet"
            );
        }
    }

    let scenarios = parse_scenarios();
    let mut results = Vec::new();

    for scenario in scenarios {
        let result = match scenario.as_str() {
            "external_non_cpi" => run_external_non_cpi(&mut harness, &repo_root),
            "external_interface_mapping_non_cpi" => {
                run_external_interface_mapping_non_cpi(&mut harness, &repo_root)
            }
            "external_burst_non_cpi" => run_external_burst_non_cpi(&mut harness, &repo_root),
            "token_full_e2e" => {
                let fixture = fixture_for("token_full_e2e", &repo_root).expect("fixture path");
                run_fixture_scenario(&mut harness, &repo_root, &fixture, "token_full_e2e", None)
            }
            "memory_string_heavy" => {
                let fixture = fixture_for("memory_string_heavy", &repo_root).expect("fixture path");
                let filter = [
                    "init_mint",
                    "init_token_account_user1",
                    "init_token_account_user2",
                    "init_token_account_user3",
                    "mint_to_user1",
                    "mint_to_user2",
                    "mint_to_user3",
                    "transfer_user2_to_user3",
                    "approve_user3_to_user2",
                    "transfer_from_user3_to_user1_by_user2",
                    "revoke_user3",
                    "burn_user1",
                    "freeze_user2",
                    "thaw_user2",
                    "disable_mint",
                ];
                run_fixture_scenario(
                    &mut harness,
                    &repo_root,
                    &fixture,
                    "memory_string_heavy",
                    Some(&filter),
                )
            }
            "arithmetic_intensive" => {
                let fixture = fixture_for("arithmetic_intensive", &repo_root).expect("fixture path");
                run_fixture_scenario(&mut harness, &repo_root, &fixture, "arithmetic_intensive", None)
            }
            _ => panic!("unsupported scenario `{}`", scenario),
        }
        .unwrap_or_else(|e| panic!("scenario `{}` failed: {}", scenario, e));

        print_summary_line(&result);
        results.push(result);
    }

    let output_path = std::env::var("FIVE_CU_RESULTS_FILE").ok().map(PathBuf::from);
    let report = harness
        .write_report(results, output_path)
        .unwrap_or_else(|e| panic!("failed writing validator run report: {}", e));
    println!("BPF_CU validator report={}", report.display());
}
