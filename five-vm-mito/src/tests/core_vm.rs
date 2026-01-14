//! Core VM operation tests
//!
//! Comprehensive unit tests for core VM operations including arithmetic,
//! logical operations, stack operations, and control flow.

#[cfg(all(test, feature = "test-utils"))]
mod core_vm_tests {
    use crate::tests::framework::{AccountUtils, TestUtils};
    use crate::{opcodes, push_bool, push_u64, test_bytecode};
    use crate::{MitoVM, VMError, Value};
    use five_protocol::opcodes::*;

    /// Test basic stack operations
    mod stack_operations {
        use super::*;

        #[test]
        fn test_push_u64() {
            let bytecode = test_bytecode![push_u64!(42)];
            TestUtils::assert_execution_success(&bytecode, Some(Value::U64(42)));
        }

        #[test]
        fn test_push_bool_true() {
            let bytecode = test_bytecode![push_bool!(true)];
            TestUtils::assert_execution_success(&bytecode, Some(Value::Bool(true)));
        }

        #[test]
        fn test_push_bool_false() {
            let bytecode = test_bytecode![push_bool!(false)];
            TestUtils::assert_execution_success(&bytecode, Some(Value::Bool(false)));
        }

        #[test]
        fn test_dup() {
            // PUSH 42, DUP, ADD => 84
            let bytecode = test_bytecode![
                push_u64!(42),
                opcodes![DUP], // 0x10
                opcodes![ADD], // 0x20
            ];
            TestUtils::assert_execution_success(&bytecode, Some(Value::U64(84)));
        }

        #[test]
        fn test_swap() {
            // PUSH 10, PUSH 5, SWAP, SUB => 5 (was 10-5, now 5-10 = -5, but as u64)
            let bytecode = test_bytecode![
                push_u64!(10),
                push_u64!(5),
                opcodes![SWAP], // 0x11
                opcodes![SUB],  // 0x21
            ];
            // After SWAP: stack has [5, 10], SUB does 5-10 which underflows to large u64
            let result = TestUtils::execute_simple(&bytecode);
            assert!(result.is_ok(), "SWAP-SUB should execute without error");
        }

        #[test]
        fn test_pop() {
            // PUSH 42, PUSH 10, POP => should return 42 (10 is popped)
            let bytecode = test_bytecode![
                push_u64!(42),
                push_u64!(10),
                opcodes![POP], // 0x12
            ];
            TestUtils::assert_execution_success(&bytecode, Some(Value::U64(42)));
        }

        #[test]
        fn test_stack_underflow() {
            // Try to POP from empty stack
            let bytecode = test_bytecode![opcodes![POP]];
            TestUtils::assert_execution_error(&bytecode, VMError::StackError);
        }
    }

    /// Test arithmetic operations
    mod arithmetic_operations {
        use super::*;

        #[test]
        fn test_add() {
            let bytecode = test_bytecode![
                push_u64!(100),
                push_u64!(25),
                opcodes![ADD], // 0x20
            ];
            TestUtils::assert_execution_success(&bytecode, Some(Value::U64(125)));
        }

        #[test]
        fn test_subtract() {
            let bytecode = test_bytecode![
                push_u64!(100),
                push_u64!(25),
                opcodes![SUB], // 0x21
            ];
            TestUtils::assert_execution_success(&bytecode, Some(Value::U64(75)));
        }

        #[test]
        fn test_multiply() {
            let bytecode = test_bytecode![
                push_u64!(12),
                push_u64!(8),
                opcodes![MUL], // 0x22
            ];
            TestUtils::assert_execution_success(&bytecode, Some(Value::U64(96)));
        }

        #[test]
        fn test_divide() {
            let bytecode = test_bytecode![
                push_u64!(100),
                push_u64!(4),
                opcodes![DIV], // 0x23
            ];
            TestUtils::assert_execution_success(&bytecode, Some(Value::U64(25)));
        }

        #[test]
        fn test_divide_by_zero() {
            let bytecode = test_bytecode![push_u64!(100), push_u64!(0), opcodes![DIV],];
            TestUtils::assert_execution_error(&bytecode, VMError::DivisionByZero);
        }

        #[test]
        fn test_modulo() {
            let bytecode = test_bytecode![
                push_u64!(17),
                push_u64!(5),
                opcodes![MOD], // 0x24
            ];
            TestUtils::assert_execution_success(&bytecode, Some(Value::U64(2)));
        }

        #[test]
        fn test_arithmetic_stack_underflow() {
            // Try ADD with only one value on stack
            let bytecode = test_bytecode![push_u64!(42), opcodes![ADD],];
            TestUtils::assert_execution_error(&bytecode, VMError::StackError);
        }
    }

