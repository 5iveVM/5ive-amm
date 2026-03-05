//! Stack operation macros for MitoVM
//!
//! This module provides declarative macros to reduce boilerplate in stack operations
//! across Five VM handlers. The macros provide type-safe stack manipulation with
//! consistent error handling and debug logging.

/// Extract u64 from stack top with type validation and automatic error propagation.
/// Now supports narrowing from u128 with overflow detection for polymorphic arithmetic.
///
/// # Example
/// ```rust,ignore
/// # use five_vm_mito::*;
/// # use five_protocol::ValueRef;
/// # use five_vm_mito::StackStorage;
/// # let bytecode: &[u8] = &[b'5', b'I', b'V', b'E', 0, 0, 0, 0, 1, 1];
/// # let mut storage = StackStorage::new();
/// # let mut ctx = ExecutionManager::new(bytecode, &[], pinocchio::pubkey::Pubkey::default(), &[], 0, &mut storage, 1, 1);
/// # ctx.push(ValueRef::U64(42)).unwrap();
/// let value = pop_u64!(ctx);
/// assert_eq!(value, 42);
/// ```
#[macro_export]
macro_rules! pop_u64 {
    ($ctx:expr) => {
        $crate::utils::resolve_u64($ctx.pop()?, &$ctx)?
    };
}

/// Extract u8 from stack top with type validation and automatic error propagation.
#[macro_export]
macro_rules! pop_u8 {
    ($ctx:expr) => {
        match $ctx.pop()? {
            five_protocol::ValueRef::U8(val) => val,
            _ => return Err($crate::error::VMErrorCode::TypeMismatch.into()),
        }
    };
}

/// Extract signed 64-bit integer from stack with type validation.
#[macro_export]
macro_rules! pop_i64 {
    ($ctx:expr) => {
        match $ctx.pop()? {
            five_protocol::ValueRef::I64(val) => val,
            _ => return Err($crate::error::VMErrorCode::TypeMismatch.into()),
        }
    };
}

/// Extract u128 from stack top with type validation and automatic error propagation.
/// MITO-style: zero-copy, direct stack access, BPF-optimized.
#[macro_export]
macro_rules! pop_u128 {
    ($ctx:expr) => {
        match $ctx.pop()? {
            five_protocol::ValueRef::U128(val) => val,
            _ => return Err($crate::error::VMErrorCode::TypeMismatch.into()),
        }
    };
}

/// Extract boolean from stack top with type validation.
#[macro_export]
macro_rules! pop_bool {
    ($ctx:expr) => {
        $crate::utils::resolve_bool($ctx.pop()?, &$ctx)?
    };
}

/// Handle logical binary operations (AND, OR, XOR) with automatic boolean resolution
#[macro_export]
macro_rules! logical_binary_op {
    ($ctx:expr, $op_name:literal, $op:tt) => {{
        debug_log!($op_name);
        debug_log!("Stack before: {}", $ctx.len() as u32);
        let b = pop_bool!($ctx);
        let a = pop_bool!($ctx);
        let result = a $op b;
        debug_log!("Result: {}", if result { 1 } else { 0 });
        debug_log!("Stack after: {}", ($ctx.len() + 1) as u32);
        vm_push_bool!($ctx, result);
    }};
}

/// Push u64 value onto execution stack as ValueRef::U64.
///
/// # Example
/// ```rust,ignore
/// # use five_vm_mito::*;
/// # use five_vm_mito::StackStorage;
/// # let bytecode: &[u8] = &[b'5', b'I', b'V', b'E', 0, 0, 0, 0, 1, 1];
/// # let mut storage = StackStorage::new();
/// # let mut ctx = ExecutionManager::new(bytecode, &[], pinocchio::pubkey::Pubkey::default(), &[], 0, &mut storage, 1, 1);
/// vm_push_u64!(ctx, 42);
/// assert_eq!(ctx.size(), 1);
/// # Ok::<(), VMError>(())
/// ```
#[macro_export]
macro_rules! vm_push_u64 {
    ($ctx:expr, $val:expr) => {
        $ctx.push(five_protocol::ValueRef::U64($val))?
    };
}

