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
                    // "MitoVM: JUMP_IF to offset {} (condition true)",
                    // offset as u16
                // );
            } else {
            }
        }
        JUMP_IF_NOT => {
            let offset = ctx.fetch_u16()? as usize;
            let condition = ctx.pop()?;
            if !condition.is_truthy() {
                validate_and_jump(ctx, offset)?;
                    // "MitoVM: JUMP_IF_NOT to offset {} (condition false)",
                    // offset as u16
                // );
            } else {
            }
        }
        REQUIRE => {
            check_condition(ctx)?;
        }
        ASSERT => {
            check_condition(ctx)?;
        }
        RETURN => {
                // "MitoVM: RETURN encountered - call depth: {}, stack size: {}",
                // ctx.call_depth() as u32,
                // ctx.size() as u32
            // );
                // "MitoVM: RETURN BEFORE - SP={}, local_base={}, local_count={}, IP={}",
                // ctx.size() as u32,
                // ctx.local_base() as u32,
                // ctx.local_count() as u32,
                // ctx.ip() as u32
            // );

            // Check if we're in a function call
            if ctx.call_depth() > 0 {
                debug_log!(
                    "MitoVM: RETURN from function - call depth: {}",
                    ctx.call_depth() as u32
                );

                // CRITICAL FIX (Issue 1.4): Use pop_call_frame() for atomic frame access + depth decrement
                // This prevents accessing wrong frame or out-of-bounds (same pattern as RETURN_VALUE)
                let frame = ctx.pop_call_frame()?;

                // Safety check: Validate return address
                if (frame.return_address as usize) >= ctx.script().len() {
                    debug_log!(
                        "MitoVM: ERROR - Invalid return address: {} (script length: {})",
                        frame.return_address as u32,
                        ctx.script().len() as u32
                    );
                    return Err(VMErrorCode::InvalidInstructionPointer);
                }

                // Restore previous state safely including local base offset
                ctx.set_ip(frame.return_address as usize);
                ctx.set_local_count(frame.local_count);
                ctx.set_local_base(frame.local_base); // Restore per-frame local window
                ctx.set_script(frame.bytecode);
                ctx.restore_parameters(frame.param_start, frame.param_len); // Restore caller's parameters

                    // "MitoVM: RETURN - returning to address: {}, depth: {}, params restored",
                    // ctx.ip() as u32,
                    // ctx.call_depth() as u32
                // );
                    // "MitoVM: RETURN AFTER - SP={}, local_base={}, local_count={}, IP={}",
                    // ctx.size() as u32,
                    // ctx.local_base() as u32,
                    // ctx.local_count() as u32,
                    // ctx.ip() as u32
                // );
            } else {
                // Top-level return, halt the script
                ctx.set_halted(true);
            }
        }
        RETURN_VALUE => {
            // PHASE 1 DEBUGGING: Confirm we've entered the RETURN_VALUE handler
                // "🔍 PHASE1_DEBUG: Successfully matched RETURN_VALUE case in control_flow handler"
            // );

                // "MitoVM: RETURN_VALUE encountered - call depth: {}, stack size: {}",
                // ctx.call_depth() as u32,
                // ctx.size() as u32
            // );

            // Enhanced debugging: Log current stack state before return
            // for _i in 0..ctx.size().min(5) {
            //     debug_log!("MitoVM: RETURN_VALUE stack[{}] = value present", _i as u32);
            // }

            // Safety check: Ensure there's a value on the stack to return
            if ctx.is_empty() {
                debug_log!(
                    "MitoVM: ERROR - RETURN_VALUE with empty stack - this will cause StackError"
                );
                debug_log!(
                    "MitoVM: RETURN_VALUE - function should have produced a return value on stack"
                );
                return Err(VMErrorCode::StackError);
            } else {
                    // "MitoVM: RETURN_VALUE - returning top stack value (stack size: {})",
                    // ctx.size() as u32
                // );
            }

            // Check if we're in a function call
            if ctx.call_depth() > 0 {
                    // "MitoVM: RETURN_VALUE from function - call depth: {}",
                    // ctx.call_depth() as u32
                // );

                // Enhanced debugging: Log current parameter state before restoration
                // for i in 0..8 {
                //     if !ctx.parameters()[i].is_empty() {
                //         debug_log!("MitoVM: RETURN_VALUE current parameters[{}] has value", i as u32);
                //     }
                // }

                // Safety check: Ensure call depth is valid
                if ctx.call_depth() == 0 {
                    debug_log!("MitoVM: ERROR - Call depth underflow detected");
                    return Err(VMErrorCode::CallStackUnderflow);
                }

                // CRITICAL FIX: The return value is already on the stack from the function execution
                // We do NOT need to pop it or modify it - just restore the call frame and let it remain
                    // "MitoVM: RETURN_VALUE keeping return value on stack during frame restoration"
                // );

                // Pop call stack safely using pop_call_frame instead of manual access
                let frame = ctx.pop_call_frame()?;
                let _new_call_depth = ctx.call_depth();

                // Safety check: Validate return address
                if (frame.return_address as usize) >= ctx.script().len() {
                    debug_log!(
                        "MitoVM: ERROR - Invalid return address: {} (script length: {})",
                        frame.return_address as u32,
                        ctx.script().len() as u32
                    );
                    return Err(VMErrorCode::InvalidInstructionPointer);
                }

                // Enhanced debugging: Log frame restoration details
                //     "MitoVM: RETURN_VALUE - return address: {}",
                //     frame.return_address as u32
                // );
                //     "MitoVM: RETURN_VALUE - local count: {}",
                //     frame.local_count as u32
                // );

                // Restore previous state safely - CRITICAL: Leave return value on stack untouched
                ctx.set_ip(frame.return_address as usize);
                ctx.set_local_count(frame.local_count);
                ctx.set_local_base(frame.local_base); // Restore per-frame local window
                ctx.set_script(frame.bytecode);
                ctx.restore_parameters(frame.param_start, frame.param_len); // Restore caller's parameters

                // RETURN_VALUE semantics: The return value remains on top of the stack
                // for the calling function to use (e.g., SET_LOCAL, arithmetic operations, etc.)

                // Enhanced debugging: Log restored parameter state
                // for i in 0..8 {
                //     if !ctx.parameters()[i].is_empty() {
                //         debug_log!(
                //             "MitoVM: RETURN_VALUE restored parameters[{}] has value",
                //             i as u32
                //         );
                //     }
                // }

                // Enhanced debugging: Log final stack state after return
                //     "MitoVM: RETURN_VALUE - stack size: {}",
                //     ctx.size() as u32
                // );
                // if !ctx.is_empty() {
                //     debug_log!("MitoVM: RETURN_VALUE - return value preserved on stack");
                // } else {
                //     debug_log!("MitoVM: RETURN_VALUE - WARNING: stack is empty after return");
                // }

                //     "MitoVM: RETURN_VALUE - returning to address: {}, depth: {}, params restored",
                //     ctx.ip() as u32,
                //     new_call_depth as u32
                // );

                // IMPORTANT: Do NOT consume the return value here even if new_call_depth == 0.
                // We may have just returned from an internal function back into the entry function.
                // The caller (entry function) still needs the return value to remain on the stack
                // for subsequent SET_LOCAL or arithmetic operations.
                // Top-level value capture happens only in the "else" branch below when halting.
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

                // PHASE 1 DEBUGGING: Track the ctx.set_halted(true) call execution
                ctx.set_halted(true);
                    // "🔍 PHASE1_DEBUG: ctx.set_halted(true) completed - VM should be halted now"
                // );
            }

            // PHASE 1 DEBUGGING: Confirm RETURN_VALUE handler completion
            /*
            debug_log!(
                "🔍 PHASE1_DEBUG: Final VM state - IP: {}, stack: {}, halted: {}",
                ctx.ip() as u32,
                ctx.size() as u32,
                ctx.halted() as u8
            );
            */
        }
        NOP => {
            // No operation - do nothing
            debug_log!("MitoVM: NOP - no operation");
        }
        BR_EQ_U8 => {
            // Fused compare-and-branch: compare top stack value with u8, jump if equal
            let compare_value = ctx.fetch_byte()?;
            let offset = ctx.fetch_vle_u16()? as i32;
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
        _ => return Err(VMErrorCode::InvalidInstruction),
    }
    Ok(())
}
