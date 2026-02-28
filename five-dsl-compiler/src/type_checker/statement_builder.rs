//! Statement builder for type-safe statement construction
//!
//! Provides helper methods for building type-safe Statement nodes during type checking.
//! These statements automatically convert to AstNode via the From trait.

use crate::ast::{generated::*, AssertionType, AstNode, EventFieldAssignment, MatchArm, TypeNode};

/// Helper methods for building statements
#[allow(dead_code)]
impl super::types::TypeCheckerContext {
    /// Create an if statement
    pub(crate) fn build_if_statement(
        condition: AstNode,
        then_branch: AstNode,
        else_branch: Option<AstNode>,
    ) -> Statement {
        Statement::IfStatement(IfStatementNode {
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            else_branch: else_branch.map(Box::new),
        })
    }

    /// Create a while loop
    pub(crate) fn build_while_loop(condition: AstNode, body: AstNode) -> Statement {
        Statement::WhileLoop(WhileLoopNode {
            condition: Box::new(condition),
            body: Box::new(body),
        })
    }

    /// Create a for loop
    pub(crate) fn build_for_loop(
        init: Option<AstNode>,
        condition: Option<AstNode>,
        update: Option<AstNode>,
        body: AstNode,
    ) -> Statement {
        Statement::ForLoop(ForLoopNode {
            init: init.map(Box::new),
            condition: condition.map(Box::new),
            update: update.map(Box::new),
            body: Box::new(body),
        })
    }

    /// Create a return statement
    pub(crate) fn build_return_statement(value: Option<AstNode>) -> Statement {
        Statement::ReturnStatement(ReturnStatementNode {
            value: value.map(Box::new),
        })
    }

    /// Create a let statement (variable declaration)
    pub(crate) fn build_let_statement(
        name: String,
        value: AstNode,
        type_annotation: Option<TypeNode>,
        is_mutable: bool,
    ) -> Statement {
        Statement::LetStatement(LetStatementNode {
            name,
            value: Box::new(value),
            type_annotation: type_annotation.map(Box::new),
            is_mutable,
        })
    }

    /// Create an assignment statement
    pub(crate) fn build_assignment(target: String, value: AstNode) -> Statement {
        Statement::Assignment(AssignmentNode {
            target,
            value: Box::new(value),
        })
    }

    /// Create a field assignment statement
    pub(crate) fn build_field_assignment(
        object: AstNode,
        field: String,
        value: AstNode,
    ) -> Statement {
        Statement::FieldAssignment(FieldAssignmentNode {
            object: Box::new(object),
            field,
            value: Box::new(value),
        })
    }

    /// Create a require statement (constraint)
    pub(crate) fn build_require_statement(condition: AstNode) -> Statement {
        Statement::RequireStatement(RequireStatementNode {
            condition: Box::new(condition),
        })
    }

    /// Create an emit statement (event emission)
    pub(crate) fn build_emit_statement(
        event_name: String,
        fields: Vec<EventFieldAssignment>,
    ) -> Statement {
        Statement::EmitStatement(EmitStatementNode { event_name, fields })
    }

    /// Create a match expression statement
    pub(crate) fn build_match_expression(expression: AstNode, arms: Vec<MatchArm>) -> Statement {
        Statement::MatchExpression(MatchExpressionNode {
            expression: Box::new(expression),
            arms,
        })
    }

    /// Create a tuple destructuring statement
    pub(crate) fn build_tuple_destructuring(targets: Vec<String>, value: AstNode) -> Statement {
        Statement::TupleDestructuring(TupleDestructuringNode {
            targets,
            value: Box::new(value),
        })
    }

    /// Create a tuple assignment statement
    pub(crate) fn build_tuple_assignment(targets: Vec<AstNode>, value: AstNode) -> Statement {
        Statement::TupleAssignment(TupleAssignmentNode {
            targets,
            value: Box::new(value),
        })
    }

    /// Create an assert statement (for testing)
    pub(crate) fn build_assert_statement(
        assertion_type: AssertionType,
        args: Vec<AstNode>,
    ) -> Statement {
        Statement::AssertStatement(AssertStatementNode {
            assertion_type,
            args,
        })
    }

    /// Create a break statement
    pub(crate) fn build_break_statement(label: Option<String>) -> Statement {
        Statement::BreakStatement(BreakStatementNode { label })
    }

    /// Create a continue statement
    pub(crate) fn build_continue_statement(label: Option<String>) -> Statement {
        Statement::ContinueStatement(ContinueStatementNode { label })
    }
}
