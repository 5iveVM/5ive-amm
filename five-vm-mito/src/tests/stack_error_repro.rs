
#[cfg(test)]
mod tests {
    use crate::tests::framework::TestUtils;
    
    #[test]
    fn test_repro_call_stack_error() {
        // Reproduce: Stack size 12, Param count 8.
        
        let mut main_code = Vec::new();
        // Push 12 items (0 to 11)
        for i in 0..12 {
            main_code.push(0x1C); // PUSH_I64
            // Manual VLE for small values (< 128) is just the byte
            main_code.push(i as u8); 
        }

        // CALL with param_count 8, targeting function_code
        // function_code starts at offset 10 (header) + main_code.len() + 4 (CALL) + 1 (metadata) + 1 (RETURN_VALUE)
        // We need to account for the CALL instruction and metadata when calculating offset
        let call_instruction_len = 4 + 1 + 1; // CALL (4 bytes) + metadata (1 byte) + RETURN (1 byte)
        let func_offset = 10 + main_code.len() + call_instruction_len;
        main_code.extend_from_slice(&[
            0x90, // CALL opcode
            0x08, // param_count = 8
            (func_offset & 0xFF) as u8, // low byte of offset
            ((func_offset >> 8) & 0xFF) as u8, // high byte of offset
        ]);
        // Inline metadata (required when FEATURE_FUNCTION_METADATA is set)
        // 0x00 = no function name (marker byte indicating 0 bytes follow)
        main_code.push(0x00);
        main_code.push(0x07); // RETURN_VALUE after call returns

        // Function code: just HALT to terminate cleanly
        let function_code = [0x00]; // HALT

        let bytecode = TestUtils::create_function_bytecode(&main_code, &function_code);
        let result = TestUtils::execute_simple(&bytecode);
        
        // This should SUCCEED if logic is correct (12 >= 8).
        if let Err(e) = result {
             panic!("Test Failed with error: {:?}", e);
        }
    }
}
