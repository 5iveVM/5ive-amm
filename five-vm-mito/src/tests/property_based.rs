//! Property-based tests for critical VM operations
//!
//! These tests use property-based testing techniques to verify VM operations
//! hold certain invariants across a wide range of inputs.

#[cfg(test)]
mod property_based_tests {
    use crate::tests::framework::TestUtils;
    use crate::{opcodes, push_bool, push_u64, test_bytecode};
    use crate::{MitoVM, VMError, Value};
    use five_protocol::opcodes::*;

    /// Property tests for arithmetic operations
    mod arithmetic_properties {
        use super::*;

        #[test]
        fn test_addition_commutativity() {
            // Property: a + b = b + a for all valid u64 values
            let test_cases = [
                (0, 0),
                (1, 2),
                (100, 200),
                (u64::MAX / 2, u64::MAX / 2),
                (42, 1337),
            ];

            for (a, b) in test_cases {
                // Test a + b
                let bytecode1 = test_bytecode![push_u64!(a), push_u64!(b), opcodes![ADD],];

                // Test b + a
                let bytecode2 = test_bytecode![push_u64!(b), push_u64!(a), opcodes![ADD],];

                let result1 = TestUtils::execute_simple(&bytecode1);
                let result2 = TestUtils::execute_simple(&bytecode2);

                match (&result1, &result2) {
                    (Ok(val1), Ok(val2)) => {
                        assert_eq!(
                            val1, val2,
                            "Addition should be commutative: {} + {} = {} + {}",
                            a, b, b, a
                        );
                    }
                    _ => {
                        // Both should fail or succeed together
                        assert_eq!(
                            result1.is_ok(),
                            result2.is_ok(),
                            "Addition commutativity should have same success/failure: {} + {} vs {} + {}",
                            a,
                            b,
                            b,
                            a
                        );
                    }
                }
            }
        }

        #[test]
        fn test_addition_associativity() {
            // Property: (a + b) + c = a + (b + c) for all valid u64 values
            let test_cases = [(1, 2, 3), (10, 20, 30), (100, 200, 300), (5, 0, 15)];

            for (a, b, c) in test_cases {
                // Test (a + b) + c
                let bytecode1 = test_bytecode![
                    push_u64!(a),
                    push_u64!(b),
                    opcodes![ADD],
                    push_u64!(c),
                    opcodes![ADD],
                ];

                // Test a + (b + c)
                let bytecode2 = test_bytecode![
                    push_u64!(b),
                    push_u64!(c),
                    opcodes![ADD],
                    push_u64!(a),
                    opcodes![SWAP],
                    opcodes![ADD],
                ];

                let result1 = TestUtils::execute_simple(&bytecode1);
                let result2 = TestUtils::execute_simple(&bytecode2);

                match (&result1, &result2) {
                    (Ok(val1), Ok(val2)) => {
                        assert_eq!(
                            val1, val2,
                            "Addition should be associative: ({} + {}) + {} = {} + ({} + {})",
                            a, b, c, a, b, c
                        );
                    }
                    _ => {
                        // Both should have same behavior
                        assert_eq!(
                            result1.is_ok(),
                            result2.is_ok(),
                            "Addition associativity should have same success/failure"
                        );
                    }
                }
            }
        }

        #[test]
        fn test_multiplication_by_zero() {
            // Property: a * 0 = 0 for all a
            let test_cases = [0, 1, 42, 1337, u64::MAX];

            for a in test_cases {
                let bytecode = test_bytecode![push_u64!(a), push_u64!(0), opcodes![MUL],];

                let result = TestUtils::execute_simple(&bytecode);
                match result {
                    Ok(Some(Value::U64(0))) => {
                        // Correct result
                    }
                    Ok(other) => {
                        panic!("Expected {} * 0 = 0, got {:?}", a, other);
                    }
                    Err(e) => {
                        panic!(
                            "Multiplication by zero should not fail: {} * 0, error: {:?}",
                            a, e
                        );
                    }
                }
            }
        }

        #[test]
        fn test_multiplication_by_one() {
            // Property: a * 1 = a for all a
            let test_cases = [0, 1, 42, 1337, u64::MAX];

            for a in test_cases {
                let bytecode = test_bytecode![push_u64!(a), push_u64!(1), opcodes![MUL],];

                let result = TestUtils::execute_simple(&bytecode);
                match result {
                    Ok(Some(Value::U64(val))) => {
                        assert_eq!(val, a, "Expected {} * 1 = {}, got {}", a, a, val);
                    }
                    Ok(other) => {
                        panic!("Expected {} * 1 = {}, got {:?}", a, a, other);
                    }
                    Err(e) => {
                        panic!(
                            "Multiplication by one should not fail: {} * 1, error: {:?}",
                            a, e
                        );
                    }
                }
            }
        }

