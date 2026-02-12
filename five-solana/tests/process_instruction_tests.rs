// Simple tests for process_instruction that don't require mock accounts

#[cfg(test)]
mod tests {
    use five::instructions::{
        execute, set_fees, FIVEInstruction,
        DEPLOY_INSTRUCTION, EXECUTE_INSTRUCTION,
    };
    use five::state::{FIVEVMState, ScriptAccountHeader};
    use five_protocol::bytecode;
    use pinocchio::account_info::AccountInfo;
    use pinocchio::program_error::ProgramError;
    use pinocchio::pubkey::Pubkey;

    fn create_account(
        key: &Pubkey,
        is_signer: bool,
        is_writable: bool,
        lamports: &mut u64,
        data: &mut [u8],
        owner: &Pubkey,
    ) -> AccountInfo {
        AccountInfo::new(key, is_signer, is_writable, lamports, data, owner, false, 0)
    }

    fn key(seed: u8) -> Pubkey {
        [seed; 32]
    }

    #[test]
    fn test_instruction_parsing() {
        // Test Initialize
        let init_data = vec![0, 0]; // Disc + Bump
        let init_ix = FIVEInstruction::try_from(init_data.as_slice()).unwrap();
        assert!(matches!(init_ix, FIVEInstruction::Initialize { bump: _ }));

        // Test InitLargeProgram
        let mut init_large_data = vec![4];
        init_large_data.extend_from_slice(&1234u32.to_le_bytes());
        let init_large_ix = FIVEInstruction::try_from(init_large_data.as_slice()).unwrap();
        match init_large_ix {
            FIVEInstruction::InitLargeProgram { expected_size, chunk_data: _ } => {
                assert_eq!(expected_size, 1234);
            }
            _ => panic!("Expected InitLargeProgram instruction"),
        }

        // Test AppendBytecode
        let append_data = vec![5, 1, 2, 3];
        let append_ix = FIVEInstruction::try_from(append_data.as_slice()).unwrap();
        match append_ix {
            FIVEInstruction::AppendBytecode { data } => {
                assert_eq!(data, &[1, 2, 3]);
            }
            _ => panic!("Expected AppendBytecode instruction"),
        }

        // Test Deploy (v4: now includes permissions byte)
        let bytecode = vec![0x35, 0x49, 0x56, 0x45, 0x00]; // 5IVE + HALT
        let permissions = 0x00u8; // No permissions
        let mut deploy_data = vec![DEPLOY_INSTRUCTION];
        deploy_data.extend_from_slice(&(bytecode.len() as u32).to_le_bytes());
        deploy_data.push(permissions);
        deploy_data.extend_from_slice(&0u32.to_le_bytes());
        deploy_data.extend_from_slice(&bytecode);

        let deploy_ix = FIVEInstruction::try_from(deploy_data.as_slice()).unwrap();
        match deploy_ix {
            FIVEInstruction::Deploy { bytecode: bc, metadata, permissions: perms } => {
                assert_eq!(bc, &bytecode[..]);
                assert!(metadata.is_empty());
                assert_eq!(perms, permissions);
            }
            _ => panic!("Expected Deploy instruction"),
        }

        // Test Execute with canonical payload:
        // [function_index:u32 LE][param_count:u32 LE]
        let mut input_params = Vec::new();
        input_params.extend_from_slice(&0u32.to_le_bytes());
        input_params.extend_from_slice(&0u32.to_le_bytes());
        let mut exec_data = vec![EXECUTE_INSTRUCTION];
        exec_data.extend_from_slice(&input_params);

        let exec_ix = FIVEInstruction::try_from(exec_data.as_slice()).unwrap();
        match exec_ix {
            FIVEInstruction::Execute { params } => {
                assert_eq!(params, &input_params[..]);
            }
            _ => panic!("Expected Execute instruction"),
        }

    }

    #[test]
    fn test_invalid_instructions() {
        // Empty data
        assert!(matches!(
            FIVEInstruction::try_from(&[][..]),
            Err(ProgramError::InvalidInstructionData)
        ));

        // Invalid discriminator
        assert!(matches!(
            FIVEInstruction::try_from(&[99][..]),
            Err(ProgramError::InvalidInstructionData)
        ));

        // Truncated deploy (missing permissions and bytecode)
        assert!(matches!(
            FIVEInstruction::try_from(&[DEPLOY_INSTRUCTION, 10, 0, 0][..]),
            Err(ProgramError::InvalidInstructionData)
        ));

        // Truncated InitLargeProgram
        assert!(matches!(
            FIVEInstruction::try_from(&[4, 1, 2][..]),
            Err(ProgramError::InvalidInstructionData)
        ));

        // Truncated AppendBytecode
        assert!(matches!(
            FIVEInstruction::try_from(&[5][..]),
            Err(ProgramError::InvalidInstructionData)
        ));

        // Truncated ABI
        assert!(matches!(
            FIVEInstruction::try_from(&[3, 1, 2, 3][..]), // Not enough bytes for hash
            Err(ProgramError::InvalidInstructionData)
        ));
    }

