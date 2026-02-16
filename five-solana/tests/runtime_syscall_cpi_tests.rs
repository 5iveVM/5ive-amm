//! Syscall/CPI runtime behavior is covered in BPF suites.
//! Keep only fast payload-shape checks in-process.

mod harness;

use harness::fixtures::{canonical_execute_payload, TypedParam};

#[test]
fn payload_can_encode_syscall_style_params() {
    let payload = canonical_execute_payload(1, &[TypedParam::U64(42)]);
    assert!(payload.len() > 8);
    assert_eq!(&payload[0..4], &1u32.to_le_bytes());
}
