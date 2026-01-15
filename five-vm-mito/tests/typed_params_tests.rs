use five_protocol::encoding::VLE;
use five_protocol::types;
use five_protocol::ValueRef;
use five_vm_mito::{AccountInfo, ExecutionContext, FIVE_VM_PROGRAM_ID, Pubkey, StackStorage, TEMP_BUFFER_SIZE};

#[test]
fn parse_typed_string_parameter() {
    let (func_size, func_bytes) = VLE::encode_u32(2);
    let (count_size, count_bytes) = VLE::encode_u32(1);
    let (len_size, len_bytes) = VLE::encode_u32(2);

    let mut input = Vec::new();
    input.extend_from_slice(&func_bytes[..func_size]);
    input.push(0x80); // Raw sentinel byte, NOT VLE encoded
    input.extend_from_slice(&count_bytes[..count_size]);
    input.push(types::STRING);
    input.extend_from_slice(&len_bytes[..len_size]);
    input.extend_from_slice(b"hi");

    let accounts: [AccountInfo; 0] = [];
    let mut storage = StackStorage::new(&[]);
    let mut ctx = ExecutionContext::new(
        &[],
        &accounts,
        Pubkey::default(),
        &input,
        0,
        &mut storage,
        0,
        0,
    );

    ctx.parse_parameters().unwrap();

    assert_eq!(ctx.parameters()[0], ValueRef::U64(2));
    let string_ref = match ctx.parameters()[1] {
        ValueRef::StringRef(offset) => offset as usize,
        other => panic!("Expected StringRef, got {:?}", other),
    };

    assert!(string_ref < TEMP_BUFFER_SIZE);
    let buffer = ctx.temp_buffer();
    assert_eq!(buffer[string_ref], 2);
    assert_eq!(buffer[string_ref + 1], 1);
    assert_eq!(&buffer[string_ref + 2..string_ref + 4], b"hi");
}
