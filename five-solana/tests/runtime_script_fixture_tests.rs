mod harness;

use five_dsl_compiler::DslCompiler;
use five_protocol::opcodes::{HALT, PUSH_BOOL, REQUIRE};
use harness::fixtures::{TypedParam, canonical_execute_payload};
use harness::{AccountSeed, ExpectedOutcome, RuntimeHarness, ScriptFixture, script_with_header, unique_pubkey};

#[test]
fn fixture_runner_handles_success_and_failure_cases() {
    let program_id = unique_pubkey(41);
    let mut rt = RuntimeHarness::start(program_id);

    rt.add_account(
        "owner",
        AccountSeed {
            key: unique_pubkey(42),
            owner: program_id,
            lamports: 10_000,
            data: vec![],
            is_signer: true,
            is_writable: true,
            executable: false,
        },
    );
    rt.add_account("vm_state", RuntimeHarness::create_vm_state_seed(program_id));
    rt.init_vm_state("vm_state", "owner");
    rt.add_account("script", RuntimeHarness::create_script_account_seed(program_id, 256));

    let success_fixture = ScriptFixture {
        bytecode: script_with_header(1, 1, &[HALT]),
        permissions: 0,
        execute_payload: canonical_execute_payload(0, &[TypedParam::U64(42)]),
        initial_accounts: vec![],
        expectation: ExpectedOutcome::Success,
    };
    let success = rt.run_fixture(&success_fixture, "script", "vm_state", "owner");
    assert!(success.success);

    rt.add_account(
        "script_fail",
        RuntimeHarness::create_script_account_seed(program_id, 256),
    );
    let fail_deploy = rt.deploy_script(
        "script_fail",
        "vm_state",
        "owner",
        &script_with_header(1, 1, &[PUSH_BOOL, 0, REQUIRE, HALT]),
        0,
        None,
    );
    assert!(fail_deploy.success, "fixture deploy should succeed");
    let failure = rt.execute_script(
        "script_fail",
        "vm_state",
        &["owner"],
        &canonical_execute_payload(0, &[]),
    );
    assert!(!failure.success);
}

#[test]
fn fixture_runner_accepts_string_heavy_execute_payload() {
    let program_id = unique_pubkey(51);
    let mut rt = RuntimeHarness::start(program_id);

    rt.add_account(
        "owner",
        AccountSeed {
            key: unique_pubkey(52),
            owner: program_id,
            lamports: 10_000,
            data: vec![],
            is_signer: true,
            is_writable: true,
            executable: false,
        },
    );
    rt.add_account("vm_state", RuntimeHarness::create_vm_state_seed(program_id));
    rt.init_vm_state("vm_state", "owner");

    let bytecode = script_with_header(1, 1, &[HALT]);
    rt.add_account("script", RuntimeHarness::create_script_account_seed(program_id, bytecode.len()));

    let deploy = rt.deploy_script("script", "vm_state", "owner", &bytecode, 0, None);
    assert!(deploy.success, "deploy failed: {:?}", deploy.error);

    let payload = canonical_execute_payload(
        0,
        &[
            TypedParam::String("TestToken".to_string()),
            TypedParam::String("TEST".to_string()),
            TypedParam::String("https://example.com/token".to_string()),
        ],
    );

    let result = rt.execute_script("script", "vm_state", &["owner"], &payload);
    assert!(result.success, "string-heavy payload should execute through canonical envelope: {:?}", result.error);
}

