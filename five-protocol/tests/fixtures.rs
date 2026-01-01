//! Golden Fixtures for Parser/Verifier Coverage Tests
//!
//! This module contains pre-built bytecode samples that exercise:
//! - Valid headers and instructions
//! - Invalid CALL targets (out-of-bounds function indices)
//! - VLE truncation (incomplete variable-length encoding)
//!
//! These fixtures are used to test the parser's acceptance/rejection logic
//! and ensure consistency between offline parsing and on-chain verification.

use five_protocol::{parser::parse_bytecode, BytecodeBuilder, CALL};

// Fixtures generated at runtime via BytecodeBuilder (avoid raw const byte arrays)
//
// These helper functions construct the same bytecode samples previously encoded
// as raw constants but use the `BytecodeBuilder` so tests never rely on manual
// raw bytes. This also enables usage of `emit_partial_vle_u32` to produce
// truncated VLE sequences deterministically for tests.

/// Generate a valid header with a single HALT instruction
pub fn valid_header() -> Vec<u8> {
    let mut b = BytecodeBuilder::new();
    b.emit_header(1, 2);
    b.emit_halt();
    b.build()
}

/// Generate an invalid CALL target (CALL to function index 5 while total functions = 2)
pub fn invalid_call_target() -> Vec<u8> {
    let mut b = BytecodeBuilder::new();
    b.emit_header(1, 2);
    // Use VLE-encoded function index via emit_call (builder emits correct VLE)
    b.emit_call(5);
    b.build()
}

/// Generate a VLE-truncated CALL: emit CALL opcode then a partial VLE byte sequence
pub fn vle_truncation() -> Vec<u8> {
    let mut b = BytecodeBuilder::new();
    b.emit_header(1, 2);
    // Emit opcode byte directly and then a partial VLE to simulate truncation
    b.emit_opcode(CALL);
    // Emit only the first continuation byte to force an incomplete VLE
    b.emit_partial_vle_u32(0x80, 1);
    b.build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use five_protocol::parser::ParseError;

    #[test]
    fn parser_accepts_valid_header() {
        let bc = valid_header();
        let parsed = parse_bytecode(&bc);
        assert!(
            parsed.errors.is_empty(),
            "Valid header should parse without errors"
        );
        assert_eq!(parsed.header.magic, *b"5IVE");
        assert_eq!(parsed.header.total_function_count, 2);
    }

    #[test]
    fn parser_rejects_invalid_call_target() {
        let bc = invalid_call_target();
        let parsed = parse_bytecode(&bc);
        assert!(
            !parsed.errors.is_empty(),
            "Invalid CALL target should produce errors"
        );
        assert!(
            parsed.errors.contains(&ParseError::CallTargetOutOfBounds),
            "Should detect CALL target out of bounds"
        );
    }

    #[test]
    fn parser_rejects_vle_truncation() {
        let bc = vle_truncation();
        let parsed = parse_bytecode(&bc);
        assert!(
            !parsed.errors.is_empty(),
            "Truncated VLE should produce errors"
        );
        // The exact error might be InvalidVLE or InstructionOutOfBounds, depending on implementation
        assert!(
            parsed.errors.contains(&ParseError::InvalidVLE)
                || parsed.errors.contains(&ParseError::InstructionOutOfBounds),
            "Should detect VLE truncation as an error"
        );
    }
}
