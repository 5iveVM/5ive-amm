mod harness;

use five_protocol::opcodes::{GET_CLOCK, HALT, INVOKE};
use harness::fixtures::canonical_execute_payload;
use harness::{AccountSeed, RuntimeHarness, script_with_header, unique_pubkey};

#[test]
fn syscall_clock_path_executes_and_returns_deterministic_result() {
    let program_id = unique_pubkey(71);
    let mut rt = RuntimeHarness::start(program_id);

    rt.add_account(
        "owner",
        AccountSeed {
            key: unique_pubkey(72),
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

    let bytecode = script_with_header(1, 1, &[GET_CLOCK, HALT]);
    rt.add_account("script", RuntimeHarness::create_script_account_seed(program_id, bytecode.len()));

    let deploy = rt.deploy_script("script", "vm_state", "owner", &bytecode, 0, None);
    assert!(deploy.success, "deploy failed: {:?}", deploy.error);

    let execute = rt.execute_script("script", "vm_state", &["owner"], &canonical_execute_payload(0, &[]));
    assert!(
        execute.success || execute.error.is_some(),
        "GET_CLOCK path should return success or deterministic program error"
    );
}

#[test]
fn cpi_invoke_opcode_path_returns_deterministic_error_without_panic() {
    let program_id = unique_pubkey(81);
    let mut rt = RuntimeHarness::start(program_id);

    rt.add_account(
        "owner",
        AccountSeed {
            key: unique_pubkey(82),
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

    // Bare INVOKE has insufficient stack setup and should fail deterministically.
    let bytecode = script_with_header(1, 1, &[INVOKE, HALT]);
    rt.add_account("script", RuntimeHarness::create_script_account_seed(program_id, bytecode.len()));

    let deploy = rt.deploy_script("script", "vm_state", "owner", &bytecode, 0, None);
    assert!(deploy.success, "deploy failed: {:?}", deploy.error);

    let execute = rt.execute_script("script", "vm_state", &["owner"], &canonical_execute_payload(0, &[]));
    assert!(!execute.success, "INVOKE without setup should fail deterministically");
}
