use std::str::FromStr;

use five::state::FIVEVMState;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey};

const DEFAULT_PROGRAM_ID: &str = "4Qxf3pbCse2veUgZVMiAm3nWqJrYo2pT4suxHKMJdK1d";
const DEFAULT_VM_STATE: &str = "8ip3qGGETf8774jo6kXbsTTrMm5V9bLuGC4znmyZjT3z";
const DEFAULT_RPC_URL: &str = "http://127.0.0.1:8899";
const DEFAULT_DEPLOY_FEE_LAMPORTS: u32 = 10_000;
const DEFAULT_EXECUTE_FEE_LAMPORTS: u32 = 85_734;

#[test]
#[ignore = "requires localnet validator with cloned vm program/state"]
fn vm_state_parity_gate_localnet() {
    let rpc_url = std::env::var("FIVE_PARITY_RPC_URL").unwrap_or_else(|_| DEFAULT_RPC_URL.to_string());
    let program_id = Pubkey::from_str(
        &std::env::var("FIVE_PARITY_PROGRAM_ID").unwrap_or_else(|_| DEFAULT_PROGRAM_ID.to_string()),
    )
    .expect("invalid FIVE_PARITY_PROGRAM_ID");
    let vm_state = Pubkey::from_str(
        &std::env::var("FIVE_PARITY_VM_STATE").unwrap_or_else(|_| DEFAULT_VM_STATE.to_string()),
    )
    .expect("invalid FIVE_PARITY_VM_STATE");

    let expected_authority = Pubkey::from_str(
        &std::env::var("FIVE_PARITY_EXPECTED_AUTHORITY")
            .expect("missing FIVE_PARITY_EXPECTED_AUTHORITY"),
    )
    .expect("invalid FIVE_PARITY_EXPECTED_AUTHORITY");

    let expected_deploy_fee = std::env::var("FIVE_PARITY_EXPECTED_DEPLOY_FEE")
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(DEFAULT_DEPLOY_FEE_LAMPORTS);
    let expected_execute_fee = std::env::var("FIVE_PARITY_EXPECTED_EXECUTE_FEE")
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(DEFAULT_EXECUTE_FEE_LAMPORTS);

    let rpc = RpcClient::new_with_commitment(rpc_url.clone(), CommitmentConfig::confirmed());

    let (canonical_vm_state, bump) =
        Pubkey::find_program_address(&[b"vm_state"], &program_id);
    assert_eq!(
        vm_state, canonical_vm_state,
        "vm_state mismatch: expected canonical {}, got {}",
        canonical_vm_state, vm_state
    );

    let vm_account = rpc
        .get_account(&vm_state)
        .unwrap_or_else(|e| panic!("failed fetching vm_state {} from {}: {}", vm_state, rpc_url, e));
    assert_eq!(
        vm_account.owner, program_id,
        "vm_state owner mismatch: expected {}, got {}",
        program_id, vm_account.owner
    );
    assert!(
        vm_account.data.len() >= FIVEVMState::LEN,
        "vm_state data too small: expected >= {}, got {}",
        FIVEVMState::LEN,
        vm_account.data.len()
    );

    let vm_state_data = FIVEVMState::from_account_data(&vm_account.data)
        .expect("failed to decode FIVEVMState");
    let authority_bytes: [u8; 32] = vm_state_data
        .authority
        .as_ref()
        .try_into()
        .expect("invalid authority length");
    let authority = Pubkey::new_from_array(authority_bytes);

    println!("VM_STATE_PARITY");
    println!("  rpc_url: {}", rpc_url);
    println!("  program_id: {}", program_id);
    println!("  vm_state: {}", vm_state);
    println!("  canonical_bump: {}", bump);
    println!("  owner: {}", vm_account.owner);
    println!("  authority: {}", authority);
    println!("  script_count: {}", vm_state_data.script_count);
    println!("  deploy_fee_lamports: {}", vm_state_data.deploy_fee_lamports);
    println!("  execute_fee_lamports: {}", vm_state_data.execute_fee_lamports);
    println!("  is_initialized: {}", vm_state_data.is_initialized());

    assert_eq!(
        authority, expected_authority,
        "authority mismatch: expected {}, got {}",
        expected_authority, authority
    );
    assert!(
        vm_state_data.is_initialized(),
        "vm_state is not initialized"
    );
    assert_eq!(
        vm_state_data.deploy_fee_lamports, expected_deploy_fee,
        "deploy fee mismatch: expected {}, got {}",
        expected_deploy_fee, vm_state_data.deploy_fee_lamports
    );
    assert_eq!(
        vm_state_data.execute_fee_lamports, expected_execute_fee,
        "execute fee mismatch: expected {}, got {}",
        expected_execute_fee, vm_state_data.execute_fee_lamports
    );
}
