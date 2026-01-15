// TDD: Write failing tests first for Pinocchio integration
use pinocchio::program_error::ProgramError;

#[cfg(test)]
mod tests {
    use super::*;
    use five::instructions::{FIVEInstruction, DEPLOY_INSTRUCTION, EXECUTE_INSTRUCTION};
    use five::state::FIVEVMState;
    use pinocchio::pubkey::Pubkey;

    // Since we're in tests and Pinocchio AccountInfo is not constructible directly,
    // we focus on instruction parsing and state management

    #[test]
    fn test_instruction_deserialization() {
        // Test Initialize instruction
        let init_data = vec![0u8; 1];
        let init_ix = FIVEInstruction::try_from(init_data.as_slice()).unwrap();
        assert!(matches!(init_ix, FIVEInstruction::Initialize));

        // Test Deploy instruction (v4: now includes permissions byte)
        let bytecode = vec![0x35u8, 0x49, 0x56, 0x45, 0x00]; // 5IVE + HALT
        let permissions = 0x04u8; // PERMISSION_PDA_SPECIAL_CHARS
        let mut deploy_data = vec![0u8; 1 + 4 + 1 + bytecode.len()];
        deploy_data[0] = DEPLOY_INSTRUCTION;
        deploy_data[1..5].copy_from_slice(&(bytecode.len() as u32).to_le_bytes());
        deploy_data[5] = permissions;
        deploy_data[6..6 + bytecode.len()].copy_from_slice(&bytecode);

        let deploy_ix = FIVEInstruction::try_from(&deploy_data[..]).unwrap();
        match deploy_ix {
            FIVEInstruction::Deploy { bytecode: bc, permissions: perms } => {
                assert_eq!(bc, &bytecode[..]);
                assert_eq!(perms, permissions);
            }
            _ => panic!("Expected Deploy instruction"),
        }

        // Test Execute instruction
        let params = vec![1u8, 2, 3];
        let mut exec_data = vec![0u8; 1 + params.len()];
        exec_data[0] = EXECUTE_INSTRUCTION;
        exec_data[1..].copy_from_slice(&params);

        let exec_ix = FIVEInstruction::try_from(&exec_data[..]).unwrap();
        match exec_ix {
            FIVEInstruction::Execute { params: p } => {
                assert_eq!(p, &params[..]);
            }
            _ => panic!("Expected Execute instruction"),
        }
    }

    #[test]
    fn test_invalid_instruction() {
        // Test empty data
        let result = FIVEInstruction::try_from(&[][..]);
        assert!(matches!(result, Err(ProgramError::InvalidInstructionData)));

        // Test invalid instruction type
        let result = FIVEInstruction::try_from(&[99][..]);
        assert!(matches!(result, Err(ProgramError::InvalidInstructionData)));

        // Test truncated deploy instruction
        let result = FIVEInstruction::try_from(&[DEPLOY_INSTRUCTION, 10, 0, 0][..]); // Missing permissions byte and bytecode
        assert!(matches!(result, Err(ProgramError::InvalidInstructionData)));
    }

    #[test]
    fn test_state_serialization() {
        use bytemuck;

        let mut state = FIVEVMState::new();
        let authority: Pubkey = [1u8; 32];
        state.initialize(authority);
        let id = state.create_script_id();
        assert_eq!(id, 0);
        assert!(state.is_initialized());

        let bytes = bytemuck::bytes_of(&state);
        assert_eq!(bytes.len(), FIVEVMState::LEN);
        let deserialized = bytemuck::from_bytes::<FIVEVMState>(bytes);
        assert_eq!(deserialized.authority, authority);
        assert_eq!(deserialized.script_count, 1);
    }

    #[test]
    fn test_adapter_key_conversion() {
        // Test that keys convert properly between formats
        let test_key: Pubkey = [42u8; 32];
        let key_bytes: [u8; 32] = test_key;
        assert_eq!(test_key, key_bytes);

        // Test key round-trip
        let original: Pubkey = [
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
            25, 26, 27, 28, 29, 30, 31, 32,
        ];
        let as_bytes: [u8; 32] = original;
        let back_to_pubkey: Pubkey = as_bytes;
        assert_eq!(original, back_to_pubkey);
    }
}