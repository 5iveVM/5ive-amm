//! Syscall/CPI runtime behavior is covered in BPF suites.
//! Keep only fast payload-shape checks in-process.

mod harness;

use harness::fixtures::{canonical_execute_payload, TypedParam};
use pinocchio::pubkey::Pubkey;

#[test]
fn payload_can_encode_syscall_style_params() {
    let payload = canonical_execute_payload(1, &[TypedParam::U64(42)]);
    assert!(payload.len() > 8);
    assert_eq!(&payload[0..4], &1u32.to_le_bytes());
}

#[test]
fn script_scoped_pda_domains_prevent_cross_script_vault_collision() {
    let program_id = Pubkey::from([41u8; 32]);
    let script_a = Pubkey::from([42u8; 32]);
    let script_b = Pubkey::from([43u8; 32]);
    let user_seed = b"vault_authority";

    let (a_pda, _a_bump) = five_vm_mito::utils::find_program_address_offchain(
        &[script_a.as_ref(), user_seed],
        &program_id,
    )
    .expect("derive script A signer PDA");
    let (b_pda, _b_bump) = five_vm_mito::utils::find_program_address_offchain(
        &[script_b.as_ref(), user_seed],
        &program_id,
    )
    .expect("derive script B signer PDA");

    assert_ne!(
        a_pda, b_pda,
        "same user seeds must resolve to different signer domains per script"
    );
}
