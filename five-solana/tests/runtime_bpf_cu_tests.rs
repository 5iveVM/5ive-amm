mod harness;

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};

use five::instructions::{DEPLOY_INSTRUCTION, EXECUTE_INSTRUCTION};
use five::state::{FIVEVMState, ScriptAccountHeader};
use five_dsl_compiler::DslCompiler;
use five_protocol::{
    opcodes::{self, CALL_EXTERNAL, HALT},
    parser::parse_code_bounds,
};
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

fn print_external_cache_metrics(label: &str) {
    #[cfg(not(target_os = "solana"))]
    {
        let (hits, misses, verify_hits) = five_vm_mito::MitoVM::last_external_cache_metrics();
        if hits == 0 && misses == 0 && verify_hits == 0 {
            println!(
                "BPF_CU {} external_cache_metrics=unavailable_in_bpf_program_test",
                label
            );
        } else {
            println!(
                "BPF_CU {} external_cache_hits={} external_cache_misses={} import_verify_cache_hits={}",
                label, hits, misses, verify_hits
            );
        }
    }
}

fn print_external_call_opcode_mix(label: &str, bytecode: &[u8]) {
    let mut call_external = 0usize;
    if let Ok((header, mut offset, code_end)) = parse_code_bounds(bytecode) {
        let pool_enabled = (header.features & five_protocol::FEATURE_CONSTANT_POOL) != 0;
        while offset < code_end {
            let opcode = bytecode[offset];
            if opcode == CALL_EXTERNAL {
                call_external += 1;
            }
            let remaining = &bytecode[offset + 1..];
            let Some(operand_bytes) = opcodes::operand_size(opcode, remaining, pool_enabled) else {
                break;
            };
            let Some(next) = offset.checked_add(1 + operand_bytes) else {
                break;
            };
            if next > code_end {
                break;
            }
            offset = next;
        }
    }
    println!("BPF_CU {} call_external={}", label, call_external);
}

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
    #[serde(alias = "deploy_fee_bps")]
    deploy_fee_lamports: u32,
    #[serde(alias = "execute_fee_bps")]
    execute_fee_lamports: u32,
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

const CU_EXECUTE_FEE_LAMPORTS: u32 = 500;
const CU_FEE_STEP_HEADROOM: u64 = 6_000;

const TOKEN_ALL_PUBLIC_CALLS: [&str; 14] = [
    "mint_to(mint_account, user1_token, user1, 1000);",
    "mint_to(mint_account, user2_token, user1, 500);",
    "mint_to(mint_account, user3_token, user1, 500);",
    "transfer(user2_token, user3_token, user2, 100);",
    "approve(user3_token, user3, new_mint_authority_pk, 150);",
    "transfer_from(user3_token, user1_token, user2, 50);",
    "revoke(user3_token, user3);",
    "burn(mint_account, user1_token, user1, 100);",
    "freeze_account(mint_account, user2_token, user1);",
    "thaw_account(mint_account, user2_token, user1);",
    "transfer(user1_token, user2_token, user1, 10);",
    "transfer(user2_token, user1_token, user2, 10);",
    "approve(user1_token, user1, new_mint_authority_pk, 1);",
    "revoke(user1_token, user1);",
];

const TOKEN_ALL_PUBLIC_POST_CALLS: [&str; 4] = [
    "set_mint_authority(mint_account, user1, new_mint_authority_pk);",
    "set_freeze_authority(mint_account, user1, new_freeze_authority_pk);",
    "disable_mint(mint_account, user2);",
    "disable_freeze(mint_account, user2);",
];

struct ExternalAllPublicRun {
    deploy_token_units: u64,
    deploy_caller_units: u64,
    execute_units: u64,
    caller_bytecode_size: usize,
    token_bytecode_size: usize,
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
async fn external_token_transfer_non_cpi_bpf_compute_units() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let bpf_dir = repo_root.join("target/deploy");
    std::env::set_var("BPF_OUT_DIR", &bpf_dir);

    let program_id = read_keypair_file(bpf_dir.join("five-keypair.json"))
        .expect("missing target/deploy/five-keypair.json; run `cargo-build-sbf --manifest-path five-solana/Cargo.toml`")
        .pubkey();

    let token_bytecode_path = repo_root.join("five-templates/token/src/token.bin");
    let token_bytecode = fs::read(&token_bytecode_path)
        .unwrap_or_else(|e| panic!("failed reading {}: {}", token_bytecode_path.display(), e));

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

