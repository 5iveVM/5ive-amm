//! Function calling convention.

use crate::{FunctionSignature, Value, ValueRef, MAX_CALL_DEPTH, MAX_FUNCTION_PARAMS, MAX_LOCALS};

/// Call frame for function calls.
#[derive(Debug, Clone, Copy)]
pub struct CallFrame {
    /// Return address (instruction pointer).
    pub return_address: usize,
    /// Function being called.
    pub function_index: u8,
    /// Number of parameters passed.
    pub param_count: u8,
    /// Number of local slots allocated.
    pub local_slots: u8,
    /// Base offset for local variables in locals array.
    pub locals_base: u8,
    /// Saved stack size for cleanup.
    pub saved_stack_size: usize,
}

impl CallFrame {
    /// Create new call frame.
    #[inline]
    pub const fn new(
        return_address: usize,
        function_index: u8,
        param_count: u8,
        local_slots: u8,
        locals_base: u8,
        saved_stack_size: usize,
    ) -> Self {
        Self {
            return_address,
            function_index,
            param_count,
            local_slots,
            locals_base,
            saved_stack_size,
        }
    }

    /// Check if frame is valid.
    #[inline]
    pub const fn is_valid(&self) -> bool {
        (self.param_count as usize) <= MAX_FUNCTION_PARAMS
            && (self.local_slots as usize) <= MAX_LOCALS
            && (self.locals_base as usize + self.local_slots as usize) <= MAX_LOCALS
    }
}

/// Call stack for managing function calls.
#[derive(Debug)]
pub struct CallStack {
    /// Call frames array.
    frames: [CallFrame; MAX_CALL_DEPTH],
    /// Current depth.
    depth: u8,
}

impl Default for CallStack {
    fn default() -> Self {
        Self::new()
    }
}

impl CallStack {
    /// Create new call stack.
    #[inline]
    pub const fn new() -> Self {
        const EMPTY_FRAME: CallFrame = CallFrame::new(0, 0, 0, 0, 0, 0);
        Self {
            frames: [EMPTY_FRAME; MAX_CALL_DEPTH],
            depth: 0,
        }
    }

    /// Push new call frame.
    #[inline]
    pub fn push(&mut self, frame: CallFrame) -> Result<(), CallError> {
        if self.depth >= MAX_CALL_DEPTH as u8 {
            return Err(CallError::StackOverflow);
        }
        if !frame.is_valid() {
            return Err(CallError::InvalidFrame);
        }

        self.frames[self.depth as usize] = frame;
        self.depth += 1;
        Ok(())
    }

    /// Pop call frame.
    #[inline]
    pub fn pop(&mut self) -> Result<CallFrame, CallError> {
        if self.depth == 0 {
            return Err(CallError::StackUnderflow);
        }

        self.depth -= 1;
        Ok(self.frames[self.depth as usize])
    }

    /// Get current frame (top of stack).
    #[inline]
    pub fn current(&self) -> Option<&CallFrame> {
        if self.depth == 0 {
            None
        } else {
            Some(&self.frames[self.depth as usize - 1])
        }
    }

    /// Get current call depth.
    #[inline]
    pub const fn depth(&self) -> u8 {
        self.depth
    }

    /// Check if stack is empty.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.depth == 0
    }

    /// Check if stack is full.
    #[inline]
    pub const fn is_full(&self) -> bool {
        self.depth >= MAX_CALL_DEPTH as u8
    }

    /// Clear the call stack.
    #[inline]
    pub fn clear(&mut self) {
        self.depth = 0;
    }
}

/// Parameter passing protocol.
#[derive(Debug, Clone)]
pub struct ParameterProtocol {
    /// Function signature.
    pub signature: FunctionSignature,
    /// Parameter values.
    pub values: [Value; MAX_FUNCTION_PARAMS],
    /// Actual parameter count.
    pub count: u8,
}

impl ParameterProtocol {
    /// Create new parameter protocol.
    #[inline]
    pub const fn new(signature: FunctionSignature) -> Self {
        Self {
            signature,
            values: [Value::Empty; MAX_FUNCTION_PARAMS],
            count: 0,
        }
    }

    /// Add parameter value.
    #[inline]
    pub fn add_param(&mut self, value: Value) -> Result<(), CallError> {
        if self.count >= MAX_FUNCTION_PARAMS as u8 {
            return Err(CallError::TooManyParameters);
        }
        if self.count >= self.signature.parameter_count {
            return Err(CallError::TooManyParameters);
        }

        // Validate parameter type
        let param_spec = &self.signature.parameters[self.count as usize];
        if !param_spec.validate(&value) {
            return Err(CallError::InvalidParameterType);
        }

        self.values[self.count as usize] = value;
        self.count += 1;
        Ok(())
    }

