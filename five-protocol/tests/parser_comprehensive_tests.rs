use five_protocol::bytecode_builder::BytecodeBuilder;
use five_protocol::parser::{parse_instruction, ParseError};
use five_protocol::opcodes::*;
use five_protocol::encoding::VLE;

#[test]
fn test_arg_type_u8() {
    // PUSH_U8 (0x18) takes one U8 argument
    let mut builder = BytecodeBuilder::new();
    builder.emit_u8(PUSH_U8).emit_u8(42);
    let bytecode = builder.build();

    let (inst, size) = parse_instruction(&bytecode, 0).expect("Failed to parse PUSH_U8");
    assert_eq!(inst.opcode, PUSH_U8);
    assert_eq!(inst.arg1, 42);
    assert_eq!(inst.size, 2);
    assert_eq!(size, 2);

    // Error case: truncated
    assert_eq!(parse_instruction(&[PUSH_U8], 0), Err(ParseError::InstructionOutOfBounds));
}

#[test]
fn test_arg_type_u16() {
    // PUSH_U16 (0x19) takes ArgType::U16 (Fixed 2 bytes LE)
    let mut builder = BytecodeBuilder::new();
    builder.emit_u8(PUSH_U16).emit_u16(0x1234);
    let bytecode = builder.build();

    let (inst, size) = parse_instruction(&bytecode, 0).expect("Failed to parse PUSH_U16");
    assert_eq!(inst.opcode, PUSH_U16);
    assert_eq!(inst.arg1, 0x1234);
    assert_eq!(inst.size, 3); // 1 opcode + 2 arg
    assert_eq!(size, 3);

    // Error case: truncated (1 byte)
    assert_eq!(parse_instruction(&[PUSH_U16, 0x34], 0), Err(ParseError::InstructionOutOfBounds));
}

#[test]
fn test_arg_type_u32_vle() {
    // PUSH_U32 (0x1A) takes ArgType::U32 (VLE)
    let mut builder = BytecodeBuilder::new();
    let (len, bytes) = VLE::encode_u32(0x123456); // Fits in 3 bytes
    builder.emit_u8(PUSH_U32).emit_bytes(&bytes[..len]);
    let bytecode = builder.build();

    let (inst, size) = parse_instruction(&bytecode, 0).expect("Failed to parse PUSH_U32");
    assert_eq!(inst.opcode, PUSH_U32);
    assert_eq!(inst.arg1, 0x123456);
    assert_eq!(inst.size, 1 + len);
    assert_eq!(size, 1 + len);

    // Error case: Invalid VLE (truncated)
    // 0x80 means more bytes coming
    assert_eq!(parse_instruction(&[PUSH_U32, 0x80], 0), Err(ParseError::InvalidVLE));
}

#[test]
fn test_arg_type_u64_vle() {
    // PUSH_U64 (0x1B) takes ArgType::U64 (VLE)
    let mut builder = BytecodeBuilder::new();
    let val = 0x1234567890ABCDEF;
    let (len, bytes) = VLE::encode_u64(val);
    builder.emit_u8(PUSH_U64).emit_bytes(&bytes[..len]);
    let bytecode = builder.build();

    let (inst, size) = parse_instruction(&bytecode, 0).expect("Failed to parse PUSH_U64");
    assert_eq!(inst.opcode, PUSH_U64);
    // ParsedInstruction stores arg1 as u64.
    assert_eq!(inst.arg1, val);
    assert_eq!(inst.size, 1 + len);
    assert_eq!(size, 1 + len);

    // Error case: Invalid VLE
    assert_eq!(parse_instruction(&[PUSH_U64, 0x80], 0), Err(ParseError::InvalidVLE));
}

#[test]
fn test_arg_type_account_index() {
    // LOAD_ACCOUNT (0x51) takes ArgType::AccountIndex (VLE)
    let mut builder = BytecodeBuilder::new();
    let idx = 150; // > 127, so 2 bytes VLE
    let (len, bytes) = VLE::encode_u32(idx);
    builder.emit_u8(LOAD_ACCOUNT).emit_bytes(&bytes[..len]);
    let bytecode = builder.build();

    let (inst, _size) = parse_instruction(&bytecode, 0).expect("Failed to parse LOAD_ACCOUNT");
    assert_eq!(inst.opcode, LOAD_ACCOUNT);
    assert_eq!(inst.arg1, idx as u64);
    assert_eq!(inst.size, 1 + len);

    // Error case
    assert_eq!(parse_instruction(&[LOAD_ACCOUNT, 0x80], 0), Err(ParseError::InvalidVLE));
}

