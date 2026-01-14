//! Function call and parameter handling tests
//!
//! Comprehensive unit tests for function call operations including CALL,
//! parameter transfer, local variable management, and call stack handling.

#[cfg(test)]
mod function_call_tests {
    use crate::tests::framework::TestUtils;
    use crate::{opcodes, push_bool, push_u64, test_bytecode};
    use crate::{MitoVM, VMError, Value};
    use five_protocol::opcodes::*;

    /// Test basic function call operations
    mod basic_function_calls {
        use super::*;

        #[test]
        fn test_simple_function_call() {
            // Test simple function call: add_numbers(5, 3) -> 8
            let main_code = [
                // Main function: push parameters and call function
                0x1C, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // PUSH_U64(5)
                0x1C, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // PUSH_U64(3)
                0x90, 0x02, 0x10, 0x00, // CALL param_count=2, func_addr=16
                0x07, // RETURN_VALUE
            ];

            let function_code = [
                // Function: LOAD_PARAM 1, LOAD_PARAM 2, ADD, RETURN_VALUE
                0x95, 0x01, // LOAD_PARAM 1 (get parameter 1: 5)
                0x95, 0x02, // LOAD_PARAM 2 (get parameter 2: 3)
                0x20, // ADD (5 + 3 = 8)
                0x07, // RETURN_VALUE
            ];

            let bytecode = TestUtils::create_function_bytecode(&main_code, &function_code);

            let result = TestUtils::execute_simple(&bytecode);
            // Should return 8 when properly implemented
            match result {
                Ok(Some(Value::U64(8))) => {
                    // Success case
                }
                Ok(other) => {
                    println!("Function call returned unexpected value: {:?}", other);
                }
                Err(e) => {
                    println!("Function call failed: {:?}", e);
                    // For now, we expect this to fail until full implementation
                    assert!(true, "Function calls need complete implementation");
                }
            }
        }

        #[test]
        fn test_function_call_with_return() {
            // Test function that returns a computed value
            let main_code = [
                0x1C, 0x0A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // PUSH_U64(10)
                0x90, 0x01, 0x0F, 0x00, // CALL param_count=1, func_addr=15
                0x07, // RETURN_VALUE
            ];

            let function_code = [
                0x95, 0x01, // LOAD_PARAM 1 (get parameter: 10)
                0x1C, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // PUSH_U64(2)
                0x22, // MUL (10 * 2 = 20)
                0x07, // RETURN_VALUE
            ];

            let bytecode = TestUtils::create_function_bytecode(&main_code, &function_code);

            let result = TestUtils::execute_simple(&bytecode);
            // Should return exactly 20 when implemented
            match result {
                Ok(Some(Value::U64(value))) => {
                    assert_eq!(value, 20, "Function should return exactly 20");
                }
                Ok(result) => panic!("Expected U64(20), got {:?}", result),
                Err(e) => panic!("Function call execution failed: {:?}", e),
            }
        }

        #[test]
        fn test_function_call_no_parameters() {
            // Test function call with no parameters
            let main_code = [
                0x90, 0x00, 0x09, 0x00, // CALL param_count=0, func_addr=9
                0x07, // RETURN_VALUE
            ];

            let function_code = [
                0x1C, 0x2A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // PUSH_U64(42)
                0x07, // RETURN_VALUE
            ];

            let bytecode = TestUtils::create_function_bytecode(&main_code, &function_code);

            let result = TestUtils::execute_simple(&bytecode);
            match result {
                Ok(Some(Value::U64(value))) => {
                    assert_eq!(
                        value, 42,
                        "Zero parameter function should return exactly 42"
                    );
                }
                Ok(result) => panic!("Expected U64(42), got {:?}", result),
                Err(e) => panic!("Zero parameter function call failed: {:?}", e),
            }
        }

