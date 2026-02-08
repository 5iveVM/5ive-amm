//! Performance tests for Five VM optimization features
//!
//! This module tests the performance characteristics of key optimizations:
//! - Lazy account validation
//! - Bitwise constraint checking  
//! - Optimized header parsing and dispatch efficiency

use five_vm_mito::{
    context::ExecutionContext,
    lazy_validation::{LazyAccountValidator, ValidationStats},
    StackStorage,
};
use pinocchio::{account_info::AccountInfo, pubkey::Pubkey};
use std::time::Instant;

fn account_info_with_flags(
    key: Pubkey,
    lamports: u64,
    data: Vec<u8>,
    owner: Pubkey,
    is_signer: bool,
    is_writable: bool,
    executable: bool,
) -> AccountInfo {
    let key_ref = Box::leak(Box::new(key));
    let owner_ref = Box::leak(Box::new(owner));
    let lamports_ref = Box::leak(Box::new(lamports));
    let data_ref = Box::leak(data.into_boxed_slice());

    AccountInfo::new(
        key_ref,
        is_signer,
        is_writable,
        lamports_ref,
        data_ref,
        owner_ref,
        executable,
        0,
    )
}

/// Test lazy validation performance characteristics
#[test]
fn test_lazy_validation_performance() {
    const TOTAL_ACCOUNTS: usize = 32;
    const ACCOUNTS_ACCESSED: usize = 8;
    const ITERATIONS: usize = 1_000;

    // Create mock accounts
    let mut accounts = Vec::new();
    for i in 0..TOTAL_ACCOUNTS {
        let owner = Pubkey::from([i as u8; 32]);
        let account =
            account_info_with_flags(owner, 1000u64, vec![42u8; 100], owner, false, true, false);
        accounts.push(account);
    }

    // Test lazy validation
    let start = Instant::now();
    let validator = LazyAccountValidator::new(TOTAL_ACCOUNTS);

    for _ in 0..ITERATIONS {
        // Simulate accessing only the first few accounts
        for i in 0..ACCOUNTS_ACCESSED {
            let _ = validator.ensure_validated(i as u8, &accounts);
        }
    }

    let lazy_duration = start.elapsed();
    let stats = ValidationStats::calculate(&validator);

    println!("Lazy Validation Performance Test:");
    println!("  Total accounts: {}", TOTAL_ACCOUNTS);
    println!("  Accounts accessed: {}", ACCOUNTS_ACCESSED);
    println!("  Iterations: {}", ITERATIONS);
    println!("  Duration: {:?}", lazy_duration);
    println!("  Validation hit rate: {:.1}%", stats.lazy_hit_rate);
    println!(
        "  Validated: {} / {} accounts",
        stats.validated_accounts, stats.total_accounts
    );

    // Verify correctness
    assert_eq!(stats.total_accounts, TOTAL_ACCOUNTS as u8);
    assert_eq!(stats.validated_accounts, ACCOUNTS_ACCESSED as u8);
    assert_eq!(
        stats.lazy_hit_rate,
        (ACCOUNTS_ACCESSED as f32 / TOTAL_ACCOUNTS as f32) * 100.0
    );
}

/// Test bitwise constraint validation performance
#[test]
fn test_bitwise_constraint_performance() {
    const ITERATIONS: usize = 10_000;

    let validator = LazyAccountValidator::new(4);

    // Create minimal test accounts
    let owner = Pubkey::from([1u8; 32]);
    let account1 =
        account_info_with_flags(owner, 1000u64, vec![1u8; 32], owner, false, true, false);

    let account2 = account_info_with_flags(owner, 2000u64, vec![2u8; 64], owner, false, true, true);

    let accounts = [account1, account2];

    // Benchmark bitwise constraint validation
    let constraints = 2u64; // Require 2 accounts minimum

    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let result = validator.validate_constraints_bitwise(constraints, &accounts);
        assert!(result.is_ok());
    }
    let duration = start.elapsed();

    println!("Bitwise Constraint Validation Performance:");
    println!("  Iterations: {}", ITERATIONS);
    println!("  Duration: {:?}", duration);
    println!(
        "  Average per validation: {:?}",
        duration / (ITERATIONS as u32)
    );

    // Performance should be sub-microsecond for simple constraints
    let avg_nanos = duration.as_nanos() / ITERATIONS as u128;
    println!("  Average: {} nanoseconds", avg_nanos);
    assert!(
        avg_nanos < 10_000,
        "Constraint validation should be very fast"
    );
}

