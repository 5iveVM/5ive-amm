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
            ctx.parameters_mut()[..8].copy_from_slice(&parsed_params);
        }

        // Store parsed parameters metadata
        debug_log!("MitoVM: Setting pre-parsed parameters in ExecutionContext...");
        ctx.frame.param_start = 0;
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
            if !ctx.parameters()[i].is_empty() {
                param_count = param_count.saturating_add(1);
                max_param_index = i as u8;
            }
        }

        // Set param_len to actual count (not MAX_PARAMETERS)
        ctx.frame.param_len = param_count;

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
            let param = ctx.parameters()[i];
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
        let dispatch_ip = if !ctx.parameters()[0].is_empty() {
            if let ValueRef::U64(func_index) = ctx.parameters()[0] {
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

        match opcode & 0xF0 {
            0x00 => {
                // Control flow operations (HALT, JUMP, etc.)
                handle_control_flow(opcode, ctx)
            }
            0x10..=0x1F => {
                handle_stack_ops(opcode, ctx)
            }
            0x20..=0x2F => {
                handle_arithmetic(opcode, ctx)
            }
            0x30..=0x3F => {
                handle_logical(opcode, ctx)
            }
            0x40..=0x4F => {
                // Memory instructions (STORE/LOAD/STORE_FIELD etc)
                handle_memory(opcode, ctx)
            }
            0x50..=0x5F => {
                handle_accounts(opcode, ctx)
            }
            0x60..=0x6F => {
                // 🎯 LOGICAL REORGANIZATION: Arrays now at 0x60 (moved from scattered locations)
                handle_arrays(opcode, ctx)
            }
            0x70..=0x7F => {
                // 🎯 LOGICAL REORGANIZATION: Constraints moved from 0x60 to 0x70
                handle_constraints(opcode, ctx)
            }
            0x80..=0x8F => {
                // 🎯 LOGICAL REORGANIZATION: System operations moved from 0x70 to 0x80
                handle_system_ops(opcode, ctx)
            }
            0x90..=0x9F => {
                // 🎯 LOGICAL REORGANIZATION: Functions moved from 0x80 to 0x90
                handle_functions(opcode, ctx)
            }
            0xA0..=0xAF => {
                // 🎯 LOGICAL REORGANIZATION: Locals moved from 0x90 to 0xA0 + general operations
                handle_locals(opcode, ctx)
            }
            0xB0..=0xBF => {
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
            let result = Self::dispatch_opcode_range(opcode, ctx);

            // Check result and provide clear error context
            if let Err(e) = result {
                // Enhanced error context with full VM state
                stack_error_context!(opcode, ctx, "EXECUTION_FAILED", 0, ctx.size());
                error_log!("MitoVM: ERROR_DETAILS: error_occurred at current_ip: {}", current_ip as u64);
                error_log!("OPCODE FAILED: {}", opcode as u64);
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
        debug_log!("MitoVM: execute_direct ENTRY - script={} input={} accounts={}",
            script.len() as u32, input_data.len() as u32, accounts.len() as u32);

        let mut storage = crate::stack::StackStorage::new(script);
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
        // Phase 1: Initialize execution context with script validation and function dispatch
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
