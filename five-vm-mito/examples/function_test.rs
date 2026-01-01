//! Function call test for MitoVM
//!
//! This demonstrates MitoVM's basic arithmetic capabilities

use five_vm_mito::{MitoVM, Value};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing MitoVM arithmetic...");

    // Create simple test bytecode: 21 * 2 = 42
    let bytecode = vec![
        0x35, 0x49, 0x56, 0x45, // 5IVE magic
        0x1C, 0x15, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // PUSH_U64(21)
        0x1C, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // PUSH_U64(2)
        0x22, // MUL (21 * 2)
        0x00, // HALT (should result in 42)
    ];

    println!("✅ Created arithmetic bytecode: {} bytes", bytecode.len());

    // Execute with MitoVM
    println!("🚀 Executing arithmetic with MitoVM...");

    match MitoVM::execute_direct(&bytecode, &[], &[]) {
        Ok(result) => {
            println!("✅ Execution successful!");
            match result {
                Some(Value::U64(val)) => {
                    println!("📤 Result: {}", val);
                    if val == 42 {
                        println!("🎯 Arithmetic worked correctly: 21 * 2 = {}", val);
                    } else {
                        println!("⚠️  Unexpected result: expected 42, got {}", val);
                    }
                }
                Some(other) => {
                    println!("📤 Unexpected value type: {:?}", other);
                }
                None => println!("📤 No return value"),
            }
        }
        Err(e) => {
            println!("❌ Execution failed: {:?}", e);
        }
    }

    Ok(())
}
