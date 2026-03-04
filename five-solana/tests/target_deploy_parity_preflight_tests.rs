mod harness;

#[test]
fn target_deploy_parity_message_identifies_both_program_ids_and_rebuild_command() {
    let err = harness::validate_target_deploy_program_id_parity(
        "ExpectedProgram1111111111111111111111111111111",
        "ActualProgram22222222222222222222222222222222",
    )
    .expect_err("mismatched program ids must fail parity validation");

    assert!(err.contains("generated_constants.rs VM_PROGRAM_ID"));
    assert!(err.contains("ExpectedProgram1111111111111111111111111111111"));
    assert!(err.contains("target/deploy/five-keypair.json pubkey"));
    assert!(err.contains("ActualProgram22222222222222222222222222222222"));
    assert!(err.contains("./scripts/build-five-solana-cluster.sh --cluster localnet"));
}

#[test]
fn target_deploy_parity_accepts_matching_program_ids() {
    harness::validate_target_deploy_program_id_parity(
        "SameProgram333333333333333333333333333333333",
        "SameProgram333333333333333333333333333333333",
    )
    .expect("matching program ids must pass parity validation");
}
