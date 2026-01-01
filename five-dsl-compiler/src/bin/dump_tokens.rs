use std::env;
use std::fs;
use std::process;

use five_dsl_compiler::tokenizer::{DslTokenizer, Token};
use five_vm_mito::error::VMError;

/// Simple utility to print a readable dump of tokens produced by the DSL tokenizer.
///
/// Usage:
///   cargo run --bin dump_tokens -- path/to/source.v
fn main() {
    let mut args = env::args().skip(1);

    let file_path = match args.next() {
        Some(p) => p,
        None => {
            eprintln!("Usage: dump_tokens <source-file.v>");
            process::exit(2);
        }
    };

    let source = match fs::read_to_string(&file_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to read '{}': {}", file_path, e);
            process::exit(3);
        }
    };

    match dump_tokens(&source) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Tokenizer error: {:?}", e);
            process::exit(4);
        }
    }
}

/// Tokenize the given source and print each token on its own line with useful detail.
fn dump_tokens(source: &str) -> Result<(), VMError> {
    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize()?;

    println!("Token count: {}", tokens.len());
    for (i, token) in tokens.iter().enumerate() {
        println!("{:04}: {}", i + 1, format_token(token));
    }

    Ok(())
}

/// Format token with readable representation.
/// For tokens that carry data (identifiers, literals, types, strings), include the payload.
fn format_token(token: &Token) -> String {
    match token {
        Token::Init => "Init".into(),
        Token::Constraints => "Constraints".into(),
        Token::Instruction => "Instruction".into(),
        Token::Use => "Use".into(),
        Token::Import => "Import".into(),
        Token::Interface => "Interface".into(),
        Token::When => "When".into(),
        Token::Event => "Event".into(),
        Token::Emit => "Emit".into(),
        Token::Query => "Query".into(),
        Token::Return => "Return".into(),
        Token::If => "If".into(),
        Token::Else => "Else".into(),
        Token::Match => "Match".into(),
        Token::Let => "Let".into(),
        Token::Mut => "Mut".into(),
        Token::Of => "Of".into(),
        Token::OrInit => "OrInit".into(),
        Token::In => "In".into(),
        Token::Realloc => "Realloc".into(),
        Token::Pda => "Pda".into(),
        Token::While => "While".into(),
        Token::For => "For".into(),
        Token::Do => "Do".into(),
        Token::Break => "Break".into(),
        Token::Continue => "Continue".into(),
        Token::True => "True".into(),
        Token::False => "False".into(),
        Token::Require => "Require".into(),
        Token::Error => "Error".into(),
        Token::As => "As".into(),
        Token::Fn => "Fn".into(),
        Token::Async => "Async".into(),
        Token::Script => "Script".into(),
        Token::Enum => "Enum".into(),
        Token::Field => "Field".into(),
        Token::Pub => "Pub".into(),

        // Operators (no payload)
        Token::Plus => "+".into(),
        Token::PlusChecked => "+?".into(),
        Token::Minus => "-".into(),
        Token::MinusChecked => "-?".into(),
        Token::Star => "*".into(),
        Token::Multiply => "*".into(),
        Token::MultiplyChecked => "*?".into(),
        Token::Slash => "/".into(),
        Token::Divide => "/".into(),
        Token::Percent => "%".into(),
        Token::Equal => "==".into(),
        Token::NotEqual => "!=".into(),
        Token::LT => "<".into(),
        Token::LE => "<=".into(),
        Token::LessEqual => "<=".into(),
        Token::GT => ">".into(),
        Token::GE => ">=".into(),
        Token::GreaterEqual => ">=".into(),
        Token::LogicalAnd => "&&".into(),
        Token::LogicalOr => "||".into(),
        Token::Bang => "!".into(),
        Token::Assign => "=".into(),
        Token::PlusAssign => "+=".into(),
        Token::MinusAssign => "-=".into(),
        Token::MultiplyAssign => "*=".into(),
        Token::DivideAssign => "/=".into(),
        Token::Arrow => "->".into(),
        Token::FatArrow => "=>".into(),
        Token::Question => "?".into(),
        Token::DoubleColon => "::".into(),

        // Account attributes
        Token::AtSigner => "@signer".into(),
        Token::AtMut => "@mut".into(),
        Token::AtInit => "@init".into(),

        // Punctuation
        Token::At => "@".into(),
        Token::LeftBracket => "[".into(),
        Token::RightBracket => "]".into(),
        Token::LeftBrace => "{".into(),
        Token::RightBrace => "}".into(),
        Token::LeftParen => "(".into(),
        Token::RightParen => ")".into(),
        Token::Comma => ",".into(),
        Token::Colon => ":".into(),
        Token::Semicolon => ";".into(),
        Token::Dot => ".".into(),
        Token::Hash => "#".into(),

        // Data carrying tokens
        Token::Identifier(name) => format!("Identifier(\"{}\")", name),
        Token::StringLiteral(s) => format!("StringLiteral(\"{}\")", s),
        Token::NumberLiteral(n) => format!("NumberLiteral({})", n),
        Token::Type(t) => format!("Type(\"{}\")", t),

        // Account keyword
        Token::Account => "Account".into(),

        // Test assertions and additional tokens
        Token::AssertEq => "AssertEq".into(),
        Token::AssertTrue => "AssertTrue".into(),
        Token::AssertFalse => "AssertFalse".into(),
        Token::AssertFails => "AssertFails".into(),
        Token::AssertApproxEq => "AssertApproxEq".into(),

        // Additional
        Token::Range => "..".into(),
        Token::Some => "Some".into(),
        Token::None => "None".into(),
        Token::Ok => "Ok".into(),
        Token::Err => "Err".into(),
        Token::Result => "Result".into(),
        Token::Option => "Option".into(),
        Token::Ignore => "Ignore".into(),
        Token::ShouldFail => "ShouldFail".into(),
        Token::Timeout => "Timeout".into(),
        Token::Test => "Test".into(),
        Token::Discriminator => "Discriminator".into(),

        Token::Eof => "Eof".into(),

        // Fallback debug representation (in case new variants added)
        other => format!("{:?}", other),
    }
}