    let mut vm_state_data = vec![0u8; FIVEVMState::LEN];
    {
        let vm_state = FIVEVMState::from_account_data_mut(&mut vm_state_data)
            .expect("invalid vm state account layout");
        vm_state.initialize(owner_pubkey.to_bytes());
        vm_state.deploy_fee_lamports = 0;
        vm_state.execute_fee_lamports = 0;
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

    let token_script_pubkey = Pubkey::new_unique();
    accounts.insert(
        "token_script".to_string(),
        RuntimeAccount {
            pubkey: token_script_pubkey,
            signer: None,
            owner: program_id,
            lamports: Rent::default().minimum_balance(ScriptAccountHeader::LEN + token_bytecode.len()),
            data: vec![0u8; ScriptAccountHeader::LEN + token_bytecode.len()],
            is_signer: false,
            is_writable: false,
            executable: false,
        },
    );

    let token_import_address = bs58::encode(token_script_pubkey.to_bytes()).into_string();
    let caller_source = format!(
        r#"
        use "{token_import_address}"::{{transfer}};

        pub fn call_transfer(
            source_account: account @mut,
            destination_account: account @mut,
            owner: account @mut,
            ext0: account
        ) {{
            transfer(source_account, destination_account, owner, 50);
        }}
    "#
    );
    let caller_bytecode =
        DslCompiler::compile_dsl(&caller_source).expect("caller script should compile");
    print_external_call_opcode_mix("external_non_cpi", &caller_bytecode);
    accounts.insert(
        "caller_script".to_string(),
        RuntimeAccount {
            pubkey: Pubkey::new_unique(),
            signer: None,
            owner: program_id,
            lamports: Rent::default().minimum_balance(ScriptAccountHeader::LEN + caller_bytecode.len()),
            data: vec![0u8; ScriptAccountHeader::LEN + caller_bytecode.len()],
            is_signer: false,
            is_writable: true,
            executable: false,
        },
    );

    let mint_pubkey = Pubkey::new_unique();
    let source_token_pubkey = Pubkey::new_unique();
    let destination_token_pubkey = Pubkey::new_unique();

    let mut source_data = vec![0u8; 192];
    source_data[0..32].copy_from_slice(owner_pubkey.as_ref());
    source_data[32..64].copy_from_slice(mint_pubkey.as_ref());
    source_data[64..72].copy_from_slice(&500u64.to_le_bytes());
    source_data[72] = 0;
    accounts.insert(
        "source_token".to_string(),
        RuntimeAccount {
            pubkey: source_token_pubkey,
            signer: None,
            owner: program_id,
            lamports: Rent::default().minimum_balance(source_data.len()),
            data: source_data,
            is_signer: false,
            is_writable: true,
            executable: false,
        },
    );

    let mut destination_data = vec![0u8; 192];
    destination_data[0..32].copy_from_slice(destination_token_pubkey.as_ref());
    destination_data[32..64].copy_from_slice(mint_pubkey.as_ref());
    destination_data[64..72].copy_from_slice(&100u64.to_le_bytes());
    destination_data[72] = 0;
    accounts.insert(
        "destination_token".to_string(),
        RuntimeAccount {
            pubkey: destination_token_pubkey,
            signer: None,
            owner: program_id,
            lamports: Rent::default().minimum_balance(destination_data.len()),
            data: destination_data,
            is_signer: false,
            is_writable: true,
            executable: false,
        },
    );

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
    let mut ctx = program_test.start_with_context().await;

    let deploy_token_ix = build_deploy_instruction(
        program_id,
        &accounts,
        "token_script",
        "vm_state",
        "owner",
        &token_bytecode,
        0,
    );
    let deploy_token = simulate_and_process(
        &mut ctx,
        vec![deploy_token_ix],
        collect_signers(&accounts, &["owner"]),
        Some(1_400_000),
    )
    .await;
    assert!(deploy_token.success, "token deploy failed: {:?}", deploy_token.error);

    let deploy_caller_ix = build_deploy_instruction(
        program_id,
        &accounts,
        "caller_script",
        "vm_state",
        "owner",
        &caller_bytecode,
        0,
    );
    let deploy_caller = simulate_and_process(
        &mut ctx,
        vec![deploy_caller_ix],
        collect_signers(&accounts, &["owner"]),
        Some(1_400_000),
    )
    .await;
    assert!(deploy_caller.success, "caller deploy failed: {:?}", deploy_caller.error);

    let step = StepFixture {
        name: "external_transfer_non_cpi".to_string(),
        function_index: 0,
        extras: vec![
            "source_token".to_string(),
            "destination_token".to_string(),
            "owner".to_string(),
            "token_script".to_string(),
        ],
        params: vec![
            ParamFixture::AccountRef {
                account: "source_token".to_string(),
            },
            ParamFixture::AccountRef {
                account: "destination_token".to_string(),
            },
            ParamFixture::AccountRef {
                account: "owner".to_string(),
            },
            ParamFixture::AccountRef {
                account: "token_script".to_string(),
            },
        ],
        expected: ExpectedFixture::Success,
    };
    let execute_ix = build_execute_instruction(
        program_id,
        &accounts,
        "caller_script",
        "vm_state",
        &step,
        build_payload(&accounts, &step),
    );
    let execute = simulate_and_process(
        &mut ctx,
        vec![execute_ix],
        collect_signers(&accounts, &["owner"]),
        Some(1_400_000),
    )
    .await;
    assert!(
        execute.success,
        "external transfer execution failed: {:?}",
        execute.error
    );

    let source_after = ctx
        .banks_client
        .get_account(source_token_pubkey)
        .await
        .expect("fetch source account")
        .expect("source token account missing");
    let destination_after = ctx
        .banks_client
        .get_account(destination_token_pubkey)
        .await
        .expect("fetch destination account")
        .expect("destination token account missing");
    let source_balance = u64::from_le_bytes(source_after.data[64..72].try_into().unwrap());
    let destination_balance = u64::from_le_bytes(destination_after.data[64..72].try_into().unwrap());
    assert_eq!(source_balance, 450);
    assert_eq!(destination_balance, 150);

    println!(
        "BPF_CU external_non_cpi deploy_token={} deploy_caller={} execute={} total={} caller_bytecode_size={} token_bytecode_size={}",
        deploy_token.units_consumed,
        deploy_caller.units_consumed,
        execute.units_consumed,
        deploy_token
            .units_consumed
            .saturating_add(deploy_caller.units_consumed)
            .saturating_add(execute.units_consumed),
        caller_bytecode.len(),
        token_bytecode.len()
    );
    print_external_cache_metrics("external_non_cpi");
}

#[tokio::test(flavor = "multi_thread")]
async fn external_token_transfer_burst_non_cpi_bpf_compute_units() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let bpf_dir = repo_root.join("target/deploy");
    std::env::set_var("BPF_OUT_DIR", &bpf_dir);

    let program_id = read_keypair_file(bpf_dir.join("five-keypair.json"))
        .expect("missing target/deploy/five-keypair.json; run `cargo-build-sbf --manifest-path five-solana/Cargo.toml`")
        .pubkey();

    let token_bytecode_path = repo_root.join("five-templates/token/src/token.bin");
    let token_bytecode = fs::read(&token_bytecode_path)
        .unwrap_or_else(|e| panic!("failed reading {}: {}", token_bytecode_path.display(), e));

