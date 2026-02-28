// Compiler metrics and analytics for the Five DSL pipeline.

use core::fmt;
use serde::{
    de::{SeqAccess, Visitor},
    ser::SerializeSeq,
    Deserialize, Deserializer, Serialize, Serializer,
};
use std::collections::HashMap;
use web_time::{Duration, Instant, SystemTime};

const SAMPLE_WINDOW: usize = 16;

#[derive(Debug, Clone)]
pub struct RollingWindow {
    samples: [Duration; SAMPLE_WINDOW],
    index: usize,
    len: usize,
    total: Duration,
}

impl RollingWindow {
    pub const fn new() -> Self {
        Self {
            samples: [Duration::from_secs(0); SAMPLE_WINDOW],
            index: 0,
            len: 0,
            total: Duration::from_secs(0),
        }
    }

    pub fn add(&mut self, sample: Duration) {
        if self.len == SAMPLE_WINDOW {
            let old = self.samples[self.index];
            self.total -= old;
        } else {
            self.len += 1;
        }
        self.samples[self.index] = sample;
        self.total += sample;
        self.index = (self.index + 1) % SAMPLE_WINDOW;
    }

    pub fn iter(&self) -> impl Iterator<Item = &Duration> {
        (0..self.len).map(move |i| {
            let idx = (self.index + SAMPLE_WINDOW - self.len + i) % SAMPLE_WINDOW;
            &self.samples[idx]
        })
    }
}

impl Default for RollingWindow {
    fn default() -> Self {
        Self::new()
    }
}

impl Serialize for RollingWindow {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.len))?;
        for sample in self.iter() {
            seq.serialize_element(sample)?;
        }
        seq.end()
    }
}

impl<'de> Deserialize<'de> for RollingWindow {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct RWVisitor;
        impl<'de> Visitor<'de> for RWVisitor {
            type Value = RollingWindow;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a sequence of durations")
            }
            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut rw = RollingWindow::new();
                while let Some(value) = seq.next_element()? {
                    rw.add(value);
                }
                Ok(rw)
            }
        }
        deserializer.deserialize_seq(RWVisitor)
    }
}

/// Metrics collector for the Five DSL compiler
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilerMetrics {
    /// Session metadata
    pub session_id: String,
    pub timestamp: u64,
    pub total_compilations: u64,

    /// Compilation performance metrics
    pub performance: PerformanceMetrics,

    /// Opcode usage frequency tracking
    pub opcode_stats: OpcodeStats,

    /// Memory usage analytics
    pub memory_analytics: MemoryAnalytics,

    /// Bytecode analysis
    pub bytecode_analytics: BytecodeAnalytics,

    /// Function complexity metrics
    pub function_complexity: FunctionComplexityMetrics,

    /// Error pattern analytics
    pub error_patterns: ErrorPatternAnalytics,

    /// Source code statistics
    pub source_stats: SourceStats,
}

/// Performance metrics for each compilation phase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Tokenization phase timings
    pub tokenization_time: Duration,
    pub tokenization_samples: RollingWindow,

    /// Parsing phase timings
    pub parsing_time: Duration,
    pub parsing_samples: RollingWindow,

    /// Type checking phase timings
    pub type_checking_time: Duration,
    pub type_checking_samples: RollingWindow,

    /// Bytecode generation phase timings
    pub bytecode_generation_time: Duration,
    pub bytecode_generation_samples: RollingWindow,

    /// Total compilation time
    pub total_compilation_time: Duration,
    pub total_compilation_samples: RollingWindow,

    /// Throughput metrics
    pub lines_per_second: f64,
    pub tokens_per_second: f64,
    pub bytecode_per_second: f64,
}

impl PerformanceMetrics {
    fn record_phase(&mut self, phase: &str, duration: Duration) {
        match phase {
            "tokenization" => {
                self.tokenization_time += duration;
                self.tokenization_samples.add(duration);
            }
            "parsing" => {
                self.parsing_time += duration;
                self.parsing_samples.add(duration);
            }
            "type_checking" => {
                self.type_checking_time += duration;
                self.type_checking_samples.add(duration);
            }
            "bytecode_generation" => {
                self.bytecode_generation_time += duration;
                self.bytecode_generation_samples.add(duration);
            }
            _ => {}
        }
    }
}

