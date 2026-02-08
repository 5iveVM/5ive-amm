//! Comprehensive tests for polymorphic arithmetic operations
//!
//! This module tests the new polymorphic arithmetic system that supports
//! mixed u64/u128 operations with automatic type promotion.

use crate::{ExecutionContext, Pubkey, Result, StackStorage};
use five_protocol::{opcodes::*, ValueRef};

/// Test u64×u64 operations maintain fast path and u64 results
#[test]
fn test_u64_arithmetic_fast_path() -> Result<()> {
    let mut storage = StackStorage::new();
    let mut ctx = ExecutionContext::new(&[], &[], Pubkey::default(), &[], 0, &mut storage, 0, 0, 0, 0, 0, 0);

    // Test ADD: 10 + 5 = 15
    ctx.push(ValueRef::U64(10))?;
    ctx.push(ValueRef::U64(5))?;
    crate::handlers::handle_arithmetic(ADD, &mut ctx)?;

    match ctx.pop()? {
        ValueRef::U64(result) => assert_eq!(result, 15),
        other => panic!("Expected U64, got {:?}", other),
    }

    // Test SUB: 10 - 3 = 7
    ctx.push(ValueRef::U64(10))?;
    ctx.push(ValueRef::U64(3))?;
    crate::handlers::handle_arithmetic(SUB, &mut ctx)?;

    match ctx.pop()? {
        ValueRef::U64(result) => assert_eq!(result, 7),
        other => panic!("Expected U64, got {:?}", other),
    }

    // Test MUL: 6 * 7 = 42
    ctx.push(ValueRef::U64(6))?;
    ctx.push(ValueRef::U64(7))?;
    crate::handlers::handle_arithmetic(MUL, &mut ctx)?;

    match ctx.pop()? {
        ValueRef::U64(result) => assert_eq!(result, 42),
        other => panic!("Expected U64, got {:?}", other),
    }

    // Test DIV: 42 / 6 = 7
    ctx.push(ValueRef::U64(42))?;
    ctx.push(ValueRef::U64(6))?;
    crate::handlers::handle_arithmetic(DIV, &mut ctx)?;

    match ctx.pop()? {
        ValueRef::U64(result) => assert_eq!(result, 7),
        other => panic!("Expected U64, got {:?}", other),
    }

    Ok(())
}

/// Test u128×u128 operations produce u128 results
#[test]
fn test_u128_arithmetic() -> Result<()> {
    let mut storage = StackStorage::new();
    let mut ctx = ExecutionContext::new(&[], &[], Pubkey::default(), &[], 0, &mut storage, 0, 0, 0, 0, 0, 0);

    // Test ADD: large u128 values
    let a = 0xFFFFFFFFFFFFFFFF_u128 + 100; // Beyond u64::MAX
    let b = 1000_u128;

    ctx.push(ValueRef::U128(a))?;
    ctx.push(ValueRef::U128(b))?;
    crate::handlers::handle_arithmetic(ADD, &mut ctx)?;

    match ctx.pop()? {
        ValueRef::U128(result) => assert_eq!(result, a.wrapping_add(b)),
        other => panic!("Expected U128, got {:?}", other),
    }

    // Test SUB with underflow wrapping
    ctx.push(ValueRef::U128(100))?;
    ctx.push(ValueRef::U128(200))?;
    crate::handlers::handle_arithmetic(SUB, &mut ctx)?;

    match ctx.pop()? {
        ValueRef::U128(result) => assert_eq!(result, 100_u128.wrapping_sub(200)),
        other => panic!("Expected U128, got {:?}", other),
    }

    Ok(())
}

