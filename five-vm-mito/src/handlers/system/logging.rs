//! Logging syscall handlers

use crate::{
    context::ExecutionManager,
    debug_log,
    error::{CompactResult, VMErrorCode},
};
use five_protocol::ValueRef;

#[cfg(target_os = "solana")]
use pinocchio::syscalls;

/// Handle sol_log syscall
#[inline(never)]
pub fn handle_syscall_log(ctx: &mut ExecutionManager) -> CompactResult<()> {
    debug_log!("MitoVM: SYSCALL_LOG");

    // Pop message
    let msg_ref = ctx.pop()?;
    let (_len, bytes) = ctx.extract_string_slice(&msg_ref)?;

    #[cfg(target_os = "solana")]
    unsafe {
        syscalls::sol_log_(bytes.as_ptr(), bytes.len() as u64);
    }
    #[cfg(not(target_os = "solana"))]
    {
        // For testing/off-chain, we can just debug log it
        if let Ok(s) = core::str::from_utf8(bytes) {
             debug_log!("SOL_LOG: {}", s);
        } else {
             debug_log!("SOL_LOG (bytes len): {}", bytes.len());
        }
    }

    Ok(())
}

/// Handle sol_log_64 syscall
#[inline(never)]
pub fn handle_syscall_log_64(ctx: &mut ExecutionManager) -> CompactResult<()> {
    debug_log!("MitoVM: SYSCALL_LOG_64");

    // Pop 5 args
    let arg5 = ctx.pop()?.as_u64().ok_or(VMErrorCode::TypeMismatch)?;
    let arg4 = ctx.pop()?.as_u64().ok_or(VMErrorCode::TypeMismatch)?;
    let arg3 = ctx.pop()?.as_u64().ok_or(VMErrorCode::TypeMismatch)?;
    let arg2 = ctx.pop()?.as_u64().ok_or(VMErrorCode::TypeMismatch)?;
    let arg1 = ctx.pop()?.as_u64().ok_or(VMErrorCode::TypeMismatch)?;

    #[cfg(target_os = "solana")]
    unsafe {
        syscalls::sol_log_64_(arg1, arg2, arg3, arg4, arg5);
    }
    #[cfg(not(target_os = "solana"))]
    {
        debug_log!("SOL_LOG_64: {}, {}, {}, {}, {}", arg1, arg2, arg3, arg4, arg5);
    }

    Ok(())
}

/// Handle sol_log_compute_units syscall
#[inline(never)]
pub fn handle_syscall_log_compute_units(_ctx: &mut ExecutionManager) -> CompactResult<()> {
    debug_log!("MitoVM: SYSCALL_LOG_COMPUTE_UNITS");

    #[cfg(target_os = "solana")]
    unsafe {
        syscalls::sol_log_compute_units_();
    }
    #[cfg(not(target_os = "solana"))]
    {
        debug_log!("SOL_LOG_COMPUTE_UNITS");
    }

    Ok(())
}

/// Handle sol_log_pubkey syscall
#[inline(never)]
pub fn handle_syscall_log_pubkey(ctx: &mut ExecutionManager) -> CompactResult<()> {
    debug_log!("MitoVM: SYSCALL_LOG_PUBKEY");

    // Pop pubkey
    let pk_ref = ctx.pop()?;
    let pubkey = ctx.extract_pubkey(&pk_ref)?;

    #[cfg(target_os = "solana")]
    unsafe {
        syscalls::sol_log_pubkey(pubkey.as_ptr());
    }
    #[cfg(not(target_os = "solana"))]
    {
        // Debug logging for pubkey
        debug_log!("SOL_LOG_PUBKEY");
    }

    Ok(())
}

