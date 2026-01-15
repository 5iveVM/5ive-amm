//! Array and block statement generation
//!
//! This module handles generation of arrays, byte arrays, and statement blocks.

use super::super::OpcodeEmitter;
use super::types::ASTGenerator;
use crate::ast::AstNode;
use five_protocol::opcodes::*;
use five_vm_mito::error::VMError;

impl ASTGenerator {
    /// Generate bytecode for a byte array
    pub(super) fn generate_byte_array<T: OpcodeEmitter>(
        &mut self,
        emitter: &mut T,
        node: &AstNode,
    ) -> Result<(), VMError> {
        if let AstNode::ArrayLiteral { elements } = node {
            emitter.emit_opcode(PUSH_U8);
            emitter.emit_u8(elements.len() as u8);
            for element in elements {
                self.generate_ast_node(emitter, element)?;
            }
        } else {
            return Err(VMError::TypeMismatch);
        }
        Ok(())
    }

    /// Generate bytecode for an array of any type
    pub(super) fn generate_array<T: OpcodeEmitter>(
        &mut self,
        emitter: &mut T,
        node: &AstNode,
    ) -> Result<(), VMError> {
        if let AstNode::ArrayLiteral { elements } = node {
            emitter.emit_opcode(PUSH_U8);
            emitter.emit_u8(elements.len() as u8);
            for element in elements {
                self.generate_ast_node(emitter, element)?;
            }
        } else {
            return Err(VMError::TypeMismatch);
        }
        Ok(())
    }

    /// Generate statement block
    pub(super) fn generate_statement_block<T: OpcodeEmitter>(
        &mut self,
        emitter: &mut T,
        statements: &[AstNode],
    ) -> Result<(), VMError> {
        for statement in statements {
            self.generate_ast_node(emitter, statement)?;
        }
        Ok(())
    }
}
