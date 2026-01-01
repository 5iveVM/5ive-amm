//! Conversion implementations between new type-safe AST and legacy AstNode enum
//!
//! This module provides TryFrom implementations to enable conversion
//! from the legacy AstNode enum back to the new type-safe AST structures
//! (Expression, Statement, Definition) for migration purposes.
//!
//! NOTE: From implementations (AstNode <- Expression/Statement/Definition) are
//! auto-generated in src/ast/generated.rs by the generate_ast tool.

#[cfg(test)]
mod tests {
    // use super::*; // Module is empty, so wildcard import is unused
    use crate::ast::{AstNode, generated::*};

    #[test]
    fn test_expression_to_astnode_conversion() {
        let expr = Expression::Identifier("test".to_string());
        let node: AstNode = expr.into();
        assert!(matches!(node, AstNode::Identifier(_)));
    }

    #[test]
    fn test_statement_to_astnode_conversion() {
        let stmt = Statement::RequireStatement(RequireStatementNode {
            condition: Box::new(AstNode::Identifier("x".to_string())),
        });
        let node: AstNode = stmt.into();
        assert!(matches!(node, AstNode::RequireStatement { .. }));
    }
}