    #[test]
    fn test_deploy_instruction_bounds() {
        // Test maximum reasonable bytecode size
        let large_bytecode = vec![0x35, 0x49, 0x56, 0x45]; // Just 5IVE magic
        let permissions = 0x00u8;
        let mut deploy_data = vec![DEPLOY_INSTRUCTION];
        deploy_data.extend_from_slice(&(large_bytecode.len() as u32).to_le_bytes());
        deploy_data.push(permissions);
        deploy_data.extend_from_slice(&0u32.to_le_bytes());
        deploy_data.extend_from_slice(&large_bytecode);

        let deploy_ix = FIVEInstruction::try_from(deploy_data.as_slice()).unwrap();
        match deploy_ix {
            FIVEInstruction::Deploy { bytecode, metadata, permissions: perms } => {
                assert_eq!(bytecode.len(), 4);
                assert_eq!(bytecode, &[0x35, 0x49, 0x56, 0x45]);
                assert!(metadata.is_empty());
                assert_eq!(perms, permissions);
            }
            _ => panic!("Expected Deploy instruction"),
        }
    }

    #[test]
    fn test_execute_instruction_empty_input() {
        // Test Execute with no input data
        let exec_data = vec![EXECUTE_INSTRUCTION];

        let exec_ix = FIVEInstruction::try_from(exec_data.as_slice()).unwrap();
        match exec_ix {
            FIVEInstruction::Execute { params } => {
                assert_eq!(params.len(), 0);
            }
            _ => panic!("Expected Execute instruction"),
        }
    }

    #[test]
    fn test_set_fees_updates_state() {
        let program_id = key(1);
        let authority_key = key(2);
        let mut vm_lamports = 0u64;
        let mut vm_data = vec![0u8; FIVEVMState::LEN];
        let mut authority_lamports = 0u64;
        let mut authority_data = [];

        {
            let vm_state = FIVEVMState::from_account_data_mut(&mut vm_data).unwrap();
            vm_state.initialize(authority_key);
        }

        let vm_key = key(3);
        let vm_account = create_account(
            &vm_key,
            false,
            true,
            &mut vm_lamports,
            &mut vm_data,
            &program_id,
        );
        let authority_account = create_account(
            &authority_key,
            true,
            false,
            &mut authority_lamports,
            &mut authority_data,
            &program_id,
        );

        let deploy_fee_lamports = 250;
        let execute_fee_lamports = 150;
        let accounts = [vm_account, authority_account];
        set_fees(&program_id, &accounts, deploy_fee_lamports, execute_fee_lamports).unwrap();

        let updated_data = accounts[0].try_borrow_data().unwrap();
        let updated = FIVEVMState::from_account_data(&updated_data).unwrap();
        assert_eq!(updated.deploy_fee_lamports, deploy_fee_lamports);
        assert_eq!(updated.execute_fee_lamports, execute_fee_lamports);
    }

    #[test]
    fn test_execute_transfers_fee_to_admin() {
        let program_id = key(10);
        let admin_key = key(11);
        let payer_key = key(12);
        let script_key = key(13);
        let vm_key = key(14);

        let mut vm_lamports = 0u64;
        let mut vm_data = vec![0u8; FIVEVMState::LEN];
        {
            let vm_state = FIVEVMState::from_account_data_mut(&mut vm_data).unwrap();
            vm_state.initialize(admin_key);
            vm_state.execute_fee_lamports = 200;
        }

        let test_bytecode = bytecode!(emit_header(0, 0), emit_halt());
        let mut script_data = vec![0u8; ScriptAccountHeader::LEN + test_bytecode.len()];
        let header = ScriptAccountHeader::create_from_bytecode(
            &test_bytecode,
            payer_key,
            0,
            0,
        );
        header.copy_into_account(&mut script_data).unwrap();
        script_data[ScriptAccountHeader::LEN..ScriptAccountHeader::LEN + test_bytecode.len()]
            .copy_from_slice(&test_bytecode);

        let mut script_lamports = 0u64;
        let mut payer_lamports = 1_000u64;
        let mut admin_lamports = 0u64;
        let mut payer_data = [];
        let mut admin_data = [];

        let script_account = create_account(
            &script_key,
            false,
            false,
            &mut script_lamports,
            &mut script_data,
            &program_id,
        );
        let vm_account = create_account(
            &vm_key,
            false,
            false,
            &mut vm_lamports,
            &mut vm_data,
            &program_id,
        );
        let payer_account = create_account(
            &payer_key,
            true,
            true,
            &mut payer_lamports,
            &mut payer_data,
            &program_id,
        );
        let admin_account = create_account(
            &admin_key,
            false,
            false,
            &mut admin_lamports,
            &mut admin_data,
            &program_id,
        );

        let fee = 200u64;
        execute(
            &program_id,
            &[script_account, vm_account, payer_account, admin_account],
            &[],
        )
        .unwrap();

        assert_eq!(payer_account.lamports(), 1_000u64 - fee);
        assert_eq!(admin_account.lamports(), fee);
    }

