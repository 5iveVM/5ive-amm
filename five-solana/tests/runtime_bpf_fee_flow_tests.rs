mod harness;

use std::path::PathBuf;

// Runtime behavior source-of-truth lives in ProgramTest (BPF), not RuntimeHarness.
use five::state::{FIVEVMState, ScriptAccountHeader};
use five_protocol::opcodes::HALT;
use harness::addresses::{fee_vault_shard0_pda, vm_state_pda};
use harness::fixtures::canonical_execute_payload;
use harness::instruction_builders::{canonical_deploy_instruction, canonical_execute_instruction};
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

fn bpf_program_id() -> Pubkey {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    harness::load_target_deploy_program_id_checked(&repo_root)
        .expect("target/deploy artifact parity preflight failed")
}

async fn read_lamports(
    ctx: &mut solana_program_test::ProgramTestContext,
    key: Pubkey,
    label: &str,
) -> u64 {
    ctx.banks_client
        .get_account(key)
        .await
        .unwrap_or_else(|_| panic!("{label} account fetch failed"))
        .unwrap_or_else(|| panic!("{label} account missing"))
        .lamports
}

async fn initialize_and_set_fees(
    ctx: &mut solana_program_test::ProgramTestContext,
    program_id: Pubkey,
    vm_state: Pubkey,
    authority: &Keypair,
    vm_bump: u8,
    deploy_fee_lamports: u32,
    execute_fee_lamports: u32,
) {
    let init_ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(vm_state, false),
            AccountMeta::new_readonly(authority.pubkey(), true),
        ],
        data: vec![0, vm_bump],
    };
    let init_tx = Transaction::new_signed_with_payer(
        &[init_ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, authority],
        ctx.last_blockhash,
    );
    ctx.banks_client
        .process_transaction(init_tx)
        .await
        .expect("initialize vm_state must succeed");

    ctx.last_blockhash = ctx
        .banks_client
        .get_latest_blockhash()
        .await
        .expect("latest blockhash for set_fees");

    let mut set_fees_data = Vec::with_capacity(9);
    set_fees_data.push(6);
    set_fees_data.extend_from_slice(&deploy_fee_lamports.to_le_bytes());
    set_fees_data.extend_from_slice(&execute_fee_lamports.to_le_bytes());

    let set_fees_ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(vm_state, false),
            AccountMeta::new_readonly(authority.pubkey(), true),
        ],
        data: set_fees_data,
    };
    let set_fees_tx = Transaction::new_signed_with_payer(
        &[set_fees_ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, authority],
        ctx.last_blockhash,
    );
    ctx.banks_client
        .process_transaction(set_fees_tx)
        .await
        .expect("set_fees must succeed");
}

