use five_dsl_compiler::{DslBytecodeGenerator, DslParser, DslTokenizer, DslTypeChecker};
use five_vm_mito::error::VMError;
use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <script.v> [--debug-bytecode]", args[0]);
        process::exit(1);
    }

    // Detect optional --debug-bytecode flag
    let debug_bytecode = args.iter().any(|a| a == "--debug-bytecode");

    // Find the first non-flag argument after program name and treat it as file path
    let file_path = args
        .iter()
        .skip(1)
        .find(|a| !a.starts_with('-'))
        .map(|s| s.as_str())
        .unwrap_or_else(|| {
            eprintln!("Usage: {} <script.v> [--debug-bytecode]", args[0]);
            process::exit(1);
        });

    // Read the source file
    let source = match fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error reading file '{}': {}", file_path, err);
            process::exit(1);
        }
    };

    println!("Debug compilation of: {}", file_path);
    println!("DEBUG COMPILE VERSION 2 - FIX APPLIED");
    println!("Source code:");
    println!("{}", source);
    println!("{}", "=".repeat(60));

    // Step 1: Tokenization
    println!("\n1. TOKENIZATION");
    println!("{}", "-".repeat(20));
    let mut tokenizer = DslTokenizer::new(&source);
    let tokens = match tokenizer.tokenize() {
        Ok(tokens) => {
            println!("✓ Tokenization successful! Found {} tokens:", tokens.len());
            for (i, token) in tokens.iter().enumerate() {
                println!("  {}: {:?}", i, token);
            }
            tokens
        }
        Err(err) => {
            match err {
                VMError::InvalidScript => {
                    eprintln!("✗ Tokenization failed: Invalid script");
                }
                other => {
                    eprintln!("✗ Tokenization failed: {:?}", other);
                }
            }
            process::exit(1);
        }
    };

    // Step 2: Parsing
    println!("\n2. PARSING");
    println!("{}", "-".repeat(20));
    let mut parser = DslParser::new(tokens.clone());
    let ast = match parser.parse() {
        Ok(ast) => {
            println!("✓ Parsing successful!");
            println!("  AST: {:#?}", ast);
            ast
        }
        Err(err) => {
            eprintln!("✗ Parsing failed: {:?}", err);
            process::exit(1);
        }
    };

    // Step 3: Type Checking
    println!("\n3. TYPE CHECKING");
    println!("{}", "-".repeat(20));
    let mut type_checker = DslTypeChecker::new();
    match type_checker.check_types(&ast) {
        Ok(_) => {
            println!("✓ Type checking successful!");
            println!("  Type checking passed");
        }
        Err(error) => {
            eprintln!("✗ Type checking failed: {}", error);
            process::exit(1);
        }
    }

    // Step 4: Bytecode Generation
    println!("\n4. BYTECODE GENERATION");
    println!("{}", "-".repeat(20));
    let mut bytecode_gen = DslBytecodeGenerator::new();

    // If user requested debug bytecode capture, enable generator diagnostic capture
    if debug_bytecode {
        bytecode_gen.set_debug_on_error(true);
    }

    let bytecode = match bytecode_gen.generate(&ast) {
        Ok(bytecode) => {
            println!("✓ Bytecode generation successful!");
            println!("  Bytecode length: {} bytes", bytecode.len());
            println!("  Bytecode (hex): {}", hex::encode(&bytecode));

            // Disassemble bytecode for debugging
            println!("\n  Disassembly:");
            five_dsl_compiler::disassembler::disassemble_bytecode(&bytecode);

            // If debug-bytecode was enabled, print captured compilation log from generator
            if debug_bytecode {
                let logs = bytecode_gen.get_compilation_log();
                if !logs.is_empty() {
                    println!("\n  Captured bytecode diagnostics:");
                    for line in logs.iter() {
                        println!("    {}", line);
                    }
                }
            }

            bytecode
        }
        Err(err) => {
            eprintln!("✗ Bytecode generation failed: {:?}", err);
            process::exit(1);
        }
    };

    // Step 5: ABI Generation
    println!("\n5. ABI GENERATION");
    println!("{}", "-".repeat(20));
    let abi = match bytecode_gen.generate_abi(&ast) {
        Ok(abi) => {
            println!("✓ ABI generation successful!");
            println!("  Program: {}", abi.program_name);
            println!("  Functions: {}", abi.functions.len());
            println!("  Fields: {}", abi.fields.len());

            for func in &abi.functions {
                println!(
                    "    Function {}: {} (index {})",
                    func.index, func.name, func.index
                );
                for param in &func.parameters {
                    println!(
                        "      - {}: {} (account: {})",
                        param.name, param.param_type, param.is_account
                    );
                }
            }

            abi
        }
        Err(err) => {
            eprintln!("✗ ABI generation failed: {:?}", err);
            process::exit(1);
        }
    };

    // Write binary output (.bin file)
    let bin_output = file_path.replace(".v", ".bin");
    match fs::write(&bin_output, &bytecode) {
        Ok(_) => println!("\n✓ Bytecode written to: {}", bin_output),
        Err(err) => eprintln!("Warning: Could not write bytecode file: {}", err),
    }

    // Write ABI output (.abi.json file)
    let abi_output = file_path.replace(".v", ".abi.json");
    let abi_json = match serde_json::to_string_pretty(&abi) {
        Ok(json) => json,
        Err(err) => {
            eprintln!("Warning: Could not serialize ABI to JSON: {}", err);
            "{}".to_string()
        }
    };
    match fs::write(&abi_output, &abi_json) {
        Ok(_) => println!("✓ ABI written to: {}", abi_output),
        Err(err) => eprintln!("Warning: Could not write ABI file: {}", err),
    }

    // Write debug output
    let debug_output = format!("{}.debug", file_path);
    let debug_info = format!(
        "Debug compilation report for: {}\n\
         Source length: {} characters\n\
         Tokens: {} items\n\
         Bytecode: {} bytes\n\
         Bytecode (hex): {}\n\
         ABI functions: {}\n\
         ABI fields: {}\n",
        file_path,
        source.len(),
        tokens.len(),
        bytecode.len(),
        hex::encode(&bytecode),
        abi.functions.len(),
        abi.fields.len()
    );

    match fs::write(&debug_output, debug_info) {
        Ok(_) => println!("✓ Debug info written to: {}", debug_output),
        Err(err) => eprintln!("Warning: Could not write debug file: {}", err),
    }
}


mod hex {
    pub fn encode(data: &[u8]) -> String {
        data.iter()
            .map(|b| format!("{:02x}", b))
            .collect::<Vec<_>>()
            .join("")
    }
}
