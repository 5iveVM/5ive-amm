// Five DSL Compiler Metrics CLI
//
// Command-line interface for compiler metrics visualization and reporting

use five_dsl_compiler::{export_metrics, CompilationMode, DslCompiler, ExportFormat};
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        return;
    }

    match args[1].as_str() {
        "compile-with-metrics" => {
            if args.len() < 3 {
                eprintln!("Usage: {} compile-with-metrics <source-file> [--format json|csv|dashboard] [--output <file>]", args[0]);
                return;
            }

            let source_file = &args[2];
            let mut format = ExportFormat::Json;
            let mut output_file = None;

            // Parse additional arguments
            let mut i = 3;
            while i < args.len() {
                match args[i].as_str() {
                    "--format" => {
                        if i + 1 < args.len() {
                            format = match args[i + 1].as_str() {
                                "json" => ExportFormat::Json,
                                "csv" => ExportFormat::Csv,
                                "dashboard" => ExportFormat::Dashboard,
                                _ => {
                                    eprintln!(
                                        "Invalid format: {}. Use json, csv, or dashboard",
                                        args[i + 1]
                                    );
                                    return;
                                }
                            };
                            i += 2;
                        } else {
                            eprintln!("--format requires a value");
                            return;
                        }
                    }
                    "--output" => {
                        if i + 1 < args.len() {
                            output_file = Some(args[i + 1].clone());
                            i += 2;
                        } else {
                            eprintln!("--output requires a value");
                            return;
                        }
                    }
                    _ => {
                        eprintln!("Unknown argument: {}", args[i]);
                        return;
                    }
                }
            }

            compile_with_metrics(source_file, format, output_file);
        }
        "benchmark" => {
            if args.len() < 3 {
                eprintln!(
                    "Usage: {} benchmark <source-file> [--iterations <n>]",
                    args[0]
                );
                return;
            }

            let source_file = &args[2];
            let mut iterations = 10;

            // Parse iterations
            if args.len() >= 5 && args[3] == "--iterations" {
                iterations = args[4].parse().unwrap_or(10);
            }

            benchmark_compilation(source_file, iterations);
        }
        "analyze" => {
            if args.len() < 3 {
                eprintln!("Usage: {} analyze <source-file>", args[0]);
                return;
            }

            let source_file = &args[2];
            analyze_source(source_file);
        }
        "help" | "--help" | "-h" => {
            print_usage();
        }
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            print_usage();
        }
    }
}

fn print_usage() {
    println!("Five DSL Compiler Metrics CLI");
    println!();
    println!("USAGE:");
    println!("    metrics_cli <COMMAND> [OPTIONS]");
    println!();
    println!("COMMANDS:");
    println!("    compile-with-metrics  Compile a Five script and collect detailed metrics");
    println!("    benchmark            Run compilation benchmark tests");
    println!("    analyze              Analyze source code complexity and patterns");
    println!("    help                 Show this help message");
    println!();
    println!("OPTIONS:");
    println!("    --format <FORMAT>    Output format: json, csv, dashboard (default: json)");
    println!("    --output <FILE>      Output file (default: stdout)");
    println!("    --iterations <N>     Number of benchmark iterations (default: 10)");
    println!();
    println!("EXAMPLES:");
    println!(
        "    metrics_cli compile-with-metrics script.v --format dashboard --output metrics.json"
    );
    println!("    metrics_cli benchmark script.v --iterations 100");
    println!("    metrics_cli analyze complex_script.v");
}

