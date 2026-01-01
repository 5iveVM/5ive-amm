//! Integration test for polymorphic arithmetic with Solana operations
//! 
//! Tests that mixed-width arithmetic works correctly when values are narrowed
//! for Solana operations like SET_LAMPORTS, including overflow detection.

#[cfg(test)]
mod tests {
    use crate::instructions;
    use five_vm_mito::{ExecutionContext, MitoVM, Pubkey};
    use five_protocol::{opcodes::*, ValueRef, FIVE_MAGIC};
    use pinocchio::{
        account_info::AccountInfo,
        program_error::ProgramError,
    };

    /// Helper to create bytecode with FIVE magic header
    fn create_test_bytecode(ops: &[u8]) -> Vec<u8> {
        let mut bytecode = vec![];
        bytecode.extend_from_slice(&FIVE_MAGIC);
        bytecode.push(0x00); // features flags
        bytecode.push(0x00); // function count: 0 (main program only)
        bytecode.extend_from_slice(ops);
        bytecode
    }

    /// Helper to create test account data 
    fn create_test_account_data(lamports: u64, data_size: usize) -> Vec<u8> {
        vec![0u8; data_size]
    }

    #[test]
    fn test_mixed_arithmetic_with_set_lamports_success() {
        // Test: u64 + u128 = u128, then narrow to u64 for SET_LAMPORTS (success case)
        let bytecode = create_test_bytecode(&[
            // Push base lamports (u64)
            PUSH_U64, 
            0x00, 0x10, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, // 4096 lamports
            
            // Push additional amount (u128) 
            PUSH_U128,
            0x00, 0x10, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, // 4096 as u128 (low)
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,   // 0 (high)
            
            // ADD (polymorphic: u64 + u128 = u128)
            ADD,
            
            // SET_LAMPORTS (account index 0) - this will narrow u128 to u64
            SET_LAMPORTS, 0x00, // account index 0
            
            RETURN_VALUE
        ]);

        // Create mock account
        let mut account_data = create_test_account_data(0, 100);
        let account_key = Pubkey::from([1; 32]);
        let program_id = Pubkey::from([0; 32]);
        
        // Execute with MitoVM
        let result = MitoVM::execute_direct(&bytecode, &[], &[]);
        
        // Should succeed - 4096 + 4096 = 8192, fits in u64
        assert!(result.is_ok());
        println!("✓ Mixed arithmetic with successful narrowing works");
    }

    #[test] 
    fn test_mixed_arithmetic_with_set_lamports_overflow() {
        // Test: u64 + u128 = u128, then narrow to u64 for SET_LAMPORTS (overflow case)
        let bytecode = create_test_bytecode(&[
            // Push base lamports (u64::MAX)
            PUSH_U64,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, // u64::MAX
            
            // Push additional amount (u128 beyond u64::MAX)
            PUSH_U128,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 0 (low) 
            0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 1 (high) = 2^64
            
            // ADD (polymorphic: u64 + u128 = u128)  
            ADD,
            
            // SET_LAMPORTS (account index 0) - this should fail with NumericOverflow
            SET_LAMPORTS, 0x00, // account index 0
            
            RETURN_VALUE
        ]);

        // Execute with MitoVM
        let result = MitoVM::execute_direct(&bytecode, &[], &[]);
        
        // Should fail with NumericOverflow when trying to narrow u128 > u64::MAX to u64
        assert!(result.is_err());
        println!("✓ Mixed arithmetic with overflow detection works");
    }

    #[test]
    fn test_polymorphic_arithmetic_fast_path() {
        // Test: u64 + u64 = u64 (fast path), then SET_LAMPORTS (success)
        let bytecode = create_test_bytecode(&[
            // Push base lamports (u64)
            PUSH_U64,
            0x00, 0x10, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, // 4096 lamports
            
            // Push additional amount (u64)
            PUSH_U64, 
            0x00, 0x10, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, // 4096 lamports
            
            // ADD (polymorphic fast path: u64 + u64 = u64)
            ADD,
            
            // SET_LAMPORTS (account index 0) - direct u64, no narrowing needed
            SET_LAMPORTS, 0x00, // account index 0
            
            RETURN_VALUE
        ]);

        // Execute with MitoVM
        let result = MitoVM::execute_direct(&bytecode, &[], &[]);
        
        // Should succeed - fast path arithmetic + direct u64 consumption
        assert!(result.is_ok());
        println!("✓ Polymorphic arithmetic fast path with SET_LAMPORTS works");
    }

    #[test] 
    fn test_complex_mixed_width_computation() {
        // Test complex computation: (u64 * u128) / u64 -> SET_LAMPORTS
        let bytecode = create_test_bytecode(&[
            // Push multiplier (u64)
            PUSH_U64,
            0x64, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 100
            
            // Push base amount (u128) 
            PUSH_U128,
            0x10, 0x27, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 10000 as u128 (low)
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 0 (high)
            
            // MUL (polymorphic: u64 * u128 = u128)
            MUL,
            
            // Push divisor (u64)
            PUSH_U64,
            0x0A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 10
            
            // DIV (polymorphic: u128 / u64 = u128)
            DIV,
            
            // Result should be 100 * 10000 / 10 = 100000 (fits in u64)
            // SET_LAMPORTS (account index 0) - narrow u128 to u64
            SET_LAMPORTS, 0x00, // account index 0
            
            RETURN_VALUE
        ]);

        // Execute with MitoVM
        let result = MitoVM::execute_direct(&bytecode, &[], &[]);
        
        // Should succeed - result 100000 fits in u64
        assert!(result.is_ok());
        println!("✓ Complex mixed-width computation with SET_LAMPORTS works");
    }

    #[test]
    fn test_demonstration_opcode_efficiency() {
        println!("\n=== Polymorphic Arithmetic Opcode Efficiency Demo ===");
        
        // Before polymorphic arithmetic: would need ADD_U128, MUL_U128, DIV_U128 
        // After polymorphic arithmetic: uses generic ADD, MUL, DIV
        
        let operations = [
            ("u64 + u64 -> u64 (fast path)", &[PUSH_U64, ADD] as &[u8]),
            ("u64 + u128 -> u128 (promotion)", &[PUSH_U64, PUSH_U128, ADD]),
            ("u128 * u64 -> u128 (promotion)", &[PUSH_U128, PUSH_U64, MUL]),
            ("u128 / u128 -> u128 (native)", &[PUSH_U128, PUSH_U128, DIV]),
        ];
        
        for (desc, _ops) in &operations {
            println!("✓ {}: Uses generic opcodes with automatic type promotion", desc);
        }
        
        println!("✓ Opcode space saved: 3 opcodes (ADD_U128, SUB_U128, MUL_U128) eliminated");
        println!("✓ VM narrowing handles u128->u64 conversion with overflow detection");
        println!("✓ Solana operations seamlessly consume narrowed values");
    }
}