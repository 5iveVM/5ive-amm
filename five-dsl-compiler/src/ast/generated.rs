// AUTO-GENERATED FILE: Do not edit manually
// Generated from node_metadata.toml by generate_ast tool
// Run: cargo run --bin generate_ast

use crate::ast::{
    AssertionType, AstNode, BlockKind, ErrorVariant, EventFieldAssignment, ImportItem,
    InstructionParameter, MatchArm, ModuleSpecifier, StructField, StructLiteralField, SwitchCase,
    TestAttribute, TypeNode, Visibility,
};
use five_protocol::Value;

// ============================================================================
// INDIVIDUAL NODE STRUCTS
// ============================================================================

/// Field access on object (obj.field)
#[derive(Debug, Clone, PartialEq)]
pub struct FieldAccessNode {
    pub object: Box<AstNode>,
    pub field: String,
}

/// Binary operators (a + b, a == b, etc)
#[derive(Debug, Clone, PartialEq)]
pub struct BinaryExpressionNode {
    pub left: Box<AstNode>,
    pub operator: String,
    pub right: Box<AstNode>,
}

/// Template literal with interpolation
#[derive(Debug, Clone, PartialEq)]
pub struct TemplateLiteralNode {
    pub parts: Vec<AstNode>,
}

/// C-style for loop
#[derive(Debug, Clone, PartialEq)]
pub struct ForLoopNode {
    pub body: Box<AstNode>,
    pub update: Option<Box<AstNode>>,
    pub condition: Option<Box<AstNode>>,
    pub init: Option<Box<AstNode>>,
}

/// Variable assignment statement
#[derive(Debug, Clone, PartialEq)]
pub struct AssignmentNode {
    pub value: Box<AstNode>,
    pub target: String,
}

/// Return from function with optional value
#[derive(Debug, Clone, PartialEq)]
pub struct ReturnStatementNode {
    pub value: Option<Box<AstNode>>,
}

/// Pattern matching expression
#[derive(Debug, Clone, PartialEq)]
pub struct MatchExpressionNode {
    pub arms: Vec<MatchArm>,
    pub expression: Box<AstNode>,
}

/// For-of loop iterating over values
#[derive(Debug, Clone, PartialEq)]
pub struct ForOfLoopNode {
    pub variable: String,
    pub body: Box<AstNode>,
    pub iterable: Box<AstNode>,
}

/// Block of statements enclosed in braces
#[derive(Debug, Clone, PartialEq)]
pub struct BlockNode {
    pub statements: Vec<AstNode>,
    pub kind: BlockKind,
}

/// Assign to multiple targets in tuple form
#[derive(Debug, Clone, PartialEq)]
pub struct TupleAssignmentNode {
    pub value: Box<AstNode>,
    pub targets: Vec<AstNode>,
}

/// On-chain instruction definition
#[derive(Debug, Clone, PartialEq)]
pub struct InstructionDefinitionNode {
    pub visibility: Visibility,
    pub name: String,
    pub return_type: Option<Box<TypeNode>>,
    pub parameters: Vec<InstructionParameter>,
    pub body: Box<AstNode>,
}

/// Error propagation operator (?)
#[derive(Debug, Clone, PartialEq)]
pub struct ErrorPropagationNode {
    pub expression: Box<AstNode>,
}

/// Switch statement with multiple cases
#[derive(Debug, Clone, PartialEq)]
pub struct SwitchStatementNode {
    pub default_case: Option<Box<AstNode>>,
    pub discriminant: Box<AstNode>,
    pub cases: Vec<SwitchCase>,
}

/// Method in an interface
#[derive(Debug, Clone, PartialEq)]
pub struct InterfaceFunctionNode {
    pub return_type: Option<Box<TypeNode>>,
    pub parameters: Vec<InstructionParameter>,
    pub name: String,
    pub discriminator: Option<u8>,
    pub discriminator_bytes: Option<Vec<u8>>,
    pub is_anchor: bool,
}

/// Account type definition
#[derive(Debug, Clone, PartialEq)]
pub struct AccountDefinitionNode {
    pub fields: Vec<StructField>,
    pub name: String,
    pub visibility: Visibility,
}

/// Tuple literal (...)
#[derive(Debug, Clone, PartialEq)]
pub struct TupleLiteralNode {
    pub elements: Vec<AstNode>,
}

