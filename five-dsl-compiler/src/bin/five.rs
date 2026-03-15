// Five DSL compiler CLI.

use clap::{ArgAction, Parser, Subcommand};
use five_dsl_compiler::{
    export_metrics, CompilationConfig, CompilationMode, DslCompiler, ExportFormat,
};
use std::env;
use std::fs;
use std::path::PathBuf;

/// Five DSL Compiler CLI
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

        /// Suppress deprecation warnings for legacy @session syntax.
        #[arg(long)]
        suppress_session_deprecation_warnings: bool,
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

        /// Suppress deprecation warnings for legacy @session syntax.
        #[arg(long)]
        suppress_session_deprecation_warnings: bool,
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

        /// Suppress deprecation warnings for legacy @session syntax.
        #[arg(long)]
        suppress_session_deprecation_warnings: bool,
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
            suppress_session_deprecation_warnings,
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
            suppress_session_deprecation_warnings,
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
            suppress_session_deprecation_warnings,
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
            suppress_session_deprecation_warnings,
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
            suppress_session_deprecation_warnings,
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
            suppress_session_deprecation_warnings,
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
    suppress_session_deprecation_warnings: bool,
    verbose: bool,
    quiet: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if suppress_session_deprecation_warnings {
        env::set_var("FIVE_SUPPRESS_SESSION_DEPRECATION_WARNINGS", "1");
    }
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
        let (bytecode, mut compilation_metrics) =
            DslCompiler::compile_with_metrics_and_config(&source_code, &config)?;

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
    suppress_session_deprecation_warnings: bool,
    verbose: bool,
    quiet: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if suppress_session_deprecation_warnings {
        env::set_var("FIVE_SUPPRESS_SESSION_DEPRECATION_WARNINGS", "1");
    }
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
    suppress_session_deprecation_warnings: bool,
    verbose: bool,
    quiet: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if suppress_session_deprecation_warnings {
        env::set_var("FIVE_SUPPRESS_SESSION_DEPRECATION_WARNINGS", "1");
    }
    let config_path = path.join("five.toml");
    if !config_path.exists() {
        return Err(format!("five.toml not found in {}", path.display()).into());
    }

    if verbose && !quiet {
        println!("📖 Reading configuration from {}", config_path.display());
    }

    let config_content = fs::read_to_string(&config_path)?;
    let config: FiveConfig =
        toml::from_str(&config_content).map_err(|e| format!("Failed to parse five.toml: {}", e))?;

    // Determine entry point
    let entry_point_str = if let Some(build) = &config.build {
        build.entry_point.clone()
    } else {
        None
    }
    .or_else(|| {
        config.modules.get("main").and_then(|v| v.first().cloned())
    })
    .ok_or("Could not determine entry point. Please specify project.entry_point or build.entry_point in five.toml")?;

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
        suppress_session_deprecation_warnings,
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

    let bytecode =
        fs::read(&bytecode_path).map_err(|e| format!("Failed to read bytecode file: {}", e))?;

    println!("File: {}", bytecode_path.display());
    println!("Size: {} bytes", bytecode.len());

    if disasm {
        println!("\nDisassembly:");
        five_dsl_compiler::disassembler::disassemble_bytecode(&bytecode);
    } else {
        println!("Use --disasm to view bytecode instructions.");
    }

    Ok(())
}
