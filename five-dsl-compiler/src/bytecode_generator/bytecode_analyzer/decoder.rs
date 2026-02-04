use super::{AdvancedBytecodeAnalyzer, ControlFlowInfo, InstructionAnalysis, InstructionCategory, OperandInfo};
use five_protocol::opcodes::*;
use five_vm_mito::error::VMError;

/// Decode all instructions with full semantic understanding
pub(crate) fn decode_instructions(analyzer: &mut AdvancedBytecodeAnalyzer) -> Result<(), VMError> {
    analyzer.position = 0;
    analyzer.instructions.clear();

    // Skip magic bytes if present
    if analyzer.bytecode.len() >= 4 && &analyzer.bytecode[0..4] == b"5IVE" {
        analyzer.position = 4;
    }

    while analyzer.position < analyzer.bytecode.len() {
        let instruction = decode_single_instruction(analyzer)?;
        analyzer.instructions.push(instruction);
    }

    Ok(())
}

/// Decode a single instruction with full understanding of what follows
fn decode_single_instruction(analyzer: &mut AdvancedBytecodeAnalyzer) -> Result<InstructionAnalysis, VMError> {
    if analyzer.position >= analyzer.bytecode.len() {
        return Err(VMError::InvalidOperation);
    }

    let start_offset = analyzer.position;
    let opcode = analyzer.bytecode[analyzer.position];
    analyzer.position += 1;

    // Get opcode information from protocol definitions
    let opcode_info = five_protocol::opcodes::get_opcode_info(opcode);

    let (name, arg_type, stack_effect, compute_cost) = if let Some(info) = opcode_info {
        (
            info.name.to_string(),
            info.arg_type,
            info.stack_effect,
            info.compute_cost,
        )
    } else {
        (
            format!("UNKNOWN_{:02X}", opcode),
            five_protocol::opcodes::ArgType::None,
            0,
            1,
        )
    };

    // Decode operands based on argument type - this is the key intelligence!
    let operands = decode_operands(analyzer, arg_type, opcode)?;

    let size = analyzer.position - start_offset;
    let raw_bytes = analyzer.bytecode[start_offset..analyzer.position].to_vec();

    // Generate semantic description
    let description = generate_instruction_description(opcode, &name, &operands);

    // Determine category
    let category = categorize_instruction(opcode);

    // Analyze control flow for this instruction
    let control_flow = analyze_instruction_control_flow(opcode, &operands);

    Ok(InstructionAnalysis {
        offset: start_offset,
        opcode,
        name,
        operands,
        size,
        stack_effect: stack_effect as i32,
        compute_cost: compute_cost as u32,
        description,
        category,
        control_flow,
        raw_bytes,
    })
}

/// Helper to read VLE encoded value
fn read_vle(analyzer: &mut AdvancedBytecodeAnalyzer) -> Result<(u64, Vec<u8>, usize), VMError> {
    let mut value = 0u64;
    let mut size = 0;
    let mut bytes = Vec::new();
    let mut shift = 0;

    while analyzer.position < analyzer.bytecode.len() && size < 10 {
        let byte = analyzer.bytecode[analyzer.position];
        analyzer.position += 1;
        size += 1;
        bytes.push(byte);

        value |= ((byte & 0x7F) as u64) << shift;
        shift += 7;

        if (byte & 0x80) == 0 {
            return Ok((value, bytes, size));
        }
    }

    // If we reach here, VLE was too long or truncated
    if size == 0 {
            Err(VMError::InvalidInstructionPointer)
    } else {
            // Return what we have if truncated, or valid result if ended properly
            Ok((value, bytes, size))
    }
}

