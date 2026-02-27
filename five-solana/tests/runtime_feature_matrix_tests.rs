//! Manifest-backed runtime matrix.
//!
//! Generic scenarios execute through ProgramTest when the compiled BPF artifact
//! is available. Template-backed scenarios validate referenced runtime fixtures
//! so complex account/CPI coverage stays wired into the shared matrix.

mod harness;

use std::{collections::BTreeMap, fs, path::PathBuf};

use five::instructions::{DEPLOY_INSTRUCTION, EXECUTE_INSTRUCTION};
use five::state::{FIVEVMState, ScriptAccountHeader};
use five_dsl_compiler::DslCompiler;
use harness::addresses::{canonical_execute_fee_header, fee_vault_shard0_pda, vm_state_pda};
use harness::fixtures::{canonical_execute_payload, TypedParam};
use serde::Deserialize;
use solana_program_test::{ProgramTest, ProgramTestContext};
use solana_sdk::{
    account::Account,
    compute_budget::ComputeBudgetInstruction,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    rent::Rent,
    signature::{read_keypair_file, Keypair, Signer},
    system_program,
    transaction::Transaction,
};

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
    layers: Layers,
    runtime_mode: String,
    runtime_fixture: Option<String>,
    requires_accounts: bool,
    requires_cpi: bool,
}

#[derive(Debug, Deserialize)]
struct Layers {
    solana_runtime: bool,
}

#[derive(Debug)]
struct RuntimeAccount {
    pubkey: Pubkey,
    signer: Option<Keypair>,
    owner: Pubkey,
    lamports: u64,
    data: Vec<u8>,
    is_signer: bool,
    is_writable: bool,
    executable: bool,
}

#[derive(Debug)]
struct TxOutcome {
    success: bool,
    error: Option<String>,
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

fn typed_params(params: &[serde_json::Value]) -> Vec<TypedParam> {
    params
        .iter()
        .map(|param| match param {
            serde_json::Value::Number(number) => {
                TypedParam::U64(number.as_u64().expect("runtime matrix numeric param must be u64"))
            }
            serde_json::Value::Bool(value) => TypedParam::Bool(*value),
            other => panic!("unsupported runtime matrix param: {}", other),
        })
        .collect()
}

fn load_program_id() -> Option<Pubkey> {
    let root = repo_root();
    let bpf_dir = root.join("target/deploy");
    let keypair = bpf_dir.join("five-keypair.json");
    let program = bpf_dir.join("five.so");
    if !keypair.exists() || !program.exists() {
        eprintln!(
            "SKIP runtime_feature_matrix_generic_executes_manifest_scenarios: build BPF first with cargo-build-sbf --manifest-path five-solana/Cargo.toml"
        );
        return None;
    }
    std::env::set_var("BPF_OUT_DIR", &bpf_dir);
    Some(
        read_keypair_file(keypair)
            .expect("read five-keypair.json")
            .pubkey(),
    )
}

fn base_accounts(program_id: Pubkey) -> BTreeMap<String, RuntimeAccount> {
    let mut accounts = BTreeMap::<String, RuntimeAccount>::new();
    let owner_signer = Keypair::new();
    let owner_pubkey = owner_signer.pubkey();
    accounts.insert(
        "owner".to_string(),
        RuntimeAccount {
            pubkey: owner_pubkey,
            signer: Some(owner_signer),
            owner: system_program::id(),
            lamports: 20_000_000,
            data: vec![],
            is_signer: true,
            is_writable: true,
            executable: false,
        },
    );

    let (vm_state_pubkey, vm_state_bump) = vm_state_pda(&program_id);
    let (fee_vault_pubkey, _fee_vault_bump) = fee_vault_shard0_pda(&program_id);

    let mut vm_state_data = vec![0u8; FIVEVMState::LEN];
    {
        let vm_state = FIVEVMState::from_account_data_mut(&mut vm_state_data)
            .expect("invalid vm state layout");
        vm_state.initialize(owner_pubkey.to_bytes(), vm_state_bump);
        vm_state.deploy_fee_lamports = 0;
        vm_state.execute_fee_lamports = 0;
    }

    accounts.insert(
        "vm_state".to_string(),
        RuntimeAccount {
            pubkey: vm_state_pubkey,
            signer: None,
            owner: program_id,
            lamports: Rent::default().minimum_balance(FIVEVMState::LEN),
            data: vm_state_data,
            is_signer: false,
            is_writable: true,
            executable: false,
        },
    );
    accounts.insert(
        "fee_vault".to_string(),
        RuntimeAccount {
            pubkey: fee_vault_pubkey,
            signer: None,
            owner: program_id,
            lamports: Rent::default().minimum_balance(0),
            data: vec![],
            is_signer: false,
            is_writable: true,
            executable: false,
        },
    );

    accounts
}

async fn start_context(
    program_id: Pubkey,
    accounts: &BTreeMap<String, RuntimeAccount>,
) -> ProgramTestContext {
    let mut program_test = ProgramTest::new("five", program_id, None);
    program_test.prefer_bpf(true);

    for account in accounts.values() {
        if account.pubkey == program_id || account.pubkey == system_program::id() {
            continue;
        }
        program_test.add_account(
            account.pubkey,
            Account {
                lamports: account.lamports,
                data: account.data.clone(),
                owner: account.owner,
                executable: account.executable,
                rent_epoch: 0,
            },
        );
    }

    program_test.start_with_context().await
}

fn build_deploy_instruction(
    program_id: Pubkey,
    accounts: &BTreeMap<String, RuntimeAccount>,
    bytecode: &[u8],
) -> Instruction {
    let mut data = Vec::with_capacity(10 + bytecode.len());
    data.push(DEPLOY_INSTRUCTION);
    data.extend_from_slice(&(bytecode.len() as u32).to_le_bytes());
    data.push(0);
    data.extend_from_slice(&0u32.to_le_bytes());
    data.extend_from_slice(bytecode);
    data.push(0);

    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(accounts["script"].pubkey, false),
            AccountMeta::new(accounts["vm_state"].pubkey, false),
            AccountMeta::new(accounts["owner"].pubkey, true),
            AccountMeta::new(accounts["fee_vault"].pubkey, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data,
    }
}

fn build_execute_instruction(
    program_id: Pubkey,
    accounts: &BTreeMap<String, RuntimeAccount>,
    payload: Vec<u8>,
) -> Instruction {
    let mut data = Vec::with_capacity(4 + payload.len());
    data.push(EXECUTE_INSTRUCTION);
    data.extend_from_slice(&canonical_execute_fee_header(0));
    data.extend_from_slice(&payload);

    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(accounts["script"].pubkey, false),
            AccountMeta::new(accounts["vm_state"].pubkey, false),
            AccountMeta::new(accounts["owner"].pubkey, true),
            AccountMeta::new(accounts["fee_vault"].pubkey, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data,
    }
}

fn collect_signers<'a>(
    accounts: &'a BTreeMap<String, RuntimeAccount>,
    names: &[&str],
) -> Vec<&'a Keypair> {
    names
        .iter()
        .filter_map(|name| accounts[*name].signer.as_ref())
        .collect()
}

