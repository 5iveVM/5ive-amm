// Opcode emission utilities for the bytecode generator
//
// This module contains utilities for emitting bytecode opcodes and managing
// the bytecode generation process.

use five_protocol::opcodes;

/// Trait for opcode emission - to be implemented by the main generator
pub trait OpcodeEmitter {
    /// Emit a single opcode
    fn emit_opcode(&mut self, opcode: u8);

    /// Emit a single byte
    fn emit_u8(&mut self, value: u8);

    /// Emit a 16-bit value in little-endian format
    fn emit_u16(&mut self, value: u16);

    /// Emit a 32-bit value in little-endian format
    fn emit_u32(&mut self, value: u32);

    /// Emit a 64-bit value in little-endian format
    fn emit_u64(&mut self, value: u64);

    /// Emit multiple bytes
    fn emit_bytes(&mut self, bytes: &[u8]);

    /// Emit a VLE-encoded u32 value
    fn emit_vle_u32(&mut self, value: u32);

    /// Emit a VLE-encoded u16 value
    fn emit_vle_u16(&mut self, value: u16);

    /// Emit a VLE-encoded u64 value
    fn emit_vle_u64(&mut self, value: u64);

    /// Get current bytecode position
    fn get_position(&self) -> usize;

    /// Patch a 32-bit value at a given position
    fn patch_u32(&mut self, position: usize, value: u32);

    /// Patch a 16-bit value at a given position
    fn patch_u16(&mut self, position: usize, value: u16);

    /// Check if test functions should be included in compilation
    fn should_include_tests(&self) -> bool;
}

/// Implementation of OpcodeEmitter for the main generator
impl super::DslBytecodeGenerator {
    /// Emit a single opcode to the bytecode
    pub fn emit_opcode(&mut self, opcode: u8) {
        self.bytecode.push(opcode);
        self.position += 1;
    }

    /// Emit a single byte to the bytecode
    pub fn emit_u8(&mut self, value: u8) {
        self.bytecode.push(value);
        self.position += 1;
    }

    /// Emit a 16-bit value in little-endian format
    pub fn emit_u16(&mut self, value: u16) {
        let bytes = value.to_le_bytes();
        self.emit_bytes(&bytes);
    }

    /// Emit a 32-bit value in little-endian format
    pub fn emit_u32(&mut self, value: u32) {
        let bytes = value.to_le_bytes();
        self.emit_bytes(&bytes);
    }

    /// Emit a 64-bit value in little-endian format
    pub fn emit_u64(&mut self, value: u64) {
        let bytes = value.to_le_bytes();
        self.emit_bytes(&bytes);
    }

    /// Emit multiple bytes to the bytecode
    pub fn emit_bytes(&mut self, bytes: &[u8]) {
        self.bytecode.extend_from_slice(bytes);
        self.position += bytes.len();
    }

    /// Get current bytecode position
    pub fn get_position(&self) -> usize {
        self.position
    }

    /// Update position counter
    pub fn advance_position(&mut self, bytes: usize) {
        self.position += bytes;
    }
}

impl OpcodeEmitter for super::DslBytecodeGenerator {
    fn emit_opcode(&mut self, opcode: u8) {
        self.bytecode.push(opcode);
        self.position += 1;
    }

    fn emit_u8(&mut self, value: u8) {
        self.bytecode.push(value);
        self.position += 1;
    }

    fn emit_u16(&mut self, value: u16) {
        let bytes = value.to_le_bytes();
        self.bytecode.extend_from_slice(&bytes);
        self.position += bytes.len();
    }

    fn emit_u32(&mut self, value: u32) {
        let bytes = value.to_le_bytes();
        println!("DEBUG: emit_u32 value: {}, bytes: {:?}", value, bytes);
        self.bytecode.extend_from_slice(&bytes);
        self.position += bytes.len();
    }

    fn emit_u64(&mut self, value: u64) {
        let bytes = value.to_le_bytes();
        self.bytecode.extend_from_slice(&bytes);
        self.position += bytes.len();
    }

    fn emit_bytes(&mut self, bytes: &[u8]) {
        self.bytecode.extend_from_slice(bytes);
        self.position += bytes.len();
    }

    fn emit_vle_u32(&mut self, value: u32) {
        use five_protocol::VLE;
        let (size, bytes) = VLE::encode_u32(value);
        for i in 0..size {
            self.emit_u8(bytes[i]);
        }
    }

    fn emit_vle_u16(&mut self, value: u16) {
        use five_protocol::VLE;
        let (size, bytes) = VLE::encode_u16(value);
        for i in 0..size {
            self.emit_u8(bytes[i]);
        }
    }

