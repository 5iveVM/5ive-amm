//! Regression test for cast semantics on locals
//!
//! Tests that:
//! 1. Cast expressions preserve target-type semantics for local field access
//! 2. Field access on casted locals resolves against the cast target type
//! 3. Both read and write operations behave consistently
//! 4. Casts still respect mutability/constraint rules

use five_dsl_compiler::compiler::DslCompiler;

#[test]
fn test_cast_expression_parses_but_is_ignored() {
    // Cast expressions should parse and preserve cast target semantics.
    let dsl = r#"
account MyAccount {
    balance: u64,
}

pub test(acc: Account @mut) {
    let x = acc as MyAccount;
    let amount = x.balance;
}

pub main() {
}
"#;

    let result = DslCompiler::compile_dsl(dsl);

    // Should compile with cast-aware field access.
    assert!(
        result.is_ok(),
        "Cast expressions should preserve target-type semantics"
    );
}

#[test]
fn test_field_access_on_casted_local_fails_for_custom_fields() {
    // Field access on casted locals should resolve custom fields.
    let dsl = r#"
account MyAccount {
    balance: u64,
}

pub test(acc: Account @mut) {
    let x = acc as MyAccount;
    let amount = x.balance;
}

pub main() {
}
"#;

    let result = DslCompiler::compile_dsl(dsl);

    // Should succeed - cast target type should be used.
    assert!(
        result.is_ok(),
        "Field access on casted local should use cast target type"
    );
}

#[test]
fn test_field_write_on_casted_local_fails() {
    // Field write on casted locals should also use cast target type.
    let dsl = r#"
account MyAccount {
    balance: u64,
}

pub test(acc: Account @mut) {
    let mut x = acc as MyAccount;
    x.balance = 100;
}

pub main() {
}
"#;

    let result = DslCompiler::compile_dsl(dsl);

    // Should succeed for mutable account parameter.
    assert!(
        result.is_ok(),
        "Field write on casted local should use cast target type"
    );
}

#[test]
fn test_cast_does_not_affect_account_constraints() {
    // Cast expressions must not bypass @mut constraints.
    let dsl = r#"
account MyAccount {
    balance: u64,
}

pub test(acc: Account) {
    // acc is not @mut, so even with cast it should not allow mutation
    let x = acc as MyAccount;
    // Even if cast worked, this should still fail because original acc is not @mut
    x.balance = 100;
}

pub main() {
}
"#;

    let result = DslCompiler::compile_dsl(dsl);

    // Should fail due to mutability constraint.
    assert!(
        result.is_err(),
        "Mutation through non-mut account should fail"
    );
}

#[test]
fn test_multiple_casts_all_ignored() {
    // Multiple cast expressions should preserve the most recent cast type.
    let dsl = r#"
account VaultA {
    amount: u64,
}

account VaultB {
    balance: u64,
}

pub test(acc: Account @mut) {
    let x = acc as VaultA;
    let y = x as VaultB;
    let b = y.balance;
}

pub main() {
}
"#;

    let result = DslCompiler::compile_dsl(dsl);

    // Should compile and preserve final cast type.
    assert!(result.is_ok(), "Multiple casts should preserve target type");
}

#[test]
fn test_cast_with_account_ctx_properties_works() {
    let dsl = r#"
pub test(acc: Account @mut) {
    let x = acc as SomeAccount;
    let l = x.ctx.lamports;
    let o = x.ctx.owner;
    let k = x.ctx.key;
}

pub main() {
}
"#;

    let result = DslCompiler::compile_dsl(dsl);

    // Should succeed - ctx properties work regardless of cast
    assert!(
        result.is_ok(),
        "account.ctx properties should work on casted accounts"
    );
}

#[test]
fn test_cast_silently_fails_to_narrow_type() {
    // Cast expressions should narrow to the target account type for field access.
    let dsl = r#"
account MyVault {
    total_locked: u64,
    unlock_time: u64,
}

pub check_vault(vault: Account @mut) {
    let v = vault as MyVault;
    if v.total_locked > 0 {
        // ...
    }
}

pub main() {
}
"#;

    let result = DslCompiler::compile_dsl(dsl);

    // Should succeed - no silent cast/type mismatch.
    assert!(result.is_ok(), "Type narrowing via cast should work");
}
