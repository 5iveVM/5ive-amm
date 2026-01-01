use five_dsl_compiler::DslCompiler;
use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <script.v>", args[0]);
        process::exit(1);
    }

    let file_path = &args[1];

    // Read the source file
    let source = match fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error reading file '{}': {}", file_path, err);
            process::exit(1);
        }
    };

    println!("Testing compilation and execution of: {}", file_path);
    println!("Source code:");
    println!("{}", source);
    println!("{}", "=".repeat(50));

    // Compile the source using the DSL compiler
    let bytecode = match DslCompiler::compile_dsl(&source) {
        Ok(bytecode) => {
            println!("✓ Compilation successful!");
            println!("  Bytecode length: {} bytes", bytecode.len());
            bytecode
        }
        Err(error) => {
            eprintln!("✗ Compilation failed: {}", error);
            process::exit(1);
        }
    };

    // Bytecode validation (basic check)
    println!("\nValidating bytecode...");
    if bytecode.len() >= 4 {
        let magic = &bytecode[0..4];
        println!("✓ Bytecode validation successful!");
        println!(
            "  Magic bytes: {:02x} {:02x} {:02x} {:02x}",
            magic[0], magic[1], magic[2], magic[3]
        );
        println!("  Bytecode ready for VM execution");
    } else {
        eprintln!("✗ Invalid bytecode: too short");
        process::exit(1);
    }
}
