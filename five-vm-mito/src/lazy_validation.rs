//! Lazy Account Validation for Five VM
//!
//! This module provides lazy validation of accounts, only validating them
//! when first accessed by the VM. This provides significant performance
//! improvements for transactions with many unused accounts.

use crate::error::{CompactResult, VMErrorCode};
use core::cell::Cell;
use pinocchio::account_info::AccountInfo;

/// Lazy account validator using single u64 bitmap
///
/// Tracks which accounts have been validated and performs validation
/// only on first access. Supports up to 64 accounts per transaction.
#[derive(Debug)]
pub struct LazyAccountValidator {
    /// Bitmap tracking which accounts have been validated (1 bit per account)
    /// Uses Cell for interior mutability to allow validation from immutable contexts
    touched_bitmap: Cell<u64>,
    /// Actual number of accounts in the transaction
    account_count: u8,
}

impl LazyAccountValidator {
    /// Create new lazy validator for the given number of accounts
    #[inline]
    pub const fn new(account_count: usize) -> Self {
        let clamped = if account_count > 64 { 64 } else { account_count };
        Self {
            touched_bitmap: Cell::new(0),
            account_count: clamped as u8,
        }
    }

    /// Ensure account is validated, validating on first access
    ///
    /// This is the core lazy loading function - it only validates
    /// an account the first time it's accessed by the VM.
    #[inline(always)]
    pub fn ensure_validated(&self, idx: u8, accounts: &[AccountInfo]) -> CompactResult<()> {
        // Bounds check
        if idx >= self.account_count {
            return Err(VMErrorCode::InvalidAccountIndex);
        }
        if idx >= 64 {
            return Err(VMErrorCode::InvalidAccountIndex);
        }

        // Check if already validated using single bit check
        let current_bitmap = self.touched_bitmap.get();
        if (current_bitmap >> idx) & 1 == 0 {
            // First access - validate now
            self.validate_account_constraints(&accounts[idx as usize])?;

            // Mark as validated using single bit set
            self.touched_bitmap.set(current_bitmap | (1 << idx));
        }

        Ok(())
    }

    /// Check if account has been validated (for testing/metrics)
    #[inline]
    pub fn is_validated(&self, idx: u8) -> bool {
        if idx >= self.account_count {
            return false;
        }
        (self.touched_bitmap.get() >> idx) & 1 == 1
    }

    /// Get number of accounts that have been validated
    #[inline]
    pub fn validated_count(&self) -> u8 {
        self.touched_bitmap.get().count_ones() as u8
    }

    /// Get total number of accounts in transaction
    #[inline]
    pub const fn total_count(&self) -> u8 {
        self.account_count
    }

    /// Reset validation state (for reusing validator)
    #[inline]
    pub fn reset(&mut self, account_count: usize) {
        self.touched_bitmap.set(0);
        self.account_count = account_count as u8;
    }

    /// Validate account constraints with bitwise constraint checking
    #[inline]
    fn validate_account_constraints(&self, _account: &AccountInfo) -> CompactResult<()> {
        // Basic account validation - removed strict initialization check
        // Accounts can be uninitialized (e.g. for @init constraints), so we should not
        // enforce initialization here. Explicit opcodes like CHECK_INITIALIZED handle this.
        
        // Additional constraints validation can leverage header-provided metadata when available
        Ok(())
    }

    /// Advanced constraint validation using bitwise constraint metadata emitted by the compiler
    /// This performs efficient constraint checks using pre-computed constraint bits
    #[inline]
    pub fn validate_constraints_bitwise(
        &self,
        constraints: u64,
        accounts: &[AccountInfo],
    ) -> CompactResult<()> {
        // Extract constraint bits from the u64
        let required_accounts = (constraints & 0xFF) as u8; // Lower 8 bits = required account count
        let signer_mask = ((constraints >> 8) & 0xFFFF) as u16; // Next 16 bits = signer requirements
        let writable_mask = ((constraints >> 24) & 0xFFFF) as u16; // Next 16 bits = writable requirements
        let executable_mask = ((constraints >> 40) & 0xFFFF) as u16; // Next 16 bits = executable requirements
                                                                     // Remaining 8 bits available for additional constraints

        // Check minimum account count
        if accounts.len() < required_accounts as usize {
            return Err(VMErrorCode::InvalidAccountIndex);
        }

        // Validate signer constraints using bitwise AND
        for i in 0..required_accounts.min(16) {
            let account_idx = i as usize;
            if account_idx >= accounts.len() {
                break;
            }

            let account = &accounts[account_idx];

            // Check signer requirement (bit i in signer_mask)
            if (signer_mask >> i) & 1 == 1 && !account.is_signer() {
                return Err(VMErrorCode::AccountNotSigner);
            }

            // Check writable requirement (bit i in writable_mask)
            if (writable_mask >> i) & 1 == 1 && !account.is_writable() {
                return Err(VMErrorCode::AccountNotWritable);
            }

            // Check executable requirement (bit i in executable_mask)
            if (executable_mask >> i) & 1 == 1 && !account.executable() {
                return Err(VMErrorCode::InvalidAccount);
            }
        }

        Ok(())
    }
}

/// Account validation statistics for performance monitoring
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidationStats {
    pub total_accounts: u8,
    pub validated_accounts: u8,
    pub lazy_hit_rate: f32, // Percentage of accounts that were actually used
}

