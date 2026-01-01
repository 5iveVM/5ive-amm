// Test MitoVM with simple arithmetic operations
use five_vm_mito::{MitoVM, Value};

fn main() {
    println!("Testing MitoVM with fixed values...");

    // Test simple addition: 100 + 200 = 300
    let bytecode = vec![
        0x35, 0x49, 0x56, 0x45, // 5IVE magic
        0x1C, 0x64, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // PUSH_U64(100)
        0x1C, 0xC8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // PUSH_U64(200)
        0x20, // ADD
        0x00, // HALT
    ];

    match MitoVM::execute_direct(&bytecode, &[], &[]) {
        Ok(Some(Value::U64(result))) => {
            println!("✅ Success! 100 + 200 = {}", result);
            assert_eq!(result, 300, "Addition should work correctly");
        }
        Ok(other) => {
            println!("❌ Unexpected result: {:?}", other);
        }
        Err(e) => {
            println!("❌ Execution failed: {:?}", e);
        }
    }

    // Test boolean operations: true AND false = false
    let bytecode2 = vec![
        0x35, 0x49, 0x56, 0x45, // 5IVE magic
        0x1E, 0x01, // PUSH_BOOL(true)
        0x1E, 0x00, // PUSH_BOOL(false)
        0x2B, // AND
        0x00, // HALT
    ];

    match MitoVM::execute_direct(&bytecode2, &[], &[]) {
        Ok(Some(Value::Bool(result))) => {
            println!("✅ Success! true AND false = {}", result);
            assert!(!result, "Boolean AND should work correctly");
        }
        Ok(other) => {
            println!("❌ Unexpected result: {:?}", other);
        }
        Err(e) => {
            println!("❌ Execution failed: {:?}", e);
        }
    }

    println!("All tests completed!");
}
