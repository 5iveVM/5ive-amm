// AST (Abstract Syntax Tree) Module
//
// Defines the AST node types for parsed .five syntax.
// Grammar rules for each AST node are defined in the single-source-of-truth metadata file.
// These are used by the `generate-grammar` CLI tool to auto-generate grammar.js

use five_protocol::Value;
use serde::{Serialize, Deserialize};

pub mod registry;
pub mod generated;
pub mod conversions;

// Re-export registry for public API
pub use registry::{NodeRegistry, NODE_REGISTRY, RegistryError};

// Re-export generated AST structures for forward compatibility
// These are type-safe versions that can be converted to/from the original AstNode enum
pub use generated::{Expression, Statement, Definition};

// Note: Conversions module provides From/Into implementations for compatibility

/// Source location for error reporting and AST traversal
///
/// Tracks the position of an AST node in the source code for:
/// - Accurate error messages with line/column information
/// - Finding nodes at cursor position (for hover, go-to-def, etc.)
/// - Scope-aware symbol resolution with position information
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct SourceLocation {
    /// 0-indexed line number
    pub line: u32,
    /// 0-indexed column (character) number
    pub column: u32,
    /// Length of the token/span in characters
    pub length: u32,
}

impl SourceLocation {
    /// Create a new source location
    pub fn new(line: u32, column: u32, length: u32) -> Self {
        Self { line, column, length }
    }

    /// Get the end column of this span
    pub fn end_column(&self) -> u32 {
        self.column.saturating_add(self.length)
    }

    /// Check if this location contains a given position (for find-at-position)
    pub fn contains(&self, line: u32, column: u32) -> bool {
        line == self.line && column >= self.column && column < self.end_column()
    }
}

/// Different kinds of code blocks
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BlockKind {
    Regular,
    Init,
    Constraints,
}

/// Visibility modifier for cross-module access control
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[derive(Default)]
pub enum Visibility {
    /// `pub` - on-chain callable, exported for imports
    Public,
    /// No modifier - module-internal, can be imported but not on-chain callable
    #[default]
    Internal,
    /// Private to module only (for future use, not in syntax yet)
    Private,
}

impl Visibility {
    /// Check if this symbol can be imported by other modules
    pub fn is_importable(&self) -> bool {
        matches!(self, Visibility::Public | Visibility::Internal)
    }

    /// Check if this is on-chain callable
    pub fn is_on_chain_callable(&self) -> bool {
        matches!(self, Visibility::Public)
    }
}


