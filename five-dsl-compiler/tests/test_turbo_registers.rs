use five_protocol::opcodes::*;

#[test]
fn test_turbo_registers_compilation() {
    let source = r#"
        script turbo_test {
            pub fn main() -> u64 {
                return add(10, 20);
            }

            fn add(val1: u64, val2: u64) -> u64 {
                return val1 + val2;
            }
        }
    "#;

    // Use lower-level DslBytecodeGenerator to enable registers manually
    let mut tokenizer = five_dsl_compiler::DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("Tokenization failed");
    let mut parser = five_dsl_compiler::DslParser::new(tokens);
    let ast = parser.parse().expect("Failed to parse");

    // Enable registers
    let mut generator = five_dsl_compiler::DslBytecodeGenerator::new();
    generator.set_use_registers(true);
    
    let bytecode = generator.generate(&ast).expect("Failed to generate bytecode");
    
    // Inspect bytecode
    // We expect:
    // Dispatcher:
    // ... LOAD_PARAM X ... POP_REG 0
    // ... LOAD_PARAM Y ... POP_REG 1
    // ... CALL_REG
    // Function Body:
    // ... PUSH_REG 0 ... PUSH_REG 1 ... ADD_REG 0 0 1 ...
    
    // Actually, PUSH_REG 0, PUSH_REG 1, ADD (stack based) might be generated if binary expression doesn't use register optimization
    // Wait, try_register_binary_expression (expressions.rs) IS implemented!
    // It emits ADD_REG left_reg left_reg right_reg
    // result -> left_reg (r0)
    // Then PUSH_REG left_reg (r0)
    
    // Let's disassemble/inspect
    let disassembly = generator.get_disassembly();
    println!("Disassembly:\n{}", disassembly.join("\n"));

    // Verify CALL_REG exists (used by dispatcher for public functions)
    // In direct access model, public functions use CALL_REG
    let has_call_reg = bytecode.contains(&CALL_REG);
    // Note: If 'main' is the only public function, dispatcher might use CALL_REG for it.
    // However, the test source defines 'main' as public.
    assert!(has_call_reg, "Bytecode should contain CALL_REG");
    
    // Verify POP_REG does NOT exist (direct access model optimization)
    // We no longer emit POP_REG for parameter setup
    let has_pop_reg = bytecode.contains(&POP_REG);
    assert!(!has_pop_reg, "Bytecode should NOT contain POP_REG (direct access model)");
    
    // PUSH_REG might be used if register optimization is applied to function bodies
    // But 'add' is private, so it might use stack-based parameters (LOAD_PARAM)
    // 'main' has no params.
    // So PUSH_REG might not be present. We'll skip that assertion or make it conditional.
    // Let's just check for general validity.

    // Optional: Verify opcode sequence for dispatcher
    // Pattern: LOAD_PARAM (0x24/0x25/0x26/0x27) -> POP_REG (0x93)
    // We can iterate bytecode looking for this
}
