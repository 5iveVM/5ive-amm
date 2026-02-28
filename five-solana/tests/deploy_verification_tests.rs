#[cfg(test)]
mod deploy_verification_tests {
    use five::instructions::verify_bytecode_content;
    use five_dsl_compiler::bytecode_generator::disassembler::BytecodeInspector;
    use five_dsl_compiler::DslCompiler;
    use five_protocol::{
        bytecode,
        opcodes::*,
        test_fixtures::{invalid_call_target, operand_truncation, valid_header},
    };

    #[test]
    fn test_verify_valid_minimal_bytecode() {
        // Create minimal valid bytecode with header + HALT
        let bytecode_data = bytecode!(emit_header(0, 0), emit_halt());
        assert!(verify_bytecode_content(&bytecode_data).is_ok());
    }

    #[test]
    fn test_verify_valid_bytecode_with_call() {
        // Create bytecode with header + valid CALL to function address 0 with 0 params
        let bytecode_data = bytecode!(emit_header(1, 1), emit_call(0, 0), emit_halt());
        assert!(verify_bytecode_content(&bytecode_data).is_ok());
    }

    #[test]
    fn test_verify_invalid_call_out_of_bounds() {
        // Create bytecode with CALL pointing to function address > bytecode length (invalid)
        let bytecode_data = bytecode!(
            emit_header(0, 0),
            emit_call(200, 0) // calling address 200, which is outside bytecode len (~14)
        );
        assert!(verify_bytecode_content(&bytecode_data).is_err());
    }

    #[test]
    fn test_verify_invalid_opcode() {
        // Create bytecode with unknown opcode (0x7A is not defined in OPCODE_TABLE)
        // Reserved for future constraint operations but currently undefined
        let bytecode_data = bytecode!(
            emit_header(0, 0),
            emit_u8(0x7A) // invalid opcode (not in OPCODE_TABLE)
        );
        assert!(verify_bytecode_content(&bytecode_data).is_err());
    }

    #[test]
    fn test_verify_incomplete_instruction() {
        // Create bytecode with truncated PUSH_U64 instruction
        let bytecode_data = bytecode!(
            emit_header(0, 0),
            emit_u8(PUSH_U64),
            emit_u8(1),
            emit_u8(2),
            emit_u8(3) // incomplete - needs 8 bytes total
        );
        assert!(verify_bytecode_content(&bytecode_data).is_err());
    }

    #[test]
    fn test_verify_arithmetic_operations() {
        // Test ADD instruction
        let bytecode_data = bytecode!(emit_header(0, 0), emit_u8(ADD), emit_halt());
        assert!(verify_bytecode_content(&bytecode_data).is_ok());
    }

    #[test]
    fn test_verify_local_operations() {
        // Test LOAD_PARAM, GET_LOCAL, SET_LOCAL
        let bytecode_data = bytecode!(
            emit_header(0, 0),
            emit_load_param(1),
            emit_set_local(0),
            emit_get_local(0),
            emit_store_param(0),
            emit_halt()
        );
        assert!(verify_bytecode_content(&bytecode_data).is_ok());
    }

    #[test]
    fn test_verify_header_too_small() {
        // Bytecode too small for header
        let bytecode_data = vec![b'5', b'I', b'V', b'E', 0x00];
        assert!(verify_bytecode_content(&bytecode_data).is_err());
    }

    #[test]
    fn test_verify_incomplete_call() {
        // CALL with incomplete bytes
        let bytecode_data = bytecode!(
            emit_header(0, 0),
            emit_u8(CALL) // incomplete - needs fixed-size function index
        );
        assert!(verify_bytecode_content(&bytecode_data).is_err());
    }

    #[test]
    fn test_verify_push_u64_boundary() {
        // PUSH_U64 with full 8-byte argument
        let bytecode_data = bytecode!(emit_header(0, 0), emit_push_u64(42));
        assert!(verify_bytecode_content(&bytecode_data).is_ok());
    }