    /// Validate all parameters are provided.
    #[inline]
    pub fn validate(&self) -> Result<(), CallError> {
        if self.count != self.signature.parameter_count {
            return Err(CallError::MissingParameters);
        }

        // Additional validation could go here
        Ok(())
    }

    /// Get parameter by index.
    #[inline]
    pub fn get_param(&self, index: u8) -> Option<&Value> {
        if index < self.count {
            Some(&self.values[index as usize])
        } else {
            None
        }
    }
}

use alloc::vec::Vec;

/// Local variable storage protocol
#[derive(Debug)]
pub struct LocalStorage {
    /// Local variables array (heap-allocated)
    /// Each entry is (slot_id, value) for sparse allocation
    locals: Vec<(u8, ValueRef)>,
    /// Number of allocated slots
    allocated: u8,
    /// Current function's base offset
    base_offset: u8,
}

impl Default for LocalStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl LocalStorage {
    /// Create new local storage
    #[inline]
    pub fn new() -> Self {
        // Removed const as Vec allocation is needed
        let mut locals = Vec::with_capacity(MAX_LOCALS);
        for _ in 0..MAX_LOCALS {
            locals.push((0, ValueRef::Empty));
        }
        Self {
            locals,
            allocated: 0,
            base_offset: 0,
        }
    }

    /// Allocate local slots for function
    #[inline]
    pub fn allocate(&mut self, slots: u8, base_offset: u8) -> Result<(), CallError> {
        if (base_offset as usize + slots as usize) > MAX_LOCALS {
            return Err(CallError::InsufficientLocals);
        }

        // Initialize allocated slots
        let mut i = 0;
        while i < slots {
            let slot_index = (base_offset + i) as usize;
            if slot_index < self.locals.len() {
                self.locals[slot_index] = (base_offset + i, ValueRef::Empty);
            }
            i += 1;
        }

        self.allocated = slots;
        self.base_offset = base_offset;
        Ok(())
    }

    /// Deallocate local slots
    #[inline]
    pub fn deallocate(&mut self) {
        // Clear allocated slots
        let mut i = 0;
        while i < self.allocated {
            let slot_index = (self.base_offset + i) as usize;
            if slot_index < self.locals.len() {
                self.locals[slot_index] = (0, ValueRef::Empty);
            }
            i += 1;
        }

        self.allocated = 0;
        self.base_offset = 0;
    }

    /// Set local variable value
    #[inline]
    pub fn set_local(&mut self, slot: u8, value: ValueRef) -> Result<(), CallError> {
        let actual_slot = self.base_offset + slot;
        if (actual_slot as usize) >= MAX_LOCALS {
            return Err(CallError::InvalidLocalSlot);
        }
        if slot >= self.allocated {
            return Err(CallError::InvalidLocalSlot);
        }

        if (actual_slot as usize) < self.locals.len() {
            self.locals[actual_slot as usize] = (actual_slot, value);
        }
        Ok(())
    }

    /// Get local variable value
    #[inline]
    pub fn get_local(&self, slot: u8) -> Result<&ValueRef, CallError> {
        let actual_slot = self.base_offset + slot;
        if (actual_slot as usize) >= MAX_LOCALS {
            return Err(CallError::InvalidLocalSlot);
        }
        if slot >= self.allocated {
            return Err(CallError::InvalidLocalSlot);
        }

        if (actual_slot as usize) < self.locals.len() {
            Ok(&self.locals[actual_slot as usize].1)
        } else {
            Err(CallError::InvalidLocalSlot)
        }
    }

    /// Clear local variable
    #[inline]
    pub fn clear_local(&mut self, slot: u8) -> Result<(), CallError> {
        self.set_local(slot, ValueRef::Empty)
    }
}

/// Call error types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallError {
    StackOverflow,
    StackUnderflow,
    InvalidFrame,
    TooManyParameters,
    MissingParameters,
    InvalidParameterType,
    InvalidLocalSlot,
    InsufficientLocals,
    InvalidFunction,
    InvalidReturnType,
}

impl CallError {
    /// Get error message
    #[inline]
    pub const fn message(&self) -> &'static str {
        match self {
            CallError::StackOverflow => "Call stack overflow",
            CallError::StackUnderflow => "Call stack underflow",
            CallError::InvalidFrame => "Invalid call frame",
            CallError::TooManyParameters => "Too many parameters",
            CallError::MissingParameters => "Missing parameters",
            CallError::InvalidParameterType => "Invalid parameter type",
            CallError::InvalidLocalSlot => "Invalid local slot",
            CallError::InsufficientLocals => "Insufficient local slots",
            CallError::InvalidFunction => "Invalid function",
            CallError::InvalidReturnType => "Invalid return type",
        }
    }
}
