#[cfg(test)]
mod script_header_v3_tests {
    use five::{instructions::verify_bytecode_content, state::ScriptAccountHeader};
    use five_protocol::{
        bytecode, parser,
        test_fixtures::{invalid_call_target, operand_truncation, valid_header},
        HALT,
    };

    #[test]
    fn test_script_header_v3_creation() {
        let test_bytecode = bytecode!(
            emit_magic(),
            emit_u8(0x01),
            emit_u8(0x01),
            emit_push_u64(42),
            emit_halt()
        );

        let owner = [1u8; 32];
        let script_id = 12345u64;
        let header = ScriptAccountHeader::new(test_bytecode.len(), owner, script_id);
        assert_eq!(header.magic, ScriptAccountHeader::MAGIC);
        assert_eq!(header.script_id, script_id);
        assert_eq!(header.bytecode_len(), test_bytecode.len());
    }

    #[test]
    fn test_deploy_time_verification_matches_parser() {
        let valid_bytecode = bytecode!(emit_header(1, 2), emit_halt());
        assert!(verify_bytecode_content(&valid_bytecode).is_ok());
        let parsed = parser::parse_bytecode(&valid_bytecode);
        assert!(parsed.errors.is_empty());

        let invalid_magic = vec![b'B', b'A', b'D', b'X', 0, 1, 2, HALT];
        assert!(verify_bytecode_content(&invalid_magic).is_err());
        let parsed_invalid = parser::parse_bytecode(&invalid_magic);
        assert!(!parsed_invalid.errors.is_empty());

        let too_short = vec![b'5', b'I', b'V', b'E'];
        assert!(verify_bytecode_content(&too_short).is_err());
        let parsed_short = parser::parse_bytecode(&too_short);
        assert!(!parsed_short.errors.is_empty());

        let invalid_opcode = bytecode!(emit_header(1, 2), emit_u8(0xC0));
        assert!(verify_bytecode_content(&invalid_opcode).is_err());
        let parsed_bad_op = parser::parse_bytecode(&invalid_opcode);
        assert!(!parsed_bad_op.errors.is_empty());
    }

    #[test]
    fn test_verifier_alignment_with_golden_fixtures() {
        let valid_bc = valid_header();
        assert!(verify_bytecode_content(&valid_bc).is_ok());
        let parsed = parser::parse_bytecode(&valid_bc);
        assert!(parsed.errors.is_empty());

        let invalid_bc = invalid_call_target();
        assert!(verify_bytecode_content(&invalid_bc).is_err());
        let parsed_invalid = parser::parse_bytecode(&invalid_bc);
        assert!(!parsed_invalid.errors.is_empty());

        let truncated_bc = operand_truncation();
        assert!(verify_bytecode_content(&truncated_bc).is_err());
        let parsed_truncated = parser::parse_bytecode(&truncated_bc);
        assert!(!parsed_truncated.errors.is_empty());
    }
}