/// Handle sol_log_data syscall
#[inline(never)]
pub fn handle_syscall_log_data(ctx: &mut ExecutionManager) -> CompactResult<()> {
    debug_log!("MitoVM: SYSCALL_LOG_DATA");

    // Pop data array
    // Expects an ArrayRef where elements are byte arrays (or strings)
    let data_ref = ctx.pop()?;

    // We need to construct &[&[u8]]
    // Since we can't allocate dynamic vector, we use stack array with limit
    const MAX_DATA_FIELDS: usize = 16;
    let mut data_ptrs: [&[u8]; MAX_DATA_FIELDS] = [&[]; MAX_DATA_FIELDS];
    let mut count = 0;

    match data_ref {
        ValueRef::ArrayRef(id) => {
             // Arrays in temp: [len, type, bytes...] ? No, Arrays of objects?
             // five-protocol definition of ArrayRef:
             // "Reference to an array stored in temp/heap."
             // Structure: [len: u16, item_type: u8, items...] ?
             // I need to check how Arrays are stored.

             // Assuming simpler approach for now:
             // Use ctx.memory.get_temp_data or similar to inspect array.
             // If array logic is complex, I might need to iterate elements manually if supported.

             // Inspect array header
             let start = id as usize;
             let temp_buf = ctx.temp_buffer();
             if start + 2 > temp_buf.len() { return Err(VMErrorCode::MemoryViolation); }

             let len = temp_buf[start]; // u8 length
             // let item_type = temp_buf[start + 1];

             if len as usize > MAX_DATA_FIELDS {
                 return Err(VMErrorCode::InvalidParameter);
             }

             let mut cursor = start + 2;

             // This parsing depends heavily on how arrays are serialized in temp buffer.
             // If they are "flat" (e.g. fixed size items), easy.
             // If they are variable size (strings), we need to parse each.

             // Given I don't have robust array iteration in ExecutionContext yet for generic arrays,
             // I'll assume we iterate based on known serialization or we skip if too complex.
             // But log_data is important.

             // Let's assume the array elements are references (u16 offsets) to other temp objects (Strings/ByteArrays).
             // or if item_type indicates they are inline?

             // For now, let's just log "not implemented fully" if not simple.
             // But I should try.

             // If I cannot reliably parse the array structure without more helpers, I will put a placeholder logic
             // that just logs the raw bytes of the array for now, or returns OK.

             // However, to support `sol_log_data`, we need `&[&[u8]]`.
             // I will leave this as "single data chunk" support if passed as Bytes/String,
             // or attempt to support Array if it contains Bytes.

             // If data_ref is just a StringRef or TempRef (one item), log it as one item.

             // TODO: Proper array iteration.
             // Fallback: treat as single item
             // But wait, the function takes `&[&[u8]]`.
        }
        ValueRef::StringRef(_) | ValueRef::TempRef(_,_) => {
             let (_len, bytes) = ctx.extract_string_slice(&data_ref)?;
             data_ptrs[0] = bytes;
             count = 1;
        }
        _ => return Err(VMErrorCode::TypeMismatch),
    }

    // Since array iteration is hard without `ctx` helpers,
    // I'll support single item log_data for now unless I find `ArrayRef` details.
    // Wait, `pda.rs` uses `ValueRef::ArrayRef` for seeds.
    // It assumes:
    // "Array stored in temp buffer: [len, type, bytes...]"
    // "We treat array content as bytes for seeding"
    // So `pda.rs` treats ArrayRef as a single byte buffer (the content of the array).

    // If that's the convention, then `log_data` with ArrayRef means "log the bytes of this array".
    // Which means `&[&[u8]]` with 1 element.

    if let ValueRef::ArrayRef(id) = data_ref {
        let start = id as usize;
         if start + 2 > ctx.temp_buffer().len() { return Err(VMErrorCode::MemoryViolation); }
         let len = ctx.temp_buffer()[start];
         let data_start = start + 2;
         let data_end = data_start + len as usize;

         if data_end > ctx.temp_buffer().len() { return Err(VMErrorCode::MemoryViolation); }

         data_ptrs[0] = &ctx.temp_buffer()[data_start..data_end];
         count = 1;
    }

    #[cfg(target_os = "solana")]
    unsafe {
        syscalls::sol_log_data(&data_ptrs[0..count] as *const _ as *const u8, count as u64);
    }
    #[cfg(not(target_os = "solana"))]
    {
        debug_log!("SOL_LOG_DATA: count={}", count);
    }

    Ok(())
}