fn compile_with_metrics(source_file: &str, format: ExportFormat, output_file: Option<String>) {
    // Read source file
    let source = match fs::read_to_string(source_file) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading source file {}: {}", source_file, e);
            return;
        }
    };

    // Compile with metrics
    println!("Compiling {} with metrics collection...", source_file);
    let result = DslCompiler::compile_with_metrics(&source, CompilationMode::Testing, true);

    match result {
        Ok((bytecode, metrics)) => {
            println!("✅ Compilation successful!");
            println!("  - Bytecode size: {} bytes", bytecode.len());
            println!("  - Total opcodes: {}", metrics.opcode_stats.total_opcodes);
            println!(
                "  - Compilation time: {:?}",
                metrics.performance.total_compilation_time
            );
            println!(
                "  - Lines per second: {:.2}",
                metrics.performance.lines_per_second
            );

            // Export metrics
            match export_metrics(&metrics, format) {
                Ok(exported) => {
                    if let Some(file) = output_file {
                        match fs::write(&file, &exported) {
                            Ok(_) => println!("📊 Metrics exported to {}", file),
                            Err(e) => eprintln!("Error writing to {}: {}", file, e),
                        }
                    } else {
                        println!("\n📊 Metrics:");
                        println!("{}", exported);
                    }
                }
                Err(e) => eprintln!("Error exporting metrics: {}", e),
            }
        }
        Err(e) => {
            eprintln!("❌ Compilation failed: {:?}", e);
        }
    }
}

fn benchmark_compilation(source_file: &str, iterations: usize) {
    // Read source file
    let source = match fs::read_to_string(source_file) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading source file {}: {}", source_file, e);
            return;
        }
    };

    println!(
        "🏃 Running compilation benchmark for {} ({} iterations)...",
        source_file, iterations
    );

    let mut successful_compilations = 0;
    let mut total_compile_time = std::time::Duration::default();
    let mut bytecode_sizes = Vec::new();
    let mut all_metrics = Vec::new();

    for i in 0..iterations {
        let start = std::time::Instant::now();

        match DslCompiler::compile_with_metrics(&source, CompilationMode::Testing, true) {
            Ok((bytecode, metrics)) => {
                successful_compilations += 1;
                let compile_time = start.elapsed();
                total_compile_time += compile_time;
                bytecode_sizes.push(bytecode.len());
                all_metrics.push(metrics);

                if i % (iterations / 10).max(1) == 0 {
                    print!(".");
                    std::io::Write::flush(&mut std::io::stdout()).unwrap();
                }
            }
            Err(e) => {
                eprintln!("\nCompilation failed at iteration {}: {:?}", i + 1, e);
            }
        }
    }

    println!("\n");

    if successful_compilations > 0 {
        let avg_compile_time = total_compile_time / successful_compilations as u32;
        let avg_bytecode_size = bytecode_sizes.iter().sum::<usize>() / bytecode_sizes.len();

        println!("📈 Benchmark Results:");
        println!(
            "  - Successful compilations: {}/{}",
            successful_compilations, iterations
        );
        println!(
            "  - Success rate: {:.1}%",
            (successful_compilations as f64 / iterations as f64) * 100.0
        );
        println!("  - Average compile time: {:?}", avg_compile_time);
        println!("  - Average bytecode size: {} bytes", avg_bytecode_size);
        println!(
            "  - Compilations per second: {:.2}",
            1.0 / avg_compile_time.as_secs_f64()
        );

        // Calculate aggregate metrics
        if !all_metrics.is_empty() {
            let total_opcodes: u64 = all_metrics
                .iter()
                .map(|m| m.opcode_stats.total_opcodes)
                .sum();
            let avg_opcodes = total_opcodes / all_metrics.len() as u64;

            println!("  - Average opcodes per compilation: {}", avg_opcodes);

            // Find most common opcodes across all compilations
            let mut opcode_totals = std::collections::HashMap::new();
            for metrics in &all_metrics {
                for (opcode, count) in &metrics.opcode_stats.usage_frequency {
                    *opcode_totals.entry(opcode.clone()).or_insert(0) += count;
                }
            }

            let mut opcode_vec: Vec<_> = opcode_totals.iter().collect();
            opcode_vec.sort_by(|a, b| b.1.cmp(a.1));

            println!("  - Top 5 opcodes:");
            for (opcode, count) in opcode_vec.iter().take(5) {
                println!("    - {}: {} times", opcode, count);
            }
        }
    } else {
        println!("❌ No successful compilations in benchmark");
    }
}