    fn emit_vle_u64(&mut self, value: u64) {
        use five_protocol::VLE;
        let (size, bytes) = VLE::encode_u64(value);
        for i in 0..size {
            self.emit_u8(bytes[i]);
        }
    }

    fn get_position(&self) -> usize {
        self.position
    }

    fn patch_u32(&mut self, position: usize, value: u32) {
        let bytes = value.to_le_bytes();
        self.bytecode[position..position + 4].copy_from_slice(&bytes);
    }

    fn patch_u16(&mut self, position: usize, value: u16) {
        let bytes = value.to_le_bytes();
        self.bytecode[position..position + 2].copy_from_slice(&bytes);
    }

    fn should_include_tests(&self) -> bool {
        self.should_include_tests()
    }
}

/// Common opcode emission patterns for convenient use
pub struct OpcodePatterns;

impl OpcodePatterns {
    /// Emit a PUSH_U64 instruction with a 64-bit value
    pub fn emit_push_u64(emitter: &mut impl OpcodeEmitter, value: u64) {
        // Optimization: Use dedicated 1-byte opcodes for 0-3
        match value {
            0 => emitter.emit_opcode(opcodes::PUSH_ZERO),
            1 => emitter.emit_opcode(opcodes::PUSH_ONE),
            2 => emitter.emit_opcode(opcodes::PUSH_2),
            3 => emitter.emit_opcode(opcodes::PUSH_3),
            _ => {
                emitter.emit_opcode(opcodes::PUSH_U64);
                // VM expects VLE encoded value for PUSH_U64
                emitter.emit_vle_u64(value);
            }
        }
    }

    /// Emit a PUSH_U32 instruction with a 32-bit value (VLE encoded)
    pub fn emit_push_u32(emitter: &mut impl OpcodeEmitter, value: u32) {
        emitter.emit_opcode(opcodes::PUSH_U32);
        emitter.emit_vle_u32(value);
    }

    /// Emit a PUSH_U16 instruction with a 16-bit value (VLE encoded)
    pub fn emit_push_u16(emitter: &mut impl OpcodeEmitter, value: u16) {
        emitter.emit_opcode(opcodes::PUSH_U16);
        emitter.emit_vle_u16(value);
    }

    /// Emit a PUSH_U128 instruction with a 128-bit value - MITO-style BPF-optimized
    pub fn emit_push_u128(emitter: &mut impl OpcodeEmitter, value: u128) {
        emitter.emit_opcode(opcodes::PUSH_U128);
        emitter.emit_bytes(&value.to_le_bytes());
    }

    /// Emit a PUSH_U8 instruction with a u8 value
    pub fn emit_push_u8(emitter: &mut impl OpcodeEmitter, value: u8) {
        match value {
            0 => emitter.emit_opcode(opcodes::PUSH_ZERO),
            1 => emitter.emit_opcode(opcodes::PUSH_ONE),
            2 => emitter.emit_opcode(opcodes::PUSH_2),
            3 => emitter.emit_opcode(opcodes::PUSH_3),
            _ => {
                emitter.emit_opcode(opcodes::PUSH_U8);
                emitter.emit_u8(value);
            }
        }
    }

    /// Emit a PUSH_BOOL instruction with a boolean value
    pub fn emit_push_bool(emitter: &mut impl OpcodeEmitter, value: bool) {
        emitter.emit_opcode(opcodes::PUSH_BOOL);
        emitter.emit_u8(if value { 1 } else { 0 });
    }

    /// Emit a PUSH_PUBKEY instruction with a pubkey
    pub fn emit_push_pubkey(emitter: &mut impl OpcodeEmitter, value: &[u8; 32]) {
        emitter.emit_opcode(opcodes::PUSH_PUBKEY);
        emitter.emit_bytes(value);
    }

    /// Emit a PUSH_STRING instruction with a string index
    pub fn emit_push_string(emitter: &mut impl OpcodeEmitter, value: u8) {
        emitter.emit_opcode(opcodes::PUSH_STRING);
        emitter.emit_u8(value);
    }

    /// Emit account reference as a PUSH_U8 instruction (PUSH_ACCOUNT was removed)
    pub fn emit_push_account(emitter: &mut impl OpcodeEmitter, value: u8) {
        emitter.emit_opcode(opcodes::PUSH_U8);
        emitter.emit_u8(value);
    }

