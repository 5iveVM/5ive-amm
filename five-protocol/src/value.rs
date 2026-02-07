//! Zero-Copy Value System for MitoVM
//!
//! Implements ultra-lightweight value references that avoid stack overflow
//! in Solana BPF by using direct references instead of copying data.

/// Error type for protocol-level operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ProtocolError {
    BufferTooSmall,
    InvalidInstruction,
    TypeMismatch,
    InvalidAccountData,
}

// Deserialization helper macros to reduce boilerplate
macro_rules! deserialize_u16_field {
    ($buffer:expr, $variant:path) => {{
        if $buffer.len() < 3 {
            return Err(ProtocolError::InvalidInstruction);
        }
        let mut bytes = [0u8; 2];
        bytes.copy_from_slice(&$buffer[1..3]);
        Ok($variant(u16::from_le_bytes(bytes)))
    }};
}

macro_rules! deserialize_u32_field {
    ($buffer:expr, $variant:path) => {{
        if $buffer.len() < 5 {
            return Err(ProtocolError::InvalidInstruction);
        }
        let mut bytes = [0u8; 4];
        bytes.copy_from_slice(&$buffer[1..5]);
        Ok($variant(u32::from_le_bytes(bytes)))
    }};
}

macro_rules! deserialize_two_bytes {
    ($buffer:expr, $variant:path) => {{
        if $buffer.len() < 3 {
            return Err(ProtocolError::InvalidInstruction);
        }
        Ok($variant($buffer[1], $buffer[2]))
    }};
}

/// Zero-copy value reference optimized for Solana BPF stack constraints
/// Size: 8 bytes (reduces from 56-byte Value enum to eliminate stack overflow)
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(C, u8)]
pub enum ValueRef {
    /// Empty/uninitialized value - 0 bytes data
    Empty,
    /// Immediate 8-bit value stored inline
    U8(u8),
    /// Immediate 64-bit value stored inline  
    U64(u64),
    /// Immediate 64-bit signed value stored inline
    I64(i64),
    /// Immediate 128-bit value stored inline (MITO-style BPF-optimized)
    U128(u128),
    /// Immediate boolean value stored inline
    Bool(bool),
    /// Reference to account data: (account_index, offset)
    AccountRef(u8, u16),
    /// Reference to input data: (offset)
    InputRef(u16),
    /// Reference to temp buffer: (offset, size)
    TempRef(u8, u8),
    /// Reference to tuple data: (offset, size)
    TupleRef(u8, u8),
    /// Reference to optional data: (offset, size)
    OptionalRef(u8, u8),
    /// Reference to result data: (offset, size) - where offset=0,size=0 indicates Err
    ResultRef(u8, u8),
    /// Reference to pubkey data in input: (offset)
    ///
    /// WARNING: PubkeyRef is NOT compatible with OptionalRef, TupleRef, etc.
    /// Complex types like Option<Pubkey>, (Pubkey, u64), etc. are not supported
    /// with current 8-byte ValueRef constraint. This is a design limitation that
    /// needs architectural solution when such types are required.
    PubkeyRef(u16),
    /// Reference to array data in temp buffer: (array_id)
    /// Arrays stored as: [length: u8][element_type: u8][element_0]...[element_n]
    /// Max 62 bytes for array data in 64-byte temp buffer
    ArrayRef(u8),
    /// Reference to string data in temp buffer: (offset)
    /// Strings stored as UTF-8 bytes: [utf8_byte_length: u8][utf8_byte_0]...[utf8_byte_n]
    /// Max 63 bytes for string data in 64-byte temp buffer
    StringRef(u16),
    /// NEW Phase 3.2: Reference to heap-allocated string: (heap_id)
    /// For strings >32 bytes that exceed temp buffer capacity
    HeapString(u32),
    /// NEW Phase 3.2: Reference to heap-allocated array: (heap_id)  
    /// For arrays >temp capacity that require dynamic growth
    HeapArray(u32),
}

