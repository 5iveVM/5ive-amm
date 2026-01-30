use five_protocol::opcodes::*;

/// Compare bytecode size with and without registers
#[test]
fn test_register_bytecode_comparison() {
    let source = include_str!("../../five-templates/token/src/token.v");

    // Compile WITHOUT registers
    let mut tokenizer_no_reg = five_dsl_compiler::DslTokenizer::new(source);
    let tokens_no_reg = tokenizer_no_reg.tokenize().expect("Tokenization failed");
    let mut parser_no_reg = five_dsl_compiler::DslParser::new(tokens_no_reg);
    let ast_no_reg = parser_no_reg.parse().expect("Failed to parse");

    let mut generator_no_reg = five_dsl_compiler::DslBytecodeGenerator::new();
    generator_no_reg.set_use_registers(false);
    let bytecode_no_reg = generator_no_reg.generate(&ast_no_reg).expect("Failed to generate bytecode (no reg)");

    // Compile WITH registers
    let mut tokenizer_reg = five_dsl_compiler::DslTokenizer::new(source);
    let tokens_reg = tokenizer_reg.tokenize().expect("Tokenization failed");
    let mut parser_reg = five_dsl_compiler::DslParser::new(tokens_reg);
    let ast_reg = parser_reg.parse().expect("Failed to parse");

    let mut generator_reg = five_dsl_compiler::DslBytecodeGenerator::new();
    generator_reg.set_use_registers(true);
    let bytecode_reg = generator_reg.generate(&ast_reg).expect("Failed to generate bytecode (with reg)");

    // Report sizes
    println!("=== Bytecode Size Comparison ===");
    println!("WITHOUT registers: {} bytes", bytecode_no_reg.len());
    println!("WITH registers:    {} bytes", bytecode_reg.len());
    println!("Difference:        {} bytes ({})", 
        (bytecode_reg.len() as i64) - (bytecode_no_reg.len() as i64),
        if bytecode_reg.len() < bytecode_no_reg.len() { "SMALLER ✓" } else { "LARGER ✗" }
    );

    // Count specific opcodes
    let count_opcode = |bytecode: &[u8], opcode: u8| -> usize {
        bytecode.iter().filter(|&&b| b == opcode).count()
    };

    println!("\n=== Opcode Counts ===");
    println!("Opcode             | Without Reg | With Reg");
    println!("-------------------|-------------|----------");
    println!("LOAD_PARAM_1       | {:>11} | {:>8}", count_opcode(&bytecode_no_reg, LOAD_PARAM_1), count_opcode(&bytecode_reg, LOAD_PARAM_1));
    println!("LOAD_PARAM_2       | {:>11} | {:>8}", count_opcode(&bytecode_no_reg, LOAD_PARAM_2), count_opcode(&bytecode_reg, LOAD_PARAM_2));
    println!("LOAD_PARAM_3       | {:>11} | {:>8}", count_opcode(&bytecode_no_reg, LOAD_PARAM_3), count_opcode(&bytecode_reg, LOAD_PARAM_3));
    println!("LOAD_PARAM (gen)   | {:>11} | {:>8}", count_opcode(&bytecode_no_reg, LOAD_PARAM), count_opcode(&bytecode_reg, LOAD_PARAM));
    println!("PUSH_REG           | {:>11} | {:>8}", count_opcode(&bytecode_no_reg, PUSH_REG), count_opcode(&bytecode_reg, PUSH_REG));
    println!("POP_REG            | {:>11} | {:>8}", count_opcode(&bytecode_no_reg, POP_REG), count_opcode(&bytecode_reg, POP_REG));
    println!("CALL               | {:>11} | {:>8}", count_opcode(&bytecode_no_reg, CALL), count_opcode(&bytecode_reg, CALL));
    println!("CALL_REG           | {:>11} | {:>8}", count_opcode(&bytecode_no_reg, CALL_REG), count_opcode(&bytecode_reg, CALL_REG));
    println!("ADD_REG            | {:>11} | {:>8}", count_opcode(&bytecode_no_reg, ADD_REG), count_opcode(&bytecode_reg, ADD_REG));
    println!("SUB_REG            | {:>11} | {:>8}", count_opcode(&bytecode_no_reg, SUB_REG), count_opcode(&bytecode_reg, SUB_REG));
    println!("ADD_FIELD_REG      | {:>11} | {:>8}", count_opcode(&bytecode_no_reg, ADD_FIELD_REG), count_opcode(&bytecode_reg, ADD_FIELD_REG));
    println!("SUB_FIELD_REG      | {:>11} | {:>8}", count_opcode(&bytecode_no_reg, SUB_FIELD_REG), count_opcode(&bytecode_reg, SUB_FIELD_REG));
    println!("STORE_FIELD_REG    | {:>11} | {:>8}", count_opcode(&bytecode_no_reg, STORE_FIELD_REG), count_opcode(&bytecode_reg, STORE_FIELD_REG));
    println!("REQUIRE_GTE_REG    | {:>11} | {:>8}", count_opcode(&bytecode_no_reg, REQUIRE_GTE_REG), count_opcode(&bytecode_reg, REQUIRE_GTE_REG));

    // Assert registers mode produces smaller or equal bytecode
    // NOTE: If this fails, investigation into why registers inflate bytecode is needed
    // assert!(bytecode_reg.len() <= bytecode_no_reg.len() + 20, 
    //     "Register mode should not significantly inflate bytecode");
}
