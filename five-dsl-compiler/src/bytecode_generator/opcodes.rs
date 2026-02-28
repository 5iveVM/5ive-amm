// Opcode emission utilities for bytecode generation.

use five_protocol::opcodes;
use five_vm_mito::error::VMError;

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

    /// Get current bytecode position
    fn get_position(&self) -> usize;

    /// Patch a 32-bit value at a given position
    fn patch_u32(&mut self, position: usize, value: u32);

    /// Patch a 16-bit value at a given position
    fn patch_u16(&mut self, position: usize, value: u16);

    /// Check if test functions should be included in compilation
    fn should_include_tests(&self) -> bool;

    /// Emit constant pool-backed literals
    fn emit_const_u8(&mut self, value: u8) -> Result<(), VMError>;
    fn emit_const_u16(&mut self, value: u16) -> Result<(), VMError>;
    fn emit_const_u32(&mut self, value: u32) -> Result<(), VMError>;
    fn emit_const_u64(&mut self, value: u64) -> Result<(), VMError>;
    fn emit_const_i64(&mut self, value: i64) -> Result<(), VMError>;
    fn emit_const_bool(&mut self, value: bool) -> Result<(), VMError>;
    fn emit_const_u128(&mut self, value: u128) -> Result<(), VMError>;
    fn emit_const_pubkey(&mut self, value: &[u8; 32]) -> Result<(), VMError>;
    fn emit_const_string(&mut self, value: &[u8]) -> Result<(), VMError>;
    fn emit_const_bytes(&mut self, value: &[u8]) -> Result<(), VMError> {
        self.emit_const_string(value)
    }
    /// Intern a u16 constant in the constant pool and return its pool slot index.
    fn intern_u16_const(&mut self, value: u16) -> Result<u16, VMError>;
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

    fn emit_const_u8(&mut self, value: u8) -> Result<(), VMError> {
        let idx = self.constant_pool.add_u64(value as u64)?;
        self.emit_pool_indexed(opcodes::PUSH_U8, opcodes::PUSH_U8_W, idx);
        Ok(())
    }

    fn emit_const_u16(&mut self, value: u16) -> Result<(), VMError> {
        let idx = self.constant_pool.add_u64(value as u64)?;
        self.emit_pool_indexed(opcodes::PUSH_U16, opcodes::PUSH_U16_W, idx);
        Ok(())
    }

    fn emit_const_u32(&mut self, value: u32) -> Result<(), VMError> {
        let idx = self.constant_pool.add_u64(value as u64)?;
        self.emit_pool_indexed(opcodes::PUSH_U32, opcodes::PUSH_U32_W, idx);
        Ok(())
    }

    fn emit_const_u64(&mut self, value: u64) -> Result<(), VMError> {
        let idx = self.constant_pool.add_u64(value)?;
        self.emit_pool_indexed(opcodes::PUSH_U64, opcodes::PUSH_U64_W, idx);
        Ok(())
    }

    fn emit_const_i64(&mut self, value: i64) -> Result<(), VMError> {
        let idx = self.constant_pool.add_u64(value as u64)?;
        self.emit_pool_indexed(opcodes::PUSH_I64, opcodes::PUSH_I64_W, idx);
        Ok(())
    }

    fn emit_const_bool(&mut self, value: bool) -> Result<(), VMError> {
        let idx = self.constant_pool.add_u64(if value { 1 } else { 0 })?;
        self.emit_pool_indexed(opcodes::PUSH_BOOL, opcodes::PUSH_BOOL_W, idx);
        Ok(())
    }

    fn emit_const_u128(&mut self, value: u128) -> Result<(), VMError> {
        let idx = self.constant_pool.add_u128(value)?;
        self.emit_pool_indexed(opcodes::PUSH_U128, opcodes::PUSH_U128_W, idx);
        Ok(())
    }

    fn emit_const_pubkey(&mut self, value: &[u8; 32]) -> Result<(), VMError> {
        let idx = self.constant_pool.add_pubkey(value)?;
        self.emit_pool_indexed(opcodes::PUSH_PUBKEY, opcodes::PUSH_PUBKEY_W, idx);
        Ok(())
    }

    fn emit_const_string(&mut self, value: &[u8]) -> Result<(), VMError> {
        let idx = self.constant_pool.add_string(value)?;
        self.emit_pool_indexed(opcodes::PUSH_STRING, opcodes::PUSH_STRING_W, idx);
        Ok(())
    }

    fn emit_const_bytes(&mut self, value: &[u8]) -> Result<(), VMError> {
        let idx = self.constant_pool.add_string(value)?;
        self.emit_pool_indexed(opcodes::PUSH_BYTES, opcodes::PUSH_BYTES_W, idx);
        Ok(())
    }

    fn intern_u16_const(&mut self, value: u16) -> Result<u16, VMError> {
        let idx = self.constant_pool.add_u64(value as u64)?;
        Ok(idx)
    }
}

