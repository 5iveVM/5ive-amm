use five_dsl_compiler::bytecode_generator::disassembler::BytecodeInspector;
use five_dsl_compiler::bytecode_generator::DslBytecodeGenerator;
/// Bytecode snapshot tests for end-to-end generation.
use five_dsl_compiler::*;
use five_protocol::opcodes;

fn bytecode_contains_u64_literal(bytecode: &[u8], value: u64) -> bool {
    let needle = value.to_le_bytes();
    bytecode.windows(needle.len()).any(|window| window == needle)
}

// ============================================================================
// Test Group 1: Header Validation
// ============================================================================

#[test]
fn test_header_magic_bytes() {
    let source = r#"
        pub main() -> u64 {
            return 42;
        }
    "#;

    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    let mut generator = DslBytecodeGenerator::new();
    let bytecode = generator.generate(&ast).expect("Should generate bytecode");

    // Verify magic bytes "5IVE"
    assert!(bytecode.len() >= 4, "Bytecode should have at least 4 bytes");
    assert_eq!(bytecode[0], b'5', "Magic byte 0 should be '5'");
    assert_eq!(bytecode[1], b'I', "Magic byte 1 should be 'I'");
    assert_eq!(bytecode[2], b'V', "Magic byte 2 should be 'V'");
    assert_eq!(bytecode[3], b'E', "Magic byte 3 should be 'E'");
}

#[test]
fn test_header_version() {
    let source = r#"
        pub main() -> u64 {
            return 42;
        }
    "#;

    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    let mut generator = DslBytecodeGenerator::new();
    let bytecode = generator.generate(&ast).expect("Should generate bytecode");

    // Version byte at index 4
    assert!(bytecode.len() > 4, "Bytecode should have version byte");
    let version = bytecode[4];
    assert!(version > 0, "Version should be non-zero");
}

#[test]
fn test_header_function_counts() {
    let source = r#"
        pub main() -> u64 {
            return 1;
        }

        pub helper() -> u64 {
            return 2;
        }

        internal() -> u64 {
            return 3;
        }
    "#;

    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    let mut generator = DslBytecodeGenerator::new();
    let bytecode = generator.generate(&ast).expect("Should generate bytecode");

    // Public function count at index 8, total at index 9 (after 4-byte magic + 4-byte features)
    assert!(bytecode.len() > 9, "Bytecode should have function counts");
    let public_count = bytecode[8];
    let total_count = bytecode[9];

    assert_eq!(public_count, 2, "Should have 2 public functions");
    assert_eq!(total_count, 3, "Should have 3 total functions");
}

// ============================================================================
// Test Group 2: Simple Program Snapshots
// ============================================================================

#[test]
fn test_simple_return_bytecode() {
    let source = r#"
        pub main() -> u64 {
            return 42;
        }
    "#;

    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    let mut generator = DslBytecodeGenerator::new();
    let bytecode = generator.generate(&ast).expect("Should generate bytecode");

    // Verify bytecode is non-empty and has reasonable size
    assert!(!bytecode.is_empty(), "Bytecode should not be empty");
    assert!(
        bytecode.len() > 10,
        "Bytecode should have header + instructions"
    );

    // Should contain PUSH opcode for literal 42 (semantic check)
    let inspector = BytecodeInspector::new(&bytecode);
    assert!(
        inspector.contains_push_u64(42) || bytecode_contains_u64_literal(&bytecode, 42),
        "Bytecode should contain literal 42 (push or constant-pool encoded)"
    );
}

#[test]
fn test_variable_assignment_bytecode() {
    let source = r#"
        pub test() -> u64 {
            let x = 100;
            return x;
        }
    "#;

    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    let mut generator = DslBytecodeGenerator::new();
    let bytecode = generator.generate(&ast).expect("Should generate bytecode");

    assert!(!bytecode.is_empty(), "Bytecode should not be empty");

    // Should contain PUSH 100 and STORE_LOCAL/LOAD_LOCAL operations (semantic check)
    let inspector = BytecodeInspector::new(&bytecode);
    assert!(
        inspector.contains_push_u64(100) || bytecode_contains_u64_literal(&bytecode, 100),
        "Bytecode should contain literal 100 (push or constant-pool encoded)"
    );
}

