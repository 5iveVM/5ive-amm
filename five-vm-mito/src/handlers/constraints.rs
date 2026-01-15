//! Constraint operations handler for MitoVM (0x70-0x7F)
//!
//! 🎯 LOGICAL REORGANIZATION: All constraint validation operations consolidated
//! This module handles constraint validation operations including CHECK_SIGNER,
//! CHECK_WRITABLE, CHECK_OWNER, CHECK_INITIALIZED, CHECK_PDA, CHECK_UNINITIALIZED.
//! It manages Solana account constraint validation for security.

use crate::{
    context::ExecutionManager,
    debug_log,
    error::{CompactResult, VMErrorCode},
};
use five_protocol::opcodes::*;
use pinocchio::pubkey::Pubkey;

// System program ID constant
const SYSTEM_PROGRAM_ID: [u8; 32] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
]; // Solana system program ID: 11111111111111111111111111111111

macro_rules! check_constraint {
    ($ctx:expr, $name:literal, $account:ident, $check:expr) => {{
        let account_idx = $ctx.fetch_byte()?;
        let $account = $ctx.get_account(account_idx)?;
        if !($check) {
             debug_log!("MitoVM: {} failed - account {} check failed", $name, account_idx);
             return Err(VMErrorCode::ConstraintViolation);
        }
        debug_log!("MitoVM: {} passed for account {}", $name, account_idx);
    }};
}

/// Handle constraint operations (0x70-0x7F)
#[inline(never)]
pub fn handle_constraints(opcode: u8, ctx: &mut ExecutionManager) -> CompactResult<()> {
    debug_log!("MitoVM: handle_constraints opcode: {}", opcode);
    debug_log!("MitoVM: CHECK_SIGNER constant: {}", CHECK_SIGNER);
    match opcode {
        // ===== CONSTRAINT VALIDATION OPERATIONS (0x70-0x7F) =====
        // Core account constraint checking operations
        CHECK_SIGNER => {
            check_constraint!(ctx, "CHECK_SIGNER", account, account.is_signer());
        }
        CHECK_WRITABLE => {
            check_constraint!(ctx, "CHECK_WRITABLE", account, account.is_writable());
        }
        CHECK_OWNER => {
            let account_idx = ctx.fetch_byte()?; // Fetch account_idx from bytecode
            let expected_owner_ref = ctx.fetch_pubkey_to_temp()?; // Fetch Pubkey from bytecode to temp buffer

            // Get account first and copy owner data
            let account = ctx.get_account(account_idx)?;
            let actual_owner_bytes = *account.owner();

            // Extract expected owner pubkey directly
            let expected_owner_bytes = ctx.get_temp_data(expected_owner_ref, 32)?;

            if actual_owner_bytes.as_ref() != expected_owner_bytes {
                return Err(VMErrorCode::ConstraintViolation);
            }
            debug_log!("MitoVM: CHECK_OWNER passed for account {}", account_idx);
        }
        CHECK_INITIALIZED => {
            // Check if account has data (is initialized)
            check_constraint!(ctx, "CHECK_INITIALIZED", account, account.data_len() > 0);
        }
        CHECK_UNINITIALIZED => {
            let account_idx = ctx.fetch_byte()?; // Fetch account_idx directly from bytecode
            let account = ctx.get_account(account_idx)?;

            // Account should be uninitialized (empty data) for @init
            // SAFETY: We only read the data slice; mutable borrows are ruled out by `ExecutionManager`.
            if !unsafe { account.borrow_data_unchecked() }.is_empty() {
                debug_log!("MitoVM: CHECK_UNINITIALIZED failed - data_len={} (expected 0)", account.data_len());
                return Err(VMErrorCode::ConstraintViolation);
            }

            // Also check that account owner is the System Program for new accounts
            let account_owner = *account.owner();
            if account_owner != Pubkey::from(SYSTEM_PROGRAM_ID) {
                debug_log!("MitoVM: CHECK_UNINITIALIZED failed - owner mismatch (expected SystemProgram)");
                return Err(VMErrorCode::ConstraintViolation);
            }

            debug_log!(
                "MitoVM: CHECK_UNINITIALIZED passed for account {}",
                account_idx
            );
        }
        CHECK_PDA => {
            debug_log!("MitoVM: CHECK_PDA - validating PDA from stack parameters");

            // Stack layout (top to bottom): expected_pda, program_id, seeds_count, seed1, seed2, ...
            let expected_pda_ref = ctx.pop()?;

            // Extract pubkeys directly
            let expected_pda_bytes = ctx.extract_pubkey(&expected_pda_ref)?;

            // Convert to Pinocchio Pubkeys
            use pinocchio::pubkey::Pubkey;
            let expected_pubkey = Pubkey::from(expected_pda_bytes);

            // Use shared helper to collect and process seeds (handles program_id and seeds)
            use crate::handlers::system::pda::with_pda_seeds;
            with_pda_seeds(ctx, |_, program_pubkey, seeds| {
                // Derive PDA using the same logic as DERIVE_PDA
                use pinocchio::pubkey::create_program_address;
                match create_program_address(seeds, &program_pubkey) {
                    Ok(derived_pda) => {
                        // Compare derived PDA with expected PDA
                        if derived_pda != expected_pubkey {
                            debug_log!("MitoVM: CHECK_PDA failed - PDA mismatch");
                            return Err(VMErrorCode::ConstraintViolation);
                        }
                        debug_log!("MitoVM: CHECK_PDA passed - PDA validation successful");
                        Ok(())
                    }
                    Err(_) => {
                        debug_log!("MitoVM: CHECK_PDA failed - PDA derivation error");
                        Err(VMErrorCode::ConstraintViolation)
                    }
                }
            })?;
        }

        _ => {
            debug_log!("MitoVM: Unknown constraint opcode: {}", opcode);
            return Err(VMErrorCode::InvalidInstruction);
        },
    }
    Ok(())
}