#[test]
fn test_arg_type_local_index() {
    // CLEAR_LOCAL (0xA4) takes ArgType::LocalIndex (VLE)
    let mut builder = BytecodeBuilder::new();
    let idx = 300;
    let (len, bytes) = VLE::encode_u32(idx);
    builder.emit_u8(CLEAR_LOCAL).emit_bytes(&bytes[..len]);
    let bytecode = builder.build();

    let (inst, size) = parse_instruction(&bytecode, 0).expect("Failed to parse CLEAR_LOCAL");
    assert_eq!(inst.opcode, CLEAR_LOCAL);
    assert_eq!(inst.arg1, idx as u64);
    assert_eq!(size, 1 + len);
}

#[test]
fn test_arg_type_register_index() {
    // LOAD_REG_U8 (0xB0) takes ArgType::RegisterIndex (U8)
    let mut builder = BytecodeBuilder::new();
    builder.emit_u8(LOAD_REG_U8).emit_u8(5);
    let bytecode = builder.build();

    let (inst, size) = parse_instruction(&bytecode, 0).expect("Failed to parse LOAD_REG_U8");
    assert_eq!(inst.opcode, LOAD_REG_U8);
    assert_eq!(inst.arg1, 5);
    assert_eq!(size, 2);

    assert_eq!(parse_instruction(&[LOAD_REG_U8], 0), Err(ParseError::InstructionOutOfBounds));
}

#[test]
fn test_arg_type_two_registers() {
    // COPY_REG (0xBE) takes ArgType::TwoRegisters (U8, U8)
    let mut builder = BytecodeBuilder::new();
    builder.emit_u8(COPY_REG).emit_u8(1).emit_u8(2);
    let bytecode = builder.build();

    let (inst, size) = parse_instruction(&bytecode, 0).expect("Failed to parse COPY_REG");
    assert_eq!(inst.opcode, COPY_REG);
    assert_eq!(inst.arg1, 1);
    assert_eq!(inst.arg2, 2);
    assert_eq!(size, 3);

    assert_eq!(parse_instruction(&[COPY_REG, 1], 0), Err(ParseError::InstructionOutOfBounds));
}

#[test]
fn test_arg_type_three_registers() {
    // ADD_REG (0xB5) takes ArgType::ThreeRegisters (U8, U8, U8)
    // Parser packs arg2 and arg3 into inst.arg2:
    // arg2 = ((byte2 as u32) << 8) | byte3 as u32
    let mut builder = BytecodeBuilder::new();
    builder.emit_u8(ADD_REG).emit_u8(10).emit_u8(20).emit_u8(30);
    let bytecode = builder.build();

    let (inst, size) = parse_instruction(&bytecode, 0).expect("Failed to parse ADD_REG");
    assert_eq!(inst.opcode, ADD_REG);
    assert_eq!(inst.arg1, 10);
    // arg2 = (20 << 8) | 30 = 5120 + 30 = 5150
    assert_eq!(inst.arg2, (20 << 8) | 30);
    assert_eq!(size, 4);

    assert_eq!(parse_instruction(&[ADD_REG, 10, 20], 0), Err(ParseError::InstructionOutOfBounds));
}

#[test]
fn test_arg_type_call_internal() {
    // CALL (0x90) takes ArgType::CallInternal
    // Layout: param_count (u8) + function_address (u16)
    // Parser: arg1 = func_addr, arg2 = param_count
    let mut builder = BytecodeBuilder::new();
    let func_addr: u16 = 0x1234;
    let param_count: u8 = 3;
    builder.emit_u8(CALL).emit_u8(param_count).emit_u16(func_addr);
    let bytecode = builder.build();

    let (inst, size) = parse_instruction(&bytecode, 0).expect("Failed to parse CALL");
    assert_eq!(inst.opcode, CALL);
    assert_eq!(inst.arg1, func_addr as u64);
    assert_eq!(inst.arg2, param_count as u64);
    assert_eq!(size, 4); // 1 + 1 + 2

    assert_eq!(parse_instruction(&[CALL, 3, 0x34], 0), Err(ParseError::InstructionOutOfBounds));
}

