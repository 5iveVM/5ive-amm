use five_protocol::{opcodes::*, FIVE_HEADER_OPTIMIZED_SIZE, FIVE_MAGIC};
use five_vm_mito::{AccountInfo, FIVE_VM_PROGRAM_ID, MitoVM, Pubkey, VMError, stack::StackStorage, Value};

fn execute_test(bytecode: &[u8], input: &[u8], accounts: &[AccountInfo]) -> five_vm_mito::Result<Option<Value>> {
    let mut storage = StackStorage::new();
    MitoVM::execute_direct(bytecode, input, accounts, &FIVE_VM_PROGRAM_ID, &mut storage)
}
use solana_sdk::system_program;

fn build_script(body: &[u8]) -> Vec<u8> {
    let mut script = Vec::with_capacity(FIVE_HEADER_OPTIMIZED_SIZE + body.len());
    script.extend_from_slice(&FIVE_MAGIC);
    // Header V3: features(4 bytes LE) + public_function_count(1) + total_function_count(1)
    script.push(0); // features byte 0
    script.push(0); // features byte 1
    script.push(0); // features byte 2
    script.push(0); // features byte 3
    script.push(1); // public functions
    script.push(1); // total functions
    script.extend_from_slice(body);
    script
}

// Helper to create a simple account with specified data length
fn create_account(data_len: usize) -> AccountInfo {
    // Static keys and owners for simplicity
    let key = system_program::ID.to_bytes();
    let owner: Pubkey = [1u8; 32];
    // Allocate lamports and data on the heap to satisfy lifetime requirements
    let lamports: Box<u64> = Box::new(0);
    let data = vec![0u8; data_len].into_boxed_slice();
    // Create AccountInfo using the pinocchio-style constructor
    AccountInfo::new(
        &key,
        false,
        true,
        Box::leak(lamports),
        Box::leak(data),
        &owner,
        false,
        0,
    )
}

#[test]
fn load_field_out_of_bounds() {
    // Create a single program account with insufficient data
    let account = create_account(4); // Less than 8 bytes
    let accounts = [account];

    // Bytecode: LOAD_FIELD account_index=0 offset=32 (fixed-width); HALT
    // Protocol V3 format: LOAD_FIELD account_index_u8, offset_u32
    let mut body = vec![LOAD_FIELD];
    body.push(0); // account_index = 0
    body.extend_from_slice(&32u32.to_le_bytes()); // Fixed size offset
    body.push(HALT);
    let bytecode = build_script(&body);

    let result = execute_test(&bytecode, &[], &accounts);
    assert!(matches!(result, Err(VMError::InvalidAccountData)));
}

#[test]
fn load_external_field_out_of_bounds() {
    // Program account (unused) and external account with insufficient data
    let program_account = create_account(0);
    let external_account = create_account(4); // Less than 8 bytes
    let accounts = [program_account, external_account];

    // Bytecode: LOAD_EXTERNAL_FIELD account_index=1 offset=32; HALT
    let mut body = vec![LOAD_EXTERNAL_FIELD, 0x01];
    body.extend_from_slice(&32u32.to_le_bytes());
    body.push(HALT);
    let bytecode = build_script(&body);

    let result = execute_test(&bytecode, &[], &accounts);
    assert!(matches!(result, Err(VMError::InvalidAccountData)));
}