fn analyze_source(source_file: &str) {
    // Read source file
    let source = match fs::read_to_string(source_file) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading source file {}: {}", source_file, e);
            return;
        }
    };

    println!("🔍 Analyzing source code: {}", source_file);

    // Basic source analysis
    let lines: Vec<&str> = source.lines().collect();
    let total_lines = lines.len();
    let code_lines = lines
        .iter()
        .filter(|line| !line.trim().is_empty() && !line.trim().starts_with("//"))
        .count();
    let comment_lines = lines
        .iter()
        .filter(|line| line.trim().starts_with("//"))
        .count();
    let blank_lines = lines.iter().filter(|line| line.trim().is_empty()).count();

    println!("\n📊 Source Statistics:");
    println!("  - Total lines: {}", total_lines);
    println!(
        "  - Code lines: {} ({:.1}%)",
        code_lines,
        (code_lines as f64 / total_lines as f64) * 100.0
    );
    println!(
        "  - Comment lines: {} ({:.1}%)",
        comment_lines,
        (comment_lines as f64 / total_lines as f64) * 100.0
    );
    println!(
        "  - Blank lines: {} ({:.1}%)",
        blank_lines,
        (blank_lines as f64 / total_lines as f64) * 100.0
    );

    // Complexity analysis
    let function_count = source.matches("fn ").count() + source.matches("init ").count();
    let if_count = source.matches("if ").count();
    let while_count = source.matches("while ").count();
    let match_count = source.matches("match ").count();
    let max_nesting = calculate_max_nesting(&source);

    println!("\n🧮 Complexity Analysis:");
    println!("  - Functions: {}", function_count);
    println!("  - Conditional statements (if): {}", if_count);
    println!("  - Loops (while): {}", while_count);
    println!("  - Pattern matching (match): {}", match_count);
    println!("  - Maximum nesting depth: {}", max_nesting);

    // Language feature usage
    let mut features = Vec::new();
    if source.contains("Option<") {
        features.push("Option types");
    }
    if source.contains("Result<") {
        features.push("Result types");
    }
    if source.contains("Some(") || source.contains("None") {
        features.push("Option constructors");
    }
    if source.contains("Ok(") || source.contains("Err(") {
        features.push("Result constructors");
    }
    if source.contains("match ") {
        features.push("Pattern matching");
    }
    if source.contains("struct ") {
        features.push("Structs");
    }
    if source.contains("enum ") {
        features.push("Enums");
    }
    if source.contains("get_clock()") {
        features.push("System calls");
    }
    if source.contains("[") && source.contains("]") {
        features.push("Arrays");
    }
    if source.contains("string") {
        features.push("Strings");
    }

    println!("\n🚀 Language Features Used:");
    for feature in features {
        println!("  - {}", feature);
    }

    // Try compilation with metrics
    match DslCompiler::compile_with_metrics(&source, CompilationMode::Testing, true) {
        Ok((bytecode, metrics)) => {
            println!("\n✅ Compilation Analysis:");
            println!("  - Compilation successful");
            println!("  - Bytecode size: {} bytes", bytecode.len());
            println!(
                "  - Opcodes generated: {}",
                metrics.opcode_stats.total_opcodes
            );
            println!(
                "  - Compilation time: {:?}",
                metrics.performance.total_compilation_time
            );

            if !metrics.opcode_stats.top_opcodes.is_empty() {
                println!("  - Top opcodes:");
                for (opcode, count) in metrics.opcode_stats.top_opcodes.iter().take(5) {
                    println!("    - {}: {} times", opcode, count);
                }
            }
        }
        Err(e) => {
            println!("\n❌ Compilation failed: {:?}", e);
            println!("  - Consider fixing syntax errors before analysis");
        }
    }
}

fn calculate_max_nesting(source: &str) -> u32 {
    let mut max_depth = 0;
    let mut current_depth = 0;

    for ch in source.chars() {
        match ch {
            '{' => {
                current_depth += 1;
                if current_depth > max_depth {
                    max_depth = current_depth;
                }
            }
            '}' => {
                if current_depth > 0 {
                    current_depth -= 1;
                }
            }
            _ => {}
        }
    }

    max_depth
}