/// Push u8 value onto execution stack as ValueRef::U8.
#[macro_export]
macro_rules! push_u8 {
    ($ctx:expr, $val:expr) => {
        $ctx.push(five_protocol::ValueRef::U8($val))?
    };
}

/// Push signed 64-bit integer onto execution stack.
#[macro_export]
macro_rules! push_i64 {
    ($ctx:expr, $val:expr) => {
        $ctx.push(five_protocol::ValueRef::I64($val))?
    };
}

/// Push boolean value onto execution stack.
#[macro_export]
macro_rules! vm_push_bool {
    ($ctx:expr, $val:expr) => {
        $ctx.push(five_protocol::ValueRef::Bool($val))?
    };
}

/// Push u128 value onto execution stack as ValueRef::U128.
/// MITO-style: zero-copy, direct stack push, BPF-optimized.
#[macro_export]
macro_rules! vm_push_u128 {
    ($ctx:expr, $val:expr) => {
        $ctx.push(five_protocol::ValueRef::U128($val))?
    };
}

/// Execute binary arithmetic with automatic stack management and debug output.
///
/// # Example
/// ```ignore
/// # use five_vm_mito::*;
/// # use five_protocol::ValueRef;
/// # use crate::StackStorage;
/// # let bytecode: &[u8] = &[];
/// # let mut storage = StackStorage::new();
/// # let mut ctx = ExecutionContext::new(bytecode, &[], Default::default(), &[], 0, &mut storage, 0, 0);
/// # ctx.push(ValueRef::U64(10)).unwrap(, 0, 0, 0, 0);
/// # ctx.push(ValueRef::U64(5)).unwrap(, 0, 0, 0, 0);
/// binary_op!(ctx, "ADD", saturating_add);
/// let result = pop_u64!(ctx);
/// assert_eq!(result, 15);
/// ```
#[macro_export]
macro_rules! binary_op {
    ($ctx:expr, $op_name:literal, $op:ident) => {{
        debug_log!($op_name);
        debug_log!("Stack before: {}", $ctx.len() as u32);
        let b = pop_u64!($ctx);
        let a = pop_u64!($ctx);
        let result = a.$op(b);
        debug_log!("Result: {}", result);
        debug_log!("Stack after: {}", ($ctx.len() + 1) as u32);
        vm_push_u64!($ctx, result);
    }};
}

/// Handle binary arithmetic operations with division by zero check
#[macro_export]
macro_rules! binary_op_checked {
    ($ctx:expr, $op_name:literal, $op:ident) => {{
        debug_log!($op_name);
        debug_log!("Stack before: {}", $ctx.len() as u32);
        let b = pop_u64!($ctx);
        let a = pop_u64!($ctx);
        if b == 0 {
            return Err($crate::error::VMErrorCode::DivisionByZero.into());
        }
        let result = a.$op(b);
        debug_log!("Result: {}", result);
        debug_log!("Stack after: {}", ($ctx.len() + 1) as u32);
        vm_push_u64!($ctx, result);
    }};
}

/// Handle unary operations on the top stack value
#[macro_export]
macro_rules! unary_op {
    ($ctx:expr, $op_name:literal, $op:ident) => {{
        debug_log!($op_name);
        debug_log!("Stack before: {}", $ctx.len() as u32);
        let a = pop_u64!($ctx);
        let result = a.$op();
        debug_log!("Result: {}", result);
        debug_log!("Stack after: {}", ($ctx.len() + 1) as u32);
        vm_push_u64!($ctx, result);
    }};
}

/// Handle comparison operations that return boolean results
#[macro_export]
macro_rules! comparison_op {
    ($ctx:expr, $op_name:literal, $op:tt) => {{
        debug_log!($op_name);
        debug_log!("Stack before: {}", $ctx.len() as u32);
        let b = pop_u64!($ctx);
        let a = pop_u64!($ctx);
        let result = a $op b;
        debug_log!("Result: {}", result as u32);
        debug_log!("Stack after: {}", ($ctx.len() + 1) as u32);
        vm_push_bool!($ctx, result);
    }};
}