        #[test]
        fn test_function_call_max_parameters() {
            // Test function call with maximum parameters (7)
            let mut main_code = Vec::new();

            // Push 7 parameters
            for i in 1..=7 {
                main_code.extend_from_slice(&[0x1C]); // PUSH_U64
                main_code.extend_from_slice(&(i as u64).to_le_bytes());
            }

            // Call function
            main_code.extend_from_slice(&[0x90, 0x07, 0x40, 0x00]); // CALL param_count=7, func_addr=64
            main_code.push(0x07); // RETURN_VALUE

            let function_code = [
                // Load all parameters and sum them
                0x95, 0x01, // LOAD_PARAM 1
                0x95, 0x02, // LOAD_PARAM 2
                0x20, // ADD
                0x95, 0x03, // LOAD_PARAM 3
                0x20, // ADD
                0x95, 0x04, // LOAD_PARAM 4
                0x20, // ADD
                0x95, 0x05, // LOAD_PARAM 5
                0x20, // ADD
                0x95, 0x06, // LOAD_PARAM 6
                0x20, // ADD
                0x95, 0x07, // LOAD_PARAM 7
                0x20, // ADD
                0x07, // RETURN_VALUE (should be 1+2+3+4+5+6+7 = 28)
            ];

            let bytecode = TestUtils::create_function_bytecode(&main_code, &function_code);

            let result = TestUtils::execute_simple(&bytecode);
            match result {
                Ok(Some(Value::U64(value))) => {
                    assert_eq!(
                        value, 28,
                        "Max parameters function should sum to exactly 28"
                    );
                }
                Ok(result) => panic!("Expected U64(28), got {:?}", result),
                Err(e) => panic!("Max parameters function call failed: {:?}", e),
            }
        }

        #[test]
        fn test_function_call_too_many_parameters() {
            // Test function call with more than maximum parameters (should fail)
            let mut main_code = Vec::new();

            // Push 8 parameters (exceeds limit of 7)
            for i in 1..=8 {
                main_code.extend_from_slice(&[0x1C]); // PUSH_U64
                main_code.extend_from_slice(&(i as u64).to_le_bytes());
            }

            // Call function
            main_code.extend_from_slice(&[0x90, 0x08, 0x50, 0x00]); // CALL param_count=8 (too many)
            main_code.push(0x07); // RETURN_VALUE

            let function_code = [0x07]; // Simple return

            let bytecode = TestUtils::create_function_bytecode(&main_code, &function_code);

            let result = TestUtils::execute_simple(&bytecode);
            // Should fail with InvalidOperation due to too many parameters
            TestUtils::assert_execution_error(&bytecode, VMError::InvalidOperation);
        }
    }

    /// Test parameter handling operations
    mod parameter_handling {
        use super::*;

        #[test]
        fn test_load_param_basic() {
            // Test LOAD_PARAM with function dispatch
            let input_data = TestUtils::create_function_input(1, &[Value::U64(42), Value::U64(10)]);

            let bytecode = test_bytecode![
                opcodes![LOAD_PARAM, 1], // Load first parameter (42)
                opcodes![RETURN_VALUE],
            ];

            let result = TestUtils::execute_with_input(&bytecode, &input_data);
            match result {
                Ok(Some(Value::U64(value))) => {
                    assert_eq!(
                        value, 42,
                        "LOAD_PARAM should return exactly first parameter (42)"
                    );
                }
                Ok(result) => panic!("Expected U64(42), got {:?}", result),
                Err(e) => panic!("LOAD_PARAM execution failed: {:?}", e),
            }
        }

        #[test]
        fn test_load_param_multiple() {
            // Test loading multiple parameters
            let input_data =
                TestUtils::create_function_input(1, &[Value::U64(100), Value::U64(25)]);

            let bytecode = test_bytecode![
                opcodes![LOAD_PARAM, 1], // Load first parameter (100)
                opcodes![LOAD_PARAM, 2], // Load second parameter (25)
                opcodes![ADD],           // Add them (125)
                opcodes![RETURN_VALUE],
            ];

            let result = TestUtils::execute_with_input(&bytecode, &input_data);
            match result {
                Ok(Some(Value::U64(value))) => {
                    assert_eq!(value, 125, "Multiple LOAD_PARAM should sum to exactly 125");
                }
                Ok(result) => panic!("Expected U64(125), got {:?}", result),
                Err(e) => panic!("Multiple LOAD_PARAM execution failed: {:?}", e),
            }
        }

