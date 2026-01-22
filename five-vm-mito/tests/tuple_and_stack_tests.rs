//! Tuple and Stack Operations Tests for Five VM
//!
//! Tests advanced operations including tuple operations and stack management.
//!
//! Coverage:
//! - CREATE_TUPLE (0xF8) - Create tuple from stack values
//! - TUPLE_GET (0xF9) - Get tuple element
//! - UNPACK_TUPLE (0xFA) - Unpack tuple to stack

use five_protocol::{encoding::VLE, opcodes::*, FIVE_HEADER_OPTIMIZED_SIZE, FIVE_MAGIC};
use five_vm_mito::{FIVE_VM_PROGRAM_ID, MitoVM, Result as VmResult, Value, stack::StackStorage};

fn build_script(build: impl FnOnce(&mut Vec<u8>)) -> Vec<u8> {
    let mut script = Vec::with_capacity(FIVE_HEADER_OPTIMIZED_SIZE + 64);
    script.extend_from_slice(&FIVE_MAGIC);
    // Header V3: features(4 bytes LE) + public_function_count(1) + total_function_count(1)
    script.push(0); // features byte 0
    script.push(0); // features byte 1
    script.push(0); // features byte 2
    script.push(0); // features byte 3
    script.push(1); // public entry functions
    script.push(1); // total functions
    build(&mut script);
    script
}

fn execute_script(build: impl FnOnce(&mut Vec<u8>)) -> VmResult<Option<Value>> {
    let script = build_script(build);
    let mut storage = StackStorage::new(&script);
    MitoVM::execute_direct(&script, &[], &[], &FIVE_VM_PROGRAM_ID, &mut storage)
}

fn push_u64(script: &mut Vec<u8>, value: u64) {
    script.push(PUSH_U64);
    let (len, encoded) = VLE::encode_u64(value);
    script.extend_from_slice(&encoded[..len]);
}

fn push_u8(script: &mut Vec<u8>, value: u8) {
    script.push(PUSH_U8);
    script.push(value);
}

#[cfg(test)]
mod tuple_operations_tests {
    use super::*;

    #[test]
    fn test_create_tuple_basic() {
        let result = execute_script(|script| {
            push_u64(script, 10);
            push_u64(script, 20);
            // CREATE_TUPLE takes immediate byte for size
            script.push(CREATE_TUPLE);
            script.push(2); // Size 2
            script.push(HALT);
        });

        match result {
            Ok(Some(Value::Array(_))) => {
                 // Success - we got an array/tuple ref
            },
            Ok(val) => panic!("Expected Array, got {:?}", val),
            Err(e) => panic!("CREATE_TUPLE failed: {:?}", e),
        }
    }

    #[test]
    fn test_tuple_get_element() {
        let result = execute_script(|script| {
            push_u64(script, 100);
            push_u64(script, 200);
            push_u64(script, 300);
            script.push(CREATE_TUPLE);
            script.push(3); // Size 3. Tuple is [100, 200, 300] (based on stack order usually popped? or indexed? Let's assume order preserved or reversed. Array usually stores [0]=bottom?)
            // handlers/advanced.rs says:
            // for i in 0..element_count { let idx = ctx.stack.sp - 1 - i; ... }
            // "Serialize elements directly in reverse order"
            // "ctx.pop()?" in loop.
            // If stack is [100, 200, 300(top)], popping gives 300, 200, 100.
            // If it serializes in reverse order of pops?
            // "write_offset -= size; element.serialize_into..."
            // It fills from end of buffer backwards.
            // Pop 300. Puts at end.
            // Pop 200. Puts before 300.
            // Pop 100. Puts before 200.
            // So Array should be [100, 200, 300]. Index 0 is 100. Index 1 is 200.

            push_u8(script, 1); // Index 1
            script.push(TUPLE_GET);
            script.push(HALT);
        });

        match result {
            Ok(Some(Value::U64(element_value))) => {
                assert_eq!(element_value, 200, "Tuple element at index 1 should be 200");
            },
            Ok(val) => panic!("Expected U64(200), got {:?}", val),
            Err(e) => panic!("TUPLE_GET failed: {:?}", e),
        }
    }

    #[test]
    fn test_unpack_tuple() {
        let result = execute_script(|script| {
            push_u64(script, 50);
            push_u64(script, 75);
            script.push(CREATE_TUPLE);
            script.push(2); // Size 2. Stack: [Tuple(50, 75)]

            script.push(UNPACK_TUPLE); // Stack: [50, 75] (75 on top)
            script.push(HALT);
        });

        match result {
            Ok(Some(Value::U64(top_value))) => {
                assert_eq!(top_value, 75, "Top element after unpack should be 75");
            },
            Ok(val) => panic!("Expected U64(75), got {:?}", val),
            Err(e) => panic!("UNPACK_TUPLE failed: {:?}", e),
        }
    }

    #[test]
    fn test_complex_tuple_operations() {
        let result = execute_script(|script| {
            push_u64(script, 1);
            push_u64(script, 2);
            script.push(CREATE_TUPLE);
            script.push(2); // inner tuple (1, 2)

            push_u64(script, 3);
            // Stack: [Tuple(1,2), 3]
            // We want outer tuple (Tuple(1,2), 3)
            script.push(CREATE_TUPLE);
            script.push(2); // outer tuple

            // Stack: [OuterTuple]
            push_u8(script, 0); // index 0 is InnerTuple(1,2)
            script.push(TUPLE_GET);

            // Stack: [InnerTuple]
            push_u8(script, 1); // index 1 is 2
            script.push(TUPLE_GET);
            script.push(HALT);
        });

        match result {
            Ok(Some(Value::U64(nested_value))) => {
                assert_eq!(nested_value, 2, "Nested tuple access should return 2");
            },
            Ok(val) => panic!("Expected U64(2), got {:?}", val),
            Err(e) => panic!("Complex tuple operations failed: {:?}", e),
        }
    }
}
