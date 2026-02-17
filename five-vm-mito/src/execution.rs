//! Core execution engine for MitoVM with function call support.

use crate::{
    context::ExecutionManager,
    debug_log,
    error_log,
    error::{CompactResult, Result, VMErrorCode, VMError},
    handlers::{
        handle_accounts, handle_arithmetic, handle_arrays, handle_constraints, handle_control_flow,
        handle_functions, handle_locals, handle_logical, handle_memory, handle_nibble_locals,
        handle_option_result_ops, handle_stack_ops, handle_system_ops,
    },
    stack_error_context,
    FIVE_MAGIC,
};
use five_protocol::{ConstantPoolDescriptor, Value, ValueRef, FIVE_HEADER_OPTIMIZED_SIZE};


use pinocchio::{account_info::AccountInfo, pubkey::Pubkey};
#[cfg(not(target_os = "solana"))]
use std::sync::atomic::{AtomicU64, Ordering};
#[cfg(feature = "debug-logs")]
use heapless::String as HString;
#[cfg(not(target_os = "solana"))]
static LAST_COMPUTE_UNITS: AtomicU64 = AtomicU64::new(0);
#[cfg(not(target_os = "solana"))]
static LAST_EXTERNAL_CACHE_HITS: AtomicU64 = AtomicU64::new(0);
#[cfg(not(target_os = "solana"))]
static LAST_EXTERNAL_CACHE_MISSES: AtomicU64 = AtomicU64::new(0);
#[cfg(not(target_os = "solana"))]
static LAST_IMPORT_VERIFY_CACHE_HITS: AtomicU64 = AtomicU64::new(0);

/// Execution state snapshot returned from VM operations.
/// Used primarily for WASM integration and external monitoring.
#[derive(Debug)]
#[cfg(not(target_os = "solana"))]
pub struct VMExecutionContext {
    pub instruction_pointer: usize,
    pub halted: bool,
    pub error: Option<VMError>,
    pub memory: [u8; crate::TEMP_BUFFER_SIZE],
    pub failed_opcode: Option<u8>,
}

/// Virtual machine optimized for Solana's execution environment.
///
/// # Example
/// ```rust,no_run
/// use five_vm_mito::{MitoVM, Value};
/// use pinocchio::account_info::AccountInfo;
/// use pinocchio::pubkey::Pubkey;
///
/// // Execute simple arithmetic: push 10, push 5, add them
/// // FIVE header (10 bytes): magic(4) + features(4) + public_count(1) + total_count(1)
/// let bytecode = &[
///     b'5', b'I', b'V', b'E', // FIVE magic
///     0x00, 0x00, 0x00, 0x00, // features
///     0x01,                   // public_count
///     0x01,                   // total_count
///     0x11, 10,               // PUSH_U8 10
///     0x11, 5,                // PUSH_U8 5
///     0x07                    // RETURN_VALUE
/// ];
///
/// let mut storage = five_vm_mito::StackStorage::new();
/// let result = MitoVM::execute_direct(bytecode, &[], &[], &Pubkey::default(), &mut storage)?;
/// assert_eq!(result, Some(Value::U8(15)));
/// # Ok::<(), five_vm_mito::VMError>(())
/// ```
pub struct MitoVM;

impl MitoVM {
    #[cfg(not(target_os = "solana"))]
    #[inline]
    pub fn last_compute_units_consumed() -> u64 {
        LAST_COMPUTE_UNITS.load(Ordering::Relaxed)
    }

    #[cfg(not(target_os = "solana"))]
    #[inline]
    pub fn last_external_cache_metrics() -> (u64, u64, u64) {
        (
            LAST_EXTERNAL_CACHE_HITS.load(Ordering::Relaxed),
            LAST_EXTERNAL_CACHE_MISSES.load(Ordering::Relaxed),
            LAST_IMPORT_VERIFY_CACHE_HITS.load(Ordering::Relaxed),
        )
    }