/// Test mixed u64×u128 operations with automatic promotion
#[test]
fn test_mixed_arithmetic_promotion() -> Result<()> {
    let mut storage = StackStorage::new();
    let mut ctx = ExecutionContext::new(&[], &[], Pubkey::default(), &[], 0, &mut storage, 0, 0, 0, 0, 0, 0);

    // Test u64 + u128 → u128
    ctx.push(ValueRef::U64(1000))?;
    ctx.push(ValueRef::U128(2000))?;
    crate::handlers::handle_arithmetic(ADD, &mut ctx)?;

    match ctx.pop()? {
        ValueRef::U128(result) => assert_eq!(result, 3000),
        other => panic!("Expected U128, got {:?}", other),
    }

    // Test u128 + u64 → u128
    ctx.push(ValueRef::U128(5000))?;
    ctx.push(ValueRef::U64(1000))?;
    crate::handlers::handle_arithmetic(ADD, &mut ctx)?;

    match ctx.pop()? {
        ValueRef::U128(result) => assert_eq!(result, 6000),
        other => panic!("Expected U128, got {:?}", other),
    }

    // Test mixed multiplication
    ctx.push(ValueRef::U64(123))?;
    ctx.push(ValueRef::U128(456))?;
    crate::handlers::handle_arithmetic(MUL, &mut ctx)?;

    match ctx.pop()? {
        ValueRef::U128(result) => assert_eq!(result, 123 * 456),
        other => panic!("Expected U128, got {:?}", other),
    }

    Ok(())
}

/// Test division with mixed types and zero checking
#[test]
fn test_mixed_division() -> Result<()> {
    let mut storage = StackStorage::new();
    let mut ctx = ExecutionContext::new(&[], &[], Pubkey::default(), &[], 0, &mut storage, 0, 0, 0, 0, 0, 0);

    // Test u64 / u128 → u128
    ctx.push(ValueRef::U64(1000))?;
    ctx.push(ValueRef::U128(10))?;
    crate::handlers::handle_arithmetic(DIV, &mut ctx)?;

    match ctx.pop()? {
        ValueRef::U128(result) => assert_eq!(result, 100),
        other => panic!("Expected U128, got {:?}", other),
    }

    // Test u128 / u64 → u128
    ctx.push(ValueRef::U128(10000))?;
    ctx.push(ValueRef::U64(100))?;
    crate::handlers::handle_arithmetic(DIV, &mut ctx)?;

    match ctx.pop()? {
        ValueRef::U128(result) => assert_eq!(result, 100),
        other => panic!("Expected U128, got {:?}", other),
    }

    Ok(())
}

/// Test division by zero detection in mixed types
#[test]
fn test_division_by_zero_mixed() {
    let mut storage = StackStorage::new();
    let mut ctx = ExecutionContext::new(&[], &[], Pubkey::default(), &[], 0, &mut storage, 0, 0);

    // Test u64 / u128(0) → DivisionByZero error
    ctx.push(ValueRef::U64(100)).unwrap(, 0, 0, 0, 0);
    ctx.push(ValueRef::U128(0)).unwrap(, 0, 0, 0, 0);
    let result = crate::handlers::handle_arithmetic(DIV, &mut ctx);

    assert!(matches!(result, Err(crate::error::VMErrorCode::DivisionByZero)));

    // Test u128 / u64(0) → DivisionByZero error
    ctx.push(ValueRef::U128(100)).unwrap();
    ctx.push(ValueRef::U64(0)).unwrap();
    let result = crate::handlers::handle_arithmetic(DIV, &mut ctx);

    assert!(matches!(result, Err(crate::error::VMErrorCode::DivisionByZero)));
}

