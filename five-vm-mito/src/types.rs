//! Type definitions for MitoVM.

use crate::MAX_LOCALS;
use five_protocol::ValueRef;

/// Stack-allocated local variable storage optimized for minimal memory usage.
/// Each slot holds a single [`ValueRef`]; [`ValueRef::Empty`] marks uninitialized locals.
pub type LocalVariables = [core::mem::MaybeUninit<ValueRef>; MAX_LOCALS];

/// Root context identifier for bytecode (original script).
pub const ROOT_CONTEXT: u8 = u8::MAX;

/// Function call frame containing return state and saved parameters.
///
/// # Example
/// ```rust
/// use five_vm_mito::{CallFrame, types::ROOT_CONTEXT};
///
/// let frame = CallFrame::new(100, 2, 0, ROOT_CONTEXT);
/// assert_eq!(frame.return_address, 100);
/// assert_eq!(frame.local_count, 2);
/// assert_eq!(frame.bytecode_context, ROOT_CONTEXT);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct CallFrame {
    pub return_address: u16,
    pub local_count: u8,
    pub local_base: u8,     // Base offset for this frame's locals in shared array
    pub param_start: u8,    // Start index of caller parameters in shared array
    pub param_len: u8,      // Length of caller parameter slice
    pub bytecode_context: u8, // Context identifier: u8::MAX = Root, otherwise account index
}

impl CallFrame {
    /// Create call frame with return address and local variable count.
    pub fn new(return_address: u16, local_count: u8, local_base: u8, bytecode_context: u8) -> Self {
        Self {
            return_address,
            local_count,
            local_base,
            param_start: 0,
            param_len: 0,
            bytecode_context,
        }
    }

    /// Create call frame with saved caller parameters for restoration on return.
    pub fn with_parameters(
        return_address: u16,
        local_count: u8,
        local_base: u8,
        param_start: u8,
        param_len: u8,
        bytecode_context: u8,
    ) -> Self {
        Self {
            return_address,
            local_count,
            local_base,
            param_start,
            param_len,
            bytecode_context,
        }
    }
}
