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
// Import all opcodes - using hierarchical match structure to prevent stack overflow.
use five_protocol::opcodes::*;

#[cfg(not(target_os = "solana"))]
static LAST_COMPUTE_UNITS: AtomicU64 = AtomicU64::new(0);

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

            // Dispatch opcode to appropriate handler.
            let result = match opcode {
                // Control Flow (0x00-0x0F)
                HALT => handle_control_flow(HALT, ctx),
                JUMP => handle_control_flow(JUMP, ctx),
                JUMP_IF => handle_control_flow(JUMP_IF, ctx),
                JUMP_IF_NOT => handle_control_flow(JUMP_IF_NOT, ctx),
                REQUIRE => handle_control_flow(REQUIRE, ctx),
                ASSERT => handle_control_flow(ASSERT, ctx),
                RETURN => handle_control_flow(RETURN, ctx),
                RETURN_VALUE => handle_control_flow(RETURN_VALUE, ctx),
                NOP => handle_control_flow(NOP, ctx),
                BR_EQ_U8 => handle_control_flow(BR_EQ_U8, ctx),
                CMP_EQ_JUMP => handle_control_flow(CMP_EQ_JUMP, ctx),
                DEC_JUMP_NZ => handle_control_flow(DEC_JUMP_NZ, ctx),

                // Stack Operations (0x10-0x1F)
                POP => handle_stack_ops(POP, ctx),
                DUP => handle_stack_ops(DUP, ctx),
                DUP2 => handle_stack_ops(DUP2, ctx),
                SWAP => handle_stack_ops(SWAP, ctx),
                PICK => handle_stack_ops(PICK, ctx),
                ROT => handle_stack_ops(ROT, ctx),
                DROP => handle_stack_ops(DROP, ctx),
                OVER => handle_stack_ops(OVER, ctx),
                PUSH_U8 => handle_stack_ops(PUSH_U8, ctx),
                PUSH_U16 => handle_stack_ops(PUSH_U16, ctx),
                PUSH_U32 => handle_stack_ops(PUSH_U32, ctx),
                PUSH_U64 => handle_stack_ops(PUSH_U64, ctx),
                PUSH_I64 => handle_stack_ops(PUSH_I64, ctx),
                PUSH_BOOL => handle_stack_ops(PUSH_BOOL, ctx),
                PUSH_PUBKEY => handle_stack_ops(PUSH_PUBKEY, ctx),
                PUSH_U128 => handle_stack_ops(PUSH_U128, ctx),
                PUSH_U8_W => handle_stack_ops(PUSH_U8_W, ctx),
                PUSH_U16_W => handle_stack_ops(PUSH_U16_W, ctx),
                PUSH_U32_W => handle_stack_ops(PUSH_U32_W, ctx),
                PUSH_U64_W => handle_stack_ops(PUSH_U64_W, ctx),
                PUSH_I64_W => handle_stack_ops(PUSH_I64_W, ctx),
                PUSH_BOOL_W => handle_stack_ops(PUSH_BOOL_W, ctx),
                PUSH_PUBKEY_W => handle_stack_ops(PUSH_PUBKEY_W, ctx),
                PUSH_U128_W => handle_stack_ops(PUSH_U128_W, ctx),

                // Arithmetic Operations (0x20-0x2F)
                ADD => handle_arithmetic(ADD, ctx),
                SUB => handle_arithmetic(SUB, ctx),
                MUL => handle_arithmetic(MUL, ctx),
                DIV => handle_arithmetic(DIV, ctx),
                MOD => handle_arithmetic(MOD, ctx),
                GT => handle_arithmetic(GT, ctx),
                LT => handle_arithmetic(LT, ctx),
                EQ => handle_arithmetic(EQ, ctx),
                GTE => handle_arithmetic(GTE, ctx),
                LTE => handle_arithmetic(LTE, ctx),
                NEQ => handle_arithmetic(NEQ, ctx),
                NEG => handle_arithmetic(NEG, ctx),
                ADD_CHECKED => handle_arithmetic(ADD_CHECKED, ctx),
                SUB_CHECKED => handle_arithmetic(SUB_CHECKED, ctx),
                MUL_CHECKED => handle_arithmetic(MUL_CHECKED, ctx),

                // Logical Operations (0x30-0x3F)
                AND => handle_logical(AND, ctx),
                OR => handle_logical(OR, ctx),
                NOT => handle_logical(NOT, ctx),
                XOR => handle_logical(XOR, ctx),
                BITWISE_NOT => handle_logical(BITWISE_NOT, ctx),
                BITWISE_AND => handle_logical(BITWISE_AND, ctx),
                BITWISE_OR => handle_logical(BITWISE_OR, ctx),
                BITWISE_XOR => handle_logical(BITWISE_XOR, ctx),
                SHIFT_LEFT => handle_logical(SHIFT_LEFT, ctx),
                SHIFT_RIGHT => handle_logical(SHIFT_RIGHT, ctx),
                SHIFT_RIGHT_ARITH => handle_logical(SHIFT_RIGHT_ARITH, ctx),
                ROTATE_LEFT => handle_logical(ROTATE_LEFT, ctx),
                ROTATE_RIGHT => handle_logical(ROTATE_RIGHT, ctx),
                BYTE_SWAP_16 => handle_logical(BYTE_SWAP_16, ctx),
                BYTE_SWAP_32 => handle_logical(BYTE_SWAP_32, ctx),
                BYTE_SWAP_64 => handle_logical(BYTE_SWAP_64, ctx),

                // Memory Operations (0x40-0x4F)
                STORE => handle_memory(STORE, ctx),
                LOAD => handle_memory(LOAD, ctx),
                STORE_FIELD => handle_memory(STORE_FIELD, ctx),
                LOAD_FIELD => handle_memory(LOAD_FIELD, ctx),
                LOAD_INPUT => handle_memory(LOAD_INPUT, ctx),
                STORE_GLOBAL => handle_memory(STORE_GLOBAL, ctx),
                LOAD_GLOBAL => handle_memory(LOAD_GLOBAL, ctx),
                LOAD_EXTERNAL_FIELD => handle_memory(LOAD_EXTERNAL_FIELD, ctx),
                LOAD_FIELD_PUBKEY => handle_memory(LOAD_FIELD_PUBKEY, ctx),

                // Account Operations (0x50-0x5F)
                CREATE_ACCOUNT => handle_accounts(CREATE_ACCOUNT, ctx),
                LOAD_ACCOUNT => handle_accounts(LOAD_ACCOUNT, ctx),
                SAVE_ACCOUNT => handle_accounts(SAVE_ACCOUNT, ctx),
                GET_ACCOUNT => handle_accounts(GET_ACCOUNT, ctx),
                GET_LAMPORTS => handle_accounts(GET_LAMPORTS, ctx),
                SET_LAMPORTS => handle_accounts(SET_LAMPORTS, ctx),
                GET_DATA => handle_accounts(GET_DATA, ctx),
                GET_KEY => handle_accounts(GET_KEY, ctx),
                GET_OWNER => handle_accounts(GET_OWNER, ctx),
                TRANSFER => handle_accounts(TRANSFER, ctx),
                TRANSFER_SIGNED => handle_accounts(TRANSFER_SIGNED, ctx),

                // Array Operations (0x60-0x6F)
                CREATE_ARRAY => handle_arrays(CREATE_ARRAY, ctx),
                PUSH_ARRAY_LITERAL => handle_arrays(PUSH_ARRAY_LITERAL, ctx),
                ARRAY_INDEX => handle_arrays(ARRAY_INDEX, ctx),
                ARRAY_LENGTH => handle_arrays(ARRAY_LENGTH, ctx),
                ARRAY_SET => handle_arrays(ARRAY_SET, ctx),
                ARRAY_GET => handle_arrays(ARRAY_GET, ctx),
                PUSH_STRING_LITERAL => handle_arrays(PUSH_STRING_LITERAL, ctx),
                PUSH_STRING => handle_arrays(PUSH_STRING, ctx),
                PUSH_STRING_W => handle_arrays(PUSH_STRING_W, ctx),

                // Constraint Operations (0x70-0x7F)
                CHECK_SIGNER => handle_constraints(CHECK_SIGNER, ctx),
                CHECK_WRITABLE => handle_constraints(CHECK_WRITABLE, ctx),
                CHECK_OWNER => handle_constraints(CHECK_OWNER, ctx),
                CHECK_INITIALIZED => handle_constraints(CHECK_INITIALIZED, ctx),
                CHECK_PDA => handle_constraints(CHECK_PDA, ctx),
                CHECK_UNINITIALIZED => handle_constraints(CHECK_UNINITIALIZED, ctx),
                CHECK_DEDUPE_TABLE => handle_constraints(CHECK_DEDUPE_TABLE, ctx),
                CHECK_CACHED => handle_constraints(CHECK_CACHED, ctx),
                CHECK_COMPLEXITY_GROUP => handle_constraints(CHECK_COMPLEXITY_GROUP, ctx),
                CHECK_DEDUPE_MASK => handle_constraints(CHECK_DEDUPE_MASK, ctx),
                REQUIRE_OWNER => handle_constraints(REQUIRE_OWNER, ctx),

                // System Operations (0x80-0x8F)
                // Universal Fused Operations (0xC0-0xCF)
                REQUIRE_GTE_U64 => crate::handlers::fused_ops::handle_fused_ops(REQUIRE_GTE_U64, ctx),
                REQUIRE_NOT_BOOL => crate::handlers::fused_ops::handle_fused_ops(REQUIRE_NOT_BOOL, ctx),
                FIELD_ADD_PARAM => crate::handlers::fused_ops::handle_fused_ops(FIELD_ADD_PARAM, ctx),
                FIELD_SUB_PARAM => crate::handlers::fused_ops::handle_fused_ops(FIELD_SUB_PARAM, ctx),
                REQUIRE_PARAM_GT_ZERO => crate::handlers::fused_ops::handle_fused_ops(REQUIRE_PARAM_GT_ZERO, ctx),
                REQUIRE_EQ_PUBKEY => crate::handlers::fused_ops::handle_fused_ops(REQUIRE_EQ_PUBKEY, ctx),
                CHECK_SIGNER_WRITABLE => crate::handlers::fused_ops::handle_fused_ops(CHECK_SIGNER_WRITABLE, ctx),
                // Tier 3 fused opcodes (0xC7-0xCA)
                STORE_PARAM_TO_FIELD => crate::handlers::fused_ops::handle_fused_ops(STORE_PARAM_TO_FIELD, ctx),
                STORE_FIELD_ZERO => crate::handlers::fused_ops::handle_fused_ops(STORE_FIELD_ZERO, ctx),
                STORE_KEY_TO_FIELD => crate::handlers::fused_ops::handle_fused_ops(STORE_KEY_TO_FIELD, ctx),
                REQUIRE_EQ_FIELDS => crate::handlers::fused_ops::handle_fused_ops(REQUIRE_EQ_FIELDS, ctx),
                REQUIRE_FIELD_EQ_IMM => crate::handlers::fused_ops::handle_fused_ops(REQUIRE_FIELD_EQ_IMM, ctx),
                FIELD_SUB_ADD_PARAM => crate::handlers::fused_ops::handle_fused_ops(FIELD_SUB_ADD_PARAM, ctx),
                REQUIRE_PARAM_LTE_IMM => crate::handlers::fused_ops::handle_fused_ops(REQUIRE_PARAM_LTE_IMM, ctx),

                // System Operations (0x80-0x8F)
                INVOKE => handle_system_ops(INVOKE, ctx),
                INVOKE_SIGNED => handle_system_ops(INVOKE_SIGNED, ctx),
                GET_CLOCK => handle_system_ops(GET_CLOCK, ctx),
                GET_RENT => handle_system_ops(GET_RENT, ctx),
                INIT_ACCOUNT => handle_system_ops(INIT_ACCOUNT, ctx),
                INIT_PDA_ACCOUNT => handle_system_ops(INIT_PDA_ACCOUNT, ctx),
                DERIVE_PDA => handle_system_ops(DERIVE_PDA, ctx),
                FIND_PDA => handle_system_ops(FIND_PDA, ctx),
                DERIVE_PDA_PARAMS => handle_system_ops(DERIVE_PDA_PARAMS, ctx),
                FIND_PDA_PARAMS => handle_system_ops(FIND_PDA_PARAMS, ctx),

                // Function Operations (0x90-0x9F)
                CALL => handle_functions(CALL, ctx),
                CALL_EXTERNAL => handle_functions(CALL_EXTERNAL, ctx),
                CALL_NATIVE => handle_functions(CALL_NATIVE, ctx),
                PREPARE_CALL => handle_functions(PREPARE_CALL, ctx),
                FINISH_CALL => handle_functions(FINISH_CALL, ctx),

                // Locals & General (0xA0-0xAF)
                ALLOC_LOCALS => handle_locals(ALLOC_LOCALS, ctx),
                DEALLOC_LOCALS => handle_locals(DEALLOC_LOCALS, ctx),
                SET_LOCAL => handle_locals(SET_LOCAL, ctx),
                GET_LOCAL => handle_locals(GET_LOCAL, ctx),
                CLEAR_LOCAL => handle_locals(CLEAR_LOCAL, ctx),
                LOAD_PARAM => handle_locals(LOAD_PARAM, ctx),
                STORE_PARAM => handle_locals(STORE_PARAM, ctx),
                WRITE_DATA => handle_locals(WRITE_DATA, ctx),
                DATA_LEN => handle_locals(DATA_LEN, ctx),
                EMIT_EVENT => handle_locals(EMIT_EVENT, ctx),
                LOG_DATA => handle_locals(LOG_DATA, ctx),
                GET_SIGNER_KEY => handle_locals(GET_SIGNER_KEY, ctx),
                RESULT_UNWRAP => handle_option_result_ops(RESULT_UNWRAP, ctx),
                RESULT_GET_VALUE => handle_option_result_ops(RESULT_GET_VALUE, ctx),
                RESULT_GET_ERROR => handle_option_result_ops(RESULT_GET_ERROR, ctx),
                CAST => handle_locals(CAST, ctx),

                // Nibble Locals (0xD0-0xDF)
                GET_LOCAL_0 => handle_nibble_locals(GET_LOCAL_0, ctx),
                GET_LOCAL_1 => handle_nibble_locals(GET_LOCAL_1, ctx),
                GET_LOCAL_2 => handle_nibble_locals(GET_LOCAL_2, ctx),
                GET_LOCAL_3 => handle_nibble_locals(GET_LOCAL_3, ctx),
                SET_LOCAL_0 => handle_nibble_locals(SET_LOCAL_0, ctx),
                SET_LOCAL_1 => handle_nibble_locals(SET_LOCAL_1, ctx),
                SET_LOCAL_2 => handle_nibble_locals(SET_LOCAL_2, ctx),
                SET_LOCAL_3 => handle_nibble_locals(SET_LOCAL_3, ctx),
                PUSH_0 => handle_nibble_locals(PUSH_0, ctx),
                PUSH_1 => handle_nibble_locals(PUSH_1, ctx),
                PUSH_2 => handle_nibble_locals(PUSH_2, ctx),
                PUSH_3 => handle_nibble_locals(PUSH_3, ctx),
                LOAD_PARAM_0 => handle_nibble_locals(LOAD_PARAM_0, ctx),
                LOAD_PARAM_1 => handle_nibble_locals(LOAD_PARAM_1, ctx),
                LOAD_PARAM_2 => handle_nibble_locals(LOAD_PARAM_2, ctx),
                LOAD_PARAM_3 => handle_nibble_locals(LOAD_PARAM_3, ctx),

                // Advanced / Option Result (0xF0-0xFF)
                RESULT_OK => handle_option_result_ops(RESULT_OK, ctx),
                RESULT_ERR => handle_option_result_ops(RESULT_ERR, ctx),
                OPTIONAL_SOME => handle_option_result_ops(OPTIONAL_SOME, ctx),
                OPTIONAL_NONE => handle_option_result_ops(OPTIONAL_NONE, ctx),
                OPTIONAL_UNWRAP => handle_option_result_ops(OPTIONAL_UNWRAP, ctx),
                OPTIONAL_IS_SOME => handle_option_result_ops(OPTIONAL_IS_SOME, ctx),
                OPTIONAL_GET_VALUE => handle_option_result_ops(OPTIONAL_GET_VALUE, ctx),
                CREATE_TUPLE => handle_option_result_ops(CREATE_TUPLE, ctx),
                TUPLE_GET => handle_option_result_ops(TUPLE_GET, ctx),
                UNPACK_TUPLE => handle_option_result_ops(UNPACK_TUPLE, ctx),
                OPTIONAL_IS_NONE => handle_option_result_ops(OPTIONAL_IS_NONE, ctx),
                RESULT_IS_OK => handle_option_result_ops(RESULT_IS_OK, ctx),
                RESULT_IS_ERR => handle_option_result_ops(RESULT_IS_ERR, ctx),

                // Fallthrough for gaps, removed opcodes (0xC0-0xCF), and unimplemented pattern fusion (0xE0-0xEF)
                _ => {
                    debug_log!(
                        "MitoVM: FATAL_ERROR: UNKNOWN/UNIMPLEMENTED OPCODE {} at ip {}",
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
                LAST_COMPUTE_UNITS.store(ctx.compute_units_consumed(), Ordering::Relaxed);
                // Clear temp buffer to avoid reusing stale data between runs
                ctx.reset_temp_buffer();
                result
            }
            Err(e) => {
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
