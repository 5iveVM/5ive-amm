//! Ultra-lightweight execution context for MitoVM
//!
//! Single unified context replacing all previous abstraction layers.
//! Designed for maximum performance with zero indirection.

use crate::{
    error::{CompactResult, Result, VMError, VMErrorCode},
    error_log,
    metadata::ImportMetadata,
    stack::StackStorage,
    types::CallFrame,
    MAX_CALL_DEPTH, MAX_LOCALS, MAX_PARAMETERS, MAX_SCRIPT_SIZE, STACK_SIZE,
};

#[cfg(feature = "debug-logs")]
use crate::debug_log;
use five_protocol::ValueRef;
use heapless::Vec;
use pinocchio::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction, Seed, Signer},
    program::{invoke, invoke_signed},
    pubkey::Pubkey,
};

// System program ID constant
const SYSTEM_PROGRAM_ID: [u8; 32] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
]; // Solana system program ID: 11111111111111111111111111111111

// Shared parameter storage (only one copy, indexed by call frames)
const SHARED_PARAM_SIZE: usize = MAX_PARAMETERS + 1;

/// Single unified execution context for maximum performance
/// Temp buffer is stack-based to keep the entire context on the stack
/// while remaining within Solana's 4KB BPF stack limit.
/// Replaces: ValueRefStack, ExecutionContext, CoreExecutionContext,
/// MemoryContext, CallContext, ExternalContext, ExecutionManager
pub struct ExecutionContext<'a> {
    // --- Core execution state ---
    pub bytecode: &'a [u8],
    pub pc: u16,
    // --- Unified stack storage ---
    pub storage: &'a mut StackStorage<'a>,
    pub sp: u8,
    pub temp_pos: usize,
    pub csp: u8,

    // --- Function metadata (optimized header V2) ---
    pub public_function_count: u8, // For external dispatch validation
    pub total_function_count: u8,  // For internal CALL validation
    pub header_features: u32,      // Raw header feature flags

    // --- External Solana state ---
    pub accounts: &'a [AccountInfo],
    pub program_id: Pubkey,
    pub instruction_data: &'a [u8],

    // --- Shared parameter storage (single copy) ---
    pub parameters: [ValueRef; SHARED_PARAM_SIZE],
    pub param_start: u8,
    pub param_len: u8,

    // --- Execution state ---
    pub halted: bool,
    pub return_value: Option<ValueRef>,

    // --- Compute tracking (minimal for ultra-lightweight VM) ---
    pub compute_units_consumed: u64,

    // --- Input data processing ---
    pub input_ptr: u8,

    // --- Local variable tracking ---
    pub local_count: u8,
    pub local_base: u8, // Base offset in locals array for current frame

    // --- Lazy account validation ---
    pub lazy_validator: crate::lazy_validation::LazyAccountValidator,

    // --- Import verification metadata ---
    pub import_metadata: ImportMetadata<'a>,
}

// Helper trait for little-endian byte conversion without heap allocations
trait FromLeBytes<const N: usize>: Sized {
    fn from_le_bytes(bytes: [u8; N]) -> Self;
}

impl FromLeBytes<2> for u16 {
    #[inline(always)]
    fn from_le_bytes(bytes: [u8; 2]) -> Self {
        u16::from_le_bytes(bytes)
    }
}

impl FromLeBytes<4> for u32 {
    #[inline(always)]
    fn from_le_bytes(bytes: [u8; 4]) -> Self {
        u32::from_le_bytes(bytes)
    }
}

impl FromLeBytes<8> for u64 {
    #[inline(always)]
    fn from_le_bytes(bytes: [u8; 8]) -> Self {
        u64::from_le_bytes(bytes)
    }
}

impl FromLeBytes<16> for u128 {
    #[inline(always)]
    fn from_le_bytes(bytes: [u8; 16]) -> Self {
        u128::from_le_bytes(bytes)
    }
}

impl<'a> ExecutionContext<'a> {
    /// Create new execution context with OptimizedHeader V2
    #[inline]
    pub fn new(
        bytecode: &'a [u8],
        accounts: &'a [AccountInfo],
        program_id: Pubkey,
        instruction_data: &'a [u8],
        start_pc: u16,
        storage: &'a mut StackStorage<'a>,
        public_function_count: u8,
        total_function_count: u8,
    ) -> Self {
        Self {
            bytecode,
            pc: start_pc,
            storage,
            sp: 0,
            temp_pos: 0,
            csp: 0,
            public_function_count,
            total_function_count,
            header_features: 0,
            accounts,
            program_id,
            instruction_data,
            parameters: [ValueRef::Empty; SHARED_PARAM_SIZE],
            param_start: 0,
            param_len: 0,
            halted: false,
            return_value: None,
            compute_units_consumed: 0,
            input_ptr: 0,
            local_count: 0,
            local_base: 0,
            lazy_validator: crate::lazy_validation::LazyAccountValidator::new(accounts.len()),
            import_metadata: ImportMetadata::new(bytecode, bytecode.len()).unwrap_or_else(|_| {
                // If parsing fails, create empty metadata (backward compatible)
                ImportMetadata::new(&[], 0).unwrap()
            }),
        }
    }

    // --- Stack operations (zero indirection) ---

    #[inline(always)]
    pub fn push(&mut self, value: ValueRef) -> CompactResult<()> {
        if self.sp as usize >= STACK_SIZE {
            return Err(VMErrorCode::StackOverflow);
        }
        self.storage.stack[self.sp as usize] = value;
        self.sp += 1;
        Ok(())
    }

