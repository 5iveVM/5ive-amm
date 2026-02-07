//! Arithmetic operations handler for MitoVM
//!
//! This module handles arithmetic operations including ADD, SUB, MUL, DIV, MOD, NEG
//! and comparison operations like GT, LT, EQ, GTE, LTE, NEQ.
//!
//! # Integer Overflow Behavior
//!
//! **Current Implementation: Wrapping Arithmetic (Default)**
//!
//! MitoVM uses **wrapping arithmetic** for ADD, SUB, MUL operations (two's complement wraparound).
//! This means overflow is **silent** and wraps around the integer boundaries:
//!
//! ## Examples:
//! - `u64::MAX + 1` → `0` (wraps around)
//! - `0u64 - 1` → `u64::MAX` (wraps around)
//! - `u64::MAX * 2` → `u64::MAX - 1` (wraps around)
//!
//! ## Rationale:
//! - **Performance**: Zero overhead for performance-critical arithmetic
//! - **Predictability**: Deterministic behavior across all platforms
//! - **Consistency**: Matches low-level hardware behavior (two's complement)
//!
//! ## Safety Considerations:
//! - ⚠️ **Financial code**: Wrapping can cause silent errors in token amounts
//! - ⚠️ **Counter logic**: Loop counters may wrap unexpectedly
//! - ✅ **Hash operations**: Wrapping is desired for cryptographic operations
//! - ✅ **Bit manipulation**: Wrapping matches hardware behavior
//!
//! ## Future: Checked Arithmetic (Planned)
//!
//! To support safe financial calculations, checked arithmetic opcodes are planned:
//! - `ADD_CHECKED (0x2C)`: Returns error on overflow
//! - `SUB_CHECKED (0x2D)`: Returns error on underflow
//! - `MUL_CHECKED (0x2E)`: Returns error on overflow
//!
//! DSL developers will be able to choose explicitly:
//! ```rust,ignore
//! let fast = a + b;     // Wrapping (current, fast)
//! let safe = a +? b;    // Checked (planned, errors on overflow)
//! ```
//!
//! This gives developers **explicit control** over performance vs. safety tradeoffs.

use crate::{
    context::ExecutionManager,
    debug_log,
    error::{CompactResult, VMErrorCode},
    polymorphic_binary_op,
    polymorphic_binary_op_checked,
    polymorphic_binary_op_checked_overflow,
    polymorphic_comparison_op,
    // Import stack operation macros
    pop_u64,
    vm_push_bool,
    vm_push_u128,
    vm_push_u64,
};

#[cfg(feature = "debug-logs")]
#[allow(unused_imports)]
use crate::debug_stack_op;
use five_protocol::{opcodes::*, ValueRef};

/// Helper function to check equality between two values
/// Handles type promotion and special comparisons (AccountRef, PubkeyRef, etc.)
fn check_equality(a: ValueRef, b: ValueRef, ctx: &mut ExecutionManager) -> CompactResult<bool> {
    // Fast path: direct equality (works for U64, U8, I64, Bool, Empty if values match)
    if a == b {
        return Ok(true);
    }

    match (a, b) {
        // Numeric comparisons with promotion
        (ValueRef::U64(a), ValueRef::U128(b)) => Ok((a as u128) == b),
        (ValueRef::U128(a), ValueRef::U64(b)) => Ok(a == (b as u128)),

        // AccountRef data equality (fallback from fast path pointer equality)
        (ValueRef::AccountRef(_, _), ValueRef::AccountRef(_, _)) => {
            let val_a = crate::utils::resolve_u64(a, ctx)?;
            let val_b = crate::utils::resolve_u64(b, ctx)?;
            Ok(val_a == val_b)
        }

        // 32-byte Pubkey comparisons (Account vs Pubkey/Temp32)
        (ValueRef::AccountRef(_, _), ValueRef::PubkeyRef(_))
        | (ValueRef::PubkeyRef(_), ValueRef::AccountRef(_, _))
        | (ValueRef::AccountRef(_, _), ValueRef::TempRef(_, 32))
        | (ValueRef::TempRef(_, 32), ValueRef::AccountRef(_, _)) => {
            let pk_a = ctx.extract_pubkey(&a)?;
            let pk_b = ctx.extract_pubkey(&b)?;
            Ok(pk_a == pk_b)
        }

        // Account data vs Integer comparisons
        (ValueRef::AccountRef(_, _), ValueRef::U64(b)) => {
            Ok(crate::utils::resolve_u64(a, ctx)? == b)
        }
        (ValueRef::U64(a), ValueRef::AccountRef(_, _)) => {
            Ok(a == crate::utils::resolve_u64(b, ctx)?)
        }

        // Pubkey/Temp comparisons
        (ValueRef::PubkeyRef(_), _)
        | (_, ValueRef::PubkeyRef(_))
        | (ValueRef::TempRef(_, 32), _)
        | (_, ValueRef::TempRef(_, 32)) => {
            // Try pubkey extraction first
            if let (Ok(pk_a), Ok(pk_b)) = (ctx.extract_pubkey(&a), ctx.extract_pubkey(&b)) {
                Ok(pk_a == pk_b)
            } else {
                Ok(false) // Fallback: if not both pubkeys, and not equal (checked at start), then false
            }
        }

        // Default fallback
        _ => Ok(false),
    }
}

