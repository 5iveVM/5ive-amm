//! Constraint Validation Tests for Five VM
//!
//! Tests critical security constraint opcodes that validate account properties.

mod support;

use five_vm_mito::{FIVE_VM_PROGRAM_ID, MitoVM, Value, VMError};
use five_vm_mito::error::VMErrorCode;
use pinocchio::pubkey::Pubkey;
use support::accounts::{create_test_accounts, derive_pda_real};

#[cfg(test)]
mod basic_constraint_tests {
    use super::*;

    // Helper to get mocked accounts
    pub fn setup_accounts<'a>(
        lamports: &'a mut u64,
        data: &'a mut [u8],
        payer_lamports: &'a mut u64,
        payer_data: &'a mut [u8],
        sys_lamports: &'a mut u64,
        sys_data: &'a mut [u8]
    ) -> ([five_vm_mito::AccountInfo; 3], Pubkey, Pubkey, u8) {
        let program_id = Pubkey::from([0xAA; 32]);
        let seeds: &[&[u8]] = &[b"test"];
        let (pda_address, bump) = derive_pda_real(seeds, &program_id);

        let accounts = create_test_accounts(
            &program_id,
            &pda_address,
            lamports,
            data,
            payer_lamports,
            payer_data,
            sys_lamports,
            sys_data,
        );
        (accounts, program_id, pda_address, bump)
    }

    #[test]
    fn test_check_signer_valid() {
        let mut lamports = 100u64;
        let mut data = [0u8; 32];
        let mut payer_lamports = 1_000_000_000;
        let mut payer_data = [0u8; 0];
        let mut sys_lamports = 0u64;
        let mut sys_data = [0u8; 0];
        let (accounts, program_id, _, _) = setup_accounts(&mut lamports, &mut data, &mut payer_lamports, &mut payer_data, &mut sys_lamports, &mut sys_data);

        // Account 0 (payer) is signer
        let bytecode = vec![
            0x35, 0x49, 0x56, 0x45, 0, 0, 0, 0, 0, 0,
            0x70, // CHECK_SIGNER
            0x00, // Account index 0
            0x00, // HALT
        ];

        let result = MitoVM::execute_direct(&bytecode, &[], &accounts, &program_id);
        result.expect("CHECK_SIGNER should succeed for valid signer");
    }

    #[test]
    fn test_check_signer_invalid() {
        let mut lamports = 100u64;
        let mut data = [0u8; 32];
        let mut payer_lamports = 1_000_000_000;
        let mut payer_data = [0u8; 0];
        let mut sys_lamports = 0u64;
        let mut sys_data = [0u8; 0];
        let (accounts, program_id, _, _) = setup_accounts(&mut lamports, &mut data, &mut payer_lamports, &mut payer_data, &mut sys_lamports, &mut sys_data);

        // Account 2 (system program) is NOT signer
        let bytecode = vec![
            0x35, 0x49, 0x56, 0x45, 0, 0, 0, 0, 0, 0,
            0x70, // CHECK_SIGNER
            0x02, // Account index 2
            0x00,
        ];

        let result = MitoVM::execute_direct(&bytecode, &[], &accounts, &program_id);
        match result {
            Err(_) => println!("✅ CHECK_SIGNER correctly failed"),
            Ok(_) => panic!("CHECK_SIGNER should fail for non-signer"),
        }
    }

    #[test]
    fn test_check_writable_valid() {
        let mut lamports = 100u64;
        let mut data = [0u8; 32];
        let mut payer_lamports = 1_000_000_000;
        let mut payer_data = [0u8; 0];
        let mut sys_lamports = 0u64;
        let mut sys_data = [0u8; 0];
        let (accounts, program_id, _, _) = setup_accounts(&mut lamports, &mut data, &mut payer_lamports, &mut payer_data, &mut sys_lamports, &mut sys_data);

        // Account 0 is writable
        let bytecode = vec![
            0x35, 0x49, 0x56, 0x45, 0, 0, 0, 0, 0, 0,
            0x71, // CHECK_WRITABLE
            0x00, // Account index 0
            0x00, // HALT
        ];

        let result = MitoVM::execute_direct(&bytecode, &[], &accounts, &program_id);
        result.expect("CHECK_WRITABLE should succeed for writable account");
    }

    #[test]
    fn test_check_writable_invalid() {
        let mut lamports = 100u64;
        let mut data = [0u8; 32];
        let mut payer_lamports = 1_000_000_000;
        let mut payer_data = [0u8; 0];
        let mut sys_lamports = 0u64;
        let mut sys_data = [0u8; 0];
        let (accounts, program_id, _, _) = setup_accounts(&mut lamports, &mut data, &mut payer_lamports, &mut payer_data, &mut sys_lamports, &mut sys_data);

        // Account 2 is not writable
        let bytecode = vec![
            0x35, 0x49, 0x56, 0x45, 0, 0, 0, 0, 0, 0,
            0x71, // CHECK_WRITABLE
            0x02, // Account index 2
            0x00, // HALT
        ];

        let result = MitoVM::execute_direct(&bytecode, &[], &accounts, &program_id);
        match result {
            Err(_) => println!("✅ CHECK_WRITABLE correctly failed for read-only"),
            Ok(_) => panic!("CHECK_WRITABLE should fail for read-only account"),
        }
    }

    #[test]
    fn test_check_owner_valid() {
        let mut lamports = 100u64;
        let mut data = [0u8; 32];
        let mut payer_lamports = 1_000_000_000;
        let mut payer_data = [0u8; 0];
        let mut sys_lamports = 0u64;
        let mut sys_data = [0u8; 0];
        let (accounts, program_id, _, _) = setup_accounts(&mut lamports, &mut data, &mut payer_lamports, &mut payer_data, &mut sys_lamports, &mut sys_data);

        // Account 1 owner is program_id
        let expected_owner = program_id;

        let mut bytecode = vec![
            0x35, 0x49, 0x56, 0x45, 0, 0, 0, 0, 0, 0,
            0x72, // CHECK_OWNER
            0x01, // Account index 1
        ];

        // expected_owner (32 bytes)
        bytecode.extend_from_slice(expected_owner.as_ref());
        bytecode.push(0x00); // HALT

        let result = MitoVM::execute_direct(&bytecode, &[], &accounts, &program_id);
        result.expect("CHECK_OWNER should succeed");
    }

    #[test]
    fn test_check_initialized_valid() {
        let mut lamports = 100u64;
        let mut data = [0u8; 32]; // Initialized (len > 0)
        let mut payer_lamports = 1_000_000_000;
        let mut payer_data = [0u8; 0];
        let mut sys_lamports = 0u64;
        let mut sys_data = [0u8; 0];
        let (accounts, program_id, _, _) = setup_accounts(&mut lamports, &mut data, &mut payer_lamports, &mut payer_data, &mut sys_lamports, &mut sys_data);

        // Account 1 has data
        let bytecode = vec![
            0x35, 0x49, 0x56, 0x45, 0, 0, 0, 0, 0, 0,
            0x73, // CHECK_INITIALIZED
            0x01, // Account index 1
            0x00, // HALT
        ];

        let result = MitoVM::execute_direct(&bytecode, &[], &accounts, &program_id);
        result.expect("CHECK_INITIALIZED should succeed for initialized account");
    }

    #[test]
    fn test_check_uninitialized_valid() {
        let mut lamports = 100u64;
        let mut data = [0u8; 32];
        let mut payer_lamports = 1_000_000_000;
        let mut payer_data = [0u8; 0];
        let mut sys_lamports = 0u64;
        let mut sys_data = [0u8; 0];
        let (accounts, program_id, _, _) = setup_accounts(&mut lamports, &mut data, &mut payer_lamports, &mut payer_data, &mut sys_lamports, &mut sys_data);

        // Account 2 (system program) has empty data and owner is System Program ([0u8; 32])
        // create_test_accounts uses [0u8; 32] as system_program_key.
        // And sets Account 2 owner to it.
        // CHECK_UNINITIALIZED checks account.data.is_empty() AND owner == SystemProgramID (which is also [0;32] in constraints.rs)

        let bytecode = vec![
            0x35, 0x49, 0x56, 0x45, 0, 0, 0, 0, 0, 0,
            0x75, // CHECK_UNINITIALIZED
            0x02, // Account index 2
            0x00, // HALT
        ];

        let result = MitoVM::execute_direct(&bytecode, &[], &accounts, &program_id);
        result.expect("CHECK_UNINITIALIZED should succeed for uninitialized account");
    }
}