/// Decode operands based on ArgType - this provides the intelligence about what follows each opcode
fn decode_operands(
    analyzer: &mut AdvancedBytecodeAnalyzer,
    arg_type: five_protocol::opcodes::ArgType,
    opcode: u8,
) -> Result<Vec<OperandInfo>, VMError> {
    use five_protocol::opcodes::{ArgType, *};

    let mut operands = Vec::new();

    match arg_type {
        ArgType::None => {
            // No operands follow this instruction
        }
        ArgType::U8 => {
            if analyzer.position < analyzer.bytecode.len() {
                let value = analyzer.bytecode[analyzer.position];
                operands.push(OperandInfo {
                    operand_type: "u8".to_string(),
                    raw_value: vec![value],
                    decoded_value: Some(value.to_string()),
                    size: 1,
                    description: "8-bit unsigned integer".to_string(),
                });
                analyzer.position += 1;
            }
        }
        ArgType::U16 => {
            // JUMP family uses fixed u16, PUSH_U16 uses VLE
            if opcode == JUMP || opcode == JUMP_IF || opcode == JUMP_IF_NOT {
                if analyzer.position + 1 < analyzer.bytecode.len() {
                    let value = u16::from_le_bytes([
                        analyzer.bytecode[analyzer.position],
                        analyzer.bytecode[analyzer.position + 1],
                    ]);
                    operands.push(OperandInfo {
                        operand_type: "u16_fixed".to_string(),
                        raw_value: analyzer.bytecode[analyzer.position..analyzer.position + 2].to_vec(),
                        decoded_value: Some(value.to_string()),
                        size: 2,
                        description: "16-bit unsigned integer (Fixed)".to_string(),
                    });
                    analyzer.position += 2;
                }
            } else {
                // PUSH_U16 and others use VLE
                let (value, bytes, size) = read_vle(analyzer)?;
                operands.push(OperandInfo {
                    operand_type: "u16_vle".to_string(),
                    raw_value: bytes,
                    decoded_value: Some(value.to_string()),
                    size,
                    description: "16-bit unsigned integer (VLE)".to_string(),
                });
            }
        }
        ArgType::U32 => {
            if opcode == STORE {
                // STORE uses [account_index_u8, field_offset_u32] (Fixed)
                // Protocol metadata says ArgType::U32 but VM reads u8 + u32
                if analyzer.position < analyzer.bytecode.len() {
                        let acc_idx = analyzer.bytecode[analyzer.position];
                        analyzer.position += 1;
                        operands.push(OperandInfo {
                        operand_type: "account_index".to_string(),
                        raw_value: vec![acc_idx],
                        decoded_value: Some(acc_idx.to_string()),
                        size: 1,
                        description: "Account Index".to_string(),
                    });
                }
                if analyzer.position + 3 < analyzer.bytecode.len() {
                    let value = u32::from_le_bytes([
                        analyzer.bytecode[analyzer.position],
                        analyzer.bytecode[analyzer.position + 1],
                        analyzer.bytecode[analyzer.position + 2],
                        analyzer.bytecode[analyzer.position + 3],
                    ]);
                    operands.push(OperandInfo {
                        operand_type: "u32_fixed".to_string(),
                        raw_value: analyzer.bytecode[analyzer.position..analyzer.position + 4].to_vec(),
                        decoded_value: Some(value.to_string()),
                        size: 4,
                        description: "32-bit unsigned integer (Fixed)".to_string(),
                    });
                    analyzer.position += 4;
                }
            } else if analyzer.position + 3 < analyzer.bytecode.len() && (opcode == LOAD || opcode == LOAD_GLOBAL || opcode == STORE_GLOBAL) {
                    // Legacy/Other ops using fixed U32?
                    // PUSH_U32 uses VLE.
                    // LOAD_GLOBAL uses ArgType::U16 according to table.
                    // LOAD uses ArgType::U32. VM says LOAD is not implemented.
                    // Safe to default to VLE for PUSH_U32, check others.
                    if opcode == PUSH_U32 {
                        let (value, bytes, size) = read_vle(analyzer)?;
                        operands.push(OperandInfo {
                        operand_type: "u32_vle".to_string(),
                        raw_value: bytes,
                        decoded_value: Some(value.to_string()),
                        size,
                        description: "32-bit unsigned integer (VLE)".to_string(),
                    });
                    } else {
                        // Default to Fixed for unknown/legacy ops sharing ArgType::U32 if any
                        let value = u32::from_le_bytes([
                        analyzer.bytecode[analyzer.position],
                        analyzer.bytecode[analyzer.position + 1],
                        analyzer.bytecode[analyzer.position + 2],
                        analyzer.bytecode[analyzer.position + 3],
                    ]);
                    operands.push(OperandInfo {
                        operand_type: "u32".to_string(),
                        raw_value: analyzer.bytecode[analyzer.position..analyzer.position + 4].to_vec(),
                        decoded_value: Some(value.to_string()),
                        size: 4,
                        description: "32-bit unsigned integer".to_string(),
                    });
                    analyzer.position += 4;
                    }
            } else {
                    // Default VLE for PUSH_U32
                    let (value, bytes, size) = read_vle(analyzer)?;
                    operands.push(OperandInfo {
                    operand_type: "u32_vle".to_string(),
                    raw_value: bytes,
                    decoded_value: Some(value.to_string()),
                    size,
                    description: "32-bit unsigned integer (VLE)".to_string(),
                });
            }
        }
        ArgType::U64 => {
            // PUSH_U64 / PUSH_I64 use VLE
            let (value, bytes, size) = read_vle(analyzer)?;
            operands.push(OperandInfo {
                operand_type: "u64_vle".to_string(),
                raw_value: bytes,
                decoded_value: Some(value.to_string()),
                size,
                description: "64-bit unsigned integer (VLE)".to_string(),
            });
        }
        ArgType::ValueType => {
            // This is for PUSH instructions - they have a type byte followed by the value
            decode_push_operands(analyzer, &mut operands)?;
        }
        ArgType::FunctionIndex => {
            if analyzer.position < analyzer.bytecode.len() {
                let value = analyzer.bytecode[analyzer.position];
                operands.push(OperandInfo {
                    operand_type: "function_index".to_string(),
                    raw_value: vec![value],
                    decoded_value: Some(format!("function_{}", value)),
                    size: 1,
                    description: "Function index for dispatch".to_string(),
                });
                analyzer.position += 1;
            }
        }
        ArgType::LocalIndex => {
            if analyzer.position < analyzer.bytecode.len() {
                let value = analyzer.bytecode[analyzer.position];
                operands.push(OperandInfo {
                    operand_type: "local_index".to_string(),
                    raw_value: vec![value],
                    decoded_value: Some(format!("local_{}", value)),
                    size: 1,
                    description: "Local variable index".to_string(),
                });
                analyzer.position += 1;
            }
        }
        ArgType::AccountIndex => {
            if analyzer.position < analyzer.bytecode.len() {
                let value = analyzer.bytecode[analyzer.position];
                operands.push(OperandInfo {
                    operand_type: "account_index".to_string(),
                    raw_value: vec![value],
                    decoded_value: Some(format!("account_{}", value)),
                    size: 1,
                    description: "Account index in transaction".to_string(),
                });
                analyzer.position += 1;
            }
        }
        ArgType::RegisterIndex => {
            if analyzer.position < analyzer.bytecode.len() {
                let value = analyzer.bytecode[analyzer.position];
                operands.push(OperandInfo {
                    operand_type: "register_index".to_string(),
                    raw_value: vec![value],
                    decoded_value: Some(format!("r{}", value)),
                    size: 1,
                    description: "Register index (0-15)".to_string(),
                });
                analyzer.position += 1;
            }
        }
        ArgType::TwoRegisters => {
            if analyzer.position + 1 < analyzer.bytecode.len() {
                let reg1 = analyzer.bytecode[analyzer.position];
                let reg2 = analyzer.bytecode[analyzer.position + 1];
                operands.push(OperandInfo {
                    operand_type: "two_registers".to_string(),
                    raw_value: vec![reg1, reg2],
                    decoded_value: Some(format!("r{}, r{}", reg1, reg2)),
                    size: 2,
                    description: "Two register indices (dest, src)".to_string(),
                });
                analyzer.position += 2;
            }
        }
        ArgType::ThreeRegisters => {
            if analyzer.position + 2 < analyzer.bytecode.len() {
                let reg1 = analyzer.bytecode[analyzer.position];
                let reg2 = analyzer.bytecode[analyzer.position + 1];
                let reg3 = analyzer.bytecode[analyzer.position + 2];
                operands.push(OperandInfo {
                    operand_type: "three_registers".to_string(),
                    raw_value: vec![reg1, reg2, reg3],
                    decoded_value: Some(format!("r{}, r{}, r{}", reg1, reg2, reg3)),
                    size: 3,
                    description: "Three register indices (dest, src1, src2)".to_string(),
                });
                analyzer.position += 3;
            }
        }
        ArgType::CallExternal => {
            if analyzer.position + 3 < analyzer.bytecode.len() {
                let account_idx = analyzer.bytecode[analyzer.position];
                let func_offset = u16::from_le_bytes([
                    analyzer.bytecode[analyzer.position + 1],
                    analyzer.bytecode[analyzer.position + 2],
                ]);
                let param_count = analyzer.bytecode[analyzer.position + 3];

                operands.push(OperandInfo {
                    operand_type: "account_index".to_string(),
                    raw_value: vec![account_idx],
                    decoded_value: Some(format!("account_{}", account_idx)),
                    size: 1,
                    description: "External account index".to_string(),
                });

                operands.push(OperandInfo {
                    operand_type: "func_offset".to_string(),
                    raw_value: vec![analyzer.bytecode[analyzer.position + 1], analyzer.bytecode[analyzer.position + 2]],
                    decoded_value: Some(format!("offset_{}", func_offset)),
                    size: 2,
                    description: "Function entry offset".to_string(),
                });

                operands.push(OperandInfo {
                    operand_type: "param_count".to_string(),
                    raw_value: vec![param_count],
                    decoded_value: Some(param_count.to_string()),
                    size: 1,
                    description: "Parameter count".to_string(),
                });

                analyzer.position += 4;
            }
        }
        ArgType::AccountField => {
            // Account field access: account_index (u8) + field_offset (VLE)
            if analyzer.position < analyzer.bytecode.len() {
                let account_idx = analyzer.bytecode[analyzer.position];
                operands.push(OperandInfo {
                    operand_type: "account_index".to_string(),
                    raw_value: vec![account_idx],
                    decoded_value: Some(format!("account_{}", account_idx)),
                    size: 1,
                    description: "Account index for field access".to_string(),
                });
                analyzer.position += 1;

                // Decode VLE for field offset
                if analyzer.position < analyzer.bytecode.len() {
                    let mut field_offset = 0u64;
                    let mut vle_size = 0;
                    let mut byte_val;

                    while analyzer.position < analyzer.bytecode.len() && vle_size < 9 {
                        byte_val = analyzer.bytecode[analyzer.position];
                        analyzer.position += 1;
                        vle_size += 1;

                        field_offset |= ((byte_val & 0x7f) as u64) << (7 * (vle_size - 1));

                        if (byte_val & 0x80) == 0 {
                            break;
                        }
                    }

                    operands.push(OperandInfo {
                        operand_type: "field_offset".to_string(),
                        raw_value: analyzer.bytecode[analyzer.position - vle_size..analyzer.position].to_vec(),
                        decoded_value: Some(format!("offset_{}", field_offset)),
                        size: vle_size,
                        description: "Field offset (VLE encoded)".to_string(),
                    });
                }
            }
        }
        ArgType::CallInternal => {
            if analyzer.position + 2 < analyzer.bytecode.len() {
                let param_count = analyzer.bytecode[analyzer.position];
                let addr_bytes = [
                    analyzer.bytecode[analyzer.position + 1],
                    analyzer.bytecode[analyzer.position + 2],
                ];
                let func_addr = u16::from_le_bytes(addr_bytes);

                operands.push(OperandInfo {
                    operand_type: "param_count".to_string(),
                    raw_value: vec![param_count],
                    decoded_value: Some(param_count.to_string()),
                    size: 1,
                    description: "Parameter count for internal call".to_string(),
                });

                operands.push(OperandInfo {
                    operand_type: "func_addr".to_string(),
                    raw_value: vec![analyzer.bytecode[analyzer.position + 1], analyzer.bytecode[analyzer.position + 2]],
                    decoded_value: Some(format!("addr_{}", func_addr)),
                    size: 2,
                    description: "Internal function address".to_string(),
                });

                analyzer.position += 3;
            }
        }
        ArgType::AccountFieldParam => {
            // acc(u8) + offset(VLE) + param(u8)
            if analyzer.position < analyzer.bytecode.len() {
                let acc = analyzer.bytecode[analyzer.position];
                operands.push(OperandInfo {
                    operand_type: "account_index".to_string(),
                    raw_value: vec![acc],
                    decoded_value: Some(format!("account_{}", acc)),
                    size: 1,
                    description: "Account index".to_string(),
                });
                analyzer.position += 1;

                if analyzer.position < analyzer.bytecode.len() {
                    let (val, bytes, size) = read_vle(analyzer)?;
                    operands.push(OperandInfo {
                        operand_type: "field_offset".to_string(),
                        raw_value: bytes,
                        decoded_value: Some(format!("offset_{}", val)),
                        size: size,
                        description: "Field offset (VLE)".to_string(),
                    });

                    if analyzer.position < analyzer.bytecode.len() {
                        let param = analyzer.bytecode[analyzer.position];
                        operands.push(OperandInfo {
                            operand_type: "param_index".to_string(),
                            raw_value: vec![param],
                            decoded_value: Some(format!("param_{}", param)),
                            size: 1,
                            description: "Parameter index".to_string(),
                        });
                        analyzer.position += 1;
                    }
                }
            }
        }
        ArgType::FusedAccAcc => {
            // acc1(u8) + offset1(VLE) + acc2(u8) + offset2(VLE)
            if analyzer.position < analyzer.bytecode.len() {
                let acc1 = analyzer.bytecode[analyzer.position];
                operands.push(OperandInfo {
                    operand_type: "account_index".to_string(),
                    raw_value: vec![acc1],
                    decoded_value: Some(format!("acc1_{}", acc1)),
                    size: 1,
                    description: "First Account Index".to_string(),
                });
                analyzer.position += 1;

                let (val1, bytes1, size1) = read_vle(analyzer)?;
                operands.push(OperandInfo {
                    operand_type: "field_offset".to_string(),
                    raw_value: bytes1,
                    decoded_value: Some(format!("offset1_{}", val1)),
                    size: size1,
                    description: "First Field Offset".to_string(),
                });

                if analyzer.position < analyzer.bytecode.len() {
                    let acc2 = analyzer.bytecode[analyzer.position];
                    operands.push(OperandInfo {
                        operand_type: "account_index".to_string(),
                        raw_value: vec![acc2],
                        decoded_value: Some(format!("acc2_{}", acc2)),
                        size: 1,
                        description: "Second Account Index".to_string(),
                    });
                    analyzer.position += 1;

                    let (val2, bytes2, size2) = read_vle(analyzer)?;
                    operands.push(OperandInfo {
                        operand_type: "field_offset".to_string(),
                        raw_value: bytes2,
                        decoded_value: Some(format!("offset2_{}", val2)),
                        size: size2,
                        description: "Second Field Offset".to_string(),
                    });
                }
            }
        }
        ArgType::CallReg => {
            if analyzer.position + 1 < analyzer.bytecode.len() {
                let value = u16::from_le_bytes([
                    analyzer.bytecode[analyzer.position],
                    analyzer.bytecode[analyzer.position + 1],
                ]);
                operands.push(OperandInfo {
                    operand_type: "function_index".to_string(),
                    raw_value: analyzer.bytecode[analyzer.position..analyzer.position + 2].to_vec(),
                    decoded_value: Some(format!("func_{}", value)),
                    size: 2,
                    description: "Function index for register call".to_string(),
                });
                analyzer.position += 2;
            }
        }
        ArgType::RegAccountField => {
            // reg(u8) + acc(u8) + offset(VLE)
            if analyzer.position < analyzer.bytecode.len() {
                let reg = analyzer.bytecode[analyzer.position];
                operands.push(OperandInfo {
                    operand_type: "register_index".to_string(),
                    raw_value: vec![reg],
                    decoded_value: Some(format!("r{}", reg)),
                    size: 1,
                    description: "Register Index".to_string(),
                });
                analyzer.position += 1;

                if analyzer.position < analyzer.bytecode.len() {
                    let acc = analyzer.bytecode[analyzer.position];
                    operands.push(OperandInfo {
                        operand_type: "account_index".to_string(),
                        raw_value: vec![acc],
                        decoded_value: Some(format!("account_{}", acc)),
                        size: 1,
                        description: "Account Index".to_string(),
                    });
                    analyzer.position += 1;

                    let (val, bytes, size) = read_vle(analyzer)?;
                    operands.push(OperandInfo {
                        operand_type: "field_offset".to_string(),
                        raw_value: bytes,
                        decoded_value: Some(format!("offset_{}", val)),
                        size: size,
                        description: "Field Offset (VLE)".to_string(),
                    });
                }
            }
        }
        ArgType::U16Fixed => {
            if analyzer.position + 1 < analyzer.bytecode.len() {
                let value = u16::from_le_bytes([
                    analyzer.bytecode[analyzer.position],
                    analyzer.bytecode[analyzer.position + 1],
                ]);
                operands.push(OperandInfo {
                    operand_type: "u16_fixed".to_string(),
                    raw_value: analyzer.bytecode[analyzer.position..analyzer.position + 2].to_vec(),
                    decoded_value: Some(value.to_string()),
                    size: 2,
                    description: "16-bit unsigned integer (Fixed)".to_string(),
                });
                analyzer.position += 2;
            }
        }
        ArgType::U32Fixed => {
            if analyzer.position + 3 < analyzer.bytecode.len() {
                let value = u32::from_le_bytes([
                    analyzer.bytecode[analyzer.position],
                    analyzer.bytecode[analyzer.position + 1],
                    analyzer.bytecode[analyzer.position + 2],
                    analyzer.bytecode[analyzer.position + 3],
                ]);
                operands.push(OperandInfo {
                    operand_type: "u32_fixed".to_string(),
                    raw_value: analyzer.bytecode[analyzer.position..analyzer.position + 4].to_vec(),
                    decoded_value: Some(value.to_string()),
                    size: 4,
                    description: "32-bit unsigned integer (Fixed)".to_string(),
                });
                analyzer.position += 4;
            }
        }
        ArgType::FusedSubAdd => {
            // acc1(u8) + off1(VLE) + acc2(u8) + off2(VLE) + param(u8)
            if analyzer.position < analyzer.bytecode.len() {
                let acc1 = analyzer.bytecode[analyzer.position];
                operands.push(OperandInfo {
                    operand_type: "account_index".to_string(),
                    raw_value: vec![acc1],
                    decoded_value: Some(format!("acc1_{}", acc1)),
                    size: 1,
                    description: "First Account Index".to_string(),
                });
                analyzer.position += 1;

                let (val1, bytes1, size1) = read_vle(analyzer)?;
                operands.push(OperandInfo {
                    operand_type: "field_offset".to_string(),
                    raw_value: bytes1,
                    decoded_value: Some(format!("offset1_{}", val1)),
                    size: size1,
                    description: "First Field Offset".to_string(),
                });

                if analyzer.position < analyzer.bytecode.len() {
                    let acc2 = analyzer.bytecode[analyzer.position];
                    operands.push(OperandInfo {
                        operand_type: "account_index".to_string(),
                        raw_value: vec![acc2],
                        decoded_value: Some(format!("acc2_{}", acc2)),
                        size: 1,
                        description: "Second Account Index".to_string(),
                    });
                    analyzer.position += 1;

                    let (val2, bytes2, size2) = read_vle(analyzer)?;
                    operands.push(OperandInfo {
                        operand_type: "field_offset".to_string(),
                        raw_value: bytes2,
                        decoded_value: Some(format!("offset2_{}", val2)),
                        size: size2,
                        description: "Second Field Offset".to_string(),
                    });

                    if analyzer.position < analyzer.bytecode.len() {
                        let param = analyzer.bytecode[analyzer.position];
                        operands.push(OperandInfo {
                            operand_type: "param_index".to_string(),
                            raw_value: vec![param],
                            decoded_value: Some(format!("param_{}", param)),
                            size: 1,
                            description: "Parameter Index".to_string(),
                        });
                        analyzer.position += 1;
                    }
                }
            }
        }
        ArgType::ParamImm => {
            // param(u8) + imm(u8)
            if analyzer.position + 1 < analyzer.bytecode.len() {
                let param = analyzer.bytecode[analyzer.position];
                let imm = analyzer.bytecode[analyzer.position + 1];
                
                operands.push(OperandInfo {
                    operand_type: "param_index".to_string(),
                    raw_value: vec![param],
                    decoded_value: Some(format!("param_{}", param)),
                    size: 1,
                    description: "Parameter Index".to_string(),
                });

                operands.push(OperandInfo {
                    operand_type: "u8_imm".to_string(),
                    raw_value: vec![imm],
                    decoded_value: Some(imm.to_string()),
                    size: 1,
                    description: "Immediate Value (u8)".to_string(),
                });
                analyzer.position += 2;
            }
        }
        ArgType::FieldImm => {
            // acc(u8) + off(VLE) + imm(u8)
            if analyzer.position < analyzer.bytecode.len() {
                let acc = analyzer.bytecode[analyzer.position];
                operands.push(OperandInfo {
                    operand_type: "account_index".to_string(),
                    raw_value: vec![acc],
                    decoded_value: Some(format!("acc_{}", acc)),
                    size: 1,
                    description: "Account Index".to_string(),
                });
                analyzer.position += 1;

                let (val, bytes, size) = read_vle(analyzer)?;
                operands.push(OperandInfo {
                    operand_type: "field_offset".to_string(),
                    raw_value: bytes,
                    decoded_value: Some(format!("offset_{}", val)),
                    size: size,
                    description: "Field Offset".to_string(),
                });

                if analyzer.position < analyzer.bytecode.len() {
                    let imm = analyzer.bytecode[analyzer.position];
                    operands.push(OperandInfo {
                        operand_type: "u8_imm".to_string(),
                        raw_value: vec![imm],
                        decoded_value: Some(imm.to_string()),
                        size: 1,
                        description: "Immediate Value (u8)".to_string(),
                    });
                    analyzer.position += 1;
                }
            }
        }
    }

    // Handle special cases for specific opcodes that have unique operand patterns
    decode_special_operands(analyzer, opcode, &mut operands)?;

    Ok(operands)
}

