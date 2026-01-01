// FIVE VM implementation using Pinocchio for maximum performance
//
// This implementation leverages Pinocchio's zero-copy deserialization
// and direct memory access for optimal compute unit usage.

use pinocchio::{
    // program_entrypoint,
    account_info::AccountInfo,
    entrypoint,
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};

// Conditional logging macro for debug/error levels
#[macro_export]
macro_rules! log_if_debug {
    ($level:ident, $fmt:literal $(, $arg:expr)*) => {
        #[cfg(feature = "debug-logs")]
        {
            // pinocchio_log requires literal format strings; callers pass literals.
            pinocchio_log::log!($fmt $(, $arg)*);
        }
    };
}

// Temporarily disabled legacy tests during refactoring
#[cfg(test)]
mod tests;

// #[cfg(test)]
// mod tests_extended;

// #[cfg(test)]
// mod tests_integration;
#[cfg(test)]
mod tests_process_instruction;

// Test utilities for all tests
#[cfg(test)]
pub mod test_utils;

// DSL compiler functionality removed from onchain program
mod common;
mod error;
pub use error::FIVEError;
mod instructions;
mod state;

// #[cfg(test)]
// mod large_program_tests;

// #[cfg(test)]
// mod test_polymorphic_integration;

#[cfg(test)]
mod test_script_header_v3;

#[cfg(test)]
mod test_call_external;

#[cfg(test)]
mod test_call_external_public_functions;

#[cfg(test)]
mod test_call_external_constraints;

#[cfg(test)]
mod test_deploy_verification;

use instructions::FIVEInstruction;

// Use the optimized Pinocchio entrypoint (no allocator) for minimal CU
//program_entrypoint!(process_instruction);
entrypoint!(process_instruction); // Basic entrypoint (includes allocator/panic handler)

/// Optimized program entrypoint with no allocator (zero heap allocations)
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    log_if_debug!(
        debug,
        "FIVE Optimized: Processing instruction with no_allocator"
    );

    // Validate we have instruction data
    if instruction_data.is_empty() {
        log_if_debug!(error, "Error: Empty instruction data");
        return Err(ProgramError::InvalidInstructionData);
    }

    // Log instruction details for debugging
    log_if_debug!(debug, "Program ID: {}", program_id);
    log_if_debug!(debug, "Accounts provided: {}", accounts.len());
    log_if_debug!(debug, "Instruction data length: {}", instruction_data.len());
    log_if_debug!(debug, "Instruction discriminator: {}", instruction_data[0]);

    // Deserialize instruction using zero-copy deserialization
    let instruction = match FIVEInstruction::try_from(instruction_data) {
        Ok(ix) => ix,
        Err(e) => {
            log_if_debug!(error, "Failed to deserialize instruction");
            return Err(e);
        }
    };

    // Process each instruction type
    let result = match instruction {
        FIVEInstruction::Initialize => {
            log_if_debug!(debug, "Processing Initialize instruction");
            instructions::initialize(program_id, accounts)
        }
        FIVEInstruction::InitLargeProgram { expected_size, chunk_data } => {
            log_if_debug!(
                debug,
                "Processing InitLargeProgram instruction (expected size {}, chunk {})",
                expected_size,
                chunk_data.map(|c| c.len()).unwrap_or(0)
            );
            instructions::init_large_program(program_id, accounts, expected_size, chunk_data)
        }
        FIVEInstruction::AppendBytecode { data } => {
            log_if_debug!(
                debug,
                "Processing AppendBytecode instruction with {} bytes",
                data.len()
            );
            instructions::append_bytecode(program_id, accounts, data)
        }
        FIVEInstruction::SetFees {
            deploy_fee_bps,
            execute_fee_bps,
        } => {
            log_if_debug!(
                debug,
                "Processing SetFees instruction: deploy={} bps, execute={} bps",
                deploy_fee_bps,
                execute_fee_bps
            );
            instructions::set_fees(program_id, accounts, deploy_fee_bps, execute_fee_bps)
        }
        FIVEInstruction::Deploy { bytecode, permissions } => {
            log_if_debug!(
                debug,
                "Processing Deploy instruction with {} bytes of bytecode, permissions: 0x{}",
                bytecode.len(),
                permissions
            );
            instructions::deploy(program_id, accounts, bytecode, permissions)
        }
        FIVEInstruction::Execute { params } => {
            log_if_debug!(
                debug,
                "Processing Execute instruction with {} bytes of params",
                params.len()
            );
            instructions::execute(program_id, accounts, params)
        }
        FIVEInstruction::FinalizeScript => {
            log_if_debug!(debug, "Processing FinalizeScript instruction");
            instructions::finalize_script_upload(program_id, accounts)
        }
    };

    // Log result only in debug mode
    match &result {
        Ok(()) => {
            log_if_debug!(debug, "Instruction processed successfully");
        }
        Err(e) => {
            #[cfg(feature = "debug-logs")]
            {
                let code: u64 = (*e).into();
                log_if_debug!(error, "Instruction failed with error code: {}", code);
            }
            #[cfg(not(feature = "debug-logs"))]
            {
                let _ = e; // Suppress unused variable warning
            }
        }
    }

    result
}
