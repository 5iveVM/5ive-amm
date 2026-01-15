use five_vm_mito::error::VMError;
use std::fmt;

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

/// Payload-free token kinds for efficient comparison
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TokenKind {
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
    Plus,
    PlusChecked,
    Minus,
    MinusChecked,
    Star,
    Multiply,
    MultiplyChecked,
    Slash,
    Divide,
    Percent,
    Equal,
    NotEqual,
    LT,
    LE,
    LessEqual,
    GT,
    GE,
    GreaterEqual,
    LogicalAnd,
    LogicalOr,
    Bang,
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
    Assign,
    PlusAssign,
    MinusAssign,
    MultiplyAssign,
    DivideAssign,
    Arrow,
    FatArrow,
    Question,
    DoubleColon,

    // Account attributes
    AtSigner,
    AtMut,
    AtInit,

    // Punctuation
    At,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
    LeftParen,
    RightParen,
    Comma,
    Colon,
    Semicolon,
    Dot,
    Hash,

    // Literals and identifiers (represented by their kind)
    Identifier,
    StringLiteral,
    NumberLiteral,
    Type,

    // Account keyword
    Account,

    // Test assertions
    AssertEq,
    AssertTrue,
    AssertFalse,
    AssertFails,
    AssertApproxEq,

    // Additional missing tokens from parser
    Range,
    Some,
    None,
    Ok,
    Err,
    Result,
    Option,
    Ignore,
    ShouldFail,
    Timeout,
    Test,
    Discriminator,
    Serializer,
    DiscriminatorBytes,

    // Control
    Eof,
}

/// Complete token types for the Five DSL parser
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
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

