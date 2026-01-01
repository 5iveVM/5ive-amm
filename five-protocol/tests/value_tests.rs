use five_protocol::{ProtocolError, ValueRef};

#[test]
fn test_valueref_serialization_roundtrip() {
    let test_cases = vec![
        ValueRef::Empty,
        ValueRef::U8(42),
        ValueRef::U64(1234567890),
        ValueRef::I64(-1234567890),
        ValueRef::U128(123456789012345678901234567890),
        ValueRef::Bool(true),
        ValueRef::Bool(false),
        ValueRef::AccountRef(5, 100),
        ValueRef::InputRef(200),
        ValueRef::TempRef(10, 20),
        ValueRef::TupleRef(30, 40),
        ValueRef::OptionalRef(50, 60),
        ValueRef::ResultRef(70, 80),
        ValueRef::PubkeyRef(300),
        ValueRef::ArrayRef(5),
        ValueRef::StringRef(400),
        ValueRef::HeapString(1000),
        ValueRef::HeapArray(2000),
    ];

    for value in test_cases {
        // Test serialized_size
        let expected_size = value.serialized_size();
        let mut buffer = vec![0u8; expected_size];

        // Test serialize_into
        let size = value.serialize_into(&mut buffer).expect("Serialization failed");
        assert_eq!(size, expected_size, "Serialized size mismatch for {:?}", value);

        // Test deserialize_from
        let deserialized = ValueRef::deserialize_from(&buffer).expect("Deserialization failed");
        assert_eq!(value, deserialized, "Roundtrip mismatch for {:?}", value);
    }
}

#[test]
fn test_valueref_buffer_too_small() {
    let value = ValueRef::U64(100);
    let mut buffer = vec![0u8; 5]; // U64 needs 9 bytes (1 + 8)
    let result = value.serialize_into(&mut buffer);
    assert_eq!(result, Err(ProtocolError::BufferTooSmall));
}

#[test]
fn test_valueref_kiss_option_result() {
    // Test Option::None
    let none = ValueRef::option_none();
    assert!(none.is_option_none());
    assert!(!none.is_option_some());
    assert!(none.get_option_data().is_none());

    // Test Option::Some
    let some = ValueRef::option_some(1, 10);
    assert!(some.is_option_some());
    assert!(!some.is_option_none());
    assert_eq!(some.get_option_data(), Some((1, 10)));

    // Test Result::Err
    let err = ValueRef::result_err(5);
    assert!(err.is_result_err());
    assert!(!err.is_result_ok());
    assert_eq!(err.get_result_data(), Err(5));

    // Test Result::Ok
    let ok = ValueRef::result_ok(2, 20);
    assert!(ok.is_result_ok());
    assert!(!ok.is_result_err());
    assert_eq!(ok.get_result_data(), Ok((2, 20)));
}

#[test]
fn test_valueref_immediate_helpers() {
    // U64 helpers
    let v_u64 = ValueRef::U64(100);
    assert!(v_u64.is_immediate());
    assert_eq!(v_u64.immediate_as_u64(), Some(100));
    assert_eq!(v_u64.immediate_as_i64(), Some(100));
    assert_eq!(v_u64.immediate_as_u8(), Some(100));
    assert_eq!(v_u64.immediate_as_bool(), Some(true));

    // I64 helpers
    let v_i64 = ValueRef::I64(-100);
    assert!(v_i64.is_immediate());
    assert_eq!(v_i64.immediate_as_i64(), Some(-100));
    assert_eq!(v_i64.immediate_as_u64(), None); // Negative i64 -> u64 is None

    // Bool helpers
    let v_bool = ValueRef::Bool(true);
    assert!(v_bool.is_immediate());
    assert_eq!(v_bool.immediate_as_bool(), Some(true));

    // Non-immediate
    let v_ref = ValueRef::TempRef(0, 0);
    assert!(!v_ref.is_immediate());
    assert!(v_ref.is_reference());
}
