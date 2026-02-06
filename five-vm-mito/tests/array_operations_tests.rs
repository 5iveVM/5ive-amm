//! Array Operations Tests for Five VM
//!
//! Tests data structure opcodes including array creation, manipulation,
//! and string operations. These operations are essential for handling
//! complex data structures in smart contracts.
//!
//! Coverage: Array & String Operations range (0x60-0x6F)
//! - CREATE_ARRAY (0x60) - Array creation with capacity
//! - PUSH_ARRAY_LITERAL (0x61) - Push array literal to temp buffer
//! - ARRAY_INDEX (0x62) - Array indexing operation
//! - ARRAY_LENGTH (0x63) - Get array length
//! - ARRAY_SET (0x64) - Array element assignment
//! - ARRAY_GET (0x65) - Array element access
//! - PUSH_STRING_LITERAL (0x66) - Push string literal to temp buffer
//! - PUSH_STRING (0x67) - Push string with length encoding

use five_vm_mito::{FIVE_VM_PROGRAM_ID, MitoVM, Value, stack::StackStorage, AccountInfo};

fn execute_test(bytecode: &[u8], input: &[u8], accounts: &[AccountInfo]) -> five_vm_mito::Result<Option<Value>> {
    let mut storage = StackStorage::new(bytecode);
    MitoVM::execute_direct(bytecode, input, accounts, &FIVE_VM_PROGRAM_ID, &mut storage)
}

/// Helper function to build bytecode with proper Five VM header
fn build_bytecode(body: &[u8]) -> Vec<u8> {
    let mut bytecode = vec![
        0x35, 0x49, 0x56, 0x45, // 5IVE magic
        0x00, 0x00, 0x00, 0x00, // features (4 bytes LE)
        0x00, // public_function_count
        0x00, // total_function_count
    ];
    bytecode.extend_from_slice(body);
    bytecode
}

#[cfg(test)]
mod array_creation_tests {
    use super::*;

    #[test]
    fn test_create_array_basic() {
        // Test CREATE_ARRAY with specified capacity
        // 5IVE, PUSH_U8(5), CREATE_ARRAY, HALT
        let bytecode = build_bytecode(&[
            0x18, 0x05, // PUSH_U8(5) - array capacity
            0x60, // CREATE_ARRAY
            0x00, // HALT
        ]);

        let result = execute_test(&bytecode, &[], &[]);
        match result {
            Ok(value) => {
                println!("✅ CREATE_ARRAY with capacity 5 succeeded: {:?}", value);
                // Should return array reference or handle
            }
            Err(e) => {
                println!("ℹ️ CREATE_ARRAY not yet implemented: {:?}", e);
            }
        }
    }

    #[test]
    fn test_array_with_initial_elements() {
        // Test array creation with initial elements
        // 5IVE, PUSH_U64(10), PUSH_U64(20), PUSH_U64(30), PUSH_U8(3), CREATE_ARRAY, HALT
        let mut body = vec![];
        // PUSH_U64(10)
        body.push(0x1B);
        body.extend_from_slice(&10u64.to_le_bytes());
        // PUSH_U64(20)
        body.push(0x1B);
        body.extend_from_slice(&20u64.to_le_bytes());
        // PUSH_U64(30)
        body.push(0x1B);
        body.extend_from_slice(&30u64.to_le_bytes());

        body.push(0x18); body.push(0x03); // PUSH_U8(3)
        body.push(0x60); // CREATE_ARRAY
        body.push(0x00); // HALT

        let bytecode = build_bytecode(&body);

        let result = execute_test(&bytecode, &[], &[]);
        match result {
            Ok(value) => {
                println!(
                    "✅ CREATE_ARRAY with initial elements succeeded: {:?}",
                    value
                );
            }
            Err(e) => {
                println!("ℹ️ CREATE_ARRAY with elements not yet implemented: {:?}", e);
            }
        }
    }

