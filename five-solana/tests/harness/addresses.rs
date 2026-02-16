use solana_sdk::pubkey::Pubkey;

pub const VM_STATE_SEED: &[u8] = b"vm_state";
pub const FEE_VAULT_SEED_PREFIX: &[u8] = b"\xFFfive_vm_fee_vault_v1";

pub fn vm_state_pda(program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[VM_STATE_SEED], program_id)
}

pub fn fee_vault_pda(program_id: &Pubkey, shard_index: u8) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[FEE_VAULT_SEED_PREFIX, &[shard_index]], program_id)
}

pub fn fee_vault_shard0_pda(program_id: &Pubkey) -> (Pubkey, u8) {
    fee_vault_pda(program_id, 0)
}

pub fn canonical_execute_fee_header(shard_index: u8) -> [u8; 3] {
    [0xFF, 0x53, shard_index]
}