#[test]
fn harness_executes_external_token_transfer_without_cpi() {
    let program_id = unique_pubkey(71);
    let mut rt = RuntimeHarness::start(program_id);

    rt.add_account(
        "owner",
        AccountSeed {
            key: unique_pubkey(72),
            owner: program_id,
            lamports: 10_000_000,
            data: vec![],
            is_signer: true,
            is_writable: true,
            executable: false,
        },
    );
    rt.add_account("vm_state", RuntimeHarness::create_vm_state_seed(program_id));
    rt.init_vm_state("vm_state", "owner");

    let token_bytecode = std::fs::read(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../five-templates/token/src/token.bin"),
    )
    .expect("token.bin fixture should exist");
    let token_script_key = unique_pubkey(76);
    rt.add_account(
        "token_script",
        AccountSeed {
            key: token_script_key,
            owner: program_id,
            lamports: 0,
            data: vec![0u8; five::state::ScriptAccountHeader::LEN + token_bytecode.len()],
            is_signer: false,
            is_writable: false,
            executable: false,
        },
    );

    let token_deploy = rt.deploy_script(
        "token_script",
        "vm_state",
        "owner",
        &token_bytecode,
        0,
        None,
    );
    assert!(
        token_deploy.success,
        "token deploy should succeed: {:?}",
        token_deploy.error
    );

    let token_import_address = bs58::encode(token_script_key).into_string();
    let caller_source = format!(
        r#"
        use "{token_import_address}"::{{transfer}};

        pub fn call_transfer(
            source_account: account @mut,
            destination_account: account @mut,
            owner: account @mut,
            token_bytecode: account
        ) {{
            transfer(source_account, destination_account, owner, 50);
        }}
    "#
    );
    let repo_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    harness::compile::maybe_write_generated_v(
        &repo_root,
        "generated/runtime-script-fixture-transfer-caller.v",
        &caller_source,
    );
    let caller_bytecode =
        DslCompiler::compile_dsl(&caller_source).expect("caller script should compile");
    assert!(
        caller_bytecode
            .iter()
            .any(|op| *op == five_protocol::opcodes::CALL_EXTERNAL),
        "caller bytecode should contain CALL_EXTERNAL"
    );

    rt.add_account(
        "caller_script",
        RuntimeHarness::create_script_account_seed(program_id, caller_bytecode.len()),
    );
    let caller_deploy = rt.deploy_script(
        "caller_script",
        "vm_state",
        "owner",
        &caller_bytecode,
        0,
        None,
    );
    assert!(
        caller_deploy.success,
        "caller deploy should succeed: {:?}",
        caller_deploy.error
    );

    let mint_key = unique_pubkey(73);
    let source_key = unique_pubkey(74);
    let destination_key = unique_pubkey(75);

    let mut source_data = vec![0u8; 192];
    source_data[0..32].copy_from_slice(rt.fetch_account("owner").key.as_ref());
    source_data[32..64].copy_from_slice(mint_key.as_ref());
    source_data[64..72].copy_from_slice(&500u64.to_le_bytes());
    source_data[72] = 0;

    let mut destination_data = vec![0u8; 192];
    destination_data[0..32].copy_from_slice(destination_key.as_ref());
    destination_data[32..64].copy_from_slice(mint_key.as_ref());
    destination_data[64..72].copy_from_slice(&100u64.to_le_bytes());
    destination_data[72] = 0;

    rt.add_account(
        "source_token",
        AccountSeed {
            key: source_key,
            owner: program_id,
            lamports: 0,
            data: source_data,
            is_signer: false,
            is_writable: true,
            executable: false,
        },
    );
    rt.add_account(
        "destination_token",
        AccountSeed {
            key: destination_key,
            owner: program_id,
            lamports: 0,
            data: destination_data,
            is_signer: false,
            is_writable: true,
            executable: false,
        },
    );

    let payload = canonical_execute_payload(
        0,
        &[
            TypedParam::Account(1),
            TypedParam::Account(2),
            TypedParam::Account(3),
            TypedParam::Account(4),
        ],
    );
    let execute = rt.execute_script(
        "caller_script",
        "vm_state",
        &["source_token", "destination_token", "owner", "token_script"],
        &payload,
    );
    let cu = five_vm_mito::MitoVM::last_compute_units_consumed();
    println!(
        "external_call_metrics: caller_bytecode_len={} token_bytecode_len={} cu={}",
        caller_bytecode.len(),
        token_bytecode.len(),
        cu
    );
    assert!(
        execute.success,
        "external transfer execution should succeed: {:?}",
        execute.error
    );

    let source_after = rt.fetch_account("source_token");
    let destination_after = rt.fetch_account("destination_token");
    assert_eq!(
        read_u64(&source_after.data, 64),
        450,
        "source balance should decrease via external transfer"
    );
    assert_eq!(
        read_u64(&destination_after.data, 64),
        150,
        "destination balance should increase via external transfer"
    );
}

fn read_u64(data: &[u8], offset: usize) -> u64 {
    let end = offset + 8;
    let bytes = data
        .get(offset..end)
        .expect("read_u64 offset should be in-bounds");
    let mut arr = [0u8; 8];
    arr.copy_from_slice(bytes);
    u64::from_le_bytes(arr)
}
