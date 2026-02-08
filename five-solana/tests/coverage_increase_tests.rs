#[cfg(test)]
mod coverage_increase_tests {
    use five::instructions::{
        init_large_program, append_bytecode, finalize_script_upload,
    };
    use five::state::{FIVEVMState, ScriptAccountHeader};
    use five_protocol::bytecode;
    use pinocchio::account_info::AccountInfo;
    use pinocchio::program_error::ProgramError;
    use pinocchio::pubkey::Pubkey;

    // --- Helper Functions ---

    fn create_account<'a>(
        key: &'a Pubkey,
        is_signer: bool,
        is_writable: bool,
        lamports: &'a mut u64,
        data: &'a mut [u8],
        owner: &'a Pubkey,
    ) -> AccountInfo {
        AccountInfo::new(key, is_signer, is_writable, lamports, data, owner, false, 0)
    }

    fn key(seed: u8) -> Pubkey {
        [seed; 32]
    }

    fn create_vm_state(admin_key: Pubkey) -> (u64, Vec<u8>) {
        let vm_lamports = 0u64;
        let mut vm_data = vec![0u8; FIVEVMState::LEN];
        {
            // SAFETY: We are creating fresh data for tests
            let vm_state = FIVEVMState::from_account_data_mut(&mut vm_data).unwrap();
            vm_state.initialize(admin_key);
            // Disable deploy fee to avoid Rent syscall in tests
            vm_state.deploy_fee_bps = 0;
        }
        (vm_lamports, vm_data)
    }

    #[test]
    fn test_finalize_script_upload_manual() {
        let program_id = key(80);
        let admin_key = key(81);
        let owner_key = key(82);
        let script_key = key(83);
        let vm_key = key(84);

        let (mut vm_lamports, mut vm_data) = create_vm_state(admin_key);

        let test_bytecode = bytecode!(emit_header(0, 0), emit_halt());
        let expected_size = test_bytecode.len() as u32;
        let required_size = ScriptAccountHeader::LEN + expected_size as usize;

        let mut script_lamports = 0u64;
        let mut script_data = vec![0u8; required_size];
        let mut owner_lamports = 10_000u64;
        let mut owner_data = [];

        let script_account = create_account(
            &script_key,
            false,
            true,
            &mut script_lamports,
            &mut script_data,
            &program_id,
        );
        let owner_account = create_account(
            &owner_key,
            true,
            true,
            &mut owner_lamports,
            &mut owner_data,
            &program_id,
        );
        let vm_account = create_account(
            &vm_key,
            false,
            true,
            &mut vm_lamports,
            &mut vm_data,
            &program_id,
        );

        // 1. Initialize
        let accounts = [script_account.clone(), owner_account.clone(), vm_account.clone()];
        init_large_program(&program_id, &accounts, expected_size, None).unwrap();

        // 2. Simulate upload complete but not finalized
        {
            let mut script_data_ref = script_account.try_borrow_mut_data().unwrap();
            let header = ScriptAccountHeader::from_account_data_mut(&mut script_data_ref).unwrap();
            header.set_upload_len(expected_size);
            // header.set_upload_mode(true); // Already true from init
            // And write the bytecode
            script_data_ref[ScriptAccountHeader::LEN..ScriptAccountHeader::LEN + expected_size as usize]
                .copy_from_slice(&test_bytecode);
        }

        // Now call finalize
        let accounts_finalize = [script_account.clone(), owner_account.clone()];
        let result = finalize_script_upload(&program_id, &accounts_finalize);
        assert!(result.is_ok());

        let script_data_ref = script_account.try_borrow_data().unwrap();
        let header = ScriptAccountHeader::from_account_data(&script_data_ref).unwrap();
        assert!(!header.upload_mode());
        assert!(header.upload_complete());
    }

    #[test]
    fn test_init_large_program_chunk_too_large() {
        let program_id = key(90);
        let admin_key = key(91);
        let owner_key = key(92);
        let script_key = key(93);
        let vm_key = key(94);

        let (mut vm_lamports, mut vm_data) = create_vm_state(admin_key);
        let expected_size = 10;
        let required_size = ScriptAccountHeader::LEN + expected_size as usize;

        let mut script_lamports = 0u64;
        let mut script_data = vec![0u8; required_size];
        let mut owner_lamports = 10_000u64;
        let mut owner_data = [];

        let script_account = create_account(
            &script_key,
            false,
            true,
            &mut script_lamports,
            &mut script_data,
            &program_id,
        );
        let owner_account = create_account(
            &owner_key,
            true,
            true,
            &mut owner_lamports,
            &mut owner_data,
            &program_id,
        );
        let vm_account = create_account(
            &vm_key,
            false,
            true,
            &mut vm_lamports,
            &mut vm_data,
            &program_id,
        );

        let accounts = [script_account.clone(), owner_account.clone(), vm_account.clone()];

        let chunk = vec![0u8; 20]; // Larger than expected_size
        let result = init_large_program(
            &program_id,
            &accounts,
            expected_size,
            Some(&chunk),
        );

        // Error 8207: Initial chunk too large
        assert_eq!(result, Err(ProgramError::Custom(8207)));
    }

    #[test]
    fn test_append_bytecode_empty_chunk() {
        let program_id = key(100);
        let admin_key = key(101);
        let owner_key = key(102);
        let script_key = key(103);
        let vm_key = key(104);

        let (mut vm_lamports, mut vm_data) = create_vm_state(admin_key);
        let expected_size = 100;
        let required_size = ScriptAccountHeader::LEN + expected_size as usize;

        let mut script_lamports = 0u64;
        let mut script_data = vec![0u8; required_size];
        let mut owner_lamports = 10_000u64;
        let mut owner_data = [];

        let script_account = create_account(
            &script_key,
            false,
            true,
            &mut script_lamports,
            &mut script_data,
            &program_id,
        );
        let owner_account = create_account(
            &owner_key,
            true,
            true,
            &mut owner_lamports,
            &mut owner_data,
            &program_id,
        );
        let vm_account = create_account(
            &vm_key,
            false,
            true,
            &mut vm_lamports,
            &mut vm_data,
            &program_id,
        );

        let accounts = [script_account.clone(), owner_account.clone(), vm_account.clone()];
        init_large_program(&program_id, &accounts, expected_size, None).unwrap();

        let empty_chunk = vec![];
        let result = append_bytecode(
            &program_id,
            &accounts,
            &empty_chunk,
        );

        // Error 8201: Empty chunk
        assert_eq!(result, Err(ProgramError::Custom(8201)));
    }
}
