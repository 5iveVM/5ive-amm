use five::instructions::{DEPLOY_INSTRUCTION, EXECUTE_INSTRUCTION};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program,
};

use super::addresses::{canonical_execute_fee_header, fee_vault_shard0_pda};

pub fn canonical_deploy_instruction(
    program_id: Pubkey,
    script: Pubkey,
    vm_state: Pubkey,
    owner_signer: Pubkey,
    bytecode: &[u8],
    permissions: u8,
    metadata: &[u8],
    fee_shard_index: Option<u8>,
) -> Instruction {
    let (fee_vault, _bump) = fee_vault_shard0_pda(&program_id);
    let mut data = Vec::with_capacity(10 + metadata.len() + bytecode.len() + 1);
    data.push(DEPLOY_INSTRUCTION);
    data.extend_from_slice(&(bytecode.len() as u32).to_le_bytes());
    data.push(permissions);
    data.extend_from_slice(&(metadata.len() as u32).to_le_bytes());
    data.extend_from_slice(metadata);
    data.extend_from_slice(bytecode);
    if let Some(shard) = fee_shard_index {
        data.push(shard);
    }

    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(script, false),
            AccountMeta::new(vm_state, false),
            AccountMeta::new(owner_signer, true),
            AccountMeta::new(fee_vault, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data,
    }
}

pub fn canonical_execute_instruction(
    program_id: Pubkey,
    script: Pubkey,
    vm_state: Pubkey,
    payer_signer_writable: Pubkey,
    payload: &[u8],
    fee_shard_index: Option<u8>,
) -> Instruction {
    let (fee_vault, _bump) = fee_vault_shard0_pda(&program_id);
    canonical_execute_instruction_with_fee_vault(
        program_id,
        script,
        vm_state,
        payer_signer_writable,
        fee_vault,
        payload,
        fee_shard_index,
    )
}

pub fn canonical_execute_instruction_with_fee_vault(
    program_id: Pubkey,
    script: Pubkey,
    vm_state: Pubkey,
    payer_signer_writable: Pubkey,
    fee_vault: Pubkey,
    payload: &[u8],
    fee_shard_index: Option<u8>,
) -> Instruction {
    let mut data = Vec::with_capacity(1 + payload.len() + 3);
    data.push(EXECUTE_INSTRUCTION);
    if let Some(shard) = fee_shard_index {
        data.extend_from_slice(&canonical_execute_fee_header(shard));
    }
    data.extend_from_slice(payload);

    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(script, false),
            AccountMeta::new(vm_state, false),
            AccountMeta::new(payer_signer_writable, true),
            AccountMeta::new(fee_vault, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data,
    }
}
