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
        handle_functions, handle_init_ops, handle_invoke_ops, handle_locals, handle_logical,
        handle_memory, handle_native_ops, handle_nibble_locals, handle_option_result_ops,
        handle_pda_ops, handle_registers, handle_stack_ops, handle_sysvar_ops,
    },
    stack_error_context, // Import enhanced debugging macros
    FIVE_MAGIC,
};
use five_protocol::{Value, ValueRef, FIVE_HEADER_OPTIMIZED_SIZE};


// ExecutionResult removed - fake CU tracking eliminated for production
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
}

/// Ultra-lightweight virtual machine optimized for Solana's execution environment.
/// Features zero-allocation execution, function calls, and sub-50 CU cold start overhead.
///
/// # Example
/// ```rust,no_run
/// use five_vm_mito::{MitoVM, Value};
/// use pinocchio::account_info::AccountInfo;
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
/// let result = MitoVM::execute_direct(bytecode, &[], &[])?;
/// assert_eq!(result, Some(Value::U8(15)));
/// # Ok::<(), five_vm_mito::VMError>(())
/// ```
pub struct MitoVM;

impl MitoVM {
    /// Dispatch system-level operations including CPI, PDA operations, and account initialization.
    #[inline(never)]
    fn handle_system(opcode: u8, ctx: &mut ExecutionManager<'_>) -> CompactResult<()> {
        match opcode {
            // Cross-program invocation operations (INVOKE, INVOKE_SIGNED)
            INVOKE | INVOKE_SIGNED => handle_invoke_ops(opcode, ctx),
            // Blockchain sysvar operations (GET_CLOCK, GET_RENT)
            GET_CLOCK | GET_RENT => handle_sysvar_ops(opcode, ctx),
            // Account initialization operations (INIT_ACCOUNT, INIT_PDA_ACCOUNT)
            INIT_ACCOUNT | INIT_PDA_ACCOUNT => handle_init_ops(opcode, ctx),
            // Program Derived Address operations (DERIVE_PDA, FIND_PDA, etc.)
            DERIVE_PDA | FIND_PDA | DERIVE_PDA_PARAMS | FIND_PDA_PARAMS => {
                handle_pda_ops(opcode, ctx)
            }
            _ => Err(VMErrorCode::InvalidInstruction),
        }
    }

    /// Prepare execution environment with optimized parameter parsing and minimal overhead.
    /// Validates script format, parses VLE parameters, and sets up function dispatch.
    #[inline(never)]
    fn initialize_execution_context<'a>(
        script: &'a [u8],
        input_data: &'a [u8],
        accounts: &'a [AccountInfo],
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