    /// Test comparison operations
    mod comparison_operations {
        use super::*;

        #[test]
        fn test_equal() {
            // Test equality: 42 == 42 = true
            let bytecode = test_bytecode![
                push_u64!(42),
                push_u64!(42),
                opcodes![EQ], // 0x26
            ];
            TestUtils::assert_execution_success(&bytecode, Some(Value::Bool(true)));

            // Test inequality: 42 == 10 = false
            let bytecode2 = test_bytecode![push_u64!(42), push_u64!(10), opcodes![EQ],];
            TestUtils::assert_execution_success(&bytecode2, Some(Value::Bool(false)));
        }

        #[test]
        fn test_not_equal() {
            let bytecode = test_bytecode![
                push_u64!(42),
                push_u64!(10),
                opcodes![NEQ], // 0x2A
            ];
            TestUtils::assert_execution_success(&bytecode, Some(Value::Bool(true)));
        }

        #[test]
        fn test_less_than() {
            let bytecode = test_bytecode![
                push_u64!(10),
                push_u64!(42),
                opcodes![LT], // 0x27
            ];
            TestUtils::assert_execution_success(&bytecode, Some(Value::Bool(true)));
        }

        #[test]
        fn test_greater_than() {
            let bytecode = test_bytecode![
                push_u64!(42),
                push_u64!(10),
                opcodes![GT], // 0x25
            ];
            TestUtils::assert_execution_success(&bytecode, Some(Value::Bool(true)));
        }

        #[test]
        fn test_less_than_or_equal() {
            // Test LTE: 10 <= 42 = true
            let bytecode = test_bytecode![
                push_u64!(10),
                push_u64!(42),
                opcodes![LTE], // 0x29
            ];
            TestUtils::assert_execution_success(&bytecode, Some(Value::Bool(true)));

            // Test LTE: 42 <= 42 = true (equal case)
            let bytecode2 = test_bytecode![push_u64!(42), push_u64!(42), opcodes![LTE],];
            TestUtils::assert_execution_success(&bytecode2, Some(Value::Bool(true)));
        }

        #[test]
        fn test_greater_than_or_equal() {
            let bytecode = test_bytecode![
                push_u64!(42),
                push_u64!(10),
                opcodes![GTE], // 0x28
            ];
            TestUtils::assert_execution_success(&bytecode, Some(Value::Bool(true)));
        }
    }

    /// Test logical operations
    mod logical_operations {
        use super::*;

        #[test]
        fn test_logical_and() {
            // true AND true = true
            let bytecode = test_bytecode![
                push_bool!(true),
                push_bool!(true),
                opcodes![AND], // 0x30
            ];
            TestUtils::assert_execution_success(&bytecode, Some(Value::Bool(true)));

            // true AND false = false
            let bytecode2 = test_bytecode![push_bool!(true), push_bool!(false), opcodes![AND],];
            TestUtils::assert_execution_success(&bytecode2, Some(Value::Bool(false)));
        }

        #[test]
        fn test_logical_or() {
            // false OR true = true
            let bytecode = test_bytecode![
                push_bool!(false),
                push_bool!(true),
                opcodes![OR], // 0x31
            ];
            TestUtils::assert_execution_success(&bytecode, Some(Value::Bool(true)));

            // false OR false = false
            let bytecode2 = test_bytecode![push_bool!(false), push_bool!(false), opcodes![OR],];
            TestUtils::assert_execution_success(&bytecode2, Some(Value::Bool(false)));
        }

        #[test]
        fn test_logical_not() {
            // NOT true = false
            let bytecode = test_bytecode![
                push_bool!(true),
                opcodes![NOT], // 0x32
            ];
            TestUtils::assert_execution_success(&bytecode, Some(Value::Bool(false)));

            // NOT false = true
            let bytecode2 = test_bytecode![push_bool!(false), opcodes![NOT],];
            TestUtils::assert_execution_success(&bytecode2, Some(Value::Bool(true)));
        }
    }

    /// Test control flow operations
    mod control_flow_operations {
        use super::*;

        #[test]
        fn test_halt() {
            let bytecode = test_bytecode![push_u64!(42)];
            TestUtils::assert_execution_success(&bytecode, Some(Value::U64(42)));
        }

        #[test]
        fn test_require_success() {
            let bytecode = test_bytecode![
                push_bool!(true),
                opcodes![REQUIRE], // 0x04
                push_u64!(42),     // Should execute after REQUIRE passes
            ];
            TestUtils::assert_execution_success(&bytecode, Some(Value::U64(42)));
        }

        #[test]
        fn test_require_failure() {
            let bytecode = test_bytecode![
                push_bool!(false),
                opcodes![REQUIRE],
                push_u64!(42), // Should not execute
            ];
            TestUtils::assert_execution_error(&bytecode, VMError::ConstraintViolation);
        }

