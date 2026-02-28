//! Onchain test demonstrating MitoVM execution
//!
//! This example loads the compiled onchain_with_mito.bin and executes it
//! using MitoVM to demonstrate zero-allocation execution.

use five_vm_mito::{stack::StackStorage, MitoVM, FIVE_VM_PROGRAM_ID};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing MitoVM with onchain bytecode...");

    // Load the compiled bytecode
    let bytecode_path = "../five_scripts/onchain_with_mito.bin";
    let bytecode = match fs::read(bytecode_path) {
        Ok(data) => data,
        Err(_) => {
            println!(
                "⚠️  Could not find {}, trying current directory...",
                bytecode_path
            );
            fs::read("onchain_with_mito.bin")?
        }
    };

    println!("✅ Loaded bytecode: {} bytes", bytecode.len());

    // Show the structure
    println!(
        "📝 Magic bytes: {:?}",
        std::str::from_utf8(&bytecode[0..4]).unwrap_or("Invalid")
    );

    // Validate structure
    if &bytecode[0..4] != b"STKS" {
        return Err("Invalid FIVE bytecode format".into());
    }

    println!("✨ Valid FIVE bytecode detected!");

    // Execute with MitoVM (no accounts needed for this simple test)
    println!("🚀 Executing with MitoVM...");

    let mut storage = StackStorage::new();
    match MitoVM::execute_direct(&bytecode, &[], &[], &FIVE_VM_PROGRAM_ID, &mut storage) {
        Ok(result) => {
            println!("✅ Execution successful!");
            match result {
                Some(value) => println!("📤 Result: {:?}", value),
                None => println!("📤 No return value"),
            }
        }
        Err(e) => {
            println!("❌ Execution failed: {:?}", e);
            return Err(format!("MitoVM execution error: {:?}", e).into());
        }
    }

    println!("🎯 MitoVM onchain test completed successfully!");
    Ok(())
}
