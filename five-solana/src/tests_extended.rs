// Extended tests for comprehensive coverage
use pinocchio::program_error::ProgramError;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instructions::{FIVEInstruction, DEPLOY_INSTRUCTION, EXECUTE_INSTRUCTION};
    use crate::state::FIVEVMState;
    use pinocchio::pubkey::Pubkey;

    #[test]
    fn test_edge_case_deploy_lengths() {
        // Test minimum valid deploy
        let mut min_deploy = [0u8; 9];
        min_deploy[0] = DEPLOY_INSTRUCTION;
        min_deploy[1..5].copy_from_slice(&4u32.to_le_bytes());
        min_deploy[5..9].copy_from_slice(&[0x35, 0x49, 0x56, 0x45]); // Just magic bytes

        let result = FIVEInstruction::try_from(&min_deploy[..]);
        assert!(result.is_ok());

        // Test deploy with too short length declaration
        let mut bad_deploy = [0u8; 8];
        bad_deploy[0] = DEPLOY_INSTRUCTION;
        bad_deploy[1..5].copy_from_slice(&10u32.to_le_bytes()); // Claims 10 bytes
        bad_deploy[5..8].copy_from_slice(&[0x35, 0x49, 0x56]); // Only provides 3

        let result = FIVEInstruction::try_from(&bad_deploy[..]);
        assert!(result.is_err());
    }

    #[test]
    fn test_state_overflow() {
        let mut state = FIVEVMState::new();
        let authority = Pubkey::from([0x77; 32]);
        state.initialize(authority);

        // Generate many script IDs
        for i in 0..100 {
            let id = state.create_script_id();
            assert_eq!(id, i);
        }
        assert_eq!(state.script_count, 100);
    }
}