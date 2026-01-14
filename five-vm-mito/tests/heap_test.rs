use five_vm_mito::{FIVE_VM_PROGRAM_ID, MitoVM, Value};

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

#[test]
fn test_heap_string_allocation() {
    // Create a string of length 70 (exceeds 64 byte temp buffer limit logic in handlers/arrays.rs)
    let mut bytecode_body = vec![
        0x66, 70, // PUSH_STRING_LITERAL length 70
    ];
    // Add 70 bytes of 'A'
    bytecode_body.extend(std::iter::repeat(b'A').take(70));
    bytecode_body.push(0x00); // HALT

    let bytecode = build_bytecode(&bytecode_body);

    let result = MitoVM::execute_direct(&bytecode, &[], &[], &FIVE_VM_PROGRAM_ID);

    match result {
        Ok(value) => {
             println!("Success: {:?}", value);
             if let Some(Value::String(_)) = value {
                 // OK
             } else {
                 panic!("Expected String value");
             }
        }
        Err(e) => {
             println!("Failed: {:?}", e);
             panic!("Should have succeeded with heap allocation");
        }
    }
}
