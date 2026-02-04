#[test]
fn test_disassemble_register_opcodes() {
    use five_dsl_compiler::bytecode_generator::disassembler::disassemble;
    use five_protocol::opcodes;

    // Create a simple bytecode with register opcodes to test disassembler
    // This tests that all register opcode decode arms are working
    let mut bytes = vec![
        // Minimal header
        0x35, 0x49, 0x56, 0x45, // "5IVE" magic
        0x0F, 0x01, 0x00, 0x00, // features and function counts
        0x00, 0x00, // metadata size
    ];

    // Now add register opcodes one by one
    // LOAD_REG_U8 (0xB0): opcode + reg + value
    bytes.push(opcodes::LOAD_REG_U8);  // 0xB0
    bytes.push(0x01);  // register 1
    bytes.push(0x42);  // value 0x42

    // LOAD_REG_U64 (0xB2): opcode + reg + 8 bytes
    bytes.push(opcodes::LOAD_REG_U64); // 0xB2
    bytes.push(0x02);  // register 2
    bytes.extend_from_slice(&0x1234567890ABCDEFu64.to_le_bytes());  // 8 bytes

    // PUSH_REG (0xBC): opcode + reg
    bytes.push(opcodes::PUSH_REG);  // 0xBC
    bytes.push(0x01);  // register 1

    // ADD_REG (0xB5): opcode + dest + src1 + src2
    bytes.push(opcodes::ADD_REG);  // 0xB5
    bytes.push(0x00);  // dest register 0
    bytes.push(0x01);  // src1 register 1
    bytes.push(0x02);  // src2 register 2

    // REQUIRE_GTE_U64 (0xC0): acc + offset(VLE) + param
    bytes.push(opcodes::REQUIRE_GTE_U64);  // 0xC0
    bytes.push(0x01);  // account 1
    bytes.push(0x08);  // offset (VLE encoded, 1 byte)
    bytes.push(0x00);  // param 0

    // STORE_FIELD_REG (0xCD): reg + acc + offset(VLE)
    bytes.push(opcodes::STORE_FIELD_REG);  // 0xCD
    bytes.push(0x03);  // register 3
    bytes.push(0x02);  // account 2
    bytes.push(0x10);  // offset (VLE encoded, 1 byte)

    // HALT
    bytes.push(opcodes::HALT);  // 0x00

    // Disassemble
    let lines = disassemble(&bytes);

    // Verify that the output contains register opcodes
    let output = lines.join("\n");
    println!("Disassembly output:\n{}", output);

    // Check for register operations
    assert!(output.contains("LOAD_REG_U8"), "Should contain LOAD_REG_U8");
    assert!(output.contains("LOAD_REG_U64"), "Should contain LOAD_REG_U64");
    assert!(output.contains("PUSH_REG"), "Should contain PUSH_REG");
    assert!(output.contains("ADD_REG"), "Should contain ADD_REG");

    // Check for fused operations
    assert!(output.contains("REQUIRE_GTE_U64"), "Should contain REQUIRE_GTE_U64");
    assert!(output.contains("STORE_FIELD_REG"), "Should contain STORE_FIELD_REG");

    // Verify no truncation errors in disassembly
    let truncation_count = output.matches("<truncated>").count();
    println!("\nTruncation errors: {}", truncation_count);
    assert_eq!(truncation_count, 0, "Disassembly should not have truncation errors");

    println!("\nDisassembly successful! Total lines: {}", lines.len());
}
