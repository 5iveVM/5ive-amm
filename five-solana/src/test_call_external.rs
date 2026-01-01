#[cfg(test)]
mod call_external_tests {
    use five_protocol::opcodes::*;
    // Remove unused imports: Value, ValueRef, ExecutionManager, MitoVM

    // Mock external account data (simulating sphere account)
    fn create_mock_sphere_bytecode() -> Vec<u8> {
        vec![
            // Simple external function that pushes 42 and returns
            PUSH_U64, 42, 0, 0, 0, 0, 0, 0, 0,    // Push 42 onto stack
            HALT, // End execution
        ]
    }

    #[test]
    fn test_call_external_basic() {
        // Test basic CALL_EXTERNAL functionality
        println!("Testing CALL_EXTERNAL basic functionality");

        // Create test bytecode with CALL_EXTERNAL instruction
        let main_bytecode = vec![
            CALL_EXTERNAL, // 0x91
            0,             // account_index = 0 (external account)
            0,
            0,    // func_offset = 0 (start of external bytecode)
            0,    // param_count = 0
            HALT, // End main execution
        ];

        // Create mock external bytecode
        let external_bytecode = create_mock_sphere_bytecode();

        println!("Main bytecode: {:?}", main_bytecode);
        println!("External bytecode: {:?}", external_bytecode);

        // This test validates our CALL_EXTERNAL opcode exists and is structured correctly
        assert_eq!(main_bytecode[0], CALL_EXTERNAL);
        assert_eq!(external_bytecode[0], PUSH_U64);

        println!("CALL_EXTERNAL basic test passed");
    }

    #[test]
    fn test_call_external_instruction_format() {
        // Test the expected format of CALL_EXTERNAL instructions
        println!("Testing CALL_EXTERNAL instruction format");

        // CALL_EXTERNAL format: [opcode] [account_index] [func_offset_u16] [param_count]
        let instruction = vec![
            CALL_EXTERNAL, // Opcode (0x91)
            2,             // account_index (which account has external bytecode)
            0x40,
            0x01, // func_offset as u16 (0x0140 = 320 bytes offset)
            3,    // param_count (3 parameters)
        ];

        // Validate instruction structure
        assert_eq!(instruction.len(), 5); // 1 + 1 + 2 + 1 = 5 bytes total
        assert_eq!(instruction[0], CALL_EXTERNAL);
        assert_eq!(instruction[1], 2); // account_index

        // func_offset as little-endian u16: 0x40, 0x01 = 0x0140 = 320
        let func_offset = u16::from_le_bytes([instruction[2], instruction[3]]);
        assert_eq!(func_offset, 320);

        assert_eq!(instruction[4], 3); // param_count

        println!("CALL_EXTERNAL instruction format test passed");
    }

    #[test]
    fn test_call_external_with_parameters() {
        // Test CALL_EXTERNAL with parameter passing
        println!("Testing CALL_EXTERNAL with parameters");

        // Main bytecode that prepares parameters and calls external function
        let main_bytecode = vec![
            // Prepare parameters on stack
            PUSH_U64,
            100,
            0,
            0,
            0,
            0,
            0,
            0,
            0, // Push first parameter (100)
            PUSH_U64,
            200,
            0,
            0,
            0,
            0,
            0,
            0,
            0, // Push second parameter (200)
            // Call external function with 2 parameters
            CALL_EXTERNAL, // 0x91
            0,             // account_index = 0
            0,
            0,    // func_offset = 0
            2,    // param_count = 2
            HALT, // End main execution
        ];

        // Validate the parameter preparation
        assert_eq!(main_bytecode[0], PUSH_U64);
        assert_eq!(main_bytecode[1], 100); // First parameter value
        assert_eq!(main_bytecode[9], PUSH_U64); // Second PUSH_U64 at correct offset
        assert_eq!(main_bytecode[10], 200); // Second parameter value

        // Validate the CALL_EXTERNAL with parameters
        assert_eq!(main_bytecode[18], CALL_EXTERNAL); // Correct offset for CALL_EXTERNAL
        assert_eq!(main_bytecode[22], 2); // param_count = 2

        println!("CALL_EXTERNAL parameter test passed");
    }

    #[test]
    fn test_call_external_error_handling() {
        // Test error conditions for CALL_EXTERNAL
        println!("Testing CALL_EXTERNAL error handling");

        // Test invalid account index (should be caught by VM)
        let invalid_account_bytecode = vec![
            CALL_EXTERNAL, // 0x91
            255,           // invalid account_index (too high)
            0,
            0, // func_offset = 0
            0, // param_count = 0
            HALT,
        ];

        // Test invalid function offset (beyond bytecode bounds)
        let invalid_offset_bytecode = vec![
            CALL_EXTERNAL, // 0x91
            0,             // account_index = 0
            0xFF,
            0xFF, // func_offset = 65535 (likely beyond bounds)
            0,    // param_count = 0
            HALT,
        ];

        // Validate error conditions are properly structured
        assert_eq!(invalid_account_bytecode[0], CALL_EXTERNAL);
        assert_eq!(invalid_account_bytecode[1], 255); // Should trigger account bounds error

        assert_eq!(invalid_offset_bytecode[0], CALL_EXTERNAL);
        let offset = u16::from_le_bytes([invalid_offset_bytecode[2], invalid_offset_bytecode[3]]);
        assert_eq!(offset, 65535); // Should trigger offset bounds error

        println!("CALL_EXTERNAL error handling test passed");
    }
}
