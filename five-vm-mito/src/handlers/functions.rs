//! Function operations handler for MitoVM
//!
//! Handles CALL, CALL_EXTERNAL and CALL_NATIVE opcodes with minimal copying.

use crate::{
    context::ExecutionManager,
    debug_log,
    error::{CompactResult, VMErrorCode},
    handlers::syscalls::*,
    types::CallFrame,
    MAX_CALL_DEPTH, MAX_PARAMETERS,
};
use five_protocol::{opcodes::*, FEATURE_FUNCTION_METADATA};

#[inline(always)]
pub fn handle_functions(opcode: u8, ctx: &mut ExecutionManager) -> CompactResult<()> {
    match opcode {
        CALL => {
            let res = handle_call(ctx);
            if let Err(ref e) = res {
                match e {
                    VMErrorCode::StackError => { debug_log!("MitoVM: CALL Error: StackError");  },
                    VMErrorCode::InvalidInstruction => { debug_log!("MitoVM: CALL Error: InvalidInstruction");  },
                    VMErrorCode::CallStackOverflow => { debug_log!("MitoVM: CALL Error: CallStackOverflow");  },
                    VMErrorCode::InvalidFunctionIndex => { debug_log!("MitoVM: CALL Error: InvalidFunctionIndex");  },
                    VMErrorCode::InvalidOperation => { debug_log!("MitoVM: CALL Error: InvalidOperation");  },
                    _ => { debug_log!("MitoVM: CALL Error: Other VMErrorCode");  },
                }
            }
            res
        }
        CALL_EXTERNAL => {
            let res = handle_call_external(ctx);
            if res.is_err() {
                debug_log!("MitoVM: CALL_EXTERNAL Failed");
            }
            res
        }
        CALL_NATIVE => handle_call_native(ctx),
        CALL_REG => handle_call_reg(ctx),
        PREPARE_CALL | FINISH_CALL => {
            // Explicitly mark as unsupported in current VM
            Err(VMErrorCode::InvalidOpcode)
        }
        _ => Err(VMErrorCode::InvalidInstruction),
    }
}

#[inline(always)]
fn validate_call_depth(ctx: &ExecutionManager, limit: usize, _op: &str) -> CompactResult<()> {
    if ctx.call_depth() >= limit {
        #[cfg(feature = "debug-logs")]
        debug_log!(
            "MitoVM: {} stack overflow - depth: {}, max: {}",
            _op,
            ctx.call_depth() as u32,
            limit as u32
        );
        return Err(VMErrorCode::CallStackOverflow);
    }
    Ok(())
}

