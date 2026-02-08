//! Robust PDA Account Initialization Tests
//!
//! Comprehensive tests for `INIT_PDA_ACCOUNT` focusing on validation logic,
//! error conditions, and negative test cases.

#[path = "support/accounts.rs"]
mod support_accounts;

use five_vm_mito::{MitoVM, VMError, stack::StackStorage, AccountInfo, Value};

fn execute_test(bytecode: &[u8], input: &[u8], accounts: &[AccountInfo], program_id: &pinocchio::pubkey::Pubkey) -> five_vm_mito::Result<Option<Value>> {
    let mut storage = StackStorage::new();
    MitoVM::execute_direct(bytecode, input, accounts, program_id, &mut storage)
}
use pinocchio::pubkey::Pubkey;
use support_accounts::{create_test_accounts, derive_pda_real};

#[test]
fn test_init_pda_account_success() {
    // 1. Setup: Define seeds and derive valid PDA
    let program_id = Pubkey::from([0xAA; 32]);
    let seeds: &[&[u8]] = &[b"vault", &[1, 2, 3]];

    // Derive valid PDA address
    let (pda_address, bump) = derive_pda_real(seeds, &program_id);

    // 2. Setup Accounts: Use the derived PDA as the key for account #1
    let mut lamports = 0u64;
    let mut data = [0u8; 1024]; // Increase buffer to allow resize
    let mut payer_lamports = 1_000_000_000;
    let mut payer_data = [0u8; 0];
    let mut sys_lamports = 0u64;
    let mut sys_data = [0u8; 0];
    let accounts_storage = create_test_accounts(&program_id, &pda_address, &mut lamports, &mut data, &mut payer_lamports, &mut payer_data, &mut sys_lamports, &mut sys_data);
    let accounts = &accounts_storage; // Slice

    // 3. Bytecode: Simulate INIT_PDA_ACCOUNT with correct parameters
    let mut bytecode = vec![
        0x35, 0x49, 0x56, 0x45, // Magic
        0x00, 0x00, 0x00, 0x00, // Features
        0x00, 0x00,             // Counts
    ];

    // Push bump
    bytecode.push(0x18); bytecode.push(bump);
    // Push seed 1: "vault" (len 5 u32)
    bytecode.extend_from_slice(&[0x67, 0x05, 0x00, 0x00, 0x00, b'v', b'a', b'u', b'l', b't']);
    // Push seed 2: [1, 2, 3] (len 3 u32)
    bytecode.extend_from_slice(&[0x67, 0x03, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03]);
    // Push seeds count (2)
    bytecode.push(0x18); bytecode.push(0x02);
    // Push owner (0) - U64 8 bytes
    bytecode.push(0x1B); bytecode.extend_from_slice(&0u64.to_le_bytes());
    // Push lamports (1_000_000)
    bytecode.push(0x1B); bytecode.extend_from_slice(&1_000_000u64.to_le_bytes());
    // Push payer_idx (0)
    bytecode.push(0x18); bytecode.push(0x00);
    // Push space (100)
    bytecode.push(0x1B); bytecode.extend_from_slice(&100u64.to_le_bytes());
    // Push account_idx (1)
    bytecode.push(0x18); bytecode.push(0x01);
    // Call INIT_PDA_ACCOUNT
    bytecode.push(0x85);
    bytecode.push(0x00);

    // 4. Execution
    let result = execute_test(&bytecode, &[], accounts, &program_id);

    match result {
        Ok(_) => {
            println!("✅ INIT_PDA_ACCOUNT success test passed");
        },
        Err(e) => {
            panic!("INIT_PDA_ACCOUNT failed unexpected: {:?}", e);
        }
    }
}