impl super::DslBytecodeGenerator {
    fn emit_pool_indexed(&mut self, opcode_u8: u8, opcode_u16: u8, index: u16) {
        if index <= u8::MAX as u16 {
            self.emit_opcode(opcode_u8);
            self.emit_u8(index as u8);
        } else {
            self.emit_opcode(opcode_u16);
            self.emit_u16(index);
        }
    }
}

/// Common opcode emission patterns for convenient use
pub struct OpcodePatterns;

impl OpcodePatterns {
    /// Emit a PUSH_U64 instruction with a 64-bit value
    pub fn emit_push_u64(emitter: &mut impl OpcodeEmitter, value: u64) -> Result<(), VMError> {
        // Optimization: Use dedicated 1-byte opcodes for 0-3
        match value {
            0 => emitter.emit_opcode(opcodes::PUSH_ZERO),
            1 => emitter.emit_opcode(opcodes::PUSH_ONE),
            2 => emitter.emit_opcode(opcodes::PUSH_2),
            3 => emitter.emit_opcode(opcodes::PUSH_3),
            _ => {
                emitter.emit_const_u64(value)?;
            }
        }
        Ok(())
    }

    /// Emit a PUSH_U32 instruction with a 32-bit value (fixed LE)
    pub fn emit_push_u32(emitter: &mut impl OpcodeEmitter, value: u32) -> Result<(), VMError> {
        emitter.emit_const_u32(value)
    }

    /// Emit a PUSH_U16 instruction with a 16-bit value (fixed LE)
    pub fn emit_push_u16(emitter: &mut impl OpcodeEmitter, value: u16) -> Result<(), VMError> {
        emitter.emit_const_u16(value)
    }

    /// Emit a PUSH_U128 instruction with a 128-bit value - MITO-style BPF-optimized
    pub fn emit_push_u128(emitter: &mut impl OpcodeEmitter, value: u128) -> Result<(), VMError> {
        emitter.emit_const_u128(value)
    }

    /// Emit a PUSH_U8 instruction with a u8 value
    pub fn emit_push_u8(emitter: &mut impl OpcodeEmitter, value: u8) -> Result<(), VMError> {
        match value {
            0 => emitter.emit_opcode(opcodes::PUSH_ZERO),
            1 => emitter.emit_opcode(opcodes::PUSH_ONE),
            2 => emitter.emit_opcode(opcodes::PUSH_2),
            3 => emitter.emit_opcode(opcodes::PUSH_3),
            _ => {
                emitter.emit_const_u8(value)?;
            }
        }
        Ok(())
    }

    /// Emit a PUSH_BOOL instruction with a boolean value
    pub fn emit_push_bool(emitter: &mut impl OpcodeEmitter, value: bool) -> Result<(), VMError> {
        emitter.emit_const_bool(value)
    }

    /// Emit a PUSH_PUBKEY instruction with a pubkey
    pub fn emit_push_pubkey(
        emitter: &mut impl OpcodeEmitter,
        value: &[u8; 32],
    ) -> Result<(), VMError> {
        emitter.emit_const_pubkey(value)
    }

    /// Emit a PUSH_STRING instruction with a string index
    pub fn emit_push_string(emitter: &mut impl OpcodeEmitter, value: &[u8]) -> Result<(), VMError> {
        emitter.emit_const_string(value)
    }

    /// Emit account reference as a PUSH_U8 instruction (PUSH_ACCOUNT was removed)
    pub fn emit_push_account(emitter: &mut impl OpcodeEmitter, value: u8) -> Result<(), VMError> {
        emitter.emit_const_u8(value)
    }