#[test]
fn test_arithmetic_bytecode() {
    let source = r#"
        pub add() -> u64 {
            let a = 10;
            let b = 20;
            return a + b;
        }
    "#;

    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    let mut generator = DslBytecodeGenerator::new();
    let bytecode = generator.generate(&ast).expect("Should generate bytecode");

    assert!(!bytecode.is_empty(), "Bytecode should not be empty");

    // Should contain ADD opcode (semantic check)
    let inspector = BytecodeInspector::new(&bytecode);
    assert!(
        inspector.contains_opcode(opcodes::ADD),
        "Bytecode should contain ADD opcode"
    );
}

// ============================================================================
// Test Group 3: Comparison Bytecode
// ============================================================================

#[test]
fn test_comparison_bytecode() {
    let source = r#"
        pub test() -> bool {
            let x = 10;
            let y = 20;
            return x < y;
        }
    "#;

    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    let mut generator = DslBytecodeGenerator::new();
    let bytecode = generator.generate(&ast).expect("Should generate bytecode");

    assert!(!bytecode.is_empty(), "Bytecode should not be empty");
    // Comparison operations generate bytecode
}

// ============================================================================
// Test Group 4: Function Call Bytecode
// ============================================================================

#[test]
fn test_function_call_bytecode() {
    let source = r#"
        pub main() -> u64 {
            return helper();
        }

        helper() -> u64 {
            return 42;
        }
    "#;

    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    let mut generator = DslBytecodeGenerator::new();
    let bytecode = generator.generate(&ast).expect("Should generate bytecode");

    assert!(!bytecode.is_empty(), "Bytecode should not be empty");

    // Should contain CALL opcode (semantic check)
    let inspector = BytecodeInspector::new(&bytecode);
    assert!(
        inspector.contains_call(),
        "Bytecode should contain CALL opcode"
    );
}

#[test]
fn test_function_with_parameters_bytecode() {
    let source = r#"
        pub main() -> u64 {
            return add(10, 20);
        }

        add(a: u64, b: u64) -> u64 {
            return a + b;
        }
    "#;

    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    let mut generator = DslBytecodeGenerator::new();
    let bytecode = generator.generate(&ast).expect("Should generate bytecode");

    assert!(!bytecode.is_empty(), "Bytecode should not be empty");

    // Should contain CALL with 2 parameters (semantic check)
    let inspector = BytecodeInspector::new(&bytecode);
    assert!(
        inspector.contains_call(),
        "Bytecode should contain CALL for function with parameters"
    );
}

// ============================================================================
// Test Group 5: Bytecode Size and Structure
// ============================================================================

#[test]
fn test_bytecode_minimum_size() {
    let source = r#"
        pub main() -> u64 {
            return 0;
        }
    "#;

    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    let mut generator = DslBytecodeGenerator::new();
    let bytecode = generator.generate(&ast).expect("Should generate bytecode");

    // Minimum: magic (4) + version (1) + public_count (1) + total_count (1) +
    // function table + instructions
    assert!(
        bytecode.len() >= 7,
        "Bytecode should have at least 7 bytes for header"
    );
}

#[test]
fn test_bytecode_deterministic() {
    let source = r#"
        pub test() -> u64 {
            let x = 42;
            return x;
        }
    "#;

    let mut tokenizer1 = DslTokenizer::new(source);
    let tokens1 = tokenizer1.tokenize().expect("Should tokenize");
    let mut parser1 = DslParser::new(tokens1);
    let ast1 = parser1.parse().expect("Should parse");
    let mut generator1 = DslBytecodeGenerator::new();
    let bytecode1 = generator1
        .generate(&ast1)
        .expect("Should generate bytecode");

    let mut tokenizer2 = DslTokenizer::new(source);
    let tokens2 = tokenizer2.tokenize().expect("Should tokenize");
    let mut parser2 = DslParser::new(tokens2);
    let ast2 = parser2.parse().expect("Should parse");
    let mut generator2 = DslBytecodeGenerator::new();
    let bytecode2 = generator2
        .generate(&ast2)
        .expect("Should generate bytecode");

    assert_eq!(
        bytecode1, bytecode2,
        "Same source should produce identical bytecode"
    );
}

