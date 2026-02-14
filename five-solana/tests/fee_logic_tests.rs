#[cfg(test)]
mod fee_logic_tests {
    use five::state::FIVEVMState;
    use pinocchio::pubkey::Pubkey;

    #[test]
    fn test_vm_state_defaults_use_flat_lamport_fees() {
        let mut vm_state = FIVEVMState::new();
        vm_state.initialize(Pubkey::default(), 0);
        assert_eq!(vm_state.deploy_fee_lamports, 10_000);
        assert_eq!(vm_state.execute_fee_lamports, 85_734);
    }
}
