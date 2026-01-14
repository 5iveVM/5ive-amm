//! Logical operations handler for MitoVM
//!
//! This module handles logical operations including AND, OR, NOT, XOR, and BITWISE_NOT.
//! These operations work on boolean values and bitwise operations on u64 values.

use crate::{
    context::ExecutionManager,
    debug_log,
    error::{CompactResult, VMErrorCode},
};
use five_protocol::{opcodes::*, ValueRef};

macro_rules! bitwise_op {
    ($ctx:expr, $op_name:expr, $op:tt) => {{
        let b = $ctx.pop()?.as_u64().ok_or(VMErrorCode::TypeMismatch)?;
        let a = $ctx.pop()?.as_u64().ok_or(VMErrorCode::TypeMismatch)?;
        let result = a $op b;
        debug_log!("MitoVM: {} {} {} {} = {}", $op_name, a, stringify!($op), b, result);
        $ctx.push(ValueRef::U64(result))?;
    }};
}

macro_rules! shift_op {
    ($ctx:expr, $op_name:expr, $op:tt) => {{
        let shift_amount = $ctx.pop()?.as_u64().ok_or(VMErrorCode::TypeMismatch)?;
        let value = $ctx.pop()?.as_u64().ok_or(VMErrorCode::TypeMismatch)?;
        // Limit shift amount to prevent undefined behavior
        let safe_shift = (shift_amount % 64) as u32;
        let result = value $op safe_shift;
        debug_log!(
            "MitoVM: {} {} {} {} = {}",
            $op_name,
            value,
            stringify!($op),
            safe_shift,
            result
        );
        $ctx.push(ValueRef::U64(result))?;
    }};
}

macro_rules! rotate_op {
    ($ctx:expr, $op_name:expr, $method:ident) => {{
        let rotate_amount = $ctx.pop()?.as_u64().ok_or(VMErrorCode::TypeMismatch)?;
        let value = $ctx.pop()?.as_u64().ok_or(VMErrorCode::TypeMismatch)?;
        // Rotate amount modulo 64 for circular rotation
        let safe_rotate = (rotate_amount % 64) as u32;
        let result = value.$method(safe_rotate);
        debug_log!(
            "MitoVM: {} {} {} {} = {}",
            $op_name,
            value,
            stringify!($method),
            safe_rotate,
            result
        );
        $ctx.push(ValueRef::U64(result))?;
    }};
}

