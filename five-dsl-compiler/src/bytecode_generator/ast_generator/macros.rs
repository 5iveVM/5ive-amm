//! Macros for reducing code repetition in AST generation
//!
//! This module provides macros to eliminate common patterns across the AST generator.

/// Validate function argument counts
///
/// # Examples
///
/// ```ignore
/// validate_args!(args, 1);  // Exactly 1 argument
/// validate_args!(args, 0);  // No arguments
/// validate_args!(args, max 2);  // At most 2 arguments
/// ```
#[macro_export]
macro_rules! validate_args {
    ($args:expr, $count:expr) => {
        if $args.len() != $count {
            return Err(VMError::InvalidParameterCount);
        }
    };
    ($args:expr, max $count:expr) => {
        if $args.len() > $count {
            return Err(VMError::InvalidParameterCount);
        }
    };
}

/// Emit a native syscall with argument validation
///
/// # Examples
///
/// ```ignore
/// emit_syscall!(emitter, args, 1, args empty);  // abort() - no args
/// emit_syscall!(emitter, args, 2, args max 1);  // panic(msg?) - 0 or 1 args
/// emit_syscall!(emitter, args, 80, args = 1);   // sha256(data) - exactly 1 arg
/// ```
#[macro_export]
macro_rules! emit_syscall {
    ($emitter:expr, $args:expr, $id:expr, args = $count:expr) => {{
        validate_args!($args, $count);
        $emitter.emit_opcode(CALL_NATIVE);
        $emitter.emit_u8($id);
    }};
    ($emitter:expr, $args:expr, $id:expr, args empty) => {{
        validate_args!($args, 0);
        $emitter.emit_opcode(CALL_NATIVE);
        $emitter.emit_u8($id);
    }};
    ($emitter:expr, $args:expr, $id:expr, args max $count:expr) => {{
        validate_args!($args, max $count);
        $emitter.emit_opcode(CALL_NATIVE);
        $emitter.emit_u8($id);
    }};
}

/// Try constant folding for binary operations on literals
///
/// # Examples
///
/// ```ignore
/// try_constant_fold!(self, emitter, left, right, wrapping_add);
/// try_constant_fold!(self, emitter, left, right, wrapping_sub);
/// ```
#[macro_export]
macro_rules! try_constant_fold {
    ($self:expr, $emitter:expr, $left:expr, $right:expr, $op:ident) => {{
        // Try u64 constant folding
        if let (AstNode::Literal(Value::U64(a)), AstNode::Literal(Value::U64(b))) = ($left, $right)
        {
            let result = a.$op(*b);
            $self.emit_literal_value($emitter, &Value::U64(result))?;
            return Ok(true);
        }
        // Try u128 constant folding
        if let (AstNode::Literal(Value::U128(a)), AstNode::Literal(Value::U128(b))) =
            ($left, $right)
        {
            let result = a.$op(*b);
            $self.emit_literal_value($emitter, &Value::U128(result))?;
            return Ok(true);
        }
    }};
}
