//! Ultra-lightweight execution context for MitoVM
//!
//! Single unified context replacing all previous abstraction layers.
//! Designed for maximum performance with zero indirection.

use crate::{
    error::{CompactResult, Result, VMError, VMErrorCode},
    metadata::ImportMetadata,
    stack::StackStorage,
    types::CallFrame,
    MAX_CALL_DEPTH, MAX_LOCALS, MAX_PARAMETERS, MAX_SCRIPT_SIZE, STACK_SIZE,
};

use crate::debug_log;
use five_protocol::ValueRef;
#[cfg(target_os = "solana")]
use alloc::vec::Vec;
#[cfg(not(target_os = "solana"))]
use std::vec::Vec;
use pinocchio::{
    account_info::AccountInfo,
    instruction::{Instruction, Signer},
    program::{invoke, invoke_signed},
    pubkey::Pubkey,
};
#[cfg(any(target_os = "solana", test))]
use pinocchio::instruction::{AccountMeta, Seed};

use crate::systems::{
    accounts::AccountManager,
    frame::FrameManager,
    memory::MemoryManager,
    stack::StackManager,
};

// System program ID constant
const SYSTEM_PROGRAM_ID: [u8; 32] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
]; // Solana system program ID: 11111111111111111111111111111111

const MAX_ACCOUNT_SIZE: u64 = 10 * 1024 * 1024; // 10MB limit

// Shared parameter storage (only one copy, indexed by call frames)
pub(crate) const SHARED_PARAM_SIZE: usize = MAX_PARAMETERS + 1;

