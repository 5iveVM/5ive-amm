mod harness;

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

    let fail_deploy = rt.deploy_script(
        "script",
        "vm_state",
        "owner",
        &script_with_header(1, 1, &[PUSH_BOOL, 0, REQUIRE, HALT]),
        0,
        None,
    );
    assert!(fail_deploy.success, "fixture deploy should succeed");
    let failure = rt.execute_script("script", "vm_state", &["owner"], &canonical_execute_payload(0, &[]));
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