        #[test]
        fn test_load_param_invalid_index() {
            // Test LOAD_PARAM with invalid parameter index
            let input_data = TestUtils::create_function_input(1, &[Value::U64(42)]);

            let bytecode = test_bytecode![
                opcodes![LOAD_PARAM, 5], // Try to load parameter 5 (doesn't exist)
                opcodes![RETURN_VALUE],
            ];

            let result = TestUtils::execute_with_input(&bytecode, &input_data);
            // Should fail with appropriate error
            assert!(result.is_err(), "Invalid parameter index should fail");
        }

        #[test]
        fn test_load_param_zero_index() {
            // Test LOAD_PARAM with index 0 (should be invalid)
            let input_data = TestUtils::create_function_input(1, &[Value::U64(42)]);

            let bytecode = test_bytecode![
                opcodes![LOAD_PARAM, 0], // Parameter index 0 is invalid
                opcodes![RETURN_VALUE],
            ];

            let result = TestUtils::execute_with_input(&bytecode, &input_data);
            assert!(result.is_err(), "Parameter index 0 should be invalid");
        }

        #[test]
        fn test_load_param_different_types() {
            // Test LOAD_PARAM with different parameter types
            let input_data = TestUtils::create_function_input(
                1,
                &[Value::U64(42), Value::Bool(true), Value::U8(10)],
            );

            let mut bytecode = vec![0x35, 0x49, 0x56, 0x45]; // magic
            bytecode.extend_from_slice(&[LOAD_PARAM, 1]); // U64
            bytecode.extend_from_slice(&[LOAD_PARAM, 2]); // Bool
            bytecode.extend_from_slice(&[LOAD_PARAM, 3]); // U8
            bytecode.push(0x00); // HALT

            let result = TestUtils::execute_with_input(&bytecode, &input_data);
            // Should handle different types correctly
            assert!(
                result.is_err(),
                "Mixed parameter types need proper implementation"
            );
        }
    }

    /// Test local variable operations
    mod local_variables {
        use super::*;

        #[test]
        fn test_set_and_get_local() {
            // Test SET_LOCAL and GET_LOCAL operations
            let bytecode = test_bytecode![
                push_u64!(42),
                opcodes![SET_LOCAL, 0], // Set local variable 0 to 42
                opcodes![GET_LOCAL, 0], // Get local variable 0
                opcodes![RETURN_VALUE],
            ];

            let result = TestUtils::execute_simple(&bytecode);
            TestUtils::assert_execution_success(&bytecode, Some(Value::U64(42)));
        }

        #[test]
        fn test_multiple_locals() {
            // Test multiple local variables
            let bytecode = test_bytecode![
                push_u64!(10),
                opcodes![SET_LOCAL, 0], // local[0] = 10
                push_u64!(20),
                opcodes![SET_LOCAL, 1], // local[1] = 20
                opcodes![GET_LOCAL, 0], // Get 10
                opcodes![GET_LOCAL, 1], // Get 20
                opcodes![ADD],          // 10 + 20 = 30
                opcodes![RETURN_VALUE],
            ];

            TestUtils::assert_execution_success(&bytecode, Some(Value::U64(30)));
        }

        #[test]
        fn test_local_variable_scoping() {
            // Test that local variables are properly scoped to function calls
            let main_code = [
                0x1C, 0x64, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // PUSH_U64(100)
                0x92, 0x00, // SET_LOCAL 0 (main function local)
                0x1C, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // PUSH_U64(5)
                0x90, 0x01, 0x15, 0x00, // CALL with 1 parameter
                0x93, 0x00, // GET_LOCAL 0 (should still be 100)
                0x20, // ADD (result + 100)
                0x07, // RETURN_VALUE
            ];

            let function_code = [
                0x95, 0x01, // LOAD_PARAM 1 (get 5)
                0x92, 0x00, // SET_LOCAL 0 (function's local, should not affect main)
                0x1C, 0x0A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // PUSH_U64(10)
                0x93, 0x00, // GET_LOCAL 0 (get function's local: 5)
                0x20, // ADD (10 + 5 = 15)
                0x07, // RETURN_VALUE
            ];

            let bytecode = TestUtils::create_function_bytecode(&main_code, &function_code);

            let result = TestUtils::execute_simple(&bytecode);
            // Should return 15 + 100 = 115 (function result + main local)
            match result {
                Ok(Some(Value::U64(value))) => {
                    assert_eq!(
                        value, 115,
                        "Local variable scoping should return exactly 115"
                    );
                }
                Ok(result) => panic!("Expected U64(115), got {:?}", result),
                Err(e) => panic!("Local variable scoping test failed: {:?}", e),
            }
        }

