//! Pattern Fusion Opcode Handlers (0xE0-0xEF)
//!
//! Handles optimized fused instructions that combine multiple primitive
//! operations into single instructions for bytecode size reduction and
//! execution efficiency.

use crate::{
    context::ExecutionManager,
    debug_log,
    error::{CompactResult, VMErrorCode},
    vm_push_u64,
    push_i64, // Added this import
};
use five_protocol::{opcodes::*, ValueRef};

/// Handle fusion opcodes (0xE0-0xEF)
#[inline(never)]
pub fn handle_fusion_ops(opcode: u8, ctx: &mut ExecutionManager) -> CompactResult<()> {
    match opcode {
        // Constants (0xE0-0xE1)
        PUSH_ZERO => {
            vm_push_u64!(ctx, 0);
        }
        PUSH_ONE => {
            vm_push_u64!(ctx, 1);
        }

        // Arithmetic Fusion (0xE2-0xE4)
        DUP_ADD => {
            // x -> x+x (equivalent to x * 2)
            // Implementation: DUP then ADD
            // Optimized: peek top, double it, replace top

            // Check stack depth
            if ctx.is_empty() {
                return Err(VMErrorCode::StackUnderflow);
            }

            // We can optimize by peeking and modifying in place for simple types
            // but `ExecutionManager` abstraction encourages pop/push for safety
            let val = ctx.pop()?;
            match val {
                ValueRef::U64(v) => {
                    // Check overflow
                    let (res, overflow) = v.overflowing_add(v);
                    if overflow {
                        return Err(VMErrorCode::ArithmeticOverflow);
                    }
                    vm_push_u64!(ctx, res);
                },
                ValueRef::I64(v) => {
                    let (res, overflow) = v.overflowing_add(v);
                    if overflow {
                        return Err(VMErrorCode::ArithmeticOverflow);
                    }
                    push_i64!(ctx, res);
                },
                ValueRef::U8(v) => {
                    let (res, overflow) = v.overflowing_add(v);
                    if overflow {
                        return Err(VMErrorCode::ArithmeticOverflow);
                    }
                    ctx.push(ValueRef::U8(res))?;
                },
                // For other types, fallback to DUP + ADD logic
                _ => {
                    ctx.push(val)?; // Restore
                    ctx.dup()?;
                    crate::handlers::handle_arithmetic(ADD, ctx)?;
                }
            }
        }
        DUP_SUB => {
            // x -> x-x = 0
            // Optimized: pop, push 0 (of appropriate type)
            let val = ctx.pop()?;
             match val {
                ValueRef::U64(_) => vm_push_u64!(ctx, 0),
                ValueRef::I64(_) => push_i64!(ctx, 0),
                ValueRef::U8(_) => ctx.push(ValueRef::U8(0))?,
                _ => {
                    ctx.push(val)?;
                    ctx.dup()?;
                    crate::handlers::handle_arithmetic(SUB, ctx)?;
                }
            }
        }
        DUP_MUL => {
            // x -> x*x (square)
            let val = ctx.pop()?;
            match val {
                ValueRef::U64(v) => {
                    let (res, overflow) = v.overflowing_mul(v);
                    if overflow {
                        return Err(VMErrorCode::ArithmeticOverflow);
                    }
                    vm_push_u64!(ctx, res);
                },
                ValueRef::I64(v) => {
                    let (res, overflow) = v.overflowing_mul(v);
                    if overflow {
                        return Err(VMErrorCode::ArithmeticOverflow);
                    }
                    push_i64!(ctx, res);
                },
                _ => {
                    ctx.push(val)?;
                    ctx.dup()?;
                    crate::handlers::handle_arithmetic(MUL, ctx)?;
                }
            }
        }

        // Validation Fusion (0xE5-0xE7)
        VALIDATE_AMOUNT_NONZERO => {
            // amount > 0 + REQUIRE
            // Consumes stack top
            let val = ctx.pop()?;
            let is_positive = match val {
                ValueRef::U64(v) => v > 0,
                ValueRef::I64(v) => v > 0,
                ValueRef::U128(v) => v > 0,
                ValueRef::U8(v) => v > 0,
                _ => false,
            };

            if !is_positive {
                return Err(VMErrorCode::ConstraintViolation);
            }
        }
        VALIDATE_SUFFICIENT => {
            // balance >= amount + REQUIRE
            // Stack: [balance, amount] -> [] (pops both)
            // Expects balance (top-1) >= amount (top)

            let amount = ctx.pop()?;
            let balance = ctx.pop()?;

            let sufficient = match (balance, amount) {
                (ValueRef::U64(b), ValueRef::U64(a)) => b >= a,
                (ValueRef::I64(b), ValueRef::I64(a)) => b >= a,
                (ValueRef::U64(b), ValueRef::U8(a)) => b >= (a as u64),
                // Add more combinations as needed
                _ => return Err(VMErrorCode::TypeMismatch),
            };

            if !sufficient {
                return Err(VMErrorCode::ConstraintViolation);
            }
        }
        EQ_ZERO_JUMP => {
            // value == 0 ? jump : continue
            // Stack: [value] (pop)
            let offset = ctx.fetch_u16()? as usize;
            let val = ctx.pop()?;

            let is_zero = match val {
                ValueRef::U64(v) => v == 0,
                ValueRef::I64(v) => v == 0,
                ValueRef::U8(v) => v == 0,
                ValueRef::Bool(v) => !v,
                ValueRef::Empty => true,
                _ => false,
            };

            if is_zero {
                if offset >= ctx.script().len() {
                    return Err(VMErrorCode::InvalidInstructionPointer);
                }
                ctx.set_ip(offset);
            }
        }

        // Control Flow Fusion (0xEA-0xED)
        RETURN_SUCCESS => {
            // PUSH Ok(()), RETURN
            // Assuming Result::Ok with Empty/Unit
            // This is equivalent to: RESULT_OK (pops nothing/empty), RETURN
            // But RESULT_OK usually wraps the top of stack.
            // If RETURN_SUCCESS means "Success with no value", it should:
            // 1. Clear stack? No, caller expects value.
            // 2. Push Result::Ok(Empty)
            // 3. Return

            // To match RESULT_OK semantics which consumes value:
            // We'll push Empty, then call handle_advanced(RESULT_OK), then handle_control_flow(RETURN_VALUE)

            // Optimized implementation:
            ctx.push(ValueRef::Empty)?; // Push unit

            // Manually inline RESULT_OK logic
            let total_size = 1; // 1 byte tag (1=Ok) + 0 bytes empty
            let offset = ctx.alloc_temp(total_size as u8)?;
            ctx.temp_buffer_mut()[offset as usize] = 1; // Ok tag
            // Empty has size 0

            ctx.pop()?; // Pop unit
            ctx.push(ValueRef::ResultRef(offset, total_size as u8))?;

            crate::handlers::handle_control_flow(RETURN_VALUE, ctx)?;
        }
        RETURN_ERROR => {
            // PUSH Err(Empty/Default), RETURN
            // Usually errors carry a code.
            // If this op expects an error code on stack?
            // "return err() fusion"
            // Let's assume it consumes top of stack as error code

            crate::handlers::handle_advanced(RESULT_ERR, ctx)?;
            crate::handlers::handle_control_flow(RETURN_VALUE, ctx)?;
        }
        GT_ZERO_JUMP => {
             // value > 0 ? jump : continue
            let offset = ctx.fetch_u16()? as usize;
            let val = ctx.pop()?;

            let is_gt_zero = match val {
                ValueRef::U64(v) => v > 0,
                ValueRef::I64(v) => v > 0,
                ValueRef::U8(v) => v > 0,
                _ => false,
            };

            if is_gt_zero {
                 if offset >= ctx.script().len() {
                    return Err(VMErrorCode::InvalidInstructionPointer);
                }
                ctx.set_ip(offset);
            }
        }
        LT_ZERO_JUMP => {
             // value < 0 ? jump : continue
            let offset = ctx.fetch_u16()? as usize;
            let val = ctx.pop()?;

            let is_lt_zero = match val {
                ValueRef::I64(v) => v < 0,
                 // Unsigned can't be < 0
                _ => false,
            };
             if is_lt_zero {
                 if offset >= ctx.script().len() {
                    return Err(VMErrorCode::InvalidInstructionPointer);
                }
                ctx.set_ip(offset);
            }
        }

        // Transfer Fusion (0xE8-0xE9) - Stubbed for now as they require custom token logic or more context
        TRANSFER_DEBIT | TRANSFER_CREDIT => {
             // Placeholder implementation
             debug_log!("Fusion Transfer ops not fully implemented yet");
             let _idx = ctx.fetch_byte()?;
             let _amount = ctx.pop()?;
             // For now just success
        }

        _ => return Err(VMErrorCode::InvalidInstruction),
    }
    Ok(())
}