    /// Emit a PUSH_I64 instruction with an i64 value
    pub fn emit_push_i64(emitter: &mut impl OpcodeEmitter, value: i64) -> Result<(), VMError> {
        match value {
            0 => emitter.emit_opcode(opcodes::PUSH_ZERO),
            1 => emitter.emit_opcode(opcodes::PUSH_ONE),
            2 => emitter.emit_opcode(opcodes::PUSH_2),
            3 => emitter.emit_opcode(opcodes::PUSH_3),
            _ => {
                emitter.emit_const_i64(value)?;
            }
        }
        Ok(())
    }

    /// Emit a LOAD_FIELD instruction with account index and field index
    pub fn emit_load_field(emitter: &mut impl OpcodeEmitter, account_index: u8, field_index: u32) {
        emitter.emit_opcode(opcodes::LOAD_FIELD);
        emitter.emit_u8(account_index);
        emitter.emit_u32(field_index);
    }

    /// Emit a STORE_FIELD instruction with account index and field index
    pub fn emit_store_field(emitter: &mut impl OpcodeEmitter, account_index: u8, field_index: u32) {
        emitter.emit_opcode(opcodes::STORE_FIELD);
        emitter.emit_u8(account_index);
        emitter.emit_u32(field_index);
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
    /// Format: BR_EQ_U8 compare_value(u8) offset(u16)
    pub fn emit_br_eq_u8(emitter: &mut impl OpcodeEmitter, compare_value: u8, offset: u16) {
        emitter.emit_opcode(opcodes::BR_EQ_U8);
        emitter.emit_u8(compare_value); // U8 value to compare against (matches VM fetch_byte)
        emitter.emit_u16(offset); // Fixed 16-bit offset (matches VM fetch_u16)
    }
}

/// Opcode analysis utilities
pub struct OpcodeAnalyzer;

impl OpcodeAnalyzer {
    /// Check if an opcode requires immediate operands
    pub fn requires_operands(opcode: u8) -> bool {
        Self::operand_size(opcode) > 0
    }

    /// Get the logical size of operands for an opcode
    /// Returns the maximum expected size in bytes for analysis purposes.
    pub fn operand_size(opcode: u8) -> usize {
        if let Some(size) = opcodes::operand_size(opcode, &[], false) {
            return size;
        }

        if let Some(info) = opcodes::get_opcode_info(opcode) {
            return match info.arg_type {
                opcodes::ArgType::None => 0,
                opcodes::ArgType::U8
                | opcodes::ArgType::ValueType
                | opcodes::ArgType::LocalIndex
                | opcodes::ArgType::AccountIndex => 1,
                opcodes::ArgType::U16 | opcodes::ArgType::U16Fixed => 2,
                opcodes::ArgType::U32
                | opcodes::ArgType::U32Fixed
                | opcodes::ArgType::FunctionIndex => 4,
                opcodes::ArgType::U64 => 8,
                opcodes::ArgType::CallExternal => 4,
                opcodes::ArgType::CallInternal => 3,
                opcodes::ArgType::AccountField => 5,
                opcodes::ArgType::AccountFieldParam => 6,
                opcodes::ArgType::FusedAccAcc => 10,
                opcodes::ArgType::FusedSubAdd => 11,
                opcodes::ArgType::ParamImm => 2,
                opcodes::ArgType::FieldImm => 6,
                opcodes::ArgType::CompareU8Offset16 => 3,
                opcodes::ArgType::CompareU8Target16 => 3,
                opcodes::ArgType::TargetU16 => 2,
                opcodes::ArgType::LocalTarget16 => 3,
            };
        }

        0
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
                | opcodes::CMP_EQ_JUMP
                | opcodes::DEC_JUMP_NZ
                | opcodes::DEC_LOCAL_JUMP_NZ
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
                | opcodes::PUSH_U128
                | opcodes::PUSH_STRING
                | opcodes::PUSH_BYTES
                | opcodes::PUSH_U8_W
                | opcodes::PUSH_U16_W
                | opcodes::PUSH_U32_W
                | opcodes::PUSH_U64_W
                | opcodes::PUSH_I64_W
                | opcodes::PUSH_BOOL_W
                | opcodes::PUSH_PUBKEY_W
                | opcodes::PUSH_U128_W
                | opcodes::PUSH_STRING_W
                | opcodes::PUSH_BYTES_W
                | opcodes::LOAD_FIELD
                | opcodes::ADD
                | opcodes::SUB
                | opcodes::MUL
                | opcodes::MUL_DIV
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