#[test]
fn test_init_pda_account_failure_address_mismatch() {
    // 1. Setup: Define seeds and derive valid PDA
    let program_id = Pubkey::from([0xAA; 32]);
    let seeds: &[&[u8]] = &[b"vault", &[1, 2, 3]];
    let (_real_pda_address, bump) = derive_pda_real(seeds, &program_id);

    // 2. Setup Accounts: Use a RANDOM/WRONG key for account #1
    let wrong_key = Pubkey::from([0xBB; 32]); // Different from PDA
    let mut lamports = 0u64;
    let mut data = [0u8; 1024];
    let mut payer_lamports = 1_000_000_000;
    let mut payer_data = [0u8; 0];
    let mut sys_lamports = 0u64;
    let mut sys_data = [0u8; 0];
    let accounts_storage = create_test_accounts(&program_id, &wrong_key, &mut lamports, &mut data, &mut payer_lamports, &mut payer_data, &mut sys_lamports, &mut sys_data);
    let accounts = &accounts_storage;

    // 3. Bytecode: Same as success case, but account #1 key doesn't match
    let mut bytecode = vec![
        0x35, 0x49, 0x56, 0x45, // Magic
        0x00, 0x00, 0x00, 0x00, // Features
        0x00, 0x00,             // Counts
    ];

    // Push bump
    bytecode.push(0x18); bytecode.push(bump);
    // Push seed 1
    bytecode.extend_from_slice(&[0x67, 0x05, 0x00, 0x00, 0x00, b'v', b'a', b'u', b'l', b't']);
    // Push seed 2
    bytecode.extend_from_slice(&[0x67, 0x03, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03]);
    // Push seeds count
    bytecode.push(0x18); bytecode.push(0x02);
    // Push owner
    bytecode.push(0x1B); bytecode.extend_from_slice(&0u64.to_le_bytes());
    // Push lamports
    bytecode.push(0x1B); bytecode.extend_from_slice(&1_000_000u64.to_le_bytes());
    // Push payer_idx (0)
    bytecode.push(0x18); bytecode.push(0x00);
    // Push space
    bytecode.push(0x1B); bytecode.extend_from_slice(&100u64.to_le_bytes());
    // Push account_idx
    bytecode.push(0x18); bytecode.push(0x01);
    
    bytecode.push(0x85); // INIT_PDA_ACCOUNT
    bytecode.push(0x00);

    // 4. Execution
    let result = execute_test(&bytecode, &[], accounts, &program_id);

    // 5. Verification: Should fail with AccountError (address mismatch)
    match result {
        Err(VMError::AccountError) => {
            println!("✅ INIT_PDA_ACCOUNT correctly rejected mismatched address");
        },
        Err(e) => {
            panic!("Expected AccountError, got {:?}", e);
        },
        Ok(_) => {
            panic!("INIT_PDA_ACCOUNT should have failed due to address mismatch!");
        }
    }
}

#[test]
fn test_init_pda_account_failure_invalid_bump() {
    // 1. Setup
    let program_id = Pubkey::from([0xAA; 32]);
    let seeds: &[&[u8]] = &[b"vault"];
    let (pda_address, valid_bump) = derive_pda_real(seeds, &program_id);

    // Setup account with CORRECT address
    let mut lamports = 0u64;
    let mut data = [0u8; 1024];
    let mut payer_lamports = 1_000_000_000;
    let mut payer_data = [0u8; 0];
    let mut sys_lamports = 0u64;
    let mut sys_data = [0u8; 0];
    let accounts_storage = create_test_accounts(&program_id, &pda_address, &mut lamports, &mut data, &mut payer_lamports, &mut payer_data, &mut sys_lamports, &mut sys_data);
    let accounts = &accounts_storage;

    // Bytecode with WRONG bump
    let wrong_bump = valid_bump.wrapping_add(1);

    let mut bytecode = vec![
        0x35, 0x49, 0x56, 0x45, // Magic
        0x00, 0x00, 0x00, 0x00, // Features
        0x00, 0x00,             // Counts
    ];

    // Push bump
    bytecode.push(0x18); bytecode.push(wrong_bump);
    // Push seed 1
    bytecode.extend_from_slice(&[0x67, 0x05, 0x00, 0x00, 0x00, b'v', b'a', b'u', b'l', b't']);
    // Push seeds count (1)
    bytecode.push(0x18); bytecode.push(0x01);
    // Push owner
    bytecode.push(0x1B); bytecode.extend_from_slice(&0u64.to_le_bytes());
    // Push lamports
    bytecode.push(0x1B); bytecode.extend_from_slice(&1_000_000u64.to_le_bytes());
    // Push payer_idx (0)
    bytecode.push(0x18); bytecode.push(0x00);
    // Push space
    bytecode.push(0x1B); bytecode.extend_from_slice(&100u64.to_le_bytes());
    // Push account_idx
    bytecode.push(0x18); bytecode.push(0x01);

    bytecode.push(0x85); // INIT_PDA_ACCOUNT
    bytecode.push(0x00);

    let result = execute_test(&bytecode, &[], accounts, &program_id);

    match result {
        Err(VMError::AccountError) => {
            println!("✅ INIT_PDA_ACCOUNT correctly rejected wrong bump (derived address mismatch)");
        },
        // It might also fail with InvokeError if create_program_address fails for invalid bump/seeds combo off-curve
        Err(VMError::InvokeError { .. }) => {
             println!("✅ INIT_PDA_ACCOUNT correctly rejected invalid seeds/bump combination");
        },
        Err(e) => {
            // Note: VMError::from(VMErrorCode::AccountError) maps to VMError::AccountError
            panic!("Expected AccountError or InvokeError, got {:?}", e);
        },
        Ok(_) => {
            panic!("INIT_PDA_ACCOUNT should have failed due to wrong bump!");
        }
    }
}


