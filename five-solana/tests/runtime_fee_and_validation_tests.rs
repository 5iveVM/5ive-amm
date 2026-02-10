mod harness;

use five::state::{FIVEVMState, ScriptAccountHeader};
use five_protocol::opcodes::HALT;
use harness::fixtures::canonical_execute_payload;
use harness::{AccountSeed, RuntimeHarness, script_with_header, unique_pubkey};
use pinocchio::program_error::ProgramError;

#[test]
fn execute_charges_fee_to_admin_without_validator() {
    let program_id = unique_pubkey(11);
    let mut rt = RuntimeHarness::start(program_id);

    rt.add_account(
        "admin",
        AccountSeed {
            key: unique_pubkey(12),
            owner: program_id,
            lamports: 0,
            data: vec![],
            is_signer: false,
            is_writable: true,
            executable: false,
        },
    );

    rt.add_account(
        "payer",
        AccountSeed {
            key: unique_pubkey(13),
            owner: program_id,
            lamports: 20_000,
            data: vec![],
            is_signer: true,
            is_writable: true,
            executable: false,
        },
    );

    rt.add_account("vm_state", RuntimeHarness::create_vm_state_seed(program_id));
    rt.init_vm_state("vm_state", "admin");
    rt.set_vm_fees("vm_state", 0, 200);

    let bytecode = script_with_header(1, 1, &[HALT]);
    rt.add_account("script", RuntimeHarness::create_script_account_seed(program_id, bytecode.len()));

    let deploy = rt.deploy_script("script", "vm_state", "payer", &bytecode, 0, None);
    assert!(deploy.success, "deploy failed: {:?}", deploy.error);

    let payload = canonical_execute_payload(0, &[]);
    let before_payer = rt.fetch_account("payer").lamports;
    let before_admin = rt.fetch_account("admin").lamports;

    let execute = rt.execute_script("script", "vm_state", &["payer", "admin"], &payload);
    assert!(execute.success, "execute failed: {:?}", execute.error);

    let after_payer = rt.fetch_account("payer").lamports;
    let after_admin = rt.fetch_account("admin").lamports;
    assert!(after_payer < before_payer, "payer should be charged a fee");
    assert!(after_admin > before_admin, "admin should receive a fee");
}

#[test]
fn execute_fails_when_admin_account_missing_for_fee_collection() {
    let program_id = unique_pubkey(21);
    let mut rt = RuntimeHarness::start(program_id);

    rt.add_account(
        "admin",
        AccountSeed {
            key: unique_pubkey(22),
            owner: program_id,
            lamports: 0,
            data: vec![],
            is_signer: false,
            is_writable: true,
            executable: false,
        },
    );

    rt.add_account(
        "payer",
        AccountSeed {
            key: unique_pubkey(23),
            owner: program_id,
            lamports: 10_000,
            data: vec![],
            is_signer: true,
            is_writable: true,
            executable: false,
        },
    );

    rt.add_account("vm_state", RuntimeHarness::create_vm_state_seed(program_id));
    rt.init_vm_state("vm_state", "admin");
    rt.set_vm_fees("vm_state", 0, 100);

    let bytecode = script_with_header(1, 1, &[HALT]);
    rt.add_account("script", RuntimeHarness::create_script_account_seed(program_id, bytecode.len()));

    let deploy = rt.deploy_script("script", "vm_state", "payer", &bytecode, 0, None);
    assert!(deploy.success, "deploy failed: {:?}", deploy.error);

    let payload = canonical_execute_payload(0, &[]);
    let execute = rt.execute_script("script", "vm_state", &["payer"], &payload);
    assert_eq!(execute.error, Some(ProgramError::Custom(1107)));
}

#[test]
fn execute_fails_for_uninitialized_vm_state() {
    let program_id = unique_pubkey(31);
    let mut rt = RuntimeHarness::start(program_id);

    rt.add_account(
        "payer",
        AccountSeed {
            key: unique_pubkey(32),
            owner: program_id,
            lamports: 10_000,
            data: vec![],
            is_signer: true,
            is_writable: true,
            executable: false,
        },
    );

    // Deliberately uninitialized VM state.
    rt.add_account("vm_state", RuntimeHarness::create_vm_state_seed(program_id));

    let bytecode = script_with_header(1, 1, &[HALT]);
    rt.add_account("script", RuntimeHarness::create_script_account_seed(program_id, bytecode.len()));

    // Write a valid script header+bytecode manually so execute reaches validation path.
    let mut script_bytes = rt.fetch_account("script").data;
    let header = ScriptAccountHeader::create_from_bytecode(&bytecode, unique_pubkey(32), 0, 0);
    header.copy_into_account(&mut script_bytes).unwrap();
    script_bytes[ScriptAccountHeader::LEN..ScriptAccountHeader::LEN + bytecode.len()]
        .copy_from_slice(&bytecode);
    rt.set_account_data("script", script_bytes);

    let payload = canonical_execute_payload(0, &[]);
    let execute = rt.execute_script("script", "vm_state", &["payer"], &payload);

    let vm_state_data = rt.fetch_account("vm_state").data;
    let state = FIVEVMState::from_account_data(&vm_state_data).unwrap();
    assert!(!state.is_initialized());
    assert!(!execute.success);
}
