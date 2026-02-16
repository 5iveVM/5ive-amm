//! Script fixture runtime behavior is covered by BPF tests.
//! In-process fixtures are trimmed to serialization/shape checks.

mod harness;

use harness::fixtures::{canonical_execute_payload, TypedParam};

#[test]
fn fixture_payload_encoding_supports_strings() {
    let payload = canonical_execute_payload(0, &[TypedParam::String("abc".to_string())]);
    assert!(payload.len() > 8);
}
