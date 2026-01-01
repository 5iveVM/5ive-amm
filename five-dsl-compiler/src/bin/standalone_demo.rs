//! Standalone Function Name Embedding Demo
//!
//! This demo shows the complete implementation without VM dependencies.

use std::collections::HashMap;

// Minimal opcodes for demo
const CALL: u8 = 0x20;
const PUSH_U64: u8 = 0x21;
const RETURN: u8 = 0x22;

/// Function call information extracted from bytecode
#[derive(Debug, Clone)]
pub struct CallInfo {
    pub position: usize,
    pub param_count: u8,
    pub function_address: u16,
    pub function_name: Option<String>,
}

/// Simple bytecode parser for demonstration
pub struct SimpleBytecodeParser;

impl SimpleBytecodeParser {
    /// Parse bytecode and extract function call metadata
    pub fn parse_function_calls(bytecode: &[u8]) -> Result<Vec<CallInfo>, String> {
        let mut calls = Vec::new();
        let mut name_table = Vec::new();
        let mut position = 0;

        while position < bytecode.len() {
            let opcode = bytecode[position];
            position += 1;

            match opcode {
                CALL => {
                    if position + 2 >= bytecode.len() {
                        return Err("Incomplete CALL instruction".to_string());
                    }

                    let call_start = position - 1;

                    // Read required parameters (what VM reads)
                    let param_count = bytecode[position];
                    position += 1;

                    let function_address =
                        u16::from_le_bytes([bytecode[position], bytecode[position + 1]]);
                    position += 2;

                    // Check for optional function name metadata (what VM ignores)
                    let function_name = if position < bytecode.len() {
                        let name_len = bytecode[position];
                        position += 1;

                        if name_len == 0xFF {
                            // Name reference - read index
                            if position >= bytecode.len() {
                                return Err("Incomplete name reference in CALL".to_string());
                            }
                            let name_index = bytecode[position] as usize;
                            position += 1;
                            name_table.get(name_index).cloned()
                        } else {
                            // Inline name - read string
                            if position + name_len as usize > bytecode.len() {
                                return Err("Incomplete function name in CALL".to_string());
                            }

                            let name_bytes = &bytecode[position..position + name_len as usize];
                            position += name_len as usize;

                            let name = String::from_utf8(name_bytes.to_vec())
                                .map_err(|_| "Invalid UTF-8 in function name")?;

                            name_table.push(name.clone());
                            Some(name)
                        }
                    } else {
                        None
                    };

                    calls.push(CallInfo {
                        position: call_start,
                        param_count,
                        function_address,
                        function_name,
                    });
                }
                PUSH_U64 => position += 8,
                RETURN => {} // No operands
                _ => {}      // Unknown opcode, skip
            }
        }

        Ok(calls)
    }
}

/// Create example bytecode with embedded function names
fn create_example_bytecode() -> Vec<u8> {
    let mut bytecode = Vec::new();

    println!("🔧 Generating Example Bytecode...\n");

    // Function 1: transfer_tokens (first occurrence - full name)
    println!("Adding CALL #1: transfer_tokens(from, to, amount) -> 3 params, addr 0x0100");
    bytecode.extend_from_slice(&[
        CALL, 3, // param_count
        0x00, 0x01, // function_address (0x0100) little-endian
        15,   // name_len
        b't', b'r', b'a', b'n', b's', b'f', b'e', b'r', b'_', b't', b'o', b'k', b'e', b'n', b's',
    ]);

    // Function 2: get_balance (first occurrence - full name)
    println!("Adding CALL #2: get_balance(account) -> 1 param, addr 0x0200");
    bytecode.extend_from_slice(&[
        CALL, 1, // param_count
        0x00, 0x02, // function_address (0x0200)
        11,   // name_len
        b'g', b'e', b't', b'_', b'b', b'a', b'l', b'a', b'n', b'c', b'e',
    ]);

    // Function 3: transfer_tokens again (deduplication - name reference)
    println!("Adding CALL #3: transfer_tokens (repeated) -> uses reference to call #1");
    bytecode.extend_from_slice(&[
        CALL, 3, // param_count
        0x00, 0x03, // different function_address (0x0300)
        0xFF, // name reference marker
        0,    // index to first occurrence (transfer_tokens)
    ]);

    // Function 4: approve_spending (new function - full name)
    println!("Adding CALL #4: approve_spending(spender, amount) -> 2 params, addr 0x0400");
    bytecode.extend_from_slice(&[
        CALL, 2, // param_count
        0x00, 0x04, // function_address (0x0400)
        16,   // name_len
        b'a', b'p', b'p', b'r', b'o', b'v', b'e', b'_', b's', b'p', b'e', b'n', b'd', b'i', b'n',
        b'g',
    ]);

    // Function 5: get_balance again (deduplication)
    println!("Adding CALL #5: get_balance (repeated) -> uses reference to call #2");
    bytecode.extend_from_slice(&[
        CALL, 1, // param_count
        0x00, 0x05, // different function_address (0x0500)
        0xFF, // name reference marker
        1,    // index to second occurrence (get_balance)
    ]);

    // Add some other instructions
    bytecode.extend_from_slice(&[
        PUSH_U64, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, // value: 1
        RETURN,
    ]);

    println!("✅ Generated {} bytes of bytecode\n", bytecode.len());
    bytecode
}

