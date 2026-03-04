mod harness;

use std::{collections::BTreeMap, path::PathBuf};

use five::instructions::{DEPLOY_INSTRUCTION, EXECUTE_INSTRUCTION};
use five::state::{FIVEVMState, ScriptAccountHeader};
use five_protocol::opcodes::{
    CHECK_UNINITIALIZED, HALT, INIT_ACCOUNT, INIT_PDA_ACCOUNT, PUSH_STRING, PUSH_U64, PUSH_U8,
};
use harness::addresses::{canonical_execute_fee_header, fee_vault_shard0_pda, vm_state_pda};
use harness::script_with_header;
use solana_program_test::{ProgramTest, ProgramTestContext};
use solana_sdk::{
    account::Account,
    compute_budget::ComputeBudgetInstruction,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    rent::Rent,
    signature::{Keypair, Signer},
    system_program,
    transaction::Transaction,
};

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

#[derive(Debug)]
struct TxOutcome {
    success: bool,
    error: Option<String>,
}

#[tokio::test(flavor = "multi_thread")]
async fn init_account_executes_via_bpf_create_account_cpi() {
    let program_id = load_program_id();
    let mut accounts = base_accounts(program_id);
    let account_space = 64u64;
    let rent_lamports = Rent::default().minimum_balance(account_space as usize);

    let script_body = build_init_account_script(1, 2, account_space, rent_lamports);
    let bytecode = script_with_header(1, 1, &script_body);

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

    let new_account_signer = Keypair::new();
    accounts.insert(
        "new_account".to_string(),
        RuntimeAccount {
            pubkey: new_account_signer.pubkey(),
            signer: Some(new_account_signer),
            owner: system_program::id(),
            lamports: 0,
            data: vec![],
            is_signer: true,
            is_writable: true,
            executable: false,
        },
    );

    let mut ctx = start_context(program_id, &accounts).await;

    let deploy_ix = build_deploy_instruction(program_id, &accounts, "script", &bytecode);
    let deploy = simulate_and_process(
        &mut ctx,
        vec![deploy_ix],
        collect_signers(&accounts, &["owner"]),
        Some(1_400_000),
    )
    .await;
    assert!(deploy.success, "deploy failed: {:?}", deploy.error);

    let execute_ix = build_execute_instruction(
        program_id,
        &accounts,
        "script",
        &["new_account"],
        canonical_execute_payload(0),
    );
    let execute = simulate_and_process(
        &mut ctx,
        vec![execute_ix],
        collect_signers(&accounts, &["owner", "new_account"]),
        Some(1_400_000),
    )
    .await;
    assert!(execute.success, "execute failed: {:?}", execute.error);

    let created = ctx
        .banks_client
        .get_account(accounts["new_account"].pubkey)
        .await
        .expect("new account fetch should succeed")
        .expect("new account should exist after INIT_ACCOUNT");

    assert_eq!(created.owner, program_id);
    assert_eq!(created.data.len(), account_space as usize);
    assert_eq!(created.lamports, rent_lamports);
}

#[tokio::test(flavor = "multi_thread")]
async fn init_pda_account_executes_via_bpf_create_account_cpi() {
    let program_id = load_program_id();
    let mut accounts = base_accounts(program_id);
    let account_space = 64u64;
    let rent_lamports = Rent::default().minimum_balance(account_space as usize);
    let seed = b"vault";
    let (pda_pubkey, bump) = Pubkey::find_program_address(&[seed], &program_id);

    let script_body = build_init_pda_account_script(1, 2, account_space, rent_lamports, seed, bump);
    let bytecode = script_with_header(1, 1, &script_body);

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

    accounts.insert(
        "pda_account".to_string(),
        RuntimeAccount {
            pubkey: pda_pubkey,
            signer: None,
            owner: system_program::id(),
            lamports: 0,
            data: vec![],
            is_signer: false,
            is_writable: true,
            executable: false,
        },
    );

    let mut ctx = start_context(program_id, &accounts).await;

    let deploy_ix = build_deploy_instruction(program_id, &accounts, "script", &bytecode);
    let deploy = simulate_and_process(
        &mut ctx,
        vec![deploy_ix],
        collect_signers(&accounts, &["owner"]),
        Some(1_400_000),
    )
    .await;
    assert!(deploy.success, "deploy failed: {:?}", deploy.error);

    let execute_ix = build_execute_instruction(
        program_id,
        &accounts,
        "script",
        &["pda_account"],
        canonical_execute_payload(0),
    );
    let execute = simulate_and_process(
        &mut ctx,
        vec![execute_ix],
        collect_signers(&accounts, &["owner"]),
        Some(1_400_000),
    )
    .await;
    assert!(execute.success, "execute failed: {:?}", execute.error);

    let created = ctx
        .banks_client
        .get_account(accounts["pda_account"].pubkey)
        .await
        .expect("pda account fetch should succeed")
        .expect("pda account should exist after INIT_PDA_ACCOUNT");

    assert_eq!(created.owner, program_id);
    assert_eq!(created.data.len(), account_space as usize);
    assert_eq!(created.lamports, rent_lamports);
}

fn build_init_account_script(
    account_idx: u8,
    payer_idx: u8,
    space: u64,
    lamports: u64,
) -> Vec<u8> {
    let mut body = Vec::with_capacity(1 + 1 + 1 + 8 + 1 + 8 + 1 + 1 + 1 + 8 + 1 + 1 + 1);
    body.push(CHECK_UNINITIALIZED);
    body.push(account_idx);

    body.push(PUSH_U64);
    body.extend_from_slice(&0u64.to_le_bytes());

    body.push(PUSH_U64);
    body.extend_from_slice(&lamports.to_le_bytes());

    body.push(PUSH_U8);
    body.push(payer_idx);

    body.push(PUSH_U64);
    body.extend_from_slice(&space.to_le_bytes());

    body.push(PUSH_U8);
    body.push(account_idx);

    body.push(INIT_ACCOUNT);
    body.push(HALT);
    body
}