/// Standardized debug logging for stack operations
#[macro_export]
macro_rules! debug_stack_op {
    ($op_name:literal, $ctx:expr) => {
        debug_log!($op_name);
        debug_log!("Stack size: {}", $ctx.len() as u32);
    };
    ($op_name:literal, $ctx:expr, $val:expr) => {
        debug_log!($op_name);
        debug_log!("Value: {}", $val);
        debug_log!("Stack size: {}", $ctx.len() as u32);
    };
}

/// Pop two values and push one - common pattern for binary operations
#[macro_export]
macro_rules! stack_binary_pattern {
    ($ctx:expr, $a_type:ident, $b_type:ident, $result_type:ident, $operation:expr) => {{
        let b = paste::paste! { [<pop_ $b_type>]!($ctx) };
        let a = paste::paste! { [<pop_ $a_type>]!($ctx) };
        let result = $operation;
        paste::paste! { [<push_ $result_type>]!($ctx, result) };
    }};
}

/// Pop one value and push one - common pattern for unary operations  
#[macro_export]
macro_rules! stack_unary_pattern {
    ($ctx:expr, $input_type:ident, $result_type:ident, $operation:expr) => {{
        let val = paste::paste! { [<pop_ $input_type>]!($ctx) };
        let result = $operation;
        paste::paste! { [<push_ $result_type>]!($ctx, result) };
    }};
}

/// Helper macro for saturating arithmetic operations
#[macro_export]
macro_rules! saturating_binary_op {
    ($ctx:expr, $op_name:literal, $op:ident) => {
        binary_op!($ctx, $op_name, $op)
    };
}

/// Helper macro for checked arithmetic operations (division, modulo)
#[macro_export]
macro_rules! checked_binary_op {
    ($ctx:expr, $op_name:literal, $op:ident) => {
        binary_op_checked!($ctx, $op_name, $op)
    };
}

// Feature-gated stack debug helpers to avoid runtime overhead in release builds
#[cfg(feature = "debug-logs")]
#[macro_export]
macro_rules! debug_stack_state {
    ($opcode:expr, $ctx:expr, $operation:literal) => {
        debug_log!("STACK_TRACE opcode execution");
        debug_log!("Stack size: {}", $ctx.len() as u32);
        if $ctx.len() > 0 {
            debug_log!("Stack has values");
        } else {
            debug_log!("Stack is empty");
        }
    };
}

#[cfg(not(feature = "debug-logs"))]
#[macro_export]
macro_rules! debug_stack_state {
    ($opcode:expr, $ctx:expr, $operation:literal) => {};
}

#[cfg(feature = "debug-logs")]
#[macro_export]
macro_rules! stack_error_context {
    ($opcode:expr, $ctx:expr, $operation:literal, $required:expr, $available:expr) => {
        debug_log!("STACK_ERROR: Operation failed");
        debug_log!("Available: {}", $available as u32);
        debug_log!("Required: {}", $required as u32);
        debug_log!("Call depth: {}", $ctx.call_depth() as u32);
        let _params_count = $ctx.parameters().iter().filter(|p| !p.is_empty()).count();
        debug_log!("Parameters: {}", _params_count as u32);
    };
}

#[cfg(not(feature = "debug-logs"))]
#[macro_export]
macro_rules! stack_error_context {
    ($opcode:expr, $ctx:expr, $operation:literal, $required:expr, $available:expr) => {};
}

