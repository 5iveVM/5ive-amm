//! Simple MitoVM test with manually crafted bytecode
//!
//! This demonstrates MitoVM execution with valid STKX bytecode

use five_protocol::encoding::VLE;
use five_vm_mito::{enhanced_opcodes::*, FIVE_VM_PROGRAM_ID, MitoVM, stack::StackStorage};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing MitoVM with simple bytecode...");

    // Create simple bytecode: 5IVE header + PUSH 42 + HALT
    let mut bytecode = Vec::new();

    // Magic bytes (5IVE)
    bytecode.extend_from_slice(b"5IVE");

    // Optimized header: features byte + function count
    bytecode.push(0x00); // features (no special features)
    bytecode.push(0x00); // function_count (no functions)

    // PUSH 42 (U32) - using VLE encoding
    bytecode.push(PUSH_U32);
    let (size, vle_bytes) = VLE::encode_u32(42);
    bytecode.extend_from_slice(&vle_bytes[..size]); // Value: 42 in VLE encoding

    // HALT
    bytecode.push(HALT);

    println!("✅ Created bytecode: {} bytes", bytecode.len());
    println!(
        "📝 Magic bytes: {:?}",
        std::str::from_utf8(&bytecode[0..4]).unwrap()
    );

    // Execute with MitoVM
    println!("🚀 Executing with MitoVM...");

    let mut storage = StackStorage::new(&bytecode);
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

    println!("🎯 Simple MitoVM test completed successfully!");
    Ok(())
}
