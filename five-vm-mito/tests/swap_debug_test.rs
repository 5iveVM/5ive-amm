use five_vm_mito::{MitoVM, Value, Pubkey};

#[test]
fn test_swap_debug() {
    let program_id = Pubkey::default();
    // Test SWAP operation with detailed debug
    // We want to compute 10 - 3 = 7 using SWAP.
    // Stack: [bottom, ..., top]
    // 1. Push 3: [3]
    // 2. Push 10: [3, 10]
    // 3. SWAP: [10, 3]
    // 4. SUB: pops 3 (right), pops 10 (left). 10 - 3 = 7.

    let bytecode = vec![
        0x35, 0x49, 0x56, 0x45, // 5IVE magic
        0x00, 0x00, 0x00, 0x00, // Features
        0x00, 0x00,             // Counts

        0x1B, 0x03,             // PUSH_U64(3)
        0x1B, 0x0A,             // PUSH_U64(10)
        0x13,                   // SWAP
        0x21,                   // SUB
        0x00                    // HALT
    ];

    let result = MitoVM::execute_direct(&bytecode, &[], &[], &program_id).unwrap();
    println!("Result: {:?}", result);
    assert_eq!(result, Some(Value::U64(7)), "Expected 10 - 3 = 7 after swap");

    // Test without SWAP: 10 - 3 = 7 directly
    // Push 10, Push 3 -> [10, 3]. SUB -> 10 - 3 = 7.
    let bytecode_no_swap = vec![
        0x35, 0x49, 0x56, 0x45, // 5IVE magic
        0x00, 0x00, 0x00, 0x00, // Features
        0x00, 0x00,             // Counts

        0x1B, 0x0A,             // PUSH_U64(10)
        0x1B, 0x03,             // PUSH_U64(3)
        0x21,                   // SUB
        0x00                    // HALT
    ];

    let result_no_swap = MitoVM::execute_direct(&bytecode_no_swap, &[], &[], &program_id).unwrap();
    println!("Result: {:?}", result_no_swap);
    assert_eq!(result_no_swap, Some(Value::U64(7)), "Expected 10 - 3 = 7 (direct)");
}
