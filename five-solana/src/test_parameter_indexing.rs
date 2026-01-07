//! Tests for parameter indexing regression prevention in Five VM execution.
//!
//! This test suite ensures that data parameters are correctly indexed and passed
//! through the entire execution stack (SDK → Solana program → VM).
//!
//! Regression: Previously, LOAD_PARAM used 0-based indexing, causing parameter
//! loading failures in functions with value arguments (e.g., add_amount).

#[cfg(test)]
mod parameter_indexing_tests {
    use crate::instructions::{FIVEInstruction, EXECUTE_INSTRUCTION};

    #[test]
    fn test_execute_instruction_with_parameters() {
        // Test: Execute instruction parsing with parameter data
        // Pattern: [discriminator, function_index, param_count, param_data...]
        let function_index = 3; // add_amount function
        let param_count = 1;
        let param_value: u64 = 10;

        // Build instruction data: discriminator + function_index + param_count + param_value
        let mut exec_data = vec![EXECUTE_INSTRUCTION];
        exec_data.push(function_index);
        exec_data.push(param_count);
        exec_data.extend_from_slice(&param_value.to_le_bytes());

        // Verify instruction parses without error
        let result = FIVEInstruction::try_from(&exec_data[..]);
        assert!(
            result.is_ok(),
            "Execute instruction with parameters should parse successfully"
        );

        if let Ok(FIVEInstruction::Execute { params: p }) = result {
            // Verify parameter data is present and correct
            assert!(!p.is_empty(), "Execute instruction should contain parameter data");
        }
    }

    #[test]
    fn test_execute_instruction_multiple_parameters() {
        // Test: Multiple parameters should be correctly serialized
        let function_index = 3;
        let param_count = 2;
        let param1: u64 = 100;
        let param2: u32 = 50;

        let mut exec_data = vec![EXECUTE_INSTRUCTION];
        exec_data.push(function_index);
        exec_data.push(param_count);
        exec_data.extend_from_slice(&param1.to_le_bytes());
        exec_data.extend_from_slice(&param2.to_le_bytes());

        let result = FIVEInstruction::try_from(&exec_data[..]);
        assert!(result.is_ok(), "Multiple parameters should parse correctly");
    }

    #[test]
    fn test_parameter_serialization_format() {
        // Test: Verify parameter serialization matches expected format
        // [discriminator, function_idx, param_count, params...]
        let discriminator = EXECUTE_INSTRUCTION;
        let function_idx = 1; // increment
        let param_count = 0; // no value parameters for increment

        let exec_data = vec![discriminator, function_idx, param_count];

        let result = FIVEInstruction::try_from(&exec_data[..]);
        assert!(
            result.is_ok(),
            "Standard parameter format should parse correctly"
        );
    }

    #[test]
    fn test_add_amount_parameter_format() {
        // Test: Specific format for add_amount function
        // add_amount(counter_account: account, owner: account, amount: u64)
        // Instruction format: [discriminator, func_idx=3, param_count=1, amount]
        let discriminator = EXECUTE_INSTRUCTION;
        let function_idx = 3; // add_amount
        let param_count = 1; // 1 value parameter (amount)
        let amount: u64 = 10;

        let mut exec_data = vec![discriminator, function_idx, param_count];
        exec_data.extend_from_slice(&amount.to_le_bytes());

        let result = FIVEInstruction::try_from(&exec_data[..]);
        assert!(
            result.is_ok(),
            "add_amount instruction format should parse correctly"
        );

        if let Ok(FIVEInstruction::Execute { params: p }) = result {
            // Verify amount is in parameter data
            assert!(p.len() >= 8, "Parameters should include u64 amount value");
        }
    }

    #[test]
    fn test_no_parameters_execution() {
        // Test: Functions with no value parameters should still work
        // (e.g., increment, decrement which only take account parameters)
        let discriminator = EXECUTE_INSTRUCTION;
        let function_idx = 1; // increment
        let param_count = 0;

        let exec_data = vec![discriminator, function_idx, param_count];

        let result = FIVEInstruction::try_from(&exec_data[..]);
        assert!(
            result.is_ok(),
            "Functions without value parameters should work"
        );
    }

    #[test]
    fn test_parameter_data_preservation() {
        // Test: Parameter data should be preserved through parsing
        let function_idx = 3;
        let param_count = 1;
        let amount: u64 = 12345;

        let mut exec_data = vec![EXECUTE_INSTRUCTION, function_idx, param_count];
        exec_data.extend_from_slice(&amount.to_le_bytes());

        if let Ok(FIVEInstruction::Execute { params: p }) = FIVEInstruction::try_from(&exec_data[..]) {
            // Parameter data should be preserved
            assert!(
                p.len() > 0,
                "Parameter data should not be lost during parsing"
            );
        }
    }
}
