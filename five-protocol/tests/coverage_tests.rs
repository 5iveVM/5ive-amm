use five_protocol::*;

#[test]
fn test_types_coverage() {
    // Test ImportableAccountHeader
    let header = ImportableAccountHeader::new(10, 100, 200, 500);
    assert!(header.is_valid());
    assert_eq!(header.magic, five_protocol::types::FIVE_IMPORT_MAGIC);

    // Copy packed fields to locals to avoid unaligned reference error
    let func_count = header.function_count;
    let table_offset = header.function_table_offset;
    let bc_offset = header.bytecode_offset;
    let bc_size = header.bytecode_size;

    assert_eq!(func_count, 10);
    assert_eq!(table_offset, 100);
    assert_eq!(bc_offset, 200);
    assert_eq!(bc_size, 500);

    let invalid_header = ImportableAccountHeader {
        magic: [0, 0, 0, 0],
        function_count: 0,
        function_table_offset: 0,
        bytecode_offset: 0,
        bytecode_size: 0,
    };
    assert!(!invalid_header.is_valid());

    // Test ImportableFunctionEntry and function_flags
    let flags = types::function_flags::PUBLIC | types::function_flags::with_param_count(3);
    let entry = ImportableFunctionEntry::new(0x12345678, 50, 100, flags);

    // Copy packed fields
    let name_hash = entry.name_hash;
    let bc_offset_entry = entry.bytecode_offset;
    let func_size = entry.function_size;

    assert_eq!(name_hash, 0x12345678);
    assert_eq!(bc_offset_entry, 50);
    assert_eq!(func_size, 100);
    assert!(entry.is_public());
    assert_eq!(entry.param_count(), 3);

    let private_flags = types::function_flags::with_param_count(0);
    let private_entry = ImportableFunctionEntry::new(0, 0, 0, private_flags);
    assert!(!private_entry.is_public());
    assert_eq!(private_entry.param_count(), 0);

    // Test hash_function_name
    let hash = types::hash_function_name(b"test_function");
    // FNV-1a hash of "test_function" should be consistent
    assert_eq!(hash, types::hash_function_name(b"test_function"));
    assert_ne!(hash, types::hash_function_name(b"other_function"));
}

#[test]
fn test_parameter_protocol_coverage() {
    let mut sig = FunctionSignature::new(0x12345678, 2, None, 0);
    sig.parameters[0] = Parameter::new(0x1, types::U64);
    sig.parameters[1] = Parameter::new(0x2, types::BOOL);

    let mut protocol = ParameterProtocol::new(sig);
    assert_eq!(protocol.count, 0);

    // Test successful parameter addition
    assert!(protocol.add_param(Value::U64(100)).is_ok());
    assert_eq!(protocol.count, 1);
    assert_eq!(protocol.get_param(0), Some(&Value::U64(100)));

    assert!(protocol.add_param(Value::Bool(true)).is_ok());
    assert_eq!(protocol.count, 2);
    assert_eq!(protocol.get_param(1), Some(&Value::Bool(true)));

    // Test getting parameter out of bounds
    assert_eq!(protocol.get_param(2), None);

    // Test validation success
    assert!(protocol.validate().is_ok());

    // Test adding too many parameters
    assert_eq!(protocol.add_param(Value::U64(0)), Err(CallError::TooManyParameters));

    // Test invalid parameter type
    let mut protocol_bad_type = ParameterProtocol::new(sig);
    assert_eq!(protocol_bad_type.add_param(Value::Bool(true)), Err(CallError::InvalidParameterType));

    // Test missing parameters validation
    let mut protocol_missing = ParameterProtocol::new(sig);
    protocol_missing.add_param(Value::U64(100)).unwrap();
    assert_eq!(protocol_missing.validate(), Err(CallError::MissingParameters));
}

