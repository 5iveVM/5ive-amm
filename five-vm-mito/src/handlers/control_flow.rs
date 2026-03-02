//! Control flow operations handler for MitoVM
//!
//! This module handles control flow operations including HALT, JUMP, JUMP_IF,
//! JUMP_IF_NOT, REQUIRE, ASSERT, RETURN, RETURN_VALUE, and NOP. It manages
//! program flow control and execution state transitions.

use crate::{
    context::ExecutionManager,
    debug_log,
    error::{CompactResult, VMErrorCode},
};
use five_protocol::opcodes::*;

#[inline(always)]
fn value_to_u64(value: five_protocol::ValueRef) -> Option<u64> {
    match value {
        five_protocol::ValueRef::U8(v) => Some(v as u64),
        five_protocol::ValueRef::U64(v) => Some(v),
        five_protocol::ValueRef::Bool(v) => Some(v as u64),
        _ => None,
    }
}

/// Validate and perform a jump to the given offset.
/// Returns an error if the offset is out of bounds.
#[inline(always)]
fn validate_and_jump(ctx: &mut ExecutionManager, offset: usize) -> CompactResult<()> {
    if offset >= ctx.script().len() {
        return Err(VMErrorCode::InvalidInstructionPointer);
    }
    ctx.set_ip(offset);
    Ok(())
}

/// Check a condition and return ConstraintViolation error if false.
/// Used by both REQUIRE and ASSERT opcodes (functionally identical).
#[inline(always)]
fn check_condition(ctx: &mut ExecutionManager) -> CompactResult<()> {
    let condition = ctx.pop()?;
    if !condition.is_truthy() {
        return Err(VMErrorCode::ConstraintViolation);
    }
    Ok(())
}