impl Token {
    pub fn kind(&self) -> TokenKind {
        match self {
            // Keywords
            Token::Init => TokenKind::Init,
            Token::Constraints => TokenKind::Constraints,
            Token::Instruction => TokenKind::Instruction,
            Token::Use => TokenKind::Use,
            Token::Import => TokenKind::Use,
            Token::Interface => TokenKind::Interface,
            Token::When => TokenKind::When,
            Token::Event => TokenKind::Event,
            Token::Emit => TokenKind::Emit,
            Token::Query => TokenKind::Query,
            Token::Return => TokenKind::Return,
            Token::If => TokenKind::If,
            Token::Else => TokenKind::Else,
            Token::Match => TokenKind::Match,
            Token::Let => TokenKind::Let,
            Token::Mut => TokenKind::Mut,
            Token::Of => TokenKind::Of,
            Token::OrInit => TokenKind::OrInit,
            Token::In => TokenKind::In,
            Token::Realloc => TokenKind::Realloc,
            Token::Pda => TokenKind::Pda,
            Token::While => TokenKind::While,
            Token::For => TokenKind::For,
            Token::Do => TokenKind::Do,
            Token::Break => TokenKind::Break,
            Token::Continue => TokenKind::Continue,
            Token::True => TokenKind::True,
            Token::False => TokenKind::False,
            Token::Require => TokenKind::Require,
            Token::Error => TokenKind::Error,
            Token::As => TokenKind::As,
            Token::Fn => TokenKind::Fn,
            Token::Async => TokenKind::Async,
            Token::Script => TokenKind::Script,
            Token::Enum => TokenKind::Enum,
            Token::Field => TokenKind::Field,
            Token::Pub => TokenKind::Pub,

            // Operators
            Token::Plus => TokenKind::Plus,
            Token::PlusChecked => TokenKind::PlusChecked,
            Token::Minus => TokenKind::Minus,
            Token::MinusChecked => TokenKind::MinusChecked,
            Token::Star => TokenKind::Star,
            Token::Multiply => TokenKind::Multiply,
            Token::MultiplyChecked => TokenKind::MultiplyChecked,
            Token::Slash => TokenKind::Slash,
            Token::Divide => TokenKind::Divide,
            Token::Percent => TokenKind::Percent,
            Token::Equal => TokenKind::Equal,
            Token::NotEqual => TokenKind::NotEqual,
            Token::LT => TokenKind::LT,
            Token::LE => TokenKind::LE,
            Token::LessEqual => TokenKind::LessEqual,
            Token::GT => TokenKind::GT,
            Token::GE => TokenKind::GE,
            Token::GreaterEqual => TokenKind::GreaterEqual,
            Token::LogicalAnd => TokenKind::LogicalAnd,
            Token::LogicalOr => TokenKind::LogicalOr,
            Token::Bang => TokenKind::Bang,
            // Bitwise operators
            Token::BitwiseAnd => TokenKind::BitwiseAnd,
            Token::BitwiseOr => TokenKind::BitwiseOr,
            Token::BitwiseXor => TokenKind::BitwiseXor,
            Token::BitwiseTilde => TokenKind::BitwiseTilde,
            Token::LeftShift => TokenKind::LeftShift,
            Token::RightShift => TokenKind::RightShift,
            Token::ArithRightShift => TokenKind::ArithRightShift,
            Token::RotateLeft => TokenKind::RotateLeft,
            // Compound bitwise assignments
            Token::LeftShiftAssign => TokenKind::LeftShiftAssign,
            Token::RightShiftAssign => TokenKind::RightShiftAssign,
            Token::BitwiseAndAssign => TokenKind::BitwiseAndAssign,
            Token::BitwiseOrAssign => TokenKind::BitwiseOrAssign,
            Token::BitwiseXorAssign => TokenKind::BitwiseXorAssign,
            Token::Assign => TokenKind::Assign,
            Token::PlusAssign => TokenKind::PlusAssign,
            Token::MinusAssign => TokenKind::MinusAssign,
            Token::MultiplyAssign => TokenKind::MultiplyAssign,
            Token::DivideAssign => TokenKind::DivideAssign,
            Token::Arrow => TokenKind::Arrow,
            Token::FatArrow => TokenKind::FatArrow,
            Token::Question => TokenKind::Question,
            Token::DoubleColon => TokenKind::DoubleColon,

            // Account attributes
            Token::AtSigner => TokenKind::AtSigner,
            Token::AtMut => TokenKind::AtMut,
            Token::AtInit => TokenKind::AtInit,

            // Punctuation
            Token::At => TokenKind::At,
            Token::LeftBracket => TokenKind::LeftBracket,
            Token::RightBracket => TokenKind::RightBracket,
            Token::LeftBrace => TokenKind::LeftBrace,
            Token::RightBrace => TokenKind::RightBrace,
            Token::LeftParen => TokenKind::LeftParen,
            Token::RightParen => TokenKind::RightParen,
            Token::Comma => TokenKind::Comma,
            Token::Colon => TokenKind::Colon,
            Token::Semicolon => TokenKind::Semicolon,
            Token::Dot => TokenKind::Dot,
            Token::Hash => TokenKind::Hash,

            // Literals and identifiers (represented by their kind)
            Token::Identifier(_) => TokenKind::Identifier,
            Token::StringLiteral(_) => TokenKind::StringLiteral,
            Token::NumberLiteral(_) => TokenKind::NumberLiteral,
            Token::Type(_) => TokenKind::Type,

            // Account keyword
            Token::Account => TokenKind::Account,

            // Test assertions
            Token::AssertEq => TokenKind::AssertEq,
            Token::AssertTrue => TokenKind::AssertTrue,
            Token::AssertFalse => TokenKind::AssertFalse,
            Token::AssertFails => TokenKind::AssertFails,
            Token::AssertApproxEq => TokenKind::AssertApproxEq,

            // Additional missing tokens from parser
            Token::Range => TokenKind::Range,
            Token::Some => TokenKind::Some,
            Token::None => TokenKind::None,
            Token::Ok => TokenKind::Ok,
            Token::Err => TokenKind::Err,
            Token::Result => TokenKind::Result,
            Token::Option => TokenKind::Option,
            Token::Ignore => TokenKind::Ignore,
            Token::ShouldFail => TokenKind::ShouldFail,
            Token::Timeout => TokenKind::Timeout,
            Token::Test => TokenKind::Test,
            Token::Discriminator => TokenKind::Discriminator,
            Token::Serializer => TokenKind::Serializer,
            Token::DiscriminatorBytes => TokenKind::DiscriminatorBytes,

            // Control
            Token::Eof => TokenKind::Eof,
        }
    }
}
