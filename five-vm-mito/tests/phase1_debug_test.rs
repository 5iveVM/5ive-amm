use five_vm_mito::{FIVE_VM_PROGRAM_ID, MitoVM};
use pinocchio::account_info::AccountInfo;

#[test]
fn test_phase1_return_value_debugging() {
    println!("===============================");
    println!("Phase 1 RETURN_VALUE debugging test");
    println!("===============================");

    // Bytecode from our test (base64 decoded)
    let bytecode = vec![
        0x35, 0x49, 0x56, 0x45, // Magic "5IVE"
        0x02, 0x10, 0x00, 0x00, 0x00, 0x01, 0x00, // Header
        0x0f, 0x00, 0x01, 0x00, // More header
        0x19, 0x2a, // PUSH_U8 42
        0xd4, // SET_LOCAL_0
        0xd0, // GET_LOCAL_0
        0x07, // RETURN_VALUE (position 19)
        0x00, // Padding
    ];

    println!("Bytecode length: {}", bytecode.len());
    println!("RETURN_VALUE opcode at position 19: 0x{:02X}", bytecode[19]);
    println!("Expected execution sequence:");
    println!("  Position 15: PUSH_U8 (0x19)");
    println!("  Position 16: Value 42 (0x2A)");
    println!("  Position 17: SET_LOCAL_0 (0xD4)");
    println!("  Position 18: GET_LOCAL_0 (0xD0)");
    println!("  Position 19: RETURN_VALUE (0x07)");
    println!();

    // Input data (function index 0, no parameters)
    let input_data = vec![0, 0];

    // No accounts needed for this test
    let accounts: &[AccountInfo] = &[];

    println!("Executing with MitoVM...");
    println!("========================================");

    match MitoVM::execute_direct(&bytecode, &input_data, accounts, &FIVE_VM_PROGRAM_ID) {
        Ok(result) => {
            println!("========================================");
            println!("Execution completed successfully!");
            println!("Result: {:?}", result);

            // We expect to get Some(Value::U64(42)) if RETURN_VALUE works correctly
            match result {
                Some(value) => {
                    println!("SUCCESS: Got return value: {:?}", value);
                    println!("✅ RETURN_VALUE opcode is working correctly!");
                }
                None => {
                    println!("❌ ISSUE: No return value captured");
                    println!(
                        "This suggests RETURN_VALUE opcode is NOT executing or NOT halting VM"
                    );
                    println!("Phase 1 debugging should show us exactly where the issue is");
                }
            }
        }
        Err(e) => {
            println!("========================================");
            println!("❌ Execution failed!");
            println!("Error: {:?}", e);
        }
    }

    println!("===============================");
    println!("Phase 1 test completed");
    println!("===============================");
}
