//! Utility functions and extension methods.

use super::super::OpcodeEmitter;
use super::super::{
    scope_analyzer, types, AccountSystem, DslBytecodeGenerator, FunctionDispatcher,
};
use super::types::ASTGenerator;
use crate::ast::{AssertionType, AstNode, TestAttribute};
use five_vm_mito::error::VMError;
use std::collections::HashMap;

impl ASTGenerator {


    /// Parse program ID string to 32-byte array
    pub(super) fn parse_program_id(&self, program_id: &str) -> Result<[u8; 32], VMError> {
        let decoded = bs58::decode(program_id)
            .into_vec()
            .map_err(|_| VMError::InvalidOperation)?;
        if decoded.len() != 32 {
            return Err(VMError::InvalidOperation);
        }
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&decoded);
        Ok(bytes)
    }

    /// Generate test function bytecode
    pub(super) fn generate_test_function<T: OpcodeEmitter>(
        &mut self,
        emitter: &mut T,
        _name: &str,
        _attributes: &[TestAttribute],
        body: &AstNode,
    ) -> Result<(), VMError> {
        // Import test patterns from test_framework module

        // Generate function body
        self.generate_ast_node(emitter, body)?;

        Ok(())
    }

    /// Generate assertion statement bytecode
    pub(super) fn generate_assertion_statement<T: OpcodeEmitter>(
        &mut self,
        emitter: &mut T,
        _assertion_type: &AssertionType,
        args: &[AstNode],
    ) -> Result<(), VMError> {
        // Import test patterns from test_framework module

        // Generate arguments - assertions consume their arguments from stack
        for arg in args {
            self.generate_ast_node(emitter, arg)?;
        }

        // Generate the assertion as a require

        Ok(())
    }
}

impl Default for ASTGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Extension methods for the main DslBytecodeGenerator
impl DslBytecodeGenerator {
    /// Generate AST using the dedicated AST generator
    pub fn generate_with_ast_generator(&mut self, ast: &AstNode) -> Result<(), VMError> {
        let mut ast_generator = ASTGenerator::new();
        ast_generator.generate_ast_node(self, ast)?;

        // Copy symbol table and field counter back to main generator
        // This maintains state for other modules that need it
        Ok(())
    }

    /// Generate AST with function dispatcher coordination
    pub fn generate_with_ast_and_function_coordination(
        &mut self,
        ast: &AstNode,
        symbol_table: &mut HashMap<String, types::FieldInfo>,
    ) -> Result<ASTGenerator, VMError> {
        let mut ast_generator = ASTGenerator::new();
        ast_generator.global_symbol_table = symbol_table.clone();
        let mut function_dispatcher = FunctionDispatcher::new();
        let mut account_system = AccountSystem::new();
        let mut scope_analyzer = scope_analyzer::ScopeAnalyzer::new();

        // Initialize AccountSystem with account definitions from AST
        account_system.process_account_definitions(ast)?;

        // Set the account system in the AST generator for proper field offset resolution
        ast_generator.set_account_system(account_system);

        // First, collect function information from the AST
        if function_dispatcher.has_callable_functions(ast) {
            // Create a new account system for the function dispatcher since we moved the original
            let mut dispatcher_account_system = AccountSystem::new();
            dispatcher_account_system.process_account_definitions(ast)?;

            // Generate function dispatcher metadata
            function_dispatcher.generate_dispatcher(
                self,
                ast,
                &mut dispatcher_account_system,
                &mut scope_analyzer,
                &mut ast_generator,
                &mut symbol_table.clone(),
            )?;
        }

        // Generate AST with coordinated function indices
        ast_generator.generate_ast_node(self, ast)?;

        Ok(ast_generator)
    }


}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::TypeNode;
    use five_protocol::Value;

    #[test]
    fn test_ast_generator_creation() {
        let generator = ASTGenerator::new();
        assert_eq!(generator.local_symbol_table.len(), 0);
        assert_eq!(generator.field_counter, 0);
    }

    #[test]
    fn test_type_inference() {
        let generator = ASTGenerator::new();

        let u64_literal = AstNode::Literal(Value::U64(42));
        let bool_literal = AstNode::Literal(Value::Bool(true));

        assert_eq!(generator.infer_type_from_node(&u64_literal).unwrap(), "u64");
        assert_eq!(
            generator.infer_type_from_node(&bool_literal).unwrap(),
            "bool"
        );
    }

    #[test]
    fn test_simple_expression_detection() {
        let generator = ASTGenerator::new();

        let literal = AstNode::Literal(Value::U64(42));
        let identifier = AstNode::Identifier("test".to_string());
        let binary_expr = AstNode::BinaryExpression {
            left: Box::new(AstNode::Literal(Value::U64(1))),
            right: Box::new(AstNode::Literal(Value::U64(2))),
            operator: "+".to_string(),
        };

        assert!(generator.is_simple_expression(&literal));
        assert!(generator.is_simple_expression(&identifier));
        assert!(!generator.is_simple_expression(&binary_expr));
    }

    #[test]
    fn test_type_node_to_string() {
        let generator = ASTGenerator::new();

        let primitive = TypeNode::Primitive("u64".to_string());
        let array = TypeNode::Array {
            element_type: Box::new(TypeNode::Primitive("u8".to_string())),
            size: Some(10),
        };

        assert_eq!(generator.type_node_to_string(&primitive), "u64");
        assert_eq!(generator.type_node_to_string(&array), "[u8; 10]");
    }

    #[test]
    fn test_parse_program_id_base58_pubkey() {
        let generator = ASTGenerator::new();
        let parsed = generator
            .parse_program_id("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")
            .expect("valid pubkey");
        assert_eq!(
            parsed,
            [
                6, 221, 246, 225, 215, 101, 161, 147, 217, 203, 225, 70, 206, 235, 121, 172,
                28, 180, 133, 237, 95, 91, 55, 145, 58, 140, 245, 133, 126, 255, 0, 169,
            ]
        );
    }
}
