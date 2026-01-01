use super::opcodes::OpcodePatterns;
use super::types::NameDeduplication;
use super::OpcodeEmitter;
use five_protocol::MAX_U16_ADDRESS;
use five_vm_mito::error::VMError;
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct CallPatch {
    position: usize,
    function_name: String,
}

/// Tracks function CALL emission, metadata, and address patching.
pub struct CallSiteTracker {
    name_deduplication: NameDeduplication,
    patches: Vec<CallPatch>,
    function_positions: HashMap<String, usize>,
}

impl CallSiteTracker {
    pub fn new() -> Self {
        Self {
            name_deduplication: NameDeduplication::new(),
            patches: Vec::new(),
            function_positions: HashMap::new(),
        }
    }

    pub fn reset(&mut self) {
        self.patches.clear();
        self.function_positions.clear();
        self.name_deduplication = NameDeduplication::new();
    }

    pub fn record_patch_at_position(&mut self, position: usize, function_name: String) {
        self.patches.push(CallPatch {
            position,
            function_name,
        });
    }

    pub fn record_function_position(&mut self, function_name: String, position: usize) {
        self.function_positions.insert(function_name, position);
    }

    /// Emit CALL opcode, optionally embedding metadata when the feature flag is enabled.
    /// Returns the number of metadata bytes appended so callers can adjust patch offsets.
    pub fn emit_call_with_metadata<T: OpcodeEmitter>(
        &mut self,
        emitter: &mut T,
        param_count: u8,
        function_address: u16,
        function_name: &str,
    ) -> usize {
        if cfg!(feature = "call-metadata") {
            let current_position = emitter.get_position();
            if self
                .name_deduplication
                .record_name(function_name, current_position)
            {
                OpcodePatterns::emit_call_with_name(
                    emitter,
                    param_count,
                    function_address,
                    function_name,
                );
                1 + function_name.len()
            } else {
                let name_index = self
                    .name_deduplication
                    .get_name_index(function_name)
                    .expect("Function name should exist in deduplication tracker")
                    as u8;
                OpcodePatterns::emit_call_with_name_ref(
                    emitter,
                    param_count,
                    function_address,
                    name_index,
                );
                2
            }
        } else {
            OpcodePatterns::emit_call(emitter, param_count, function_address);
            0
        }
    }

    pub fn patch_calls<T: OpcodeEmitter>(&self, emitter: &mut T) -> Result<(), VMError> {
        for patch in &self.patches {
            let function_pos = self
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
            Self::patch_function_address(emitter, patch.position, *function_pos)?;
        }
        Ok(())
    }

    fn patch_function_address<T: OpcodeEmitter>(
        emitter: &mut T,
        address_pos: usize,
        function_pos: usize,
    ) -> Result<(), VMError> {
        if function_pos > MAX_U16_ADDRESS {
            return Err(VMError::InvalidFunctionIndex);
        }
        if address_pos > MAX_U16_ADDRESS {
            return Err(VMError::InvalidInstructionPointer);
        }
        emitter.patch_u16(address_pos, function_pos as u16);
        Ok(())
    }
}

impl Default for CallSiteTracker {
    fn default() -> Self {
        Self::new()
    }
}
