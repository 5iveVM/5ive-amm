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
    pub stack_sp: u8,
    pub local_count: u8,
    pub local_base: u8,  // Base offset for this frame's locals in shared array
    pub param_start: u8, // Start index of caller parameters in shared array
    pub param_len: u8,   // Length of caller parameter slice
    pub saved_temp_offset: u16, // Temp buffer cursor before entering callee
    pub saved_parameters: [ValueRef; MAX_PARAMETERS + 1], // Full caller parameter values
    pub bytecode_context: u8, // Context identifier: u8::MAX = Root, otherwise account index
    pub active_script_key: [u8; 32], // Active script account pubkey for authorization restore
    pub account_remap: [u8; MAX_PARAMETERS + 1], // External-call account remap snapshot
    pub caller_script_ptr: usize, // Raw pointer to caller bytecode slice for fast restore on RETURN
    pub caller_script_len: u32, // Length of caller bytecode slice
    pub caller_header_features: u32,
    pub caller_pool_offset: u32,
    pub caller_pool_slots: u16,
    pub caller_string_blob_offset: u32,
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

/// Aggregated transaction-local external call cache state.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct ExternalCacheState {
    pub external_call_cache: [ExternalCallCacheEntry; 32],
    pub external_call_cache_next: usize,
    pub external_import_verify_cache: [ExternalImportVerifyCacheEntry; 16],
    pub external_import_verify_cache_next: usize,
    pub external_hot_account_index: u8,
    pub external_hot_script_ptr: usize,
    pub external_hot_script_len: u32,
    pub external_hot_code_fingerprint: u32,
    pub external_hot_import_authorized: bool,
    pub external_hot_valid: bool,
    pub external_cache_hits: u32,
    pub external_cache_misses: u32,
    pub import_verify_cache_hits: u32,
}

impl ExternalCacheState {
    pub const fn empty() -> Self {
        Self {
            external_call_cache: [ExternalCallCacheEntry::empty(); 32],
            external_call_cache_next: 0,
            external_import_verify_cache: [ExternalImportVerifyCacheEntry::empty(); 16],
            external_import_verify_cache_next: 0,
            external_hot_account_index: u8::MAX,
            external_hot_script_ptr: 0,
            external_hot_script_len: 0,
            external_hot_code_fingerprint: 0,
            external_hot_import_authorized: false,
            external_hot_valid: false,
            external_cache_hits: 0,
            external_cache_misses: 0,
            import_verify_cache_hits: 0,
        }
    }
}

impl CallFrame {
    /// Create call frame with return address and local variable count.
    pub fn new(return_address: u16, local_count: u8, local_base: u8, bytecode_context: u8) -> Self {
        Self {
            return_address,
            stack_sp: 0,
            local_count,
            local_base,
            param_start: 0,
            param_len: 0,
            saved_temp_offset: 0,
            saved_parameters: [ValueRef::Empty; MAX_PARAMETERS + 1],
            bytecode_context,
            active_script_key: [0u8; 32],
            account_remap: [u8::MAX; MAX_PARAMETERS + 1],
            caller_script_ptr: 0,
            caller_script_len: 0,
            caller_header_features: 0,
            caller_pool_offset: 0,
            caller_pool_slots: 0,
            caller_string_blob_offset: 0,
        }
    }

    /// Create call frame with saved caller parameters for restoration on return.
    pub fn with_parameters(
        return_address: u16,
        stack_sp: u8,
        local_count: u8,
        local_base: u8,
        param_start: u8,
        param_len: u8,
        saved_temp_offset: u16,
        saved_parameters: [ValueRef; MAX_PARAMETERS + 1],
        bytecode_context: u8,
        active_script_key: [u8; 32],
        account_remap: [u8; MAX_PARAMETERS + 1],
        caller_script_ptr: usize,
        caller_script_len: u32,
        caller_header_features: u32,
        caller_pool_offset: u32,
        caller_pool_slots: u16,
        caller_string_blob_offset: u32,
    ) -> Self {
        Self {
            return_address,
            stack_sp,
            local_count,
            local_base,
            param_start,
            param_len,
            saved_temp_offset,
            saved_parameters,
            bytecode_context,
            active_script_key,
            account_remap,
            caller_script_ptr,
            caller_script_len,
            caller_header_features,
            caller_pool_offset,
            caller_pool_slots,
            caller_string_blob_offset,
        }
    }
}
