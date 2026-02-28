use five_dsl_compiler::DslCompiler;

#[test]
fn test_debug_compiler() {
    let source = r#"
script SingleParam {
    pub test(a: u64) -> u64 {
        return a;
    }
}
"#;

    println!("Compiling script:");
    println!("{}", source);

    match DslCompiler::compile_dsl(source) {
        Ok(bytecode) => {
            println!("Compilation successful!");
            println!("Bytecode length: {}", bytecode.len());

            // Print bytecode as hex
            print!("Bytecode: ");
            for byte in &bytecode {
                print!("{:02x} ", byte);
            }
            println!();
            assert!(!bytecode.is_empty());
        }
        Err(e) => {
            println!("Compilation failed: {:?}", e);
            panic!("Compilation failed");
        }
    }
}
