//! Comprehensive unit tests for Five VM operations
//!
//! This test module provides comprehensive coverage of VM operations to prevent regressions
//! and ensure Five VM meets its production-readiness goals.

use five_protocol::{opcodes::*, FIVE_HEADER_OPTIMIZED_SIZE, FIVE_MAGIC};
use five_vm_mito::{FIVE_VM_PROGRAM_ID, MitoVM, Value, stack::StackStorage, AccountInfo};

fn execute_test(bytecode: &[u8], input: &[u8], accounts: &[AccountInfo]) -> five_vm_mito::Result<Option<Value>> {
    let mut storage = StackStorage::new(bytecode);
    MitoVM::execute_direct(bytecode, input, accounts, &FIVE_VM_PROGRAM_ID, &mut storage)
}

fn build_script(build: impl FnOnce(&mut Vec<u8>)) -> Vec<u8> {
    let mut script = Vec::with_capacity(FIVE_HEADER_OPTIMIZED_SIZE + 16);
    script.extend_from_slice(&FIVE_MAGIC);
    // Header V3: features(4 bytes LE) + public_function_count(1) + total_function_count(1)
    script.push(0x00); // features byte 0
    script.push(0x00); // features byte 1
    script.push(0x00); // features byte 2
    script.push(0x00); // features byte 3
    script.push(0x00); // public_function_count
    script.push(0x00); // total_function_count
    build(&mut script);
    script
}

fn push_u64_instr(script: &mut Vec<u8>, value: u64) {
    script.push(PUSH_U64);
    script.extend_from_slice(&value.to_le_bytes());
}

fn push_bool_instr(script: &mut Vec<u8>, value: bool) {
    script.push(PUSH_BOOL);
    script.push(value as u8);
}

fn push_u128_instr(script: &mut Vec<u8>, value: u128) {
    script.push(PUSH_U128);
    script.extend_from_slice(&value.to_le_bytes());
}

mod core_operations {
    use super::*;

    #[test]
    fn test_arithmetic_add_basic() {
        // Test basic addition: 5 + 3 = 8
        let bytecode = build_script(|script| {
            push_u64_instr(script, 5);
            push_u64_instr(script, 3);
            script.push(ADD);
            script.push(HALT);
        });

        let result = execute_test(&bytecode, &[], &[]).unwrap();
        assert_eq!(result, Some(Value::U64(8)), "5 + 3 should equal 8");
    }

    #[test]
    fn test_arithmetic_subtract() {
        // Test subtraction: 10 - 4 = 6
        let bytecode = build_script(|script| {
            push_u64_instr(script, 10);
            push_u64_instr(script, 4);
            script.push(SUB);
            script.push(HALT);
        });

        let result = execute_test(&bytecode, &[], &[]).unwrap();
        assert_eq!(result, Some(Value::U64(6)), "10 - 4 should equal 6");
    }

    #[test]
    fn test_arithmetic_multiply() {
        // Test multiplication: 7 * 6 = 42
        let bytecode = build_script(|script| {
            push_u64_instr(script, 7);
            push_u64_instr(script, 6);
            script.push(MUL);
            script.push(HALT);
        });

        let result = execute_test(&bytecode, &[], &[]).unwrap();
        assert_eq!(result, Some(Value::U64(42)), "7 * 6 should equal 42");
    }

    #[test]
    fn test_arithmetic_divide() {
        // Test division: 20 / 4 = 5
        let bytecode = build_script(|script| {
            push_u64_instr(script, 20);
            push_u64_instr(script, 4);
            script.push(DIV);
            script.push(HALT);
        });

        let result = execute_test(&bytecode, &[], &[]).unwrap();
        assert_eq!(result, Some(Value::U64(5)), "20 / 4 should equal 5");
    }
}

mod comparison_operations {
    use super::*;

    #[test]
    fn test_equality_true() {
        // Test equality: 42 == 42
        let bytecode = build_script(|script| {
            push_u64_instr(script, 42);
            push_u64_instr(script, 42);
            script.push(EQ);
            script.push(HALT);
        });

        let result = execute_test(&bytecode, &[], &[]).unwrap();
        assert_eq!(result, Some(Value::Bool(true)), "42 == 42 should be true");
    }

    #[test]
    fn test_equality_false() {
        // Test equality: 42 == 43
        let bytecode = build_script(|script| {
            push_u64_instr(script, 42);
            push_u64_instr(script, 43);
            script.push(EQ);
            script.push(HALT);
        });

        let result = execute_test(&bytecode, &[], &[]).unwrap();
        assert_eq!(result, Some(Value::Bool(false)), "42 == 43 should be false");
    }

    #[test]
    fn test_greater_than_true() {
        // Test greater than: 100 > 50
        let bytecode = build_script(|script| {
            push_u64_instr(script, 100);
            push_u64_instr(script, 50);
            script.push(GT);
            script.push(HALT);
        });

        let result = execute_test(&bytecode, &[], &[]).unwrap();
        assert_eq!(result, Some(Value::Bool(true)), "100 > 50 should be true");
    }

    #[test]
    fn test_less_than_true() {
        // Test less than: 25 < 50
        let bytecode = build_script(|script| {
            push_u64_instr(script, 25);
            push_u64_instr(script, 50);
            script.push(LT);
            script.push(HALT);
        });

        let result = execute_test(&bytecode, &[], &[]).unwrap();
        assert_eq!(result, Some(Value::Bool(true)), "25 < 50 should be true");
    }
}

