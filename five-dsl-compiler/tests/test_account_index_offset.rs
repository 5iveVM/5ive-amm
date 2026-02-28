//! Regression tests for ACCOUNT_INDEX_OFFSET
//!
//! This test module prevents the bug from commit 5b9c973 from ever recurring.
//! That commit incorrectly changed ACCOUNT_INDEX_OFFSET from 1 to 2, causing
//! constraint checks to target wrong account indices.
//!
//! Root cause: Misunderstanding of account layout
//! - Solana instruction: [Script, VM State, param0, param1, ...]
//! - five-solana passes: &accounts[1..] to VM
//! - VM sees: [VM State, param0, param1, ...] with indices [0, 1, 2, ...]
//! - Therefore: param_index + 1 = account_index (not +2)

use five_dsl_compiler::bytecode_generator::account_utils::account_index_from_param_index;
use five_dsl_compiler::bytecode_generator::ACCOUNT_INDEX_OFFSET;

#[test]
fn test_account_index_offset_is_one() {
    // CRITICAL: VM receives &accounts[1..], so offset must be 1
    assert_eq!(
        ACCOUNT_INDEX_OFFSET, 1,
        "ACCOUNT_INDEX_OFFSET must be 1 because VM receives &accounts[1..] \
         which means Index 0=VM State, 1=param0, 2=param1, etc."
    );
}

#[test]
fn test_param_to_account_mapping() {
    // param0 (first parameter) -> account index 1 (0 + 1)
    assert_eq!(
        account_index_from_param_index(0),
        1,
        "param0 should map to account index 1"
    );

    // param1 (second parameter) -> account index 2 (1 + 1)
    assert_eq!(
        account_index_from_param_index(1),
        2,
        "param1 should map to account index 2"
    );

    // param2 (third parameter) -> account index 3 (2 + 1)
    assert_eq!(
        account_index_from_param_index(2),
        3,
        "param2 should map to account index 3"
    );
}

#[test]
fn test_offset_never_two() {
    // This test will fail if someone changes ACCOUNT_INDEX_OFFSET to 2
    // Forcing them to understand why this test exists (see commit 5b9c973)
    assert_ne!(
        ACCOUNT_INDEX_OFFSET, 2,
        "OFFSET must NOT be 2 - this breaks VM account access. \
         See commit 5b9c973 for historical context. \
         VM receives &accounts[1..], so offset must be 1, not 2."
    );
}

#[test]
fn test_boundary_cases() {
    // Test that the offset is applied correctly at boundaries
    assert_eq!(account_index_from_param_index(u8::MIN), 1);

    // Test with larger values
    assert_eq!(account_index_from_param_index(10), 11);
    assert_eq!(account_index_from_param_index(100), 101);
}
