//! Robust PDA Account Initialization Tests
//!
//! Comprehensive tests for `INIT_PDA_ACCOUNT` focusing on validation logic,
//! error conditions, and negative test cases.

use five_vm_mito::{AccountInfo, FIVE_VM_PROGRAM_ID, MitoVM, VMError, error::VMErrorCode};
use pinocchio::pubkey::Pubkey;
use solana_sdk::pubkey::Pubkey as SolanaPubkey;

fn derive_pda_real(seeds: &[&[u8]], program_id: &Pubkey) -> (Pubkey, u8) {
    // Convert pinocchio Pubkey to solana_sdk Pubkey for PDA derivation
    let solana_program_id = SolanaPubkey::new_from_array(program_id.as_ref().try_into().unwrap());
    let (pda_pubkey, bump) = SolanaPubkey::find_program_address(seeds, &solana_program_id);
    // Convert back to pinocchio Pubkey
    (Pubkey::from(pda_pubkey.to_bytes()), bump)
}

/// Helper to create a proper test environment with valid accounts
fn create_test_accounts<'a>(
    program_id: &Pubkey,
    account_key: &Pubkey,
    lamports: &'a mut u64,
    data: &'a mut [u8],
    payer_lamports: &'a mut u64,
    payer_data: &'a mut [u8],
    system_lamports: &'a mut u64,
    system_data: &'a mut [u8],
) -> [AccountInfo; 3] { // Now returns 3 accounts
    let payer_key = Pubkey::from([1u8; 32]);
    let system_program_key = Pubkey::from([0u8; 32]); // System Program ID (all zeros for test/mock usually, or standard ID)

    // Account 0: Payer (Signer, Writable)
    let payer = AccountInfo::new(
        &payer_key,
        true, // is_signer
        true, // is_writable
        payer_lamports,
        payer_data,
        program_id,
        false,
        0,
    );

    // Account 1: New Account (Signer, Writable) - to be initialized
    let new_account = AccountInfo::new(
        account_key,
        true, // is_signer
        true, // is_writable
        lamports,
        data,
        program_id,
        false,
        0,
    );

    // Account 2: System Program (Executable)
    let system_program = AccountInfo::new(
        &system_program_key,
        false, // is_signer
        false, // is_writable
        system_lamports,
        system_data,
        &system_program_key,
        true, // executable
        0,
    );

    [payer, new_account, system_program]
}

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
    let bytecode_utils_vle_0 = encode_vle(0);
    let bytecode_utils_vle_1m = encode_vle(1_000_000);
    let bytecode_utils_vle_100 = encode_vle(100);

    let mut bytecode = vec![
        0x35, 0x49, 0x56, 0x45, // Magic
        0x00, 0x00, 0x00, 0x00, // Features
        0x00, 0x00,             // Counts
    ];

    // Push bump
    bytecode.push(0x18); bytecode.push(bump);
    // Push seed 1: "vault"
    bytecode.extend_from_slice(&[0x67, 0x05, b'v', b'a', b'u', b'l', b't']);
    // Push seed 2: [1, 2, 3]
    bytecode.extend_from_slice(&[0x67, 0x03, 0x01, 0x02, 0x03]);
    // Push seeds count (2)
    bytecode.push(0x18); bytecode.push(0x02);
    // Push owner (0)
    bytecode.push(0x1B); bytecode.extend_from_slice(&bytecode_utils_vle_0);
    // Push lamports (1_000_000)
    bytecode.push(0x1B); bytecode.extend_from_slice(&bytecode_utils_vle_1m);
    // Push payer_idx (0)
    bytecode.push(0x18); bytecode.push(0x00);
    // Push space (100)
    bytecode.push(0x1B); bytecode.extend_from_slice(&bytecode_utils_vle_100);
    // Push account_idx (1)
    bytecode.push(0x18); bytecode.push(0x01);
    // Call INIT_PDA_ACCOUNT
    bytecode.push(0x85);
    bytecode.push(0x00);

    // 4. Execution
    let result = MitoVM::execute_direct(&bytecode, &[], accounts, &program_id);

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
    let bytecode_utils_vle_0 = encode_vle(0);
    let bytecode_utils_vle_1m = encode_vle(1_000_000);
    let bytecode_utils_vle_100 = encode_vle(100);

    let mut bytecode = vec![
        0x35, 0x49, 0x56, 0x45, // Magic
        0x00, 0x00, 0x00, 0x00, // Features
        0x00, 0x00,             // Counts
    ];

    // Push bump
    bytecode.push(0x18); bytecode.push(bump);
    // Push seed 1
    bytecode.extend_from_slice(&[0x67, 0x05, b'v', b'a', b'u', b'l', b't']);
    // Push seed 2
    bytecode.extend_from_slice(&[0x67, 0x03, 0x01, 0x02, 0x03]);
    // Push seeds count
    bytecode.push(0x18); bytecode.push(0x02);
    // Push owner
    bytecode.push(0x1B); bytecode.extend_from_slice(&bytecode_utils_vle_0);
    // Push lamports
    bytecode.push(0x1B); bytecode.extend_from_slice(&bytecode_utils_vle_1m);
    // Push payer_idx (0)
    bytecode.push(0x18); bytecode.push(0x00);
    // Push space
    bytecode.push(0x1B); bytecode.extend_from_slice(&bytecode_utils_vle_100);
    // Push account_idx
    bytecode.push(0x18); bytecode.push(0x01);
    
    bytecode.push(0x85); // INIT_PDA_ACCOUNT
    bytecode.push(0x00);

    // 4. Execution
    let result = MitoVM::execute_direct(&bytecode, &[], accounts, &program_id);

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

    let bytecode_utils_vle_0 = encode_vle(0);
    let bytecode_utils_vle_1m = encode_vle(1_000_000);
    let bytecode_utils_vle_100 = encode_vle(100);

    let mut bytecode = vec![
        0x35, 0x49, 0x56, 0x45, // Magic
        0x00, 0x00, 0x00, 0x00, // Features
        0x00, 0x00,             // Counts
    ];

    // Push bump
    bytecode.push(0x18); bytecode.push(wrong_bump);
    // Push seed 1
    bytecode.extend_from_slice(&[0x67, 0x05, b'v', b'a', b'u', b'l', b't']);
    // Push seeds count (1)
    bytecode.push(0x18); bytecode.push(0x01);
    // Push owner
    bytecode.push(0x1B); bytecode.extend_from_slice(&bytecode_utils_vle_0);
    // Push lamports
    bytecode.push(0x1B); bytecode.extend_from_slice(&bytecode_utils_vle_1m);
    // Push payer_idx (0)
    bytecode.push(0x18); bytecode.push(0x00);
    // Push space
    bytecode.push(0x1B); bytecode.extend_from_slice(&bytecode_utils_vle_100);
    // Push account_idx
    bytecode.push(0x18); bytecode.push(0x01);

    bytecode.push(0x85); // INIT_PDA_ACCOUNT
    bytecode.push(0x00);

    let result = MitoVM::execute_direct(&bytecode, &[], accounts, &program_id);

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

fn encode_vle(mut value: u64) -> Vec<u8> {
    let mut bytes = Vec::new();
    loop {
        let mut byte = (value & 0x7F) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        bytes.push(byte);
        if value == 0 {
            break;
        }
    }
    bytes
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
    bytecode.extend_from_slice(&[0x67, 0x05, b'v', b'a', b'u', b'l', b't']);

    // Push seeds count
    bytecode.push(0x18);
    bytecode.push(0x01);

    // Push owner (0) - VLE(0) is 0x00
    bytecode.push(0x1B); // PUSH_U64
    bytecode.push(0x00);

    // Push lamports (1_000_000)
    bytecode.push(0x1B); // PUSH_U64
    bytecode.extend_from_slice(&encode_vle(1_000_000));

    // Push payer_idx (0)
    bytecode.push(0x18);
    bytecode.push(0x00);

    // Push Excessive Space
    bytecode.push(0x1B); // PUSH_U64
    bytecode.extend_from_slice(&encode_vle(excessive_space)); 
    
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

    let result = MitoVM::execute_direct(&bytecode, &[], accounts, &program_id);

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