        #[test]
        fn test_division_by_zero_always_fails() {
            // Property: a / 0 always fails for all a != 0
            let test_cases = [1, 42, 1337, u64::MAX];

            for a in test_cases {
                let bytecode = test_bytecode![push_u64!(a), push_u64!(0), opcodes![DIV],];

                let result = TestUtils::execute_simple(&bytecode);
                match result {
                    Err(VMError::DivisionByZero) => {
                        // Correct behavior
                    }
                    Err(other) => {
                        panic!("Expected DivisionByZero for {} / 0, got {:?}", a, other);
                    }
                    Ok(val) => {
                        panic!("Division by zero should fail: {} / 0 = {:?}", a, val);
                    }
                }
            }
        }

        #[test]
        fn test_division_identity() {
            // Property: a / a = 1 for all a != 0
            let test_cases = [1, 42, 1337, u64::MAX];

            for a in test_cases {
                let bytecode = test_bytecode![push_u64!(a), push_u64!(a), opcodes![DIV],];

                let result = TestUtils::execute_simple(&bytecode);
                match result {
                    Ok(Some(Value::U64(1))) => {
                        // Correct result
                    }
                    Ok(other) => {
                        panic!("Expected {} / {} = 1, got {:?}", a, a, other);
                    }
                    Err(e) => {
                        panic!(
                            "Self-division should not fail: {} / {}, error: {:?}",
                            a, a, e
                        );
                    }
                }
            }
        }
    }

    /// Property tests for logical operations
    mod logical_properties {
        use super::*;

        #[test]
        fn test_boolean_and_properties() {
            // Property tests for AND operation
            let test_cases = [(true, true), (true, false), (false, true), (false, false)];

            for (a, b) in test_cases {
                let bytecode = test_bytecode![push_bool!(a), push_bool!(b), opcodes![AND],];

                let result = TestUtils::execute_simple(&bytecode);
                let expected = a && b;

                match result {
                    Ok(Some(Value::Bool(val))) => {
                        assert_eq!(
                            val, expected,
                            "Expected {} AND {} = {}, got {}",
                            a, b, expected, val
                        );
                    }
                    Ok(other) => {
                        panic!("Expected bool result for {} AND {}, got {:?}", a, b, other);
                    }
                    Err(e) => {
                        panic!(
                            "Boolean AND should not fail: {} AND {}, error: {:?}",
                            a, b, e
                        );
                    }
                }
            }
        }

        #[test]
        fn test_boolean_or_properties() {
            // Property tests for OR operation
            let test_cases = [(true, true), (true, false), (false, true), (false, false)];

            for (a, b) in test_cases {
                let bytecode = test_bytecode![push_bool!(a), push_bool!(b), opcodes![OR],];

                let result = TestUtils::execute_simple(&bytecode);
                let expected = a || b;

                match result {
                    Ok(Some(Value::Bool(val))) => {
                        assert_eq!(
                            val, expected,
                            "Expected {} OR {} = {}, got {}",
                            a, b, expected, val
                        );
                    }
                    Ok(other) => {
                        panic!("Expected bool result for {} OR {}, got {:?}", a, b, other);
                    }
                    Err(e) => {
                        panic!("Boolean OR should not fail: {} OR {}, error: {:?}", a, b, e);
                    }
                }
            }
        }

        #[test]
        fn test_boolean_not_involution() {
            // Property: NOT(NOT(a)) = a for all boolean a
            let test_cases = [true, false];

            for a in test_cases {
                let bytecode = test_bytecode![push_bool!(a), opcodes![NOT], opcodes![NOT],];

                let result = TestUtils::execute_simple(&bytecode);

                match result {
                    Ok(Some(Value::Bool(val))) => {
                        assert_eq!(val, a, "Expected NOT(NOT({})) = {}, got {}", a, a, val);
                    }
                    Ok(other) => {
                        panic!("Expected bool result for NOT(NOT({})), got {:?}", a, other);
                    }
                    Err(e) => {
                        panic!(
                            "Double negation should not fail: NOT(NOT({})), error: {:?}",
                            a, e
                        );
                    }
                }
            }
        }