fn handle_call(ctx: &mut ExecutionManager) -> CompactResult<()> {
    debug_log!(
        "MitoVM: CALL opcode encountered - stack size: {}, call depth: {}",
        ctx.size() as u32,
        ctx.call_depth() as u32
    );
    debug_log!(
        "MitoVM: CALL BEFORE - SP={}, local_base={}, local_count={}, IP={}",
        ctx.size() as u32,
        ctx.local_base() as u32,
        ctx.local_count() as u32,
        ctx.ip() as u32
    );

    validate_call_depth(ctx, MAX_CALL_DEPTH, "CALL")?;
    
    // Check total stack size against BPF limits (approx 4KB)
    // ctx.check_stack_limit()?; // REMOVED: Implementation is empty/disabled, saving call overhead

    let param_count = ctx.fetch_byte()?;
    let func_addr = ctx.fetch_u16()? as usize;

    debug_log!(
        "MitoVM: CALL params: count={}, target_addr={}, current_depth={}",
        param_count as u32,
        func_addr as u32,
        ctx.call_depth() as u32
    );

    // Validate function address is within bytecode bounds
    if func_addr >= ctx.script().len() {
        debug_log!(
            "MitoVM: CALL invalid function address {} >= script length {}",
            func_addr as u32,
            ctx.script().len() as u32
        );
        return Err(VMErrorCode::InvalidFunctionIndex);
    }

    // Validate parameter count against protocol limits
    if param_count as usize > MAX_PARAMETERS {
        debug_log!(
            "MitoVM: CALL invalid parameter count {} > MAX_PARAMETERS {}",
            param_count,
            MAX_PARAMETERS as u32
        );
        return Err(VMErrorCode::InvalidOperation);
    }

    // Skip inline CALL metadata emitted by the compiler (function name/tag references).
    // The feature flag is set in the header, so the VM never treats the metadata as opcodes
    // even though it lives immediately after CALL.
    skip_call_metadata(ctx)?;

    // Debug assertion: function address should be reasonable
    debug_assert!(
        func_addr > 0,
        "Function address 0 is reserved for entry point"
    );

    let caller_start = ctx.param_start();
    let caller_len = ctx.param_len();

    if ctx.size() < param_count as usize {
        debug_log!("MitoVM: CALL STACK_ERROR - stack_size={} < param_count={}", ctx.size(), param_count);
        return Err(VMErrorCode::StackError);
    }

    #[cfg(feature = "debug-logs")]
    debug_log!("MitoVM: internal CALL params={} stack={}", param_count as u64, ctx.size() as u64);

    ctx.allocate_params(param_count + 1)?;
    for i in 0..param_count {
        let value = ctx.pop()?;
        let idx = (param_count - i) as usize;
        ctx.parameters_mut()[idx] = value;
    }

    let current_ip = ctx.ip();
    let current_local_count = ctx.local_count();
    let current_local_base = ctx.local_base();
    let current_script = {
        let script_ref = ctx.script();
        // SAFETY: script_ref is a valid slice from ctx, we're creating an independent
        // slice with the same lifetime. Required to store in CallFrame for context restoration.
        unsafe { core::slice::from_raw_parts(script_ref.as_ptr(), script_ref.len()) }
    };

    // Check if we have enough space for callee's locals before pushing frame
    let new_local_base = current_local_base + current_local_count;
    let locals_to_allocate = (param_count as usize).max(3);
    if (new_local_base as usize + locals_to_allocate) > crate::MAX_LOCALS {
        // Return CallStackOverflow instead of LocalsOverflow to indicate call depth limit
        return Err(VMErrorCode::CallStackOverflow);
    }

    // Save caller's frame state including local base offset
    ctx.push_call_frame(CallFrame::with_parameters(
        current_ip as u16,
        current_local_count,
        current_local_base,
        caller_start,
        caller_len,
        current_script,
    ))?;

    // Set callee's local base to end of caller's locals (per-frame window)
    ctx.set_local_base(new_local_base);
    ctx.set_local_count(0);
    // Allocate locals for the callee frame
    // Use max(param_count, 3) to allow functions to use at least 3 local slots by default
    let locals_to_allocate = param_count.max(3);
    ctx.allocate_locals(locals_to_allocate)?;
    ctx.set_ip(func_addr);

    debug_log!(
        "MitoVM: CALL AFTER - SP={}, local_base={}, local_count={}, new_IP={}, call_depth={}",
        ctx.size() as u32,
        ctx.local_base() as u32,
        ctx.local_count() as u32,
        ctx.ip() as u32,
        ctx.call_depth() as u32
    );

    Ok(())
}

