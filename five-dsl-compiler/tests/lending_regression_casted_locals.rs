//! Regression test for Issue 3: Field Access On Casted Locals
//!
//! Tests that:
//! 1. Cast expressions parse without error (but cast info is lost)
//! 2. Field access on casted locals fails (uses original type, not cast type)
//! 3. Both read and write operations on casted locals are affected
//! 4. Cast information is completely ignored by type system

use five_dsl_compiler::compiler::DslCompiler;

#[test]
fn test_cast_expression_parses_but_is_ignored() {
    // Cast expressions parse syntactically but cast type is discarded semantically
    let dsl = r#"
account MyAccount {
    balance: u64,
}

pub test(acc: Account @mut) {
    let x = acc as MyAccount;
    // At this point: x is still typed as Account in symbol table
    // The "as MyAccount" part was parsed but discarded
}

pub main() {
}
"#;

    let result = DslCompiler::compile_dsl(dsl);

    // Should compile - cast syntax is accepted (even though it's ignored)
    assert!(
        result.is_ok(),
        "Cast expressions should parse without syntax errors"
    );
}

#[test]
fn test_field_access_on_casted_local_fails_for_custom_fields() {
    // Field access on casted locals uses original type (Account), not cast type (MyAccount)
    let dsl = r#"
account MyAccount {
    balance: u64,
}

pub test(acc: Account @mut) {
    let x = acc as MyAccount;
    let amount = x.balance;  // ERROR: MyAccount field not found on Account
}

pub main() {
}
"#;

    let result = DslCompiler::compile_dsl(dsl);

    // Should fail - field access uses original type (Account), not cast type (MyAccount)
    assert!(
        result.is_err(),
        "Field access on casted local should fail (uses original type)"
    );

    // Error should be about undefined field
    if let Err(_e) = result {
        eprintln!("Field access on casted local fails - cast information lost");
    }
}

#[test]
fn test_field_write_on_casted_local_fails() {
    // Field write on casted locals also fails (same issue as read)
    let dsl = r#"
account MyAccount {
    balance: u64,
}

pub test(acc: Account @mut) {
    let x = acc as MyAccount;
    x.balance = 100;  // ERROR: MyAccount field not found on Account
}

pub main() {
}
"#;

    let result = DslCompiler::compile_dsl(dsl);

    // Should fail - field write also uses original type
    assert!(
        result.is_err(),
        "Field write on casted local should also fail"
    );
}

#[test]
fn test_cast_does_not_affect_account_constraints() {
    // Cast expressions are completely ignored - they don't affect @mut constraints
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

    // Should fail - for multiple reasons:
    // 1. Cast type information is lost (would use Account type)
    // 2. Even if cast worked, original acc is not @mut
    assert!(result.is_err(), "Mutation through non-mut account should fail");
}

#[test]
fn test_multiple_casts_all_ignored() {
    // Multiple cast expressions are all ignored
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
    // y is still typed as Account - both casts ignored
}

pub main() {
}
"#;

    let result = DslCompiler::compile_dsl(dsl);

    // Should compile but casts are semantically ignored
    assert!(result.is_ok(), "Multiple casts should parse");
}

#[test]
fn test_cast_with_builtin_account_properties_works() {
    // Built-in account properties (lamports, owner, key, data) should still work
    // because they're available on any Account type
    let dsl = r#"
pub test(acc: Account @mut) {
    let x = acc as SomeAccount;
    let l = x.lamports;  // Should work - built-in property
    let o = x.owner;     // Should work - built-in property
    let k = x.key;       // Should work - built-in property
    // x.data is also built-in but less commonly used
}

pub main() {
}
"#;

    let result = DslCompiler::compile_dsl(dsl);

    // Should succeed - built-in properties work regardless of cast
    assert!(
        result.is_ok(),
        "Built-in account properties should work on casted accounts"
    );
}

#[test]
fn test_cast_silently_fails_to_narrow_type() {
    // Cast expressions appear to work (no syntax error) but silently fail to narrow type
    // This creates a subtle bug where what you write (acc as MyAccount) != what compiler uses (Account)
    let dsl = r#"
account MyVault {
    total_locked: u64,
    unlock_time: u64,
}

pub check_vault(vault: Account @mut) {
    let v = vault as MyVault;
    // Developer expects: v is of type MyVault
    // Compiler actually: v is of type Account
    // This silent mismatch is the core of the issue

    if v.total_locked > 0 {  // ERROR: undefined field
        // ...
    }
}

pub main() {
}
"#;

    let result = DslCompiler::compile_dsl(dsl);

    // Should fail - demonstrates the silent type inconsistency
    assert!(
        result.is_err(),
        "Type narrowing via cast silently fails to work"
    );

    if let Err(_e) = result {
        eprintln!("Silent type narrowing failure - cast syntax accepted but semantically ignored");
    }
}
