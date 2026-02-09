use five_dsl_compiler::DslCompiler;
use five_dsl_compiler::bytecode_generator::disassembler::BytecodeInspector;
use five_protocol::{
    opcodes, parser, BytecodeBuilder, FIVE_HEADER_OPTIMIZED_SIZE, FIVE_MAGIC,
};

#[test]
fn compiled_bytecode_conforms_to_protocol_header_and_opcode_table() {
    let source = r#"
        fn helper(x: u64) -> u64 {
            return x + 1;
        }

        pub main(value: u64) -> u64 {
            return helper(value);
        }
    "#;

    let bytecode = DslCompiler::compile_dsl(source).expect("dsl should compile");
    assert!(bytecode.len() >= FIVE_HEADER_OPTIMIZED_SIZE);
    assert_eq!(&bytecode[0..4], &FIVE_MAGIC);

    let public_count = bytecode[8];
    let total_count = bytecode[9];
    assert!(public_count <= total_count, "public count must not exceed total count");

    let parsed = parser::parse_bytecode(&bytecode);
    assert!(
        parsed.errors.is_empty(),
        "compiled bytecode should parse cleanly: {:?}",
        parsed.errors
    );

    for inst in &parsed.instructions {
        assert!(
            opcodes::get_opcode_info(inst.opcode).is_some(),
            "opcode 0x{:02X} missing from protocol table",
            inst.opcode
        );
    }
}

#[test]
fn compiled_call_encoding_decodes_cleanly() {
    let source = r#"
        fn helper(x: u64) -> u64 {
            return x + 1;
        }

        pub main(value: u64) -> u64 {
            let y = helper(value);
            return y;
        }
    "#;

    let bytecode = DslCompiler::compile_dsl(source).expect("dsl should compile");
    let parsed = parser::parse_bytecode(&bytecode);
    assert!(parsed.errors.is_empty(), "parser errors: {:?}", parsed.errors);

    let call_instructions = parsed
        .instructions
        .iter()
        .filter(|inst| inst.opcode == opcodes::CALL)
        .collect::<Vec<_>>();
    assert!(
        !call_instructions.is_empty(),
        "expected at least one CALL in compiled bytecode"
    );

    for call in call_instructions {
        assert!(
            (call.arg1 as usize) < bytecode.len(),
            "CALL target should remain in bytecode bounds"
        );
        assert!(call.arg2 <= u8::MAX as u64, "CALL param count must fit in u8");
    }
}

#[test]
fn protocol_parser_accepts_canonical_call_external_encoding() {
    // CALL_EXTERNAL wire format in protocol parser: account_index(u8), offset(u16), param_count(u8)
    let script = {
        let mut b = BytecodeBuilder::new();
        b.emit_header(1, 1)
            .emit_opcode(opcodes::CALL_EXTERNAL)
            .emit_u8(1)
            .emit_u16(0x0020)
            .emit_u8(2)
            .emit_halt();
        b.build()
    };

    let parsed = parser::parse_bytecode(&script);
    assert!(
        parsed.errors.is_empty(),
        "canonical CALL_EXTERNAL encoding should parse: {:?}",
        parsed.errors
    );
}

#[test]
fn inspector_uses_canonical_call_external_instruction_size() {
    let script = {
        let mut b = BytecodeBuilder::new();
        b.emit_header(1, 1)
            .emit_opcode(opcodes::CALL_EXTERNAL)
            .emit_u8(1)
            .emit_u16(0x0020)
            .emit_u8(2)
            .emit_u8(opcodes::PUSH_U8)
            .emit_u8(7)
            .emit_halt();
        b.build()
    };

    // Canonical CALL_EXTERNAL encoding is 5 bytes total:
    // opcode + account_index + offset(u16) + param_count.
    let call_external_offset = FIVE_HEADER_OPTIMIZED_SIZE;
    assert_eq!(
        BytecodeInspector::instruction_size(&script, call_external_offset),
        5
    );

    let inspector = BytecodeInspector::new(&script);
    assert!(
        inspector.contains_opcode(opcodes::PUSH_U8),
        "inspector should not skip instructions that follow CALL_EXTERNAL"
    );
}
