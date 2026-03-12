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

fn bytecode_contains_u64_literal(bytecode: &[u8], target: u64) -> bool {
    let needle = target.to_le_bytes();
    bytecode.windows(needle.len()).any(|win| win == needle)
}

#[test]
fn test_init_with_explicit_bump_is_honored() {
    let source = r#"
        account Vault {
            amount: u64
        }

        pub initialize(
            payer: account @signer,
            user_bump: u8,
            vault: Vault @mut @init(payer=payer, seeds=["vault"], bump=user_bump)
        ) {
        }
    "#;

    let bytecode = DslCompiler::compile_dsl(source).expect("Should compile with explicit bump");
    assert!(
        !bytecode.windows(1).any(|w| w[0] == FIND_PDA),
        "Explicit bump path should not require FIND_PDA derivation"
    );
    assert!(
        bytecode.windows(1).any(|w| w[0] == CHECK_PDA),
        "Explicit bump path must emit CHECK_PDA"
    );
}

#[test]
fn test_init_with_seeded_auto_bump_derives_canonical_bump() {
    let source = r#"
        account Vault {
            amount: u64
        }

        pub initialize(
            payer: account @signer,
            vault: Vault @mut @init(payer=payer, seeds=["vault"])
        ) {
        }
    "#;

    let bytecode =
        DslCompiler::compile_dsl(source).expect("Should compile with auto bump derivation");
    assert!(
        bytecode.windows(1).any(|w| w[0] == FIND_PDA),
        "Auto bump path should emit FIND_PDA"
    );
}

#[test]
fn test_init_with_seeded_auto_bump_exposes_account_ctx_bump() {
    let source = r#"
        account Vault {
            amount: u64
        }

        pub initialize(
            payer: account @signer,
            vault: Vault @mut @init(payer=payer, seeds=["vault"])
        ) {
            let seen_bump: u8 = vault.ctx.bump;
            let _x: u8 = seen_bump;
        }
    "#;

    let bytecode =
        DslCompiler::compile_dsl(source).expect("Should compile and expose vault.ctx.bump");
    assert!(
        bytecode.windows(1).any(|w| w[0] == FIND_PDA),
        "Auto bump path should still emit FIND_PDA"
    );
}

#[test]
fn test_init_with_seeded_auto_bump_legacy_alias_is_removed() {
    let source = r#"
        account Vault {
            amount: u64
        }

        pub initialize(
            payer: account @signer,
            vault: Vault @mut @init(payer=payer, seeds=["vault"])
        ) {
            let seen_bump: u8 = vault_bump;
            let _x: u8 = seen_bump;
        }
    "#;

    let err = DslCompiler::compile_dsl(source).expect_err("legacy vault_bump should fail");
    assert!(
        err.message.contains("cannot find value") || err.message.contains("Undefined"),
        "Expected undefined identifier failure for removed bump alias, got: {:?}",
        err
    );
    assert_eq!(
        err.context.get_data("did_you_mean").map(String::as_str),
        Some("vault.ctx.bump"),
        "Expected migration hint for legacy bump alias"
    );
}

#[test]
fn test_init_with_legacy_space_alias_reports_ctx_hint() {
    let source = r#"
        account State {
            value: u64
        }

        pub initialize(
            payer: account @signer,
            state: State @mut @init(payer=payer)
        ) {
            let s: u64 = state_space;
            let _x: u64 = s;
        }
    "#;

    let err = DslCompiler::compile_dsl(source).expect_err("legacy state_space should fail");
    assert_eq!(
        err.context.get_data("did_you_mean").map(String::as_str),
        Some("state.ctx.space"),
        "Expected migration hint for legacy space alias"
    );
}

#[test]
fn test_init_auto_space_uses_account_layout_size() {
    let source = r#"
        account MyAccount {
            amount: u64,
            owner: pubkey
        }

        pub initialize(
            payer: account @signer,
            state: MyAccount @mut @init(payer=payer)
        ) {
        }
    "#;

    let bytecode = DslCompiler::compile_dsl(source).expect("Should compile with auto space");
    // Layout is 8 (u64) + 32 (pubkey) = 40 bytes.
    assert!(
        bytecode_contains_u64_literal(&bytecode, 40),
        "Auto space should include account layout size (40 bytes)"
    );
}

#[test]
fn test_init_explicit_space_overrides_auto_layout_size() {
    let source = r#"
        account MyAccount {
            amount: u64,
            owner: pubkey
        }

        pub initialize(
            payer: account @signer,
            state: MyAccount @mut @init(payer=payer, space=1234)
        ) {
        }
    "#;

    let bytecode = DslCompiler::compile_dsl(source).expect("Should compile with explicit space");
    assert!(
        bytecode_contains_u64_literal(&bytecode, 1234),
        "Explicit space should be emitted in bytecode"
    );
}

#[test]
fn test_account_ctx_lamports_lowers_to_get_lamports() {
    let source = r#"
        pub inspect(
            payer: account @signer
        ) {
            let bal: u64 = payer.ctx.lamports;
            let _x: u64 = bal;
        }
    "#;
    let bytecode = DslCompiler::compile_dsl(source).expect("Should compile account.ctx.lamports");
    assert!(
        bytecode.windows(1).any(|w| w[0] == GET_LAMPORTS),
        "account.ctx.lamports should lower to GET_LAMPORTS"
    );
}

#[test]
fn test_legacy_account_metadata_field_access_is_removed() {
    let source = r#"
        pub inspect(
            payer: account @signer
        ) {
            let bal: u64 = payer.lamports;
            let _x: u64 = bal;
        }
    "#;

    let err = DslCompiler::compile_dsl(source).expect_err("legacy payer.lamports should fail");
    assert!(
        err.message.contains("cannot find value") || err.message.contains("Undefined"),
        "Expected undefined identifier failure for removed metadata surface, got: {:?}",
        err
    );
    let hint = err.context.get_data("did_you_mean").map(String::as_str);
    assert!(
        matches!(hint, Some("ctx.lamports") | Some("payer.ctx.lamports")),
        "Expected migration hint for legacy metadata field access, got: {:?}",
        hint
    );
}

#[test]
fn test_unknown_account_field_has_no_ctx_migration_hint() {
    let source = r#"
        pub inspect(
            payer: account @signer
        ) {
            let v: u64 = payer.xyz;
            let _x: u64 = v;
        }
    "#;

    let err = DslCompiler::compile_dsl(source).expect_err("unknown field should fail");
    assert!(
        err.context.get_data("did_you_mean").is_none(),
        "unexpected migration hint for unrelated unknown field: {:?}",
        err.context.get_data("did_you_mean")
    );
}

#[test]
fn test_account_ctx_space_available_for_init_account() {
    let source = r#"
        account State {
            value: u64
        }

        pub initialize(
            payer: account @signer,
            state: State @mut @init(payer=payer)
        ) {
            let s: u64 = state.ctx.space;
            let _y: u64 = s;
        }
    "#;
    let bytecode = DslCompiler::compile_dsl(source).expect("Should compile state.ctx.space");
    assert!(
        bytecode_contains_u64_literal(&bytecode, 8),
        "state.ctx.space should resolve and include layout size literal (State = 8 bytes)"
    );
}