fn build_init_pda_account_script(
    account_idx: u8,
    payer_idx: u8,
    space: u64,
    lamports: u64,
    seed: &[u8],
    bump: u8,
) -> Vec<u8> {
    let mut body =
        Vec::with_capacity(2 + 2 + 1 + 1 + 4 + seed.len() + 2 + 1 + 8 + 1 + 8 + 1 + 1 + 1 + 8 + 1 + 1 + 1);
    body.push(CHECK_UNINITIALIZED);
    body.push(account_idx);

    body.push(PUSH_U8);
    body.push(bump);

    body.push(PUSH_STRING);
    body.extend_from_slice(&(seed.len() as u32).to_le_bytes());
    body.extend_from_slice(seed);

    body.push(PUSH_U8);
    body.push(1);

    body.push(PUSH_U64);
    body.extend_from_slice(&0u64.to_le_bytes());

    body.push(PUSH_U64);
    body.extend_from_slice(&lamports.to_le_bytes());

    body.push(PUSH_U8);
    body.push(payer_idx);

    body.push(PUSH_U64);
    body.extend_from_slice(&space.to_le_bytes());

    body.push(PUSH_U8);
    body.push(account_idx);

    body.push(INIT_PDA_ACCOUNT);
    body.push(HALT);
    body
}

fn load_program_id() -> Pubkey {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    harness::load_target_deploy_program_id_checked(&repo_root)
        .expect("target/deploy artifact parity preflight failed")
}

fn base_accounts(program_id: Pubkey) -> BTreeMap<String, RuntimeAccount> {
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

    let (vm_state_pubkey, vm_state_bump) = vm_state_pda(&program_id);
    let (fee_vault_pubkey, _fee_vault_bump) = fee_vault_shard0_pda(&program_id);

    let mut vm_state_data = vec![0u8; FIVEVMState::LEN];
    {
        let vm_state = FIVEVMState::from_account_data_mut(&mut vm_state_data)
            .expect("invalid vm state account layout");
        vm_state.initialize(owner_pubkey.to_bytes(), vm_state_bump);
        vm_state.deploy_fee_lamports = 0;
        vm_state.execute_fee_lamports = 0;
    }

    accounts.insert(
        "vm_state".to_string(),
        RuntimeAccount {
            pubkey: vm_state_pubkey,
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
        "fee_vault".to_string(),
        RuntimeAccount {
            pubkey: fee_vault_pubkey,
            signer: None,
            owner: program_id,
            lamports: Rent::default().minimum_balance(0),
            data: vec![],
            is_signer: false,
            is_writable: true,
            executable: false,
        },
    );

    accounts
}

async fn start_context(
    program_id: Pubkey,
    accounts: &BTreeMap<String, RuntimeAccount>,
) -> ProgramTestContext {
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

    program_test.start_with_context().await
}

fn build_deploy_instruction(
    program_id: Pubkey,
    accounts: &BTreeMap<String, RuntimeAccount>,
    script_name: &str,
    bytecode: &[u8],
) -> Instruction {
    let mut data = Vec::with_capacity(10 + bytecode.len());
    data.push(DEPLOY_INSTRUCTION);
    data.extend_from_slice(&(bytecode.len() as u32).to_le_bytes());
    data.push(0);
    data.extend_from_slice(&0u32.to_le_bytes());
    data.extend_from_slice(bytecode);
    data.push(0);

    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(accounts[script_name].pubkey, false),
            AccountMeta::new(accounts["vm_state"].pubkey, false),
            AccountMeta::new(accounts["owner"].pubkey, true),
            AccountMeta::new(accounts["fee_vault"].pubkey, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data,
    }
}

fn build_execute_instruction(
    program_id: Pubkey,
    accounts: &BTreeMap<String, RuntimeAccount>,
    script_name: &str,
    extras: &[&str],
    payload: Vec<u8>,
) -> Instruction {
    let mut data = Vec::with_capacity(4 + payload.len());
    data.push(EXECUTE_INSTRUCTION);
    data.extend_from_slice(&canonical_execute_fee_header(0));
    data.extend_from_slice(&payload);

    let mut metas = vec![
        AccountMeta::new(accounts[script_name].pubkey, false),
        AccountMeta::new(accounts["vm_state"].pubkey, false),
    ];
    for name in extras {
        let account = &accounts[*name];
        metas.push(AccountMeta {
            pubkey: account.pubkey,
            is_signer: account.is_signer,
            is_writable: account.is_writable,
        });
    }

    let payer = &accounts["owner"];
    metas.push(AccountMeta::new(payer.pubkey, true));
    metas.push(AccountMeta::new(accounts["fee_vault"].pubkey, false));
    metas.push(AccountMeta::new_readonly(system_program::id(), false));

    Instruction {
        program_id,
        accounts: metas,
        data,
    }
}

fn canonical_execute_payload(function_index: u32) -> Vec<u8> {
    five_protocol::execute_payload::canonical_execute_payload(function_index, &[])
}

fn collect_signers<'a>(
    accounts: &'a BTreeMap<String, RuntimeAccount>,
    names: &[&str],
) -> Vec<&'a Keypair> {
    let mut out = Vec::new();
    for name in names {
        if let Some(kp) = accounts[*name].signer.as_ref() {
            out.push(kp);
        }
    }
    out
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

    match ctx.banks_client.process_transaction(tx).await {
        Ok(()) => TxOutcome {
            success: true,
            error: None,
        },
        Err(err) => TxOutcome {
            success: false,
            error: Some(err.to_string()),
        },
    }
}
