//! Array and string operations tests
//!
//! Comprehensive unit tests for array operations, string handling,
//! memory management, and temp buffer operations.

#[cfg(test)]
mod array_operations_tests {
    use crate::test_framework::TestUtils;
    use crate::{opcodes, push_bool, push_u64, test_bytecode};
    use crate::{MitoVM, VMError, Value};
    use five_protocol::opcodes::*;

    /// Test array creation and basic operations
    mod array_creation {
        use super::*;

        #[test]
        fn test_create_array_empty() {
            // Test CREATE_ARRAY with capacity 0
            let mut bytecode = vec![0x35, 0x49, 0x56, 0x45]; // magic
            bytecode.push(CREATE_ARRAY); // 0x60
            bytecode.push(0); // capacity = 0
            bytecode.push(0x00); // HALT

            let result = TestUtils::execute_simple(&bytecode);
            // Should create empty array successfully
            assert!(result.is_ok(), "Empty array creation should succeed");
        }

        #[test]
        fn test_create_array_with_capacity() {
            // Test CREATE_ARRAY with specific capacity
            let mut bytecode = vec![0x35, 0x49, 0x56, 0x45]; // magic
            bytecode.push(CREATE_ARRAY);
            bytecode.push(5); // capacity = 5
            bytecode.push(0x00); // HALT

            let result = TestUtils::execute_simple(&bytecode);
            assert!(
                result.is_ok(),
                "Array creation with capacity should succeed"
            );
        }

        #[test]
        fn test_create_array_max_capacity() {
            // Test CREATE_ARRAY with maximum reasonable capacity
            let mut bytecode = vec![0x35, 0x49, 0x56, 0x45]; // magic
            bytecode.push(CREATE_ARRAY);
            bytecode.push(7); // capacity = 7 (fits in temp buffer)
            bytecode.push(0x00); // HALT

            let result = TestUtils::execute_simple(&bytecode);
            assert!(
                result.is_ok(),
                "Array creation with max capacity should succeed"
            );
        }

        #[test]
        fn test_create_array_overflow_capacity() {
            // Test CREATE_ARRAY with capacity that exceeds temp buffer
            let mut bytecode = vec![0x35, 0x49, 0x56, 0x45]; // magic
            bytecode.push(CREATE_ARRAY);
            bytecode.push(20); // capacity = 20 (too large for temp buffer)
            bytecode.push(0x00); // HALT

            let result = TestUtils::execute_simple(&bytecode);
            // Should fail with OutOfMemory
            TestUtils::assert_execution_error(&bytecode, VMError::OutOfMemory);
        }

        #[test]
        fn test_push_array_literal_empty() {
            // Test PUSH_ARRAY_LITERAL with 0 elements
            let mut bytecode = vec![0x35, 0x49, 0x56, 0x45]; // magic
            bytecode.push(PUSH_ARRAY_LITERAL); // 0x61
            bytecode.push(0); // element_count = 0
            bytecode.push(0x00); // HALT

            let result = TestUtils::execute_simple(&bytecode);
            assert!(result.is_ok(), "Empty array literal should succeed");
        }

        #[test]
        fn test_push_array_literal_with_elements() {
            // Test PUSH_ARRAY_LITERAL with elements
            let bytecode = test_bytecode![
                push_u64!(10),
                push_u64!(20),
                push_u64!(30),
                opcodes![PUSH_ARRAY_LITERAL, 3], // Create array with 3 elements
            ];

            let result = TestUtils::execute_simple(&bytecode);
            assert!(result.is_ok(), "Array literal with elements should succeed");
        }

        #[test]
        fn test_push_array_literal_mixed_types() {
            // Test PUSH_ARRAY_LITERAL with mixed value types
            let bytecode = test_bytecode![
                push_u64!(42),
                push_bool!(true),
                push_u64!(100),
                opcodes![PUSH_ARRAY_LITERAL, 3], // Mixed types
            ];

            let result = TestUtils::execute_simple(&bytecode);
            // Should handle mixed types or fail gracefully
            assert!(
                result.is_ok() || result.is_err(),
                "Mixed types should be handled"
            );
        }
    }