    #[inline(always)]
    pub fn pop(&mut self) -> CompactResult<ValueRef> {
        if self.sp == 0 {
            return Err(VMErrorCode::StackUnderflow);
        }
        self.sp -= 1;
        Ok(self.storage.stack[self.sp as usize])
    }

    #[inline(always)]
    pub fn peek(&self) -> CompactResult<ValueRef> {
        if self.sp == 0 {
            return Err(VMErrorCode::StackUnderflow);
        }
        Ok(self.storage.stack[self.sp as usize - 1])
    }

    // --- Bytecode fetching ---

    #[inline]
    pub fn fetch_byte(&mut self) -> CompactResult<u8> {
        if self.pc as usize >= self.bytecode.len() {
            return Err(VMErrorCode::InvalidInstructionPointer);
        }
        let byte = self.bytecode[self.pc as usize];
        self.pc = self.pc.saturating_add(1);
        Ok(byte)
    }

    #[inline(always)]
    fn fetch_le<T, const N: usize>(&mut self) -> CompactResult<T>
    where
        T: FromLeBytes<N>,
    {
        let start = self.pc as usize;
        let end = start + N;
        if end > self.bytecode.len() {
            return Err(VMErrorCode::InvalidInstructionPointer);
        }
        self.pc = end as u16;
        let bytes: [u8; N] = unsafe {
            core::ptr::read_unaligned(self.bytecode.as_ptr().add(start) as *const [u8; N])
        };
        Ok(T::from_le_bytes(bytes))
    }

    #[inline]
    pub fn fetch_u16(&mut self) -> CompactResult<u16> {
        self.fetch_le::<u16, 2>()
    }

    #[inline]
    pub fn fetch_u64(&mut self) -> CompactResult<u64> {
        self.fetch_le::<u64, 8>()
    }

    /// Fetch u128 from bytecode - MITO-style direct access, zero-copy
    #[inline]
    pub fn fetch_u128(&mut self) -> CompactResult<u128> {
        self.fetch_le::<u128, 16>()
    }

    #[inline(always)]
    pub fn script(&self) -> &[u8] {
        self.bytecode
    }

    #[inline(always)]
    pub fn set_script(&mut self, bytecode: &'a [u8]) {
        self.bytecode = bytecode;
    }

    #[inline(always)]
    pub fn ip(&self) -> usize {
        self.pc as usize
    }

    #[inline(always)]
    pub fn set_ip(&mut self, ip: usize) {
        self.pc = ip as u16;
    }

    /// Get public function count for external dispatch validation
    #[inline(always)]
    pub fn public_function_count(&self) -> u8 {
        self.public_function_count
    }

    /// Get total function count for internal CALL validation
    #[inline(always)]
    pub fn total_function_count(&self) -> u8 {
        self.total_function_count
    }

    /// Get raw header feature flags for metadata detection
    #[inline(always)]
    pub fn header_features(&self) -> u32 {
        self.header_features
    }

    /// Update header feature flags when new script is loaded
    #[inline(always)]
    pub fn set_header_features(&mut self, features: u32) {
        self.header_features = features;
    }

    // --- Memory operations ---

    #[inline(always)]
    pub fn alloc_temp(&mut self, size: u8) -> CompactResult<u8> {
        if self.temp_pos + size as usize > self.storage.temp_buffer.len() {
            return Err(VMErrorCode::MemoryError);
        }
        let offset = self.temp_pos;
        self.temp_pos += size as usize;
        Ok(offset as u8)
    }

    #[inline(always)]
    pub fn get_temp_data(&self, offset: u8, size: u8) -> CompactResult<&[u8]> {
        let start = offset as usize;
        let end = start + size as usize;
        if end > self.storage.temp_buffer.len() {
            return Err(VMErrorCode::MemoryError);
        }
        Ok(&self.storage.temp_buffer[start..end])
    }

    #[inline(always)]
    pub fn get_temp_data_mut(&mut self, offset: u8, size: u8) -> CompactResult<&mut [u8]> {
        let start = offset as usize;
        let end = start + size as usize;
        if end > self.storage.temp_buffer.len() {
            return Err(VMErrorCode::MemoryError);
        }
        Ok(&mut self.storage.temp_buffer[start..end])
    }

    #[inline(always)]
    pub fn temp_buffer(&self) -> &[u8] {
        &self.storage.temp_buffer[..]
    }

    #[inline(always)]
    pub fn temp_buffer_mut(&mut self) -> &mut [u8] {
        &mut self.storage.temp_buffer[..]
    }

    /// Allocate a temp buffer slot for Option/Result storage
    /// Returns offset in temp buffer (advances temp_pos)
    #[inline(always)]
    pub fn allocate_temp_slot(&mut self) -> CompactResult<u8> {
        // Each slot is 16 bytes for ValueRef storage (+1 byte tag if Option/Result)
        // Simplified for now: just allocate 17 bytes per slot
        let slot_size = 17u8;
        if self.temp_pos + slot_size as usize > self.storage.temp_buffer.len() {
            return Err(VMErrorCode::MemoryError);
        }
        let offset = self.temp_pos as u8;
        self.temp_pos += slot_size as usize;
        Ok(offset)
    }