/// Execute polymorphic arithmetic and comparison operations with u128 support.
/// Handles the 0x20-0x2F opcode range with automatic type promotion.
/// u64×u64 operations maintain fast path, mixed operations promote to u128.
///
/// # Example Usage
/// ```rust
/// # use five_vm_mito::{*, StackStorage, ExecutionManager};
/// # use five_vm_mito::handlers::handle_arithmetic;
/// # use five_protocol::{ValueRef, opcodes::ADD};
/// # use pinocchio::pubkey::Pubkey;
/// # let bytecode: &[u8] = &[0x11, 10, 0x11, 5, 0x20, 0x07];
/// # let mut storage = StackStorage::new();
/// # let mut ctx = ExecutionManager::new(bytecode, &[], Pubkey::default(), &[], 0, &mut storage, 1, 1);
/// # ctx.push(ValueRef::U64(10)).unwrap();
/// # ctx.push(ValueRef::U128(5)).unwrap(); // Mixed types auto-promote
/// handle_arithmetic(ADD, &mut ctx)?;
/// let result = ctx.pop().map_err(|e| VMError::from(e))?;
/// assert_eq!(result, ValueRef::U128(15)); // Result is u128
/// # Ok::<(), VMError>(())
/// ```
#[inline(always)]
pub fn handle_arithmetic(opcode: u8, ctx: &mut ExecutionManager) -> CompactResult<()> {
    match opcode {
        ADD => {
            // Arithmetic Fast Paths (u64 optimization)
            // Peek directly into the stack to avoid pop overhead
            let sp = ctx.stack.sp as usize;
            if sp >= 2 {
                // Safety: Bounds checked by sp >= 2
                // Stack grows up: [..., A, B] where B is at sp-1
                let b_val = unsafe { *ctx.stack.stack.get_unchecked(sp - 1) };
                let a_val = unsafe { *ctx.stack.stack.get_unchecked(sp - 2) };
                
                if let (ValueRef::U64(a), ValueRef::U64(b)) = (a_val, b_val) {
                    // Fast path: u64 + u64
                    let result = a.wrapping_add(b);
                    // Update stack: Replace A (sp-2) with result
                    unsafe {
                        *ctx.stack.stack.get_unchecked_mut(sp - 2) = ValueRef::U64(result);
                    }
                    // Pop B (decrement sp)
                    ctx.stack.sp -= 1;
                } else {
                     // Slow path: promotion
                     polymorphic_binary_op!(ctx, "ADD", wrapping_add);
                }
            } else {
                return Err(VMErrorCode::StackUnderflow.into());
            }
        }
        SUB => {
            // SUB: Wrapping subtraction (underflow wraps around)
            let sp = ctx.stack.sp as usize;
            if sp >= 2 {
                let b_val = unsafe { *ctx.stack.stack.get_unchecked(sp - 1) };
                let a_val = unsafe { *ctx.stack.stack.get_unchecked(sp - 2) };
                
                if let (ValueRef::U64(a), ValueRef::U64(b)) = (a_val, b_val) {
                    let result = a.wrapping_sub(b);
                    unsafe {
                        *ctx.stack.stack.get_unchecked_mut(sp - 2) = ValueRef::U64(result);
                    }
                    ctx.stack.sp -= 1;
                } else {
                     polymorphic_binary_op!(ctx, "SUB", wrapping_sub);
                }
            } else {
                return Err(VMErrorCode::StackUnderflow.into());
            }
        }
        MUL => {
            // MUL: Wrapping multiplication (overflow wraps around)
            let sp = ctx.stack.sp as usize;
            if sp >= 2 {
                let b_val = unsafe { *ctx.stack.stack.get_unchecked(sp - 1) };
                let a_val = unsafe { *ctx.stack.stack.get_unchecked(sp - 2) };
                
                if let (ValueRef::U64(a), ValueRef::U64(b)) = (a_val, b_val) {
                    let result = a.wrapping_mul(b);
                    unsafe {
                        *ctx.stack.stack.get_unchecked_mut(sp - 2) = ValueRef::U64(result);
                    }
                    ctx.stack.sp -= 1;
                } else {
                     polymorphic_binary_op!(ctx, "MUL", wrapping_mul);
                }
            } else {
                return Err(VMErrorCode::StackUnderflow.into());
            }
        }
        DIV => {
            polymorphic_binary_op_checked!(ctx, "DIV", wrapping_div);
        }
        MOD => {
            polymorphic_binary_op_checked!(ctx, "MOD", wrapping_rem);
        }
        NEG => {
            // MitoVM NEG: Unary negation (-value) with two's complement arithmetic
            // Safety: Uses wrapping_neg() to handle overflow gracefully
            // Edge cases: NEG(0) = 0, NEG(i64::MIN) wraps to i64::MIN
            // debug_stack_op!("NEG", ctx);
            let a = pop_u64!(ctx);

            // Two's complement negation with explicit overflow handling
            // Cast to i64 for signed negation, then back to u64 for storage
            let result = (a as i64).wrapping_neg() as u64;

            vm_push_u64!(ctx, result);
        }
        ADD_CHECKED => {
            // ADD_CHECKED: Checked addition (errors on overflow)
            // Returns ArithmeticOverflow error instead of wrapping
            // Use this for financial calculations where overflow is a bug
            polymorphic_binary_op_checked_overflow!(ctx, "ADD_CHECKED", checked_add);
        }
        SUB_CHECKED => {
            // SUB_CHECKED: Checked subtraction (errors on underflow)
            // Returns ArithmeticOverflow error instead of wrapping
            // Use this for financial calculations where underflow is a bug
            polymorphic_binary_op_checked_overflow!(ctx, "SUB_CHECKED", checked_sub);
        }
        MUL_CHECKED => {
            // MUL_CHECKED: Checked multiplication (errors on overflow)
            // Returns ArithmeticOverflow error instead of wrapping
            // Use this for financial calculations where overflow is a bug
            polymorphic_binary_op_checked_overflow!(ctx, "MUL_CHECKED", checked_mul);
        }
        GT => {
            polymorphic_comparison_op!(ctx, "GT", >);
        }
        LT => {
            polymorphic_comparison_op!(ctx, "LT", <);
        }
        EQ => {
            // EQ comparison works on any ValueRef type, not just u64
            // debug_stack_op!("EQ", ctx);
            let b = ctx.pop()?;
            let a = ctx.pop()?;
            let result = check_equality(a, b, ctx)?;
            vm_push_bool!(ctx, result);
        }
        GTE => {
            polymorphic_comparison_op!(ctx, "GTE", >=);
        }
        LTE => {
            polymorphic_comparison_op!(ctx, "LTE", <=);
        }
        NEQ => {
            // NEQ comparison works on any ValueRef type, not just u64
            // debug_stack_op!("NEQ", ctx);
            let b = ctx.pop()?;
            let a = ctx.pop()?;
            let result = !check_equality(a, b, ctx)?;
            vm_push_bool!(ctx, result);
        }

        _ => return Err(VMErrorCode::InvalidInstruction.into()),
    }
    Ok(())
}