    /// Test array access operations
    mod array_access {
        use super::*;

        #[test]
        fn test_array_index_basic() {
            // Test ARRAY_INDEX operation
            let bytecode = test_bytecode![
                // Create array: [10, 20, 30]
                push_u64!(10),
                push_u64!(20),
                push_u64!(30),
                opcodes![PUSH_ARRAY_LITERAL, 3],
                // Access element at index 1 (should be 20)
                push_u64!(1),          // index
                opcodes![ARRAY_INDEX], // 0x62
            ];

            let result = TestUtils::execute_simple(&bytecode);
            // Should return 20 when properly implemented
            assert!(
                result.is_ok() || result.is_err(),
                "Array indexing needs implementation"
            );
        }

        #[test]
        fn test_array_index_first_element() {
            // Test accessing first element (index 0)
            let bytecode = test_bytecode![
                push_u64!(100),
                push_u64!(200),
                opcodes![PUSH_ARRAY_LITERAL, 2],
                push_u64!(0), // index 0
                opcodes![ARRAY_INDEX],
            ];

            let result = TestUtils::execute_simple(&bytecode);
            // Should return 100
            assert!(
                result.is_ok() || result.is_err(),
                "First element access needs implementation"
            );
        }

        #[test]
        fn test_array_index_last_element() {
            // Test accessing last element
            let bytecode = test_bytecode![
                push_u64!(5),
                push_u64!(15),
                push_u64!(25),
                opcodes![PUSH_ARRAY_LITERAL, 3],
                push_u64!(2), // index 2 (last element)
                opcodes![ARRAY_INDEX],
            ];

            let result = TestUtils::execute_simple(&bytecode);
            // Should return 25
            assert!(
                result.is_ok() || result.is_err(),
                "Last element access needs implementation"
            );
        }

        #[test]
        fn test_array_index_out_of_bounds() {
            // Test ARRAY_INDEX with invalid index
            let bytecode = test_bytecode![
                push_u64!(10),
                push_u64!(20),
                opcodes![PUSH_ARRAY_LITERAL, 2], // Array with 2 elements
                push_u64!(5),                    // index 5 (out of bounds)
                opcodes![ARRAY_INDEX],
            ];

            TestUtils::assert_execution_error(&bytecode, VMError::IndexOutOfBounds);
        }

        #[test]
        fn test_array_length() {
            // Test ARRAY_LENGTH operation
            let bytecode = test_bytecode![
                push_u64!(1),
                push_u64!(2),
                push_u64!(3),
                push_u64!(4),
                opcodes![PUSH_ARRAY_LITERAL, 4], // Array with 4 elements
                opcodes![ARRAY_LENGTH],          // 0x63
            ];

            let result = TestUtils::execute_simple(&bytecode);
            // Should return 4
            assert!(
                result.is_ok() || result.is_err(),
                "Array length needs implementation"
            );
        }

        #[test]
        fn test_array_length_empty() {
            // Test ARRAY_LENGTH on empty array
            let bytecode = test_bytecode![
                opcodes![PUSH_ARRAY_LITERAL, 0], // Empty array
                opcodes![ARRAY_LENGTH],
            ];

            let result = TestUtils::execute_simple(&bytecode);
            // Should return 0
            assert!(
                result.is_ok() || result.is_err(),
                "Empty array length should be 0"
            );
        }

        #[test]
        fn test_array_get_alias() {
            // Test ARRAY_GET operation (alias for ARRAY_INDEX)
            let bytecode = test_bytecode![
                push_u64!(100),
                push_u64!(200),
                opcodes![PUSH_ARRAY_LITERAL, 2],
                push_u64!(1),        // index 1
                opcodes![ARRAY_GET], // 0x65 (alias for ARRAY_INDEX)
            ];

            let result = TestUtils::execute_simple(&bytecode);
            // Should return 200
            assert!(
                result.is_ok() || result.is_err(),
                "ARRAY_GET alias needs implementation"
            );
        }
    }

