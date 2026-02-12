mod harness;

use std::path::PathBuf;

use five::instructions::{DEPLOY_INSTRUCTION, EXECUTE_INSTRUCTION};
use five::state::{FIVEVMState, ScriptAccountHeader};
use five_protocol::opcodes::HALT;
use harness::fixtures::canonical_execute_payload;
use harness::script_with_header;
use solana_program_test::ProgramTest;
use solana_sdk::{
    account::Account,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_program,
    transaction::Transaction,
};

#[tokio::test(flavor = "multi_thread")]
async fn deploy_and_execute_fees_are_paid_to_admin() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    std::env::set_var("BPF_OUT_DIR", repo_root.join("target/deploy"));

    let program_id = Pubkey::new_unique();
    let mut program_test = ProgramTest::new("five", program_id, None);

    let owner = Keypair::new();
    let admin = Keypair::new();
    let vm_state = Keypair::new();
    let script = Keypair::new();

    let deploy_fee_lamports = 500u32;
    let execute_fee_lamports = 200u32;

    let mut vm_state_data = vec![0u8; FIVEVMState::LEN];
    {
        let state = FIVEVMState::from_account_data_mut(&mut vm_state_data)
            .expect("vm state layout");
        state.initialize(admin.pubkey().to_bytes());
        state.deploy_fee_lamports = deploy_fee_lamports;
        state.execute_fee_lamports = execute_fee_lamports;
    }

    let bytecode = script_with_header(1, 1, &[HALT]);
    let script_account_len = ScriptAccountHeader::LEN + bytecode.len();

    program_test.add_account(
        owner.pubkey(),
        Account {
            lamports: 1_000_000_000,
            data: vec![],
            owner: system_program::id(),
            executable: false,
            rent_epoch: 0,
        },
    );
    program_test.add_account(
        admin.pubkey(),
        Account {
            lamports: 1_000_000,
            data: vec![],
            owner: system_program::id(),
            executable: false,
            rent_epoch: 0,
        },
    );
    program_test.add_account(
        vm_state.pubkey(),
        Account {
            lamports: 10_000_000,
            data: vm_state_data,
            owner: program_id,
            executable: false,
            rent_epoch: 0,
        },
    );
    program_test.add_account(
        script.pubkey(),
        Account {
            lamports: 10_000_000,
            data: vec![0u8; script_account_len],
            owner: program_id,
            executable: false,
            rent_epoch: 0,
        },
    );

    let mut ctx = program_test.start_with_context().await;

    let deploy_fee_expected = deploy_fee_lamports as u64;
    let execute_fee_expected = execute_fee_lamports as u64;

    let owner_before_deploy = ctx
        .banks_client
        .get_account(owner.pubkey())
        .await
        .expect("owner before deploy account fetch")
        .expect("owner before deploy missing")
        .lamports;
    let admin_before_deploy = ctx
        .banks_client
        .get_account(admin.pubkey())
        .await
        .expect("admin before deploy account fetch")
        .expect("admin before deploy missing")
        .lamports;

    let mut deploy_data = Vec::with_capacity(6 + bytecode.len());
    deploy_data.push(DEPLOY_INSTRUCTION);
    deploy_data.extend_from_slice(&(bytecode.len() as u32).to_le_bytes());
    deploy_data.push(0);
    deploy_data.extend_from_slice(&bytecode);

    let deploy_ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(script.pubkey(), false),
            AccountMeta::new(vm_state.pubkey(), false),
            AccountMeta::new(owner.pubkey(), true),
            AccountMeta::new(admin.pubkey(), false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data: deploy_data,
    };

    let deploy_tx = Transaction::new_signed_with_payer(
        &[deploy_ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &owner],
        ctx.last_blockhash,
    );
    ctx.banks_client
        .process_transaction(deploy_tx)
        .await
        .expect("deploy transaction must succeed");

    let owner_after_deploy = ctx
        .banks_client
        .get_account(owner.pubkey())
        .await
        .expect("owner after deploy account fetch")
        .expect("owner after deploy missing")
        .lamports;
    let admin_after_deploy = ctx
        .banks_client
        .get_account(admin.pubkey())
        .await
        .expect("admin after deploy account fetch")
        .expect("admin after deploy missing")
        .lamports;

    assert_eq!(
        admin_after_deploy - admin_before_deploy,
        deploy_fee_expected,
        "deploy fee should be credited to admin",
    );
    assert!(
        owner_before_deploy - owner_after_deploy >= deploy_fee_expected,
        "owner should pay at least deploy fee plus network tx costs",
    );

    let payload = canonical_execute_payload(0, &[]);
    let mut execute_data = Vec::with_capacity(1 + payload.len());
    execute_data.push(EXECUTE_INSTRUCTION);
    execute_data.extend_from_slice(&payload);

    let execute_ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(script.pubkey(), false),
            AccountMeta::new(vm_state.pubkey(), false),
            AccountMeta::new(owner.pubkey(), true),
            AccountMeta::new(admin.pubkey(), false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data: execute_data,
    };

    let owner_before_execute = owner_after_deploy;
    let admin_before_execute = admin_after_deploy;

    ctx.last_blockhash = ctx
        .banks_client
        .get_latest_blockhash()
        .await
        .expect("latest blockhash for execute");

    let execute_tx = Transaction::new_signed_with_payer(
        &[execute_ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &owner],
        ctx.last_blockhash,
    );
    ctx.banks_client
        .process_transaction(execute_tx)
        .await
        .expect("execute transaction must succeed");

    let owner_after_execute = ctx
        .banks_client
        .get_account(owner.pubkey())
        .await
        .expect("owner after execute account fetch")
        .expect("owner after execute missing")
        .lamports;
    let admin_after_execute = ctx
        .banks_client
        .get_account(admin.pubkey())
        .await
        .expect("admin after execute account fetch")
        .expect("admin after execute missing")
        .lamports;

    assert_eq!(
        admin_after_execute - admin_before_execute,
        execute_fee_expected,
        "execute fee should be credited to admin",
    );
    assert!(
        owner_before_execute - owner_after_execute >= execute_fee_expected,
        "payer should pay at least execute fee plus network tx costs",
    );
}
