use five_protocol::*;

#[test]
fn test_opcode_definitions() {
    // Test that all opcodes are unique
    let mut opcodes = [false; 256];
    for info in OPCODE_TABLE {
        assert!(
            !opcodes[info.opcode as usize],
            "Duplicate opcode: 0x{:02X}",
            info.opcode
        );
        opcodes[info.opcode as usize] = true;
    }
}

#[test]
fn test_opcode_lookup() {
    // Test opcode information lookup
    assert_eq!(opcode_name(HALT), "HALT");
    assert_eq!(opcode_name(PUSH_U64), "PUSH_U64");
    assert_eq!(opcode_name(CALL), "CALL");
    assert_eq!(opcode_name(RESULT_IS_ERR), "RESULT_IS_ERR");

    assert!(is_valid_opcode(HALT));
    assert!(is_valid_opcode(CALL));
    assert!(is_valid_opcode(RESULT_IS_ERR));
}

#[test]
fn test_value_conversions() {
    let val_u64 = Value::U64(42);
    assert_eq!(val_u64.as_u64(), Some(42));
    assert_eq!(val_u64.as_bool(), Some(true));
    assert_eq!(val_u64.type_id(), five_protocol::types::U64); // 4 (FIXED: was 2)

    let val_bool = Value::Bool(false);
    assert_eq!(val_bool.as_bool(), Some(false));
    assert_eq!(val_bool.type_id(), five_protocol::types::BOOL); // 9 (FIXED: was 5)

    let val_empty = Value::Empty;
    assert_eq!(val_empty.as_u64(), None);
    assert_eq!(val_empty.type_id(), five_protocol::types::EMPTY); // 0 (correct)
}

#[test]
fn test_valueref_u128_serialization() {
    // Test U128 serialization/deserialization roundtrip
    let original = ValueRef::U128(0x123456789ABCDEF0123456789ABCDEF0);

    // Test type_id is correct (now matches types::U128 constant)
    assert_eq!(original.type_id(), five_protocol::types::U128); // 14 (FIXED: was 4)

    // Test is_immediate includes U128
    assert!(original.is_immediate());

    // Test serialization size
    let expected_size = 17; // 1 byte type_id + 16 bytes u128
    assert_eq!(original.serialized_size(), expected_size);

    // Test roundtrip serialization
    let mut buffer = [0u8; 32]; // Oversized buffer
    let serialized_size = original.serialize_into(&mut buffer).unwrap();
    assert_eq!(serialized_size, expected_size);

    // Test deserialization
    let deserialized = ValueRef::deserialize_from(&buffer[..serialized_size]).unwrap();
    assert_eq!(deserialized, original);

    // Test edge cases
    let zero = ValueRef::U128(0);
    let mut buffer = [0u8; 32];
    let size = zero.serialize_into(&mut buffer).unwrap();
    let recovered = ValueRef::deserialize_from(&buffer[..size]).unwrap();
    assert_eq!(recovered, zero);

    let max = ValueRef::U128(u128::MAX);
    let mut buffer = [0u8; 32];
    let size = max.serialize_into(&mut buffer).unwrap();
    let recovered = ValueRef::deserialize_from(&buffer[..size]).unwrap();
    assert_eq!(recovered, max);
}

#[test]
fn test_call_stack() {
    let mut stack = CallStack::new();
    assert!(stack.is_empty());
    assert_eq!(stack.depth(), 0);

    let frame = CallFrame::new(100, 1, 2, 3, 0, 10);
    assert!(stack.push(frame).is_ok());
    assert!(!stack.is_empty());
    assert_eq!(stack.depth(), 1);

    let popped = stack.pop().unwrap();
    assert_eq!(popped.return_address, frame.return_address);
    assert!(stack.is_empty());
}

#[test]
fn test_function_table() {
    let mut table = FunctionTable::new();
    assert_eq!(table.count(), 0);

    let sig = FunctionSignature::new(0x12345678, 2, Some(2), 4);
    let index = table.add_function(sig, 1000).unwrap();
    assert_eq!(index, 0);
    assert_eq!(table.count(), 1);

    assert_eq!(table.get_offset(0), Some(1000));
    assert_eq!(table.get_offset(1), None);

    let retrieved_sig = table.get_signature(0).unwrap();
    assert_eq!(retrieved_sig.name_hash, 0x12345678);
    assert_eq!(retrieved_sig.parameter_count, 2);
}

#[test]
fn test_instruction_encoding() {
    let inst = Instruction::new(CALL, 42, 1337);
    let encoded = inst.encode();
    let decoded = Instruction::decode(&encoded).unwrap();

    assert_eq!(decoded.opcode, CALL);
    assert_eq!(decoded.arg1, 42);
    assert_eq!(decoded.arg2, 1337);
}

#[test]
fn test_jump_table() {
    let mut table = JumpTable::new();

    let index1 = table.add_entry(1000).unwrap();
    let index2 = table.add_entry(2000).unwrap();
    assert_eq!(index1, 0);
    assert_eq!(index2, 1);

    assert_eq!(table.get_offset(0), Some(1000));
    assert_eq!(table.get_offset(1), Some(2000));
    assert_eq!(table.get_offset(2), None);

    // Test encoding/decoding
    let encoded = table.encode().unwrap();
    let decoded = JumpTable::decode(&encoded).unwrap();

    assert_eq!(decoded.get_offset(0), Some(1000));
    assert_eq!(decoded.get_offset(1), Some(2000));
}

#[test]
fn test_call_protocol() {
    let mut protocol = CallProtocol::new();

    // Add function to table
    let mut table = FunctionTable::new();
    let mut sig = FunctionSignature::new(0x12345678, 1, Some(five_protocol::types::U64), 2);
    sig.parameters[0] = Parameter::new(0x11111111, five_protocol::types::U64); // U64 parameter (type_id=4)
    table.add_function(sig, 1000).unwrap();
    protocol.initialize(table);

    // Test function call preparation
    let params = [Value::U64(42)];
    let offset = protocol.prepare_call(0, &params).unwrap();
    assert_eq!(offset, 1000);
    assert!(protocol.in_function());
    assert_eq!(protocol.call_depth(), 1);

    // Test local variable access
    protocol.set_local(0, ValueRef::Bool(true)).unwrap();
    let val = protocol.get_local(0).unwrap();
    assert_eq!(*val, ValueRef::Bool(true));

    // Test function return
    let return_addr = protocol.finish_call().unwrap();
    assert_eq!(return_addr, 9); // Size of call instruction
    assert!(!protocol.in_function());
    assert_eq!(protocol.call_depth(), 0);
}
