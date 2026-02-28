//! Regression test for Issue 1: Cryptic Field Limit Error
//!
//! Tests that:
//! 1. Structs/accounts with ≤64 fields compile successfully
//! 2. Structs/accounts with >64 fields fail with appropriate error
//! 3. Error message indicates the field limit issue (currently generic InvalidScript)

use five_dsl_compiler::compiler::DslCompiler;

#[test]
fn test_field_limit_64_fields_succeeds() {
    // Account with exactly 64 fields should compile
    let dsl = r#"
account LargeAccount {
    f0: u64, f1: u64, f2: u64, f3: u64, f4: u64, f5: u64, f6: u64, f7: u64,
    f8: u64, f9: u64, f10: u64, f11: u64, f12: u64, f13: u64, f14: u64, f15: u64,
    f16: u64, f17: u64, f18: u64, f19: u64, f20: u64, f21: u64, f22: u64, f23: u64,
    f24: u64, f25: u64, f26: u64, f27: u64, f28: u64, f29: u64, f30: u64, f31: u64,
    f32: u64, f33: u64, f34: u64, f35: u64, f36: u64, f37: u64, f38: u64, f39: u64,
    f40: u64, f41: u64, f42: u64, f43: u64, f44: u64, f45: u64, f46: u64, f47: u64,
    f48: u64, f49: u64, f50: u64, f51: u64, f52: u64, f53: u64, f54: u64, f55: u64,
    f56: u64, f57: u64, f58: u64, f59: u64, f60: u64, f61: u64, f62: u64, f63: u64,
}

pub main() {
}
"#;

    let result = DslCompiler::compile_dsl(dsl);

    // Should succeed - 64 fields is at the limit
    assert!(
        result.is_ok(),
        "Compilation should succeed with exactly 64 fields"
    );
}

#[test]
fn test_field_limit_65_fields_compiles() {
    // CURRENT BEHAVIOR: Account with 65 fields actually COMPILES successfully
    // The 64-field limit only exists at serialization time, not at compilation time
    // This is part of the root cause - the limit is not enforced early
    let dsl = r#"
account LargeAccount {
    f0: u64, f1: u64, f2: u64, f3: u64, f4: u64, f5: u64, f6: u64, f7: u64,
    f8: u64, f9: u64, f10: u64, f11: u64, f12: u64, f13: u64, f14: u64, f15: u64,
    f16: u64, f17: u64, f18: u64, f19: u64, f20: u64, f21: u64, f22: u64, f23: u64,
    f24: u64, f25: u64, f26: u64, f27: u64, f28: u64, f29: u64, f30: u64, f31: u64,
    f32: u64, f33: u64, f34: u64, f35: u64, f36: u64, f37: u64, f38: u64, f39: u64,
    f40: u64, f41: u64, f42: u64, f43: u64, f44: u64, f45: u64, f46: u64, f47: u64,
    f48: u64, f49: u64, f50: u64, f51: u64, f52: u64, f53: u64, f54: u64, f55: u64,
    f56: u64, f57: u64, f58: u64, f59: u64, f60: u64, f61: u64, f62: u64, f63: u64,
    f64: u64,
}

pub main() {
}
"#;

    let result = DslCompiler::compile_dsl(dsl);

    // FIXED: Compilation now fails early with >64 fields
    // Field limit is now enforced during account registration
    assert!(
        result.is_err(),
        "Account with >64 fields should fail compilation"
    );
}

#[test]
fn test_field_limit_struct_definition() {
    // Test that struct definitions with >64 fields FAIL at compilation
    // (Unlike accounts, which pass compilation but fail at serialization)
    let dsl = r#"
struct LargeStruct {
    f0: u64, f1: u64, f2: u64, f3: u64, f4: u64, f5: u64, f6: u64, f7: u64,
    f8: u64, f9: u64, f10: u64, f11: u64, f12: u64, f13: u64, f14: u64, f15: u64,
    f16: u64, f17: u64, f18: u64, f19: u64, f20: u64, f21: u64, f22: u64, f23: u64,
    f24: u64, f25: u64, f26: u64, f27: u64, f28: u64, f29: u64, f30: u64, f31: u64,
    f32: u64, f33: u64, f34: u64, f35: u64, f36: u64, f37: u64, f38: u64, f39: u64,
    f40: u64, f41: u64, f42: u64, f43: u64, f44: u64, f45: u64, f46: u64, f47: u64,
    f48: u64, f49: u64, f50: u64, f51: u64, f52: u64, f53: u64, f54: u64, f55: u64,
    f56: u64, f57: u64, f58: u64, f59: u64, f60: u64, f61: u64, f62: u64, f63: u64,
    f64: u64,
}

pub main() {
}
"#;

    let result = DslCompiler::compile_dsl(dsl);

    // CURRENT BEHAVIOR: Struct definitions with >64 fields FAIL at compilation
    // (earlier than accounts, which pass compilation)
    assert!(
        result.is_err(),
        "Struct definitions with >64 fields should fail"
    );
}

#[test]
fn test_field_limit_event_definition() {
    // Test that event definitions compile regardless of field count
    // (limit is only at serialization time)
    let dsl = r#"
event LargeEvent {
    f0: u64, f1: u64, f2: u64, f3: u64, f4: u64, f5: u64, f6: u64, f7: u64,
    f8: u64, f9: u64, f10: u64, f11: u64, f12: u64, f13: u64, f14: u64, f15: u64,
    f16: u64, f17: u64, f18: u64, f19: u64, f20: u64, f21: u64, f22: u64, f23: u64,
    f24: u64, f25: u64, f26: u64, f27: u64, f28: u64, f29: u64, f30: u64, f31: u64,
    f32: u64, f33: u64, f34: u64, f35: u64, f36: u64, f37: u64, f38: u64, f39: u64,
    f40: u64, f41: u64, f42: u64, f43: u64, f44: u64, f45: u64, f46: u64, f47: u64,
    f48: u64, f49: u64, f50: u64, f51: u64, f52: u64, f53: u64, f54: u64, f55: u64,
    f56: u64, f57: u64, f58: u64, f59: u64, f60: u64, f61: u64, f62: u64, f63: u64,
    f64: u64,
}

pub main() {
}
"#;

    let result = DslCompiler::compile_dsl(dsl);

    // CURRENT BEHAVIOR: Event definitions compile even with >64 fields
    // The error would only occur at serialization time
    assert!(
        result.is_ok(),
        "Event definitions compile regardless of field count"
    );
}
