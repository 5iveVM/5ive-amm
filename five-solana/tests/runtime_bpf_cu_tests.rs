mod harness;

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};

use five::instructions::{DEPLOY_INSTRUCTION, EXECUTE_INSTRUCTION};
use five::state::{FIVEVMState, ScriptAccountHeader};
use five_protocol::opcodes::HALT;
use harness::compile::load_or_compile_bytecode;
use harness::fixtures::{canonical_execute_payload, TypedParam};
use serde::Deserialize;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError, program_option::COption, pubkey::Pubkey as ProgramPubkey,
};
use solana_program_test::{ProgramTest, ProgramTestContext};
use solana_sdk::{
    account::Account,
    compute_budget::ComputeBudgetInstruction,
    instruction::{AccountMeta, Instruction},
    program_pack::Pack,
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
    #[serde(default)]
    skip_deploy: bool,
    authority: AuthorityFixture,
    vm_state_name: String,
    script_name: String,
    #[serde(default)]
    vm_fees: Option<FeeFixture>,
    #[serde(default)]
    extra_accounts: Vec<AccountFixture>,
    #[serde(default)]
    external_programs: Vec<ExternalProgramFixture>,
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
    #[serde(default)]
    pubkey: Option<String>,
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
    SplTokenProgram,
    AnchorTokenProgram,
}

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
enum ExternalProgramKind {
    SplToken,
    AnchorTokenComparisonStub,
    AnchorTokenComparison,
}

#[derive(Debug, Deserialize)]
struct ExternalProgramFixture {
    kind: ExternalProgramKind,
}

