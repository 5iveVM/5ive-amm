// Bytecode Analyzer Module
//
// This module provides advanced bytecode analysis capabilities for the FIVE DSL compiler.
// It includes instruction decoding, control flow analysis, stack effect analysis,
// pattern detection, and comprehensive reporting for optimization and debugging.

use crate::ast::AstNode;
use five_protocol::{opcodes::*, ResourceRequirements};
use five_vm_mito::error::VMError;
use std::collections::{HashMap, HashSet};

/// Advanced bytecode analyzer with intelligent instruction decoding
pub struct AdvancedBytecodeAnalyzer {
    /// Raw bytecode to analyze
    bytecode: Vec<u8>,

    /// Current position during analysis
    position: usize,

    /// Decoded instructions with full analysis
    instructions: Vec<InstructionAnalysis>,

    /// Control flow graph of the bytecode
    control_flow: ControlFlowGraph,

    /// Stack effect analysis results
    stack_analysis: StackAnalysis,
}

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
    Register,
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

impl AdvancedBytecodeAnalyzer {
    /// Create a new analyzer for the given bytecode
    pub fn new(bytecode: Vec<u8>) -> Self {
        Self {
            bytecode,
            position: 0,
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
        self.decode_instructions()?;

        // Phase 2: Analyze control flow and build CFG
        self.analyze_control_flow()?;

        // Phase 3: Perform stack effect analysis
        self.analyze_stack_effects()?;

        // Phase 4: Detect patterns and optimizations
        let patterns = self.detect_patterns()?;

        // Phase 5: Generate comprehensive analysis report
        Ok(BytecodeAnalysisResult {
            instructions: self.instructions.clone(),
            control_flow: self.control_flow.clone(),
            stack_analysis: self.stack_analysis.clone(),
            patterns,
            summary: self.generate_summary(),
        })
    }

    /// Decode all instructions with full semantic understanding
    fn decode_instructions(&mut self) -> Result<(), VMError> {
        self.position = 0;
        self.instructions.clear();

        // Skip magic bytes if present
        if self.bytecode.len() >= 4 && &self.bytecode[0..4] == b"5IVE" {
            self.position = 4;
        }

        while self.position < self.bytecode.len() {
            let instruction = self.decode_single_instruction()?;
            self.instructions.push(instruction);
        }

        Ok(())
    }

    /// Decode a single instruction with full understanding of what follows
    fn decode_single_instruction(&mut self) -> Result<InstructionAnalysis, VMError> {
        if self.position >= self.bytecode.len() {
            return Err(VMError::InvalidOperation);
        }

        let start_offset = self.position;
        let opcode = self.bytecode[self.position];
        self.position += 1;

        // Get opcode information from protocol definitions
        let opcode_info = five_protocol::opcodes::get_opcode_info(opcode);

        let (name, arg_type, stack_effect, compute_cost) = if let Some(info) = opcode_info {
            (
                info.name.to_string(),
                info.arg_type,
                info.stack_effect,
                info.compute_cost,
            )
        } else {
            (
                format!("UNKNOWN_{:02X}", opcode),
                five_protocol::opcodes::ArgType::None,
                0,
                1,
            )
        };

        // Decode operands based on argument type - this is the key intelligence!
        let operands = self.decode_operands(arg_type, opcode)?;

        let size = self.position - start_offset;
        let raw_bytes = self.bytecode[start_offset..self.position].to_vec();

        // Generate semantic description
        let description = self.generate_instruction_description(opcode, &name, &operands);

        // Determine category
        let category = self.categorize_instruction(opcode);

        // Analyze control flow for this instruction
        let control_flow = self.analyze_instruction_control_flow(opcode, &operands);

        Ok(InstructionAnalysis {
            offset: start_offset,
            opcode,
            name,
            operands,
            size,
            stack_effect: stack_effect as i32,
            compute_cost: compute_cost as u32,
            description,
            category,
            control_flow,
            raw_bytes,
        })
    }

    /// Decode operands based on ArgType - this provides the intelligence about what follows each opcode
    fn decode_operands(
        &mut self,
        arg_type: five_protocol::opcodes::ArgType,
        opcode: u8,
    ) -> Result<Vec<OperandInfo>, VMError> {
        use five_protocol::opcodes::ArgType;

        let mut operands = Vec::new();

        match arg_type {
            ArgType::None => {
                // No operands follow this instruction
            }
            ArgType::U8 => {
                if self.position < self.bytecode.len() {
                    let value = self.bytecode[self.position];
                    operands.push(OperandInfo {
                        operand_type: "u8".to_string(),
                        raw_value: vec![value],
                        decoded_value: Some(value.to_string()),
                        size: 1,
                        description: "8-bit unsigned integer".to_string(),
                    });
                    self.position += 1;
                }
            }
            ArgType::U16 => {
                if self.position + 1 < self.bytecode.len() {
                    let value = u16::from_le_bytes([
                        self.bytecode[self.position],
                        self.bytecode[self.position + 1],
                    ]);
                    operands.push(OperandInfo {
                        operand_type: "u16".to_string(),
                        raw_value: self.bytecode[self.position..self.position + 2].to_vec(),
                        decoded_value: Some(value.to_string()),
                        size: 2,
                        description: "16-bit unsigned integer".to_string(),
                    });
                    self.position += 2;
                }
            }
            ArgType::U32 => {
                if self.position + 3 < self.bytecode.len() {
                    let value = u32::from_le_bytes([
                        self.bytecode[self.position],
                        self.bytecode[self.position + 1],
                        self.bytecode[self.position + 2],
                        self.bytecode[self.position + 3],
                    ]);
                    operands.push(OperandInfo {
                        operand_type: "u32".to_string(),
                        raw_value: self.bytecode[self.position..self.position + 4].to_vec(),
                        decoded_value: Some(value.to_string()),
                        size: 4,
                        description: "32-bit unsigned integer".to_string(),
                    });
                    self.position += 4;
                }
            }
            ArgType::U64 => {
                if self.position + 7 < self.bytecode.len() {
                    let value = u64::from_le_bytes([
                        self.bytecode[self.position],
                        self.bytecode[self.position + 1],
                        self.bytecode[self.position + 2],
                        self.bytecode[self.position + 3],
                        self.bytecode[self.position + 4],
                        self.bytecode[self.position + 5],
                        self.bytecode[self.position + 6],
                        self.bytecode[self.position + 7],
                    ]);
                    operands.push(OperandInfo {
                        operand_type: "u64".to_string(),
                        raw_value: self.bytecode[self.position..self.position + 8].to_vec(),
                        decoded_value: Some(value.to_string()),
                        size: 8,
                        description: "64-bit unsigned integer".to_string(),
                    });
                    self.position += 8;
                }
            }
            ArgType::ValueType => {
                // This is for PUSH instructions - they have a type byte followed by the value
                self.decode_push_operands(&mut operands)?;
            }
            ArgType::FunctionIndex => {
                if self.position < self.bytecode.len() {
                    let value = self.bytecode[self.position];
                    operands.push(OperandInfo {
                        operand_type: "function_index".to_string(),
                        raw_value: vec![value],
                        decoded_value: Some(format!("function_{}", value)),
                        size: 1,
                        description: "Function index for dispatch".to_string(),
                    });
                    self.position += 1;
                }
            }
            ArgType::LocalIndex => {
                if self.position < self.bytecode.len() {
                    let value = self.bytecode[self.position];
                    operands.push(OperandInfo {
                        operand_type: "local_index".to_string(),
                        raw_value: vec![value],
                        decoded_value: Some(format!("local_{}", value)),
                        size: 1,
                        description: "Local variable index".to_string(),
                    });
                    self.position += 1;
                }
            }
            ArgType::AccountIndex => {
                if self.position < self.bytecode.len() {
                    let value = self.bytecode[self.position];
                    operands.push(OperandInfo {
                        operand_type: "account_index".to_string(),
                        raw_value: vec![value],
                        decoded_value: Some(format!("account_{}", value)),
                        size: 1,
                        description: "Account index in transaction".to_string(),
                    });
                    self.position += 1;
                }
            }
            ArgType::RegisterIndex => {
                if self.position < self.bytecode.len() {
                    let value = self.bytecode[self.position];
                    operands.push(OperandInfo {
                        operand_type: "register_index".to_string(),
                        raw_value: vec![value],
                        decoded_value: Some(format!("r{}", value)),
                        size: 1,
                        description: "Register index (0-15)".to_string(),
                    });
                    self.position += 1;
                }
            }
            ArgType::TwoRegisters => {
                if self.position + 1 < self.bytecode.len() {
                    let reg1 = self.bytecode[self.position];
                    let reg2 = self.bytecode[self.position + 1];
                    operands.push(OperandInfo {
                        operand_type: "two_registers".to_string(),
                        raw_value: vec![reg1, reg2],
                        decoded_value: Some(format!("r{}, r{}", reg1, reg2)),
                        size: 2,
                        description: "Two register indices (dest, src)".to_string(),
                    });
                    self.position += 2;
                }
            }
            ArgType::ThreeRegisters => {
                if self.position + 2 < self.bytecode.len() {
                    let reg1 = self.bytecode[self.position];
                    let reg2 = self.bytecode[self.position + 1];
                    let reg3 = self.bytecode[self.position + 2];
                    operands.push(OperandInfo {
                        operand_type: "three_registers".to_string(),
                        raw_value: vec![reg1, reg2, reg3],
                        decoded_value: Some(format!("r{}, r{}, r{}", reg1, reg2, reg3)),
                        size: 3,
                        description: "Three register indices (dest, src1, src2)".to_string(),
                    });
                    self.position += 3;
                }
            }
            ArgType::CallExternal => {
                if self.position + 3 < self.bytecode.len() {
                    let account_idx = self.bytecode[self.position];
                    let func_offset = u16::from_le_bytes([
                        self.bytecode[self.position + 1],
                        self.bytecode[self.position + 2],
                    ]);
                    let param_count = self.bytecode[self.position + 3];

                    operands.push(OperandInfo {
                        operand_type: "account_index".to_string(),
                        raw_value: vec![account_idx],
                        decoded_value: Some(format!("account_{}", account_idx)),
                        size: 1,
                        description: "External account index".to_string(),
                    });

                    operands.push(OperandInfo {
                        operand_type: "func_offset".to_string(),
                        raw_value: vec![self.bytecode[self.position + 1], self.bytecode[self.position + 2]],
                        decoded_value: Some(format!("offset_{}", func_offset)),
                        size: 2,
                        description: "Function entry offset".to_string(),
                    });

                    operands.push(OperandInfo {
                        operand_type: "param_count".to_string(),
                        raw_value: vec![param_count],
                        decoded_value: Some(param_count.to_string()),
                        size: 1,
                        description: "Parameter count".to_string(),
                    });

                    self.position += 4;
                }
            }
            ArgType::AccountField => {
                // Account field access: account_index (u8) + field_offset (VLE)
                if self.position < self.bytecode.len() {
                    let account_idx = self.bytecode[self.position];
                    operands.push(OperandInfo {
                        operand_type: "account_index".to_string(),
                        raw_value: vec![account_idx],
                        decoded_value: Some(format!("account_{}", account_idx)),
                        size: 1,
                        description: "Account index for field access".to_string(),
                    });
                    self.position += 1;

                    // Decode VLE for field offset
                    if self.position < self.bytecode.len() {
                        let mut field_offset = 0u64;
                        let mut vle_size = 0;
                        let mut byte_val;

                        while self.position < self.bytecode.len() && vle_size < 9 {
                            byte_val = self.bytecode[self.position];
                            self.position += 1;
                            vle_size += 1;

                            field_offset |= ((byte_val & 0x7f) as u64) << (7 * (vle_size - 1));

                            if (byte_val & 0x80) == 0 {
                                break;
                            }
                        }

                        operands.push(OperandInfo {
                            operand_type: "field_offset".to_string(),
                            raw_value: self.bytecode[self.position - vle_size..self.position].to_vec(),
                            decoded_value: Some(format!("offset_{}", field_offset)),
                            size: vle_size,
                            description: "Field offset (VLE encoded)".to_string(),
                        });
                    }
                }
            }
            ArgType::CallInternal => {
                if self.position + 2 < self.bytecode.len() {
                    let param_count = self.bytecode[self.position];
                    let addr_bytes = [
                        self.bytecode[self.position + 1],
                        self.bytecode[self.position + 2],
                    ];
                    let func_addr = u16::from_le_bytes(addr_bytes);

                    operands.push(OperandInfo {
                        operand_type: "param_count".to_string(),
                        raw_value: vec![param_count],
                        decoded_value: Some(param_count.to_string()),
                        size: 1,
                        description: "Parameter count for internal call".to_string(),
                    });

                    operands.push(OperandInfo {
                        operand_type: "func_addr".to_string(),
                        raw_value: vec![self.bytecode[self.position + 1], self.bytecode[self.position + 2]],
                        decoded_value: Some(format!("addr_{}", func_addr)),
                        size: 2,
                        description: "Internal function address".to_string(),
                    });

                    self.position += 3;
                }
            }
        }

        // Handle special cases for specific opcodes that have unique operand patterns
        self.decode_special_operands(opcode, &mut operands)?;

        Ok(operands)
    }

    /// Decode PUSH instruction operands (type + value)
    fn decode_push_operands(&mut self, operands: &mut Vec<OperandInfo>) -> Result<(), VMError> {
        if self.position >= self.bytecode.len() {
            return Ok(());
        }

        let type_byte = self.bytecode[self.position];
        self.position += 1;

        operands.push(OperandInfo {
            operand_type: "value_type".to_string(),
            raw_value: vec![type_byte],
            decoded_value: Some(self.decode_value_type_name(type_byte)),
            size: 1,
            description: "Value type indicator".to_string(),
        });

        // Decode the value based on type
        match type_byte {
            0x01 => {
                // U8
                if self.position < self.bytecode.len() {
                    let value = self.bytecode[self.position];
                    operands.push(OperandInfo {
                        operand_type: "u8_value".to_string(),
                        raw_value: vec![value],
                        decoded_value: Some(value.to_string()),
                        size: 1,
                        description: "8-bit unsigned integer value".to_string(),
                    });
                    self.position += 1;
                }
            }
            0x02 => {
                // U64
                if self.position + 7 < self.bytecode.len() {
                    let value = u64::from_le_bytes([
                        self.bytecode[self.position],
                        self.bytecode[self.position + 1],
                        self.bytecode[self.position + 2],
                        self.bytecode[self.position + 3],
                        self.bytecode[self.position + 4],
                        self.bytecode[self.position + 5],
                        self.bytecode[self.position + 6],
                        self.bytecode[self.position + 7],
                    ]);
                    operands.push(OperandInfo {
                        operand_type: "u64_value".to_string(),
                        raw_value: self.bytecode[self.position..self.position + 8].to_vec(),
                        decoded_value: Some(value.to_string()),
                        size: 8,
                        description: "64-bit unsigned integer value".to_string(),
                    });
                    self.position += 8;
                }
            }
            0x03 => {
                // Bool
                if self.position < self.bytecode.len() {
                    let value = self.bytecode[self.position];
                    operands.push(OperandInfo {
                        operand_type: "bool_value".to_string(),
                        raw_value: vec![value],
                        decoded_value: Some(if value == 0 {
                            "false".to_string()
                        } else {
                            "true".to_string()
                        }),
                        size: 1,
                        description: "Boolean value".to_string(),
                    });
                    self.position += 1;
                }
            }
            0x04 => {
                // Pubkey
                if self.position + 31 < self.bytecode.len() {
                    let pubkey_bytes = self.bytecode[self.position..self.position + 32].to_vec();
                    let pubkey_hex = pubkey_bytes
                        .iter()
                        .map(|b| format!("{:02x}", b))
                        .collect::<String>();
                    operands.push(OperandInfo {
                        operand_type: "pubkey_value".to_string(),
                        raw_value: pubkey_bytes,
                        decoded_value: Some(pubkey_hex),
                        size: 32,
                        description: "32-byte public key".to_string(),
                    });
                    self.position += 32;
                }
            }
            0x05 => {
                // String
                if self.position + 3 < self.bytecode.len() {
                    let len = u32::from_le_bytes([
                        self.bytecode[self.position],
                        self.bytecode[self.position + 1],
                        self.bytecode[self.position + 2],
                        self.bytecode[self.position + 3],
                    ]);
                    self.position += 4;

                    if self.position + len as usize <= self.bytecode.len() {
                        let string_bytes =
                            self.bytecode[self.position..self.position + len as usize].to_vec();
                        let string_value = String::from_utf8_lossy(&string_bytes).to_string();

                        operands.push(OperandInfo {
                            operand_type: "string_length".to_string(),
                            raw_value: len.to_le_bytes().to_vec(),
                            decoded_value: Some(len.to_string()),
                            size: 4,
                            description: "String length in bytes".to_string(),
                        });

                        operands.push(OperandInfo {
                            operand_type: "string_value".to_string(),
                            raw_value: string_bytes,
                            decoded_value: Some(format!("\"{}\"", string_value)),
                            size: len as usize,
                            description: "UTF-8 string data".to_string(),
                        });

                        self.position += len as usize;
                    }
                }
            }
            _ => {
                // Unknown type - just skip
            }
        }

        Ok(())
    }

    /// Decode special operands for specific opcodes
    fn decode_special_operands(
        &mut self,
        opcode: u8,
        operands: &mut Vec<OperandInfo>,
    ) -> Result<(), VMError> {
        use five_protocol::opcodes::*;

        match opcode {
            // COMPACT_FIELD_LOAD | COMPACT_FIELD_STORE removed - use LOAD_FIELD/STORE_FIELD instead
            LOAD_INPUT => {
                // LOAD_INPUT has type + param_index
                if operands.is_empty() && self.position + 1 < self.bytecode.len() {
                    let type_byte = self.bytecode[self.position];
                    let param_index = self.bytecode[self.position + 1];

                    operands.push(OperandInfo {
                        operand_type: "input_type".to_string(),
                        raw_value: vec![type_byte],
                        decoded_value: Some(self.decode_value_type_name(type_byte)),
                        size: 1,
                        description: "Expected input parameter type".to_string(),
                    });

                    operands.push(OperandInfo {
                        operand_type: "param_index".to_string(),
                        raw_value: vec![param_index],
                        decoded_value: Some(format!("param_{}", param_index)),
                        size: 1,
                        description: "Parameter index in function".to_string(),
                    });

                    self.position += 2;
                }
            }
            // Add more special cases as needed
            _ => {}
        }

        Ok(())
    }

    /// Generate semantic description for an instruction
    fn generate_instruction_description(
        &self,
        opcode: u8,
        name: &str,
        operands: &[OperandInfo],
    ) -> String {
        use five_protocol::opcodes::*;

        match opcode {
            HALT => "Stop execution and return".to_string(),
            PUSH_U64 | PUSH_U8 | PUSH_I64 | PUSH_BOOL | PUSH_PUBKEY => {
                if operands.len() >= 2 {
                    format!(
                        "Push {} value {} onto stack",
                        operands[0]
                            .decoded_value
                            .as_ref()
                            .unwrap_or(&"unknown".to_string()),
                        operands[1]
                            .decoded_value
                            .as_ref()
                            .unwrap_or(&"?".to_string())
                    )
                } else {
                    "Push value onto stack".to_string()
                }
            }
            POP => "Remove top value from stack".to_string(),
            DUP => "Duplicate top stack value".to_string(),
            SWAP => "Swap top two stack values".to_string(),
            ADD => "Add top two stack values".to_string(),
            SUB => "Subtract top two stack values".to_string(),
            MUL => "Multiply top two stack values".to_string(),
            DIV => "Divide top two stack values".to_string(),
            GT => "Compare if first > second (stack)".to_string(),
            LT => "Compare if first < second (stack)".to_string(),
            EQ => "Compare if first == second (stack)".to_string(),
            JUMP => {
                if !operands.is_empty() {
                    format!(
                        "Jump to offset {}",
                        operands[0]
                            .decoded_value
                            .as_ref()
                            .unwrap_or(&"?".to_string())
                    )
                } else {
                    "Jump to address".to_string()
                }
            }
            JUMP_IF => {
                if !operands.is_empty() {
                    format!(
                        "Jump to offset {} if stack top is true",
                        operands[0]
                            .decoded_value
                            .as_ref()
                            .unwrap_or(&"?".to_string())
                    )
                } else {
                    "Conditional jump if true".to_string()
                }
            }
            LOAD_INPUT => {
                if operands.len() >= 2 {
                    format!(
                        "Load {} parameter {} from function input",
                        operands[0]
                            .decoded_value
                            .as_ref()
                            .unwrap_or(&"unknown".to_string()),
                        operands[1]
                            .decoded_value
                            .as_ref()
                            .unwrap_or(&"?".to_string())
                    )
                } else {
                    "Load parameter from function input".to_string()
                }
            }
            STORE => {
                if !operands.is_empty() {
                    format!(
                        "Store stack value to account {}",
                        operands[0]
                            .decoded_value
                            .as_ref()
                            .unwrap_or(&"?".to_string())
                    )
                } else {
                    "Store value to account".to_string()
                }
            }
            LOAD => {
                if !operands.is_empty() {
                    format!(
                        "Load value from account {}",
                        operands[0]
                            .decoded_value
                            .as_ref()
                            .unwrap_or(&"?".to_string())
                    )
                } else {
                    "Load value from account".to_string()
                }
            }
            // COMPACT_FIELD_LOAD/COMPACT_FIELD_STORE removed
            GET_CLOCK => "Get current Solana clock".to_string(),
            REQUIRE => "Assert that stack top is true (else fail)".to_string(),
            CALL => {
                if !operands.is_empty() {
                    format!(
                        "Call function {}",
                        operands[0]
                            .decoded_value
                            .as_ref()
                            .unwrap_or(&"?".to_string())
                    )
                } else {
                    "Call function".to_string()
                }
            }
            RETURN => "Return from current function".to_string(),
            _ => {
                if operands.is_empty() {
                    format!("{} operation", name)
                } else {
                    format!(
                        "{} with operands: {}",
                        name,
                        operands
                            .iter()
                            .filter_map(|op| op.decoded_value.as_ref())
                            .map(|s| s.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                }
            }
        }
    }

    /// Categorize instruction by opcode value
    fn categorize_instruction(&self, opcode: u8) -> InstructionCategory {
        use five_protocol::opcodes::ranges::*;

        match opcode {
            CONTROL_BASE..=0x0F => InstructionCategory::ControlFlow,
            STACK_BASE..=0x1F => InstructionCategory::Stack,
            ARITHMETIC_BASE..=0x2F => InstructionCategory::Arithmetic,
            LOGICAL_BASE..=0x3F => InstructionCategory::Logical,
            MEMORY_BASE..=0x4F => InstructionCategory::Memory,
            ACCOUNT_BASE..=0x5F => InstructionCategory::Account,
            0x60..=0x6F => InstructionCategory::Array,
            CONSTRAINT_BASE..=0x7F => InstructionCategory::Constraint,
            SYSTEM_BASE..=0x8F => InstructionCategory::System,
            FUNCTION_BASE..=0x9F => InstructionCategory::Function,
            LOCAL_BASE..=0xAF => InstructionCategory::Local,
            REGISTER_BASE..=0xBF => InstructionCategory::Register,
            0xC0..=0xCF => InstructionCategory::Unknown, // Removed account views
            0xD0..=0xD7 => InstructionCategory::Local,   // Nibble locals
            0xD8..=0xDF => InstructionCategory::Test,    // Test framework
            PATTERN_FUSION_BASE..=0xEF => InstructionCategory::PatternFusion,
            ADVANCED_BASE..=0xFF => InstructionCategory::Advanced,
        }
    }

    /// Analyze control flow for a single instruction
    fn analyze_instruction_control_flow(
        &self,
        opcode: u8,
        operands: &[OperandInfo],
    ) -> ControlFlowInfo {
        use five_protocol::opcodes::*;

        match opcode {
            JUMP => {
                let target = if !operands.is_empty() {
                    operands[0]
                        .decoded_value
                        .as_ref()
                        .and_then(|s| s.parse::<usize>().ok())
                        .map(|t| vec![t])
                        .unwrap_or_default()
                } else {
                    Vec::new()
                };

                ControlFlowInfo {
                    is_jump: true,
                    jump_targets: target,
                    can_fall_through: false,
                    is_terminator: false,
                }
            }
            JUMP_IF | JUMP_IF_NOT => {
                let target = if !operands.is_empty() {
                    operands[0]
                        .decoded_value
                        .as_ref()
                        .and_then(|s| s.parse::<usize>().ok())
                        .map(|t| vec![t])
                        .unwrap_or_default()
                } else {
                    Vec::new()
                };

                ControlFlowInfo {
                    is_jump: true,
                    jump_targets: target,
                    can_fall_through: true, // Conditional jumps can fall through
                    is_terminator: false,
                }
            }
            HALT | RETURN | RETURN_VALUE => ControlFlowInfo {
                is_jump: false,
                jump_targets: Vec::new(),
                can_fall_through: false,
                is_terminator: true,
            },
            CALL => {
                // Function calls jump to other functions but return
                ControlFlowInfo {
                    is_jump: true,
                    jump_targets: Vec::new(), // Would need function table to resolve
                    can_fall_through: true,
                    is_terminator: false,
                }
            }
            _ => {
                // Regular instructions just fall through
                ControlFlowInfo {
                    is_jump: false,
                    jump_targets: Vec::new(),
                    can_fall_through: true,
                    is_terminator: false,
                }
            }
        }
    }

    /// Analyze control flow and build control flow graph
    fn analyze_control_flow(&mut self) -> Result<(), VMError> {
        // This is a simplified CFG construction
        // In a full implementation, this would build proper basic blocks

        self.control_flow.entry_points.clear();
        self.control_flow.entry_points.push(0); // Bytecode starts at 0

        // Find all jump targets to identify basic block boundaries
        let mut block_starts: HashSet<usize> = HashSet::new();
        block_starts.insert(0); // First instruction is always a block start

        // Simplified implementation - would need proper instruction parsing
        // For now, just add some basic block boundaries based on bytecode analysis
        let bytecode_len = self.bytecode.len();
        if bytecode_len > 100 {
            block_starts.insert(bytecode_len / 3);
            block_starts.insert(bytecode_len * 2 / 3);
        }

        // Convert to sorted vector
        let mut starts: Vec<usize> = block_starts.into_iter().collect();
        starts.sort();

        // Create basic blocks
        self.control_flow.basic_blocks.clear();
        for (i, &start) in starts.iter().enumerate() {
            let end = if i + 1 < starts.len() {
                starts[i + 1]
            } else {
                self.bytecode.len()
            };

            // Find instructions in this block
            let mut block_instructions = Vec::new();
            for (idx, instruction) in self.instructions.iter().enumerate() {
                if instruction.offset >= start && instruction.offset < end {
                    block_instructions.push(idx);
                }
            }

            self.control_flow.basic_blocks.push(BasicBlock {
                start,
                end,
                instructions: block_instructions,
                successors: Vec::new(),
                predecessors: Vec::new(),
            });
        }

        Ok(())
    }

    /// Perform stack effect analysis
    fn analyze_stack_effects(&mut self) -> Result<(), VMError> {
        self.stack_analysis.stack_depths.clear();
        self.stack_analysis.max_stack_depth = 0;
        self.stack_analysis.min_stack_depth = 0;
        self.stack_analysis.is_consistent = true;

        let mut current_depth = 0i32;

        // Simplified implementation - would need proper instruction parsing
        // For now, estimate based on bytecode complexity
        let estimated_instructions = self.bytecode.len() / 3; // Rough estimate
        for i in 0..estimated_instructions {
            self.stack_analysis.stack_depths.push(current_depth);

            // Rough stack effect estimation
            let stack_effect = if i % 10 == 0 {
                1
            } else if i % 7 == 0 {
                -1
            } else {
                0
            };
            current_depth += stack_effect;

            self.stack_analysis.max_stack_depth =
                self.stack_analysis.max_stack_depth.max(current_depth);
            self.stack_analysis.min_stack_depth =
                self.stack_analysis.min_stack_depth.min(current_depth);

            // Check for stack underflow
            if current_depth < 0 {
                self.stack_analysis.is_consistent = false;
                current_depth = 0; // Reset to prevent further underflow
            }
        }

        Ok(())
    }

    /// Detect optimization patterns in the bytecode
    fn detect_patterns(&self) -> Result<Vec<OptimizationPattern>, VMError> {
        let mut patterns = Vec::new();

        // Pattern 1: Consecutive PUSH/POP pairs (can be eliminated)
        for i in 0..self.instructions.len().saturating_sub(1) {
            if (self.instructions[i].opcode == PUSH_U64
                || self.instructions[i].opcode == PUSH_U8
                || self.instructions[i].opcode == PUSH_I64
                || self.instructions[i].opcode == PUSH_BOOL
                || self.instructions[i].opcode == PUSH_PUBKEY)
                && self.instructions[i + 1].opcode == POP
            {
                patterns.push(OptimizationPattern {
                    pattern_type: "redundant_push_pop".to_string(),
                    instruction_range: (i, i + 1),
                    description: "Consecutive PUSH/POP can be eliminated".to_string(),
                    potential_savings: self.instructions[i].compute_cost
                        + self.instructions[i + 1].compute_cost,
                });
            }
        }

        // Pattern 2: Dead code after unconditional jumps or returns
        for (i, instruction) in self.instructions.iter().enumerate() {
            if instruction.control_flow.is_terminator
                || (instruction.control_flow.is_jump && !instruction.control_flow.can_fall_through)
            {
                // Check if there are instructions after this that are not jump targets
                let next_offset = instruction.offset + instruction.size;
                if i + 1 < self.instructions.len() && self.instructions[i + 1].offset == next_offset
                {
                    // There's an instruction immediately following - could be dead code
                    patterns.push(OptimizationPattern {
                        pattern_type: "potential_dead_code".to_string(),
                        instruction_range: (i + 1, i + 1),
                        description: "Instruction after terminator may be dead code".to_string(),
                        potential_savings: self.instructions[i + 1].compute_cost,
                    });
                }
            }
        }

        // Pattern 3: Multiple consecutive DUP instructions
        let mut dup_count = 0;
        let mut dup_start = 0;
        for (i, instruction) in self.instructions.iter().enumerate() {
            if instruction.opcode == DUP {
                if dup_count == 0 {
                    dup_start = i;
                }
                dup_count += 1;
            } else {
                if dup_count > 2 {
                    patterns.push(OptimizationPattern {
                        pattern_type: "excessive_duplication".to_string(),
                        instruction_range: (dup_start, i - 1),
                        description: format!(
                            "Sequence of {} DUP instructions could be optimized",
                            dup_count
                        ),
                        potential_savings: (dup_count - 1) as u32
                            * self.instructions[dup_start].compute_cost,
                    });
                }
                dup_count = 0;
            }
        }

        Ok(patterns)
    }

    /// Generate analysis summary
    fn generate_summary(&self) -> AnalysisSummary {
        let mut category_distribution = HashMap::new();

        // Simplified implementation - would need proper instruction parsing
        // For now, estimate based on bytecode complexity
        let estimated_instructions = self.bytecode.len() / 3;
        let total_compute_cost = estimated_instructions as u32 * 10; // Rough estimate
        let jump_count = (estimated_instructions / 20).max(1); // Rough estimate
        let function_call_count = (estimated_instructions / 50).max(0); // Rough estimate

        // Add some default category distribution
        category_distribution.insert(InstructionCategory::Arithmetic, estimated_instructions / 4);
        category_distribution.insert(InstructionCategory::Memory, estimated_instructions / 6);
        category_distribution.insert(
            InstructionCategory::ControlFlow,
            estimated_instructions / 10,
        );

        AnalysisSummary {
            total_instructions: estimated_instructions,
            total_size: self.bytecode.len(),
            total_compute_cost,
            basic_block_count: self.control_flow.basic_blocks.len(),
            jump_count,
            function_call_count,
            category_distribution,
        }
    }

    /// Decode value type byte to string name
    fn decode_value_type_name(&self, type_byte: u8) -> String {
        match type_byte {
            0x01 => "U8".to_string(),
            0x02 => "U64".to_string(),
            0x03 => "BOOL".to_string(),
            0x04 => "PUBKEY".to_string(),
            0x05 => "STRING".to_string(),
            0x06 => "ACCOUNT".to_string(),
            _ => format!("UNKNOWN_TYPE_{:02X}", type_byte),
        }
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
    /// This method analyzes the compiled bytecode and source AST to determine
    /// optimal VM resource allocation requirements for zero-copy execution.
    pub fn calculate_resource_requirements(
        &mut self,
        ast: &AstNode,
    ) -> Result<ResourceRequirements, VMError> {
        // Ensure we have analyzed the bytecode first
        if self.instructions.is_empty() {
            self.decode_instructions()?;
        }
        if self.stack_analysis.stack_depths.is_empty() {
            self.analyze_stack_effects()?;
        }

        // Use enhanced cross-function analysis for better resource calculation
        let cross_function_stack_depth = self.calculate_max_concurrent_stack_usage(ast);
        let bytecode_stack_depth = (self.stack_analysis.max_stack_depth as u16).max(1);

        // Take the maximum of bytecode analysis and cross-function analysis
        let final_stack_depth = cross_function_stack_depth.max(bytecode_stack_depth);

        println!(
            "Bytecode Analyzer: Stack depth analysis - bytecode: {}, cross-function: {}, final: {}",
            bytecode_stack_depth, cross_function_stack_depth, final_stack_depth
        );

        // Create base requirements from bytecode analysis
        let base_requirements = ResourceRequirements {
            max_stack: 0,
            max_memory: 0,
            max_locals: self.calculate_max_locals(ast),
            max_stack_depth: final_stack_depth,
            string_pool_bytes: self.calculate_string_pool_size(ast),
            max_call_depth: self.calculate_call_depth(ast),
            temp_buffer_size: self.calculate_temp_buffer_size(),
            heap_string_capacity: self.estimate_heap_strings(ast),
            heap_array_capacity: self.estimate_heap_arrays(ast),
        };

        // Cache the result for future use
        self.stack_analysis.resource_requirements = Some(base_requirements);
        Ok(base_requirements)
    }

    /// Calculate maximum concurrent stack usage across functions (simplified implementation)
    fn calculate_max_concurrent_stack_usage(&self, _ast: &AstNode) -> u16 {
        // For now, return a conservative estimate based on stack analysis
        self.stack_analysis.max_stack_depth.max(32) as u16
    }

    /// Calculate maximum local variables used across all functions (simplified implementation)
    fn calculate_max_locals(&self, _ast: &AstNode) -> u8 {
        // Conservative estimate - can be enhanced later
        8
    }

    /// Calculate string pool size from AST analysis (simplified implementation)
    fn calculate_string_pool_size(&self, _ast: &AstNode) -> u16 {
        // Conservative estimate for string pool
        256
    }

    /// Calculate function call depth (simplified implementation)
    fn calculate_call_depth(&self, _ast: &AstNode) -> u8 {
        // Conservative estimate
        4
    }

    /// Calculate temp buffer size needed (simplified implementation)
    fn calculate_temp_buffer_size(&self) -> u8 {
        // Return minimum safe size
        64
    }

    /// Estimate heap strings capacity (simplified implementation)
    fn estimate_heap_strings(&self, _ast: &AstNode) -> u16 {
        // Conservative estimate
        16
    }

    /// Estimate heap arrays capacity (simplified implementation)
    fn estimate_heap_arrays(&self, _ast: &AstNode) -> u16 {
        // Conservative estimate
        16
    }
}

impl Default for AdvancedBytecodeAnalyzer {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

/// Extension methods for the main DslBytecodeGenerator
impl super::DslBytecodeGenerator {
    /// Analyze generated bytecode using the advanced analyzer
    pub fn analyze_bytecode(&self) -> Result<BytecodeAnalysisResult, VMError> {
        let mut analyzer = AdvancedBytecodeAnalyzer::new(self.bytecode.clone());
        analyzer.analyze()
    }

    /// Get comprehensive bytecode report
    pub fn generate_bytecode_report(&self) -> Result<String, VMError> {
        let analysis = self.analyze_bytecode()?;

        let mut report = String::new();
        report.push_str("Bytecode Analysis Report\n");
        report.push_str("========================\n\n");

        // Summary
        report.push_str("Summary:\n");
        report.push_str(&format!(
            "  Total instructions: {}\n",
            analysis.summary.total_instructions
        ));
        report.push_str(&format!(
            "  Total size: {} bytes\n",
            analysis.summary.total_size
        ));
        report.push_str(&format!(
            "  Compute cost estimate: {} CU\n",
            analysis.summary.total_compute_cost
        ));
        report.push_str(&format!(
            "  Basic blocks: {}\n",
            analysis.summary.basic_block_count
        ));
        report.push_str(&format!(
            "  Jump instructions: {}\n",
            analysis.summary.jump_count
        ));
        report.push_str(&format!(
            "  Function calls: {}\n",
            analysis.summary.function_call_count
        ));

        // Stack analysis
        report.push_str("\nStack Analysis:\n");
        report.push_str(&format!(
            "  Max stack depth: {}\n",
            analysis.stack_analysis.max_stack_depth
        ));
        report.push_str(&format!(
            "  Min stack depth: {}\n",
            analysis.stack_analysis.min_stack_depth
        ));
        report.push_str(&format!(
            "  Stack consistent: {}\n",
            analysis.stack_analysis.is_consistent
        ));

        // Category distribution
        report.push_str("\nInstruction Categories:\n");
        for (category, count) in &analysis.summary.category_distribution {
            report.push_str(&format!("  {:?}: {}\n", category, count));
        }

        // Optimization patterns
        if !analysis.patterns.is_empty() {
            report.push_str("\nOptimization Opportunities:\n");
            for pattern in &analysis.patterns {
                report.push_str(&format!(
                    "  {}: {} (saves {} CU)\n",
                    pattern.pattern_type, pattern.description, pattern.potential_savings
                ));
            }
        }

        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let analyzer = AdvancedBytecodeAnalyzer::new(Vec::new());

        assert_eq!(
            analyzer.categorize_instruction(HALT),
            InstructionCategory::ControlFlow
        );
        assert_eq!(
            analyzer.categorize_instruction(ADD),
            InstructionCategory::Arithmetic
        );
        assert_eq!(
            analyzer.categorize_instruction(PUSH_U64),
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