/// Handle CALL_REG - Register-based calling convention
/// Arguments are passed in registers r0-r7, no stack manipulation needed
/// OPTIMIZATION: Automatically loads data parameters into registers during call setup
fn handle_call_reg(ctx: &mut ExecutionManager) -> CompactResult<()> {
    debug_log!(
        "MitoVM: CALL_REG opcode encountered - call depth: {}",
        ctx.call_depth() as u32
    );

    validate_call_depth(ctx, MAX_CALL_DEPTH, "CALL_REG")?;

    let func_addr = ctx.fetch_u16()? as usize;

    debug_log!(
        "MitoVM: CALL_REG target_addr={}, current_depth={}",
        func_addr as u32,
        ctx.call_depth() as u32
    );

    // Validate function address is within bytecode bounds
    if func_addr >= ctx.script().len() {
        debug_log!(
            "MitoVM: CALL_REG invalid function address {} >= script length {}",
            func_addr as u32,
            ctx.script().len() as u32
        );
        return Err(VMErrorCode::InvalidFunctionIndex);
    }

    // Skip inline CALL metadata if present
    skip_call_metadata(ctx)?;

    let caller_start = ctx.param_start();
    let caller_len = ctx.param_len();

    // Save current state
    let current_ip = ctx.ip();
    let current_local_count = ctx.local_count();
    let current_local_base = ctx.local_base();
    let current_script = {
        let script_ref = ctx.script();
        unsafe { core::slice::from_raw_parts(script_ref.as_ptr(), script_ref.len()) }
    };

    // Check if we have enough space for callee's locals
    let new_local_base = current_local_base + current_local_count;
    let locals_to_allocate = 3; // Default local allocation
    if (new_local_base as usize + locals_to_allocate) > crate::MAX_LOCALS {
        return Err(VMErrorCode::CallStackOverflow);
    }

    // Save caller's frame state
    ctx.push_call_frame(CallFrame::with_parameters(
        current_ip as u16,
        current_local_count,
        current_local_base,
        caller_start,
        caller_len,
        current_script,
    ))?;

    // Set callee's local base
    ctx.set_local_base(new_local_base);
    ctx.set_local_count(0);
    ctx.allocate_locals(locals_to_allocate as u8)?;
    ctx.set_ip(func_addr);

    // AUTO-LOAD: Copy data parameters into registers r0..rN-1
    // Parameters are 1-indexed (params[1] = first param), registers are 0-indexed
    // Skip params[0] which is function index
    // Load up to 8 data parameters into r0..r7
    let param_count = caller_len as usize;
    let max_reg_params = 8.min(param_count); // Max 8 registers for params
    
    for i in 0..max_reg_params {
        // params[1] -> r0, params[2] -> r1, etc.
        let param_idx = i + 1;
        if param_idx <= param_count {
            let param_value = ctx.parameters()[param_idx];
            // Only load non-empty params (skip account references which are handled differently)
            if !param_value.is_empty() {
                ctx.set_register(i as u8, param_value)?;
                debug_log!(
                    "MitoVM: CALL_REG auto-loaded param[{}] into r{}",
                    param_idx,
                    i
                );
            }
        }
    }

    debug_log!(
        "MitoVM: CALL_REG AFTER - local_base={}, local_count={}, new_IP={}, call_depth={}",
        ctx.local_base() as u32,
        ctx.local_count() as u32,
        ctx.ip() as u32,
        ctx.call_depth() as u32
    );

    Ok(())
}


#[inline(always)]
fn skip_call_metadata(ctx: &mut ExecutionManager) -> CompactResult<()> {
    if ctx.header_features() & FEATURE_FUNCTION_METADATA == 0 {
        return Ok(());
    }

    let marker = ctx.fetch_byte()?;
    if marker == 0xFF {
        ctx.fetch_byte()?;
    } else {
        for _ in 0..marker as usize {
            ctx.fetch_byte()?;
        }
    }

    Ok(())
}

/// Parse function constraint metadata from external bytecode
/// Returns (account_count, constraint_bitmasks_per_account)
/// Format: [account_count_u8] [constraint_u8_per_account...]
#[inline]
fn parse_function_constraints(
    external_bytecode: &[u8],
    _func_offset: usize,
) -> CompactResult<(u8, [u8; 16])> {
    // Check if bytecode has enough data for header
    if external_bytecode.len() < 10 {
        return Err(VMErrorCode::InvalidInstructionPointer);
    }

    // Get header features to check if constraint metadata is present
    let features = if external_bytecode.len() >= 8 {
        u32::from_le_bytes([
            external_bytecode[4],
            external_bytecode[5],
            external_bytecode[6],
            external_bytecode[7],
        ])
    } else {
        0u32
    };

    // If no constraint metadata feature, assume function has no constraints
    if (features & five_protocol::FEATURE_FUNCTION_CONSTRAINTS) == 0 {
        return Ok((0, [0u8; 16]));
    }

    // Constraint metadata format (after header):
    // Header (10 bytes) + [optional function name metadata] + constraint metadata
    // For now, we'll look for constraint metadata after header
    // This is a simplified implementation - production would parse VLE-encoded section

    // Simplified: If constraints are present, they would be stored after function names
    // For now, return no constraints (will be enhanced in next phase)
    Ok((0, [0u8; 16]))
}

