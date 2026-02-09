use five_protocol::MAX_CALL_DEPTH as PROTOCOL_MAX_CALL_DEPTH;
use five_vm_mito::MAX_CALL_DEPTH as VM_MAX_CALL_DEPTH;

#[test]
fn protocol_and_vm_call_depth_limits_match() {
    assert_eq!(
        PROTOCOL_MAX_CALL_DEPTH,
        VM_MAX_CALL_DEPTH,
        "protocol and VM MAX_CALL_DEPTH must stay aligned"
    );
    assert!(VM_MAX_CALL_DEPTH > 0, "MAX_CALL_DEPTH must be non-zero");
}
