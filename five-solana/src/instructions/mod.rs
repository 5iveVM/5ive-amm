pub mod deploy;
pub mod execute;
pub mod fees;
pub mod verify;

// Re-export functions to maintain compatibility with lib.rs usage
pub use deploy::{deploy, initialize, init_large_program, append_bytecode, finalize_script_upload};
pub use execute::execute;
pub use fees::{set_fees, STANDARD_TX_FEE};
pub use verify::verify_bytecode_content;

use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, ProgramResult, sysvars::Sysvar,
};

use crate::debug_log;

// Script deployment and execution instructions
pub const DEPLOY_INSTRUCTION: u8 = 8;
pub const EXECUTE_INSTRUCTION: u8 = 9;

/// Ensure the required number of accounts are present
#[inline(always)]
pub fn require_min_accounts(accounts: &[AccountInfo], min: usize) -> ProgramResult {
    if accounts.len() < min {
        return Err(ProgramError::NotEnoughAccountKeys);
    }
    Ok(())
}

/// Ensure an account is a signer
#[inline(always)]
pub fn require_signer(account: &AccountInfo) -> ProgramResult {
    if !account.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }
    Ok(())
}

/// Helper to safely reallocate an account.
#[allow(dead_code)]
pub fn safe_realloc(account: &AccountInfo, payer: &AccountInfo, new_size: usize) -> ProgramResult {
    let required_lamports = pinocchio::sysvars::rent::Rent::get()
        .map_err(|_| ProgramError::AccountNotRentExempt)?
        .minimum_balance(new_size);

    let current_lamports = account.lamports();
    if current_lamports < required_lamports {
        let additional = required_lamports - current_lamports;
        if payer.lamports() < additional {
            return Err(ProgramError::InsufficientFunds);
        }
        *payer.try_borrow_mut_lamports()? -= additional;
        *account.try_borrow_mut_lamports()? += additional;
    }

    let old_len = account.data_len();
    account.resize(new_size)?; // runtime zeroes the added region
    if new_size > old_len {
        let mut data = account.try_borrow_mut_data()?;
        data[old_len..].fill(0); // explicitly zero for deterministic security
    }
    Ok(())
}

/// Instruction enum
pub enum FIVEInstruction<'a> {
    Initialize { bump: u8 },
    InitLargeProgram { expected_size: u32, chunk_data: Option<&'a [u8]> },
    AppendBytecode { data: &'a [u8] },
    SetFees { deploy_fee_bps: u32, execute_fee_bps: u32 },
    Deploy { bytecode: &'a [u8], permissions: u8 },
    Execute { params: &'a [u8] },
    FinalizeScript,
}

impl<'a> TryFrom<&'a [u8]> for FIVEInstruction<'a> {
    type Error = ProgramError;

    fn try_from(data: &'a [u8]) -> Result<Self, ProgramError> {
        debug_log!("FIVEInstruction::try_from - data length: {}", data.len());

        if data.is_empty() {
            debug_log!("FIVEInstruction::try_from - data is empty");
            return Err(ProgramError::InvalidInstructionData);
        }

        debug_log!("FIVEInstruction::try_from - discriminator: {}", data[0]);

        match data[0] {
            0 => {
                debug_log!("FIVEInstruction::try_from - Initialize instruction");
                if data.len() < 2 {
                    debug_log!("FIVEInstruction::try_from - Initialize: data too short (no bump)");
                    return Err(ProgramError::InvalidInstructionData);
                }
                Ok(FIVEInstruction::Initialize { bump: data[1] })
            }
            4 => {
                debug_log!("FIVEInstruction::try_from - InitLargeProgram instruction");
                if data.len() < 5 {
                    debug_log!("FIVEInstruction::try_from - InitLargeProgram: data too short");
                    return Err(ProgramError::InvalidInstructionData);
                }
                let expected_size = u32::from_le_bytes(data[1..5].try_into().unwrap());
                // Check if chunk data is present (InitLargeProgramWithChunk optimization)
                let chunk_data = if data.len() > 5 { Some(&data[5..]) } else { None };
                if let Some(chunk) = chunk_data {
                    #[cfg(feature = "debug-logs")]
                    debug_log!("InitLargeProgram with {} byte first chunk", chunk.len());
                    #[cfg(not(feature = "debug-logs"))]
                    let _ = chunk;
                }
                Ok(FIVEInstruction::InitLargeProgram { expected_size, chunk_data })
            }
            5 => {
                debug_log!("FIVEInstruction::try_from - AppendBytecode instruction");
                if data.len() < 2 {
                    debug_log!("FIVEInstruction::try_from - AppendBytecode: data too short");
                    return Err(ProgramError::InvalidInstructionData);
                }
                Ok(FIVEInstruction::AppendBytecode { data: &data[1..] })
            }
            6 => {
                debug_log!("FIVEInstruction::try_from - SetFees instruction");
                if data.len() < 9 {
                    debug_log!("FIVEInstruction::try_from - SetFees: data too short");
                    return Err(ProgramError::InvalidInstructionData);
                }
                let deploy_fee_bps = u32::from_le_bytes(data[1..5].try_into().unwrap());
                let execute_fee_bps = u32::from_le_bytes(data[5..9].try_into().unwrap());
                Ok(FIVEInstruction::SetFees { deploy_fee_bps, execute_fee_bps })
            }
            DEPLOY_INSTRUCTION => {
                debug_log!("FIVEInstruction::try_from - Deploy instruction (8)");
                if data.len() < crate::instructions::deploy::MIN_DEPLOY_LEN {
                    debug_log!("FIVEInstruction::try_from - Deploy: data too short ({}< {})", data.len(), crate::instructions::deploy::MIN_DEPLOY_LEN);
                    return Err(ProgramError::InvalidInstructionData);
                }
                let len = u32::from_le_bytes(data[1..5].try_into().unwrap()) as usize;
                let permissions = data[5];
                debug_log!("FIVEInstruction::try_from - Deploy: bytecode length: {}, permissions: 0x{}", len, permissions);
                let total_len = crate::instructions::deploy::MIN_DEPLOY_LEN + len;
                debug_log!("FIVEInstruction::try_from - Deploy: total expected: {}, actual: {}", total_len, data.len());
                if data.len() < total_len {
                    debug_log!("FIVEInstruction::try_from - Deploy: not enough data");
                    return Err(ProgramError::InvalidInstructionData);
                }
                Ok(FIVEInstruction::Deploy {
                    bytecode: &data[6..total_len],
                    permissions,
                })
            }
            EXECUTE_INSTRUCTION => {
                debug_log!("FIVEInstruction::try_from - Execute instruction (9)");
                Ok(FIVEInstruction::Execute { params: &data[1..] })
            }
            7 => {
                debug_log!("FIVEInstruction::try_from - FinalizeScript instruction");
                Ok(FIVEInstruction::FinalizeScript)
            }
            _ => {
                debug_log!("FIVEInstruction::try_from - Unknown discriminator: {}", data[0]);
                Err(ProgramError::InvalidInstructionData)
            }
        }
    }
}
