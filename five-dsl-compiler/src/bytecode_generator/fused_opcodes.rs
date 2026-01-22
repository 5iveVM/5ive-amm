// Fused Opcode Optimization Module
//
// This module provides pattern matching and emission of fused opcodes
// to reduce CU consumption by combining common multi-opcode patterns
// into single opcodes.

use crate::ast::{AstNode, TypeNode};
use crate::bytecode_generator::OpcodeEmitter;
use crate::bytecode_generator::ast_generator::types::ASTGenerator;
use crate::FieldInfo;
use five_protocol::opcodes::*;
use five_vm_mito::error::VMError;

/// Pattern matcher for fused opcode optimization
pub struct FusedOpcodeOptimizer<'a> {
    ast_generator: &'a mut ASTGenerator,
}

impl<'a> FusedOpcodeOptimizer<'a> {
    pub fn new(ast_generator: &'a mut ASTGenerator) -> Self {
        Self { ast_generator }
    }

    /// Try to emit a fused opcode for a require statement condition.
    /// Returns Ok(true) if a fused opcode was emitted, Ok(false) if not.
    pub fn try_emit_fused_require<T: OpcodeEmitter>(
        &mut self,
        emitter: &mut T,
        condition: &AstNode,
    ) -> Result<bool, VMError> {
        // Pattern 1: field >= param (REQUIRE_GTE_U64)
        if let Some((acc_idx, offset, param_idx)) = self.match_field_gte_param(condition) {
            emitter.emit_opcode(REQUIRE_GTE_U64);
            emitter.emit_u8(acc_idx);
            emitter.emit_vle_u32(offset);
            emitter.emit_u8(param_idx);
            return Ok(true);
        }

        // Pattern 2: !field (REQUIRE_NOT_BOOL) - for frozen/paused checks
        if let Some((acc_idx, offset)) = self.match_not_bool_field(condition) {
            emitter.emit_opcode(REQUIRE_NOT_BOOL);
            emitter.emit_u8(acc_idx);
            emitter.emit_vle_u32(offset);
            return Ok(true);
        }

        // Pattern 3: param > 0 (REQUIRE_PARAM_GT_ZERO)
        if let Some(param_idx) = self.match_param_gt_zero(condition) {
            emitter.emit_opcode(REQUIRE_PARAM_GT_ZERO);
            emitter.emit_u8(param_idx);
            return Ok(true);
        }

        Ok(false)
    }

    /// Match pattern: account.field >= param
    /// Returns (account_index, field_offset, param_index)
    fn match_field_gte_param(&self, condition: &AstNode) -> Option<(u8, u32, u8)> {
        if let AstNode::BinaryExpression { left, operator, right } = condition {
            if operator != ">=" {
                return None;
            }

            // Left side should be a field access
            let (acc_idx, offset) = self.match_u64_field_access(left)?;

            // Right side should be a parameter
            let param_idx = self.match_parameter(right)?;

            return Some((acc_idx, offset, param_idx));
        }
        None
    }

    /// Match pattern: !account.field (for bool fields)
    /// Returns (account_index, field_offset)
    fn match_not_bool_field(&self, condition: &AstNode) -> Option<(u8, u32)> {
        if let AstNode::UnaryExpression { operator, operand } = condition {
            if operator != "!" {
                return None;
            }

            // Operand should be a field access to a bool
            return self.match_bool_field_access(operand);
        }
        None
    }

    /// Match pattern: param > 0
    /// Returns param_index
    fn match_param_gt_zero(&self, condition: &AstNode) -> Option<u8> {
        if let AstNode::BinaryExpression { left, operator, right } = condition {
            if operator != ">" {
                return None;
            }

            // Left should be a parameter
            let param_idx = self.match_parameter(left)?;

            // Right should be literal 0
            if self.is_literal_zero(right) {
                return Some(param_idx);
            }
        }
        None
    }

    /// Match a u64 field access: account.field
    /// Returns (account_bytecode_index, field_offset)
    fn match_u64_field_access(&self, node: &AstNode) -> Option<(u8, u32)> {
        if let AstNode::FieldAccess { object, field } = node {
            if let AstNode::Identifier(account_name) = object.as_ref() {
                // Look up account in symbol table
                if let Some(field_info) = self.ast_generator.local_symbol_table.get(account_name) {
                    let account_type = &field_info.field_type;
                    
                    // Calculate field offset
                    if let Ok(offset) = self.ast_generator.calculate_account_field_offset(account_type, field) {
                        let acc_idx = crate::bytecode_generator::account_utils::account_index_from_param_offset(
                            field_info.offset
                        );
                        return Some((acc_idx, offset));
                    }
                }
            }
        }
        None
    }

    /// Match a bool field access: account.field
    /// Returns (account_bytecode_index, field_offset)
    fn match_bool_field_access(&self, node: &AstNode) -> Option<(u8, u32)> {
        if let AstNode::FieldAccess { object, field } = node {
            if let AstNode::Identifier(account_name) = object.as_ref() {
                // Look up account in symbol table
                if let Some(field_info) = self.ast_generator.local_symbol_table.get(account_name) {
                    let account_type = &field_info.field_type;
                    
                    // Verify field is bool type via account system
                    if let Some(account_system) = &self.ast_generator.account_system {
                        if let Some(type_info) = account_system
                            .get_account_registry()
                            .account_types
                            .get(account_type)
                        {
                            if let Some(struct_field) = type_info.fields.get(field) {
                                if struct_field.field_type != "bool" {
                                    return None;
                                }
                            }
                        }
                    }
                    
                    // Calculate field offset
                    if let Ok(offset) = self.ast_generator.calculate_account_field_offset(account_type, field) {
                        let acc_idx = crate::bytecode_generator::account_utils::account_index_from_param_offset(
                            field_info.offset
                        );
                        return Some((acc_idx, offset));
                    }
                }
            }
        }
        None
    }

    /// Match a parameter identifier
    /// Returns param_index (1-based as used by VM)
    fn match_parameter(&self, node: &AstNode) -> Option<u8> {
        if let AstNode::Identifier(name) = node {
            if let Some(field_info) = self.ast_generator.local_symbol_table.get(name) {
                if field_info.is_parameter {
                    // Return 1-based index as used by LOAD_PARAM
                    return Some((field_info.offset + 1) as u8);
                }
            }
        }
        None
    }

    /// Check if node is literal 0
    fn is_literal_zero(&self, node: &AstNode) -> bool {
        if let AstNode::Literal(value) = node {
            return value.as_u64() == Some(0);
        }
        false
    }
}
