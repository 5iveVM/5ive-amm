use five_protocol::types;
use five_vm_mito::{error::VMErrorCode, ExecutionContext, StackStorage, ValueRef};
use pinocchio::{account_info::AccountInfo, pubkey::Pubkey};

fn new_context<'a>(input_data: &'a [u8], storage: &'a mut StackStorage) -> ExecutionContext<'a> {
    let bytecode: &'a [u8] = &[];
    let accounts: &'a [AccountInfo] = &[];
    let program_id = Pubkey::default();
    ExecutionContext::new(
        bytecode, accounts, program_id, input_data, 0, storage, 0, 0, 0, 0, 0, 0,
    )
}

fn canonical_typed_payload() -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&2u32.to_le_bytes()); // function index
    out.extend_from_slice(&5u32.to_le_bytes()); // param count
    out.push(types::U64);
    out.extend_from_slice(&42u64.to_le_bytes());
    out.push(types::BOOL);
    out.extend_from_slice(&1u32.to_le_bytes());
    out.push(types::STRING);
    out.extend_from_slice(&2u32.to_le_bytes());
    out.extend_from_slice(b"hi");
    out.push(types::PUBKEY);
    out.extend_from_slice(&[7u8; 32]);
    out.push(types::ACCOUNT);
    out.extend_from_slice(&3u32.to_le_bytes());
    out
}

fn token_like_init_mint_payload() -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&0u32.to_le_bytes()); // function index: init_mint
    out.extend_from_slice(&7u32.to_le_bytes()); // 7 params

    // freeze_authority: pubkey
    out.push(types::PUBKEY);
    out.extend_from_slice(&[9u8; 32]);

    // decimals: u8 (fixed-width envelope uses u32 payload)
    out.push(types::U8);
    out.extend_from_slice(&6u32.to_le_bytes());

    // name: "TestToken" (9)
    out.push(types::STRING);
    out.extend_from_slice(&9u32.to_le_bytes());
    out.extend_from_slice(b"TestToken");

    // symbol: "TEST" (4)
    out.push(types::STRING);
    out.extend_from_slice(&4u32.to_le_bytes());
    out.extend_from_slice(b"TEST");

    // uri: "https://example.com/token" (25)
    out.push(types::STRING);
    out.extend_from_slice(&25u32.to_le_bytes());
    out.extend_from_slice(b"https://example.com/token");

    // account placeholders (mint_account, authority)
    out.push(types::ACCOUNT);
    out.extend_from_slice(&0u32.to_le_bytes());
    out.push(types::ACCOUNT);
    out.extend_from_slice(&1u32.to_le_bytes());

    out
}

fn many_large_strings_payload(string_count: u32, string_len: u32) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&string_count.to_le_bytes());
    for _ in 0..string_count {
        out.push(types::STRING);
        out.extend_from_slice(&string_len.to_le_bytes());
        out.extend_from_slice(&vec![b'x'; string_len as usize]);
    }
    out
}

#[test]
fn parse_parameters_decodes_fixed_width_execute_envelope() {
    let payload = canonical_typed_payload();
    let mut storage = StackStorage::new();
    let mut ctx = new_context(&payload, &mut storage);

    ctx.parse_parameters()
        .expect("canonical payload should parse");
    let params = ctx.parameters();

    assert_eq!(params[0], ValueRef::U64(2));
    assert_eq!(params[1], ValueRef::U64(42));
    assert_eq!(params[2], ValueRef::Bool(true));
    assert!(matches!(params[3], ValueRef::StringRef(_)));
    assert!(matches!(params[4], ValueRef::TempRef(_, 32)));
    assert_eq!(params[5], ValueRef::Empty);
}

#[test]
fn parse_parameters_rejects_truncated_function_index() {
    let payload = vec![0x01, 0x02, 0x03];
    let mut storage = StackStorage::new();
    let mut ctx = new_context(&payload, &mut storage);
    let err = ctx.parse_parameters().unwrap_err();
    assert_eq!(err, VMErrorCode::InvalidInstructionPointer);
}

#[test]
fn parse_parameters_rejects_truncated_param_count() {
    let mut payload = Vec::new();
    payload.extend_from_slice(&1u32.to_le_bytes());
    payload.extend_from_slice(&[0xAA, 0xBB]); // short param count
    let mut storage = StackStorage::new();
    let mut ctx = new_context(&payload, &mut storage);
    let err = ctx.parse_parameters().unwrap_err();
    assert_eq!(err, VMErrorCode::InvalidInstructionPointer);
}

#[test]
fn parse_parameters_rejects_truncated_u64_param() {
    let mut payload = Vec::new();
    payload.extend_from_slice(&1u32.to_le_bytes());
    payload.extend_from_slice(&1u32.to_le_bytes());
    payload.push(types::U64);
    payload.extend_from_slice(&[0x11, 0x22, 0x33, 0x44]); // short u64

    let mut storage = StackStorage::new();
    let mut ctx = new_context(&payload, &mut storage);
    let err = ctx.parse_parameters().unwrap_err();
    assert_eq!(err, VMErrorCode::InvalidInstructionPointer);
}

#[test]
fn parse_parameters_rejects_unknown_type_id() {
    let mut payload = Vec::new();
    payload.extend_from_slice(&1u32.to_le_bytes());
    payload.extend_from_slice(&1u32.to_le_bytes());
    payload.push(0xFF);
    payload.extend_from_slice(&0u64.to_le_bytes());

    let mut storage = StackStorage::new();
    let mut ctx = new_context(&payload, &mut storage);
    let err = ctx.parse_parameters().unwrap_err();
    assert_eq!(err, VMErrorCode::TypeMismatch);
}

#[test]
fn parse_parameters_token_shape_parses_without_panicking() {
    let payload = token_like_init_mint_payload();
    let mut storage = StackStorage::new();
    let mut ctx = new_context(&payload, &mut storage);

    ctx.parse_parameters()
        .expect("token-shaped payload should parse");
    let params = ctx.parameters();
    assert!(matches!(params[1], ValueRef::TempRef(_, 32)));
    assert_eq!(params[2], ValueRef::U8(6));
    assert!(matches!(params[3], ValueRef::StringRef(_)));
    assert!(matches!(params[4], ValueRef::StringRef(_)));
    assert!(matches!(params[5], ValueRef::StringRef(_)));
    assert_eq!(params[6], ValueRef::Empty);
    assert_eq!(params[7], ValueRef::Empty);
}

#[test]
fn parse_parameters_many_large_strings_fails_gracefully() {
    let payload = many_large_strings_payload(4, 130);
    let mut storage = StackStorage::new();
    let mut ctx = new_context(&payload, &mut storage);

    let err = ctx.parse_parameters().unwrap_err();
    assert!(matches!(
        err,
        VMErrorCode::MemoryError | VMErrorCode::OutOfMemory
    ));
}

#[test]
fn parse_parameters_rejects_large_string_without_panicking() {
    let mut payload = Vec::new();
    payload.extend_from_slice(&0u32.to_le_bytes());
    payload.extend_from_slice(&1u32.to_le_bytes());
    payload.push(types::STRING);
    payload.extend_from_slice(&300u32.to_le_bytes());
    payload.extend_from_slice(&vec![b'a'; 300]);

    let mut storage = StackStorage::new();
    let mut ctx = new_context(&payload, &mut storage);
    let err = ctx.parse_parameters().unwrap_err();
    assert_eq!(err, VMErrorCode::OutOfMemory);
}