#[test]
fn test_arg_type_call_external() {
    // CALL_EXTERNAL (0x91) takes ArgType::CallExternal
    // Layout: account_index (u8) + func_offset (u16) + param_count (u8)
    // Parser: arg1 = (account_idx << 24) | func_offset
    //         arg2 = param_count
    let mut builder = BytecodeBuilder::new();
    let acc_idx: u8 = 5;
    let func_off: u16 = 0x1000;
    let param_cnt: u8 = 2;
    builder.emit_u8(CALL_EXTERNAL)
        .emit_u8(acc_idx)
        .emit_u16(func_off)
        .emit_u8(param_cnt);
    let bytecode = builder.build();

    let (inst, size) = parse_instruction(&bytecode, 0).expect("Failed to parse CALL_EXTERNAL");
    assert_eq!(inst.opcode, CALL_EXTERNAL);
    assert_eq!(inst.arg1, (5 << 24) | 0x1000);
    assert_eq!(inst.arg2, 2);
    assert_eq!(size, 5); // 1 + 1 + 2 + 1

    assert_eq!(parse_instruction(&[CALL_EXTERNAL, 5, 0x00, 0x10], 0), Err(ParseError::InstructionOutOfBounds));
}

#[test]
fn test_arg_type_account_field() {
    // LOAD_FIELD (0x43) takes ArgType::AccountField
    // Layout: account_index (u8) + field_offset (VLE)
    // Parser: arg1 = account_idx, arg2 = field_offset
    let mut builder = BytecodeBuilder::new();
    let acc_idx: u8 = 7;
    let field_off: u32 = 12345;
    let (len, bytes) = VLE::encode_u32(field_off);
    builder.emit_u8(LOAD_FIELD)
        .emit_u8(acc_idx)
        .emit_bytes(&bytes[..len]);
    let bytecode = builder.build();

    let (inst, size) = parse_instruction(&bytecode, 0).expect("Failed to parse LOAD_FIELD");
    assert_eq!(inst.opcode, LOAD_FIELD);
    assert_eq!(inst.arg1, 7);
    assert_eq!(inst.arg2, 12345);
    assert_eq!(size, 1 + 1 + len);

    // Truncated VLE
    assert_eq!(parse_instruction(&[LOAD_FIELD, 7, 0x80], 0), Err(ParseError::InvalidVLE));
    // Missing VLE
    assert_eq!(parse_instruction(&[LOAD_FIELD, 7], 0), Err(ParseError::InvalidVLE));
}

#[test]
fn test_push_string_literal() {
    // PUSH_STRING_LITERAL (0x66) - special handling in ArgType::U8
    // U8 is length, then bytes follow.
    let mut builder = BytecodeBuilder::new();
    let s = "Hello";
    builder.emit_u8(PUSH_STRING_LITERAL)
        .emit_u8(s.len() as u8)
        .emit_bytes(s.as_bytes());
    let bytecode = builder.build();

    let (inst, size) = parse_instruction(&bytecode, 0).expect("Failed to parse PUSH_STRING_LITERAL");
    assert_eq!(inst.opcode, PUSH_STRING_LITERAL);
    assert_eq!(inst.arg1, 5); // Length
    assert_eq!(size, 1 + 1 + 5);

    // Bounds check
    // Length says 5, but only 4 bytes provided
    let mut builder = BytecodeBuilder::new();
    builder.emit_u8(PUSH_STRING_LITERAL)
        .emit_u8(5)
        .emit_bytes(b"Hell");
    let bytecode = builder.build();
    assert_eq!(parse_instruction(&bytecode, 0), Err(ParseError::InstructionOutOfBounds));
}

#[test]
fn test_push_string() {
    // PUSH_STRING (0x67) - special handling in ArgType::U32
    // U32 (VLE) is length, then bytes follow.
    let mut builder = BytecodeBuilder::new();
    let s = "World";
    let (len, len_bytes) = VLE::encode_u32(s.len() as u32);
    builder.emit_u8(PUSH_STRING)
        .emit_bytes(&len_bytes[..len])
        .emit_bytes(s.as_bytes());
    let bytecode = builder.build();

    let (inst, size) = parse_instruction(&bytecode, 0).expect("Failed to parse PUSH_STRING");
    assert_eq!(inst.opcode, PUSH_STRING);
    assert_eq!(inst.arg1, 5);
    assert_eq!(size, 1 + len + 5);

    // Bounds check
    // Length says 5, but 0 bytes provided
    let mut builder = BytecodeBuilder::new();
    builder.emit_u8(PUSH_STRING)
        .emit_bytes(&len_bytes[..len]); // 5
    let bytecode = builder.build();
    assert_eq!(parse_instruction(&bytecode, 0), Err(ParseError::InstructionOutOfBounds));
}