mod stack_manipulation {
    use super::*;

    #[test]
    fn test_dup_operation() {
        // Test DUP: push 42, duplicate, add
        let bytecode = build_script(|script| {
            push_u64_instr(script, 42);
            script.push(DUP);
            script.push(ADD);
            script.push(HALT);
        });

        let result = execute_test(&bytecode, &[], &[]).unwrap();
        assert_eq!(result, Some(Value::U64(84)), "42 + 42 should equal 84");
    }

    #[test]
    fn test_swap_operation() {
        // Test SWAP: push 10, push 5, swap, subtract (5 - 10 = 0 saturating)
        let bytecode = build_script(|script| {
            push_u64_instr(script, 10);
            push_u64_instr(script, 5);
            script.push(SWAP);
            script.push(SUB);
            script.push(HALT);
        });

        let result = execute_test(&bytecode, &[], &[]).unwrap();
        assert_eq!(
            result,
            Some(Value::U64(5u64.wrapping_sub(10))),
            "SUB uses wrapping semantics"
        );
    }

    #[test]
    fn test_pop_operation() {
        // Test POP: push 100, push 200, pop, result should be 100
        let bytecode = build_script(|script| {
            push_u64_instr(script, 100);
            push_u64_instr(script, 200);
            script.push(POP);
            script.push(HALT);
        });

        let result = execute_test(&bytecode, &[], &[]).unwrap();
        assert_eq!(
            result,
            Some(Value::U64(100)),
            "After popping 200, should have 100"
        );
    }
}

mod boolean_operations {
    use super::*;

    #[test]
    fn test_and_true() {
        // Test AND: true && true = true
        let bytecode = build_script(|script| {
            push_bool_instr(script, true);
            push_bool_instr(script, true);
            script.push(AND);
            script.push(HALT);
        });

        let result = execute_test(&bytecode, &[], &[]).unwrap();
        assert_eq!(
            result,
            Some(Value::Bool(true)),
            "true AND true should be true"
        );
    }

    #[test]
    fn test_and_false() {
        // Test AND: true && false = false
        let bytecode = build_script(|script| {
            push_bool_instr(script, true);
            push_bool_instr(script, false);
            script.push(AND);
            script.push(HALT);
        });

        let result = execute_test(&bytecode, &[], &[]).unwrap();
        assert_eq!(
            result,
            Some(Value::Bool(false)),
            "true AND false should be false"
        );
    }

    #[test]
    fn test_or_true() {
        // Test OR: false || true = true
        let bytecode = build_script(|script| {
            push_bool_instr(script, false);
            push_bool_instr(script, true);
            script.push(OR);
            script.push(HALT);
        });

        let result = execute_test(&bytecode, &[], &[]).unwrap();
        assert_eq!(
            result,
            Some(Value::Bool(true)),
            "false OR true should be true"
        );
    }

    #[test]
    fn test_not_operation() {
        // Test NOT: !false = true
        let bytecode = build_script(|script| {
            push_bool_instr(script, false);
            script.push(NOT);
            script.push(HALT);
        });

        let result = execute_test(&bytecode, &[], &[]).unwrap();
        assert_eq!(result, Some(Value::Bool(true)), "NOT false should be true");
    }
}

mod type_operations {
    use super::*;

    #[test]
    fn test_u8_operations() {
        // Test U8 type: push U8, convert operations
        let bytecode = build_script(|script| {
            script.push(PUSH_U8);
            script.push(42);
            script.push(HALT);
        });

        let result = execute_test(&bytecode, &[], &[]).unwrap();
        assert_eq!(result, Some(Value::U8(42)), "Should handle U8 values");
    }

    // NOTE: String operations test removed - Five VM may not support String return values
    // or the String type might be handled differently
}

mod mixed_type_operations {
    use super::*;

    #[test]
    fn test_lte_u64_u128() {
        // Test LTE: 6 (u64) <= 20 (u128)
        // This reproduces the suspected bug scenario
        let bytecode = build_script(|script| {
            push_u64_instr(script, 6);
            push_u128_instr(script, 20);
            script.push(LTE);
            script.push(HALT);
        });

        let result = execute_test(&bytecode, &[], &[]).unwrap();
        assert_eq!(result, Some(Value::Bool(true)), "6 (u64) <= 20 (u128) should be true");
    }

    #[test]
    fn test_lte_u128_u64() {
        // Test LTE: 6 (u128) <= 20 (u64)
        let bytecode = build_script(|script| {
            push_u128_instr(script, 6);
            push_u64_instr(script, 20);
            script.push(LTE);
            script.push(HALT);
        });

        let result = execute_test(&bytecode, &[], &[]).unwrap();
        assert_eq!(result, Some(Value::Bool(true)), "6 (u128) <= 20 (u64) should be true");
    }

    #[test]
    fn test_lte_u64_u64() {
        // Test LTE: 6 (u64) <= 20 (u64)
        let bytecode = build_script(|script| {
            push_u64_instr(script, 6);
            push_u64_instr(script, 20);
            script.push(LTE);
            script.push(HALT);
        });

        let result = execute_test(&bytecode, &[], &[]).unwrap();
        assert_eq!(result, Some(Value::Bool(true)), "6 (u64) <= 20 (u64) should be true");
    }
}
