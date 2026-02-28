use five_protocol::{
    get_opcode_info,
    is_valid_opcode,
    opcode_compute_cost,
    opcode_name,
    operand_size,
    parser,
    BytecodeBuilder,
    ParseError,
    // Import some key opcodes to test against
    ADD,
    ALLOC_LOCALS,
    BR_EQ_U8,
    CAST,
    CREATE_TUPLE,
    HALT,
    JUMP,
    LOAD,
    OPCODE_TABLE,
    PUSH_ARRAY_LITERAL,
    PUSH_STRING,
    PUSH_STRING_LITERAL,
    PUSH_U16,
    PUSH_U16_W,
    RETURN,
    STORE,
};
use std::collections::HashSet;

#[test]
fn test_opcode_table_integrity() {
    let mut opcodes = HashSet::new();

    for info in OPCODE_TABLE.iter() {
        // Check for duplicates
        if !opcodes.insert(info.opcode) {
            panic!(
                "Duplicate opcode definition found: {} ({:#04x})",
                info.name, info.opcode
            );
        }

        // Check that name is not empty
        assert!(
            !info.name.is_empty(),
            "Opcode {:#04x} has empty name",
            info.opcode
        );

        // Compute cost should be reasonable (at least 1)
        assert!(
            info.compute_cost >= 1,
            "Opcode {} has invalid compute cost {}",
            info.name,
            info.compute_cost
        );
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
        assert_eq!(
            info.opcode, opcode,
            "Opcode constant {} mismatch with table",
            name
        );
        assert_eq!(info.name, name, "Opcode name mismatch for {}", name);
    };

    check_opcode(ADD, "ADD");
    check_opcode(JUMP, "JUMP");
    check_opcode(RETURN, "RETURN");
    // check_opcode(PUSH_U8, "PUSH_U8"); // PUSH_U8 is not imported, let's stick to imported ones
}

#[test]
fn test_call_native_in_opcode_table() {
    // Test that CALL_NATIVE (0x92) is correctly in the opcode table
    const CALL_NATIVE: u8 = 0x92;
    let info = get_opcode_info(CALL_NATIVE);
    assert!(
        info.is_some(),
        "CALL_NATIVE (0x92) should be in opcode table"
    );
    let info = info.unwrap();
    assert_eq!(info.name, "CALL_NATIVE");
    assert_eq!(info.opcode, CALL_NATIVE);
    println!("CALL_NATIVE lookup successful: {:?}", info.name);
}

#[test]
fn test_operand_size_uses_canonical_fixed_and_variable_widths() {
    assert_eq!(operand_size(PUSH_U16, &[0x34, 0x12], false), Some(2));
    assert_eq!(operand_size(PUSH_U16, &[0x07], true), Some(1));
    assert_eq!(operand_size(PUSH_U16_W, &[0x34, 0x12], true), Some(2));

    assert_eq!(operand_size(CREATE_TUPLE, &[0x03], false), Some(1));
    assert_eq!(operand_size(ALLOC_LOCALS, &[0x04], false), Some(1));
    assert_eq!(operand_size(BR_EQ_U8, &[0x02, 0x34, 0x12], false), Some(3));
    assert_eq!(
        operand_size(STORE, &[0x01, 0x44, 0x33, 0x22, 0x11], false),
        Some(5)
    );
    assert_eq!(operand_size(LOAD, &[], false), Some(0));
    assert_eq!(operand_size(CAST, &[0x01], false), Some(1));

    // PUSH_STRING uses u32 length prefix + bytes.
    assert_eq!(
        operand_size(
            PUSH_STRING,
            &[0x03, 0x00, 0x00, 0x00, b'a', b'b', b'c'],
            false
        ),
        Some(7)
    );
    assert_eq!(operand_size(PUSH_STRING, &[0x03, 0x00], false), None);

    // Literal builders use a single immediate count byte in bytecode.
    assert_eq!(operand_size(PUSH_ARRAY_LITERAL, &[0x04], false), Some(1));
    assert_eq!(operand_size(PUSH_STRING_LITERAL, &[0x03], false), Some(1));
}

