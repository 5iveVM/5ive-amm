use five_dsl_compiler::{DslCompiler};

fn main() {
    let source = r#"
script SingleParam {
    test(a: u64) -> u64 {
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
        }
        Err(e) => {
            println!("Compilation failed: {:?}", e);
        }
    }
}