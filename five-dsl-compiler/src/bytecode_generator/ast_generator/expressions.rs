//! Expression generation methods.

use super::super::opcodes::OpcodePatterns;
use super::super::OpcodeEmitter;
use super::types::ASTGenerator;
use crate::ast::AstNode;
use five_protocol::{opcodes::*, Value};
use five_vm_mito::error::VMError;
use heapless::String;

impl ASTGenerator {
    #[inline]
    fn literal_pow2_shift(node: &AstNode) -> Option<u8> {
        let value = match node {
            AstNode::Literal(Value::U8(v)) => *v as u64,
            AstNode::Literal(Value::U64(v)) => *v,
            _ => return None,
        };
        if value != 0 && value.is_power_of_two() {
            Some(value.trailing_zeros() as u8)
        } else {
            None
        }
    }

    /// Generate binary expression bytecode
    pub(super) fn generate_binary_expression<T: OpcodeEmitter>(
        &mut self,
        emitter: &mut T,
        left: &AstNode,
        right: &AstNode,
        operator: &str,
    ) -> Result<(), VMError> {
        // Preserve language-level short-circuit semantics for logical operators.
        if operator == "&&" || operator == "||" {
            let short_label = self.new_label();
            let end_label = self.new_label();

            self.generate_ast_node(emitter, left)?;
            emitter.emit_opcode(DUP);

            if operator == "&&" {
                self.emit_jump(emitter, JUMP_IF_NOT, short_label.clone());
            } else {
                self.emit_jump(emitter, JUMP_IF, short_label.clone());
            }

            // Left operand does not determine the result; discard it and evaluate RHS.
            emitter.emit_opcode(POP);
            self.generate_ast_node(emitter, right)?;
            self.emit_jump(emitter, JUMP, end_label.clone());

            // Left operand already on stack is the result in short-circuit path.
            self.place_label(emitter, short_label);
            self.place_label(emitter, end_label);
            return Ok(());
        }

        // Try optimized binary expression first
        if self.try_optimized_binary_expression(emitter, left, right, operator)? {
            return Ok(());
        }

        // Fall back to stack-based evaluation
        self.generate_ast_node(emitter, left)?;
        self.generate_ast_node(emitter, right)?;

        // Type inference for potential future optimizations
        let left_type = self
            .infer_type_from_node(left)
            .unwrap_or("unknown".to_string());
        let _is_u128 = left_type == "u128"; // Available for future use

        // Emit appropriate operator opcode - polymorphic arithmetic handles all types
        // Support both standard and checked arithmetic operator syntax:
        //   +   -> ADD
        //   -   -> SUB
        //   *   -> MUL
        //   +?  -> ADD_CHECKED (errors on overflow)
        //   -?  -> SUB_CHECKED (errors on underflow)
        //   *?  -> MUL_CHECKED (errors on overflow)
        match operator {
            "+" => emitter.emit_opcode(ADD),
            "+?" => emitter.emit_opcode(ADD_CHECKED),
            "-" => emitter.emit_opcode(SUB),
            "-?" => emitter.emit_opcode(SUB_CHECKED),
            "*" => emitter.emit_opcode(MUL),
            "*?" => emitter.emit_opcode(MUL_CHECKED),
            "/" => emitter.emit_opcode(DIV),
            "%" => emitter.emit_opcode(MOD),
            "==" => emitter.emit_opcode(EQ),
            "!=" => emitter.emit_opcode(NEQ),
            "<" => emitter.emit_opcode(LT),
            "<=" => emitter.emit_opcode(LTE),
            ">" => emitter.emit_opcode(GT),
            ">=" => emitter.emit_opcode(GTE),
            "&&" => emitter.emit_opcode(AND),
            "||" => emitter.emit_opcode(OR),
            // Bitwise operators (use correct BITWISE_* opcodes)
            "&" => emitter.emit_opcode(BITWISE_AND),
            "|" => emitter.emit_opcode(BITWISE_OR),
            "^" => emitter.emit_opcode(BITWISE_XOR),
            // Shift operators
            "<<" => emitter.emit_opcode(SHIFT_LEFT),
            ">>" => emitter.emit_opcode(SHIFT_RIGHT),
            ">>>" => emitter.emit_opcode(SHIFT_RIGHT_ARITH),
            "<<<" => emitter.emit_opcode(ROTATE_LEFT),
            _ => return Err(VMError::InvalidScript),
        }

        Ok(())
    }

