#![cfg(feature = "test-fixtures")]

use five_protocol::{
    execute_payload::{canonical_execute_payload, TypedParam},
    test_fixtures::{
        execute_payload_minimal, execute_payload_typed_sample,
        execute_payload_truncated_function_index, execute_payload_truncated_param_count,
        execute_payload_truncated_u64_param, execute_payload_unknown_type,
    },
    types,
};

#[test]
fn minimal_execute_payload_has_fixed_width_header() {
    let payload = execute_payload_minimal();
    assert_eq!(payload.len(), 8);
    assert_eq!(u32::from_le_bytes(payload[0..4].try_into().unwrap()), 0);
    assert_eq!(u32::from_le_bytes(payload[4..8].try_into().unwrap()), 0);
}

#[test]
fn typed_execute_payload_has_expected_layout() {
    let payload = execute_payload_typed_sample();
    assert_eq!(u32::from_le_bytes(payload[0..4].try_into().unwrap()), 2);
    assert_eq!(u32::from_le_bytes(payload[4..8].try_into().unwrap()), 5);

    // param 1: U64(42)
    assert_eq!(payload[8], types::U64);
    assert_eq!(u64::from_le_bytes(payload[9..17].try_into().unwrap()), 42);

    // param 2: BOOL(true) as u32
    assert_eq!(payload[17], types::BOOL);
    assert_eq!(u32::from_le_bytes(payload[18..22].try_into().unwrap()), 1);

    // param 3: STRING("hi")
    assert_eq!(payload[22], types::STRING);
    assert_eq!(u32::from_le_bytes(payload[23..27].try_into().unwrap()), 2);
    assert_eq!(&payload[27..29], b"hi");

    // param 4: PUBKEY([7; 32])
    assert_eq!(payload[29], types::PUBKEY);
    assert_eq!(payload[30..62], [7u8; 32]);

    // param 5: ACCOUNT(3) as u32
    assert_eq!(payload[62], types::ACCOUNT);
    assert_eq!(u32::from_le_bytes(payload[63..67].try_into().unwrap()), 3);
}

#[test]
fn typed_fixture_matches_canonical_encoder_bytes() {
    let encoded = canonical_execute_payload(
        2,
        &[
            TypedParam::U64(42),
            TypedParam::Bool(true),
            TypedParam::String("hi".to_string()),
            TypedParam::Pubkey([7u8; 32]),
            TypedParam::Account(3),
        ],
    );
    assert_eq!(encoded, execute_payload_typed_sample());
}

#[test]
fn malformed_execute_payload_variants_are_produced() {
    assert_eq!(execute_payload_truncated_function_index().len(), 3);
    assert_eq!(execute_payload_truncated_param_count().len(), 6);

    let truncated_u64 = execute_payload_truncated_u64_param();
    assert_eq!(u32::from_le_bytes(truncated_u64[0..4].try_into().unwrap()), 1);
    assert_eq!(u32::from_le_bytes(truncated_u64[4..8].try_into().unwrap()), 1);
    assert_eq!(truncated_u64[8], types::U64);
    assert_eq!(truncated_u64.len(), 13);

    let unknown = execute_payload_unknown_type();
    assert_eq!(u32::from_le_bytes(unknown[0..4].try_into().unwrap()), 1);
    assert_eq!(u32::from_le_bytes(unknown[4..8].try_into().unwrap()), 1);
    assert_eq!(unknown[8], 0xFF);
}