fn main() {
    println!("🚀 Five Protocol Function Name Embedding Demo");
    println!("============================================\n");

    // Generate example bytecode
    let bytecode = create_example_bytecode();

    println!("📊 Bytecode Analysis:");
    println!("Total size: {} bytes", bytecode.len());

    // Calculate size breakdown
    let mut vm_bytes = 0;
    let mut metadata_bytes = 0;
    let mut pos = 0;

    while pos < bytecode.len() {
        match bytecode[pos] {
            CALL => {
                vm_bytes += 4; // CALL + param_count + addr (what VM reads)
                pos += 4;

                if pos < bytecode.len() {
                    let name_len = bytecode[pos];
                    if name_len == 0xFF {
                        metadata_bytes += 2; // marker + index (what VM ignores)
                        pos += 2;
                    } else {
                        metadata_bytes += 1 + name_len as usize; // len + name (what VM ignores)
                        pos += 1 + name_len as usize;
                    }
                }
            }
            PUSH_U64 => {
                vm_bytes += 9;
                pos += 9;
            }
            RETURN => {
                vm_bytes += 1;
                pos += 1;
            }
            _ => pos += 1,
        }
    }

    println!(
        "  • VM execution bytes: {} ({:.1}%)",
        vm_bytes,
        (vm_bytes as f32 / bytecode.len() as f32) * 100.0
    );
    println!(
        "  • Metadata bytes: {} ({:.1}%)",
        metadata_bytes,
        (metadata_bytes as f32 / bytecode.len() as f32) * 100.0
    );
    println!(
        "  • Zero overhead: VM ignores {} bytes completely! 🎯\n",
        metadata_bytes
    );

    // Parse the bytecode
    match SimpleBytecodeParser::parse_function_calls(&bytecode) {
        Ok(calls) => {
            println!("🔍 Extracted Function Calls:");
            for (i, call) in calls.iter().enumerate() {
                println!(
                    "  {}. Position 0x{:04X}: {} ({} params, addr 0x{:04X})",
                    i + 1,
                    call.position,
                    call.function_name.as_deref().unwrap_or("unnamed"),
                    call.param_count,
                    call.function_address
                );
            }

            // Show ecosystem benefits
            println!("\n🌐 Ecosystem Composability:");
            let mut interfaces: HashMap<String, Vec<u16>> = HashMap::new();
            for call in &calls {
                if let Some(name) = &call.function_name {
                    interfaces
                        .entry(name.clone())
                        .or_default()
                        .push(call.function_address);
                }
            }

            println!("Available functions for import:");
            for (name, addresses) in interfaces {
                println!("  • {} (found at {} locations)", name, addresses.len());
            }

            println!("\n💡 Usage in Five DSL:");
            println!("use ContractAddress::*; // Auto-discovers all functions above");
            println!("use ContractAddress::{{transfer_tokens, get_balance}}; // Selective import");

            println!("\n✅ Implementation Benefits:");
            println!("  🎯 Zero VM overhead - metadata completely ignored during execution");
            println!(
                "  🧠 Smart deduplication - {} space savings for repeated names",
                metadata_bytes.saturating_sub(
                    calls
                        .iter()
                        .map(|c| c.function_name.as_ref().map_or(0, |n| n.len() + 1))
                        .sum::<usize>()
                )
            );
            println!("  🔍 Rich tooling - function names available for IDEs and debuggers");
            println!("  🌐 Ecosystem ready - automatic function discovery for imports");
        }
        Err(e) => {
            println!("❌ Failed to parse bytecode: {}", e);
        }
    }
}