/// AST node types for parsed .stacks syntax
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AstNode {
    Program {
        program_name: String,
        field_definitions: Vec<AstNode>,
        instruction_definitions: Vec<AstNode>,
        event_definitions: Vec<AstNode>,
        account_definitions: Vec<AstNode>,
        interface_definitions: Vec<AstNode>,
        import_statements: Vec<AstNode>,
        init_block: Option<Box<AstNode>>,
        constraints_block: Option<Box<AstNode>>,
    },
    Block {
        statements: Vec<AstNode>,
        kind: BlockKind,
    },
    Assignment {
        target: String,
        value: Box<AstNode>,
    },
    FieldAssignment {
        object: Box<AstNode>,
        field: String,
        value: Box<AstNode>,
    },
    RequireStatement {
        condition: Box<AstNode>,
    },
    MethodCall {
        object: Box<AstNode>,
        method: String,
        args: Vec<AstNode>,
    },
    FieldDefinition {
        name: String,
        field_type: Box<TypeNode>,
        is_mutable: bool,
        is_optional: bool,
        default_value: Option<Box<AstNode>>,
        visibility: Visibility, // pub for cross-module access
    },
    LetStatement {
        name: String,
        type_annotation: Option<Box<TypeNode>>,
        is_mutable: bool,
        value: Box<AstNode>,
    },
    TupleDestructuring {
        targets: Vec<String>,
        value: Box<AstNode>,
    },
    TupleAssignment {
        targets: Vec<AstNode>, // Can be Identifier or FieldAccess
        value: Box<AstNode>,
    },
    InstructionDefinition {
        name: String,
        parameters: Vec<InstructionParameter>,
        return_type: Option<Box<TypeNode>>,
        body: Box<AstNode>,
        visibility: Visibility, // pub = on-chain callable, otherwise module-internal/importable
        // Deprecated: is_public kept for backwards compatibility, but visibility should be used
        #[deprecated(since = "0.2.0", note = "use visibility field instead")]
        is_public: bool,
    },
    // Function call statement
    FunctionCall {
        name: String,
        args: Vec<AstNode>,
    },
    EventDefinition {
        name: String,
        fields: Vec<StructField>,
        visibility: Visibility, // pub for cross-module access
    },
    EmitStatement {
        event_name: String,
        fields: Vec<EventFieldAssignment>,
    },

    IfStatement {
        condition: Box<AstNode>,
        then_branch: Box<AstNode>,
        else_branch: Option<Box<AstNode>>,
    },
    MatchExpression {
        expression: Box<AstNode>,
        arms: Vec<MatchArm>,
    },
    ReturnStatement {
        value: Option<Box<AstNode>>,
    },

    ErrorTypeDefinition {
        name: String,
        variants: Vec<ErrorVariant>,
    },

    // Account system: Account type definitions
    AccountDefinition {
        name: String,
        fields: Vec<StructField>,
        visibility: Visibility, // pub for cross-module access
    },

    StructLiteral {
        fields: Vec<StructLiteralField>,
    },
    ArrayLiteral {
        elements: Vec<AstNode>,
    },
    StringLiteral {
        value: String,
    },
    TupleLiteral {
        elements: Vec<AstNode>,
    },
    FieldAccess {
        object: Box<AstNode>,
        field: String,
    },
    TupleAccess {
        object: Box<AstNode>,
        index: u32,
    },
    ArrayAccess {
        array: Box<AstNode>,
        index: Box<AstNode>,
    },

    // Enum variant access: VaultError::InsufficientFunds
    EnumVariantAccess {
        enum_name: String,
        variant_name: String,
    },

    // Error propagation operator: expression?
    ErrorPropagation {
        expression: Box<AstNode>,
    },

    // Basic template literal (kept for log compilation)
    TemplateLiteral {
        parts: Vec<AstNode>, // Alternating strings and expressions
    },

    // Unary expressions: !expr, -expr, +expr
    UnaryExpression {
        operator: String,
        operand: Box<AstNode>,
    },

    // Binary expressions: a + b, a == b, a != b, etc.
    BinaryExpression {
        operator: String,
        left: Box<AstNode>,
        right: Box<AstNode>,
    },

    // Advanced control flow
    ForLoop {
        init: Option<Box<AstNode>>,
        condition: Option<Box<AstNode>>,
        update: Option<Box<AstNode>>,
        body: Box<AstNode>,
    },
    ForInLoop {
        variable: String,
        iterable: Box<AstNode>,
        body: Box<AstNode>,
    },
    ForOfLoop {
        variable: String,
        iterable: Box<AstNode>,
        body: Box<AstNode>,
    },
    WhileLoop {
        condition: Box<AstNode>,
        body: Box<AstNode>,
    },
    DoWhileLoop {
        body: Box<AstNode>,
        condition: Box<AstNode>,
    },
    SwitchStatement {
        discriminant: Box<AstNode>,
        cases: Vec<SwitchCase>,
        default_case: Option<Box<AstNode>>,
    },
    BreakStatement {
        label: Option<String>,
    },
    ContinueStatement {
        label: Option<String>,
    },

    // Function types
    ArrowFunction {
        parameters: Vec<InstructionParameter>,
        return_type: Option<Box<TypeNode>>,
        body: Box<AstNode>,
        is_async: bool,
    },

    // Testing system AST nodes
    TestFunction {
        name: String,
        attributes: Vec<TestAttribute>,
        body: Box<AstNode>,
    },
    TestModule {
        name: String,
        attributes: Vec<TestAttribute>,
        body: Box<AstNode>,
    },
    AssertStatement {
        assertion_type: AssertionType,
        args: Vec<AstNode>,
    },

    // Interface system AST nodes
    InterfaceDefinition {
        name: String,
        program_id: Option<String>, // Optional custom program ID
        serializer: Option<String>, // Optional serializer hint (raw, borsh, bincode)
        functions: Vec<AstNode>,
    },
    InterfaceFunction {
        name: String,
        parameters: Vec<InstructionParameter>,
        return_type: Option<Box<TypeNode>>,
        discriminator: Option<u8>, // Custom discriminator for the instruction
        discriminator_bytes: Option<Vec<u8>>, // Optional multi-byte discriminator
    },

    // Import system AST nodes
    ImportStatement {
        module_specifier: ModuleSpecifier,
        imported_items: Option<Vec<String>>, // Specific functions to import (None = all)
    },

    Identifier(String),
    Literal(Value),
}