#[derive(Debug, Deserialize)]
struct StepFixture {
    name: String,
    function_index: u32,
    #[serde(default)]
    extras: Vec<String>,
    #[serde(default)]
    params: Vec<ParamFixture>,
    #[serde(default = "default_expected_fixture")]
    expected: ExpectedFixture,
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum ExpectedFixture {
    Success,
    Error,
    SuccessOrError,
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
    let fixture_path = fixture_path_from_env(&repo_root);
    run_fixture_bpf_compute_units(&repo_root, &fixture_path, None).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn spl_token_interface_cpi_bpf_compute_units() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let fixture_path = repo_root.join("five-templates/cpi-examples/runtime-fixtures/spl-token-mint-e2e.json");
    run_fixture_bpf_compute_units(&repo_root, &fixture_path, Some(120_000)).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn anchor_interface_cpi_bpf_compute_units() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let fixture_path = repo_root.join("five-templates/cpi-examples/runtime-fixtures/anchor-program-call-e2e.json");
    run_fixture_bpf_compute_units(&repo_root, &fixture_path, Some(120_000)).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn anchor_interface_manual_borsh_cpi_bpf_compute_units() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let fixture_path =
        repo_root.join("five-templates/cpi-examples/runtime-fixtures/anchor-program-call-e2e-manual.json");
    run_fixture_bpf_compute_units(&repo_root, &fixture_path, Some(120_000)).await;
}

async fn run_fixture_bpf_compute_units(
    repo_root: &Path,
    fixture_path: &Path,
    total_budget_override: Option<u64>,
) {
    let bpf_dir = repo_root.join("target/deploy");
    std::env::set_var("BPF_OUT_DIR", &bpf_dir);

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

    if fixture.skip_deploy {
        let mut script_data = vec![0u8; ScriptAccountHeader::LEN + bytecode.len()];
        let header = ScriptAccountHeader::create_from_bytecode(
            &bytecode,
            authority_pubkey.to_bytes(),
            1,
            fixture.permissions,
        );
        header
            .copy_into_account(&mut script_data)
            .expect("failed writing predeployed script header");
        script_data[ScriptAccountHeader::LEN..ScriptAccountHeader::LEN + bytecode.len()]
            .copy_from_slice(&bytecode);
        if let Some(script_account) = accounts.get_mut(&fixture.script_name) {
            script_account.data = script_data;
            script_account.lamports =
                Rent::default().minimum_balance(script_account.data.len());
        }
    }

    for extra in &fixture.extra_accounts {
        let pubkey = if let Some(pubkey_str) = &extra.pubkey {
            Pubkey::from_str(pubkey_str)
                .unwrap_or_else(|_| panic!("invalid pubkey '{}' for fixture account '{}'", pubkey_str, extra.name))
        } else if extra.name == "system_program" {
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
    if fixture.name == "anchor_token_cpi_e2e" || fixture.name == "anchor_token_cpi_e2e_manual" {
        seed_anchor_token_accounts(&mut accounts, &fixture.authority.name);
    }

    let mut program_test = ProgramTest::new("five", program_id, None);
    // Register external CPI target processors as builtins first.
    program_test.prefer_bpf(false);
    register_external_programs(&mut program_test, &fixture.external_programs, &bpf_dir);
    let external_program_ids = external_program_ids(&fixture.external_programs);
    for account in accounts.values() {
        if account.pubkey == program_id {
            panic!("fixture account collides with program id");
        }
        if account.pubkey == system_program::id() {
            continue;
        }
        if external_program_ids.contains(&account.pubkey) {
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
    let mut total_units = 0u64;
    if !fixture.skip_deploy {
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
        total_units = deploy_result.units_consumed;
    }
    if fixture.name == "spl_token_cpi_e2e" {
        let setup_units = initialize_spl_token_accounts(&mut ctx, &accounts, &fixture.authority.name).await;
        total_units = total_units.saturating_add(setup_units);
        println!("BPF_CU spl_setup={}", setup_units);
    }
    // Regression guardrails (BPF CU). Tighten over time.
    let step_budget = |name: &str| -> u64 {
        match name {
            "init_mint" => 10_000,
            "init_token_account_user1" | "init_token_account_user2" | "init_token_account_user3" => 8_500,
            "mint_to_user1" | "mint_to_user2" | "mint_to_user3" => 7_000,
            "transfer_user2_to_user3" => 7_000,
            "approve_user3_to_user2" => 7_000,
            "transfer_from_user3_to_user1_by_user2" => 8_000,
            "revoke_user3" => 7_000,
            "burn_user1" => 8_000,
            "freeze_user2" | "thaw_user2" => 8_000,
            "anchor_mint_to_user1" | "anchor_mint_to_user2" | "anchor_mint_to_user3" => 12_000,
            "anchor_transfer_user2_to_user3" => 12_000,
            "anchor_approve_user3_to_user2" => 12_000,
            "anchor_transfer_from_user3_to_user1_by_user2" => 12_000,
            "anchor_revoke_user3" => 12_000,
            "anchor_burn_user1" => 12_000,
            "anchor_freeze_user2" | "anchor_thaw_user2" => 12_000,
            "disable_mint" => 6_200,
            "stable_swap_invariant_iterative" => 35_000,
            "utilization_kink_rate" => 8_000,
            "funding_rate_path" => 18_000,
            "collateral_health_loop" => 16_000,
            "anchor_increment" => 25_000,
            _ => 12_000,
        }
    };
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
        match step.expected {
            ExpectedFixture::Success => {
                assert!(result.success, "step {} failed: {:?}", step.name, result.error);
            }
            ExpectedFixture::Error => {
                assert!(
                    !result.success,
                    "step {} expected deterministic error but succeeded",
                    step.name
                );
            }
            ExpectedFixture::SuccessOrError => {}
        }
        if step.expected != ExpectedFixture::Error {
            assert!(
                result.units_consumed <= step_budget(&step.name),
                "step {} consumed {} CU above budget {}",
                step.name,
                result.units_consumed,
                step_budget(&step.name)
            );
        }
        total_units = total_units.saturating_add(result.units_consumed);
        println!(
            "BPF_CU step={} expected={:?} success={} units={}",
            step.name,
            step.expected,
            result.success,
            result.units_consumed
        );
    }

    println!("BPF_CU fixture={} total_units={}", fixture.name, total_units);
    if fixture.name == "spl_token_cpi_e2e" {
        assert_spl_token_fixture_result(&mut ctx, &accounts, &fixture.authority.name).await;
    }
    if fixture.name == "anchor_token_cpi_e2e" {
        assert_anchor_token_fixture_result(&mut ctx, &accounts, &fixture.authority.name).await;
    }
    if fixture.name == "anchor_token_cpi_e2e_manual" {
        assert_anchor_token_manual_fixture_result(&mut ctx, &accounts, &fixture.authority.name).await;
    }
    let total_budget = total_budget_override.unwrap_or_else(|| {
        if fixture.name == "token_full_e2e" {
            480_000
        } else {
            700_000
        }
    });
    assert!(
        total_units <= total_budget,
        "fixture total {} exceeds regression budget",
        total_units
    );
}

fn register_external_programs(
    program_test: &mut ProgramTest,
    external_programs: &[ExternalProgramFixture],
    bpf_dir: &Path,
) {
    for external in external_programs {
        match external.kind {
            ExternalProgramKind::SplToken => {
                program_test.add_program(
                    "spl_token",
                    spl_token::id(),
                    solana_program_test::processor!(spl_token::processor::Processor::process),
                );
            }
            ExternalProgramKind::AnchorTokenComparisonStub => {
                let anchor_program_id = anchor_token_program_id();
                program_test.add_program(
                    "anchor_token_comparison_stub",
                    anchor_program_id,
                    solana_program_test::processor!(anchor_token_comparison_stub_process),
                );
            }
            ExternalProgramKind::AnchorTokenComparison => {
                let anchor_program_id = anchor_token_program_id();
                let so_path = bpf_dir.join("anchor_token_comparison.so");
                let data = fs::read(&so_path).unwrap_or_else(|e| {
                    panic!(
                        "missing {} ({}). Build with: cargo-build-sbf --manifest-path five-templates/anchor-token-comparison/programs/anchor-token-comparison/Cargo.toml --sbf-out-dir target/deploy",
                        so_path.display(),
                        e
                    )
                });
                program_test.add_account(
                    anchor_program_id,
                    Account {
                        lamports: Rent::default().minimum_balance(data.len()).max(1),
                        data,
                        owner: solana_sdk::bpf_loader::id(),
                        executable: true,
                        rent_epoch: 0,
                    },
                );
            }
        }
    }
}

fn external_program_ids(external_programs: &[ExternalProgramFixture]) -> Vec<Pubkey> {
    external_programs
        .iter()
        .map(|external| match external.kind {
            ExternalProgramKind::SplToken => spl_token::id(),
            ExternalProgramKind::AnchorTokenComparisonStub => anchor_token_program_id(),
            ExternalProgramKind::AnchorTokenComparison => anchor_token_program_id(),
        })
        .collect()
}

fn anchor_token_comparison_stub_process(
    _program_id: &ProgramPubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    if accounts.len() < 2 {
        return Err(ProgramError::NotEnoughAccountKeys);
    }
    if !accounts[1].is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if !accounts[0].is_writable {
        return Err(ProgramError::InvalidAccountData);
    }

    const DISC: [u8; 8] = [0xAA, 0x12, 0x34, 0x56, 0x78, 0xAB, 0xCD, 0xEF];
    if instruction_data.len() != 16 {
        return Err(ProgramError::InvalidInstructionData);
    }
    if instruction_data[0..8] != DISC {
        return Err(ProgramError::InvalidInstructionData);
    }

    let amount = u64::from_le_bytes(
        instruction_data[8..16]
            .try_into()
            .map_err(|_| ProgramError::InvalidInstructionData)?,
    );
    let mut counter_data = accounts[0].try_borrow_mut_data()?;
    if counter_data.len() < 8 {
        return Err(ProgramError::InvalidAccountData);
    }
    let current = u64::from_le_bytes(
        counter_data[0..8]
            .try_into()
            .map_err(|_| ProgramError::InvalidAccountData)?,
    );
    let next = current
        .checked_add(amount)
        .ok_or(ProgramError::InvalidInstructionData)?;
    counter_data[0..8].copy_from_slice(&next.to_le_bytes());
    Ok(())
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
            expected: ExpectedFixture::Success,
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
    assert!(
        execute_result.units_consumed <= 1_200,
        "minimal floor regressed to {}",
        execute_result.units_consumed
    );
}

fn resolve_owner(owner: AccountOwner, program_id: Pubkey, authority: Pubkey, self_key: Pubkey) -> Pubkey {
    match owner {
        AccountOwner::Program => program_id,
        AccountOwner::System => system_program::id(),
        AccountOwner::Authority => authority,
        AccountOwner::SelfAccount => self_key,
        AccountOwner::SplTokenProgram => spl_token::id(),
        AccountOwner::AnchorTokenProgram => anchor_token_program_id(),
    }
}

fn anchor_token_program_id() -> Pubkey {
    Pubkey::from_str("EXYTTMwHkRziMdQ1guGGrThxzX6dJDvhJBzz57JGKmsw")
        .expect("invalid anchor token program id")
}

async fn initialize_spl_token_accounts(
    ctx: &mut ProgramTestContext,
    accounts: &BTreeMap<String, RuntimeAccount>,
    authority_name: &str,
) -> u64 {
    let mint = accounts
        .get("mint")
        .unwrap_or_else(|| panic!("missing `mint` account for spl setup"))
        .pubkey;
    let authority = accounts
        .get(authority_name)
        .unwrap_or_else(|| panic!("missing authority account `{}` for spl setup", authority_name))
        .pubkey;

    let mut setup_ixs = vec![spl_token::instruction::initialize_mint2(
        &spl_token::id(),
        &mint,
        &authority,
        Some(&authority),
        0,
    )
    .expect("failed building initialize_mint2")];

    if let Some(dest) = accounts.get("dest") {
        setup_ixs.push(
            spl_token::instruction::initialize_account3(
                &spl_token::id(),
                &dest.pubkey,
                &mint,
                &authority,
            )
            .expect("failed building initialize_account3 for dest"),
        );
    }

    if let Some(user1_token) = accounts.get("user1_token") {
        setup_ixs.push(
            spl_token::instruction::initialize_account3(
                &spl_token::id(),
                &user1_token.pubkey,
                &mint,
                &authority,
            )
            .expect("failed building initialize_account3 for user1"),
        );
    }
    if let (Some(user2_token), Some(user2)) = (accounts.get("user2_token"), accounts.get("user2")) {
        setup_ixs.push(
            spl_token::instruction::initialize_account3(
                &spl_token::id(),
                &user2_token.pubkey,
                &mint,
                &user2.pubkey,
            )
            .expect("failed building initialize_account3 for user2"),
        );
    }
    if let (Some(user3_token), Some(user3)) = (accounts.get("user3_token"), accounts.get("user3")) {
        setup_ixs.push(
            spl_token::instruction::initialize_account3(
                &spl_token::id(),
                &user3_token.pubkey,
                &mint,
                &user3.pubkey,
            )
            .expect("failed building initialize_account3 for user3"),
        );
    }

    let result = simulate_and_process(ctx, setup_ixs, vec![], None).await;
    assert!(
        result.success,
        "spl token setup failed: {:?}",
        result.error
    );
    result.units_consumed
}

async fn assert_spl_token_fixture_result(
    ctx: &mut ProgramTestContext,
    accounts: &BTreeMap<String, RuntimeAccount>,
    authority_name: &str,
) {
    let mint = accounts
        .get("mint")
        .unwrap_or_else(|| panic!("missing `mint` account for spl assertion"))
        .pubkey;
    let authority = accounts
        .get(authority_name)
        .unwrap_or_else(|| panic!("missing authority account `{}` for spl assertion", authority_name))
        .pubkey;

    let mint_account = ctx
        .banks_client
        .get_account(mint)
        .await
        .expect("fetch mint account")
        .expect("mint account not found");
    let mint_state = spl_token::state::Mint::unpack(&mint_account.data)
        .expect("failed to unpack mint account");
    let has_full_flow_accounts = accounts.contains_key("user1_token")
        && accounts.contains_key("user2_token")
        && accounts.contains_key("user3_token");
    if has_full_flow_accounts {
        assert_eq!(mint_state.supply, 1900, "unexpected mint supply after full flow");
    } else {
        assert_eq!(mint_state.supply, 1, "unexpected mint supply");
    }
    assert_eq!(mint_state.mint_authority, COption::Some(authority));
    if has_full_flow_accounts {
        let user1_token = accounts["user1_token"].pubkey;
        let user2_token = accounts["user2_token"].pubkey;
        let user3_token = accounts["user3_token"].pubkey;

        let user1_state = spl_token::state::Account::unpack(
            &ctx.banks_client
                .get_account(user1_token)
                .await
                .expect("fetch user1 token")
                .expect("user1 token not found")
                .data,
        )
        .expect("unpack user1 token");
        let user2_state = spl_token::state::Account::unpack(
            &ctx.banks_client
                .get_account(user2_token)
                .await
                .expect("fetch user2 token")
                .expect("user2 token not found")
                .data,
        )
        .expect("unpack user2 token");
        let user3_state = spl_token::state::Account::unpack(
            &ctx.banks_client
                .get_account(user3_token)
                .await
                .expect("fetch user3 token")
                .expect("user3 token not found")
                .data,
        )
        .expect("unpack user3 token");

        assert_eq!(user1_state.amount, 950, "unexpected user1 token amount");
        assert_eq!(user2_state.amount, 400, "unexpected user2 token amount");
        assert_eq!(user3_state.amount, 550, "unexpected user3 token amount");
        assert_eq!(
            user2_state.state,
            spl_token::state::AccountState::Initialized,
            "user2 token account should be thawed"
        );
    } else {
        let dest = accounts
            .get("dest")
            .unwrap_or_else(|| panic!("missing `dest` account for spl assertion"))
            .pubkey;
        let dest_account = ctx
            .banks_client
            .get_account(dest)
            .await
            .expect("fetch dest token account")
            .expect("dest token account not found");
        let dest_state = spl_token::state::Account::unpack(&dest_account.data)
            .expect("failed to unpack token account");
        assert_eq!(dest_state.amount, 1, "unexpected token destination amount");
        assert_eq!(dest_state.owner, authority, "unexpected token owner");
    }
}

fn seed_anchor_token_accounts(
    accounts: &mut BTreeMap<String, RuntimeAccount>,
    authority_name: &str,
) {
    let mint_key = accounts
        .get("mint")
        .unwrap_or_else(|| panic!("missing `mint` account for anchor setup"))
        .pubkey;
    let user1 = accounts
        .get(authority_name)
        .unwrap_or_else(|| panic!("missing authority `{}` for anchor setup", authority_name))
        .pubkey;
    let user2 = accounts
        .get("user2")
        .unwrap_or_else(|| panic!("missing `user2` for anchor setup"))
        .pubkey;
    let user3 = accounts
        .get("user3")
        .unwrap_or_else(|| panic!("missing `user3` for anchor setup"))
        .pubkey;

    if let Some(mint) = accounts.get_mut("mint") {
        mint.owner = anchor_token_program_id();
        mint.data = encode_anchor_mint(user1, user1, 0, 0, "", "", "");
    }
    if let Some(user1_token) = accounts.get_mut("user1_token") {
        user1_token.owner = anchor_token_program_id();
        user1_token.data = encode_anchor_token_account(user1, mint_key, 0, false, 0, Pubkey::default(), true);
    }
    if let Some(user2_token) = accounts.get_mut("user2_token") {
        user2_token.owner = anchor_token_program_id();
        user2_token.data = encode_anchor_token_account(user2, mint_key, 0, false, 0, Pubkey::default(), true);
    }
    if let Some(user3_token) = accounts.get_mut("user3_token") {
        user3_token.owner = anchor_token_program_id();
        user3_token.data = encode_anchor_token_account(user3, mint_key, 0, false, 0, Pubkey::default(), true);
    }
}

async fn assert_anchor_token_fixture_result(
    ctx: &mut ProgramTestContext,
    accounts: &BTreeMap<String, RuntimeAccount>,
    authority_name: &str,
) {
    let user1 = accounts
        .get(authority_name)
        .unwrap_or_else(|| panic!("missing authority `{}` for anchor assertion", authority_name))
        .pubkey;
    let mint_key = accounts["mint"].pubkey;
    let mint_data = ctx
        .banks_client
        .get_account(mint_key)
        .await
        .expect("fetch anchor mint")
        .expect("anchor mint missing")
        .data;
    let mint_state = decode_anchor_mint(&mint_data);
    assert_eq!(mint_state.authority, user1, "unexpected anchor mint authority");
    assert_eq!(mint_state.freeze_authority, user1, "unexpected anchor freeze authority");
    assert_eq!(mint_state.supply, 1900, "unexpected anchor mint supply");

    let user1_token = decode_anchor_token_account(
        &ctx.banks_client
            .get_account(accounts["user1_token"].pubkey)
            .await
            .expect("fetch anchor user1 token")
            .expect("anchor user1 token missing")
            .data,
    );
    let user2_token = decode_anchor_token_account(
        &ctx.banks_client
            .get_account(accounts["user2_token"].pubkey)
            .await
            .expect("fetch anchor user2 token")
            .expect("anchor user2 token missing")
            .data,
    );
    let user3_token = decode_anchor_token_account(
        &ctx.banks_client
            .get_account(accounts["user3_token"].pubkey)
            .await
            .expect("fetch anchor user3 token")
            .expect("anchor user3 token missing")
            .data,
    );
    assert_eq!(user1_token.balance, 950, "unexpected anchor user1 token amount");
    assert_eq!(user2_token.balance, 400, "unexpected anchor user2 token amount");
    assert_eq!(user3_token.balance, 550, "unexpected anchor user3 token amount");
    assert!(!user2_token.is_frozen, "anchor user2 token should be thawed");
}

async fn assert_anchor_token_manual_fixture_result(
    ctx: &mut ProgramTestContext,
    accounts: &BTreeMap<String, RuntimeAccount>,
    authority_name: &str,
) {
    let user1 = accounts
        .get(authority_name)
        .unwrap_or_else(|| panic!("missing authority `{}` for anchor assertion", authority_name))
        .pubkey;
    let mint_key = accounts["mint"].pubkey;
    let mint_data = ctx
        .banks_client
        .get_account(mint_key)
        .await
        .expect("fetch anchor mint")
        .expect("anchor mint missing")
        .data;
    let mint_state = decode_anchor_mint(&mint_data);
    assert_eq!(mint_state.authority, user1, "unexpected anchor mint authority");
    assert_eq!(mint_state.freeze_authority, user1, "unexpected anchor freeze authority");
    assert_eq!(mint_state.supply, 1900, "unexpected anchor mint supply");

    let user1_token = decode_anchor_token_account(
        &ctx.banks_client
            .get_account(accounts["user1_token"].pubkey)
            .await
            .expect("fetch anchor user1 token")
            .expect("anchor user1 token missing")
            .data,
    );
    let user2_token = decode_anchor_token_account(
        &ctx.banks_client
            .get_account(accounts["user2_token"].pubkey)
            .await
            .expect("fetch anchor user2 token")
            .expect("anchor user2 token missing")
            .data,
    );
    let user3_token = decode_anchor_token_account(
        &ctx.banks_client
            .get_account(accounts["user3_token"].pubkey)
            .await
            .expect("fetch anchor user3 token")
            .expect("anchor user3 token missing")
            .data,
    );
    assert_eq!(user1_token.balance, 900, "unexpected anchor user1 token amount");
    assert_eq!(user2_token.balance, 400, "unexpected anchor user2 token amount");
    assert_eq!(user3_token.balance, 600, "unexpected anchor user3 token amount");
    assert!(!user2_token.is_frozen, "anchor user2 token should be thawed");
}

#[derive(Debug)]
struct AnchorMintState {
    authority: Pubkey,
    freeze_authority: Pubkey,
    supply: u64,
}

#[derive(Debug)]
struct AnchorTokenState {
    balance: u64,
    is_frozen: bool,
}

fn anchor_account_discriminator(name: &str) -> [u8; 8] {
    let digest = solana_program::hash::hashv(&[format!("account:{name}").as_bytes()]).to_bytes();
    let mut out = [0u8; 8];
    out.copy_from_slice(&digest[0..8]);
    out
}

fn encode_anchor_mint(
    authority: Pubkey,
    freeze_authority: Pubkey,
    supply: u64,
    decimals: u8,
    name: &str,
    symbol: &str,
    uri: &str,
) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&anchor_account_discriminator("Mint"));
    out.extend_from_slice(authority.as_ref());
    out.extend_from_slice(freeze_authority.as_ref());
    out.extend_from_slice(&supply.to_le_bytes());
    out.push(decimals);
    append_anchor_string(&mut out, name);
    append_anchor_string(&mut out, symbol);
    append_anchor_string(&mut out, uri);
    out
}

fn encode_anchor_token_account(
    owner: Pubkey,
    mint: Pubkey,
    balance: u64,
    is_frozen: bool,
    delegated_amount: u64,
    delegate: Pubkey,
    initialized: bool,
) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&anchor_account_discriminator("TokenAccount"));
    out.extend_from_slice(owner.as_ref());
    out.extend_from_slice(mint.as_ref());
    out.extend_from_slice(&balance.to_le_bytes());
    out.push(if is_frozen { 1 } else { 0 });
    out.extend_from_slice(&delegated_amount.to_le_bytes());
    out.extend_from_slice(delegate.as_ref());
    out.push(if initialized { 1 } else { 0 });
    out
}