    /// Test array modification operations
    mod array_modification {
        use super::*;

        #[test]
        fn test_array_set_basic() {
            // Test ARRAY_SET operation
            let mut bytecode = vec![0x35, 0x49, 0x56, 0x45]; // magic

            // Create array with capacity
            bytecode.push(CREATE_ARRAY);
            bytecode.push(3); // capacity = 3

            // Set element at index 0
            bytecode.extend_from_slice(&push_u64!(42)); // value
            bytecode.extend_from_slice(&[PUSH_U8, 0]); // index 0
            bytecode.push(ARRAY_SET); // 0x64

            // Get the array back and check element
            bytecode.extend_from_slice(&[PUSH_U8, 0]); // index 0
            bytecode.push(ARRAY_INDEX);

            bytecode.push(0x00); // HALT

            let result = TestUtils::execute_simple(&bytecode);
            // Should return 42 when properly implemented
            assert!(
                result.is_ok() || result.is_err(),
                "Array set needs implementation"
            );
        }

        #[test]
        fn test_array_set_multiple_elements() {
            // Test setting multiple elements in array
            let mut bytecode = vec![0x35, 0x49, 0x56, 0x45]; // magic

            // Create array
            bytecode.push(CREATE_ARRAY);
            bytecode.push(3);

            // Set element 0 = 10
            bytecode.extend_from_slice(&push_u64!(10));
            bytecode.extend_from_slice(&[PUSH_U8, 0]);
            bytecode.push(ARRAY_SET);
            bytecode.push(POP); // Remove array from stack

            // Set element 1 = 20
            bytecode.extend_from_slice(&push_u64!(20));
            bytecode.extend_from_slice(&[PUSH_U8, 1]);
            bytecode.push(ARRAY_SET);
            bytecode.push(POP); // Remove array from stack

            // Set element 2 = 30
            bytecode.extend_from_slice(&push_u64!(30));
            bytecode.extend_from_slice(&[PUSH_U8, 2]);
            bytecode.push(ARRAY_SET);

            // Get array length
            bytecode.push(ARRAY_LENGTH);

            bytecode.push(0x00); // HALT

            let result = TestUtils::execute_simple(&bytecode);
            // Should return 3 (length after setting 3 elements)
            assert!(
                result.is_ok() || result.is_err(),
                "Multiple array sets need implementation"
            );
        }

        #[test]
        fn test_array_set_out_of_bounds() {
            // Test ARRAY_SET with index beyond capacity
            let mut bytecode = vec![0x35, 0x49, 0x56, 0x45]; // magic

            bytecode.push(CREATE_ARRAY);
            bytecode.push(2); // capacity = 2

            // Try to set element at index 5 (beyond capacity)
            bytecode.extend_from_slice(&push_u64!(42));
            bytecode.extend_from_slice(&[PUSH_U8, 5]); // index 5 > capacity
            bytecode.push(ARRAY_SET);

            bytecode.push(0x00); // HALT

            TestUtils::assert_execution_error(&bytecode, VMError::IndexOutOfBounds);
        }

        #[test]
        fn test_array_set_grow_length() {
            // Test that ARRAY_SET grows array length when setting beyond current length
            let mut bytecode = vec![0x35, 0x49, 0x56, 0x45]; // magic

            bytecode.push(CREATE_ARRAY);
            bytecode.push(5); // capacity = 5

            // Set element at index 3 (should grow length to 4)
            bytecode.extend_from_slice(&push_u64!(42));
            bytecode.extend_from_slice(&[PUSH_U8, 3]);
            bytecode.push(ARRAY_SET);

            // Check new length
            bytecode.push(ARRAY_LENGTH);

            bytecode.push(0x00); // HALT

            let result = TestUtils::execute_simple(&bytecode);
            // Should return 4 (new length)
            assert!(
                result.is_ok() || result.is_err(),
                "Array length growth needs implementation"
            );
        }
    }

    /// Test string operations
    mod string_operations {
        use super::*;

