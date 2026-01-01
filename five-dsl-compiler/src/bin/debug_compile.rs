use five_dsl_compiler::{DslBytecodeGenerator, DslParser, DslTokenizer, DslTypeChecker};
use five_vm_mito::error::VMError;
use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <script.v> [--debug-bytecode]", args[0]);
        process::exit(1);
    }

    // Detect optional --debug-bytecode flag
    let debug_bytecode = args.iter().any(|a| a == "--debug-bytecode");

    // Find the first non-flag argument after program name and treat it as file path
    let file_path = args
        .iter()
        .skip(1)
        .find(|a| !a.starts_with('-'))
        .map(|s| s.as_str())
        .unwrap_or_else(|| {
            eprintln!("Usage: {} <script.v> [--debug-bytecode]", args[0]);
            process::exit(1);
        });

    // Read the source file
    let source = match fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error reading file '{}': {}", file_path, err);
            process::exit(1);
        }
    };

    println!("Debug compilation of: {}", file_path);
    println!("Source code:");
    println!("{}", source);
    println!("{}", "=".repeat(60));

    // Step 1: Tokenization
    println!("\n1. TOKENIZATION");
    println!("{}", "-".repeat(20));
    let mut tokenizer = DslTokenizer::new(&source);
    let tokens = match tokenizer.tokenize() {
        Ok(tokens) => {
            println!("✓ Tokenization successful! Found {} tokens:", tokens.len());
            for (i, token) in tokens.iter().enumerate() {
                println!("  {}: {:?}", i, token);
            }
            tokens
        }
        Err(err) => {
            match err {
                VMError::InvalidScript => {
                    eprintln!("✗ Tokenization failed: Invalid script");
                }
                other => {
                    eprintln!("✗ Tokenization failed: {:?}", other);
                }
            }
            process::exit(1);
        }
    };

    // Step 2: Parsing
    println!("\n2. PARSING");
    println!("{}", "-".repeat(20));
    let mut parser = DslParser::new(tokens.clone());
    let ast = match parser.parse() {
        Ok(ast) => {
            println!("✓ Parsing successful!");
            println!("  AST: {:#?}", ast);
            ast
        }
        Err(err) => {
            eprintln!("✗ Parsing failed: {:?}", err);
            process::exit(1);
        }
    };

    // Step 3: Type Checking
    println!("\n3. TYPE CHECKING");
    println!("{}", "-".repeat(20));
    let mut type_checker = DslTypeChecker::new();
    match type_checker.check_types(&ast) {
        Ok(_) => {
            println!("✓ Type checking successful!");
            println!("  Type checking passed");
        }
        Err(error) => {
            eprintln!("✗ Type checking failed: {}", error);
            process::exit(1);
        }
    }

    // Step 4: Bytecode Generation
    println!("\n4. BYTECODE GENERATION");
    println!("{}", "-".repeat(20));
    let mut bytecode_gen = DslBytecodeGenerator::new();

    // If user requested debug bytecode capture, enable generator diagnostic capture
    if debug_bytecode {
        bytecode_gen.set_debug_on_error(true);
    }

    let bytecode = match bytecode_gen.generate(&ast) {
        Ok(bytecode) => {
            println!("✓ Bytecode generation successful!");
            println!("  Bytecode length: {} bytes", bytecode.len());
            println!("  Bytecode (hex): {}", hex::encode(&bytecode));

            // Disassemble bytecode for debugging
            println!("\n  Disassembly:");
            disassemble_bytecode(&bytecode);

            // If debug-bytecode was enabled, print captured compilation log from generator
            if debug_bytecode {
                let logs = bytecode_gen.get_compilation_log();
                if !logs.is_empty() {
                    println!("\n  Captured bytecode diagnostics:");
                    for line in logs.iter() {
                        println!("    {}", line);
                    }
                }
            }

            bytecode
        }
        Err(err) => {
            eprintln!("✗ Bytecode generation failed: {:?}", err);
            process::exit(1);
        }
    };

    // Step 5: ABI Generation
    println!("\n5. ABI GENERATION");
    println!("{}", "-".repeat(20));
    let abi = match bytecode_gen.generate_abi(&ast) {
        Ok(abi) => {
            println!("✓ ABI generation successful!");
            println!("  Program: {}", abi.program_name);
            println!("  Functions: {}", abi.functions.len());
            println!("  Fields: {}", abi.fields.len());

            for func in &abi.functions {
                println!(
                    "    Function {}: {} (index {})",
                    func.index, func.name, func.index
                );
                for param in &func.parameters {
                    println!(
                        "      - {}: {} (account: {})",
                        param.name, param.param_type, param.is_account
                    );
                }
            }

            abi
        }
        Err(err) => {
            eprintln!("✗ ABI generation failed: {:?}", err);
            process::exit(1);
        }
    };

    // Write binary output (.bin file)
    let bin_output = file_path.replace(".v", ".bin");
    match fs::write(&bin_output, &bytecode) {
        Ok(_) => println!("\n✓ Bytecode written to: {}", bin_output),
        Err(err) => eprintln!("Warning: Could not write bytecode file: {}", err),
    }

    // Write ABI output (.abi.json file)
    let abi_output = file_path.replace(".v", ".abi.json");
    let abi_json = match serde_json::to_string_pretty(&abi) {
        Ok(json) => json,
        Err(err) => {
            eprintln!("Warning: Could not serialize ABI to JSON: {}", err);
            "{}".to_string()
        }
    };
    match fs::write(&abi_output, &abi_json) {
        Ok(_) => println!("✓ ABI written to: {}", abi_output),
        Err(err) => eprintln!("Warning: Could not write ABI file: {}", err),
    }

    // Write debug output
    let debug_output = format!("{}.debug", file_path);
    let debug_info = format!(
        "Debug compilation report for: {}\n\
         Source length: {} characters\n\
         Tokens: {} items\n\
         Bytecode: {} bytes\n\
         Bytecode (hex): {}\n\
         ABI functions: {}\n\
         ABI fields: {}\n",
        file_path,
        source.len(),
        tokens.len(),
        bytecode.len(),
        hex::encode(&bytecode),
        abi.functions.len(),
        abi.fields.len()
    );

    match fs::write(&debug_output, debug_info) {
        Ok(_) => println!("✓ Debug info written to: {}", debug_output),
        Err(err) => eprintln!("Warning: Could not write debug file: {}", err),
    }
}