    let mut accounts = BTreeMap::<String, RuntimeAccount>::new();
    let owner_signer = Keypair::new();
    let owner_pubkey = owner_signer.pubkey();
    accounts.insert(
        "owner".to_string(),
        RuntimeAccount {
            pubkey: owner_pubkey,
            signer: Some(owner_signer),
            owner: system_program::id(),
            lamports: 50_000_000,
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
        vm_state.initialize(owner_pubkey.to_bytes());
        vm_state.deploy_fee_lamports = 0;
        vm_state.execute_fee_lamports = 0;
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

    let token_script_pubkey = Pubkey::new_unique();
    accounts.insert(
        "token_script".to_string(),
        RuntimeAccount {
            pubkey: token_script_pubkey,
            signer: None,
            owner: program_id,
            lamports: Rent::default().minimum_balance(ScriptAccountHeader::LEN + token_bytecode.len()),
            data: vec![0u8; ScriptAccountHeader::LEN + token_bytecode.len()],
            is_signer: false,
            is_writable: false,
            executable: false,
        },
    );

    let token_import_address = bs58::encode(token_script_pubkey.to_bytes()).into_string();
    let caller_source = format!(
        r#"
        use "{token_import_address}"::{{transfer}};

        pub fn burst_transfer(
            s1: account @mut, d1: account @mut,
            s2: account @mut, d2: account @mut,
            s3: account @mut, d3: account @mut,
            s4: account @mut, d4: account @mut,
            owner: account @mut,
            ext0: account
        ) {{
            transfer(s1, d1, owner, 10);
            transfer(s2, d2, owner, 20);
            transfer(s3, d3, owner, 30);
            transfer(s4, d4, owner, 40);
        }}
    "#
    );
    let caller_bytecode =
        DslCompiler::compile_dsl(&caller_source).expect("caller burst script should compile");
    print_external_call_opcode_mix("external_burst_non_cpi", &caller_bytecode);
    accounts.insert(
        "caller_script".to_string(),
        RuntimeAccount {
            pubkey: Pubkey::new_unique(),
            signer: None,
            owner: program_id,
            lamports: Rent::default().minimum_balance(ScriptAccountHeader::LEN + caller_bytecode.len()),
            data: vec![0u8; ScriptAccountHeader::LEN + caller_bytecode.len()],
            is_signer: false,
            is_writable: true,
            executable: false,
        },
    );

    let mint_pubkey = Pubkey::new_unique();
    for i in 1..=4 {
        let src_key = Pubkey::new_unique();
        let dst_key = Pubkey::new_unique();
        let mut src_data = vec![0u8; 192];
        src_data[0..32].copy_from_slice(owner_pubkey.as_ref());
        src_data[32..64].copy_from_slice(mint_pubkey.as_ref());
        src_data[64..72].copy_from_slice(&1000u64.to_le_bytes());
        src_data[72] = 0;
        accounts.insert(
            format!("source_token_{}", i),
            RuntimeAccount {
                pubkey: src_key,
                signer: None,
                owner: program_id,
                lamports: Rent::default().minimum_balance(src_data.len()),
                data: src_data,
                is_signer: false,
                is_writable: true,
                executable: false,
            },
        );

        let mut dst_data = vec![0u8; 192];
        dst_data[0..32].copy_from_slice(Pubkey::new_unique().as_ref());
        dst_data[32..64].copy_from_slice(mint_pubkey.as_ref());
        // Increase balance to accommodate all transfers
        dst_data[64..72].copy_from_slice(&15000u64.to_le_bytes());
        dst_data[72] = 0;
        accounts.insert(
            format!("dest_token_{}", i),
            RuntimeAccount {
                pubkey: dst_key,
                signer: None,
                owner: program_id,
                lamports: Rent::default().minimum_balance(dst_data.len()),
                data: dst_data,
                is_signer: false,
                is_writable: true,
                executable: false,
            },
        );
    }

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
    let mut ctx = program_test.start_with_context().await;

    let deploy_token_ix = build_deploy_instruction(
        program_id,
        &accounts,
        "token_script",
        "vm_state",
        "owner",
        &token_bytecode,
        0,
    );
    let deploy_token = simulate_and_process(
        &mut ctx,
        vec![deploy_token_ix],
        collect_signers(&accounts, &["owner"]),
        Some(1_400_000),
    )
    .await;
    assert!(deploy_token.success, "token deploy failed: {:?}", deploy_token.error);

    let deploy_caller_ix = build_deploy_instruction(
        program_id,
        &accounts,
        "caller_script",
        "vm_state",
        "owner",
        &caller_bytecode,
        0,
    );
    let deploy_caller = simulate_and_process(
        &mut ctx,
        vec![deploy_caller_ix],
        collect_signers(&accounts, &["owner"]),
        Some(1_400_000),
    )
    .await;
    assert!(deploy_caller.success, "caller deploy failed: {:?}", deploy_caller.error);

    let burst_step = StepFixture {
        name: "external_transfer_burst_non_cpi".to_string(),
        function_index: 0,
        extras: vec![
            "source_token_1".to_string(), "dest_token_1".to_string(),
            "source_token_2".to_string(), "dest_token_2".to_string(),
            "source_token_3".to_string(), "dest_token_3".to_string(),
            "source_token_4".to_string(), "dest_token_4".to_string(),
            "owner".to_string(),
            "token_script".to_string(),
        ],
        params: vec![
            ParamFixture::AccountRef { account: "source_token_1".to_string() },
            ParamFixture::AccountRef { account: "dest_token_1".to_string() },
            ParamFixture::AccountRef { account: "source_token_2".to_string() },
            ParamFixture::AccountRef { account: "dest_token_2".to_string() },
            ParamFixture::AccountRef { account: "source_token_3".to_string() },
            ParamFixture::AccountRef { account: "dest_token_3".to_string() },
            ParamFixture::AccountRef { account: "source_token_4".to_string() },
            ParamFixture::AccountRef { account: "dest_token_4".to_string() },
            ParamFixture::AccountRef { account: "owner".to_string() },
            ParamFixture::AccountRef { account: "token_script".to_string() },
        ],
        expected: ExpectedFixture::Success,
    };

    let execute_ix = build_execute_instruction(
        program_id,
        &accounts,
        "caller_script",
        "vm_state",
        &burst_step,
        build_payload(&accounts, &burst_step),
    );
    let execute = simulate_and_process(
        &mut ctx,
        vec![execute_ix],
        collect_signers(&accounts, &["owner"]),
        Some(1_400_000),
    )
    .await;
    assert!(
        execute.success,
        "burst external transfer execution failed: {:?}",
        execute.error
    );

    let transfer_amounts = [10u64, 20, 30, 40];
    for (i, amount) in transfer_amounts.iter().enumerate() {
        let src_name = format!("source_token_{}", i + 1);
        let dst_name = format!("dest_token_{}", i + 1);
        let src_pk = accounts[&src_name].pubkey;
        let dst_pk = accounts[&dst_name].pubkey;
        let src_after = ctx
            .banks_client
            .get_account(src_pk)
            .await
            .expect("fetch source account")
            .expect("source token account missing");
        let dst_after = ctx
            .banks_client
            .get_account(dst_pk)
            .await
            .expect("fetch destination account")
            .expect("destination token account missing");
        let src_balance = u64::from_le_bytes(src_after.data[64..72].try_into().unwrap());
        let dst_balance = u64::from_le_bytes(dst_after.data[64..72].try_into().unwrap());
        assert_eq!(src_balance, 1000 - amount);
        assert_eq!(dst_balance, 15000 + amount);
    }

    println!(
        "BPF_CU external_burst_non_cpi deploy_token={} deploy_caller={} execute={} total={} caller_bytecode_size={} token_bytecode_size={} transfers={}",
        deploy_token.units_consumed,
        deploy_caller.units_consumed,
        execute.units_consumed,
        deploy_token
            .units_consumed
            .saturating_add(deploy_caller.units_consumed)
            .saturating_add(execute.units_consumed),
        caller_bytecode.len(),
        token_bytecode.len(),
        transfer_amounts.len()
    );
    print_external_cache_metrics("external_burst_non_cpi");
}

#[tokio::test(flavor = "multi_thread")]
async fn external_token_transfer_mass_non_cpi_bpf_compute_units() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let bpf_dir = repo_root.join("target/deploy");
    std::env::set_var("BPF_OUT_DIR", &bpf_dir);

    let program_id = read_keypair_file(bpf_dir.join("five-keypair.json"))
        .expect("missing target/deploy/five-keypair.json; run `cargo-build-sbf --manifest-path five-solana/Cargo.toml`")
        .pubkey();

    let token_bytecode_path = repo_root.join("five-templates/token/src/token.bin");
    let token_bytecode = fs::read(&token_bytecode_path)
        .unwrap_or_else(|e| panic!("failed reading {}: {}", token_bytecode_path.display(), e));

    // Maximize transfers: 11 pairs (22 accounts) + owner + ext0 = 24 params (at limit)
    let transfer_amounts: Vec<u64> = (1u64..=11).map(|n| n * 10).collect();
    let pair_count = transfer_amounts.len();

    let mut accounts = BTreeMap::<String, RuntimeAccount>::new();
    let owner_signer = Keypair::new();
    let owner_pubkey = owner_signer.pubkey();
    accounts.insert(
        "owner".to_string(),
        RuntimeAccount {
            pubkey: owner_pubkey,
            signer: Some(owner_signer),
            owner: system_program::id(),
            lamports: 120_000_000,
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
        vm_state.initialize(owner_pubkey.to_bytes());
        vm_state.deploy_fee_lamports = 0;
        vm_state.execute_fee_lamports = 0;
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

    let token_script_pubkey = Pubkey::new_unique();
    accounts.insert(
        "token_script".to_string(),
        RuntimeAccount {
            pubkey: token_script_pubkey,
            signer: None,
            owner: program_id,
            lamports: Rent::default().minimum_balance(ScriptAccountHeader::LEN + token_bytecode.len()),
            data: vec![0u8; ScriptAccountHeader::LEN + token_bytecode.len()],
            is_signer: false,
            is_writable: false,
            executable: false,
        },
    );

    let token_import_address = bs58::encode(token_script_pubkey.to_bytes()).into_string();
    let mut signature_parts = Vec::new();
    let mut call_lines = Vec::new();
    for (idx, amount) in transfer_amounts.iter().enumerate() {
        let i = idx + 1;
        signature_parts.push(format!("s{i}: account @mut"));
        signature_parts.push(format!("d{i}: account @mut"));
        // Do many transfers per pair to maximize CU usage (18 calls per pair)
        for _ in 0..18 {
            call_lines.push(format!("transfer(s{i}, d{i}, owner, {amount});"));
        }
    }
    signature_parts.push("owner: account @mut".to_string());
    signature_parts.push("ext0: account".to_string());

    let caller_source = format!(
        r#"
        use "{token_import_address}"::{{transfer}};

        pub fn mass_transfer(
            {}
        ) {{
            {}
        }}
    "#,
        signature_parts.join(",\n            "),
        call_lines.join("\n            ")
    );

    // Write generated script to file so it can be inspected and compiled manually
    let output_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let script_file = output_dir.join("five-templates/mass-transfer-generated.v");
    let _ = fs::write(&script_file, &caller_source).map_err(|e| {
        eprintln!("Warning: could not write generated script to {}: {}", script_file.display(), e)
    });
    println!("Generated mass_transfer script written to: {}", script_file.display());

    let caller_bytecode =
        DslCompiler::compile_dsl(&caller_source).expect("caller mass-transfer script should compile");
    print_external_call_opcode_mix("external_mass_transfer_non_cpi", &caller_bytecode);
    accounts.insert(
        "caller_script".to_string(),
        RuntimeAccount {
            pubkey: Pubkey::new_unique(),
            signer: None,
            owner: program_id,
            lamports: Rent::default().minimum_balance(ScriptAccountHeader::LEN + caller_bytecode.len()),
            data: vec![0u8; ScriptAccountHeader::LEN + caller_bytecode.len()],
            is_signer: false,
            is_writable: true,
            executable: false,
        },
    );

    let mint_pubkey = Pubkey::new_unique();
    for i in 1..=pair_count {
        let src_key = Pubkey::new_unique();
        let dst_key = Pubkey::new_unique();
        let mut src_data = vec![0u8; 192];
        src_data[0..32].copy_from_slice(owner_pubkey.as_ref());
        src_data[32..64].copy_from_slice(mint_pubkey.as_ref());
        // Increase balance to support 20 transfers per pair: 10 * 20 * 11 pairs = ~2200, use 30000 to be safe
        src_data[64..72].copy_from_slice(&30000u64.to_le_bytes());
        src_data[72] = 0;
        accounts.insert(
            format!("source_token_{}", i),
            RuntimeAccount {
                pubkey: src_key,
                signer: None,
                owner: program_id,
                lamports: Rent::default().minimum_balance(src_data.len()),
                data: src_data,
                is_signer: false,
                is_writable: true,
                executable: false,
            },
        );

        let mut dst_data = vec![0u8; 192];
        dst_data[0..32].copy_from_slice(Pubkey::new_unique().as_ref());
        dst_data[32..64].copy_from_slice(mint_pubkey.as_ref());
        // Increase balance to accommodate all transfers
        dst_data[64..72].copy_from_slice(&15000u64.to_le_bytes());
        dst_data[72] = 0;
        accounts.insert(
            format!("dest_token_{}", i),
            RuntimeAccount {
                pubkey: dst_key,
                signer: None,
                owner: program_id,
                lamports: Rent::default().minimum_balance(dst_data.len()),
                data: dst_data,
                is_signer: false,
                is_writable: true,
                executable: false,
            },
        );
    }

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
    let mut ctx = program_test.start_with_context().await;

    let deploy_token_ix = build_deploy_instruction(
        program_id,
        &accounts,
        "token_script",
        "vm_state",
        "owner",
        &token_bytecode,
        0,
    );
    let deploy_token = simulate_and_process(
        &mut ctx,
        vec![deploy_token_ix],
        collect_signers(&accounts, &["owner"]),
        Some(1_400_000),
    )
    .await;
    assert!(deploy_token.success, "token deploy failed: {:?}", deploy_token.error);

    let deploy_caller_ix = build_deploy_instruction(
        program_id,
        &accounts,
        "caller_script",
        "vm_state",
        "owner",
        &caller_bytecode,
        0,
    );
    let deploy_caller = simulate_and_process(
        &mut ctx,
        vec![deploy_caller_ix],
        collect_signers(&accounts, &["owner"]),
        Some(1_400_000),
    )
    .await;
    assert!(deploy_caller.success, "caller deploy failed: {:?}", deploy_caller.error);

    let mut extras = Vec::with_capacity(pair_count * 2 + 2);
    let mut params = Vec::with_capacity(pair_count * 2 + 2);
    for i in 1..=pair_count {
        let s = format!("source_token_{}", i);
        let d = format!("dest_token_{}", i);
        extras.push(s.clone());
        extras.push(d.clone());
        params.push(ParamFixture::AccountRef { account: s });
        params.push(ParamFixture::AccountRef { account: d });
    }
    extras.push("owner".to_string());
    extras.push("token_script".to_string());
    params.push(ParamFixture::AccountRef {
        account: "owner".to_string(),
    });
    params.push(ParamFixture::AccountRef {
        account: "token_script".to_string(),
    });

    let mass_step = StepFixture {
        name: "external_transfer_mass_non_cpi".to_string(),
        function_index: 0,
        extras,
        params,
        expected: ExpectedFixture::Success,
    };

    let execute_ix = build_execute_instruction(
        program_id,
        &accounts,
        "caller_script",
        "vm_state",
        &mass_step,
        build_payload(&accounts, &mass_step),
    );
    let execute = simulate_and_process(
        &mut ctx,
        vec![execute_ix],
        collect_signers(&accounts, &["owner"]),
        Some(1_400_000),
    )
    .await;
    assert!(
        execute.success,
        "mass external transfer execution failed: {:?}",
        execute.error
    );

    // Verify final balances after all transfers
    // Each pair gets transferred 18 times from source to destination
    // Initial: src = 30000, dst = 15000
    // Final: src = 30000 - (18 * amount)
    //        dst = 15000 + (18 * amount)
    for (i, amount) in transfer_amounts.iter().enumerate() {
        let src_name = format!("source_token_{}", i + 1);
        let dst_name = format!("dest_token_{}", i + 1);
        let src_pk = accounts[&src_name].pubkey;
        let dst_pk = accounts[&dst_name].pubkey;
        let src_after = ctx
            .banks_client
            .get_account(src_pk)
            .await
            .expect("fetch source account")
            .expect("source token account missing");
        let dst_after = ctx
            .banks_client
            .get_account(dst_pk)
            .await
            .expect("fetch destination account")
            .expect("destination token account missing");
        let src_balance = u64::from_le_bytes(src_after.data[64..72].try_into().unwrap());
        let dst_balance = u64::from_le_bytes(dst_after.data[64..72].try_into().unwrap());
        assert_eq!(src_balance, 30000 - (18 * amount), "source {} balance mismatch", i + 1);
        assert_eq!(dst_balance, 15000 + (18 * amount), "destination {} balance mismatch", i + 1);
    }

    // Total transfer calls: 18 calls per pair
    let total_transfer_calls = transfer_amounts.len() * 18;
    println!(
        "BPF_CU external_mass_transfer_non_cpi deploy_token={} deploy_caller={} execute={} total={} caller_bytecode_size={} token_bytecode_size={} transfer_pairs={} total_calls={}",
        deploy_token.units_consumed,
        deploy_caller.units_consumed,
        execute.units_consumed,
        deploy_token
            .units_consumed
            .saturating_add(deploy_caller.units_consumed)
            .saturating_add(execute.units_consumed),
        caller_bytecode.len(),
        token_bytecode.len(),
        transfer_amounts.len(),
        total_transfer_calls
    );
    print_external_cache_metrics("external_mass_transfer_non_cpi");
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "Pending external selector/runtime support for non-transfer public functions"]
async fn external_token_all_public_non_cpi_bpf_compute_units() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let full_call_count = TOKEN_ALL_PUBLIC_CALLS.len() + TOKEN_ALL_PUBLIC_POST_CALLS.len();

    let mut prefix_runs: Vec<ExternalAllPublicRun> = Vec::with_capacity(full_call_count + 1);
    for prefix in 0..=full_call_count {
        prefix_runs.push(run_external_token_all_public_profile(&repo_root, prefix, false).await);
    }

    println!("BPF_CU external_all_public_non_cpi_per_function_deltas_begin");
    for idx in 1..=full_call_count {
        let total_with = prefix_runs[idx].execute_units;
        let total_prev = prefix_runs[idx - 1].execute_units;
        let delta = total_with.saturating_sub(total_prev);
        let call_name = if idx <= TOKEN_ALL_PUBLIC_CALLS.len() {
            TOKEN_ALL_PUBLIC_CALLS[idx - 1]
        } else {
            TOKEN_ALL_PUBLIC_POST_CALLS[idx - TOKEN_ALL_PUBLIC_CALLS.len() - 1]
        };
        println!(
            "BPF_CU external_call_delta idx={} delta={} total_with_prefix={} caller_bytecode_size={} call={}",
            idx,
            delta,
            total_with,
            prefix_runs[idx].caller_bytecode_size,
            call_name
        );
    }
    println!("BPF_CU external_all_public_non_cpi_per_function_deltas_end");

    let full = run_external_token_all_public_profile(&repo_root, full_call_count, true).await;
    println!(
        "BPF_CU external_all_public_non_cpi deploy_token={} deploy_caller={} execute={} total={} caller_bytecode_size={} token_bytecode_size={} calls={}",
        full.deploy_token_units,
        full.deploy_caller_units,
        full.execute_units,
        full.deploy_token_units
            .saturating_add(full.deploy_caller_units)
            .saturating_add(full.execute_units),
        full.caller_bytecode_size,
        full.token_bytecode_size,
        full_call_count,
    );
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
    let fee_admin_name = "__fee_admin".to_string();
    let fee_admin_pubkey = Pubkey::new_unique();
    let execute_fee_lamports = fixture
        .vm_fees
        .as_ref()
        .map(|fees| fees.execute_fee_lamports)
        .filter(|fee| *fee > 0)
        .unwrap_or(CU_EXECUTE_FEE_LAMPORTS);

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
        vm_state.initialize(fee_admin_pubkey.to_bytes());
        vm_state.deploy_fee_lamports = 0;
        vm_state.execute_fee_lamports = execute_fee_lamports;
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
    accounts.insert(
        fee_admin_name.clone(),
        RuntimeAccount {
            pubkey: fee_admin_pubkey,
            signer: None,
            owner: system_program::id(),
            lamports: 5_000_000,
            data: vec![],
            is_signer: false,
            is_writable: true,
            executable: false,
        },
    );
    if !accounts.contains_key("system_program") {
        accounts.insert(
            "system_program".to_string(),
            RuntimeAccount {
                pubkey: system_program::id(),
                signer: None,
                owner: system_program::id(),
                lamports: 1,
                data: vec![],
                is_signer: false,
                is_writable: false,
                executable: true,
            },
        );
    }

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
        let mut effective_extras = step.extras.clone();
        if !effective_extras.iter().any(|name| name == &fixture.authority.name) {
            effective_extras.push(fixture.authority.name.clone());
        }
        if !effective_extras.iter().any(|name| name == &fee_admin_name) {
            effective_extras.push(fee_admin_name.clone());
        }
        if !effective_extras.iter().any(|name| name == "system_program") {
            effective_extras.push("system_program".to_string());
        }

        let execute_ix = build_execute_instruction_with_extras(
            program_id,
            &accounts,
            &fixture.script_name,
            &fixture.vm_state_name,
            &effective_extras,
            payload,
        );

        let signer_names: Vec<&str> = effective_extras
            .iter()
            .filter_map(|name| {
                accounts
                    .get(name)
                    .and_then(|a| if a.is_signer { Some(name.as_str()) } else { None })
            })
            .collect();

        let admin_before = ctx
            .banks_client
            .get_account(fee_admin_pubkey)
            .await
            .expect("fee admin fetch before execute")
            .expect("fee admin account must exist")
            .lamports;

        let result = simulate_and_process(
            &mut ctx,
            vec![execute_ix],
            collect_signers(&accounts, &signer_names),
            None,
        )
        .await;

        let admin_after = ctx
            .banks_client
            .get_account(fee_admin_pubkey)
            .await
            .expect("fee admin fetch after execute")
            .expect("fee admin account must exist")
            .lamports;
        let fee_paid = admin_after.saturating_sub(admin_before);
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
                result.units_consumed <= step_budget(&step.name).saturating_add(CU_FEE_STEP_HEADROOM),
                "step {} consumed {} CU above budget {} (+fee headroom {})",
                step.name,
                result.units_consumed,
                step_budget(&step.name),
                CU_FEE_STEP_HEADROOM
            );
        }
        assert_eq!(
            fee_paid,
            execute_fee_lamports as u64,
            "step {} should charge exactly one execute fee",
            step.name
        );
        total_units = total_units.saturating_add(result.units_consumed);
        println!(
            "BPF_CU step={} expected={:?} success={} units={} fee_paid={} fee_expected={} fee_admin={}",
            step.name,
            step.expected,
            result.success,
            result.units_consumed,
            fee_paid,
            execute_fee_lamports,
            fee_admin_pubkey
        );
    }

    println!(
        "BPF_CU fixture={} total_units={} execute_fee_lamports={} fee_admin={}",
        fixture.name, total_units, execute_fee_lamports, fee_admin_pubkey
    );
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
    })
    .saturating_add((fixture.steps.len() as u64).saturating_mul(CU_FEE_STEP_HEADROOM));
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
        vm_state.deploy_fee_lamports = 0;
        vm_state.execute_fee_lamports = 0;
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