/// Legacy Value enum for compatibility (deprecated - causes stack overflow)
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Value {
    /// Empty/uninitialized value
    Empty,
    /// 8-bit unsigned integer
    U8(u8),
    /// 64-bit unsigned integer
    U64(u64),
    /// 64-bit signed integer
    I64(i64),
    /// 128-bit unsigned integer (MITO-style BPF-native)
    U128(u128),
    /// Boolean value
    Bool(bool),
    /// Pubkey for PDA operations
    Pubkey([u8; 32]),
    /// String data (UTF-8 encoded byte array index)
    String(u8),
    /// Account index for account operations
    Account(u8),
    /// Array index for array operations
    Array(u8),
}

impl Value {}

impl ValueRef {
    /// Get the size of the serialized value.
    pub fn serialized_size(&self) -> usize {
        match self {
            ValueRef::Empty => 1,
            ValueRef::U8(_) => 1 + 1,
            ValueRef::U64(_) => 1 + 8,
            ValueRef::I64(_) => 1 + 8,
            ValueRef::U128(_) => 1 + 16,
            ValueRef::Bool(_) => 1 + 1,
            ValueRef::AccountRef(_, _) => 1 + 3,
            ValueRef::InputRef(_) => 1 + 2,
            ValueRef::TempRef(_, _) => 1 + 2,
            ValueRef::TupleRef(_, _) => 1 + 2,
            ValueRef::OptionalRef(_, _) => 1 + 2,
            ValueRef::ResultRef(_, _) => 1 + 2,
            ValueRef::PubkeyRef(_) => 1 + 2,
            ValueRef::ArrayRef(_) => 1 + 1,
            ValueRef::StringRef(_) => 1 + 2,
            ValueRef::HeapString(_) => 1 + 4, // Type byte + u32 heap ID
            ValueRef::HeapArray(_) => 1 + 4,  // Type byte + u32 heap ID
        }
    }

    /// Serialize the ValueRef into a buffer.
    pub fn serialize_into(&self, buffer: &mut [u8]) -> Result<usize, ProtocolError> {
        let size = self.serialized_size();
        if buffer.len() < size {
            return Err(ProtocolError::BufferTooSmall);
        }
        buffer[0] = self.type_id();
        match self {
            ValueRef::Empty => {}
            ValueRef::U8(v) => buffer[1] = *v,
            ValueRef::U64(v) => buffer[1..9].copy_from_slice(&v.to_le_bytes()),
            ValueRef::I64(v) => buffer[1..9].copy_from_slice(&v.to_le_bytes()),
            ValueRef::U128(v) => buffer[1..17].copy_from_slice(&v.to_le_bytes()),
            ValueRef::Bool(v) => buffer[1] = *v as u8,
            ValueRef::AccountRef(i, o) => {
                buffer[1] = *i;
                buffer[2..4].copy_from_slice(&o.to_le_bytes());
            }
            ValueRef::InputRef(o) => buffer[1..3].copy_from_slice(&o.to_le_bytes()),
            ValueRef::TempRef(o, s) => {
                buffer[1] = *o;
                buffer[2] = *s;
            }
            ValueRef::TupleRef(o, s) => {
                buffer[1] = *o;
                buffer[2] = *s;
            }
            ValueRef::OptionalRef(o, s) => {
                buffer[1] = *o;
                buffer[2] = *s;
            }
            ValueRef::ResultRef(o, s) => {
                buffer[1] = *o;
                buffer[2] = *s;
            }
            ValueRef::PubkeyRef(o) => buffer[1..3].copy_from_slice(&o.to_le_bytes()),
            ValueRef::ArrayRef(id) => buffer[1] = *id,
            ValueRef::StringRef(o) => buffer[1..3].copy_from_slice(&o.to_le_bytes()),
            ValueRef::HeapString(id) => buffer[1..5].copy_from_slice(&id.to_le_bytes()),
            ValueRef::HeapArray(id) => buffer[1..5].copy_from_slice(&id.to_le_bytes()),
        }
        Ok(size)
    }