/// Opcode usage frequency tracking
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OpcodeStats {
    /// Frequency count for each opcode
    pub usage_frequency: HashMap<String, u64>,

    /// Total opcodes generated
    pub total_opcodes: u64,

    /// Most used opcodes (top 10)
    pub top_opcodes: Vec<(String, u64)>,

    /// Opcode distribution by category
    pub category_distribution: HashMap<String, u64>,

    /// Advanced opcodes usage (complex operations)
    pub advanced_usage: HashMap<String, u64>,

    /// Opcode patterns (sequences of 2-3 opcodes)
    pub opcode_patterns: HashMap<String, u64>,
}

/// Memory usage analytics during compilation
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryAnalytics {
    /// Peak memory usage during compilation
    pub peak_memory_usage: u64,

    /// Memory usage by compilation phase
    pub phase_memory: HashMap<String, u64>,

    /// AST node count and memory estimation
    pub ast_node_count: u64,
    pub ast_memory_estimate: u64,

    /// Symbol table memory usage
    pub symbol_table_memory: u64,

    /// Temporary buffer allocations
    pub temp_allocations: u64,
    pub temp_deallocations: u64,
}

/// Bytecode size and optimization analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BytecodeAnalytics {
    /// Final bytecode size
    pub final_size: u64,

    /// Size distribution by section
    pub size_by_section: HashMap<String, u64>,

    /// Compression efficiency
    pub compression_ratio: f64,
    pub uncompressed_size: u64,

    /// Optimization impact tracking
    pub optimization_savings: HashMap<String, u64>,

    /// Bytecode density metrics
    pub instructions_per_kb: f64,
    pub average_instruction_size: f64,

    /// Size growth over compilation stages
    pub size_progression: Vec<u64>,
}

/// Function complexity analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionComplexityMetrics {
    /// Number of functions compiled
    pub function_count: u64,

    /// Complexity score distribution
    pub complexity_distribution: HashMap<String, u64>, // complexity_range -> count

    /// Call depth analysis
    pub max_call_depth: u32,
    pub average_call_depth: f64,

    /// Variable usage patterns
    pub variable_usage: HashMap<String, u64>, // usage_pattern -> count

    /// Instruction count per function
    pub instructions_per_function: Vec<u64>,

    /// Control flow complexity
    pub control_flow_complexity: HashMap<String, u64>,
}

/// Error pattern analytics for debugging
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ErrorPatternAnalytics {
    /// Error frequency by type
    pub error_frequency: HashMap<String, u64>,

    /// Error location patterns
    pub error_locations: HashMap<String, u64>, // phase -> count

    /// Most common error messages
    pub common_errors: Vec<(String, u64)>,

    /// Recovery success rate
    pub recovery_attempts: u64,
    pub recovery_successes: u64,

    /// Error correlation with source patterns
    pub source_error_correlation: HashMap<String, u64>,
}

/// Source code statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SourceStats {
    /// Lines of code metrics
    pub total_lines: u64,
    pub code_lines: u64,
    pub comment_lines: u64,
    pub blank_lines: u64,

    /// Token statistics
    pub total_tokens: u64,
    pub unique_tokens: u64,
    pub token_distribution: HashMap<String, u64>,

    /// Language feature usage
    pub feature_usage: HashMap<String, u64>,

    /// Complexity indicators
    pub cyclomatic_complexity: u32,
    pub nesting_depth: u32,
    pub function_length_distribution: HashMap<String, u64>,
}

impl Default for CompilerMetrics {
    fn default() -> Self {
        Self {
            session_id: uuid::Uuid::new_v4().to_string(),
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            total_compilations: 0,
            performance: PerformanceMetrics::default(),
            opcode_stats: OpcodeStats::default(),
            memory_analytics: MemoryAnalytics::default(),
            bytecode_analytics: BytecodeAnalytics::default(),
            function_complexity: FunctionComplexityMetrics::default(),
            error_patterns: ErrorPatternAnalytics::default(),
            source_stats: SourceStats::default(),
        }
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            tokenization_time: Duration::default(),
            tokenization_samples: RollingWindow::new(),
            parsing_time: Duration::default(),
            parsing_samples: RollingWindow::new(),
            type_checking_time: Duration::default(),
            type_checking_samples: RollingWindow::new(),
            bytecode_generation_time: Duration::default(),
            bytecode_generation_samples: RollingWindow::new(),
            total_compilation_time: Duration::default(),
            total_compilation_samples: RollingWindow::new(),
            lines_per_second: 0.0,
            tokens_per_second: 0.0,
            bytecode_per_second: 0.0,
        }
    }
}

