use crate::bytecode_generator::disassembler::call_decoder::{call_size, decode_call_at};
use five_protocol::opcodes::{self, get_opcode_info};

pub fn disassemble_bytecode(bytecode: &[u8]) {
    let (header, start_offset) = match five_protocol::parse_header(bytecode) {
        Ok(res) => res,
        Err(_) => {
            let off = if bytecode.len() >= 4 && &bytecode[0..4] == b"5IVE" {
                10
            } else {
                0
            };
            (
                five_protocol::ScriptBytecodeHeaderV1 {
                    magic: [0; 4],
                    features: 0,
                    public_function_count: 0,
                    total_function_count: 0,
                },
                off,
            )
        }
    };
    let pool_enabled = (header.features & five_protocol::FEATURE_CONSTANT_POOL) != 0;

    if start_offset > 0 {
        println!(
            "HEADER: features=0x{:08X} public={} total={}",
            header.features, header.public_function_count, header.total_function_count
        );
    }

    println!("Disassembly relative to offset {}:", start_offset);
    let mut pc = start_offset;

    while pc < bytecode.len() {
        let opcode = bytecode[pc];
        let name = get_opcode_info(opcode).map(|i| i.name).unwrap_or("UNKNOWN");
        print!("  {:04x}: {:02x} {} ", pc, opcode, name);

        if opcode == opcodes::CALL {
            match decode_call_at(bytecode, pc) {
                Some(call) => {
                    print!("params:{} addr:{}", call.param_count, call.function_address);
                    if let Some(meta) = call.name_metadata {
                        print!(" meta:{}", meta);
                    }
                    println!();
                    let size = call_size(bytecode, pc);
                    pc += if size > 0 { size } else { 1 };
                    continue;
                }
                None => {
                    println!("(incomplete)");
                    pc += 1;
                    continue;
                }
            }
        }

        let args_start = pc + 1;
        let remaining = bytecode.get(args_start..).unwrap_or(&[]);
        let operand_size = opcodes::operand_size(opcode, remaining, pool_enabled);
        let len = match operand_size {
            Some(sz) if pc + 1 + sz <= bytecode.len() => {
                match sz {
                    0 => {}
                    1 => print!("{}", bytecode[args_start]),
                    2 => {
                        let val =
                            u16::from_le_bytes([bytecode[args_start], bytecode[args_start + 1]]);
                        print!("{}", val);
                    }
                    4 => {
                        let val = u32::from_le_bytes([
                            bytecode[args_start],
                            bytecode[args_start + 1],
                            bytecode[args_start + 2],
                            bytecode[args_start + 3],
                        ]);
                        print!("{}", val);
                    }
                    8 => {
                        let val = u64::from_le_bytes([
                            bytecode[args_start],
                            bytecode[args_start + 1],
                            bytecode[args_start + 2],
                            bytecode[args_start + 3],
                            bytecode[args_start + 4],
                            bytecode[args_start + 5],
                            bytecode[args_start + 6],
                            bytecode[args_start + 7],
                        ]);
                        print!("{}", val);
                    }
                    _ if opcode == opcodes::PUSH_STRING && !pool_enabled => {
                        let str_len = u32::from_le_bytes([
                            bytecode[args_start],
                            bytecode[args_start + 1],
                            bytecode[args_start + 2],
                            bytecode[args_start + 3],
                        ]);
                        print!("len:{}", str_len);
                    }
                    _ => print!("({} bytes)", sz),
                }
                1 + sz
            }
            _ => {
                print!("(incomplete)");
                1
            }
        };

        println!();
        pc += len;
    }
}
