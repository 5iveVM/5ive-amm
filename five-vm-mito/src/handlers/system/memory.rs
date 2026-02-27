//! Memory syscall handlers

use crate::{
    context::ExecutionManager,
    debug_log,
    error::{CompactResult, VMErrorCode},
};
use five_protocol::ValueRef;

#[cfg(target_os = "solana")]
use pinocchio::syscalls;

/// Helper to get mutable pointer to a buffer defined by ValueRef
fn get_mut_ptr(
    ctx: &mut ExecutionManager,
    reference: ValueRef,
    len: u64,
) -> CompactResult<*mut u8> {
    match reference {
        ValueRef::TempRef(offset, size) => {
            if (size as u64) < len {
                return Err(VMErrorCode::MemoryViolation);
            }
            let start = offset as usize;
            let end = start + len as usize;
            if end > ctx.temp_buffer().len() {
                return Err(VMErrorCode::MemoryViolation);
            }
            Ok(unsafe { ctx.temp_buffer_mut().as_mut_ptr().add(start) })
        }
        // Add HeapRef support if needed
        _ => Err(VMErrorCode::TypeMismatch),
    }
}

/// Helper to get const pointer to a buffer defined by ValueRef
fn get_ptr(ctx: &mut ExecutionManager, reference: ValueRef, len: u64) -> CompactResult<*const u8> {
    match reference {
        ValueRef::TempRef(offset, size) => {
            if (size as u64) < len {
                return Err(VMErrorCode::MemoryViolation);
            }
            let start = offset as usize;
            let end = start + len as usize;
            if end > ctx.temp_buffer().len() {
                return Err(VMErrorCode::MemoryViolation);
            }
            Ok(unsafe { ctx.temp_buffer().as_ptr().add(start) })
        }
        ValueRef::StringRef(_) | ValueRef::ArrayRef(_) => {
            // For String/Array, reuse extract_string_slice and return content bytes.
            let (slen, bytes) = ctx.extract_string_slice(&reference)?;
            if (slen as u64) < len {
                return Err(VMErrorCode::MemoryViolation);
            }
            Ok(bytes.as_ptr())
        }
        _ => Err(VMErrorCode::TypeMismatch),
    }
}

/// Handle sol_memcpy syscall
#[inline(always)]
pub fn handle_syscall_memcpy(ctx: &mut ExecutionManager) -> CompactResult<()> {
    debug_log!("MitoVM: SYSCALL_MEMCPY");

    // Pop: n, src, dst (standard memcpy order: dst, src, n)
    // Stack: push dst, push src, push n.
    // Pop: n, src, dst.

    let n = ctx.pop()?.as_u64().ok_or(VMErrorCode::TypeMismatch)?;
    let src_ref = ctx.pop()?;
    let dst_ref = ctx.pop()?;

    let dst_ptr = get_mut_ptr(ctx, dst_ref, n)?;
    let src_ptr = get_ptr(ctx, src_ref, n)?;

    #[cfg(target_os = "solana")]
    unsafe {
        syscalls::sol_memcpy_(dst_ptr, src_ptr, n);
    }
    #[cfg(not(target_os = "solana"))]
    {
        // Safety: We verified bounds in get_ptr helpers (assuming temp buffer is contiguous)
        // But since we have two pointers into same buffer (potentially), Rust rules might be violated if we used safe slices.
        // Unsafe copy is appropriate here for simulation too.
        unsafe {
            core::ptr::copy_nonoverlapping(src_ptr, dst_ptr, n as usize);
        }
    }

    Ok(())
}

/// Handle sol_memmove syscall
#[inline(always)]
pub fn handle_syscall_memmove(ctx: &mut ExecutionManager) -> CompactResult<()> {
    debug_log!("MitoVM: SYSCALL_MEMMOVE");

    let n = ctx.pop()?.as_u64().ok_or(VMErrorCode::TypeMismatch)?;
    let src_ref = ctx.pop()?;
    let dst_ref = ctx.pop()?;

    let dst_ptr = get_mut_ptr(ctx, dst_ref, n)?;
    let src_ptr = get_ptr(ctx, src_ref, n)?;

    #[cfg(target_os = "solana")]
    unsafe {
        syscalls::sol_memmove_(dst_ptr, src_ptr, n);
    }
    #[cfg(not(target_os = "solana"))]
    {
        unsafe {
            core::ptr::copy(src_ptr, dst_ptr, n as usize);
        }
    }

    Ok(())
}

/// Handle sol_memset syscall
#[inline(always)]
pub fn handle_syscall_memset(ctx: &mut ExecutionManager) -> CompactResult<()> {
    debug_log!("MitoVM: SYSCALL_MEMSET");

    // memset(dst, val, n)
    // Stack: push dst, push val, push n.
    // Pop: n, val, dst.

    let n = ctx.pop()?.as_u64().ok_or(VMErrorCode::TypeMismatch)?;
    let val = ctx.pop()?.as_u8().ok_or(VMErrorCode::TypeMismatch)?;
    let dst_ref = ctx.pop()?;

    let dst_ptr = get_mut_ptr(ctx, dst_ref, n)?;

    #[cfg(target_os = "solana")]
    unsafe {
        syscalls::sol_memset_(dst_ptr, val, n);
    }
    #[cfg(not(target_os = "solana"))]
    {
        unsafe {
            core::ptr::write_bytes(dst_ptr, val, n as usize);
        }
    }

    Ok(())
}

/// Handle sol_memcmp syscall
#[inline(always)]
pub fn handle_syscall_memcmp(ctx: &mut ExecutionManager) -> CompactResult<()> {
    debug_log!("MitoVM: SYSCALL_MEMCMP");

    // memcmp(s1, s2, n, result) -> result is written to *mut i32?
    // Pinocchio syscalls: `fn sol_memcmp_(s1: *const u8, s2: *const u8, n: u64, result: *mut i32)`
    // Arguments: result_ptr, n, s2, s1.
    // Stack: push s1, push s2, push n, push result.
    // Pop: result, n, s2, s1.

    // But result is an output.
    // Typically in VM, we push result to stack.
    // If the syscall requires a pointer, we must allocate a temp slot for it.
    // Or does the user provide a buffer for result?
    // If we follow generic CALL_NATIVE pattern where arguments match syscall args:
    // Then user provides result buffer (TempRef 4 bytes).

    let res_ref = ctx.pop()?;
    let n = ctx.pop()?.as_u64().ok_or(VMErrorCode::TypeMismatch)?;
    let s2_ref = ctx.pop()?;
    let s1_ref = ctx.pop()?;

    let s1_ptr = get_ptr(ctx, s1_ref, n)?;
    let s2_ptr = get_ptr(ctx, s2_ref, n)?;
    let res_ptr = get_mut_ptr(ctx, res_ref, 4)? as *mut i32;

    #[cfg(target_os = "solana")]
    unsafe {
        syscalls::sol_memcmp_(s1_ptr, s2_ptr, n, res_ptr);
    }
    #[cfg(not(target_os = "solana"))]
    {
        // Implementation of memcmp
        let s1 = unsafe { core::slice::from_raw_parts(s1_ptr, n as usize) };
        let s2 = unsafe { core::slice::from_raw_parts(s2_ptr, n as usize) };
        let mut cmp = 0;
        for i in 0..n as usize {
            if s1[i] != s2[i] {
                cmp = if s1[i] < s2[i] { -1 } else { 1 };
                break;
            }
        }
        unsafe {
            *res_ptr = cmp;
        }
    }

    Ok(())
}
