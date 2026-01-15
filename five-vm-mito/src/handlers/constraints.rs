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
    handlers::system::pda::pop_and_process_seeds,
    // Import stack operation macros
    pop_u8,
};
use five_protocol::opcodes::*;
use pinocchio::pubkey::Pubkey;

// System program ID constant
const SYSTEM_PROGRAM_ID: [u8; 32] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
]; // Solana system program ID: 11111111111111111111111111111111

/// Handle constraint operations (0x70-0x7F)
#[inline(never)]
pub fn handle_constraints(opcode: u8, ctx: &mut ExecutionManager) -> CompactResult<()> {
    debug_log!("MitoVM: handle_constraints opcode: {}", opcode);
    debug_log!("MitoVM: CHECK_SIGNER constant: {}", CHECK_SIGNER);
    match opcode {
        // ===== CONSTRAINT VALIDATION OPERATIONS (0x70-0x7F) =====
        // Core account constraint checking operations
        CHECK_SIGNER => {
            let account_idx = ctx.fetch_byte()?; // Fetch account_idx directly from bytecode
            debug_log!("MitoVM: CHECK_SIGNER checking account {}", account_idx);
            let account = ctx.get_account(account_idx)?;
            if !account.is_signer() {
                debug_log!("MitoVM: CHECK_SIGNER failed - account {} is not signer", account_idx);
                return Err(VMErrorCode::ConstraintViolation);
            }
            debug_log!("MitoVM: CHECK_SIGNER passed for account {}", account_idx);
        }
        CHECK_WRITABLE => {
            let account_idx = ctx.fetch_byte()?; // Fetch account_idx directly from bytecode
            let account = ctx.get_account(account_idx)?;
            if !account.is_writable() {
                return Err(VMErrorCode::ConstraintViolation);
            }
            debug_log!("MitoVM: CHECK_WRITABLE passed for account {}", account_idx);
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
            let account_idx = ctx.fetch_byte()?; // Fetch account_idx directly from bytecode
            let account = ctx.get_account(account_idx)?;
            // SAFETY: We only read the account data to verify initialization state and
            // `ExecutionManager` ensures no concurrent mutable borrows.
            if unsafe { account.borrow_data_unchecked() }.is_empty() {
                return Err(VMErrorCode::ConstraintViolation);
            }
            debug_log!(
                "MitoVM: CHECK_INITIALIZED passed for account {}",
                account_idx
            );
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
            let program_id_ref = ctx.pop()?;
            let seeds_count = pop_u8!(ctx);

            debug_log!("MitoVM: CHECK_PDA seeds_count: {}", seeds_count);

            // Extract pubkeys directly
            let expected_pda_bytes = ctx.extract_pubkey(&expected_pda_ref)?;
            let program_id_bytes = ctx.extract_pubkey(&program_id_ref)?;

            // Convert to Pinocchio Pubkeys
            use pinocchio::pubkey::Pubkey;
            let expected_pubkey = Pubkey::from(expected_pda_bytes);
            let program_pubkey = Pubkey::from(program_id_bytes);

            // Stack-allocated seed storage (same as PDA operations)
            const MAX_SEEDS: usize = 8;
            let mut seeds: [[u8; 32]; MAX_SEEDS] = [[0; 32]; MAX_SEEDS];
            let mut seed_lens: [usize; MAX_SEEDS] = [0; MAX_SEEDS];

            // Use shared helper to collect and process seeds
            pop_and_process_seeds(ctx, seeds_count, &mut seeds, &mut seed_lens)?;

            // Create stack-based seed reference array
            let mut seed_refs: [&[u8]; MAX_SEEDS] = [&[]; MAX_SEEDS];
            for i in 0..seeds_count as usize {
                seed_refs[i] = &seeds[i][..seed_lens[i]];
            }

            // Derive PDA using the same logic as DERIVE_PDA
            use pinocchio::pubkey::create_program_address;
            match create_program_address(&seed_refs[..seeds_count as usize], &program_pubkey) {
                Ok(derived_pda) => {
                    // Compare derived PDA with expected PDA
                    if derived_pda != expected_pubkey {
                        debug_log!("MitoVM: CHECK_PDA failed - PDA mismatch");
                        return Err(VMErrorCode::ConstraintViolation);
                    }
                    debug_log!("MitoVM: CHECK_PDA passed - PDA validation successful");
                }
                Err(_) => {
                    debug_log!("MitoVM: CHECK_PDA failed - PDA derivation error");
                    return Err(VMErrorCode::ConstraintViolation);
                }
            }
        }

        _ => {
            debug_log!("MitoVM: Unknown constraint opcode: {}", opcode);
            return Err(VMErrorCode::InvalidInstruction);
        },
    }
    Ok(())
}