/// Decode PUSH instruction operands (type + value)
fn decode_push_operands(analyzer: &mut AdvancedBytecodeAnalyzer, operands: &mut Vec<OperandInfo>) -> Result<(), VMError> {
    if analyzer.position >= analyzer.bytecode.len() {
        return Ok(());
    }

    let type_byte = analyzer.bytecode[analyzer.position];
    analyzer.position += 1;

    operands.push(OperandInfo {
        operand_type: "value_type".to_string(),
        raw_value: vec![type_byte],
        decoded_value: Some(decode_value_type_name(type_byte)),
        size: 1,
        description: "Value type indicator".to_string(),
    });

    // Decode the value based on type
    match type_byte {
        0x01 => {
            // U8
            if analyzer.position < analyzer.bytecode.len() {
                let value = analyzer.bytecode[analyzer.position];
                operands.push(OperandInfo {
                    operand_type: "u8_value".to_string(),
                    raw_value: vec![value],
                    decoded_value: Some(value.to_string()),
                    size: 1,
                    description: "8-bit unsigned integer value".to_string(),
                });
                analyzer.position += 1;
            }
        }
        0x02 => {
            // U64
            if analyzer.position + 7 < analyzer.bytecode.len() {
                let value = u64::from_le_bytes([
                    analyzer.bytecode[analyzer.position],
                    analyzer.bytecode[analyzer.position + 1],
                    analyzer.bytecode[analyzer.position + 2],
                    analyzer.bytecode[analyzer.position + 3],
                    analyzer.bytecode[analyzer.position + 4],
                    analyzer.bytecode[analyzer.position + 5],
                    analyzer.bytecode[analyzer.position + 6],
                    analyzer.bytecode[analyzer.position + 7],
                ]);
                operands.push(OperandInfo {
                    operand_type: "u64_value".to_string(),
                    raw_value: analyzer.bytecode[analyzer.position..analyzer.position + 8].to_vec(),
                    decoded_value: Some(value.to_string()),
                    size: 8,
                    description: "64-bit unsigned integer value".to_string(),
                });
                analyzer.position += 8;
            }
        }
        0x03 => {
            // Bool
            if analyzer.position < analyzer.bytecode.len() {
                let value = analyzer.bytecode[analyzer.position];
                operands.push(OperandInfo {
                    operand_type: "bool_value".to_string(),
                    raw_value: vec![value],
                    decoded_value: Some(if value == 0 {
                        "false".to_string()
                    } else {
                        "true".to_string()
                    }),
                    size: 1,
                    description: "Boolean value".to_string(),
                });
                analyzer.position += 1;
            }
        }
        0x04 => {
            // Pubkey
            if analyzer.position + 31 < analyzer.bytecode.len() {
                let pubkey_bytes = analyzer.bytecode[analyzer.position..analyzer.position + 32].to_vec();
                let pubkey_hex = pubkey_bytes
                    .iter()
                    .map(|b| format!("{:02x}", b))
                    .collect::<String>();
                operands.push(OperandInfo {
                    operand_type: "pubkey_value".to_string(),
                    raw_value: pubkey_bytes,
                    decoded_value: Some(pubkey_hex),
                    size: 32,
                    description: "32-byte public key".to_string(),
                });
                analyzer.position += 32;
            }
        }
        0x05 => {
            // String
            if analyzer.position + 3 < analyzer.bytecode.len() {
                let len = u32::from_le_bytes([
                    analyzer.bytecode[analyzer.position],
                    analyzer.bytecode[analyzer.position + 1],
                    analyzer.bytecode[analyzer.position + 2],
                    analyzer.bytecode[analyzer.position + 3],
                ]);
                analyzer.position += 4;

                if analyzer.position + len as usize <= analyzer.bytecode.len() {
                    let string_bytes =
                        analyzer.bytecode[analyzer.position..analyzer.position + len as usize].to_vec();
                    let string_value = String::from_utf8_lossy(&string_bytes).to_string();

                    operands.push(OperandInfo {
                        operand_type: "string_length".to_string(),
                        raw_value: len.to_le_bytes().to_vec(),
                        decoded_value: Some(len.to_string()),
                        size: 4,
                        description: "String length in bytes".to_string(),
                    });

                    operands.push(OperandInfo {
                        operand_type: "string_value".to_string(),
                        raw_value: string_bytes,
                        decoded_value: Some(format!("\"{}\"", string_value)),
                        size: len as usize,
                        description: "UTF-8 string data".to_string(),
                    });

                    analyzer.position += len as usize;
                }
            }
        }
        _ => {
            // Unknown type - just skip
        }
    }

    Ok(())
}