/// Assertion statement for testing
#[derive(Debug, Clone, PartialEq)]
pub struct AssertStatementNode {
    pub assertion_type: AssertionType,
    pub args: Vec<AstNode>,
}

/// Unit test function
#[derive(Debug, Clone, PartialEq)]
pub struct TestFunctionNode {
    pub body: Box<AstNode>,
    pub name: String,
    pub attributes: Vec<TestAttribute>,
}

/// Struct literal Type { ... }
#[derive(Debug, Clone, PartialEq)]
pub struct StructLiteralNode {
    pub fields: Vec<StructLiteralField>,
}

/// Function call function(...)
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionCallNode {
    pub args: Vec<AstNode>,
    pub name: String,
}

/// Field definition in account or struct
#[derive(Debug, Clone, PartialEq)]
pub struct FieldDefinitionNode {
    pub visibility: Visibility,
    pub is_mutable: bool,
    pub name: String,
    pub default_value: Option<Box<AstNode>>,
    pub is_optional: bool,
    pub field_type: Box<TypeNode>,
}

/// Enum variant access (Enum::Variant)
#[derive(Debug, Clone, PartialEq)]
pub struct EnumVariantAccessNode {
    pub enum_name: String,
    pub variant_name: String,
}

/// Emit an event with field assignments
#[derive(Debug, Clone, PartialEq)]
pub struct EmitStatementNode {
    pub event_name: String,
    pub fields: Vec<EventFieldAssignment>,
}

/// Cross-program interface definition
#[derive(Debug, Clone, PartialEq)]
pub struct InterfaceDefinitionNode {
    pub name: String,
    pub program_id: Option<String>,
    pub serializer: Option<String>,
    pub is_anchor: bool,
    pub functions: Vec<AstNode>,
}

/// String literal
#[derive(Debug, Clone, PartialEq)]
pub struct StringLiteralNode {
    pub value: String,
}

/// Event type definition
#[derive(Debug, Clone, PartialEq)]
pub struct EventDefinitionNode {
    pub visibility: Visibility,
    pub name: String,
    pub fields: Vec<StructField>,
}

/// Conditional statement with optional else block
#[derive(Debug, Clone, PartialEq)]
pub struct IfStatementNode {
    pub else_branch: Option<Box<AstNode>>,
    pub condition: Box<AstNode>,
    pub then_branch: Box<AstNode>,
}

/// While loop with condition
#[derive(Debug, Clone, PartialEq)]
pub struct WhileLoopNode {
    pub body: Box<AstNode>,
    pub condition: Box<AstNode>,
}

/// Import/use statement for modules
#[derive(Debug, Clone, PartialEq)]
pub struct ImportStatementNode {
    pub module_specifier: ModuleSpecifier,
    pub imported_items: Option<Vec<ImportItem>>,
}

/// Array literal [...]
#[derive(Debug, Clone, PartialEq)]
pub struct ArrayLiteralNode {
    pub elements: Vec<AstNode>,
}

/// Custom error type with variants
#[derive(Debug, Clone, PartialEq)]
pub struct ErrorTypeDefinitionNode {
    pub variants: Vec<ErrorVariant>,
    pub name: String,
}

/// Do-while loop executing at least once
#[derive(Debug, Clone, PartialEq)]
pub struct DoWhileLoopNode {
    pub body: Box<AstNode>,
    pub condition: Box<AstNode>,
}

/// Root node representing entire DSL program
#[derive(Debug, Clone, PartialEq)]
pub struct ProgramNode {
    pub init_block: Option<Box<AstNode>>,
    pub account_definitions: Vec<AstNode>,
    pub constraints_block: Option<Box<AstNode>>,
    pub program_name: String,
    pub event_definitions: Vec<AstNode>,
    pub instruction_definitions: Vec<AstNode>,
    pub interface_definitions: Vec<AstNode>,
    pub import_statements: Vec<AstNode>,
    pub field_definitions: Vec<AstNode>,
}

/// Array/collection access (arr[index])
#[derive(Debug, Clone, PartialEq)]
pub struct ArrayAccessNode {
    pub index: Box<AstNode>,
    pub array: Box<AstNode>,
}

/// For-in loop iterating over object properties
#[derive(Debug, Clone, PartialEq)]
pub struct ForInLoopNode {
    pub iterable: Box<AstNode>,
    pub variable: String,
    pub body: Box<AstNode>,
}

