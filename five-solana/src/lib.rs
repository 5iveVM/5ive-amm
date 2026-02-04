
// FIVE VM implementation.

use pinocchio::{
    // program_entrypoint,
    account_info::AccountInfo,
    entrypoint,
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};

#[macro_export]
macro_rules! debug_log {
    ($fmt:literal $(, $arg:expr)*) => {
        #[cfg(feature = "debug-logs")]
        {
            // pinocchio_log requires literal format strings; callers pass literals.
            pinocchio_log::log!($fmt $(, $arg)*);
        }
    };
}

// VM Storage is now allocated on the stack in execute.rs
// This avoids the previous issue with const buffers in read-only memory
// Stack allocation is writable and has zero syscall overhead

mod common;
mod error;
pub use error::FIVEError;
pub mod instructions;
pub mod state;
pub mod upgrade;

use instructions::FIVEInstruction;

//program_entrypoint!(process_instruction);
entrypoint!(process_instruction); // Basic entrypoint (includes allocator/panic handler)

/// Program entrypoint.
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    #[cfg(feature = "debug-logs")]
    {
        pinocchio::log::sol_log("@@@ FIVE ENTRYPOINT REACHED @@@");
        pinocchio::log::sol_log("FIVE VM: PROCESS_INSTRUCTION START");
    }
    #[cfg(feature = "debug-logs")]
    pinocchio::log::sol_log("FORCE LOG ENTRY: FIVE VM ALIVE");

    #[cfg(feature = "debug-logs")]
    pinocchio::log::sol_log("@@@ UNCONDITIONAL LOG: FIVE VM ENTRY @@@");

    debug_log!(
        "FIVE Optimized: Processing instruction with no_allocator"
    );

    // Validate we have instruction data
    if instruction_data.is_empty() {
        debug_log!("Error: Empty instruction data");
        return Err(ProgramError::InvalidInstructionData);
    }

    // Log instruction details for debugging
    debug_log!("Program ID: {}", program_id);
    debug_log!("Accounts provided: {}", accounts.len());

    // Detailed account logging
    #[cfg(feature = "debug-logs")]
    for (i, account) in accounts.iter().enumerate() {
        let key_bytes = account.key().as_ref();
        let owner_bytes = account.owner().as_ref();
        debug_log!(
            "  Account {}: Key={} {} Owner={} {} DataLen={} Writable={} Signer={}",
            i,
            key_bytes[0], key_bytes[1],
            owner_bytes[0], owner_bytes[1],
            account.data_len(),
            if account.is_writable() { 1 } else { 0 },
            if account.is_signer() { 1 } else { 0 }
        );
    }

    debug_log!("Instruction data length: {}", instruction_data.len());
    debug_log!("Instruction discriminator: {}", instruction_data[0]);

    // 🎯 OPTIMIZATION: Hot path restructuring
    // Handle EXECUTE instruction (9) immediately to maximize branch predictor efficiency
    if instruction_data[0] == instructions::EXECUTE_INSTRUCTION {
        #[cfg(feature = "debug-logs")]
        {
            pinocchio::log::sol_log("FIVE VM: EXECUTE START");
            pinocchio::log::sol_log_64(0, 0, 0, 0, instruction_data.len() as u64 - 1);
            pinocchio::log::sol_log_64(0, 0, 0, 0, accounts.len() as u64);
        }
        return instructions::execute(program_id, accounts, &instruction_data[1..]);
    }

    // Handle administrative and deployment instructions (cold path)
    process_administrative_instruction(program_id, accounts, instruction_data)
}

/// Cold path for administrative and deployment instructions.
/// Separated to keep the entrypoint hot path clean and cache-friendly.
#[inline(never)]
fn process_administrative_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // Deserialize instruction using zero-copy deserialization
    let instruction = match FIVEInstruction::try_from(instruction_data) {
        Ok(ix) => ix,
        Err(e) => {
            debug_log!("Failed to deserialize instruction");
            return Err(e);
        }
    };

    // Process each instruction type
    let result = match instruction {
        FIVEInstruction::Initialize { bump } => {
            debug_log!("Processing Initialize instruction with bump {}", bump);
            instructions::initialize(program_id, accounts, bump)
        }
        FIVEInstruction::InitLargeProgram { expected_size, chunk_data } => {
            debug_log!(
                "Processing InitLargeProgram instruction (expected size {}, chunk {})",
                expected_size,
                chunk_data.map(|c| c.len()).unwrap_or(0)
            );
            instructions::init_large_program(program_id, accounts, expected_size, chunk_data)
        }
        FIVEInstruction::AppendBytecode { data } => {
            debug_log!(
                "Processing AppendBytecode instruction with {} bytes",
                data.len()
            );
            instructions::append_bytecode(program_id, accounts, data)
        }
        FIVEInstruction::SetFees {
            deploy_fee_bps,
            execute_fee_bps,
        } => {
            debug_log!(
                "Processing SetFees instruction: deploy={} bps, execute={} bps",
                deploy_fee_bps,
                execute_fee_bps
            );
            instructions::set_fees(program_id, accounts, deploy_fee_bps, execute_fee_bps)
        }
        FIVEInstruction::Deploy { bytecode, permissions } => {
            debug_log!(
                "Processing Deploy instruction with {} bytes of bytecode, permissions: 0x{}",
                bytecode.len(),
                permissions
            );
            instructions::deploy(program_id, accounts, bytecode, permissions)
        }
        FIVEInstruction::Execute { .. } => {
            // Already handled in hot path
            unreachable!()
        }
        FIVEInstruction::FinalizeScript => {
            debug_log!("Processing FinalizeScript instruction");
            instructions::finalize_script_upload(program_id, accounts)
        }
    };

    // Log result only in debug mode
    match &result {
        Ok(()) => {
            debug_log!("Instruction processed successfully");
        }
        Err(e) => {
            #[cfg(feature = "debug-logs")]
            {
                let code: u64 = (*e).into();
                debug_log!("Instruction failed with error code: {}", code);
            }
            #[cfg(not(feature = "debug-logs"))]
            {
                let _ = e; // Suppress unused variable warning
            }
        }
    }

    result
}
