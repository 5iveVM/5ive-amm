use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

use js_sys::{Array, Object, Reflect, Uint8Array};

use five_protocol::types;
use five_vm_wasm::ParameterEncoder;

wasm_bindgen_test_configure!(run_in_browser);

fn make_param(type_name: &str, value: JsValue) -> JsValue {
    let obj = Object::new();
    Reflect::set(&obj, &"type".into(), &JsValue::from_str(type_name)).unwrap();
    Reflect::set(&obj, &"value".into(), &value).unwrap();
    obj.into()
}

fn make_sized_string_param(value: &str, max_len: u32) -> JsValue {
    let obj = Object::new();
    Reflect::set(&obj, &"type".into(), &JsValue::from_str("string")).unwrap();
    Reflect::set(&obj, &"value".into(), &JsValue::from_str(value)).unwrap();
    Reflect::set(&obj, &"maxLen".into(), &JsValue::from_f64(max_len as f64)).unwrap();
    obj.into()
}

fn make_account_param(index: u32) -> JsValue {
    let obj = Object::new();
    Reflect::set(&obj, &"isAccount".into(), &JsValue::from_bool(true)).unwrap();
    Reflect::set(&obj, &"value".into(), &JsValue::from_f64(index as f64)).unwrap();
    obj.into()
}

#[wasm_bindgen_test]
fn encode_execute_encodes_fixed_size_typed_params() {
    let params = Array::new();

    // u64 42
    params.push(&make_param("u64", JsValue::from_f64(42.0)));
    // bool true
    params.push(&make_param("bool", JsValue::from_bool(true)));
    // string "hi"
    params.push(&make_param("string", JsValue::from_str("hi")));
    // account index 3
    params.push(&make_account_param(3));
    // pubkey from Uint8Array
    let mut pk_bytes = vec![0u8; 32];
    for (i, b) in pk_bytes.iter_mut().enumerate() {
        *b = i as u8;
    }
    let pk_array = Uint8Array::from(pk_bytes.as_slice());
    params.push(&make_param("pubkey", pk_array.into()));

    let encoded = ParameterEncoder::encode_execute(0, params).unwrap();
    let encoded_bytes = encoded.to_vec();

    let mut expected = Vec::new();
    // u64
    expected.push(types::U64);
    expected.extend_from_slice(&42u64.to_le_bytes());
    // bool
    expected.push(types::BOOL);
    expected.extend_from_slice(&1u32.to_le_bytes());
    // string "hi"
    expected.push(types::STRING);
    expected.extend_from_slice(&(2u32).to_le_bytes());
    expected.extend_from_slice(b"hi");
    // account index 3
    expected.push(types::ACCOUNT);
    expected.extend_from_slice(&3u32.to_le_bytes());
    // pubkey
    expected.push(types::PUBKEY);
    expected.extend_from_slice(&pk_bytes);

    assert_eq!(encoded_bytes, expected);
    assert_ne!(encoded_bytes.first().copied(), Some(0x80));
}

#[wasm_bindgen_test]
fn encode_execute_bytes_param_uses_string_format() {
    let params = Array::new();
    let bytes = vec![1u8, 2, 3, 4, 5];
    let array = Uint8Array::from(bytes.as_slice());
    params.push(&make_param("bytes", array.into()));

    let encoded = ParameterEncoder::encode_execute(0, params).unwrap();
    let encoded_bytes = encoded.to_vec();

    let mut expected = Vec::new();
    expected.push(types::STRING);
    expected.extend_from_slice(&(5u32).to_le_bytes());
    expected.extend_from_slice(&bytes);

    assert_eq!(encoded_bytes, expected);
}

#[wasm_bindgen_test]
fn encode_execute_rejects_unsupported_types() {
    let params = Array::new();
    params.push(&make_param("u16", JsValue::from_f64(10.0)));
    let err = ParameterEncoder::encode_execute(0, params).unwrap_err();
    assert!(err.as_string().unwrap_or_default().contains("U16"));
}

#[wasm_bindgen_test]
fn encode_execute_rejects_negative_i64() {
    let params = Array::new();
    params.push(&make_param("i64", JsValue::from_f64(-1.0)));
    let err = ParameterEncoder::encode_execute(0, params).unwrap_err();
    assert!(err.as_string().unwrap_or_default().contains("I64 negative"));
}