    /// Get mutable reference to temp buffer as fixed-size array for ValueAccessContext
    #[inline]
    pub fn temp_buffer_64_mut(&mut self) -> Result<&mut [u8; crate::TEMP_BUFFER_SIZE]> {
        if crate::TEMP_BUFFER_SIZE != 64 {
            return Err(VMError::MemoryViolation);
        }
        Ok(&mut self.storage.temp_buffer)
    }

    /// Write a [`ValueRef`] into the temp buffer, encoding the full type tag and
    /// byte representation. Returns the offset where the value was written.
    #[inline]
    pub fn write_value_to_temp(&mut self, value: &ValueRef) -> Result<u16> {
        let size = value.serialized_size();

        if self.temp_pos + size > crate::TEMP_BUFFER_SIZE {
            return Err(VMError::MemoryError);
        }

        let offset = self.temp_pos;
        value
            .serialize_into(&mut self.storage.temp_buffer[offset..offset + size])
            .map_err(|_| VMError::ProtocolError)?;
        self.temp_pos += size;
        Ok(offset as u16)
    }

    /// Deserialize a [`ValueRef`] previously written with
    /// [`write_value_to_temp`].
    #[inline]
    pub fn read_value_from_temp(&self, offset: u16) -> Result<ValueRef> {
        if offset as usize >= self.storage.temp_buffer.len() {
            return Err(VMError::MemoryError);
        }

        ValueRef::deserialize_from(&self.storage.temp_buffer[offset as usize..])
            .map_err(|_| VMError::ProtocolError)
    }

    // --- Call stack operations ---

    #[inline(always)]
    pub fn push_call_frame(&mut self, frame: CallFrame<'a>) -> Result<()> {
        if self.csp as usize >= MAX_CALL_DEPTH {
            return Err(VMError::CallStackOverflow);
        }
        debug_assert!(
            (self.csp as usize) < self.storage.call_stack.len(),
            "CallFrame push index out of bounds: {} >= {}",
            self.csp,
            self.storage.call_stack.len()
        );
        self.storage.call_stack[self.csp as usize] = frame;
        self.csp += 1;
        Ok(())
    }

