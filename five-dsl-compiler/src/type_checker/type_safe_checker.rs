//! Type-safe type checker for new AST structures
//!
//! This module provides type checking methods that work directly with the new
//! Expression, Statement, and Definition types from the generated AST.
//! This enables compile-time type safety and eliminates impossible states.

use super::types::TypeCheckerContext;
use crate::ast::generated::*;
use five_vm_mito::error::VMError;

impl TypeCheckerContext {
    /// Check type safety of an Expression
    /// This provides type-safe checking without need for pattern matching on impossible states
    pub(crate) fn check_expression_safe(&mut self, expr: &Expression) -> Result<(), VMError> {
        match expr {
            Expression::Identifier(name) => self.check_identifier(name),
            Expression::Literal(_value) => Ok(()),
            Expression::StringLiteral(_) => Ok(()),
            Expression::ArrayLiteral(node) => {
                for element in &node.elements {
                    self.check_types(element)?;
                }
                Ok(())
            }
            Expression::TupleLiteral(node) => {
                for element in &node.elements {
                    self.check_types(element)?;
                }
                Ok(())
            }
            Expression::StructLiteral(node) => {
                for field in &node.fields {
                    self.check_types(&field.value)?;
                }
                Ok(())
            }
            Expression::TemplateLiteral(node) => {
                for part in &node.parts {
                    self.check_types(part)?;
                }
                Ok(())
            }
            Expression::FieldAccess(node) => {
                self.check_types(&node.object)?;
                Ok(())
            }
            Expression::ArrayAccess(node) => {
                self.check_types(&node.array)?;
                self.check_types(&node.index)?;
                Ok(())
            }
            Expression::TupleAccess(node) => {
                self.check_types(&node.object)?;
                Ok(())
            }
            Expression::FunctionCall(node) => {
                for arg in &node.args {
                    self.check_types(arg)?;
                }
                Ok(())
            }
            Expression::MethodCall(node) => {
                self.check_types(&node.object)?;
                for arg in &node.args {
                    self.check_types(arg)?;
                }
                self.check_method_call_safety(&node.method)?;
                Ok(())
            }
            Expression::EnumVariantAccess(_) => Ok(()),
            Expression::ErrorPropagation(node) => {
                self.check_types(&node.expression)?;
                Ok(())
            }
            Expression::UnaryExpression(node) => {
                self.check_types(&node.operand)?;
                self.check_unary_operator_safety(&node.operator)?;
                Ok(())
            }
            Expression::BinaryExpression(node) => {
                self.check_types(&node.left)?;
                self.check_types(&node.right)?;
                self.check_binary_operator_safety(&node.operator)?;
                Ok(())
            }
        }
    }

    /// Check type safety of a Statement
    pub(crate) fn check_statement_safe(&mut self, stmt: &Statement) -> Result<(), VMError> {
        match stmt {
            Statement::Assignment(node) => {
                self.check_types(&node.value)?;
                Ok(())
            }
            Statement::FieldAssignment(node) => {
                self.check_types(&node.object)?;
                self.check_types(&node.value)?;
                Ok(())
            }
            Statement::RequireStatement(node) => {
                self.check_types(&node.condition)?;
                Ok(())
            }
            Statement::LetStatement(node) => {
                self.check_types(&node.value)?;
                if let Some(type_annotation) = &node.type_annotation {
                    if !self.is_valid_type_node(type_annotation) {
                        return Err(VMError::InvalidScript);
                    }
                }
                Ok(())
            }
            Statement::TupleDestructuring(node) => {
                self.check_types(&node.value)?;
                Ok(())
            }
            Statement::TupleAssignment(node) => {
                self.check_types(&node.value)?;
                for target in &node.targets {
                    self.check_types(target)?;
                }
                Ok(())
            }
            Statement::IfStatement(node) => {
                self.check_types(&node.condition)?;
                self.check_types(&node.then_branch)?;
                if let Some(else_branch) = &node.else_branch {
                    self.check_types(else_branch)?;
                }
                Ok(())
            }
            Statement::MatchExpression(node) => {
                self.check_types(&node.expression)?;
                for arm in &node.arms {
                    self.check_types(&arm.pattern)?;
                    if let Some(guard) = &arm.guard {
                        self.check_types(guard)?;
                    }
                    self.check_types(&arm.body)?;
                }
                Ok(())
            }
            Statement::ReturnStatement(node) => {
                if let Some(value) = &node.value {
                    self.check_types(value)?;
                }
                Ok(())
            }
            Statement::ForLoop(node) => {
                if let Some(init) = &node.init {
                    self.check_types(init)?;
                }
                if let Some(condition) = &node.condition {
                    self.check_types(condition)?;
                }
                if let Some(update) = &node.update {
                    self.check_types(update)?;
                }
                self.check_types(&node.body)?;
                Ok(())
            }
            Statement::ForInLoop(node) => {
                self.check_types(&node.iterable)?;
                self.check_types(&node.body)?;
                Ok(())
            }
            Statement::ForOfLoop(node) => {
                self.check_types(&node.iterable)?;
                self.check_types(&node.body)?;
                Ok(())
            }
            Statement::WhileLoop(node) => {
                self.check_types(&node.condition)?;
                self.check_types(&node.body)?;
                Ok(())
            }
            Statement::DoWhileLoop(node) => {
                self.check_types(&node.body)?;
                self.check_types(&node.condition)?;
                Ok(())
            }
            Statement::SwitchStatement(node) => {
                self.check_types(&node.discriminant)?;
                for case in &node.cases {
                    self.check_types(&case.pattern)?;
                    for stmt in &case.body {
                        self.check_types(stmt)?;
                    }
                }
                if let Some(default) = &node.default_case {
                    self.check_types(default)?;
                }
                Ok(())
            }
            Statement::BreakStatement(_) => Ok(()),
            Statement::ContinueStatement(_) => Ok(()),
            Statement::EmitStatement(node) => {
                for field in &node.fields {
                    self.check_types(&field.value)?;
                }
                Ok(())
            }
            Statement::AssertStatement(node) => {
                for arg in &node.args {
                    self.check_types(arg)?;
                }
                Ok(())
            }
        }
    }