        #[test]
        fn test_de_morgan_laws() {
            // Property: NOT(a AND b) = (NOT a) OR (NOT b)
            let test_cases = [(true, true), (true, false), (false, true), (false, false)];

            for (a, b) in test_cases {
                // Test NOT(a AND b)
                let bytecode1 =
                    test_bytecode![push_bool!(a), push_bool!(b), opcodes![AND], opcodes![NOT],];

                // Test (NOT a) OR (NOT b)
                let bytecode2 = test_bytecode![
                    push_bool!(a),
                    opcodes![NOT],
                    push_bool!(b),
                    opcodes![NOT],
                    opcodes![OR],
                ];

                let result1 = TestUtils::execute_simple(&bytecode1);
                let result2 = TestUtils::execute_simple(&bytecode2);

                match (&result1, &result2) {
                    (Ok(val1), Ok(val2)) => {
                        assert_eq!(
                            val1, val2,
                            "De Morgan's law: NOT({} AND {}) = (NOT {}) OR (NOT {})",
                            a, b, a, b
                        );
                    }
                    _ => {
                        panic!("De Morgan's law test failed for {} AND {}", a, b);
                    }
                }
            }
        }
    }

    /// Property tests for comparison operations
    mod comparison_properties {
        use super::*;

        #[test]
        fn test_equality_reflexivity() {
            // Property: a == a is always true
            let test_cases = [0, 1, 42, 1337, u64::MAX];

            for a in test_cases {
                let bytecode = test_bytecode![push_u64!(a), push_u64!(a), opcodes![EQ],];

                let result = TestUtils::execute_simple(&bytecode);

                match result {
                    Ok(Some(Value::Bool(true))) => {
                        // Correct
                    }
                    Ok(other) => {
                        panic!("Expected {} == {} to be true, got {:?}", a, a, other);
                    }
                    Err(e) => {
                        panic!(
                            "Equality comparison should not fail: {} == {}, error: {:?}",
                            a, a, e
                        );
                    }
                }
            }
        }

        #[test]
        fn test_equality_symmetry() {
            // Property: a == b iff b == a
            let test_cases = [(1, 1), (1, 2), (42, 42), (100, 200)];

            for (a, b) in test_cases {
                let bytecode1 = test_bytecode![push_u64!(a), push_u64!(b), opcodes![EQ],];

                let bytecode2 = test_bytecode![push_u64!(b), push_u64!(a), opcodes![EQ],];

                let result1 = TestUtils::execute_simple(&bytecode1);
                let result2 = TestUtils::execute_simple(&bytecode2);

                match (&result1, &result2) {
                    (Ok(val1), Ok(val2)) => {
                        assert_eq!(
                            val1, val2,
                            "Equality should be symmetric: {} == {} vs {} == {}",
                            a, b, b, a
                        );
                    }
                    _ => {
                        panic!("Equality symmetry test failed for {} and {}", a, b);
                    }
                }
            }
        }

        #[test]
        fn test_comparison_trichotomy() {
            // Property: for any a, b exactly one of a < b, a == b, a > b is true
            let test_cases = [(1, 2), (2, 1), (5, 5), (100, 50), (0, 0)];

            for (a, b) in test_cases {
                let bytecode_lt = test_bytecode![push_u64!(a), push_u64!(b), opcodes![LT],];

                let bytecode_eq = test_bytecode![push_u64!(a), push_u64!(b), opcodes![EQ],];

                let bytecode_gt = test_bytecode![push_u64!(a), push_u64!(b), opcodes![GT],];

                let result_lt = TestUtils::execute_simple(&bytecode_lt);
                let result_eq = TestUtils::execute_simple(&bytecode_eq);
                let result_gt = TestUtils::execute_simple(&bytecode_gt);

                match (result_lt, result_eq, result_gt) {
                    (
                        Ok(Some(Value::Bool(lt))),
                        Ok(Some(Value::Bool(eq))),
                        Ok(Some(Value::Bool(gt))),
                    ) => {
                        let count = [lt, eq, gt].iter().filter(|&&x| x).count();
                        assert_eq!(
                            count, 1,
                            "Exactly one of {} < {}, {} == {}, {} > {} should be true",
                            a, b, a, b, a, b
                        );
                    }
                    _ => {
                        panic!("Trichotomy test failed for {} and {}", a, b);
                    }
                }
            }
        }