/// Decode special operands for specific opcodes
fn decode_special_operands(
    analyzer: &mut AdvancedBytecodeAnalyzer,
    opcode: u8,
    operands: &mut Vec<OperandInfo>,
) -> Result<(), VMError> {
    use five_protocol::opcodes::*;

    match opcode {
        // COMPACT_FIELD_LOAD | COMPACT_FIELD_STORE removed - use LOAD_FIELD/STORE_FIELD instead
        LOAD_INPUT => {
            // LOAD_INPUT has type + param_index
            if operands.is_empty() && analyzer.position + 1 < analyzer.bytecode.len() {
                let type_byte = analyzer.bytecode[analyzer.position];
                let param_index = analyzer.bytecode[analyzer.position + 1];

                operands.push(OperandInfo {
                    operand_type: "input_type".to_string(),
                    raw_value: vec![type_byte],
                    decoded_value: Some(decode_value_type_name(type_byte)),
                    size: 1,
                    description: "Expected input parameter type".to_string(),
                });

                operands.push(OperandInfo {
                    operand_type: "param_index".to_string(),
                    raw_value: vec![param_index],
                    decoded_value: Some(format!("param_{}", param_index)),
                    size: 1,
                    description: "Parameter index in function".to_string(),
                });

                analyzer.position += 2;
            }
        }
        // Add more special cases as needed
        _ => {}
    }

    Ok(())
}

