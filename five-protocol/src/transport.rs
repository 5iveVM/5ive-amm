//! Function Transport Protocol Implementation
//!
//! Provides the complete function transport protocol for bytecode encoding,
//! function dispatch, and execution coordination between compiler and VMs.

use crate::MAX_FUNCTIONS;
use crate::{CallError, CallFrame, CallStack, FunctionSignature, LocalStorage, Value, ValueRef};
use alloc::vec::Vec;

/// Function dispatch table for jump table resolution
#[derive(Debug, Clone)]
pub struct FunctionTable {
    /// Function signatures (heap-allocated)
    signatures: Vec<FunctionSignature>,
    /// Function bytecode offsets
    offsets: Vec<usize>,
    /// Number of functions in table
    count: u8,
}

impl Default for FunctionTable {
    fn default() -> Self {
        Self::new()
    }
}

impl FunctionTable {
    /// Create new function table
    #[inline]
    pub fn new() -> Self {
        Self {
            signatures: Vec::new(),
            offsets: Vec::new(),
            count: 0,
        }
    }

    /// Add function to table
    #[inline]
    pub fn add_function(
        &mut self,
        signature: FunctionSignature,
        offset: usize,
    ) -> Result<u8, CallError> {
        if self.count >= MAX_FUNCTIONS as u8 {
            return Err(CallError::InvalidFunction);
        }

        let index = self.count;
        self.signatures.push(signature);
        self.offsets.push(offset);
        self.count += 1;

        Ok(index)
    }

    /// Get function signature by index
    #[inline]
    pub fn get_signature(&self, index: u8) -> Option<FunctionSignature> {
        if (index as usize) < self.signatures.len() {
            Some(self.signatures[index as usize])
        } else {
            None
        }
    }

    /// Get function offset by index
    #[inline]
    pub fn get_offset(&self, index: u8) -> Option<usize> {
        if (index as usize) < self.offsets.len() {
            Some(self.offsets[index as usize])
        } else {
            None
        }
    }

    /// Get function count
    #[inline]
    pub const fn count(&self) -> u8 {
        self.count
    }

    /// Find function by name hash
    #[inline]
    pub fn find_function(&self, name_hash: u32) -> Option<u8> {
        let mut i = 0;
        while i < self.signatures.len() {
            if self.signatures[i].name_hash == name_hash {
                return Some(i as u8);
            }
            i += 1;
        }
        None
    }

    /// Validate function index
    #[inline]
    pub fn is_valid_index(&self, index: u8) -> bool {
        (index as usize) < self.signatures.len()
    }
}

/// Bytecode instruction encoding for function transport
#[derive(Debug, Clone, Copy)]
pub struct Instruction {
    pub opcode: u8,
    pub arg1: u32, // Generic argument (can be u8, u16, u32)
    pub arg2: u32, // Second argument for complex instructions
}

impl Instruction {
    /// Create new instruction
    #[inline]
    pub const fn new(opcode: u8, arg1: u32, arg2: u32) -> Self {
        Self { opcode, arg1, arg2 }
    }

    /// Create instruction with single argument
    #[inline]
    pub const fn with_arg(opcode: u8, arg: u32) -> Self {
        Self::new(opcode, arg, 0)
    }

    /// Create instruction with no arguments
    #[inline]
    pub const fn simple(opcode: u8) -> Self {
        Self::new(opcode, 0, 0)
    }

    /// Encode instruction to bytes (little-endian)
    #[inline]
    pub fn encode(&self) -> [u8; 9] {
        let mut bytes = [0u8; 9];
        bytes[0] = self.opcode;
        bytes[1..5].copy_from_slice(&self.arg1.to_le_bytes());
        bytes[5..9].copy_from_slice(&self.arg2.to_le_bytes());
        bytes
    }

    /// Decode instruction from bytes
    #[inline]
    pub fn decode(bytes: &[u8]) -> Result<Self, TransportError> {
        if bytes.len() < 9 {
            return Err(TransportError::InvalidInstruction);
        }

        let opcode = bytes[0];
        let arg1 = u32::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]);
        let arg2 = u32::from_le_bytes([bytes[5], bytes[6], bytes[7], bytes[8]]);

        Ok(Self::new(opcode, arg1, arg2))
    }

    /// Get instruction size in bytes
    #[inline]
    pub const fn size() -> usize {
        9 // 1 opcode + 4 arg1 + 4 arg2
    }
}

/// Function call protocol state machine
#[derive(Debug)]
pub struct CallProtocol {
    /// Function table for dispatch
    pub function_table: FunctionTable,
    /// Call stack for function calls
    pub call_stack: CallStack,
    /// Local storage for variables
    pub local_storage: LocalStorage,
    /// Current instruction pointer
    pub instruction_pointer: usize,
    /// Current function index (255 = no function)
    pub current_function: u8,
}

impl Default for CallProtocol {
    fn default() -> Self {
        Self::new()
    }
}

impl CallProtocol {
    /// Create new call protocol
    #[inline]
    pub fn new() -> Self {
        Self {
            function_table: FunctionTable::new(),
            call_stack: CallStack::new(),
            local_storage: LocalStorage::new(),
            instruction_pointer: 0,
            current_function: 255, // Invalid function index
        }
    }

    /// Initialize with function table
    #[inline]
    pub fn initialize(&mut self, function_table: FunctionTable) {
        self.function_table = function_table;
        self.call_stack.clear();
        self.instruction_pointer = 0;
        self.current_function = 255;
    }

