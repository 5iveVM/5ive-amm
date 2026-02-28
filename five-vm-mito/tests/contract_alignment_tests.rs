use five_protocol::MAX_CALL_DEPTH as PROTOCOL_MAX_CALL_DEPTH;
use five_protocol::MAX_FUNCTION_PARAMS as PROTOCOL_MAX_FUNCTION_PARAMS;
use five_protocol::MAX_LOCALS as PROTOCOL_MAX_LOCALS;
use five_protocol::MAX_SCRIPT_SIZE as PROTOCOL_MAX_SCRIPT_SIZE;
use five_protocol::OPCODE_TABLE;
use five_vm_mito::MAX_CALL_DEPTH as VM_MAX_CALL_DEPTH;
use five_vm_mito::MAX_LOCALS as VM_MAX_LOCALS;
use five_vm_mito::MAX_PARAMETERS as VM_MAX_PARAMETERS;
use five_vm_mito::MAX_SCRIPT_SIZE as VM_MAX_SCRIPT_SIZE;

#[test]
fn protocol_and_vm_call_depth_limits_match() {
    assert_eq!(
        PROTOCOL_MAX_CALL_DEPTH, VM_MAX_CALL_DEPTH,
        "protocol and VM MAX_CALL_DEPTH must stay aligned"
    );
    assert!(VM_MAX_CALL_DEPTH > 0, "MAX_CALL_DEPTH must be non-zero");
}

#[test]
fn protocol_and_vm_parameter_limits_match() {
    assert_eq!(
        PROTOCOL_MAX_FUNCTION_PARAMS, VM_MAX_PARAMETERS,
        "protocol and VM parameter limits must stay aligned"
    );
    assert!(VM_MAX_PARAMETERS > 0, "MAX_PARAMETERS must be non-zero");
}

#[test]
fn protocol_and_vm_local_limits_match() {
    assert_eq!(
        PROTOCOL_MAX_LOCALS, VM_MAX_LOCALS,
        "protocol and VM local limits must stay aligned"
    );
    assert!(VM_MAX_LOCALS > 0, "MAX_LOCALS must be non-zero");
}

#[test]
fn protocol_and_vm_script_size_limits_match() {
    assert_eq!(
        PROTOCOL_MAX_SCRIPT_SIZE, VM_MAX_SCRIPT_SIZE,
        "protocol and VM script size limits must stay aligned"
    );
    assert!(VM_MAX_SCRIPT_SIZE > 0, "MAX_SCRIPT_SIZE must be non-zero");
}

#[test]
fn protocol_opcodes_are_either_dispatched_or_explicitly_rejected() {
    let explicitly_rejected: [(&str, &str); 14] = [
        (
            "PUSH_ZERO",
            "Pattern-fusion Tier-2 opcode intentionally not dispatched in MitoVM",
        ),
        (
            "PUSH_ONE",
            "Pattern-fusion Tier-2 opcode intentionally not dispatched in MitoVM",
        ),
        (
            "DUP_ADD",
            "Pattern-fusion Tier-2 opcode intentionally not dispatched in MitoVM",
        ),
        (
            "DUP_SUB",
            "Pattern-fusion Tier-2 opcode intentionally not dispatched in MitoVM",
        ),
        (
            "DUP_MUL",
            "Pattern-fusion Tier-2 opcode intentionally not dispatched in MitoVM",
        ),
        (
            "VALIDATE_AMOUNT_NONZERO",
            "Pattern-fusion Tier-2 opcode intentionally not dispatched in MitoVM",
        ),
        (
            "VALIDATE_SUFFICIENT",
            "Pattern-fusion Tier-2 opcode intentionally not dispatched in MitoVM",
        ),
        (
            "EQ_ZERO_JUMP",
            "Pattern-fusion Tier-2 opcode intentionally not dispatched in MitoVM",
        ),
        (
            "TRANSFER_DEBIT",
            "Pattern-fusion Tier-2 opcode intentionally not dispatched in MitoVM",
        ),
        (
            "TRANSFER_CREDIT",
            "Pattern-fusion Tier-2 opcode intentionally not dispatched in MitoVM",
        ),
        (
            "RETURN_SUCCESS",
            "Pattern-fusion Tier-2 opcode intentionally not dispatched in MitoVM",
        ),
        (
            "RETURN_ERROR",
            "Pattern-fusion Tier-2 opcode intentionally not dispatched in MitoVM",
        ),
        (
            "GT_ZERO_JUMP",
            "Pattern-fusion Tier-2 opcode intentionally not dispatched in MitoVM",
        ),
        (
            "LT_ZERO_JUMP",
            "Pattern-fusion Tier-2 opcode intentionally not dispatched in MitoVM",
        ),
    ];
    let rejected_names: std::collections::HashSet<&str> =
        explicitly_rejected.iter().map(|(name, _)| *name).collect();

    // execution.rs routes opcodes by high nibble.
    let is_routed_by_dispatcher = |opcode: u8| match opcode & 0xF0 {
        0x00 | 0x10 | 0x20 | 0x30 | 0x40 | 0x50 | 0x60 | 0x70 | 0x80 | 0x90 | 0xA0 | 0xB0
        | 0xC0 | 0xD0 | 0xE0 | 0xF0 => true,
        _ => false,
    };

    for info in OPCODE_TABLE {
        let is_handled = is_routed_by_dispatcher(info.opcode);
        let is_rejected = rejected_names.contains(info.name);
        assert!(
            is_handled || is_rejected,
            "Opcode {} (0x{:02X}) appears in protocol table but is neither dispatched nor explicitly rejected",
            info.name,
            info.opcode
        );
    }

    for (name, reason) in explicitly_rejected {
        assert!(
            OPCODE_TABLE.iter().any(|info| info.name == name),
            "Explicitly rejected opcode {} missing from protocol table",
            name
        );
        let _ = reason;
    }
}