fn build_external_all_public_caller_source(token_import_address: &str, call_count: usize) -> String {
    let calls = TOKEN_ALL_PUBLIC_CALLS
        .iter()
        .chain(TOKEN_ALL_PUBLIC_POST_CALLS.iter())
        .take(call_count)
        .copied()
        .collect::<Vec<_>>()
        .join("\n            ");

    format!(
        r#"
        use "{token_import_address}"::{{
            init_mint,
            init_token_account,
            mint_to,
            transfer,
            transfer_from,
            approve,
            revoke,
            burn,
            freeze_account,
            thaw_account,
            set_mint_authority,
            set_freeze_authority,
            disable_mint,
            disable_freeze
        }};

        pub fn call_all_public_functions(
            mint_account: account @mut,
            user1_token: account @mut,
            user2_token: account @mut,
            user3_token: account @mut,
            user1: account @mut,
            user2: account @mut,
            user3: account @mut,
            ext0: account,
            new_mint_authority_pk: pubkey,
            new_freeze_authority_pk: pubkey
        ) {{
            {calls}
        }}
    "#
    )
}

fn seed_external_token_all_public_accounts(
    accounts: &mut BTreeMap<String, RuntimeAccount>,
    program_id: Pubkey,
    user1_pubkey: Pubkey,
    user2_pubkey: Pubkey,
    user3_pubkey: Pubkey,
) {
    let mint_pubkey = Pubkey::new_unique();
    let mut mint_data = vec![0u8; 256];
    mint_data[0..32].copy_from_slice(user1_pubkey.as_ref()); // authority
    mint_data[32..64].copy_from_slice(user1_pubkey.as_ref()); // freeze authority
    mint_data[64..72].copy_from_slice(&0u64.to_le_bytes()); // supply
    mint_data[72] = 6; // decimals
    accounts.insert(
        "mint_account".to_string(),
        RuntimeAccount {
            pubkey: mint_pubkey,
            signer: None,
            owner: program_id,
            lamports: Rent::default().minimum_balance(mint_data.len()),
            data: mint_data,
            is_signer: false,
            is_writable: true,
            executable: false,
        },
    );

    for (name, owner_pk) in [
        ("user1_token", user1_pubkey),
        ("user2_token", user2_pubkey),
        ("user3_token", user3_pubkey),
    ] {
        let token_pubkey = Pubkey::new_unique();
        let mut token_data = vec![0u8; 192];
        token_data[0..32].copy_from_slice(owner_pk.as_ref());
        token_data[32..64].copy_from_slice(mint_pubkey.as_ref());
        token_data[64..72].copy_from_slice(&0u64.to_le_bytes());
        token_data[72] = 0; // is_frozen
        token_data[73..81].copy_from_slice(&0u64.to_le_bytes()); // delegated_amount
        token_data[81..113].copy_from_slice(&[0u8; 32]); // delegate
        token_data[113] = 1; // initialized
        accounts.insert(
            name.to_string(),
            RuntimeAccount {
                pubkey: token_pubkey,
                signer: None,
                owner: program_id,
                lamports: Rent::default().minimum_balance(token_data.len()),
                data: token_data,
                is_signer: false,
                is_writable: true,
                executable: false,
            },
        );
    }
}

