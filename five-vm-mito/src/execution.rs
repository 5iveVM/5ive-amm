//! Core execution engine for MitoVM with function call support
//!
//! Enhanced with minimal function call transport:
//! - 8-level call stack (stack-allocated)
//! - Zero-copy account data access
//! - Enhanced data types for real-world use cases

use crate::{
    context::ExecutionManager, // Import ExecutionManager for VM execution
    debug_log,
    error_log,
    error::{CompactResult, Result, VMErrorCode, VMError},
    handlers::{
        handle_accounts, handle_arithmetic, handle_arrays, handle_constraints, handle_control_flow,
        handle_functions, handle_locals, handle_logical, handle_memory, handle_nibble_locals,
        handle_option_result_ops, handle_registers, handle_stack_ops, handle_system_ops,
    },
    stack_error_context, // Import enhanced debugging macros
    FIVE_MAGIC,
};
use five_protocol::{Value, ValueRef, FIVE_HEADER_OPTIMIZED_SIZE};


use pinocchio::{account_info::AccountInfo, pubkey::Pubkey};
#[cfg(feature = "debug-logs")]
use heapless::String as HString;
// Import all opcodes - using hierarchical match structure to prevent stack overflow
use five_protocol::opcodes::*;

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

/// Ultra-lightweight virtual machine optimized for Solana's execution environment.
/// Features zero-allocation execution, function calls, and sub-50 CU cold start overhead.
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
///     0x20,                   // ADD
///     0x07                    // RETURN_VALUE
/// ];
///
/// let result = MitoVM::execute_direct(bytecode, &[], &[], &Pubkey::default())?;
/// assert_eq!(result, Some(Value::U8(15)));
/// # Ok::<(), five_vm_mito::VMError>(())
/// ```
pub struct MitoVM;

impl MitoVM {
    /// Prepare execution environment with optimized parameter parsing and minimal overhead.
    /// Validates script format, parses VLE parameters, and sets up function dispatch.
    #[inline(never)]
    fn initialize_execution_context<'a>(
        script: &'a [u8],
        input_data: &'a [u8],
        accounts: &'a [AccountInfo],
        program_id: &Pubkey,
        storage: &'a mut crate::stack::StackStorage<'a>,
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

        let (start_ip, public_function_count, total_function_count, header_features) =
            Self::parse_optimized_header(script)?;

        debug_log!("MitoVM: Creating ExecutionManager...");