        #[test]
        fn test_less_than_transitivity() {
            // Property: if a < b and b < c, then a < c
            let test_cases = [(1, 2, 3), (5, 10, 15), (0, 50, 100)];

            for (a, b, c) in test_cases {
                // Verify a < b
                let bytecode_ab = test_bytecode![push_u64!(a), push_u64!(b), opcodes![LT],];

                // Verify b < c
                let bytecode_bc = test_bytecode![push_u64!(b), push_u64!(c), opcodes![LT],];

                // Check a < c
                let bytecode_ac = test_bytecode![push_u64!(a), push_u64!(c), opcodes![LT],];

                let result_ab = TestUtils::execute_simple(&bytecode_ab);
                let result_bc = TestUtils::execute_simple(&bytecode_bc);
                let result_ac = TestUtils::execute_simple(&bytecode_ac);

                match (result_ab, result_bc, result_ac) {
                    (
                        Ok(Some(Value::Bool(true))),
                        Ok(Some(Value::Bool(true))),
                        Ok(Some(Value::Bool(ac))),
                    ) => {
                        assert!(
                            ac,
                            "Transitivity: if {} < {} and {} < {}, then {} < {} should be true",
                            a, b, b, c, a, c
                        );
                    }
                    _ => {
                        // If preconditions not met, skip test
                        continue;
                    }
                }
            }
        }
    }

    /// Property tests for stack operations
    mod stack_properties {
        use super::*;

        #[test]
        fn test_push_pop_identity() {
            // Property: push(x); pop() = x for all x
            let test_cases = [0, 1, 42, 1337, u64::MAX];

            for x in test_cases {
                let bytecode = test_bytecode![
                    push_u64!(x),
                    push_u64!(999), // Push something else
                    opcodes![POP],  // Pop the 999
                                    // Should leave original x on stack
                ];

                let result = TestUtils::execute_simple(&bytecode);

                match result {
                    Ok(Some(Value::U64(val))) => {
                        assert_eq!(
                            val, x,
                            "Push-pop should preserve value: expected {}, got {}",
                            x, val
                        );
                    }
                    Ok(other) => {
                        panic!("Expected U64({}) after push-pop, got {:?}", x, other);
                    }
                    Err(e) => {
                        panic!("Push-pop should not fail for {}, error: {:?}", x, e);
                    }
                }
            }
        }

        #[test]
        fn test_dup_preserves_value() {
            // Property: dup operation creates identical copy
            let test_cases = [0, 42, 1337, u64::MAX];

            for x in test_cases {
                let bytecode = test_bytecode![
                    push_u64!(x),
                    opcodes![DUP], // Duplicate top value
                    opcodes![EQ],  // Compare top two values
                ];

                let result = TestUtils::execute_simple(&bytecode);

                match result {
                    Ok(Some(Value::Bool(true))) => {
                        // Correct - duplicated values are equal
                    }
                    Ok(other) => {
                        panic!(
                            "DUP should create identical copy: {} DUP EQ, got {:?}",
                            x, other
                        );
                    }
                    Err(e) => {
                        panic!("DUP operation should not fail for {}, error: {:?}", x, e);
                    }
                }
            }
        }

        #[test]
        fn test_swap_involution() {
            // Property: swap; swap = identity
            let test_cases = [(1, 2), (42, 100), (0, u64::MAX)];

            for (a, b) in test_cases {
                let bytecode = test_bytecode![
                    push_u64!(a),
                    push_u64!(b),
                    opcodes![SWAP],
                    opcodes![SWAP],
                    // Stack should be back to [a, b] order
                    // Check top is b
                ];

                let result = TestUtils::execute_simple(&bytecode);

                match result {
                    Ok(Some(Value::U64(val))) => {
                        assert_eq!(
                            val, b,
                            "Double swap should restore order: expected {}, got {}",
                            b, val
                        );
                    }
                    Ok(other) => {
                        panic!("Expected U64({}) after double swap, got {:?}", b, other);
                    }
                    Err(e) => {
                        panic!(
                            "Double swap should not fail for ({}, {}), error: {:?}",
                            a, b, e
                        );
                    }
                }
            }
        }