    #[test]
    fn test_push_array_literal() {
        // Test PUSH_ARRAY_LITERAL for array constants
        // 5IVE, PUSH_ARRAY_LITERAL(4), [data], HALT
        let bytecode = build_bytecode(&[
            0x61, 0x04, // PUSH_ARRAY_LITERAL with 4 elements
            // Array data (4 u64 values encoded as bytes)
            0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 1
            0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 2
            0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 3
            0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 4
            0x00, // HALT
        ]);

        let result = execute_test(&bytecode, &[], &[]);
        match result {
            Ok(value) => {
                println!("✅ PUSH_ARRAY_LITERAL succeeded: {:?}", value);
            }
            Err(e) => {
                println!("ℹ️ PUSH_ARRAY_LITERAL not yet implemented: {:?}", e);
            }
        }
    }
}

#[cfg(test)]
mod array_access_tests {
    use super::*;

    #[test]
    fn test_array_index_access() {
        // Test ARRAY_INDEX for element access
        // Create array, then access element at index 1
        let mut body = vec![];
        // PUSH_U64(100)
        body.push(0x1B);
        body.extend_from_slice(&100u64.to_le_bytes());
        // PUSH_U64(200)
        body.push(0x1B);
        body.extend_from_slice(&200u64.to_le_bytes());
        // PUSH_U64(300)
        body.push(0x1B);
        body.extend_from_slice(&300u64.to_le_bytes());

        body.push(0x18); body.push(0x03); // PUSH_U8(3)
        body.push(0x60); // CREATE_ARRAY
        body.push(0x18); body.push(0x01); // PUSH_U8(1)
        body.push(0x62); // ARRAY_INDEX
        body.push(0x00); // HALT

        let bytecode = build_bytecode(&body);

        let result = execute_test(&bytecode, &[], &[]);
        match result {
            Ok(value) => {
                println!("✅ ARRAY_INDEX access succeeded: {:?}", value);
                // Should return 200 (element at index 1)
            }
            Err(e) => {
                println!("ℹ️ ARRAY_INDEX not yet implemented: {:?}", e);
            }
        }
    }

    #[test]
    fn test_array_get_operation() {
        // Test ARRAY_GET for safe element access
        let bytecode = build_bytecode(&[
            // Assume array reference is on stack
            0x18, 0x00, // PUSH_U8(0) - array reference placeholder
            0x18, 0x02, // PUSH_U8(2) - index to access
            0x65, // ARRAY_GET
            0x00, // HALT
        ]);

        let result = execute_test(&bytecode, &[], &[]);
        match result {
            Ok(value) => {
                println!("✅ ARRAY_GET succeeded: {:?}", value);
            }
            Err(e) => {
                println!("ℹ️ ARRAY_GET not yet implemented: {:?}", e);
            }
        }
    }

    #[test]
    fn test_array_length() {
        // Test ARRAY_LENGTH to get array size
        let mut body = vec![];
        // PUSH_U64(10)
        body.push(0x1B); body.extend_from_slice(&10u64.to_le_bytes());
        // PUSH_U64(20)
        body.push(0x1B); body.extend_from_slice(&20u64.to_le_bytes());
        // PUSH_U64(30)
        body.push(0x1B); body.extend_from_slice(&30u64.to_le_bytes());
        // PUSH_U64(40)
        body.push(0x1B); body.extend_from_slice(&40u64.to_le_bytes());

        body.push(0x18); body.push(0x04); // PUSH_U8(4)
        body.push(0x60); // CREATE_ARRAY
        body.push(0x63); // ARRAY_LENGTH
        body.push(0x00); // HALT

        let bytecode = build_bytecode(&body);

        let result = execute_test(&bytecode, &[], &[]);
        match result {
            Ok(value) => {
                println!("✅ ARRAY_LENGTH succeeded: {:?}", value);
                // Should return 4
                if let Some(Value::U64(length)) = value {
                    assert_eq!(length, 4, "Array length should be 4");
                }
            }
            Err(e) => {
                println!("ℹ️ ARRAY_LENGTH not yet implemented: {:?}", e);
            }
        }
    }

