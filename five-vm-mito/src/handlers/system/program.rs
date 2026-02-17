//! Program data syscall handlers

use crate::{
    context::ExecutionManager,
    debug_log,
    error::{CompactResult, VMErrorCode},
};
use five_protocol::ValueRef;

#[cfg(target_os = "solana")]
use pinocchio::{syscalls, pubkey::Pubkey};

/// Handle sol_get_return_data syscall
#[inline(always)]
pub fn handle_syscall_get_return_data(ctx: &mut ExecutionManager) -> CompactResult<()> {
    debug_log!("MitoVM: SYSCALL_GET_RETURN_DATA");

    // Pop arguments in reverse push order: pid, len, data.

    let pid_ref = ctx.pop()?;
    let length = ctx.pop()?.as_u64().ok_or(VMErrorCode::TypeMismatch)?;
    let data_ref = ctx.pop()?;

    // Get mutable buffers
    // We need to write to these buffers.
    // They must be TempRefs or HeapRefs.

    // Check buffers
    // Program ID buffer (32 bytes)
    let pid_offset = match pid_ref {
        ValueRef::TempRef(offset, len) => {
            if len < 32 { return Err(VMErrorCode::MemoryViolation); }
            offset
        }
        ValueRef::ArrayRef(_offset) => {
            // ArrayRef buffers are not supported for program ID writes in this syscall.
            debug_log!(
                "SYSCALL_GET_RETURN_DATA: ArrayRef {} not supported for pid buffer",
                _offset
            );
            return Err(VMErrorCode::TypeMismatch); // Enforce TempRef for now
        }
         _ => return Err(VMErrorCode::TypeMismatch),
    };

    // Data buffer
    let data_offset = match data_ref {
        ValueRef::TempRef(offset, len) => {
            if (len as u64) < length { return Err(VMErrorCode::MemoryViolation); }
            offset
        }
         _ => return Err(VMErrorCode::TypeMismatch),
    };

    let result_len = {
        #[cfg(target_os = "solana")]
        unsafe {
            // We need raw pointers to the temp buffer slots
            let temp_base = ctx.temp_buffer_mut().as_mut_ptr();
            let pid_ptr = temp_base.add(pid_offset as usize) as *mut Pubkey;
            let data_ptr = temp_base.add(data_offset as usize);

            syscalls::sol_get_return_data(data_ptr, length, pid_ptr)
        }

    #[cfg(not(target_os = "solana"))]
    {
        let _ = (pid_offset, data_offset);
        // Mock
        debug_log!(
            "SOL_GET_RETURN_DATA: length={} pid_offset={} data_offset={}",
                length,
                pid_offset,
                data_offset
            );
            0
        }
    };

    ctx.push(ValueRef::U64(result_len))?;
    Ok(())
}

/// Handle sol_set_return_data syscall
#[inline(always)]
pub fn handle_syscall_set_return_data(ctx: &mut ExecutionManager) -> CompactResult<()> {
    debug_log!("MitoVM: SYSCALL_SET_RETURN_DATA");

    // Pop data
    let data_ref = ctx.pop()?;
    let (_len, bytes) = ctx.extract_string_slice(&data_ref)?;

    #[cfg(target_os = "solana")]
    unsafe {
        syscalls::sol_set_return_data(bytes.as_ptr(), _len as u64);
    }
    #[cfg(not(target_os = "solana"))]
    {
        let _ = bytes;
        debug_log!(
            "SOL_SET_RETURN_DATA: len={} first_byte={}",
            _len,
            bytes.get(0).copied().unwrap_or(0)
        );
    }

    Ok(())
}

/// Handle sol_get_stack_height syscall
#[inline(always)]
pub fn handle_syscall_get_stack_height(ctx: &mut ExecutionManager) -> CompactResult<()> {
    debug_log!("MitoVM: SYSCALL_GET_STACK_HEIGHT");

    let height;
    #[cfg(target_os = "solana")]
    unsafe {
        height = syscalls::sol_get_stack_height();
    }
    #[cfg(not(target_os = "solana"))]
    {
        height = 0;
    }

    ctx.push(ValueRef::U64(height))?;
    Ok(())
}

/// Handle sol_get_processed_sibling_instruction syscall
#[inline(always)]
pub fn handle_syscall_get_processed_sibling_instruction(_ctx: &mut ExecutionManager) -> CompactResult<()> {
    debug_log!("MitoVM: SYSCALL_GET_PROCESSED_SIBLING_INSTRUCTION - Not Implemented Fully");
    // This syscall requires complex struct mapping (ProcessedSiblingInstruction).
    // Stubbed: return 0 (false/failure).
    // To implement fully, we need to pop buffers for meta, program_id, data, accounts.

    // Placeholder behavior:
    Err(VMErrorCode::InvalidOperation)
}