/// Polymorphic binary arithmetic with automatic u128 promotion.
/// Maintains fast path for u64×u64 operations while supporting mixed-type arithmetic.
/// When any operand is u128, the result is promoted to u128.
/// U8 values are promoted to U64 for arithmetic operations.
#[macro_export]
macro_rules! dispatch_polymorphic_op {
    ($ctx:expr, $a:expr, $b:expr, $op_macro:path, $($args:tt)*) => {
        match ($a, $b) {
            // AccountRef support - read 8 bytes as u64
            (five_protocol::ValueRef::AccountRef(_, _), _) | (_, five_protocol::ValueRef::AccountRef(_, _)) => {
                let a_val = $crate::utils::resolve_u64($a, &$ctx)?;
                let b_val = $crate::utils::resolve_u64($b, &$ctx)?;
                $op_macro!(u64, a_val, b_val, $ctx, $($args)*)
            }
            // Fast path: u64 × u64 (unchanged performance)
            (five_protocol::ValueRef::U64(a_val), five_protocol::ValueRef::U64(b_val)) => {
                $op_macro!(u64, a_val, b_val, $ctx, $($args)*)
            }
            (five_protocol::ValueRef::U128(a_val), five_protocol::ValueRef::U128(b_val)) => {
                $op_macro!(u128, a_val, b_val, $ctx, $($args)*)
            }
            // Promotion paths: any u128 involvement → u128 result
            (five_protocol::ValueRef::U128(a_val), rhs) => {
                let b_promoted = match rhs {
                    five_protocol::ValueRef::U128(v) => v,
                    five_protocol::ValueRef::U64(v) => v as u128,
                    five_protocol::ValueRef::U32(v) => v as u128,
                    five_protocol::ValueRef::U16(v) => v as u128,
                    five_protocol::ValueRef::U8(v) => v as u128,
                    five_protocol::ValueRef::I8(v) if v >= 0 => v as u128,
                    five_protocol::ValueRef::I16(v) if v >= 0 => v as u128,
                    five_protocol::ValueRef::I32(v) if v >= 0 => v as u128,
                    five_protocol::ValueRef::I64(v) if v >= 0 => v as u128,
                    five_protocol::ValueRef::Bool(v) => if v { 1 } else { 0 },
                    _ => return Err($crate::error::VMErrorCode::TypeMismatch.into()),
                };
                $op_macro!(u128, a_val, b_promoted, $ctx, $($args)*)
            }
            (lhs, five_protocol::ValueRef::U128(b_val)) => {
                let a_promoted = match lhs {
                    five_protocol::ValueRef::U128(v) => v,
                    five_protocol::ValueRef::U64(v) => v as u128,
                    five_protocol::ValueRef::U32(v) => v as u128,
                    five_protocol::ValueRef::U16(v) => v as u128,
                    five_protocol::ValueRef::U8(v) => v as u128,
                    five_protocol::ValueRef::I8(v) if v >= 0 => v as u128,
                    five_protocol::ValueRef::I16(v) if v >= 0 => v as u128,
                    five_protocol::ValueRef::I32(v) if v >= 0 => v as u128,
                    five_protocol::ValueRef::I64(v) if v >= 0 => v as u128,
                    five_protocol::ValueRef::Bool(v) => if v { 1 } else { 0 },
                    _ => return Err($crate::error::VMErrorCode::TypeMismatch.into()),
                };
                $op_macro!(u128, a_promoted, b_val, $ctx, $($args)*)
            }
            // General scalar coercion path (u8/u16/u32/u64 and non-negative signed)
            (lhs, rhs) => {
                let a_val = $crate::utils::ValueRefUtils::as_u64(lhs)?;
                let b_val = $crate::utils::ValueRefUtils::as_u64(rhs)?;
                $op_macro!(u64, a_val, b_val, $ctx, $($args)*)
            }
        }
    }
}

// Implementations

#[macro_export]
macro_rules! wrapping_op_impl {
    (u64, $a:expr, $b:expr, $ctx:expr, $op_name:literal, $method:ident) => {{
        let result = $a.$method($b);
        debug_log!("Result (u64): {}", result);
        vm_push_u64!($ctx, result);
    }};
    (u128, $a:expr, $b:expr, $ctx:expr, $op_name:literal, $method:ident) => {{
        let result = $a.$method($b);
        debug_log!("Result (u128): {}", result);
        vm_push_u128!($ctx, result);
    }};
}

#[macro_export]
macro_rules! checked_op_impl {
    (u64, $a:expr, $b:expr, $ctx:expr, $op_name:literal, $method:ident) => {{
        if $b == 0 {
            return Err($crate::error::VMErrorCode::DivisionByZero.into());
        }
        let result = $a.$method($b);
        debug_log!("Result (u64): {}", result);
        vm_push_u64!($ctx, result);
    }};
    (u128, $a:expr, $b:expr, $ctx:expr, $op_name:literal, $method:ident) => {{
        if $b == 0 {
            return Err($crate::error::VMErrorCode::DivisionByZero.into());
        }
        let result = $a.$method($b);
        debug_log!("Result (u128): {}", result);
        vm_push_u128!($ctx, result);
    }};
}

