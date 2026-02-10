//! Logging syscall handlers.

use crate::{
    context::ExecutionManager,
    debug_log,
    error::{CompactResult, VMErrorCode},
};
use five_protocol::ValueRef;

#[cfg(target_os = "solana")]
use pinocchio::syscalls;

/// Handle sol_log syscall.
#[inline(always)]
pub fn handle_syscall_log(ctx: &mut ExecutionManager) -> CompactResult<()> {
    debug_log!("MitoVM: SYSCALL_LOG");

    let msg_ref = ctx.pop()?;
    let (_len, bytes) = ctx.extract_string_slice(&msg_ref)?;

    #[cfg(target_os = "solana")]
    unsafe {
        syscalls::sol_log_(bytes.as_ptr(), bytes.len() as u64);
    }
    #[cfg(not(target_os = "solana"))]
    {
        if let Ok(s) = core::str::from_utf8(bytes) {
             debug_log!("SOL_LOG: {}", s);
        } else {
             debug_log!("SOL_LOG (bytes len): {}", bytes.len());
        }
    }

    Ok(())
}

/// Handle sol_log_64 syscall.
#[inline(always)]
pub fn handle_syscall_log_64(ctx: &mut ExecutionManager) -> CompactResult<()> {
    debug_log!("MitoVM: SYSCALL_LOG_64");

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

/// Handle sol_log_compute_units syscall.
#[inline(always)]
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

/// Handle sol_log_pubkey syscall.
#[inline(always)]
pub fn handle_syscall_log_pubkey(ctx: &mut ExecutionManager) -> CompactResult<()> {
    debug_log!("MitoVM: SYSCALL_LOG_PUBKEY");

    let pk_ref = ctx.pop()?;
    let pubkey = ctx.extract_pubkey(&pk_ref)?;

    #[cfg(target_os = "solana")]
    unsafe {
        syscalls::sol_log_pubkey(pubkey.as_ptr());
    }
    #[cfg(not(target_os = "solana"))]
    {
        debug_log!("SOL_LOG_PUBKEY");
    }

    Ok(())
}

/// Handle sol_log_data syscall.
#[inline(always)]
pub fn handle_syscall_log_data(ctx: &mut ExecutionManager) -> CompactResult<()> {
    debug_log!("MitoVM: SYSCALL_LOG_DATA");

    let data_ref = ctx.pop()?;
    const MAX_DATA_FIELDS: usize = 16;
    let mut data_ptrs: [&[u8]; MAX_DATA_FIELDS] = [&[]; MAX_DATA_FIELDS];
    let count = match data_ref {
        ValueRef::ArrayRef(id) => {
            let start = id as usize;
            if start + 2 > ctx.temp_buffer().len() { return Err(VMErrorCode::MemoryViolation); }
            let len = ctx.temp_buffer()[start];
            let data_start = start + 2;
            let data_end = data_start + len as usize;

            if data_end > ctx.temp_buffer().len() { return Err(VMErrorCode::MemoryViolation); }

            data_ptrs[0] = &ctx.temp_buffer()[data_start..data_end];
            1
        }
        ValueRef::StringRef(_) | ValueRef::TempRef(_,_) => {
             let (_len, bytes) = ctx.extract_string_slice(&data_ref)?;
             data_ptrs[0] = bytes;
             1
        }
        _ => return Err(VMErrorCode::TypeMismatch),
    };

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
