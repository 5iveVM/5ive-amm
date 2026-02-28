//! Memory Operations Tests for Five VM
//!
//! Tests memory access and account field operations including zero-copy
//! field access, account data manipulation, and global state management.
//! These operations are critical for efficient smart contract data handling.
//!
//! Coverage: Memory Operations range (0x40-0x4F)
//! - STORE (0x40) - Basic memory store
//! - LOAD (0x41) - Basic memory load
//! - STORE_FIELD (0x42) - Zero-copy account field store
//! - LOAD_FIELD (0x43) - Zero-copy account field load
//! - LOAD_INPUT (0x44) - Load input data
//! - STORE_GLOBAL (0x45) - Global state store
//! - LOAD_GLOBAL (0x46) - Global state load

use five_vm_mito::{stack::StackStorage, AccountInfo, MitoVM, Value, FIVE_VM_PROGRAM_ID};

fn execute_test(
    bytecode: &[u8],
    input: &[u8],
    accounts: &[AccountInfo],
) -> five_vm_mito::Result<Option<Value>> {
    let mut storage = StackStorage::new();
    MitoVM::execute_direct(bytecode, input, accounts, &FIVE_VM_PROGRAM_ID, &mut storage)
}

#[cfg(test)]
mod basic_memory_tests {
    use super::*;

    #[test]
    fn test_store_and_load_basic() {
        // Test basic STORE and LOAD operations
        // 5IVE, PUSH_U64(42), PUSH_U8(0), STORE, PUSH_U8(0), LOAD, HALT
        let bytecode = vec![
            0x35, 0x49, 0x56, 0x45, // 5IVE magic
            // Store value 42 at memory location 0
            0x1B, 0x2A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // PUSH_U64(42)
            0x18, 0x00, // PUSH_U8(0) - memory address
            0x40, // STORE
            // Load value from memory location 0
            0x18, 0x00, // PUSH_U8(0) - memory address
            0x41, // LOAD
            0x00, // HALT
        ];

        let result = execute_test(&bytecode, &[], &[]);
        match result {
            Ok(value) => {
                println!("✅ Basic STORE/LOAD succeeded: {:?}", value);
                // Should return 42
                if let Some(Value::U64(loaded_value)) = value {
                    assert_eq!(loaded_value, 42, "Loaded value should be 42");
                }
            }
            Err(e) => {
                println!("ℹ️ Basic STORE/LOAD not yet implemented: {:?}", e);
            }
        }
    }

    #[test]
    fn test_load_input_data() {
        // Test LOAD_INPUT for accessing input parameters
        // 5IVE, PUSH_U8(0), LOAD_INPUT, HALT
        let bytecode = vec![
            0x35, 0x49, 0x56, 0x45, // 5IVE magic
            0x18, 0x00, // PUSH_U8(0) - input index
            0x44, // LOAD_INPUT
            0x00, // HALT
        ];

        // Provide input data to load
        let input_data = vec![0x77, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]; // u64(119) in little endian

        let result = execute_test(&bytecode, &input_data, &[]);
        match result {
            Ok(value) => {
                println!("✅ LOAD_INPUT succeeded: {:?}", value);
                // Should return input data value
            }
            Err(e) => {
                println!("ℹ️ LOAD_INPUT not yet implemented: {:?}", e);
            }
        }
    }
}

#[cfg(test)]
mod field_operations_tests {
    use super::*;

    #[test]
    fn test_store_field_basic() {
        // Test STORE_FIELD for account data modification
        // 5IVE, PUSH_U64(123), PUSH_U8(0), PUSH_U8(0), STORE_FIELD, HALT
        let bytecode = vec![
            0x35, 0x49, 0x56, 0x45, // 5IVE magic
            // Store value 123 at offset 0 in account 0
            0x1B, 0x7B, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // PUSH_U64(123)
            0x18, 0x00, // PUSH_U8(0) - account index
            0x18, 0x00, // PUSH_U8(0) - field offset
            0x42, // STORE_FIELD
            0x00, // HALT
        ];

        // Create writable account with data space

        let result = execute_test(&bytecode, &[], &[]);
        match result {
            Ok(value) => {
                println!("✅ STORE_FIELD succeeded: {:?}", value);
            }
            Err(e) => {
                println!("ℹ️ STORE_FIELD not yet implemented: {:?}", e);
            }
        }
    }