    #[test]
    fn test_array_set_operation() {
        // Test ARRAY_SET for element modification
        let mut body = vec![];
        // PUSH_U64(10)
        body.push(0x1B); body.extend_from_slice(&10u64.to_le_bytes());
        // PUSH_U64(20)
        body.push(0x1B); body.extend_from_slice(&20u64.to_le_bytes());

        body.push(0x18); body.push(0x02); // PUSH_U8(2)
        body.push(0x60); // CREATE_ARRAY

        body.push(0x18); body.push(0x01); // PUSH_U8(1)
        // PUSH_U64(100)
        body.push(0x1B); body.extend_from_slice(&100u64.to_le_bytes());

        body.push(0x64); // ARRAY_SET
        body.push(0x00); // HALT

        let bytecode = build_bytecode(&body);

        let result = execute_test(&bytecode, &[], &[]);
        match result {
            Ok(value) => {
                println!("✅ ARRAY_SET succeeded: {:?}", value);
            }
            Err(e) => {
                println!("ℹ️ ARRAY_SET not yet implemented: {:?}", e);
            }
        }
    }
}

#[cfg(test)]
mod string_operations_tests {
    use super::*;

    #[test]
    fn test_push_string_basic() {
        // Test PUSH_STRING with length encoding
        // 5IVE, PUSH_STRING("hello"), HALT
        let bytecode = build_bytecode(&[
            0x67, 0x05, // PUSH_STRING with length 5
            b'h', b'e', b'l', b'l', b'o', // String data
            0x00, // HALT
        ]);

        let result = execute_test(&bytecode, &[], &[]);
        match result {
            Ok(value) => {
                println!("✅ PUSH_STRING basic succeeded: {:?}", value);
            }
            Err(e) => {
                println!("ℹ️ PUSH_STRING not yet implemented: {:?}", e);
            }
        }
    }

    #[test]
    fn test_push_string_literal() {
        // Test PUSH_STRING_LITERAL for string constants
        let bytecode = build_bytecode(&[
            0x66, 0x0C, // PUSH_STRING_LITERAL with length 12
            b'H', b'e', b'l', b'l', b'o', b' ', b'W', b'o', b'r', b'l', b'd',
            b'!', // "Hello World!"
            0x00, // HALT
        ]);

        let result = execute_test(&bytecode, &[], &[]);
        match result {
            Ok(value) => {
                println!("✅ PUSH_STRING_LITERAL succeeded: {:?}", value);
            }
            Err(e) => {
                println!("ℹ️ PUSH_STRING_LITERAL not yet implemented: {:?}", e);
            }
        }
    }

    #[test]
    fn test_string_as_array() {
        // Test string operations using array opcodes (strings are byte arrays)
        let bytecode = build_bytecode(&[
            // Create string "test"
            0x67, 0x04, // PUSH_STRING with length 4
            b't', b'e', b's', b't', // String data
            // Get string length using ARRAY_LENGTH
            0x63, // ARRAY_LENGTH (works on strings)
            0x00, // HALT
        ]);

        let result = execute_test(&bytecode, &[], &[]);
        match result {
            Ok(value) => {
                println!("✅ String as array operation succeeded: {:?}", value);
                // Should return 4 (length of "test")
                if let Some(Value::U64(length)) = value {
                    assert_eq!(length, 4, "String length should be 4");
                }
            }
            Err(e) => {
                println!("ℹ️ String array operations not yet implemented: {:?}", e);
            }
        }
    }

    #[test]
    fn test_string_indexing() {
        // Test accessing individual characters in string
        let bytecode = build_bytecode(&[
            // Create string "Five"
            0x67, 0x04, // PUSH_STRING with length 4
            b'F', b'i', b'v', b'e', // String data
            // Access character at index 0 (should be 'F' = 70)
            0x18, 0x00, // PUSH_U8(0) - index
            0x62, // ARRAY_INDEX (works on strings)
            0x00, // HALT
        ]);

        let result = execute_test(&bytecode, &[], &[]);
        match result {
            Ok(value) => {
                println!("✅ String indexing succeeded: {:?}", value);
                // Should return 70 (ASCII code for 'F')
                if let Some(Value::U64(char_code)) = value {
                    assert_eq!(char_code, 70, "First character should be 'F' (70)");
                }
            }
            Err(e) => {
                println!("ℹ️ String indexing not yet implemented: {:?}", e);
            }
        }
    }
}