#[cfg(test)]
mod pda_constraint_tests {
    use super::*;
    use super::basic_constraint_tests::setup_accounts;

    #[test]
    fn test_check_pda_valid() {
        let mut lamports = 100u64;
        let mut data = [0u8; 32];
        let mut payer_lamports = 1_000_000_000;
        let mut payer_data = [0u8; 0];
        let mut sys_lamports = 0u64;
        let mut sys_data = [0u8; 0];
        let (accounts, program_id, pda_address, bump) = setup_accounts(&mut lamports, &mut data, &mut payer_lamports, &mut payer_data, &mut sys_lamports, &mut sys_data);

        // Account 1 is the PDA derived from seeds "test" and program_id
        let mut bytecode = vec![
            0x35, 0x49, 0x56, 0x45, 0, 0, 0, 0, 0, 0,
        ];

        // 1. Push seeds "test"
        // PUSH_STRING "test"
        bytecode.extend_from_slice(&[0x67, 0x04, b't', b'e', b's', b't']);

        // 2. Push bump (u8) - as a seed
        bytecode.push(0x18); // PUSH_U8
        bytecode.push(bump);

        // 3. Push seeds count (2) - "test" + bump
        bytecode.extend_from_slice(&[0x18, 0x02]);

        // 4. Push program_id
        bytecode.push(0x1E); // PUSH_PUBKEY
        bytecode.extend_from_slice(program_id.as_ref());

        // 5. Push expected PDA address (which is Account 1's key)
        bytecode.push(0x1E); // PUSH_PUBKEY
        bytecode.extend_from_slice(pda_address.as_ref());

        // 6. CHECK_PDA
        bytecode.push(0x74);

        bytecode.push(0x00); // HALT

        let result = MitoVM::execute_direct(&bytecode, &[], &accounts, &program_id);
        result.expect("CHECK_PDA should succeed for valid PDA");
    }
}

#[cfg(test)]
mod unimplemented_constraint_tests {
    use super::*;

    // These opcodes are not implemented yet and should return InvalidInstruction

    #[test]
    fn test_check_dedupe_table() {
        let bytecode = vec![
            0x35, 0x49, 0x56, 0x45, 0, 0, 0, 0, 0, 0,
            0x76, // CHECK_DEDUPE_TABLE
            0x00,
        ];
        let result = MitoVM::execute_direct(&bytecode, &[], &[], &FIVE_VM_PROGRAM_ID);
        match result {
            Err(VMError::InvalidInstruction) => {}, // Correct
            _ => panic!("Expected InvalidInstruction for CHECK_DEDUPE_TABLE"),
        }
    }
}
