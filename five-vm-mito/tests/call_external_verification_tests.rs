// VM Integration Tests - Import Verification for CALL_EXTERNAL
//
// These tests verify that the VM correctly validates Five bytecode accounts
// during CALL_EXTERNAL operations using import verification metadata.

use five_protocol::{opcodes::*, FEATURE_IMPORT_VERIFICATION, FIVE_MAGIC};
use five_vm_mito::{FIVE_VM_PROGRAM_ID, metadata::ImportMetadata};

// Note: We don't need to create AccountInfo structures for these tests
// We're testing the metadata parsing and verification logic directly

/// Build bytecode with import verification metadata
/// Returns (bytecode, metadata_offset)
fn build_bytecode_with_import_metadata(import_type: u8, import_data: &[u8], function_name: &str) -> Vec<u8> {
    let mut bytecode = Vec::new();

    // Header (10 bytes)
    bytecode.extend_from_slice(&FIVE_MAGIC); // 0..4
    let features = FEATURE_IMPORT_VERIFICATION; // Enable import verification
    bytecode.extend_from_slice(&features.to_le_bytes()); // 4..8
    bytecode.push(1); // public_function_count = 1 (byte 8)
    bytecode.push(1); // total_function_count = 1 (byte 9)

    // Main bytecode (simple HALT)
    bytecode.push(HALT);

    // Import metadata section (at end of bytecode)
    // Format: [import_count: u8][import_type: u8][...data...][name_len: u8][name: bytes]
    bytecode.push(1); // import_count = 1
    bytecode.push(import_type); // import_type (0 = address, 1 = PDA)
    bytecode.extend_from_slice(import_data); // Import-specific data
    bytecode.push(function_name.len() as u8); // name_len
    bytecode.extend_from_slice(function_name.as_bytes()); // name

    bytecode
}

#[test]
fn test_call_external_import_verification_with_valid_address() {
    // Test CALL_EXTERNAL succeeds when account matches import verification metadata

    // Create test address (32 bytes)
    let test_key: [u8; 32] = [1u8; 32];

    // Build main bytecode WITH import verification metadata for test_key
    let bytecode = build_bytecode_with_import_metadata(
        0, // type = address
        &test_key, // 32-byte address
        "test_func",
    );

    let metadata_offset = 11; // After header (10) + HALT (1)
    let import_metadata = ImportMetadata::new(&bytecode, metadata_offset)
        .expect("Should create import metadata");

    // Verify the account matches (this is what CALL_EXTERNAL will check)
    let program_id: [u8; 32] = [0u8; 32];
    let matches = import_metadata.verify_account(
        &test_key,
        &program_id,
        None, // No PDA derivation needed for address mode
    );

    assert!(
        matches,
        "Account should match import verification metadata"
    );

    println!("✅ CALL_EXTERNAL with valid address import test passed!");
    println!("  - Import metadata parsed successfully");
    println!("  - Account address matches import metadata");
    println!("  - CALL_EXTERNAL would succeed");
}

#[test]
fn test_call_external_import_verification_with_wrong_address() {
    // Test CALL_EXTERNAL fails when account doesn't match import metadata

    // Authorized address in metadata
    let authorized_key: [u8; 32] = [1u8; 32];

    // Different address provided at runtime (attacker's account)
    let attacker_key: [u8; 32] = [2u8; 32];

    // Build bytecode with import metadata for authorized_key
    let bytecode = build_bytecode_with_import_metadata(
        0, // type = address
        &authorized_key,
        "trusted_func",
    );

    let metadata_offset = 11;
    let import_metadata = ImportMetadata::new(&bytecode, metadata_offset)
        .expect("Should create import metadata");

    // Try to verify attacker_key (should fail)
    let program_id: [u8; 32] = [0u8; 32];
    let matches = import_metadata.verify_account(
        &attacker_key,
        &program_id,
        None,
    );

    assert!(
        !matches,
        "Attacker's account should NOT match import metadata"
    );

    // This simulates what would happen in CALL_EXTERNAL handler:
    // if !ctx.import_metadata.verify_account(...) {
    //     return Err(VMError::UnauthorizedBytecodeInvocation);
    // }

    println!("✅ CALL_EXTERNAL with wrong address test passed!");
    println!("  - Import metadata parsed successfully");
    println!("  - Attacker's account rejected (address mismatch)");
    println!("  - CALL_EXTERNAL would return UnauthorizedBytecodeInvocation");
}

#[test]
fn test_call_external_backward_compatibility_no_metadata() {
    // Test that old bytecode WITHOUT import verification flag still works

    // Create simple bytecode WITHOUT FEATURE_IMPORT_VERIFICATION flag
    let mut bytecode = Vec::new();
    bytecode.extend_from_slice(&FIVE_MAGIC); // magic
    let features = 0u32; // NO import verification flag
    bytecode.extend_from_slice(&features.to_le_bytes());
    bytecode.push(1); // public_function_count
    bytecode.push(1); // total_function_count
    bytecode.push(HALT); // simple instruction

    // Create metadata with offset beyond bytecode (empty metadata)
    let metadata_offset = bytecode.len();
    let import_metadata = ImportMetadata::new(&bytecode, metadata_offset)
        .expect("Should create empty import metadata");

    assert!(
        import_metadata.is_empty(),
        "Import metadata should be empty for old bytecode"
    );

    // Any account should be accepted (backward compatible)
    let any_key: [u8; 32] = [99u8; 32];
    let program_id: [u8; 32] = [0u8; 32];

    let matches = import_metadata.verify_account(
        &any_key,
        &program_id,
        None,
    );

    assert!(
        matches,
        "Old bytecode without import verification should accept any account"
    );

    println!("✅ Backward compatibility test passed!");
    println!("  - Old bytecode (no import verification flag) works");
    println!("  - Empty metadata accepts any account");
    println!("  - CALL_EXTERNAL maintains backward compatibility");
}

