// Five VM program.

use pinocchio::{
    account_info::AccountInfo, default_allocator, default_panic_handler, program_entrypoint,
    program_error::ProgramError, pubkey::Pubkey, ProgramResult,
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

mod common;
mod error;
mod generated_constants;
pub use error::FIVEError;
pub mod instructions;
pub mod state;
pub mod upgrade;

#[cfg(all(feature = "production", feature = "cu-bypass-fees"))]
compile_error!("feature `cu-bypass-fees` must never be enabled with `production`");

use instructions::FIVEInstruction;

const MAX_ACCOUNTS: usize = (u8::MAX - 1) as usize;

pub fn hardcoded_program_id() -> Pubkey {
    Pubkey::from(generated_constants::VM_PROGRAM_ID_BYTES)
}

program_entrypoint!(process_instruction, MAX_ACCOUNTS);
default_allocator!();
default_panic_handler!();

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    #[cfg(feature = "debug-logs")]
    pinocchio::log::sol_log("FIVE VM: PROCESS_INSTRUCTION START");

    if instruction_data.is_empty() {
        debug_log!("Error: Empty instruction data");
        return Err(ProgramError::InvalidInstructionData);
    }

    debug_log!("Program ID");
    debug_log!("Accounts provided: {}", accounts.len());

    debug_log!("Instruction data length: {}", instruction_data.len());
    debug_log!("Instruction discriminator: {}", instruction_data[0]);

    if instruction_data[0] == instructions::EXECUTE_INSTRUCTION {
        #[cfg(feature = "debug-logs")]
        {
            pinocchio::log::sol_log("FIVE VM: EXECUTE START");
            pinocchio::log::sol_log_64(0, 0, 0, 0, instruction_data.len() as u64 - 1);
            pinocchio::log::sol_log_64(0, 0, 0, 0, accounts.len() as u64);
        }
        return instructions::execute(program_id, accounts, &instruction_data[1..]);
    }

    process_administrative_instruction(program_id, accounts, instruction_data)
}

/// Cold path for administrative and deployment instructions.
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
        FIVEInstruction::InitLargeProgram {
            expected_size,
            chunk_data,
        } => {
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
            deploy_fee_lamports,
            execute_fee_lamports,
        } => {
            debug_log!(
                "Processing SetFees instruction: deploy={} lamports, execute={} lamports",
                deploy_fee_lamports,
                execute_fee_lamports
            );
            instructions::set_fees(
                program_id,
                accounts,
                deploy_fee_lamports,
                execute_fee_lamports,
            )
        }
        FIVEInstruction::InitFeeVault { shard_index, bump } => {
            debug_log!("Processing InitFeeVault instruction");
            instructions::init_fee_vault(program_id, accounts, shard_index, bump)
        }
        FIVEInstruction::WithdrawScriptFees {
            script,
            shard_index,
            lamports,
        } => {
            debug_log!("Processing WithdrawScriptFees instruction");
            instructions::withdraw_script_fees(
                program_id,
                accounts,
                Pubkey::from(script),
                shard_index,
                lamports,
            )
        }
        FIVEInstruction::Deploy {
            bytecode,
            metadata,
            permissions,
            fee_shard_index,
        } => {
            debug_log!(
                "Processing Deploy instruction with {} bytes of bytecode, permissions: 0x{}",
                bytecode.len(),
                permissions
            );
            instructions::deploy(
                program_id,
                accounts,
                bytecode,
                metadata,
                permissions,
                fee_shard_index,
            )
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
