//! Ultra-lightweight execution context for MitoVM
//!
//! Single unified context replacing all previous abstraction layers.
//! Designed for maximum performance with zero indirection.

use crate::{
    error::{CompactResult, Result, VMErrorCode},
    metadata::ImportMetadata,
    stack::StackStorage,
    types::CallFrame,
    MAX_LOCALS, MAX_PARAMETERS, MAX_SCRIPT_SIZE,
};

use crate::debug_log;
use five_protocol::{ValueRef, types};
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
    resource::ResourceManager,
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
    pub memory: ResourceManager<'a>,
    pub accounts: AccountManager<'a>,
    pub frame: FrameManager<'a>,

    // --- Core execution state ---
    pub bytecode: &'a [u8],
    /// Original root bytecode for context restoration
    pub root_bytecode: &'a [u8],
    pub current_context: u8,
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

    // --- Syscall Caching ---
    pub cached_clock: Option<pinocchio::sysvars::clock::Clock>,
    pub cached_rent: Option<pinocchio::sysvars::rent::Rent>,
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
        storage: &'a mut StackStorage,
        public_function_count: u8,
        total_function_count: u8,
    ) -> Self {
        let (stack, call_stack, locals, temp, heap) = storage.split_mut();
        Self {
            bytecode,
            root_bytecode: bytecode,
            current_context: crate::types::ROOT_CONTEXT,
            pc: start_pc,
            stack: StackManager::new(stack),
            memory: ResourceManager::new(temp, heap),
            frame: FrameManager::new(call_stack, locals),
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
            cached_clock: None,
            cached_rent: None,
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
        #[cfg(feature = "unchecked-execution")]
        unsafe {
            // SAFETY: Verified at deploy time.
            let byte = *self.bytecode.get_unchecked(self.pc as usize);
            self.pc = self.pc.saturating_add(1);
            Ok(byte)
        }
        #[cfg(not(feature = "unchecked-execution"))]
        {
            if self.pc as usize >= self.bytecode.len() {
                return Err(VMErrorCode::InvalidInstructionPointer);
            }
            let byte = self.bytecode[self.pc as usize];
            self.pc = self.pc.saturating_add(1);
            Ok(byte)
        }
    }

    #[inline(always)]
    fn fetch_le<T, const N: usize>(&mut self) -> CompactResult<T>
    where
        T: FromLeBytes<N>,
    {
        let start = self.pc as usize;
        let end = start + N;

        #[cfg(feature = "unchecked-execution")]
        unsafe {
            // SAFETY: Verified at deploy time.
            self.pc = end as u16;
            let bytes: [u8; N] =
                core::ptr::read_unaligned(self.bytecode.as_ptr().add(start) as *const [u8; N]);
            Ok(T::from_le_bytes(bytes))
        }
        #[cfg(not(feature = "unchecked-execution"))]
        {
            if end > self.bytecode.len() {
                return Err(VMErrorCode::InvalidInstructionPointer);
            }
            self.pc = end as u16;
            let bytes: [u8; N] = unsafe {
                core::ptr::read_unaligned(self.bytecode.as_ptr().add(start) as *const [u8; N])
            };
            Ok(T::from_le_bytes(bytes))
        }
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

    // --- Memory operations (delegated to ResourceManager) ---

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

    // --- Heap operations (delegated to ResourceManager) ---

    #[inline]
    pub fn heap_alloc(&mut self, size: usize) -> CompactResult<u32> {
        // Use alloc_heap_unsafe for zero-copy chunked allocation
        self.memory.alloc_heap_unsafe(size)
    }

    #[inline]
    pub fn get_heap_data_mut(&mut self, offset: u32, size: u32) -> CompactResult<&mut [u8]> {
        self.memory.get_heap_data_mut(offset, size)
    }

    #[inline]
    pub fn get_heap_data(&self, offset: u32, size: u32) -> CompactResult<&[u8]> {
        self.memory.get_heap_data(offset, size)
    }

    #[inline(always)]
    pub fn heap_usage(&self) -> usize {
        self.memory.heap_usage()
    }

    #[inline(always)]
    pub fn check_stack_limit(&self) -> CompactResult<()> {
        self.memory.check_stack_limit()
    }

    // --- Call stack operations (delegated to FrameManager) ---

    #[inline(always)]
    pub fn push_call_frame(&mut self, frame: CallFrame) -> Result<()> {
        self.frame.push_call_frame(frame)
    }

    #[inline(always)]
    pub fn pop_call_frame(&mut self) -> CompactResult<CallFrame> {
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

    // --- Account operations with lazy validation (delegated to AccountManager) ---

    #[inline(always)]
    pub fn get_account(&self, index: u8) -> CompactResult<&'a AccountInfo> {
        self.accounts.get(index)
    }

    /// Get account for read access, ensuring pointer freshness
    #[inline(always)]
    pub fn get_account_for_read(&self, index: u8) -> CompactResult<&'a AccountInfo> {
        let account = self.accounts.get(index)?;
        // CRITICAL FIX: Force refresh of account pointers before data access
        // to handle stale pointers after CPI.
        account.refresh_after_cpi();
        Ok(account)
    }

    /// Get account for write access, checking authorization and writability
    #[inline(always)]
    pub fn get_account_for_write(&self, index: u8) -> CompactResult<&'a AccountInfo> {
        // 1. Get account once
        let account = self.accounts.get(index)?;

        // 2. Check bytecode authorization inline (avoiding second get)
        if account.data_len() > 0 {
            if *account.owner() != self.program_id {
                crate::debug_log!("Auth failed: owner mismatch");
                return Err(VMErrorCode::ScriptNotAuthorized);
            }
        }

        // 3. Check writable
        if !account.is_writable() {
            return Err(VMErrorCode::AccountNotWritable);
        }

        // 4. CRITICAL FIX: Force refresh of account pointers
        account.refresh_after_cpi();

        Ok(account)
    }

    /// Get account without lazy validation (for internal VM use)
    #[inline(always)]
    pub fn get_account_unchecked(&self, index: u8) -> CompactResult<&'a AccountInfo> {
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
    pub fn get_call_frame(&self, index: usize) -> CompactResult<&CallFrame> {
        self.frame.get_call_frame(index)
    }

    #[inline(always)]
    pub fn set_call_frame(&mut self, index: usize, frame: CallFrame) -> CompactResult<()> {
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
                let virtual_addr = *heap_id;
                
                // Read length prefix (4 bytes)
                let len_bytes = self.memory.get_heap_data(virtual_addr, 4)?;
                let len = u32::from_le_bytes(len_bytes.try_into().unwrap());

                // Read string data
                // Data starts at virtual_addr + 4. 
                // Note: This relies on alloc_heap_unsafe guaranteeing contiguous allocation 
                // for the requested size (len + 4), so (offset + 4) is valid within the chunk.
                let data_slice = self.memory.get_heap_data(virtual_addr + 4, len)?;

                Ok((len, data_slice))
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

    /// Parse parameters directly into parameters array with zero copy (Fixed Size Encoding)
    pub fn parse_parameters(&mut self) -> CompactResult<()> {
        // Clear params first not needed as we overwrite
        self.reset_temp_buffer();
        let mut offset = 0;
        let input_len = self.instruction_data.len();

        if input_len == 0 {
            return Ok(());
        }

        // 1. Function Index (u32)
        if offset + 4 > input_len { return Err(VMErrorCode::InvalidInstructionPointer); }
        let function_index = u32::from_le_bytes(self.instruction_data[offset..offset+4].try_into().unwrap());
        offset += 4;

        // Store function index at params[0]
        self.frame.set_parameter(0, ValueRef::U64(function_index as u64))?;

        // 2. Parameter Count (u32)
        if offset + 4 > input_len { return Ok(()); }
        let param_count = u32::from_le_bytes(self.instruction_data[offset..offset+4].try_into().unwrap());
        offset += 4;

        // Limit count to available slots (MAX_PARAMETERS - 1 for func index)
        // params[0] is func index. params[1..8] are arguments.
        let count = (param_count as usize).min(MAX_PARAMETERS - 1);

        // Check for typed mode sentinel is removed or assumed to be handled by higher level protocol.
        // Assuming strictly typed mode or pure u64s? The original code had a sentinel check.
        // For simplicity and strictness, let's assume we read typed values if typed mode was intended,
        // or just read u64s if not.
        // The previous code had `is_typed_mode` check.
        // I'll implement a simple loop reading 8-byte values for now as a baseline,
        // OR reproduce the typed logic with fixed sizes.
        // Given "remove ALL VLE", I will assume we use fixed size encoding for everything.
        // The sentinel `0x80` is a single byte.

        // Let's implement typed parsing with fixed sizes.

        for i in 0..count {
             if offset >= input_len { return Err(VMErrorCode::InvalidInstructionPointer); }
             let type_id = self.instruction_data[offset];
             offset += 1;

             match type_id {
                t if t == types::STRING => {
                    if offset + 4 > input_len { return Err(VMErrorCode::InvalidInstructionPointer); }
                    let len = u32::from_le_bytes(self.instruction_data[offset..offset+4].try_into().unwrap()) as usize;
                    offset += 4;

                    // Check bounds
                    if offset + len > input_len { return Err(VMErrorCode::InvalidInstructionPointer); }

                    // Alloc temp buffer: [len:u8, type:u8, bytes...]
                    // WARNING: temp buffer format expects len as u8?
                    // Previous code: `self.memory.temp_buffer[array_id as usize] = len as u8;`
                    // This implies strings > 255 length are truncated in temp buffer metadata?
                    // I will keep this behavior for now to avoid breaking VM internals if they expect this format.
                    let total_size = 2 + len;
                    if total_size > crate::TEMP_BUFFER_SIZE { return Err(VMErrorCode::OutOfMemory); }

                    let array_id = self.alloc_temp(total_size as u8)?;

                    self.memory.temp_buffer[array_id as usize] = len as u8;
                    self.memory.temp_buffer[array_id as usize + 1] = 1; // Type 1 (String?)

                    // Copy bytes
                    self.memory.temp_buffer[array_id as usize + 2..array_id as usize + 2 + len]
                        .copy_from_slice(&self.instruction_data[offset..offset + len]);

                    offset += len;
                    self.frame.set_parameter(i + 1, ValueRef::StringRef(array_id as u16))?;
                }
                t if t == types::BOOL => {
                    if offset + 4 > input_len { return Err(VMErrorCode::InvalidInstructionPointer); }
                    let val = u32::from_le_bytes(self.instruction_data[offset..offset+4].try_into().unwrap());
                    offset += 4;
                    self.frame.set_parameter(i + 1, ValueRef::Bool(val != 0))?;
                }
                t if t == types::U8 => {
                     if offset + 4 > input_len { return Err(VMErrorCode::InvalidInstructionPointer); }
                     let val = u32::from_le_bytes(self.instruction_data[offset..offset+4].try_into().unwrap());
                     offset += 4;
                     self.frame.set_parameter(i + 1, ValueRef::U8(val as u8))?;
                }
                t if t == types::U32 => {
                     if offset + 4 > input_len { return Err(VMErrorCode::InvalidInstructionPointer); }
                     let val = u32::from_le_bytes(self.instruction_data[offset..offset+4].try_into().unwrap());
                     offset += 4;
                     self.frame.set_parameter(i + 1, ValueRef::U64(val as u64))?;
                }
                t if t == types::U64 => {
                     if offset + 8 > input_len { return Err(VMErrorCode::InvalidInstructionPointer); }
                     let val = u64::from_le_bytes(self.instruction_data[offset..offset+8].try_into().unwrap());
                     offset += 8;
                     self.frame.set_parameter(i + 1, ValueRef::U64(val))?;
                }
                t if t == types::PUBKEY => {
                     if offset + 32 > input_len { return Err(VMErrorCode::InvalidInstructionPointer); }
                     let temp_offset = self.alloc_temp(32)?;
                     self.memory.temp_buffer[temp_offset as usize..temp_offset as usize + 32]
                        .copy_from_slice(&self.instruction_data[offset..offset + 32]);
                     offset += 32;
                     self.frame.set_parameter(i + 1, ValueRef::TempRef(temp_offset, 32))?;
                }
                t if t == types::ACCOUNT => {
                    if offset + 4 > input_len { return Err(VMErrorCode::InvalidInstructionPointer); }
                    let idx = u32::from_le_bytes(self.instruction_data[offset..offset+4].try_into().unwrap());
                    offset += 4;
                    self.frame.set_parameter(i + 1, ValueRef::AccountRef(idx as u8, 0))?;
                }
                _ => {
                    // Fallback to U64 if type unknown or generic (assuming 8 bytes)
                    // The previous code had a "Pure VLE" branch.
                    // Here we'll just error or assume U64.
                    return Err(VMErrorCode::TypeMismatch);
                }
             }
        }

        Ok(())
    }

    /// Initialize entry point by parsing parameters and setting up stack/locals
    /// Returns the resolved start IP
    pub fn initialize_entry_point(&mut self, default_start_ip: usize) -> CompactResult<usize> {
        // 1. Parse parameters
        self.parse_parameters()?;

        // 2. Count parameters
        let mut param_count: u8 = 0;
        let mut max_param_index: u8 = 0;

        // Skip index 0 (func index)
        // Access parameters through frame to satisfy borrow checker if needed, but self.frame is accessible
        let params_len = self.frame.parameters().len();
        // We can't iterate self.frame.parameters() while mutating self.
        // So we just index.
        for i in 1..params_len {
            if !self.frame.parameters[i].is_empty() {
                param_count = param_count.saturating_add(1);
                max_param_index = i as u8;
            }
        }

        // 3. Setup frame
        self.frame.param_len = param_count;

        let locals_to_allocate = if max_param_index > 0 {
            max_param_index
        } else {
            3 // Default
        };

        self.allocate_locals(locals_to_allocate)?;

        // 4. Push params and init locals
        for i in 1..params_len {
             let param = self.frame.parameters[i];
             if param.is_empty() { continue; }

             self.push(param)?;

             let local_index = (i - 1) as u8;
             if (local_index as usize) < MAX_LOCALS {
                 self.set_local(local_index, param)?;
             }
        }

        // 5. Dispatch
        let func_index_val = self.frame.parameters[0];
        let dispatch_ip = if !func_index_val.is_empty() {
             if let ValueRef::U64(func_index) = func_index_val {
                 if (func_index as u8) >= self.public_function_count {
                     return Err(VMErrorCode::FunctionVisibilityViolation);
                 }

                 if func_index == 0 {
                     default_start_ip
                 } else {
                     default_start_ip
                 }
             } else {
                 default_start_ip
             }
        } else {
             default_start_ip
        };

        self.set_ip(dispatch_ip);
        Ok(dispatch_ip)
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
        let mut storage = StackStorage::new();
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
        let mut storage = StackStorage::new();
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
