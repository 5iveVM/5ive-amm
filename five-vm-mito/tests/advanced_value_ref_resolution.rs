use five_protocol::ValueRef;
use five_vm_mito::{
    error::VMErrorCode, utils::ValueRefUtils, AccountInfo, ExecutionContext, MitoVM,
    StackStorage, Value,
    systems::resource::ResourceManager,
};
use pinocchio::pubkey::Pubkey;

#[test]
fn resolve_temp_ref_valid() {
    let accounts: [AccountInfo; 0] = [];
    let mut storage = StackStorage::new();

    // Temp buffer size is 64 bytes (typically).

    let mut temp_buffer = vec![0u8; 64];
    let val_ref = ValueRef::U64(42);
    let mut writer = &mut temp_buffer[0..];
    val_ref.serialize_into(&mut writer).unwrap();
    let serialized_len = 9; // 1 byte tag + 8 bytes value

    let mut ctx = ExecutionContext::new(
        &[],
        &accounts,
        Pubkey::default(),
        &[],
        0,
        &mut storage,
        0,
        0,
    );

    ctx.temp_buffer_mut()[..serialized_len].copy_from_slice(&temp_buffer[..serialized_len]);

    let value = MitoVM::resolve_value_ref(&ValueRef::TempRef(0, serialized_len as u8), &ctx).unwrap();
    assert_eq!(value, Value::U64(42));
}

#[test]
fn resolve_temp_ref_raw_bytes() {
    let accounts: [AccountInfo; 0] = [];
    let mut storage = StackStorage::new();
    let mut ctx = ExecutionContext::new(
        &[],
        &accounts,
        Pubkey::default(),
        &[],
        0,
        &mut storage,
        0,
        0,
    );

    // Put raw bytes representing 12345u64 at offset 10
    let val = 12345u64;
    let bytes = val.to_le_bytes();

    // Make sure we are within bounds (size 64)
    ctx.temp_buffer_mut()[10..18].copy_from_slice(&bytes);

    // 12345 = 0x3039. Byte 0 is 0x39 = 57. 57 is not a valid ValueRef tag (0-16 typically).
    // So deserialize will fail, falling back to raw bytes.
    // This fallback behavior allows treating raw memory as u64 when it's not a valid ValueRef.

    let value = MitoVM::resolve_value_ref(&ValueRef::TempRef(10, 8), &ctx).unwrap();
    assert_eq!(value, Value::U64(12345));
}

#[test]
fn resolve_temp_ref_out_of_bounds() {
    // Force a small buffer to verify OOB check
    let mut small_buffer = [0u8; 10];

    let accounts: [AccountInfo; 0] = [];
    let mut storage = StackStorage::new();
    let mut heap_buffer = [0u8; 2048];
    let mut ctx = ExecutionContext::new(
        &[],
        &accounts,
        Pubkey::default(),
        &[],
        0,
        &mut storage,
        0,
        0,
    );

    ctx.memory = ResourceManager::new(&mut small_buffer, &mut heap_buffer);

    // Offset 10 is OOB (buffer size 10, index 0..9)
    let offset = 10u8;
    let size = 1u8;

    let err = MitoVM::resolve_value_ref(&ValueRef::TempRef(offset, size), &ctx).unwrap_err();
    assert_eq!(err, VMErrorCode::MemoryViolation);
}

#[test]
fn resolve_tuple_ref() {
    let accounts: [AccountInfo; 0] = [];
    let mut storage = StackStorage::new();
    let ctx = ExecutionContext::new(
        &[],
        &accounts,
        Pubkey::default(),
        &[],
        0,
        &mut storage,
        0,
        0,
    );

    // TupleRef should resolve to Value::Array
    let value = MitoVM::resolve_value_ref(&ValueRef::TupleRef(10, 5), &ctx).unwrap();
    assert_eq!(value, Value::Array(10));
}

#[test]
fn resolve_optional_some() {
    let accounts: [AccountInfo; 0] = [];
    let mut storage = StackStorage::new();
    let mut ctx = ExecutionContext::new(
        &[],
        &accounts,
        Pubkey::default(),
        &[],
        0,
        &mut storage,
        0,
        0,
    );

    // Write Some(U8(99))
    let inner_val = ValueRef::U8(99);
    let mut buffer = vec![0u8; 10];
    buffer[0] = 1; // Some
    let mut writer = &mut buffer[1..];
    inner_val.serialize_into(&mut writer).unwrap();

    ctx.temp_buffer_mut()[20..30].copy_from_slice(&buffer);

    let value = MitoVM::resolve_value_ref(&ValueRef::OptionalRef(20, 3), &ctx).unwrap();
    assert_eq!(value, Value::U8(99));
}

#[test]
fn resolve_optional_none() {
    let accounts: [AccountInfo; 0] = [];
    let mut storage = StackStorage::new();
    let mut ctx = ExecutionContext::new(
        &[],
        &accounts,
        Pubkey::default(),
        &[],
        0,
        &mut storage,
        0,
        0,
    );

    // Write None at offset 50
    ctx.temp_buffer_mut()[50] = 0; // None

    let value = MitoVM::resolve_value_ref(&ValueRef::OptionalRef(50, 1), &ctx).unwrap();
    assert_eq!(value, Value::Empty);
}

