use five_protocol::types;
use five_vm_mito::{error::VMErrorCode, ExecutionContext, StackStorage, ValueRef};
use pinocchio::{account_info::AccountInfo, pubkey::Pubkey};

fn new_context<'a>(input_data: &'a [u8], storage: &'a mut StackStorage) -> ExecutionContext<'a> {
    let bytecode: &'a [u8] = &[];
    let accounts: &'a [AccountInfo] = &[];
    let program_id = Pubkey::default();
    ExecutionContext::new(
        bytecode,
        accounts,
        program_id,
        input_data,
        0,
        storage,
        0,
        0,
        0,
        0,
        0,
        0,
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

#[test]
fn parse_parameters_decodes_fixed_width_execute_envelope() {
    let payload = canonical_typed_payload();
    let mut storage = StackStorage::new();
    let mut ctx = new_context(&payload, &mut storage);

    ctx.parse_parameters().expect("canonical payload should parse");
    let params = ctx.parameters();

    assert_eq!(params[0], ValueRef::U64(2));
    assert_eq!(params[1], ValueRef::U64(42));
    assert_eq!(params[2], ValueRef::Bool(true));
    assert!(matches!(params[3], ValueRef::StringRef(_)));
    assert!(matches!(params[4], ValueRef::TempRef(_, 32)));
    assert_eq!(params[5], ValueRef::AccountRef(3, 0));
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
