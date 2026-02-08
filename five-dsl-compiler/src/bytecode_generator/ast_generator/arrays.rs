//! Array and block statement generation.

use super::super::OpcodeEmitter;
use super::types::ASTGenerator;
use crate::ast::AstNode;
use five_vm_mito::error::VMError;

impl ASTGenerator {
    /// Generate bytecode for a byte array
    pub(super) fn generate_byte_array<T: OpcodeEmitter>(
        &mut self,
        emitter: &mut T,
        node: &AstNode,
    ) -> Result<(), VMError> {
        if let AstNode::ArrayLiteral { elements } = node {
            emitter.emit_const_u8(elements.len() as u8)?;
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
            emitter.emit_const_u8(elements.len() as u8)?;
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
        let mut i = 0;
        while i < statements.len() {
            // Tier 3 Optimization: Check for multi-statement fusion patterns
            // Example: double-entry bookkeeping (sub/add pairs) -> FIELD_SUB_ADD_PARAM
            if let Some(consumed) = self.try_emit_fused_assignment_block(emitter, statements, i)? {
                #[cfg(debug_assertions)]
                println!("FUSED_DEBUG: Consumed {} statements for fused block pattern", consumed);
                i += consumed;
                continue;
            }

            // Fallback: Generate single statement
            self.generate_ast_node(emitter, &statements[i])?;
            i += 1;
        }
        Ok(())
    }
}
