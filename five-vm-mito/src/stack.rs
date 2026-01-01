use crate::{
    types::{CallFrame, LocalVariables},
    MAX_CALL_DEPTH, MAX_LOCALS, STACK_SIZE, TEMP_BUFFER_SIZE,
};
use five_protocol::ValueRef;

/// Aggregate storage for all stack-allocated arrays used by the VM.
///
/// This keeps all large arrays in a single struct that can live on the
/// stack and be borrowed by [`ExecutionContext`]. This avoids heap usage
/// while providing zero-copy access to execution state.
pub struct StackStorage<'a> {
    /// Operand stack
    pub stack: [ValueRef; STACK_SIZE],
    /// Function call frames
    pub call_stack: [CallFrame<'a>; MAX_CALL_DEPTH],
    /// Local variables
    pub locals: LocalVariables,
    /// General purpose registers
    pub registers: [ValueRef; 8],
    /// Temporary byte buffer
    pub temp_buffer: [u8; TEMP_BUFFER_SIZE],
}

impl<'a> StackStorage<'a> {
    /// Create a new initialized storage block for a given script.
    #[inline]
    pub fn new(bytecode: &'a [u8]) -> Self {
        Self {
            stack: [ValueRef::Empty; STACK_SIZE],
            call_stack: [CallFrame::new(0, 0, 0, bytecode); MAX_CALL_DEPTH],
            locals: [ValueRef::Empty; MAX_LOCALS],
            registers: [ValueRef::Empty; 8],
            temp_buffer: [0; TEMP_BUFFER_SIZE],
        }
    }
}
