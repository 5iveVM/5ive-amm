//! Native syscall handler for MitoVM
//!
//! Provides a generic dispatch mechanism for invoking Pinocchio/solana
//! syscalls from MitoVM bytecode via the CALL_NATIVE opcode.

use crate::{
    context::ExecutionManager,
    debug_log,
    error::{CompactResult, VMErrorCode},
};
use core::{mem, ptr};
use five_protocol::ValueRef;
use pinocchio::sysvars::{clock::Clock, Sysvar};

/// Maximum number of arguments a syscall can accept.
const MAX_SYSCALL_ARGS: usize = 8;

/// Entry describing a syscall mapping.
struct SyscallEntry {
    func: fn(&mut ExecutionManager, &[ValueRef]) -> CompactResult<ValueRef>,
    cu_cost: u64,
    #[allow(dead_code)]
    name: &'static str,
}

/// Wrapper for `sol_log_` – logs a UTF-8 string from the temp buffer.
fn syscall_sol_log(ctx: &mut ExecutionManager, args: &[ValueRef]) -> CompactResult<ValueRef> {
    if args.len() != 1 {
        return Err(VMErrorCode::InvalidOperation);
    }
    let msg_slice = match args[0] {
        ValueRef::TempRef(offset, len) => {
            let start = offset as usize;
            let end = start + len as usize;
            &ctx.temp_buffer()[start..end]
        }
        _ => return Err(VMErrorCode::TypeMismatch),
    };
    if let Ok(_msg) = core::str::from_utf8(msg_slice) {
        #[cfg(feature = "debug-logs")]
        debug_log!("MitoVM: sol_log_ invoked with message: {}", _msg);
    } else {
        debug_log!("MitoVM: sol_log_ received non UTF-8 data");
    }
    Ok(ValueRef::Empty)
}

/// Wrapper for `sol_get_clock_sysvar` – returns the Clock sysvar.
fn syscall_get_clock(ctx: &mut ExecutionManager, _args: &[ValueRef]) -> CompactResult<ValueRef> {
    let clock = Clock::get().map_err(|_| VMErrorCode::InvalidOperation)?;
    let buf = ctx.temp_buffer_mut();
    let size = mem::size_of::<Clock>();
    if buf.len() < size {
        return Err(VMErrorCode::MemoryViolation);
    }
    unsafe {
        ptr::copy_nonoverlapping(&clock as *const Clock as *const u8, buf.as_mut_ptr(), size);
    }
    Ok(ValueRef::TupleRef(0, size as u8))
}

/// Static table mapping syscall IDs to wrappers and CU costs.
static SYSCALLS: &[SyscallEntry] = &[
    SyscallEntry {
        func: syscall_sol_log,
        cu_cost: 200,
        name: "sol_log_",
    },
    SyscallEntry {
        func: syscall_get_clock,
        cu_cost: 200,
        name: "sol_get_clock_sysvar",
    },
];

/// Dispatch CALL_NATIVE to the appropriate syscall handler.
#[inline(always)]
pub fn handle_native_ops(ctx: &mut ExecutionManager) -> CompactResult<()> {
    debug_log!("MitoVM: CALL_NATIVE operation");

    let syscall_id = ctx.pop()?.as_u8().ok_or(VMErrorCode::TypeMismatch)? as usize;
    let arg_count = ctx.pop()?.as_u8().ok_or(VMErrorCode::TypeMismatch)? as usize;

    if arg_count > MAX_SYSCALL_ARGS {
        return Err(VMErrorCode::InvalidOperation);
    }

    let mut args = [ValueRef::Empty; MAX_SYSCALL_ARGS];
    for i in 0..arg_count {
        args[arg_count - 1 - i] = ctx.pop()?;
    }

    let entry = SYSCALLS
        .get(syscall_id)
        .ok_or(VMErrorCode::InvalidInstruction)?;
    ctx.consume_compute_units(entry.cu_cost);

    let result = (entry.func)(ctx, &args[..arg_count])?;
    if !matches!(result, ValueRef::Empty) {
        ctx.push(result)?;
    }
    Ok(())
}