/// Test account access patterns that demonstrate lazy loading benefits
#[test]
fn test_real_world_access_patterns() {
    const TOTAL_ACCOUNTS: usize = 16;

    // Create accounts with varying sizes to simulate real usage
    let mut accounts = Vec::new();
    for i in 0..TOTAL_ACCOUNTS {
        let owner = Pubkey::from([i as u8; 32]);
        let data_size = match i % 4 {
            0 => 0,
            1 => 32,
            2 => 165,
            _ => 1024,
        };
        let account = account_info_with_flags(
            owner,
            (i + 1) as u64 * 1000,
            vec![i as u8; data_size],
            owner,
            false,
            true,
            i == 0,
        );
        accounts.push(account);
    }

    let _validator = LazyAccountValidator::new(TOTAL_ACCOUNTS);

    // Simulate typical transaction patterns:
    // 1. Access program account (0)
    // 2. Access user accounts (1, 2)
    // 3. Maybe access additional accounts (3, 4)
    let access_patterns = [
        vec![0],          // Program only
        vec![0, 1],       // Program + user account
        vec![0, 1, 2],    // Program + multiple accounts
        vec![0, 1, 2, 3], // Program + several accounts
    ];

    for (pattern_idx, pattern) in access_patterns.iter().enumerate() {
        let validator = LazyAccountValidator::new(TOTAL_ACCOUNTS);

        for &account_idx in pattern {
            let result = validator.ensure_validated(account_idx, &accounts);
            assert!(
                result.is_ok(),
                "Validation should succeed for account {}",
                account_idx
            );
        }

        let stats = ValidationStats::calculate(&validator);
        let expected_hit_rate = (pattern.len() as f32 / TOTAL_ACCOUNTS as f32) * 100.0;

        println!("Access Pattern {}: {:?}", pattern_idx + 1, pattern);
        println!(
            "  Validated: {} / {} accounts",
            stats.validated_accounts, stats.total_accounts
        );
        println!(
            "  Hit rate: {:.1}% (expected: {:.1}%)",
            stats.lazy_hit_rate, expected_hit_rate
        );

        assert_eq!(stats.validated_accounts, pattern.len() as u8);
        assert!((stats.lazy_hit_rate - expected_hit_rate).abs() < 0.1);
    }
}

/// Integration test showing ExecutionContext with lazy validation
#[test]
fn test_execution_context_lazy_validation() {
    // Create test accounts
    let owner = Pubkey::from([1u8; 32]);
    let account =
        account_info_with_flags(owner, 1000u64, vec![42u8; 100], owner, false, true, false);

    let accounts = [account];
    let bytecode = &[0x07]; // RETURN opcode
    let program_id = Pubkey::from([2u8; 32]);

    // Create ExecutionContext with lazy validation
    let mut storage = StackStorage::new();
    let ctx = ExecutionContext::new(bytecode, &accounts, program_id, &[], 0, &mut storage, 1, 1, 0, 0, 0, 0);

    // Verify lazy validator is initialized
    assert_eq!(ctx.validated_account_count(), 0);
    assert_eq!(ctx.validation_stats().total_accounts, 1);
    assert_eq!(ctx.validation_stats().validated_accounts, 0);
    assert_eq!(ctx.validation_stats().lazy_hit_rate, 0.0);

    // Access an account (this should trigger lazy validation)
    let result = ctx.get_account(0);
    assert!(result.is_ok());

    // Check that validation occurred
    assert_eq!(ctx.validated_account_count(), 1);
    assert!(ctx.is_account_validated(0));
    assert_eq!(ctx.validation_stats().lazy_hit_rate, 100.0);

    println!("ExecutionContext lazy validation test passed");
    println!("  Final stats: {:?}", ctx.validation_stats());
}
