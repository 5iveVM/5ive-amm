//! Expression generation methods
//!
//! This module handles generation of binary expressions, unary expressions,
//! and literal values with optimizations like constant folding.

use super::super::opcodes::OpcodePatterns;
use super::super::OpcodeEmitter;
use super::types::ASTGenerator;
use crate::ast::AstNode;
use five_protocol::{opcodes::*, Value};
use five_vm_mito::error::VMError;
use heapless::String;

impl ASTGenerator {
    /// Generate binary expression bytecode
    pub(super) fn generate_binary_expression<T: OpcodeEmitter>(
        &mut self,
        emitter: &mut T,
        left: &AstNode,
        right: &AstNode,
        operator: &str,
    ) -> Result<(), VMError> {
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
        // Check if both operands are simple (literals or identifiers)
        if self.is_simple_expression(left) && self.is_simple_expression(right) {
            // For simple expressions, try constant folding optimization
            match operator {
                "-" => {
                    // Optimization: DUP_SUB for x - x
                     if let (AstNode::Identifier(left_name), AstNode::Identifier(right_name)) = (left, right) {
                         if left_name == right_name {
                             self.generate_ast_node(emitter, left)?;
                             emitter.emit_opcode(DUP_SUB);
                             return Ok(true);
                         }
                     }
                    try_constant_fold!(self, emitter, left, right, wrapping_sub)
                },
                "*" => {
                    // Optimization: DUP_MUL for x * x
                     if let (AstNode::Identifier(left_name), AstNode::Identifier(right_name)) = (left, right) {
                         if left_name == right_name {
                             self.generate_ast_node(emitter, left)?;
                             emitter.emit_opcode(DUP_MUL);
                             return Ok(true);
                         }
                     }
                    try_constant_fold!(self, emitter, left, right, wrapping_mul)
                },
                 // Optimization: DUP_ADD for x + x
                 "+" => {
                     // Check if both operands are the same identifier (e.g. x + x)
                     if let (AstNode::Identifier(left_name), AstNode::Identifier(right_name)) = (left, right) {
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

        // No temporary register allocation - static registers are for variables, not expressions
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
                   // Fits in u8 - specific optimization
                   emitter.emit_opcode(PUSH_U8);
                   emitter.emit_u8(*n as u8);
                } else if *n <= u16::MAX as u64 {
                    // Fits in u16
                    emitter.emit_opcode(PUSH_U16);
                    emitter.emit_vle_u16(*n as u16);
                } else if *n <= u32::MAX as u64 {
                    // Fits in u32
                    emitter.emit_opcode(PUSH_U32);
                    emitter.emit_vle_u32(*n as u32);
                } else {
                    // Requires u64
                    emitter.emit_opcode(PUSH_U64);
                    emitter.emit_vle_u64(*n);
                }
            }
            Value::U128(n) => {
                // Native U128 literal - MITO-style BPF-optimized
                OpcodePatterns::emit_push_u128(emitter, *n);
            }
            Value::Bool(b) => {
                emitter.emit_opcode(PUSH_BOOL);
                emitter.emit_u8(if *b { 1 } else { 0 });
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
                emitter.emit_opcode(PUSH_PUBKEY);
                emitter.emit_bytes(key);
            }
            Value::U8(n) => {
                emitter.emit_opcode(PUSH_U8);
                emitter.emit_u8(*n);
            }
            Value::I64(n) => {
                emitter.emit_opcode(PUSH_I64);
                emitter.emit_vle_u64(*n as u64);
            }
            Value::Account(idx) => {
                OpcodePatterns::emit_push_account(emitter, *idx);
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
