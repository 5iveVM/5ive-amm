//! Arithmetic operations handler for MitoVM
//!
//! This module handles arithmetic operations including ADD, SUB, MUL, DIV, MOD, NEG
//! and comparison operations like GT, LT, EQ, GTE, LTE, NEQ.
//!
//! # Integer Overflow Behavior
//!
//! MitoVM uses **wrapping arithmetic** for ADD, SUB, MUL operations.
//! Checked arithmetic is supported via ADD_CHECKED, SUB_CHECKED, and MUL_CHECKED opcodes.

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

        // AccountRef vs Integer comparisons
        // Normal account refs should compare underlying field data.
        // Sentinel refs (idx 0/255) are used as Option/Result tags and should compare by tag.
        (ValueRef::AccountRef(account_idx, _), ValueRef::U64(b)) => {
            if account_idx == 0 || account_idx == 255 {
                Ok((account_idx as u64) == b)
            } else {
                let val_a = crate::utils::resolve_u64(a, ctx)?;
                Ok(val_a == b)
            }
        }
        (ValueRef::U64(a), ValueRef::AccountRef(account_idx, _)) => {
            if account_idx == 0 || account_idx == 255 {
                Ok(a == (account_idx as u64))
            } else {
                let val_b = crate::utils::resolve_u64(b, ctx)?;
                Ok(a == val_b)
            }
        }
        (ValueRef::AccountRef(account_idx, _), ValueRef::U8(b)) => {
            if account_idx == 0 || account_idx == 255 {
                Ok(account_idx == b)
            } else {
                let val_a = crate::utils::resolve_u8(a, ctx)?;
                Ok(val_a == b)
            }
        }
        (ValueRef::U8(a), ValueRef::AccountRef(account_idx, _)) => {
            if account_idx == 0 || account_idx == 255 {
                Ok(a == account_idx)
            } else {
                let val_b = crate::utils::resolve_u8(b, ctx)?;
                Ok(a == val_b)
            }
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

        // Numeric fallback: compare coercible scalar values
        _ => {
            let a_num = crate::utils::ValueRefUtils::as_u64(a);
            let b_num = crate::utils::ValueRefUtils::as_u64(b);
            if let (Ok(av), Ok(bv)) = (a_num, b_num) {
                Ok(av == bv)
            } else {
                Ok(false)
            }
        }
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
/// # use five_vm_mito::{ValueRef, opcodes::ADD};
/// # use pinocchio::pubkey::Pubkey;
/// # let bytecode: &[u8] = &[0x11, 10, 0x11, 5, 0x20, 0x07];
/// # let mut storage = StackStorage::new();
/// # let mut ctx = ExecutionManager::new(
/// #     bytecode,
/// #     &[],
/// #     Pubkey::default(),
/// #     &[],
/// #     0,
/// #     &mut storage,
/// #     1,
/// #     1,
/// #     0,
/// #     0,
/// #     0,
/// #     0,
/// # );
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
        MUL_DIV => {
            // MUL_DIV: Fused (a * b) / c for common DeFi math patterns.
            // Stack before: [..., a, b, c]
            // Stack after:  [..., (a*b)/c]
            let sp = ctx.stack.sp as usize;
            if sp >= 3 {
                let c_val = unsafe { *ctx.stack.stack.get_unchecked(sp - 1) };
                let b_val = unsafe { *ctx.stack.stack.get_unchecked(sp - 2) };
                let a_val = unsafe { *ctx.stack.stack.get_unchecked(sp - 3) };

                // Hot path: all operands are u64/u8 and stay in u64.
                let a_u64 = match a_val {
                    ValueRef::U64(v) => Some(v),
                    ValueRef::U8(v) => Some(v as u64),
                    _ => None,
                };
                let b_u64 = match b_val {
                    ValueRef::U64(v) => Some(v),
                    ValueRef::U8(v) => Some(v as u64),
                    _ => None,
                };
                let c_u64 = match c_val {
                    ValueRef::U64(v) => Some(v),
                    ValueRef::U8(v) => Some(v as u64),
                    _ => None,
                };

                if let (Some(a), Some(b), Some(c)) = (a_u64, b_u64, c_u64) {
                    if c == 0 {
                        return Err(VMErrorCode::DivisionByZero.into());
                    }
                    let result = a.wrapping_mul(b).wrapping_div(c);
                    unsafe {
                        *ctx.stack.stack.get_unchecked_mut(sp - 3) = ValueRef::U64(result);
                    }
                    ctx.stack.sp -= 2;
                    return Ok(());
                }

                // Slow path: any u128 participation promotes operation to u128.
                let a_u128 = match a_val {
                    ValueRef::U128(v) => Some(v),
                    ValueRef::U64(v) => Some(v as u128),
                    ValueRef::U8(v) => Some(v as u128),
                    _ => None,
                }
                .ok_or(VMErrorCode::TypeMismatch)?;
                let b_u128 = match b_val {
                    ValueRef::U128(v) => Some(v),
                    ValueRef::U64(v) => Some(v as u128),
                    ValueRef::U8(v) => Some(v as u128),
                    _ => None,
                }
                .ok_or(VMErrorCode::TypeMismatch)?;
                let c_u128 = match c_val {
                    ValueRef::U128(v) => Some(v),
                    ValueRef::U64(v) => Some(v as u128),
                    ValueRef::U8(v) => Some(v as u128),
                    _ => None,
                }
                .ok_or(VMErrorCode::TypeMismatch)?;

                if c_u128 == 0 {
                    return Err(VMErrorCode::DivisionByZero.into());
                }

                let result = a_u128.wrapping_mul(b_u128).wrapping_div(c_u128);
                unsafe {
                    *ctx.stack.stack.get_unchecked_mut(sp - 3) = ValueRef::U128(result);
                }
                ctx.stack.sp -= 2;
            } else {
                return Err(VMErrorCode::StackUnderflow.into());
            }
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