#[test]
fn parser_advances_correctly_after_literal_builder_opcodes() {
    let script = {
        let mut b = BytecodeBuilder::new();
        b.emit_header(1, 1)
            .emit_opcode(PUSH_ARRAY_LITERAL)
            .emit_u8(0x04)
            .emit_halt()
            .emit_opcode(PUSH_STRING_LITERAL)
            .emit_u8(0x02)
            .emit_halt();
        b.build()
    };

    let parsed = parser::parse_bytecode(&script);
    assert!(
        parsed.errors.is_empty(),
        "parser errors: {:?}",
        parsed.errors
    );
    assert_eq!(parsed.instructions.len(), 4);
    assert_eq!(parsed.instructions[0].opcode, PUSH_ARRAY_LITERAL);
    assert_eq!(parsed.instructions[0].size, 2);
    assert_eq!(parsed.instructions[1].opcode, HALT);
    assert_eq!(parsed.instructions[1].size, 1);
    assert_eq!(parsed.instructions[2].opcode, PUSH_STRING_LITERAL);
    assert_eq!(parsed.instructions[2].size, 2);
    assert_eq!(parsed.instructions[3].opcode, HALT);
    assert_eq!(parsed.instructions[3].size, 1);
}

#[test]
fn parser_advances_correctly_after_cast_immediate() {
    let script = {
        let mut b = BytecodeBuilder::new();
        b.emit_header(1, 1)
            .emit_opcode(CAST)
            .emit_u8(0x01)
            .emit_opcode(HALT);
        b.build()
    };

    let parsed = parser::parse_bytecode(&script);
    assert!(
        parsed.errors.is_empty(),
        "parser errors: {:?}",
        parsed.errors
    );
    assert_eq!(parsed.instructions.len(), 2);
    assert_eq!(parsed.instructions[0].opcode, CAST);
    assert_eq!(parsed.instructions[0].size, 2);
    assert_eq!(parsed.instructions[1].opcode, HALT);
    assert_eq!(parsed.instructions[1].size, 1);
}

#[test]
fn parser_uses_canonical_widths_for_corrected_opcodes() {
    let script = {
        let mut b = BytecodeBuilder::new();
        b.emit_header(1, 1)
            .emit_opcode(ALLOC_LOCALS)
            .emit_u8(2)
            .emit_opcode(CREATE_TUPLE)
            .emit_u8(3)
            .emit_opcode(STORE)
            .emit_u8(1)
            .emit_u32(0x1122_3344)
            .emit_opcode(LOAD)
            .emit_halt();
        b.build()
    };

    let parsed = parser::parse_bytecode(&script);
    assert!(
        parsed.errors.is_empty(),
        "parser errors: {:?}",
        parsed.errors
    );

    let alloc = parsed.instructions[0];
    assert_eq!(alloc.opcode, ALLOC_LOCALS);
    assert_eq!(alloc.size, 2);
    assert_eq!(alloc.arg1, 2);

    let tuple = parsed.instructions[1];
    assert_eq!(tuple.opcode, CREATE_TUPLE);
    assert_eq!(tuple.size, 2);
    assert_eq!(tuple.arg1, 3);

    let store = parsed.instructions[2];
    assert_eq!(store.opcode, STORE);
    assert_eq!(store.size, 6);
    assert_eq!(store.arg1, 1);
    assert_eq!(store.arg2, 0x1122_3344);

    let load = parsed.instructions[3];
    assert_eq!(load.opcode, LOAD);
    assert_eq!(load.size, 1);
    assert_eq!(load.arg1, 0);
    assert_eq!(load.arg2, 0);
}

#[test]
fn parser_rejects_legacy_short_store_operand_form() {
    let script = {
        let mut b = BytecodeBuilder::new();
        b.emit_header(1, 1)
            .emit_opcode(STORE)
            .emit_u8(1)
            .emit_u8(0x7f)
            .emit_halt();
        b.build()
    };

    let parsed = parser::parse_bytecode(&script);
    assert!(
        parsed.errors.contains(&ParseError::InstructionOutOfBounds),
        "expected canonical parser to reject short STORE operand form: {:?}",
        parsed.errors
    );
}

#[test]
fn parser_decodes_br_eq_u8_compare_and_offset() {
    let script = {
        let mut b = BytecodeBuilder::new();
        b.emit_header(1, 1)
            .emit_opcode(BR_EQ_U8)
            .emit_u8(7)
            .emit_u16(0x0012)
            .emit_halt();
        b.build()
    };

    let parsed = parser::parse_bytecode(&script);
    assert!(
        parsed.errors.is_empty(),
        "parser errors: {:?}",
        parsed.errors
    );
    let br = parsed.instructions[0];
    assert_eq!(br.opcode, BR_EQ_U8);
    assert_eq!(br.size, 4);
    assert_eq!(br.arg1, 7);
    assert_eq!(br.arg2, 0x0012);
}
