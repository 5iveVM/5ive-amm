mod harness;

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use five::instructions::{DEPLOY_INSTRUCTION, EXECUTE_INSTRUCTION};
use five::state::{FIVEVMState, ScriptAccountHeader};
use five_protocol::opcodes::HALT;
use harness::compile::load_or_compile_bytecode;
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
struct RuntimeFixture {
    name: String,
    bytecode_path: String,
    permissions: u8,
    authority: AuthorityFixture,
    vm_state_name: String,
    script_name: String,
    #[serde(default)]
    vm_fees: Option<FeeFixture>,
    #[serde(default)]
    extra_accounts: Vec<AccountFixture>,
    steps: Vec<StepFixture>,
}

#[derive(Debug, Deserialize)]
struct AuthorityFixture {
    name: String,
    #[serde(default = "default_authority_lamports")]
    lamports: u64,
}

#[derive(Debug, Deserialize)]
struct FeeFixture {
    deploy_fee_bps: u32,
    execute_fee_bps: u32,
}

#[derive(Debug, Deserialize)]
struct AccountFixture {
    name: String,
    owner: AccountOwner,
    #[serde(default)]
    lamports: u64,
    #[serde(default)]
    data_len: usize,
    #[serde(default)]
    is_signer: bool,
    #[serde(default)]
    is_writable: bool,
    #[serde(default)]
    executable: bool,
}

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
enum AccountOwner {
    Program,
    System,
    Authority,
    SelfAccount,
}

