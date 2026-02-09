use five_protocol::MAX_CALL_DEPTH as PROTOCOL_MAX_CALL_DEPTH;
use five_protocol::MAX_FUNCTION_PARAMS as PROTOCOL_MAX_FUNCTION_PARAMS;
use five_protocol::MAX_LOCALS as PROTOCOL_MAX_LOCALS;
use five_protocol::MAX_SCRIPT_SIZE as PROTOCOL_MAX_SCRIPT_SIZE;
use five_vm_mito::MAX_CALL_DEPTH as VM_MAX_CALL_DEPTH;
use five_vm_mito::MAX_LOCALS as VM_MAX_LOCALS;
use five_vm_mito::MAX_PARAMETERS as VM_MAX_PARAMETERS;
use five_vm_mito::MAX_SCRIPT_SIZE as VM_MAX_SCRIPT_SIZE;

#[test]
fn protocol_and_vm_call_depth_limits_match() {
    assert_eq!(
        PROTOCOL_MAX_CALL_DEPTH,
        VM_MAX_CALL_DEPTH,
        "protocol and VM MAX_CALL_DEPTH must stay aligned"
    );
    assert!(VM_MAX_CALL_DEPTH > 0, "MAX_CALL_DEPTH must be non-zero");
}

#[test]
fn protocol_and_vm_parameter_limits_match() {
    assert_eq!(
        PROTOCOL_MAX_FUNCTION_PARAMS,
        VM_MAX_PARAMETERS,
        "protocol and VM parameter limits must stay aligned"
    );
    assert!(VM_MAX_PARAMETERS > 0, "MAX_PARAMETERS must be non-zero");
}

#[test]
fn protocol_and_vm_local_limits_match() {
    assert_eq!(
        PROTOCOL_MAX_LOCALS,
        VM_MAX_LOCALS,
        "protocol and VM local limits must stay aligned"
    );
    assert!(VM_MAX_LOCALS > 0, "MAX_LOCALS must be non-zero");
}

#[test]
fn protocol_and_vm_script_size_limits_match() {
    assert_eq!(
        PROTOCOL_MAX_SCRIPT_SIZE,
        VM_MAX_SCRIPT_SIZE,
        "protocol and VM script size limits must stay aligned"
    );
    assert!(VM_MAX_SCRIPT_SIZE > 0, "MAX_SCRIPT_SIZE must be non-zero");
}
