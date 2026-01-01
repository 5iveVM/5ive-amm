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

/// Handle logical operations (0x30-0x3F)
#[inline(never)]
pub fn handle_logical(opcode: u8, ctx: &mut ExecutionManager) -> CompactResult<()> {
    match opcode {
        AND => {
            let b = ctx.pop()?.as_bool().ok_or(VMErrorCode::TypeMismatch)?;
            let a = ctx.pop()?.as_bool().ok_or(VMErrorCode::TypeMismatch)?;
            ctx.push(ValueRef::Bool(a && b))?;
        }
        OR => {
            let b = ctx.pop()?.as_bool().ok_or(VMErrorCode::TypeMismatch)?;
            let a = ctx.pop()?.as_bool().ok_or(VMErrorCode::TypeMismatch)?;
            ctx.push(ValueRef::Bool(a || b))?;
        }
        NOT => {
            let a = ctx.pop()?.as_bool().ok_or(VMErrorCode::TypeMismatch)?;
            ctx.push(ValueRef::Bool(!a))?;
        }
        XOR => {
            let b = ctx.pop()?.as_bool().ok_or(VMErrorCode::TypeMismatch)?;
            let a = ctx.pop()?.as_bool().ok_or(VMErrorCode::TypeMismatch)?;
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
        BITWISE_AND => {
            let b = ctx.pop()?.as_u64().ok_or(VMErrorCode::TypeMismatch)?;
            let a = ctx.pop()?.as_u64().ok_or(VMErrorCode::TypeMismatch)?;
            let result = a & b;
            debug_log!("MitoVM: BITWISE_AND {} & {} = {}", a, b, result);
            ctx.push(ValueRef::U64(result))?;
        }
        BITWISE_OR => {
            let b = ctx.pop()?.as_u64().ok_or(VMErrorCode::TypeMismatch)?;
            let a = ctx.pop()?.as_u64().ok_or(VMErrorCode::TypeMismatch)?;
            let result = a | b;
            debug_log!("MitoVM: BITWISE_OR {} | {} = {}", a, b, result);
            ctx.push(ValueRef::U64(result))?;
        }
        BITWISE_XOR => {
            let b = ctx.pop()?.as_u64().ok_or(VMErrorCode::TypeMismatch)?;
            let a = ctx.pop()?.as_u64().ok_or(VMErrorCode::TypeMismatch)?;
            let result = a ^ b;
            debug_log!("MitoVM: BITWISE_XOR {} ^ {} = {}", a, b, result);
            ctx.push(ValueRef::U64(result))?;
        }

        // ===== SHIFT OPERATIONS =====
        SHIFT_LEFT => {
            let shift_amount = ctx.pop()?.as_u64().ok_or(VMErrorCode::TypeMismatch)?;
            let value = ctx.pop()?.as_u64().ok_or(VMErrorCode::TypeMismatch)?;
            // Limit shift amount to prevent undefined behavior
            let safe_shift = (shift_amount % 64) as u32;
            let result = value << safe_shift;
            debug_log!(
                "MitoVM: SHIFT_LEFT {} << {} = {}",
                value,
                safe_shift,
                result
            );
            ctx.push(ValueRef::U64(result))?;
        }
        SHIFT_RIGHT => {
            let shift_amount = ctx.pop()?.as_u64().ok_or(VMErrorCode::TypeMismatch)?;
            let value = ctx.pop()?.as_u64().ok_or(VMErrorCode::TypeMismatch)?;
            // Limit shift amount to prevent undefined behavior
            let safe_shift = (shift_amount % 64) as u32;
            let result = value >> safe_shift;
            debug_log!(
                "MitoVM: SHIFT_RIGHT {} >> {} = {}",
                value,
                safe_shift,
                result
            );
            ctx.push(ValueRef::U64(result))?;
        }
        SHIFT_RIGHT_ARITH => {
            let shift_amount = ctx.pop()?.as_u64().ok_or(VMErrorCode::TypeMismatch)?;
            let value = ctx.pop()?.as_u64().ok_or(VMErrorCode::TypeMismatch)?;
            // Limit shift amount to prevent undefined behavior
            let safe_shift = (shift_amount % 64) as u32;
            // Arithmetic right shift preserves sign bit
            let result = ((value as i64) >> safe_shift) as u64;
            debug_log!(
                "MitoVM: SHIFT_RIGHT_ARITH {} >> {} = {}",
                value,
                safe_shift,
                result
            );
            ctx.push(ValueRef::U64(result))?;
        }

        // ===== ROTATE OPERATIONS =====
        ROTATE_LEFT => {
            let rotate_amount = ctx.pop()?.as_u64().ok_or(VMErrorCode::TypeMismatch)?;
            let value = ctx.pop()?.as_u64().ok_or(VMErrorCode::TypeMismatch)?;
            // Rotate amount modulo 64 for circular rotation
            let safe_rotate = (rotate_amount % 64) as u32;
            let result = value.rotate_left(safe_rotate);
            debug_log!(
                "MitoVM: ROTATE_LEFT {} <<< {} = {}",
                value,
                safe_rotate,
                result
            );
            ctx.push(ValueRef::U64(result))?;
        }
        ROTATE_RIGHT => {
            let rotate_amount = ctx.pop()?.as_u64().ok_or(VMErrorCode::TypeMismatch)?;
            let value = ctx.pop()?.as_u64().ok_or(VMErrorCode::TypeMismatch)?;
            // Rotate amount modulo 64 for circular rotation
            let safe_rotate = (rotate_amount % 64) as u32;
            let result = value.rotate_right(safe_rotate);
            debug_log!(
                "MitoVM: ROTATE_RIGHT {} >>> {} = {}",
                value,
                safe_rotate,
                result
            );
            ctx.push(ValueRef::U64(result))?;
        }

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