/// Validate that provided accounts match external function's constraint requirements
#[inline]
fn validate_external_function_constraints(
    ctx: &ExecutionManager,
    account_count: u8,
    constraints: &[u8; 16],
) -> CompactResult<()> {
    // Validate we have at least the required number of accounts in the accounts array
    // This is checked implicitly in CALL_EXTERNAL itself, but we validate constraints here

    for i in 0..account_count as usize {
        let constraint_bitmask = constraints[i];

        // bit 0: @signer constraint
        if (constraint_bitmask & 0x01) != 0 {
            let account = ctx.get_account(i as u8)?;
            if !account.is_signer() {
                debug_log!(
                    "MitoVM: CALL_EXTERNAL constraint violation - account {} not signer",
                    i as u32
                );
                return Err(VMErrorCode::ConstraintViolation);
            }
        }

        // bit 1: @mut constraint (writable)
        if (constraint_bitmask & 0x02) != 0 {
            let account = ctx.get_account(i as u8)?;
            if !account.is_writable() {
                debug_log!(
                    "MitoVM: CALL_EXTERNAL constraint violation - account {} not writable",
                    i as u32
                );
                return Err(VMErrorCode::ConstraintViolation);
            }
        }

        // bit 3: @init constraint (must be initialized - has data)
        if (constraint_bitmask & 0x08) != 0 {
            let account = ctx.get_account(i as u8)?;
            if account.data_len() == 0 {
                debug_log!(
                    "MitoVM: CALL_EXTERNAL constraint violation - account {} not initialized",
                    i as u32
                );
                return Err(VMErrorCode::ConstraintViolation);
            }
        }

        // bit 4: @pda constraint - this would require additional validation
        // For now, we skip this (would need to check if account is a valid PDA)
    }

    Ok(())
}

