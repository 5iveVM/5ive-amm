mod harness;

use std::fs;
use std::path::{Path, PathBuf};

use harness::fixtures::{TypedParam, canonical_execute_payload};
use harness::{AccountSeed, RuntimeHarness, unique_pubkey};
use pinocchio::pubkey;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct RuntimeFixture {
    name: String,
    #[serde(default = "default_program_seed")]
    program_seed: u8,
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
    #[serde(default)]
    final_assertions: Vec<AssertionFixture>,
}

#[derive(Debug, Deserialize)]
struct AuthorityFixture {
    name: String,
    #[serde(default = "default_authority_key_seed")]
    key_seed: u8,
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
    key_seed: Option<u8>,
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum AccountOwner {
    Program,
    System,
    Authority,
    SelfAccount,
    SplTokenProgram,
    AnchorTokenProgram,
}

#[derive(Debug, Deserialize)]
struct ExternalProgramFixture {
    kind: String,
}

#[derive(Debug, Deserialize)]
struct StepFixture {
    name: String,
    function_index: u32,
    #[serde(default)]
    extras: Vec<String>,
    #[serde(default)]
    params: Vec<ParamFixture>,
    expected: ExpectedFixture,
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ExpectedFixture {
    Success,
    Error,
    SuccessOrError,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum AssertionFixture {
    AccountU64Le {
        account: String,
        offset: usize,
        value: u64,
    },
    AccountByte {
        account: String,
        offset: usize,
        value: u8,
    },
    AccountPubkeyEqualsAccount {
        account: String,
        offset: usize,
        equals_account: String,
    },
    AccountPubkeyIsZero {
        account: String,
        offset: usize,
    },
}

fn default_authority_lamports() -> u64 {
    200_000
}

fn default_program_seed() -> u8 {
    42
}

fn default_authority_key_seed() -> u8 {
    99
}

#[test]
fn template_runtime_fixtures_execute_without_localnet() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let fixture_files = discover_fixture_files(&repo_root.join("five-templates"));
    assert!(
        !fixture_files.is_empty(),
        "no runtime fixtures found under five-templates/**/runtime-fixtures/*.json"
    );

    let filter = std::env::var("FIVE_TEMPLATE_FILTER").ok();
    let mut matched = 0usize;
    let mut ran = 0usize;
    let mut skipped = 0usize;

    for fixture_file in fixture_files {
        let fixture_path_str = fixture_file.to_string_lossy();
        if let Some(f) = &filter {
            if !fixture_path_str.contains(f) {
                continue;
            }
        }

        matched += 1;
        let fixture = load_fixture(&fixture_file);
        if fixture.skip_deploy || !fixture.external_programs.is_empty() {
            skipped += 1;
            continue;
        }
        run_fixture(&repo_root, &fixture_file, &fixture);
        ran += 1;
    }

    assert!(matched > 0, "no fixtures matched filter");
    assert!(
        ran > 0,
        "all matched fixtures were skipped (skip_deploy/external_programs): matched={}, skipped={}",
        matched,
        skipped
    );
}

fn run_fixture(repo_root: &Path, fixture_path: &Path, fixture: &RuntimeFixture) {
    assert!(
        !fixture.steps.is_empty(),
        "fixture {} must contain at least one step",
        fixture.name
    );

    let program_id = unique_pubkey(fixture.program_seed);
    let (canonical_vm_state, _) = pubkey::find_program_address(&[b"vm_state"], &program_id);
    let mut rt = RuntimeHarness::start(program_id);

    rt.add_account(
        &fixture.authority.name,
        AccountSeed {
            key: unique_pubkey(fixture.authority.key_seed),
            // Fee payer must be a system-owned account.
            owner: [0u8; 32],
            lamports: fixture.authority.lamports,
            data: vec![],
            is_signer: true,
            is_writable: true,
            executable: false,
        },
    );

    rt.add_account(
        &fixture.vm_state_name,
        AccountSeed {
            key: canonical_vm_state,
            owner: program_id,
            lamports: 0,
            data: vec![0u8; five::state::FIVEVMState::LEN],
            is_signer: false,
            is_writable: true,
            executable: false,
        },
    );
    rt.init_vm_state(&fixture.vm_state_name, &fixture.authority.name);

    if let Some(fees) = &fixture.vm_fees {
        rt.set_vm_fees(
            &fixture.vm_state_name,
            fees.deploy_fee_lamports,
            fees.execute_fee_lamports,
        );
    }

    let bytecode_path = resolve_bytecode_path(repo_root, fixture_path, &fixture.bytecode_path);
    let bytecode = harness::compile::load_or_compile_bytecode(&bytecode_path)
        .unwrap_or_else(|e| panic!("failed loading bytecode for {}: {}", fixture.name, e));

    rt.add_account(
        &fixture.script_name,
        AccountSeed {
            key: unique_pubkey(fixture.program_seed.wrapping_add(2)),
            owner: program_id,
            lamports: 0,
            data: vec![0u8; five::state::ScriptAccountHeader::LEN + bytecode.len()],
            is_signer: false,
            is_writable: true,
            executable: false,
        },
    );

    for (idx, account) in fixture.extra_accounts.iter().enumerate() {
        let key = match &account.pubkey {
            Some(value) => decode_pubkey(value),
            None => unique_pubkey(
                account
                    .key_seed
                    .unwrap_or_else(|| fixture.program_seed.wrapping_add(10).wrapping_add(idx as u8)),
            ),
        };
        let owner = match account.owner {
            AccountOwner::Program => program_id,
            AccountOwner::System => [0u8; 32],
            AccountOwner::Authority => rt.fetch_account(&fixture.authority.name).key,
            AccountOwner::SelfAccount => key,
            AccountOwner::SplTokenProgram => [0u8; 32],
            AccountOwner::AnchorTokenProgram => [0u8; 32],
        };
        rt.add_account(
            &account.name,
            AccountSeed {
                key,
                owner,
                lamports: account.lamports,
                data: vec![0u8; account.data_len],
                is_signer: account.is_signer,
                is_writable: account.is_writable,
                executable: account.executable,
            },
        );
    }

    let deploy = rt.deploy_script(
        &fixture.script_name,
        &fixture.vm_state_name,
        &fixture.authority.name,
        &bytecode,
        fixture.permissions,
        None,
    );
    assert!(
        deploy.success,
        "fixture {} deploy failed: {:?}",
        fixture.name,
        deploy.error
    );

    for (idx, step) in fixture.steps.iter().enumerate() {
        let payload = build_payload(&rt, step);
        let extras: Vec<&str> = step.extras.iter().map(String::as_str).collect();
        let execute = rt.execute_script(&fixture.script_name, &fixture.vm_state_name, &extras, &payload);

        match step.expected {
            ExpectedFixture::Success => assert!(
                execute.success,
                "fixture {} step {} ({}) expected success, got {:?}",
                fixture.name,
                idx,
                step.name,
                execute.error
            ),
            ExpectedFixture::Error => assert!(
                execute.error.is_some(),
                "fixture {} step {} ({}) expected deterministic error, got success",
                fixture.name,
                idx,
                step.name
            ),
            ExpectedFixture::SuccessOrError => assert!(
                execute.success || execute.error.is_some(),
                "fixture {} step {} ({}) expected deterministic completion",
                fixture.name,
                idx,
                step.name
            ),
        }
    }

    run_assertions(&rt, fixture);
}

fn run_assertions(rt: &RuntimeHarness, fixture: &RuntimeFixture) {
    for assertion in &fixture.final_assertions {
        match assertion {
            AssertionFixture::AccountU64Le {
                account,
                offset,
                value,
            } => {
                let snapshot = rt.fetch_account(account);
                let bytes = read_slice(&snapshot.data, *offset, 8, account, "u64");
                let mut arr = [0u8; 8];
                arr.copy_from_slice(bytes);
                let actual = u64::from_le_bytes(arr);
                assert_eq!(
                    actual, *value,
                    "fixture {} assertion failed: account {} u64@{} expected {}, got {}",
                    fixture.name, account, offset, value, actual
                );
            }
            AssertionFixture::AccountByte {
                account,
                offset,
                value,
            } => {
                let snapshot = rt.fetch_account(account);
                let bytes = read_slice(&snapshot.data, *offset, 1, account, "byte");
                assert_eq!(
                    bytes[0], *value,
                    "fixture {} assertion failed: account {} byte@{} expected {}, got {}",
                    fixture.name, account, offset, value, bytes[0]
                );
            }
            AssertionFixture::AccountPubkeyEqualsAccount {
                account,
                offset,
                equals_account,
            } => {
                let snapshot = rt.fetch_account(account);
                let expected = rt.fetch_account(equals_account).key;
                let bytes = read_slice(&snapshot.data, *offset, 32, account, "pubkey");
                let mut actual = [0u8; 32];
                actual.copy_from_slice(bytes);
                assert_eq!(
                    actual, expected,
                    "fixture {} assertion failed: account {} pubkey@{} expected key of {}",
                    fixture.name, account, offset, equals_account
                );
            }
            AssertionFixture::AccountPubkeyIsZero { account, offset } => {
                let snapshot = rt.fetch_account(account);
                let bytes = read_slice(&snapshot.data, *offset, 32, account, "pubkey");
                assert!(
                    bytes.iter().all(|b| *b == 0),
                    "fixture {} assertion failed: account {} pubkey@{} expected zero",
                    fixture.name,
                    account,
                    offset
                );
            }
        }
    }
}

fn read_slice<'a>(data: &'a [u8], offset: usize, len: usize, account: &str, what: &str) -> &'a [u8] {
    let end = offset.saturating_add(len);
    assert!(
        end <= data.len(),
        "assertion read out of bounds for account {}: {} at [{}..{}) with len {}",
        account,
        what,
        offset,
        end,
        data.len()
    );
    &data[offset..end]
}

fn build_payload(rt: &RuntimeHarness, step: &StepFixture) -> Vec<u8> {
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
                params.push(TypedParam::Pubkey(rt.fetch_account(account).key));
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
        .unwrap_or_else(|| {
            panic!(
                "step `{}` account ref `{}` not found in extras {:?}",
                step.name, account, step.extras
            )
        });

    // MitoVM account index 0 is the vm_state account.
    // Step extras are appended after vm_state at index 1+.
    (pos as u8) + 1
}

fn decode_pubkey(encoded: &str) -> [u8; 32] {
    let bytes = bs58::decode(encoded)
        .into_vec()
        .unwrap_or_else(|_| panic!("invalid fixture pubkey: {}", encoded));
    assert_eq!(
        bytes.len(),
        32,
        "fixture pubkey must decode to 32 bytes: {}",
        encoded
    );
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    out
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

fn discover_fixture_files(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    recurse_fixture_files(root, &mut out);
    out.sort();
    out
}

fn recurse_fixture_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            recurse_fixture_files(&path, out);
            continue;
        }

        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }

        if !path.to_string_lossy().contains("runtime-fixtures") {
            continue;
        }

        out.push(path);
    }
}