async fn run_external_token_all_public_profile(
    repo_root: &Path,
    call_count: usize,
    assert_full_state: bool,
) -> ExternalAllPublicRun {
    let bpf_dir = repo_root.join("target/deploy");
    std::env::set_var("BPF_OUT_DIR", &bpf_dir);

    let program_id = read_keypair_file(bpf_dir.join("five-keypair.json"))
        .expect("missing target/deploy/five-keypair.json; run `cargo-build-sbf --manifest-path five-solana/Cargo.toml`")
        .pubkey();

    let token_bytecode_path = repo_root.join("five-templates/token/src/token.bin");
    let token_bytecode = fs::read(&token_bytecode_path)
        .unwrap_or_else(|e| panic!("failed reading {}: {}", token_bytecode_path.display(), e));

    let mut accounts = BTreeMap::<String, RuntimeAccount>::new();
    let payer_signer = Keypair::new();
    let payer_pubkey = payer_signer.pubkey();
    accounts.insert(
        "payer".to_string(),
        RuntimeAccount {
            pubkey: payer_pubkey,
            signer: Some(payer_signer),
            owner: system_program::id(),
            lamports: 80_000_000,
            data: vec![],
            is_signer: true,
            is_writable: true,
            executable: false,
        },
    );

    let user1_signer = Keypair::new();
    let user1_pubkey = user1_signer.pubkey();
    accounts.insert(
        "user1".to_string(),
        RuntimeAccount {
            pubkey: user1_pubkey,
            signer: Some(user1_signer),
            owner: system_program::id(),
            lamports: 40_000_000,
            data: vec![],
            is_signer: true,
            is_writable: true,
            executable: false,
        },
    );
    let user2_signer = Keypair::new();
    let user2_pubkey = user2_signer.pubkey();
    accounts.insert(
        "user2".to_string(),
        RuntimeAccount {
            pubkey: user2_pubkey,
            signer: Some(user2_signer),
            owner: system_program::id(),
            lamports: 40_000_000,
            data: vec![],
            is_signer: true,
            is_writable: true,
            executable: false,
        },
    );
    let user3_signer = Keypair::new();
    let user3_pubkey = user3_signer.pubkey();
    accounts.insert(
        "user3".to_string(),
        RuntimeAccount {
            pubkey: user3_pubkey,
            signer: Some(user3_signer),
            owner: system_program::id(),
            lamports: 40_000_000,
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
        vm_state.initialize(payer_pubkey.to_bytes());
        vm_state.deploy_fee_lamports = 0;
        vm_state.execute_fee_lamports = 0;
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

    let token_script_pubkey = Pubkey::new_unique();
    accounts.insert(
        "token_script".to_string(),
        RuntimeAccount {
            pubkey: token_script_pubkey,
            signer: None,
            owner: program_id,
            lamports: Rent::default().minimum_balance(ScriptAccountHeader::LEN + token_bytecode.len()),
            data: vec![0u8; ScriptAccountHeader::LEN + token_bytecode.len()],
            is_signer: false,
            is_writable: false,
            executable: false,
        },
    );

    seed_external_token_all_public_accounts(
        &mut accounts,
        program_id,
        user1_pubkey,
        user2_pubkey,
        user3_pubkey,
    );

    let token_import_address = bs58::encode(token_script_pubkey.to_bytes()).into_string();
    let caller_source = build_external_all_public_caller_source(&token_import_address, call_count);
    let caller_bytecode =
        DslCompiler::compile_dsl(&caller_source).expect("external all-public caller should compile");
    accounts.insert(
        "caller_script".to_string(),
        RuntimeAccount {
            pubkey: Pubkey::new_unique(),
            signer: None,
            owner: program_id,
            lamports: Rent::default().minimum_balance(ScriptAccountHeader::LEN + caller_bytecode.len()),
            data: vec![0u8; ScriptAccountHeader::LEN + caller_bytecode.len()],
            is_signer: false,
            is_writable: true,
            executable: false,
        },
    );

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
    let mut ctx = program_test.start_with_context().await;

    let deploy_token_ix = build_deploy_instruction(
        program_id,
        &accounts,
        "token_script",
        "vm_state",
        "payer",
        &token_bytecode,
        0,
    );
    let deploy_token = simulate_and_process(
        &mut ctx,
        vec![deploy_token_ix],
        collect_signers(&accounts, &["payer"]),
        Some(1_400_000),
    )
    .await;
    assert!(deploy_token.success, "token deploy failed: {:?}", deploy_token.error);

    let deploy_caller_ix = build_deploy_instruction(
        program_id,
        &accounts,
        "caller_script",
        "vm_state",
        "payer",
        &caller_bytecode,
        0,
    );
    let deploy_caller = simulate_and_process(
        &mut ctx,
        vec![deploy_caller_ix],
        collect_signers(&accounts, &["payer"]),
        Some(1_400_000),
    )
    .await;
    assert!(deploy_caller.success, "caller deploy failed: {:?}", deploy_caller.error);

    let step = StepFixture {
        name: "external_all_public_non_cpi".to_string(),
        function_index: 0,
        extras: vec![
            "mint_account".to_string(),
            "user1_token".to_string(),
            "user2_token".to_string(),
            "user3_token".to_string(),
            "user1".to_string(),
            "user2".to_string(),
            "user3".to_string(),
            "token_script".to_string(),
        ],
        params: vec![
            ParamFixture::AccountRef {
                account: "mint_account".to_string(),
            },
            ParamFixture::AccountRef {
                account: "user1_token".to_string(),
            },
            ParamFixture::AccountRef {
                account: "user2_token".to_string(),
            },
            ParamFixture::AccountRef {
                account: "user3_token".to_string(),
            },
            ParamFixture::AccountRef {
                account: "user1".to_string(),
            },
            ParamFixture::AccountRef {
                account: "user2".to_string(),
            },
            ParamFixture::AccountRef {
                account: "user3".to_string(),
            },
            ParamFixture::AccountRef {
                account: "token_script".to_string(),
            },
            ParamFixture::PubkeyAccount {
                account: "user2".to_string(),
            },
            ParamFixture::PubkeyAccount {
                account: "user2".to_string(),
            },
        ],
        expected: ExpectedFixture::Success,
    };

    let execute_ix = build_execute_instruction(
        program_id,
        &accounts,
        "caller_script",
        "vm_state",
        &step,
        build_payload(&accounts, &step),
    );
    let execute = simulate_and_process(
        &mut ctx,
        vec![execute_ix],
        collect_signers(
            &accounts,
            &["user1", "user2", "user3"],
        ),
        Some(1_400_000),
    )
    .await;
    assert!(
        execute.success,
        "external all-public execution failed (call_count={}): {:?}",
        call_count,
        execute.error
    );

    if assert_full_state {
        let mint_account_pk = accounts["mint_account"].pubkey;
        let user1_token_pk = accounts["user1_token"].pubkey;
        let user2_token_pk = accounts["user2_token"].pubkey;
        let user3_token_pk = accounts["user3_token"].pubkey;

        let mint_after = ctx
            .banks_client
            .get_account(mint_account_pk)
            .await
            .expect("fetch mint account")
            .expect("mint account missing");
        let user1_after = ctx
            .banks_client
            .get_account(user1_token_pk)
            .await
            .expect("fetch user1 token account")
            .expect("user1 token account missing");
        let user2_after = ctx
            .banks_client
            .get_account(user2_token_pk)
            .await
            .expect("fetch user2 token account")
            .expect("user2 token account missing");
        let user3_after = ctx
            .banks_client
            .get_account(user3_token_pk)
            .await
            .expect("fetch user3 token account")
            .expect("user3 token account missing");

        let supply = u64::from_le_bytes(mint_after.data[64..72].try_into().unwrap());
        let user1_balance = u64::from_le_bytes(user1_after.data[64..72].try_into().unwrap());
        let user2_balance = u64::from_le_bytes(user2_after.data[64..72].try_into().unwrap());
        let user3_balance = u64::from_le_bytes(user3_after.data[64..72].try_into().unwrap());
        assert_eq!(supply, 1900);
        assert_eq!(user1_balance, 950);
        assert_eq!(user2_balance, 400);
        assert_eq!(user3_balance, 550);

        // Mint authority and freeze authority are zeroed by disable_* calls.
        assert_eq!(&mint_after.data[0..32], &[0u8; 32]);
        assert_eq!(&mint_after.data[32..64], &[0u8; 32]);
        // user2 was frozen then thawed.
        assert_eq!(user2_after.data[72], 0);
    }

    ExternalAllPublicRun {
        deploy_token_units: deploy_token.units_consumed,
        deploy_caller_units: deploy_caller.units_consumed,
        execute_units: execute.units_consumed,
        caller_bytecode_size: caller_bytecode.len(),
        token_bytecode_size: token_bytecode.len(),
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
        let is_external_script = name != script_name && name.ends_with("_script");
        metas.push(AccountMeta {
            pubkey: a.pubkey,
            is_signer: a.is_signer,
            // Imported bytecode accounts must be read-only during execution.
            is_writable: if is_external_script { false } else { a.is_writable },
        });
    }

    Instruction {
        program_id,
        accounts: metas,
        data,
    }
}

fn build_execute_instruction_with_extras(
    program_id: Pubkey,
    accounts: &BTreeMap<String, RuntimeAccount>,
    script_name: &str,
    vm_state_name: &str,
    extras: &[String],
    payload: Vec<u8>,
) -> Instruction {
    let mut data = Vec::with_capacity(1 + payload.len());
    data.push(EXECUTE_INSTRUCTION);
    data.extend_from_slice(&payload);

    let mut metas = vec![
        AccountMeta::new(accounts[script_name].pubkey, false),
        AccountMeta::new(accounts[vm_state_name].pubkey, false),
    ];
    for name in extras {
        let a = &accounts[name];
        let is_external_script = name != script_name && name.ends_with("_script");
        metas.push(AccountMeta {
            pubkey: a.pubkey,
            is_signer: a.is_signer,
            // Imported bytecode accounts must be read-only during execution.
            is_writable: if is_external_script { false } else { a.is_writable },
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