#[tokio::test(flavor = "multi_thread")]
async fn deploy_and_execute_fees_are_paid_to_fee_vault() {
    let program_id = bpf_program_id();
    let mut program_test = ProgramTest::new("five", program_id, None);

    let owner = Keypair::new();
    let authority = Keypair::new();
    let (vm_state, vm_bump) = vm_state_pda(&program_id);
    let (fee_vault, _fee_vault_bump) = fee_vault_shard0_pda(&program_id);
    let script = Keypair::new();

    let deploy_fee_lamports = 500u32;
    let execute_fee_lamports = 200u32;

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
        authority.pubkey(),
        Account {
            lamports: 1_000_000,
            data: vec![],
            owner: system_program::id(),
            executable: false,
            rent_epoch: 0,
        },
    );
    program_test.add_account(
        vm_state,
        Account {
            lamports: 10_000_000,
            data: vec![0u8; FIVEVMState::LEN],
            owner: program_id,
            executable: false,
            rent_epoch: 0,
        },
    );
    program_test.add_account(
        fee_vault,
        Account {
            lamports: 1_000_000,
            data: vec![],
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
    initialize_and_set_fees(
        &mut ctx,
        program_id,
        vm_state,
        &authority,
        vm_bump,
        deploy_fee_lamports,
        execute_fee_lamports,
    )
    .await;

    ctx.last_blockhash = ctx
        .banks_client
        .get_latest_blockhash()
        .await
        .expect("latest blockhash for deploy");

    let owner_before_deploy = read_lamports(&mut ctx, owner.pubkey(), "owner before deploy").await;
    let vault_before_deploy = read_lamports(&mut ctx, fee_vault, "fee vault before deploy").await;

    let deploy_ix = canonical_deploy_instruction(
        program_id,
        script.pubkey(),
        vm_state,
        owner.pubkey(),
        &bytecode,
        0,
        &[],
        None,
    );

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

    let owner_after_deploy = read_lamports(&mut ctx, owner.pubkey(), "owner after deploy").await;
    let vault_after_deploy = read_lamports(&mut ctx, fee_vault, "fee vault after deploy").await;

    assert_eq!(
        vault_after_deploy - vault_before_deploy,
        deploy_fee_lamports as u64,
        "deploy fee should be credited to fee vault",
    );
    assert!(
        owner_before_deploy - owner_after_deploy >= deploy_fee_lamports as u64,
        "owner should pay at least deploy fee plus tx costs",
    );

    let payload = canonical_execute_payload(0, &[]);
    let execute_ix = canonical_execute_instruction(
        program_id,
        script.pubkey(),
        vm_state,
        owner.pubkey(),
        &payload,
        None,
    );

    let owner_before_execute = owner_after_deploy;
    let vault_before_execute = vault_after_deploy;

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

    let owner_after_execute = read_lamports(&mut ctx, owner.pubkey(), "owner after execute").await;
    let vault_after_execute = read_lamports(&mut ctx, fee_vault, "fee vault after execute").await;

    assert_eq!(
        vault_after_execute - vault_before_execute,
        execute_fee_lamports as u64,
        "execute fee should be credited to fee vault",
    );
    assert!(
        owner_before_execute - owner_after_execute >= execute_fee_lamports as u64,
        "owner should pay at least execute fee plus tx costs",
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn deploy_fails_when_owner_cannot_pay_deploy_fee() {
    let program_id = bpf_program_id();
    let mut program_test = ProgramTest::new("five", program_id, None);

    let owner = Keypair::new();
    let authority = Keypair::new();
    let (vm_state, vm_bump) = vm_state_pda(&program_id);
    let (fee_vault, _fee_vault_bump) = fee_vault_shard0_pda(&program_id);
    let script = Keypair::new();

    let deploy_fee_lamports = 5_000u32;

    let bytecode = script_with_header(1, 1, &[HALT]);
    let script_account_len = ScriptAccountHeader::LEN + bytecode.len();

    program_test.add_account(
        owner.pubkey(),
        Account {
            lamports: 500,
            data: vec![],
            owner: system_program::id(),
            executable: false,
            rent_epoch: 0,
        },
    );
    program_test.add_account(
        authority.pubkey(),
        Account {
            lamports: 1_000_000,
            data: vec![],
            owner: system_program::id(),
            executable: false,
            rent_epoch: 0,
        },
    );
    program_test.add_account(
        vm_state,
        Account {
            lamports: 10_000_000,
            data: vec![0u8; FIVEVMState::LEN],
            owner: program_id,
            executable: false,
            rent_epoch: 0,
        },
    );
    program_test.add_account(
        fee_vault,
        Account {
            lamports: 1_000_000,
            data: vec![],
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
    initialize_and_set_fees(
        &mut ctx,
        program_id,
        vm_state,
        &authority,
        vm_bump,
        deploy_fee_lamports,
        1,
    )
    .await;

    ctx.last_blockhash = ctx
        .banks_client
        .get_latest_blockhash()
        .await
        .expect("latest blockhash for deploy");

    let owner_before = read_lamports(&mut ctx, owner.pubkey(), "owner before deploy").await;
    let vault_before = read_lamports(&mut ctx, fee_vault, "fee vault before deploy").await;

    let deploy_ix = canonical_deploy_instruction(
        program_id,
        script.pubkey(),
        vm_state,
        owner.pubkey(),
        &bytecode,
        0,
        &[],
        None,
    );

    let deploy_tx = Transaction::new_signed_with_payer(
        &[deploy_ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &owner],
        ctx.last_blockhash,
    );
    let err = ctx
        .banks_client
        .process_transaction(deploy_tx)
        .await
        .expect_err("deploy must fail when owner cannot pay deploy fee");
    assert!(
        format!("{err:?}").contains("InsufficientFunds"),
        "expected InsufficientFunds, got {err:?}",
    );

    let owner_after = read_lamports(&mut ctx, owner.pubkey(), "owner after deploy").await;
    let vault_after = read_lamports(&mut ctx, fee_vault, "fee vault after deploy").await;

    assert_eq!(
        owner_after, owner_before,
        "owner lamports must be unchanged"
    );
    assert_eq!(
        vault_after, vault_before,
        "fee vault must not receive partial fee"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn execute_fails_when_owner_cannot_pay_execute_fee() {
    let program_id = bpf_program_id();
    let mut program_test = ProgramTest::new("five", program_id, None);

    let owner = Keypair::new();
    let authority = Keypair::new();
    let (vm_state, vm_bump) = vm_state_pda(&program_id);
    let (fee_vault, _fee_vault_bump) = fee_vault_shard0_pda(&program_id);
    let script = Keypair::new();

    let execute_fee_lamports = 5_000u32;

    let bytecode = script_with_header(1, 1, &[HALT]);
    let script_account_len = ScriptAccountHeader::LEN + bytecode.len();

    program_test.add_account(
        owner.pubkey(),
        Account {
            lamports: 500,
            data: vec![],
            owner: system_program::id(),
            executable: false,
            rent_epoch: 0,
        },
    );
    program_test.add_account(
        authority.pubkey(),
        Account {
            lamports: 1_000_000,
            data: vec![],
            owner: system_program::id(),
            executable: false,
            rent_epoch: 0,
        },
    );
    program_test.add_account(
        vm_state,
        Account {
            lamports: 10_000_000,
            data: vec![0u8; FIVEVMState::LEN],
            owner: program_id,
            executable: false,
            rent_epoch: 0,
        },
    );
    program_test.add_account(
        fee_vault,
        Account {
            lamports: 1_000_000,
            data: vec![],
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
    initialize_and_set_fees(
        &mut ctx,
        program_id,
        vm_state,
        &authority,
        vm_bump,
        1,
        execute_fee_lamports,
    )
    .await;

    ctx.last_blockhash = ctx
        .banks_client
        .get_latest_blockhash()
        .await
        .expect("latest blockhash for deploy");

    let deploy_ix = canonical_deploy_instruction(
        program_id,
        script.pubkey(),
        vm_state,
        owner.pubkey(),
        &bytecode,
        0,
        &[],
        None,
    );
    let deploy_tx = Transaction::new_signed_with_payer(
        &[deploy_ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &owner],
        ctx.last_blockhash,
    );
    ctx.banks_client
        .process_transaction(deploy_tx)
        .await
        .expect("deploy without deploy fee must succeed");

    let owner_before = read_lamports(&mut ctx, owner.pubkey(), "owner before execute").await;
    let vault_before = read_lamports(&mut ctx, fee_vault, "fee vault before execute").await;

    let payload = canonical_execute_payload(0, &[]);
    let execute_ix = canonical_execute_instruction(
        program_id,
        script.pubkey(),
        vm_state,
        owner.pubkey(),
        &payload,
        None,
    );

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
    let err = ctx
        .banks_client
        .process_transaction(execute_tx)
        .await
        .expect_err("execute must fail when owner cannot pay execute fee");
    assert!(
        format!("{err:?}").contains("Custom(7808)"),
        "expected execute fee transfer failure Custom(7808), got {err:?}",
    );

    let owner_after = read_lamports(&mut ctx, owner.pubkey(), "owner after execute").await;
    let vault_after = read_lamports(&mut ctx, fee_vault, "fee vault after execute").await;

    assert_eq!(
        owner_after, owner_before,
        "owner lamports must be unchanged"
    );
    assert_eq!(
        vault_after, vault_before,
        "fee vault must not receive partial fee"
    );
}