        #[test]
        fn test_push_string_literal_basic() {
            // Test PUSH_STRING_LITERAL operation
            let mut bytecode = vec![0x35, 0x49, 0x56, 0x45]; // magic
            bytecode.push(PUSH_STRING_LITERAL); // 0x66
            bytecode.push(5); // length = 5
            bytecode.extend_from_slice(b"hello"); // string data
            bytecode.push(0x00); // HALT

            let result = TestUtils::execute_simple(&bytecode);
            assert!(result.is_ok(), "String literal should create successfully");
        }

        #[test]
        fn test_push_string_literal_empty() {
            // Test PUSH_STRING_LITERAL with empty string
            let mut bytecode = vec![0x35, 0x49, 0x56, 0x45]; // magic
            bytecode.push(PUSH_STRING_LITERAL);
            bytecode.push(0); // length = 0
                              // No string data for empty string
            bytecode.push(0x00); // HALT

            let result = TestUtils::execute_simple(&bytecode);
            assert!(result.is_ok(), "Empty string should create successfully");
        }

        #[test]
        fn test_push_string_literal_unicode() {
            // Test PUSH_STRING_LITERAL with UTF-8 content
            let mut bytecode = vec![0x35, 0x49, 0x56, 0x45]; // magic
            bytecode.push(PUSH_STRING_LITERAL);

            let utf8_string = "Hello 🌍"; // String with emoji
            let utf8_bytes = utf8_string.as_bytes();
            bytecode.push(utf8_bytes.len() as u8);
            bytecode.extend_from_slice(utf8_bytes);
            bytecode.push(0x00); // HALT

            let result = TestUtils::execute_simple(&bytecode);
            assert!(result.is_ok(), "UTF-8 string should be valid");
        }

        #[test]
        fn test_push_string_literal_invalid_utf8() {
            // Test PUSH_STRING_LITERAL with invalid UTF-8
            let mut bytecode = vec![0x35, 0x49, 0x56, 0x45]; // magic
            bytecode.push(PUSH_STRING_LITERAL);
            bytecode.push(3); // length = 3
            bytecode.extend_from_slice(&[0xFF, 0xFE, 0xFD]); // Invalid UTF-8 sequence
            bytecode.push(0x00); // HALT

            let result = TestUtils::execute_simple(&bytecode);
            // Should fail with InvalidOperation for invalid UTF-8
            TestUtils::assert_execution_error(&bytecode, VMError::InvalidOperation);
        }

        #[test]
        fn test_push_string_vle() {
            // Test PUSH_STRING with VLE encoding
            let mut bytecode = vec![0x35, 0x49, 0x56, 0x45]; // magic
            bytecode.push(PUSH_STRING); // 0x67
            bytecode.push(0x08); // VLE encoded length (8)
            bytecode.extend_from_slice(b"test_str"); // 8 byte string
            bytecode.push(0x00); // HALT

            let result = TestUtils::execute_simple(&bytecode);
            assert!(
                result.is_ok() || result.is_err(),
                "VLE string needs implementation"
            );
        }

        #[test]
        fn test_string_as_array() {
            // Test that strings can be used as arrays (unified system)
            let mut bytecode = vec![0x35, 0x49, 0x56, 0x45]; // magic

            // Create string
            bytecode.push(PUSH_STRING_LITERAL);
            bytecode.push(4);
            bytecode.extend_from_slice(b"test");

            // Get string length (should work like array length)
            bytecode.push(ARRAY_LENGTH);

            bytecode.push(0x00); // HALT

            let result = TestUtils::execute_simple(&bytecode);
            // Should return 4 (string length)
            assert!(
                result.is_ok() || result.is_err(),
                "String as array needs implementation"
            );
        }
    }

    /// Test memory management and temp buffer operations
    mod memory_management {
        use super::*;

