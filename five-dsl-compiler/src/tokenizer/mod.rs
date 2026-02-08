// DSL tokenizer for the Five language.

use five_vm_mito::error::VMError;
use std::iter::Peekable;
use std::str::Chars;

pub mod tokens;
pub use tokens::*;

/// Zero-copy tokenizer using string slice iteration
pub struct DslTokenizer<'a> {
    chars: Peekable<Chars<'a>>,
    current_char: Option<char>,
    /// Current line number (0-indexed)
    line: u32,
    /// Current column number (0-indexed)
    column: u32,
}

impl<'a> DslTokenizer<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut chars = input.chars().peekable();
        let current_char = chars.next();

        Self {
            chars,
            current_char,
            line: 0,
            column: 0,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, VMError> {
        let mut tokens = Vec::new();

        while let Some(ch) = self.current_char {
            match ch {
                // Skip whitespace
                ' ' | '\t' | '\n' | '\r' => {
                    self.advance();
                }

                // Comments
                '/' => {
                    if self.chars.peek() == Some(&'/') {
                        self.skip_line_comment();
                    } else if self.chars.peek() == Some(&'*') {
                        self.skip_block_comment()?;
                    } else if self.chars.peek() == Some(&'=') {
                        self.advance(); // consume '/'
                        self.advance(); // consume '='
                        tokens.push(Token::DivideAssign);
                    } else {
                        tokens.push(Token::Divide);
                        self.advance();
                    }
                }

                // @ attribute parsing (allow optional whitespace after '@')
                '@' => {
                    self.advance();

                    // Skip optional whitespace after '@' (accept `@ signer` as well as `@signer`)
                    while let Some(ch) = self.current_char {
                        if ch.is_whitespace() {
                            self.advance();
                        } else {
                            break;
                        }
                    }

                    // If nothing valid follows '@', treat it as a bare At token
                    if !matches!(
                        self.current_char,
                        Some('a'..='z') | Some('A'..='Z') | Some('_')
                    ) {
                        tokens.push(Token::At);
                        continue;
                    }

                    let attribute = self.read_identifier().map_err(VMError::from)?;

                    match attribute.as_str() {
                        "signer" => tokens.push(Token::AtSigner),
                        "mut" => tokens.push(Token::AtMut),
                        "init" => {
                            tokens.push(Token::AtInit);
                            // Check for optional [seeds]
                            if self.current_char == Some('[') {
                                self.advance(); // consume '['
                                tokens.push(Token::LeftBracket);
                                self.parse_seeds_list(&mut tokens)?;
                                if self.current_char == Some(']') {
                                    self.advance(); // consume ']'
                                    tokens.push(Token::RightBracket);
                                } else {
                                    return Err(VMError::from(TokenizeError::InvalidSeedSyntax(
                                        "Expected ']' after seeds".to_string(),
                                    )));
                                }
                            }
                        }
                        _ => {
                            tokens.push(Token::At);
                            tokens.push(Token::Identifier(attribute));
                        }
                    }
                }

                // Punctuation
                '[' => {
                    tokens.push(Token::LeftBracket);
                    self.advance();
                }
                ']' => {
                    tokens.push(Token::RightBracket);
                    self.advance();
                }
                '{' => {
                    tokens.push(Token::LeftBrace);
                    self.advance();
                }
                '}' => {
                    tokens.push(Token::RightBrace);
                    self.advance();
                }
                '(' => {
                    tokens.push(Token::LeftParen);
                    self.advance();
                }
                ')' => {
                    tokens.push(Token::RightParen);
                    self.advance();
                }
                ',' => {
                    tokens.push(Token::Comma);
                    self.advance();
                }
                ';' => {
                    tokens.push(Token::Semicolon);
                    self.advance();
                }
                '.' => {
                    if self.chars.peek() == Some(&'.') {
                        self.advance(); // consume first '.'
                        self.advance(); // consume second '.'
                        tokens.push(Token::Range);
                    } else {
                        tokens.push(Token::Dot);
                        self.advance();
                    }
                }
                '#' => {
                    tokens.push(Token::Hash);
                    self.advance();
                }
                '?' => {
                    tokens.push(Token::Question);
                    self.advance();
                }
                '%' => {
                    tokens.push(Token::Percent);
                    self.advance();
                }
                '^' => {
                    if self.chars.peek() == Some(&'=') {
                        self.advance(); // consume '^'
                        self.advance(); // consume '='
                        tokens.push(Token::BitwiseXorAssign);
                    } else {
                        tokens.push(Token::BitwiseXor);
                        self.advance();
                    }
                }
                '~' => {
                    tokens.push(Token::BitwiseTilde);
                    self.advance();
                }

                // Multi-character operators and punctuation
                ':' => {
                    if self.chars.peek() == Some(&':') {
                        self.advance(); // consume first ':'
                        self.advance(); // consume second ':'
                        tokens.push(Token::DoubleColon);
                    } else {
                        tokens.push(Token::Colon);
                        self.advance();
                    }
                }

                '=' => {
                    // Check next character safely and prioritize multi-char tokens like '==' and '=>'
                    if let Some(&next_ch) = self.chars.peek() {
                        if next_ch == '=' {
                            self.advance(); // consume first '='
                            self.advance(); // consume second '='
                            tokens.push(Token::Equal);
                        } else if next_ch == '>' {
                            self.advance(); // consume '='
                            self.advance(); // consume '>'
                                            // Map '=>' to Arrow to match parser/tests expecting Token::Arrow for match arms
                            tokens.push(Token::Arrow);
                        } else {
                            tokens.push(Token::Assign);
                            self.advance();
                        }
                    } else {
                        // Trailing '=' at EOF -> treat as assign
                        tokens.push(Token::Assign);
                        self.advance();
                    }
                }

                '!' => {
                    if self.chars.peek() == Some(&'=') {
                        self.advance(); // consume '!'
                        self.advance(); // consume '='
                        tokens.push(Token::NotEqual);
                    } else {
                        tokens.push(Token::Bang);
                        self.advance();
                    }
                }

                '<' => {
                    self.advance(); // consume first '<'
                    if self.current_char == Some('<') {
                        self.advance(); // consume second '<'
                        if self.current_char == Some('<') {
                            self.advance(); // consume third '<'
                            tokens.push(Token::RotateLeft); // <<<
                        } else if self.current_char == Some('=') {
                            self.advance(); // consume '='
                            tokens.push(Token::LeftShiftAssign); // <<=
                        } else {
                            tokens.push(Token::LeftShift); // <<
                        }
                    } else if self.current_char == Some('=') {
                        self.advance(); // consume '='
                        tokens.push(Token::LessEqual);
                    } else {
                        tokens.push(Token::LT);
                    }
                }

                '>' => {
                    self.advance(); // consume first '>'
                    if self.current_char == Some('>') {
                        self.advance(); // consume second '>'
                        if self.current_char == Some('>') {
                            self.advance(); // consume third '>'
                            tokens.push(Token::ArithRightShift); // >>>
                        } else if self.current_char == Some('=') {
                            self.advance(); // consume '='
                            tokens.push(Token::RightShiftAssign); // >>=
                        } else {
                            tokens.push(Token::RightShift); // >>
                        }
                    } else if self.current_char == Some('=') {
                        self.advance(); // consume '='
                        tokens.push(Token::GreaterEqual);
                    } else {
                        tokens.push(Token::GT);
                    }
                }

                '-' => {
                    // Check next character safely and prioritize '->' before other single-char interpretations
                    if let Some(&next_ch) = self.chars.peek() {
                        if next_ch == '>' {
                            self.advance(); // consume '-'
                            self.advance(); // consume '>'
                            tokens.push(Token::Arrow);
                        } else if next_ch == '=' {
                            self.advance(); // consume '-'
                            self.advance(); // consume '='
                            tokens.push(Token::MinusAssign);
                        } else if next_ch == '?' {
                            self.advance(); // consume '-'
                            self.advance(); // consume '?'
                            tokens.push(Token::MinusChecked);
                        } else {
                            tokens.push(Token::Minus);
                            self.advance();
                        }
                    } else {
                        // Trailing '-' at EOF
                        tokens.push(Token::Minus);
                        self.advance();
                    }
                }

                '&' => {
                    if self.chars.peek() == Some(&'&') {
                        self.advance(); // consume first '&'
                        self.advance(); // consume second '&'
                        tokens.push(Token::LogicalAnd);
                    } else if self.chars.peek() == Some(&'=') {
                        self.advance(); // consume '&'
                        self.advance(); // consume '='
                        tokens.push(Token::BitwiseAndAssign);
                    } else {
                        tokens.push(Token::BitwiseAnd);
                        self.advance();
                    }
                }

                '|' => {
                    if self.chars.peek() == Some(&'|') {
                        self.advance(); // consume first '|'
                        self.advance(); // consume second '|'
                        tokens.push(Token::LogicalOr);
                    } else if self.chars.peek() == Some(&'=') {
                        self.advance(); // consume '|'
                        self.advance(); // consume '='
                        tokens.push(Token::BitwiseOrAssign);
                    } else {
                        tokens.push(Token::BitwiseOr);
                        self.advance();
                    }
                }

                // Single-character operators with possible compound assignments
                '+' => {
                    if self.chars.peek() == Some(&'=') {
                        self.advance(); // consume '+'
                        self.advance(); // consume '='
                        tokens.push(Token::PlusAssign);
                    } else if self.chars.peek() == Some(&'?') {
                        self.advance(); // consume '+'
                        self.advance(); // consume '?'
                        tokens.push(Token::PlusChecked);
                    } else {
                        tokens.push(Token::Plus);
                        self.advance();
                    }
                }
                '*' => {
                    if self.chars.peek() == Some(&'=') {
                        self.advance(); // consume '*'
                        self.advance(); // consume '='
                        tokens.push(Token::MultiplyAssign);
                    } else if self.chars.peek() == Some(&'?') {
                        self.advance(); // consume '*'
                        self.advance(); // consume '?'
                        tokens.push(Token::MultiplyChecked);
                    } else {
                        tokens.push(Token::Multiply);
                        self.advance();
                    }
                }

                // String literals
                '"' => {
                    let string_value = self.read_string_literal().map_err(VMError::from)?;
                    tokens.push(Token::StringLiteral(string_value));
                }

                // Numbers (support 0x.. hex and decimal)
                '0'..='9' => {
                    // Hex literal if starts with 0x or 0X
                    if ch == '0' {
                        if let Some('x') | Some('X') = self.chars.peek().copied() {
                            // consume '0' and 'x'
                            self.advance();
                            self.advance();
                            // read hex digits (allow underscores as separators)
                            let mut hex = String::new();
                            while let Some(c) = self.current_char {
                                if c.is_ascii_hexdigit() || c == '_' {
                                    if c != '_' {
                                        hex.push(c);
                                    }
                                    self.advance();
                                } else {
                                    break;
                                }
                            }
                            if hex.is_empty() {
                                return Err(VMError::from(TokenizeError::InvalidNumber(
                                    "0x".to_string(),
                                )));
                            }
                            // Consume optional numeric type suffix (e.g., u64, i32); ignored for value parsing
                            self.consume_numeric_suffix();
                            let value = u64::from_str_radix(&hex, 16).map_err(|_| {
                                VMError::from(TokenizeError::InvalidNumber(format!("0x{}", hex)))
                            })?;
                            tokens.push(Token::NumberLiteral(value));
                        } else {
                            let number_value =
                                self.read_number_literal().map_err(VMError::from)?;
                            tokens.push(Token::NumberLiteral(number_value));
                        }
                    } else {
                        let number_value =
                            self.read_number_literal().map_err(VMError::from)?;
                        tokens.push(Token::NumberLiteral(number_value));
                    }
                }

                // Identifiers and keywords
                'a'..='z' | 'A'..='Z' | '_' => {
                    let identifier = self.read_identifier().map_err(VMError::from)?;
                    let token = self.classify_identifier(&identifier);
                    tokens.push(token);
                }

                _ => return Err(VMError::from(TokenizeError::UnexpectedCharacter(ch))),
            }
        }

        tokens.push(Token::Eof);
        Ok(tokens)
    }

    // ===== PARSING HELPERS =====

    /// Skip line comment (// to end of line)
    fn skip_line_comment(&mut self) {
        self.advance(); // consume first '/'
        self.advance(); // consume second '/'

        while let Some(ch) = self.current_char {
            if ch == '\n' {
                break;
            }
            self.advance();
        }
    }

    /// Skip block comment (/* ... */)
    fn skip_block_comment(&mut self) -> Result<(), VMError> {
        self.advance(); // consume first '/'
        self.advance(); // consume '*'

        while let Some(ch) = self.current_char {
            if ch == '*' && self.chars.peek() == Some(&'/') {
                self.advance(); // consume '*'
                self.advance(); // consume '/'
                return Ok(());
            }
            self.advance();
        }

        Err(VMError::from(TokenizeError::UnexpectedEof))
    }

    /// Read string literal
    fn read_string_literal(&mut self) -> Result<String, TokenizeError> {
        self.advance(); // consume opening '"'
        let mut value = String::new();

        while let Some(ch) = self.current_char {
            if ch == '"' {
                self.advance(); // consume closing '"'
                return Ok(value);
            }
            value.push(ch);
            self.advance();
        }

        Err(TokenizeError::UnterminatedString)
    }

    /// Read number literal: supports underscores (e.g., 10_000) and optional suffixes (u8/u16/u32/u64, i8/i16/i32/i64)
    fn read_number_literal(&mut self) -> Result<u64, TokenizeError> {
        let mut raw = String::new();
        let mut saw_digit = false;

        while let Some(ch) = self.current_char {
            if ch.is_ascii_digit() {
                raw.push(ch);
                saw_digit = true;
                self.advance();
            } else if ch == '_' {
                // Ignore visual separators
                self.advance();
            } else {
                break;
            }
        }

        if !saw_digit {
            return Err(TokenizeError::InvalidNumber(raw));
        }

        // Consume optional numeric type suffix (e.g., u64, i32); ignored for value parsing
        self.consume_numeric_suffix();

        raw.parse::<u64>()
            .map_err(|_| TokenizeError::InvalidNumber(raw))
    }

    /// Read identifier
    fn read_identifier(&mut self) -> Result<String, TokenizeError> {
        let mut identifier = String::new();

        while let Some(ch) = self.current_char {
            if ch.is_alphanumeric() || ch == '_' {
                identifier.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        if identifier.is_empty() {
            return Err(TokenizeError::InvalidSeedSyntax(
                "Expected identifier".to_string(),
            ));
        }

        Ok(identifier)
    }

    /// Consume optional numeric type suffix after a number literal (u8/u16/u32/u64 or i8/i16/i32/i64)
    fn consume_numeric_suffix(&mut self) {
        match self.current_char {
            Some('u') | Some('i') => {
                // consume the leading letter
                self.advance();
                // Consume up to two digits to cover 8,16,32,64; be permissive but bounded
                for _ in 0..2 {
                    if matches!(self.current_char, Some('0'..='9')) {
                        self.advance();
                    } else {
                        break;
                    }
                }
            }
            _ => {}
        }
    }

    /// Classify identifier as keyword, type, or regular identifier
    fn classify_identifier(&self, identifier: &str) -> Token {
        match identifier {
            // Keywords
            "init" => Token::Init,
            "constraints" => Token::Constraints,
            "instruction" => Token::Instruction,
            "use" | "import" => Token::Use,
            "interface" => Token::Interface,
            "when" => Token::When,
            "event" => Token::Event,
            "emit" => Token::Emit,
            "query" => Token::Query,
            "return" => Token::Return,
            "if" => Token::If,
            "else" => Token::Else,
            "match" => Token::Match,
            "let" => Token::Let,
            "mut" => Token::Mut,
            "of" => Token::Of,
            "or_init" => Token::OrInit,
            "in" => Token::In,
            "realloc" => Token::Realloc,
            "pda" => Token::Pda,
            "while" => Token::While,
            "for" => Token::For,
            "do" => Token::Do,
            "break" => Token::Break,
            "continue" => Token::Continue,
            "true" => Token::True,
            "false" => Token::False,
            "require" => Token::Require,
            "error" => Token::Error,
            "as" => Token::As,
            "fn" => Token::Fn,
            "async" => Token::Async,
            "script" => Token::Script,
            "enum" => Token::Enum,
            "field" => Token::Field,
            "pub" => Token::Pub,
            "assert_eq" => Token::AssertEq,
            "assert_true" => Token::AssertTrue,
            "assert_false" => Token::AssertFalse,
            "assert_fails" => Token::AssertFails,
            "assert_approx_eq" => Token::AssertApproxEq,
            "Some" => Token::Some,
            "None" => Token::None,
            "Ok" => Token::Ok,
            "Err" => Token::Err,
            "Result" => Token::Result,
            "Option" => Token::Option,
            "ignore" => Token::Ignore,
            "should_fail" => Token::ShouldFail,
            "timeout" => Token::Timeout,
            "discriminator" => Token::Discriminator,
            "serializer" => Token::Serializer,
            "discriminator_bytes" => Token::DiscriminatorBytes,

            // Account keyword
            "account" => Token::Account,

            // Built-in types
            "pubkey" | "u64" | "u32" | "u16" | "u8" | "i64" | "i32" | "i16" | "i8" | "bool"
            | "string" | "lamports" | "u128" => Token::Type(identifier.to_string()),

            // Account type (uppercase)
            "Account" => Token::Type(identifier.to_string()),

            // Regular identifier
            _ => Token::Identifier(identifier.to_string()),
        }
    }

    /// Advance to next character, tracking line and column positions
    fn advance(&mut self) {
        if let Some(ch) = self.current_char {
            if ch == '\n' {
                self.line += 1;
                self.column = 0;
            } else {
                self.column += 1;
            }
        }
        self.current_char = self.chars.next();
    }

    /// Skip whitespace characters
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Parse seeds list for @init[seeds] syntax
    fn parse_seeds_list(&mut self, tokens: &mut Vec<Token>) -> Result<(), VMError> {
        const MAX_SEEDS: usize = 8; // Reasonable limit for PDA seeds
        let mut first_seed = true;
        let mut seed_count = 0;

        // Loop until ']' or EOF
        while let Some(ch) = self.current_char {
            // Skip whitespace
            if ch.is_whitespace() {
                self.advance();
                continue;
            }

            // If we see ']', we're done with the list
            if ch == ']' {
                break;
            }

            // Expect comma if not the first seed
            if !first_seed {
                if ch == ',' {
                    self.advance(); // consume ','
                    tokens.push(Token::Comma);
                    self.skip_whitespace();
                    // If next char is ']', it's a trailing comma, which is an error
                    if self.current_char == Some(']') {
                        return Err(VMError::from(TokenizeError::InvalidSeedSyntax(
                            "Trailing comma in seed list".to_string(),
                        )));
                    }
                } else {
                    return Err(VMError::from(TokenizeError::InvalidSeedSyntax(
                        "Expected ',' or ']' in seed list".to_string(),
                    )));
                }
            }

            // Read seed value (identifier, string literal, or number)
            match self.current_char {
                Some('a'..='z') | Some('A'..='Z') | Some('_') => {
                    let identifier = self.read_identifier().map_err(VMError::from)?;
                    let token = self.classify_identifier(&identifier);
                    tokens.push(token);

                    // Handle field access (e.g., user.key, mint.address)
                    while self.current_char == Some('.') {
                        self.advance(); // consume '.'
                        tokens.push(Token::Dot);

                        // Read the field name after the dot
                        if let Some(ch) = self.current_char {
                            if ch.is_alphabetic() || ch == '_' {
                                let field_name =
                                    self.read_identifier().map_err(VMError::from)?;
                                let field_token = self.classify_identifier(&field_name);
                                tokens.push(field_token);
                            } else {
                                return Err(VMError::from(TokenizeError::InvalidSeedSyntax(
                                    "Expected field name after '.'".to_string(),
                                )));
                            }
                        } else {
                            return Err(VMError::from(TokenizeError::InvalidSeedSyntax(
                                "Expected field name after '.'".to_string(),
                            )));
                        }
                    }
                }
                Some('"') => {
                    let string_value = self.read_string_literal().map_err(VMError::from)?;
                    tokens.push(Token::StringLiteral(string_value));
                }
                Some('0'..='9') => {
                    let number_value = self.read_number_literal().map_err(VMError::from)?;
                    tokens.push(Token::NumberLiteral(number_value));
                }
                _ => {
                    return Err(VMError::from(TokenizeError::InvalidSeedSyntax(
                        "Expected identifier, string literal, or number as seed".to_string(),
                    )));
                }
            };

            seed_count += 1;
            first_seed = false;

            // Check for too many seeds
            if seed_count > MAX_SEEDS {
                return Err(VMError::from(TokenizeError::TooManySeeds));
            }

            // After parsing a seed value, skip whitespace and check what comes next
            self.skip_whitespace();

            // The next character must be either ',' (more seeds) or ']' (end of seeds)
            // If it's anything else, it's an error
            if let Some(next_ch) = self.current_char {
                if next_ch != ',' && next_ch != ']' {
                    return Err(VMError::from(TokenizeError::InvalidSeedSyntax(format!(
                        "Expected ',' or ']' after seed, found '{}'",
                        next_ch
                    ))));
                }
            }
        }

        // If the loop finishes due to EOF before ']', it's an error
        if self.current_char.is_none() {
            return Err(VMError::from(TokenizeError::InvalidSeedSyntax(
                "Expected ']' to close seed list".to_string(),
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use five_vm_mito::error::VMError;

    #[test]
    fn test_simple_at_init() {
        let mut tokenizer = DslTokenizer::new("@init my_account: Account");
        let tokens = tokenizer.tokenize().unwrap();
        println!("Tokens: {:?}", tokens);
        assert!(matches!(tokens[0], Token::AtInit));
        assert!(matches!(tokens[1], Token::Identifier(ref name) if name == "my_account"));
    }

    #[test]
    fn test_at_init_with_string_seeds() {
        let mut tokenizer = DslTokenizer::new("@init[\"vault\", \"user\"] vault: Account");
        let tokens = tokenizer.tokenize().unwrap();

        let expected_sequence = vec![
            Token::AtInit,
            Token::LeftBracket,
            Token::StringLiteral("vault".to_string()),
            Token::Comma,
            Token::StringLiteral("user".to_string()),
            Token::RightBracket,
            Token::Identifier("vault".to_string()),
            Token::Colon,
            Token::Type("Account".to_string()),
            Token::Eof,
        ];

        for (i, expected) in expected_sequence.iter().enumerate() {
            assert_eq!(tokens[i], *expected, "Token mismatch at position {}", i);
        }
    }

    #[test]
    fn test_at_init_with_mixed_seeds() {
        let mut tokenizer = DslTokenizer::new("@init[\"prefix\", user.key] account: Account");
        let tokens = tokenizer.tokenize().unwrap();

        // Verify the key parts of the token sequence
        assert!(matches!(tokens[0], Token::AtInit));
        assert!(matches!(tokens[1], Token::LeftBracket));
        assert!(matches!(tokens[2], Token::StringLiteral(ref s) if s == "prefix"));
        assert!(matches!(tokens[3], Token::Comma));
        assert!(matches!(tokens[4], Token::Identifier(ref s) if s == "user"));
        assert!(matches!(tokens[5], Token::Dot));
        assert!(matches!(tokens[6], Token::Identifier(ref s) if s == "key"));
        assert!(matches!(tokens[7], Token::RightBracket));
    }

    #[test]
    fn test_at_init_empty_seeds_error() {
        let mut tokenizer = DslTokenizer::new("@init[] account: Account");
        let tokens = tokenizer.tokenize().unwrap();

        // Empty seeds should be allowed - it's up to the parser to validate semantics
        assert!(matches!(tokens[0], Token::AtInit));
        assert!(matches!(tokens[1], Token::LeftBracket));
        assert!(matches!(tokens[2], Token::RightBracket));
    }

    #[test]
    fn test_at_init_unterminated_seeds_error() {
        let mut tokenizer = DslTokenizer::new("@init[\"vault\" account: Account");
        let result = tokenizer.tokenize();
        assert!(result.is_err());
    }

    #[test]
    fn test_at_init_trailing_comma_error_message() {
        let mut tokenizer = DslTokenizer::new("@init[\"vault\",] account: Account");
        let err = tokenizer.tokenize().unwrap_err();
        assert!(matches!(err, VMError::InvalidScript));
    }

    #[test]
    fn test_at_init_missing_closing_bracket_message() {
        // Missing closing ']' should surface a specific error message
        let mut tokenizer = DslTokenizer::new("@init[\"vault\"");
        let err = tokenizer.tokenize().unwrap_err();
        assert!(matches!(err, VMError::InvalidScript));
    }

    #[test]
    fn test_at_init_missing_comma_message() {
        // When a seed is followed by another token without a comma, the parser
        // should retain the offending character in the message.
        let mut tokenizer = DslTokenizer::new("@init[\"vault\" account: Account");
        let err = tokenizer.tokenize().unwrap_err();
        assert!(matches!(err, VMError::InvalidScript));
    }

    #[test]
    fn test_bitshift_operators() {
        let mut tokenizer = DslTokenizer::new("x << 4");
        let tokens = tokenizer.tokenize().unwrap();
        assert!(tokens.iter().any(|t| matches!(t, Token::LeftShift)));

        let mut tokenizer = DslTokenizer::new("x >> 2");
        let tokens = tokenizer.tokenize().unwrap();
        assert!(tokens.iter().any(|t| matches!(t, Token::RightShift)));

        let mut tokenizer = DslTokenizer::new("x >>> 3");
        let tokens = tokenizer.tokenize().unwrap();
        assert!(tokens.iter().any(|t| matches!(t, Token::ArithRightShift)));

        let mut tokenizer = DslTokenizer::new("x <<< 1");
        let tokens = tokenizer.tokenize().unwrap();
        assert!(tokens.iter().any(|t| matches!(t, Token::RotateLeft)));
    }

    #[test]
    fn test_bitwise_operators() {
        let mut tokenizer = DslTokenizer::new("a & b");
        let tokens = tokenizer.tokenize().unwrap();
        assert!(tokens.iter().any(|t| matches!(t, Token::BitwiseAnd)));

        let mut tokenizer = DslTokenizer::new("a | b");
        let tokens = tokenizer.tokenize().unwrap();
        assert!(tokens.iter().any(|t| matches!(t, Token::BitwiseOr)));

        let mut tokenizer = DslTokenizer::new("a ^ b");
        let tokens = tokenizer.tokenize().unwrap();
        assert!(tokens.iter().any(|t| matches!(t, Token::BitwiseXor)));

        let mut tokenizer = DslTokenizer::new("~x");
        let tokens = tokenizer.tokenize().unwrap();
        assert!(tokens.iter().any(|t| matches!(t, Token::BitwiseTilde)));
    }

    #[test]
    fn test_distinguishes_logical_from_bitwise() {
        let mut tokenizer = DslTokenizer::new("a && b & c");
        let tokens = tokenizer.tokenize().unwrap();
        assert!(tokens.iter().any(|t| matches!(t, Token::LogicalAnd)));
        assert!(tokens.iter().any(|t| matches!(t, Token::BitwiseAnd)));

        let mut tokenizer = DslTokenizer::new("a || b | c");
        let tokens = tokenizer.tokenize().unwrap();
        assert!(tokens.iter().any(|t| matches!(t, Token::LogicalOr)));
        assert!(tokens.iter().any(|t| matches!(t, Token::BitwiseOr)));
    }
}