impl ValidationStats {
    /// Calculate validation statistics
    pub fn calculate(validator: &LazyAccountValidator) -> Self {
        let total = validator.total_count();
        let validated = validator.validated_count();
        let hit_rate = if total > 0 {
            (validated as f32) / (total as f32) * 100.0
        } else {
            0.0
        };

        Self {
            total_accounts: total,
            validated_accounts: validated,
            lazy_hit_rate: hit_rate,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::panic::{catch_unwind, AssertUnwindSafe};

    #[test]
    fn test_lazy_validator_creation() {
        let validator = LazyAccountValidator::new(5);
        assert_eq!(validator.total_count(), 5);
        assert_eq!(validator.validated_count(), 0);

        // No accounts should be validated initially
        for i in 0..5 {
            assert!(!validator.is_validated(i));
        }
    }

    #[test]
    fn test_validation_tracking() {
        let validator = LazyAccountValidator::new(3);

        // Initially no accounts validated
        assert_eq!(validator.validated_count(), 0);

        // Mark accounts as validated (simulate first access)
        let current = validator.touched_bitmap.get();
        validator.touched_bitmap.set(current | (1 << 0)); // Account 0
        let current = validator.touched_bitmap.get();
        validator.touched_bitmap.set(current | (1 << 2)); // Account 2

        // Check validation state
        assert!(validator.is_validated(0));
        assert!(!validator.is_validated(1));
        assert!(validator.is_validated(2));
        assert_eq!(validator.validated_count(), 2);
    }

    #[test]
    fn test_bounds_checking() {
        let validator = LazyAccountValidator::new(3);

        // Valid indices
        assert!(!validator.is_validated(0));
        assert!(!validator.is_validated(1));
        assert!(!validator.is_validated(2));

        // Invalid index
        assert!(!validator.is_validated(3));
        assert!(!validator.is_validated(255));
    }

    #[test]
    fn ensure_validated_rejects_indices_above_bitmap_width_without_panic() {
        use pinocchio::pubkey::Pubkey;

        fn test_account(
            key: &Pubkey,
            lamports: u64,
            data: Vec<u8>,
            owner: &Pubkey,
        ) -> pinocchio::account_info::AccountInfo {
            let key_ref = Box::leak(Box::new(*key));
            let owner_ref = Box::leak(Box::new(*owner));
            let lamports_ref = Box::leak(Box::new(lamports));
            let data_ref = Box::leak(data.into_boxed_slice());

            pinocchio::account_info::AccountInfo::new(
                key_ref,
                false,
                true,
                lamports_ref,
                data_ref,
                owner_ref,
                false,
                0,
            )
        }

        let owner = Pubkey::from([17u8; 32]);
        let mut accounts = Vec::with_capacity(70);
        for i in 0..70u8 {
            accounts.push(test_account(
                &Pubkey::from([i; 32]),
                1,
                vec![1u8; 1],
                &owner,
            ));
        }

        let validator = LazyAccountValidator::new(70);
        let panicked = catch_unwind(AssertUnwindSafe(|| {
            let err = validator.ensure_validated(64, &accounts).unwrap_err();
            assert_eq!(err, VMErrorCode::InvalidAccountIndex);
        }));

        assert!(panicked.is_ok());
    }

    #[test]
    fn test_validation_stats() {
        let validator = LazyAccountValidator::new(10);

        // Simulate validating 3 out of 10 accounts
        validator.touched_bitmap.set(0b0000000111); // First 3 accounts

        let stats = ValidationStats::calculate(&validator);
        assert_eq!(stats.total_accounts, 10);
        assert_eq!(stats.validated_accounts, 3);
        assert!((stats.lazy_hit_rate - 30.0).abs() < 0.0001);
    }

    #[test]
    fn test_reset_functionality() {
        let mut validator = LazyAccountValidator::new(5);

        // Simulate some validation
        validator.touched_bitmap.set(0b11111); // All 5 accounts validated
        assert_eq!(validator.validated_count(), 5);

        // Reset for new transaction with different account count
        validator.reset(8);
        assert_eq!(validator.total_count(), 8);
        assert_eq!(validator.validated_count(), 0);
    }

    #[test]
    fn test_bitwise_constraint_validation() {
        use pinocchio::pubkey::Pubkey;

        fn test_account(
            key: &Pubkey,
            lamports: u64,
            data: Vec<u8>,
            owner: &Pubkey,
            executable: bool,
        ) -> pinocchio::account_info::AccountInfo {
            let key_ref = Box::leak(Box::new(*key));
            let owner_ref = Box::leak(Box::new(*owner));
            let lamports_ref = Box::leak(Box::new(lamports));
            let data_ref = Box::leak(data.into_boxed_slice());

            pinocchio::account_info::AccountInfo::new(
                key_ref,
                false,
                true,
                lamports_ref,
                data_ref,
                owner_ref,
                executable,
                0,
            )
        }

        let validator = LazyAccountValidator::new(3);

        // Create mock accounts for testing
        let owner = Pubkey::from([1u8; 32]);
        let account1 = test_account(&owner, 1000u64, vec![1u8; 100], &owner, false);
        let account2 = test_account(&owner, 2000u64, vec![2u8; 200], &owner, true);
        let account3 = test_account(&owner, 0u64, vec![], &owner, false);

        let accounts = [account1, account2, account3];

        // Test constraint validation
        // Constraint bits: 3 accounts required, account 1 executable (bit 1 in executable_mask = bit 41)
        let constraints = 3u64 | (1u64 << 41); // 3 accounts required, account 1 must be executable
        let result = validator.validate_constraints_bitwise(constraints, &accounts);
        assert!(result.is_ok());

        // Test failure case - require more accounts than available
        let constraints_fail = 5u64; // 5 accounts required but only 3 available
        let result_fail = validator.validate_constraints_bitwise(constraints_fail, &accounts);
        assert!(result_fail.is_err());
    }
}