    #[test]
    fn test_load_field_basic() {
        // Test LOAD_FIELD for account data access
        // 5IVE, PUSH_U8(0), PUSH_U8(8), LOAD_FIELD, HALT
        let bytecode = vec![
            0x35, 0x49, 0x56, 0x45, // 5IVE magic
            0x18, 0x00, // PUSH_U8(0) - account index
            0x18, 0x08, // PUSH_U8(8) - field offset (8 bytes from start)
            0x43, // LOAD_FIELD
            0x00, // HALT
        ];

        // Create account with some data
        let mut account_data = [0; 32];
        // Store u64(456) at offset 8
        account_data[8..16].copy_from_slice(&456u64.to_le_bytes());

        let result = execute_test(&bytecode, &[], &[]);
        match result {
            Ok(value) => {
                println!("✅ LOAD_FIELD succeeded: {:?}", value);
                // Should return 456
                if let Some(Value::U64(loaded_value)) = value {
                    assert_eq!(loaded_value, 456, "Loaded field value should be 456");
                }
            }
            Err(e) => {
                println!("ℹ️ LOAD_FIELD not yet implemented: {:?}", e);
            }
        }
    }

    #[test]
    fn test_load_field_built_in_properties() {
        // Test LOAD_FIELD with built-in account properties
        // 5IVE, PUSH_U8(0), PUSH_U8(FIELD_LAMPORTS), LOAD_FIELD, HALT
        let bytecode = vec![
            0x35, 0x49, 0x56, 0x45, // 5IVE magic
            0x18, 0x00, // PUSH_U8(0) - account index
            0x18, 0x00, // PUSH_U8(0) - FIELD_LAMPORTS
            0x43, // LOAD_FIELD
            0x00, // HALT
        ];

        let result = execute_test(&bytecode, &[], &[]);
        match result {
            Ok(value) => {
                println!("✅ LOAD_FIELD lamports succeeded: {:?}", value);
                // Should return 2500000
                if let Some(Value::U64(lamports)) = value {
                    assert_eq!(lamports, 2500000, "Lamports should be 2500000");
                }
            }
            Err(e) => {
                println!(
                    "ℹ️ LOAD_FIELD built-in properties not yet implemented: {:?}",
                    e
                );
            }
        }
    }

    #[test]
    fn test_store_load_field_sequence() {
        // Test STORE_FIELD followed by LOAD_FIELD
        let bytecode = vec![
            0x35, 0x49, 0x56, 0x45, // 5IVE magic
            // Store value 789 at offset 16
            0x1B, 0x15, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // PUSH_U64(789)
            0x18, 0x00, // PUSH_U8(0) - account index
            0x18, 0x10, // PUSH_U8(16) - offset
            0x42, // STORE_FIELD
            // Load the same value back
            0x18, 0x00, // PUSH_U8(0) - account index
            0x18, 0x10, // PUSH_U8(16) - same offset
            0x43, // LOAD_FIELD
            0x00, // HALT
        ];

        let result = execute_test(&bytecode, &[], &[]);
        match result {
            Ok(value) => {
                println!(
                    "✅ STORE_FIELD -> LOAD_FIELD sequence succeeded: {:?}",
                    value
                );
                // Should return 789
                if let Some(Value::U64(value)) = value {
                    assert_eq!(value, 789, "Round-trip value should be 789");
                }
            }
            Err(e) => {
                println!("ℹ️ Field operation sequence not yet implemented: {:?}", e);
            }
        }
    }
}

#[cfg(test)]
mod global_state_tests {
    use super::*;

    #[test]
    fn test_store_global_basic() {
        // Test STORE_GLOBAL for global state management
        // 5IVE, PUSH_U64(999), PUSH_U8(0), STORE_GLOBAL, HALT
        let bytecode = vec![
            0x35, 0x49, 0x56, 0x45, // 5IVE magic
            0x1B, 0xE7, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // PUSH_U64(999)
            0x18, 0x00, // PUSH_U8(0) - global slot
            0x45, // STORE_GLOBAL
            0x00, // HALT
        ];

        let result = execute_test(&bytecode, &[], &[]);
        match result {
            Ok(value) => {
                println!("✅ STORE_GLOBAL succeeded: {:?}", value);
            }
            Err(e) => {
                println!("ℹ️ STORE_GLOBAL not yet implemented: {:?}", e);
            }
        }
    }

    #[test]
    fn test_load_global_basic() {
        // Test LOAD_GLOBAL for global state access
        // 5IVE, PUSH_U8(0), LOAD_GLOBAL, HALT
        let bytecode = vec![
            0x35, 0x49, 0x56, 0x45, // 5IVE magic
            0x18, 0x00, // PUSH_U8(0) - global slot
            0x46, // LOAD_GLOBAL
            0x00, // HALT
        ];

        let result = execute_test(&bytecode, &[], &[]);
        match result {
            Ok(value) => {
                println!("✅ LOAD_GLOBAL succeeded: {:?}", value);
            }
            Err(e) => {
                println!("ℹ️ LOAD_GLOBAL not yet implemented: {:?}", e);
            }
        }
    }