/// Execute control flow opcodes including jumps, conditionals, and program termination.
/// Handles the 0x00-0x0F opcode range.
#[inline(always)]
pub fn handle_control_flow(opcode: u8, ctx: &mut ExecutionManager) -> CompactResult<()> {
    match opcode {
        HALT => {
            ctx.set_halted(true);
        }
        JUMP => {
            let offset = ctx.fetch_u16()? as usize;
            validate_and_jump(ctx, offset)?;
        }
        JUMP_IF => {
            let offset = ctx.fetch_u16()? as usize;
            let condition = ctx.pop()?;
            if condition.is_truthy() {
                validate_and_jump(ctx, offset)?;
            }
        }
        JUMP_IF_NOT => {
            let offset = ctx.fetch_u16()? as usize;
            let condition = ctx.pop()?;
            if !condition.is_truthy() {
                validate_and_jump(ctx, offset)?;
            }
        }
        REQUIRE => {
            check_condition(ctx)?;
        }
        ASSERT => {
            check_condition(ctx)?;
        }
        RETURN => {
            // Check if we're in a function call
            if ctx.call_depth() > 0 {
                debug_log!(
                    "MitoVM: RETURN from function - call depth: {}",
                    ctx.call_depth() as u32
                );

                // CRITICAL FIX (Issue 1.4): Use pop_call_frame() for atomic frame access + depth decrement
                let frame = ctx.pop_call_frame()?;

                // Safety check: Validate return address
                // Note: We check against the script length AFTER restoring context logic below,
                // but frame.return_address refers to offset in the CALLER script.
                // So we need to restore context first.

                // Restore previous state safely including local base offset
                ctx.set_ip(frame.return_address as usize);
                ctx.set_local_count(frame.local_count);
                ctx.set_local_base(frame.local_base); // Restore per-frame local window

                // Restore caller bytecode directly from saved script slice pointer.
                if frame.bytecode_context != crate::types::ROOT_CONTEXT
                    && frame.caller_script_ptr != 0
                    && frame.caller_script_len > 0
                {
                    // SAFETY: Stored from a live script slice at call time and valid for tx lifetime.
                    let script = unsafe {
                        core::slice::from_raw_parts(
                            frame.caller_script_ptr as *const u8,
                            frame.caller_script_len as usize,
                        )
                    };
                    ctx.set_script(script);
                } else if frame.bytecode_context == crate::types::ROOT_CONTEXT {
                    // Compatibility fallback for old frames.
                    ctx.set_script(ctx.root_bytecode);
                } else {
                    let account = ctx.accounts.get_unchecked(frame.bytecode_context)?;
                    let data = unsafe { account.borrow_data_unchecked() };
                    const SCRIPT_ACCOUNT_HEADER_LEN: usize = 64;
                    if data.len() < SCRIPT_ACCOUNT_HEADER_LEN {
                        return Err(VMErrorCode::AccountDataEmpty);
                    }
                    ctx.set_script(&data[SCRIPT_ACCOUNT_HEADER_LEN..]);
                }
                ctx.current_context = frame.bytecode_context;
                ctx.set_temp_offset(frame.saved_temp_offset as usize);

                // Verify IP against restored script
                if ctx.ip() >= ctx.script().len() {
                    debug_log!(
                        "MitoVM: ERROR - Invalid return address: {} (script length: {})",
                        ctx.ip() as u32,
                        ctx.script().len() as u32
                    );
                    return Err(VMErrorCode::InvalidInstructionPointer);
                }

                ctx.set_external_account_remap(frame.account_remap);
                ctx.parameters_mut().copy_from_slice(&frame.saved_parameters);
                ctx.restore_parameters(frame.param_start, frame.param_len); // Restore caller's parameters
            } else {
                // Top-level return, halt the script
                ctx.set_halted(true);
            }
        }
        RETURN_VALUE => {
            // Safety check: Ensure there's a value on the stack to return
            if ctx.is_empty() {
                debug_log!(
                    "MitoVM: ERROR - RETURN_VALUE with empty stack - this will cause StackError"
                );
                return Err(VMErrorCode::StackError);
            }

            // Check if we're in a function call
            if ctx.call_depth() > 0 {
                // Safety check: Ensure call depth is valid
                if ctx.call_depth() == 0 {
                    debug_log!("MitoVM: ERROR - Call depth underflow detected");
                    return Err(VMErrorCode::CallStackUnderflow);
                }

                // Pop call stack safely using pop_call_frame instead of manual access
                let frame = ctx.pop_call_frame()?;

                // Restore previous state safely - CRITICAL: Leave return value on stack untouched
                ctx.set_ip(frame.return_address as usize);
                ctx.set_local_count(frame.local_count);
                ctx.set_local_base(frame.local_base); // Restore per-frame local window

                // Restore caller bytecode directly from saved script slice pointer.
                if frame.bytecode_context != crate::types::ROOT_CONTEXT
                    && frame.caller_script_ptr != 0
                    && frame.caller_script_len > 0
                {
                    // SAFETY: Stored from a live script slice at call time and valid for tx lifetime.
                    let script = unsafe {
                        core::slice::from_raw_parts(
                            frame.caller_script_ptr as *const u8,
                            frame.caller_script_len as usize,
                        )
                    };
                    ctx.set_script(script);
                } else if frame.bytecode_context == crate::types::ROOT_CONTEXT {
                    // Compatibility fallback for old frames.
                    ctx.set_script(ctx.root_bytecode);
                } else {
                    let account = ctx.accounts.get_unchecked(frame.bytecode_context)?;
                    let data = unsafe { account.borrow_data_unchecked() };
                    const SCRIPT_ACCOUNT_HEADER_LEN: usize = 64;
                    if data.len() < SCRIPT_ACCOUNT_HEADER_LEN {
                        return Err(VMErrorCode::AccountDataEmpty);
                    }
                    ctx.set_script(&data[SCRIPT_ACCOUNT_HEADER_LEN..]);
                }
                ctx.current_context = frame.bytecode_context;
                ctx.set_temp_offset(frame.saved_temp_offset as usize);

                // Verify IP against restored script
                if ctx.ip() > ctx.script().len() {
                    return Err(VMErrorCode::InvalidInstructionPointer);
                }

                ctx.set_external_account_remap(frame.account_remap);
                ctx.parameters_mut().copy_from_slice(&frame.saved_parameters);
                ctx.restore_parameters(frame.param_start, frame.param_len); // Restore caller's parameters

            // RETURN_VALUE semantics: The return value remains on top of the stack
            // for the calling function to use (e.g., SET_LOCAL, arithmetic operations, etc.)
            } else {
                // Top-level return, halt the script

                // CRITICAL FIX: Capture return value before halting
                // This fixes the stack persistence issue after ExecutionManager refactoring
                if !ctx.is_empty() {
                    let return_value_ref = ctx.pop()?;
                    ctx.set_return_value(Some(return_value_ref));
                } else {
                    ctx.set_return_value(None);
                }

                ctx.set_halted(true);
            }
        }
        NOP => {
            // No operation - do nothing
            debug_log!("MitoVM: NOP - no operation");
        }
        BR_EQ_U8 => {
            // Fused compare-and-branch: compare top stack value with u8, jump if equal
            let compare_value = ctx.fetch_byte()?;
            let offset = ctx.fetch_u16()? as i32;
            let stack_value = ctx.pop()?;

            let is_equal = match stack_value {
                five_protocol::ValueRef::U8(val) => val == compare_value,
                five_protocol::ValueRef::U64(val) => val as u8 == compare_value,
                five_protocol::ValueRef::Bool(val) => (val as u8) == compare_value,
                _ => false, // Other types don't match u8
            };

            if is_equal {
                let new_ip = (ctx.ip() as i32 + offset) as usize;
                if new_ip >= ctx.script().len() {
                    return Err(VMErrorCode::InvalidInstructionPointer);
                }
                ctx.set_ip(new_ip);
                debug_log!("MitoVM: BR_EQ_U8 taken");
                debug_log!("Compare value: {}", compare_value as u32);
                debug_log!("Offset: {}", offset as u32);
                debug_log!("New IP: {}", new_ip as u32);
            } else {
                debug_log!("MitoVM: BR_EQ_U8 not taken");
                debug_log!("Compare value: {}", compare_value as u32);
            }
        }
        CMP_EQ_JUMP => {
            // Compare top stack value with u8 immediate and jump to absolute target when equal.
            let compare_value = ctx.fetch_byte()?;
            let target = ctx.fetch_u16()? as usize;
            let lhs = value_to_u64(ctx.pop()?).ok_or(VMErrorCode::TypeMismatch)?;
            if lhs == compare_value as u64 {
                validate_and_jump(ctx, target)?;
            }
        }
        DEC_JUMP_NZ => {
            // Decrement top stack value in place and jump when result is non-zero.
            let target = ctx.fetch_u16()? as usize;
            let current = value_to_u64(ctx.pop()?).ok_or(VMErrorCode::TypeMismatch)?;
            let next = current.wrapping_sub(1);
            ctx.push(five_protocol::ValueRef::U64(next))?;
            if next != 0 {
                validate_and_jump(ctx, target)?;
            }
        }
        DEC_LOCAL_JUMP_NZ => {
            // Decrement local[index] and jump to target when new value is non-zero.
            let local_index = ctx.fetch_byte()?;
            let target = ctx.fetch_u16()? as usize;
            let next = ctx.dec_local_u64(local_index)?;
            if next != 0 {
                validate_and_jump(ctx, target)?;
            }
        }
        _ => return Err(VMErrorCode::InvalidInstruction),
    }
    Ok(())
}
