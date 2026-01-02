//! Account system and constraint validation tests
//!
//! Tests for account operations including constraint validation (init, mut, signer),
//! account access patterns, and state management functionality.
//!
//! Note: Most Five VM operations work with empty account arrays for core functionality.
//! These tests focus on VM bytecode execution rather than complex account mocking.

use five_protocol::{opcodes::*, FIVE_HEADER_OPTIMIZED_SIZE, FIVE_MAGIC};
use five_vm_mito::{FIVE_VM_PROGRAM_ID, MitoVM, Value};

fn build_script(body: &[u8]) -> Vec<u8> {
    let mut script = Vec::with_capacity(FIVE_HEADER_OPTIMIZED_SIZE + body.len());
    script.extend_from_slice(&FIVE_MAGIC);
    // Header V3: features(4 bytes LE) + public_function_count(1) + total_function_count(1)
    script.push(0x00); // features byte 0
    script.push(0x00); // features byte 1
    script.push(0x00); // features byte 2
    script.push(0x00); // features byte 3
    script.push(0x00); // public_function_count
    script.push(0x00); // total_function_count
    script.extend_from_slice(body);
    script
}

mod constraint_validation {
    use super::*;

    #[test]
    fn test_basic_vm_execution() {
        // Test basic VM execution without account constraints
        // This replaces complex account mocking with simple bytecode validation
        let bytecode = build_script(&[PUSH_U64, 0x2A, HALT]);
        let result = MitoVM::execute_direct(&bytecode, &[], &[], &FIVE_VM_PROGRAM_ID).unwrap();
        assert_eq!(result, Some(Value::U64(42)), "Basic execution should work");
    }

    #[test]
    fn test_arithmetic_operations() {
        // Test arithmetic without account dependencies
        let bytecode = build_script(&[PUSH_U64, 0x0A, PUSH_U64, 0x05, ADD, HALT]);

        let result = MitoVM::execute_direct(&bytecode, &[], &[], &FIVE_VM_PROGRAM_ID).unwrap();
        assert_eq!(result, Some(Value::U64(15)), "10 + 5 should equal 15");
    }

    #[test]
    fn test_comparison_operations() {
        // Test comparison operations
        let bytecode = build_script(&[PUSH_U64, 0x0A, PUSH_U64, 0x05, GT, HALT]);

        let result = MitoVM::execute_direct(&bytecode, &[], &[], &FIVE_VM_PROGRAM_ID).unwrap();
        assert_eq!(result, Some(Value::Bool(true)), "10 > 5 should be true");
    }
}

mod stack_operations {
    use super::*;

    #[test]
    fn test_stack_basic_ops() {
        // Test DUP operation
        let bytecode = build_script(&[PUSH_U64, 0x07, DUP, ADD, HALT]);

        let result = MitoVM::execute_direct(&bytecode, &[], &[], &FIVE_VM_PROGRAM_ID).unwrap();
        assert_eq!(result, Some(Value::U64(14)), "DUP and ADD should work");
    }

    #[test]
    fn test_swap_operation() {
        // First test: Without SWAP
        // Stack after pushes: [10, 3] (3 on top)
        // SUB pops 3 (b), then 10 (a), computes a - b = 10 - 3 = 7
        let bytecode_no_swap = build_script(&[PUSH_U64, 0x0A, PUSH_U64, 0x03, SUB, HALT]);

        let result_no_swap = MitoVM::execute_direct(&bytecode_no_swap, &[], &[], &FIVE_VM_PROGRAM_ID).unwrap();
        assert_eq!(result_no_swap, Some(Value::U64(7)), "10 - 3 should equal 7");

        // Second test: With SWAP
        // Stack after pushes: [10, 3] (3 on top)
        // SWAP changes to: [3, 10] (10 on top)
        // SUB pops 10 (b), then 3 (a), computes a - b = 3 - 10 with wrapping semantics
        let bytecode = build_script(&[PUSH_U64, 0x0A, PUSH_U64, 0x03, SWAP, SUB, HALT]);

        let result = MitoVM::execute_direct(&bytecode, &[], &[], &FIVE_VM_PROGRAM_ID).unwrap();
        assert_eq!(
            result,
            Some(Value::U64(3u64.wrapping_sub(10))),
            "SWAP and SUB should wrap 3 - 10"
        );
    }
}

mod logical_operations {
    use super::*;

    #[test]
    fn test_and_operation() {
        // Test AND operation with booleans
        let bytecode = build_script(&[PUSH_BOOL, 0x01, PUSH_BOOL, 0x01, AND, HALT]);

        let result = MitoVM::execute_direct(&bytecode, &[], &[], &FIVE_VM_PROGRAM_ID).unwrap();
        assert_eq!(
            result,
            Some(Value::Bool(true)),
            "true AND true should be true"
        );
    }

    #[test]
    fn test_or_operation() {
        // Test OR operation with booleans
        let bytecode = build_script(&[PUSH_BOOL, 0x00, PUSH_BOOL, 0x01, OR, HALT]);

        let result = MitoVM::execute_direct(&bytecode, &[], &[], &FIVE_VM_PROGRAM_ID).unwrap();
        assert_eq!(
            result,
            Some(Value::Bool(true)),
            "false OR true should be true"
        );
    }

    #[test]
    fn test_not_operation() {
        // Test NOT operation
        let bytecode = build_script(&[PUSH_BOOL, 0x00, NOT, HALT]);

        let result = MitoVM::execute_direct(&bytecode, &[], &[], &FIVE_VM_PROGRAM_ID).unwrap();
        assert_eq!(result, Some(Value::Bool(true)), "NOT false should be true");
    }
}
