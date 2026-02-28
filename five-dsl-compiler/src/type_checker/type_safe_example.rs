//! Examples for the type-safe checker API.

#![allow(dead_code)]

use super::types::TypeCheckerContext;
use crate::ast::{generated::*, AstNode};
use five_vm_mito::error::VMError;

/// Type-check an expression using the type-safe API.
#[allow(unused)]
fn example_type_safe_expression_checking(
    checker: &mut TypeCheckerContext,
    expr: &Expression,
) -> Result<(), VMError> {
    checker.check_expression_safe(expr)
}

/// Type-check a statement using the type-safe API.
#[allow(unused)]
fn example_type_safe_statement_checking(
    checker: &mut TypeCheckerContext,
    stmt: &Statement,
) -> Result<(), VMError> {
    checker.check_statement_safe(stmt)
}

/// Build a type-safe statement during type checking.
#[allow(unused)]
fn example_build_statement(
    checker: &mut TypeCheckerContext,
    condition: AstNode,
    then_branch: AstNode,
) -> AstNode {
    let statement = TypeCheckerContext::build_if_statement(condition, then_branch, None);

    statement.into()
}

/// Migration pattern for a type checker function.
#[allow(unused)]
fn example_migrated_check_function(
    checker: &mut TypeCheckerContext,
    node: &AstNode,
) -> Result<(), VMError> {
    match node {
        AstNode::IfStatement {
            condition,
            then_branch,
            else_branch,
        } => {
            checker.check_types(condition)?;
            checker.check_types(then_branch)?;
            if let Some(else_b) = else_branch {
                checker.check_types(else_b)?;
            }
            Ok(())
        }
        _ => checker.check_types(node),
    }
}

/// Migration example for a type checker module section.
impl TypeCheckerContext {
    #[allow(unused)]
    pub(crate) fn check_expression_migrated(&mut self, expr: &AstNode) -> Result<(), VMError> {
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
            | AstNode::UnaryExpression { .. } => self.check_types(expr),
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
