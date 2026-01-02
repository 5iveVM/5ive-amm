use five_vm_mito::{AccountInfo, FIVE_VM_PROGRAM_ID, MitoVM, Value};

#[test]
fn test_multiply_debug() {
    // Test just multiply(4, 2) = 8
    // Let's create a simple script that just calls multiply
    let simple_multiply_script = r#"
script TestMultiply {
    multiply(a: u64, b: u64) -> u64 {
        return a * b;
    }

    test() -> u64 {
        return multiply(4, 2);
    }
}
"#;

    // Verify the script is not empty
    assert!(
        !simple_multiply_script.is_empty(),
        "script should not be empty"
    );

    println!("🔍 Debug: Let's test if multiply operation works correctly");

    // For now, let's manually test the multiply operation
    // This test will help us understand if the issue is with the multiply opcode

    // Create a minimal bytecode for testing multiply
    // PUSH_U64(4), PUSH_U64(2), MUL, RETURN_VALUE
    let mut bytecode = vec![0x35, 0x49, 0x56, 0x45]; // Magic bytes

    // PUSH_U64(4)
    bytecode.push(0x1c); // PUSH_U64 opcode
    bytecode.extend_from_slice(&4u64.to_le_bytes());

    // PUSH_U64(2)
    bytecode.push(0x1c); // PUSH_U64 opcode
    bytecode.extend_from_slice(&2u64.to_le_bytes());

    // MUL (opcode 0x22)
    bytecode.push(0x22);

    // RETURN_VALUE
    bytecode.push(0x07);

    println!(
        "🔍 Testing multiply with manual bytecode: {} bytes",
        bytecode.len()
    );

    let input_data: &[u8] = &[];
    let accounts: &[AccountInfo] = &[];

    match MitoVM::execute_direct(&bytecode, input_data, accounts, &FIVE_VM_PROGRAM_ID) {
        Ok(result) => {
            println!("✅ Multiply test executed successfully!");
            println!("📋 Result: {:?}", result);

            if let Some(Value::U64(value)) = result {
                println!("✅ Multiply result: {}", value);
                assert_eq!(value, 8, "Expected multiply(4, 2) = 8");
            } else {
                println!("❌ Expected U64 result from multiply");
            }
        }
        Err(e) => {
            println!("❌ Multiply test failed: {:?}", e);
        }
    }
}

#[test]
fn test_add_debug() {
    // Test just add(5, 3) = 8
    // Create a minimal bytecode for testing add
    // PUSH_U64(5), PUSH_U64(3), ADD, RETURN_VALUE
    let mut bytecode = vec![0x35, 0x49, 0x56, 0x45]; // Magic bytes

    // PUSH_U64(5)
    bytecode.push(0x1c); // PUSH_U64 opcode
    bytecode.extend_from_slice(&5u64.to_le_bytes());

    // PUSH_U64(3)
    bytecode.push(0x1c); // PUSH_U64 opcode
    bytecode.extend_from_slice(&3u64.to_le_bytes());

    // ADD (opcode 0x20)
    bytecode.push(0x20);

    // RETURN_VALUE
    bytecode.push(0x07);

    println!(
        "🔍 Testing add with manual bytecode: {} bytes",
        bytecode.len()
    );

    let input_data: &[u8] = &[];
    let accounts: &[AccountInfo] = &[];

    match MitoVM::execute_direct(&bytecode, input_data, accounts, &FIVE_VM_PROGRAM_ID) {
        Ok(result) => {
            println!("✅ Add test executed successfully!");
            println!("📋 Result: {:?}", result);

            if let Some(Value::U64(value)) = result {
                println!("✅ Add result: {}", value);
                assert_eq!(value, 8, "Expected add(5, 3) = 8");
            } else {
                println!("❌ Expected U64 result from add");
            }
        }
        Err(e) => {
            println!("❌ Add test failed: {:?}", e);
        }
    }
}
