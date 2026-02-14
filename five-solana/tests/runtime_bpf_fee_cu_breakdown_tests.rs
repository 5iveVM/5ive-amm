mod harness;

use std::path::PathBuf;

use five::instructions::{DEPLOY_INSTRUCTION, EXECUTE_INSTRUCTION};
use five::state::{FIVEVMState, ScriptAccountHeader};
use five_protocol::opcodes::HALT;
use harness::fixtures::canonical_execute_payload;
use harness::script_with_header;
use solana_program_test::{ProgramTest, ProgramTestBanksClientExt, ProgramTestContext};
use solana_sdk::{
    account::Account,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_program,
    transaction::Transaction,
};

#[derive(Debug)]
struct TxOutcome {
    success: bool,
    units_consumed: u64,
    error: Option<String>,
}

async fn simulate_and_process(
    ctx: &mut ProgramTestContext,
    instructions: Vec<Instruction>,
    extra_signers: Vec<&Keypair>,
) -> TxOutcome {
    ctx.last_blockhash = ctx
        .banks_client
        .get_new_latest_blockhash(&ctx.last_blockhash)
        .await
        .expect("new latest blockhash");

    let mut signers: Vec<&Keypair> = Vec::with_capacity(1 + extra_signers.len());
    signers.push(&ctx.payer);
    signers.extend(extra_signers);

    let tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&ctx.payer.pubkey()),
        &signers,
        ctx.last_blockhash,
    );

    let simulation = ctx.banks_client.simulate_transaction(tx.clone()).await;
    let simulated_units = match simulation {
        Ok(sim_result) => sim_result
            .simulation_details
            .as_ref()
            .map(|d| d.units_consumed)
            .unwrap_or(0),
        Err(err) => {
            return TxOutcome {
                success: false,
                units_consumed: 0,
                error: Some(format!("simulate failed: {}", err)),
            };
        }
    };

    match ctx.banks_client.process_transaction(tx).await {
        Ok(()) => TxOutcome {
            success: true,
            units_consumed: simulated_units,
            error: None,
        },
        Err(err) => TxOutcome {
            success: false,
            units_consumed: simulated_units,
            error: Some(err.to_string()),
        },
    }
}

fn deploy_data(bytecode: &[u8], fee_shard_index: Option<u8>, fee_vault_bump: Option<u8>) -> Vec<u8> {
    let mut data = Vec::with_capacity(12 + bytecode.len());
    data.push(DEPLOY_INSTRUCTION);
    data.extend_from_slice(&(bytecode.len() as u32).to_le_bytes());
    data.push(0u8);
    data.extend_from_slice(&(0u32).to_le_bytes());
    data.extend_from_slice(bytecode);
    if let (Some(shard), Some(bump)) = (fee_shard_index, fee_vault_bump) {
        data.push(shard);
        data.push(bump);
    }
    data
}

#[derive(Debug)]
struct FeePathCu {
    deploy_units: u64,
    execute_units: u64,
}