    /// Emit a PUSH_I64 instruction with an i64 value
    pub fn emit_push_i64(emitter: &mut impl OpcodeEmitter, value: i64) {
        match value {
            0 => emitter.emit_opcode(opcodes::PUSH_ZERO),
            1 => emitter.emit_opcode(opcodes::PUSH_ONE),
            2 => emitter.emit_opcode(opcodes::PUSH_2),
            3 => emitter.emit_opcode(opcodes::PUSH_3),
            _ => {
                emitter.emit_opcode(opcodes::PUSH_I64);
                // I64 uses VLE encoded u64
                emitter.emit_vle_u64(value as u64);
            }
        }
    }

    /// Emit a LOAD_FIELD instruction with account index and field index
    pub fn emit_load_field(emitter: &mut impl OpcodeEmitter, account_index: u8, field_index: u32) {
        emitter.emit_opcode(opcodes::LOAD_FIELD);
        emitter.emit_u8(account_index);
        emitter.emit_vle_u32(field_index);
    }

    /// Emit a STORE_FIELD instruction with account index and field index
    pub fn emit_store_field(emitter: &mut impl OpcodeEmitter, account_index: u8, field_index: u32) {
        emitter.emit_opcode(opcodes::STORE_FIELD);
        emitter.emit_u8(account_index);
        emitter.emit_vle_u32(field_index);
    }

    /// Emit a conditional jump instruction
    pub fn emit_jump_if_false(emitter: &mut impl OpcodeEmitter, target: u16) {
        emitter.emit_opcode(opcodes::JUMP_IF_NOT);
        emitter.emit_u16(target);
    }

    /// Emit an unconditional jump instruction
    pub fn emit_jump(emitter: &mut impl OpcodeEmitter, target: u16) {
        emitter.emit_opcode(opcodes::JUMP);
        emitter.emit_u16(target);
    }

    /// Emit a function call instruction
    /// Format: CALL param_count(u8) function_address(u16) [optional_name_len(u8) name_bytes]
    pub fn emit_call(emitter: &mut impl OpcodeEmitter, param_count: u8, function_address: u16) {
        emitter.emit_opcode(opcodes::CALL);
        emitter.emit_u8(param_count); // Parameter count (matches VM fetch_byte)
        emitter.emit_u16(function_address); // Function address (matches VM fetch_u16)
    }

    /// Emit a function call instruction with embedded function name for tooling
    /// Format: CALL param_count(u8) function_address(u16) name_len(u8) name_bytes
    pub fn emit_call_with_name(
        emitter: &mut impl OpcodeEmitter,
        param_count: u8,
        function_address: u16,
        function_name: &str,
    ) {
        emitter.emit_opcode(opcodes::CALL);
        emitter.emit_u8(param_count); // Parameter count (matches VM fetch_byte)
        emitter.emit_u16(function_address); // Function address (matches VM fetch_u16)

        // Embed function name as bytecode metadata (VM ignores this completely)
        let name_bytes = function_name.as_bytes();
        emitter.emit_u8(name_bytes.len() as u8); // Name length
        emitter.emit_bytes(name_bytes); // Function name bytes
    }

    /// Emit a function call instruction with name reference for deduplication
    /// Format: CALL param_count(u8) function_address(u16) name_index(u8)
    pub fn emit_call_with_name_ref(
        emitter: &mut impl OpcodeEmitter,
        param_count: u8,
        function_address: u16,
        name_index: u8,
    ) {
        emitter.emit_opcode(opcodes::CALL);
        emitter.emit_u8(param_count); // Parameter count (matches VM fetch_byte)
        emitter.emit_u16(function_address); // Function address (matches VM fetch_u16)

        // Embed name index as bytecode metadata (VM ignores this completely)
        // Use 0xFF as marker for name reference instead of inline name
        emitter.emit_u8(0xFF); // Name reference marker
        emitter.emit_u8(name_index); // Index to first occurrence
    }

    /// Emit a return instruction
    pub fn emit_return(emitter: &mut impl OpcodeEmitter) {
        emitter.emit_opcode(opcodes::RETURN);
    }

    /// Emit a halt instruction
    pub fn emit_halt(emitter: &mut impl OpcodeEmitter) {
        emitter.emit_opcode(opcodes::HALT);
    }

    /// Emit a BR_EQ_U8 fused compare-branch instruction
    /// Format: BR_EQ_U8 compare_value(u8) vle_offset(vle_u16)
    pub fn emit_br_eq_u8(emitter: &mut impl OpcodeEmitter, compare_value: u8, vle_offset: u16) {
        emitter.emit_opcode(opcodes::BR_EQ_U8);
        emitter.emit_u8(compare_value); // U8 value to compare against (matches VM fetch_byte)
        emitter.emit_vle_u16(vle_offset); // VLE-encoded relative offset (matches VM fetch_vle_u16)
    }
}