impl Default for BytecodeAnalytics {
    fn default() -> Self {
        Self {
            final_size: 0,
            size_by_section: HashMap::new(),
            compression_ratio: 1.0,
            uncompressed_size: 0,
            optimization_savings: HashMap::new(),
            instructions_per_kb: 0.0,
            average_instruction_size: 0.0,
            size_progression: Vec::new(),
        }
    }
}

impl Default for FunctionComplexityMetrics {
    fn default() -> Self {
        Self {
            function_count: 0,
            complexity_distribution: HashMap::new(),
            max_call_depth: 0,
            average_call_depth: 0.0,
            variable_usage: HashMap::new(),
            instructions_per_function: Vec::new(),
            control_flow_complexity: HashMap::new(),
        }
    }
}

/// Metrics collector that instruments the compilation pipeline
pub struct MetricsCollector {
    metrics: CompilerMetrics,
    current_phase: Option<String>,
    phase_start_time: Option<Instant>,
    memory_tracker: MemoryTracker,
}

/// Memory usage tracker
struct MemoryTracker {
    baseline_memory: u64,
    current_memory: u64,
    peak_memory: u64,
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            metrics: CompilerMetrics::default(),
            current_phase: None,
            phase_start_time: None,
            memory_tracker: MemoryTracker {
                baseline_memory: Self::get_memory_usage(),
                current_memory: 0,
                peak_memory: 0,
            },
        }
    }

    /// Start timing a compilation phase
    pub fn start_phase(&mut self, phase_name: &str) {
        if let Some(_current) = &self.current_phase {
            self.end_phase(); // End previous phase if still running
        }

        self.current_phase = Some(phase_name.to_string());
        self.phase_start_time = Some(Instant::now());
        self.update_memory_usage();
    }

    /// End timing the current compilation phase
    pub fn end_phase(&mut self) {
        if let (Some(phase), Some(start_time)) = (&self.current_phase, self.phase_start_time) {
            let duration = start_time.elapsed();

            self.metrics.performance.record_phase(phase, duration);

            // Record memory usage for this phase
            self.metrics
                .memory_analytics
                .phase_memory
                .insert(phase.clone(), self.memory_tracker.current_memory);
        }

        self.current_phase = None;
        self.phase_start_time = None;
    }

    /// Record opcode usage
    pub fn record_opcode(&mut self, opcode: &str) {
        let counter = self
            .metrics
            .opcode_stats
            .usage_frequency
            .entry(opcode.to_string())
            .or_insert(0);
        *counter += 1;
        self.metrics.opcode_stats.total_opcodes += 1;

        // Update category distribution
        let category = self.get_opcode_category(opcode);
        let cat_counter = self
            .metrics
            .opcode_stats
            .category_distribution
            .entry(category)
            .or_insert(0);
        *cat_counter += 1;
    }

    /// Record source code statistics
    pub fn record_source_stats(&mut self, source: &str, tokens: &[crate::tokenizer::Token]) {
        self.metrics.source_stats.total_lines = source.lines().count() as u64;
        self.metrics.source_stats.code_lines = source
            .lines()
            .filter(|line| !line.trim().is_empty() && !line.trim().starts_with("//"))
            .count() as u64;
        self.metrics.source_stats.comment_lines = source
            .lines()
            .filter(|line| line.trim().starts_with("//"))
            .count() as u64;
        self.metrics.source_stats.blank_lines =
            source.lines().filter(|line| line.trim().is_empty()).count() as u64;

        self.metrics.source_stats.total_tokens = tokens.len() as u64;

        // Token distribution
        for token in tokens {
            let token_name = format!("{:?}", token);
            let counter = self
                .metrics
                .source_stats
                .token_distribution
                .entry(token_name)
                .or_insert(0);
            *counter += 1;
        }
    }

    /// Record compilation error
    pub fn record_error(&mut self, error: &str, phase: &str) {
        let error_counter = self
            .metrics
            .error_patterns
            .error_frequency
            .entry(error.to_string())
            .or_insert(0);
        *error_counter += 1;

        let location_counter = self
            .metrics
            .error_patterns
            .error_locations
            .entry(phase.to_string())
            .or_insert(0);
        *location_counter += 1;
    }

    /// Record bytecode analytics
    pub fn record_bytecode_analytics(&mut self, bytecode: &[u8], uncompressed_size: usize) {
        self.metrics.bytecode_analytics.final_size = bytecode.len() as u64;
        self.metrics.bytecode_analytics.uncompressed_size = uncompressed_size as u64;
        self.metrics.bytecode_analytics.compression_ratio =
            uncompressed_size as f64 / bytecode.len() as f64;

        // Calculate instruction density
        if !bytecode.is_empty() {
            self.metrics.bytecode_analytics.instructions_per_kb =
                (self.metrics.opcode_stats.total_opcodes as f64 / bytecode.len() as f64) * 1024.0;
            self.metrics.bytecode_analytics.average_instruction_size =
                bytecode.len() as f64 / self.metrics.opcode_stats.total_opcodes as f64;
        }
    }

    /// Finalize metrics collection
    pub fn finalize(&mut self) {
        self.end_phase(); // End any running phase

        // Calculate performance throughput
        if self
            .metrics
            .performance
            .total_compilation_time
            .as_secs_f64()
            > 0.0
        {
            self.metrics.performance.lines_per_second = self.metrics.source_stats.total_lines
                as f64
                / self
                    .metrics
                    .performance
                    .total_compilation_time
                    .as_secs_f64();
            self.metrics.performance.tokens_per_second = self.metrics.source_stats.total_tokens
                as f64
                / self
                    .metrics
                    .performance
                    .total_compilation_time
                    .as_secs_f64();
            self.metrics.performance.bytecode_per_second =
                self.metrics.bytecode_analytics.final_size as f64
                    / self
                        .metrics
                        .performance
                        .total_compilation_time
                        .as_secs_f64();
        }

        // Update top opcodes
        let mut opcode_vec: Vec<_> = self.metrics.opcode_stats.usage_frequency.iter().collect();
        opcode_vec.sort_by(|a, b| b.1.cmp(a.1));
        self.metrics.opcode_stats.top_opcodes = opcode_vec
            .into_iter()
            .take(10)
            .map(|(k, v)| (k.clone(), *v))
            .collect();

        // Update common errors
        let mut error_vec: Vec<_> = self.metrics.error_patterns.error_frequency.iter().collect();
        error_vec.sort_by(|a, b| b.1.cmp(a.1));
        self.metrics.error_patterns.common_errors = error_vec
            .into_iter()
            .take(10)
            .map(|(k, v)| (k.clone(), *v))
            .collect();

        self.metrics.total_compilations += 1;
        self.metrics.memory_analytics.peak_memory_usage = self.memory_tracker.peak_memory;
        self.metrics
            .performance
            .total_compilation_samples
            .add(self.metrics.performance.total_compilation_time);
    }

    /// Get the collected metrics
    pub fn get_metrics(&self) -> &CompilerMetrics {
        &self.metrics
    }

    /// Reset metrics for a new compilation session
    pub fn reset(&mut self) {
        self.metrics = CompilerMetrics::default();
        self.current_phase = None;
        self.phase_start_time = None;
        self.memory_tracker = MemoryTracker {
            baseline_memory: Self::get_memory_usage(),
            current_memory: 0,
            peak_memory: 0,
        };
    }

    // Helper methods

    fn update_memory_usage(&mut self) {
        self.memory_tracker.current_memory =
            Self::get_memory_usage() - self.memory_tracker.baseline_memory;
        if self.memory_tracker.current_memory > self.memory_tracker.peak_memory {
            self.memory_tracker.peak_memory = self.memory_tracker.current_memory;
        }
    }

    fn get_memory_usage() -> u64 {
        // Placeholder memory tracking.
        0 // Placeholder
    }

    fn get_opcode_category(&self, opcode: &str) -> String {
        match opcode {
            "PUSH" | "POP" | "DUP" | "SWAP" => "Stack Operations".to_string(),
            "ADD" | "SUB" | "MUL" | "DIV" | "MOD" => "Arithmetic".to_string(),
            "EQ" | "NE" | "LT" | "LE" | "GT" | "GE" => "Comparison".to_string(),
            "AND" | "OR" | "NOT" | "XOR" => "Logical".to_string(),
            "JMP" | "JMPT" | "JMPF" | "CALL" | "RET" => "Control Flow".to_string(),
            "LOAD" | "STORE" | "LOAD_LOCAL" | "STORE_LOCAL" => "Memory".to_string(),
            "CPI" | "CPI_SIGNED" => "Cross-Program Invocation".to_string(),
            "GET_CLOCK" | "GET_RENT" => "System Calls".to_string(),
            "PUSH_STRING_LITERAL" | "STRING_LENGTH" => "String Operations".to_string(),
            "PUSH_ARRAY_LITERAL" | "ARRAY_INDEX" | "ARRAY_LENGTH" | "ARRAY_CONCAT"
            | "CREATE_ARRAY" | "ARRAY_SET" | "ARRAY_GET" => "Array Operations".to_string(),
            "OPTIONAL_SOME" | "OPTIONAL_NONE" | "RESULT_OK" | "RESULT_ERR" => {
                "Option/Result".to_string()
            }
            _ => "Other".to_string(),
        }
    }
}

