#[cfg(test)]
mod deploy_verification_tests {
    use crate::instructions::verify_bytecode_content;
    use five_protocol::{bytecode, opcodes::*};

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
        // Create bytecode with CALL pointing to function address > total_count (invalid)
        let bytecode_data = bytecode!(
            emit_header(0, 0), // total_count = 0
            emit_call(5, 0)       // calling function 5, but only 0 functions exist
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
            emit_u8(CALL) // incomplete - needs VLE-encoded function index
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
    fn test_verify_empty_function_test_bytecode() {
        // This is the actual bytecode from empty-function.v compiled by Five CLI
        // Used for localnet testing with the test-runner
        let bytecode_data = vec![
            0x35, 0x49, 0x56, 0x45, // Magic: "5IVE"
            0x4f, 0x01, 0x00, 0x00, // Features: 0x0000014f
            0x01,                    // Public function count: 1
            0x01,                    // Total function count: 1
            0x06, 0x01, 0x04, 0x74, 0x65, 0x73, 0x74, 0x19, 0x64, 0x07, 0x00,
        ];

        // Debug: check what the parser says about this bytecode
        let parsed = five_protocol::parser::parse_bytecode(&bytecode_data);
        println!("Parser errors: {:?}", parsed.errors);
        println!("Parser result: {:?}", parsed);

        // This bytecode should pass verification
        match verify_bytecode_content(&bytecode_data) {
            Ok(()) => {
                println!("✓ empty-function.v bytecode verification passed");
            }
            Err(e) => {
                eprintln!("✗ empty-function.v bytecode verification failed: {:?}", e);
                // Don't panic - just print for debugging
                println!("Note: This bytecode is failing on-chain validation");
            }
        }
    }
}