/// Opcode analysis utilities
pub struct OpcodeAnalyzer;

impl OpcodeAnalyzer {
    /// Check if an opcode requires immediate operands
    pub fn requires_operands(opcode: u8) -> bool {
        matches!(
            opcode,
            opcodes::PUSH_U8
                | opcodes::PUSH_U16
                | opcodes::PUSH_U32
                | opcodes::PUSH_U64
                | opcodes::PUSH_I64
                | opcodes::PUSH_BOOL
                | opcodes::PUSH_PUBKEY
                | opcodes::PUSH_STRING
                | opcodes::LOAD_FIELD
                | opcodes::STORE_FIELD
                | opcodes::JUMP
                | opcodes::JUMP_IF_NOT
                | opcodes::JUMP_IF
                | opcodes::CALL
                | opcodes::CALL_EXTERNAL
                | opcodes::BR_EQ_U8
                | opcodes::PUSH_ARRAY_LITERAL
                | opcodes::PUSH_STRING_LITERAL
                | opcodes::CREATE_ARRAY
                | opcodes::CHECK_SIGNER
                | opcodes::CHECK_WRITABLE
                | opcodes::CHECK_OWNER
                | opcodes::CHECK_INITIALIZED
                | opcodes::CHECK_PDA
                | opcodes::CHECK_UNINITIALIZED
                | opcodes::LOAD_ACCOUNT
                | opcodes::SAVE_ACCOUNT
                | opcodes::GET_ACCOUNT
                | opcodes::GET_LAMPORTS
                | opcodes::SET_LAMPORTS
                | opcodes::GET_DATA
                | opcodes::GET_KEY
                | opcodes::GET_OWNER
                | opcodes::INIT_ACCOUNT
                | opcodes::INIT_PDA_ACCOUNT
                | opcodes::SET_LOCAL
                | opcodes::GET_LOCAL
                | opcodes::LOAD_PARAM
                | opcodes::STORE_PARAM
                | opcodes::CAST
                | opcodes::LOAD_REG_U8
                | opcodes::LOAD_REG_U32
                | opcodes::LOAD_REG_U64
                | opcodes::LOAD_REG_BOOL
                | opcodes::LOAD_REG_PUBKEY
                | opcodes::ADD_REG
                | opcodes::SUB_REG
                | opcodes::MUL_REG
                | opcodes::DIV_REG
                | opcodes::EQ_REG
                | opcodes::GT_REG
                | opcodes::LT_REG
                | opcodes::PUSH_REG
                | opcodes::POP_REG
                | opcodes::COPY_REG
                | opcodes::CLEAR_REG
                | opcodes::TRANSFER_DEBIT
                | opcodes::TRANSFER_CREDIT
                | opcodes::EQ_ZERO_JUMP
                | opcodes::GT_ZERO_JUMP
                | opcodes::LT_ZERO_JUMP
        )
    }

