use crate::types::CallFrame;
use five_protocol::ValueRef;
use crate::error::{CompactResult, Result, VMErrorCode, VMError};
use crate::{MAX_CALL_DEPTH, MAX_LOCALS, MAX_PARAMETERS};
use crate::debug_log;

const SHARED_PARAM_SIZE: usize = MAX_PARAMETERS + 1;

pub struct FrameManager<'a> {
    pub call_stack: &'a mut [CallFrame],
    pub locals: &'a mut [core::mem::MaybeUninit<ValueRef>],
    pub csp: u8,

    // Current frame local state
    pub local_count: u8,
    pub local_base: u8,

    // Parameters
    pub parameters: [ValueRef; SHARED_PARAM_SIZE],
    pub param_start: u8,
    pub param_len: u8,
}

impl<'a> FrameManager<'a> {
    #[inline(always)]
    pub fn new(call_stack: &'a mut [CallFrame], locals: &'a mut [core::mem::MaybeUninit<ValueRef>]) -> Self {
        Self {
            call_stack,
            locals,
            csp: 0,
            local_count: 0,
            local_base: 0,
            parameters: [ValueRef::Empty; SHARED_PARAM_SIZE],
            param_start: 0,
            param_len: 0,
        }
    }

    // --- Call stack operations ---

    #[inline(always)]
    pub fn push_call_frame(&mut self, frame: CallFrame) -> Result<()> {
        if self.csp as usize >= MAX_CALL_DEPTH {
            return Err(VMError::CallStackOverflow);
        }
        self.call_stack[self.csp as usize] = frame;
        self.csp += 1;
        Ok(())
    }

    #[inline(always)]
    pub fn pop_call_frame(&mut self) -> CompactResult<CallFrame> {
        if self.csp == 0 {
            return Err(VMErrorCode::CallStackUnderflow);
        }
        self.csp -= 1;
        Ok(self.call_stack[self.csp as usize])
    }

    #[inline(always)]
    pub fn call_depth(&self) -> usize {
        self.csp as usize
    }

    #[inline(always)]
    pub fn set_call_depth(&mut self, depth: u8) -> CompactResult<()> {
        if depth as usize >= MAX_CALL_DEPTH {
            return Err(VMErrorCode::CallStackOverflow);
        }
        self.csp = depth;
        Ok(())
    }

    #[inline(always)]
    pub fn get_call_frame(&self, index: usize) -> CompactResult<&CallFrame> {
        if index < self.csp as usize {
            Ok(&self.call_stack[index])
        } else {
            Err(VMErrorCode::InvalidOperation)
        }
    }

    #[inline(always)]
    pub fn set_call_frame(&mut self, index: usize, frame: CallFrame) -> CompactResult<()> {
        if index < self.csp as usize {
            self.call_stack[index] = frame;
            Ok(())
        } else {
            Err(VMErrorCode::InvalidOperation)
        }
    }

    // --- Local variables ---

    #[inline]
    pub fn get_local(&self, index: u8) -> CompactResult<ValueRef> {
        if index >= self.local_count {
            debug_log!("LOCAL_DEBUG: get_local index out of bounds: {} >= {}", index, self.local_count);
            return Err(VMErrorCode::LocalsOverflow);
        }
        // SAFETY: We assume the compiler/verifier ensures locals are initialized before use.
        // Bounds checking is done above.
        unsafe {
            Ok(self.locals[self.local_base as usize + index as usize].assume_init())
        }
    }

    #[inline(always)]
    pub fn set_local(&mut self, index: u8, value: ValueRef) -> CompactResult<()> {
        if index >= self.local_count {
            debug_log!("LOCAL_DEBUG: set_local index out of bounds: {} >= {}", index, self.local_count);
            return Err(VMErrorCode::LocalsOverflow);
        }
        self.locals[self.local_base as usize + index as usize] = core::mem::MaybeUninit::new(value);
        Ok(())
    }

    #[inline(always)]
    pub fn clear_local(&mut self, index: u8) -> CompactResult<()> {
        if index >= self.local_count {
            return Err(VMErrorCode::LocalsOverflow);
        }

        let absolute_index = (self.local_base + index) as usize;
        if absolute_index >= self.locals.len() {
            return Err(VMErrorCode::LocalsOverflow);
        }

        // We mark it as Empty for safety, though technically not required if we trust the compiler
        self.locals[absolute_index] = core::mem::MaybeUninit::new(ValueRef::Empty);

        if index + 1 == self.local_count {
            while self.local_count > 0 {
                let pos = (self.local_base + self.local_count - 1) as usize;
                if pos < self.locals.len() {
                    // Trim only when clearing the last slot.
                    break;
                }
                self.local_count -= 1;
            }
        }

        Ok(())
    }

    #[inline(always)]
    pub fn local_count(&self) -> u8 {
        self.local_count
    }

    #[inline(always)]
    pub fn set_local_count(&mut self, count: u8) {
        self.local_count = count;
    }

    #[inline(always)]
    pub fn local_base(&self) -> u8 {
        self.local_base
    }

    #[inline(always)]
    pub fn set_local_base(&mut self, base: u8) {
        self.local_base = base;
    }

    #[inline(always)]
    pub fn allocate_locals(&mut self, count: u8) -> CompactResult<()> {
        if (self.local_base as usize + count as usize) > MAX_LOCALS {
            return Err(VMErrorCode::LocalsOverflow);
        }

        // Optimization: Zero-Cost Locals Initialization
        // We do NOT initialize memory. It contains garbage or previous values.
        // Security relies on the specific script compiler ensuring variables 
        // are assigned before read (STORE_LOCAL before LOAD_LOCAL).
        
        self.local_count = count;
        Ok(())
    }

    #[inline(always)]
    pub fn deallocate_locals(&mut self) {
        // Optimization: No-Op Deallocation
        // We don't need to clear values, just reset the count.
        self.local_count = 0;
    }

    // --- Parameter operations ---

    #[inline(always)]
    pub fn set_parameters(&mut self, params: [ValueRef; 8]) {
        self.parameters[..8].copy_from_slice(&params);
        self.param_start = 0;
        self.param_len = MAX_PARAMETERS as u8;
    }

    #[inline(always)]
    pub fn parameters(&self) -> &[ValueRef] {
        &self.parameters[..]
    }

    #[inline(always)]
    pub fn parameters_mut(&mut self) -> &mut [ValueRef] {
        &mut self.parameters[..]
    }

    #[inline(always)]
    pub fn param_start(&self) -> u8 {
        self.param_start
    }

    #[inline(always)]
    pub fn param_len(&self) -> u8 {
        self.param_len
    }

    #[inline(always)]
    pub fn allocate_params(&mut self, count: u8) -> CompactResult<()> {
        // Hot path: callers always write params[1..=count] immediately and LOAD_PARAM
        // enforces param_len bounds, so eagerly clearing this span is redundant.
        self.param_start = 0;
        self.param_len = count;
        Ok(())
    }

    #[inline(always)]
    pub fn restore_parameters(&mut self, start: u8, len: u8) {
        self.param_start = start;
        self.param_len = len;
    }

    #[inline(always)]
    pub fn set_parameter(&mut self, index: usize, value: ValueRef) -> CompactResult<()> {
        if index < self.parameters.len() {
            self.parameters[index] = value;
            Ok(())
        } else {
            Err(VMErrorCode::InvalidParameter)
        }
    }
}