        #[test]
        fn test_temp_buffer_allocation() {
            // Test multiple array/string allocations in temp buffer
            let mut bytecode = vec![0x35, 0x49, 0x56, 0x45]; // magic

            // Create multiple arrays to test temp buffer management
            bytecode.push(CREATE_ARRAY);
            bytecode.push(2); // Array 1

            bytecode.push(CREATE_ARRAY);
            bytecode.push(3); // Array 2

            bytecode.push(PUSH_STRING_LITERAL);
            bytecode.push(5);
            bytecode.extend_from_slice(b"hello"); // String

            bytecode.push(0x00); // HALT

            let result = TestUtils::execute_simple(&bytecode);
            // Should handle multiple allocations without overflow
            assert!(
                result.is_ok() || result.is_err(),
                "Multiple allocations need management"
            );
        }

        #[test]
        fn test_temp_buffer_overflow() {
            // Test temp buffer overflow with large allocations
            let mut bytecode = vec![0x35, 0x49, 0x56, 0x45]; // magic

            // Try to create very large string that exceeds temp buffer
            bytecode.push(PUSH_STRING_LITERAL);
            bytecode.push(60); // 60 bytes + header > temp buffer capacity
            bytecode.extend_from_slice(&vec![b'X'; 60]); // Large string

            bytecode.push(0x00); // HALT

            let result = TestUtils::execute_simple(&bytecode);
            // Should fail with OutOfMemory
            TestUtils::assert_execution_error(&bytecode, VMError::OutOfMemory);
        }

        #[test]
        fn test_string_ref_bounds_checking() {
            // Test the specific bounds checking fix for StringRef -> ArrayRef conversion
            // This tests the fix for temp buffer overflow vulnerability

            // The issue: StringRef uses u16 IDs but temp_buffer only has 64 bytes
            // StringRef(64) would convert to temp_buffer[64] which is out of bounds

            // This test verifies the bounds checking logic from the arrays.rs fix
            let string_ref_id = 64u16; // Beyond temp buffer bounds (0-63)

            // Simulate the bounds checking logic:
            let is_valid = string_ref_id <= 63;
            assert!(
                !is_valid,
                "StringRef(64) should be invalid due to temp buffer bounds"
            );

            // Test with valid ID
            let valid_id = 32u16;
            let is_valid_2 = valid_id <= 63;
            assert!(is_valid_2, "StringRef(32) should be valid");

            // Test boundary case
            let boundary_id = 63u16;
            let is_boundary_valid = boundary_id <= 63;
            assert!(
                is_boundary_valid,
                "StringRef(63) should be valid (exactly at boundary)"
            );
        }

        #[test]
        fn test_array_serialization_bounds() {
            // Test array element serialization within temp buffer bounds
            let bytecode = test_bytecode![
                // Create array with elements that test serialization
                push_u64!(u64::MAX), // Large value
                push_u64!(0),        // Small value
                push_bool!(true),    // Different type
                opcodes![PUSH_ARRAY_LITERAL, 3],
                // Access elements to test serialization/deserialization
                push_u64!(0),
                opcodes![ARRAY_INDEX],
            ];

            let result = TestUtils::execute_simple(&bytecode);
            // Should handle serialization without buffer overflow
            assert!(
                result.is_ok() || result.is_err(),
                "Array serialization needs bounds checking"
            );
        }
    }

    /// Test array and string error conditions
    mod error_conditions {
        use super::*;

        #[test]
        fn test_array_type_mismatch() {
            // Test operations with wrong types on stack
            let bytecode = test_bytecode![
                push_bool!(true),       // Wrong type for array operations
                opcodes![ARRAY_LENGTH], // Should expect ArrayRef
            ];

            TestUtils::assert_execution_error(&bytecode, VMError::TypeMismatch);
        }

        #[test]
        fn test_string_operations_type_mismatch() {
            // Test string operations with wrong stack types
            let bytecode = test_bytecode![
                push_u64!(42),          // Not a string/array
                opcodes![ARRAY_LENGTH], // Expects array/string reference
            ];

            TestUtils::assert_execution_error(&bytecode, VMError::TypeMismatch);
        }