#[wasm_bindgen_test]
fn encode_execute_accepts_base58_pubkey_string() {
    let params = Array::new();
    let mut pk_bytes = vec![0u8; 32];
    for (i, b) in pk_bytes.iter_mut().enumerate() {
        *b = (i + 1) as u8;
    }
    let pk_base58 = bs58::encode(&pk_bytes).into_string();
    params.push(&make_param("pubkey", JsValue::from_str(&pk_base58)));

    let encoded = ParameterEncoder::encode_execute(0, params).unwrap();
    let encoded_bytes = encoded.to_vec();

    let mut expected = Vec::new();
    expected.push(types::PUBKEY);
    expected.extend_from_slice(&pk_bytes);

    assert_eq!(encoded_bytes, expected);
}

#[wasm_bindgen_test]
fn encode_execute_rejects_invalid_base58_pubkey_string() {
    let params = Array::new();
    // Invalid base58 character '0' (zero) should fail decode
    params.push(&make_param(
        "pubkey",
        JsValue::from_str("0OIlInvalidBase58"),
    ));
    let err = ParameterEncoder::encode_execute(0, params).unwrap_err();
    assert!(err
        .as_string()
        .unwrap_or_default()
        .contains("Invalid base58"));
}

#[wasm_bindgen_test]
fn encode_execute_rejects_wrong_length_pubkey_string() {
    let params = Array::new();
    let pk_bytes = vec![1u8; 31];
    let pk_base58 = bs58::encode(&pk_bytes).into_string();
    params.push(&make_param("pubkey", JsValue::from_str(&pk_base58)));
    let err = ParameterEncoder::encode_execute(0, params).unwrap_err();
    assert!(err.as_string().unwrap_or_default().contains("32 bytes"));
}

#[wasm_bindgen_test]
fn encode_execute_rejects_non_numeric_account() {
    let params = Array::new();
    let obj = Object::new();
    Reflect::set(&obj, &"isAccount".into(), &JsValue::from_bool(true)).unwrap();
    Reflect::set(&obj, &"value".into(), &JsValue::from_str("not-a-number")).unwrap();
    params.push(&obj.into());
    let err = ParameterEncoder::encode_execute(0, params).unwrap_err();
    assert!(err.as_string().unwrap_or_default().contains("ACCOUNT"));
}

#[wasm_bindgen_test]
fn encode_execute_accepts_sized_string_at_exact_boundary() {
    let params = Array::new();
    params.push(&make_sized_string_param("abcd", 4));

    let encoded = ParameterEncoder::encode_execute(0, params).unwrap();
    let encoded_bytes = encoded.to_vec();

    let mut expected = Vec::new();
    expected.push(types::STRING);
    expected.extend_from_slice(&(4u32).to_le_bytes());
    expected.extend_from_slice(b"abcd");

    assert_eq!(encoded_bytes, expected);
}

#[wasm_bindgen_test]
fn encode_execute_rejects_sized_string_when_over_boundary() {
    let params = Array::new();
    params.push(&make_sized_string_param("abcde", 4));

    let err = ParameterEncoder::encode_execute(0, params).unwrap_err();
    let msg = err.as_string().unwrap_or_default();
    assert!(msg.contains("exceeds declared size"));
    assert!(msg.contains("max 4"));
}

#[wasm_bindgen_test]
fn encode_execute_enforces_utf8_byte_length_for_sized_string() {
    let params = Array::new();
    // "é" is 2 bytes in UTF-8
    params.push(&make_sized_string_param("éé", 4)); // exactly 4 bytes
    let ok = ParameterEncoder::encode_execute(0, params);
    assert!(ok.is_ok());

    let over = Array::new();
    over.push(&make_sized_string_param("ééé", 4)); // 6 bytes > 4
    let err = ParameterEncoder::encode_execute(0, over).unwrap_err();
    let msg = err.as_string().unwrap_or_default();
    assert!(msg.contains("exceeds declared size"));
    assert!(msg.contains("max 4"));
}