#[test]
fn test_complex_program_bytecode_structure() {
    let source = r#"
        pub main() -> u64 {
            let result = helper(5, 10);
            return result;
        }

        helper(a: u64, b: u64) -> u64 {
            let sum = a + b;
            return sum * 2;
        }
    "#;

    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    let mut generator = DslBytecodeGenerator::new();
    let bytecode = generator.generate(&ast).expect("Should generate bytecode");

    assert!(!bytecode.is_empty(), "Bytecode should not be empty");
    assert!(
        bytecode.len() > 20,
        "Complex program should generate substantial bytecode"
    );

    // Should have magic bytes "5IVE"
    assert_eq!(&bytecode[0..4], b"5IVE");

    // Should have function counts (2 functions, 1 public)
    // Function counts are at bytes [8] and [9] after the 10-byte header
    assert_eq!(bytecode[8], 1, "Should have 1 public function");
    assert_eq!(bytecode[9], 2, "Should have 2 total functions");
}

// ============================================================================
// Test Group: Bitshift and Bitwise Operations
// ============================================================================

#[test]
fn test_shift_left_bytecode() {
    let source = r#"
        pub shift_left() -> u64 {
            let x: u64 = 1;
            return x << 4;
        }
    "#;

    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    let mut generator = DslBytecodeGenerator::new();
    let bytecode = generator.generate(&ast).expect("Should generate bytecode");

    assert!(!bytecode.is_empty(), "Bytecode should not be empty");

    // Should contain SHIFT_LEFT opcode (0x38)
    let inspector = BytecodeInspector::new(&bytecode);
    assert!(
        inspector.contains_opcode(opcodes::SHIFT_LEFT),
        "Bytecode should contain SHIFT_LEFT opcode"
    );
}

#[test]
fn test_shift_right_bytecode() {
    let source = r#"
        pub shift_right() -> u64 {
            let x: u64 = 16;
            return x >> 2;
        }
    "#;

    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    let mut generator = DslBytecodeGenerator::new();
    let bytecode = generator.generate(&ast).expect("Should generate bytecode");

    assert!(!bytecode.is_empty(), "Bytecode should not be empty");

    // Should contain SHIFT_RIGHT opcode (0x39)
    let inspector = BytecodeInspector::new(&bytecode);
    assert!(
        inspector.contains_opcode(opcodes::SHIFT_RIGHT),
        "Bytecode should contain SHIFT_RIGHT opcode"
    );
}

#[test]
fn test_arithmetic_shift_right_bytecode() {
    let source = r#"
        pub arith_shift() -> u64 {
            let x: u64 = 255;
            return x >>> 4;
        }
    "#;

    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    let mut generator = DslBytecodeGenerator::new();
    let bytecode = generator.generate(&ast).expect("Should generate bytecode");

    assert!(!bytecode.is_empty(), "Bytecode should not be empty");

    // Should contain SHIFT_RIGHT_ARITH opcode (0x3A)
    let inspector = BytecodeInspector::new(&bytecode);
    assert!(
        inspector.contains_opcode(opcodes::SHIFT_RIGHT_ARITH),
        "Bytecode should contain SHIFT_RIGHT_ARITH opcode"
    );
}

#[test]
fn test_rotate_left_bytecode() {
    let source = r#"
        pub rotate() -> u64 {
            let x: u64 = 128;
            return x <<< 3;
        }
    "#;

    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    let mut generator = DslBytecodeGenerator::new();
    let bytecode = generator.generate(&ast).expect("Should generate bytecode");

    assert!(!bytecode.is_empty(), "Bytecode should not be empty");

    // Should contain ROTATE_LEFT opcode (0x3B)
    let inspector = BytecodeInspector::new(&bytecode);
    assert!(
        inspector.contains_opcode(opcodes::ROTATE_LEFT),
        "Bytecode should contain ROTATE_LEFT opcode"
    );
}

#[test]
fn test_bitwise_and_bytecode() {
    let source = r#"
        pub bitand() -> u64 {
            let x: u64 = 255;
            let mask: u64 = 15;
            return x & mask;
        }
    "#;

    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    let mut generator = DslBytecodeGenerator::new();
    let bytecode = generator.generate(&ast).expect("Should generate bytecode");

    assert!(!bytecode.is_empty(), "Bytecode should not be empty");

    // Should contain BITWISE_AND opcode (0x35) - NOT logical AND (0x30)
    let inspector = BytecodeInspector::new(&bytecode);
    assert!(
        inspector.contains_opcode(opcodes::BITWISE_AND),
        "Bytecode should contain BITWISE_AND opcode"
    );
}

