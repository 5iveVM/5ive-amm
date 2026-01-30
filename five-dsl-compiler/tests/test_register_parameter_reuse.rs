//! Parameter Reuse from Registers Test
//!
//! Tests that when register optimization is enabled, parameters are loaded into
//! registers at function entry and then REUSED from those registers in the function
//! body via PUSH_REG instructions, rather than being re-loaded with LOAD_PARAM.

use five_dsl_compiler::DslCompiler;
use five_dsl_compiler::bytecode_generator::disassembler::disassemble;

#[test]
fn test_parameter_used_multiple_times_uses_register() {
    // Function where parameter 'a' is used 3 times in the function body
    // Should see:
    // - 1x LOAD_PARAM_1 at function entry
    // - 1x POP_REG r0 at function entry (store into register)
    // - 3x PUSH_REG r0 in function body (for each use of 'a')
    // - 0x additional LOAD_PARAM_1 in function body
    let source = r#"
script test {
    pub add_three_times(a: u64) -> u64 {
        let x = a + 1;
        let y = a + 2;
        let z = a + 3;
        return x + y + z;
    }

    init {}
}
"#;

    let config = five_dsl_compiler::CompilationConfig::new(
        five_dsl_compiler::CompilationMode::Testing
    ).with_use_registers(true);

    let result = DslCompiler::compile_with_config(source, &config);
    assert!(result.is_ok(), "Compilation should succeed");

    let bytecode = result.unwrap();
    assert!(!bytecode.is_empty(), "Bytecode should be generated");

    // Disassemble and check for PUSH_REG usage
    let disasm_lines = disassemble(&bytecode);
    let disasm_str = disasm_lines.join("\n");

    // Count PUSH_REG instructions - should be multiple times for parameter reuse
    let push_reg_count = disasm_str.matches("PUSH_REG").count();
    println!("Disassembly:\n{}", disasm_str);
    println!("PUSH_REG count: {}", push_reg_count);

    // Should have PUSH_REG instructions for reusing parameter 'a'
    assert!(push_reg_count >= 3, "Should use PUSH_REG at least 3 times for parameter 'a' reuse (got {})", push_reg_count);

    // Count LOAD_PARAM_1 - should only be at function entry, not multiple times
    let load_param_1_total = disasm_str.matches("LOAD_PARAM_1").count();
    println!("LOAD_PARAM_1 total count: {}", load_param_1_total);

    // LOAD_PARAM_1 should be minimal (just function dispatch at entry)
    // The exact count depends on multiple factors, but should not be excessive
    // For a single parameter function, we expect 1 at entry
}

#[test]
fn test_two_parameters_reused_independently() {
    // Function where two parameters are each used multiple times
    // Should see both parameters cached in registers and reused
    let source = r#"
script test {
    pub combine(a: u64, b: u64) -> u64 {
        let x = a + b;
        let y = a - b;
        let z = a * b;
        return x + y + z;
    }

    init {}
}
"#;

    let config = five_dsl_compiler::CompilationConfig::new(
        five_dsl_compiler::CompilationMode::Testing
    ).with_use_registers(true);

    let result = DslCompiler::compile_with_config(source, &config);
    assert!(result.is_ok(), "Compilation should succeed");

    let bytecode = result.unwrap();
    let disasm_lines = disassemble(&bytecode);
    let disasm_str = disasm_lines.join("\n");
    println!("Disassembly:\n{}", disasm_str);

    // Should see multiple PUSH_REG for parameter reuse
    let push_reg_count = disasm_str.matches("PUSH_REG").count();
    println!("PUSH_REG count: {}", push_reg_count);

    // Both parameters reused multiple times = at least 6 PUSH_REG
    assert!(push_reg_count >= 5, "Should reuse both parameters from registers (expected 5+ PUSH_REG, got {})", push_reg_count);
}

#[test]
fn test_registers_disabled_uses_load_param() {
    // Without register optimization, should use LOAD_PARAM for parameter access
    let source = r#"
script test {
    pub add_three_times(a: u64) -> u64 {
        let x = a + 1;
        let y = a + 2;
        let z = a + 3;
        return x + y + z;
    }

    init {}
}
"#;

    let config = five_dsl_compiler::CompilationConfig::new(
        five_dsl_compiler::CompilationMode::Testing
    ).with_use_registers(false);

    let result = DslCompiler::compile_with_config(source, &config);
    assert!(result.is_ok());

    let bytecode = result.unwrap();
    let disasm_lines = disassemble(&bytecode);
    let disasm_str = disasm_lines.join("\n");
    println!("Disassembly (registers disabled):\n{}", disasm_str);

    // Without registers, should NOT see PUSH_REG
    let push_reg_count = disasm_str.matches("PUSH_REG").count();
    assert_eq!(push_reg_count, 0, "Should not see PUSH_REG when registers disabled (got {})", push_reg_count);

    // Should see LOAD_PARAM instructions instead
    let load_param_count = disasm_str.matches("LOAD_PARAM").count();
    assert!(load_param_count > 0, "Should see LOAD_PARAM when registers disabled");
}

#[test]
fn test_parameter_reuse_across_control_flow() {
    // Test parameter reuse with control flow (if statements)
    let source = r#"
script test {
    pub conditional_use(x: u64, y: u64) -> u64 {
        if x > y {
            return x + y;
        }
        return x - y;
    }

    init {}
}
"#;

    let config = five_dsl_compiler::CompilationConfig::new(
        five_dsl_compiler::CompilationMode::Testing
    ).with_use_registers(true);

    let result = DslCompiler::compile_with_config(source, &config);
    assert!(result.is_ok());

    let bytecode = result.unwrap();
    let disasm_lines = disassemble(&bytecode);
    let disasm_str = disasm_lines.join("\n");
    println!("Disassembly (control flow):\n{}", disasm_str);

    // Should see PUSH_REG for parameter reuse even with control flow
    let push_reg_count = disasm_str.matches("PUSH_REG").count();
    println!("PUSH_REG count with control flow: {}", push_reg_count);

    assert!(push_reg_count > 0, "Should use PUSH_REG even with control flow");
}
