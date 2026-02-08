use five_protocol::opcodes::{self, ArgType, get_opcode_info};

fn read_vle(data: &[u8]) -> Option<(u64, usize)> {
    let mut value: u64 = 0;
    let mut shift = 0;
    let mut len = 0;
    for &byte in data {
        len += 1;
        value |= ((byte & 0x7F) as u64) << shift;
        if byte & 0x80 == 0 {
            return Some((value, len));
        }
        shift += 7;
        if shift >= 64 { return None; } // Overflow protection
    }
    None // Incomplete
}

pub fn disassemble_bytecode(bytecode: &[u8]) {
    // Use five_protocol::parse_header to correctly skip magic bytes, feature flags,
    // and all metadata sections (function names, etc.)
    let (header, start_offset) = match five_protocol::parse_header(bytecode) {
        Ok(res) => res,
        Err(_) => {
            // Fallback for legacy scripts: skip 10 bytes if magic matches, else 0
            let off = if bytecode.len() >= 4 && &bytecode[0..4] == b"5IVE" { 10 } else { 0 };
            (five_protocol::OptimizedHeader {
                magic: [0; 4],
                features: 0,
                public_function_count: 0,
                total_function_count: 0,
            }, off)
        }
    };

    if start_offset > 0 {
        println!("HEADER: features=0x{:08X} public={} total={}", 
            header.features, header.public_function_count, header.total_function_count);
    }

    println!("Disassembly relative to offset {}:", start_offset);
    let mut pc = start_offset;

    while pc < bytecode.len() {
        let opcode = bytecode[pc];
        print!("  {:04x}: {:02x} ", pc, opcode);

        let info = get_opcode_info(opcode);
        let name = info.map(|i| i.name).unwrap_or("UNKNOWN");
        print!("{} ", name);

        let args_start = pc + 1;
        let mut len = 1;

        // Handle specific opcodes that might deviate from ArgType or need custom display
        match opcode {
            opcodes::PUSH_PUBKEY => {
                // 32 bytes fixed
                if args_start + 32 <= bytecode.len() {
                     print!("(32 bytes)");
                     len += 32;
                } else { print!("(incomplete)"); }
            }
            opcodes::PUSH_U128 => {
                // 16 bytes fixed
                if args_start + 16 <= bytecode.len() {
                     print!("(16 bytes)");
                     len += 16;
                } else { print!("(incomplete)"); }
            }
            opcodes::PUSH_STRING => {
                // VLE length + bytes
                if let Some((str_len, vle_len)) = read_vle(&bytecode[args_start..]) {
                     let total_len = vle_len + str_len as usize;
                     if args_start + total_len <= bytecode.len() {
                         print!("len:{}", str_len);
                         // Optionally print string content if valid UTF-8?
                         if let Ok(s) = std::str::from_utf8(&bytecode[args_start+vle_len..args_start+total_len]) {
                             print!(" \"{}\"", s);
                         }
                         len += total_len;
                     } else { print!("(incomplete string data)"); }
                } else { print!("(incomplete length)"); }
            }
            // For others, use ArgType generic handling
            _ => {
                if let Some(info) = info {
                    match info.arg_type {
                        ArgType::None => {},
                        ArgType::U8 | ArgType::FunctionIndex | ArgType::LocalIndex | ArgType::AccountIndex => {
                            if args_start < bytecode.len() {
                                print!("{}", bytecode[args_start]);
                                len += 1;
                            } else { print!("(incomplete)"); }
                        },
                        ArgType::U16 | ArgType::U32 | ArgType::U64 => {
                             // Assuming VLE for these types as per protocol optimization
                             if let Some((val, l)) = read_vle(&bytecode[args_start..]) {
                                 print!("{}", val);
                                 len += l;
                             } else { print!("(incomplete)"); }
                        },
                        ArgType::ValueType => {
                             if args_start < bytecode.len() {
                                print!("type={}", bytecode[args_start]);
                                len += 1;
                            } else { print!("(incomplete)"); }
                        },
                        ArgType::CallInternal => {
                            // param_count(u8) + function_address(u16 fixed)
                            if args_start + 2 < bytecode.len() {
                                let params = bytecode[args_start];
                                let addr = u16::from_le_bytes([bytecode[args_start+1], bytecode[args_start+2]]);
                                print!("params:{} addr:{}", params, addr);
                                len += 3;
                            } else { print!("(incomplete)"); }
                        },
                        ArgType::CallExternal => {
                            // account_index(u8) + function_offset(u16 fixed) + param_count(u8)
                            if args_start + 3 < bytecode.len() {
                                let acc = bytecode[args_start];
                                let offset = u16::from_le_bytes([bytecode[args_start+1], bytecode[args_start+2]]);
                                let params = bytecode[args_start+3];
                                print!("acc:{} offset:{} params:{}", acc, offset, params);
                                len += 4;
                            } else { print!("(incomplete)"); }
                        },
                        ArgType::AccountFieldParam => {
                            // account_index(u8) + field_offset(VLE) + param(u8)
                             if args_start < bytecode.len() {
                                let acc = bytecode[args_start];
                                print!("acc:{} ", acc);
                                let mut current_offset = args_start + 1;
                                if let Some((val, l)) = read_vle(&bytecode[current_offset..]) {
                                     print!("offset:{} ", val);
                                     current_offset += l;
                                     if current_offset < bytecode.len() {
                                         let param = bytecode[current_offset];
                                         print!("param:{}", param);
                                         len = 1 + (current_offset + 1) - args_start;
                                     } else { print!("param:(incomplete)"); len = 1 + current_offset - args_start; }
                                } else { print!("offset:(incomplete)"); len = 1 + current_offset - args_start; }
                             } else { print!("(incomplete)"); }
                        },
                        ArgType::AccountField => {
                            // account_index(u8) + field_offset(VLE)
                             if args_start < bytecode.len() {
                                let acc = bytecode[args_start];
                                print!("acc:{} ", acc);
                                if let Some((val, l)) = read_vle(&bytecode[args_start+1..]) {
                                     print!("offset:{}", val);
                                     len += 1 + l;
                                } else { print!("offset:(incomplete)"); len += 1; }
                             } else { print!("(incomplete)"); }
                        },
                        ArgType::FusedAccAcc => {
                            // acc1(u8) + offset1(VLE) + acc2(u8) + offset2(VLE)
                            if args_start < bytecode.len() {
                                let acc1 = bytecode[args_start];
                                print!("acc1:{} ", acc1);
                                let mut current_offset = args_start + 1;
                                
                                if let Some((val1, l1)) = read_vle(&bytecode[current_offset..]) {
                                    print!("offset1:{} ", val1);
                                    current_offset += l1;
                                    
                                    if current_offset < bytecode.len() {
                                        let acc2 = bytecode[current_offset];
                                        print!("acc2:{} ", acc2);
                                        current_offset += 1;
                                        
                                        if let Some((val2, l2)) = read_vle(&bytecode[current_offset..]) {
                                            print!("offset2:{}", val2);
                                            current_offset += l2;
                                            len = 1 + current_offset - args_start;
                                        } else { print!("offset2:(incomplete)"); len = 1 + current_offset - args_start; }
                                    } else { print!("acc2:(incomplete)"); len = 1 + current_offset - args_start; }
                                } else { print!("offset1:(incomplete)"); len = 1 + current_offset - args_start; }
                            } else { print!("(incomplete)"); }
                        }
                        ArgType::U16Fixed => {
                            if args_start + 1 < bytecode.len() {
                                let val = u16::from_le_bytes([bytecode[args_start], bytecode[args_start+1]]);
                                print!("{}", val);
                                len += 2;
                            } else { print!("(incomplete)"); }
                        }
                        ArgType::U32Fixed => {
                            if args_start + 3 < bytecode.len() {
                                let val = u32::from_le_bytes([
                                    bytecode[args_start], bytecode[args_start+1],
                                    bytecode[args_start+2], bytecode[args_start+3]
                                ]);
                                print!("{}", val);
                                len += 4;
                            } else { print!("(incomplete)"); }
                        }
                        ArgType::FusedSubAdd => {
                            // acc1(u8) + off1(VLE) + acc2(u8) + off2(VLE) + param(u8)
                            if args_start < bytecode.len() {
                                let acc1 = bytecode[args_start];
                                print!("acc1:{} ", acc1);
                                let mut curr = args_start + 1;
                                if let Some((off1, l1)) = read_vle(&bytecode[curr..]) {
                                    print!("off1:{} ", off1);
                                    curr += l1;
                                    if curr < bytecode.len() {
                                        let acc2 = bytecode[curr];
                                        print!("acc2:{} ", acc2);
                                        curr += 1;
                                        if let Some((off2, l2)) = read_vle(&bytecode[curr..]) {
                                            print!("off2:{} ", off2);
                                            curr += l2;
                                            if curr < bytecode.len() {
                                                let param = bytecode[curr];
                                                print!("param:{}", param);
                                                len = (curr + 1) - args_start;
                                            } else { print!("param:(incomplete)"); len = curr - args_start; }
                                        } else { print!("off2:(incomplete)"); len = curr - args_start; }
                                    } else { print!("acc2:(incomplete)"); len = curr - args_start; }
                                } else { print!("off1:(incomplete)"); len = curr - args_start; }
                            } else { print!("(incomplete)"); }
                        }
                        ArgType::ParamImm => {
                            // param(u8) + imm(u8)
                            if args_start + 1 < bytecode.len() {
                                print!("param:{} imm:{}", bytecode[args_start], bytecode[args_start+1]);
                                len += 2;
                            } else { print!("(incomplete)"); }
                        }
                        ArgType::FieldImm => {
                            // acc(u8) + off(VLE) + imm(u8)
                            if args_start < bytecode.len() {
                                let acc = bytecode[args_start];
                                print!("acc:{} ", acc);
                                let mut curr = args_start + 1;
                                if let Some((off, l)) = read_vle(&bytecode[curr..]) {
                                    print!("off:{} ", off);
                                    curr += l;
                                    if curr < bytecode.len() {
                                        let imm = bytecode[curr];
                                        print!("imm:{}", imm);
                                        len = (curr + 1) - args_start;
                                    } else { print!("imm:(incomplete)"); len = curr - args_start; }
                                } else { print!("off:(incomplete)"); len = curr - args_start; }
                            } else { print!("(incomplete)"); }
                        }
                    }
                }
            }
        }
        println!();
        pc += len;
    }
}