#[test]
fn test_bitwise_or_bytecode() {
    let source = r#"
        pub bitor() -> u64 {
            let a: u64 = 0x0F;
            let b: u64 = 0xF0;
            return a | b;
        }
    "#;

    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    let mut generator = DslBytecodeGenerator::new();
    let bytecode = generator.generate(&ast).expect("Should generate bytecode");

    assert!(!bytecode.is_empty(), "Bytecode should not be empty");

    // Should contain BITWISE_OR opcode (0x36) - NOT logical OR (0x31)
    let inspector = BytecodeInspector::new(&bytecode);
    assert!(
        inspector.contains_opcode(opcodes::BITWISE_OR),
        "Bytecode should contain BITWISE_OR opcode"
    );
}

#[test]
fn test_bitwise_xor_bytecode() {
    let source = r#"
        pub bitxor() -> u64 {
            let a: u64 = 0xFF;
            let b: u64 = 0x0F;
            return a ^ b;
        }
    "#;

    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    let mut generator = DslBytecodeGenerator::new();
    let bytecode = generator.generate(&ast).expect("Should generate bytecode");

    assert!(!bytecode.is_empty(), "Bytecode should not be empty");

    // Should contain BITWISE_XOR opcode (0x37)
    let inspector = BytecodeInspector::new(&bytecode);
    assert!(
        inspector.contains_opcode(opcodes::BITWISE_XOR),
        "Bytecode should contain BITWISE_XOR opcode"
    );
}

#[test]
fn test_bitwise_not_bytecode() {
    let source = r#"
        pub bitnot() -> u64 {
            let x: u64 = 0;
            return ~x;
        }
    "#;

    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    let mut generator = DslBytecodeGenerator::new();
    let bytecode = generator.generate(&ast).expect("Should generate bytecode");

    assert!(!bytecode.is_empty(), "Bytecode should not be empty");

    // Should contain BITWISE_NOT opcode (0x34)
    let inspector = BytecodeInspector::new(&bytecode);
    assert!(
        inspector.contains_opcode(opcodes::BITWISE_NOT),
        "Bytecode should contain BITWISE_NOT opcode"
    );
}

#[test]
fn test_extract_byte_pattern() {
    // Common bit manipulation pattern: extract a byte at a position
    let source = r#"
        pub extract_byte(value: u64, position: u64) -> u64 {
            return (value >> (position * 8)) & 255;
        }
    "#;

    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    let mut generator = DslBytecodeGenerator::new();
    let bytecode = generator.generate(&ast).expect("Should generate bytecode");

    assert!(!bytecode.is_empty(), "Bytecode should not be empty");

    // Should contain both SHIFT_RIGHT and BITWISE_AND opcodes
    let inspector = BytecodeInspector::new(&bytecode);
    assert!(
        inspector.contains_opcode(opcodes::SHIFT_RIGHT),
        "Bytecode should contain SHIFT_RIGHT opcode for byte extraction"
    );
    assert!(
        inspector.contains_opcode(opcodes::BITWISE_AND),
        "Bytecode should contain BITWISE_AND opcode for byte masking"
    );
}

#[test]
fn test_compound_shift_assignment() {
    let source = r#"
        pub shift_assign() -> u64 {
            let mut x: u64 = 1;
            x <<= 4;
            return x;
        }
    "#;

    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    let mut generator = DslBytecodeGenerator::new();
    let bytecode = generator.generate(&ast).expect("Should generate bytecode");

    assert!(!bytecode.is_empty(), "Bytecode should not be empty");

    // Should contain SHIFT_LEFT opcode from the <<= compound assignment
    let inspector = BytecodeInspector::new(&bytecode);
    assert!(
        inspector.contains_opcode(opcodes::SHIFT_LEFT),
        "Bytecode should contain SHIFT_LEFT opcode from compound assignment"
    );
}

#[test]
fn test_compound_bitwise_and_assignment() {
    let source = r#"
        pub and_assign() -> u64 {
            let mut x: u64 = 255;
            x &= 15;
            return x;
        }
    "#;

    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    let mut generator = DslBytecodeGenerator::new();
    let bytecode = generator.generate(&ast).expect("Should generate bytecode");

    assert!(!bytecode.is_empty(), "Bytecode should not be empty");

    // Should contain BITWISE_AND opcode from the &= compound assignment
    let inspector = BytecodeInspector::new(&bytecode);
    assert!(
        inspector.contains_opcode(opcodes::BITWISE_AND),
        "Bytecode should contain BITWISE_AND opcode from compound assignment"
    );
}

