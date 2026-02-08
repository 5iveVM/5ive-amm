//! Bytecode builder for tests and deployment.

use crate::opcodes::*;
use alloc::vec::Vec;

/// Macro to simplify bytecode builder usage.
#[macro_export]
macro_rules! bytecode {
    ($($method:tt($($arg:tt)*)),* $(,)?) => {{
        let mut _b = $crate::BytecodeBuilder::new();
        $(_b.$method($($arg)*);)*
        _b.build()
    }};
}

/// A lightweight bytecode builder for test and deployment bytecode generation.
#[derive(Debug, Clone)]
pub struct BytecodeBuilder {
    bytecode: Vec<u8>,
}

impl BytecodeBuilder {
    /// Create a new bytecode builder.
    #[inline]
    pub fn new() -> Self {
        Self {
            bytecode: Vec::new(),
        }
    }

    /// Create with pre-allocated capacity.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            bytecode: Vec::with_capacity(capacity),
        }
    }

    /// Emit magic header bytes ("5IVE").
    #[inline]
    pub fn emit_magic(&mut self) -> &mut Self {
        self.bytecode.extend_from_slice(b"5IVE");
        self
    }

    /// Emit optimized header (V3 format).
    #[inline]
    pub fn emit_header(&mut self, public_count: u8, total_count: u8) -> &mut Self {
        self.emit_magic();
        self.emit_u32(0u32); // features as little-endian u32
        self.bytecode.push(public_count);
        self.bytecode.push(total_count);
        self
    }

    /// Emit a single opcode byte.
    #[inline]
    pub fn emit_u8(&mut self, byte: u8) -> &mut Self {
        self.bytecode.push(byte);
        self
    }

    /// Emit an opcode (alias for emit_u8).
    #[inline]
    pub fn emit_opcode(&mut self, opcode: u8) -> &mut Self {
        self.emit_u8(opcode)
    }

    /// Emit little-endian u16.
    #[inline]
    pub fn emit_u16(&mut self, value: u16) -> &mut Self {
        self.bytecode.extend_from_slice(&value.to_le_bytes());
        self
    }

    /// Emit little-endian u32.
    #[inline]
    pub fn emit_u32(&mut self, value: u32) -> &mut Self {
        self.bytecode.extend_from_slice(&value.to_le_bytes());
        self
    }

    /// Emit little-endian u64.
    #[inline]
    pub fn emit_u64(&mut self, value: u64) -> &mut Self {
        self.bytecode.extend_from_slice(&value.to_le_bytes());
        self
    }

    /// Emit raw bytes.
    #[inline]
    pub fn emit_bytes(&mut self, bytes: &[u8]) -> &mut Self {
        self.bytecode.extend_from_slice(bytes);
        self
    }

    /// Emit common opcodes with immediate values.
    #[inline]
    pub fn emit_push_u64(&mut self, value: u64) -> &mut Self {
        self.emit_u8(PUSH_U64).emit_u64(value)
    }

    /// PUSH_U32 value (little-endian).
    #[inline]
    pub fn emit_push_u32(&mut self, value: u32) -> &mut Self {
        self.emit_u8(PUSH_U32).emit_u32(value)
    }

    /// PUSH_BOOL value.
    #[inline]
    pub fn emit_push_bool(&mut self, value: bool) -> &mut Self {
        self.emit_u8(PUSH_BOOL).emit_u8(if value { 1 } else { 0 })
    }

    /// CALL with func_addr (u16) and param_count (u8)
    #[inline]
    pub fn emit_call(&mut self, func_addr: u16, param_count: u8) -> &mut Self {
        self.emit_u8(CALL).emit_u8(param_count).emit_u16(func_addr)
    }

    /// LOAD_PARAM param_index
    #[inline]
    pub fn emit_load_param(&mut self, param_index: u8) -> &mut Self {
        self.emit_u8(LOAD_PARAM).emit_u8(param_index)
    }

    /// STORE_PARAM param_index
    #[inline]
    pub fn emit_store_param(&mut self, param_index: u8) -> &mut Self {
        self.emit_u8(STORE_PARAM).emit_u8(param_index)
    }

    /// SET_LOCAL local_index
    #[inline]
    pub fn emit_set_local(&mut self, local_index: u8) -> &mut Self {
        self.emit_u8(SET_LOCAL).emit_u8(local_index)
    }

    /// GET_LOCAL local_index
    #[inline]
    pub fn emit_get_local(&mut self, local_index: u8) -> &mut Self {
        self.emit_u8(GET_LOCAL).emit_u8(local_index)
    }

    /// HALT instruction (terminates execution)
    #[inline]
    pub fn emit_halt(&mut self) -> &mut Self {
        self.emit_u8(HALT)
    }

    /// Get the current position in the bytecode (for patching jumps)
    #[inline]
    pub fn position(&self) -> usize {
        self.bytecode.len()
    }

    /// Patch a u16 value at the given position (for jump addresses)
    pub fn patch_u16(&mut self, position: usize, value: u16) -> Result<(), &'static str> {
        if position + 2 > self.bytecode.len() {
            return Err("patch position out of bounds");
        }
        let bytes = value.to_le_bytes();
        self.bytecode[position] = bytes[0];
        self.bytecode[position + 1] = bytes[1];
        Ok(())
    }

    /// Patch a u8 value at the given position (for small immediates / flags)
    pub fn patch_u8(&mut self, position: usize, value: u8) -> Result<(), &'static str> {
        if position >= self.bytecode.len() {
            return Err("patch position out of bounds");
        }
        self.bytecode[position] = value;
        Ok(())
    }

    /// Patch a u32 value at the given position
    pub fn patch_u32(&mut self, position: usize, value: u32) -> Result<(), &'static str> {
        if position + 4 > self.bytecode.len() {
            return Err("patch position out of bounds");
        }
        let bytes = value.to_le_bytes();
        self.bytecode[position] = bytes[0];
        self.bytecode[position + 1] = bytes[1];
        self.bytecode[position + 2] = bytes[2];
        self.bytecode[position + 3] = bytes[3];
        Ok(())
    }

    /// Get the finalized bytecode
    #[inline]
    pub fn build(self) -> Vec<u8> {
        self.bytecode
    }

    /// Get a reference to the current bytecode (for testing without consuming)
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytecode
    }

    /// Get the length of the bytecode
    #[inline]
    pub fn len(&self) -> usize {
        self.bytecode.len()
    }

    /// Check if bytecode is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.bytecode.is_empty()
    }
}

