mod harness;

use five::state::{FIVEVMState, ScriptAccountHeader};
use five_protocol::opcodes::HALT;
use harness::fixtures::canonical_execute_payload;
use harness::{AccountSeed, RuntimeHarness, script_with_header, unique_pubkey};
use pinocchio::program_error::ProgramError;

fn canonical_vm_state_seed(program_id: pinocchio::pubkey::Pubkey) -> AccountSeed {
    let (vm_state, _vm_bump) =
        five_vm_mito::utils::find_program_address_offchain(&[b"vm_state"], &program_id)
            .expect("derive canonical vm_state");
    AccountSeed {
        key: vm_state,
        owner: program_id,
        lamports: 0,
        data: vec![0u8; FIVEVMState::LEN],
        is_signer: false,
        is_writable: true,
        executable: false,
    }
}

#[test]
fn execute_charges_fee_to_fee_vault_without_validator() {
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

    rt.add_account("vm_state", canonical_vm_state_seed(program_id));
    rt.init_vm_state("vm_state", "admin");
    rt.set_vm_fees("vm_state", 0, 200);

    let bytecode = script_with_header(1, 1, &[HALT]);
    rt.add_account("script", RuntimeHarness::create_script_account_seed(program_id, bytecode.len()));

    let deploy = rt.deploy_script("script", "vm_state", "payer", &bytecode, 0, None);
    assert!(deploy.success, "deploy failed: {:?}", deploy.error);

    let payload = canonical_execute_payload(0, &[]);
    let before_payer = rt.fetch_account("payer").lamports;
    let before_vault = rt.fetch_account("fee_vault").lamports;

    let execute = rt.execute_script(
        "script",
        "vm_state",
        &["payer", "fee_vault", "system_program"],
        &payload,
    );
    assert!(execute.success, "execute failed: {:?}", execute.error);

    let after_payer = rt.fetch_account("payer").lamports;
    let after_vault = rt.fetch_account("fee_vault").lamports;
    assert!(after_payer < before_payer, "payer should be charged a fee");
    assert!(after_vault > before_vault, "fee vault should receive a fee");
}

#[test]
fn execute_fails_when_fee_tail_accounts_missing() {
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

    rt.add_account("vm_state", canonical_vm_state_seed(program_id));
    rt.init_vm_state("vm_state", "admin");
    rt.set_vm_fees("vm_state", 0, 100);

    let bytecode = script_with_header(1, 1, &[HALT]);
    rt.add_account("script", RuntimeHarness::create_script_account_seed(program_id, bytecode.len()));

    let deploy = rt.deploy_script("script", "vm_state", "payer", &bytecode, 0, None);
    assert!(deploy.success, "deploy failed: {:?}", deploy.error);

    let payload = canonical_execute_payload(0, &[]);
    let execute = rt.execute_script("script", "vm_state", &["payer"], &payload);
    assert_eq!(execute.error, Some(ProgramError::NotEnoughAccountKeys));
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

    // Deliberately uninitialized VM state, but with current version stamped.
    let mut vm_state_seed = canonical_vm_state_seed(program_id);
    {
        let vm_state = FIVEVMState::from_account_data_mut(&mut vm_state_seed.data)
            .expect("vm_state layout");
        vm_state.version = FIVEVMState::VERSION;
        vm_state.is_initialized = 0;
    }
    rt.add_account("vm_state", vm_state_seed);

    let bytecode = script_with_header(1, 1, &[HALT]);
    rt.add_account("script", RuntimeHarness::create_script_account_seed(program_id, bytecode.len()));
    let (fee_vault, _fee_vault_bump) = five_vm_mito::utils::find_program_address_offchain(
        &[b"\xFFfive_vm_fee_vault_v1", &[0u8]],
        &program_id,
    )
    .expect("derive fee vault");
    rt.add_account(
        "fee_vault",
        AccountSeed {
            key: fee_vault,
            owner: program_id,
            lamports: 0,
            data: vec![],
            is_signer: false,
            is_writable: true,
            executable: false,
        },
    );
    rt.add_account(
        "system_program",
        AccountSeed {
            key: pinocchio::pubkey::Pubkey::default(),
            owner: pinocchio::pubkey::Pubkey::default(),
            lamports: 0,
            data: vec![],
            is_signer: false,
            is_writable: false,
            executable: false,
        },
    );

    // Write a valid script header+bytecode manually so execute reaches validation path.
    let mut script_bytes = rt.fetch_account("script").data;
    let header = ScriptAccountHeader::create_from_bytecode(&bytecode, unique_pubkey(32), 0, 0);
    header.copy_into_account(&mut script_bytes).unwrap();
    script_bytes[ScriptAccountHeader::LEN..ScriptAccountHeader::LEN + bytecode.len()]
        .copy_from_slice(&bytecode);
    rt.set_account_data("script", script_bytes);

    let payload = canonical_execute_payload(0, &[]);
    let execute = rt.execute_script(
        "script",
        "vm_state",
        &["payer", "fee_vault", "system_program"],
        &payload,
    );

    let vm_state_data = rt.fetch_account("vm_state").data;
    let state = FIVEVMState::from_account_data(&vm_state_data).expect("version-stamped vm_state");
    assert!(!state.is_initialized());
    assert!(!execute.success);
    assert_eq!(execute.error, Some(ProgramError::Custom(1022)));
}