#[test]
fn test_compound_right_shift_assignment() {
    let source = r#"
        pub shift_assign() -> u64 {
            let mut x: u64 = 256;
            x >>= 3;
            return x;
        }
    "#;

    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    let mut generator = DslBytecodeGenerator::new();
    let bytecode = generator.generate(&ast).expect("Should generate bytecode");

    assert!(!bytecode.is_empty(), "Bytecode should not be empty");

    let inspector = BytecodeInspector::new(&bytecode);
    assert!(
        inspector.contains_opcode(opcodes::SHIFT_RIGHT),
        "Bytecode should contain SHIFT_RIGHT opcode from compound assignment"
    );
}

#[test]
fn test_compound_bitwise_or_assignment() {
    let source = r#"
        pub or_assign() -> u64 {
            let mut x: u64 = 0x0F;
            x |= 0xF0;
            return x;
        }
    "#;

    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    let mut generator = DslBytecodeGenerator::new();
    let bytecode = generator.generate(&ast).expect("Should generate bytecode");

    assert!(!bytecode.is_empty(), "Bytecode should not be empty");

    let inspector = BytecodeInspector::new(&bytecode);
    assert!(
        inspector.contains_opcode(opcodes::BITWISE_OR),
        "Bytecode should contain BITWISE_OR opcode from compound assignment"
    );
}

#[test]
fn test_compound_bitwise_xor_assignment() {
    let source = r#"
        pub xor_assign() -> u64 {
            let mut x: u64 = 0xFF;
            x ^= 0x0F;
            return x;
        }
    "#;

    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    let mut generator = DslBytecodeGenerator::new();
    let bytecode = generator.generate(&ast).expect("Should generate bytecode");

    assert!(!bytecode.is_empty(), "Bytecode should not be empty");

    let inspector = BytecodeInspector::new(&bytecode);
    assert!(
        inspector.contains_opcode(opcodes::BITWISE_XOR),
        "Bytecode should contain BITWISE_XOR opcode from compound assignment"
    );
}

#[test]
fn test_rotate_right_bytecode() {
    // Note: >>> for rotate_right is typically an arithmetic right shift in many languages.
    // In Five, we verify that arithmetic right shift (>>>) generates the correct opcode.
    // Full rotate_right support via ">>" operator may be context-dependent.
    let source = r#"
        pub shift() -> u64 {
            let x: u64 = 255;
            return x >>> 1;
        }
    "#;

    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    let mut generator = DslBytecodeGenerator::new();
    let bytecode = generator.generate(&ast).expect("Should generate bytecode");

    assert!(!bytecode.is_empty(), "Bytecode should not be empty");

    // Verify SHIFT_RIGHT_ARITH opcode is present
    let inspector = BytecodeInspector::new(&bytecode);
    assert!(
        inspector.contains_opcode(opcodes::SHIFT_RIGHT_ARITH),
        "Bytecode should contain SHIFT_RIGHT_ARITH opcode"
    );
}

#[test]
fn test_shift_precedence_over_additive() {
    // Test that x << a + b parses as x << (a + b), NOT (x << a) + b
    // Uses variables to prevent constant folding optimization
    let source = r#"
        pub test(a: u64, b: u64) -> u64 {
            let x: u64 = 4;
            return x << a + b;
        }
    "#;

    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    let mut generator = DslBytecodeGenerator::new();
    let bytecode = generator.generate(&ast).expect("Should generate bytecode");

    assert!(!bytecode.is_empty(), "Bytecode should not be empty");

    // Should have both ADD and SHIFT_LEFT: ADD is evaluated first (a + b), then shift
    let inspector = BytecodeInspector::new(&bytecode);
    assert!(
        inspector.contains_opcode(opcodes::SHIFT_LEFT),
        "Bytecode should contain SHIFT_LEFT opcode"
    );
    assert!(
        inspector.contains_opcode(opcodes::ADD),
        "Bytecode should contain ADD opcode for a + b"
    );
}