#[test]
fn test_init_pda_account_failure_space_limit() {
    // 1. Setup
    let program_id = Pubkey::from([0xAA; 32]);
    let seeds: &[&[u8]] = &[b"vault"];
    let (pda_address, bump) = derive_pda_real(seeds, &program_id);

    let mut lamports = 0u64;
    let mut data = [0u8; 0];

    // Bytecode with EXCESSIVE space (11MB)
    let excessive_space = 11 * 1024 * 1024u64;

    let mut bytecode = vec![
        0x35, 0x49, 0x56, 0x45, // Magic
        0x00, 0x00, 0x00, 0x00, // Features
        0x00, 0x00,             // Counts
    ];

    // Push bump
    bytecode.push(0x18);
    bytecode.push(bump);

    // Push seed 1 ("vault")
    bytecode.extend_from_slice(&[0x67, 0x05, 0x00, 0x00, 0x00, b'v', b'a', b'u', b'l', b't']);

    // Push seeds count
    bytecode.push(0x18);
    bytecode.push(0x01);

    // Push owner (0)
    bytecode.push(0x1B); // PUSH_U64
    bytecode.extend_from_slice(&0u64.to_le_bytes());

    // Push lamports (1_000_000)
    bytecode.push(0x1B); // PUSH_U64
    bytecode.extend_from_slice(&1_000_000u64.to_le_bytes());

    // Push payer_idx (0)
    bytecode.push(0x18);
    bytecode.push(0x00);

    // Push Excessive Space
    bytecode.push(0x1B); // PUSH_U64
    bytecode.extend_from_slice(&excessive_space.to_le_bytes());
    
    // Push account_idx
    bytecode.push(0x18);
    bytecode.push(0x01);

    bytecode.push(0x85); // INIT_PDA_ACCOUNT
    bytecode.push(0x00);

    let mut payer_lamports = 1_000_000_000;
    let mut payer_data = [0u8; 0];
    let mut sys_lamports = 0u64;
    let mut sys_data = [0u8; 0];
    let accounts_storage = create_test_accounts(&program_id, &pda_address, &mut lamports, &mut data, &mut payer_lamports, &mut payer_data, &mut sys_lamports, &mut sys_data);
    let accounts = &accounts_storage;

    let result = execute_test(&bytecode, &[], accounts, &program_id);

    match result {
        Err(VMError::InvalidParameter) => {
            println!("✅ INIT_PDA_ACCOUNT correctly rejected excessive space");
        },
        Err(e) => {
            panic!("Expected InvalidParameter, got {:?}", e);
        },
        Ok(_) => {
            panic!("INIT_PDA_ACCOUNT should have failed due to excessive space!");
        }
    }
}