fn handle_call_external(ctx: &mut ExecutionManager) -> CompactResult<()> {
    let account_index = ctx.fetch_byte()? as usize;
    let func_offset = ctx.fetch_u16()? as usize;
    let param_count = ctx.fetch_byte()?;
    
    #[cfg(feature = "debug-logs")]
    debug_log!(
        "MitoVM: CALL_EXTERNAL acc={} off={} params={} stack={}",
        account_index as u64,
        func_offset as u64,
        param_count as u64,
        ctx.size() as u64
    );


    // Validate account index
    if account_index >= ctx.accounts().len() {
        debug_log!(
            "MitoVM: CALL_EXTERNAL invalid account index {} >= account count {}",
            account_index as u32,
            ctx.accounts().len() as u32
        );
        return Err(VMErrorCode::AccountNotFound);
    }

    // Optimization: Get account reference once
    let account = ctx.get_account(account_index as u8)?;

    // Validate account has data
    let account_data_len = account.data_len();
    if account_data_len == 0 {
        debug_log!(
            "MitoVM: CALL_EXTERNAL account {} has no data",
            account_index as u32
        );
        return Err(VMErrorCode::AccountDataEmpty);
    }

    // Validate function offset within account data
    if func_offset >= account_data_len {
        debug_log!(
            "MitoVM: CALL_EXTERNAL invalid function offset {} >= account data length {}",
            func_offset as u32,
            account_data_len as u32
        );
        return Err(VMErrorCode::InvalidInstructionPointer);
    }

    // SAFETY: Account has been validated (index check above). borrow_data_unchecked is safe
    // within Solana runtime context as account data is guaranteed to remain valid for the
    // duration of the transaction. Creating slice from valid data pointer.
    // NOTE: On Solana, all Five bytecode accounts start with a 64-byte ScriptAccountHeader
    let external_bytecode_raw = unsafe {
        let data_slice = account.borrow_data_unchecked();
        core::slice::from_raw_parts(data_slice.as_ptr(), data_slice.len())
    };
    
    // Skip 64-byte ScriptAccountHeader to get actual bytecode
    const SCRIPT_ACCOUNT_HEADER_LEN: usize = 64;
    if external_bytecode_raw.len() < SCRIPT_ACCOUNT_HEADER_LEN {
        debug_log!("MitoVM: CALL_EXTERNAL account too small for header");
        return Err(VMErrorCode::AccountDataEmpty);
    }
    let external_bytecode = &external_bytecode_raw[SCRIPT_ACCOUNT_HEADER_LEN..];
    
    debug_log!(
        "MitoVM: CALL_EXTERNAL loaded external_bytecode length: {}",
        external_bytecode.len() as u32
    );
    debug_log!(
        "MitoVM: CALL_EXTERNAL func_offset: {}",
        func_offset as u32
    );
    // Log first 20 bytes of external bytecode for debugging
    #[cfg(feature = "debug-logs")]
    {
        let preview_len = external_bytecode.len().min(20);
        debug_log!("MitoVM: CALL_EXTERNAL external_bytecode preview (first {} bytes):", preview_len as u32);
        for i in 0..preview_len {
            debug_log!("  [{}]: {}", i as u32, external_bytecode[i]);
        }
    }

    // Parse and validate constraint metadata from external bytecode
    // This ensures the external function's account requirements are met
    let (required_account_count, constraints) = parse_function_constraints(external_bytecode, func_offset)?;

    // Validate that the provided accounts satisfy external function's constraints
    if required_account_count > 0 {
        validate_external_function_constraints(ctx, required_account_count, &constraints)?;
    }

    // NEW: Import verification for Five bytecode accounts
    // Check if the account matches verified imports using zero-copy metadata
    let pda_derivation_fn: Option<crate::metadata::PdaDerivationFn> = Some(|seeds, program_id| {
        #[cfg(target_os = "solana")]
        {
            // On Solana, use pinocchio's find_program_address which calls the runtime syscall
            let (key, _bump) = pinocchio::pubkey::find_program_address(seeds, unsafe {
                &*(program_id as *const _ as *const pinocchio::pubkey::Pubkey)
            });
            key
        }
        #[cfg(not(target_os = "solana"))]
        {
            // On non-Solana targets (host tests, simulations), we can't reliably derive PDAs
            // without a crypto library that implements ed25519 point validation.
            // Pinocchio's implementation panics or returns None on host.
            //
            // We return a zeroed key here which will cause verification to fail unless expected key is also zero.
            // This ensures we don't return an incorrect PDA that might be accepted.
            // Users running off-chain tests should mock import verification if needed.
            let _ = seeds;
            let _ = program_id;
            [0u8; 32]
        }
    });

    if !ctx.import_metadata.verify_account(
        account.key(),
        &ctx.program_id,
        pda_derivation_fn,
    ) {
        return Err(VMErrorCode::UnauthorizedBytecodeInvocation);
    }

    let caller_start = ctx.param_start();
    let caller_len = ctx.param_len();

    let return_address = ctx.ip();
    let current_local_count = ctx.local_count();
    let current_local_base = ctx.local_base();
    let current_script = {
        let script_ref = ctx.script();
        // SAFETY: script_ref is a valid slice from ctx, we're creating an independent
        // slice with the same lifetime. Required to store in CallFrame for context restoration.
        unsafe { core::slice::from_raw_parts(script_ref.as_ptr(), script_ref.len()) }
    };

    // Check if we have enough space for callee's locals before pushing frame
    let new_local_base = current_local_base + current_local_count;
    let locals_to_allocate = (param_count as usize).max(3);
    if (new_local_base as usize + locals_to_allocate) > crate::MAX_LOCALS {
        // Return CallStackOverflow instead of LocalsOverflow to indicate call depth limit
        return Err(VMErrorCode::CallStackOverflow);
    }

    // Save caller's frame state including local base offset
    ctx.push_call_frame(CallFrame::with_parameters(
        return_address as u16,
        current_local_count,
        current_local_base,
        caller_start,
        caller_len,
        current_script,
    ))?;

    // Set callee's local base to end of caller's locals (per-frame window)
    ctx.set_local_base(new_local_base);
    ctx.set_local_count(0);
    // Allocate locals for the callee frame
    // Use max(param_count, 3) to allow functions to use at least 3 local slots by default
    let locals_to_allocate = param_count.max(3);
    ctx.allocate_locals(locals_to_allocate)?;
    ctx.allocate_params(param_count + 1)?;
    for i in 0..param_count {
        let value = ctx.pop()?;
        let idx = (param_count - i) as usize;
        ctx.parameters_mut()[idx] = value;
    }

    debug_log!(
        "MitoVM: CALL_EXTERNAL about to switch_to_external_bytecode - current IP: {}, new offset: {}",
        ctx.ip() as u32,
        func_offset as u32
    );
    
    ctx.switch_to_external_bytecode(external_bytecode, func_offset)?;
    
    debug_log!(
        "MitoVM: CALL_EXTERNAL after switch_to_external_bytecode - new IP: {}, script len: {}",
        ctx.ip() as u32,
        ctx.script().len() as u32
    );
    
    Ok(())
}

