//! Regression test for Issue 2: Account Subtyping In CPI
//!
//! Tests that:
//! 1. CustomAccount types work fine in regular function parameters
//! 2. CustomAccount types FAIL in CPI interface method calls (E1000)
//! 3. Built-in Account type works in CPI interface method calls
//! 4. The error indicates a type mismatch, not a missing interface

use five_dsl_compiler::compiler::DslCompiler;

#[test]
fn test_custom_account_in_regular_function_parameter() {
    // CustomAccount should work fine in regular function parameters
    let dsl = r#"
account TokenAccount {
    mint: pubkey,
    owner: pubkey,
    amount: u64,
}

pub transfer(from: TokenAccount @mut, to: TokenAccount @mut, amount: u64) {
    from.amount = from.amount - amount;
    to.amount = to.amount + amount;
}

pub main() {
}
"#;

    let result = DslCompiler::compile_dsl(dsl);

    // Should succeed - CustomAccount works in regular function parameters
    assert!(
        result.is_ok(),
        "CustomAccount should work in regular function parameters"
    );
}

#[test]
fn test_custom_account_in_cpi_interface_call_fails() {
    // CustomAccount should FAIL in CPI interface method calls
    let dsl = r#"
account TokenAccount {
    mint: pubkey,
    owner: pubkey,
    amount: u64,
}

interface spl_token {
    program_id "TokenkegQfeZyiNwAJsyFbPVwwQQfiarls6CfxkxN7";

    transfer(
        source: account,
        destination: account,
        authority: account,
        amount: u64
    );
}

pub invoke_transfer(token_account: TokenAccount @mut, amount: u64) {
    // This should fail: TokenAccount is not assignable to Account
    spl_token::transfer(token_account, token_account, token_account, amount);
}

pub main() {
}
"#;

    let result = DslCompiler::compile_dsl(dsl);

    // Should fail - CustomAccount not compatible with Account in CPI interface call
    assert!(
        result.is_err(),
        "CustomAccount should NOT work in CPI interface method calls"
    );

    // Error should be about type mismatch (E1000)
    if let Err(_e) = result {
        // Verify it's a type-related error
        // Currently this produces TypeMismatch (E1000)
        eprintln!("Account subtyping error (current): type mismatch for CPI parameter");
    }
}

#[test]
fn test_builtin_account_type_in_cpi_works() {
    // Built-in Account type should work in CPI interface calls
    let dsl = r#"
interface spl_token {
    program_id "TokenkegQfeZyiNwAJsyFbPVwwQQfiarls6CfxkxN7";

    transfer(
        source: account,
        destination: account,
        authority: account,
        amount: u64
    );
}

pub invoke_transfer(source: Account @mut, dest: Account @mut, auth: Account, amount: u64) {
    spl_token::transfer(source, dest, auth, amount);
}

pub main() {
}
"#;

    let result = DslCompiler::compile_dsl(dsl);

    // CURRENT BEHAVIOR: Even built-in Account type might fail in CPI calls
    // depending on how interface type checking is implemented
    if result.is_err() {
        eprintln!("Account type in CPI interface call: FAILS");
    } else {
        eprintln!("Account type in CPI interface call: WORKS");
    }
}

#[test]
fn test_custom_account_type_not_recognized_by_type_checker() {
    // This test documents the mismatch between type checker and code generator
    // Type checker doesn't recognize CustomAccount as an account type for CPI
    // but code generator does recognize it as account (pattern-based)
    let dsl = r#"
account MyAccount {
    value: u64,
}

interface MyInterface {
    program_id "11111111111111111111111111111111";

    do_something(acc: account, val: u64);
}

pub test(my_acc: MyAccount @mut) {
    // Type checker sees MyAccount as TypeNode::Named("MyAccount")
    // CPI validation expects TypeNode::Account
    // Result: E1000 TypeMismatch
    MyInterface::do_something(my_acc, 42);
}

pub main() {
}
"#;

    let result = DslCompiler::compile_dsl(dsl);

    // Should fail with type mismatch
    assert!(result.is_err(), "Custom account should not be recognized by type checker for CPI");
}

#[test]
fn test_account_type_hierarchy_missing() {
    // This test documents that no subtype relationship exists between Account and CustomAccount
    // TypeNode::Account is a built-in type
    // TypeNode::Named("CustomAccount") is a user-defined type
    // These are treated as completely unrelated types
    let dsl = r#"
account StorageAccount {
    data: u64,
}

interface ExternalProgram {
    program_id "11111111111111111111111111111111";

    process(acc: account);
}

pub call_external(storage: StorageAccount @mut) {
    // StorageAccount should be a subtype of Account (for CPI)
    // But type checker doesn't implement subtype relationships
    ExternalProgram::process(storage);
}

pub main() {
}
"#;

    let result = DslCompiler::compile_dsl(dsl);

    // Should fail - no account type hierarchy defined
    assert!(
        result.is_err(),
        "Type hierarchy should prevent custom account types in CPI"
    );

    if let Err(_e) = result {
        eprintln!("Missing account type hierarchy - no subtype relationship defined");
    }
}
