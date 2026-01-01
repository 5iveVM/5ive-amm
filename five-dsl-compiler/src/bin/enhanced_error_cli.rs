//! Enhanced Error System CLI Demo
//!
//! This CLI demonstrates the enhanced error system in action with real Five DSL compilation.
//! It compiles various test scripts (both valid and invalid) to show rich error messages.

use five_dsl_compiler::error::integration;
use five_dsl_compiler::DslCompiler;
use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Enhanced Error System CLI Demo");
        eprintln!("Usage: {} <script.v> [--format terminal|json|lsp]", args[0]);
        eprintln!();
        eprintln!("Try these examples:");
        eprintln!("  {} demo_scripts/syntax_error.v", args[0]);
        eprintln!("  {} demo_scripts/type_error.v", args[0]);
        eprintln!("  {} demo_scripts/valid.v", args[0]);
        process::exit(1);
    }

    let script_path = &args[1];
    let format = if args.len() > 3 && args[2] == "--format" {
        args[3].as_str()
    } else {
        "terminal"
    };

    // Initialize the enhanced error system
    if let Err(e) = integration::initialize_error_system() {
        eprintln!("Warning: Failed to initialize enhanced error system: {}", e);
    }

    // Set the output format
    if let Err(e) = {
        let mut sys = integration::get_error_system_mut();
        integration::set_formatter(&mut sys, format)
    } {
        eprintln!("Warning: Failed to set formatter '{}': {}", format, e);
    }

    // Read the script file
    let source = match fs::read_to_string(script_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", script_path, e);
            process::exit(1);
        }
    };

    println!("Five DSL Compiler - Enhanced Error System Demo");
    println!("==============================================");
    println!("File: {}", script_path);
    println!("Format: {}", format);
    println!();

    // Attempt compilation
    match DslCompiler::compile_dsl(&source) {
        Ok(bytecode) => {
            println!("✅ Compilation successful!");
            println!("Generated {} bytes of bytecode", bytecode.len());

            if format == "terminal" {
                // Show a preview of the bytecode
                println!();
                println!("Bytecode preview (first 32 bytes):");
                for (i, byte) in bytecode.iter().take(32).enumerate() {
                    if i % 16 == 0 {
                        print!("{:04x}: ", i);
                    }
                    print!("{:02x} ", byte);
                    if i % 16 == 15 {
                        println!();
                    }
                }
                if bytecode.len() % 16 != 0 {
                    println!();
                }
            }
        }
        Err(_) => {
            println!("❌ Compilation failed with enhanced error reporting");
            println!();
            println!("Enhanced error messages are now displayed by the compiler");
            println!("during the compilation process. Check the output above for");
            println!("detailed error information, suggestions, and fixes.");
        }
    }

    println!();
    if format == "terminal" {
        println!("💡 Try different output formats:");
        println!("  {} {} --format json", args[0], script_path);
        println!("  {} {} --format lsp", args[0], script_path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_compilation() {
        // This test ensures the CLI functionality works
        // We can't easily test the full CLI here, but we can test compilation
        let valid_source = r#"
            script test_vault {
                mut balance: u64;

                init {
                    balance = 1000;
                }
            }
        "#;

        // This should succeed
        let result = DslCompiler::compile_dsl(valid_source);
        assert!(result.is_ok(), "Valid script should compile successfully");

        let invalid_source = r#"
            script broken_vault {
                init {
                    amount = undefined_variable;
                }
            }
        "#;

        // This should fail and show enhanced error messages
        let result = DslCompiler::compile_dsl(invalid_source);
        assert!(result.is_err(), "Invalid script should fail compilation");
    }
}
