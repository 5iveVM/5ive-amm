//! Regression test for Issue 4: u128 Gap For DeFi (Stateful Math)
//!
//! Tests that:
//! 1. u128 works fine in function parameters
//! 2. u128 works fine in local variables
//! 3. u128 FAILS in account field definitions
//! 4. Other numeric types (u64, u32, etc.) work in account fields
//! 5. The failure is during account registration, not parameter validation

use five_dsl_compiler::compiler::DslCompiler;

#[test]
fn test_u128_in_function_parameters() {
    // u128 should work fine in function parameters
    let dsl = r#"
pub transfer(amount: u128) -> u128 {
    return amount + 1;
}

pub main() {
}
"#;

    let result = DslCompiler::compile_dsl(dsl);

    // Should succeed - u128 is supported as parameter type
    assert!(
        result.is_ok(),
        "u128 should work as function parameter type"
    );
}

#[test]
fn test_u128_in_local_variables() {
    // u128 should work fine in local variables
    let dsl = r#"
pub compute() -> u128 {
    let x: u128 = 1000000000000;
    let y: u128 = 2000000000000;
    let sum = x + y;
    return sum;
}

pub main() {
}
"#;

    let result = DslCompiler::compile_dsl(dsl);

    // Should succeed - u128 works in locals
    assert!(
        result.is_ok(),
        "u128 should work as local variable type"
    );
}

#[test]
fn test_u128_in_account_fields_succeeds() {
    // FIXED: u128 now works in account field definitions
    let dsl = r#"
account Vault {
    amount: u128,
}

pub main() {
}
"#;

    let result = DslCompiler::compile_dsl(dsl);

    // Should succeed - u128 is now supported in account fields
    assert!(
        result.is_ok(),
        "u128 should work in account field definitions"
    );
}

#[test]
fn test_u64_in_account_fields_works() {
    // u64 should work fine in account fields (as baseline)
    let dsl = r#"
account Balance {
    amount: u64,
}

pub main() {
}
"#;

    let result = DslCompiler::compile_dsl(dsl);

    // Should succeed - u64 is supported in account fields
    assert!(result.is_ok(), "u64 should work in account field definitions");
}

#[test]
fn test_various_numeric_types_work() {
    // Various numeric types should work in account fields
    let dsl = r#"
account NumericAccount {
    u8_field: u8,
    u16_field: u16,
    u32_field: u32,
    u64_field: u64,
    u128_field: u128,
}

pub main() {
}
"#;

    let result = DslCompiler::compile_dsl(dsl);

    // Should succeed - numeric types including u128 now work
    assert!(
        result.is_ok(),
        "Numeric types including u128 should work in account fields"
    );
}

#[test]
fn test_u128_mixed_with_other_types() {
    // FIXED: u128 can now be mixed with other field types
    let dsl = r#"
account DeFiVault {
    locked_amount: u64,
    high_precision_value: u128,  // Now works alongside other types
    fee_rate: u64,
}

pub main() {
}
"#;

    let result = DslCompiler::compile_dsl(dsl);

    // Should succeed - u128 fields work with mixed types
    assert!(
        result.is_ok(),
        "u128 fields should work alongside other types"
    );
}

#[test]
fn test_u128_in_struct_definition() {
    // Test if u128 also fails in struct definitions
    let dsl = r#"
struct Amount {
    value: u128,
}

pub main() {
}
"#;

    let result = DslCompiler::compile_dsl(dsl);

    // Structs might have different behavior from accounts
    // Document whatever the current behavior is
    if result.is_err() {
        eprintln!("u128 in struct definition: FAILS");
    } else {
        eprintln!("u128 in struct definition: WORKS");
    }
}

#[test]
fn test_u128_in_function_locally() {
    // u128 can be used in function locals (already tested above)
    let dsl = r#"
pub process() -> u64 {
    let amount: u128 = 1000;
    return 42;  // Just return a simple value
}

pub main() {
}
"#;

    let result = DslCompiler::compile_dsl(dsl);

    // Should succeed - u128 locals work
    assert!(
        result.is_ok(),
        "u128 locals should work in functions"
    );
}

#[test]
fn test_u128_with_defi_pattern() {
    // Test realistic DeFi pattern: u128 amounts in parameters, u64 in storage
    let dsl = r#"
account Pool {
    reserve_a: u64,  // Works fine
    reserve_b: u64,  // Works fine
    // high_precision_reserve: u128,  // Would fail if uncommented
}

pub swap(amount_in: u128, min_out: u128) -> u128 {
    // u128 works for calculations
    let output = amount_in * 2;
    return output;
}

pub main() {
}
"#;

    let result = DslCompiler::compile_dsl(dsl);

    // Should succeed - demonstrates current workaround (u128 in params, u64 in storage)
    assert!(
        result.is_ok(),
        "Current workaround: u128 in parameters, u64 in account fields"
    );
}

#[test]
fn test_u128_field_size_not_calculated() {
    // Root cause: calculate_type_size() doesn't have u128 case
    // This causes account registration to fail when trying to compute field offsets
    let dsl = r#"
account TokenSwap {
    token_a_reserve: u64,
    token_b_reserve: u64,
    // Uncomment to trigger the error:
    // lp_supply: u128,  // Missing from calculate_type_size() match statement
}

pub main() {
}
"#;

    let result = DslCompiler::compile_dsl(dsl);

    // Should succeed with all u64 fields
    assert!(result.is_ok(), "All u64 fields should work");
}
