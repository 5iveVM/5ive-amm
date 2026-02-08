use five_protocol::ResourceRequirements;
use std::collections::HashMap;

/// Complete analysis result with all discovered information
#[derive(Debug, Clone)]
pub struct BytecodeAnalysisResult {
    /// All decoded instructions
    pub instructions: Vec<InstructionAnalysis>,

    /// Control flow graph
    pub control_flow: ControlFlowGraph,

    /// Stack analysis results
    pub stack_analysis: StackAnalysis,

    /// Detected patterns and optimizations
    pub patterns: Vec<OptimizationPattern>,

    /// Summary statistics
    pub summary: AnalysisSummary,
}

/// Individual instruction with comprehensive analysis
#[derive(Debug, Clone)]
pub struct InstructionAnalysis {
    /// Byte offset in bytecode
    pub offset: usize,

    /// Opcode value
    pub opcode: u8,

    /// Human-readable instruction name
    pub name: String,

    /// Decoded operands with semantic understanding
    pub operands: Vec<OperandInfo>,

    /// Instruction size in bytes
    pub size: usize,

    /// Stack effect (net change in stack depth)
    pub stack_effect: i32,

    /// Compute cost estimation
    pub compute_cost: u32,

    /// Semantic description of what this instruction does
    pub description: String,

    /// Instruction category for organization
    pub category: InstructionCategory,

    /// Control flow information
    pub control_flow: ControlFlowInfo,

    /// Raw bytecode for this instruction
    pub raw_bytes: Vec<u8>,
}

/// Operand information with type understanding
#[derive(Debug, Clone)]
pub struct OperandInfo {
    /// Operand type based on ArgType from protocol
    pub operand_type: String,
    /// Raw value bytes
    pub raw_value: Vec<u8>,
    /// Decoded value (if applicable)
    pub decoded_value: Option<String>,
    /// Size in bytes
    pub size: usize,
    /// Description of what this operand represents
    pub description: String,
}

/// Instruction categories for better organization
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InstructionCategory {
    ControlFlow,
    Stack,
    Arithmetic,
    Logical,
    Memory,
    Account,
    Array,
    Constraint,
    System,
    Function,
    Local,
    Test,
    PatternFusion,
    Advanced,
    Compression,
    Unknown,
}

/// Control flow information for each instruction
#[derive(Debug, Clone)]
pub struct ControlFlowInfo {
    /// Whether this instruction can jump to other locations
    pub is_jump: bool,
    /// Jump targets (if this is a jump instruction)
    pub jump_targets: Vec<usize>,
    /// Whether this instruction can fall through to next instruction
    pub can_fall_through: bool,
    /// Whether this instruction terminates execution
    pub is_terminator: bool,
}

/// Control flow graph for the entire bytecode
#[derive(Debug, Clone)]
pub struct ControlFlowGraph {
    /// Basic blocks in the bytecode
    pub basic_blocks: Vec<BasicBlock>,
    /// Jump relationships between blocks
    pub edges: Vec<(usize, usize)>, // (from_block, to_block)
    /// Entry points (function starts, etc.)
    pub entry_points: Vec<usize>,
}

/// Basic block in control flow
#[derive(Debug, Clone)]
pub struct BasicBlock {
    /// Start offset in bytecode
    pub start: usize,
    /// End offset in bytecode
    pub end: usize,
    /// Instructions in this block
    pub instructions: Vec<usize>, // indices into instruction array
    /// Successors (blocks this can jump to)
    pub successors: Vec<usize>,
    /// Predecessors (blocks that can jump here)
    pub predecessors: Vec<usize>,
}

/// Stack effect analysis for the entire bytecode
#[derive(Debug, Clone)]
pub struct StackAnalysis {
    /// Stack depth at each instruction
    pub stack_depths: Vec<i32>,
    /// Maximum stack depth reached
    pub max_stack_depth: i32,
    /// Minimum stack depth (could be negative if underflow)
    pub min_stack_depth: i32,
    /// Stack state consistency
    pub is_consistent: bool,
    /// Resource requirements calculated from analysis
    pub resource_requirements: Option<ResourceRequirements>,
}

/// Detected optimization patterns
#[derive(Debug, Clone)]
pub struct OptimizationPattern {
    /// Pattern type
    pub pattern_type: String,
    /// Instructions involved in pattern
    pub instruction_range: (usize, usize),
    /// Description of optimization opportunity
    pub description: String,
    /// Potential savings (compute units)
    pub potential_savings: u32,
}

/// Analysis summary with key metrics
#[derive(Debug, Clone)]
pub struct AnalysisSummary {
    /// Total instructions
    pub total_instructions: usize,
    /// Total bytecode size
    pub total_size: usize,
    /// Total compute cost estimate
    pub total_compute_cost: u32,
    /// Basic block count
    pub basic_block_count: usize,
    /// Jump instruction count
    pub jump_count: usize,
    /// Function call count
    pub function_call_count: usize,
    /// Category distribution
    pub category_distribution: HashMap<InstructionCategory, usize>,
}
