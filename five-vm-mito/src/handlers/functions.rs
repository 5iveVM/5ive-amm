//! Function operations handler for MitoVM
//!
//! Handles CALL, CALL_EXTERNAL and CALL_NATIVE opcodes with minimal copying.

use crate::{
    context::ExecutionManager,
    debug_log,
    error::{CompactResult, VMErrorCode},
    handlers::syscalls::*,
    types::{CallFrame, ExternalCallCacheEntry},
    MAX_CALL_DEPTH, MAX_PARAMETERS, STACK_SIZE,
};
use five_protocol::{opcodes::*, ValueRef, FEATURE_FUNCTION_METADATA, FEATURE_FUNCTION_NAMES};

#[inline(never)]
pub fn handle_functions(opcode: u8, ctx: &mut ExecutionManager) -> CompactResult<()> {
    match opcode {
        CALL => {
            let res = handle_call(ctx);
            if let Err(ref e) = res {
                match e {
                    VMErrorCode::StackError => {
                        debug_log!("MitoVM: CALL Error: StackError");
                    }
                    VMErrorCode::InvalidInstruction => {
                        debug_log!("MitoVM: CALL Error: InvalidInstruction");
                    }
                    VMErrorCode::CallStackOverflow => {
                        debug_log!("MitoVM: CALL Error: CallStackOverflow");
                    }
                    VMErrorCode::InvalidFunctionIndex => {
                        debug_log!("MitoVM: CALL Error: InvalidFunctionIndex");
                    }
                    VMErrorCode::InvalidOperation => {
                        debug_log!("MitoVM: CALL Error: InvalidOperation");
                    }
                    _ => {
                        debug_log!("MitoVM: CALL Error: Other VMErrorCode");
                    }
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

#[inline(always)]
fn validate_stack_limit(ctx: &ExecutionManager, _op: &str) -> CompactResult<()> {
    if ctx.size() > STACK_SIZE {
        #[cfg(feature = "debug-logs")]
        debug_log!(
            "MitoVM: {} stack overflow - size: {}, max: {}",
            _op,
            ctx.size() as u32,
            STACK_SIZE as u32
        );
        return Err(VMErrorCode::StackOverflow);
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
    validate_stack_limit(ctx, "CALL")?;

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
    let caller_temp_offset = ctx.temp_offset() as u16;
    let mut saved_caller_parameters = [ValueRef::Empty; MAX_PARAMETERS + 1];
    saved_caller_parameters.copy_from_slice(ctx.parameters());

    if ctx.size() < param_count as usize {
        debug_log!(
            "MitoVM: CALL STACK_ERROR - stack_size={} < param_count={}",
            ctx.size(),
            param_count
        );
        return Err(VMErrorCode::StackError);
    }

    #[cfg(feature = "debug-logs")]
    debug_log!(
        "MitoVM: internal CALL params={} stack={}",
        param_count as u64,
        ctx.size() as u64
    );

    let call_args = materialize_call_args(ctx, param_count)?;
    ctx.allocate_params(param_count + 1)?;
    {
        let params = ctx.parameters_mut();
        for (i, value) in call_args.iter().take(param_count as usize).enumerate() {
            params[i + 1] = *value;
        }
    }

    let current_ip = ctx.ip();
    let script_ptr = ctx.script().as_ptr() as usize;
    let script_len = ctx.script().len() as u32;
    let current_context = ctx.current_context;
    let remap = ctx.external_account_remap();
    prepare_callee_frame(
        ctx,
        param_count,
        saved_caller_parameters,
        caller_start,
        caller_len,
        caller_temp_offset,
        current_ip,
        current_context,
        remap,
        script_ptr,
        script_len,
    )?;
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
    func_selector: usize,
) -> CompactResult<(u8, [u8; 16])> {
    // Check if bytecode has enough data for header
    if external_bytecode.len() < five_protocol::FIVE_HEADER_OPTIMIZED_SIZE {
        return Err(VMErrorCode::InvalidInstructionPointer);
    }

    // Get header features to check if constraint metadata is present
    let features = u32::from_le_bytes([
        external_bytecode[4],
        external_bytecode[5],
        external_bytecode[6],
        external_bytecode[7],
    ]);
    let total_functions = external_bytecode[9] as usize;

    // If no constraint metadata feature, assume function has no constraints
    if (features & five_protocol::FEATURE_FUNCTION_CONSTRAINTS) == 0 {
        return Ok((0, [0u8; 16]));
    }

    // Metadata starts immediately after the optimized header.
    let mut offset = five_protocol::FIVE_HEADER_OPTIMIZED_SIZE;

    // Skip optional function-name metadata section:
    // [u16 section_size] [u8 name_count] [u8 name_len + bytes]...
    if (features & FEATURE_FUNCTION_NAMES) != 0 {
        if offset + 2 > external_bytecode.len() {
            return Err(VMErrorCode::InvalidInstructionPointer);
        }
        let section_size =
            u16::from_le_bytes([external_bytecode[offset], external_bytecode[offset + 1]]) as usize;
        offset += 2;
        if offset + section_size > external_bytecode.len() {
            return Err(VMErrorCode::InvalidInstructionPointer);
        }
        offset += section_size;
    }

    // Skip optional public-entry table section when present.
    if (features & five_protocol::FEATURE_PUBLIC_ENTRY_TABLE) != 0 {
        if offset + 2 > external_bytecode.len() {
            return Ok((0, [0u8; 16]));
        }
        let public_section_size =
            u16::from_le_bytes([external_bytecode[offset], external_bytecode[offset + 1]]) as usize;
        offset += 2;
        if offset + public_section_size > external_bytecode.len() {
            return Ok((0, [0u8; 16]));
        }
        offset += public_section_size;
    }

    // If constant-pool descriptor starts here, there is no dedicated constraints section.
    if (features & five_protocol::FEATURE_CONSTANT_POOL) != 0 {
        return Ok((0, [0u8; 16]));
    }

    // Constraint metadata section:
    // [u16 section_size] [entries...]
    // Entry (fixed-width): [account_count:u8] [constraint_bitmask:u8;16]
    // We also accept an optional u8 entry_count prefix inside section payload.
    if offset + 2 > external_bytecode.len() {
        return Ok((0, [0u8; 16]));
    }
    let section_size =
        u16::from_le_bytes([external_bytecode[offset], external_bytecode[offset + 1]]) as usize;
    offset += 2;
    if offset + section_size > external_bytecode.len() {
        return Ok((0, [0u8; 16]));
    }
    if section_size == 0 {
        return Ok((0, [0u8; 16]));
    }

    let section = &external_bytecode[offset..offset + section_size];
    let entry_size = 17usize;

    let (entry_count, entries_start_in_section) = if section_size == total_functions * entry_size {
        (total_functions, 0usize)
    } else if section_size >= 1 && (section_size - 1) % entry_size == 0 {
        let count = (section_size - 1) / entry_size;
        if section[0] as usize == count {
            (count, 1usize)
        } else {
            return Ok((0, [0u8; 16]));
        }
    } else if section_size % entry_size == 0 {
        (section_size / entry_size, 0usize)
    } else {
        return Ok((0, [0u8; 16]));
    };

    if func_selector >= entry_count {
        // External calls can pass a function offset in some call paths.
        // If we cannot resolve selector->entry deterministically, do not enforce.
        return Ok((0, [0u8; 16]));
    }

    let entry_offset = entries_start_in_section + (func_selector * entry_size);
    if entry_offset + entry_size > section.len() {
        return Ok((0, [0u8; 16]));
    }

    let account_count = section[entry_offset];
    if account_count > 16 {
        return Ok((0, [0u8; 16]));
    }

    let mut constraints = [0u8; 16];
    constraints.copy_from_slice(&section[entry_offset + 1..entry_offset + entry_size]);
    Ok((account_count, constraints))
}

#[inline]
fn external_selector(name: &str) -> u16 {
    const OFFSET: u32 = 0x811C9DC5;
    const PRIME: u32 = 0x01000193;
    let mut hash = OFFSET;
    for b in name.as_bytes() {
        hash ^= *b as u32;
        hash = hash.wrapping_mul(PRIME);
    }
    (hash & 0xFFFF) as u16
}

#[inline]
fn external_code_fingerprint(external_bytecode: &[u8]) -> u32 {
    const OFFSET: u32 = 0x811C9DC5;
    const PRIME: u32 = 0x01000193;
    let mut hash = OFFSET ^ (external_bytecode.len() as u32);
    let sample_len = external_bytecode.len().min(16);
    for b in &external_bytecode[..sample_len] {
        hash ^= *b as u32;
        hash = hash.wrapping_mul(PRIME);
    }
    if external_bytecode.len() >= five_protocol::FIVE_HEADER_OPTIMIZED_SIZE {
        hash ^= u32::from_le_bytes([
            external_bytecode[4],
            external_bytecode[5],
            external_bytecode[6],
            external_bytecode[7],
        ]);
        hash = hash.wrapping_mul(PRIME);
        hash ^= (external_bytecode[8] as u32) << 8 | external_bytecode[9] as u32;
    }
    hash
}

#[inline]
fn parse_external_layout(
    external_bytecode: &[u8],
) -> CompactResult<(usize, Option<(usize, u8)>, Option<(usize, usize)>)> {
    if external_bytecode.len() < five_protocol::FIVE_HEADER_OPTIMIZED_SIZE {
        return Err(VMErrorCode::InvalidInstructionPointer);
    }

    let features = u32::from_le_bytes([
        external_bytecode[4],
        external_bytecode[5],
        external_bytecode[6],
        external_bytecode[7],
    ]);

    let mut offset = five_protocol::FIVE_HEADER_OPTIMIZED_SIZE;
    let mut function_names_section: Option<(usize, usize)> = None;
    let mut public_entry_table: Option<(usize, u8)> = None;

    if (features & FEATURE_FUNCTION_NAMES) != 0 {
        if offset + 2 > external_bytecode.len() {
            return Err(VMErrorCode::InvalidInstructionPointer);
        }
        let section_size =
            u16::from_le_bytes([external_bytecode[offset], external_bytecode[offset + 1]]) as usize;
        let section_start = offset + 2;
        if section_start + section_size > external_bytecode.len() {
            return Err(VMErrorCode::InvalidInstructionPointer);
        }
        function_names_section = Some((section_start, section_size));
        offset = section_start + section_size;
    }

    if (features & five_protocol::FEATURE_PUBLIC_ENTRY_TABLE) != 0 {
        if offset + 2 > external_bytecode.len() {
            return Err(VMErrorCode::InvalidInstructionPointer);
        }
        let section_size =
            u16::from_le_bytes([external_bytecode[offset], external_bytecode[offset + 1]]) as usize;
        let section_start = offset + 2;
        if section_size == 0 || section_start + section_size > external_bytecode.len() {
            return Err(VMErrorCode::InvalidInstructionPointer);
        }
        let count = external_bytecode[section_start];
        let expected = 1usize + (count as usize) * 2;
        if expected > section_size {
            return Err(VMErrorCode::InvalidInstructionPointer);
        }
        public_entry_table = Some((section_start, count));
        offset = section_start + section_size;
    }

    let code_start = if (features & five_protocol::FEATURE_CONSTANT_POOL) != 0 {
        let desc_size = core::mem::size_of::<five_protocol::ConstantPoolDescriptor>();
        if offset + desc_size > external_bytecode.len() {
            return Err(VMErrorCode::InvalidInstructionPointer);
        }
        let pool_offset = u32::from_le_bytes([
            external_bytecode[offset],
            external_bytecode[offset + 1],
            external_bytecode[offset + 2],
            external_bytecode[offset + 3],
        ]) as usize;
        let pool_slots = u16::from_le_bytes([
            external_bytecode[offset + 12],
            external_bytecode[offset + 13],
        ]) as usize;
        let code_offset = pool_offset + (pool_slots * 8);
        if code_offset >= external_bytecode.len() {
            return Err(VMErrorCode::InvalidInstructionPointer);
        }
        code_offset
    } else {
        offset
    };

    Ok((code_start, public_entry_table, function_names_section))
}

#[inline]
fn resolve_public_entry_offset(
    external_bytecode: &[u8],
    code_start: usize,
    table_start: usize,
    table_count: u8,
    function_index: usize,
) -> CompactResult<usize> {
    if function_index >= table_count as usize {
        return Err(VMErrorCode::InvalidFunctionIndex);
    }
    let entry_pos = table_start + 1 + (function_index * 2);
    if entry_pos + 1 >= external_bytecode.len() {
        return Err(VMErrorCode::InvalidInstructionPointer);
    }

    let rel = u16::from_le_bytes([
        external_bytecode[entry_pos],
        external_bytecode[entry_pos + 1],
    ]);
    let absolute = code_start
        .checked_add(rel as usize)
        .ok_or(VMErrorCode::InvalidInstructionPointer)?;
    if absolute >= external_bytecode.len() {
        return Err(VMErrorCode::InvalidInstructionPointer);
    }
    Ok(absolute)
}

fn resolve_external_function_target(
    external_bytecode: &[u8],
    selector: u16,
) -> CompactResult<(usize, Option<usize>)> {
    let (code_start, public_entry_table, function_names) =
        parse_external_layout(external_bytecode)?;

    // 1) Preferred path: selector is FNV-1a(name) and function names metadata exists.
    if let (Some((names_start, names_size)), Some((table_start, table_count))) =
        (function_names, public_entry_table)
    {
        let mut off = names_start;
        let end = names_start + names_size;
        if off < end {
            let name_count = external_bytecode[off] as usize;
            off += 1;
            for idx in 0..name_count {
                if off >= end {
                    return Err(VMErrorCode::InvalidInstructionPointer);
                }
                let len = external_bytecode[off] as usize;
                off += 1;
                if off + len > end {
                    return Err(VMErrorCode::InvalidInstructionPointer);
                }
                let name = core::str::from_utf8(&external_bytecode[off..off + len])
                    .map_err(|_| VMErrorCode::InvalidInstructionPointer)?;
                off += len;
                if external_selector(name) == selector {
                    let abs = resolve_public_entry_offset(
                        external_bytecode,
                        code_start,
                        table_start,
                        table_count,
                        idx,
                    )?;
                    return Ok((abs, Some(idx)));
                }
            }
        }
    }

    // 2) Backward compatibility: selector as public function index.
    if let Some((table_start, table_count)) = public_entry_table {
        let selector_index = selector as usize;
        if selector_index < table_count as usize {
            let abs = resolve_public_entry_offset(
                external_bytecode,
                code_start,
                table_start,
                table_count,
                selector_index,
            )?;
            return Ok((abs, Some(selector_index)));
        }
    }

    Err(VMErrorCode::InvalidInstructionPointer)
}

/// Validate that provided accounts match external function's constraint requirements
#[inline]
fn validate_external_function_constraints(
    ctx: &ExecutionManager,
    account_count: u8,
    constraints: &[u8; 16],
    remap: &[u8; MAX_PARAMETERS + 1],
    bound_account_count: u8,
) -> CompactResult<()> {
    // External constraints must be evaluated against the call's account arguments,
    // not positional transaction accounts.
    if account_count > bound_account_count {
        debug_log!(
            "MitoVM: CALL_EXTERNAL constraint violation - required accounts {} > bound account args {}",
            account_count as u32,
            bound_account_count as u32
        );
        return Err(VMErrorCode::ConstraintViolation);
    }

    for i in 0..account_count as usize {
        let constraint_bitmask = constraints[i];
        if constraint_bitmask == 0 {
            continue;
        }

        // External account slots are 1-based in the remap table.
        let remap_slot = i + 1;
        if remap_slot >= remap.len() {
            return Err(VMErrorCode::ConstraintViolation);
        }
        let mapped_account_index = remap[remap_slot];
        if mapped_account_index == u8::MAX {
            return Err(VMErrorCode::ConstraintViolation);
        }
        let account = &ctx.accounts()[mapped_account_index as usize];

        // bit 0: @signer constraint
        if (constraint_bitmask & 0x01) != 0 {
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
            if account.data_len() == 0 {
                debug_log!(
                    "MitoVM: CALL_EXTERNAL constraint violation - account {} not initialized",
                    i as u32
                );
                return Err(VMErrorCode::ConstraintViolation);
            }
        }

        // bit 4: @pda constraint
        // Fail closed until external-call metadata includes derivation material
        // sufficient for deterministic PDA verification in this path.
        if (constraint_bitmask & CONSTRAINT_PDA) != 0 {
            debug_log!(
                "MitoVM: CALL_EXTERNAL constraint violation - account {} has unsupported @pda external constraint",
                i as u32
            );
            return Err(VMErrorCode::ConstraintViolation);
        }
    }

    Ok(())
}

#[inline]
fn build_external_account_remap(
    ctx: &ExecutionManager,
    call_args: &[ValueRef; MAX_PARAMETERS],
    param_count: u8,
) -> CompactResult<([u8; MAX_PARAMETERS + 1], u8, u8)> {
    let mut remap = [u8::MAX; MAX_PARAMETERS + 1];
    let mut ext_acc_slot = 1usize;
    let mut scalar_arg_count: u8 = 0;

    for value in call_args.iter().take(param_count as usize) {
        if let ValueRef::AccountRef(acc_idx, _) = value {
            if ext_acc_slot >= remap.len() {
                return Err(VMErrorCode::InvalidOperation);
            }
            // Always resolve against caller context so nested external calls map to
            // absolute transaction account indices.
            let resolved_idx = ctx.resolve_account_index_for_context(*acc_idx);
            if resolved_idx as usize >= ctx.accounts().len() {
                return Err(VMErrorCode::InvalidAccountIndex);
            }
            remap[ext_acc_slot] = resolved_idx;
            ext_acc_slot += 1;
        } else {
            scalar_arg_count = scalar_arg_count.saturating_add(1);
        }
    }

    Ok((remap, (ext_acc_slot - 1) as u8, scalar_arg_count))
}

fn handle_call_external(ctx: &mut ExecutionManager) -> CompactResult<()> {
    validate_stack_limit(ctx, "CALL_EXTERNAL")?;

    let account_index = ctx.fetch_byte()? as usize;
    let raw_selector = ctx.fetch_u16()?;
    let param_count = ctx.fetch_byte()?;
    let function_selector = decode_external_selector(ctx, raw_selector)?;

    #[cfg(feature = "debug-logs")]
    debug_log!(
        "MitoVM: CALL_EXTERNAL acc={} selector={} params={} stack={}",
        account_index as u64,
        function_selector as u64,
        param_count as u64,
        ctx.size() as u64
    );

    if param_count as usize > MAX_PARAMETERS {
        return Err(VMErrorCode::InvalidOperation);
    }

    if ctx.size() < param_count as usize {
        return Err(VMErrorCode::StackError);
    }

    let resolved_account_index =
        ctx.resolve_account_index_for_context(account_index as u8) as usize;
    let resolved_account_index_u8 =
        u8::try_from(resolved_account_index).map_err(|_| VMErrorCode::InvalidAccountIndex)?;

    // Validate account index
    if resolved_account_index >= ctx.accounts().len() {
        debug_log!(
            "MitoVM: CALL_EXTERNAL invalid account index {} >= account count {}",
            resolved_account_index as u32,
            ctx.accounts().len() as u32
        );
        return Err(VMErrorCode::AccountNotFound);
    }

    let (external_bytecode, code_fingerprint, is_authorized) =
        if let Some((hot_fingerprint, hot_authorized)) =
            ctx.external_hot_ctx_lookup(resolved_account_index_u8)
        {
            let script = ctx
                .external_hot_ctx_script(resolved_account_index_u8)
                .ok_or(VMErrorCode::AccountDataEmpty)?;
            (script, hot_fingerprint, hot_authorized)
        } else {
            // Optimization: resolve and validate account only on first use per transaction.
            let account = ctx.get_account(resolved_account_index_u8)?;

            // For cache safety and predictable semantics, external bytecode account must be read-only.
            if account.is_writable() {
                return Err(VMErrorCode::InvalidOperation);
            }

            // Validate account has data
            let account_data_len = account.data_len();
            if account_data_len == 0 {
                debug_log!(
                    "MitoVM: CALL_EXTERNAL account {} has no data",
                    account_index as u32
                );
                return Err(VMErrorCode::AccountDataEmpty);
            }

            // SAFETY: Account has been validated (index check above). borrow_data_unchecked is safe
            // within Solana runtime context as account data is guaranteed to remain valid for the
            // duration of the transaction. Creating slice from valid data pointer.
            // On Solana, all Five bytecode accounts start with a 64-byte ScriptAccountHeader
            let external_bytecode_raw = unsafe {
                let data_slice = account.borrow_data_unchecked();
                core::slice::from_raw_parts(data_slice.as_ptr(), data_slice.len())
            };

            // Decode ScriptAccountHeader and skip metadata region before bytecode.
            const SCRIPT_ACCOUNT_HEADER_LEN: usize = 64;
            const BYTECODE_LEN_OFFSET: usize = 48;
            const METADATA_LEN_OFFSET: usize = 52;
            if external_bytecode_raw.len() < SCRIPT_ACCOUNT_HEADER_LEN {
                debug_log!("MitoVM: CALL_EXTERNAL account too small for header");
                return Err(VMErrorCode::AccountDataEmpty);
            }
            let bytecode_len = u32::from_le_bytes(
                external_bytecode_raw[BYTECODE_LEN_OFFSET..BYTECODE_LEN_OFFSET + 4]
                    .try_into()
                    .map_err(|_| VMErrorCode::InvalidInstruction)?,
            ) as usize;
            let metadata_len = u32::from_le_bytes(
                external_bytecode_raw[METADATA_LEN_OFFSET..METADATA_LEN_OFFSET + 4]
                    .try_into()
                    .map_err(|_| VMErrorCode::InvalidInstruction)?,
            ) as usize;
            let bytecode_start = SCRIPT_ACCOUNT_HEADER_LEN
                .checked_add(metadata_len)
                .ok_or(VMErrorCode::InvalidInstruction)?;
            let bytecode_end = bytecode_start
                .checked_add(bytecode_len)
                .ok_or(VMErrorCode::InvalidInstruction)?;
            if bytecode_end > external_bytecode_raw.len() {
                debug_log!("MitoVM: CALL_EXTERNAL external script header length bounds invalid");
                return Err(VMErrorCode::InvalidInstruction);
            }
            let external_bytecode = &external_bytecode_raw[bytecode_start..bytecode_end];
            let code_fingerprint = external_code_fingerprint(external_bytecode);

            // NEW: Import verification for Five bytecode accounts.
            // Check if the account matches verified imports using zero-copy metadata.
            let pda_derivation_fn: Option<crate::metadata::PdaDerivationFn> =
                Some(|seeds, program_id| {
                    #[cfg(target_os = "solana")]
                    {
                        let (key, _bump) = pinocchio::pubkey::find_program_address(seeds, unsafe {
                            &*(program_id as *const _ as *const pinocchio::pubkey::Pubkey)
                        });
                        key
                    }
                    #[cfg(not(target_os = "solana"))]
                    {
                        let _ = seeds;
                        let _ = program_id;
                        [0u8; 32]
                    }
                });

            let is_authorized = if let Some(cached) =
                ctx.import_verify_cache_lookup(resolved_account_index_u8, code_fingerprint)
            {
                cached
            } else {
                let authorized = ctx.import_metadata.verify_account(
                    account.key(),
                    &ctx.program_id,
                    pda_derivation_fn,
                );
                ctx.import_verify_cache_store(
                    resolved_account_index_u8,
                    code_fingerprint,
                    authorized,
                );
                authorized
            };

            ctx.external_hot_ctx_store(
                resolved_account_index_u8,
                external_bytecode.as_ptr() as usize,
                external_bytecode.len(),
                code_fingerprint,
                is_authorized,
            )?;

            (external_bytecode, code_fingerprint, is_authorized)
        };

    debug_log!(
        "MitoVM: CALL_EXTERNAL loaded external_bytecode length: {}",
        external_bytecode.len() as u32
    );
    debug_log!(
        "MitoVM: CALL_EXTERNAL selector: {}",
        function_selector as u32
    );
    // Log first 20 bytes of external bytecode for debugging
    #[cfg(feature = "debug-logs")]
    {
        let preview_len = external_bytecode.len().min(20);
        debug_log!(
            "MitoVM: CALL_EXTERNAL external_bytecode preview (first {} bytes):",
            preview_len as u32
        );
        for i in 0..preview_len {
            debug_log!("  [{}]: {}", i as u32, external_bytecode[i]);
        }
    }

    // Fast path: transaction-local selector/constraint resolution cache
    let (resolved_func_offset, required_account_count, constraints) = if let Some(entry) = ctx
        .external_call_cache_lookup(
            resolved_account_index as u8,
            function_selector,
            code_fingerprint,
        ) {
        (
            entry.func_offset as usize,
            entry.required_account_count,
            entry.constraints,
        )
    } else {
        let (resolved_func_offset, resolved_func_index) =
            resolve_external_function_target(external_bytecode, function_selector)?;
        let constraint_selector = resolved_func_index.unwrap_or(0);
        let (required_account_count, constraints) =
            parse_function_constraints(external_bytecode, constraint_selector)?;
        let func_offset_u16 = u16::try_from(resolved_func_offset)
            .map_err(|_| VMErrorCode::InvalidInstructionPointer)?;
        let func_index_u8 = match resolved_func_index {
            Some(idx) => u8::try_from(idx).map_err(|_| VMErrorCode::InvalidInstructionPointer)?,
            None => u8::MAX,
        };
        ctx.external_call_cache_store(ExternalCallCacheEntry {
            resolved_account_index: resolved_account_index as u8,
            selector: function_selector,
            func_offset: func_offset_u16,
            func_index: func_index_u8,
            required_account_count,
            constraints,
            code_fingerprint,
            valid: true,
        });
        (resolved_func_offset, required_account_count, constraints)
    };

    if !is_authorized {
        return Err(VMErrorCode::UnauthorizedBytecodeInvocation);
    }

    let caller_start = ctx.param_start();
    let caller_len = ctx.param_len();
    let caller_temp_offset = ctx.temp_offset() as u16;
    let mut saved_caller_parameters = [ValueRef::Empty; MAX_PARAMETERS + 1];
    saved_caller_parameters.copy_from_slice(ctx.parameters());

    let return_address = ctx.ip();

    // Materialize CALL_EXTERNAL arguments in call order.
    let call_args = materialize_call_args(ctx, param_count)?;

    // Build external account remap: external account slots are bound from account args in call order.
    let (computed_remap, bound_account_count, scalar_arg_count) =
        build_external_account_remap(ctx, &call_args, param_count)?;

    // Validate that provided account args satisfy external function constraints.
    if required_account_count > 0 {
        validate_external_function_constraints(
            ctx,
            required_account_count,
            &constraints,
            &computed_remap,
            bound_account_count,
        )?;
    }

    let script_ptr = ctx.script().as_ptr() as usize;
    let script_len = ctx.script().len() as u32;
    let current_context = ctx.current_context;
    let remap_for_callee = computed_remap;
    prepare_callee_frame(
        ctx,
        param_count,
        saved_caller_parameters,
        caller_start,
        caller_len,
        caller_temp_offset,
        return_address,
        current_context,
        remap_for_callee,
        script_ptr,
        script_len,
    )?;
    // External functions address account arguments by account index; parameter slots are used
    // for non-account values (e.g. scalar inputs used by fused ops).
    ctx.allocate_params(scalar_arg_count.saturating_add(1))?;
    write_scalar_params(ctx, &call_args, param_count);

    ctx.set_external_account_remap(remap_for_callee);

    debug_log!(
        "MitoVM: CALL_EXTERNAL about to switch_to_external_bytecode - current IP: {}, new offset: {}",
        ctx.ip() as u32,
        resolved_func_offset as u32
    );

    ctx.switch_to_external_bytecode(external_bytecode, resolved_func_offset)?;
    ctx.current_context = resolved_account_index as u8;

    debug_log!(
        "MitoVM: CALL_EXTERNAL after switch_to_external_bytecode - new IP: {}, script len: {}",
        ctx.ip() as u32,
        ctx.script().len() as u32
    );

    Ok(())
}

#[inline]
fn materialize_call_args(
    ctx: &mut ExecutionManager,
    param_count: u8,
) -> CompactResult<[ValueRef; MAX_PARAMETERS]> {
    let mut call_args = [ValueRef::Empty; MAX_PARAMETERS];
    match param_count {
        0 => {}
        1 => {
            call_args[0] = ctx.pop()?;
        }
        2 => {
            let p2 = ctx.pop()?;
            let p1 = ctx.pop()?;
            call_args[0] = p1;
            call_args[1] = p2;
        }
        3 => {
            let p3 = ctx.pop()?;
            let p2 = ctx.pop()?;
            let p1 = ctx.pop()?;
            call_args[0] = p1;
            call_args[1] = p2;
            call_args[2] = p3;
        }
        4 => {
            let p4 = ctx.pop()?;
            let p3 = ctx.pop()?;
            let p2 = ctx.pop()?;
            let p1 = ctx.pop()?;
            call_args[0] = p1;
            call_args[1] = p2;
            call_args[2] = p3;
            call_args[3] = p4;
        }
        _ => {
            for i in 0..param_count {
                let value = ctx.pop()?;
                let idx = (param_count - i - 1) as usize;
                call_args[idx] = value;
            }
        }
    }
    Ok(call_args)
}

#[inline]
fn prepare_callee_frame(
    ctx: &mut ExecutionManager,
    param_count: u8,
    saved_parameters: [ValueRef; MAX_PARAMETERS + 1],
    caller_start: u8,
    caller_len: u8,
    caller_temp_offset: u16,
    return_address: usize,
    context_id: u8,
    remap: [u8; MAX_PARAMETERS + 1],
    script_ptr: usize,
    script_len: u32,
) -> CompactResult<()> {
    let current_local_count = ctx.local_count();
    let current_local_base = ctx.local_base();

    let new_local_base = current_local_base
        .checked_add(current_local_count)
        .ok_or(VMErrorCode::CallStackOverflow)?;
    let locals_to_allocate = (param_count as usize).max(3);
    if (new_local_base as usize + locals_to_allocate) > crate::MAX_LOCALS {
        return Err(VMErrorCode::CallStackOverflow);
    }

    ctx.push_call_frame(CallFrame::with_parameters(
        return_address as u16,
        ctx.stack.sp,
        current_local_count,
        current_local_base,
        caller_start,
        caller_len,
        caller_temp_offset,
        saved_parameters,
        context_id,
        remap,
        script_ptr,
        script_len,
    ))?;

    ctx.set_local_base(new_local_base);
    ctx.set_local_count(0);
    ctx.allocate_locals(param_count.max(3))?;
    Ok(())
}

#[inline(always)]
fn write_scalar_params(
    ctx: &mut ExecutionManager,
    call_args: &[ValueRef; MAX_PARAMETERS],
    param_count: u8,
) {
    let params = ctx.parameters_mut();
    params[0] = ValueRef::U64(0);
    let mut out_idx = 1usize;
    for value in call_args.iter().take(param_count as usize) {
        if !matches!(value, ValueRef::AccountRef(_, _)) {
            params[out_idx] = *value;
            out_idx += 1;
        }
    }
    for slot in out_idx..params.len() {
        params[slot] = ValueRef::Empty;
    }
}

#[inline(always)]
fn decode_external_selector(ctx: &ExecutionManager, raw: u16) -> CompactResult<u16> {
    // Tagged selector (bit15) means "constant pool slot in current bytecode context".
    if (raw & 0x8000) != 0 {
        if !ctx.pool_enabled() {
            return Err(VMErrorCode::InvalidInstruction);
        }
        let pool_idx = raw & 0x7FFF;
        let value = ctx.read_pool_slot_u64(pool_idx)?;
        if value > u16::MAX as u64 {
            return Err(VMErrorCode::InvalidInstruction);
        }
        return Ok(value as u16);
    }
    Ok(raw)
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
        SYSCALL_BLAKE3 => handle_syscall_blake3(ctx),
        SYSCALL_POSEIDON => handle_syscall_poseidon(ctx),
        SYSCALL_SECP256K1_RECOVER => handle_syscall_secp256k1_recover(ctx),
        SYSCALL_ALT_BN128_COMPRESSION => handle_syscall_alt_bn128_compression(ctx),
        SYSCALL_ALT_BN128_GROUP_OP => handle_syscall_alt_bn128_group_op(ctx),
        SYSCALL_BIG_MOD_EXP => handle_syscall_big_mod_exp(ctx),
        SYSCALL_CURVE_GROUP_OP => handle_syscall_curve_group_op(ctx),
        SYSCALL_CURVE_MULTISCALAR_MUL => handle_syscall_curve_multiscalar_mul(ctx),
        SYSCALL_CURVE_PAIRING_MAP => handle_syscall_curve_pairing_map(ctx),
        SYSCALL_CURVE_VALIDATE_POINT => handle_syscall_curve_validate_point(ctx),
        SYSCALL_VERIFY_ED25519_INSTRUCTION => handle_syscall_verify_ed25519_instruction(ctx),

        _ => {
            debug_log!("MitoVM: CALL_NATIVE invalid syscall_id={}", syscall_id);
            return Err(VMErrorCode::InvalidInstruction);
        }
    }?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        external_selector, handle_call_external, handle_functions, parse_function_constraints,
        resolve_external_function_target,
    };
    use crate::{
        handlers::{
            accounts::handle_accounts, arrays::handle_arrays, constraints::handle_constraints,
            control_flow::handle_control_flow, locals::handle_locals, memory::handle_memory,
            option_result::handle_option_result_ops, stack_ops::handle_stack_ops,
            system::sysvars::handle_sysvar_ops,
        },
        context::ExecutionContext, error::VMErrorCode, stack::StackStorage, MitoVM,
        MAX_PARAMETERS,
    };
    use five_dsl_compiler::DslCompiler;
    use five_protocol::ValueRef;
    use five_protocol::{BytecodeBuilder, FEATURE_FUNCTION_CONSTRAINTS, FEATURE_FUNCTION_NAMES};
    use pinocchio::{account_info::AccountInfo, pubkey::Pubkey};

    #[test]
    fn parse_constraints_returns_default_when_feature_not_set() {
        let mut b = BytecodeBuilder::new();
        b.emit_header(1, 1).emit_halt();
        let bytecode = b.build();

        let (count, constraints) = parse_function_constraints(&bytecode, 0).expect("parse");
        assert_eq!(count, 0);
        assert_eq!(constraints, [0u8; 16]);
    }

    #[test]
    fn parse_constraints_reads_fixed_width_entries() {
        let mut b = BytecodeBuilder::new();
        b.emit_header(1, 2);
        b.patch_u32(4, FEATURE_FUNCTION_CONSTRAINTS)
            .expect("features");

        // 2 entries x 17 bytes each
        b.emit_u16(34);
        // entry 0: account_count=1, signer on account 0
        b.emit_u8(1);
        b.emit_u8(0x01);
        b.emit_bytes(&[0u8; 15]);
        // entry 1: account_count=2, signer on account 0, writable on account 1
        b.emit_u8(2);
        b.emit_u8(0x01);
        b.emit_u8(0x02);
        b.emit_bytes(&[0u8; 14]);
        b.emit_halt();
        let bytecode = b.build();

        let (count, constraints) = parse_function_constraints(&bytecode, 1).expect("parse");
        assert_eq!(count, 2);
        assert_eq!(constraints[0], 0x01);
        assert_eq!(constraints[1], 0x02);
    }

    #[test]
    fn parse_constraints_skips_function_names_metadata() {
        let mut b = BytecodeBuilder::new();
        b.emit_header(1, 1);
        b.patch_u32(4, FEATURE_FUNCTION_NAMES | FEATURE_FUNCTION_CONSTRAINTS)
            .expect("features");

        // Function names section payload:
        // [name_count=1] [name_len=4] ['t' 'e' 's' 't']
        b.emit_u16(6);
        b.emit_u8(1);
        b.emit_u8(4);
        b.emit_bytes(b"test");

        // Constraints section payload: one fixed-width entry
        b.emit_u16(17);
        b.emit_u8(1);
        b.emit_u8(0x01);
        b.emit_bytes(&[0u8; 15]);
        b.emit_halt();
        let bytecode = b.build();

        let (count, constraints) = parse_function_constraints(&bytecode, 0).expect("parse");
        assert_eq!(count, 1);
        assert_eq!(constraints[0], 0x01);
    }

    fn build_external_script_with_names() -> Vec<u8> {
        let mut script = Vec::new();

        let features = five_protocol::FEATURE_FUNCTION_NAMES
            | five_protocol::FEATURE_PUBLIC_ENTRY_TABLE
            | five_protocol::FEATURE_CONSTANT_POOL;

        // Header
        script.extend_from_slice(b"5IVE");
        script.extend_from_slice(&features.to_le_bytes());
        script.push(1); // public
        script.push(1); // total

        // Function names section payload: [count=1][len=4]["ping"] => 6 bytes
        script.extend_from_slice(&(6u16).to_le_bytes());
        script.push(1);
        script.push(4);
        script.extend_from_slice(b"ping");

        // Public entry table payload: [count=1][rel_offset=0]
        script.extend_from_slice(&(3u16).to_le_bytes());
        script.push(1);
        script.extend_from_slice(&(0u16).to_le_bytes());

        // Constant pool descriptor (16 bytes) at offset 23
        // pool_offset=40 (aligned), pool_slots=0 -> code_start=40
        script.extend_from_slice(&(40u32).to_le_bytes()); // pool_offset
        script.extend_from_slice(&(41u32).to_le_bytes()); // string_blob_offset
        script.extend_from_slice(&(0u32).to_le_bytes()); // string_blob_len
        script.extend_from_slice(&(0u16).to_le_bytes()); // pool_slots
        script.extend_from_slice(&(0u16).to_le_bytes()); // reserved

        while script.len() < 40 {
            script.push(0);
        }
        script.push(five_protocol::opcodes::HALT);
        script
    }

    #[test]
    fn resolve_external_function_target_by_name_hash() {
        let script = build_external_script_with_names();
        let selector = external_selector("ping");
        let (offset, function_index) =
            resolve_external_function_target(&script, selector).expect("resolve");

        assert_eq!(function_index, Some(0));
        assert_eq!(offset, 40);
    }

    #[test]
    fn resolve_external_target_rejects_legacy_absolute_offset_into_non_public_code() {
        let mut script = build_external_script_with_names();
        // Add trailing non-public code byte to target via legacy absolute selector fallback.
        script.push(five_protocol::opcodes::HALT);

        let selector = 41u16;
        let err = resolve_external_function_target(&script, selector).unwrap_err();
        assert_eq!(err, VMErrorCode::InvalidInstructionPointer);
    }

    #[test]
    fn resolve_external_function_target_for_token_template_transfer() {
        let source = include_str!("../../../five-templates/token/src/token.v");
        let bytecode = DslCompiler::compile_dsl(source).expect("token template should compile");
        let selector = external_selector("transfer");
        let (offset, function_index) =
            resolve_external_function_target(&bytecode, selector).expect("transfer should resolve");

        assert_eq!(function_index, Some(3));
        assert!(offset > 0);
    }

    fn create_account_info<'a>(
        key: &'a Pubkey,
        is_signer: bool,
        is_writable: bool,
        lamports: &'a mut u64,
        data: &'a mut [u8],
        owner: &'a Pubkey,
    ) -> AccountInfo {
        AccountInfo::new(key, is_signer, is_writable, lamports, data, owner, false, 0)
    }

    fn minimal_external_bytecode() -> Vec<u8> {
        let mut b = Vec::new();
        let features = five_protocol::FEATURE_PUBLIC_ENTRY_TABLE;
        b.extend_from_slice(b"5IVE");
        b.extend_from_slice(&features.to_le_bytes());
        b.push(1); // public function count
        b.push(1); // total function count
                   // Public entry table payload: [count=1][rel_offset=0]
        b.extend_from_slice(&(3u16).to_le_bytes());
        b.push(1);
        b.extend_from_slice(&(0u16).to_le_bytes());
        b.push(five_protocol::opcodes::HALT);
        b
    }

    fn wrap_script_account_data(bytecode: &[u8]) -> Vec<u8> {
        let mut data = vec![0u8; 64 + bytecode.len()];
        data[48..52].copy_from_slice(&(bytecode.len() as u32).to_le_bytes());
        data[52..56].copy_from_slice(&0u32.to_le_bytes()); // metadata len
        data[64..].copy_from_slice(bytecode);
        data
    }

    #[test]
    fn call_external_preserves_computed_account_remap() {
        let program_id = Pubkey::from([51u8; 32]);
        let caller_key = Pubkey::from([52u8; 32]);
        let external_key = Pubkey::from([53u8; 32]);

        let mut caller_lamports = 1;
        let mut external_lamports = 1;
        let mut caller_data = [];
        let external_bytecode = minimal_external_bytecode();
        let mut external_data = wrap_script_account_data(&external_bytecode);

        let caller_account = create_account_info(
            &caller_key,
            false,
            false,
            &mut caller_lamports,
            &mut caller_data,
            &program_id,
        );
        let external_account = create_account_info(
            &external_key,
            false,
            false, // external bytecode account must be read-only in CALL_EXTERNAL
            &mut external_lamports,
            external_data.as_mut_slice(),
            &program_id,
        );
        let accounts = [caller_account, external_account];

        // CALL_EXTERNAL payload: account_index=1, selector=0, param_count=1.
        let call_site = [1u8, 0u8, 0u8, 1u8];
        let mut storage = StackStorage::new();
        let mut ctx = ExecutionContext::new(
            &call_site,
            &accounts,
            program_id,
            &[],
            0,
            &mut storage,
            0,
            0,
            0,
            0,
            0,
            0,
        );

        let mut stale_remap = [u8::MAX; MAX_PARAMETERS + 1];
        stale_remap[1] = 9;
        ctx.set_external_account_remap(stale_remap);
        ctx.push(ValueRef::AccountRef(0, 0)).expect("push arg");

        handle_call_external(&mut ctx).expect("CALL_EXTERNAL should succeed");

        // Regression: computed remap replaces stale remap for callee context.
        assert_eq!(ctx.external_account_remap()[1], 0);
    }

    #[test]
    fn execute_direct_compiled_external_call_path() {
        let program_id = Pubkey::from([61u8; 32]);
        let external_key = Pubkey::default();
        let source_key = Pubkey::from([63u8; 32]);
        let destination_key = Pubkey::from([64u8; 32]);
        let owner_key = Pubkey::from([65u8; 32]);
        let vm_state_key = Pubkey::from([66u8; 32]);

        let callee_source = r#"
            pub transfer(
                source_account: account @mut,
                destination_account: account @mut,
                owner: account @mut,
                amount: u64
            ) {
            }
        "#;
        let callee_bytecode = DslCompiler::compile_dsl(callee_source).expect("callee compile");

        let caller_source = format!(
            r#"
            use "{}"::{{transfer}};

            pub fn call_transfer(
                source_account: account @mut,
                destination_account: account @mut,
                owner: account @mut,
                ext0: account
            ) {{
                transfer(source_account, destination_account, owner, 1);
            }}
        "#,
            "11111111111111111111111111111111"
        );
        let caller_bytecode = DslCompiler::compile_dsl(&caller_source).expect("caller compile");

        let mut vm_state_lamports = 1;
        let mut source_lamports = 1;
        let mut destination_lamports = 1;
        let mut owner_lamports = 1;
        let mut external_lamports = 1;

        let mut vm_state_data = [0u8; 8];
        let mut source_data = [0u8; 8];
        let mut destination_data = [0u8; 8];
        let mut owner_data = [0u8; 8];
        let mut external_data = wrap_script_account_data(&callee_bytecode);

        let vm_state_account = create_account_info(
            &vm_state_key,
            false,
            false,
            &mut vm_state_lamports,
            &mut vm_state_data,
            &program_id,
        );
        let source_account = create_account_info(
            &source_key,
            false,
            true,
            &mut source_lamports,
            &mut source_data,
            &program_id,
        );
        let destination_account = create_account_info(
            &destination_key,
            false,
            true,
            &mut destination_lamports,
            &mut destination_data,
            &program_id,
        );
        let owner_account = create_account_info(
            &owner_key,
            true,
            true,
            &mut owner_lamports,
            &mut owner_data,
            &program_id,
        );
        let external_account = create_account_info(
            &external_key,
            false,
            false,
            &mut external_lamports,
            external_data.as_mut_slice(),
            &program_id,
        );

        let accounts = [
            vm_state_account,
            source_account,
            destination_account,
            owner_account,
            external_account,
        ];
        let mut input = Vec::new();
        input.extend_from_slice(&0u32.to_le_bytes());
        input.extend_from_slice(&4u32.to_le_bytes());
        for account_idx in [1u32, 2u32, 3u32, 4u32] {
            input.push(five_protocol::types::ACCOUNT);
            input.extend_from_slice(&account_idx.to_le_bytes());
        }

        let mut storage = StackStorage::new();
        let result = MitoVM::execute_direct(
            &caller_bytecode,
            &input,
            &accounts,
            &program_id,
            &mut storage,
        );
        assert!(result.is_ok(), "direct external call failed: {:?}", result);
    }

    #[test]
    fn internal_call_then_pubkey_and_concat_preserves_program_id_tempref() {
        let program_id = Pubkey::from([77u8; 32]);
        let expected_pubkey = [9u8; 32];
        let callee_pubkey = [3u8; 32];

        let mut script = vec![
            b'5', b'I', b'V', b'E', // magic
            0, 0, 0, 0, // features
            1, // public count
            2, // total count
        ];

        let main_len = 4 + 33 + 1;
        let callee_addr = (five_protocol::FIVE_HEADER_OPTIMIZED_SIZE + main_len) as u16;

        // main:
        script.push(five_protocol::opcodes::CALL);
        script.push(0);
        script.extend_from_slice(&callee_addr.to_le_bytes());
        script.push(five_protocol::opcodes::PUSH_PUBKEY);
        script.extend_from_slice(&expected_pubkey);
        script.push(five_protocol::opcodes::HALT);

        // callee:
        script.push(five_protocol::opcodes::PUSH_PUBKEY);
        script.extend_from_slice(&callee_pubkey);
        script.push(five_protocol::opcodes::RETURN);

        let mut lamports = 1;
        let mut account_data = [];
        let account = create_account_info(
            &program_id,
            false,
            false,
            &mut lamports,
            &mut account_data,
            &program_id,
        );
        let accounts = [account];

        let mut storage = StackStorage::new();
        let mut ctx = ExecutionContext::new(
            &script,
            &accounts,
            program_id,
            &[],
            five_protocol::FIVE_HEADER_OPTIMIZED_SIZE as u16,
            &mut storage,
            1,
            2,
            0,
            0,
            0,
            0,
        );

        let opcode = ctx.fetch_byte().expect("fetch CALL");
        handle_functions(opcode, &mut ctx).expect("CALL should succeed");

        let opcode = ctx.fetch_byte().expect("fetch callee PUSH_PUBKEY");
        handle_stack_ops(opcode, &mut ctx).expect("callee PUSH_PUBKEY should succeed");

        let opcode = ctx.fetch_byte().expect("fetch RETURN");
        handle_control_flow(opcode, &mut ctx).expect("RETURN should succeed");

        let opcode = ctx.fetch_byte().expect("fetch caller PUSH_PUBKEY");
        handle_stack_ops(opcode, &mut ctx).expect("caller PUSH_PUBKEY should succeed");

        let array_offset = ctx.alloc_temp(3).expect("allocate bytes array");
        ctx.temp_buffer_mut()[array_offset as usize] = 1;
        ctx.temp_buffer_mut()[array_offset as usize + 1] = 0;
        ctx.temp_buffer_mut()[array_offset as usize + 2] = 3;
        ctx.push(ValueRef::ArrayRef(array_offset))
            .expect("push fixed byte array");
        ctx.push(ValueRef::U64(42)).expect("push scalar amount");
        handle_arrays(five_protocol::opcodes::ARRAY_CONCAT, &mut ctx)
            .expect("ARRAY_CONCAT should succeed");

        let _data_ref = ctx.pop().expect("pop instruction data");
        let program_id_ref = ctx.pop().expect("pop program id ref");
        let extracted = ctx
            .extract_pubkey(&program_id_ref)
            .expect("program id tempref should still resolve");

        assert_eq!(extracted, expected_pubkey);
    }

    #[test]
    fn internal_call_with_get_clock_preserves_caller_parameters() {
        let main_len = 2 + 4 + 2 + 1;
        let func_addr = (five_protocol::FIVE_HEADER_OPTIMIZED_SIZE + main_len) as u16;
        let main_code = [
            five_protocol::opcodes::LOAD_PARAM,
            1,
            five_protocol::opcodes::CALL,
            0,
            (func_addr & 0xff) as u8,
            (func_addr >> 8) as u8,
            five_protocol::opcodes::LOAD_PARAM,
            1,
            five_protocol::opcodes::HALT,
        ];

        let function_code = [five_protocol::opcodes::GET_CLOCK, five_protocol::opcodes::RETURN];

        let mut bytecode = vec![b'5', b'I', b'V', b'E'];
        bytecode.extend_from_slice(&0u32.to_le_bytes());
        bytecode.push(0);
        bytecode.push(1);
        bytecode.extend_from_slice(&main_code);
        bytecode.extend_from_slice(&function_code);

        let mut storage = StackStorage::new();
        let mut ctx = ExecutionContext::new(
            &bytecode,
            &[],
            Pubkey::default(),
            &[],
            five_protocol::FIVE_HEADER_OPTIMIZED_SIZE as u16,
            &mut storage,
            0,
            1,
            0,
            0,
            0,
            0,
        );
        ctx.allocate_params(2).expect("allocate params");
        ctx.set_parameter(0, ValueRef::U64(0)).expect("set func index");
        ctx.set_parameter(1, ValueRef::U64(100)).expect("set caller param");

        let opcode = ctx.fetch_byte().expect("fetch initial LOAD_PARAM");
        handle_locals(opcode, &mut ctx).expect("first LOAD_PARAM should succeed");

        let opcode = ctx.fetch_byte().expect("fetch CALL");
        handle_functions(opcode, &mut ctx).expect("CALL should succeed");

        let opcode = ctx.fetch_byte().expect("fetch GET_CLOCK");
        handle_sysvar_ops(opcode, &mut ctx).expect("GET_CLOCK should succeed");

        let opcode = ctx.fetch_byte().expect("fetch RETURN");
        handle_control_flow(opcode, &mut ctx).expect("RETURN should succeed");

        let opcode = ctx.fetch_byte().expect("fetch second LOAD_PARAM");
        handle_locals(opcode, &mut ctx).expect("second LOAD_PARAM should succeed");

        let second = ctx.pop().expect("pop second param");
        let first = ctx.pop().expect("pop first param");
        assert_eq!(first.as_u64(), Some(100));
        assert_eq!(second.as_u64(), Some(100));
    }

    #[test]
    fn internal_call_with_account_arg_and_get_clock_preserves_caller_parameters() {
        let main_len = 4 + 2 + 1;
        let func_addr = (five_protocol::FIVE_HEADER_OPTIMIZED_SIZE + main_len) as u16;
        let main_code = [
            five_protocol::opcodes::CALL,
            1,
            (func_addr & 0xff) as u8,
            (func_addr >> 8) as u8,
            five_protocol::opcodes::LOAD_PARAM,
            1,
            five_protocol::opcodes::HALT,
        ];

        let function_code = [five_protocol::opcodes::GET_CLOCK, five_protocol::opcodes::RETURN];

        let mut bytecode = vec![b'5', b'I', b'V', b'E'];
        bytecode.extend_from_slice(&0u32.to_le_bytes());
        bytecode.push(0);
        bytecode.push(1);
        bytecode.extend_from_slice(&main_code);
        bytecode.extend_from_slice(&function_code);

        let mut storage = StackStorage::new();
        let mut ctx = ExecutionContext::new(
            &bytecode,
            &[],
            Pubkey::default(),
            &[],
            five_protocol::FIVE_HEADER_OPTIMIZED_SIZE as u16,
            &mut storage,
            0,
            1,
            0,
            0,
            0,
            0,
        );
        ctx.allocate_params(2).expect("allocate params");
        ctx.set_parameter(0, ValueRef::U64(0)).expect("set func index");
        ctx.set_parameter(1, ValueRef::U64(100)).expect("set caller param");
        ctx.push(ValueRef::AccountRef(0, 0))
            .expect("push synthetic callee account arg");

        let opcode = ctx.fetch_byte().expect("fetch CALL");
        handle_functions(opcode, &mut ctx).expect("CALL should succeed");

        let opcode = ctx.fetch_byte().expect("fetch GET_CLOCK");
        handle_sysvar_ops(opcode, &mut ctx).expect("GET_CLOCK should succeed");

        let opcode = ctx.fetch_byte().expect("fetch RETURN");
        handle_control_flow(opcode, &mut ctx).expect("RETURN should succeed");

        let opcode = ctx.fetch_byte().expect("fetch second LOAD_PARAM");
        handle_locals(opcode, &mut ctx).expect("second LOAD_PARAM should succeed");

        let second = ctx.pop().expect("pop caller param after return");
        assert_eq!(second.as_u64(), Some(100));
    }

    #[test]
    fn internal_call_with_account_write_and_get_clock_preserves_caller_parameters() {
        let program_id = Pubkey::from([23u8; 32]);
        let main_len = 4 + 2 + 1;
        let func_addr = (five_protocol::FIVE_HEADER_OPTIMIZED_SIZE + main_len) as u16;
        let main_code = [
            five_protocol::opcodes::CALL,
            1,
            (func_addr & 0xff) as u8,
            (func_addr >> 8) as u8,
            five_protocol::opcodes::LOAD_PARAM,
            1,
            five_protocol::opcodes::HALT,
        ];

        let mut function_code = vec![
            five_protocol::opcodes::GET_CLOCK,
            five_protocol::opcodes::LOAD_PARAM,
            1,
            five_protocol::opcodes::PUSH_U64,
        ];
        function_code.extend_from_slice(&0u64.to_le_bytes());
        function_code.push(five_protocol::opcodes::PUSH_U64);
        function_code.extend_from_slice(&1u64.to_le_bytes());
        function_code.push(five_protocol::opcodes::SAVE_ACCOUNT);
        function_code.push(five_protocol::opcodes::RETURN);

        let mut bytecode = vec![b'5', b'I', b'V', b'E'];
        bytecode.extend_from_slice(&0u32.to_le_bytes());
        bytecode.push(0);
        bytecode.push(1);
        bytecode.extend_from_slice(&main_code);
        bytecode.extend_from_slice(&function_code);

        let mut vm_state_lamports = 1;
        let mut vm_state_data = [0u8; 8];
        let vm_state = create_account_info(
            &Pubkey::from([24u8; 32]),
            false,
            false,
            &mut vm_state_lamports,
            &mut vm_state_data,
            &program_id,
        );

        let mut lamports = 1;
        let mut account_data = [0u8; 8];
        let writable_account = create_account_info(
            &Pubkey::from([22u8; 32]),
            false,
            true,
            &mut lamports,
            &mut account_data,
            &program_id,
        );
        let accounts = [vm_state, writable_account];

        let mut storage = StackStorage::new();
        let mut ctx = ExecutionContext::new(
            &bytecode,
            &accounts,
            program_id,
            &[],
            five_protocol::FIVE_HEADER_OPTIMIZED_SIZE as u16,
            &mut storage,
            0,
            1,
            0,
            0,
            0,
            0,
        );
        ctx.allocate_params(2).expect("allocate params");
        ctx.set_parameter(0, ValueRef::U64(0)).expect("set func index");
        ctx.set_parameter(1, ValueRef::U64(100)).expect("set caller param");
        ctx.push(ValueRef::AccountRef(1, 0))
            .expect("push synthetic callee account arg");

        let opcode = ctx.fetch_byte().expect("fetch CALL");
        handle_functions(opcode, &mut ctx).expect("CALL should succeed");

        let opcode = ctx.fetch_byte().expect("fetch GET_CLOCK");
        handle_sysvar_ops(opcode, &mut ctx).expect("GET_CLOCK should succeed");

        let opcode = ctx.fetch_byte().expect("fetch LOAD_PARAM");
        handle_locals(opcode, &mut ctx).expect("callee LOAD_PARAM should succeed");

        let opcode = ctx.fetch_byte().expect("fetch PUSH_U64 offset");
        handle_stack_ops(opcode, &mut ctx).expect("callee PUSH_U64 offset should succeed");

        let opcode = ctx.fetch_byte().expect("fetch PUSH_U64 value");
        handle_stack_ops(opcode, &mut ctx).expect("callee PUSH_U64 value should succeed");

        let opcode = ctx.fetch_byte().expect("fetch SAVE_ACCOUNT");
        handle_accounts(opcode, &mut ctx).expect("SAVE_ACCOUNT should succeed");

        let opcode = ctx.fetch_byte().expect("fetch RETURN");
        handle_control_flow(opcode, &mut ctx).expect("RETURN should succeed");

        let opcode = ctx.fetch_byte().expect("fetch caller LOAD_PARAM");
        handle_locals(opcode, &mut ctx).expect("caller LOAD_PARAM should succeed");

        let caller_param = ctx.pop().expect("pop caller param after return");
        assert_eq!(caller_param.as_u64(), Some(100));
    }

    #[test]
    fn internal_call_with_existing_caller_locals_preserves_post_call_local_ops() {
        let main_len = 4 + 2 + 2 + 2 + 1;
        let func_addr = (five_protocol::FIVE_HEADER_OPTIMIZED_SIZE + main_len) as u16;
        let main_code = [
            five_protocol::opcodes::CALL,
            0,
            (func_addr & 0xff) as u8,
            (func_addr >> 8) as u8,
            five_protocol::opcodes::LOAD_PARAM,
            1,
            five_protocol::opcodes::SET_LOCAL,
            1,
            five_protocol::opcodes::GET_LOCAL,
            1,
            five_protocol::opcodes::HALT,
        ];

        let function_code = [five_protocol::opcodes::GET_CLOCK, five_protocol::opcodes::RETURN];

        let mut bytecode = vec![b'5', b'I', b'V', b'E'];
        bytecode.extend_from_slice(&0u32.to_le_bytes());
        bytecode.push(0);
        bytecode.push(1);
        bytecode.extend_from_slice(&main_code);
        bytecode.extend_from_slice(&function_code);

        let mut storage = StackStorage::new();
        let mut ctx = ExecutionContext::new(
            &bytecode,
            &[],
            Pubkey::default(),
            &[],
            five_protocol::FIVE_HEADER_OPTIMIZED_SIZE as u16,
            &mut storage,
            0,
            1,
            0,
            0,
            0,
            0,
        );
        ctx.allocate_params(2).expect("allocate params");
        ctx.set_parameter(0, ValueRef::U64(0)).expect("set func index");
        ctx.set_parameter(1, ValueRef::U64(100)).expect("set caller param");
        ctx.allocate_locals(3).expect("allocate caller locals");

        let opcode = ctx.fetch_byte().expect("fetch CALL");
        handle_functions(opcode, &mut ctx).expect("CALL should succeed");

        let opcode = ctx.fetch_byte().expect("fetch GET_CLOCK");
        handle_sysvar_ops(opcode, &mut ctx).expect("GET_CLOCK should succeed");

        let opcode = ctx.fetch_byte().expect("fetch RETURN");
        handle_control_flow(opcode, &mut ctx).expect("RETURN should succeed");

        let opcode = ctx.fetch_byte().expect("fetch LOAD_PARAM");
        handle_locals(opcode, &mut ctx).expect("caller LOAD_PARAM should succeed");

        let opcode = ctx.fetch_byte().expect("fetch SET_LOCAL");
        handle_locals(opcode, &mut ctx).expect("SET_LOCAL should succeed");

        let opcode = ctx.fetch_byte().expect("fetch GET_LOCAL");
        handle_locals(opcode, &mut ctx).expect("GET_LOCAL should succeed");

        let round_tripped = ctx.pop().expect("pop round-tripped local");
        assert_eq!(round_tripped.as_u64(), Some(100));
    }

    #[test]
    fn callee_locals_after_get_clock_do_not_clobber_caller_parameters() {
        let main_len = 4 + 2 + 1;
        let func_addr = (five_protocol::FIVE_HEADER_OPTIMIZED_SIZE + main_len) as u16;
        let main_code = [
            five_protocol::opcodes::CALL,
            0,
            (func_addr & 0xff) as u8,
            (func_addr >> 8) as u8,
            five_protocol::opcodes::LOAD_PARAM,
            1,
            five_protocol::opcodes::HALT,
        ];

        let function_code = [
            five_protocol::opcodes::GET_CLOCK,
            five_protocol::opcodes::SET_LOCAL,
            0,
            five_protocol::opcodes::GET_LOCAL,
            0,
            five_protocol::opcodes::RETURN,
        ];

        let mut bytecode = vec![b'5', b'I', b'V', b'E'];
        bytecode.extend_from_slice(&0u32.to_le_bytes());
        bytecode.push(0);
        bytecode.push(1);
        bytecode.extend_from_slice(&main_code);
        bytecode.extend_from_slice(&function_code);

        let mut storage = StackStorage::new();
        let mut ctx = ExecutionContext::new(
            &bytecode,
            &[],
            Pubkey::default(),
            &[],
            five_protocol::FIVE_HEADER_OPTIMIZED_SIZE as u16,
            &mut storage,
            0,
            1,
            0,
            0,
            0,
            0,
        );
        ctx.allocate_params(2).expect("allocate params");
        ctx.set_parameter(0, ValueRef::U64(0)).expect("set func index");
        ctx.set_parameter(1, ValueRef::U64(100)).expect("set caller param");
        ctx.allocate_locals(3).expect("allocate caller locals");

        let opcode = ctx.fetch_byte().expect("fetch CALL");
        handle_functions(opcode, &mut ctx).expect("CALL should succeed");

        let opcode = ctx.fetch_byte().expect("fetch GET_CLOCK");
        handle_sysvar_ops(opcode, &mut ctx).expect("GET_CLOCK should succeed");

        let opcode = ctx.fetch_byte().expect("fetch SET_LOCAL");
        handle_locals(opcode, &mut ctx).expect("callee SET_LOCAL should succeed");

        let opcode = ctx.fetch_byte().expect("fetch GET_LOCAL");
        handle_locals(opcode, &mut ctx).expect("callee GET_LOCAL should succeed");

        let opcode = ctx.fetch_byte().expect("fetch RETURN");
        handle_control_flow(opcode, &mut ctx).expect("RETURN should succeed");

        let opcode = ctx.fetch_byte().expect("fetch caller LOAD_PARAM");
        handle_locals(opcode, &mut ctx).expect("caller LOAD_PARAM should succeed");

        let caller_param = ctx.pop().expect("pop caller param");
        assert_eq!(caller_param.as_u64(), Some(100));
    }

    #[test]
    fn nibble_locals_after_internal_call_preserve_caller_parameter() {
        let main_len = 4 + 1 + 1 + 1 + 1;
        let func_addr = (five_protocol::FIVE_HEADER_OPTIMIZED_SIZE + main_len) as u16;
        let main_code = [
            five_protocol::opcodes::CALL,
            0,
            (func_addr & 0xff) as u8,
            (func_addr >> 8) as u8,
            five_protocol::opcodes::LOAD_PARAM_1,
            five_protocol::opcodes::SET_LOCAL_1,
            five_protocol::opcodes::GET_LOCAL_1,
            five_protocol::opcodes::HALT,
        ];

        let function_code = [five_protocol::opcodes::GET_CLOCK, five_protocol::opcodes::RETURN];

        let mut bytecode = vec![b'5', b'I', b'V', b'E'];
        bytecode.extend_from_slice(&0u32.to_le_bytes());
        bytecode.push(0);
        bytecode.push(1);
        bytecode.extend_from_slice(&main_code);
        bytecode.extend_from_slice(&function_code);

        let mut storage = StackStorage::new();
        let mut ctx = ExecutionContext::new(
            &bytecode,
            &[],
            Pubkey::default(),
            &[],
            five_protocol::FIVE_HEADER_OPTIMIZED_SIZE as u16,
            &mut storage,
            0,
            1,
            0,
            0,
            0,
            0,
        );
        ctx.allocate_params(2).expect("allocate params");
        ctx.set_parameter(0, ValueRef::U64(0)).expect("set func index");
        ctx.set_parameter(1, ValueRef::U64(100)).expect("set caller param");
        ctx.allocate_locals(3).expect("allocate caller locals");

        let opcode = ctx.fetch_byte().expect("fetch CALL");
        handle_functions(opcode, &mut ctx).expect("CALL should succeed");

        let opcode = ctx.fetch_byte().expect("fetch GET_CLOCK");
        handle_sysvar_ops(opcode, &mut ctx).expect("GET_CLOCK should succeed");

        let opcode = ctx.fetch_byte().expect("fetch RETURN");
        handle_control_flow(opcode, &mut ctx).expect("RETURN should succeed");

        let opcode = ctx.fetch_byte().expect("fetch LOAD_PARAM_1");
        crate::handlers::locals::handle_nibble_locals(opcode, &mut ctx)
            .expect("LOAD_PARAM_1 should succeed");

        let opcode = ctx.fetch_byte().expect("fetch SET_LOCAL_1");
        crate::handlers::locals::handle_nibble_locals(opcode, &mut ctx)
            .expect("SET_LOCAL_1 should succeed");

        let opcode = ctx.fetch_byte().expect("fetch GET_LOCAL_1");
        crate::handlers::locals::handle_nibble_locals(opcode, &mut ctx)
            .expect("GET_LOCAL_1 should succeed");

        let round_tripped = ctx.pop().expect("pop round-tripped local");
        assert_eq!(round_tripped.as_u64(), Some(100));
    }

    #[test]
    fn handwritten_compiled_shape_preserves_amount_param() {
        let program_id = Pubkey::from([41u8; 32]);
        let header_len = five_protocol::FIVE_HEADER_OPTIMIZED_SIZE;

        let main_code = [
            five_protocol::opcodes::CHECK_WRITABLE,
            1,
            five_protocol::opcodes::CALL,
            0,
            (header_len as u16 + 16).to_le_bytes()[0],
            (header_len as u16 + 16).to_le_bytes()[1],
            five_protocol::opcodes::LOAD_PARAM_1,
            five_protocol::opcodes::SET_LOCAL_1,
            five_protocol::opcodes::GET_LOCAL_1,
            five_protocol::opcodes::STORE_FIELD,
            1,
            8,
            0,
            0,
            0,
            five_protocol::opcodes::RETURN,
        ];

        let helper_code = [
            five_protocol::opcodes::CHECK_WRITABLE,
            1,
            five_protocol::opcodes::GET_CLOCK,
            five_protocol::opcodes::PUSH_U8,
            0,
            five_protocol::opcodes::TUPLE_GET,
            five_protocol::opcodes::SET_LOCAL_0,
            five_protocol::opcodes::GET_LOCAL_0,
            five_protocol::opcodes::STORE_FIELD,
            1,
            0,
            0,
            0,
            0,
            five_protocol::opcodes::RETURN,
        ];

        let mut bytecode = vec![b'5', b'I', b'V', b'E'];
        bytecode.extend_from_slice(&0u32.to_le_bytes());
        bytecode.push(1);
        bytecode.push(2);
        bytecode.extend_from_slice(&main_code);
        bytecode.extend_from_slice(&helper_code);

        let mut vm_state_lamports = 1;
        let mut vm_state_data = [0u8; 8];
        let vm_state = create_account_info(
            &Pubkey::from([42u8; 32]),
            false,
            false,
            &mut vm_state_lamports,
            &mut vm_state_data,
            &program_id,
        );

        let mut reserve_lamports = 1;
        let mut reserve_data = [0u8; 16];
        let reserve = create_account_info(
            &Pubkey::from([43u8; 32]),
            false,
            true,
            &mut reserve_lamports,
            &mut reserve_data,
            &program_id,
        );

        let accounts = [vm_state, reserve];
        let mut storage = StackStorage::new();
        let mut ctx = ExecutionContext::new(
            &bytecode,
            &accounts,
            program_id,
            &[],
            header_len as u16,
            &mut storage,
            0,
            1,
            0,
            0,
            0,
            0,
        );
        ctx.allocate_params(2).expect("allocate params");
        ctx.set_parameter(0, ValueRef::U64(0)).expect("set func index");
        ctx.set_parameter(1, ValueRef::U64(100)).expect("set amount");
        ctx.allocate_locals(3).expect("allocate caller locals");

        let opcode = ctx.fetch_byte().expect("fetch caller CHECK_WRITABLE");
        handle_constraints(opcode, &mut ctx).expect("caller CHECK_WRITABLE should succeed");

        let opcode = ctx.fetch_byte().expect("fetch CALL");
        handle_functions(opcode, &mut ctx).expect("CALL should succeed");

        let opcode = ctx.fetch_byte().expect("fetch helper CHECK_WRITABLE");
        handle_constraints(opcode, &mut ctx).expect("helper CHECK_WRITABLE should succeed");

        let opcode = ctx.fetch_byte().expect("fetch GET_CLOCK");
        handle_sysvar_ops(opcode, &mut ctx).expect("GET_CLOCK should succeed");

        let opcode = ctx.fetch_byte().expect("fetch PUSH_U8");
        handle_stack_ops(opcode, &mut ctx).expect("PUSH_U8 should succeed");

        let opcode = ctx.fetch_byte().expect("fetch TUPLE_GET");
        handle_option_result_ops(opcode, &mut ctx).expect("TUPLE_GET should succeed");

        let opcode = ctx.fetch_byte().expect("fetch SET_LOCAL_0");
        crate::handlers::locals::handle_nibble_locals(opcode, &mut ctx)
            .expect("SET_LOCAL_0 should succeed");

        let opcode = ctx.fetch_byte().expect("fetch GET_LOCAL_0");
        crate::handlers::locals::handle_nibble_locals(opcode, &mut ctx)
            .expect("GET_LOCAL_0 should succeed");

        let opcode = ctx.fetch_byte().expect("fetch helper STORE_FIELD");
        handle_memory(opcode, &mut ctx).expect("helper STORE_FIELD should succeed");

        let opcode = ctx.fetch_byte().expect("fetch helper RETURN");
        handle_control_flow(opcode, &mut ctx).expect("helper RETURN should succeed");
        assert_eq!(
            ctx.parameters()[1].as_u64(),
            Some(100),
            "caller param should survive helper RETURN"
        );

        let opcode = ctx.fetch_byte().expect("fetch LOAD_PARAM_1");
        crate::handlers::locals::handle_nibble_locals(opcode, &mut ctx)
            .expect("LOAD_PARAM_1 should succeed");
        assert_eq!(
            ctx.peek().expect("peek after LOAD_PARAM_1").as_u64(),
            Some(100),
            "LOAD_PARAM_1 should push the original amount"
        );

        let opcode = ctx.fetch_byte().expect("fetch SET_LOCAL_1");
        crate::handlers::locals::handle_nibble_locals(opcode, &mut ctx)
            .expect("SET_LOCAL_1 should succeed");
        assert_eq!(
            ctx.get_local(1).expect("local 1 after SET_LOCAL_1").as_u64(),
            Some(100),
            "SET_LOCAL_1 should preserve the original amount"
        );

        let opcode = ctx.fetch_byte().expect("fetch GET_LOCAL_1");
        crate::handlers::locals::handle_nibble_locals(opcode, &mut ctx)
            .expect("GET_LOCAL_1 should succeed");
        assert_eq!(
            ctx.peek().expect("peek after GET_LOCAL_1").as_u64(),
            Some(100),
            "GET_LOCAL_1 should reload the original amount"
        );

        let opcode = ctx.fetch_byte().expect("fetch caller STORE_FIELD");
        handle_memory(opcode, &mut ctx).expect("caller STORE_FIELD should succeed");

        let (_account_last_update, account_protocol_fees) = {
            let account = ctx
                .get_account_for_read(1)
                .expect("get reserve account after STORE_FIELD");
            let data = unsafe { account.borrow_data_unchecked() };
            (
                u64::from_le_bytes(data[0..8].try_into().expect("account last_update bytes")),
                u64::from_le_bytes(data[8..16].try_into().expect("account protocol_fees bytes")),
            )
        };
        let last_update_slot = u64::from_le_bytes(
            reserve_data[0..8]
                .try_into()
                .expect("last_update_slot bytes"),
        );
        let protocol_fees = u64::from_le_bytes(
            reserve_data[8..16]
                .try_into()
                .expect("protocol_fees bytes"),
        );
        assert_eq!(account_protocol_fees, 100, "account view should reflect STORE_FIELD");
        assert_eq!(last_update_slot, 0, "raw backing slice stays stale in host tests");
        assert_eq!(protocol_fees, 0, "raw backing slice stays stale in host tests");
    }

    #[test]
    fn compiled_helper_get_clock_then_field_store_preserves_amount_param() {
        let source = r#"
            account Reserve {
                last_update_slot: u64,
                protocol_fees: u64,
            }

            fn refresh_reserve_internal(reserve: Reserve @mut) {
                let current_time: u64 = get_clock().slot;
                reserve.last_update_slot = current_time;
            }

            pub deposit_reserve_liquidity(reserve: Reserve @mut, amount: u64) {
                refresh_reserve_internal(reserve);
                let captured_amount: u64 = amount;
                reserve.protocol_fees = captured_amount;
                return;
            }
        "#;

        let bytecode = DslCompiler::compile_dsl(source).expect("compile reduction");
        for line in five_dsl_compiler::bytecode_generator::disassembler::disassemble(&bytecode) {
            println!("DISASM {}", line);
        }
        let program_id = Pubkey::from([31u8; 32]);

        let mut vm_state_lamports = 1;
        let mut vm_state_data = [0u8; 8];
        let vm_state = create_account_info(
            &Pubkey::from([32u8; 32]),
            false,
            false,
            &mut vm_state_lamports,
            &mut vm_state_data,
            &program_id,
        );

        let mut reserve_lamports = 1;
        let mut reserve_data = [0u8; 16];
        let reserve = create_account_info(
            &Pubkey::from([33u8; 32]),
            false,
            true,
            &mut reserve_lamports,
            &mut reserve_data,
            &program_id,
        );

        let accounts = [vm_state, reserve];
        let mut input = Vec::new();
        input.extend_from_slice(&0u32.to_le_bytes());
        input.extend_from_slice(&2u32.to_le_bytes());
        input.push(five_protocol::types::ACCOUNT);
        input.extend_from_slice(&1u32.to_le_bytes());
        input.push(five_protocol::types::U64);
        input.extend_from_slice(&100u64.to_le_bytes());

        let mut storage = StackStorage::new();
        let result = MitoVM::execute_direct(&bytecode, &input, &accounts, &program_id, &mut storage);
        assert!(result.is_ok(), "reduction execution failed: {:?}", result);

        let account_protocol_fees = {
            let account = &accounts[1];
            let data = unsafe { account.borrow_data_unchecked() };
            u64::from_le_bytes(data[8..16].try_into().expect("protocol_fees bytes"))
        };
        let raw_protocol_fees = u64::from_le_bytes(
            reserve_data[8..16]
                .try_into()
                .expect("protocol_fees bytes"),
        );
        assert_eq!(
            account_protocol_fees,
            100,
            "amount param should survive helper call in the account view"
        );
        assert_eq!(
            raw_protocol_fees,
            0,
            "raw backing slice is stale in host execute_direct tests"
        );
    }
}