/// Module specifier for use/import statements
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ModuleSpecifier {
    Local(String),              // use lib; or import lib;
    Nested(Vec<String>),        // use utils::helpers;
    External(String),           // use "0x123"::{fn1, fn2}; (contract address or PDA seeds)
}

/// Type system AST nodes for Rust+TypeScript hybrid syntax
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TypeNode {
    // Primitive types
    Primitive(String), // u64, i32, bool, string, pubkey, etc

    // Rust-style generics: Option<T>, Result<T, E>
    Generic {
        base: String,
        args: Vec<TypeNode>,
    },

    // Arrays: [T; N] (Rust style) or T[N] (TS style)
    Array {
        element_type: Box<TypeNode>,
        size: Option<u64>,
    },

    // Tuples: (T1, T2, T3)
    Tuple {
        elements: Vec<TypeNode>,
    },

    // Struct types: { field1: T1, field2: T2 }
    Struct {
        fields: Vec<StructField>,
    },

    // Sized types: string<32>
    Sized {
        base_type: String,
        size: u64,
    },

    // Union types: T | U (TypeScript style)
    Union {
        types: Vec<TypeNode>,
    },

    // Built-in Account type with implicit properties (lamports, owner, key, data)
    Account,

    // Custom/Named types
    Named(String),
}

/// Struct field definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructField {
    pub name: String,
    pub field_type: TypeNode,
    pub is_mutable: bool,
    pub is_optional: bool,
}

/// Account initialization configuration for @init constraints
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InitConfig {
    pub seeds: Option<Vec<AstNode>>, // PDA seeds like ["vault", user.key, 42]
    pub bump: Option<String>,        // Auto-generated bump variable name
    pub space: Option<u64>,          // Account size in bytes (auto-calculated)
    pub payer: Option<String>,       // Explicit payer account name for rent
}

/// Attribute definition for parameters and functions (e.g., @requires(x > 0))
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Attribute {
    pub name: String,
    pub args: Vec<AstNode>,
}

/// Instruction parameter definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InstructionParameter {
    pub name: String,
    pub param_type: TypeNode,
    pub is_optional: bool,
    pub default_value: Option<Box<AstNode>>,
    pub attributes: Vec<Attribute>, // Generalized attributes
    pub is_init: bool,           // True if @init constraint is applied
    pub init_config: Option<InitConfig>, // Configuration for account initialization
}

/// Event field assignment for emit statements
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventFieldAssignment {
    pub field_name: String,
    pub value: Box<AstNode>,
}

/// Match arm for match expressions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchArm {
    pub pattern: Box<AstNode>,
    /// Optional guard expression following the pattern (`if <expr>`)
    pub guard: Option<Box<AstNode>>,
    pub body: Box<AstNode>,
}

/// Error variant for custom error types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ErrorVariant {
    pub name: String,
    pub fields: Vec<StructField>, // For structured error data
}

/// Struct literal field assignment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructLiteralField {
    pub field_name: String,
    pub value: Box<AstNode>,
}

/// Switch case for switch statements
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SwitchCase {
    pub pattern: Box<AstNode>,
    pub body: Vec<AstNode>,
}

/// Test assertion types for unit testing
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AssertionType {
    Equal,    // assert_eq(actual, expected)
    True,     // assert_true(condition)
    False,    // assert_false(condition)
    Fails,    // assert_fails(function_call)
    ApproxEq, // assert_approx_eq(actual, expected, delta)
}

/// Test attribute for test functions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TestAttribute {
    pub name: String,
    pub args: Vec<AstNode>,
}