#[derive(Debug, Deserialize)]
struct StepFixture {
    name: String,
    function_index: u32,
    #[serde(default)]
    extras: Vec<String>,
    #[serde(default)]
    params: Vec<ParamFixture>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum ParamFixture {
    AccountRef { account: String },
    U8 { value: u8 },
    U64 { value: u64 },
    Bool { value: bool },
    String { value: String },
    PubkeyAccount { account: String },
    AccountIndex { value: u8 },
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

#[tokio::test(flavor = "multi_thread")]
async fn token_e2e_bpf_compute_units() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let bpf_dir = repo_root.join("target/deploy");
    std::env::set_var("BPF_OUT_DIR", &bpf_dir);

    let fixture_path = repo_root.join("five-templates/token/runtime-fixtures/init_mint.json");
    let fixture = load_fixture(&fixture_path);
    let bytecode_path = resolve_bytecode_path(&repo_root, &fixture_path, &fixture.bytecode_path);
    let bytecode = load_or_compile_bytecode(&bytecode_path)
        .unwrap_or_else(|e| panic!("failed loading bytecode {}: {}", bytecode_path.display(), e));

    let program_id = read_keypair_file(bpf_dir.join("five-keypair.json"))
        .expect("missing target/deploy/five-keypair.json; run `cargo-build-sbf --manifest-path five-solana/Cargo.toml`")
        .pubkey();

    let mut accounts = BTreeMap::<String, RuntimeAccount>::new();

    let authority_signer = Keypair::new();
    let authority_pubkey = authority_signer.pubkey();
    accounts.insert(
        fixture.authority.name.clone(),
        RuntimeAccount {
            pubkey: authority_pubkey,
            signer: Some(authority_signer),
            owner: system_program::id(),
            lamports: fixture.authority.lamports,
            data: vec![],
            is_signer: true,
            is_writable: true,
            executable: false,
        },
    );

    let mut vm_state_data = vec![0u8; FIVEVMState::LEN];
    {
        let vm_state = FIVEVMState::from_account_data_mut(&mut vm_state_data)
            .expect("invalid vm state account layout");
        vm_state.initialize(authority_pubkey.to_bytes());
        vm_state.deploy_fee_bps = 0;
        vm_state.execute_fee_bps = 0;
        if let Some(fees) = &fixture.vm_fees {
            vm_state.deploy_fee_bps = fees.deploy_fee_bps;
            vm_state.execute_fee_bps = fees.execute_fee_bps;
        }
    }
    accounts.insert(
        fixture.vm_state_name.clone(),
        RuntimeAccount {
            pubkey: Pubkey::new_unique(),
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
        fixture.script_name.clone(),
        RuntimeAccount {
            pubkey: Pubkey::new_unique(),
            signer: None,
            owner: program_id,
            lamports: Rent::default().minimum_balance(ScriptAccountHeader::LEN + bytecode.len()),
            data: vec![0u8; ScriptAccountHeader::LEN + bytecode.len()],
            is_signer: false,
            is_writable: true,
            executable: false,
        },
    );

    for extra in &fixture.extra_accounts {
        let pubkey = if extra.name == "system_program" {
            system_program::id()
        } else if extra.is_signer {
            Keypair::new().pubkey()
        } else {
            Pubkey::new_unique()
        };

        let signer = if extra.is_signer {
            let kp = Keypair::new();
            let pk = kp.pubkey();
            let owner = resolve_owner(extra.owner, program_id, authority_pubkey, pk);
            accounts.insert(
                extra.name.clone(),
                RuntimeAccount {
                    pubkey: pk,
                    signer: Some(kp),
                    owner,
                    lamports: extra.lamports,
                    data: vec![0u8; extra.data_len],
                    is_signer: true,
                    is_writable: extra.is_writable,
                    executable: extra.executable,
                },
            );
            continue;
        } else {
            resolve_owner(extra.owner, program_id, authority_pubkey, pubkey)
        };

        accounts.insert(
            extra.name.clone(),
            RuntimeAccount {
                pubkey,
                signer: None,
                owner: signer,
                lamports: extra.lamports,
                data: vec![0u8; extra.data_len],
                is_signer: false,
                is_writable: extra.is_writable,
                executable: extra.executable,
            },
        );
    }

    let mut program_test = ProgramTest::new("five", program_id, None);
    program_test.prefer_bpf(true);
    for account in accounts.values() {
        if account.pubkey == program_id {
            panic!("fixture account collides with program id");
        }
        if account.pubkey == system_program::id() {
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

    let mut ctx = program_test.start_with_context().await;
    let deploy_ix = build_deploy_instruction(
        program_id,
        &accounts,
        &fixture.script_name,
        &fixture.vm_state_name,
        &fixture.authority.name,
        &bytecode,
        fixture.permissions,
    );
    let deploy_signers = collect_signers(&accounts, &[fixture.authority.name.as_str()]);
    let deploy_result =
        simulate_and_process(&mut ctx, vec![deploy_ix], deploy_signers, Some(1_400_000)).await;
    assert!(deploy_result.success, "deploy failed: {:?}", deploy_result.error);
    println!("BPF_CU deploy={}", deploy_result.units_consumed);

    let mut total_units = deploy_result.units_consumed;
    for step in &fixture.steps {
        let payload = build_payload(&accounts, step);
        let execute_ix = build_execute_instruction(
            program_id,
            &accounts,
            &fixture.script_name,
            &fixture.vm_state_name,
            step,
            payload,
        );

        let signer_names: Vec<&str> = step
            .extras
            .iter()
            .filter_map(|name| {
                accounts
                    .get(name)
                    .and_then(|a| if a.is_signer { Some(name.as_str()) } else { None })
            })
            .collect();

        let result = simulate_and_process(
            &mut ctx,
            vec![execute_ix],
            collect_signers(&accounts, &signer_names),
            None,
        )
        .await;
        assert!(result.success, "step {} failed: {:?}", step.name, result.error);
        total_units = total_units.saturating_add(result.units_consumed);
        println!("BPF_CU step={} units={}", step.name, result.units_consumed);
    }

    println!("BPF_CU fixture={} total_units={}", fixture.name, total_units);
}

#[tokio::test(flavor = "multi_thread")]
async fn minimal_execute_floor_bpf_compute_units() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let bpf_dir = repo_root.join("target/deploy");
    std::env::set_var("BPF_OUT_DIR", &bpf_dir);

    let program_id = read_keypair_file(bpf_dir.join("five-keypair.json"))
        .expect("missing target/deploy/five-keypair.json; run cargo-build-sbf first")
        .pubkey();

    let mut accounts = BTreeMap::<String, RuntimeAccount>::new();
    let authority_signer = Keypair::new();
    let authority_pubkey = authority_signer.pubkey();
    accounts.insert(
        "payer".to_string(),
        RuntimeAccount {
            pubkey: authority_pubkey,
            signer: Some(authority_signer),
            owner: system_program::id(),
            lamports: 20_000_000,
            data: vec![],
            is_signer: true,
            is_writable: true,
            executable: false,
        },
    );

    let mut vm_state_data = vec![0u8; FIVEVMState::LEN];
    {
        let vm_state = FIVEVMState::from_account_data_mut(&mut vm_state_data).unwrap();
        vm_state.initialize(authority_pubkey.to_bytes());
        vm_state.deploy_fee_bps = 0;
        vm_state.execute_fee_bps = 0;
    }
    accounts.insert(
        "vm_state".to_string(),
        RuntimeAccount {
            pubkey: Pubkey::new_unique(),
            signer: None,
            owner: program_id,
            lamports: Rent::default().minimum_balance(FIVEVMState::LEN),
            data: vm_state_data,
            is_signer: false,
            is_writable: true,
            executable: false,
        },
    );

    let bytecode = {
        let mut b = vec![b'5', b'I', b'V', b'E', 0, 0, 0, 0, 1, 1];
        b.push(HALT);
        b
    };
    accounts.insert(
        "script".to_string(),
        RuntimeAccount {
            pubkey: Pubkey::new_unique(),
            signer: None,
            owner: program_id,
            lamports: Rent::default().minimum_balance(ScriptAccountHeader::LEN + bytecode.len()),
            data: vec![0u8; ScriptAccountHeader::LEN + bytecode.len()],
            is_signer: false,
            is_writable: true,
            executable: false,
        },
    );

    let mut program_test = ProgramTest::new("five", program_id, None);
    program_test.prefer_bpf(true);
    for account in accounts.values() {
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
    let mut ctx = program_test.start_with_context().await;

    let deploy_ix = build_deploy_instruction(
        program_id,
        &accounts,
        "script",
        "vm_state",
        "payer",
        &bytecode,
        0,
    );
    let deploy_result = simulate_and_process(
        &mut ctx,
        vec![deploy_ix],
        collect_signers(&accounts, &["payer"]),
        Some(1_400_000),
    )
    .await;
    assert!(deploy_result.success, "minimal deploy failed: {:?}", deploy_result.error);

    let payload = canonical_execute_payload(0, &[]);
    let execute_ix = build_execute_instruction(
        program_id,
        &accounts,
        "script",
        "vm_state",
        &StepFixture {
            name: "halt".to_string(),
            function_index: 0,
            extras: vec!["payer".to_string()],
            params: vec![],
        },
        payload,
    );
    let execute_result = simulate_and_process(
        &mut ctx,
        vec![execute_ix],
        collect_signers(&accounts, &["payer"]),
        None,
    )
    .await;
    assert!(
        execute_result.success,
        "minimal execute failed: {:?}",
        execute_result.error
    );

    println!(
        "BPF_CU minimal_execute_floor={}",
        execute_result.units_consumed
    );
}

fn resolve_owner(owner: AccountOwner, program_id: Pubkey, authority: Pubkey, self_key: Pubkey) -> Pubkey {
    match owner {
        AccountOwner::Program => program_id,
        AccountOwner::System => system_program::id(),
        AccountOwner::Authority => authority,
        AccountOwner::SelfAccount => self_key,
    }
}

fn build_deploy_instruction(
    program_id: Pubkey,
    accounts: &BTreeMap<String, RuntimeAccount>,
    script_name: &str,
    vm_state_name: &str,
    owner_name: &str,
    bytecode: &[u8],
    permissions: u8,
) -> Instruction {
    let mut data = Vec::with_capacity(6 + bytecode.len());
    data.push(DEPLOY_INSTRUCTION);
    data.extend_from_slice(&(bytecode.len() as u32).to_le_bytes());
    data.push(permissions);
    data.extend_from_slice(bytecode);

    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(accounts[script_name].pubkey, false),
            AccountMeta::new(accounts[vm_state_name].pubkey, false),
            AccountMeta::new_readonly(accounts[owner_name].pubkey, true),
        ],
        data,
    }
}

fn build_execute_instruction(
    program_id: Pubkey,
    accounts: &BTreeMap<String, RuntimeAccount>,
    script_name: &str,
    vm_state_name: &str,
    step: &StepFixture,
    payload: Vec<u8>,
) -> Instruction {
    let mut data = Vec::with_capacity(1 + payload.len());
    data.push(EXECUTE_INSTRUCTION);
    data.extend_from_slice(&payload);

    let mut metas = vec![
        AccountMeta::new(accounts[script_name].pubkey, false),
        AccountMeta::new(accounts[vm_state_name].pubkey, false),
    ];
    for name in &step.extras {
        let a = &accounts[name];
        metas.push(AccountMeta {
            pubkey: a.pubkey,
            is_signer: a.is_signer,
            is_writable: a.is_writable,
        });
    }

    Instruction {
        program_id,
        accounts: metas,
        data,
    }
}

fn build_payload(accounts: &BTreeMap<String, RuntimeAccount>, step: &StepFixture) -> Vec<u8> {
    let mut params = Vec::with_capacity(step.params.len());
    for param in &step.params {
        match param {
            ParamFixture::AccountRef { account } => {
                let idx = resolve_account_ref_index(step, account);
                params.push(TypedParam::Account(idx));
            }
            ParamFixture::U8 { value } => params.push(TypedParam::U8(*value)),
            ParamFixture::U64 { value } => params.push(TypedParam::U64(*value)),
            ParamFixture::Bool { value } => params.push(TypedParam::Bool(*value)),
            ParamFixture::String { value } => params.push(TypedParam::String(value.clone())),
            ParamFixture::PubkeyAccount { account } => {
                params.push(TypedParam::Pubkey(accounts[account].pubkey.to_bytes()));
            }
            ParamFixture::AccountIndex { value } => params.push(TypedParam::Account(*value)),
        }
    }
    canonical_execute_payload(step.function_index, &params)
}

fn resolve_account_ref_index(step: &StepFixture, account: &str) -> u8 {
    let pos = step
        .extras
        .iter()
        .position(|name| name == account)
        .unwrap_or_else(|| panic!("account `{}` not found in extras {:?}", account, step.extras));
    (pos as u8) + 1
}

fn collect_signers<'a>(accounts: &'a BTreeMap<String, RuntimeAccount>, names: &[&str]) -> Vec<&'a Keypair> {
    let mut out = Vec::new();
    for name in names {
        if let Some(kp) = accounts[*name].signer.as_ref() {
            out.push(kp);
        }
    }
    out
}

