#[cfg(test)]
mod deployment_tests {
    use five::instructions::{
        deploy, init_large_program, append_bytecode,
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
        let mut vm_lamports = 0u64;
        let mut vm_data = vec![0u8; FIVEVMState::LEN];
        {
            // SAFETY: We are creating fresh data for tests
            let vm_state = unsafe { FIVEVMState::from_account_data_mut(&mut vm_data).unwrap() };
            vm_state.initialize(admin_key);
            // Disable deploy fee to avoid Rent syscall in tests
            vm_state.deploy_fee_bps = 0;
        }
        (vm_lamports, vm_data)
    }

    // --- Deploy Instruction Tests ---

    #[test]
    fn test_deploy_standard_success() {
        let program_id = key(1);
        let admin_key = key(2);
        let owner_key = key(3);
        let script_key = key(4);
        let vm_key = key(5);

        let (mut vm_lamports, mut vm_data) = create_vm_state(admin_key);

        let test_bytecode = bytecode!(emit_header(0, 0), emit_halt());
        let required_size = ScriptAccountHeader::LEN + test_bytecode.len();

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
        let vm_account = create_account(
            &vm_key,
            false,
            true,
            &mut vm_lamports,
            &mut vm_data,
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

        let accounts = [script_account.clone(), vm_account.clone(), owner_account.clone()];

        // Execute Deploy
        let result = deploy(
            &program_id,
            &accounts,
            &test_bytecode,
            0, // No permissions
        );
        assert!(result.is_ok());

        // Verify Script Header
        let script_data_ref = script_account.try_borrow_data().unwrap();
        let header = unsafe { ScriptAccountHeader::from_account_data(&script_data_ref).unwrap() };
        assert_eq!(header.owner, owner_key);
        assert_eq!(header.permissions, 0);
        assert_eq!(header.bytecode_len(), test_bytecode.len());

        // Verify Bytecode written
        let written_bytecode = &script_data_ref[ScriptAccountHeader::LEN..];
        assert_eq!(written_bytecode, &test_bytecode[..]);

        // Verify Fee Collection (Fee is disabled for test, so no change)
        assert_eq!(owner_account.lamports(), 10_000);
        assert_eq!(vm_account.lamports(), 0);
    }

    #[test]
    fn test_deploy_permissions_requires_admin() {
        let program_id = key(10);
        let admin_key = key(11);
        let owner_key = key(12); // Not admin
        let script_key = key(13);
        let vm_key = key(14);

        let (mut vm_lamports, mut vm_data) = create_vm_state(admin_key);

        let test_bytecode = bytecode!(emit_header(0, 0), emit_halt());
        let required_size = ScriptAccountHeader::LEN + test_bytecode.len();

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
        let vm_account = create_account(
            &vm_key,
            false,
            true,
            &mut vm_lamports,
            &mut vm_data,
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

        let accounts = [script_account.clone(), vm_account.clone(), owner_account.clone()];

        // Try to deploy with permissions (e.g., 0x01) without admin signer
        // Should fail because admin account is missing from the list entirely
        let result = deploy(
            &program_id,
            &accounts,
            &test_bytecode,
            0x01, // PERMISSION_PRE_BYTECODE
        );
        assert!(matches!(result, Err(ProgramError::NotEnoughAccountKeys)));

        // Add admin account but not as signer
        let mut admin_lamports = 0u64;
        let mut admin_data = [];
        let admin_account = create_account(
            &admin_key,
            false, // Not signer!
            false,
            &mut admin_lamports,
            &mut admin_data,
            &program_id,
        );

        let accounts_with_admin = [
            script_account.clone(),
            vm_account.clone(),
            owner_account.clone(),
            admin_account.clone()
        ];

        let result = deploy(
            &program_id,
            &accounts_with_admin,
            &test_bytecode,
            0x01,
        );
        assert_eq!(result, Err(ProgramError::MissingRequiredSignature));

        // Now make admin a signer
        let admin_account_signed = create_account(
            &admin_key,
            true, // Signer!
            false,
            &mut admin_lamports,
            &mut admin_data,
            &program_id,
        );
         let accounts_valid = [
            script_account.clone(),
            vm_account.clone(),
            owner_account.clone(),
            admin_account_signed.clone()
        ];

        let result = deploy(
            &program_id,
            &accounts_valid,
            &test_bytecode,
            0x01,
        );
        assert!(result.is_ok());

        // Verify permissions set in header
        let script_data_ref = script_account.try_borrow_data().unwrap();
        let header = unsafe { ScriptAccountHeader::from_account_data(&script_data_ref).unwrap() };
        assert_eq!(header.permissions, 0x01);
    }

    // NOTE: test_deploy_fee_collection_custom is omitted because Rent::get() syscall
    // cannot be easily mocked in this unit test environment.

    #[test]
    fn test_deploy_invalid_bytecode() {
        let program_id = key(30);
        let admin_key = key(31);
        let owner_key = key(32);
        let script_key = key(33);
        let vm_key = key(34);

        let (mut vm_lamports, mut vm_data) = create_vm_state(admin_key);

        let bad_bytecode = vec![0x00, 0x00, 0x00]; // Too short, no header
        let required_size = ScriptAccountHeader::LEN + bad_bytecode.len();

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
        let vm_account = create_account(
            &vm_key,
            false,
            true,
            &mut vm_lamports,
            &mut vm_data,
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

        let accounts = [script_account.clone(), vm_account.clone(), owner_account.clone()];

        let result = deploy(
            &program_id,
            &accounts,
            &bad_bytecode,
            0,
        );
        // Should fail due to size checks or magic bytes
        assert!(result.is_err());
    }

    // --- Large Program Upload Tests ---

    #[test]
    fn test_init_large_program_success() {
        let program_id = key(40);
        let admin_key = key(41);
        let owner_key = key(42);
        let script_key = key(43);
        let vm_key = key(44);

        let (mut vm_lamports, mut vm_data) = create_vm_state(admin_key);

        let expected_size = 100u32;
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

        let result = init_large_program(
            &program_id,
            &accounts,
            expected_size,
            None,
        );
        assert!(result.is_ok());

        let script_data_ref = script_account.try_borrow_data().unwrap();
        let header = unsafe { ScriptAccountHeader::from_account_data(&script_data_ref).unwrap() };
        assert!(header.upload_mode());
        assert!(!header.upload_complete());
        assert_eq!(header.upload_len(), 0);
        assert_eq!(header.bytecode_len(), expected_size as usize);
    }

    #[test]
    fn test_append_bytecode_success_and_finalize() {
        let program_id = key(50);
        let admin_key = key(51);
        let owner_key = key(52);
        let script_key = key(53);
        let vm_key = key(54);

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

        // 2. Append Bytecode (in one chunk for simplicity, or split it)
        let accounts_append = [script_account.clone(), owner_account.clone(), vm_account.clone()];
        let result = append_bytecode(
            &program_id,
            &accounts_append,
            &test_bytecode,
        );
        assert!(result.is_ok());

        // 3. Verify Finalization
        let script_data_ref = script_account.try_borrow_data().unwrap();
        let header = unsafe { ScriptAccountHeader::from_account_data(&script_data_ref).unwrap() };

        assert!(!header.upload_mode()); // Should be false after finalize
        assert!(header.upload_complete());
        assert_eq!(header.bytecode_len(), expected_size as usize);

        // Verify content
        let written_bytecode = &script_data_ref[ScriptAccountHeader::LEN..];
        assert_eq!(written_bytecode, &test_bytecode[..]);
    }

    #[test]
    fn test_append_bytecode_chunk_exceeds_size() {
        let program_id = key(60);
        let admin_key = key(61);
        let owner_key = key(62);
        let script_key = key(63);
        let vm_key = key(64);

        let (mut vm_lamports, mut vm_data) = create_vm_state(admin_key);
        let expected_size = 5;
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

        let large_chunk = vec![0u8; 10];
        let result = append_bytecode(
            &program_id,
            &accounts,
            &large_chunk,
        );

        // Error 8202: Chunk exceeds expected size
        assert_eq!(result, Err(ProgramError::Custom(8202)));
    }

    #[test]
    fn test_init_large_program_with_chunk() {
        let program_id = key(70);
        let admin_key = key(71);
        let owner_key = key(72);
        let script_key = key(73);
        let vm_key = key(74);

        let (mut vm_lamports, mut vm_data) = create_vm_state(admin_key);

        let expected_size = 100u32;
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

        let chunk = vec![1, 2, 3, 4, 5];
        let result = init_large_program(
            &program_id,
            &accounts,
            expected_size,
            Some(&chunk),
        );
        assert!(result.is_ok());

        let script_data_ref = script_account.try_borrow_data().unwrap();
        let header = unsafe { ScriptAccountHeader::from_account_data(&script_data_ref).unwrap() };

        assert!(header.upload_mode());
        assert!(!header.upload_complete());
        assert_eq!(header.upload_len(), 5);
        assert_eq!(header.bytecode_len(), expected_size as usize);

        // Verify chunk data written
        let written_chunk = &script_data_ref[ScriptAccountHeader::LEN..ScriptAccountHeader::LEN + 5];
        assert_eq!(written_chunk, &chunk[..]);
    }
}