    /// Prepare execution environment with fixed-width parameters and function dispatch.
    #[inline(never)]
    fn initialize_execution_context<'a>(
        script: &'a [u8],
        input_data: &'a [u8],
        accounts: &'a [AccountInfo],
        program_id: &Pubkey,
        storage: &'a mut crate::stack::StackStorage,
    ) -> CompactResult<(ExecutionManager<'a>, usize)> {
        #[cfg(feature = "debug-logs")]
        use core::fmt::Write;
        #[cfg(feature = "debug-logs")]
        {
            debug_log!("MitoVM: ===== EXECUTE_DIRECT ENTRY POINT =====");
            debug_log!("MitoVM: Starting enhanced execution with function call support");
            debug_log!("MitoVM: Script length: {} bytes", script.len() as u32);
            debug_log!(
                "MitoVM: Input data length: {} bytes",
                input_data.len() as u32
            );
            debug_log!("MitoVM: Account count: {}", accounts.len() as u32);
        }

        let (
            start_ip,
            public_function_count,
            total_function_count,
            header_features,
            pool_desc,
            public_entry_table,
        ) =
            Self::parse_optimized_header(script)?;

        debug_log!("MitoVM: Creating ExecutionManager...");
        debug_log!(
            "MitoVM: Function counts from header: {} public, {} total",
            public_function_count as u32,
            total_function_count as u32
        );
        let mut ctx = ExecutionManager::new(
            script,
            accounts,
            *program_id,
            input_data,
            start_ip as u16,
            storage,
            public_function_count,
            total_function_count,
            pool_desc.map(|d| d.pool_offset).unwrap_or(0),
            pool_desc.map(|d| d.pool_slots).unwrap_or(0),
            pool_desc.map(|d| d.string_blob_offset).unwrap_or(0),
            pool_desc.map(|d| d.string_blob_len).unwrap_or(0),
        );
        ctx.set_header_features(header_features);
        if let Some((offset, count)) = public_entry_table {
            ctx.set_public_entry_table(offset, count);
        }
        let import_metadata_offset = if (header_features & five_protocol::FEATURE_IMPORT_VERIFICATION) != 0 {
            if let Some(desc) = pool_desc {
                (desc.string_blob_offset as usize).saturating_add(desc.string_blob_len as usize)
            } else {
                script.len()
            }
        } else {
            script.len()
        };
        ctx.set_import_metadata_offset(import_metadata_offset)?;
        ctx.set_ip(start_ip);
        debug_log!("MitoVM: ExecutionManager created with IP set to {}", start_ip as u32);

        let dispatch_ip = ctx.initialize_entry_point(start_ip)?;

        debug_log!(
            "MitoVM: Context initialized, starting execution at ip {}",
            ctx.ip() as u32
        );

        Ok((ctx, dispatch_ip))
    }

    /// Core execution loop that fetches and executes bytecode instructions until halt or error.
    #[inline(never)]
    fn execute_instruction_loop(ctx: &mut ExecutionManager) -> CompactResult<()> {
        debug_log!("MitoVM: ===== BEGINNING EXECUTION LOOP =====");

        // Main execution loop.
        #[cfg(feature = "debug-logs")]
        let mut _instruction_count = 0u32;
        loop {
            #[cfg(feature = "debug-logs")]
            {
                _instruction_count += 1;

            }

            // Cache values to avoid simultaneous borrows.
            let current_ip = ctx.ip();
            let script_len = ctx.script().len();
            let is_halted = ctx.halted();

            if current_ip >= script_len {
                debug_log!("MitoVM: Reached end of script, breaking execution loop");
                break;
            }

            if is_halted {
                debug_log!("MitoVM: VM halted, breaking execution loop");
                break;
            }

            // SAFETY: Bounds checked above (current_ip >= script_len).
            let opcode = unsafe { *ctx.bytecode.get_unchecked(current_ip) };
            ctx.pc += 1;

            /*
            #[cfg(feature = "trace-execution")]
            {
               debug_log!(
                   "MitoVM: EXEC LOOP - Opcode: {} at IP: {}", 
                   opcode, 
                   current_ip
               );
               if opcode == 0 { // Just to make sure it's reachable and we panic
                   panic!("PANIC_TRACE_ENABLED");
               }
            }
            */

            // Track opcode only in off-chain builds where it is read by snapshots/debug tooling.
            #[cfg(not(target_os = "solana"))]
            ctx.set_current_opcode(opcode);

            let result = Self::dispatch_opcode(opcode, ctx);

            // Check result and provide clear error context
            if let Err(e) = result {
                // Enhanced error context with full VM state
                stack_error_context!(opcode, ctx, "EXECUTION_FAILED", 0, ctx.size());
                error_log!("MitoVM: ERROR_DETAILS: error_occurred at current_ip: {}", current_ip as u64);
                error_log!("OPCODE FAILED: {}", opcode as u64);
                error_log!("Stack size: {}", ctx.size() as u64);
                return Err(e);
            }

            // Check halted flag immediately after opcode execution
            let post_execution_halted = ctx.halted();
            if post_execution_halted {
                debug_log!(
                    "MitoVM: VM halted after opcode {} execution, breaking loop",
                    opcode
                );
                debug_log!(
                    "🔍 EXECUTION_TRACE: VM halted after opcode {} at IP {}",
                    opcode,
                    ctx.ip() as u32
                );
                break;
            }
        }


        Ok(())
    }

    #[inline(always)]
    fn dispatch_opcode(opcode: u8, ctx: &mut ExecutionManager) -> CompactResult<()> {
        match opcode & 0xF0 {
            0x00 => handle_control_flow(opcode, ctx),
            0x10 | 0xB0 => Self::dispatch_stack_sparse(opcode, ctx),
            0x20 => handle_arithmetic(opcode, ctx),
            0x30 => handle_logical(opcode, ctx),
            0x40 => Self::dispatch_memory(opcode, ctx),
            0x50 => handle_accounts(opcode, ctx),
            0x60 => Self::dispatch_arrays_compat(opcode, ctx),
            0x70 => Self::dispatch_constraints_compat(opcode, ctx),
            0x80 => Self::dispatch_system_compat(opcode, ctx),
            0x90 => Self::dispatch_functions_compat(opcode, ctx),
            0xA0 => Self::dispatch_locals_sparse(opcode, ctx),
            0xC0 | 0xE0 => Self::dispatch_fused(opcode, ctx),
            0xD0 => handle_nibble_locals(opcode, ctx),
            0xF0 => handle_option_result_ops(opcode, ctx),
            _ => Self::dispatch_invalid(opcode, ctx),
        }
    }

    #[inline(always)]
    fn dispatch_stack_sparse(opcode: u8, ctx: &mut ExecutionManager) -> CompactResult<()> {
        handle_stack_ops(opcode, ctx)
    }

    #[inline(always)]
    fn dispatch_locals_sparse(opcode: u8, ctx: &mut ExecutionManager) -> CompactResult<()> {
        handle_locals(opcode, ctx)
    }

    #[inline(always)]
    fn dispatch_invalid(_opcode: u8, _ctx: &ExecutionManager) -> CompactResult<()> {
        error_log!(
            "MitoVM: FATAL_ERROR: UNKNOWN/UNIMPLEMENTED OPCODE {} at ip {}",
            _opcode,
            (_ctx.ip() - 1) as u32
        );
        Err(VMErrorCode::InvalidInstruction)
    }

    #[inline(never)]
    fn dispatch_memory(opcode: u8, ctx: &mut ExecutionManager) -> CompactResult<()> {
        handle_memory(opcode, ctx)
    }

    #[inline(never)]
    fn dispatch_arrays(opcode: u8, ctx: &mut ExecutionManager) -> CompactResult<()> {
        handle_arrays(opcode, ctx)
    }

    #[inline(always)]
    fn dispatch_arrays_compat(opcode: u8, ctx: &mut ExecutionManager) -> CompactResult<()> {
        match Self::dispatch_arrays(opcode, ctx) {
            Err(VMErrorCode::InvalidInstruction) => handle_constraints(opcode, ctx),
            other => other,
        }
    }

    #[inline(always)]
    fn dispatch_constraints_compat(opcode: u8, ctx: &mut ExecutionManager) -> CompactResult<()> {
        match handle_constraints(opcode, ctx) {
            Err(VMErrorCode::InvalidInstruction) => Self::dispatch_system(opcode, ctx),
            other => other,
        }
    }

    #[inline(never)]
    fn dispatch_system(opcode: u8, ctx: &mut ExecutionManager) -> CompactResult<()> {
        handle_system_ops(opcode, ctx)
    }

    #[inline(always)]
    fn dispatch_system_compat(opcode: u8, ctx: &mut ExecutionManager) -> CompactResult<()> {
        match Self::dispatch_system(opcode, ctx) {
            Err(VMErrorCode::InvalidInstruction) => Self::dispatch_functions(opcode, ctx),
            other => other,
        }
    }

    #[inline(never)]
    fn dispatch_functions(opcode: u8, ctx: &mut ExecutionManager) -> CompactResult<()> {
        handle_functions(opcode, ctx)
    }

    #[inline(always)]
    fn dispatch_functions_compat(opcode: u8, ctx: &mut ExecutionManager) -> CompactResult<()> {
        match Self::dispatch_functions(opcode, ctx) {
            Err(VMErrorCode::InvalidInstruction) => Self::dispatch_locals_sparse(opcode, ctx),
            other => other,
        }
    }

    #[inline(always)]
    fn dispatch_fused(opcode: u8, ctx: &mut ExecutionManager) -> CompactResult<()> {
        crate::handlers::fused_ops::handle_fused_ops(opcode, ctx)
    }

    /// Convert ValueRef (zero-copy reference) to concrete Value using current execution state.
    /// Handles complex references like TempRef, OptionalRef, and AccountRef.
    #[allow(dead_code)]
    #[inline(never)]
    pub fn resolve_value_ref(value_ref: &ValueRef, ctx: &ExecutionManager<'_>) -> CompactResult<Value> {
        // Delegate to resolution module
        crate::resolution::resolve_value_ref(value_ref, ctx)
    }

    /// Execute bytecode directly with minimal overhead.
    /// Returns optional value from RETURN_VALUE opcode or None if script completes without explicit return.
    ///
    /// # Example
    /// ```rust,no_run
    /// use five_vm_mito::MitoVM;
    /// use pinocchio::account_info::AccountInfo;
    /// use pinocchio::pubkey::Pubkey;
    ///
    /// // Simple bytecode that pushes 42 and returns it
    /// // FIVE header (10 bytes): magic(4) + features(4) + public_count(1) + total_count(1)
    /// let bytecode = &[
    ///     b'5', b'I', b'V', b'E', // FIVE magic
    ///     0x00, 0x00, 0x00, 0x00, // features
    ///     0x01,                   // public_count
    ///     0x01,                   // total_count
    ///     0x07                    // RETURN_VALUE
    /// ];
    /// let input_data = &[];
    /// let accounts = &[];
    /// let program_id = Pubkey::default();
    ///
    /// let mut storage = five_vm_mito::StackStorage::new();
    /// let result = MitoVM::execute_direct(bytecode, input_data, accounts, &program_id, &mut storage)?;
    /// assert_eq!(result, Some(five_protocol::Value::U8(42)));
    /// # Ok::<(), five_vm_mito::VMError>(())
    /// ```
    #[inline(never)]
    pub fn execute_direct<'a>(
        script: &'a [u8],
        input_data: &'a [u8],
        accounts: &'a [AccountInfo],
        program_id: &Pubkey,
        storage: &'a mut crate::stack::StackStorage,
    ) -> Result<Option<Value>> {
        // Use provided storage buffer (caller controlled allocation)
        let (mut ctx, _dispatch_ip) =
            Self::initialize_execution_context(script, input_data, accounts, program_id, storage)?;
        let execution_result = Self::execute_instruction_loop(&mut ctx);
        match execution_result {
            Ok(()) => {
                let result = crate::resolution::finalize_execution_result(&mut ctx)
                    .map_err(VMError::from);
                #[cfg(not(target_os = "solana"))]
                {
                    let (hits, misses, verify_hits) = ctx.external_cache_metrics();
                    LAST_EXTERNAL_CACHE_HITS.store(hits as u64, Ordering::Relaxed);
                    LAST_EXTERNAL_CACHE_MISSES.store(misses as u64, Ordering::Relaxed);
                    LAST_IMPORT_VERIFY_CACHE_HITS.store(verify_hits as u64, Ordering::Relaxed);
                }
                #[cfg(not(target_os = "solana"))]
                LAST_COMPUTE_UNITS.store(ctx.compute_units_consumed(), Ordering::Relaxed);
                // Clear temp buffer to avoid reusing stale data between runs
                ctx.reset_temp_buffer();
                result
            }
            Err(e) => {
                #[cfg(not(target_os = "solana"))]
                {
                    let (hits, misses, verify_hits) = ctx.external_cache_metrics();
                    LAST_EXTERNAL_CACHE_HITS.store(hits as u64, Ordering::Relaxed);
                    LAST_EXTERNAL_CACHE_MISSES.store(misses as u64, Ordering::Relaxed);
                    LAST_IMPORT_VERIFY_CACHE_HITS.store(verify_hits as u64, Ordering::Relaxed);
                }
                #[cfg(not(target_os = "solana"))]
                LAST_COMPUTE_UNITS.store(ctx.compute_units_consumed(), Ordering::Relaxed);
                Err(VMError::from(e))
            }
        }
    }

    /// Execute bytecode and return both result and execution state snapshot.
    /// Primarily used for WASM integration and external debugging tools.
    ///
    /// # Example
    /// ```rust,no_run
    /// use five_vm_mito::MitoVM;
    /// use pinocchio::pubkey::Pubkey;
    ///
    /// // FIVE header (10 bytes): magic(4) + features(4) + public_count(1) + total_count(1)
    /// let bytecode = &[
    ///     b'5', b'I', b'V', b'E', // FIVE magic
    ///     0x00, 0x00, 0x00, 0x00, // features
    ///     0x01,                   // public_count
    ///     0x01,                   // total_count
    ///     0x10, 100, 0, 0, 0, 0, 0, 0, 0 // PUSH_U64 100
    /// ];
    /// #[cfg(not(target_os = "solana"))]
    /// {
    ///     let program_id = Pubkey::default();
    ///     let (result, context) = MitoVM::execute_with_context(bytecode, &[], &[], &program_id).unwrap();
    ///     assert!(!context.halted);
    ///     assert_eq!(context.error, None);
    /// }
    /// # Ok::<(), five_vm_mito::VMError>(())
    /// ```
    #[inline(never)]
    #[cfg(not(target_os = "solana"))]
    pub fn execute_with_context(
        script: &[u8],
        input_data: &[u8],
        accounts: &[AccountInfo],
        program_id: &Pubkey,
    ) -> std::result::Result<(Option<Value>, VMExecutionContext), (VMError, VMExecutionContext)> {
        let mut storage = crate::stack::StackStorage::new();
        // Map initialization error to (VMError, EmptyContext) since we can't create a meaningful context yet
        let (mut ctx, _dispatch_ip) =
            Self::initialize_execution_context(script, input_data, accounts, program_id, &mut storage).map_err(
                |e| {
                    (
                        VMError::from(e),
                        VMExecutionContext {
                            instruction_pointer: 0,
                            halted: false,
                            error: Some(VMError::from(e)),
                            memory: [0u8; crate::TEMP_BUFFER_SIZE],
                            failed_opcode: None,
                        },
                    )
                },
            )?;

        #[cfg(feature = "debug-logs")]
        debug_log!(
            "MitoVM: INIT complete - func_count={}, call_depth={}, param_len={}, stack_size={}",
            ctx.total_function_count() as u32,
            ctx.call_depth() as u32,
            ctx.param_len() as u32,
            ctx.size() as u32
        );

        let execution_result = Self::execute_instruction_loop(&mut ctx);

        let final_result = match execution_result {
            Ok(()) => {
                // Do NOT reset temp buffer here, as we want to return it in the context
                crate::resolution::finalize_execution_result(&mut ctx).map_err(VMError::from)
            }
            Err(e) => Err(VMError::from(e)),
        };

        match final_result {
            Ok(result) => {
                // Cache values to avoid borrow conflicts
                let final_ip = ctx.ip();
                let final_halted = ctx.halted();
                let mut memory = [0u8; crate::TEMP_BUFFER_SIZE];
                memory.copy_from_slice(ctx.temp_buffer());
                let success_context = VMExecutionContext {
                    instruction_pointer: final_ip,
                    halted: final_halted,
                    error: None,
                    memory,
                    failed_opcode: None,
                };
                Ok((result, success_context))
            }
            Err(e) => {
                #[cfg(feature = "debug-logs")]
                {
                    let mut s = HString::<256>::new();
                    let _ = core::fmt::write(&mut s, format_args!("{:?}", e));
                    debug_log!(
                        "MitoVM: FINALIZE error {} - stack_size={}, param_len={}, call_depth={}",
                        s.as_str(),
                        ctx.size() as u32,
                        ctx.param_len() as u32,
                        ctx.call_depth() as u32
                    );
                }

                // Capture execution context even on error
                let final_ip = ctx.ip();
                let final_halted = ctx.halted();
                let failed_opcode = ctx.current_opcode();
                let mut memory = [0u8; crate::TEMP_BUFFER_SIZE];
                memory.copy_from_slice(ctx.temp_buffer());
                let error_context = VMExecutionContext {
                    instruction_pointer: final_ip,
                    halted: final_halted,
                    error: Some(e.clone()),
                    memory,
                    failed_opcode,
                };

                Err((e, error_context))
            }
        }
    }

    /// Parse optimized script header (10 bytes)
    /// Returns (instruction_pointer_start, public_function_count, total_function_count, features, pool_desc, public_entry_table)
    #[inline]
    fn parse_optimized_header(
        script: &[u8],
    ) -> CompactResult<(
        usize,
        u8,
        u8,
        u32,
        Option<ConstantPoolDescriptor>,
        Option<(u32, u8)>,
    )> {
        if script.len() < FIVE_HEADER_OPTIMIZED_SIZE {
            return Err(VMErrorCode::InvalidScript);
        }

        if script[0..4] != FIVE_MAGIC {
            return Err(VMErrorCode::InvalidScript);
        }

        let features = u32::from_le_bytes([script[4], script[5], script[6], script[7]]);
        let public_function_count = script[8];
        let total_function_count = script[9];

        if public_function_count > total_function_count {
            return Err(VMErrorCode::InvalidScript);
        }

        // Each function needs at least 1 byte, so total_count can't exceed available space
        let available_space = script.len().saturating_sub(FIVE_HEADER_OPTIMIZED_SIZE);
        if (total_function_count as usize) > available_space {
            return Err(VMErrorCode::InvalidScript);
        }

        let (metadata_end, public_entry_table) =
            Self::parse_metadata_sections(script, features, public_function_count)?;
        let mut start_ip = metadata_end;
        let mut pool_desc = None;

        if (features & five_protocol::FEATURE_CONSTANT_POOL) != 0 {
            let desc_size = core::mem::size_of::<ConstantPoolDescriptor>();
            if metadata_end + desc_size > script.len() {
                return Err(VMErrorCode::InvalidScript);
            }

            let desc = ConstantPoolDescriptor {
                pool_offset: u32::from_le_bytes([
                    script[metadata_end],
                    script[metadata_end + 1],
                    script[metadata_end + 2],
                    script[metadata_end + 3],
                ]),
                string_blob_offset: u32::from_le_bytes([
                    script[metadata_end + 4],
                    script[metadata_end + 5],
                    script[metadata_end + 6],
                    script[metadata_end + 7],
                ]),
                string_blob_len: u32::from_le_bytes([
                    script[metadata_end + 8],
                    script[metadata_end + 9],
                    script[metadata_end + 10],
                    script[metadata_end + 11],
                ]),
                pool_slots: u16::from_le_bytes([script[metadata_end + 12], script[metadata_end + 13]]),
                reserved: u16::from_le_bytes([script[metadata_end + 14], script[metadata_end + 15]]),
            };

            let pool_offset = desc.pool_offset as usize;
            if pool_offset % 8 != 0 {
                return Err(VMErrorCode::InvalidScript);
            }
            if pool_offset < metadata_end + desc_size {
                return Err(VMErrorCode::InvalidScript);
            }
            let pool_size = (desc.pool_slots as usize) * 8;
            let code_offset = pool_offset + pool_size;
            if code_offset > script.len() {
                return Err(VMErrorCode::InvalidScript);
            }

            if desc.string_blob_len > 0 {
                let blob_offset = desc.string_blob_offset as usize;
                let blob_end = blob_offset.saturating_add(desc.string_blob_len as usize);
                if blob_end > script.len() {
                    return Err(VMErrorCode::InvalidScript);
                }
            }

            start_ip = code_offset;
            pool_desc = Some(desc);
        }

        #[cfg(feature = "debug-logs")]
        {
            let mut hex_str = HString::<16>::new();
            let _ = core::fmt::write(&mut hex_str, format_args!("{:02X}", features));
            debug_log!(
                "MitoVM: Header parsed (features=0x{}, public={}, total={}), start_ip={}",
                hex_str.as_str(),
                public_function_count,
                total_function_count,
                start_ip as u32
            );
        }

        Ok((
            start_ip,
            public_function_count,
            total_function_count,
            features,
            pool_desc,
            public_entry_table,
        ))
    }

    /// Parse metadata sections and return final offset + optional public-entry descriptor.
    #[inline]
    fn parse_metadata_sections(
        script: &[u8],
        features: u32,
        public_count: u8,
    ) -> CompactResult<(usize, Option<(u32, u8)>)> {
        let mut offset = FIVE_HEADER_OPTIMIZED_SIZE;
        let mut public_entry = None;

        if (features & five_protocol::FEATURE_FUNCTION_NAMES) != 0 && public_count > 0 {
            if offset + 2 > script.len() {
                return Err(VMErrorCode::InvalidScript);
            }
            let section_size = u16::from_le_bytes([script[offset], script[offset + 1]]) as usize;
            offset += 2;
            if offset + section_size > script.len() {
                return Err(VMErrorCode::InvalidScript);
            }
            offset += section_size;
        }

        if (features & five_protocol::FEATURE_PUBLIC_ENTRY_TABLE) != 0 {
            if offset + 2 > script.len() {
                return Err(VMErrorCode::InvalidScript);
            }
            let section_size = u16::from_le_bytes([script[offset], script[offset + 1]]) as usize;
            offset += 2;
            if section_size == 0 || offset + section_size > script.len() {
                return Err(VMErrorCode::InvalidScript);
            }
            let count = script[offset];
            let expected = 1usize + (count as usize) * 2;
            if expected > section_size || count > public_count {
                return Err(VMErrorCode::InvalidScript);
            }
            public_entry = Some((offset as u32, count));
            offset += section_size;
        }

        Ok((offset, public_entry))
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_script(public_count: u8, total_count: u8, body: &[u8]) -> Vec<u8> {
        // Header V3: magic(4) + features(4 bytes LE) + public_count(1) + total_count(1)
        let mut script = vec![
            b'5',
            b'I',
            b'V',
            b'E',
            0x00,
            0x00,
            0x00,
            0x00,
            public_count,
            total_count,
        ];
        script.extend_from_slice(body);
        script
    }

    #[test]
    fn parse_optimized_header_success() {
        let script = build_script(3, 3, &[0x00, 0x00, 0x00]);
        let (start_ip, public_function_count, total_function_count, features, _, _) =
            MitoVM::parse_optimized_header(&script).unwrap();
        assert_eq!(start_ip, FIVE_HEADER_OPTIMIZED_SIZE);
        assert_eq!(public_function_count, 3);
        assert_eq!(total_function_count, 3);
        assert_eq!(features, 0);
    }

    #[test]
    fn parse_optimized_header_with_valid_bytes() {
        let script = vec![
            b'5', b'I', b'V', b'E',  // magic
            0x00, 0x00, 0x00, 0x00,  // features
            0x01,                     // public_count
            0x01,                     // total_count
            0x00, 0x00,               // extra bytes
        ];
        let result = MitoVM::parse_optimized_header(&script);
        assert!(result.is_ok());
        let (start_ip, public_count, total_count, _, _, _) = result.unwrap();
        assert_eq!(public_count, 1);
        assert_eq!(total_count, 1);
        assert_eq!(start_ip, 10);
    }

    #[test]
    fn parse_optimized_header_minimum_size() {
        let script = vec![
            b'5', b'I', b'V', b'E',  // magic
            0x00, 0x00, 0x00, 0x00,  // features
            0x00,                     // public_count (0)
            0x00,                     // total_count (0)
        ];
        let result = MitoVM::parse_optimized_header(&script);
        assert!(result.is_ok());
    }
}