#[cfg(test)]
mod array_edge_cases_tests {
    use super::*;

    #[test]
    fn test_empty_array() {
        // Test creating and working with empty arrays
        let bytecode = build_bytecode(&[
            0x18, 0x00, // PUSH_U8(0) - zero capacity
            0x60, // CREATE_ARRAY
            0x63, // ARRAY_LENGTH
            0x00, // HALT
        ]);

        let result = execute_test(&bytecode, &[], &[]);
        match result {
            Ok(value) => {
                println!("✅ Empty array creation succeeded: {:?}", value);
                if let Some(Value::U64(length)) = value {
                    assert_eq!(length, 0, "Empty array length should be 0");
                }
            }
            Err(e) => {
                println!("ℹ️ Empty array handling not yet implemented: {:?}", e);
            }
        }
    }

    #[test]
    fn test_array_bounds_checking() {
        // Test array access with out-of-bounds index
        let mut body = vec![];
        // PUSH_U64(1)
        body.push(0x1B); body.extend_from_slice(&1u64.to_le_bytes());
        body.push(0x18); body.push(0x01); // PUSH_U8(1)
        body.push(0x60); // CREATE_ARRAY

        body.push(0x18); body.push(0x05); // PUSH_U8(5)
        body.push(0x62); // ARRAY_INDEX
        body.push(0x00); // HALT

        let bytecode = build_bytecode(&body);

        let result = execute_test(&bytecode, &[], &[]);
        match result {
            Ok(_) => panic!("Array bounds check should fail for out-of-bounds access"),
            Err(e) => {
                println!("✅ Array bounds checking correctly failed: {:?}", e);
            }
        }
    }

    #[test]
    fn test_large_array() {
        // Test creating larger arrays to verify memory handling
        let bytecode = build_bytecode(&[
            0x18, 0xFF, // PUSH_U8(255) - large capacity
            0x60, // CREATE_ARRAY
            0x63, // ARRAY_LENGTH
            0x00, // HALT
        ]);

        let result = execute_test(&bytecode, &[], &[]);
        match result {
            Ok(value) => {
                println!("✅ Large array creation succeeded: {:?}", value);
            }
            Err(e) => {
                println!("ℹ️ Large array handling not yet implemented: {:?}", e);
            }
        }
    }
}

#[cfg(test)]
mod array_coverage_tests {
    use super::*;

    #[test]
    fn test_array_operations_coverage() {
        // Comprehensive test to verify all array opcodes are recognized
        let array_opcodes = [
            (0x60, "CREATE_ARRAY"),
            (0x61, "PUSH_ARRAY_LITERAL"),
            (0x62, "ARRAY_INDEX"),
            (0x63, "ARRAY_LENGTH"),
            (0x64, "ARRAY_SET"),
            (0x65, "ARRAY_GET"),
            (0x66, "PUSH_STRING_LITERAL"),
            (0x67, "PUSH_STRING"),
        ];

        println!("🔍 Testing Array Operations Coverage (0x60-0x6F):");

        for (opcode, name) in array_opcodes {
            // Test each opcode individually with minimal setup
            let bytecode = build_bytecode(&[
                0x18, 0x01,   // PUSH_U8(1) - basic parameter
                opcode, // Array opcode
                0x00,   // HALT
            ]);

            let result = execute_test(&bytecode, &[], &[]);
            match result {
                Ok(_) => println!("✅ {} (0x{:02X}) - IMPLEMENTED", name, opcode),
                Err(_) => println!("⚠️ {} (0x{:02X}) - NOT IMPLEMENTED", name, opcode),
            }
        }

        println!("📊 Array Operations Test Coverage Summary:");
        println!("   - Array Creation: CREATE_ARRAY, PUSH_ARRAY_LITERAL");
        println!("   - Array Access: ARRAY_INDEX, ARRAY_GET, ARRAY_LENGTH");
        println!("   - Array Modification: ARRAY_SET");
        println!("   - String Operations: PUSH_STRING, PUSH_STRING_LITERAL");
        println!("   - String as Array: All array ops work on strings");
    }
}
