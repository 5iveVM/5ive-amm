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
) -> [AccountInfo; 2] {
    let payer_key = Pubkey::from([1u8; 32]);

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
        true, // is_signer (required for init)
        true, // is_writable
        lamports,
        data,
        program_id, // Initially owned by program? Or system program?
        // Actually, uninitialized accounts usually owned by System Program
        // checking `handle_init_account` impl: checks account_idx range only.
        false,
        0,
    );

    [payer, new_account]
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
    let mut data = [0u8; 0];
    let mut payer_lamports = 1_000_000_000;
    let mut payer_data = [0u8; 0];
    let accounts_storage = create_test_accounts(&program_id, &pda_address, &mut lamports, &mut data, &mut payer_lamports, &mut payer_data);
    let accounts = &accounts_storage; // Slice

    // 3. Bytecode: Simulate INIT_PDA_ACCOUNT with correct parameters
    // Stack consumption order (Top to Bottom):
    // account_idx, space, lamports, owner, seeds_count, seedN...seed1, bump
    // Therefore Push order (Bottom to Top):
    // bump, seed1...seedN, seeds_count, owner, lamports, space, account_idx

    let bytecode = vec![
        0x35, 0x49, 0x56, 0x45, // Magic
        0x00, 0x00, 0x00, 0x00, // Features
        0x00, 0x00,             // Counts

        // Push bump
        0x18, bump, // PUSH_U8(bump)

        // Push seed 1: "vault"
        0x67, 0x05, // PUSH_STRING len=5
        b'v', b'a', b'u', b'l', b't',

        // Push seed 2: [1, 2, 3]
        0x67, 0x03,
        0x01, 0x02, 0x03,

        // Push seeds count (2)
        0x18, 0x02, // PUSH_U8(2)

        // Push owner (0 = Current Program)
        0x1B, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // PUSH_U64(0)

        // Push lamports (1_000_000)
        0x1B, 0x40, 0x42, 0x0F, 0x00, 0x00, 0x00, 0x00, 0x00, // PUSH_U64(1_000_000)

        // Push space (100)
        0x1B, 0x64, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // PUSH_U64(100)

        // Push account_idx (1)
        0x18, 0x01, // PUSH_U8(1)

        // Call INIT_PDA_ACCOUNT
        0x85, // INIT_PDA_ACCOUNT

        0x00 // HALT
    ];

    // 4. Execution
    // Note: The context.rs implementation of `create_pda_account` mocks the CPI via `invoke_signed`.
    // In test environment (host), `invoke_signed` does nothing/succeeds.
    // The validation logic inside `handle_init_pda_account` verifies `account.key()` matches derived PDA.
    // Since we set `account_key` to the derived PDA, this should PASS.

    let result = MitoVM::execute_direct(&bytecode, &[], accounts, &FIVE_VM_PROGRAM_ID);

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
    let mut data = [0u8; 0];
    let mut payer_lamports = 1_000_000_000;
    let mut payer_data = [0u8; 0];
    let accounts_storage = create_test_accounts(&program_id, &wrong_key, &mut lamports, &mut data, &mut payer_lamports, &mut payer_data);
    let accounts = &accounts_storage;

    // 3. Bytecode: Same as success case, but account #1 key doesn't match
    let bytecode = vec![
        0x35, 0x49, 0x56, 0x45, // Magic
        0x00, 0x00, 0x00, 0x00, // Features
        0x00, 0x00,             // Counts

        0x18, bump, // Valid bump for seeds
        0x67, 0x05, b'v', b'a', b'u', b'l', b't', // Seed 1
        0x67, 0x03, 0x01, 0x02, 0x03,             // Seed 2
        0x18, 0x02, // Seeds count
        0x1B, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Owner
        0x1B, 0x40, 0x42, 0x0F, 0x00, 0x00, 0x00, 0x00, 0x00, // Lamports
        0x1B, 0x64, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Space
        0x18, 0x01, // PUSH_U8(1) - account_idx

        0x85, // INIT_PDA_ACCOUNT
        0x00
    ];

    // 4. Execution
    let result = MitoVM::execute_direct(&bytecode, &[], accounts, &FIVE_VM_PROGRAM_ID);

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
    let mut data = [0u8; 0];
    let mut payer_lamports = 1_000_000_000;
    let mut payer_data = [0u8; 0];
    let accounts_storage = create_test_accounts(&program_id, &pda_address, &mut lamports, &mut data, &mut payer_lamports, &mut payer_data);
    let accounts = &accounts_storage;

    // Bytecode with WRONG bump
    let wrong_bump = valid_bump.wrapping_add(1);

    let bytecode = vec![
        0x35, 0x49, 0x56, 0x45, // Magic
        0x00, 0x00, 0x00, 0x00, // Features
        0x00, 0x00,             // Counts

        0x18, wrong_bump, // WRONG bump
        0x67, 0x05, b'v', b'a', b'u', b'l', b't', // Seed 1
        0x18, 0x01, // Seeds count (1)
        0x1B, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Owner
        0x1B, 0x40, 0x42, 0x0F, 0x00, 0x00, 0x00, 0x00, 0x00, // Lamports
        0x1B, 0x64, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Space
        0x18, 0x01, // account_idx

        0x85, // INIT_PDA_ACCOUNT
        0x00
    ];

    let result = MitoVM::execute_direct(&bytecode, &[], accounts, &FIVE_VM_PROGRAM_ID);

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

    // Manual PUSH_U64 serialization for 11MB
    let space_bytes = excessive_space.to_le_bytes();

    let mut bytecode = vec![
        0x35, 0x49, 0x56, 0x45, // Magic
        0x00, 0x00, 0x00, 0x00, // Features
        0x00, 0x00,             // Counts
    ];

    // Push bump
    bytecode.push(0x18);
    bytecode.push(bump);

    // Push seed 1
    bytecode.extend_from_slice(&[0x67, 0x05, b'v', b'a', b'u', b'l', b't']);

    // Push seeds count
    bytecode.push(0x18);
    bytecode.push(0x01);

    // Push owner
    bytecode.extend_from_slice(&[0x1B, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

    // Push lamports
    bytecode.extend_from_slice(&[0x1B, 0x40, 0x42, 0x0F, 0x00, 0x00, 0x00, 0x00, 0x00]);

    // Push Excessive Space
    bytecode.push(0x1B); // PUSH_U64
    bytecode.extend_from_slice(&space_bytes); // Space

    // Push account_idx
    bytecode.push(0x18);
    bytecode.push(0x01);

    bytecode.push(0x85); // INIT_PDA_ACCOUNT
    bytecode.push(0x00);

    let mut payer_lamports = 1_000_000_000;
    let mut payer_data = [0u8; 0];
    let accounts_storage = create_test_accounts(&program_id, &pda_address, &mut lamports, &mut data, &mut payer_lamports, &mut payer_data);
    let accounts = &accounts_storage;

    let result = MitoVM::execute_direct(&bytecode, &[], accounts, &FIVE_VM_PROGRAM_ID);

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