#[test]
fn test_call_external_verification_skipped_with_pda_no_callback() {
    // Test PDA import handling when no PDA derivation callback is provided

    // Build bytecode with PDA seed import
    let mut pda_data = Vec::new();
    pda_data.push(2); // seed_count = 2

    // Seed 1: "vault"
    pda_data.push(5); // seed_len
    pda_data.extend_from_slice(b"vault");

    // Seed 2: "user"
    pda_data.push(4); // seed_len
    pda_data.extend_from_slice(b"user");

    let bytecode = build_bytecode_with_import_metadata(
        1, // type = PDA seeds
        &pda_data,
        "pda_func",
    );

    let metadata_offset = 11;
    let import_metadata = ImportMetadata::new(&bytecode, metadata_offset)
        .expect("Should create import metadata");

    // Try to verify without PDA derivation callback
    let test_key: [u8; 32] = [1u8; 32];
    let program_id: [u8; 32] = [0u8; 32];

    let matches = import_metadata.verify_account(
        &test_key,
        &program_id,
        None, // No PDA derivation callback
    );

    // Without PDA callback, cannot verify PDA imports
    // This is expected behavior - PDA verification requires runtime derivation
    assert!(
        !matches,
        "PDA verification should fail gracefully without derivation callback"
    );

    println!("✅ PDA verification without callback test passed!");
    println!("  - PDA import metadata parsed successfully");
    println!("  - Verification skipped gracefully without PDA callback");
    println!("  - Returns false (cannot verify) instead of crashing");
}

#[test]
fn test_import_metadata_multiple_imports() {
    // Test metadata parsing with multiple imports (mixed address and PDA)

    let mut bytecode = Vec::new();

    // Header
    bytecode.extend_from_slice(&FIVE_MAGIC);
    let features = FEATURE_IMPORT_VERIFICATION;
    bytecode.extend_from_slice(&features.to_le_bytes());
    bytecode.push(1); // public_function_count
    bytecode.push(1); // total_function_count
    bytecode.push(HALT);

    // Import metadata with 2 imports
    bytecode.push(2); // import_count = 2

    // Import 1: Address type
    bytecode.push(0); // type = address
    let addr1 = [1u8; 32];
    bytecode.extend_from_slice(&addr1);
    bytecode.push(5); // name_len
    bytecode.extend_from_slice(b"func1");

    // Import 2: PDA type
    bytecode.push(1); // type = PDA
    bytecode.push(1); // seed_count = 1
    bytecode.push(4); // seed_len
    bytecode.extend_from_slice(b"test");
    bytecode.push(5); // name_len
    bytecode.extend_from_slice(b"func2");

    let metadata_offset = 11;
    let import_metadata = ImportMetadata::new(&bytecode, metadata_offset)
        .expect("Should create import metadata");

    assert!(!import_metadata.is_empty(), "Metadata should not be empty");

    // Verify first import (address) matches
    let program_id: [u8; 32] = [0u8; 32];
    let matches = import_metadata.verify_account(
        &addr1,
        &program_id,
        None,
    );

    assert!(
        matches,
        "First import (address) should match"
    );

    // Verify different address doesn't match
    let other_addr = [99u8; 32];
    let matches = import_metadata.verify_account(
        &other_addr,
        &program_id,
        None,
    );

    assert!(
        !matches,
        "Different address should not match"
    );

    println!("✅ Multiple imports test passed!");
    println!("  - Parsed 2 imports (address + PDA)");
    println!("  - Address verification works correctly");
    println!("  - Linear search finds matching import");
}

#[test]
fn test_import_metadata_bounds_checking() {
    // Test that metadata parsing handles malformed data gracefully

    // Create minimal bytecode
    let mut bytecode = Vec::new();
    bytecode.extend_from_slice(&FIVE_MAGIC);
    let features = FEATURE_IMPORT_VERIFICATION;
    bytecode.extend_from_slice(&features.to_le_bytes());
    bytecode.push(1); // public_function_count
    bytecode.push(1); // total_function_count
    bytecode.push(HALT);

    // Malformed metadata: import_count = 1, but no data follows
    bytecode.push(1); // import_count = 1
    // Missing: import_type, address/seeds, name

    let metadata_offset = 11;
    let import_metadata = ImportMetadata::new(&bytecode, metadata_offset)
        .expect("Should create import metadata");

    // Try to verify (should handle malformed data gracefully)
    let test_key: [u8; 32] = [1u8; 32];
    let program_id: [u8; 32] = [0u8; 32];

    let matches = import_metadata.verify_account(
        &test_key,
        &program_id,
        None,
    );

    // Malformed metadata should return false (not panic)
    assert!(
        !matches,
        "Malformed metadata should fail verification gracefully"
    );

    println!("✅ Bounds checking test passed!");
    println!("  - Malformed metadata handled gracefully");
    println!("  - No panic on truncated data");
    println!("  - Returns false for malformed input");
}
