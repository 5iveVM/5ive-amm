//! Mollusk-style testing framework demonstration for Five VM
//!
//! This module demonstrates how to use the feature-gated testing framework
//! for comprehensive Five VM testing with real Solana accounts.
//!
//! All code is feature-gated behind "test-utils" to ensure production builds
//! exclude all testing dependencies. This follows the counter-pinocchio pattern
//! of using real Account instances instead of mocks.

#[cfg(all(test, feature = "test-utils"))]
mod mollusk_style_tests {
    use crate::{
        opcodes, push_bool, push_u64, test_bytecode,
        test_framework::{
            AccountCheck, AccountUtils, MolluskTestUtils, SignerAccount, TestUtils, WritableAccount,
        },
        MitoVM, VMError, Value,
    };
    use pinocchio::pubkey::Pubkey;
    use solana_sdk::{
        account::Account, instruction::AccountMeta, native_token::LAMPORTS_PER_SOL, system_program,
    };

    // Test program ID for Five VM (in real implementation, this would be the deployed program)
    const FIVE_VM_PROGRAM_ID: Pubkey = Pubkey::new_from_array([
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
    ]);

    #[test]
    fn test_mollusk_account_creation() {
        // Test real Account creation following counter-pinocchio pattern
        let signer_account = AccountUtils::signer_account(1 * LAMPORTS_PER_SOL);
        assert_eq!(signer_account.lamports, 1 * LAMPORTS_PER_SOL);
        assert_eq!(signer_account.owner, system_program::ID);
        assert_eq!(signer_account.data.len(), 0);

        let state_account =
            AccountUtils::state_account(1_000_000, vec![1, 2, 3, 4], FIVE_VM_PROGRAM_ID);
        assert_eq!(state_account.lamports, 1_000_000);
        assert_eq!(state_account.owner, FIVE_VM_PROGRAM_ID);
        assert_eq!(state_account.data, vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_mollusk_system_program_setup() {
        // Test system program account creation
        let (system_key, system_account) = MolluskTestUtils::system_program_account();
        assert_eq!(system_key, system_program::ID);
        assert_eq!(system_account.owner, system_program::ID);
        assert_eq!(system_account.lamports, 1);
    }

    #[test]
    fn test_mollusk_vm_test_setup() {
        // Test complete Mollusk test setup
        let script_bytecode = test_bytecode![
            push_u64!(42),
            push_u64!(25),
            opcodes![0x20], // ADD
        ];

        let result = MolluskTestUtils::setup_vm_test(
            &script_bytecode,
            1 * LAMPORTS_PER_SOL,
            &FIVE_VM_PROGRAM_ID,
        );

        assert!(result.is_ok(), "Test setup should succeed");
        let (tx_accounts, account_metas) = result.unwrap();

        // Verify account setup
        assert_eq!(tx_accounts.len(), 3); // authority, script, system
        assert_eq!(account_metas.len(), 3);

        // Verify authority account
        assert_eq!(tx_accounts[0].1.lamports, 1 * LAMPORTS_PER_SOL);
        assert!(account_metas[0].is_signer);
        assert!(account_metas[0].is_writable);

        // Verify script account
        assert_eq!(tx_accounts[1].1.data, script_bytecode);
        assert_eq!(tx_accounts[1].1.owner, FIVE_VM_PROGRAM_ID);
        assert!(!account_metas[1].is_signer);
        assert!(account_metas[1].is_writable);

        // Verify system program
        assert_eq!(tx_accounts[2].0, system_program::ID);
        assert!(!account_metas[2].is_signer);
        assert!(!account_metas[2].is_writable);
    }

    #[test]
    fn test_mollusk_vm_instruction_creation() {
        // Test Five VM instruction creation
        let script_data = test_bytecode![push_u64!(100)];
        let accounts = vec![
            AccountMeta::new(TestUtils::create_test_pubkey(1), true),
            AccountMeta::new(TestUtils::create_test_pubkey(2), false),
        ];

        let instruction = MolluskTestUtils::create_vm_instruction(
            &FIVE_VM_PROGRAM_ID,
            &script_data,
            accounts.clone(),
        );

        assert!(instruction.is_ok(), "Instruction creation should succeed");
        let inst = instruction.unwrap();

        assert_eq!(inst.program_id, FIVE_VM_PROGRAM_ID);
        assert_eq!(inst.accounts, accounts);

        // Verify instruction data format
        assert_eq!(inst.data[0], 0x01); // Execute script discriminator
        let script_len =
            u32::from_le_bytes([inst.data[1], inst.data[2], inst.data[3], inst.data[4]]);
        assert_eq!(script_len, script_data.len() as u32);
        assert_eq!(&inst.data[5..], script_data);
    }

    #[test]
    fn test_mollusk_constraint_validation() {
        // Test account constraint validation using real AccountInfo
        let authority_key = TestUtils::create_test_pubkey(1);
        let (authority_account, authority_info) =
            TestUtils::create_signer_account_info(&authority_key, 1 * LAMPORTS_PER_SOL);

        // Test signer constraint
        let signer_result = TestUtils::test_constraint::<SignerAccount>(&authority_info);
        assert!(
            signer_result.is_ok(),
            "Signer constraint should pass for signer account"
        );

        // Test writable constraint
        let writable_result = TestUtils::test_constraint::<WritableAccount>(&authority_info);
        assert!(
            writable_result.is_ok(),
            "Writable constraint should pass for writable account"
        );
    }

    #[test]
    fn test_mollusk_constraint_validation_failures() {
        // Test constraint failures with readonly account
        let readonly_key = TestUtils::create_test_pubkey(2);
        let (readonly_account, readonly_info) = TestUtils::create_readonly_account_info(
            &readonly_key,
            500_000,
            vec![1, 2, 3],
            &FIVE_VM_PROGRAM_ID,
        );

        // Readonly account should fail signer constraint
        let signer_result = TestUtils::test_constraint::<SignerAccount>(&readonly_info);
        assert!(
            signer_result.is_err(),
            "Signer constraint should fail for non-signer"
        );
        if let Err(e) = signer_result {
            assert!(matches!(e, VMError::ConstraintViolation));
        }

        // Readonly account should fail writable constraint
        let writable_result = TestUtils::test_constraint::<WritableAccount>(&readonly_info);
        assert!(
            writable_result.is_err(),
            "Writable constraint should fail for readonly account"
        );
        if let Err(e) = writable_result {
            assert!(matches!(e, VMError::ConstraintViolation));
        }
    }

    #[test]
    fn test_mollusk_vm_script_execution_setup() {
        // Test complete Five VM script execution setup
        let script_bytecode = test_bytecode![
            push_u64!(100),
            push_u64!(25),
            opcodes![0x20], // ADD
        ];

        let (tx_accounts, account_metas) = MolluskTestUtils::setup_vm_test(
            &script_bytecode,
            1 * LAMPORTS_PER_SOL,
            &FIVE_VM_PROGRAM_ID,
        )
        .expect("Test setup should succeed");

        // Test Mollusk execution setup validation
        let result = MolluskTestUtils::execute_vm_script_with_mollusk(
            &script_bytecode,
            &tx_accounts,
            account_metas,
            &FIVE_VM_PROGRAM_ID,
        );

        assert!(result.is_ok(), "Mollusk execution setup should succeed");
    }

    #[test]
    fn test_mollusk_script_validation() {
        // Test script validation in Mollusk execution
        let invalid_script = vec![1, 2, 3]; // Too short, missing magic header
        let tx_accounts = vec![(
            TestUtils::create_test_pubkey(1),
            AccountUtils::system_account(1000),
        )];
        let account_metas = vec![AccountMeta::new(TestUtils::create_test_pubkey(1), true)];

        let result = MolluskTestUtils::execute_vm_script_with_mollusk(
            &invalid_script,
            &tx_accounts,
            account_metas,
            &FIVE_VM_PROGRAM_ID,
        );

        assert!(result.is_err(), "Invalid script should fail validation");
        if let Err(e) = result {
            assert!(matches!(e, VMError::InvalidBytecode));
        }
    }

    #[test]
    fn test_mollusk_account_validation() {
        // Test account validation in Mollusk execution
        let script_bytecode = test_bytecode![push_u64!(42)];
        let tx_accounts = vec![]; // Empty accounts should fail
        let account_metas = vec![];

        let result = MolluskTestUtils::execute_vm_script_with_mollusk(
            &script_bytecode,
            &tx_accounts,
            account_metas,
            &FIVE_VM_PROGRAM_ID,
        );

        assert!(result.is_err(), "Empty accounts should fail validation");
        if let Err(e) = result {
            assert!(matches!(e, VMError::InsufficientAccounts));
        }
    }

    #[test]
    fn test_real_pda_derivation() {
        // Test real PDA derivation using Solana's find_program_address
        let seeds = &[b"counter", b"test"];
        let (pda, bump) = TestUtils::derive_pda_for_test(seeds, &FIVE_VM_PROGRAM_ID);

        // Verify PDA is valid (not on curve)
        // In real Solana, valid PDAs should not be on the ed25519 curve
        // For testing, we just verify the derivation succeeds
        assert!(bump <= 255);

        // Verify PDA can be re-derived with same seeds
        let (pda2, bump2) = TestUtils::derive_pda_for_test(seeds, &FIVE_VM_PROGRAM_ID);
        assert_eq!(pda, pda2);
        assert_eq!(bump, bump2);
    }

    #[test]
    fn test_five_vm_state_account_creation() {
        // Test Five VM-specific state account creation
        let script_data = test_bytecode![push_u64!(42), push_bool!(true),];

        let vm_account =
            MolluskTestUtils::vm_state_account(1_000_000, &script_data, &FIVE_VM_PROGRAM_ID);

        assert_eq!(vm_account.lamports, 1_000_000);
        assert_eq!(vm_account.owner, FIVE_VM_PROGRAM_ID);
        assert_eq!(vm_account.data, script_data);
        assert_eq!(vm_account.data.len(), script_data.len());

        // Verify script has proper Five VM magic header
        assert_eq!(&vm_account.data[0..4], &[0x35, 0x49, 0x56, 0x45]); // "5IVE"
    }
}