    #[test]
    fn test_rejects_stale_empty_function_fixture() {
        let bytecode_data = vec![
            0x35, 0x49, 0x56, 0x45, 0x4f, 0x01, 0x00, 0x00, 0x01, 0x01, 0x06, 0x01, 0x04, 0x74,
            0x65, 0x73, 0x74, 0x19, 0x64, 0x07, 0x00,
        ];

        let parsed = five_protocol::parser::parse_bytecode(&bytecode_data);
        assert!(
            !parsed.errors.is_empty(),
            "legacy fixture should be parser-invalid until regenerated"
        );
        assert!(
            parsed
                .errors
                .iter()
                .any(|e| matches!(e, five_protocol::parser::ParseError::HeaderTooShort)),
            "expected HeaderTooShort from legacy fixture, got {:?}",
            parsed.errors
        );
        assert!(
            verify_bytecode_content(&bytecode_data).is_err(),
            "legacy fixture should be deploy-verifier invalid until regenerated"
        );
    }

    #[test]
    fn test_verify_public_gt_total() {
        // Create bytecode with public_function_count (2) > total_function_count (1)
        // emit_header(public, total)
        let bytecode_data = bytecode!(emit_header(2, 1), emit_halt());
        // Should return Err(ProgramError::Custom(8105))
        let result = verify_bytecode_content(&bytecode_data);
        assert!(matches!(
            result,
            Err(pinocchio::program_error::ProgramError::Custom(8105))
        ));
    }

    #[test]
    fn test_verify_public_zero_but_total_nonzero() {
        // Create bytecode with public_function_count (0) and total_function_count (1)
        let bytecode_data = bytecode!(emit_header(0, 1), emit_halt());
        // Should return Err(ProgramError::Custom(8104))
        let result = verify_bytecode_content(&bytecode_data);
        assert!(matches!(
            result,
            Err(pinocchio::program_error::ProgramError::Custom(8104))
        ));
    }

    #[test]
    fn test_verify_bytecode_too_large() {
        // Mock MAX_SCRIPT_SIZE check
        // We can't easily allocate >10KB in test without being wasteful, but we can check if it returns 8101
        // when we exceed five_protocol::MAX_SCRIPT_SIZE.
        // MAX_SCRIPT_SIZE is 1024 * 10 (10KB).

        let large_bytecode = vec![0u8; five_protocol::MAX_SCRIPT_SIZE + 1];
        let result = verify_bytecode_content(&large_bytecode);
        assert!(matches!(
            result,
            Err(pinocchio::program_error::ProgramError::Custom(8101))
        ));
    }

    #[test]
    fn verifier_and_parser_align_on_shared_fixtures() {
        let valid = valid_header();
        assert!(verify_bytecode_content(&valid).is_ok());
        assert!(five_protocol::parser::parse_bytecode(&valid)
            .errors
            .is_empty());

        let invalid_call = invalid_call_target();
        assert!(verify_bytecode_content(&invalid_call).is_err());
        assert!(!five_protocol::parser::parse_bytecode(&invalid_call)
            .errors
            .is_empty());

        let truncated = operand_truncation();
        assert!(verify_bytecode_content(&truncated).is_err());
        assert!(!five_protocol::parser::parse_bytecode(&truncated)
            .errors
            .is_empty());
    }

    #[test]
    fn verifier_accepts_push_bytes_bytecode() {
        let source = r#"
pub probe() -> u64 {
    let payload: [u8; 64] = [
        0, 1, 2, 3, 4, 5, 6, 7,
        8, 9, 10, 11, 12, 13, 14, 15,
        16, 17, 18, 19, 20, 21, 22, 23,
        24, 25, 26, 27, 28, 29, 30, 31,
        32, 33, 34, 35, 36, 37, 38, 39,
        40, 41, 42, 43, 44, 45, 46, 47,
        48, 49, 50, 51, 52, 53, 54, 55,
        56, 57, 58, 59, 60, 61, 62, 63
    ];
    return 1;
}
"#;

        let bytecode = DslCompiler::compile_dsl(source).expect("compile typed byte literal");
        let inspector = BytecodeInspector::new(&bytecode);

        assert!(
            inspector.contains_opcode(PUSH_BYTES) || inspector.contains_opcode(PUSH_BYTES_W),
            "expected compiled bytecode to use PUSH_BYTES/PUSH_BYTES_W"
        );
        assert!(verify_bytecode_content(&bytecode).is_ok());
    }
}
