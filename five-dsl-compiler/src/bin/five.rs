// Five DSL Compiler - Unified CLI
//
// A single, comprehensive CLI tool for all Five DSL compilation needs
// with excellent UX and comprehensive metrics support

use clap::{ArgAction, Parser, Subcommand};
use five_dsl_compiler::{
    export_metrics, CompilationConfig, CompilationMode, DslCompiler, ExportFormat,
};
use std::env;
use std::fs;
use std::path::PathBuf;

/// Five DSL Compiler - The complete toolchain for Five VM development
#[derive(Parser)]
#[command(name = "five")]
#[command(about = "Five DSL Compiler - Build, analyze, and optimize Five VM scripts")]
#[command(version)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Suppress all non-essential output
    #[arg(short, long, global = true)]
    quiet: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile Five DSL scripts to bytecode
    Compile {
        /// Source file to compile
        #[arg(value_name = "FILE")]
        source: PathBuf,

        /// Output file (defaults to <source>.fbin)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Compilation mode
        #[arg(short, long, default_value = "testing")]
        mode: CompilationModeArg,

        /// Enable constraint cache optimization
        #[arg(long, default_value = "true")]
        constraint_cache: bool,

        /// Enable v2-preview features (nibble immediates, BR_EQ_U8, etc.)
        #[arg(long)]
        v2_preview: bool,

        /// Collect detailed metrics
        #[arg(short = 'M', long)]
        metrics: bool,

        /// Export metrics to file
        #[arg(long, requires = "metrics")]
        metrics_output: Option<PathBuf>,

        /// Metrics export format
        #[arg(long, default_value = "json", requires = "metrics")]
        metrics_format: MetricsFormatArg,

        /// Show compilation summary
        #[arg(short, long)]
        summary: bool,

        /// Enable debug bytecode diagnostics (capture disassembly on generation errors)
        #[arg(long)]
        debug_bytecode: bool,
    },



    /// Build a project from five.toml configuration
    Build {
        /// Path to project directory (defaults to current dir)
        #[arg(short, long, default_value = ".")]
        path: PathBuf,

        /// Output file (overrides five.toml default)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Compilation mode
        #[arg(short, long, default_value = "testing")]
        mode: CompilationModeArg,

        /// Enable constraint cache optimization
        #[arg(long, default_value = "true")]
        constraint_cache: bool,

        /// Enable v2-preview features
        #[arg(long)]
        v2_preview: bool,

        /// Collect detailed metrics
        #[arg(short = 'M', long)]
        metrics: bool,

        /// Export metrics to file
        #[arg(long, requires = "metrics")]
        metrics_output: Option<PathBuf>,

        /// Metrics export format
        #[arg(long, default_value = "json", requires = "metrics")]
        metrics_format: MetricsFormatArg,

        /// Show compilation summary
        #[arg(short, long)]
        summary: bool,

        /// Enable debug bytecode diagnostics
        #[arg(long)]
        debug_bytecode: bool,
    },

    /// Compile multiple Five DSL modules into a single bytecode
    CompileMulti {
        /// Main/entry point source file
        #[arg(value_name = "MAIN")]
        main: PathBuf,

        /// Additional module files to include
        #[arg(value_name = "MODULE", action = ArgAction::Append)]
        modules: Vec<PathBuf>,

        /// Output file (defaults to main.fbin)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Compilation mode
        #[arg(short, long, default_value = "testing")]
        mode: CompilationModeArg,

        /// Enable constraint cache optimization
        #[arg(long, default_value = "true")]
        constraint_cache: bool,

        /// Enable v2-preview features
        #[arg(long)]
        v2_preview: bool,

        /// Collect detailed metrics
        #[arg(short = 'M', long)]
        metrics: bool,

        /// Export metrics to file
        #[arg(long, requires = "metrics")]
        metrics_output: Option<PathBuf>,

        /// Metrics export format
        #[arg(long, default_value = "json", requires = "metrics")]
        metrics_format: MetricsFormatArg,

        /// Show compilation summary
        #[arg(short, long)]
        summary: bool,

        /// Enable debug bytecode diagnostics
        #[arg(long)]
        debug_bytecode: bool,
    },

    /// Analyze source code and compilation metrics
    Analyze {
        /// Source file to analyze
        #[arg(value_name = "FILE")]
        source: PathBuf,

        /// Analysis depth level
        #[arg(short, long, default_value = "standard")]
        depth: AnalysisDepth,

        /// Export analysis results
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Output format
        #[arg(short, long, default_value = "text")]
        format: OutputFormat,

        /// Include performance metrics
        #[arg(short, long)]
        performance: bool,

        /// Include complexity analysis
        #[arg(short, long)]
        complexity: bool,
    },

    /// Run compilation benchmarks
    Benchmark {
        /// Source file or directory to benchmark
        #[arg(value_name = "PATH")]
        path: PathBuf,

        /// Number of iterations
        #[arg(short, long, default_value = "10")]
        iterations: usize,

        /// Number of warmup iterations
        #[arg(short, long, default_value = "3")]
        warmup: usize,

        /// Export benchmark results
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Compare against baseline
        #[arg(short, long)]
        baseline: Option<PathBuf>,

        /// Save as new baseline
        #[arg(long)]
        save_baseline: bool,
    },

    /// Interactive development mode
    Watch {
        /// Source file or directory to watch
        #[arg(value_name = "PATH")]
        path: PathBuf,

        /// Commands to run on changes
        #[arg(short, long, action = ArgAction::Append)]
        command: Vec<String>,

        /// Enable live metrics
        #[arg(short, long)]
        metrics: bool,

        /// Metrics update interval in seconds
        #[arg(long, default_value = "1")]
        interval: u64,
    },

    /// Generate project scaffolding
    Init {
        /// Project name
        #[arg(value_name = "NAME")]
        name: String,

        /// Project template
        #[arg(short, long, default_value = "basic")]
        template: ProjectTemplate,

        /// Target directory
        #[arg(short, long)]
        dir: Option<PathBuf>,
    },

    /// Show detailed information about bytecode
    Inspect {
        /// Bytecode file to inspect
        #[arg(value_name = "FILE")]
        bytecode: PathBuf,

        /// Show opcodes disassembly
        #[arg(short, long)]
        disasm: bool,

        /// Analyze bytecode patterns
        #[arg(long)]
        patterns: bool,

        /// Show performance characteristics
        #[arg(short = 'P', long)]
        performance: bool,

        /// Export inspection results
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

#[derive(clap::ValueEnum, Clone)]
enum CompilationModeArg {
    Testing,
    Deployment,
    Debug,
}

impl From<CompilationModeArg> for CompilationMode {
    fn from(mode: CompilationModeArg) -> Self {
        match mode {
            CompilationModeArg::Testing => CompilationMode::Testing,
            CompilationModeArg::Deployment => CompilationMode::Deployment,
            CompilationModeArg::Debug => CompilationMode::Testing, // Debug uses Testing mode with extended metrics
        }
    }
}

#[derive(clap::ValueEnum, Clone)]
enum MetricsFormatArg {
    Json,
    Csv,
    Dashboard,
}

impl From<MetricsFormatArg> for ExportFormat {
    fn from(format: MetricsFormatArg) -> Self {
        match format {
            MetricsFormatArg::Json => ExportFormat::Json,
            MetricsFormatArg::Csv => ExportFormat::Csv,
            MetricsFormatArg::Dashboard => ExportFormat::Dashboard,
        }
    }
}

#[derive(clap::ValueEnum, Clone)]
enum AnalysisDepth {
    Quick,
    Standard,
    Deep,
    Comprehensive,
}

#[derive(clap::ValueEnum, Clone)]
enum OutputFormat {
    Text,
    Json,
    Markdown,
    Html,
}

#[derive(clap::ValueEnum, Clone)]
enum ProjectTemplate {
    Basic,
    Advanced,
    Defi,
    Gaming,
    Nft,
}

#[derive(Debug, serde::Deserialize)]
struct FiveConfig {
    project: ProjectConfig,
    modules: std::collections::HashMap<String, Vec<String>>,
    build: Option<BuildConfig>,
}

#[derive(Debug, serde::Deserialize)]
struct ProjectConfig {
    name: String,
}

#[derive(Debug, serde::Deserialize)]
struct BuildConfig {
    entry_point: Option<String>,
}

fn main() {
    eprintln!("DEBUG_CLI: main started");
    let cli = Cli::parse();

    // Set up logging based on verbosity
    let log_level = if cli.quiet {
        "error"
    } else if cli.verbose {
        "debug"
    } else {
        "info"
    };

    env::set_var("RUST_LOG", log_level);

    let result = match cli.command {
        Commands::Compile {
            source,
            output,
            mode,
            constraint_cache,
            v2_preview,
            metrics,
            metrics_output,
            metrics_format,
            summary,
            debug_bytecode,
        } => handle_compile(
            source,
            output,
            mode.into(),
            constraint_cache,
            v2_preview,
            metrics,
            metrics_output,
            metrics_format.into(),
            summary,
            debug_bytecode,
            cli.verbose,
            cli.quiet,
        ),
        Commands::CompileMulti {
            main,
            modules,
            output,
            mode,
            constraint_cache,
            v2_preview,
            metrics,
            metrics_output,
            metrics_format,
            summary,
            debug_bytecode,
        } => handle_compile_multi(
            main,
            modules,
            output,
            mode.into(),
            constraint_cache,
            v2_preview,
            metrics,
            metrics_output,
            metrics_format.into(),
            summary,
            debug_bytecode,
            cli.verbose,
            cli.quiet,
        ),
        Commands::Build {
            path,
            output,
            mode,
            constraint_cache,
            v2_preview,
            metrics,
            metrics_output,
            metrics_format,
            summary,
            debug_bytecode,
        } => handle_build(
            path,
            output,
            mode.into(),
            constraint_cache,
            v2_preview,
            metrics,
            metrics_output,
            metrics_format.into(),
            summary,
            debug_bytecode,
            cli.verbose,
            cli.quiet,
        ),
        Commands::Analyze {
            source,
            depth,
            output,
            format,
            performance,
            complexity,
        } => handle_analyze(
            source,
            depth,
            output,
            format,
            performance,
            complexity,
            cli.verbose,
        ),
        Commands::Benchmark {
            path,
            iterations,
            warmup,
            output,
            baseline,
            save_baseline,
        } => handle_benchmark(
            path,
            iterations,
            warmup,
            output,
            baseline,
            save_baseline,
            cli.verbose,
        ),
        Commands::Watch {
            path,
            command,
            metrics,
            interval,
        } => handle_watch(path, command, metrics, interval, cli.verbose),
        Commands::Init {
            name,
            template,
            dir,
        } => handle_init(name, template, dir, cli.verbose),
        Commands::Inspect {
            bytecode,
            disasm,
            patterns,
            performance,
            output,
        } => handle_inspect(bytecode, disasm, patterns, performance, output, cli.verbose),
    };

    if let Err(e) = result {
        if !cli.quiet {
            eprintln!("❌ Error: {}", e);
        }
        std::process::exit(1);
    }
}

fn handle_compile(
    source: PathBuf,
    output: Option<PathBuf>,
    mode: CompilationMode,
    constraint_cache: bool,
    v2_preview: bool,
    metrics: bool,
    metrics_output: Option<PathBuf>,
    metrics_format: ExportFormat,
    summary: bool,
    debug_bytecode: bool,
    verbose: bool,
    quiet: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("DEBUG_CLI: handle_compile started quiet={}", quiet);
    if verbose && !quiet {
        let mode_str = if v2_preview {
            format!("{:?} (v2-preview)", mode)
        } else {
            format!("{:?}", mode)
        };
        println!("🔧 Compiling {} in {} mode...", source.display(), mode_str);
    }

    // Read source file
    let source_code =
        fs::read_to_string(&source).map_err(|e| format!("Failed to read source file: {}", e))?;

    // Determine output file
    let output_path = output.unwrap_or_else(|| {
        let mut path = source.clone();
        path.set_extension("fbin");
        path
    });

    let start_time = std::time::Instant::now();

    // Create compilation configuration using builder pattern
    let config = CompilationConfig::new(mode)
        .with_constraint_cache(constraint_cache)
        .with_v2_preview(v2_preview);

    if metrics {
        // Compile with metrics (using config)
        let (bytecode, mut compilation_metrics) = if v2_preview {
            // TODO: Add compile_with_metrics_and_config method to DslCompiler
            // For now, use fallback to regular compile and empty metrics
            let bytecode = DslCompiler::compile_with_config(&source_code, &config)?;
            (bytecode, five_dsl_compiler::CompilerMetrics::default())
        } else {
            DslCompiler::compile_with_metrics(&source_code, mode, constraint_cache)?
        };

        let compile_time = start_time.elapsed();

        // Enhance metrics for debug mode
        let is_debug_mode_arg = summary || verbose; // Approximate debug detection

        if is_debug_mode_arg {
            // Add debug-specific metrics
            compilation_metrics
                .source_stats
                .feature_usage
                .insert("debug_compilation".to_string(), 1);
            compilation_metrics
                .memory_analytics
                .phase_memory
                .insert("debug_overhead".to_string(), 2048);

            // Add opcode pattern analysis for debug
            let pattern_count = compilation_metrics.opcode_stats.opcode_patterns.len();
            compilation_metrics
                .opcode_stats
                .advanced_usage
                .insert("detected_patterns".to_string(), pattern_count as u64);
        }

        // Write bytecode
        fs::write(&output_path, &bytecode)
            .map_err(|e| format!("Failed to write bytecode: {}", e))?;

        if !quiet {
            println!("✅ Compilation successful!");

            if summary || verbose {
                println!("📊 Summary:");
                println!("  • Source: {}", source.display());
                println!("  • Output: {}", output_path.display());
                println!("  • Bytecode size: {} bytes", bytecode.len());
                println!(
                    "  • Compilation time: {:.2}ms",
                    compile_time.as_secs_f64() * 1000.0
                );
                println!(
                    "  • Source lines: {}",
                    compilation_metrics.source_stats.total_lines
                );
                println!(
                    "  • Performance: {:.0} lines/sec",
                    compilation_metrics.performance.lines_per_second
                );

                if verbose {
                    // Enhanced debug-style output
                    println!("  🔍 Debug Metrics:");
                    println!(
                        "  • Tokenization: {:.2}ms",
                        compilation_metrics
                            .performance
                            .tokenization_time
                            .as_secs_f64()
                            * 1000.0
                    );
                    println!(
                        "  • Parsing: {:.2}ms",
                        compilation_metrics.performance.parsing_time.as_secs_f64() * 1000.0
                    );
                    println!(
                        "  • Type checking: {:.2}ms",
                        compilation_metrics
                            .performance
                            .type_checking_time
                            .as_secs_f64()
                            * 1000.0
                    );
                    println!(
                        "  • Bytecode generation: {:.2}ms",
                        compilation_metrics
                            .performance
                            .bytecode_generation_time
                            .as_secs_f64()
                            * 1000.0
                    );
                    println!(
                        "  • Opcodes generated: {}",
                        compilation_metrics.opcode_stats.total_opcodes
                    );
                    println!(
                        "  • Unique opcodes: {}",
                        compilation_metrics.opcode_stats.usage_frequency.len()
                    );
                    println!(
                        "  • Peak memory: {}KB",
                        compilation_metrics.memory_analytics.peak_memory_usage / 1024
                    );
                    println!(
                        "  • Compression ratio: {:.1}x",
                        compilation_metrics.bytecode_analytics.compression_ratio
                    );

                    // Show top opcodes in debug mode
                    if !compilation_metrics.opcode_stats.top_opcodes.is_empty() {
                        println!("  • Top opcodes:");
                        for (opcode, count) in
                            compilation_metrics.opcode_stats.top_opcodes.iter().take(5)
                        {
                            println!("    - {}: {} uses", opcode, count);
                        }
                    }
                }
            }
        }

        // Export metrics if requested
        if let Some(metrics_path) = metrics_output {
            let exported = export_metrics(&compilation_metrics, metrics_format)?;
            fs::write(&metrics_path, exported)
                .map_err(|e| format!("Failed to write metrics: {}", e))?;

            if !quiet {
                println!("📈 Metrics exported to {}", metrics_path.display());
            }
        }
    } else {
        // Standard compilation (with optional debug bytecode capture)
        let (bytecode, compilation_log) = if debug_bytecode {
            // When debug_bytecode is enabled, request compilation log from the compiler
            match DslCompiler::compile_with_config_and_log(&source_code, &config) {
                Ok((bc, log)) => (bc, log),
                Err(e) => {
                    return Err(Box::new(e));
                }
            }
        } else {
            (
                DslCompiler::compile_with_config(&source_code, &config)?,
                Vec::new(),
            )
        };

        let compile_time = start_time.elapsed();

        // Write bytecode
        fs::write(&output_path, &bytecode)
            .map_err(|e| format!("Failed to write bytecode: {}", e))?;

        if !quiet {
            println!("✅ Compilation successful!");

            if summary || verbose {
                println!("📊 Summary:");
                println!("  • Source: {}", source.display());
                println!("  • Output: {}", output_path.display());
                println!("  • Bytecode size: {} bytes", bytecode.len());
                println!(
                    "  • Compilation time: {:.2}ms",
                    compile_time.as_secs_f64() * 1000.0
                );
            }

            // If debug-bytecode was enabled, show a short diagnostic summary
            if debug_bytecode && !compilation_log.is_empty() {
                println!("\n🔎 Bytecode diagnostics (captured):");
                for line in compilation_log.iter().take(20) {
                    println!("  {}", line);
                }
                if compilation_log.len() > 20 {
                    println!("  ... and {} more lines", compilation_log.len() - 20);
                }
            }
        }
    }

    Ok(())
}

fn handle_compile_multi(
    main: PathBuf,
    modules: Vec<PathBuf>,
    output: Option<PathBuf>,
    mode: CompilationMode,
    constraint_cache: bool,
    v2_preview: bool,
    mut _metrics: bool,
    _metrics_output: Option<PathBuf>,
    _metrics_format: ExportFormat,
    summary: bool,
    _debug_bytecode: bool,
    verbose: bool,
    quiet: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if verbose && !quiet {
        println!("📦 Multi-file compilation mode");
        println!("   Main: {}", main.display());
        for module in &modules {
            println!("   Module: {}", module.display());
        }
    }

    // Convert paths to strings
    let main_str = main.to_string_lossy().to_string();
    let mut module_strs = Vec::new();
    // Add main to modules list as compile_modules iteration expectation depends on implementation details
    // but typically we pass all involved files to modules list for merging, and specify entry point.
    // Looking at compile_modules implementation (line 360-391), it iterates ALL module_files.
    // Use heuristic: pass logic requires explicit list.
    
    // Actually compile_modules logic:
    // verifies entry_point is in module_files?
    // "if file_path == entry_point { set_main } else { add_module }"
    // So we need to ensure ALL files (including main) are in `module_files`.
    
    module_strs.push(main_str.clone());
    for m in &modules {
        module_strs.push(m.to_string_lossy().to_string());
    }

    let config = CompilationConfig::new(mode)
        .with_constraint_cache(constraint_cache)
        .with_v2_preview(v2_preview);

    let start_time = std::time::Instant::now();

    // Use the library function instead of broken manual implementation
    let bytecode = DslCompiler::compile_modules(module_strs, &main_str, &config)?;

    let compile_time = start_time.elapsed();

    // Determine output path
    let output_path = output.unwrap_or_else(|| {
        let mut path = main.clone();
        path.set_extension("fbin");
        path
    });

    // Write bytecode
    fs::write(&output_path, &bytecode)?;
    
    if !quiet {
        println!("✅ Compilation successful");
        println!("💾 Bytecode written to: {}", output_path.display());
        
        if summary || verbose {
            println!("\n📈 Compilation Summary:");
            println!("   Output file: {}", output_path.display());
            println!("   Bytecode size: {} bytes", bytecode.len());
            println!("   Merged modules: {}", modules.len() + 1);
            println!("   Time: {:.2}ms", compile_time.as_secs_f64() * 1000.0);
        }
    }

    Ok(())
}

fn handle_build(
    path: PathBuf,
    output: Option<PathBuf>,
    mode: CompilationMode,
    constraint_cache: bool,
    v2_preview: bool,
    metrics: bool,
    metrics_output: Option<PathBuf>,
    metrics_format: ExportFormat,
    summary: bool,
    debug_bytecode: bool,
    verbose: bool,
    quiet: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = path.join("five.toml");
    if !config_path.exists() {
        return Err(format!("five.toml not found in {}", path.display()).into());
    }

    if verbose && !quiet {
        println!("📖 Reading configuration from {}", config_path.display());
    }

    let config_content = fs::read_to_string(&config_path)?;
    let config: FiveConfig = toml::from_str(&config_content)
        .map_err(|e| format!("Failed to parse five.toml: {}", e))?;

    // Determine entry point
    let entry_point_str = if let Some(build) = &config.build {
        build.entry_point.clone()
    } else {
        None
    }
    .or_else(|| {
        config.modules.get("main").and_then(|v| v.first().cloned())
    })
    .ok_or("Could not determine entry point. Please specify [build] entry_point or [modules] main in five.toml")?;

    let entry_point = path.join(&entry_point_str);
    if !entry_point.exists() {
        return Err(format!("Entry point not found: {}", entry_point.display()).into());
    }

    // Collect all modules
    let mut modules = Vec::new();
    for files in config.modules.values() {
        for file in files {
            let file_path = path.join(file);
            if !file_path.exists() {
                return Err(format!("Module file not found: {}", file_path.display()).into());
            }
            if file_path != entry_point {
                 modules.push(file_path);
            }
        }
    }

    // Determine output
    let output_path = output.unwrap_or_else(|| {
        let target_dir = path.join("target");
        if !target_dir.exists() {
            let _ = fs::create_dir(&target_dir);
        }
        target_dir.join(format!("{}.fbin", config.project.name))
    });

    handle_compile_multi(
        entry_point,
        modules,
        Some(output_path),
        mode,
        constraint_cache,
        v2_preview,
        metrics,
        metrics_output,
        metrics_format,
        summary,
        debug_bytecode,
        verbose,
        quiet,
    )
}

fn handle_analyze(
    source: PathBuf,
    _depth: AnalysisDepth,
    _output: Option<PathBuf>,
    _format: OutputFormat,
    _performance: bool,
    _complexity: bool,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if verbose {
        println!("🔍 Analyzing {}...", source.display());
    }

    // Read source file
    let source_code =
        fs::read_to_string(&source).map_err(|e| format!("Failed to read source file: {}", e))?;

    // Compile with metrics to get analysis data (using default config)
    let (bytecode, metrics) =
        DslCompiler::compile_with_metrics(&source_code, CompilationMode::Testing, true)?;

    println!("📋 Analysis Report for {}", source.display());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // Source Statistics
    println!("\n📄 Source Code:");
    println!("  • Total lines: {}", metrics.source_stats.total_lines);
    println!(
        "  • Code lines: {} ({:.1}%)",
        metrics.source_stats.code_lines,
        (metrics.source_stats.code_lines as f64 / metrics.source_stats.total_lines as f64) * 100.0
    );
    println!("  • Comment lines: {}", metrics.source_stats.comment_lines);
    println!("  • Blank lines: {}", metrics.source_stats.blank_lines);
    println!("  • Total tokens: {}", metrics.source_stats.total_tokens);

    // Compilation Results
    println!("\n⚙️ Compilation:");
    println!("  • Bytecode size: {} bytes", bytecode.len());
    println!(
        "  • Compression ratio: {:.1}x",
        metrics.bytecode_analytics.compression_ratio
    );
    println!(
        "  • Instructions density: {:.1} per KB",
        metrics.bytecode_analytics.instructions_per_kb
    );

    // Performance Metrics
    println!("\n⚡ Performance:");
    println!(
        "  • Tokenization: {:.2}ms",
        metrics.performance.tokenization_time.as_secs_f64() * 1000.0
    );
    println!(
        "  • Parsing: {:.2}ms",
        metrics.performance.parsing_time.as_secs_f64() * 1000.0
    );
    println!(
        "  • Type checking: {:.2}ms",
        metrics.performance.type_checking_time.as_secs_f64() * 1000.0
    );
    println!(
        "  • Code generation: {:.2}ms",
        metrics.performance.bytecode_generation_time.as_secs_f64() * 1000.0
    );
    println!(
        "  • Total: {:.2}ms",
        metrics.performance.total_compilation_time.as_secs_f64() * 1000.0
    );
    println!(
        "  • Throughput: {:.0} lines/sec",
        metrics.performance.lines_per_second
    );

    // Language Features
    if !metrics.source_stats.token_distribution.is_empty() {
        println!("\n🎯 Language Features:");
        let mut features = Vec::new();

        for (token, count) in &metrics.source_stats.token_distribution {
            if token.contains("If") {
                features.push(("Conditionals", *count));
            }
            if token.contains("While") {
                features.push(("Loops", *count));
            }
            if token.contains("Match") {
                features.push(("Pattern Matching", *count));
            }
            if token.contains("Mut") {
                features.push(("Mutable State", *count));
            }
            if token.contains("Return") {
                features.push(("Function Returns", *count));
            }
        }

        for (feature, count) in features.iter().take(10) {
            println!("  • {}: {} uses", feature, count);
        }
    }

    println!("\n✅ Analysis complete");

    Ok(())
}

fn handle_benchmark(
    _path: PathBuf,
    _iterations: usize,
    _warmup: usize,
    _output: Option<PathBuf>,
    _baseline: Option<PathBuf>,
    _save_baseline: bool,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if verbose {
        println!("🏃 Running benchmark...");
    }

    println!("⏳ Benchmark functionality coming soon!");
    println!("   This will provide comprehensive performance testing");
    println!("   with statistical analysis and baseline comparisons.");

    Ok(())
}

fn handle_watch(
    _path: PathBuf,
    _command: Vec<String>,
    _metrics: bool,
    _interval: u64,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if verbose {
        println!("👀 Starting watch mode...");
    }

    println!("⏳ Watch mode functionality coming soon!");
    println!("   This will provide real-time compilation");
    println!("   with live metrics and hot reloading.");

    Ok(())
}

fn handle_init(
    _name: String,
    _template: ProjectTemplate,
    _dir: Option<PathBuf>,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if verbose {
        println!("🏗️ Initializing project...");
    }

    println!("⏳ Project scaffolding functionality coming soon!");
    println!("   This will generate project templates");
    println!("   with best practices and examples.");

    Ok(())
}

fn handle_inspect(
    bytecode_path: PathBuf,
    disasm: bool,
    _patterns: bool,
    _performance: bool,
    _output: Option<PathBuf>,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if verbose {
        println!("🔍 Inspecting bytecode: {}", bytecode_path.display());
    }

    let bytecode = fs::read(&bytecode_path)
        .map_err(|e| format!("Failed to read bytecode file: {}", e))?;

    println!("File: {}", bytecode_path.display());
    println!("Size: {} bytes", bytecode.len());

    if disasm {
        println!("\nDisassembly:");
        disassemble_bytecode(&bytecode);
    } else {
        println!("Use --disasm to view bytecode instructions.");
    }

    Ok(())
}

fn read_vle_at(data: &[u8], offset: usize) -> (u64, usize) {
    let mut value: u64 = 0;
    let mut shift = 0;
    let mut len = 0;
    while offset + len < data.len() {
        let byte = data[offset + len];
        len += 1;
        value |= ((byte & 0x7F) as u64) << shift;
        if byte & 0x80 == 0 { break; }
        shift += 7;
    }
    (value, len)
}

fn disassemble_bytecode(bytecode: &[u8]) {
    // Magic bytes check (5IVE)
    let start_offset = if bytecode.len() >= 4 && &bytecode[0..4] == b"5IVE" {
        10 // Skip 10 byte header (magic + flags + counts)
    } else {
        0
    };

    println!("Disassembly relative to offset {}:", start_offset);
    let mut pc = start_offset;
    while pc < bytecode.len() {
        let opcode = bytecode[pc];
        print!("  {:04x}: {:02x} ", pc, opcode);

        match opcode {
            0x00 => println!("HALT"),
            0x01 => {
                let (val, len) = read_vle_at(bytecode, pc + 1);
                println!("JMP {}", val);
                pc += len;
            }
            0x02 => {
                let (val, len) = read_vle_at(bytecode, pc + 1);
                println!("JMP_IF {}", val);
                pc += len;
            }
            0x03 => {
                let (val, len) = read_vle_at(bytecode, pc + 1);
                println!("JMP_IF_NOT {}", val);
                pc += len;
            }
            0x04 => println!("REQUIRE"),
            0x05 => println!("ASSERT"),
            0x06 => println!("RETURN"),
            0x07 => println!("RETURN_VALUE"),
            0x08 => println!("NOP"),
            0x09 => println!("BR_EQ_U8"),
            
            // Stack Ops 0x10-0x1F
            0x10 => println!("POP"),
            0x11 => println!("DUP"),
            0x12 => println!("DUP2"),
            0x13 => println!("SWAP"),
            0x14 => println!("PICK"),
            0x15 => println!("ROT"),
            0x16 => println!("DROP"),
            0x17 => println!("OVER"),
            0x18 => {
                 if pc + 1 < bytecode.len() {
                    println!("PUSH_U8 {}", bytecode[pc + 1]);
                    pc += 1;
                 } else { println!("PUSH_U8 (incomplete)"); }
            }
            0x19 => {
                let (val, len) = read_vle_at(bytecode, pc + 1);
                println!("PUSH_U16 {}", val);
                pc += len;
            }
            0x1A => {
                let (val, len) = read_vle_at(bytecode, pc + 1);
                println!("PUSH_U32 {}", val);
                pc += len;
            }
            0x1B => {
                let (val, len) = read_vle_at(bytecode, pc + 1);
                println!("PUSH_U64 {}", val);
                pc += len;
            }
            0x1C => {
                let (val, len) = read_vle_at(bytecode, pc + 1);
                println!("PUSH_I64 {}", val);
                pc += len;
            }
            0x1D => {
                 if pc + 1 < bytecode.len() {
                    println!("PUSH_BOOL {}", bytecode[pc + 1]);
                    pc += 1;
                 } else { println!("PUSH_BOOL (incomplete)"); }
            }
            0x1E => {
                println!("PUSH_PUBKEY (32 bytes)");
                pc += 32;
            }
            0x1F => {
                println!("PUSH_U128 (16 bytes)");
                pc += 16;
            }

            // Arithmetic 0x20-0x2F
            0x20 => println!("ADD"),
            0x21 => println!("SUB"),
            0x22 => println!("MUL"),
            0x23 => println!("DIV"),
            0x24 => println!("MOD"),
            0x25 => println!("GT"),
            0x26 => println!("LT"),
            0x27 => println!("EQ"),
            0x28 => println!("GTE"),
            0x29 => println!("LTE"),
            0x2A => println!("NEQ"),
            0x2B => println!("NEG"),
            0x2C => println!("ADD_CHECKED"),
            0x2D => println!("SUB_CHECKED"),
            0x2E => println!("MUL_CHECKED"),

            // Logical & Bitwise 0x30-0x3F
            0x30 => println!("AND"),
            0x31 => println!("OR"),
            0x32 => println!("NOT"),
            0x33 => println!("XOR"),
            0x34 => println!("BITWISE_NOT"),
            0x35 => println!("BITWISE_AND"),
            0x36 => println!("BITWISE_OR"),
            0x37 => println!("BITWISE_XOR"),
            0x38 => println!("SHIFT_LEFT"),
            0x39 => println!("SHIFT_RIGHT"),
            0x3A => println!("SHIFT_RIGHT_ARITH"),
            0x3B => println!("ROTATE_LEFT"),
            0x3C => println!("ROTATE_RIGHT"),
            0x3D => println!("BYTE_SWAP_16"),
            0x3E => println!("BYTE_SWAP_32"),
            0x3F => println!("BYTE_SWAP_64"),

            // Memory 0x40-0x4F
            0x40 => println!("STORE"),
            0x41 => println!("LOAD"),
            0x42 => { // STORE_FIELD
                if pc + 1 < bytecode.len() {
                    let acc_idx = bytecode[pc + 1];
                    let (offset, len) = read_vle_at(bytecode, pc + 2);
                    println!("STORE_FIELD acc:{} offset:{}", acc_idx, offset);
                    pc += 1 + len;
                } else { println!("STORE_FIELD (incomplete)"); }
            }
            0x43 => { // LOAD_FIELD
                if pc + 1 < bytecode.len() {
                    let acc_idx = bytecode[pc + 1];
                    let (offset, len) = read_vle_at(bytecode, pc + 2);
                    println!("LOAD_FIELD acc:{} offset:{}", acc_idx, offset);
                    pc += 1 + len;
                } else { println!("LOAD_FIELD (incomplete)"); }
            }
            0x44 => println!("LOAD_INPUT"),
            0x45 => println!("STORE_GLOBAL"),
            0x46 => println!("LOAD_GLOBAL"),
            0x47 => println!("LOAD_EXTERNAL_FIELD"),

            // Account 0x50-0x5F
            0x50 => println!("CREATE_ACCOUNT"),
            0x51 => println!("LOAD_ACCOUNT"),
            0x52 => println!("SAVE_ACCOUNT"),
            0x53 => println!("GET_ACCOUNT"),
            0x54 => println!("GET_LAMPORTS"),
            0x55 => println!("SET_LAMPORTS"),
            0x56 => println!("GET_DATA"),
            0x57 => println!("GET_KEY"),
            0x58 => println!("GET_OWNER"),
            0x59 => println!("TRANSFER"),
            0x5A => println!("TRANSFER_SIGNED"),

            // Array/String 0x60-0x6F
            0x60 => {
                 if pc + 1 < bytecode.len() {
                    println!("CREATE_ARRAY cap:{}", bytecode[pc + 1]);
                    pc += 1;
                 } else { println!("CREATE_ARRAY (incomplete)"); }
            }
            0x61 => println!("PUSH_ARRAY_LITERAL"),
            0x62 => println!("ARRAY_INDEX"),
            0x63 => println!("ARRAY_LENGTH"),
            0x64 => println!("ARRAY_SET"),
            0x65 => println!("ARRAY_GET"),
            0x66 => println!("PUSH_STRING_LITERAL"),
            0x67 => {
                let (len, len_bytes) = read_vle_at(bytecode, pc + 1);
                println!("PUSH_STRING len:{}", len);
                pc += len_bytes + len as usize;
            }

            // Constraints 0x70-0x7F
            0x70 => println!("CHECK_SIGNER"),
            0x71 => println!("CHECK_WRITABLE"),
            0x72 => println!("CHECK_OWNER"),
            0x73 => println!("CHECK_INITIALIZED"),
            0x74 => println!("CHECK_PDA"),
            0x75 => println!("CHECK_UNINITIALIZED"),
            0x76 => println!("CHECK_DEDUPE_TABLE"),
            0x77 => println!("CHECK_CACHED"),
            0x78 => println!("CHECK_COMPLEXITY_GROUP"),
            0x79 => println!("CHECK_DEDUPE_MASK"),

            // System 0x80-0x8F
            0x80 => println!("INVOKE"),
            0x81 => println!("INVOKE_SIGNED"),
            0x82 => println!("GET_CLOCK"),
            0x83 => println!("GET_RENT"),
            0x84 => println!("INIT_ACCOUNT"),
            0x85 => println!("INIT_PDA_ACCOUNT"),
            0x86 => println!("DERIVE_PDA"),
            0x87 => println!("FIND_PDA"),
            0x88 => println!("DERIVE_PDA_PARAMS"),
            0x89 => println!("FIND_PDA_PARAMS"),

            // Function 0x90-0x9F
            0x90 => { // CALL
                if pc + 3 < bytecode.len() {
                     let param_count = bytecode[pc + 1];
                     let func_addr = u16::from_le_bytes([bytecode[pc + 2], bytecode[pc + 3]]);
                     println!("CALL params:{} addr:{}", param_count, func_addr);
                     pc += 3;
                } else { println!("CALL (incomplete)"); }
            }
            0x91 => { // CALL_EXTERNAL
                if pc + 4 < bytecode.len() {
                    let acc = bytecode[pc + 1];
                    let offset = u16::from_le_bytes([bytecode[pc + 2], bytecode[pc + 3]]);
                    let params = bytecode[pc + 4];
                    println!("CALL_EXTERNAL acc:{} offset:{} params:{}", acc, offset, params);
                    pc += 4;
                } else { println!("CALL_EXTERNAL (incomplete)"); }
            }

            // Locals 0xA0-0xAF
            0xA0 => {
                 if pc + 1 < bytecode.len() {
                    println!("ALLOC_LOCALS {}", bytecode[pc + 1]);
                    pc += 1;
                 }
            }
            0xA1 => println!("DEALLOC_LOCALS"),
            0xA2 => {
                 if pc + 1 < bytecode.len() {
                    println!("SET_LOCAL {}", bytecode[pc + 1]);
                    pc += 1;
                 }
            }
            0xA3 => {
                 if pc + 1 < bytecode.len() {
                    println!("GET_LOCAL {}", bytecode[pc + 1]);
                    pc += 1;
                 }
            }
            0xA4 => {
                 if pc + 1 < bytecode.len() {
                    println!("CLEAR_LOCAL {}", bytecode[pc + 1]);
                    pc += 1;
                 }
            }
            0xA5 => {
                 if pc + 1 < bytecode.len() {
                    println!("LOAD_PARAM {}", bytecode[pc + 1]);
                    pc += 1;
                 }
            }
            0xA6 => {
                 if pc + 1 < bytecode.len() {
                    println!("STORE_PARAM {}", bytecode[pc + 1]);
                    pc += 1;
                 }
            }

            // Nibble Ops 0xD0-0xDF
            0xD0 => println!("GET_LOCAL_0"),
            0xD1 => println!("GET_LOCAL_1"),
            0xD2 => println!("GET_LOCAL_2"),
            0xD3 => println!("GET_LOCAL_3"),
            0xD4 => println!("SET_LOCAL_0"),
            0xD5 => println!("SET_LOCAL_1"),
            0xD6 => println!("SET_LOCAL_2"),
            0xD7 => println!("SET_LOCAL_3"),
            0xD8 => println!("PUSH_0"),
            0xD9 => println!("PUSH_1"),
            0xDA => println!("PUSH_2"),
            0xDB => println!("PUSH_3"),
            0xDC => println!("LOAD_PARAM_0"),
            0xDD => println!("LOAD_PARAM_1"),
            0xDE => println!("LOAD_PARAM_2"),
            0xDF => println!("LOAD_PARAM_3"),

            // Optional/Result 0xF0-0xFF
            0xF0 => println!("RESULT_OK"),
            0xF1 => println!("RESULT_ERR"),
            0xF2 => println!("OPTIONAL_SOME"),
            0xF3 => println!("OPTIONAL_NONE"),
            0xF4 => println!("OPTIONAL_UNWRAP"),
            0xF5 => println!("OPTIONAL_IS_SOME"),
            0xF6 => println!("OPTIONAL_GET_VALUE"),
            0xF7 => { // BULK_LOAD_FIELD_N
                if pc + 1 < bytecode.len() {
                    let acc = bytecode[pc + 1];
                    let (count, len) = read_vle_at(bytecode, pc + 2);
                    println!("BULK_LOAD_FIELD_N acc:{} count:{}", acc, count);
                    pc += 1 + len;
                }
            }
            _ => println!("UNKNOWN ({})", opcode),
        }
        pc += 1;
    }
}
