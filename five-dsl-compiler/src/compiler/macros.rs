// Compiler Macros Module
//
// Provides macros to eliminate DRY violations in compilation phases.
// Follows the same pattern as bytecode_generator/ast_generator/macros.rs

/// Execute a compilation phase with automatic metrics and error handling.
///
/// This macro eliminates the repetitive pattern of:
/// - Starting metrics phase
/// - Executing code block
/// - Converting VMError to CompilerError
/// - Recording errors in error collector
/// - Recording errors in metrics
/// - Ending metrics phase
///
/// # Usage
///
/// ```ignore
/// let tokens = execute_phase!(
///     "tokenization",
///     metrics,
///     error_collector,
///     source,
///     ErrorCategory::Syntax,
///     {
///         let mut tokenizer = DslTokenizer::new(source);
///         tokenizer.tokenize()
///     }
/// )?;
/// ```
///
/// # Arguments
///
/// * `$phase` - Phase name (e.g., "tokenization", "parsing")
/// * `$metrics` - Mutable reference to MetricsCollector
/// * `$error_collector` - Mutable reference to ErrorCollector
/// * `$source` - Source code string for error context
/// * `$category` - ErrorCategory for this phase
/// * `$body` - Code block that returns Result<T, VMError>
///
/// # Returns
///
/// Returns `Result<T, CompilerError>` where T is the success type from the body
#[macro_export]
macro_rules! execute_phase {
    (
        $phase:expr,
        $metrics:expr,
        $error_collector:expr,
        $source:expr,
        $filename:expr,
        $category:expr,
        $body:block
    ) => {{
        $metrics.start_phase($phase);
        let result: Result<_, five_vm_mito::error::VMError> = $body;
        match result {
            Ok(value) => {
                $metrics.end_phase();
                Ok(value)
            }
            Err(vm_error) => {
                let compiler_error = $crate::compiler::error_handling::convert_and_collect_error(
                    vm_error,
                    $category,
                    $phase,
                    $source,
                    $filename,
                    $error_collector,
                    $metrics,
                );
                Err(compiler_error)
            }
        }
    }};
}

/// Initialize compilation context (error system, metrics, error collector).
///
/// This macro eliminates the repetitive initialization pattern that appears
/// at the start of every compilation method.
///
/// # Usage
///
/// ```ignore
/// let (mut metrics, mut error_collector, start_time) = init_compilation_context!();
/// ```
///
/// # Returns
///
/// Returns a tuple of:
/// - `MetricsCollector` - Fresh metrics collector
/// - `ErrorCollector` - Fresh error collector
/// - `Instant` (only in debug builds) - Start time for duration tracking
#[macro_export]
macro_rules! init_compilation_context {
    () => {{
        if let Err(e) = $crate::error::integration::initialize_error_system() {
            eprintln!("Warning: Failed to initialize enhanced error system: {}", e);
        }

        #[cfg(debug_assertions)]
        let start_time = web_time::Instant::now();

        let metrics = $crate::metrics::MetricsCollector::new();
        let error_collector = $crate::error::integration::ErrorCollector::new();

        #[cfg(debug_assertions)]
        {
            (metrics, error_collector, start_time)
        }

        #[cfg(not(debug_assertions))]
        {
            (metrics, error_collector)
        }
    }};
}

/// Emit debug compilation metrics.
///
/// This macro eliminates the repetitive debug output pattern that appears
/// at the end of compilation methods.
///
/// # Usage
///
/// ```ignore
/// debug_metrics!(
///     start_time,
///     metrics,
///     "Compilation metrics: {opcodes} opcodes, {size} bytes, {time:?} total time"
/// );
/// ```
///
/// # Format Arguments
///
/// The format string can use these placeholders:
/// - `{opcodes}` - Total opcode count
/// - `{size}` - Final bytecode size
/// - `{time}` - Total compilation time
///
/// Additional custom arguments can be passed after the format string.
#[macro_export]
macro_rules! debug_metrics {
    ($start:expr, $metrics:expr, $fmt:literal) => {
        #[cfg(debug_assertions)]
        {
            let total_time = $start.elapsed();
            let collected_metrics = $metrics.get_metrics();
            eprintln!(
                $fmt,
                opcodes = collected_metrics.opcode_stats.total_opcodes,
                size = collected_metrics.bytecode_analytics.final_size,
                time = total_time,
            );
        }
    };
    ($start:expr, $metrics:expr, $fmt:literal, $($arg:tt)*) => {
        #[cfg(debug_assertions)]
        {
            let total_time = $start.elapsed();
            let collected_metrics = $metrics.get_metrics();
            eprintln!(
                $fmt,
                $($arg)*,
                opcodes = collected_metrics.opcode_stats.total_opcodes,
                size = collected_metrics.bytecode_analytics.final_size,
                time = total_time,
            );
        }
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_init_compilation_context() {
        #[cfg(debug_assertions)]
        let (_metrics, _error_collector, _start_time) = init_compilation_context!();

        #[cfg(not(debug_assertions))]
        let (_metrics, _error_collector) = init_compilation_context!();

        // If we get here without panic, the macro works
        assert!(true);
    }
}