        debug_log!("MitoVM: Using compile-time defaults");
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
        );
        ctx.set_header_features(header_features);
        ctx.set_ip(start_ip); // Set correct starting position via delegation
        debug_log!(
            "MitoVM: ExecutionManager created with IP set to {}",
            start_ip as u32
        );

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

        // Main execution loop
        #[cfg(feature = "debug-logs")]
        let mut _instruction_count = 0u32;
        loop {
            #[cfg(feature = "debug-logs")]
            {
                _instruction_count += 1;

            }

            // Cache values to avoid simultaneous borrows
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

            let opcode = match ctx.fetch_byte() {
                Ok(op) => op,
                Err(e) => {
                    debug_log!(
                        "MitoVM: Error fetching opcode at IP {}",
                        current_ip as u32
                    );
                    return Err(e);
                }
            };
            
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

            // Set current opcode in context for error reporting
            ctx.set_current_opcode(opcode);

            #[cfg(feature = "debug-logs")]
            if opcode == LOAD_INPUT {
                debug_log!(
                    "MitoVM: CONFIRMED - This is LOAD_INPUT opcode ({})",
                    LOAD_INPUT
                );
            }

            // Dispatch opcode to appropriate handler
            // 🎯 OPTIMIZATION: Flattened dispatch for better BPF performance
            // The compiler will inline the handlers (due to #[inline(always)])
            // and optimize this match into a single jump table or efficient tree,
            // eliminating the double-dispatch overhead.
            let result = match opcode {
                0x00..=0x0F => handle_control_flow(opcode, ctx),
                0x10..=0x1F => handle_stack_ops(opcode, ctx),
                0x20..=0x2F => handle_arithmetic(opcode, ctx),
                0x30..=0x3F => handle_logical(opcode, ctx),
                0x40..=0x4F => handle_memory(opcode, ctx),
                0x50..=0x5F => handle_accounts(opcode, ctx),
                0x60..=0x6F => handle_arrays(opcode, ctx),
                0x70..=0x7F => handle_constraints(opcode, ctx),
                0x80..=0x8F => handle_system_ops(opcode, ctx),
                0x90..=0x9F => handle_functions(opcode, ctx),
                0xA0..=0xAF => handle_locals(opcode, ctx),
                0xB0..=0xBF => handle_registers(opcode, ctx),
                0xC0..=0xCF => {
                    // Account view operations removed - use zero-copy LOAD_FIELD/STORE_FIELD instead
                    debug_log!(
                        "MitoVM: Account view opcode {} removed - use LOAD_FIELD/STORE_FIELD",
                        opcode
                    );
                    Err(VMErrorCode::InvalidInstruction)
                }
                0xD0..=0xDF => handle_nibble_locals(opcode, ctx),
                0xF0..=0xFF => handle_option_result_ops(opcode, ctx),
                _ => {
                    debug_log!(
                        "MitoVM: FATAL_ERROR: UNKNOWN OPCODE RANGE {} at ip {}",
                        opcode,
                        (ctx.ip() - 1) as u32
                    );
                    Err(VMErrorCode::InvalidInstruction)
                }
            };

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

    /// Convert ValueRef (zero-copy reference) to concrete Value using current execution state.
    /// Handles complex references like TempRef, OptionalRef, and AccountRef.
    #[allow(dead_code)]
    #[inline(always)]
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
    ///     0x11, 42,               // PUSH_U8 42
    ///     0x07                    // RETURN_VALUE
    /// ];
    /// let input_data = &[];
    /// let accounts = &[];
    /// let program_id = Pubkey::default();
    ///
    /// let result = MitoVM::execute_direct(bytecode, input_data, accounts, &program_id)?;
    /// assert_eq!(result, Some(five_protocol::Value::U8(42)));
    /// # Ok::<(), five_vm_mito::VMError>(())
    /// ```
    #[inline(never)]
    pub fn execute_direct(
        script: &[u8],
        input_data: &[u8],
        accounts: &[AccountInfo],
        program_id: &Pubkey,
    ) -> Result<Option<Value>> {
        // Allocate storage on HEAP using optimized initialization (no stack copy)
        // This solves both the Stack Overflow (by using heap) and the 5k CU regression (by avoiding memcpy)
        let mut storage = crate::stack::StackStorage::new_on_heap(script);
        
        let (mut ctx, _dispatch_ip) =
            Self::initialize_execution_context(script, input_data, accounts, program_id, &mut storage)?;
        let execution_result = Self::execute_instruction_loop(&mut ctx);
        match execution_result {
            Ok(()) => {
                let result = crate::resolution::finalize_execution_result(&mut ctx)
                    .map_err(VMError::from);
                // Clear temp buffer to avoid reusing stale data between runs
                ctx.reset_temp_buffer();
                result
            }
            Err(e) => Err(VMError::from(e)),
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
        let mut storage = crate::stack::StackStorage::new(script);
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
    /// Returns (instruction_pointer_start, public_function_count, total_function_count, features)
    #[inline]
    fn parse_optimized_header(script: &[u8]) -> CompactResult<(usize, u8, u8, u32)> {
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

        let start_ip = Self::compute_instruction_start_fast(script, features, public_function_count);

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
        ))
    }

    /// Fast metadata offset computation
    #[inline]
    fn compute_instruction_start_fast(script: &[u8], features: u32, public_count: u8) -> usize {
        const FEATURE_FUNCTION_NAMES: u32 = 1 << 8;

        if (features & FEATURE_FUNCTION_NAMES) == 0 || public_count == 0 {
            return FIVE_HEADER_OPTIMIZED_SIZE;
        }

        // Metadata format was validated at deploy-time
        let mut offset = FIVE_HEADER_OPTIMIZED_SIZE;
        let mut section_size = 0u16;
        let mut shift = 0;

        while offset < script.len() && shift < 16 {
            let byte = script[offset];
            section_size |= ((byte & 0x7F) as u16) << shift;
            offset += 1;
            if byte & 0x80 == 0 {
                break;
            }
            shift += 7;
        }

        (offset + section_size as usize).min(script.len())
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
        let (start_ip, public_function_count, total_function_count, features) =
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
        let (start_ip, public_count, total_count, _) = result.unwrap();
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
