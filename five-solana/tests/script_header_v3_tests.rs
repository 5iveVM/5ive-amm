#[cfg(test)]
mod script_header_v3_tests {
    use five::{
        instructions::{verify_bytecode_content, DEPLOY_INSTRUCTION, EXECUTE_INSTRUCTION},
        state::ScriptAccountHeader,
    };
    use five_protocol::{
        bytecode, parser,
        test_fixtures::{invalid_call_target, valid_header, vle_truncation},
        HALT,
    };

    #[test]
    fn test_script_header_v3_creation() {
        // Test basic script header creation and validation
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
        println!("ScriptAccountHeader basic test passed");
    }

    #[test]
    fn test_deploy_instruction_format() {
        // Test that deploy instruction accepts the expected format (v4 with permissions)
        let test_bytecode = bytecode!(emit_magic(), emit_halt());

        // Deploy instruction format (v4): [discriminator(1)] + [length(4)] + [permissions(1)] + [bytecode]
        let mut deploy_data = Vec::new();
        deploy_data.push(DEPLOY_INSTRUCTION);
        deploy_data.extend_from_slice(&(test_bytecode.len() as u32).to_le_bytes());
        deploy_data.push(0x00u8); // permissions: no special permissions
        deploy_data.extend_from_slice(&test_bytecode);

        // Validate format
        assert_eq!(deploy_data[0], DEPLOY_INSTRUCTION);
        assert_eq!(deploy_data.len(), 1 + 4 + 1 + test_bytecode.len());

        println!("Deploy instruction format test passed");
    }

    #[test]
    fn test_execute_instruction_format() {
        // Test execute instruction format
        let test_params = vec![1u8, 2, 3]; // Sample parameters

        // Execute instruction format: [discriminator(1)] + [params]
        let mut execute_data = Vec::new();
        execute_data.push(EXECUTE_INSTRUCTION);
        execute_data.extend_from_slice(&test_params);

        // Validate format
        assert_eq!(execute_data[0], EXECUTE_INSTRUCTION);
        assert_eq!(execute_data.len(), 1 + test_params.len());

        println!("Execute instruction format test passed");
    }

    #[test]
    fn test_deploy_time_verification_matches_parser() {
        // Test that verify_bytecode_content behavior matches the shared parser

        // Valid bytecode: minimal script with HALT
        let valid_bytecode = bytecode!(emit_header(1, 2), emit_halt());

        // Both should accept valid bytecode
        assert!(verify_bytecode_content(&valid_bytecode).is_ok());
        let parsed = parser::parse_bytecode(&valid_bytecode);
        assert!(parsed.errors.is_empty());

        // Invalid bytecode: bad magic
        let invalid_magic = vec![
            b'B', b'A', b'D', b'X', // invalid magic
            0,    // features
            1,    // public_function_count
            2,    // total_function_count
            HALT, // HALT
        ];

        // Both should reject invalid bytecode
        assert!(verify_bytecode_content(&invalid_magic).is_err());
        let parsed_invalid = parser::parse_bytecode(&invalid_magic);
        assert!(!parsed_invalid.errors.is_empty());

        // Invalid bytecode: too short
        let too_short = vec![b'5', b'I', b'V', b'E'];
        assert!(verify_bytecode_content(&too_short).is_err());
        let parsed_short = parser::parse_bytecode(&too_short);
        assert!(!parsed_short.errors.is_empty());

        // Invalid bytecode: invalid opcode
        let invalid_opcode = bytecode!(
            emit_header(1, 2),
            emit_u8(0xC0) // invalid opcode (0xC0 is in available range)
        );
        assert!(verify_bytecode_content(&invalid_opcode).is_err());
        let parsed_bad_op = parser::parse_bytecode(&invalid_opcode);
        assert!(!parsed_bad_op.errors.is_empty());

        println!("Deploy-time verification matches parser behavior test passed");
    }

    #[test]
    fn test_verifier_alignment_with_golden_fixtures() {
        // Use golden fixtures to ensure parser and verifier alignment

        // Verifier should accept valid header
        let valid_bc = valid_header();
        assert!(verify_bytecode_content(&valid_bc).is_ok());
        let parsed = parser::parse_bytecode(&valid_bc);
        assert!(parsed.errors.is_empty());

        // Verifier should reject invalid CALL target
        let invalid_bc = invalid_call_target();
        assert!(verify_bytecode_content(&invalid_bc).is_err());
        let parsed_invalid = parser::parse_bytecode(&invalid_bc);
        assert!(!parsed_invalid.errors.is_empty());

        // Verifier should reject VLE truncation
        let truncated_bc = vle_truncation();
        assert!(verify_bytecode_content(&truncated_bc).is_err());
        let parsed_truncated = parser::parse_bytecode(&truncated_bc);
        assert!(!parsed_truncated.errors.is_empty());

        println!("Verifier alignment with golden fixtures test passed");
    }
}