/// Metrics export formats
pub enum ExportFormat {
    Json,
    Csv,
    Dashboard,
    Toml,
}

/// Export metrics to different formats
pub fn export_metrics(
    metrics: &CompilerMetrics,
    format: ExportFormat,
) -> Result<String, Box<dyn std::error::Error>> {
    match format {
        ExportFormat::Json => Ok(serde_json::to_string_pretty(metrics)?),
        ExportFormat::Csv => {
            // Implement CSV export
            let mut csv = String::new();
            csv.push_str("metric,value\n");
            csv.push_str(&format!(
                "total_compilations,{}\n",
                metrics.total_compilations
            ));
            csv.push_str(&format!(
                "total_opcodes,{}\n",
                metrics.opcode_stats.total_opcodes
            ));
            csv.push_str(&format!(
                "final_bytecode_size,{}\n",
                metrics.bytecode_analytics.final_size
            ));
            csv.push_str(&format!(
                "compilation_time_ms,{}\n",
                metrics.performance.total_compilation_time.as_millis()
            ));
            Ok(csv)
        }
        ExportFormat::Dashboard => {
            // Implement dashboard-ready JSON format
            let dashboard_data = serde_json::json!({
                "overview": {
                    "total_compilations": metrics.total_compilations,
                    "success_rate": 100.0, // Calculate based on errors
                    "average_compile_time": metrics.performance.total_compilation_time.as_millis(),
                    "total_opcodes": metrics.opcode_stats.total_opcodes
                },
                "performance": {
                    "phases": {
                        "tokenization": metrics.performance.tokenization_time.as_millis(),
                        "parsing": metrics.performance.parsing_time.as_millis(),
                        "type_checking": metrics.performance.type_checking_time.as_millis(),
                        "bytecode_generation": metrics.performance.bytecode_generation_time.as_millis()
                    },
                    "throughput": {
                        "lines_per_second": metrics.performance.lines_per_second,
                        "tokens_per_second": metrics.performance.tokens_per_second,
                        "bytecode_per_second": metrics.performance.bytecode_per_second
                    }
                },
                "opcodes": {
                    "top_opcodes": metrics.opcode_stats.top_opcodes,
                    "category_distribution": metrics.opcode_stats.category_distribution
                },
                "bytecode": {
                    "final_size": metrics.bytecode_analytics.final_size,
                    "compression_ratio": metrics.bytecode_analytics.compression_ratio,
                    "instructions_per_kb": metrics.bytecode_analytics.instructions_per_kb
                }
            });
            Ok(serde_json::to_string_pretty(&dashboard_data)?)
        }
        ExportFormat::Toml => Ok(toml::to_string_pretty(metrics)?),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_collector_basic() {
        let mut collector = MetricsCollector::new();

        collector.start_phase("tokenization");
        std::thread::sleep(std::time::Duration::from_millis(10));
        collector.end_phase();

        collector.record_opcode("PUSH");
        collector.record_opcode("ADD");
        collector.record_opcode("PUSH");

        collector.finalize();

        let metrics = collector.get_metrics();
        assert_eq!(metrics.opcode_stats.total_opcodes, 3);
        assert_eq!(metrics.opcode_stats.usage_frequency.get("PUSH"), Some(&2));
        assert_eq!(metrics.opcode_stats.usage_frequency.get("ADD"), Some(&1));
    }

    #[test]
    fn test_metrics_export_json() {
        let metrics = CompilerMetrics::default();
        let json_export = export_metrics(&metrics, ExportFormat::Json).unwrap();
        assert!(json_export.contains("session_id"));
        assert!(json_export.contains("opcode_stats"));
    }

    #[test]
    fn test_metrics_export_csv() {
        let metrics = CompilerMetrics::default();
        let csv_export = export_metrics(&metrics, ExportFormat::Csv).unwrap();
        assert!(csv_export.contains("metric,value"));
        assert!(csv_export.contains("total_compilations"));
    }

    #[test]
    fn test_metrics_export_toml() {
        let metrics = CompilerMetrics::default();
        let toml_export = export_metrics(&metrics, ExportFormat::Toml).unwrap();
        assert!(toml_export.contains("session_id"));
        assert!(toml_export.contains("total_compilations"));
    }
}