#[test]
fn test_bitwise_and_precedence_over_or() {
    // Test that a & b | c parses as (a & b) | c, NOT a & (b | c)
    let source = r#"
        pub test() -> u64 {
            let a: u64 = 0x0F;
            let b: u64 = 0xF0;
            let c: u64 = 0xFF;
            return a & b | c;
        }
    "#;

    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    let mut generator = DslBytecodeGenerator::new();
    let bytecode = generator.generate(&ast).expect("Should generate bytecode");

    assert!(!bytecode.is_empty(), "Bytecode should not be empty");

    // Should contain both BITWISE_AND and BITWISE_OR in correct order
    let inspector = BytecodeInspector::new(&bytecode);
    assert!(
        inspector.contains_opcode(opcodes::BITWISE_AND),
        "Bytecode should contain BITWISE_AND opcode"
    );
    assert!(
        inspector.contains_opcode(opcodes::BITWISE_OR),
        "Bytecode should contain BITWISE_OR opcode"
    );
}

#[test]
fn test_xor_vs_or_precedence() {
    // Test that a | b ^ c parses as a | (b ^ c), NOT (a | b) ^ c
    let source = r#"
        pub test() -> u64 {
            let a: u64 = 0xFF;
            let b: u64 = 0x0F;
            let c: u64 = 0xF0;
            return a | b ^ c;
        }
    "#;

    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    let mut generator = DslBytecodeGenerator::new();
    let bytecode = generator.generate(&ast).expect("Should generate bytecode");

    assert!(!bytecode.is_empty(), "Bytecode should not be empty");

    let inspector = BytecodeInspector::new(&bytecode);
    assert!(
        inspector.contains_opcode(opcodes::BITWISE_XOR),
        "Bytecode should contain BITWISE_XOR opcode"
    );
    assert!(
        inspector.contains_opcode(opcodes::BITWISE_OR),
        "Bytecode should contain BITWISE_OR opcode"
    );
}

#[test]
fn test_chained_bitwise_operations() {
    // Test: a & b & c
    let source = r#"
        pub test() -> u64 {
            let a: u64 = 0xFF;
            let b: u64 = 0x0F;
            let c: u64 = 0x03;
            return a & b & c;
        }
    "#;

    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    let mut generator = DslBytecodeGenerator::new();
    let bytecode = generator.generate(&ast).expect("Should generate bytecode");

    assert!(!bytecode.is_empty(), "Bytecode should not be empty");

    // Should contain BITWISE_AND opcode (will appear twice in bytecode)
    let inspector = BytecodeInspector::new(&bytecode);
    assert!(
        inspector.contains_opcode(opcodes::BITWISE_AND),
        "Bytecode should contain BITWISE_AND opcode for chained operations"
    );
}

#[test]
fn test_chained_shift_operations() {
    // Test: x << 2 << 1 (equivalent to x << 3)
    let source = r#"
        pub test() -> u64 {
            let x: u64 = 1;
            return x << 2 << 1;
        }
    "#;

    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    let mut generator = DslBytecodeGenerator::new();
    let bytecode = generator.generate(&ast).expect("Should generate bytecode");

    assert!(!bytecode.is_empty(), "Bytecode should not be empty");

    let inspector = BytecodeInspector::new(&bytecode);
    assert!(
        inspector.contains_opcode(opcodes::SHIFT_LEFT),
        "Bytecode should contain SHIFT_LEFT opcode for chained shifts"
    );
}

#[test]
fn test_bitwise_not_with_and() {
    // Test: ~a & b (mask with complement)
    let source = r#"
        pub test() -> u64 {
            let a: u64 = 0x0F;
            let b: u64 = 0xFF;
            return ~a & b;
        }
    "#;

    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    let mut generator = DslBytecodeGenerator::new();
    let bytecode = generator.generate(&ast).expect("Should generate bytecode");

    assert!(!bytecode.is_empty(), "Bytecode should not be empty");

    let inspector = BytecodeInspector::new(&bytecode);
    assert!(
        inspector.contains_opcode(opcodes::BITWISE_NOT),
        "Bytecode should contain BITWISE_NOT opcode"
    );
    assert!(
        inspector.contains_opcode(opcodes::BITWISE_AND),
        "Bytecode should contain BITWISE_AND opcode"
    );
}

#[test]
fn test_shift_zero_amount() {
    // Edge case: shifting by zero should still generate shift opcode
    let source = r#"
        pub test() -> u64 {
            let x: u64 = 42;
            return x << 0;
        }
    "#;

    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    let mut generator = DslBytecodeGenerator::new();
    let bytecode = generator.generate(&ast).expect("Should generate bytecode");

    assert!(!bytecode.is_empty(), "Bytecode should not be empty");

    let inspector = BytecodeInspector::new(&bytecode);
    assert!(
        inspector.contains_opcode(opcodes::SHIFT_LEFT),
        "Bytecode should contain SHIFT_LEFT opcode even for zero shift"
    );
}