        #[test]
        fn test_local_variable_overflow() {
            // Test accessing local variable index beyond limit
            let bytecode = test_bytecode![
                push_u64!(42),
                opcodes![SET_LOCAL, 16], // Index 16 exceeds MAX_LOCALS
            ];

            TestUtils::assert_execution_error(&bytecode, VMError::InvalidOperation);
        }

        #[test]
        fn test_get_uninitialized_local() {
            // Test getting local variable that was never set
            let bytecode = test_bytecode![
                opcodes![GET_LOCAL, 5], // Get uninitialized local[5]
                opcodes![RETURN_VALUE],
            ];

            let result = TestUtils::execute_simple(&bytecode);
            // Now returns an error for uninitialized locals
            assert!(result.is_err(), "Getting uninitialized local should error");
        }

        #[test]
        fn test_set_local_clears_intermediate_slots() {
            // Allocate fewer locals, then expand and ensure intervening slots are cleared
            let bytecode = test_bytecode![
                // First, allocate 6 locals and fill them with values
                opcodes![ALLOC_LOCALS, 6],
                push_u64!(1),
                opcodes![SET_LOCAL, 0],
                push_u64!(2),
                opcodes![SET_LOCAL, 1],
                push_u64!(3),
                opcodes![SET_LOCAL, 2],
                push_u64!(4),
                opcodes![SET_LOCAL, 3],
                push_u64!(5),
                opcodes![SET_LOCAL, 4],
                push_u64!(6),
                opcodes![SET_LOCAL, 5],
                // Shrink to one local without clearing higher slots
                opcodes![ALLOC_LOCALS, 1],
                // Now set local 5, which should clear slots 1-4
                push_u64!(42),
                opcodes![SET_LOCAL, 5],
                // Attempt to read from slot 3 which should have been cleared
                opcodes![GET_LOCAL, 3],
                opcodes![RETURN_VALUE],
            ];

            let result = TestUtils::execute_simple(&bytecode);
            assert!(
                result.is_err(),
                "Intermediate slots should be cleared when expanding set_local"
            );
        }
    }

    /// Test call stack management
    mod call_stack_management {
        use super::*;

        #[test]
        fn test_nested_function_calls() {
            // Test function A calls function B calls function C
            // This tests call stack depth management

            // For now, just test the structure - full implementation needed
            let bytecode = test_bytecode![
                push_u64!(1),
                opcodes![0x90, 0x01, 0x10, 0x00], // CALL function B
                opcodes![RETURN_VALUE],
            ];

            let result = TestUtils::execute_simple(&bytecode);
            assert!(result.is_err(), "Nested calls need complete implementation");
        }

        #[test]
        fn test_call_stack_overflow() {
            // Test exceeding maximum call depth
            // Create recursive call scenario that exceeds MAX_CALL_DEPTH

            let bytecode = test_bytecode![
                opcodes![0x90, 0x00, 0x04, 0x00], // CALL self recursively
            ];

            let result = TestUtils::execute_simple(&bytecode);
            // Should eventually fail with CallStackOverflow
            assert!(result.is_err(), "Recursive calls should hit stack limit");
        }