/// Method call interface.method(...)
#[derive(Debug, Clone, PartialEq)]
pub struct MethodCallNode {
    pub args: Vec<AstNode>,
    pub method: String,
    pub object: Box<AstNode>,
}

/// Runtime assertion/constraint statement
#[derive(Debug, Clone, PartialEq)]
pub struct RequireStatementNode {
    pub condition: Box<AstNode>,
}

/// Break from loop or switch
#[derive(Debug, Clone, PartialEq)]
pub struct BreakStatementNode {
    pub label: Option<String>,
}

/// Arrow function / lambda
#[derive(Debug, Clone, PartialEq)]
pub struct ArrowFunctionNode {
    pub parameters: Vec<InstructionParameter>,
    pub body: Box<AstNode>,
    pub is_async: bool,
    pub return_type: Option<Box<TypeNode>>,
}

/// Test module grouping
#[derive(Debug, Clone, PartialEq)]
pub struct TestModuleNode {
    pub attributes: Vec<TestAttribute>,
    pub body: Box<AstNode>,
    pub name: String,
}

/// Destructure tuple into multiple variables
#[derive(Debug, Clone, PartialEq)]
pub struct TupleDestructuringNode {
    pub value: Box<AstNode>,
    pub targets: Vec<String>,
}

/// Tuple element access (tuple.0)
#[derive(Debug, Clone, PartialEq)]
pub struct TupleAccessNode {
    pub object: Box<AstNode>,
    pub index: u32,
}

/// Continue to next loop iteration
#[derive(Debug, Clone, PartialEq)]
pub struct ContinueStatementNode {
    pub label: Option<String>,
}

/// Field assignment on object (obj.field = value)
#[derive(Debug, Clone, PartialEq)]
pub struct FieldAssignmentNode {
    pub field: String,
    pub object: Box<AstNode>,
    pub value: Box<AstNode>,
}

/// Variable declaration with optional type annotation
#[derive(Debug, Clone, PartialEq)]
pub struct LetStatementNode {
    pub type_annotation: Option<Box<TypeNode>>,
    pub is_mutable: bool,
    pub value: Box<AstNode>,
    pub name: String,
}

/// Unary operators (!x, -x, +x)
#[derive(Debug, Clone, PartialEq)]
pub struct UnaryExpressionNode {
    pub operator: String,
    pub operand: Box<AstNode>,
}

// ============================================================================
// CATEGORY ENUMS (Type-safe AST organization)
// ============================================================================

/// definition nodes grouped for type safety
#[derive(Debug, Clone, PartialEq)]
pub enum Definition {
    FieldDefinition(FieldDefinitionNode),
    InstructionDefinition(InstructionDefinitionNode),
    EventDefinition(EventDefinitionNode),
    ErrorTypeDefinition(ErrorTypeDefinitionNode),
    AccountDefinition(AccountDefinitionNode),
    InterfaceDefinition(InterfaceDefinitionNode),
    InterfaceFunction(InterfaceFunctionNode),
    ImportStatement(ImportStatementNode),
    ArrowFunction(ArrowFunctionNode),
    TestFunction(TestFunctionNode),
    TestModule(TestModuleNode),
}

/// expression nodes grouped for type safety
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    StringLiteral(StringLiteralNode),
    ArrayLiteral(ArrayLiteralNode),
    TupleLiteral(TupleLiteralNode),
    StructLiteral(StructLiteralNode),
    TemplateLiteral(TemplateLiteralNode),
    FieldAccess(FieldAccessNode),
    ArrayAccess(ArrayAccessNode),
    TupleAccess(TupleAccessNode),
    FunctionCall(FunctionCallNode),
    MethodCall(MethodCallNode),
    EnumVariantAccess(EnumVariantAccessNode),
    ErrorPropagation(ErrorPropagationNode),
    UnaryExpression(UnaryExpressionNode),
    BinaryExpression(BinaryExpressionNode),
    Identifier(String),
    Literal(Value),
}

