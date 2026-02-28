mod harness;

use std::path::PathBuf;

// Runtime behavior source-of-truth lives in ProgramTest (BPF), not RuntimeHarness.
use five::state::{FIVEVMState, ScriptAccountHeader};
use five_protocol::opcodes::HALT;
use harness::addresses::{fee_vault_shard0_pda, vm_state_pda};
use harness::fixtures::canonical_execute_payload;
use harness::instruction_builders::{
    canonical_deploy_instruction, canonical_execute_instruction,
    canonical_execute_instruction_with_fee_vault,
};
use harness::script_with_header;
use solana_program_test::ProgramTest;
use solana_sdk::{
    account::Account,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{read_keypair_file, Keypair, Signer},
    system_program,
    transaction::Transaction,
};

fn bpf_program_id() -> Pubkey {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let bpf_dir = repo_root.join("target/deploy");
    std::env::set_var("BPF_OUT_DIR", &bpf_dir);
    read_keypair_file(bpf_dir.join("five-keypair.json"))
        .expect("missing target/deploy/five-keypair.json; run cargo build-sbf first")
        .pubkey()
}

async fn init_vm(
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

async fn setup_runtime_case(
    execute_fee_lamports: u32,
) -> (
    solana_program_test::ProgramTestContext,
    Pubkey,
    Keypair,
    Keypair,
    Keypair,
    Pubkey,
    Pubkey,
) {
    let program_id = bpf_program_id();
    let mut program_test = ProgramTest::new("five", program_id, None);

    let owner = Keypair::new();
    let readonly_signer = Keypair::new();
    let authority = Keypair::new();
    let (vm_state, vm_bump) = vm_state_pda(&program_id);
    let (fee_vault, _fee_vault_bump) = fee_vault_shard0_pda(&program_id);
    let script = Keypair::new();

    let bytecode = script_with_header(1, 1, &[HALT]);
    let script_account_len = ScriptAccountHeader::LEN + bytecode.len();

    for (pubkey, lamports) in [
        (owner.pubkey(), 1_000_000_000u64),
        (readonly_signer.pubkey(), 1_000_000_000u64),
        (authority.pubkey(), 1_000_000u64),
    ] {
        program_test.add_account(
            pubkey,
            Account {
                lamports,
                data: vec![],
                owner: system_program::id(),
                executable: false,
                rent_epoch: 0,
            },
        );
    }

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
    init_vm(
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
        .expect("deploy must succeed");

    (
        ctx,
        program_id,
        owner,
        readonly_signer,
        authority,
        script.pubkey(),
        vm_state,
    )
}

#[tokio::test(flavor = "multi_thread")]
async fn execute_fails_when_fee_tail_accounts_missing() {
    let (mut ctx, program_id, owner, _readonly, _authority, script, vm_state) =
        setup_runtime_case(200).await;

    let payload = canonical_execute_payload(0, &[]);
    let bad_ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(script, false),
            AccountMeta::new(vm_state, false),
            AccountMeta::new(owner.pubkey(), true),
        ],
        data: {
            let mut d = vec![five::instructions::EXECUTE_INSTRUCTION];
            d.extend_from_slice(&payload);
            d
        },
    };

    ctx.last_blockhash = ctx
        .banks_client
        .get_latest_blockhash()
        .await
        .expect("latest blockhash for execute");
    let tx = Transaction::new_signed_with_payer(
        &[bad_ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &owner],
        ctx.last_blockhash,
    );
    let err = ctx
        .banks_client
        .process_transaction(tx)
        .await
        .expect_err("execute must fail with missing canonical fee tail");
    assert!(
        format!("{err:?}").contains("NotEnoughAccountKeys"),
        "expected NotEnoughAccountKeys, got {err:?}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn execute_fails_when_fee_tail_misordered() {
    let (mut ctx, program_id, owner, _readonly, _authority, script, vm_state) =
        setup_runtime_case(200).await;
    let (fee_vault, _bump) = fee_vault_shard0_pda(&program_id);

    let payload = canonical_execute_payload(0, &[]);
    let mut execute_data = vec![five::instructions::EXECUTE_INSTRUCTION];
    execute_data.extend_from_slice(&payload);

    let misordered_ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(script, false),
            AccountMeta::new(vm_state, false),
            AccountMeta::new(fee_vault, false),
            AccountMeta::new(owner.pubkey(), true),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data: execute_data,
    };

    ctx.last_blockhash = ctx
        .banks_client
        .get_latest_blockhash()
        .await
        .expect("latest blockhash for execute");
    let tx = Transaction::new_signed_with_payer(
        &[misordered_ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &owner],
        ctx.last_blockhash,
    );
    let err = ctx
        .banks_client
        .process_transaction(tx)
        .await
        .expect_err("execute must fail with misordered fee tail");
    let msg = format!("{err:?}");
    assert!(
        msg.contains("MissingRequiredSignature") || msg.contains("InvalidArgument"),
        "expected strict signer/order validation failure, got {msg}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn execute_rejects_readonly_signer_as_fee_payer() {
    let (mut ctx, program_id, owner, readonly_signer, _authority, script, vm_state) =
        setup_runtime_case(200).await;
    let (fee_vault, _bump) = fee_vault_shard0_pda(&program_id);

    let payload = canonical_execute_payload(0, &[]);
    let mut execute_data = vec![five::instructions::EXECUTE_INSTRUCTION];
    execute_data.extend_from_slice(&payload);

    let readonly_payer_ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(script, false),
            AccountMeta::new(vm_state, false),
            AccountMeta::new_readonly(readonly_signer.pubkey(), true),
            AccountMeta::new(fee_vault, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data: execute_data,
    };

    ctx.last_blockhash = ctx
        .banks_client
        .get_latest_blockhash()
        .await
        .expect("latest blockhash for execute");
    let tx = Transaction::new_signed_with_payer(
        &[readonly_payer_ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &readonly_signer],
        ctx.last_blockhash,
    );
    let err = ctx
        .banks_client
        .process_transaction(tx)
        .await
        .expect_err("readonly signer must not be accepted as execute payer");

    assert!(
        format!("{err:?}").contains("MissingRequiredSignature"),
        "expected MissingRequiredSignature, got {err:?}"
    );

    // Also assert canonical path still succeeds in the same test context.
    ctx.last_blockhash = ctx
        .banks_client
        .get_latest_blockhash()
        .await
        .expect("latest blockhash for canonical execute");
    let ok_ix = canonical_execute_instruction_with_fee_vault(
        program_id,
        script,
        vm_state,
        owner.pubkey(),
        fee_vault,
        &payload,
        None,
    );
    let ok_tx = Transaction::new_signed_with_payer(
        &[ok_ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &owner],
        ctx.last_blockhash,
    );
    ctx.banks_client
        .process_transaction(ok_tx)
        .await
        .expect("canonical execute must succeed after negative case");
}

#[tokio::test(flavor = "multi_thread")]
async fn execute_charges_fee_to_fee_vault_canonical_path() {
    let (mut ctx, program_id, owner, _readonly, _authority, script, vm_state) =
        setup_runtime_case(200).await;
    let (fee_vault, _bump) = fee_vault_shard0_pda(&program_id);

    let before_owner = ctx
        .banks_client
        .get_account(owner.pubkey())
        .await
        .expect("owner fetch")
        .expect("owner exists")
        .lamports;
    let before_vault = ctx
        .banks_client
        .get_account(fee_vault)
        .await
        .expect("vault fetch")
        .expect("vault exists")
        .lamports;

    let payload = canonical_execute_payload(0, &[]);
    let execute_ix =
        canonical_execute_instruction(program_id, script, vm_state, owner.pubkey(), &payload, None);

    ctx.last_blockhash = ctx
        .banks_client
        .get_latest_blockhash()
        .await
        .expect("latest blockhash for execute");
    let tx = Transaction::new_signed_with_payer(
        &[execute_ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &owner],
        ctx.last_blockhash,
    );
    ctx.banks_client
        .process_transaction(tx)
        .await
        .expect("canonical execute must succeed");

    let after_owner = ctx
        .banks_client
        .get_account(owner.pubkey())
        .await
        .expect("owner fetch after")
        .expect("owner exists after")
        .lamports;
    let after_vault = ctx
        .banks_client
        .get_account(fee_vault)
        .await
        .expect("vault fetch after")
        .expect("vault exists after")
        .lamports;

    assert!(before_owner > after_owner, "owner must pay execute fee");
    assert_eq!(
        after_vault - before_vault,
        200,
        "fee vault receives full execute fee"
    );
}