    #[test]
    fn test_execute_fee_requires_admin_account() {
        let program_id = key(20);
        let admin_key = key(21);
        let payer_key = key(22);
        let script_key = key(23);
        let vm_key = key(24);

        let mut vm_lamports = 0u64;
        let mut vm_data = vec![0u8; FIVEVMState::LEN];
        {
            let vm_state = FIVEVMState::from_account_data_mut(&mut vm_data).unwrap();
            vm_state.initialize(admin_key);
            vm_state.execute_fee_lamports = 100;
        }

        let test_bytecode = bytecode!(emit_header(0, 0), emit_halt());
        let mut script_data = vec![0u8; ScriptAccountHeader::LEN + test_bytecode.len()];
        let header = ScriptAccountHeader::create_from_bytecode(
            &test_bytecode,
            payer_key,
            0,
            0,
        );
        header.copy_into_account(&mut script_data).unwrap();
        script_data[ScriptAccountHeader::LEN..ScriptAccountHeader::LEN + test_bytecode.len()]
            .copy_from_slice(&test_bytecode);

        let mut script_lamports = 0u64;
        let mut payer_lamports = 1_000u64;
        let mut payer_data = [];

        let script_account = create_account(
            &script_key,
            false,
            false,
            &mut script_lamports,
            &mut script_data,
            &program_id,
        );
        let vm_account = create_account(
            &vm_key,
            false,
            false,
            &mut vm_lamports,
            &mut vm_data,
            &program_id,
        );
        let payer_account = create_account(
            &payer_key,
            true,
            true,
            &mut payer_lamports,
            &mut payer_data,
            &program_id,
        );

        let result = execute(
            &program_id,
            &[script_account, vm_account, payer_account],
            &[],
        );
        assert!(matches!(result, Err(ProgramError::Custom(1107))));
    }

    #[test]
    fn test_initialize_sets_default_fees() {
        let authority_key = key(1);
        let mut vm_data = vec![0u8; FIVEVMState::LEN];

        {
            let vm_state = FIVEVMState::from_account_data_mut(&mut vm_data).unwrap();
            vm_state.initialize(authority_key);
        }

        let vm_state = FIVEVMState::from_account_data(&vm_data).unwrap();
        assert_eq!(vm_state.deploy_fee_lamports, 10_000);
        assert_eq!(vm_state.execute_fee_lamports, 85_734);
        assert!(vm_state.is_initialized());
    }

    #[test]
    fn test_execute_charges_full_fee() {
        let program_id = key(30);
        let admin_key = key(31);
        let payer_key = key(32);
        let script_key = key(33);
        let vm_key = key(34);

        let mut vm_lamports = 0u64;
        let mut vm_data = vec![0u8; FIVEVMState::LEN];
        {
            let vm_state = FIVEVMState::from_account_data_mut(&mut vm_data).unwrap();
            vm_state.initialize(admin_key);
            vm_state.execute_fee_lamports = 5_000;
        }

        let test_bytecode = bytecode!(emit_header(0, 0), emit_halt());
        let mut script_data = vec![0u8; ScriptAccountHeader::LEN + test_bytecode.len()];
        let header = ScriptAccountHeader::create_from_bytecode(
            &test_bytecode,
            payer_key,
            0,
            0,
        );
        header.copy_into_account(&mut script_data).unwrap();
        script_data[ScriptAccountHeader::LEN..ScriptAccountHeader::LEN + test_bytecode.len()]
            .copy_from_slice(&test_bytecode);

        let mut script_lamports = 0u64;
        let mut payer_lamports = 10_000u64;
        let mut admin_lamports = 0u64;
        let mut payer_data = [];
        let mut admin_data = [];

        let script_account = create_account(
            &script_key,
            false,
            false,
            &mut script_lamports,
            &mut script_data,
            &program_id,
        );
        let vm_account = create_account(
            &vm_key,
            false,
            false,
            &mut vm_lamports,
            &mut vm_data,
            &program_id,
        );
        let payer_account = create_account(
            &payer_key,
            true,
            true,
            &mut payer_lamports,
            &mut payer_data,
            &program_id,
        );
        let admin_account = create_account(
            &admin_key,
            false,
            false,
            &mut admin_lamports,
            &mut admin_data,
            &program_id,
        );

        let expected_fee = 5_000u64;
        execute(
            &program_id,
            &[script_account, vm_account, payer_account, admin_account],
            &[],
        )
        .unwrap();

        assert_eq!(payer_account.lamports(), 10_000u64 - expected_fee);
        assert_eq!(admin_account.lamports(), expected_fee);
    }

}