/// Generate semantic description for an instruction
fn generate_instruction_description(
    opcode: u8,
    name: &str,
    operands: &[OperandInfo],
) -> String {
    use five_protocol::opcodes::*;

    match opcode {
        HALT => "Stop execution and return".to_string(),
        PUSH_U64 | PUSH_U8 | PUSH_I64 | PUSH_BOOL | PUSH_PUBKEY => {
            if operands.len() >= 2 {
                format!(
                    "Push {} value {} onto stack",
                    operands[0]
                        .decoded_value
                        .as_ref()
                        .unwrap_or(&"unknown".to_string()),
                    operands[1]
                        .decoded_value
                        .as_ref()
                        .unwrap_or(&"?".to_string())
                )
            } else {
                "Push value onto stack".to_string()
            }
        }
        POP => "Remove top value from stack".to_string(),
        DUP => "Duplicate top stack value".to_string(),
        SWAP => "Swap top two stack values".to_string(),
        ADD => "Add top two stack values".to_string(),
        SUB => "Subtract top two stack values".to_string(),
        MUL => "Multiply top two stack values".to_string(),
        DIV => "Divide top two stack values".to_string(),
        GT => "Compare if first > second (stack)".to_string(),
        LT => "Compare if first < second (stack)".to_string(),
        EQ => "Compare if first == second (stack)".to_string(),
        JUMP => {
            if !operands.is_empty() {
                format!(
                    "Jump to offset {}",
                    operands[0]
                        .decoded_value
                        .as_ref()
                        .unwrap_or(&"?".to_string())
                )
            } else {
                "Jump to address".to_string()
            }
        }
        JUMP_IF => {
            if !operands.is_empty() {
                format!(
                    "Jump to offset {} if stack top is true",
                    operands[0]
                        .decoded_value
                        .as_ref()
                        .unwrap_or(&"?".to_string())
                )
            } else {
                "Conditional jump if true".to_string()
            }
        }
        LOAD_INPUT => {
            if operands.len() >= 2 {
                format!(
                    "Load {} parameter {} from function input",
                    operands[0]
                        .decoded_value
                        .as_ref()
                        .unwrap_or(&"unknown".to_string()),
                    operands[1]
                        .decoded_value
                        .as_ref()
                        .unwrap_or(&"?".to_string())
                )
            } else {
                "Load parameter from function input".to_string()
            }
        }
        STORE => {
            if !operands.is_empty() {
                format!(
                    "Store stack value to account {}",
                    operands[0]
                        .decoded_value
                        .as_ref()
                        .unwrap_or(&"?".to_string())
                )
            } else {
                "Store value to account".to_string()
            }
        }
        LOAD => {
            if !operands.is_empty() {
                format!(
                    "Load value from account {}",
                    operands[0]
                        .decoded_value
                        .as_ref()
                        .unwrap_or(&"?".to_string())
                )
            } else {
                "Load value from account".to_string()
            }
        }
        // COMPACT_FIELD_LOAD/COMPACT_FIELD_STORE removed
        GET_CLOCK => "Get current Solana clock".to_string(),
        REQUIRE => "Assert that stack top is true (else fail)".to_string(),
        CALL => {
            if !operands.is_empty() {
                format!(
                    "Call function {}",
                    operands[0]
                        .decoded_value
                        .as_ref()
                        .unwrap_or(&"?".to_string())
                )
            } else {
                "Call function".to_string()
            }
        }
        RETURN => "Return from current function".to_string(),
        _ => {
            if operands.is_empty() {
                format!("{} operation", name)
            } else {
                format!(
                    "{} with operands: {}",
                    name,
                    operands
                        .iter()
                        .filter_map(|op| op.decoded_value.as_ref())
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        }
    }
}

/// Categorize instruction by opcode value
pub(crate) fn categorize_instruction(opcode: u8) -> InstructionCategory {
    use five_protocol::opcodes::ranges::*;

    match opcode {
        CONTROL_BASE..=0x0F => InstructionCategory::ControlFlow,
        STACK_BASE..=0x1F => InstructionCategory::Stack,
        ARITHMETIC_BASE..=0x2F => InstructionCategory::Arithmetic,
        LOGICAL_BASE..=0x3F => InstructionCategory::Logical,
        MEMORY_BASE..=0x4F => InstructionCategory::Memory,
        ACCOUNT_BASE..=0x5F => InstructionCategory::Account,
        0x60..=0x6F => InstructionCategory::Array,
        CONSTRAINT_BASE..=0x7F => InstructionCategory::Constraint,
        SYSTEM_BASE..=0x8F => InstructionCategory::System,
        FUNCTION_BASE..=0x9F => InstructionCategory::Function,
        LOCAL_BASE..=0xAF => InstructionCategory::Local,
        REGISTER_BASE..=0xBF => InstructionCategory::Register,
        0xC0..=0xCF => InstructionCategory::Unknown, // Removed account views
        0xD0..=0xD7 => InstructionCategory::Local,   // Nibble locals
        0xD8..=0xDF => InstructionCategory::Test,    // Test framework
        PATTERN_FUSION_BASE..=0xEF => InstructionCategory::PatternFusion,
        ADVANCED_BASE..=0xFF => InstructionCategory::Advanced,
    }
}

/// Analyze control flow for a single instruction
fn analyze_instruction_control_flow(
    opcode: u8,
    operands: &[OperandInfo],
) -> ControlFlowInfo {
    use five_protocol::opcodes::*;

    match opcode {
        JUMP => {
            let target = if !operands.is_empty() {
                operands[0]
                    .decoded_value
                    .as_ref()
                    .and_then(|s| s.parse::<usize>().ok())
                    .map(|t| vec![t])
                    .unwrap_or_default()
            } else {
                Vec::new()
            };

            ControlFlowInfo {
                is_jump: true,
                jump_targets: target,
                can_fall_through: false,
                is_terminator: false,
            }
        }
        JUMP_IF | JUMP_IF_NOT => {
            let target = if !operands.is_empty() {
                operands[0]
                    .decoded_value
                    .as_ref()
                    .and_then(|s| s.parse::<usize>().ok())
                    .map(|t| vec![t])
                    .unwrap_or_default()
            } else {
                Vec::new()
            };

            ControlFlowInfo {
                is_jump: true,
                jump_targets: target,
                can_fall_through: true, // Conditional jumps can fall through
                is_terminator: false,
            }
        }
        HALT | RETURN | RETURN_VALUE => ControlFlowInfo {
            is_jump: false,
            jump_targets: Vec::new(),
            can_fall_through: false,
            is_terminator: true,
        },
        CALL => {
            // Function calls jump to other functions but return
            ControlFlowInfo {
                is_jump: true,
                jump_targets: Vec::new(), // Would need function table to resolve
                can_fall_through: true,
                is_terminator: false,
            }
        }
        _ => {
            // Regular instructions just fall through
            ControlFlowInfo {
                is_jump: false,
                jump_targets: Vec::new(),
                can_fall_through: true,
                is_terminator: false,
            }
        }
    }
}

/// Decode value type byte to string name
pub fn decode_value_type_name(type_byte: u8) -> String {
    match type_byte {
        0x01 => "U8".to_string(),
        0x02 => "U64".to_string(),
        0x03 => "BOOL".to_string(),
        0x04 => "PUBKEY".to_string(),
        0x05 => "STRING".to_string(),
        0x06 => "ACCOUNT".to_string(),
        _ => format!("UNKNOWN_TYPE_{:02X}", type_byte),
    }
}