/// Single unified execution context for maximum performance
/// Temp buffer is stack-based to keep the entire context on the stack
/// while remaining within Solana's 4KB BPF stack limit.
/// Replaces: ValueRefStack, ExecutionContext, CoreExecutionContext,
/// MemoryContext, CallContext, ExternalContext, ExecutionManager
pub struct ExecutionContext<'a> {
    // --- Systems ---
    pub stack: StackManager<'a>,
    pub memory: MemoryManager<'a>,
    pub accounts: AccountManager<'a>,
    pub frame: FrameManager<'a>,

    // --- Core execution state ---
    pub bytecode: &'a [u8],
    pub pc: u16,

    // --- Function metadata (optimized header V2) ---
    pub public_function_count: u8, // For external dispatch validation
    pub total_function_count: u8,  // For internal CALL validation
    pub header_features: u32,      // Raw header feature flags

    // --- External Solana state ---
    pub program_id: Pubkey,
    pub instruction_data: &'a [u8],

    // --- Execution state ---
    pub halted: bool,
    pub return_value: Option<ValueRef>,
    pub current_opcode: Option<u8>,

    // --- Compute tracking (minimal for ultra-lightweight VM) ---
    pub compute_units_consumed: u64,

    // --- Input data processing ---
    pub input_ptr: u8,

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
            stack: StackManager::new(&mut storage.stack, &mut storage.registers),
            memory: MemoryManager::new(&mut storage.temp_buffer),
            frame: FrameManager::new(&mut storage.call_stack, &mut storage.locals),
            accounts: AccountManager::new(accounts, program_id),

            public_function_count,
            total_function_count,
            header_features: 0,
            program_id,
            instruction_data,
            halted: false,
            return_value: None,
            current_opcode: None,
            compute_units_consumed: 0,
            input_ptr: 0,
            import_metadata: ImportMetadata::new(bytecode, bytecode.len()).unwrap_or_else(|_| {
                // If parsing fails, create empty metadata (backward compatible)
                ImportMetadata::new(&[], 0).unwrap()
            }),
        }
    }

    // --- Stack operations (delegated to StackManager) ---

    #[inline(always)]
    pub fn push(&mut self, value: ValueRef) -> CompactResult<()> {
        self.stack.push(value)
    }

    #[inline(always)]
    pub fn pop(&mut self) -> CompactResult<ValueRef> {
        self.stack.pop()
    }

    #[inline(always)]
    pub fn peek(&self) -> CompactResult<ValueRef> {
        self.stack.peek()
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

    // --- Memory operations (delegated to MemoryManager) ---

    #[inline(always)]
    pub fn alloc_temp(&mut self, size: u8) -> CompactResult<u8> {
        self.memory.alloc_temp(size)
    }

    #[inline(always)]
    pub fn get_temp_data(&self, offset: u8, size: u8) -> CompactResult<&[u8]> {
        self.memory.get_temp_data(offset, size)
    }

    #[inline(always)]
    pub fn get_temp_data_mut(&mut self, offset: u8, size: u8) -> CompactResult<&mut [u8]> {
        self.memory.get_temp_data_mut(offset, size)
    }

    #[inline(always)]
    pub fn temp_buffer(&self) -> &[u8] {
        self.memory.temp_buffer()
    }

    #[inline(always)]
    pub fn temp_buffer_mut(&mut self) -> &mut [u8] {
        self.memory.temp_buffer_mut()
    }

    /// Allocate a temp buffer slot for Option/Result storage
    #[inline(always)]
    pub fn allocate_temp_slot(&mut self) -> CompactResult<u8> {
        self.memory.allocate_temp_slot()
    }

    /// Get mutable reference to temp buffer as fixed-size array for ValueAccessContext
    #[inline]
    pub fn temp_buffer_fixed_mut(&mut self) -> Result<&mut [u8; crate::TEMP_BUFFER_SIZE]> {
        self.memory.temp_buffer_fixed_mut()
    }

    /// Write a [`ValueRef`] into the temp buffer
    #[inline]
    pub fn write_value_to_temp(&mut self, value: &ValueRef) -> Result<u16> {
        self.memory.write_value_to_temp(value)
    }

    /// Deserialize a [`ValueRef`] previously written
    #[inline]
    pub fn read_value_from_temp(&self, offset: u16) -> Result<ValueRef> {
        self.memory.read_value_from_temp(offset)
    }

    // --- Heap operations (delegated to MemoryManager) ---

    #[inline]
    pub fn heap_alloc(&mut self, size: usize) -> CompactResult<u32> {
        self.memory.heap_alloc(size)
    }

    #[inline]
    pub fn get_heap_data_mut(&mut self, offset: u32, size: u32) -> CompactResult<&mut [u8]> {
        self.memory.get_heap_data_mut(offset, size)
    }

    #[inline]
    pub fn get_heap_data(&self, offset: u32, size: u32) -> CompactResult<&[u8]> {
        self.memory.get_heap_data(offset, size)
    }

    // --- Call stack operations (delegated to FrameManager) ---

    #[inline(always)]
    pub fn push_call_frame(&mut self, frame: CallFrame<'a>) -> Result<()> {
        self.frame.push_call_frame(frame)
    }

    #[inline(always)]
    pub fn pop_call_frame(&mut self) -> CompactResult<CallFrame<'a>> {
        self.frame.pop_call_frame()
    }

    // --- Local variables (delegated to FrameManager) ---

    #[inline]
    pub fn get_local(&self, index: u8) -> CompactResult<ValueRef> {
        self.frame.get_local(index)
    }

    #[inline(always)]
    pub fn set_local(&mut self, index: u8, value: ValueRef) -> CompactResult<()> {
        self.frame.set_local(index, value)
    }

    #[inline(always)]
    pub fn clear_local(&mut self, index: u8) -> CompactResult<()> {
        self.frame.clear_local(index)
    }

    // --- Registers (delegated to StackManager) ---

    #[inline(always)]
    pub fn get_register(&self, index: u8) -> CompactResult<ValueRef> {
        self.stack.get_register(index)
    }

    #[inline(always)]
    pub fn set_register(&mut self, index: u8, value: ValueRef) -> CompactResult<()> {
        self.stack.set_register(index, value)
    }

    // --- Account operations with lazy validation (delegated to AccountManager) ---

    #[inline(always)]
    pub fn get_account(&self, index: u8) -> CompactResult<&AccountInfo> {
        self.accounts.get(index)
    }

    /// Get account without lazy validation (for internal VM use)
    #[inline(always)]
    pub fn get_account_unchecked(&self, index: u8) -> CompactResult<&AccountInfo> {
        self.accounts.get_unchecked(index)
    }

    // --- Parameter operations (delegated to FrameManager) ---

    #[inline(always)]
    pub fn set_parameters(&mut self, params: [ValueRef; 8]) {
        self.frame.set_parameters(params)
    }

    #[inline(always)]
    pub fn parameters(&self) -> &[ValueRef] {
        self.frame.parameters()
    }

    // --- Stack utility methods ---

    #[inline(always)]
    pub fn size(&self) -> usize {
        self.stack.len()
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.stack.len()
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

    #[inline(always)]
    pub fn current_opcode(&self) -> Option<u8> {
        self.current_opcode
    }

    #[inline(always)]
    pub fn set_current_opcode(&mut self, opcode: u8) {
        self.current_opcode = Some(opcode);
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

    // --- Call stack management ---

    #[inline(always)]
    pub fn call_depth(&self) -> usize {
        self.frame.call_depth()
    }

    #[inline(always)]
    pub fn set_call_depth(&mut self, depth: u8) -> CompactResult<()> {
        self.frame.set_call_depth(depth)
    }

    #[inline(always)]
    pub fn get_call_frame(&self, index: usize) -> CompactResult<&CallFrame<'a>> {
        self.frame.get_call_frame(index)
    }

    #[inline(always)]
    pub fn set_call_frame(&mut self, index: usize, frame: CallFrame<'a>) -> CompactResult<()> {
        self.frame.set_call_frame(index, frame)
    }

    // --- Data access methods ---

    #[inline(always)]
    pub fn instruction_data(&self) -> &[u8] {
        self.instruction_data
    }

    #[inline(always)]
    pub fn accounts(&self) -> &[AccountInfo] {
        self.accounts.accounts()
    }

    // --- Local variable management ---

    #[inline(always)]
    pub fn local_count(&self) -> u8 {
        self.frame.local_count()
    }

    #[inline(always)]
    pub fn set_local_count(&mut self, count: u8) {
        self.frame.set_local_count(count)
    }

    #[inline(always)]
    pub fn local_base(&self) -> u8 {
        self.frame.local_base()
    }

    #[inline(always)]
    pub fn set_local_base(&mut self, base: u8) {
        self.frame.set_local_base(base)
    }

    #[inline(always)]
    pub fn allocate_locals(&mut self, count: u8) -> CompactResult<()> {
        self.frame.allocate_locals(count)
    }

    #[inline(always)]
    pub fn deallocate_locals(&mut self) {
        self.frame.deallocate_locals()
    }

    // --- Stack operations with zero indirection ---

    #[inline(always)]
    pub fn dup(&mut self) -> CompactResult<()> {
        self.stack.dup()
    }

    #[inline(always)]
    pub fn swap(&mut self) -> CompactResult<()> {
        self.stack.swap()
    }

    #[inline(always)]
    pub fn pick(&mut self, depth: u8) -> CompactResult<()> {
        self.stack.pick(depth)
    }

    // --- Parameter management ---

    #[inline(always)]
    pub fn param_start(&self) -> u8 {
        self.frame.param_start()
    }

    #[inline(always)]
    pub fn param_len(&self) -> u8 {
        self.frame.param_len()
    }

    #[inline(always)]
    pub fn allocate_params(&mut self, count: u8) -> CompactResult<()> {
        self.frame.allocate_params(count)
    }

    #[inline(always)]
    pub fn restore_parameters(&mut self, start: u8, len: u8) {
        self.frame.restore_parameters(start, len)
    }

    #[inline(always)]
    pub fn parameters_mut(&mut self) -> &mut [ValueRef] {
        self.frame.parameters_mut()
    }

    #[inline(always)]
    pub fn set_parameter(&mut self, index: usize, value: ValueRef) -> CompactResult<()> {
        self.frame.set_parameter(index, value)
    }

    // --- Bytecode fetching extensions ---

    #[inline(always)]
    pub fn fetch_u32(&mut self) -> CompactResult<u32> {
        self.fetch_le::<u32, 4>()
    }

    #[inline(always)]
    pub fn fetch_vle_u16(&mut self) -> CompactResult<u16> {
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

    // --- Crypto & account operations ---

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
                } else {
                    // Fallback to accounts check if not in instruction data
                    // Original code: if start < self.accounts.len() { Ok(*self.accounts[start].key()) }
                    if start < self.accounts.accounts().len() {
                        Ok(*self.accounts.accounts()[start].key())
                    } else {
                        Err(VMErrorCode::MemoryError)
                    }
                }
            }
            ValueRef::TempRef(offset, len) => {
                if *len != 32 {
                    return Err(VMErrorCode::TypeMismatch);
                }
                let start = *offset as usize;
                let end = start + 32;
                // Use MemoryManager
                let temp_buf = self.memory.temp_buffer();
                if end <= temp_buf.len() {
                    let mut pubkey = [0u8; 32];
                    pubkey.copy_from_slice(&temp_buf[start..end]);
                    Ok(pubkey)
                } else {
                    Err(VMErrorCode::MemoryError)
                }
            }
            ValueRef::U64(0) => Ok(self.program_id),
            ValueRef::AccountRef(idx, offset) => {
                let account = self.accounts.get(*idx)?;
                let data = unsafe { account.borrow_data_unchecked() };
                let start = *offset as usize;
                let end = start + 32;
                if end > data.len() {
                    return Err(VMErrorCode::InvalidAccountData);
                }
                let mut pubkey = [0u8; 32];
                pubkey.copy_from_slice(&data[start..end]);
                Ok(pubkey)
            }
            _ => Err(VMErrorCode::TypeMismatch),
        }
    }

    #[inline]
    pub fn fetch_pubkey_to_temp(&mut self) -> CompactResult<u8> {
        let offset = self.memory.alloc_temp(32)?;
        for i in 0..32 {
            let byte = self.fetch_byte()?;
            self.memory.temp_buffer[offset as usize + i] = byte;
        }
        Ok(offset)
    }

    // --- Temp buffer management ---

    #[inline]
    pub fn temp_offset(&self) -> usize {
        self.memory.temp_offset()
    }

    #[inline]
    pub fn set_temp_offset(&mut self, offset: usize) {
        self.memory.set_temp_offset(offset)
    }

    #[inline]
    pub fn reset_temp_buffer(&mut self) {
        self.memory.reset_temp_buffer()
    }

    // --- Security & authorization ---

    #[inline]
    pub fn check_bytecode_authorization(&self, account_idx: u8) -> CompactResult<()> {
        self.accounts.check_authorization(account_idx)
    }

    #[inline]
    pub fn extract_string_slice(&self, value_ref: &ValueRef) -> CompactResult<(u32, &[u8])> {
        match value_ref {
            ValueRef::StringRef(offset) => {
                let start = *offset as usize;
                let temp_buf = self.memory.temp_buffer();
                
                if start >= temp_buf.len() {
                     crate::debug_log!("EXTRACT_STRING ERROR: Offset out of bounds. offset={} temp_len={}", start, temp_buf.len());
                     return Err(VMErrorCode::MemoryError);
                }
                
                let len = temp_buf[start] as usize;
                let data_start = start + 2;
                let data_end = data_start + len;
                
                if data_end > temp_buf.len() {
                    crate::debug_log!("EXTRACT_STRING ERROR: String end out of bounds. start={} end={} len={} temp_len={}", data_start, data_end, len, temp_buf.len());
                    return Err(VMErrorCode::MemoryError);
                }
                
                Ok((len as u32, &temp_buf[data_start..data_end]))
            }
            ValueRef::HeapString(heap_id) => {
                let start = *heap_id as usize;
                let heap_storage = &self.memory.heap_storage;

                if start + 4 > heap_storage.len() {
                    return Err(VMErrorCode::MemoryError);
                }

                let len_bytes = &heap_storage[start..start+4];
                let len = u32::from_le_bytes(len_bytes.try_into().unwrap()) as usize;

                let data_start = start + 4;
                let data_end = data_start + len;

                if data_end > heap_storage.len() {
                    return Err(VMErrorCode::MemoryError);
                }

                Ok((len as u32, &heap_storage[data_start..data_end]))
            }
            ValueRef::U64(0) => Ok((0, &[])),
            _ => Err(VMErrorCode::TypeMismatch),
        }
    }

    // --- Account creation ---

    #[inline]
    pub fn create_account(
        &mut self,
        account_idx: u8,
        space: u64,
        lamports: u64,
        owner: &Pubkey,
    ) -> CompactResult<()> {
        self.accounts.create_account(account_idx, space, lamports, owner)
    }

    #[inline]
    pub fn create_account_with_payer(
        &mut self,
        account_idx: u8,
        payer_idx: u8,
        space: u64,
        lamports: u64,
        owner: &Pubkey,
    ) -> CompactResult<()> {
        self.accounts.create_account_with_payer(account_idx, payer_idx, space, lamports, owner)
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
        payer_idx: u8,
    ) -> CompactResult<()> {
        self.accounts.create_pda_account(account_idx, seeds, bump, space, lamports, owner, payer_idx)
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
        // Use AccountManager
        let account = self.accounts.get(account_index as u8)?;
        if account.data_len() == 0 {
            return Err(VMErrorCode::AccountDataEmpty);
        }
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

    #[inline]
    pub fn refresh_account_pointers_after_cpi(&self, account_indices: &[usize]) -> CompactResult<()> {
        self.accounts.refresh_account_pointers_after_cpi(account_indices)
    }

    // --- Lazy validation operations ---

    #[inline]
    pub fn validation_stats(&self) -> crate::lazy_validation::ValidationStats {
        self.accounts.validation_stats()
    }

    #[inline]
    pub fn is_account_validated(&self, index: u8) -> bool {
        self.accounts.is_validated(index)
    }

    #[inline]
    pub fn validated_account_count(&self) -> u8 {
        self.accounts.validated_count()
    }

    #[inline]
    pub fn validate_bitwise_constraints(&self, constraints: u64) -> CompactResult<()> {
        self.accounts.validate_bitwise_constraints(constraints)
    }
}

// Legacy compatibility aliases for gradual migration
pub type ExecutionManager<'a> = ExecutionContext<'a>;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invoke_instruction_succeeds() {
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
        assert!(result.is_ok(), "Invoke should succeed in test env");
    }

    #[test]
    fn invoke_signed_instruction_succeeds() {
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
        assert!(result.is_ok(), "Invoke signed should succeed in test env");
    }
}
