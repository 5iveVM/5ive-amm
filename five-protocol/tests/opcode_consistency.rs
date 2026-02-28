#[cfg(test)]
mod tests {
    use five_protocol::opcodes::{self, get_opcode_info, OPCODE_TABLE};
    use std::collections::HashSet;

    #[test]
    fn test_opcode_table_integrity() {
        let mut seen_opcodes = HashSet::new();
        let mut seen_names = HashSet::new();

        for info in OPCODE_TABLE {
            // Check for duplicate opcode values
            if !seen_opcodes.insert(info.opcode) {
                panic!(
                    "Duplicate opcode value found: 0x{:02X} ({})",
                    info.opcode, info.name
                );
            }

            // Check for duplicate names
            if !seen_names.insert(info.name) {
                panic!("Duplicate opcode name found: {}", info.name);
            }

            // Check lookup consistency
            let lookup = get_opcode_info(info.opcode);
            assert!(
                lookup.is_some(),
                "Opcode 0x{:02X} not found in lookup",
                info.opcode
            );
            let lookup_info = lookup.unwrap();
            assert_eq!(lookup_info.opcode, info.opcode);
            assert_eq!(lookup_info.name, info.name);
        }
    }

    #[test]
    fn test_opcode_ranges() {
        // Verify known opcodes are in correct ranges
        let check_range = |opcode: u8, name: &str, expected_min: u8, expected_max: u8| {
            if opcode < expected_min || opcode > expected_max {
                panic!(
                    "Opcode {} (0x{:02X}) is out of expected range [0x{:02X}, 0x{:02X}]",
                    name, opcode, expected_min, expected_max
                );
            }
        };

        check_range(opcodes::HALT, "HALT", 0x00, 0x0F);
        check_range(opcodes::ADD, "ADD", 0x20, 0x2F);
        check_range(opcodes::STORE, "STORE", 0x40, 0x4F);
        check_range(opcodes::CREATE_ACCOUNT, "CREATE_ACCOUNT", 0x50, 0x5F);
        // Note: Some ranges have been merged/moved, so we just check a few key anchors
    }

    #[test]
    fn test_protocol_constants_match_table() {
        // This test manually checks a few constants to ensure they match the table.
        // It's not exhaustive but catches regressions in the most common opcodes.
        let check_const = |opcode: u8, name: &str| {
            let info = get_opcode_info(opcode).expect(&format!("Opcode {} not in table", name));
            assert_eq!(info.name, name, "Name mismatch for opcode 0x{:02X}", opcode);
        };

        check_const(opcodes::HALT, "HALT");
        check_const(opcodes::JUMP, "JUMP");
        check_const(opcodes::PUSH_U64, "PUSH_U64");
        check_const(opcodes::ADD, "ADD");
        check_const(opcodes::SUB, "SUB");
        check_const(opcodes::LOAD_FIELD, "LOAD_FIELD");
        check_const(opcodes::STORE_FIELD, "STORE_FIELD");
        check_const(opcodes::INVOKE, "INVOKE");
        check_const(opcodes::CALL, "CALL");
    }
}