    /// Check type safety of a Definition
    #[allow(dead_code)]
    pub(crate) fn check_definition_safe(&mut self, def: &Definition) -> Result<(), VMError> {
        match def {
            Definition::FieldDefinition(node) => {
                if !self.is_valid_type_node(&node.field_type) {
                    return Err(VMError::InvalidScript);
                }
                if let Some(default) = &node.default_value {
                    self.check_types(default)?;
                }
                Ok(())
            }
            Definition::InstructionDefinition(node) => {
                for param in &node.parameters {
                    if !self.is_valid_type_node(&param.param_type) {
                        return Err(VMError::InvalidScript);
                    }
                }
                if let Some(return_type) = &node.return_type {
                    if !self.is_valid_type_node(return_type) {
                        return Err(VMError::InvalidScript);
                    }
                }
                self.check_types(&node.body)?;
                Ok(())
            }
            Definition::EventDefinition(node) => {
                for field in &node.fields {
                    if !self.is_valid_type_node(&field.field_type) {
                        return Err(VMError::InvalidScript);
                    }
                }
                Ok(())
            }
            Definition::ErrorTypeDefinition(_) => Ok(()),
            Definition::AccountDefinition(node) => {
                for field in &node.fields {
                    if !self.is_valid_type_node(&field.field_type) {
                        return Err(VMError::InvalidScript);
                    }
                }
                Ok(())
            }
            Definition::InterfaceDefinition(_) => Ok(()),
            Definition::InterfaceFunction(node) => {
                for param in &node.parameters {
                    if !self.is_valid_type_node(&param.param_type) {
                        return Err(VMError::InvalidScript);
                    }
                }
                if let Some(return_type) = &node.return_type {
                    if !self.is_valid_type_node(return_type) {
                        return Err(VMError::InvalidScript);
                    }
                }
                Ok(())
            }
            Definition::ImportStatement(_) => Ok(()),
            Definition::ArrowFunction(node) => {
                for param in &node.parameters {
                    if !self.is_valid_type_node(&param.param_type) {
                        return Err(VMError::InvalidScript);
                    }
                }
                if let Some(return_type) = &node.return_type {
                    if !self.is_valid_type_node(return_type) {
                        return Err(VMError::InvalidScript);
                    }
                }
                self.check_types(&node.body)?;
                Ok(())
            }
            Definition::TestFunction(node) => {
                self.check_types(&node.body)?;
                Ok(())
            }
            Definition::TestModule(node) => {
                self.check_types(&node.body)?;
                Ok(())
            }
        }
    }

    // Helper methods for type safety checks

    fn check_identifier(&mut self, name: &str) -> Result<(), VMError> {
        match name {
            "None" | "signer" => Ok(()),
            _ => {
                if !self.symbol_table.contains_key(name)
                    && !self.interface_registry.contains_key(name)
                    && !self.imported_external_interfaces.contains(name)
                {
                    return Err(self.undefined_identifier_error(name));
                }
                Ok(())
            }
        }
    }

    fn check_method_call_safety(&self, method: &str) -> Result<(), VMError> {
        match method {
            "add" | "sub" | "mul" | "div" | "and" | "or" => Ok(()),
            _ => Ok(()), // Other methods are allowed
        }
    }

    fn check_unary_operator_safety(&self, operator: &str) -> Result<(), VMError> {
        match operator {
            "!" | "-" | "+" => Ok(()),
            _ => Err(VMError::InvalidScript),
        }
    }

    fn check_binary_operator_safety(&self, operator: &str) -> Result<(), VMError> {
        match operator {
            "+" | "-" | "*" | "/" | "%" | "==" | "!=" | "<" | "<=" | ">" | ">=" | "&&" | "||"
            | "range" => Ok(()),
            _ => Err(VMError::InvalidScript),
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_type_safe_checker_methods_exist() {
        // This test just verifies the module compiles and exports the right methods
        // Detailed testing will be done when we integrate with actual type checker
    }
}