/// statement nodes grouped for type safety
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Assignment(AssignmentNode),
    FieldAssignment(FieldAssignmentNode),
    RequireStatement(RequireStatementNode),
    LetStatement(LetStatementNode),
    TupleDestructuring(TupleDestructuringNode),
    TupleAssignment(TupleAssignmentNode),
    IfStatement(IfStatementNode),
    MatchExpression(MatchExpressionNode),
    ReturnStatement(ReturnStatementNode),
    ForLoop(ForLoopNode),
    ForInLoop(ForInLoopNode),
    ForOfLoop(ForOfLoopNode),
    WhileLoop(WhileLoopNode),
    DoWhileLoop(DoWhileLoopNode),
    SwitchStatement(SwitchStatementNode),
    BreakStatement(BreakStatementNode),
    ContinueStatement(ContinueStatementNode),
    EmitStatement(EmitStatementNode),
    AssertStatement(AssertStatementNode),
}

/// structure nodes grouped for type safety
#[derive(Debug, Clone, PartialEq)]
#[allow(non_camel_case_types)]
pub enum structure {
    Block(BlockNode),
}

// ============================================================================
// BACKWARD COMPATIBILITY CONVERSIONS
// ============================================================================

impl From<Expression> for AstNode {
    fn from(expr: Expression) -> Self {
        match expr {
            Expression::StringLiteral(node) => AstNode::StringLiteral { value: node.value },
            Expression::ArrayLiteral(node) => AstNode::ArrayLiteral {
                elements: node.elements,
            },
            Expression::TupleLiteral(node) => AstNode::TupleLiteral {
                elements: node.elements,
            },
            Expression::StructLiteral(node) => AstNode::StructLiteral {
                fields: node.fields,
            },
            Expression::TemplateLiteral(node) => AstNode::TemplateLiteral { parts: node.parts },
            Expression::FieldAccess(node) => AstNode::FieldAccess {
                object: node.object,
                field: node.field,
            },
            Expression::ArrayAccess(node) => AstNode::ArrayAccess {
                index: node.index,
                array: node.array,
            },
            Expression::TupleAccess(node) => AstNode::TupleAccess {
                object: node.object,
                index: node.index,
            },
            Expression::FunctionCall(node) => AstNode::FunctionCall {
                args: node.args,
                name: node.name,
            },
            Expression::MethodCall(node) => AstNode::MethodCall {
                args: node.args,
                method: node.method,
                object: node.object,
            },
            Expression::EnumVariantAccess(node) => AstNode::EnumVariantAccess {
                enum_name: node.enum_name,
                variant_name: node.variant_name,
            },
            Expression::ErrorPropagation(node) => AstNode::ErrorPropagation {
                expression: node.expression,
            },
            Expression::UnaryExpression(node) => AstNode::UnaryExpression {
                operator: node.operator,
                operand: node.operand,
            },
            Expression::BinaryExpression(node) => AstNode::BinaryExpression {
                left: node.left,
                operator: node.operator,
                right: node.right,
            },
            Expression::Identifier(name) => AstNode::Identifier(name),
            Expression::Literal(value) => AstNode::Literal(value),
        }
    }
}

// ============================================================================
// STATEMENT CATEGORY CONVERSIONS
// ============================================================================

impl From<Statement> for AstNode {
    fn from(stmt: Statement) -> Self {
        match stmt {
            Statement::Assignment(node) => AstNode::Assignment {
                value: node.value,
                target: node.target,
            },
            Statement::FieldAssignment(node) => AstNode::FieldAssignment {
                field: node.field,
                object: node.object,
                value: node.value,
            },
            Statement::RequireStatement(node) => AstNode::RequireStatement {
                condition: node.condition,
            },
            Statement::LetStatement(node) => AstNode::LetStatement {
                type_annotation: node.type_annotation,
                is_mutable: node.is_mutable,
                value: node.value,
                name: node.name,
            },
            Statement::TupleDestructuring(node) => AstNode::TupleDestructuring {
                value: node.value,
                targets: node.targets,
            },
            Statement::TupleAssignment(node) => AstNode::TupleAssignment {
                value: node.value,
                targets: node.targets,
            },
            Statement::IfStatement(node) => AstNode::IfStatement {
                else_branch: node.else_branch,
                condition: node.condition,
                then_branch: node.then_branch,
            },
            Statement::MatchExpression(node) => AstNode::MatchExpression {
                arms: node.arms,
                expression: node.expression,
            },
            Statement::ReturnStatement(node) => AstNode::ReturnStatement { value: node.value },
            Statement::ForLoop(node) => AstNode::ForLoop {
                body: node.body,
                update: node.update,
                condition: node.condition,
                init: node.init,
            },
            Statement::ForInLoop(node) => AstNode::ForInLoop {
                iterable: node.iterable,
                variable: node.variable,
                body: node.body,
            },
            Statement::ForOfLoop(node) => AstNode::ForOfLoop {
                variable: node.variable,
                body: node.body,
                iterable: node.iterable,
            },
            Statement::WhileLoop(node) => AstNode::WhileLoop {
                body: node.body,
                condition: node.condition,
            },
            Statement::DoWhileLoop(node) => AstNode::DoWhileLoop {
                body: node.body,
                condition: node.condition,
            },
            Statement::SwitchStatement(node) => AstNode::SwitchStatement {
                default_case: node.default_case,
                discriminant: node.discriminant,
                cases: node.cases,
            },
            Statement::BreakStatement(node) => AstNode::BreakStatement { label: node.label },
            Statement::ContinueStatement(node) => AstNode::ContinueStatement { label: node.label },
            Statement::EmitStatement(node) => AstNode::EmitStatement {
                event_name: node.event_name,
                fields: node.fields,
            },
            Statement::AssertStatement(node) => AstNode::AssertStatement {
                assertion_type: node.assertion_type,
                args: node.args,
            },
        }
    }
}

