//! Type definitions for MitoVM.

use crate::{MAX_LOCALS, MAX_PARAMETERS};
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
    pub account_remap: [u8; MAX_PARAMETERS + 1], // External-call account remap snapshot
    pub caller_script_ptr: usize, // Raw pointer to caller bytecode slice for fast restore on RETURN
    pub caller_script_len: u32, // Length of caller bytecode slice
}

/// Fixed-size entry for transaction-local CALL_EXTERNAL resolution cache.
#[derive(Debug, Clone, Copy)]
pub struct ExternalCallCacheEntry {
    pub resolved_account_index: u8,
    pub selector: u16,
    pub func_offset: u16,
    pub func_index: u8,
    pub required_account_count: u8,
    pub constraints: [u8; 16],
    pub code_fingerprint: u32,
    pub valid: bool,
}

impl ExternalCallCacheEntry {
    pub const fn empty() -> Self {
        Self {
            resolved_account_index: u8::MAX,
            selector: 0,
            func_offset: 0,
            func_index: u8::MAX,
            required_account_count: 0,
            constraints: [0u8; 16],
            code_fingerprint: 0,
            valid: false,
        }
    }
}

/// Fixed-size entry for per-account import authorization cache.
#[derive(Debug, Clone, Copy)]
pub struct ExternalImportVerifyCacheEntry {
    pub resolved_account_index: u8,
    pub code_fingerprint: u32,
    pub authorized: bool,
    pub valid: bool,
}

impl ExternalImportVerifyCacheEntry {
    pub const fn empty() -> Self {
        Self {
            resolved_account_index: u8::MAX,
            code_fingerprint: 0,
            authorized: false,
            valid: false,
        }
    }
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
            account_remap: [u8::MAX; MAX_PARAMETERS + 1],
            caller_script_ptr: 0,
            caller_script_len: 0,
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
        account_remap: [u8; MAX_PARAMETERS + 1],
        caller_script_ptr: usize,
        caller_script_len: u32,
    ) -> Self {
        Self {
            return_address,
            local_count,
            local_base,
            param_start,
            param_len,
            bytecode_context,
            account_remap,
            caller_script_ptr,
            caller_script_len,
        }
    }
}