struct TxOutcome {
    success: bool,
    units_consumed: u64,
    error: Option<String>,
}

async fn simulate_and_process(
    ctx: &mut ProgramTestContext,
    instructions: Vec<Instruction>,
    extra_signers: Vec<&Keypair>,
    cu_limit: Option<u32>,
) -> TxOutcome {
    let mut all_instructions = Vec::with_capacity(instructions.len() + 1);
    if let Some(limit) = cu_limit {
        all_instructions.push(ComputeBudgetInstruction::set_compute_unit_limit(limit));
    }
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

    let simulation = ctx.banks_client.simulate_transaction(tx.clone()).await;
    let (simulated_units, sim_logs) = match simulation {
        Ok(sim_result) => {
            let units = sim_result
                .simulation_details
                .as_ref()
                .map(|d| d.units_consumed)
                .unwrap_or(0);
            let logs = sim_result
                .simulation_details
                .as_ref()
                .map(|d| d.logs.clone())
                .unwrap_or_default();
            (units, logs)
        }
        Err(err) => {
            return TxOutcome {
                success: false,
                units_consumed: 0,
                error: Some(format!("simulate failed: {}", err)),
            };
        }
    };

    match ctx.banks_client.process_transaction(tx).await {
        Ok(()) => TxOutcome {
            success: true,
            units_consumed: simulated_units,
            error: None,
        },
        Err(err) => {
            for log in &sim_logs {
                println!("SIM_LOG {}", log);
            }
            TxOutcome {
                success: false,
                units_consumed: simulated_units,
                error: Some(err.to_string()),
            }
        }
    }
}

fn default_authority_lamports() -> u64 {
    200_000
}

fn load_fixture(path: &Path) -> RuntimeFixture {
    let raw = fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("failed reading fixture {}: {}", path.display(), e));
    serde_json::from_str(&raw)
        .unwrap_or_else(|e| panic!("failed parsing fixture {}: {}", path.display(), e))
}

fn resolve_bytecode_path(repo_root: &Path, fixture_path: &Path, configured_path: &str) -> PathBuf {
    let configured = PathBuf::from(configured_path);
    if configured.is_absolute() {
        return configured;
    }

    let rel_to_fixture = fixture_path
        .parent()
        .expect("fixture should have a parent")
        .join(&configured);
    if rel_to_fixture.exists() {
        return rel_to_fixture;
    }

    repo_root.join(configured)
}