    #[inline(always)]
    pub fn pop_call_frame(&mut self) -> CompactResult<CallFrame<'a>> {
        if self.csp == 0 {
            return Err(VMErrorCode::CallStackUnderflow);
        }
        self.csp -= 1;
        debug_assert!(
            (self.csp as usize) < self.storage.call_stack.len(),
            "CallFrame pop index out of bounds: {} >= {}",
            self.csp,
            self.storage.call_stack.len()
        );
        Ok(self.storage.call_stack[self.csp as usize])
    }

    // --- Local variables ---

    #[inline]
    pub fn get_local(&self, index: u8) -> CompactResult<ValueRef> {
        if index as usize >= self.local_count as usize {
            #[cfg(feature = "debug-logs")]
            debug_log!("LOCAL_DEBUG: get_local index out of bounds: {} >= {}", index, self.local_count);
            return Err(VMErrorCode::LocalsOverflow);
        }
        Ok(self.storage.locals[self.local_base as usize + index as usize])
    }

    #[inline(always)]
    pub fn set_local(&mut self, index: u8, value: ValueRef) -> CompactResult<()> {
        if index as usize >= self.local_count as usize {
            #[cfg(feature = "debug-logs")]
            debug_log!("LOCAL_DEBUG: set_local index out of bounds: {} >= {}", index, self.local_count);
            return Err(VMErrorCode::LocalsOverflow);
        }
        self.storage.locals[self.local_base as usize + index as usize] = value;
        Ok(())
    }

    #[inline(always)]
    pub fn clear_local(&mut self, index: u8) -> CompactResult<()> {
        if index >= self.local_count {
            return Err(VMErrorCode::LocalsOverflow);
        }

        // Apply base offset for per-frame local isolation
        let absolute_index = (self.local_base + index) as usize;
        if absolute_index >= self.storage.locals.len() {
            return Err(VMErrorCode::LocalsOverflow);
        }
        debug_assert!(
            absolute_index < self.storage.locals.len(),
            "Local absolute index {} must be < locals.len() {} (base={}, index={})",
            absolute_index,
            self.storage.locals.len(),
            self.local_base,
            index
        );

        self.storage.locals[absolute_index] = ValueRef::Empty;

        // Shrink local_count if we cleared the last local
        if index + 1 == self.local_count {
            while self.local_count > 0 {
                let pos = (self.local_base + self.local_count - 1) as usize;
                debug_assert!(
                    pos < self.storage.locals.len(),
                    "Position {} must be < locals.len() {}",
                    pos,
                    self.storage.locals.len()
                );
                if pos < self.storage.locals.len() && !self.storage.locals[pos].is_empty() {
                    break;
                }
                self.local_count -= 1;
            }
        }

        Ok(())
    }

    // --- Registers ---

    #[inline(always)]
    pub fn get_register(&self, index: u8) -> CompactResult<ValueRef> {
        if index >= 8 {
            return Err(VMErrorCode::InvalidRegister);
        }
        debug_assert!(
            (index as usize) < self.storage.registers.len(),
            "Register index {} must be < registers.len() {}",
            index,
            self.storage.registers.len()
        );
        Ok(self.storage.registers[index as usize])
    }

    #[inline(always)]
    pub fn set_register(&mut self, index: u8, value: ValueRef) -> CompactResult<()> {
        if index >= 8 {
            return Err(VMErrorCode::InvalidRegister);
        }
        debug_assert!(
            (index as usize) < self.storage.registers.len(),
            "Register index {} must be < registers.len() {}",
            index,
            self.storage.registers.len()
        );
        self.storage.registers[index as usize] = value;
        Ok(())
    }

    // --- Account operations with lazy validation ---

    #[inline(always)]
    pub fn get_account(&self, index: u8) -> CompactResult<&AccountInfo> {
        // Validate account lazily on first access
        self.lazy_validator.ensure_validated(index, self.accounts)?;

        if index as usize >= self.accounts.len() {
            return Err(VMErrorCode::InvalidAccountIndex);
        }
        debug_assert!(
            (index as usize) < self.accounts.len(),
            "Account index {} must be < accounts.len() {}",
            index,
            self.accounts.len()
        );
        Ok(&self.accounts[index as usize])
    }

    /// Get account without lazy validation (for internal VM use)
    #[inline(always)]
    pub fn get_account_unchecked(&self, index: u8) -> CompactResult<&AccountInfo> {
        if index as usize >= self.accounts.len() {
            return Err(VMErrorCode::InvalidAccountIndex);
        }
        debug_assert!(
            (index as usize) < self.accounts.len(),
            "Account index {} must be < accounts.len() {}",
            index,
            self.accounts.len()
        );
        Ok(&self.accounts[index as usize])
    }

    // --- Parameter operations ---

    #[inline(always)]
    pub fn set_parameters(&mut self, params: [ValueRef; 8]) {
        self.parameters[..8].copy_from_slice(&params);
        self.param_start = 0;
        self.param_len = MAX_PARAMETERS as u8;
    }

    #[inline(always)]
    pub fn parameters(&self) -> &[ValueRef] {
        &self.parameters[..]
    }

    // --- Stack utility methods ---

    #[inline(always)]
    pub fn size(&self) -> usize {
        self.sp as usize
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.sp == 0
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.sp as usize
    }

    // --- Execution state methods ---

    #[inline(always)]
    pub fn halted(&self) -> bool {
        self.halted
    }

    #[inline(always)]
    pub fn set_halted(&mut self, halted: bool) {
        self.halted = halted;
    }

    #[inline(always)]
    pub fn return_value(&self) -> Option<ValueRef> {
        self.return_value
    }

    #[inline(always)]
    pub fn set_return_value(&mut self, value: Option<ValueRef>) {
        self.return_value = value;
    }

    // --- Compute unit tracking ---

    #[inline(always)]
    pub fn consume_compute_units(&mut self, units: u64) {
        self.compute_units_consumed = self.compute_units_consumed.saturating_add(units);
    }

    #[inline(always)]
    pub fn compute_units_consumed(&self) -> u64 {
        self.compute_units_consumed
    }

    // === PHASE 1: CRITICAL MISSING METHODS ===

    // --- Call stack management (12 occurrences) ---

    #[inline(always)]
    pub fn call_depth(&self) -> usize {
        self.csp as usize
    }

    #[inline(always)]
    pub fn set_call_depth(&mut self, depth: u8) -> CompactResult<()> {
        if depth as usize >= MAX_CALL_DEPTH {
            return Err(VMErrorCode::CallStackOverflow);
        }
        self.csp = depth;
        Ok(())
    }

    #[inline(always)]
    pub fn get_call_frame(&self, index: usize) -> CompactResult<&CallFrame<'a>> {
        if index < self.csp as usize {
            debug_assert!(
                index < self.storage.call_stack.len(),
                "CallFrame get index out of bounds: {} >= {}",
                index,
                self.storage.call_stack.len()
            );
            Ok(&self.storage.call_stack[index])
        } else {
            Err(VMErrorCode::InvalidOperation)
        }
    }

    #[inline(always)]
    pub fn set_call_frame(&mut self, index: usize, frame: CallFrame<'a>) -> CompactResult<()> {
        if index < self.csp as usize {
            debug_assert!(
                index < self.storage.call_stack.len(),
                "CallFrame set index out of bounds: {} >= {}",
                index,
                self.storage.call_stack.len()
            );
            self.storage.call_stack[index] = frame;
            Ok(())
        } else {
            Err(VMErrorCode::InvalidOperation)
        }
    }

    // --- Data access methods (17 occurrences) ---

    #[inline(always)]
    pub fn instruction_data(&self) -> &[u8] {
        self.instruction_data
    }

    #[inline(always)]
    pub fn accounts(&self) -> &[AccountInfo] {
        self.accounts
    }

    // --- Local variable management (6 occurrences) ---

    #[inline(always)]
    pub fn local_count(&self) -> u8 {
        self.local_count
    }

    #[inline(always)]
    pub fn set_local_count(&mut self, count: u8) {
        self.local_count = count;
    }

    #[inline(always)]
    pub fn local_base(&self) -> u8 {
        self.local_base
    }

    #[inline(always)]
    pub fn set_local_base(&mut self, base: u8) {
        self.local_base = base;
    }

    #[inline(always)]
    pub fn allocate_locals(&mut self, count: u8) -> CompactResult<()> {
        // Allocate locals in current frame's window (base_offset + count)
        if (self.local_base as usize + count as usize) > MAX_LOCALS {
            return Err(VMErrorCode::LocalsOverflow);
        }

        let start = self.local_base as usize;
        let end = (self.local_base + count) as usize;
        let max_len = self.storage.locals.len();

        for slot in self.storage.locals[start..end.min(max_len)].iter_mut() {
            *slot = ValueRef::Empty;
        }
        self.local_count = count;
        Ok(())
    }

    #[inline(always)]
    pub fn deallocate_locals(&mut self) {
        // Clear only this frame's locals (base_offset to base_offset + local_count)
        let start = self.local_base as usize;
        let end = (self.local_base + self.local_count) as usize;
        let max_len = self.storage.locals.len();

        for slot in self.storage.locals[start..end.min(max_len)].iter_mut() {
            *slot = ValueRef::Empty;
        }
        self.local_count = 0;
    }

    // --- Stack operations with zero indirection (3 occurrences) ---

    #[inline(always)]
    pub fn dup(&mut self) -> CompactResult<()> {
        let value = self.peek()?;
        self.push(value)
    }

    #[inline(always)]
    pub fn swap(&mut self) -> CompactResult<()> {
        if self.sp < 2 {
            return Err(VMErrorCode::StackUnderflow);
        }
        debug_assert!(self.sp >= 2, "Stack pointer must be >= 2 for swap");
        let idx = self.sp as usize;
        debug_assert!(
            idx - 1 < STACK_SIZE && idx - 2 < STACK_SIZE,
            "Swap indices {} and {} must be < STACK_SIZE {}",
            idx - 1,
            idx - 2,
            STACK_SIZE
        );
        self.storage.stack.swap(idx - 1, idx - 2);
        Ok(())
    }

    #[inline(always)]
    pub fn pick(&mut self, depth: u8) -> CompactResult<()> {
        if depth >= self.sp {
            return Err(VMErrorCode::StackUnderflow);
        }
        debug_assert!(depth < self.sp, "Depth {} must be < sp {}", depth, self.sp);
        let idx = self.sp as usize - 1 - depth as usize;
        debug_assert!(
            idx < STACK_SIZE,
            "Pick index {} must be < STACK_SIZE {}",
            idx,
            STACK_SIZE
        );
        let value = self.storage.stack[idx];
        self.push(value)
    }

    // --- Parameter management (4 occurrences) ---

    #[inline(always)]
    pub fn param_start(&self) -> u8 {
        self.param_start
    }

    #[inline(always)]
    pub fn param_len(&self) -> u8 {
        self.param_len
    }

    #[inline(always)]
    pub fn allocate_params(&mut self, count: u8) -> CompactResult<()> {
        // With shared parameter storage, we just clear and set count
        for slot in self.parameters.iter_mut() {
            *slot = ValueRef::Empty;
        }
        self.param_start = 0;
        self.param_len = count;
        Ok(())
    }

    #[inline(always)]
    pub fn restore_parameters(&mut self, start: u8, len: u8) {
        self.param_start = start;
        self.param_len = len;
    }

    #[inline(always)]
    pub fn parameters_mut(&mut self) -> &mut [ValueRef] {
        &mut self.parameters[..]
    }

    #[inline(always)]
    pub fn set_parameter(&mut self, index: usize, value: ValueRef) -> CompactResult<()> {
        if index < self.parameters.len() {
            debug_assert!(
                index < self.parameters.len(),
                "Parameter index {} must be < parameters.len() {}",
                index,
                self.parameters.len()
            );
            self.parameters[index] = value;
            Ok(())
        } else {
            Err(VMErrorCode::InvalidParameter)
        }
    }

    // --- Bytecode fetching extensions (8 occurrences) ---

    #[inline(always)]
    pub fn fetch_u32(&mut self) -> CompactResult<u32> {
        self.fetch_le::<u32, 4>()
    }

    #[inline(always)]
    pub fn fetch_vle_u16(&mut self) -> CompactResult<u16> {
        // VLE decoding for u16
        let first_byte = self.fetch_byte()?;
        if first_byte & 0x80 == 0 {
            Ok(first_byte as u16)
        } else {
            let second_byte = self.fetch_byte()?;
            Ok(((first_byte & 0x7F) as u16) | ((second_byte as u16) << 7))
        }
    }

    #[inline]
    pub fn fetch_vle_u32(&mut self) -> CompactResult<u32> {
        // VLE decoding for u32
        let mut result = 0u32;
        let mut shift = 0;
        loop {
            let byte = self.fetch_byte()?;
            result |= ((byte & 0x7F) as u32) << shift;
            if byte & 0x80 == 0 {
                break;
            }
            shift += 7;
            if shift >= 32 {
                return Err(VMErrorCode::InvalidInstruction);
            }
        }
        Ok(result)
    }

    #[inline]
    pub fn fetch_vle_u64(&mut self) -> CompactResult<u64> {
        // VLE decoding for u64
        let mut result = 0u64;
        let mut shift = 0;
        loop {
            let byte = self.fetch_byte()?;
            result |= ((byte & 0x7F) as u64) << shift;
            if byte & 0x80 == 0 {
                break;
            }
            shift += 7;
            if shift >= 64 {
                return Err(VMErrorCode::InvalidInstruction);
            }
        }
        Ok(result)
    }

    #[inline]
    pub fn fetch_input_u8(&mut self) -> CompactResult<u8> {
        if self.input_ptr as usize >= self.instruction_data.len() {
            return Err(VMErrorCode::InvalidParameter);
        }
        debug_assert!(
            (self.input_ptr as usize) < self.instruction_data.len(),
            "Input pointer {} must be < instruction_data.len() {}",
            self.input_ptr,
            self.instruction_data.len()
        );
        let value = self.instruction_data[self.input_ptr as usize];
        self.input_ptr += 1;
        Ok(value)
    }

    #[inline]
    pub fn fetch_input_u64(&mut self) -> CompactResult<u64> {
        let mut result = 0u64;
        for i in 0..8 {
            result |= (self.fetch_input_u8()? as u64) << (i * 8);
        }
        Ok(result)
    }

    // --- Crypto & account operations (10 occurrences) ---

    #[inline]
    pub fn extract_pubkey(&self, value_ref: &ValueRef) -> CompactResult<[u8; 32]> {
        match value_ref {
            ValueRef::PubkeyRef(offset) => {
                let start = *offset as usize;
                let end = start + 32;
                if end <= self.instruction_data.len() {
                    let mut pubkey = [0u8; 32];
                    pubkey.copy_from_slice(&self.instruction_data[start..end]);
                    Ok(pubkey)
                } else if start < self.accounts.len() {
                    Ok(*self.accounts[start].key())
                } else {
                    Err(VMErrorCode::MemoryError)
                }
            }
            ValueRef::TempRef(offset, len) => {
                // Handle TempRef (created by PUSH_PUBKEY)
                if *len != 32 {
                    return Err(VMErrorCode::TypeMismatch);
                }
                let start = *offset as usize;
                let end = start + 32;
                if end <= self.temp_buffer().len() {
                    let mut pubkey = [0u8; 32];
                    pubkey.copy_from_slice(&self.temp_buffer()[start..end]);
                    Ok(pubkey)
                } else {
                    Err(VMErrorCode::MemoryError)
                }
            }
            _ => Err(VMErrorCode::TypeMismatch),
        }
    }

    #[inline]
    pub fn fetch_pubkey_to_temp(&mut self) -> CompactResult<u8> {
        let offset = self.alloc_temp(32)?;
        for i in 0..32 {
            let buf_index = offset as usize + i;
            debug_assert!(
                buf_index < self.storage.temp_buffer.len(),
                "Temp buffer index {} must be < temp_buffer.len() {}",
                buf_index,
                self.storage.temp_buffer.len()
            );
            self.storage.temp_buffer[buf_index] = self.fetch_byte()?;
        }
        Ok(offset)
    }

    // --- Temp buffer management (3 occurrences) ---

    #[inline]
    pub fn temp_offset(&self) -> usize {
        self.temp_pos
    }

    #[inline]
    pub fn set_temp_offset(&mut self, offset: usize) {
        self.temp_pos = offset;
    }

    /// Reset the temporary buffer allocation pointer so future allocations
    /// start from the beginning of the buffer again.
    ///
    /// This should be invoked after an execution completes to prevent
    /// accidentally reusing stale data left in the temp buffer.
    #[inline]
    pub fn reset_temp_buffer(&mut self) {
        self.temp_pos = 0;
    }

    // --- Security & authorization (1 occurrence) ---

    #[inline]
    pub fn check_bytecode_authorization(&self, account_idx: u8) -> CompactResult<()> {
        let account = self.get_account(account_idx)?;
        let required_authority = *account.owner();
        if self.program_id == required_authority {
            Ok(())
        } else {
            return Err(VMErrorCode::ScriptNotAuthorized);
        }
    }

    // --- Account creation stubs (2 occurrences) - minimal for compilation ---

    /// Serialize CreateAccount instruction data into the provided buffer.
    /// Format: [discriminator:4][lamports:8][space:8][owner:32]
    #[inline(always)]
    fn serialize_create_account_data(
        data: &mut [u8; 52],
        lamports: u64,
        space: u64,
        owner: &[u8; 32],
    ) {
        data[0..4].copy_from_slice(&0u32.to_le_bytes()); // CreateAccount discriminator
        data[4..12].copy_from_slice(&lamports.to_le_bytes());
        data[12..20].copy_from_slice(&space.to_le_bytes());
        data[20..52].copy_from_slice(owner);
    }

    #[inline]
    pub fn create_account(
        &mut self,
        account_idx: u8,
        space: u64,
        lamports: u64,
        owner: &Pubkey,
    ) -> CompactResult<()> {
        // Validate accounts lazily first, then access unchecked
        self.lazy_validator.ensure_validated(0, self.accounts)?;
        // DO NOT validate account_idx - it is expected to be uninitialized
        // self.lazy_validator.ensure_validated(account_idx, self.accounts)?;

        let new_account = self.get_account_unchecked(account_idx)?;

        // Find a valid payer (signer, writable, not the new account)
        let mut payer = self.get_account_unchecked(0)?;
        let mut payer_found = false;
        
        for i in 0..self.accounts.len() {
             let acc = self.get_account_unchecked(i as u8)?;
             if acc.is_signer() && acc.is_writable() && acc.key() != new_account.key() {
                 payer = acc;
                 payer_found = true;
                 #[cfg(feature = "debug-logs")]
                 crate::debug_log!("CreateAccount: Found valid payer at index {} (key: {})", i, payer.key());
                 break;
             }
        }
        
        if !payer_found {
             #[cfg(feature = "debug-logs")]
             crate::debug_log!("CreateAccount: WARNING - No valid payer found! Defaulting to index 0 (key: {})", payer.key());
        }

        // Locate the system program account
        let system_program_id = Pubkey::from(SYSTEM_PROGRAM_ID);
        
        #[cfg(feature = "debug-logs")]
        {
            let sys_bytes = system_program_id.as_ref();
            crate::debug_log!("CreateAccount: Looking for SystemProgram: {} {} {} {}", sys_bytes[0], sys_bytes[1], sys_bytes[2], sys_bytes[3]);
            for (i, acc) in self.accounts.iter().enumerate() {
                let key_bytes = acc.key().as_ref();
                crate::debug_log!("  Account {}: {} {} {} {}", i, key_bytes[0], key_bytes[1], key_bytes[2], key_bytes[3]);
            }
        }

        let system_program = self
            .accounts
            .iter()
            .find(|a| a.key() == &system_program_id)
            .ok_or(VMErrorCode::AccountNotFound)?;

        // Build instruction data for SystemProgram::CreateAccount
        let mut data = [0u8; 52];
        Self::serialize_create_account_data(&mut data, lamports, space, owner);

        let metas = [
            AccountMeta {
                pubkey: payer.key(),
                is_signer: payer.is_signer(),
                is_writable: payer.is_writable(),
            },
            AccountMeta {
                pubkey: new_account.key(),
                is_signer: new_account.is_signer(),
                is_writable: new_account.is_writable(),
            },
        ];

        let instruction = Instruction {
            program_id: &system_program_id,
            accounts: &metas,
            data: &data,
        };

        invoke::<3>(&instruction, &[payer, new_account, system_program]).map_err(|_| VMErrorCode::InvokeError)?;

        // CRITICAL FIX: Refresh pointer for the newly created account
        // After CreateAccount CPI, the account data has been reallocated by the Solana runtime.
        // Force Pinocchio to recalculate the data pointer by accessing the account again.
        let _ = self.refresh_account_pointers_after_cpi(&[account_idx as usize]);

        Ok(())
    }

    #[inline]
    pub fn create_pda_account(
        &mut self,
        account_idx: u8,
        seeds: &[&[u8]],
        bump: u8,
        space: u64,
        lamports: u64,
        owner: &Pubkey,
    ) -> CompactResult<()> {
        // Validate accounts lazily first, then access unchecked
        self.lazy_validator.ensure_validated(0, self.accounts)?;
        // DO NOT validate account_idx - it is expected to be uninitialized
        // self.lazy_validator.ensure_validated(account_idx, self.accounts)?;

        let new_account = self.get_account_unchecked(account_idx)?;

        // Find a valid payer (signer, writable, not the new account)
        let mut payer = self.get_account_unchecked(0)?;
        for i in 0..self.accounts.len() {
             let acc = self.get_account_unchecked(i as u8)?;
             if acc.is_signer() && acc.is_writable() && acc.key() != new_account.key() {
                 payer = acc;
                 break;
             }
        }

        let system_program_id = Pubkey::from(SYSTEM_PROGRAM_ID);
        let system_program = self
            .accounts
            .iter()
            .find(|a| a.key() == &system_program_id)
            .ok_or(VMErrorCode::AccountNotFound)?;

        // Instruction data identical to create_account
        let mut data = [0u8; 52];
        Self::serialize_create_account_data(&mut data, lamports, space, owner);

        let metas = [
            AccountMeta {
                pubkey: payer.key(),
                is_signer: payer.is_signer(),
                is_writable: payer.is_writable(),
            },
            AccountMeta {
                pubkey: new_account.key(),
                is_signer: new_account.is_signer(),
                is_writable: new_account.is_writable(),
            },
        ];

        let instruction = Instruction {
            program_id: &system_program_id,
            accounts: &metas,
            data: &data,
        };

        // Convert seeds and bump into Signer representation
        const MAX_SEEDS: usize = 8;
        let binding = [bump]; // Move binding declaration outside to ensure lifetime
        let mut seed_vec: Vec<Seed, MAX_SEEDS> = Vec::new();
        for s in seeds.iter() {
            seed_vec
                .push(Seed::from(*s))
                .map_err(|_| VMErrorCode::TooManySeeds)?;
        }
        seed_vec
            .push(Seed::from(&binding))
            .map_err(|_| VMErrorCode::TooManySeeds)?;
        let signer = Signer::from(seed_vec.as_slice());

        invoke_signed::<3>(
            &instruction,
            &[payer, new_account, system_program],
            &[signer],
        )
        .map_err(|_| VMErrorCode::InvokeError)?;

        // CRITICAL FIX: Refresh pointer for the newly created PDA account
        // Same as create_account - after CreateAccount CPI, the account data is reallocated.
        let _ = self.refresh_account_pointers_after_cpi(&[account_idx as usize]);

        Ok(())
    }

    // --- Solana integration ---

    #[inline]
    pub fn invoke_instruction<const N: usize>(
        &self,
        instruction: &Instruction,
        accounts: &[&AccountInfo; N],
    ) -> CompactResult<()> {
        invoke::<N>(instruction, accounts).map_err(|_| VMErrorCode::InvokeError)
    }

    #[inline]
    pub fn invoke_signed_instruction<const N: usize>(
        &self,
        instruction: &Instruction,
        accounts: &[&AccountInfo; N],
        signers: &[Signer],
    ) -> CompactResult<()> {
        invoke_signed::<N>(instruction, accounts, signers).map_err(|_| VMErrorCode::InvokeError)
    }

    /// Get account data by index for external calls
    pub fn get_account_data(&self, account_index: usize) -> CompactResult<&[u8]> {
        if account_index >= self.accounts.len() {
            return Err(VMErrorCode::AccountNotFound);
        }

        // Validate account lazily
        self.lazy_validator
            .ensure_validated(account_index as u8, self.accounts)?;

        let account = &self.accounts[account_index];
        if account.data_len() == 0 {
            return Err(VMErrorCode::AccountDataEmpty);
        }
        // SAFETY: We've verified the account contains data
        let data = unsafe { account.borrow_data_unchecked() };
        Ok(data)
    }

    /// Switch to external bytecode for CALL_EXTERNAL
    pub fn switch_to_external_bytecode(
        &mut self,
        external_bytecode: &'a [u8],
        offset: usize,
    ) -> CompactResult<()> {
        if external_bytecode.len() > MAX_SCRIPT_SIZE {
            return Err(VMErrorCode::InvalidScriptSize);
        }
        if offset >= external_bytecode.len() {
            return Err(VMErrorCode::InvalidInstructionPointer);
        }
        self.bytecode = external_bytecode;
        self.pc = offset as u16;
        Ok(())
    }

    /// Refresh account data pointers after CPI operations.
    ///
    /// When the Solana runtime executes a CPI (Cross-Program Invocation), it updates the
    /// Account struct metadata (particularly data_len) to reflect any size changes.
    /// This method calls Pinocchio's refresh_after_cpi() on affected accounts.
    ///
    /// This ensures:
    /// 1. Developers are explicit about CPI effects
    /// 2. Pinocchio uses current account metadata
    /// 3. Subsequent STORE_FIELD operations access updated data
    #[inline]
    pub fn refresh_account_pointers_after_cpi(&self, account_indices: &[usize]) -> CompactResult<()> {
        error_log!(
            "CPI_POINTER_REFRESH: Refreshing pointers for {} accounts",
            account_indices.len() as u32
        );

        // Call Pinocchio's refresh method on each affected account
        for &idx in account_indices {
            if idx >= self.accounts.len() {
                continue;
            }
            let account = &self.accounts[idx];

            // Pinocchio's refresh_after_cpi() ensures we're working with current account metadata
            // This uses our custom fork with the refresh_after_cpi() method
            account.refresh_after_cpi();

            // Log for debugging
            let data_len = account.data_len();
            let ptr = unsafe { account.borrow_data_unchecked().as_ptr() as usize };

            error_log!(
                "CPI_POINTER_REFRESH: idx={} data_len={} ptr={}",
                idx as u32,
                data_len as u32,
                ptr as u32
            );
        }

        Ok(())
    }

    // --- Lazy validation operations ---

    /// Get validation statistics for performance monitoring
    #[inline]
    pub fn validation_stats(&self) -> crate::lazy_validation::ValidationStats {
        crate::lazy_validation::ValidationStats::calculate(&self.lazy_validator)
    }

    /// Check if specific account has been validated
    #[inline]
    pub fn is_account_validated(&self, index: u8) -> bool {
        self.lazy_validator.is_validated(index)
    }

    /// Get count of validated accounts
    #[inline]
    pub fn validated_account_count(&self) -> u8 {
        self.lazy_validator.validated_count()
    }

    /// Validate account constraints using bitwise constraint checking
    /// This uses pre-computed constraint bits for O(1) validation
    #[inline]
    pub fn validate_bitwise_constraints(&self, constraints: u64) -> CompactResult<()> {
        self.lazy_validator
            .validate_constraints_bitwise(constraints, self.accounts)
    }
}

