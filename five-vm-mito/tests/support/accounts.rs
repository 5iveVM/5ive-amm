use five_vm_mito::AccountInfo;
use pinocchio::pubkey::Pubkey;
use solana_sdk::pubkey::Pubkey as SolanaPubkey;

pub fn derive_pda_real(seeds: &[&[u8]], program_id: &Pubkey) -> (Pubkey, u8) {
    // Convert pinocchio Pubkey to solana_sdk Pubkey for PDA derivation
    let solana_program_id = SolanaPubkey::new_from_array(program_id.as_ref().try_into().unwrap());
    let (pda_pubkey, bump) = SolanaPubkey::find_program_address(seeds, &solana_program_id);
    // Convert back to pinocchio Pubkey
    (Pubkey::from(pda_pubkey.to_bytes()), bump)
}

/// Helper to create a proper test environment with valid accounts
pub fn create_test_accounts<'a>(
    program_id: &Pubkey,
    account_key: &Pubkey,
    lamports: &'a mut u64,
    data: &'a mut [u8],
    payer_lamports: &'a mut u64,
    payer_data: &'a mut [u8],
    system_lamports: &'a mut u64,
    system_data: &'a mut [u8],
) -> [AccountInfo; 3] { // Now returns 3 accounts
    let payer_key = Pubkey::from([1u8; 32]);
    let system_program_key = Pubkey::from([0u8; 32]); // System Program ID (all zeros for test/mock usually, or standard ID)

    // Account 0: Payer (Signer, Writable)
    let payer = AccountInfo::new(
        &payer_key,
        true, // is_signer
        true, // is_writable
        payer_lamports,
        payer_data,
        program_id,
        false,
        0,
    );

    // Account 1: New Account (Signer, Writable) - to be initialized
    let new_account = AccountInfo::new(
        account_key,
        true, // is_signer
        true, // is_writable
        lamports,
        data,
        program_id,
        false,
        0,
    );

    // Account 2: System Program (Executable)
    let system_program = AccountInfo::new(
        &system_program_key,
        false, // is_signer
        false, // is_writable
        system_lamports,
        system_data,
        &system_program_key,
        true, // executable
        0,
    );

    [payer, new_account, system_program]
}

pub fn encode_vle(mut value: u64) -> Vec<u8> {
    let mut bytes = Vec::new();
    loop {
        let mut byte = (value & 0x7F) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        bytes.push(byte);
        if value == 0 {
            break;
        }
    }
    bytes
}
