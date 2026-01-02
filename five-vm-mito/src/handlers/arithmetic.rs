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
/// # let mut storage = StackStorage::new(bytecode);
/// # let mut ctx = ExecutionManager::new(bytecode, &[], Pubkey::default(), &[], 0, &mut storage, 1, 1);
/// # ctx.push(ValueRef::U64(10)).unwrap();
/// # ctx.push(ValueRef::U128(5)).unwrap(); // Mixed types auto-promote
/// handle_arithmetic(ADD, &mut ctx)?;
/// let result = ctx.pop().map_err(|e| VMError::from(e))?;
/// assert_eq!(result, ValueRef::U128(15)); // Result is u128
/// # Ok::<(), VMError>(())
/// ```
pub fn handle_arithmetic(opcode: u8, ctx: &mut ExecutionManager) -> CompactResult<()> {
    match opcode {
        ADD => {
            // ADD: Wrapping addition (overflow wraps around)
            // Example: u64::MAX + 1 → 0
            // For checked addition that errors on overflow, use ADD_CHECKED (planned)
            polymorphic_binary_op!(ctx, "ADD", wrapping_add);
        }
        SUB => {
            // SUB: Wrapping subtraction (underflow wraps around)
            // Example: 0 - 1 → u64::MAX
            // For checked subtraction that errors on underflow, use SUB_CHECKED (planned)
            polymorphic_binary_op!(ctx, "SUB", wrapping_sub);
        }
        MUL => {
            // MUL: Wrapping multiplication (overflow wraps around)
            // Example: u64::MAX * 2 → u64::MAX - 1
            // For checked multiplication that errors on overflow, use MUL_CHECKED (planned)
            polymorphic_binary_op!(ctx, "MUL", wrapping_mul);
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
            let result = match (a, b) {
                (ValueRef::U64(a_val), ValueRef::U64(b_val)) => a_val == b_val,
                (ValueRef::U64(a_val), ValueRef::U128(b_val)) => (a_val as u128) == b_val,
                (ValueRef::U128(a_val), ValueRef::U64(b_val)) => a_val == (b_val as u128),
                (ValueRef::U128(a_val), ValueRef::U128(b_val)) => a_val == b_val,
                
                // Handle Pubkey comparison (TempRef or PubkeyRef) by value
                (ValueRef::TempRef(_, _), ValueRef::TempRef(_, _)) |
                (ValueRef::TempRef(_, _), ValueRef::PubkeyRef(_)) |
                (ValueRef::PubkeyRef(_), ValueRef::TempRef(_, _)) |
                (ValueRef::PubkeyRef(_), ValueRef::PubkeyRef(_)) => {
                     // Try to extract as pubkeys. If extraction fails (e.g. size != 32), fall back to exact match
                     if let (Ok(pk_a), Ok(pk_b)) = (ctx.extract_pubkey(&a), ctx.extract_pubkey(&b)) {
                         pk_a == pk_b
                     } else {
                         a == b
                     }
                },

                (left, right) => left == right,
            };
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
            let result = match (a, b) {
                (ValueRef::U64(a_val), ValueRef::U64(b_val)) => a_val != b_val,
                (ValueRef::U64(a_val), ValueRef::U128(b_val)) => (a_val as u128) != b_val,
                (ValueRef::U128(a_val), ValueRef::U64(b_val)) => a_val != (b_val as u128),
                (ValueRef::U128(a_val), ValueRef::U128(b_val)) => a_val != b_val,
                
                // Handle Pubkey comparison (TempRef or PubkeyRef) by value
                (ValueRef::TempRef(_, _), ValueRef::TempRef(_, _)) |
                (ValueRef::TempRef(_, _), ValueRef::PubkeyRef(_)) |
                (ValueRef::PubkeyRef(_), ValueRef::TempRef(_, _)) |
                (ValueRef::PubkeyRef(_), ValueRef::PubkeyRef(_)) => {
                     // Try to extract as pubkeys
                     if let (Ok(pk_a), Ok(pk_b)) = (ctx.extract_pubkey(&a), ctx.extract_pubkey(&b)) {
                         pk_a != pk_b
                     } else {
                         a != b
                     }
                },

                (left, right) => left != right,
            };
            vm_push_bool!(ctx, result);
        }

        _ => return Err(VMErrorCode::InvalidInstruction.into()),
    }
    Ok(())
}