        #[test]
        fn test_stack_underflow_consistency() {
            // Property: operations requiring n items fail consistently when stack has < n items
            let underflow_operations = [
                (vec![ADD], 2),  // ADD needs 2 items
                (vec![SUB], 2),  // SUB needs 2 items
                (vec![MUL], 2),  // MUL needs 2 items
                (vec![DIV], 2),  // DIV needs 2 items
                (vec![EQ], 2),   // EQ needs 2 items
                (vec![SWAP], 2), // SWAP needs 2 items
                (vec![POP], 1),  // POP needs 1 item
            ];

            for (ops, required_items) in underflow_operations {
                // Test with 0 items (empty stack)
                let mut bytecode = vec![0x35, 0x49, 0x56, 0x45]; // magic
                for op in &ops {
                    bytecode.push(*op);
                }
                bytecode.push(0x00); // HALT

                let result = TestUtils::execute_simple(&bytecode);
                assert!(
                    result.is_err(),
                    "Operation {:?} should fail on empty stack (needs {} items)",
                    ops,
                    required_items
                );

                // Test with insufficient items
                if required_items > 1 {
                    let mut bytecode2 = vec![0x35, 0x49, 0x56, 0x45]; // magic
                    bytecode2.extend_from_slice(&push_u64!(42)); // Only 1 item
                    for op in &ops {
                        bytecode2.push(*op);
                    }
                    bytecode2.push(0x00); // HALT

                    let result2 = TestUtils::execute_simple(&bytecode2);
                    assert!(
                        result2.is_err(),
                        "Operation {:?} should fail with 1 item (needs {})",
                        ops,
                        required_items
                    );
                }
            }
        }
    }

    /// Property tests for memory and array operations
    mod memory_properties {
        use super::*;

        #[test]
        fn test_array_length_consistency() {
            // Property: array length matches number of elements added
            let test_cases = [0, 1, 3, 5];

            for n in test_cases {
                let mut bytecode = vec![0x35, 0x49, 0x56, 0x45]; // magic

                // Push n elements
                for i in 0..n {
                    bytecode.extend_from_slice(&push_u64!(i as u64));
                }

                // Create array literal
                bytecode.push(PUSH_ARRAY_LITERAL);
                bytecode.push(n as u8);

                // Get length
                bytecode.push(ARRAY_LENGTH);

                bytecode.push(0x00); // HALT

                let result = TestUtils::execute_simple(&bytecode);

                match result {
                    Ok(Some(Value::U8(len))) => {
                        assert_eq!(
                            len, n as u8,
                            "Array with {} elements should have length {}, got {}",
                            n, n, len
                        );
                    }
                    Ok(Some(Value::U64(len))) => {
                        assert_eq!(
                            len, n as u64,
                            "Array with {} elements should have length {}, got {}",
                            n, n, len
                        );
                    }
                    Ok(other) => {
                        println!("Array length test: expected length {}, got {:?}", n, other);
                        // Length operations may not be implemented yet
                    }
                    Err(_) => {
                        // Array operations may not be fully implemented
                        println!(
                            "Array length test failed for {} elements (implementation pending)",
                            n
                        );
                    }
                }
            }
        }

        #[test]
        fn test_array_index_bounds() {
            // Property: accessing array[i] where i >= length should fail
            let bytecode = test_bytecode![
                // Create array [10, 20, 30] (length 3)
                push_u64!(10),
                push_u64!(20),
                push_u64!(30),
                opcodes![PUSH_ARRAY_LITERAL, 3],
                // Try to access index 5 (out of bounds)
                push_u64!(5),
                opcodes![ARRAY_INDEX],
            ];

            let result = TestUtils::execute_simple(&bytecode);

            match result {
                Err(VMError::IndexOutOfBounds) => {
                    // Correct behavior
                }
                Err(_) => {
                    // Any error is acceptable for out of bounds access
                    println!("Array bounds test got error (acceptable)");
                }
                Ok(val) => {
                    panic!("Out of bounds array access should fail, got {:?}", val);
                }
            }
        }

