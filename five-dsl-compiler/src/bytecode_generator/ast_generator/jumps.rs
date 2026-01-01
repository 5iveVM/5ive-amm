//! Jump and patch management
//!
//! This module handles jump instructions, label management, and bytecode patching.

use super::super::opcodes::OpcodePatterns;
use super::super::OpcodeEmitter;
use super::types::{ASTGenerator, FunctionPatch, JumpPatch};
use five_vm_mito::error::VMError;

impl ASTGenerator {
    /// Helper to patch jump offsets in bytecode
    pub(super) fn patch_jump_offset<T: OpcodeEmitter>(
        &self,
        emitter: &mut T,
        offset_pos: usize,
        target: usize,
    ) -> Result<(), VMError> {
        // Validate u16 addressing limits
        if target > five_protocol::MAX_U16_ADDRESS {
            return Err(VMError::InvalidInstructionPointer);
        }
        if offset_pos > five_protocol::MAX_U16_ADDRESS {
            return Err(VMError::InvalidInstructionPointer);
        }

        let offset = target as u16; // Use absolute address
        emitter.patch_u16(offset_pos, offset);
        Ok(())
    }

    /// Patches a function call address with the correct function position
    pub(super) fn patch_function_address<T: OpcodeEmitter>(
        &self,
        emitter: &mut T,
        address_pos: usize,
        function_pos: usize,
    ) -> Result<(), VMError> {
        // Validate u16 addressing limits for function addresses
        if function_pos > five_protocol::MAX_U16_ADDRESS {
            return Err(VMError::InvalidFunctionIndex);
        }
        if address_pos > five_protocol::MAX_U16_ADDRESS {
            return Err(VMError::InvalidInstructionPointer);
        }

        // Function addresses are direct bytecode positions (no offset calculation needed)
        emitter.patch_u16(address_pos, function_pos as u16);
        Ok(())
    }

    /// Helper to patch BR_EQ_U8 VLE offsets in bytecode
    pub(super) fn patch_br_eq_u8_offset<T: OpcodeEmitter>(
        &self,
        emitter: &mut T,
        offset_pos: usize,
        target: usize,
    ) -> Result<(), VMError> {
        // Calculate relative offset from the VLE offset position to target
        // BR_EQ_U8 offset is relative to the current instruction pointer
        let relative_offset = target as i32 - (offset_pos as i32 + 2);

        // Validate that the offset fits in i16 range (VLE u16 with sign interpretation)
        if relative_offset < i16::MIN as i32 || relative_offset > i16::MAX as i32 {
            return Err(VMError::InvalidInstructionPointer);
        }

        // Force 2-byte VLE encoding for the offset to fill the reserved space (emitted as u16(0))
        // Format: 0x80 | (low 7 bits) , (high 7 bits)
        // This ensures check_br_eq_u8 matches patch size exactly.
        let val = relative_offset as u16;
        let byte0 = 0x80 | (val & 0x7F) as u8;
        let byte1 = ((val >> 7) & 0x7F) as u8;
        
        // Patch using u16 (LE) which writes [byte0, byte1]
        let patch_val = (byte1 as u16) << 8 | (byte0 as u16);
        emitter.patch_u16(offset_pos, patch_val);

        Ok(())
    }

    /// Creates a new unique label.
    pub(super) fn new_label(&mut self) -> String {
        let label = format!("L{}", self.label_counter);
        self.label_counter += 1;
        label
    }

    /// Places a label at the current bytecode position.
    pub(super) fn place_label<T: OpcodeEmitter>(&mut self, emitter: &mut T, label: String) {
        self.label_positions.insert(label, emitter.get_position());
    }

    /// Records a function call patch at a specific position.
    pub fn record_function_patch_at_position(&mut self, position: usize, function_name: String) {
        self.function_patches.push(FunctionPatch {
            position,
            function_name: function_name.clone(),
        });
    }

    /// Records the position of a function in the bytecode.
    pub fn record_function_position<T: OpcodeEmitter>(
        &mut self,
        emitter: &mut T,
        function_name: String,
    ) {
        let position = emitter.get_position();
        self.function_positions
            .insert(function_name.clone(), position);
    }

    /// Emits a jump instruction and records it for patching.
    pub(super) fn emit_jump<T: OpcodeEmitter>(
        &mut self,
        emitter: &mut T,
        opcode: u8,
        target_label: String,
    ) {
        emitter.emit_opcode(opcode);
        let patch_position = emitter.get_position();
        emitter.emit_u16(0); // Placeholder offset (u16 for protocol consistency)
        self.jump_patches.push(JumpPatch {
            position: patch_position,
            target_label,
        });
    }

    /// Patches all recorded jumps and function calls with their correct offsets/addresses.
    pub fn patch<T: OpcodeEmitter>(&mut self, emitter: &mut T) -> Result<(), VMError> {
        for patch in &self.jump_patches {
            let target_position = self
                .label_positions
                .get(&patch.target_label)
                .ok_or(VMError::InvalidScript)?; // Should not happen
            self.patch_jump_offset(emitter, patch.position, *target_position)?;
        }

        // Patch BR_EQ_U8 instructions with VLE-encoded relative offsets
        for patch in &self.br_eq_u8_patches {
            let target_position = self
                .label_positions
                .get(&patch.target_label)
                .ok_or(VMError::InvalidScript)?; // Should not happen
            self.patch_br_eq_u8_offset(emitter, patch.position, *target_position)?;
        }

        // Patch function calls with correct addresses
        for patch in &self.function_patches {
            let function_address = self
                .function_positions
                .get(&patch.function_name)
                .ok_or_else(|| {
                    eprintln!(
                        "ERROR: Function '{}' not found for patching. Available functions: {:?}",
                        patch.function_name,
                        self.function_positions.keys().collect::<Vec<_>>()
                    );
                    VMError::InvalidScript
                })?;

            self.patch_function_address(emitter, patch.position, *function_address)?;
        }
        Ok(())
    }

    /// Emit CALL opcode with deduplication-aware function name metadata
    ///
    /// Returns the size of the embedded metadata for position calculation.
    ///
    /// # Errors
    ///
    /// Returns VMError::InvalidScript if the function name is not found in the
    /// deduplication tracker when attempting to reference it.
    #[allow(dead_code)]
    pub(super) fn emit_call_with_deduplication<T: OpcodeEmitter>(
        &mut self,
        emitter: &mut T,
        param_count: u8,
        function_address: u16,
        function_name: &str,
    ) -> Result<usize, VMError> {
        let current_position = emitter.get_position();

        // Check if this is the first occurrence of the function name
        if self
            .name_deduplication
            .record_name(function_name, current_position)
        {
            // First occurrence - emit full name
            OpcodePatterns::emit_call_with_name(
                emitter,
                param_count,
                function_address,
                function_name,
            );
            Ok(1 + function_name.len()) // name_len(1) + name_bytes
        } else {
            // Repeated occurrence - emit name reference
            let name_index = self
                .name_deduplication
                .get_name_index(function_name)
                .ok_or(VMError::InvalidScript)? as u8; // Proper error handling instead of expect()
            OpcodePatterns::emit_call_with_name_ref(
                emitter,
                param_count,
                function_address,
                name_index,
            );
            Ok(2) // marker(1) + index(1)
        }
    }
}
