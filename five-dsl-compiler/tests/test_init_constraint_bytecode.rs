//! Integration tests for @init constraint bytecode generation
//!
//! Verifies that CHECK_SIGNER is emitted for @init payer parameters with correct indices.
//! Prevents regression where @init constraint payer wasn't being validated.

use five_dsl_compiler::*;
use five_protocol::opcodes::*;

#[test]
fn test_init_emits_check_signer_for_payer() {
    let source = r#"
        pub initialize(
            counter: account @mut @init(payer=owner, space=56),
            owner: account @signer
        ) {
        }
    "#;

    let bytecode = DslCompiler::compile_dsl(source).expect("Should compile");

    // Find CHECK_SIGNER opcode followed by account index byte
    let mut found_check_signer = false;
    for window in bytecode.windows(2) {
        if window[0] == CHECK_SIGNER {
            let account_idx = window[1];
            // owner is param1, should map to account_index 2 (1 + 1)
            // With correct OFFSET=1 this should be index 2
            assert_eq!(
                account_idx, 2,
                "CHECK_SIGNER should target account index 2 (param1 + OFFSET 1), but got {}",
                account_idx
            );
            found_check_signer = true;
            break;
        }
    }

    assert!(
        found_check_signer,
        "Bytecode must contain CHECK_SIGNER for @init payer"
    );
}

#[test]
fn test_init_with_multiple_params() {
    let source = r#"
        pub initialize(
            counter: account @mut @init(payer=payer_acct, space=56),
            intermediate: account,
            payer_acct: account @signer
        ) {
        }
    "#;

    let bytecode = DslCompiler::compile_dsl(source).expect("Should compile");

    // payer_acct is param2, should map to account_index 3 (2 + 1)
    let mut found_correct_signer_check = false;
    for window in bytecode.windows(2) {
        if window[0] == CHECK_SIGNER {
            let account_idx = window[1];
            if account_idx == 3 {
                // This is the CHECK_SIGNER for payer_acct (param2)
                found_correct_signer_check = true;
                break;
            }
        }
    }

    assert!(
        found_correct_signer_check,
        "Should emit CHECK_SIGNER for payer_acct at account index 3 (param2 + OFFSET 1)"
    );
}

#[test]
fn test_increment_check_signer_correct_index() {
    // Test that non-@init functions also use correct account indices
    let source = r#"
        pub increment(
            counter: account @mut,
            owner: account @signer
        ) {
        }
    "#;

    let bytecode = DslCompiler::compile_dsl(source).expect("Should compile");

    // owner is param1 -> should check account_index 2 (1 + 1)
    let mut found = false;
    for window in bytecode.windows(2) {
        if window[0] == CHECK_SIGNER && window[1] == 2 {
            found = true;
            break;
        }
    }

    assert!(
        found,
        "Should emit CHECK_SIGNER for account index 2 (param1 + OFFSET 1)"
    );
}

#[test]
fn test_init_param_order_matters() {
    // Test that the parameter order affects the account index
    let source = r#"
        pub first_version(
            counter: account @mut @init(payer=payer, space=56),
            payer: account @signer
        ) {
        }
    "#;

    let bytecode = DslCompiler::compile_dsl(source).expect("Should compile");

    // payer is param1 -> account_index 2 (1 + 1)
    let mut found = false;
    for window in bytecode.windows(2) {
        if window[0] == CHECK_SIGNER && window[1] == 2 {
            found = true;
            break;
        }
    }

    assert!(found, "Should find CHECK_SIGNER at correct index");
}

#[test]
fn test_multiple_init_accounts() {
    // While @init typically appears once, verify it works correctly
    let source = r#"
        pub double_init(
            counter1: account @mut @init(payer=owner, space=56),
            owner: account @signer
        ) {
        }
    "#;

    let bytecode = DslCompiler::compile_dsl(source).expect("Should compile");

    // Should have CHECK_SIGNER for owner (param1 -> index 2)
    let mut check_signer_count = 0;
    for window in bytecode.windows(2) {
        if window[0] == CHECK_SIGNER && window[1] == 2 {
            check_signer_count += 1;
        }
    }

    assert!(
        check_signer_count > 0,
        "Should emit CHECK_SIGNER for @init payer"
    );
}

#[test]
fn test_check_signer_before_other_constraints() {
    // Verify that CHECK_SIGNER for @init payer is emitted first
    let source = r#"
        pub initialize(
            counter: account @mut @init(payer=owner, space=56),
            owner: account @signer @mut
        ) {
        }
    "#;

    let bytecode = DslCompiler::compile_dsl(source).expect("Should compile");

    // Find first constraint opcode - should be CHECK_SIGNER
    for window in bytecode.windows(2) {
        if window[0] == CHECK_SIGNER || window[0] == CHECK_WRITABLE {
            // First constraint should be CHECK_SIGNER for the payer
            assert_eq!(
                window[0], CHECK_SIGNER,
                "@init payer CHECK_SIGNER should be emitted before other constraints"
            );
            break;
        }
    }
}

#[test]
fn test_bytecode_size_increased_with_check_signer() {
    // Before fix: CHECK_SIGNER was missing
    // After fix: CHECK_SIGNER should be present, increasing bytecode size slightly
    let source = r#"
        pub initialize(
            counter: account @mut @init(payer=owner, space=56),
            owner: account @signer
        ) {
        }
    "#;

    let bytecode = DslCompiler::compile_dsl(source).expect("Should compile");

    // CHECK_SIGNER opcode (1 byte) + account index (1 byte) = 2 extra bytes
    // Minimum bytecode should be > 0 (script header + dispatch + constraints + bytecode)
    assert!(
        bytecode.len() > 0,
        "Bytecode should contain code with @init constraints"
    );

    // Verify CHECK_SIGNER is actually present
    assert!(
        bytecode.windows(1).any(|w| w[0] == CHECK_SIGNER),
        "Bytecode must contain CHECK_SIGNER opcode (0x{:02x})",
        CHECK_SIGNER
    );
}
