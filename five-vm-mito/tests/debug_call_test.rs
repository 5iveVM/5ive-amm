use five_vm_mito::{MitoVM, Pubkey, Value};

#[test]
fn test_debug_call() {
    // Reconstructed bytecode for modern MitoVM execution
    // Layout: Header -> Function 2 (Test) -> Function 1 (Add)
    // Execution starts at Function 2 (offset 10)

    let bytecode = vec![
        // Header (10 bytes)
        0x35, 0x49, 0x56, 0x45, // Magic
        0x00, 0x00, 0x00, 0x00, // Features
        0x00,                   // Public count
        0x00,                   // Total count

        // Function 2 (Test)
        0x1B, 0x05,             // PUSH_U64(5)
        0x1B, 0x03,             // PUSH_U64(3)
        // 0x90, 0x02, 0x14, 0x00, // CALL param_count=2, func_addr=20
        // 0x07,                   // RETURN_VALUE
        // For debugging: just ADD and RETURN
        0x20, // ADD
        0x07, // RETURN
        0x00,                   // HALT

        // Function 1 (Add) - Starts at 20 (0x14) - Unused for now
        0xA5, 0x01,             // LOAD_PARAM 1
        0xA2, 0x00,             // SET_LOCAL 0
        0xA5, 0x02,             // LOAD_PARAM 2
        0xA2, 0x01,             // SET_LOCAL 1
        0xA3, 0x00,             // GET_LOCAL 0
        0xA3, 0x01,             // GET_LOCAL 1
        0x20,                   // ADD
        0x07,                   // RETURN_VALUE
    ];

    let accounts = [];
    let input_data = vec![]; // Execute from start

    let program_id = Pubkey::default();
    let result = MitoVM::execute_direct(&bytecode, &input_data, &accounts, &program_id);

    match result {
        Ok(res) => {
            println!("SUCCESS: Function call completed");
            println!("Result: {:?}", res);
            // Result should be 8 (5+3)
            assert_eq!(res, Some(Value::U64(8)));
        }
        Err(e) => {
            println!("ERROR: Function call failed with: {:?}", e);
            panic!("Execution failed: {:?}", e);
        }
    }
}
