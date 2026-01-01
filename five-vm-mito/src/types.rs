//! Type definitions for MitoVM
//!
//! This module contains core type definitions used throughout the MitoVM execution engine.
//! Extracted from the main execution module for better organization and maintainability.

use crate::MAX_LOCALS;
use five_protocol::ValueRef;

/// Stack-allocated local variable storage optimized for minimal memory usage.
/// Each slot holds a single [`ValueRef`]; [`ValueRef::Empty`] marks
/// uninitialized locals.
pub type LocalVariables = [ValueRef; MAX_LOCALS];

/// Function call frame containing return state and saved parameters.
///
/// # Example
/// ```rust
/// use five_vm_mito::CallFrame;
///
/// let bytecode = &[0x00, 0x07]; // NOP, RETURN_VALUE
/// let frame = CallFrame::new(100, 2, 0, bytecode);
/// assert_eq!(frame.return_address, 100);
/// assert_eq!(frame.local_count, 2);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct CallFrame<'a> {
    pub return_address: usize,
    pub local_count: u8,
    pub local_base: u8,     // Base offset for this frame's locals in shared array
    pub param_start: u8,    // Start index of caller parameters in shared array
    pub param_len: u8,      // Length of caller parameter slice
    pub bytecode: &'a [u8], // Caller bytecode for context restoration
}

impl<'a> CallFrame<'a> {
    /// Create call frame with return address and local variable count.
    pub fn new(return_address: usize, local_count: u8, local_base: u8, bytecode: &'a [u8]) -> Self {
        Self {
            return_address,
            local_count,
            local_base,
            param_start: 0,
            param_len: 0,
            bytecode,
        }
    }

    /// Create call frame with saved caller parameters for restoration on return.
    pub fn with_parameters(
        return_address: usize,
        local_count: u8,
        local_base: u8,
        param_start: u8,
        param_len: u8,
        bytecode: &'a [u8],
    ) -> Self {
        Self {
            return_address,
            local_count,
            local_base,
            param_start,
            param_len,
            bytecode,
        }
    }
}