#[test]
fn resolve_optional_invalid_size() {
    let accounts: [AccountInfo; 0] = [];
    let mut storage = StackStorage::new();
    let mut ctx = ExecutionContext::new(
        &[],
        &accounts,
        Pubkey::default(),
        &[],
        0,
        &mut storage,
        0,
        0,
    );

    // Size 0 is invalid
    let err = MitoVM::resolve_value_ref(&ValueRef::OptionalRef(0, 0), &ctx).unwrap_err();
    assert_eq!(err, VMErrorCode::ProtocolError);

    // Size 1 but Some (tag 1) -> needs inner value
    ctx.temp_buffer_mut()[0] = 1; // Some
    let err = MitoVM::resolve_value_ref(&ValueRef::OptionalRef(0, 1), &ctx).unwrap_err();
    assert_eq!(err, VMErrorCode::ProtocolError);
}

#[test]
fn resolve_result_ok() {
    let accounts: [AccountInfo; 0] = [];
    let mut storage = StackStorage::new();
    let mut ctx = ExecutionContext::new(
        &[],
        &accounts,
        Pubkey::default(),
        &[],
        0,
        &mut storage,
        0,
        0,
    );

    let inner_val = ValueRef::U8(77);
    let mut buffer = vec![0u8; 10];
    buffer[0] = 1; // Tag
    let mut writer = &mut buffer[1..];
    inner_val.serialize_into(&mut writer).unwrap();

    ctx.temp_buffer_mut()[40..50].copy_from_slice(&buffer);

    let value = MitoVM::resolve_value_ref(&ValueRef::ResultRef(40, 3), &ctx).unwrap();
    assert_eq!(value, Value::U8(77));
}

#[test]
fn resolve_recursion_limit() {
    let accounts: [AccountInfo; 0] = [];
    let mut storage = StackStorage::new();
    let mut ctx = ExecutionContext::new(
        &[],
        &accounts,
        Pubkey::default(),
        &[],
        0,
        &mut storage,
        0,
        0,
    );

    // Need 9 levels of TempRef to bust 8 limit.
    // serialized TempRef is 1 byte tag + 1 byte offset + 1 byte size = 3 bytes! (because u8 offset/size)
    // Wait, ValueRef::TempRef(u8, u8)
    // serialize: [tag, u8, u8] = 3 bytes.

    // Buffer size 64.
    // 3 bytes per ref. 9 refs = 27 bytes. Fits easily.

    let mut offset = 0;
    let mut writer_buf = vec![0u8; 64];

    // 0: TempRef(3, 3)
    // 3: TempRef(6, 3)
    // ...

    for _i in 0..10 {
        let next_offset = offset + 3;
        // Point to next one
        let val_ref = ValueRef::TempRef(next_offset as u8, 3);
        let mut slice = &mut writer_buf[offset..];
        val_ref.serialize_into(&mut slice).unwrap();
        offset = next_offset;
    }

    ctx.temp_buffer_mut()[0..64].copy_from_slice(&writer_buf);

    let start_ref = ValueRef::TempRef(0, 3);
    let err = MitoVM::resolve_value_ref(&start_ref, &ctx).unwrap_err();
    assert_eq!(err, VMErrorCode::StackOverflow);
}

#[test]
fn test_value_ref_utils() {
    // as_u64
    assert_eq!(ValueRefUtils::as_u64(ValueRef::U64(100)).unwrap(), 100);
    assert_eq!(ValueRefUtils::as_u64(ValueRef::U8(100)).unwrap(), 100);
    assert_eq!(ValueRefUtils::as_u64(ValueRef::Bool(true)).unwrap(), 1);
    assert_eq!(ValueRefUtils::as_u64(ValueRef::Bool(false)).unwrap(), 0);
    // Type mismatch
    assert_eq!(ValueRefUtils::as_u64(ValueRef::Empty).unwrap_err(), VMErrorCode::TypeMismatch);

    // as_bool
    assert_eq!(ValueRefUtils::as_bool(ValueRef::Bool(true)).unwrap(), true);
    assert_eq!(ValueRefUtils::as_bool(ValueRef::U64(1)).unwrap(), true);
    assert_eq!(ValueRefUtils::as_bool(ValueRef::U64(0)).unwrap(), false);

    // as_i64
    assert_eq!(ValueRefUtils::as_i64(ValueRef::I64(-5)).unwrap(), -5);
    assert_eq!(ValueRefUtils::as_i64(ValueRef::U64(5)).unwrap(), 5);
}

#[test]
fn resolve_account_ref_valid() {
    let program_id = Pubkey::default();
    let key = Pubkey::from([1u8; 32]);
    let mut lamports = 0;
    let mut data = [];
    let account = AccountInfo::new(
        &key,
        false,
        false,
        &mut lamports,
        &mut data,
        &program_id,
        false,
        0,
    );
    let accounts = [account];
    let mut storage = StackStorage::new();
    let ctx = ExecutionContext::new(
        &[],
        &accounts,
        Pubkey::default(),
        &[],
        0,
        &mut storage,
        0,
        0,
    );

    let value = MitoVM::resolve_value_ref(&ValueRef::AccountRef(0, 0), &ctx).unwrap();
    assert_eq!(value, Value::Account(0));
}