async fn run_fee_path_case(
    deploy_fee_lamports: u32,
    execute_fee_lamports: u32,
    program_owned_payer: bool,
    include_bump_header: bool,
) -> FeePathCu {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let bpf_dir = repo_root.join("target/deploy");
    std::env::set_var("BPF_OUT_DIR", &bpf_dir);
    let program_id = solana_sdk::signature::read_keypair_file(bpf_dir.join("five-keypair.json"))
        .expect("missing target/deploy/five-keypair.json; run cargo build-sbf first")
        .pubkey();
    let mut program_test = ProgramTest::new("five", program_id, None);

    let owner = Keypair::new();
    let payer_owner = if program_owned_payer {
        program_id
    } else {
        system_program::id()
    };
    let (vm_state, vm_bump) = Pubkey::find_program_address(&[b"vm_state"], &program_id);
    let (fee_vault, fee_vault_bump) =
        Pubkey::find_program_address(&[b"\xFFfive_vm_fee_vault_v1", &[0u8]], &program_id);
    let script = Keypair::new();

    let bytecode = script_with_header(1, 1, &[HALT]);
    let script_account_len = ScriptAccountHeader::LEN + bytecode.len();

    program_test.add_account(
        owner.pubkey(),
        Account {
            lamports: 2_000_000_000,
            data: vec![],
            owner: payer_owner,
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

    let init_ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(vm_state, false),
            AccountMeta::new_readonly(owner.pubkey(), true),
        ],
        data: vec![0, vm_bump],
    };
    let init = simulate_and_process(&mut ctx, vec![init_ix], vec![&owner]).await;
    assert!(init.success, "initialize failed: {:?}", init.error);

    let mut set_fees_data = Vec::with_capacity(9);
    set_fees_data.push(6);
    set_fees_data.extend_from_slice(&deploy_fee_lamports.to_le_bytes());
    set_fees_data.extend_from_slice(&execute_fee_lamports.to_le_bytes());
    let set_fees_ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(vm_state, false),
            AccountMeta::new_readonly(owner.pubkey(), true),
        ],
        data: set_fees_data,
    };
    let set_fees = simulate_and_process(&mut ctx, vec![set_fees_ix], vec![&owner]).await;
    assert!(set_fees.success, "set_fees failed: {:?}", set_fees.error);

    let deploy_ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(script.pubkey(), false),
            AccountMeta::new(vm_state, false),
            AccountMeta::new(owner.pubkey(), true),
            AccountMeta::new(fee_vault, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data: deploy_data(
            &bytecode,
            if include_bump_header { Some(0) } else { None },
            if include_bump_header {
                Some(fee_vault_bump)
            } else {
                None
            },
        ),
    };
    let deploy = simulate_and_process(&mut ctx, vec![deploy_ix], vec![&owner]).await;
    assert!(deploy.success, "deploy failed: {:?}", deploy.error);

    let payload = canonical_execute_payload(0, &[]);
    let mut execute_data = Vec::with_capacity(1 + payload.len());
    execute_data.push(EXECUTE_INSTRUCTION);
    if include_bump_header {
        execute_data.extend_from_slice(&[0xFF, 0x53, 0, fee_vault_bump]);
    }
    execute_data.extend_from_slice(&payload);

    let execute_ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(script.pubkey(), false),
            AccountMeta::new(vm_state, false),
            AccountMeta::new(owner.pubkey(), true),
            AccountMeta::new(fee_vault, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data: execute_data,
    };
    let execute = simulate_and_process(&mut ctx, vec![execute_ix], vec![&owner]).await;
    assert!(execute.success, "execute failed: {:?}", execute.error);

    FeePathCu {
        deploy_units: deploy.units_consumed,
        execute_units: execute.units_consumed,
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn bpf_fee_path_cu_breakdown() {
    let base = run_fee_path_case(0, 0, false, true).await;
    let fee_system_owned_payer = run_fee_path_case(500, 500, false, true).await;
    let fee_program_owned_payer = run_fee_path_case(500, 500, true, true).await;

    println!("BPF_CU_BREAKDOWN base_deploy={}", base.deploy_units);
    println!("BPF_CU_BREAKDOWN base_execute={}", base.execute_units);
    println!(
        "BPF_CU_BREAKDOWN fee_deploy_system_owned_payer={}",
        fee_system_owned_payer.deploy_units
    );
    println!(
        "BPF_CU_BREAKDOWN fee_execute_system_owned_payer={}",
        fee_system_owned_payer.execute_units
    );
    println!(
        "BPF_CU_BREAKDOWN fee_deploy_program_owned_payer={}",
        fee_program_owned_payer.deploy_units
    );
    println!(
        "BPF_CU_BREAKDOWN fee_execute_program_owned_payer={}",
        fee_program_owned_payer.execute_units
    );
    println!(
        "BPF_CU_BREAKDOWN fee_overhead_deploy_system_owned_payer={}",
        fee_system_owned_payer
            .deploy_units
            .saturating_sub(base.deploy_units)
    );
    println!(
        "BPF_CU_BREAKDOWN fee_overhead_deploy_program_owned_payer={}",
        fee_program_owned_payer
            .deploy_units
            .saturating_sub(base.deploy_units)
    );
    println!(
        "BPF_CU_BREAKDOWN fee_overhead_system_owned_payer={}",
        fee_system_owned_payer
            .execute_units
            .saturating_sub(base.execute_units)
    );
    println!(
        "BPF_CU_BREAKDOWN fee_overhead_program_owned_payer={}",
        fee_program_owned_payer
            .execute_units
            .saturating_sub(base.execute_units)
    );

    assert!(
        fee_system_owned_payer.execute_units >= base.execute_units,
        "fee path should not be cheaper than base"
    );
    assert!(
        fee_program_owned_payer.execute_units >= base.execute_units,
        "fee path should not be cheaper than base"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn bpf_fee_vault_bump_header_cu_delta() {
    let with_bump = run_fee_path_case(500, 500, false, true).await;
    let without_bump = run_fee_path_case(500, 500, false, false).await;
    let deploy_delta = without_bump.deploy_units as i64 - with_bump.deploy_units as i64;
    let execute_delta = without_bump.execute_units as i64 - with_bump.execute_units as i64;
    println!(
        "BPF_CU_BREAKDOWN bump_header_deploy_with={} without={} delta_signed={}",
        with_bump.deploy_units,
        without_bump.deploy_units,
        deploy_delta,
    );
    println!(
        "BPF_CU_BREAKDOWN bump_header_execute_with={} without={} delta_signed={}",
        with_bump.execute_units,
        without_bump.execute_units,
        execute_delta,
    );
    assert!(
        deploy_delta.unsigned_abs() <= 500 && execute_delta.unsigned_abs() <= 500,
        "bump-header path drift too large deploy_delta={} execute_delta={}",
        deploy_delta,
        execute_delta,
    );
}