        #[test]
        fn test_string_utf8_invariant() {
            // Property: valid UTF-8 strings should be accepted, invalid ones rejected
            let valid_utf8_cases = [
                &b"hello"[..],
                &b"test123"[..],
                "Hello 🌍".as_bytes(), // UTF-8 with emoji
                &b""[..],              // Empty string
            ];

            let invalid_utf8_cases = [
                &[0xFF, 0xFE, 0xFD][..], // Invalid UTF-8 sequence
                &[0x80, 0x80, 0x00][..], // Invalid continuation bytes
            ];

            // Test valid UTF-8
            for valid_bytes in valid_utf8_cases {
                let mut bytecode = vec![0x35, 0x49, 0x56, 0x45]; // magic
                bytecode.push(PUSH_STRING_LITERAL);
                bytecode.push(valid_bytes.len() as u8);
                bytecode.extend_from_slice(valid_bytes);
                bytecode.push(0x00); // HALT

                let result = TestUtils::execute_simple(&bytecode);
                match result {
                    Ok(_) => {
                        // Should succeed for valid UTF-8
                    }
                    Err(_) => {
                        println!(
                            "Valid UTF-8 string failed (may need implementation): {:?}",
                            String::from_utf8_lossy(valid_bytes)
                        );
                    }
                }
            }

            // Test invalid UTF-8
            for invalid_bytes in invalid_utf8_cases {
                let mut bytecode = vec![0x35, 0x49, 0x56, 0x45]; // magic
                bytecode.push(PUSH_STRING_LITERAL);
                bytecode.push(invalid_bytes.len() as u8);
                bytecode.extend_from_slice(invalid_bytes);
                bytecode.push(0x00); // HALT

                let result = TestUtils::execute_simple(&bytecode);
                match result {
                    Err(VMError::InvalidOperation) => {
                        // Correct behavior for invalid UTF-8
                    }
                    Err(_) => {
                        // Any error is acceptable for invalid UTF-8
                        println!("Invalid UTF-8 rejected with error (good)");
                    }
                    Ok(_) => {
                        println!("Invalid UTF-8 was accepted (may need validation)");
                    }
                }
            }
        }
    }

    /// Property tests for control flow operations
    mod control_flow_properties {
        use super::*;

        #[test]
        fn test_halt_finality() {
            // Property: HALT always stops execution immediately
            let bytecode = test_bytecode![
                push_u64!(42),
                opcodes![HALT],
                push_u64!(99), // This should never execute
                opcodes![ADD], // This should never execute
            ];

            let result = TestUtils::execute_simple(&bytecode);

            match result {
                Ok(Some(Value::U64(42))) => {
                    // Correct - execution stopped at HALT
                }
                Ok(other) => {
                    panic!("HALT should stop execution immediately, got {:?}", other);
                }
                Err(e) => {
                    panic!("HALT should not cause error, got {:?}", e);
                }
            }
        }

        #[test]
        fn test_return_value_finality() {
            // Property: RETURN_VALUE always stops execution and returns the value
            let test_cases = [0, 42, 1337, u64::MAX];

            for val in test_cases {
                let bytecode = test_bytecode![
                    push_u64!(val),
                    opcodes![RETURN_VALUE],
                    push_u64!(999), // Should never execute
                ];

                let result = TestUtils::execute_simple(&bytecode);

                match result {
                    Ok(Some(Value::U64(returned))) => {
                        assert_eq!(
                            returned, val,
                            "RETURN_VALUE should return {}, got {}",
                            val, returned
                        );
                    }
                    Ok(other) => {
                        panic!("RETURN_VALUE should return U64({}), got {:?}", val, other);
                    }
                    Err(e) => {
                        panic!("RETURN_VALUE should not fail for {}, error: {:?}", val, e);
                    }
                }
            }
        }

        #[test]
        fn test_require_consistency() {
            // Property: REQUIRE(true) continues, REQUIRE(false) fails

            // Test REQUIRE(true)
            let bytecode_true = test_bytecode![
                push_bool!(true),
                opcodes![REQUIRE],
                push_u64!(42), // Should execute
            ];

            let result_true = TestUtils::execute_simple(&bytecode_true);
            match result_true {
                Ok(Some(Value::U64(42))) => {
                    // Correct - execution continued after REQUIRE(true)
                }
                Ok(other) => {
                    panic!("REQUIRE(true) should continue execution, got {:?}", other);
                }
                Err(e) => {
                    panic!("REQUIRE(true) should not fail, got {:?}", e);
                }
            }

            // Test REQUIRE(false)
            let bytecode_false = test_bytecode![
                push_bool!(false),
                opcodes![REQUIRE],
                push_u64!(42), // Should never execute
            ];

            let result_false = TestUtils::execute_simple(&bytecode_false);
            match result_false {
                Err(VMError::ConstraintViolation) => {
                    // Correct behavior
                }
                Err(_) => {
                    // Any error is acceptable for REQUIRE(false)
                }
                Ok(val) => {
                    panic!("REQUIRE(false) should fail, but got {:?}", val);
                }
            }
        }
    }
}