    /// Try to generate optimized binary expression
    pub(super) fn try_optimized_binary_expression<T: OpcodeEmitter>(
        &mut self,
        emitter: &mut T,
        left: &AstNode,
        right: &AstNode,
        operator: &str,
    ) -> Result<bool, VMError> {
        // Strength reduction for unsigned hot paths:
        //   u64 * (2^k) => u64 << k
        //   u64 / (2^k) => u64 >> k
        // Keep this strictly typed to avoid changing i64 semantics.
        let left_type = self
            .infer_type_from_node(left)
            .unwrap_or_else(|_| "unknown".to_string());
        if left_type == "u64" {
            match operator {
                "*" => {
                    if let Some(shift) = Self::literal_pow2_shift(right) {
                        self.generate_ast_node(emitter, left)?;
                        emitter.emit_const_u8(shift)?;
                        emitter.emit_opcode(SHIFT_LEFT);
                        return Ok(true);
                    }
                    if let Some(shift) = Self::literal_pow2_shift(left) {
                        self.generate_ast_node(emitter, right)?;
                        emitter.emit_const_u8(shift)?;
                        emitter.emit_opcode(SHIFT_LEFT);
                        return Ok(true);
                    }
                }
                "/" => {
                    if let Some(shift) = Self::literal_pow2_shift(right) {
                        self.generate_ast_node(emitter, left)?;
                        emitter.emit_const_u8(shift)?;
                        emitter.emit_opcode(SHIFT_RIGHT);
                        return Ok(true);
                    }
                }
                _ => {}
            }
        }

        // Check if both operands are simple (literals or identifiers)
        if self.is_simple_expression(left) && self.is_simple_expression(right) {
            // For simple expressions, try constant folding optimization
            match operator {
                "-" => {
                    // Optimization: DUP_SUB for x - x
                    if let (AstNode::Identifier(left_name), AstNode::Identifier(right_name)) =
                        (left, right)
                    {
                        if left_name == right_name {
                            self.generate_ast_node(emitter, left)?;
                            emitter.emit_opcode(DUP_SUB);
                            return Ok(true);
                        }
                    }
                    try_constant_fold!(self, emitter, left, right, wrapping_sub)
                }
                "*" => {
                    // Optimization: DUP_MUL for x * x
                    if let (AstNode::Identifier(left_name), AstNode::Identifier(right_name)) =
                        (left, right)
                    {
                        if left_name == right_name {
                            self.generate_ast_node(emitter, left)?;
                            emitter.emit_opcode(DUP_MUL);
                            return Ok(true);
                        }
                    }
                    try_constant_fold!(self, emitter, left, right, wrapping_mul)
                }
                // Optimization: DUP_ADD for x + x
                "+" => {
                    // Check if both operands are the same identifier (e.g. x + x)
                    if let (AstNode::Identifier(left_name), AstNode::Identifier(right_name)) =
                        (left, right)
                    {
                        if left_name == right_name {
                            // Emit regular load for the first operand
                            self.generate_ast_node(emitter, left)?;
                            // Emit DUP_ADD to double it (replacing second load + ADD)
                            emitter.emit_opcode(DUP_ADD);
                            return Ok(true);
                        }
                    }
                    // Standard constant folding for addition
                    try_constant_fold!(self, emitter, left, right, wrapping_add);
                }
                _ => {} // Other operators use standard evaluation
            }
        }

        // No temporary allocation for binary expressions.
        Ok(false)
    }

    /// Generate unary expression
    pub(super) fn generate_unary_expression<T: OpcodeEmitter>(
        &mut self,
        emitter: &mut T,
        operand: &AstNode,
        operator: &str,
    ) -> Result<(), VMError> {
        self.generate_ast_node(emitter, operand)?;

        match operator {
            "-" => emitter.emit_opcode(NEG),
            "!" => emitter.emit_opcode(NOT),
            "not" => emitter.emit_opcode(NOT), // Handle parser's "not" operator
            "~" => emitter.emit_opcode(BITWISE_NOT),
            _ => return Err(VMError::InvalidScript),
        }

        Ok(())
    }

    /// Emit literal value with appropriate opcode
    pub(super) fn emit_literal_value<T: OpcodeEmitter>(
        &self,
        emitter: &mut T,
        value: &Value,
    ) -> Result<(), VMError> {
        match value {
            Value::U64(n) => {
                // Optimization: Use nibble constants (0-3) if available
                match n {
                    0 => {
                        emitter.emit_opcode(PUSH_0);
                        return Ok(());
                    }
                    1 => {
                        emitter.emit_opcode(PUSH_1);
                        return Ok(());
                    }
                    2 => {
                        emitter.emit_opcode(PUSH_2);
                        return Ok(());
                    }
                    3 => {
                        emitter.emit_opcode(PUSH_3);
                        return Ok(());
                    }
                    _ => {}
                }

                if *n <= u8::MAX as u64 {
                    emitter.emit_const_u8(*n as u8)?;
                } else if *n <= u16::MAX as u64 {
                    emitter.emit_const_u16(*n as u16)?;
                } else if *n <= u32::MAX as u64 {
                    emitter.emit_const_u32(*n as u32)?;
                } else {
                    emitter.emit_const_u64(*n)?;
                }
            }
            Value::U128(n) => {
                // Native U128 literal - MITO-style BPF-optimized
                OpcodePatterns::emit_push_u128(emitter, *n)?;
            }
            Value::Bool(b) => {
                emitter.emit_const_bool(*b)?;
            }
            Value::String(_idx) => {
                // String table approach eliminated in favor of unified Array<u8> representation
                return Err(VMError::ParseError {
                    expected: String::<32>::try_from("string literal").unwrap(),
                    found: String::<32>::try_from("string table reference").unwrap(),
                    position: 0,
                });
            }
            Value::Pubkey(key) => {
                emitter.emit_const_pubkey(key)?;
            }
            Value::U8(n) => {
                emitter.emit_const_u8(*n)?;
            }
            Value::I64(n) => {
                emitter.emit_const_i64(*n)?;
            }
            Value::Account(idx) => {
                OpcodePatterns::emit_push_account(emitter, *idx)?;
            }
            Value::Empty => {
                // Empty value - emit NOP or default value
                emitter.emit_opcode(NOP);
            }
            Value::Array(_) => {
                // Array values - not implemented yet
                return Err(VMError::InvalidScript);
            }
        }
        Ok(())
    }

    /// Check if an expression is simple (literal or identifier)
    pub(super) fn is_simple_expression(&self, expr: &AstNode) -> bool {
        matches!(expr, AstNode::Literal(_) | AstNode::Identifier(_))
    }
}
