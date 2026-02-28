//! Expression builder for constructing new type-safe Expression nodes
//!
//! This module provides a builder API for creating Expression nodes from the
//! new generated AST structures. These can be automatically converted to AstNode
//! via the From trait.

use crate::ast::{generated::*, AstNode, StructLiteralField};
use five_protocol::Value;

/// Helper to build expression nodes
#[allow(dead_code)]
impl crate::parser::DslParser {
    /// Create a binary expression from new types
    pub(crate) fn build_binary_expression(
        operator: String,
        left: AstNode,
        right: AstNode,
    ) -> Expression {
        Expression::BinaryExpression(BinaryExpressionNode {
            operator,
            left: Box::new(left),
            right: Box::new(right),
        })
    }

    /// Create a unary expression from new types
    pub(crate) fn build_unary_expression(operator: String, operand: AstNode) -> Expression {
        Expression::UnaryExpression(UnaryExpressionNode {
            operator,
            operand: Box::new(operand),
        })
    }

    /// Create a field access expression
    pub(crate) fn build_field_access(object: AstNode, field: String) -> Expression {
        Expression::FieldAccess(FieldAccessNode {
            object: Box::new(object),
            field,
        })
    }

    /// Create an array access expression
    pub(crate) fn build_array_access(array: AstNode, index: AstNode) -> Expression {
        Expression::ArrayAccess(ArrayAccessNode {
            array: Box::new(array),
            index: Box::new(index),
        })
    }

    /// Create a tuple access expression
    pub(crate) fn build_tuple_access(object: AstNode, index: u32) -> Expression {
        Expression::TupleAccess(TupleAccessNode {
            object: Box::new(object),
            index,
        })
    }

    /// Create a function call expression
    pub(crate) fn build_function_call(name: String, args: Vec<AstNode>) -> Expression {
        Expression::FunctionCall(FunctionCallNode { name, args })
    }

    /// Create a method call expression
    pub(crate) fn build_method_call(
        object: AstNode,
        method: String,
        args: Vec<AstNode>,
    ) -> Expression {
        Expression::MethodCall(MethodCallNode {
            object: Box::new(object),
            method,
            args,
        })
    }

    /// Create a string literal expression
    pub(crate) fn build_string_literal(value: String) -> Expression {
        Expression::StringLiteral(StringLiteralNode { value })
    }

    /// Create an array literal expression
    pub(crate) fn build_array_literal(elements: Vec<AstNode>) -> Expression {
        Expression::ArrayLiteral(ArrayLiteralNode { elements })
    }

    /// Create a tuple literal expression
    pub(crate) fn build_tuple_literal(elements: Vec<AstNode>) -> Expression {
        Expression::TupleLiteral(TupleLiteralNode { elements })
    }

    /// Create a struct literal expression
    pub(crate) fn build_struct_literal(fields: Vec<StructLiteralField>) -> Expression {
        Expression::StructLiteral(StructLiteralNode { fields })
    }

    /// Create a template literal expression
    pub(crate) fn build_template_literal(parts: Vec<AstNode>) -> Expression {
        Expression::TemplateLiteral(TemplateLiteralNode { parts })
    }

    /// Create an enum variant access expression
    pub(crate) fn build_enum_variant_access(enum_name: String, variant_name: String) -> Expression {
        Expression::EnumVariantAccess(EnumVariantAccessNode {
            enum_name,
            variant_name,
        })
    }

    /// Create an error propagation expression
    pub(crate) fn build_error_propagation(expression: AstNode) -> Expression {
        Expression::ErrorPropagation(ErrorPropagationNode {
            expression: Box::new(expression),
        })
    }

    /// Create an identifier expression
    pub(crate) fn build_identifier(name: String) -> Expression {
        Expression::Identifier(name)
    }

    /// Create a literal expression
    pub(crate) fn build_literal(value: Value) -> Expression {
        Expression::Literal(value)
    }
}
