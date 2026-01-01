use five_vm_mito::{MitoVM, Value};

fn main() {
    // Test SWAP operation with detailed debug
    let bytecode = vec![
        0x35, 0x49, 0x56, 0x45, // 5IVE magic
        0x1B, 0x0A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // PUSH_U64(10)
        0x1B, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // PUSH_U64(3)
        0x13, // SWAP
        0x21, // SUB (should be 10 - 3 = 7)
        0x00  // HALT
    ];
    
    println!("Testing SWAP operation:");
    println!("1. PUSH_U64(10)");
    println!("2. PUSH_U64(3)");
    println!("3. SWAP (stack should become: [10, 3] -> [3, 10])");
    println!("4. SUB (should compute 10 - 3 = 7)");
    
    let result = MitoVM::execute_direct(&bytecode, &[], &[]).unwrap();
    println!("Result: {:?}", result);
    
    // Let's also test without SWAP to see what happens
    let bytecode_no_swap = vec![
        0x35, 0x49, 0x56, 0x45, // 5IVE magic
        0x1B, 0x0A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // PUSH_U64(10)
        0x1B, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // PUSH_U64(3)
        0x21, // SUB (should be 3 - 10 = saturating to 0)
        0x00  // HALT
    ];
    
    println!("\nTesting without SWAP (3 - 10):");
    let result_no_swap = MitoVM::execute_direct(&bytecode_no_swap, &[], &[]).unwrap();
    println!("Result: {:?}", result_no_swap);
}