/// Handle logical operations (0x30-0x3F)
#[inline(never)]
pub fn handle_logical(opcode: u8, ctx: &mut ExecutionManager) -> CompactResult<()> {
    match opcode {
        AND => {
            let b_val = ctx.pop()?;
            let b = crate::utils::resolve_bool(b_val, ctx)?;
            let a_val = ctx.pop()?;
            let a = crate::utils::resolve_bool(a_val, ctx)?;
            ctx.push(ValueRef::Bool(a && b))?;
        }
        OR => {
            let b_val = ctx.pop()?;
            let b = crate::utils::resolve_bool(b_val, ctx)?;
            let a_val = ctx.pop()?;
            let a = crate::utils::resolve_bool(a_val, ctx)?;
            ctx.push(ValueRef::Bool(a || b))?;
        }
        NOT => {
            let a_val = ctx.pop()?;
            let a = crate::utils::resolve_bool(a_val, ctx)?;
            ctx.push(ValueRef::Bool(!a))?;
        }
        XOR => {
            let b_val = ctx.pop()?;
            let b = crate::utils::resolve_bool(b_val, ctx)?;
            let a_val = ctx.pop()?;
            let a = crate::utils::resolve_bool(a_val, ctx)?;
            ctx.push(ValueRef::Bool(a ^ b))?;
        }
        BITWISE_NOT => {
            // MitoVM BITWISE_NOT: Bitwise complement (~value) flipping all bits
            // Safety: Pure bitwise operation with no overflow risk
            // Edge cases: ~0 = 0xFFFFFFFFFFFFFFFF, ~0xFFFFFFFFFFFFFFFF = 0
            let a = ctx.pop()?.as_u64().ok_or(VMErrorCode::TypeMismatch)?;

            debug_log!("MitoVM: BITWISE_NOT input={}", a);

            // Bitwise NOT operation: flip all 64 bits
            let result = !a;

            debug_log!("MitoVM: BITWISE_NOT result={}", result);

            ctx.push(ValueRef::U64(result))?;
        }

        // ===== BITWISE OPERATIONS =====
        BITWISE_AND => bitwise_op!(ctx, "BITWISE_AND", &),
        BITWISE_OR => bitwise_op!(ctx, "BITWISE_OR", |),
        BITWISE_XOR => bitwise_op!(ctx, "BITWISE_XOR", ^),

        // ===== SHIFT OPERATIONS =====
        SHIFT_LEFT => shift_op!(ctx, "SHIFT_LEFT", <<),
        SHIFT_RIGHT => shift_op!(ctx, "SHIFT_RIGHT", >>),

        SHIFT_RIGHT_ARITH => {
            let shift_amount = ctx.pop()?.as_u64().ok_or(VMErrorCode::TypeMismatch)?;
            let value = ctx.pop()?;

            // Limit shift amount to prevent undefined behavior
            let safe_shift = (shift_amount % 64) as u32;

            // Handle both U64 and I64 types for arithmetic shift
            match value {
                ValueRef::U64(v) => {
                    // Arithmetic right shift on U64 interpreted as I64 preserves sign bit
                    let result = ((v as i64) >> safe_shift) as u64;
                    debug_log!(
                        "MitoVM: SHIFT_RIGHT_ARITH(U64) {} >> {} = {}",
                        v,
                        safe_shift,
                        result
                    );
                    ctx.push(ValueRef::U64(result))?;
                }
                ValueRef::I64(v) => {
                    // Direct arithmetic right shift on I64
                    let result = v >> safe_shift;
                    debug_log!(
                        "MitoVM: SHIFT_RIGHT_ARITH(I64) {} >> {} = {}",
                        v,
                        safe_shift,
                        result
                    );
                    ctx.push(ValueRef::I64(result))?;
                }
                _ => return Err(VMErrorCode::TypeMismatch),
            }
        }

        // ===== ROTATE OPERATIONS =====
        ROTATE_LEFT => rotate_op!(ctx, "ROTATE_LEFT", rotate_left),
        ROTATE_RIGHT => rotate_op!(ctx, "ROTATE_RIGHT", rotate_right),

        // ===== BYTE MANIPULATION OPERATIONS =====
        BYTE_SWAP_16 => {
            let value = ctx.pop()?.as_u64().ok_or(VMErrorCode::TypeMismatch)?;
            // Swap bytes in lower 16 bits, preserve upper bits
            let result = (value & 0xFFFFFFFFFFFF0000) | ((value as u16).swap_bytes() as u64);
            debug_log!("MitoVM: BYTE_SWAP_16 operation completed");
            ctx.push(ValueRef::U64(result))?;
        }
        BYTE_SWAP_32 => {
            let value = ctx.pop()?.as_u64().ok_or(VMErrorCode::TypeMismatch)?;
            // Swap bytes in lower 32 bits, preserve upper bits
            let result = (value & 0xFFFFFFFF00000000) | ((value as u32).swap_bytes() as u64);
            debug_log!("MitoVM: BYTE_SWAP_32 operation completed");
            ctx.push(ValueRef::U64(result))?;
        }
        BYTE_SWAP_64 => {
            let value = ctx.pop()?.as_u64().ok_or(VMErrorCode::TypeMismatch)?;
            // Swap all bytes in 64-bit value
            let result = value.swap_bytes();
            debug_log!("MitoVM: BYTE_SWAP_64 operation completed");
            ctx.push(ValueRef::U64(result))?;
        }
        _ => return Err(VMErrorCode::InvalidInstruction),
    }
    Ok(())
}
