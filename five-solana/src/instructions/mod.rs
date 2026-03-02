pub mod deploy;
pub mod execute;
pub mod fees;
pub mod verify;

// Re-export functions to maintain compatibility with lib.rs usage
pub use deploy::{
    append_bytecode, deploy, finalize_script_upload, init_large_program, init_large_program_v2,
    initialize,
};
pub use execute::execute;
pub use fees::{init_fee_vault, set_fees, withdraw_script_fees};
pub use verify::verify_bytecode_content;

use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, sysvars::Sysvar, ProgramResult,
};

// Script deployment and execution instructions
pub const DEPLOY_INSTRUCTION: u8 = 8;
pub const EXECUTE_INSTRUCTION: u8 = 9;
pub const INIT_LARGE_PROGRAM_V2_INSTRUCTION: u8 = 13;

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
    Initialize {
        bump: u8,
    },
    InitLargeProgram {
        expected_size: u32,
        chunk_data: Option<&'a [u8]>,
    },
    InitLargeProgramV2 {
        bytecode_size: u32,
        metadata_len: u32,
        chunk_data: Option<&'a [u8]>,
    },
    AppendBytecode {
        data: &'a [u8],
    },
    SetFees {
        deploy_fee_lamports: u32,
        execute_fee_lamports: u32,
    },
    InitFeeVault {
        shard_index: u8,
        bump: u8,
    },
    WithdrawScriptFees {
        script: [u8; 32],
        shard_index: u8,
        lamports: u64,
    },
    Deploy {
        bytecode: &'a [u8],
        metadata: &'a [u8],
        permissions: u8,
        fee_shard_index: u8,
    },
    Execute {
        params: &'a [u8],
        fee_shard_index: u8,
    },
    FinalizeScript,
}

impl<'a> TryFrom<&'a [u8]> for FIVEInstruction<'a> {
    type Error = ProgramError;

    fn try_from(data: &'a [u8]) -> Result<Self, ProgramError> {
        if data.is_empty() {
            return Err(ProgramError::InvalidInstructionData);
        }

        match data[0] {
            0 => {
                if data.len() < 2 {
                    return Err(ProgramError::InvalidInstructionData);
                }
                Ok(FIVEInstruction::Initialize { bump: data[1] })
            }
            4 => {
                if data.len() < 5 {
                    return Err(ProgramError::InvalidInstructionData);
                }
                let expected_size = u32::from_le_bytes(data[1..5].try_into().unwrap());
                // Check if chunk data is present (InitLargeProgramWithChunk optimization)
                let chunk_data = if data.len() > 5 {
                    Some(&data[5..])
                } else {
                    None
                };
                Ok(FIVEInstruction::InitLargeProgram {
                    expected_size,
                    chunk_data,
                })
            }
            INIT_LARGE_PROGRAM_V2_INSTRUCTION => {
                if data.len() < 9 {
                    return Err(ProgramError::InvalidInstructionData);
                }
                let bytecode_size = u32::from_le_bytes(data[1..5].try_into().unwrap());
                let metadata_len = u32::from_le_bytes(data[5..9].try_into().unwrap());
                let chunk_data = if data.len() > 9 {
                    Some(&data[9..])
                } else {
                    None
                };
                Ok(FIVEInstruction::InitLargeProgramV2 {
                    bytecode_size,
                    metadata_len,
                    chunk_data,
                })
            }
            5 => {
                if data.len() < 2 {
                    return Err(ProgramError::InvalidInstructionData);
                }
                Ok(FIVEInstruction::AppendBytecode { data: &data[1..] })
            }
            6 => {
                if data.len() < 9 {
                    return Err(ProgramError::InvalidInstructionData);
                }
                let deploy_fee_lamports = u32::from_le_bytes(data[1..5].try_into().unwrap());
                let execute_fee_lamports = u32::from_le_bytes(data[5..9].try_into().unwrap());
                Ok(FIVEInstruction::SetFees {
                    deploy_fee_lamports,
                    execute_fee_lamports,
                })
            }
            11 => {
                if data.len() < 3 {
                    return Err(ProgramError::InvalidInstructionData);
                }
                Ok(FIVEInstruction::InitFeeVault {
                    shard_index: data[1],
                    bump: data[2],
                })
            }
            12 => {
                if data.len() < 42 {
                    return Err(ProgramError::InvalidInstructionData);
                }
                let mut script = [0u8; 32];
                script.copy_from_slice(&data[1..33]);
                let shard_index = data[33];
                let lamports = u64::from_le_bytes(data[34..42].try_into().unwrap());
                Ok(FIVEInstruction::WithdrawScriptFees {
                    script,
                    shard_index,
                    lamports,
                })
            }
            DEPLOY_INSTRUCTION => {
                if data.len() < crate::instructions::deploy::MIN_DEPLOY_LEN {
                    return Err(ProgramError::InvalidInstructionData);
                }
                let len = u32::from_le_bytes(data[1..5].try_into().unwrap()) as usize;
                let permissions = data[5];
                let metadata_len = u32::from_le_bytes(data[6..10].try_into().unwrap()) as usize;
                let total_len = crate::instructions::deploy::MIN_DEPLOY_LEN + metadata_len + len;
                if data.len() < total_len {
                    return Err(ProgramError::InvalidInstructionData);
                }
                let metadata_start = crate::instructions::deploy::MIN_DEPLOY_LEN;
                let metadata_end = metadata_start + metadata_len;
                let fee_shard_index = if data.len() >= total_len + 1 {
                    data[total_len]
                } else {
                    0u8
                };
                Ok(FIVEInstruction::Deploy {
                    metadata: &data[metadata_start..metadata_end],
                    bytecode: &data[metadata_end..total_len],
                    permissions,
                    fee_shard_index,
                })
            }
            EXECUTE_INSTRUCTION => {
                if data.len() >= 4 && data[1] == 0xFF && data[2] == 0x53 {
                    Ok(FIVEInstruction::Execute {
                        fee_shard_index: data[3],
                        params: &data[4..],
                    })
                } else {
                    Ok(FIVEInstruction::Execute {
                        params: &data[1..],
                        fee_shard_index: 0,
                    })
                }
            }
            7 => Ok(FIVEInstruction::FinalizeScript),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}