        #[test]
        fn test_return_value() {
            let bytecode = test_bytecode![
                push_u64!(42),
                opcodes![RETURN_VALUE], // 0x07
                push_u64!(99),          // Should not execute after RETURN_VALUE
            ];
            TestUtils::assert_execution_success(&bytecode, Some(Value::U64(42)));
        }
    }

    /// Test error conditions and edge cases
    mod error_conditions {
        use super::*;

        #[test]
        fn test_invalid_instruction() {
            let bytecode = vec![
                0x35, 0x49, 0x56, 0x45, // "5IVE" magic
                0xFF, // Invalid opcode
                0x00, // HALT
            ];
            TestUtils::assert_execution_error(&bytecode, VMError::InvalidInstruction);
        }

        #[test]
        fn test_type_mismatch_arithmetic() {
            // Try to ADD with a bool on stack (should fail during type checking)
            let bytecode = test_bytecode![push_u64!(42), push_bool!(true), opcodes![ADD],];
            TestUtils::assert_execution_error(&bytecode, VMError::TypeMismatch);
        }

        #[test]
        fn test_empty_script() {
            let bytecode = vec![0x35, 0x49, 0x56, 0x45]; // Just magic, no operations
            let result = TestUtils::execute_simple(&bytecode);
            assert!(result.is_ok(), "Empty script should execute successfully");
            assert_eq!(result.unwrap(), None, "Empty script should return None");
        }
    }

    /// Test enhanced V3 pattern fusion operations
    mod pattern_fusion_operations {
        use super::*;

        #[test]
        fn test_push_zero() {
            let bytecode = test_bytecode![opcodes![PUSH_ZERO]]; // 0x65
            TestUtils::assert_execution_success(&bytecode, Some(Value::U64(0)));
        }

        #[test]
        fn test_push_one() {
            let bytecode = test_bytecode![opcodes![PUSH_ONE]]; // 0xA2
            TestUtils::assert_execution_success(&bytecode, Some(Value::U64(1)));
        }

        #[test]
        fn test_dup_add() {
            // Push 7, then DUP_ADD should result in 14
            let bytecode = test_bytecode![
                push_u64!(7),
                opcodes![DUP_ADD], // 0xA7
            ];
            TestUtils::assert_execution_success(&bytecode, Some(Value::U64(14)));
        }

        #[test]
        fn test_swap_sub() {
            // Push 10, Push 6, SWAP, SUB => after SWAP: [6, 10], SUB: 6-10 = 0 (saturating)
            let bytecode = test_bytecode![
                push_u64!(10),
                push_u64!(6),
                opcodes![SWAP], // 0x13
                opcodes![SUB],  // 0x21
            ];
            TestUtils::assert_execution_success(&bytecode, Some(Value::U64(0)));
        }

        #[test]
        fn test_validate_amount_nonzero() {
            // Test with non-zero amount (should pass)
            let bytecode = test_bytecode![
                push_u64!(100),
                opcodes![VALIDATE_AMOUNT_NONZERO], // 0x66
                push_u64!(42),
            ];
            TestUtils::assert_execution_success(&bytecode, Some(Value::U64(42)));

            // Test with zero amount (should fail)
            let bytecode_fail = test_bytecode![push_u64!(0), opcodes![VALIDATE_AMOUNT_NONZERO],];
            TestUtils::assert_execution_error(&bytecode_fail, VMError::ConstraintViolation);
        }

        #[test]
        fn test_eq_zero_jump() {
            // Test with zero - should jump
            let mut bytecode = vec![0x35, 0x49, 0x56, 0x45]; // magic
            bytecode.extend_from_slice(&push_u64!(0));
            bytecode.extend_from_slice(&[EQ_ZERO_JUMP, 0x03]); // Jump 3 bytes forward
            bytecode.extend_from_slice(&push_u64!(99)); // This should be skipped
            bytecode.extend_from_slice(&push_u64!(42)); // This should execute
            bytecode.push(0x00); // HALT

            TestUtils::assert_execution_success(&bytecode, Some(Value::U64(42)));

            // Test with non-zero - should not jump
            let mut bytecode2 = vec![0x35, 0x49, 0x56, 0x45]; // magic
            bytecode2.extend_from_slice(&push_u64!(5));
            bytecode2.extend_from_slice(&[EQ_ZERO_JUMP, 0x01]); // Would jump 1 byte
            bytecode2.extend_from_slice(&push_u64!(77)); // This should execute
            bytecode2.push(0x00); // HALT

            TestUtils::assert_execution_success(&bytecode2, Some(Value::U64(77)));
        }
    }
}
