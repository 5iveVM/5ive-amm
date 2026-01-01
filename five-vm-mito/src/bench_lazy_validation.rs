//! Performance benchmarks for lazy account validation
//!
//! This module provides benchmarks to measure the performance improvements
//! from lazy account validation compared to eager validation.

#![allow(dead_code)]

use crate::{
    error::{Result, VMError},
    lazy_validation::{LazyAccountValidator, ValidationStats},
};
use pinocchio::{account_info::AccountInfo, pubkey::Pubkey};
use std::time::Instant;

/// Benchmark configuration
#[derive(Debug, Clone)]
pub struct BenchConfig {
    pub total_accounts: usize,
    pub accounts_accessed: usize,
    pub iterations: usize,
}

impl Default for BenchConfig {
    fn default() -> Self {
        Self {
            total_accounts: 32,
            accounts_accessed: 8,
            iterations: 10_000,
        }
    }
}

/// Benchmark results
#[derive(Debug)]
pub struct BenchResult {
    pub config: BenchConfig,
    pub lazy_validation_ns: u128,
    pub eager_validation_ns: u128,
    pub speedup_factor: f64,
    pub validation_stats: ValidationStats,
}

/// Create mock accounts for benchmarking
fn create_mock_accounts(count: usize) -> Vec<AccountInfo> {
    let mut accounts = Vec::with_capacity(count);

    for i in 0..count {
        let key_value = Pubkey::from([i as u8; 32]);
        let lamports_value = 1000u64 + i as u64;
        let data_slice = vec![i as u8; 100].into_boxed_slice();

        let key_ref = Box::leak(Box::new(key_value));
        let lamports_ref = Box::leak(Box::new(lamports_value));
        let data_ref = Box::leak(data_slice);

        let account = AccountInfo::new(
            key_ref,
            false,
            true,
            lamports_ref,
            data_ref,
            key_ref,
            i % 4 == 0,
            0,
        );

        accounts.push(account);
    }

    accounts
}

/// Simulate eager validation (validates all accounts upfront)
fn eager_validation(accounts: &[AccountInfo]) -> Result<()> {
    for account in accounts {
        // Basic validation similar to what LazyAccountValidator does
        if account.data_len() == 0 && account.lamports() == 0 {
            return Err(VMError::UninitializedAccount);
        }
    }
    Ok(())
}

/// Simulate account access pattern (only access some accounts)
fn simulate_account_access(
    lazy_validator: &LazyAccountValidator,
    accounts: &[AccountInfo],
    accounts_to_access: &[usize],
) -> Result<()> {
    for &idx in accounts_to_access {
        if idx < accounts.len() {
            lazy_validator.ensure_validated(idx as u8, accounts)?;
        }
    }
    Ok(())
}

/// Run lazy validation benchmark
pub fn benchmark_lazy_validation(config: BenchConfig) -> BenchResult {
    let accounts = create_mock_accounts(config.total_accounts);

    // Pattern: access first N accounts (simulating typical usage)
    let access_pattern: Vec<usize> = (0..config.accounts_accessed).collect();

    // Benchmark lazy validation
    let lazy_start = Instant::now();
    for _ in 0..config.iterations {
        let lazy_validator = LazyAccountValidator::new(config.total_accounts);
        let _ = simulate_account_access(&lazy_validator, &accounts, &access_pattern);
    }
    let lazy_duration = lazy_start.elapsed();

    // Benchmark eager validation
    let eager_start = Instant::now();
    for _ in 0..config.iterations {
        let _ = eager_validation(&accounts);
    }
    let eager_duration = eager_start.elapsed();

    // Calculate results
    let lazy_ns = lazy_duration.as_nanos() / config.iterations as u128;
    let eager_ns = eager_duration.as_nanos() / config.iterations as u128;
    let speedup = eager_ns as f64 / lazy_ns as f64;

    // Get validation statistics
    let validator = LazyAccountValidator::new(config.total_accounts);
    let _ = simulate_account_access(&validator, &accounts, &access_pattern);
    let stats = ValidationStats::calculate(&validator);

    BenchResult {
        config,
        lazy_validation_ns: lazy_ns,
        eager_validation_ns: eager_ns,
        speedup_factor: speedup,
        validation_stats: stats,
    }
}

/// Run comprehensive benchmark suite
pub fn run_benchmark_suite() -> Vec<BenchResult> {
    let configs = vec![
        BenchConfig {
            total_accounts: 16,
            accounts_accessed: 4,
            iterations: 10_000,
        },
        BenchConfig {
            total_accounts: 32,
            accounts_accessed: 8,
            iterations: 10_000,
        },
        BenchConfig {
            total_accounts: 64,
            accounts_accessed: 16,
            iterations: 5_000,
        },
    ];

    configs.into_iter().map(benchmark_lazy_validation).collect()
}

/// Print benchmark results in a formatted table
pub fn print_results(results: &[BenchResult]) {
    println!("┌─────────────────────────────────────────────────────────────────┐");
    println!("│                    Lazy Validation Benchmarks                  │");
    println!("├─────────────────────────────────────────────────────────────────┤");
    println!("│ Total │ Accessed │  Lazy (ns) │  Eager (ns) │ Speedup │ Hit Rate │");
    println!("├─────────────────────────────────────────────────────────────────┤");

    for result in results {
        println!(
            "│ {:5} │    {:5} │ {:10} │ {:11} │ {:6.2}x │  {:6.1}% │",
            result.config.total_accounts,
            result.config.accounts_accessed,
            result.lazy_validation_ns,
            result.eager_validation_ns,
            result.speedup_factor,
            result.validation_stats.lazy_hit_rate,
        );
    }

    println!("└─────────────────────────────────────────────────────────────────┘");

    // Summary statistics
    let avg_speedup: f64 =
        results.iter().map(|r| r.speedup_factor).sum::<f64>() / results.len() as f64;
    let max_speedup = results
        .iter()
        .map(|r| r.speedup_factor)
        .fold(0.0f64, f64::max);

    println!("\nSummary:");
    println!("• Average speedup: {:.2}x", avg_speedup);
    println!("• Maximum speedup: {:.2}x", max_speedup);
    println!("• Lazy validation shows significant performance gains when many accounts");
    println!("  are provided but only a subset are actually accessed by the VM.");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_suite() {
        let results = run_benchmark_suite();
        assert!(!results.is_empty());

        for result in &results {
            assert!(result.lazy_validation_ns > 0);
            assert!(result.eager_validation_ns > 0);
            assert!(result.speedup_factor > 0.0);
            assert!(result.validation_stats.lazy_hit_rate >= 0.0);
            assert!(result.validation_stats.lazy_hit_rate <= 100.0);
        }

        print_results(&results);
    }

    #[test]
    fn test_single_benchmark() {
        let config = BenchConfig::default();
        let result = benchmark_lazy_validation(config);

        assert!(result.lazy_validation_ns > 0);
        assert!(result.eager_validation_ns > 0);
        assert!(result.speedup_factor > 0.0);

        println!("Single benchmark result: {:#?}", result);
    }
}
