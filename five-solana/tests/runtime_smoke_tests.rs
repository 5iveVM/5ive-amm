//! In-process smoke tests are intentionally minimal.
//! Runtime behavior truth is covered in BPF ProgramTest suites.

mod harness;

use harness::fixtures::canonical_execute_payload;

#[test]
fn execute_payload_encoding_smoke() {
    let payload = canonical_execute_payload(0, &[]);
    assert_eq!(payload.len(), 8);
    assert_eq!(&payload[0..4], &0u32.to_le_bytes());
    assert_eq!(&payload[4..8], &0u32.to_le_bytes());
}