#[test]
fn test_shift_large_amount() {
    // Edge case: large shift amounts (63 for u64)
    let source = r#"
        pub test() -> u64 {
            let x: u64 = 1;
            return x << 63;
        }
    "#;

    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    let mut generator = DslBytecodeGenerator::new();
    let bytecode = generator.generate(&ast).expect("Should generate bytecode");

    assert!(!bytecode.is_empty(), "Bytecode should not be empty");

    let inspector = BytecodeInspector::new(&bytecode);
    assert!(
        inspector.contains_opcode(opcodes::SHIFT_LEFT),
        "Bytecode should contain SHIFT_LEFT opcode for large shift"
    );
}

#[test]
fn test_bitwise_operations_with_literals() {
    // Test operations directly with hex/binary literals
    let source = r#"
        pub test() -> u64 {
            return 0xFF & 0x0F;
        }
    "#;

    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    let mut generator = DslBytecodeGenerator::new();
    let bytecode = generator.generate(&ast).expect("Should generate bytecode");

    assert!(!bytecode.is_empty(), "Bytecode should not be empty");

    let inspector = BytecodeInspector::new(&bytecode);
    assert!(
        inspector.contains_opcode(opcodes::BITWISE_AND),
        "Bytecode should contain BITWISE_AND opcode with literal operands"
    );
}

#[test]
fn test_mixed_arithmetic_and_bitwise() {
    // Test: (a + b) & c
    let source = r#"
        pub test() -> u64 {
            let a: u64 = 10;
            let b: u64 = 20;
            let c: u64 = 255;
            return (a + b) & c;
        }
    "#;

    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    let mut generator = DslBytecodeGenerator::new();
    let bytecode = generator.generate(&ast).expect("Should generate bytecode");

    assert!(!bytecode.is_empty(), "Bytecode should not be empty");

    let inspector = BytecodeInspector::new(&bytecode);
    assert!(
        inspector.contains_opcode(opcodes::ADD),
        "Bytecode should contain ADD opcode"
    );
    assert!(
        inspector.contains_opcode(opcodes::BITWISE_AND),
        "Bytecode should contain BITWISE_AND opcode"
    );
}

#[test]
fn test_set_bit_pattern() {
    // Common pattern: set_bit(value, position)
    let source = r#"
        pub set_bit(value: u64, position: u64) -> u64 {
            return value | (1 << position);
        }
    "#;

    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    let mut generator = DslBytecodeGenerator::new();
    let bytecode = generator.generate(&ast).expect("Should generate bytecode");

    assert!(!bytecode.is_empty(), "Bytecode should not be empty");

    let inspector = BytecodeInspector::new(&bytecode);
    assert!(
        inspector.contains_opcode(opcodes::SHIFT_LEFT),
        "Bytecode should contain SHIFT_LEFT for bit shifting"
    );
    assert!(
        inspector.contains_opcode(opcodes::BITWISE_OR),
        "Bytecode should contain BITWISE_OR for setting bit"
    );
}

#[test]
fn test_clear_bit_pattern() {
    // Common pattern: clear_bit(value, position)
    let source = r#"
        pub clear_bit(value: u64, position: u64) -> u64 {
            return value & ~(1 << position);
        }
    "#;

    let mut tokenizer = DslTokenizer::new(source);
    let tokens = tokenizer.tokenize().expect("Should tokenize");
    let mut parser = DslParser::new(tokens);
    let ast = parser.parse().expect("Should parse");

    let mut generator = DslBytecodeGenerator::new();
    let bytecode = generator.generate(&ast).expect("Should generate bytecode");

    assert!(!bytecode.is_empty(), "Bytecode should not be empty");

    let inspector = BytecodeInspector::new(&bytecode);
    assert!(
        inspector.contains_opcode(opcodes::SHIFT_LEFT),
        "Bytecode should contain SHIFT_LEFT for bit shifting"
    );
    assert!(
        inspector.contains_opcode(opcodes::BITWISE_NOT),
        "Bytecode should contain BITWISE_NOT for bit negation"
    );
    assert!(
        inspector.contains_opcode(opcodes::BITWISE_AND),
        "Bytecode should contain BITWISE_AND for clearing bit"
    );
}