#[macro_export]
macro_rules! checked_overflow_op_impl {
    (u64, $a:expr, $b:expr, $ctx:expr, $op_name:literal, $method:ident) => {{
        match $a.$method($b) {
            Some(result) => {
                debug_log!("Result (u64): {}", result);
                vm_push_u64!($ctx, result);
            }
            None => {
                let _unused = 0;
                #[cfg(feature = "debug-logs")]
                debug_log!("{} overflow detected: {} op {}", $op_name, $a, $b);
                return Err($crate::error::VMErrorCode::ArithmeticOverflow.into());
            }
        }
    }};
    (u128, $a:expr, $b:expr, $ctx:expr, $op_name:literal, $method:ident) => {{
        match $a.$method($b) {
            Some(result) => {
                debug_log!("Result (u128): {}", result);
                vm_push_u128!($ctx, result);
            }
            None => {
                let _unused = 0;
                #[cfg(feature = "debug-logs")]
                debug_log!("{} overflow detected: {} op {}", $op_name, $a, $b);
                return Err($crate::error::VMErrorCode::ArithmeticOverflow.into());
            }
        }
    }};
}

#[macro_export]
macro_rules! comparison_op_impl {
    ($type:ident, $a:expr, $b:expr, $ctx:expr, $op_name:literal, $op:tt) => {{
        let result = $a $op $b;
        debug_log!("Comp: {} {} {} result={}", $a, stringify!($op), $b, if result { 1 } else { 0 });
        vm_push_bool!($ctx, result);
    }};
}

// Rewritten polymorphic macros

#[macro_export]
macro_rules! polymorphic_binary_op {
    ($ctx:expr, $op_name:literal, $op:ident) => {{
        debug_log!($op_name);
        debug_log!("Stack before: {}", $ctx.len() as u32);
        let b = $ctx.pop()?;
        let a = $ctx.pop()?;
        $crate::dispatch_polymorphic_op!($ctx, a, b, $crate::wrapping_op_impl, $op_name, $op);
        debug_log!("Stack after: {}", $ctx.len() as u32);
    }};
}

#[macro_export]
macro_rules! polymorphic_binary_op_checked {
    ($ctx:expr, $op_name:literal, $op:ident) => {{
        debug_log!($op_name);
        debug_log!("Stack before: {}", $ctx.len() as u32);
        let b = $ctx.pop()?;
        let a = $ctx.pop()?;
        $crate::dispatch_polymorphic_op!($ctx, a, b, $crate::checked_op_impl, $op_name, $op);
        debug_log!("Stack after: {}", $ctx.len() as u32);
    }};
}

#[macro_export]
macro_rules! polymorphic_binary_op_checked_overflow {
    ($ctx:expr, $op_name:literal, $op:ident) => {{
        debug_log!($op_name);
        debug_log!("Stack before: {}", $ctx.len() as u32);
        let b = $ctx.pop()?;
        let a = $ctx.pop()?;
        $crate::dispatch_polymorphic_op!(
            $ctx,
            a,
            b,
            $crate::checked_overflow_op_impl,
            $op_name,
            $op
        );
        debug_log!("Stack after: {}", $ctx.len() as u32);
    }};
}

#[macro_export]
macro_rules! polymorphic_comparison_op {
    ($ctx:expr, $op_name:literal, $op:tt) => {{
        debug_log!($op_name);
        debug_log!("Stack before: {}", $ctx.len() as u32);
        let b = $ctx.pop()?;
        let a = $ctx.pop()?;
        #[cfg(feature = "debug-logs")]
        {
            debug_log!("LTE DEBUG: Comparing types...");

            // Explicitly verify U64/U8 match
            if let (five_protocol::ValueRef::U64(av), five_protocol::ValueRef::U8(bv)) = (&a, &b) {
                debug_log!("LTE DEBUG: MATCHED U64({}) / U8({})", *av, *bv);
            } else {
                debug_log!("LTE DEBUG: DATA MISMATCH or OTHER TYPE");
                // Attempt to identify type of A (limited)
                if let five_protocol::ValueRef::U64(v) = &a {
                    debug_log!("  A is U64: {}", *v);
                } else if let five_protocol::ValueRef::U8(v) = &a {
                    debug_log!("  A is U8: {}", *v);
                } else {
                    debug_log!("  A is OTHER (Ref/Enum?)");
                }

                if let five_protocol::ValueRef::U64(v) = &b {
                    debug_log!("  B is U64: {}", *v);
                } else if let five_protocol::ValueRef::U8(v) = &b {
                    debug_log!("  B is U8: {}", *v);
                } else {
                    debug_log!("  B is OTHER");
                }
            }
        }
        $crate::dispatch_polymorphic_op!($ctx, a, b, $crate::comparison_op_impl, $op_name, $op);
        debug_log!("Stack after: {}", $ctx.len() as u32);
    }};
}