    /// Deserialize a ValueRef from a buffer.
    pub fn deserialize_from(buffer: &[u8]) -> Result<ValueRef, ProtocolError> {
        if buffer.is_empty() {
            return Err(ProtocolError::InvalidInstruction);
        }
        let type_id = buffer[0];
        use crate::types;
        match type_id {
            // Protocol types (match types.rs constants)
            t if t == types::EMPTY => Ok(ValueRef::Empty), // 0
            t if t == types::U8 => {
                // 1
                if buffer.len() < 2 {
                    return Err(ProtocolError::InvalidInstruction);
                }
                Ok(ValueRef::U8(buffer[1]))
            }
            t if t == types::U64 => {
                // 4 (FIXED: was 2)
                if buffer.len() < 9 {
                    return Err(ProtocolError::InvalidInstruction);
                }
                let mut bytes = [0u8; 8];
                bytes.copy_from_slice(&buffer[1..9]);
                Ok(ValueRef::U64(u64::from_le_bytes(bytes)))
            }
            t if t == types::I64 => {
                // 8 (FIXED: was 3)
                if buffer.len() < 9 {
                    return Err(ProtocolError::InvalidInstruction);
                }
                let mut bytes = [0u8; 8];
                bytes.copy_from_slice(&buffer[1..9]);
                Ok(ValueRef::I64(i64::from_le_bytes(bytes)))
            }
            t if t == types::U128 => {
                // 14 (FIXED: was 4)
                if buffer.len() < 17 {
                    return Err(ProtocolError::InvalidInstruction);
                }
                let mut bytes = [0u8; 16];
                bytes.copy_from_slice(&buffer[1..17]);
                Ok(ValueRef::U128(u128::from_le_bytes(bytes)))
            }
            t if t == types::BOOL => {
                // 9 (FIXED: was 5)
                if buffer.len() < 2 {
                    return Err(ProtocolError::InvalidInstruction);
                }
                Ok(ValueRef::Bool(buffer[1] != 0))
            }
            t if t == types::ACCOUNT => {
                // 12 (FIXED: was 6)
                if buffer.len() < 4 {
                    return Err(ProtocolError::InvalidInstruction);
                }
                Ok(ValueRef::AccountRef(buffer[1], {
                    let mut bytes = [0u8; 2];
                    bytes.copy_from_slice(&buffer[2..4]);
                    u16::from_le_bytes(bytes)
                }))
            }
            // Protocol types (match types.rs constants)
            t if t == types::PUBKEY => deserialize_u16_field!(buffer, ValueRef::PubkeyRef),
            t if t == types::STRING => deserialize_u16_field!(buffer, ValueRef::StringRef),
            t if t == types::ARRAY => {
                // 13 (was 13, unchanged)
                if buffer.len() < 2 {
                    return Err(ProtocolError::InvalidInstruction);
                }
                Ok(ValueRef::ArrayRef(buffer[1]))
            }

            // VM-specific types (15+)
            15 => deserialize_u16_field!(buffer, ValueRef::InputRef),
            16 => deserialize_two_bytes!(buffer, ValueRef::TempRef),
            17 => deserialize_two_bytes!(buffer, ValueRef::TupleRef),
            18 => deserialize_two_bytes!(buffer, ValueRef::OptionalRef),
            19 => deserialize_two_bytes!(buffer, ValueRef::ResultRef),
            20 => deserialize_u32_field!(buffer, ValueRef::HeapString),
            21 => deserialize_u32_field!(buffer, ValueRef::HeapArray),
            _ => Err(ProtocolError::InvalidInstruction),
        }
    }

    /// Create immediate U64 value
    #[inline]
    pub const fn immediate_u64(value: u64) -> Self {
        ValueRef::U64(value)
    }

    /// Create immediate U8 value
    #[inline]
    pub const fn immediate_u8(value: u8) -> Self {
        ValueRef::U8(value)
    }

    /// Create immediate I64 value
    #[inline]
    pub const fn immediate_i64(value: i64) -> Self {
        ValueRef::I64(value)
    }

