use five_dsl_compiler::{CompilationConfig, CompilationMode, DslCompiler};
use std::env;
use std::fs;
use std::path::Path;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 || args.len() > 3 {
        eprintln!("Usage: {} <script.v> [--enable-cache]", args[0]);
        eprintln!("Options:");
        eprintln!(
            "  --enable-cache     Enable constraint cache (increases bytecode size and CU usage)"
        );
        process::exit(1);
    }

    let file_path = &args[1];
    let enable_cache = args.contains(&"--enable-cache".to_string());

    let path = Path::new(file_path);
    if let Err(err) = fs::metadata(path) {
        eprintln!("Error reading file '{}': {}", file_path, err);
        process::exit(1);
    }

    // File-based compilation should resolve imported modules and bundled stdlib assets.
    let config =
        CompilationConfig::new(CompilationMode::Deployment).with_constraint_cache(enable_cache);
    match DslCompiler::compile_with_auto_discovery(path, &config) {
        Ok(bytecode) => {
            println!("Compilation successful!");
            println!("Bytecode length: {} bytes", bytecode.len());
            println!(
                "Constraint cache: {} (default: disabled for optimal performance)",
                if enable_cache { "enabled" } else { "disabled" }
            );

            // Write bytecode to output file
            let output_path = format!("{}.bin", file_path.trim_end_matches(".v"));
            match fs::write(&output_path, &bytecode) {
                Ok(_) => println!("Bytecode written to: {}", output_path),
                Err(err) => {
                    eprintln!("Error writing bytecode: {}", err);
                    process::exit(1);
                }
            }
        }
        Err(error) => {
            eprintln!("Compilation failed: {}", error);
            process::exit(1);
        }
    }
}