fn disassemble_bytecode(bytecode: &[u8]) {
    let mut pc = 4; // Skip 4-byte magic number
    while pc < bytecode.len() {
        let opcode = bytecode[pc];
        print!("  {:04x}: {:02x} ", pc, opcode);

        match opcode {
            0x00 => println!("HALT"),
            0x01 => {
                if pc + 2 < bytecode.len() {
                    let offset = u16::from_le_bytes([bytecode[pc + 1], bytecode[pc + 2]]);
                    println!("JMP {}", offset);
                    pc += 2;
                } else {
                    println!("JMP (incomplete)");
                }
            }
            0x02 => {
                if pc + 2 < bytecode.len() {
                    let offset = u16::from_le_bytes([bytecode[pc + 1], bytecode[pc + 2]]);
                    println!("JMP_IF {}", offset);
                    pc += 2;
                } else {
                    println!("JMP_IF (incomplete)");
                }
            }
            0x03 => {
                if pc + 2 < bytecode.len() {
                    let offset = u16::from_le_bytes([bytecode[pc + 1], bytecode[pc + 2]]);
                    println!("JMP_IF_NOT {}", offset);
                    pc += 2;
                } else {
                    println!("JMP_IF_NOT (incomplete)");
                }
            }
            0x04 => println!("REQUIRE"),
            0x05 => println!("ASSERT"),
            0x06 => println!("RETURN"),
            0x07 => println!("RETURN_VALUE"),
            0x08 => println!("NOP"),
            0x11 => println!("POP"),
            0x12 => println!("DUP"),
            0x13 => println!("CREATE_TUPLE"),
            0x14 => println!("TUPLE_GET"),
            0x15 => println!("UNPACK_TUPLE"),
            0x16 => println!("OPTIONAL_SOME"),
            0x17 => println!("SWAP"),
            0x18 => println!("PICK"),
            0x19 => println!("OPTIONAL_NONE"),
            0x1A => println!("OPTIONAL_UNWRAP"),
            0x1B => println!("PUSH_U8"),
            0x1C => {
                if pc + 8 < bytecode.len() {
                    let value = u64::from_le_bytes([
                        bytecode[pc + 1],
                        bytecode[pc + 2],
                        bytecode[pc + 3],
                        bytecode[pc + 4],
                        bytecode[pc + 5],
                        bytecode[pc + 6],
                        bytecode[pc + 7],
                        bytecode[pc + 8],
                    ]);
                    println!("PUSH_U64 {}", value);
                    pc += 8;
                } else {
                    println!("PUSH_U64 (incomplete)");
                }
            }
            0x1D => println!("PUSH_I64"),
            0x1E => println!("PUSH_BOOL"),
            0x1F => println!("PUSH_PUBKEY"),
            0x20 => println!("ADD"),
            0x21 => println!("SUB"),
            0x22 => println!("MUL"),
            0x23 => println!("DIV"),
            0x24 => println!("MOD"),
            0x25 => println!("GT"),
            0x26 => println!("LT"),
            0x27 => println!("EQ"),
            0x28 => println!("GTE"),
            0x29 => println!("LTE"),
            0x2A => println!("NEQ"),
            0x2B => println!("NEG"),
            0x30 => println!("AND"),
            0x31 => println!("OR"),
            0x32 => println!("NOT"),
            0x33 => println!("XOR"),
            0x34 => println!("BITWISE_NOT"),
            0x40 => println!("STORE"),
            0x41 => println!("LOAD"),
            0x42 => println!("STORE_FIELD"),
            0x43 => println!("LOAD_FIELD"),
            0x44 => println!("LOAD_INPUT"),
            0x45 => println!("STORE_GLOBAL"),
            0x46 => println!("LOAD_GLOBAL"),
            0x50 => println!("CREATE_ACCOUNT"),
            0x51 => println!("LOAD_ACCOUNT"),
            0x52 => println!("SAVE_ACCOUNT"),
            0x53 => println!("GET_ACCOUNT"),
            0x54 => println!("GET_LAMPORTS"),
            0x55 => println!("SET_LAMPORTS"),
            0x56 => println!("GET_DATA"),
            0x57 => println!("GET_KEY"),
            0x58 => println!("GET_OWNER"),
            0x59 => println!("TRANSFER"),
            0x5A => println!("TRANSFER_SIGNED"),
            0x60 => println!("CHECK_SIGNER"),
            0x61 => println!("CHECK_WRITABLE"),
            0x62 => println!("CHECK_OWNER"),
            0x63 => println!("CHECK_INITIALIZED"),
            0x64 => println!("CHECK_PDA"),
            0x65 => println!("CHECK_SIGNER_IMM"),
            0x66 => println!("CHECK_WRITABLE_IMM"),
            0x67 => println!("CHECK_OWNER_IMM"),
            0x68 => println!("CHECK_INITIALIZED_IMM"),
            0x69 => println!("CHECK_PDA_IMM"),
            0x6A => println!("CHECK_BATCH_IMM"),
            0x6B => println!("CHECK_UNINITIALIZED_IMM"),
            0x70 => println!("INVOKE"),
            0x71 => println!("INVOKE_SIGNED"),
            0x72 => println!("GET_CLOCK"),
            0x73 => println!("GET_RENT"),
            0x74 => println!("INIT_ACCOUNT"),
            0x75 => println!("INIT_PDA_ACCOUNT"),
            0x76 => println!("DERIVE_PDA"),
            0x77 => println!("FIND_PDA"),
            0x78 => println!("DERIVE_PDA_PARAMS"),
            0x79 => println!("FIND_PDA_PARAMS"),
            0x7A => println!("CHECK_UNINITIALIZED"),
            0x80 => {
                if pc + 2 < bytecode.len() {
                    let param_count = bytecode[pc + 1];
                    let function_address = u16::from_le_bytes([bytecode[pc + 2], bytecode[pc + 3]]);
                    println!("CALL {} {}", param_count, function_address);
                    pc += 3;
                } else {
                    println!("CALL (incomplete)");
                }
            }
            0x81 => println!("CALL_INDIRECT"),
            0x92 => {
                if pc + 1 < bytecode.len() {
                    let syscall_id = bytecode[pc + 1];
                    println!("CALL_NATIVE {}", syscall_id);
                    pc += 1;
                } else {
                    println!("CALL_NATIVE (incomplete)");
                }
            }
            0x83 => println!("PREPARE_CALL"),
            0x84 => println!("FINISH_CALL"),
            0x90 => println!("ALLOC_LOCALS"),
            0x91 => println!("DEALLOC_LOCALS"),
            0xA2 => {
                if pc + 1 < bytecode.len() {
                    let index = bytecode[pc + 1];
                    println!("SET_LOCAL {}", index);
                    pc += 1;
                } else {
                    println!("SET_LOCAL (incomplete)");
                }
            }
            0xA3 => {
                if pc + 1 < bytecode.len() {
                    let index = bytecode[pc + 1];
                    println!("GET_LOCAL {}", index);
                    pc += 1;
                } else {
                    println!("GET_LOCAL (incomplete)");
                }
            }
            0x94 => println!("CLEAR_LOCAL"),
            0x95 => {
                if pc + 1 < bytecode.len() {
                    let index = bytecode[pc + 1];
                    println!("LOAD_PARAM {}", index);
                    pc += 1;
                } else {
                    println!("LOAD_PARAM (incomplete)");
                }
            }
            0x96 => {
                if pc + 1 < bytecode.len() {
                    let index = bytecode[pc + 1];
                    println!("STORE_PARAM {}", index);
                    pc += 1;
                } else {
                    println!("STORE_PARAM (incomplete)");
                }
            }
            0xA0 => println!("STORE_FIELD_ZEROCOPY"),
            0xA1 => println!("LOAD_FIELD_ZEROCOPY"),
            0xA4 => println!("DATA_LEN"),
            0xA5 => println!("ARRAY_SET"),
            0xA6 => println!("ARRAY_GET"),
            0xA7 => println!("ARRAY_LEN"),
            0xA8 => println!("CREATE_ARRAY"),
            0xA9 => println!("EMIT_EVENT"),
            0xAA => println!("LOG_DATA"),
            0xAB => println!("GET_SIGNER_KEY"),
            0xAC => println!("PUSH_ARRAY_LITERAL"),
            0xAD => println!("ARRAY_INDEX"),
            0xAE => println!("ARRAY_LENGTH"),
            0xAF => println!("PUSH_STRING_LITERAL"),
            0xB0 => println!("STRING_LENGTH"),
            0xB1 => println!("ARRAY_CONCAT"),
            0xB2 => println!("CHECK_DEDUPE_TABLE"),
            0xB3 => println!("CHECK_CACHED"),
            0xB4 => println!("CHECK_COMPLEXITY_GROUP"),
            0xB5 => println!("INIT_CONSTRAINT_CACHE"),
            0xB6 => println!("CHECK_DEDUPE_MASK"),
            0xB7 => println!("CHECK_LIFTED"),
            0xB8 => println!("INVALIDATE_CACHE"),
            0xB9 => println!("RESULT_ERR"),
            0xC0..=0xCF => {
                println!("[REMOVED] Account view operation - use LOAD_FIELD/STORE_FIELD")
            }
            0xD8 => println!("PUSH_0 (nibble immediate)"),
            0xD9 => println!("PUSH_1 (nibble immediate)"),
            0xDA => println!("PUSH_2 (nibble immediate)"),
            0xDB => println!("PUSH_3 (nibble immediate)"),
            0xDC => println!("LOAD_PARAM_0 (nibble immediate)"),
            0xDD => println!("LOAD_PARAM_1 (nibble immediate)"),
            0xDE => println!("LOAD_PARAM_2 (nibble immediate)"),
            0xDF => println!("LOAD_PARAM_3 (nibble immediate)"),
            0xE0 => println!("LOAD_REG_U8"),
            0xE1 => println!("LOAD_REG_U32"),
            0xE2 => println!("LOAD_REG_U64"),
            0xE3 => println!("LOAD_REG_BOOL"),
            0xE4 => println!("LOAD_REG_PUBKEY"),
            0xE5 => println!("ADD_REG"),
            0xE6 => println!("SUB_REG"),
            0xE7 => println!("MUL_REG"),
            0xE8 => println!("DIV_REG"),
            0xE9 => println!("EQ_REG"),
            0xEA => println!("GT_REG"),
            0xEB => println!("LT_REG"),
            0xEC => println!("PUSH_REG"),
            0xED => println!("POP_REG"),
            0xEE => println!("COPY_REG"),
            0xEF => println!("CLEAR_REG"),
            0xF0 => println!("RESERVED_F0 (was COMPACT_FIELD_LOAD)"),
            0xF1 => println!("RESERVED_F1 (was COMPACT_FIELD_STORE)"),
            0xF2 => println!("PUSH_IMMEDIATE_U64"),
            0xF3 => println!("PUSH_IMMEDIATE_U32"),
            0xF4 => println!("PUSH_IMMEDIATE_U16"),
            0xF5 => println!("PUSH_IMMEDIATE_U8"),
            0xF7 => println!("RESERVED_F7 (was STORE_FIELD_VLE)"),
            0xF8 => println!("RESERVED_F8 (was LOAD_FIELD_VLE)"),
            0xF9 => println!("JUMP_VLE"),
            0xFA => println!("JUMP_IF_VLE"),
            0xFB => println!("RLE_PUSH_ZERO"),
            0xFC => println!("RLE_NOP"),
            0xFD => println!("COMPRESSION_MARKER"),
            0xFE => println!("VLE_MARKER"),
            0xFF => println!("COMPACT_SECTION_END"),
            _ => println!("UNKNOWN ({})", opcode),
        }
        pc += 1;
    }
}

mod hex {
    pub fn encode(data: &[u8]) -> String {
        data.iter()
            .map(|b| format!("{:02x}", b))
            .collect::<Vec<_>>()
            .join("")
    }
}