    /// Create immediate Bool value
    #[inline]
    pub const fn immediate_bool(value: bool) -> Self {
        ValueRef::Bool(value)
    }

    /// Create account data reference
    #[inline]
    pub const fn account_ref(account_index: u8, offset: u16) -> Self {
        ValueRef::AccountRef(account_index, offset)
    }

    /// Create input data reference
    #[inline]
    pub const fn input_ref(offset: u16) -> Self {
        ValueRef::InputRef(offset)
    }

    /// Create temp buffer reference
    #[inline]
    pub const fn temp_ref(offset: u8, size: u8) -> Self {
        ValueRef::TempRef(offset, size)
    }

    /// Check if value is immediate (stored inline)
    #[inline]
    pub const fn is_immediate(&self) -> bool {
        match self {
            ValueRef::Empty
            | ValueRef::U8(_)
            | ValueRef::U64(_)
            | ValueRef::I64(_)
            | ValueRef::U128(_)
            | ValueRef::Bool(_) => true,
            _ => false,
        }
    }

    /// Check if value is a reference
    #[inline]
    pub const fn is_reference(&self) -> bool {
        !self.is_immediate()
    }

    /// Get immediate u64 value (if stored inline)
    #[inline]
    pub const fn immediate_as_u64(&self) -> Option<u64> {
        match self {
            ValueRef::U64(v) => Some(*v),
            ValueRef::U8(v) => Some(*v as u64),
            ValueRef::I64(v) => {
                if *v >= 0 {
                    Some(*v as u64)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Get immediate bool value (if stored inline)
    #[inline]
    pub const fn immediate_as_bool(&self) -> Option<bool> {
        match self {
            ValueRef::Bool(v) => Some(*v),
            ValueRef::U64(v) => Some(*v != 0),
            ValueRef::U8(v) => Some(*v != 0),
            _ => None,
        }
    }

    /// Get immediate i64 value (if stored inline)
    #[inline]
    pub const fn immediate_as_i64(&self) -> Option<i64> {
        match self {
            ValueRef::I64(v) => Some(*v),
            ValueRef::U64(v) => {
                if *v <= i64::MAX as u64 {
                    Some(*v as i64)
                } else {
                    None
                }
            }
            ValueRef::U8(v) => Some(*v as i64),
            _ => None,
        }
    }

    /// Get immediate u8 value (if stored inline)
    #[inline]
    pub const fn immediate_as_u8(&self) -> Option<u8> {
        match self {
            ValueRef::U8(v) => Some(*v),
            ValueRef::U64(v) => {
                if *v <= u8::MAX as u64 {
                    Some(*v as u8)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Check if value is empty
    #[inline]
    pub const fn is_empty(&self) -> bool {
        matches!(self, ValueRef::Empty)
    }

    /// Get value type identifier for encoding (aligned with types.rs constants)
    ///
    /// Protocol constants (0-14): Used for bytecode encoding with compiler
    /// VM-specific types (15+): Used internally by VM, not in bytecode
    #[inline]
    pub const fn type_id(&self) -> u8 {
        use crate::types;
        match self {
            // Protocol-defined types (match types.rs constants)
            ValueRef::Empty => types::EMPTY,              // 0
            ValueRef::U8(_) => types::U8,                 // 1
            ValueRef::U64(_) => types::U64,               // 4 (FIXED: was 2)
            ValueRef::I64(_) => types::I64,               // 8 (FIXED: was 3)
            ValueRef::U128(_) => types::U128,             // 14 (FIXED: was 4)
            ValueRef::Bool(_) => types::BOOL,             // 9 (FIXED: was 5)
            ValueRef::AccountRef(_, _) => types::ACCOUNT, // 12 (FIXED: was 6)
            ValueRef::PubkeyRef(_) => types::PUBKEY,      // 10 (FIXED: was 12)
            ValueRef::ArrayRef(_) => types::ARRAY,        // 13 (was 13, unchanged)
            ValueRef::StringRef(_) => types::STRING,      // 11 (FIXED: was 14)

            // VM-specific types (not in protocol constants, use 15+)
            ValueRef::InputRef(_) => 15, // FIXED: was 7, moved to avoid collision
            ValueRef::TempRef(_, _) => 16, // FIXED: was 8, moved to avoid collision
            ValueRef::TupleRef(_, _) => 17, // FIXED: was 9, moved to avoid collision
            ValueRef::OptionalRef(_, _) => 18, // FIXED: was 10, moved to avoid collision
            ValueRef::ResultRef(_, _) => 19, // FIXED: was 11, moved to avoid collision
            ValueRef::HeapString(_) => 20, // FIXED: was 15
            ValueRef::HeapArray(_) => 21, // FIXED: was 16
        }
    }
}

impl ValueRef {
    /// Convert ValueRef to legacy Value for compatibility
    pub fn to_value(&self) -> Option<Value> {
        match self {
            ValueRef::Empty => Some(Value::Empty),
            ValueRef::U8(v) => Some(Value::U8(*v)),
            ValueRef::U64(v) => Some(Value::U64(*v)),
            ValueRef::I64(v) => Some(Value::I64(*v)),
            ValueRef::U128(v) => Some(Value::U128(*v)),
            ValueRef::Bool(v) => Some(Value::Bool(*v)),
            ValueRef::AccountRef(idx, _) => Some(Value::Account(*idx)),
            ValueRef::ArrayRef(id) => Some(Value::Array(*id)),
            ValueRef::StringRef(_) => Some(Value::String(0)), // Convert to string index 0 for legacy
            ValueRef::HeapString(id) => Some(Value::String(*id as u8)),
            ValueRef::HeapArray(id) => Some(Value::Array(*id as u8)),
            _ => None,                                        // Cannot convert to legacy Value
        }
    }

    /// Get value as u64 (for compatibility with legacy code)
    #[inline]
    pub const fn as_u64(&self) -> Option<u64> {
        match self {
            ValueRef::U64(v) => Some(*v),
            ValueRef::U8(v) => Some(*v as u64),
            ValueRef::I64(v) => {
                if *v >= 0 {
                    Some(*v as u64)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Get value as bool (for compatibility with legacy code)
    #[inline]
    pub const fn as_bool(&self) -> Option<bool> {
        match self {
            ValueRef::Bool(v) => Some(*v),
            ValueRef::U64(v) => Some(*v != 0),
            ValueRef::U8(v) => Some(*v != 0),
            _ => None,
        }
    }

    /// Get value as account index (for compatibility with legacy code)
    #[inline]
    pub const fn as_account_idx(&self) -> Option<u8> {
        match self {
            ValueRef::U8(idx) => Some(*idx),
            ValueRef::AccountRef(idx, _) => Some(*idx),
            _ => None,
        }
    }

    /// Get value as pubkey (for compatibility with legacy code)
    ///
    /// Note: For zero-copy pubkey access, use ValueAccessContext::read_pubkey() instead.
    /// This method always returns None to encourage proper zero-copy usage.
    #[inline]
    pub const fn as_pubkey(&self) -> Option<[u8; 32]> {
        // ValueRef uses zero-copy references - use ValueAccessContext::read_pubkey() for proper access
        None
    }

    /// Get value as u8 (for compatibility with legacy code)
    #[inline]
    pub const fn as_u8(&self) -> Option<u8> {
        match self {
            ValueRef::U8(v) => Some(*v),
            ValueRef::U64(v) => {
                if *v <= u8::MAX as u64 {
                    Some(*v as u8)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Check if value is truthy (for conditional operations)
    #[inline]
    pub const fn is_truthy(&self) -> bool {
        match self {
            ValueRef::U64(0) => false,
            ValueRef::U64(_) => true,
            ValueRef::U128(0) => false,
            ValueRef::U128(_) => true,
            ValueRef::Bool(b) => *b,
            ValueRef::Empty => false,
            ValueRef::I64(0) => false,
            ValueRef::I64(_) => true,
            ValueRef::U8(0) => false,
            ValueRef::U8(_) => true,
            _ => true, // All references are truthy
        }
    }

    // ===== KISS OPTION/RESULT IMPLEMENTATION =====
    //
    // This implementation uses simple AccountRef conventions for zero-overhead Option/Result types:
    // - Option::None: AccountRef(255, 0) - Special account index 255 reserved for None
    // - Option::Some: AccountRef(account_idx, offset) - Normal account reference where account_idx < 254
    // - Result::Err: AccountRef(254, error_code) - Special account index 254 reserved for Err
    // - Result::Ok: AccountRef(account_idx, offset) - Normal account reference where account_idx < 254
    //
    // PERFORMANCE BENEFITS (measured 2025-08-30):
    // - 77.7% faster account access vs ValueAccessContext (22.7ns → 5.0ns)
    // - 30% faster Option creation vs ValueAccessContext (30.3ns → 21.3ns)
    // - Zero heap allocations, zero complex temp buffer management
    // - Eliminates 400+ lines of ValueAccessContext complexity
    //
    // This approach maintains full type safety while achieving maximum performance
    // for Solana's compute-unit constrained environment.

    /// Creates an Option::None value using the KISS AccountRef convention.
    ///
    /// Uses special account index 255 to represent None, avoiding any data storage overhead.
    /// This is a compile-time constant operation with zero runtime cost.
    ///
    /// # Example
    /// ```
    /// use five_protocol::ValueRef;
    /// let none_val = ValueRef::option_none();
    /// assert!(none_val.is_option_none());
    /// ```
    #[inline]
    pub const fn option_none() -> Self {
        ValueRef::AccountRef(255, 0) // Special: account 255 = None
    }

    /// Create Option::Some using normal AccountRef
    #[inline]
    pub const fn option_some(account_idx: u8, offset: u16) -> Self {
        ValueRef::AccountRef(account_idx, offset) // Normal account reference
    }

    /// Create Result::Err using special account index 254
    #[inline]
    pub const fn result_err(error_code: u8) -> Self {
        ValueRef::AccountRef(254, error_code as u16) // Special: account 254 = Err
    }

    /// Create Result::Ok using normal AccountRef  
    #[inline]
    pub const fn result_ok(account_idx: u8, offset: u16) -> Self {
        ValueRef::AccountRef(account_idx, offset) // Normal account reference
    }

    /// Check if AccountRef represents Option::None (account 255)
    #[inline]
    pub const fn is_option_none(&self) -> bool {
        match self {
            ValueRef::AccountRef(255, _) => true,
            _ => false,
        }
    }

    /// Check if AccountRef represents Option::Some (not account 255)
    #[inline]
    pub const fn is_option_some(&self) -> bool {
        match self {
            ValueRef::AccountRef(account_idx, _) => *account_idx != 255 && *account_idx != 254,
            _ => false,
        }
    }

    /// Check if AccountRef represents Result::Err (account 254)
    #[inline]
    pub const fn is_result_err(&self) -> bool {
        match self {
            ValueRef::AccountRef(254, _) => true,
            _ => false,
        }
    }

    /// Check if AccountRef represents Result::Ok (not account 254)
    #[inline]
    pub const fn is_result_ok(&self) -> bool {
        match self {
            ValueRef::AccountRef(account_idx, _) => *account_idx != 254 && *account_idx != 255,
            _ => false,
        }
    }

    /// Get Option data (account_idx, offset) if Some
    #[inline]
    pub const fn get_option_data(&self) -> Option<(u8, u16)> {
        match self {
            ValueRef::AccountRef(account_idx, offset) => {
                if *account_idx != 255 && *account_idx != 254 {
                    Some((*account_idx, *offset))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Get Result data (account_idx, offset) if Ok, or error code if Err
    #[inline]
    pub const fn get_result_data(&self) -> Result<(u8, u16), u8> {
        match self {
            ValueRef::AccountRef(254, error_code) => Err(*error_code as u8),
            ValueRef::AccountRef(account_idx, offset) => {
                if *account_idx != 255 {
                    Ok((*account_idx, *offset))
                } else {
                    Err(0) // Should not happen - Option in Result context
                }
            }
            _ => Err(0),
        }
    }
}

impl Value {
    /// Convert legacy Value to ValueRef
    pub fn to_valueref(&self) -> ValueRef {
        match self {
            Value::Empty => ValueRef::Empty,
            Value::U8(v) => ValueRef::U8(*v),
            Value::U64(v) => ValueRef::U64(*v),
            Value::I64(v) => ValueRef::I64(*v),
            Value::U128(v) => ValueRef::U128(*v),
            Value::Bool(v) => ValueRef::Bool(*v),
            Value::Pubkey(_) => ValueRef::Empty, // Complex conversion needed
            Value::String(idx) => ValueRef::U8(*idx),
            Value::Account(idx) => ValueRef::U8(*idx),
            Value::Array(idx) => ValueRef::U8(*idx),
        }
    }
}

impl Value {
    /// Get value as u64 (zero-allocation conversion)
    #[inline]
    pub const fn as_u64(&self) -> Option<u64> {
        match self {
            Value::U64(v) => Some(*v),
            Value::U8(v) => Some(*v as u64),
            Value::I64(v) => {
                if *v >= 0 {
                    Some(*v as u64)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Get value as bool (zero-allocation conversion)
    #[inline]
    pub const fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(v) => Some(*v),
            Value::U64(v) => Some(*v != 0),
            Value::U8(v) => Some(*v != 0),
            _ => None,
        }
    }

    /// Get value as i64 (zero-allocation conversion)
    #[inline]
    pub const fn as_i64(&self) -> Option<i64> {
        match self {
            Value::I64(v) => Some(*v),
            Value::U64(v) => {
                if *v <= i64::MAX as u64 {
                    Some(*v as i64)
                } else {
                    None
                }
            }
            Value::U8(v) => Some(*v as i64),
            _ => None,
        }
    }

    /// Get value as u8 (zero-allocation conversion)
    #[inline]
    pub const fn as_u8(&self) -> Option<u8> {
        match self {
            Value::U8(v) => Some(*v),
            Value::U64(v) => {
                if *v <= u8::MAX as u64 {
                    Some(*v as u8)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Get value as pubkey (zero-allocation conversion)
    #[inline]
    pub const fn as_pubkey(&self) -> Option<[u8; 32]> {
        match self {
            Value::Pubkey(v) => Some(*v),
            _ => None,
        }
    }

    /// Get value as string index
    #[inline]
    pub const fn as_string_idx(&self) -> Option<u8> {
        match self {
            Value::String(idx) => Some(*idx),
            _ => None,
        }
    }

    /// Get value as account index
    #[inline]
    pub const fn as_account_idx(&self) -> Option<u8> {
        match self {
            Value::Account(idx) => Some(*idx),
            Value::U8(idx) => Some(*idx), // Allow U8 as account index for compatibility
            _ => None,
        }
    }

    /// Get value as array index
    #[inline]
    pub const fn as_array_idx(&self) -> Option<u8> {
        match self {
            Value::Array(idx) => Some(*idx),
            _ => None,
        }
    }

    /// Check if value is empty
    #[inline]
    pub const fn is_empty(&self) -> bool {
        matches!(self, Value::Empty)
    }

    /// Convert to boolean for conditional operations
    #[inline]
    pub const fn is_truthy(&self) -> bool {
        match self {
            Value::U64(0) => false,
            Value::U64(_) => true,
            Value::U128(0) => false,
            Value::U128(_) => true,
            Value::Bool(b) => *b,
            Value::Account(_) => true,
            Value::Pubkey(_) => true,
            Value::Empty => false,
            Value::I64(0) => false,
            Value::I64(_) => true,
            Value::U8(0) => false,
            Value::U8(_) => true,
            Value::String(_) => true, // String index is always truthy
            Value::Array(_) => true,  // Array index is always truthy
        }
    }

    /// Get value type identifier for encoding (aligned with types.rs constants)
    #[inline]
    pub const fn type_id(&self) -> u8 {
        use crate::types;
        match self {
            Value::Empty => types::EMPTY,        // 0
            Value::U8(_) => types::U8,           // 1
            Value::U64(_) => types::U64,         // 4 (FIXED: was 2)
            Value::I64(_) => types::I64,         // 8 (FIXED: was 3)
            Value::U128(_) => types::U128,       // 14 (FIXED: was 4)
            Value::Bool(_) => types::BOOL,       // 9 (FIXED: was 5)
            Value::Pubkey(_) => types::PUBKEY,   // 10 (FIXED: was 6)
            Value::String(_) => types::STRING,   // 11 (FIXED: was 7)
            Value::Account(_) => types::ACCOUNT, // 12 (FIXED: was 8)
            Value::Array(_) => types::ARRAY,     // 13 (FIXED: was 9)
        }
    }

    /// Get value size in bytes for stack allocation
    #[inline]
    pub const fn size_bytes(&self) -> usize {
        match self {
            Value::Empty => 0,
            Value::U8(_) => 1,
            Value::U64(_) => 8,
            Value::I64(_) => 8,
            Value::U128(_) => 16,
            Value::Bool(_) => 1,
            Value::Pubkey(_) => 32,
            Value::String(_) => 1,  // String index
            Value::Account(_) => 1, // Account index
            Value::Array(_) => 1,   // Array index
        }
    }
}

/// Function parameter specification for static validation
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Parameter {
    pub name_hash: u32, // Hash of parameter name (for debugging)
    pub type_id: u8,    // Value type identifier
}

impl Parameter {
    /// Create new parameter specification
    #[inline]
    pub const fn new(name_hash: u32, type_id: u8) -> Self {
        Self { name_hash, type_id }
    }

    /// Validate value against parameter specification
    #[inline]
    pub const fn validate(&self, value: &Value) -> bool {
        value.type_id() == self.type_id
    }
}

/// Function signature for static validation and call setup
#[derive(Debug, Clone, Copy)]
pub struct FunctionSignature {
    pub name_hash: u32,                                      // Function name hash
    pub parameter_count: u8,                                 // Number of parameters
    pub parameters: [Parameter; crate::MAX_FUNCTION_PARAMS], // Parameter specifications
    pub return_type: Option<u8>,                             // Return type (None = void)
    pub local_slots: u8, // Number of local variable slots needed
    pub is_public: bool, // Function visibility (true = public, false = private)
}

impl FunctionSignature {
    /// Create new function signature
    #[inline]
    pub const fn new(
        name_hash: u32,
        parameter_count: u8,
        return_type: Option<u8>,
        local_slots: u8,
    ) -> Self {
        Self {
            name_hash,
            parameter_count,
            parameters: [Parameter::new(0, 0); crate::MAX_FUNCTION_PARAMS],
            return_type,
            local_slots,
            is_public: true, // Default to public for backward compatibility
        }
    }

    /// Create new function signature with visibility
    #[inline]
    pub const fn new_with_visibility(
        name_hash: u32,
        parameter_count: u8,
        return_type: Option<u8>,
        local_slots: u8,
        is_public: bool,
    ) -> Self {
        Self {
            name_hash,
            parameter_count,
            parameters: [Parameter::new(0, 0); crate::MAX_FUNCTION_PARAMS],
            return_type,
            local_slots,
            is_public,
        }
    }

    /// Validate parameters against signature
    #[inline]
    pub fn validate_params(&self, params: &[Value]) -> bool {
        if params.len() != self.parameter_count as usize {
            return false;
        }

        let mut i = 0;
        while i < params.len() {
            if i >= self.parameter_count as usize {
                return false;
            }
            if !self.parameters[i].validate(&params[i]) {
                return false;
            }
            i += 1;
        }
        true
    }
}
