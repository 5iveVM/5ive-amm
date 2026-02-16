//! Runtime fee and validation behavior is covered in BPF tests:
//! - runtime_bpf_fee_flow_tests
//! - runtime_bpf_invocation_semantics_tests

mod harness;

use harness::fixtures::canonical_execute_payload;

#[test]
fn canonical_execute_payload_header_is_stable() {
    let payload = canonical_execute_payload(7, &[]);
    assert_eq!(&payload[0..4], &7u32.to_le_bytes());
    assert_eq!(&payload[4..8], &0u32.to_le_bytes());
}
