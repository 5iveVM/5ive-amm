use five_vm_mito::error::VMError;
use std::fmt;
use crate::ast::SourceLocation;

/// Tokenization errors
#[derive(Debug, Clone, PartialEq)]
pub enum TokenizeError {
    UnterminatedString,
    InvalidSeedSyntax(String),
    TooManySeeds,
    EmptySeeds,
    UnexpectedCharacter(char),
    InvalidNumber(String),
    UnexpectedEof,
}

impl fmt::Display for TokenizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenizeError::UnterminatedString => write!(f, "unterminated string literal"),
            TokenizeError::InvalidSeedSyntax(msg) => write!(f, "{}", msg),
            TokenizeError::TooManySeeds => write!(f, "too many seeds"),
            TokenizeError::EmptySeeds => write!(f, "expected at least one seed"),
            TokenizeError::UnexpectedCharacter(ch) => write!(f, "unexpected character '{}'", ch),
            TokenizeError::InvalidNumber(num) => write!(f, "invalid number literal '{}'", num),
            TokenizeError::UnexpectedEof => write!(f, "unexpected end of input"),
        }
    }
}

impl std::error::Error for TokenizeError {}

impl From<TokenizeError> for VMError {
    fn from(err: TokenizeError) -> Self {
        match err {
            TokenizeError::UnterminatedString | TokenizeError::UnexpectedEof => {
                VMError::UnexpectedEndOfInput
            }
            _other => VMError::InvalidScript,
        }
    }
}

macro_rules! define_token_types {
    // Base case: emit the enums and impl
    (@step { $($kind_variant:tt)* } { $($token_variant:tt)* } { $($kind_arm:tt)* }
        []
    ) => {
        /// Payload-free token kinds for efficient comparison
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        pub enum TokenKind {
            $($kind_variant)*
        }

        /// Complete token types for the Five DSL parser
        #[derive(Debug, Clone, PartialEq)]
        pub enum Token {
            $($token_variant)*
        }

        impl Token {
            pub fn kind(&self) -> TokenKind {
                match self {
                    $($kind_arm)*
                }
            }
        }
    };

    // Case 1: Variant with payload
    (@step { $($kind_variant:tt)* } { $($token_variant:tt)* } { $($kind_arm:tt)* }
        [ $(#[$meta:meta])* $name:ident ( $payload:ty ), $($rest:tt)* ]
    ) => {
        define_token_types!(@step
            { $($kind_variant)* $(#[$meta])* $name, }
            { $($token_variant)* $(#[$meta])* $name($payload), }
            { $($kind_arm)* Token::$name(_) => TokenKind::$name, }
            [ $($rest)* ]
        );
    };

    // Case 2: Variant without payload
    (@step { $($kind_variant:tt)* } { $($token_variant:tt)* } { $($kind_arm:tt)* }
        [ $(#[$meta:meta])* $name:ident, $($rest:tt)* ]
    ) => {
        define_token_types!(@step
            { $($kind_variant)* $(#[$meta])* $name, }
            { $($token_variant)* $(#[$meta])* $name, }
            { $($kind_arm)* Token::$name => TokenKind::$name, }
            [ $($rest)* ]
        );
    };

    // Entry point
    ( $($input:tt)* ) => {
        define_token_types!(@step {} {} {} [ $($input)* ]);
    };
}

define_token_types! {
    // Keywords
    Init,
    Constraints,
    Instruction,
    Use,
    Import,
    Interface,
    When,
    Event,
    Emit,
    Query,
    Return,
    If,
    Else,
    Match,
    Let,
    Mut,
    Of,
    OrInit,
    In,
    Realloc,
    Pda,
    While,
    For,
    Do,
    Break,
    Continue,
    True,
    False,
    Require,
    Error,
    As,
    Fn,
    Async,
    Script,
    Enum,
    Field,
    Pub,

    // Operators
    Plus,            // +
    PlusChecked,     // +?
    Minus,           // -
    MinusChecked,    // -?
    Star,            // *
    Multiply,        // * (alias for Star)
    MultiplyChecked, // *?
    Slash,           // /
    Divide,          // / (alias for Slash)
    Percent,         // %
    Equal,           // ==
    NotEqual,        // !=
    LT,              // <
    LE,              // <=
    LessEqual,       // <= (alias for LE)
    GT,              // >
    GE,              // >=
    GreaterEqual,    // >= (alias for GE)
    LogicalAnd,      // &&
    LogicalOr,       // ||
    Bang,            // !
    // Bitwise operators
    BitwiseAnd,        // &
    BitwiseOr,         // |
    BitwiseXor,        // ^
    BitwiseTilde,      // ~
    LeftShift,         // <<
    RightShift,        // >>
    ArithRightShift,   // >>>
    RotateLeft,        // <<<
    // Compound bitwise assignments
    LeftShiftAssign,   // <<=
    RightShiftAssign,  // >>=
    BitwiseAndAssign,  // &=
    BitwiseOrAssign,   // |=
    BitwiseXorAssign,  // ^=
    Assign,          // =
    PlusAssign,      // +=
    MinusAssign,     // -=
    MultiplyAssign,  // *=
    DivideAssign,    // /=
    Arrow,           // ->
    FatArrow,        // =>
    Question,        // ?
    DoubleColon,     // ::

    // Account attributes
    AtSigner, // @signer
    AtMut,    // @mut
    AtInit,   // @init

    // Punctuation
    At,           // @
    LeftBracket,  // [
    RightBracket, // ]
    LeftBrace,    // {
    RightBrace,   // }
    LeftParen,    // (
    RightParen,   // )
    Comma,        // ,
    Colon,        // :
    Semicolon,    // ;
    Dot,          // .
    Hash,         // # (for attributes)
    Dollar,       // $

    // Literals and identifiers
    Identifier(String),
    StringLiteral(String),
    NumberLiteral(u64),
    Type(String),

    // Account keyword
    Account,

    // Test assertions
    AssertEq,
    AssertTrue,
    AssertFalse,
    AssertFails,
    AssertApproxEq,

    // Additional missing tokens from parser
    Range,         // ..
    Some,          // Some
    None,          // None
    Ok,            // Ok
    Err,           // Err
    Result,        // Result
    Option,        // Option
    Ignore,        // ignore
    ShouldFail,    // should_fail
    Timeout,       // timeout
    Test,          // For #[test] attribute
    Discriminator, // For discriminator(N) attribute
    Serializer,    // For serializer("borsh") attribute
    DiscriminatorBytes, // For discriminator_bytes([...]) attribute

    // Control
    Eof,
}

/// Token paired with its source location for error reporting and AST position tracking
#[derive(Debug, Clone, PartialEq)]
pub struct TokenWithPos {
    pub token: Token,
    pub position: SourceLocation,
}
