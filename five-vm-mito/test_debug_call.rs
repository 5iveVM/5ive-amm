//! Debug test for CALL TypeMismatch error
//! This test reproduces the exact issue described in the problem statement

use five_vm_mito::MitoVM;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Bytecode equivalent to:
    // script DebugSimpleCall {
    //     add_numbers(a: u64, b: u64) -> u64 {
    //         return a + b;  // LOAD_PARAM 1, SET_LOCAL 0, LOAD_PARAM 2, SET_LOCAL 1, GET_LOCAL 0, GET_LOCAL 1, ADD, RETURN_VALUE
    //     }
    //     
    //     test() -> u64 {
    //         return add_numbers(5, 3);  // PUSH_U64 5, PUSH_U64 3, CALL 2 4 0, RETURN_VALUE
    //     }
    // }
    
    let bytecode = vec![
        // Magic bytes (5IVE - FIVE_DEPLOY_MAGIC for function dispatch)
        0x35, 0x49, 0x56, 0x45, // "5IVE" deploy magic
        
        // Function 1 (add_numbers): at offset 4
        0x94, 0x01,             // LOAD_PARAM 1 (first parameter a: u64)
        0x91, 0x00,             // SET_LOCAL 0 (store a in local[0])
        0x94, 0x02,             // LOAD_PARAM 2 (second parameter b: u64)
        0x91, 0x01,             // SET_LOCAL 1 (store b in local[1])
        0x92, 0x00,             // GET_LOCAL 0 (get a)
        0x92, 0x01,             // GET_LOCAL 1 (get b)
        0x20,                   // ADD (a + b)
        0x08,                   // RETURN_VALUE
        
        // Function 2 (test): at offset 18
        0x16, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // PUSH_U64(5)
        0x16, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // PUSH_U64(3)
        0x80, 0x02, 0x04, 0x00, // CALL param_count=2, func_addr=4 (function 1)
        0x08,                   // RETURN_VALUE
        
        0x00                    // HALT
    ];
    
    println!("Bytecode analysis:");
    println!("Length: {} bytes", bytecode.len());
    println!("Magic: {:02x} {:02x} {:02x} {:02x}", bytecode[0], bytecode[1], bytecode[2], bytecode[3]);
    println!("Function 1 starts at offset 4");
    println!("Function 2 starts at offset 18");
    println!("CALL instruction: {:02x} {:02x} {:02x} {:02x}", 
        bytecode[27], bytecode[28], bytecode[29], bytecode[30]);
    
    let accounts = [];
    let input_data = vec![0x02]; // Function index 2 (test function)
    
    println!("\nExecuting Five VM with function call...");
    match MitoVM::execute_direct(&bytecode, &input_data, &accounts) {
        Ok(result) => {
            println!("SUCCESS: Function call completed");
            println!("Result: {:?}", result);
        }
        Err(e) => {
            println!("ERROR: Function call failed with: {:?}", e);
            if format!("{:?}", e).contains("TypeMismatch") {
                println!("\n=== TYPEMISMATCH ERROR CONFIRMED ===");
                println!("This is the exact error we need to debug!");
            }
        }
    }
    
    Ok(())
}