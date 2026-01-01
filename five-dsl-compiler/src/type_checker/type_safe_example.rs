//! Example of type-safe type checking with new AST
//!
//! This module demonstrates how to use the type-safe checker API.
//! It shows the pattern for migrating existing type checking code.

#![allow(dead_code)]

use super::types::TypeCheckerContext;
use crate::ast::{AstNode, generated::*};
use five_vm_mito::error::VMError;

/// Example: Type-check an expression using the type-safe API
///
/// Before migration (old way):
/// ```ignore
/// match expr {
///     AstNode::BinaryExpression { left, right, .. } => {
///         self.check_types(left)?;
///         self.check_types(right)?;
///         Ok(())
///     }
///     AstNode::StringLiteral { .. } => Ok(()),
///     AstNode::Identifier(name) => {
///         // Check identifier is defined...
///     }
///     // ... more cases
///     _ => Err(VMError::InvalidScript),
/// }
/// ```
///
/// After migration (new way):
/// ```ignore
/// let expr: Expression = node.try_into()?;
/// self.check_expression_safe(&expr)?;
/// ```
#[allow(unused)]
fn example_type_safe_expression_checking(
    checker: &mut TypeCheckerContext,
    expr: &Expression,
) -> Result<(), VMError> {
    // The type-safe checker ensures:
    // 1. All expression variants are handled
    // 2. No invalid variants can slip through
    // 3. Type safety at compile time
    checker.check_expression_safe(expr)
}

/// Example: Type-check a statement using the type-safe API
///
/// This eliminates the need for massive match statements and
/// provides compile-time guarantees about correctness.
#[allow(unused)]
fn example_type_safe_statement_checking(
    checker: &mut TypeCheckerContext,
    stmt: &Statement,
) -> Result<(), VMError> {
    // Type safe statement checking with guaranteed coverage
    checker.check_statement_safe(stmt)
}

/// Example: Build a type-safe statement during type checking
///
/// Instead of manually constructing AstNode::IfStatement { ... },
/// we use the builder API which:
/// 1. Ensures all required fields are present
/// 2. Uses strong types (not strings/boxes everywhere)
/// 3. Automatically converts to AstNode when needed
#[allow(unused)]
fn example_build_statement(
    checker: &mut TypeCheckerContext,
    condition: AstNode,
    then_branch: AstNode,
) -> AstNode {
    let statement = TypeCheckerContext::build_if_statement(
        condition,
        then_branch,
        None,
    );

    // Automatic conversion to AstNode
    statement.into()
}

/// Example: Pattern showing how to migrate a type checker function
///
/// This is the migration pattern:
/// 1. Keep function signature the same (takes AstNode)
/// 2. Convert to type-safe type at start
/// 3. Use type-safe checking
/// 4. Return same results
#[allow(unused)]
fn example_migrated_check_function(
    checker: &mut TypeCheckerContext,
    node: &AstNode,
) -> Result<(), VMError> {
    match node {
        AstNode::IfStatement { condition, then_branch, else_branch } => {
            // With type-safe approach, we could construct a Statement::IfStatement
            // and use type-safe checking. For now, we keep existing behavior.
            checker.check_types(condition)?;
            checker.check_types(then_branch)?;
            if let Some(else_b) = else_branch {
                checker.check_types(else_b)?;
            }
            Ok(())
        }
        _ => {
            // For other nodes, keep existing behavior
            checker.check_types(node)
        }
    }
}

/// Example: Complete migration of a type checker module section
///
/// This pattern can be applied module-by-module to migrate the entire
/// type checker without breaking existing code.
impl TypeCheckerContext {
    #[allow(unused)]
    pub(crate) fn check_expression_migrated(
        &mut self,
        expr: &AstNode,
    ) -> Result<(), VMError> {
        // Step 1: Check if this is an expression-type node
        match expr {
            AstNode::Identifier(_)
            | AstNode::Literal(_)
            | AstNode::StringLiteral { .. }
            | AstNode::FieldAccess { .. }
            | AstNode::ArrayAccess { .. }
            | AstNode::FunctionCall { .. }
            | AstNode::MethodCall { .. }
            | AstNode::BinaryExpression { .. }
            | AstNode::UnaryExpression { .. } => {
                // These are expression-type nodes
                // In a real migration, we could use the type-safe checker
                // for better type safety and guarantees
                self.check_types(expr)
            }
            _ => {
                // Falls back to old checking for non-expressions
                self.check_types(expr)
            }
        }
    }
}

/// Benefits of the type-safe approach:
///
/// 1. **Compile-time Safety**: Can't pass Statement where Expression expected
/// 2. **No Match Exhaustion Bugs**: Compiler ensures all variants handled
/// 3. **Better Refactoring**: Adding a new variant forces update at compile time
/// 4. **Clearer Intent**: Code expresses what it's checking
/// 5. **Incremental Migration**: Can migrate function-by-function
/// 6. **Zero Runtime Cost**: Conversions compile away, same performance
/// 7. **Backward Compatibility**: Existing AstNode code unchanged

#[cfg(test)]
mod tests {
    

    #[test]
    fn test_type_safe_pattern() {
        // This test just demonstrates the pattern compiles
        // Actual tests are in the individual modules
    }
}