impl Default for BytecodeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_bytecode_generation() {
        let bytecode = {
            let mut b = BytecodeBuilder::new();
            b.emit_magic();
            b.emit_u8(0);
            b.emit_u8(1);
            b.emit_u8(2);
            b.emit_halt();
            b.build()
        };

        assert_eq!(&bytecode[0..4], b"5IVE");
        assert_eq!(bytecode[4], 0);
        assert_eq!(bytecode[5], 1);
        assert_eq!(bytecode[6], 2);
        assert_eq!(bytecode[7], HALT);
    }

    #[test]
    fn test_header_emission() {
        let bytecode = {
            let mut b = BytecodeBuilder::new();
            b.emit_header(1, 2);
            b.emit_halt();
            b.build()
        };

        assert_eq!(&bytecode[0..4], b"5IVE");
        // features is now a u32 at [4..8]
        assert_eq!(
            u32::from_le_bytes([bytecode[4], bytecode[5], bytecode[6], bytecode[7]]),
            0
        );
        assert_eq!(bytecode[8], 1); // public_count
        assert_eq!(bytecode[9], 2); // total_count
        assert_eq!(bytecode[10], HALT);
    }

    #[test]
    fn test_u64_emission() {
        let value: u64 = 0x0123456789ABCDEF;
        let bytecode = {
            let mut b = BytecodeBuilder::new();
            b.emit_u64(value);
            b.build()
        };

        assert_eq!(bytecode.len(), 8);
        assert_eq!(
            u64::from_le_bytes([
                bytecode[0],
                bytecode[1],
                bytecode[2],
                bytecode[3],
                bytecode[4],
                bytecode[5],
                bytecode[6],
                bytecode[7],
            ]),
            value
        );
    }

    #[test]
    fn test_push_u64_instruction() {
        let bytecode = {
            let mut b = BytecodeBuilder::new();
            b.emit_push_u64(42);
            b.build()
        };

        assert_eq!(bytecode[0], PUSH_U64);
        assert_eq!(
            u64::from_le_bytes([
                bytecode[1],
                bytecode[2],
                bytecode[3],
                bytecode[4],
                bytecode[5],
                bytecode[6],
                bytecode[7],
                bytecode[8],
            ]),
            42
        );
    }

    #[test]
    fn test_chaining() {
        let bytecode = {
            let mut b = BytecodeBuilder::new();
            b.emit_magic();
            b.emit_u8(0);
            b.emit_u8(1);
            b.emit_u8(2);
            b.emit_load_param(0);
            b.emit_set_local(0);
            b.emit_get_local(0);
            b.emit_halt();
            b.build()
        };

        assert_eq!(bytecode[0..4], *b"5IVE");
        assert_eq!(bytecode[4], 0); // First u8
        assert_eq!(bytecode[5], 1); // Second u8
        assert_eq!(bytecode[6], 2); // Third u8
        assert_eq!(bytecode[7], LOAD_PARAM); // LOAD_PARAM opcode
        assert_eq!(bytecode[8], 0); // LOAD_PARAM parameter
        assert_eq!(bytecode[9], SET_LOCAL); // SET_LOCAL opcode
    }

    #[test]
    fn test_patching() {
        let mut builder = BytecodeBuilder::new();
        builder.emit_u32(0); // placeholder at position 0
        let _pos = builder.position();
        builder.emit_halt();

        builder.patch_u32(0, 0x12345678).unwrap();
        let bytecode = builder.build();

        assert_eq!(
            u32::from_le_bytes([bytecode[0], bytecode[1], bytecode[2], bytecode[3]]),
            0x12345678
        );
        assert_eq!(bytecode[4], HALT);
    }
}
