//! Jump and patch management.

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

    /// Helper to patch BR_EQ_U8 fixed-width offsets in bytecode
    pub(super) fn patch_br_eq_u8_offset<T: OpcodeEmitter>(
        &self,
        emitter: &mut T,
        offset_pos: usize,
        target: usize,
    ) -> Result<(), VMError> {
        // Calculate relative offset from the offset field to target.
        // BR_EQ_U8 offset is relative to the current instruction pointer
        let relative_offset = target as i32 - (offset_pos as i32 + 2);

        // Validate that the offset fits in i16 range.
        if relative_offset < i16::MIN as i32 || relative_offset > i16::MAX as i32 {
            return Err(VMError::InvalidInstructionPointer);
        }

        // Patch using canonical fixed-width little-endian u16 encoding.
        emitter.patch_u16(offset_pos, relative_offset as u16);

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
    ///
    /// Assumes `label_positions` reflect the final bytecode layout produced during generation.
    /// If a later pass mutates instruction sizes, label positions must be recomputed
    /// before calling this method.
    pub fn patch_with_base<T: OpcodeEmitter>(
        &mut self,
        emitter: &mut T,
        base_offset: usize,
    ) -> Result<(), VMError> {
        // We rely on label_positions computed at emission time.
        // If a post-pass changes instruction sizes, the caller must recalculate
        // label_positions before patching (see recalculate_label_positions stub).

        for patch in &self.jump_patches {
            let target_position = self
                .label_positions
                .get(&patch.target_label)
                .ok_or(VMError::InvalidScript)?; // Should not happen

            #[cfg(debug_assertions)]
            eprintln!(
                "JUMP_PATCH: patch_pos=0x{:04X} target_label={} target_pos=0x{:04X}",
                patch.position, patch.target_label, *target_position
            );

            self.patch_jump_offset(emitter, patch.position, *target_position + base_offset)?;
        }

        // Patch BR_EQ_U8 instructions with fixed-width relative offsets.
        for patch in &self.br_eq_u8_patches {
            let target_position = self
                .label_positions
                .get(&patch.target_label)
                .ok_or(VMError::InvalidScript)?; // Should not happen
            self.patch_br_eq_u8_offset(emitter, patch.position, *target_position)?;
        }

        // Patch function calls with correct addresses
        for patch in &self.function_patches {
            let function_address = if let Some(addr) =
                self.function_positions.get(&patch.function_name)
            {
                *addr
            } else {
                let mut candidate_suffixes = Vec::new();
                candidate_suffixes.push(patch.function_name.clone());
                if let Some((_, tail)) = patch.function_name.split_once("::") {
                    candidate_suffixes.push(tail.to_string());
                }
                let parts: Vec<&str> = patch.function_name.split("::").collect();
                if parts.len() >= 2 {
                    candidate_suffixes.push(format!(
                        "{}::{}",
                        parts[parts.len() - 2],
                        parts[parts.len() - 1]
                    ));
                    candidate_suffixes.push(parts[parts.len() - 1].to_string());
                }
                candidate_suffixes.sort();
                candidate_suffixes.dedup();

                let mut candidates: Vec<(&String, &usize)> = self
                    .function_positions
                    .iter()
                    .filter(|(name, _)| {
                        candidate_suffixes.iter().any(|candidate| {
                            *name == candidate || name.ends_with(&format!("::{}", candidate))
                        })
                    })
                    .collect();
                candidates.sort_by(|a, b| a.0.cmp(b.0));

                if candidates.len() == 1 {
                    *candidates[0].1
                } else {
                    eprintln!(
                        "ERROR: Function '{}' not found or ambiguous for patching. Candidates: {:?}",
                        patch.function_name,
                        candidates.iter().map(|(k, _)| (*k).clone()).collect::<Vec<_>>()
                    );
                    return Err(VMError::InvalidScript);
                }
            };

            self.patch_function_address(emitter, patch.position, function_address + base_offset)?;
        }
        Ok(())
    }

    /// Recalculates label positions based on actual bytecode structure.
    ///
    /// This is currently a no-op placeholder. If we add post-generation transforms that
    /// change instruction sizes, this method should rescan the bytecode and refresh
    /// `label_positions` before patching.
    pub fn recalculate_label_positions<T: OpcodeEmitter>(
        &mut self,
        _emitter: &mut T,
    ) -> Result<(), VMError> {
        // Full implementation would:
        // 1. Scan bytecode for all instructions and their canonical operand widths.
        // 2. Build an offset map for actual instruction positions.
        // 3. Update all label_positions entries to match real bytecode structure.
        // 4. Verify all jump_patches reference valid labels.
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