fn decode_anchor_mint(data: &[u8]) -> AnchorMintState {
    assert!(data.len() >= 8 + 32 + 32 + 8 + 1, "anchor mint data too small");
    assert_eq!(
        &data[0..8],
        &anchor_account_discriminator("Mint"),
        "anchor mint discriminator mismatch"
    );
    let authority = Pubkey::new_from_array(data[8..40].try_into().expect("authority bytes"));
    let freeze_authority = Pubkey::new_from_array(data[40..72].try_into().expect("freeze bytes"));
    let supply = u64::from_le_bytes(data[72..80].try_into().expect("supply bytes"));
    AnchorMintState {
        authority,
        freeze_authority,
        supply,
    }
}

fn decode_anchor_token_account(data: &[u8]) -> AnchorTokenState {
    assert!(data.len() >= 8 + 32 + 32 + 8 + 1 + 8 + 32 + 1, "anchor token data too small");
    assert_eq!(
        &data[0..8],
        &anchor_account_discriminator("TokenAccount"),
        "anchor token discriminator mismatch"
    );
    let balance = u64::from_le_bytes(data[72..80].try_into().expect("balance bytes"));
    let is_frozen = data[80] != 0;
    AnchorTokenState { balance, is_frozen }
}

fn append_anchor_string(out: &mut Vec<u8>, s: &str) {
    let bytes = s.as_bytes();
    out.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
    out.extend_from_slice(bytes);
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

fn default_expected_fixture() -> ExpectedFixture {
    ExpectedFixture::Success
}

fn fixture_path_from_env(repo_root: &Path) -> PathBuf {
    if let Ok(override_path) = std::env::var("FIVE_BPF_FIXTURE") {
        let p = PathBuf::from(&override_path);
        if p.is_absolute() {
            return p;
        }
        return repo_root.join(p);
    }
    repo_root.join("five-templates/token/runtime-fixtures/init_mint.json")
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