#[test]
fn test_value_conversion_coverage() {
    // Value -> ValueRef -> Value roundtrip where possible

    // U8
    let v_u8 = Value::U8(42);
    let r_u8 = v_u8.to_valueref();
    assert_eq!(r_u8, ValueRef::U8(42));
    assert_eq!(r_u8.to_value(), Some(v_u8));
    assert_eq!(v_u8.as_u8(), Some(42));
    assert_eq!(r_u8.as_u8(), Some(42));

    // U64
    let v_u64 = Value::U64(12345);
    let r_u64 = v_u64.to_valueref();
    assert_eq!(r_u64, ValueRef::U64(12345));
    assert_eq!(r_u64.to_value(), Some(v_u64));
    assert_eq!(v_u64.as_u64(), Some(12345));
    assert_eq!(r_u64.as_u64(), Some(12345));

    // I64
    let v_i64 = Value::I64(-12345);
    let r_i64 = v_i64.to_valueref();
    assert_eq!(r_i64, ValueRef::I64(-12345));
    assert_eq!(r_i64.to_value(), Some(v_i64));
    assert_eq!(v_i64.as_i64(), Some(-12345));
    assert_eq!(r_i64.immediate_as_i64(), Some(-12345));

    // U128
    let v_u128 = Value::U128(u128::MAX);
    let r_u128 = v_u128.to_valueref();
    assert_eq!(r_u128, ValueRef::U128(u128::MAX));
    assert_eq!(r_u128.to_value(), Some(v_u128));

    // Bool
    let v_bool = Value::Bool(true);
    let r_bool = v_bool.to_valueref();
    assert_eq!(r_bool, ValueRef::Bool(true));
    assert_eq!(r_bool.to_value(), Some(v_bool));
    assert_eq!(v_bool.as_bool(), Some(true));
    assert_eq!(r_bool.as_bool(), Some(true));

    // Empty
    let v_empty = Value::Empty;
    let r_empty = v_empty.to_valueref();
    assert_eq!(r_empty, ValueRef::Empty);
    assert_eq!(r_empty.to_value(), Some(v_empty));

    // Account
    let v_account = Value::Account(5);
    let r_account = v_account.to_valueref();
    assert_eq!(r_account, ValueRef::U8(5)); // Note: Value::Account converts to ValueRef::U8 in current impl
    // When converting back, it becomes Value::U8(5), not Value::Account(5)
    // This is expected behavior based on Value::to_valueref implementation
    assert_eq!(r_account.to_value(), Some(Value::U8(5)));
    assert_eq!(v_account.as_account_idx(), Some(5));

    // Array
    let v_array = Value::Array(10);
    let r_array = v_array.to_valueref();
    assert_eq!(r_array, ValueRef::U8(10)); // Converts to U8
    assert_eq!(r_array.to_value(), Some(Value::U8(10)));
    assert_eq!(v_array.as_array_idx(), Some(10));

    // String
    let v_string = Value::String(20);
    let r_string = v_string.to_valueref();
    assert_eq!(r_string, ValueRef::U8(20)); // Converts to U8
    assert_eq!(r_string.to_value(), Some(Value::U8(20)));
    assert_eq!(v_string.as_string_idx(), Some(20));

    // Pubkey (Complex conversion needed, returns Empty currently)
    let v_pubkey = Value::Pubkey([1u8; 32]);
    let r_pubkey = v_pubkey.to_valueref();
    assert_eq!(r_pubkey, ValueRef::Empty);

    // ValueRef specific conversions to legacy Value
    // HeapString
    let r_heap_str = ValueRef::HeapString(55);
    assert_eq!(r_heap_str.to_value(), Some(Value::String(55)));

    // HeapArray
    let r_heap_arr = ValueRef::HeapArray(66);
    assert_eq!(r_heap_arr.to_value(), Some(Value::Array(66)));

    // AccountRef
    let r_acc_ref = ValueRef::AccountRef(7, 100);
    assert_eq!(r_acc_ref.to_value(), Some(Value::Account(7)));

    // AsRef / ArrayRef (mapped to legacy Array)
    let r_arr_ref = ValueRef::ArrayRef(8);
    assert_eq!(r_arr_ref.to_value(), Some(Value::Array(8)));

    // Test is_truthy
    assert!(Value::U64(1).is_truthy());
    assert!(!Value::U64(0).is_truthy());
    assert!(Value::Bool(true).is_truthy());
    assert!(!Value::Bool(false).is_truthy());
    assert!(!Value::Empty.is_truthy());

    assert!(ValueRef::U64(1).is_truthy());
    assert!(!ValueRef::U64(0).is_truthy());
    assert!(ValueRef::Bool(true).is_truthy());
    assert!(!ValueRef::Bool(false).is_truthy());
    assert!(!ValueRef::Empty.is_truthy());
}
