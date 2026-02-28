// Bytecode Analyzer Module
//
// This module provides advanced bytecode analysis capabilities for the FIVE DSL compiler.
// It includes instruction decoding, control flow analysis, stack effect analysis,
// pattern detection, and comprehensive reporting for optimization and debugging.

use crate::ast::AstNode;
use five_protocol::ResourceRequirements;
use five_vm_mito::error::VMError;

pub mod analysis;
pub mod decoder;
pub mod types;

pub use types::*;

/// Advanced bytecode analyzer with intelligent instruction decoding
pub struct AdvancedBytecodeAnalyzer {
    /// Raw bytecode to analyze
    pub(crate) bytecode: Vec<u8>,

    /// Current position during analysis
    pub(crate) position: usize,

    /// Parsed header feature flags (0 if no header)
    pub(crate) features: u32,

    /// Start offset of instruction stream
    pub(crate) start_offset: usize,

    /// Decoded instructions with full analysis
    pub(crate) instructions: Vec<InstructionAnalysis>,

    /// Control flow graph of the bytecode
    pub(crate) control_flow: ControlFlowGraph,

    /// Stack effect analysis results
    pub(crate) stack_analysis: StackAnalysis,
}

impl AdvancedBytecodeAnalyzer {
    /// Create a new analyzer for the given bytecode
    pub fn new(bytecode: Vec<u8>) -> Self {
        Self {
            bytecode,
            position: 0,
            features: 0,
            start_offset: 0,
            instructions: Vec::new(),
            control_flow: ControlFlowGraph {
                basic_blocks: Vec::new(),
                edges: Vec::new(),
                entry_points: Vec::new(),
            },
            stack_analysis: StackAnalysis {
                stack_depths: Vec::new(),
                max_stack_depth: 0,
                min_stack_depth: 0,
                is_consistent: true,
                resource_requirements: None,
            },
        }
    }

    /// Perform complete intelligent analysis of the bytecode
    pub fn analyze(&mut self) -> Result<BytecodeAnalysisResult, VMError> {
        // Phase 1: Decode all instructions with semantic understanding
        decoder::decode_instructions(self)?;

        // Phase 2: Analyze control flow and build CFG
        analysis::analyze_control_flow(self)?;

        // Phase 3: Perform stack effect analysis
        analysis::analyze_stack_effects(self)?;

        // Phase 4: Detect patterns and optimizations
        let patterns = analysis::detect_patterns(self)?;

        // Phase 5: Generate comprehensive analysis report
        Ok(BytecodeAnalysisResult {
            instructions: self.instructions.clone(),
            control_flow: self.control_flow.clone(),
            stack_analysis: self.stack_analysis.clone(),
            patterns,
            summary: analysis::generate_summary(self),
        })
    }

    /// Decode value type byte to string name
    pub fn decode_value_type_name(&self, type_byte: u8) -> String {
        decoder::decode_value_type_name(type_byte)
    }

    /// Get bytecode reference
    pub fn get_bytecode(&self) -> &[u8] {
        &self.bytecode
    }

    /// Get decoded instructions
    pub fn get_instructions(&self) -> &[InstructionAnalysis] {
        &self.instructions
    }

    /// Calculate resource requirements from AST and bytecode analysis
    pub fn calculate_resource_requirements(
        &mut self,
        ast: &AstNode,
    ) -> Result<ResourceRequirements, VMError> {
        analysis::calculate_resource_requirements(self, ast)
    }
}

impl Default for AdvancedBytecodeAnalyzer {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use five_protocol::opcodes::*;

    #[test]
    fn test_analyzer_creation() {
        let analyzer = AdvancedBytecodeAnalyzer::new(vec![0x35, 0x49, 0x56, 0x45]); // "5IVE"
        assert_eq!(analyzer.bytecode.len(), 4);
        assert_eq!(analyzer.position, 0);
    }

    #[test]
    fn test_value_type_decoding() {
        let analyzer = AdvancedBytecodeAnalyzer::new(Vec::new());

        assert_eq!(analyzer.decode_value_type_name(0x01), "U8");
        assert_eq!(analyzer.decode_value_type_name(0x02), "U64");
        assert_eq!(analyzer.decode_value_type_name(0x03), "BOOL");
        assert_eq!(analyzer.decode_value_type_name(0x04), "PUBKEY");
        assert_eq!(analyzer.decode_value_type_name(0x05), "STRING");
    }

    #[test]
    fn test_instruction_categorization() {
        assert_eq!(
            decoder::categorize_instruction(HALT),
            InstructionCategory::ControlFlow
        );
        assert_eq!(
            decoder::categorize_instruction(ADD),
            InstructionCategory::Arithmetic
        );
        assert_eq!(
            decoder::categorize_instruction(PUSH_U64),
            InstructionCategory::Stack
        );
    }

    #[test]
    fn test_empty_bytecode_analysis() {
        let mut analyzer = AdvancedBytecodeAnalyzer::new(Vec::new());
        let result = analyzer.analyze();
        assert!(result.is_ok());

        let analysis = result.unwrap();
        assert_eq!(analysis.instructions.len(), 0);
        assert_eq!(analysis.summary.total_instructions, 0);
    }
}