#[macro_export]
macro_rules! bitwise_op {
    ($ctx:expr, $op_name:expr, $op:tt) => {{
        let b_ref = $ctx.pop()?;
        let a_ref = $ctx.pop()?;
        let b = $crate::utils::resolve_u64(b_ref, &$ctx)?;
        let a = $crate::utils::resolve_u64(a_ref, &$ctx)?;
        let result = a $op b;
        debug_log!("MitoVM: {} {} {} {} = {}", $op_name, a, stringify!($op), b, result);
        $ctx.push(five_protocol::ValueRef::U64(result))?;
    }};
}

#[macro_export]
macro_rules! shift_op {
    ($ctx:expr, $op_name:expr, $op:tt) => {{
        let shift_amount = $ctx.pop()?.as_u64().ok_or($crate::error::VMErrorCode::TypeMismatch)?;
        let value = $ctx.pop()?.as_u64().ok_or($crate::error::VMErrorCode::TypeMismatch)?;
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
        $ctx.push(five_protocol::ValueRef::U64(result))?;
    }};
}

#[macro_export]
macro_rules! rotate_op {
    ($ctx:expr, $op_name:expr, $method:ident) => {{
        let rotate_amount = $ctx
            .pop()?
            .as_u64()
            .ok_or($crate::error::VMErrorCode::TypeMismatch)?;
        let value = $ctx
            .pop()?
            .as_u64()
            .ok_or($crate::error::VMErrorCode::TypeMismatch)?;
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
        $ctx.push(five_protocol::ValueRef::U64(result))?;
    }};
}

#[cfg(test)]
mod tests {
    use crate::{debug_log, error::CompactResult, stack::StackStorage, ExecutionContext, Pubkey};

    #[test]
    fn test_pop_push_macros() -> CompactResult<()> {
        let mut storage = StackStorage::new();
        let mut ctx = ExecutionContext::new(
            &[],
            &[],
            Pubkey::default(),
            &[],
            0,
            &mut storage,
            0,
            0,
            0,
            0,
            0,
            0,
        );

        vm_push_u64!(ctx, 42);
        push_u8!(ctx, 7);
        push_i64!(ctx, -3);
        vm_push_bool!(ctx, true);

        assert_eq!(ctx.len(), 4);

        assert!(pop_bool!(ctx));
        assert_eq!(pop_i64!(ctx), -3);
        assert_eq!(pop_u8!(ctx), 7);
        assert_eq!(pop_u64!(ctx), 42);

        assert_eq!(ctx.len(), 0);
        Ok(())
    }

    #[test]
    fn test_binary_op_macro() -> CompactResult<()> {
        let mut storage = StackStorage::new();
        let mut ctx = ExecutionContext::new(
            &[],
            &[],
            Pubkey::default(),
            &[],
            0,
            &mut storage,
            0,
            0,
            0,
            0,
            0,
            0,
        );

        vm_push_u64!(ctx, 10);
        vm_push_u64!(ctx, 5);

        binary_op!(ctx, "ADD", saturating_add);

        assert_eq!(ctx.len(), 1);
        assert_eq!(pop_u64!(ctx), 15);

        Ok(())
    }
}

// ===== END OF POLYMORPHIC MACROS =====
// Polymorphic arithmetic eliminates need for dedicated u128 opcodes
// All integer operations now support mixed u64/u128 with auto-promotion