/// Test comparison operations with mixed types
#[test]
fn test_mixed_comparisons() -> Result<()> {
    let mut storage = StackStorage::new();
    let mut ctx = ExecutionContext::new(&[], &[], Pubkey::default(), &[], 0, &mut storage, 0, 0, 0, 0, 0, 0);

    // Test u64 < u128
    ctx.push(ValueRef::U64(100))?;
    ctx.push(ValueRef::U128(200))?;
    crate::handlers::handle_arithmetic(LT, &mut ctx)?;

    match ctx.pop()? {
        ValueRef::Bool(result) => assert!(result),
        other => panic!("Expected Bool, got {:?}", other),
    }

    // Test u128 > u64
    ctx.push(ValueRef::U128(500))?;
    ctx.push(ValueRef::U64(100))?;
    crate::handlers::handle_arithmetic(GT, &mut ctx)?;

    match ctx.pop()? {
        ValueRef::Bool(result) => assert!(result),
        other => panic!("Expected Bool, got {:?}", other),
    }

    // Test u64 == u128 (same value)
    ctx.push(ValueRef::U64(42))?;
    ctx.push(ValueRef::U128(42))?;
    crate::handlers::handle_arithmetic(EQ, &mut ctx)?;

    match ctx.pop()? {
        ValueRef::Bool(result) => assert!(result),
        other => panic!("Expected Bool, got {:?}", other),
    }

    Ok(())
}

/// Test narrowing u128 to u64 with overflow detection
#[test]
fn test_u128_to_u64_narrowing() -> Result<()> {
    let mut storage = StackStorage::new();
    let mut ctx = ExecutionContext::new(&[], &[], Pubkey::default(), &[], 0, &mut storage, 0, 0, 0, 0, 0, 0);

    // Test successful narrowing (value fits in u64)
    ctx.push(ValueRef::U128(1000))?;
    let result = crate::pop_u64!(ctx, 0, 0, 0, 0);
    assert_eq!(result, 1000);

    // Test overflow detection (value exceeds u64::MAX)
    let large_value = (u64::MAX as u128) + 1;
    ctx.push(ValueRef::U128(large_value))?;

    // This should trigger NumericOverflow error
    let mut overflow_test = || -> crate::error::CompactResult<u64> { Ok(crate::pop_u64!(ctx)) };

    let result = overflow_test();
    assert!(matches!(
        result,
        Err(crate::error::VMErrorCode::NumericOverflow)
    ));

    Ok(())
}

/// Test edge cases and boundary conditions
#[test]
fn test_arithmetic_edge_cases() -> Result<()> {
    let mut storage = StackStorage::new();
    let mut ctx = ExecutionContext::new(&[], &[], Pubkey::default(), &[], 0, &mut storage, 0, 0, 0, 0, 0, 0);

    // Test u64::MAX + u64(1) → u128 (promotion prevents overflow)
    ctx.push(ValueRef::U64(u64::MAX))?;
    ctx.push(ValueRef::U64(1))?;
    crate::handlers::handle_arithmetic(ADD, &mut ctx)?;

    // This should stay u64 and wrap
    match ctx.pop()? {
        ValueRef::U64(result) => assert_eq!(result, u64::MAX.wrapping_add(1)),
        other => panic!("Expected U64, got {:?}", other),
    }

    // Test u64::MAX + u128(1) → u128 (promotion handles properly)
    ctx.push(ValueRef::U64(u64::MAX))?;
    ctx.push(ValueRef::U128(1))?;
    crate::handlers::handle_arithmetic(ADD, &mut ctx)?;

    match ctx.pop()? {
        ValueRef::U128(result) => assert_eq!(result, (u64::MAX as u128) + 1),
        other => panic!("Expected U128, got {:?}", other),
    }

    Ok(())
}

/// Test that type mismatch still produces errors for invalid types
#[test]
fn test_type_mismatch_errors() {
    let mut storage = StackStorage::new();
    let mut ctx = ExecutionContext::new(&[], &[], Pubkey::default(), &[], 0, &mut storage, 0, 0);

    // Test arithmetic with Bool (should fail)
    ctx.push(ValueRef::U64(100)).unwrap(, 0, 0, 0, 0);
    ctx.push(ValueRef::Bool(true)).unwrap(, 0, 0, 0, 0);
    let result = crate::handlers::handle_arithmetic(ADD, &mut ctx);

    assert!(matches!(result, Err(crate::error::VMErrorCode::TypeMismatch)));
}