async fn simulate_and_process(
    ctx: &mut ProgramTestContext,
    instructions: Vec<Instruction>,
    extra_signers: Vec<&Keypair>,
) -> TxOutcome {
    let mut all_instructions = Vec::with_capacity(instructions.len() + 1);
    all_instructions.push(ComputeBudgetInstruction::set_compute_unit_limit(1_400_000));
    all_instructions.extend(instructions);

    let mut signers: Vec<&Keypair> = Vec::with_capacity(1 + extra_signers.len());
    signers.push(&ctx.payer);
    signers.extend(extra_signers);

    let tx = Transaction::new_signed_with_payer(
        &all_instructions,
        Some(&ctx.payer.pubkey()),
        &signers,
        ctx.last_blockhash,
    );

    match ctx.banks_client.process_transaction(tx).await {
        Ok(()) => TxOutcome {
            success: true,
            error: None,
        },
        Err(error) => TxOutcome {
            success: false,
            error: Some(error.to_string()),
        },
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn runtime_feature_matrix_generic_executes_manifest_scenarios() {
    let Some(program_id) = load_program_id() else {
        return;
    };

    let root = repo_root();
    let matrix = load_matrix();

    for scenario in matrix.scenarios.iter().filter(|scenario| {
        scenario.layers.solana_runtime
            && scenario.runtime_mode == "generic"
            && scenario.kind == "positive"
            && !scenario.requires_accounts
            && !scenario.requires_cpi
    }) {
        let source = fs::read_to_string(source_path(&root, scenario)).expect("read matrix source");
        let bytecode = DslCompiler::compile_dsl(&source)
            .unwrap_or_else(|error| panic!("scenario {} failed to compile: {}", scenario.id, error));
        let params = scenario_params(&source, scenario);
        let typed = typed_params(&params);
        let payload = canonical_execute_payload(scenario.function.unwrap_or(0), &typed);

        let mut accounts = base_accounts(program_id);
        accounts.insert(
            "script".to_string(),
            RuntimeAccount {
                pubkey: Pubkey::new_unique(),
                signer: None,
                owner: program_id,
                lamports: Rent::default()
                    .minimum_balance(ScriptAccountHeader::LEN + bytecode.len()),
                data: vec![0u8; ScriptAccountHeader::LEN + bytecode.len()],
                is_signer: false,
                is_writable: true,
                executable: false,
            },
        );

        let mut ctx = start_context(program_id, &accounts).await;
        let deploy = simulate_and_process(
            &mut ctx,
            vec![build_deploy_instruction(program_id, &accounts, &bytecode)],
            collect_signers(&accounts, &["owner"]),
        )
        .await;
        assert!(
            deploy.success,
            "scenario {} deploy failed: {:?}",
            scenario.id,
            deploy.error
        );

        let execute = simulate_and_process(
            &mut ctx,
            vec![build_execute_instruction(program_id, &accounts, payload)],
            collect_signers(&accounts, &["owner"]),
        )
        .await;
        assert!(
            execute.success,
            "scenario {} execute failed: {:?}",
            scenario.id,
            execute.error
        );
    }
}

#[test]
fn runtime_feature_matrix_template_fixtures_exist_and_parse() {
    let root = repo_root();
    let matrix = load_matrix();

    for scenario in matrix.scenarios.iter().filter(|scenario| {
        scenario.layers.solana_runtime && scenario.runtime_mode == "template_fixture"
    }) {
        let fixture = root.join(
            scenario
                .runtime_fixture
                .as_ref()
                .expect("template_fixture scenario runtime_fixture"),
        );
        let content = fs::read_to_string(&fixture)
            .unwrap_or_else(|error| panic!("scenario {} missing fixture {}: {}", scenario.id, fixture.display(), error));
        serde_json::from_str::<serde_json::Value>(&content)
            .unwrap_or_else(|error| panic!("scenario {} fixture {} invalid JSON: {}", scenario.id, fixture.display(), error));
    }
}