// ============================================================================
// DEFINITION CATEGORY CONVERSIONS
// ============================================================================

impl From<Definition> for AstNode {
    fn from(def: Definition) -> Self {
        match def {
            Definition::FieldDefinition(node) => AstNode::FieldDefinition {
                visibility: node.visibility,
                is_mutable: node.is_mutable,
                name: node.name,
                default_value: node.default_value,
                is_optional: node.is_optional,
                field_type: node.field_type,
            },
            Definition::InstructionDefinition(node) => AstNode::InstructionDefinition {
                visibility: node.visibility,
                is_public: node.visibility.is_on_chain_callable(),
                name: node.name,
                return_type: node.return_type,
                parameters: node.parameters,
                body: node.body,
            },
            Definition::EventDefinition(node) => AstNode::EventDefinition {
                visibility: node.visibility,
                name: node.name,
                fields: node.fields,
            },
            Definition::ErrorTypeDefinition(node) => AstNode::ErrorTypeDefinition {
                variants: node.variants,
                name: node.name,
            },
            Definition::AccountDefinition(node) => AstNode::AccountDefinition {
                fields: node.fields,
                name: node.name,
                visibility: node.visibility,
            },
            Definition::InterfaceDefinition(node) => AstNode::InterfaceDefinition {
                name: node.name,
                program_id: node.program_id,
                serializer: node.serializer,
                is_anchor: node.is_anchor,
                functions: node.functions,
            },
            Definition::InterfaceFunction(node) => AstNode::InterfaceFunction {
                return_type: node.return_type,
                parameters: node.parameters,
                name: node.name,
                discriminator: node.discriminator,
                discriminator_bytes: node.discriminator_bytes,
                is_anchor: node.is_anchor,
            },
            Definition::ImportStatement(node) => AstNode::ImportStatement {
                module_specifier: node.module_specifier,
                imported_items: node.imported_items,
                location: None,
            },
            Definition::ArrowFunction(node) => AstNode::ArrowFunction {
                parameters: node.parameters,
                body: node.body,
                is_async: node.is_async,
                return_type: node.return_type,
            },
            Definition::TestFunction(node) => AstNode::TestFunction {
                body: node.body,
                name: node.name,
                attributes: node.attributes,
            },
            Definition::TestModule(node) => AstNode::TestModule {
                attributes: node.attributes,
                body: node.body,
                name: node.name,
            },
        }
    }
}

// ============================================================================
// STRUCTURE CONVERSIONS
// ============================================================================

impl From<BlockNode> for AstNode {
    fn from(node: BlockNode) -> Self {
        AstNode::Block {
            statements: node.statements,
            kind: node.kind,
        }
    }
}

impl From<ProgramNode> for AstNode {
    fn from(node: ProgramNode) -> Self {
        AstNode::Program {
            program_name: node.program_name,
            field_definitions: node.field_definitions,
            instruction_definitions: node.instruction_definitions,
            event_definitions: node.event_definitions,
            account_definitions: node.account_definitions,
            interface_definitions: node.interface_definitions,
            import_statements: node.import_statements,
            init_block: node.init_block,
            constraints_block: node.constraints_block,
        }
    }
}