            // Detailed input data analysis
            if input_data.len() > 0 {
                debug_log!(
                    "MitoVM: Input data first byte (function index): {}",
                    input_data[0]
                );
                if input_data.len() >= 4 {
                    debug_log!(
                        "MitoVM: Input data first 4 bytes: {} {} {} {}",
                        input_data[0],
                        input_data[1],
                        input_data[2],
                        input_data[3]
                    );
                }
            }
        }

        // TRUST: Script format validated at deploy-time, skip validation here
        // This enables blazing-fast execution path

        // Parse optimized production header V2 (magic + features + public_count + total_count)
        let (start_ip, public_function_count, total_function_count, header_features) =
            Self::parse_optimized_header(script)?;

        // Create execution manager with optimized approach
        debug_log!("MitoVM: Creating ExecutionManager...");

        let program_id = Pubkey::from(crate::FIVE_VM_PROGRAM_ID);

        // 🚀 OPTIMIZED: Direct ExecutionManager creation - no expensive resource parsing
        debug_log!("MitoVM: ⚡ Using compile-time defaults for optimal performance");
        debug_log!(
            "MitoVM: Function counts from header: {} public, {} total",
            public_function_count as u32,
            total_function_count as u32
        );
        let mut ctx = ExecutionManager::new(
            script,
            accounts,
            program_id,
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

        // Pre-parse VLE parameters directly into execution context for zero-copy access
        {
            let mut parsed_params = [ValueRef::Empty; 8];
            if !input_data.is_empty() {
                debug_log!("MitoVM: Parsing VLE parameters using unified function...");
                crate::utils::parse_vle_parameters_unified(&mut ctx, input_data, &mut parsed_params)?;
                #[cfg(feature = "debug-logs")]
                {
                    let mut s = heapless::String::<64>::new();
                    let _ = write!(&mut s, "{:?}", &parsed_params);
                    debug_log!(
                        "MitoVM: Parsed {} VLE parameters successfully: {}",
                        parsed_params.iter().filter(|p| !p.is_empty()).count() as u32,
                        s.as_str()
                    );
                }
            }
            ctx.parameters[..8].copy_from_slice(&parsed_params);
        }

        // Store parsed parameters metadata
        debug_log!("MitoVM: Setting pre-parsed parameters in ExecutionContext...");
        ctx.param_start = 0;
        debug_log!(
            "MitoVM: Pre-parsed parameters stored successfully. ctx.parameters.len(): {}",
            ctx.parameters().len() as u32
        );

        // Initialize ValueAccessContext for zero-copy parameter access
        debug_log!("MitoVM: Initializing ValueAccessContext...");
        debug_log!("MitoVM: ValueAccessContext components prepared successfully");

        // Push parsed VLE parameters onto the stack and mirror into locals for nibble GET_LOCAL_* access
        // Skip the first parameter (function index) as it's metadata, not a function parameter
        debug_log!("MitoVM: Pushing VLE parameters onto stack and initializing locals...");

        // Count actual function parameters (excluding index 0) - iterate slice directly
        // We need to find the highest index that is set to ensure we allocate enough locals
        // even if there are gaps (sparse parameters)
        let mut param_count: u8 = 0;
        let mut max_param_index: u8 = 0;
        for i in 1..8 {
            if !ctx.parameters[i].is_empty() {
                param_count = param_count.saturating_add(1);
                max_param_index = i as u8;
            }
        }

        // Set param_len to actual count (not MAX_PARAMETERS)
        ctx.param_len = param_count;

        // Initialize locals to mirror parameters for compilers that lower params to locals 0..N-1
        // Allocate based on max_param_index to handle sparse parameters
        let locals_to_allocate = if max_param_index > 0 {
            // Allocate enough locals to cover up to the last parameter
            max_param_index
        } else {
            // Allocate default locals for main frame even with no parameters (3 allows 4 call levels with 12 max locals)
            3
        };
        ctx.allocate_locals(locals_to_allocate)?;

        for i in 1..8 {
            let param = ctx.parameters[i];
            if param.is_empty() {
                continue;
            }

            // Push onto stack for code that expects parameters on stack
            #[cfg(feature = "debug-logs")]
            {
                let mut s = heapless::String::<64>::new();
                write!(&mut s, "{:?}", param).unwrap();
                debug_log!(
                    "MitoVM: Pushing parameter {} ({}) onto stack",
                    i as u32,
                    s.as_str()
                );
            }
            ctx.push(param)?;

            // Mirror into local variable slot (index starts at 0)
            let local_index = (i - 1) as u8;
            if (local_index as u16) < crate::MAX_LOCALS as u16 {
                ctx.set_local(local_index, param)?;
                debug_log!(
                    "MitoVM: Initialized local {} from parameter {}",
                    local_index as u32,
                    i as u32
                );
            }
        }

        // Handle function dispatch with visibility validation
        let dispatch_ip = if !ctx.parameters[0].is_empty() {
            if let ValueRef::U64(func_index) = ctx.parameters[0] {
                debug_log!(
                    "MitoVM: Function dispatch requested for function index: {}",
                    func_index
                );

                // Validate function visibility for external calls
                // 🚀 OPTIMIZED: Explicit visibility validation using public_function_count
                // External calls can only target public functions (indices 0..public_count-1)
                // The compiler ensures public functions are at indices 0..(public_count-1)
                if func_index as u8 >= ctx.public_function_count() {
                    debug_log!(
                        "MitoVM: ERROR: Function index {} >= public_function_count {}",
                        func_index,
                        ctx.public_function_count()
                    );
                    return Err(VMErrorCode::FunctionVisibilityViolation);
                }
                debug_log!(
                    "MitoVM: ✓ Function visibility check passed - index {} < public_count {}",
                    func_index,
                    ctx.public_function_count()
                );

                // 🚀 OPTIMIZED: Simple function dispatch
                // Function 0 is the main entry point at start_ip
                // Other functions are called via direct CALL instructions in bytecode
                if func_index == 0 {
                    debug_log!(
                        "MitoVM: ⚡ Dispatching to main function (0) at start_ip: {}",
                        start_ip as u32
                    );
                    start_ip
                } else {
                    debug_log!(
                        "MitoVM: ⚡ Function {} dispatch - bytecode should use CALL instructions",
                        func_index
                    );
                    // For non-zero functions, start at beginning and let bytecode handle routing
                    start_ip
                }
            } else {
                debug_log!("MitoVM: Function index parameter is not U64, using default start_ip");
                start_ip
            }
        } else {
            debug_log!("MitoVM: No function dispatch requested, using default start_ip");
            start_ip
        };

        ctx.set_ip(dispatch_ip);
        debug_log!(
            "MitoVM: Context created, starting execution at ip {}",
            ctx.ip() as u32
        );

        Ok((ctx, dispatch_ip))
    }

    /// Route opcodes to specialized handlers based on upper nibble (16 opcodes per group).
    /// Prevents stack overflow through hierarchical dispatch architecture.
    #[inline(never)]
    fn dispatch_opcode_range(opcode: u8, ctx: &mut ExecutionManager) -> CompactResult<()> {
        // Hierarchical opcode dispatch to prevent stack overflow
        // Dispatch based on opcode ranges (16 opcodes per range)

        // PHASE 1 DEBUGGING: Enhanced logging for RETURN_VALUE (0x07) opcode
        /*
        if opcode == 0x07 {
            debug_log!(
                "🔍 PHASE1_DEBUG: RETURN_VALUE opcode (0x07) detected at IP {}",
                (ctx.ip() - 1) as u32
            );
            debug_log!(
                "🔍 PHASE1_DEBUG: Opcode range calculation: 0x07 & 0xF0 = {}",
                (opcode & 0xF0)
            );
            debug_log!("🔍 PHASE1_DEBUG: Will dispatch to 0x00 range (control_flow handler)");
            debug_log!(
                "🔍 PHASE1_DEBUG: Current stack size before handler: {}",
                ctx.size() as u32
            );
            debug_log!("🔍 PHASE1_DEBUG: Current halted state before handler");
        }
        */

        match opcode & 0xF0 {
            0x00 => {
                // PHASE 1 DEBUGGING: Enhanced logging specifically for RETURN_VALUE
                if opcode == 0x07 {
                    debug_log!(
                        "🔍 PHASE1_DEBUG: About to call handle_control_flow() for RETURN_VALUE"
                    );
                }

                // TRY CONTROL FLOW FIRST (HALT, JUMP, etc.)
                let result = handle_control_flow(opcode, ctx);
                
                match result {
                    Ok(_) => Ok(()),
                    Err(VMErrorCode::InvalidInstruction) => {
                        // FALLBACK: Try stack ops (e.g. PUSH_U64=0x01, POP=0x02 if not in control_flow)
                        handle_stack_ops(opcode, ctx)
                    },
                    Err(e) => {
                         debug_log!("MitoVM: CONTROL_FLOW_ERROR: Opcode {} failed", opcode);
                         Err(e)
                    }
                }
            }
            0x10..=0x1F => {
                // debug_log!(
                //     "MitoVM: Dispatching to stack_ops handler for opcode {}",
                //     opcode
                // );
                handle_stack_ops(opcode, ctx)
            }
            0x20..=0x2F => {
                // debug_log!(
                //     "MitoVM: Dispatching to arithmetic handler for opcode {}",
                //     opcode
                // );
                handle_arithmetic(opcode, ctx)
            }
            0x30..=0x3F => {
                // debug_log!(
                //     "MitoVM: Dispatching to logical handler for opcode {}",
                //     opcode
                // );
                handle_logical(opcode, ctx)
            }
            0x40..=0x4F => {
                // debug_log!(
                //     "MitoVM: Dispatching to memory handler for opcode {}",
                //     opcode
                // );
                
                // TRY CONTROL FLOW FIRST (JUMP=0x40, JUMP_IF=0x41 etc might be here)
                let result = handle_control_flow(opcode, ctx);
                
                match result {
                    Ok(_) => Ok(()),
                    Err(VMErrorCode::InvalidInstruction) => {
                        // FALLBACK: Memory instructions (STORE/LOAD/STORE_FIELD etc)
                        handle_memory(opcode, ctx)
                    },
                    Err(e) => {
                         debug_log!("MitoVM: CONTROL_FLOW_JUMP_ERROR: Opcode {} failed", opcode);
                         Err(e)
                    }
                }
            }
            0x50..=0x5F => {
                // debug_log!(
                //     "MitoVM: Dispatching to accounts handler for opcode {}",
                //     opcode
                // );
                handle_accounts(opcode, ctx)
            }
            0x60..=0x6F => {
                // 🎯 LOGICAL REORGANIZATION: Arrays now at 0x60 (moved from scattered locations)
                // debug_log!(
                //     "MitoVM: Dispatching to arrays handler for opcode {}",
                //     opcode
                // );
                handle_arrays(opcode, ctx)
            }
            0x70..=0x7F => {
                // 🎯 LOGICAL REORGANIZATION: Constraints moved from 0x60 to 0x70
                // debug_log!(
                //     "MitoVM: Dispatching to constraints handler for opcode {}",
                //     opcode
                // );
                handle_constraints(opcode, ctx)
            }
            0x80..=0x8F => {
                // 🎯 LOGICAL REORGANIZATION: System operations moved from 0x70 to 0x80
                // debug_log!(
                //     "MitoVM: Dispatching to system handler for opcode {}",
                //     opcode
                // );
                Self::handle_system(opcode, ctx)
            }
            0x90..=0x9F => {
                // 🎯 LOGICAL REORGANIZATION: Functions moved from 0x80 to 0x90
                if opcode == CALL_NATIVE {
                    // debug_log!(
                    //     "MitoVM: Dispatching to native syscall handler for opcode {}",
                    //     opcode
                    // );
                    handle_native_ops(ctx)
                } else {
                    // debug_log!(
                    //     "MitoVM: Dispatching to functions handler for opcode {}",
                    //     opcode
                    // );
                    handle_functions(opcode, ctx)
                }
            }
            0xA0..=0xAF => {
                // 🎯 LOGICAL REORGANIZATION: Locals moved from 0x90 to 0xA0 + general operations
                // debug_log!(
                //     "MitoVM: Dispatching to locals/general handler for opcode {}",
                //     opcode
                // );
                handle_locals(opcode, ctx)
            }
            0xB0..=0xBF => {
                // debug_log!(
                //     "MitoVM: Dispatching to registers handler for opcode {}",
                //     opcode
                // );
                handle_registers(opcode, ctx)
            }
            0xC0..=0xCF => {
                // [REMOVED] Account view operations - use zero-copy LOAD_FIELD/STORE_FIELD instead
                debug_log!(
                    "MitoVM: Account view opcode {} removed - use LOAD_FIELD/STORE_FIELD",
                    opcode
                );
                Err(VMErrorCode::InvalidInstruction)
            }
            0xD0..=0xDF => {
                handle_nibble_locals(opcode, ctx)
            }
            0xF0..=0xFF => {
                // Option and Result operations
                handle_option_result_ops(opcode, ctx)
            }
            _ => {
                debug_log!(
                    "MitoVM: FATAL_ERROR: UNKNOWN OPCODE RANGE {} at ip {}",
                    opcode,
                    (ctx.ip() - 1) as u32
                );
                Err(VMErrorCode::InvalidInstruction)
            }
        }
    }

    /// Core execution loop that fetches and executes bytecode instructions until halt or error.
    #[inline(never)]
    fn execute_instruction_loop(ctx: &mut ExecutionManager) -> CompactResult<()> {
        debug_log!("MitoVM: ===== BEGINNING EXECUTION LOOP =====");
        debug_log!("🔍 EXECUTION_TRACE: ===== EXECUTION LOOP STARTING =====");
        debug_log!(
            "🔍 EXECUTION_TRACE: Starting IP: {}, Script length: {}",
            ctx.ip() as u32,
            ctx.script().len() as u32
        );

        // Main execution loop with enhanced opcodes
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

            {
                // Removed log
            }
            let opcode = match ctx.fetch_byte() {
                Ok(op) => {
                    // debug_log!(
                    //     "MitoVM: Fetched opcode {} ({}) at IP {} (script_len: {})",
                    //     op,
                    //     op,
                    //     (ctx.ip() - 1) as u32,
                    //     ctx.script().len() as u32
                    // );

                    op
                }
                Err(e) => {
                    debug_log!(
                        "MitoVM: Error fetching opcode at IP {}",
                        current_ip as u32
                    );
                    return Err(e);
                }
            };
            // debug_log!(
            //     "MitoVM: *** EXECUTING OPCODE {} at ip {} ***",
            //     opcode,
            //     (ctx.ip() - 1) as u32
            // );

            // Debug opcode verification
            #[cfg(feature = "debug-logs")]
            if opcode == LOAD_INPUT {
                debug_log!(
                    "MitoVM: CONFIRMED - This is LOAD_INPUT opcode ({})",
                    LOAD_INPUT
                );
            }

            // Compute unit consumption from protocol opcode table (single source of truth)
            //let cu_cost = crate::opcodes::opcode_cu_cost(opcode) as u64;
            //ctx.consume_compute_units(cu_cost);

            // Enhanced debugging: Log stack state before opcode execution
            //debug_stack_state!(opcode, ctx, "BEFORE");

            // Dispatch opcode to appropriate handler
            let result = Self::dispatch_opcode_range(opcode, ctx);

            // Check result and provide clear error context
            if let Err(e) = result {
                // Enhanced error context with full VM state
                stack_error_context!(opcode, ctx, "EXECUTION_FAILED", 0, ctx.size());
                error_log!("MitoVM: ERROR_DETAILS: error_occurred at current_ip: {}", current_ip as u64);
                error_log!("Stack size: {}", ctx.size() as u64);
                return Err(e);
            }

            // CRITICAL FIX: Check halted flag immediately after opcode execution
            // This fixes the regression where RETURN_VALUE sets halted=true but loop continues
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

            // Enhanced debugging: Log stack state after successful opcode execution
            //debug_stack_state!(opcode, ctx, "AFTER");
        }

        // DIAGNOSTIC: Stack state at the END of execution loop
        #[cfg(feature = "debug-logs")]
        {
            debug_log!(
                "STACK_DEBUG: END of execute_instruction_loop - stack size: {}, halted: {}",
                ctx.size() as u32,
                ctx.halted() as u8
            );
            if !ctx.is_empty() {
                debug_log!("STACK_DEBUG: Final stack has items, returning to execute_with_context");
            } else {
                debug_log!("STACK_DEBUG: WARNING - Final stack is EMPTY, this may be the problem!");
            }
        }

        Ok(())
    }

    /// Convert ValueRef (zero-copy reference) to concrete Value using current execution state.
    /// Handles complex references like TempRef, OptionalRef, and AccountRef.
    #[allow(dead_code)] // Function is used recursively, compiler doesn't detect this
    #[inline(never)]
    pub fn resolve_value_ref(value_ref: &ValueRef, ctx: &ExecutionManager<'_>) -> CompactResult<Value> {
        Self::resolve_value_ref_with_depth(value_ref, ctx, 0)
    }

    /// Internal recursive implementation of resolve_value_ref with depth tracking
    fn resolve_value_ref_with_depth(
        value_ref: &ValueRef,
        ctx: &ExecutionManager<'_>,
        depth: u8,
    ) -> CompactResult<Value> {
        // Prevent infinite recursion/stack overflow
        const MAX_VALUE_REF_DEPTH: u8 = 8;
        if depth > MAX_VALUE_REF_DEPTH {
            return Err(VMErrorCode::StackOverflow);
        }

        match value_ref {
            // Immediate values - no context needed
            ValueRef::Empty => Ok(Value::Empty),
            ValueRef::U8(v) => Ok(Value::U8(*v)), // Preserve U8 type
            ValueRef::U64(v) => Ok(Value::U64(*v)),
            ValueRef::U128(v) => Ok(Value::U128(*v)),
            ValueRef::I64(v) => Ok(Value::I64(*v)),
            ValueRef::Bool(v) => Ok(Value::Bool(*v)),

            // Reference types - need context resolution
            ValueRef::TempRef(offset, size) => {
                let start = *offset as usize;
                let end = start + *size as usize;
                if end > ctx.temp_buffer().len() {
                    return Err(VMErrorCode::MemoryViolation);
                }

                // Try to deserialize as ValueRef first, then extract the actual value
                match ValueRef::deserialize_from(&ctx.temp_buffer()[start..end]) {
                    Ok(inner_ref) => Self::resolve_value_ref_with_depth(&inner_ref, ctx, depth + 1),
                    Err(_) => {
                        // If deserialization fails, treat as raw bytes and convert to u64 if possible
                        if *size == 8 {
                            let bytes: [u8; 8] = ctx.temp_buffer()[start..end]
                                .try_into()
                                .map_err(|_| VMErrorCode::ProtocolError)?;
                            Ok(Value::U64(u64::from_le_bytes(bytes)))
                        } else if *size == 1 {
                            Ok(Value::U64(ctx.temp_buffer()[start] as u64))
                        } else {
                            // For other sizes, return Empty
                            Ok(Value::Empty)
                        }
                    }
                }
            }

            ValueRef::TupleRef(_offset, _size) => {
                // Tuple refs are complex - for now return Empty, but log for debugging
                debug_log!(
                    "MitoVM: TupleRef resolution not fully implemented, offset: {}, size: {}",
                    *_offset,
                    *_size
                );
                Ok(Value::Empty)
            }

            ValueRef::OptionalRef(offset, size) => {
                if *size == 0 {
                    return Err(VMErrorCode::ProtocolError);
                }
                let tag = ctx.temp_buffer()[*offset as usize];
                if tag == 0 {
                    Ok(Value::Empty)
                } else {
                    if (*size as usize) <= 1 {
                        return Err(VMErrorCode::ProtocolError);
                    }
                    let inner_ref = ValueRef::deserialize_from(
                        &ctx.temp_buffer()[*offset as usize + 1..*offset as usize + *size as usize],
                    )
                    .map_err(|_| VMErrorCode::ProtocolError)?;
                    Self::resolve_value_ref_with_depth(&inner_ref, ctx, depth + 1)
                }
            }

            ValueRef::ResultRef(offset, size) => {
                if *size == 0 {
                    return Err(VMErrorCode::ProtocolError);
                }
                let tag = ctx.temp_buffer()[*offset as usize];
                if (*size as usize) > 1 {
                    let inner_ref = ValueRef::deserialize_from(
                        &ctx.temp_buffer()[*offset as usize + 1..*offset as usize + *size as usize],
                    )
                    .map_err(|_| VMErrorCode::ProtocolError)?;
                    let inner_val = Self::resolve_value_ref_with_depth(&inner_ref, ctx, depth + 1)?;
                    if tag != 0 {
                        Ok(inner_val)
                    } else {
                        Ok(Value::Empty)
                    }
                } else {
                    Ok(Value::Empty)
                }
            }

            ValueRef::AccountRef(idx, _offset) => Ok(Value::Account(*idx)),
            ValueRef::InputRef(offset) => {
                let start = *offset as usize;
                let end = start + 8;
                let data = ctx.instruction_data();
                if end > data.len() {
                    return Err(VMErrorCode::InvalidOperation);
                }
                let bytes: [u8; 8] = data[start..end]
                    .try_into()
                    .map_err(|_| VMErrorCode::InvalidOperation)?;
                Ok(Value::U64(u64::from_le_bytes(bytes)))
            }
            ValueRef::PubkeyRef(offset) => {
                let start = *offset as usize;
                let end = start + 32;
                let data = ctx.instruction_data();
                if end <= data.len() {
                    let mut pk_bytes = [0u8; 32];
                    pk_bytes.copy_from_slice(&data[start..end]);
                    Ok(Value::Pubkey(Pubkey::from(pk_bytes)))
                } else if start < ctx.accounts().len() {
                    let pk = *ctx.accounts()[start].key();
                    Ok(Value::Pubkey(pk))
                } else {
                    Err(VMErrorCode::InvalidOperation)
                }
            }
            ValueRef::ArrayRef(id) => Ok(Value::Array(*id)),
            ValueRef::StringRef(id) => Ok(Value::String(*id as u8)),
            ValueRef::HeapString(id) => Ok(Value::String(*id as u8)),
            ValueRef::HeapArray(id) => Ok(Value::Array(*id as u8)),
        }
    }

    /// Extract final execution result, prioritizing captured return values over stack contents.
    #[inline(never)]
    fn finalize_execution_result(ctx: &mut ExecutionManager<'_>) -> CompactResult<Option<Value>> {
        // NEW APPROACH: Use captured return value instead of relying on stack
        // This fixes the stack persistence issue after ExecutionManager refactoring
        match ctx.return_value() {
            Some(value) => Ok(value.to_value()),
            None => {
                // No return value captured, check if there's something on the stack as fallback
                if !ctx.is_empty() {
                    let value_ref = ctx.pop()?;
                    match value_ref {
                        ValueRef::U64(val) => Ok(Some(Value::U64(val))),
                        ValueRef::U8(val) => Ok(Some(Value::U8(val))),
                        ValueRef::I64(val) => Ok(Some(Value::I64(val))),
                        ValueRef::U128(val) => Ok(Some(Value::U128(val))),
                        ValueRef::Bool(val) => Ok(Some(Value::Bool(val))),
                        _ => Ok(None), // Complex types return None
                    }
                } else {
                    Ok(None) // No return value
                }
            }
        }
    }

    /// Execute bytecode directly with minimal overhead.
    /// Returns optional value from RETURN_VALUE opcode or None if script completes without explicit return.
    ///
    /// # Example
    /// ```rust,no_run
    /// use five_vm_mito::MitoVM;
    /// use pinocchio::account_info::AccountInfo;
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
    ///
    /// let result = MitoVM::execute_direct(bytecode, input_data, accounts)?;
    /// assert_eq!(result, Some(five_protocol::Value::U8(42)));
    /// # Ok::<(), five_vm_mito::VMError>(())
    /// ```
    #[inline(never)]
    pub fn execute_direct(
        script: &[u8],
        input_data: &[u8],
        accounts: &[AccountInfo],
    ) -> Result<Option<Value>> {
        // UNCONDITIONAL LOG - use error_log which is always compiled in
        error_log!("MitoVM_ENTRY: script={} input={} accounts={}",
            script.len() as u32, input_data.len() as u32, accounts.len() as u32);

        // Use error_log which is always compiled in to verify logging works
        error_log!("MitoVM: execute_direct ENTRY - script={} input={} accounts={}",
            script.len() as u32, input_data.len() as u32, accounts.len() as u32);

        let mut storage = crate::stack::StackStorage::new(script);
        let (mut ctx, _dispatch_ip) =
            Self::initialize_execution_context(script, input_data, accounts, &mut storage)?;
        let execution_result = Self::execute_instruction_loop(&mut ctx);
        match execution_result {
            Ok(()) => {
                let result = Self::finalize_execution_result(&mut ctx)
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
    ///     let (result, context) = MitoVM::execute_with_context(bytecode, &[], &[]).unwrap();
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
    ) -> Result<(Option<Value>, VMExecutionContext)> {
        // Phase 1: Initialize execution context with script validation and function dispatch
        let mut storage = crate::stack::StackStorage::new(script);
        let (mut ctx, _dispatch_ip) =
            Self::initialize_execution_context(script, input_data, accounts, &mut storage)?;

        #[cfg(feature = "debug-logs")]
        debug_log!(
            "MitoVM: INIT complete - func_count={}, call_depth={}, param_len={}, stack_size={}",
            ctx.total_function_count() as u32,
            ctx.call_depth() as u32,
            ctx.param_len() as u32,
            ctx.size() as u32
        );

        // DIAGNOSTIC: Stack should be empty after initialization
        #[cfg(feature = "debug-logs")]
        debug_log!(
            "STACK_DEBUG: After initialization - stack size: {}",
            ctx.size() as u32
        );

        // Phase 2: Execute main instruction loop
        let execution_result = Self::execute_instruction_loop(&mut ctx);

        // DIAGNOSTIC: Check stack size immediately after execution loop
        #[cfg(feature = "debug-logs")]
        {
            debug_log!(
                "STACK_DEBUG: After execution loop - stack size: {}, halted: {}",
                ctx.size() as u32,
                ctx.halted() as u8
            );
            if !ctx.is_empty() {
                debug_log!("STACK_DEBUG: Stack has items after execution");
            }
        }

        // Phase 3: (trimmed) Build minimal context only for success path below

        // Phase 4: Finalize and extract result if execution succeeded
        let final_result = match execution_result {
            Ok(()) => {
                // DIAGNOSTIC: Check stack size right before finalization
                #[cfg(feature = "debug-logs")]
                debug_log!(
                    "STACK_DEBUG: Right before finalize_execution_result - stack size: {}",
                    ctx.size() as u32
                );
                
                // Do NOT reset temp buffer here, as we want to return it in the context
                Self::finalize_execution_result(&mut ctx)
                    .map_err(VMError::from)
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
                Err(e)
            }
        }
    }

    // Decomposed execute_direct() for improved maintainability and performance
    // Four focused functions handle initialization, execution, dispatch, and finalization

    /// Parse optimized script header V3 (10 bytes)
    ///
    /// **TRUST Deploy-Time Verification**
    /// This function assumes bytecode was verified during deployment:
    /// - Header format is valid (magic, features, counts)
    /// - All opcodes are valid
    /// - CALL targets are within bounds
    /// - Function counts are consistent and within limits
    /// - Function name metadata format is valid (if present)
    ///
    /// Returns (instruction_pointer_start, public_function_count, total_function_count, features)
    #[inline]
    fn parse_optimized_header(script: &[u8]) -> CompactResult<(usize, u8, u8, u32)> {
        // Minimum bounds check for safety
        if script.len() < FIVE_HEADER_OPTIMIZED_SIZE {
            return Err(VMErrorCode::InvalidScript);
        }

        // Validate magic bytes
        if script[0..4] != FIVE_MAGIC {
            return Err(VMErrorCode::InvalidScript);
        }

        // Fast extraction from verified bytecode
        let features = u32::from_le_bytes([script[4], script[5], script[6], script[7]]);
        let public_function_count = script[8];
        let total_function_count = script[9];

        // Validate function count consistency
        if public_function_count > total_function_count {
            return Err(VMErrorCode::InvalidScript);
        }

        // Validate total_count is reasonable for script size
        // Each function needs at least 1 byte, so total_count can't exceed available space
        let available_space = script.len().saturating_sub(FIVE_HEADER_OPTIMIZED_SIZE);
        if (total_function_count as usize) > available_space {
            return Err(VMErrorCode::InvalidScript);
        }

        // Compute instruction start offset (skip metadata if present)
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

    /// Fast metadata offset computation (trust deploy-time validation)
    /// Skips VLE validation since deploy-time ensures format is valid
    #[inline]
    fn compute_instruction_start_fast(script: &[u8], features: u32, public_count: u8) -> usize {
        const FEATURE_FUNCTION_NAMES: u32 = 1 << 8;

        // No metadata or no public functions = standard 10-byte header
        if (features & FEATURE_FUNCTION_NAMES) == 0 || public_count == 0 {
            return FIVE_HEADER_OPTIMIZED_SIZE;
        }

        // TRUST: Metadata format was validated at deploy-time
        // Quick VLE decode without bounds checking (format guaranteed valid)
        let mut offset = FIVE_HEADER_OPTIMIZED_SIZE;
        let mut section_size = 0u16;
        let mut shift = 0;

        // Simplified VLE decode (no format validation needed)
        while offset < script.len() && shift < 16 {
            let byte = script[offset];
            section_size |= ((byte & 0x7F) as u16) << shift;
            offset += 1;
            if byte & 0x80 == 0 {
                break;
            }
            shift += 7;
        }

        // instruction_start = 10 (header) + VLE bytes + metadata bytes
        (offset + section_size as usize).min(script.len())
    }

    // 🚀 OPTIMIZED: Function validation removed for performance
    // Simple bounds checking is done inline during function dispatch
    // Function 0 is always the main entry point
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
        // NOTE: We no longer validate magic bytes at execute-time
        // Trust deploy-time verification instead for performance
        // This test verifies the parser still works with valid bytecode

        // Create script with 10+ bytes (minimum valid format)
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
        assert_eq!(start_ip, 10); // No metadata, standard header size
    }

    #[test]
    fn parse_optimized_header_minimum_size() {
        // Minimum valid script is 10 bytes (header)
        // NOTE: This test trusts deploy-time verified bytecode

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