// Legacy compatibility aliases for gradual migration
pub type ExecutionManager<'a> = ExecutionContext<'a>;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "CPI invoke succeeds in test environment"]
    fn invoke_instruction_propagates_error() {
        let program_id = Pubkey::from([1u8; 32]);
        let mut lamports = 0u64;
        let mut data_buf: [u8; 0] = [];
        let account = AccountInfo::new(
            &program_id,
            false,
            false,
            &mut lamports,
            &mut data_buf,
            &program_id,
            true,
            0,
        );
        let accounts = [account];
        let mut storage = StackStorage::new(&[]);
        let ctx = ExecutionContext::new(&[], &accounts, program_id, &[], 0, &mut storage, 0, 0);
        let metas = [
            AccountMeta {
                pubkey: account.key(),
                is_signer: account.is_signer(),
                is_writable: account.is_writable(),
            },
        ];
        let instruction = Instruction {
            program_id: &program_id,
            accounts: &metas,
            data: &[],
        };
        let result = ctx.invoke_instruction::<1>(&instruction, &[&accounts[0]]);
        assert!(matches!(result, Err(VMErrorCode::InvokeError)));
    }

    #[test]
    #[ignore = "CPI invoke_signed succeeds in test environment"]
    fn invoke_signed_instruction_propagates_error() {
        let program_id = Pubkey::from([2u8; 32]);
        let mut lamports = 0u64;
        let mut data_buf: [u8; 0] = [];
        let account = AccountInfo::new(
            &program_id,
            false,
            false,
            &mut lamports,
            &mut data_buf,
            &program_id,
            true,
            0,
        );
        let accounts = [account];
        let mut storage = StackStorage::new(&[]);
        let ctx = ExecutionContext::new(&[], &accounts, program_id, &[], 0, &mut storage, 0, 0);
        let metas = [
            AccountMeta {
                pubkey: account.key(),
                is_signer: account.is_signer(),
                is_writable: account.is_writable(),
            },
        ];
        let instruction = Instruction {
            program_id: &program_id,
            accounts: &metas,
            data: &[],
        };
        let seed = Seed::from(&[1u8][..]);
        let signer_seeds = [seed];
        let signer = Signer::from(&signer_seeds);
        let result = ctx.invoke_signed_instruction::<1>(&instruction, &[&accounts[0]], &[signer]);
        assert!(matches!(result, Err(VMErrorCode::InvokeError)));
    }
}