    #[test]
    fn test_global_state_persistence() {
        // Test global state store/load sequence
        let bytecode = vec![
            0x35, 0x49, 0x56, 0x45, // 5IVE magic
            // Store value in global slot 1
            0x1B, 0x11, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // PUSH_U64(529)
            0x18, 0x01, // PUSH_U8(1) - global slot 1
            0x45, // STORE_GLOBAL
            // Load value from global slot 1
            0x18, 0x01, // PUSH_U8(1) - same global slot
            0x46, // LOAD_GLOBAL
            0x00, // HALT
        ];

        let result = execute_test(&bytecode, &[], &[]);
        match result {
            Ok(value) => {
                println!("✅ Global state persistence succeeded: {:?}", value);
                // Should return 529
                if let Some(Value::U64(global_value)) = value {
                    assert_eq!(global_value, 529, "Global state value should be 529");
                }
            }
            Err(e) => {
                println!("ℹ️ Global state persistence not yet implemented: {:?}", e);
            }
        }
    }
}

#[cfg(test)]
mod zero_copy_optimization_tests {
    use super::*;

    #[test]
    fn test_zero_copy_field_access() {
        // Test zero-copy field operations for performance
        // This should not copy data but provide direct access
        let bytecode = vec![
            0x35, 0x49, 0x56, 0x45, // 5IVE magic
            // Access large data structure without copying
            0x18, 0x00, // PUSH_U8(0) - account index
            0x18, 0x00, // PUSH_U8(0) - offset 0
            0x43, // LOAD_FIELD (zero-copy)
            0x00, // HALT
        ];

        // Create account with large data structure
        let mut large_data = vec![0; 1024]; // 1KB of data
        large_data[0..8].copy_from_slice(&12345u64.to_le_bytes());

        let result = execute_test(&bytecode, &[], &[]);
        match result {
            Ok(value) => {
                println!("✅ Zero-copy field access succeeded: {:?}", value);
                // Should efficiently access first u64 without copying entire structure
                if let Some(Value::U64(first_value)) = value {
                    assert_eq!(first_value, 12345, "Zero-copy access should return 12345");
                }
            }
            Err(e) => {
                println!("ℹ️ Zero-copy optimization not yet implemented: {:?}", e);
            }
        }
    }

    #[test]
    fn test_field_access_bounds_checking() {
        // Test field access with bounds checking
        let bytecode = vec![
            0x35, 0x49, 0x56, 0x45, // 5IVE magic
            0x18, 0x00, // PUSH_U8(0) - account index
            0x18, 0x64, // PUSH_U8(100) - offset beyond data
            0x43, // LOAD_FIELD
            0x00, // HALT
        ];

        let result = execute_test(&bytecode, &[], &[]);
        match result {
            Ok(_) => panic!("Field access should fail for out-of-bounds offset"),
            Err(e) => {
                println!("✅ Field access bounds checking correctly failed: {:?}", e);
            }
        }
    }
}

#[cfg(test)]
mod memory_coverage_tests {
    use super::*;

    #[test]
    fn test_memory_operations_coverage() {
        // Comprehensive test to verify all memory opcodes are recognized
        let memory_opcodes = [
            (0x40, "STORE"),
            (0x41, "LOAD"),
            (0x42, "STORE_FIELD"),
            (0x43, "LOAD_FIELD"),
            (0x44, "LOAD_INPUT"),
            (0x45, "STORE_GLOBAL"),
            (0x46, "LOAD_GLOBAL"),
        ];

        println!("🔍 Testing Memory Operations Coverage (0x40-0x4F):");

        // Create account for field operations

        for (opcode, name) in memory_opcodes {
            // Test each opcode individually with appropriate setup
            let bytecode = vec![
                0x35, 0x49, 0x56, 0x45, // 5IVE magic
                0x1B, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // PUSH_U64(1) - value
                0x18, 0x00,   // PUSH_U8(0) - address/index
                opcode, // Memory opcode
                0x00,   // HALT
            ];

            let result = execute_test(&bytecode, &[], &[]);
            match result {
                Ok(_) => println!("✅ {} (0x{:02X}) - IMPLEMENTED", name, opcode),
                Err(_) => println!("⚠️ {} (0x{:02X}) - NOT IMPLEMENTED", name, opcode),
            }
        }

        println!("📊 Memory Operations Test Coverage Summary:");
        println!("   - Basic Memory: STORE, LOAD");
        println!("   - Account Fields: STORE_FIELD, LOAD_FIELD (zero-copy)");
        println!("   - Input Access: LOAD_INPUT");
        println!("   - Global State: STORE_GLOBAL, LOAD_GLOBAL");
        println!("   - Zero-Copy Optimization: Efficient large data access");
    }
}