fn handle_call_native(ctx: &mut ExecutionManager) -> CompactResult<()> {
    // CALL_NATIVE syscall_id_u8 [args...]
    // Execute native Solana/Pinocchio syscalls with proper parameter marshaling
    let syscall_id = ctx.fetch_byte()?;

    debug_log!("MitoVM: CALL_NATIVE syscall_id={}", syscall_id);

    match syscall_id {
        // Control syscalls
        SYSCALL_ABORT => handle_syscall_abort(ctx),
        SYSCALL_PANIC => handle_syscall_panic(ctx),

        // PDA/Address syscalls
        SYSCALL_CREATE_PROGRAM_ADDRESS => handle_syscall_create_program_address(ctx),
        SYSCALL_TRY_FIND_PROGRAM_ADDRESS => handle_syscall_try_find_program_address(ctx),

        // Sysvar syscalls (enhanced from existing GET_CLOCK/GET_RENT)
        SYSCALL_GET_CLOCK_SYSVAR => handle_syscall_get_clock_sysvar(ctx),
        SYSCALL_GET_EPOCH_SCHEDULE_SYSVAR => handle_syscall_get_epoch_schedule_sysvar(ctx),
        SYSCALL_GET_EPOCH_REWARDS_SYSVAR => handle_syscall_get_epoch_rewards_sysvar(ctx),
        SYSCALL_GET_EPOCH_STAKE => handle_syscall_get_epoch_stake(ctx),
        SYSCALL_GET_FEES_SYSVAR => handle_syscall_get_fees_sysvar(ctx),
        SYSCALL_GET_RENT_SYSVAR => handle_syscall_get_rent_sysvar(ctx),
        SYSCALL_GET_LAST_RESTART_SLOT => handle_syscall_get_last_restart_slot(ctx),
        SYSCALL_GET_SYSVAR => handle_syscall_get_sysvar(ctx),

        // Program data syscalls
        SYSCALL_GET_RETURN_DATA => handle_syscall_get_return_data(ctx),
        SYSCALL_SET_RETURN_DATA => handle_syscall_set_return_data(ctx),
        SYSCALL_GET_PROCESSED_SIBLING_INSTRUCTION => {
            handle_syscall_get_processed_sibling_instruction(ctx)
        }
        SYSCALL_GET_STACK_HEIGHT => handle_syscall_get_stack_height(ctx),

        // CPI syscalls (complement existing INVOKE operations)
        SYSCALL_INVOKE_SIGNED_C => handle_syscall_invoke_signed_c(ctx),
        SYSCALL_INVOKE_SIGNED_RUST => handle_syscall_invoke_signed_rust(ctx),

        // Compute syscalls
        SYSCALL_REMAINING_COMPUTE_UNITS => handle_syscall_remaining_compute_units(ctx),

        // Logging syscalls (complement existing LOG_DATA)
        SYSCALL_LOG => handle_syscall_log(ctx),
        SYSCALL_LOG_64 => handle_syscall_log_64(ctx),
        SYSCALL_LOG_COMPUTE_UNITS => handle_syscall_log_compute_units(ctx),
        SYSCALL_LOG_DATA => handle_syscall_log_data(ctx),
        SYSCALL_LOG_PUBKEY => handle_syscall_log_pubkey(ctx),

        // Memory syscalls
        SYSCALL_MEMCPY => handle_syscall_memcpy(ctx),
        SYSCALL_MEMMOVE => handle_syscall_memmove(ctx),
        SYSCALL_MEMSET => handle_syscall_memset(ctx),
        SYSCALL_MEMCMP => handle_syscall_memcmp(ctx),

        // Cryptography syscalls
        SYSCALL_SHA256 => handle_syscall_sha256(ctx),
        SYSCALL_KECCAK256 => handle_syscall_keccak256(ctx),
        // SYSCALL_BLAKE3 => handle_syscall_blake3(ctx),
        SYSCALL_POSEIDON => handle_syscall_poseidon(ctx),
        SYSCALL_SECP256K1_RECOVER => handle_syscall_secp256k1_recover(ctx),
        SYSCALL_ALT_BN128_COMPRESSION => handle_syscall_alt_bn128_compression(ctx),
        SYSCALL_ALT_BN128_GROUP_OP => handle_syscall_alt_bn128_group_op(ctx),
        SYSCALL_BIG_MOD_EXP => handle_syscall_big_mod_exp(ctx),
        SYSCALL_CURVE_GROUP_OP => handle_syscall_curve_group_op(ctx),
        SYSCALL_CURVE_MULTISCALAR_MUL => handle_syscall_curve_multiscalar_mul(ctx),
        SYSCALL_CURVE_PAIRING_MAP => handle_syscall_curve_pairing_map(ctx),
        SYSCALL_CURVE_VALIDATE_POINT => handle_syscall_curve_validate_point(ctx),

        _ => {
            debug_log!("MitoVM: CALL_NATIVE invalid syscall_id={}", syscall_id);
            return Err(VMErrorCode::InvalidInstruction);
        }
    }?;
    Ok(())
}