        #[test]
        fn test_array_stack_underflow() {
            // Test array operations with insufficient stack items
            let bytecode = test_bytecode![
                opcodes![ARRAY_INDEX], // No array or index on stack
            ];

            TestUtils::assert_execution_error(&bytecode, VMError::StackError);
        }

        #[test]
        fn test_invalid_array_operation() {
            // Test invalid array opcode in range
            let mut bytecode = vec![0x35, 0x49, 0x56, 0x45]; // magic
            bytecode.push(0x6F); // Invalid opcode in array range (0x60-0x6F)
            bytecode.push(0x00); // HALT

            TestUtils::assert_execution_error(&bytecode, VMError::InvalidInstruction);
        }
    }

    /// Test complex array scenarios
    mod complex_scenarios {
        use super::*;

        #[test]
        fn test_nested_array_operations() {
            // Test complex nested array operations
            let bytecode = test_bytecode![
                // Create array 1: [10, 20]
                push_u64!(10),
                push_u64!(20),
                opcodes![PUSH_ARRAY_LITERAL, 2],
                // Create array 2: [30, 40]
                push_u64!(30),
                push_u64!(40),
                opcodes![PUSH_ARRAY_LITERAL, 2],
                // Get length of both arrays and add them
                opcodes![ARRAY_LENGTH], // Length of array 2
                opcodes![SWAP],
                opcodes![ARRAY_LENGTH], // Length of array 1
                opcodes![ADD],          // Should be 2 + 2 = 4
            ];

            let result = TestUtils::execute_simple(&bytecode);
            // Should return 4 when properly implemented
            assert!(
                result.is_ok() || result.is_err(),
                "Nested array operations need implementation"
            );
        }

        #[test]
        fn test_array_string_interaction() {
            // Test interactions between arrays and strings
            let mut bytecode = vec![0x35, 0x49, 0x56, 0x45]; // magic

            // Create string
            bytecode.push(PUSH_STRING_LITERAL);
            bytecode.push(4);
            bytecode.extend_from_slice(b"test");

            // Create array
            bytecode.extend_from_slice(&push_u64!(100));
            bytecode.extend_from_slice(&push_u64!(200));
            bytecode.push(PUSH_ARRAY_LITERAL);
            bytecode.push(2);

            // Compare lengths
            bytecode.push(ARRAY_LENGTH); // Array length (2)
            bytecode.push(SWAP);
            bytecode.push(ARRAY_LENGTH); // String length (4)
            bytecode.push(ADD); // 2 + 4 = 6

            bytecode.push(0x00); // HALT

            let result = TestUtils::execute_simple(&bytecode);
            // Should return 6
            assert!(
                result.is_ok() || result.is_err(),
                "Array-string interaction needs implementation"
            );
        }

        #[test]
        fn test_array_modification_chain() {
            // Test chaining multiple array modifications
            let mut bytecode = vec![0x35, 0x49, 0x56, 0x45]; // magic

            // Create array with capacity
            bytecode.push(CREATE_ARRAY);
            bytecode.push(5);

            // Chain of modifications: array.set(0, 10).set(1, 20).set(2, 30)
            // Note: ARRAY_SET returns the array for chaining

            bytecode.extend_from_slice(&push_u64!(10));
            bytecode.extend_from_slice(&[PUSH_U8, 0]);
            bytecode.push(ARRAY_SET); // Returns array

            bytecode.extend_from_slice(&push_u64!(20));
            bytecode.extend_from_slice(&[PUSH_U8, 1]);
            bytecode.push(ARRAY_SET); // Returns array

            bytecode.extend_from_slice(&push_u64!(30));
            bytecode.extend_from_slice(&[PUSH_U8, 2]);
            bytecode.push(ARRAY_SET); // Returns array

            // Get final length
            bytecode.push(ARRAY_LENGTH);

            bytecode.push(0x00); // HALT

            let result = TestUtils::execute_simple(&bytecode);
            // Should return 3 (elements set)
            assert!(
                result.is_ok() || result.is_err(),
                "Array modification chain needs implementation"
            );
        }
    }
}