    /// Prepare function call
    #[inline]
    pub fn prepare_call(
        &mut self,
        function_index: u8,
        params: &[Value],
    ) -> Result<usize, CallError> {
        // Validate function index
        let signature = self
            .function_table
            .get_signature(function_index)
            .ok_or(CallError::InvalidFunction)?;

        let offset = self
            .function_table
            .get_offset(function_index)
            .ok_or(CallError::InvalidFunction)?;

        // Validate parameters
        if !signature.validate_params(params) {
            return Err(CallError::InvalidParameterType);
        }

        // Calculate local storage base (after current allocated locals)
        let locals_base = if let Some(frame) = self.call_stack.current() {
            frame.locals_base + frame.local_slots
        } else {
            0
        };

        // Create call frame
        let frame = CallFrame::new(
            self.instruction_pointer + Instruction::size(), // Return address after call instruction
            function_index,
            params.len() as u8,
            signature.local_slots,
            locals_base,
            0, // Will be set by VM with actual stack size
        );

        // Push call frame
        self.call_stack.push(frame)?;

        // Allocate local storage
        self.local_storage
            .allocate(signature.local_slots, locals_base)?;

        // Update current function
        self.current_function = function_index;

        Ok(offset)
    }

    /// Finish function call (return)
    #[inline]
    pub fn finish_call(&mut self) -> Result<usize, CallError> {
        // Pop call frame
        let frame = self.call_stack.pop()?;

        // Deallocate local storage
        self.local_storage.deallocate();

        // Update current function
        self.current_function = if let Some(current_frame) = self.call_stack.current() {
            current_frame.function_index
        } else {
            255 // No function
        };

        Ok(frame.return_address)
    }

    /// Set local variable
    #[inline]
    pub fn set_local(&mut self, slot: u8, value: ValueRef) -> Result<(), CallError> {
        self.local_storage.set_local(slot, value)
    }

    /// Get local variable
    #[inline]
    pub fn get_local(&self, slot: u8) -> Result<&ValueRef, CallError> {
        self.local_storage.get_local(slot)
    }

    /// Get current function signature
    #[inline]
    pub fn current_signature(&self) -> Option<FunctionSignature> {
        if self.current_function == 255 {
            None
        } else {
            self.function_table.get_signature(self.current_function)
        }
    }

    /// Check if in function call
    #[inline]
    pub const fn in_function(&self) -> bool {
        self.current_function != 255
    }

    /// Get call depth
    #[inline]
    pub const fn call_depth(&self) -> u8 {
        self.call_stack.depth()
    }
}

/// Jump table encoding for function dispatch
#[derive(Debug, Clone)]
pub struct JumpTable {
    /// Table entries (function_index -> offset)
    entries: [usize; MAX_FUNCTIONS],
    /// Number of entries
    count: u8,
}

impl Default for JumpTable {
    fn default() -> Self {
        Self::new()
    }
}

impl JumpTable {
    /// Create new jump table
    #[inline]
    pub const fn new() -> Self {
        Self {
            entries: [0; MAX_FUNCTIONS],
            count: 0,
        }
    }

    /// Add jump table entry
    #[inline]
    pub fn add_entry(&mut self, offset: usize) -> Result<u8, TransportError> {
        if self.count >= MAX_FUNCTIONS as u8 {
            return Err(TransportError::JumpTableFull);
        }

        let index = self.count;
        self.entries[index as usize] = offset;
        self.count += 1;

        Ok(index)
    }

    /// Get jump offset by function index
    #[inline]
    pub fn get_offset(&self, function_index: u8) -> Option<usize> {
        if function_index < self.count {
            Some(self.entries[function_index as usize])
        } else {
            None
        }
    }

    /// Encode jump table to bytecode
    #[inline]
    pub fn encode(&self) -> Result<[u8; 1 + 4 * MAX_FUNCTIONS], TransportError> {
        let mut bytes = [0u8; 1 + 4 * MAX_FUNCTIONS];
        bytes[0] = self.count;

        let mut i = 0;
        while i < self.count {
            let offset_bytes = self.entries[i as usize].to_le_bytes();
            let start = 1 + (i as usize * 4);
            bytes[start..start + 4].copy_from_slice(&offset_bytes[..4]);
            i += 1;
        }

        Ok(bytes)
    }

    /// Decode jump table from bytecode
    #[inline]
    pub fn decode(bytes: &[u8]) -> Result<Self, TransportError> {
        if bytes.is_empty() {
            return Err(TransportError::InvalidJumpTable);
        }

        let count = bytes[0];
        if count > MAX_FUNCTIONS as u8 {
            return Err(TransportError::JumpTableFull);
        }

        if bytes.len() < 1 + (count as usize * 4) {
            return Err(TransportError::InvalidJumpTable);
        }

        let mut table = Self::new();
        table.count = count;

        let mut i = 0;
        while i < count {
            let start = 1 + (i as usize * 4);
            let offset_bytes = [
                bytes[start],
                bytes[start + 1],
                bytes[start + 2],
                bytes[start + 3],
            ];
            table.entries[i as usize] = u32::from_le_bytes(offset_bytes) as usize;
            i += 1;
        }

        Ok(table)
    }
}

/// Transport error types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportError {
    InvalidInstruction,
    InvalidJumpTable,
    JumpTableFull,
    FunctionNotFound,
    InvalidBytecode,
    CallError(CallError),
}

impl From<CallError> for TransportError {
    #[inline]
    fn from(err: CallError) -> Self {
        TransportError::CallError(err)
    }
}

impl TransportError {
    /// Get error message
    #[inline]
    pub const fn message(&self) -> &'static str {
        match self {
            TransportError::InvalidInstruction => "Invalid instruction encoding",
            TransportError::InvalidJumpTable => "Invalid jump table format",
            TransportError::JumpTableFull => "Jump table is full",
            TransportError::FunctionNotFound => "Function not found",
            TransportError::InvalidBytecode => "Invalid bytecode format",
            TransportError::CallError(call_err) => call_err.message(),
        }
    }
}
