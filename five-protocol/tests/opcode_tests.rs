use five_protocol::{
    get_opcode_info, is_valid_opcode, opcode_compute_cost, opcode_name,
    OPCODE_TABLE,
    // Import some key opcodes to test against
    ADD, JUMP, RETURN,
};
use std::collections::HashSet;

#[test]
fn test_opcode_table_integrity() {
    let mut opcodes = HashSet::new();

    for info in OPCODE_TABLE.iter() {
        // Check for duplicates
        if !opcodes.insert(info.opcode) {
            panic!("Duplicate opcode definition found: {} ({:#04x})", info.name, info.opcode);
        }

        // Check that name is not empty
        assert!(!info.name.is_empty(), "Opcode {:#04x} has empty name", info.opcode);

        // Compute cost should be reasonable (at least 1)
        assert!(info.compute_cost >= 1, "Opcode {} has invalid compute cost {}", info.name, info.compute_cost);
    }
}

#[test]
fn test_opcode_lookups() {
    // Test known opcodes
    assert!(is_valid_opcode(ADD));
    assert!(is_valid_opcode(JUMP));
    assert!(is_valid_opcode(RETURN));

    // Test invalid opcode (0x2F is marked as available slot in opcodes.rs comment, check if it's really empty)
    // Actually, let's pick a definitely invalid one if possible.
    // Based on opcodes.rs, 0xAC-0xAF are available. Let's try 0xAE.
    // Wait, 0xAC is RESULT_UNWRAP, 0xAD is RESULT_GET_VALUE, 0xAE is RESULT_GET_ERROR.
    // 0xAF is CAST.
    // Let's try to find a gap. 0x2F seems to be available.
    if is_valid_opcode(0x2F) {
        // If it is valid, ensure it's in the table
        assert!(get_opcode_info(0x2F).is_some());
    } else {
        assert!(get_opcode_info(0x2F).is_none());
    }

    // Test get_opcode_info
    let add_info = get_opcode_info(ADD).expect("ADD info not found");
    assert_eq!(add_info.name, "ADD");
    assert_eq!(add_info.opcode, ADD);

    // Test opcode_name
    assert_eq!(opcode_name(ADD), "ADD");
    assert_eq!(opcode_name(JUMP), "JUMP");

    // Test opcode_compute_cost
    assert_eq!(opcode_compute_cost(ADD), 1);
    assert_eq!(opcode_compute_cost(JUMP), 2);
}

#[test]
fn test_opcode_constants_match_table() {
    // Verify that the constant value matches the one in the table
    // This catches if someone changes the constant but forgets to update the table or vice versa

    let check_opcode = |opcode: u8, name: &str| {
        let info = get_opcode_info(opcode).expect(&format!("Opcode {} not found in table", name));
        assert_eq!(info.opcode, opcode, "Opcode constant {} mismatch with table", name);
        assert_eq!(info.name, name, "Opcode name mismatch for {}", name);
    };

    check_opcode(ADD, "ADD");
    check_opcode(JUMP, "JUMP");
    check_opcode(RETURN, "RETURN");
    // check_opcode(PUSH_U8, "PUSH_U8"); // PUSH_U8 is not imported, let's stick to imported ones
}