    /// Get the logical size of operands for an opcode (not necessarily encoded size for VLE)
    /// Returns the maximum expected size in bytes for analysis purposes.
    pub fn operand_size(opcode: u8) -> usize {
        match opcode {
            opcodes::PUSH_U64 | opcodes::PUSH_I64 => 8, // VLE (logical max)
            opcodes::PUSH_U128 => 16,
            opcodes::PUSH_BOOL => 1,
            opcodes::PUSH_PUBKEY => 32,
            opcodes::PUSH_STRING => 1,
            opcodes::PUSH_U8 => 1,
            opcodes::PUSH_U16 => 2, // Fixed 2 bytes (based on parser)
            opcodes::PUSH_U32 => 4, // VLE (logical max)
            opcodes::LOAD_FIELD | opcodes::STORE_FIELD => 5, // u8 + u32 (VLE max)
            opcodes::JUMP | opcodes::JUMP_IF_NOT | opcodes::JUMP_IF => 2, // 16-bit offset
            opcodes::CALL => 3, // 8-bit param_count + 16-bit function_address
            opcodes::BR_EQ_U8 => 3, // u8 + u16 offset
            opcodes::CALL_EXTERNAL => 4, // account_index(1) + offset(2) + param_count(1)
            opcodes::PUSH_ARRAY_LITERAL | opcodes::PUSH_STRING_LITERAL | opcodes::CREATE_ARRAY => 1,
            opcodes::CHECK_SIGNER | opcodes::CHECK_WRITABLE | opcodes::CHECK_OWNER | opcodes::CHECK_INITIALIZED | opcodes::CHECK_PDA | opcodes::CHECK_UNINITIALIZED => 1, // u8 account index
            opcodes::LOAD_ACCOUNT | opcodes::SAVE_ACCOUNT | opcodes::GET_ACCOUNT | opcodes::GET_LAMPORTS | opcodes::SET_LAMPORTS | opcodes::GET_DATA | opcodes::GET_KEY | opcodes::GET_OWNER | opcodes::INIT_ACCOUNT | opcodes::INIT_PDA_ACCOUNT => 1, // u8 account index
            opcodes::SET_LOCAL | opcodes::GET_LOCAL | opcodes::LOAD_PARAM | opcodes::STORE_PARAM | opcodes::CAST => 1,
            opcodes::LOAD_REG_U8 | opcodes::LOAD_REG_U32 | opcodes::LOAD_REG_U64 | opcodes::LOAD_REG_BOOL | opcodes::LOAD_REG_PUBKEY => 1, // Reg index
            opcodes::ADD_REG | opcodes::SUB_REG | opcodes::MUL_REG | opcodes::DIV_REG | opcodes::EQ_REG | opcodes::GT_REG | opcodes::LT_REG => 3, // 3 Reg indices
            opcodes::PUSH_REG | opcodes::POP_REG | opcodes::CLEAR_REG => 1,
            opcodes::COPY_REG => 2,
            opcodes::TRANSFER_DEBIT | opcodes::TRANSFER_CREDIT => 1, // u8 account index
            opcodes::EQ_ZERO_JUMP | opcodes::GT_ZERO_JUMP | opcodes::LT_ZERO_JUMP => 2, // u16 offset
            _ => 0,
        }
    }

    /// Check if an opcode is a control flow instruction
    pub fn is_control_flow(opcode: u8) -> bool {
        matches!(
            opcode,
            opcodes::JUMP
                | opcodes::JUMP_IF_NOT
                | opcodes::JUMP_IF
                | opcodes::CALL
                | opcodes::RETURN
                | opcodes::HALT
                | opcodes::BR_EQ_U8
                | opcodes::CALL_EXTERNAL
                | opcodes::EQ_ZERO_JUMP
                | opcodes::GT_ZERO_JUMP
                | opcodes::LT_ZERO_JUMP
        )
    }

    /// Check if an opcode modifies the stack
    pub fn modifies_stack(opcode: u8) -> bool {
        matches!(
            opcode,
            opcodes::PUSH_U8
                | opcodes::PUSH_U16
                | opcodes::PUSH_U32
                | opcodes::PUSH_U64
                | opcodes::PUSH_I64
                | opcodes::PUSH_BOOL
                | opcodes::PUSH_PUBKEY
                | opcodes::PUSH_STRING
                | opcodes::LOAD_FIELD
                | opcodes::ADD
                | opcodes::SUB
                | opcodes::MUL
                | opcodes::ADD_CHECKED
                | opcodes::SUB_CHECKED
                | opcodes::MUL_CHECKED
                | opcodes::DIV
                | opcodes::MOD
                | opcodes::EQ
                | opcodes::NEQ
                | opcodes::LT
                | opcodes::LTE
                | opcodes::GT
                | opcodes::GTE
                | opcodes::AND
                | opcodes::OR
                | opcodes::NOT
                | opcodes::PUSH_U128
                | opcodes::PUSH_ARRAY_LITERAL
                | opcodes::PUSH_STRING_LITERAL
                | opcodes::CREATE_ARRAY
                | opcodes::ARRAY_GET
                | opcodes::GET_LAMPORTS
                | opcodes::GET_DATA
                | opcodes::GET_KEY
                | opcodes::GET_OWNER
                | opcodes::LOAD_ACCOUNT
                | opcodes::GET_ACCOUNT
                | opcodes::LOAD_GLOBAL
                | opcodes::GET_LOCAL
                | opcodes::LOAD_PARAM
                | opcodes::CAST
                | opcodes::PUSH_REG
                | opcodes::POP_REG
                | opcodes::PUSH_ZERO
                | opcodes::PUSH_ONE
                | opcodes::PUSH_0
                | opcodes::PUSH_1
                | opcodes::PUSH_2
                | opcodes::PUSH_3
                | opcodes::GET_LOCAL_0
                | opcodes::GET_LOCAL_1
                | opcodes::GET_LOCAL_2
                | opcodes::GET_LOCAL_3
                | opcodes::LOAD_PARAM_0
                | opcodes::LOAD_PARAM_1
                | opcodes::LOAD_PARAM_2
                | opcodes::LOAD_PARAM_3
        )
    }
}