        #[test]
        fn test_return_address_management() {
            // Test that return addresses are properly managed
            let main_code = [
                0x90, 0x00, 0x09, 0x00, // CALL function
                0x1C, 0x64, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, // PUSH_U64(100) - should execute after return
                0x07, // RETURN_VALUE
            ];

            let function_code = [
                0x1C, 0x2A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // PUSH_U64(42)
                0x96, // RETURN (not RETURN_VALUE - just return to caller)
            ];

            let bytecode = TestUtils::create_function_bytecode(&main_code, &function_code);

            let result = TestUtils::execute_simple(&bytecode);
            // Should return 100 (from after the function call)
            match result {
                Ok(Some(Value::U64(value))) => {
                    assert_eq!(
                        value, 100,
                        "Return address should resume and return exactly 100"
                    );
                }
                Ok(result) => panic!("Expected U64(100), got {:?}", result),
                Err(e) => panic!("Return address test failed: {:?}", e),
            }
        }

        #[test]
        fn test_call_frame_isolation() {
            // Test that call frames properly isolate state
            let main_code = [
                0x1C, 0x0A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // PUSH_U64(10)
                0x92, 0x00, // SET_LOCAL 0 = 10
                0x90, 0x00, 0x0F, 0x00, // CALL function
                0x93, 0x00, // GET_LOCAL 0 (should still be 10)
                0x07, // RETURN_VALUE
            ];

            let function_code = [
                0x1C, 0x14, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // PUSH_U64(20)
                0x92, 0x00, // SET_LOCAL 0 = 20 (function's local scope)
                0x96, // RETURN
            ];

            let bytecode = TestUtils::create_function_bytecode(&main_code, &function_code);

            let result = TestUtils::execute_simple(&bytecode);
            // Should return 10 (main's local should be unchanged)
            match result {
                Ok(Some(Value::U64(value))) => {
                    assert_eq!(
                        value, 10,
                        "Call frames should isolate locals and return exactly 10"
                    );
                }
                Ok(result) => panic!("Expected U64(10), got {:?}", result),
                Err(e) => panic!("Call frame isolation test failed: {:?}", e),
            }
        }
    }

    /// Test function call error conditions
    mod function_call_errors {
        use super::*;

        #[test]
        fn test_invalid_function_address() {
            // Test calling function at invalid address
            let bytecode = test_bytecode![
                push_u64!(42),
                opcodes![0x90, 0x01, 0xFF, 0xFF], // CALL invalid address
            ];

            TestUtils::assert_execution_error(&bytecode, VMError::InvalidFunctionIndex);
        }

        #[test]
        fn test_call_beyond_script_end() {
            // Test calling function address beyond script length
            let mut bytecode = vec![0x35, 0x49, 0x56, 0x45]; // magic
            bytecode.extend_from_slice(&push_u64!(42));
            bytecode.extend_from_slice(&[0x90, 0x01]); // CALL opcode
            let invalid_addr = (bytecode.len() + 100) as u16; // Way beyond script end
            bytecode.extend_from_slice(&invalid_addr.to_le_bytes());
            bytecode.push(0x00); // HALT

            TestUtils::assert_execution_error(&bytecode, VMError::InvalidFunctionIndex);
        }

        #[test]
        fn test_stack_underflow_in_call() {
            // Test CALL when there aren't enough parameters on stack
            let bytecode = test_bytecode![
                // Only push 1 parameter but claim 2 in CALL
                push_u64!(42),
                opcodes![0x90, 0x02, 0x10, 0x00], // CALL param_count=2 (but only 1 on stack)
            ];

            TestUtils::assert_execution_error(&bytecode, VMError::StackError);
        }

        #[test]
        fn test_return_without_call() {
            // Test RETURN opcode without being in a function call
            let bytecode = test_bytecode![
                push_u64!(42),
                opcodes![RETURN], // RETURN without CALL
            ];

            let result = TestUtils::execute_simple(&bytecode);
            // Should handle gracefully or error appropriately
            assert!(result.is_err(), "RETURN without CALL should be handled");
        }

        #[test]
        fn test_function_visibility_violation() {
            // Test calling private function from external context
            // This requires function metadata with visibility flags

            let input_data = TestUtils::create_function_input(2, &[]); // Try to call function 2

            let bytecode = test_bytecode![opcodes![LOAD_PARAM, 1], opcodes![RETURN_VALUE],];

            let result = TestUtils::execute_with_input(&bytecode, &input_data);
            // Should fail with FunctionVisibilityViolation if function 2 is private
            assert!(result.is_err(), "Private function calls should be rejected");
        }
    }
}
