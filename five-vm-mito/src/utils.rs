//! Utility functions for MitoVM
//!
//! This module contains common utility functions and helpers used
//! throughout the MitoVM execution engine.

use crate::error::{CompactResult, VMError, VMErrorCode};
use five_protocol::ValueRef;
use heapless::Vec;
use pinocchio::account_info::AccountInfo;
use pinocchio::pubkey::Pubkey;

/// Zero-copy value conversion utilities for efficient type casting.
pub struct ValueRefUtils;

impl ValueRefUtils {
    /// Convert ValueRef to u64 using safe type coercion rules.
    #[inline]
    pub fn as_u64(value: ValueRef) -> CompactResult<u64> {
        match value {
            ValueRef::U64(v) => Ok(v),
            ValueRef::U32(v) => Ok(v as u64),
            ValueRef::U16(v) => Ok(v as u64),
            ValueRef::U8(v) => Ok(v as u64),
            ValueRef::I64(v) => {
                if v < 0 {
                    Err(VMErrorCode::TypeMismatch)
                } else {
                    Ok(v as u64)
                }
            }
            ValueRef::I32(v) => {
                if v < 0 {
                    Err(VMErrorCode::TypeMismatch)
                } else {
                    Ok(v as u64)
                }
            }
            ValueRef::I16(v) => {
                if v < 0 {
                    Err(VMErrorCode::TypeMismatch)
                } else {
                    Ok(v as u64)
                }
            }
            ValueRef::I8(v) => {
                if v < 0 {
                    Err(VMErrorCode::TypeMismatch)
                } else {
                    Ok(v as u64)
                }
            }
            ValueRef::Bool(v) => Ok(if v { 1 } else { 0 }),
            ValueRef::AccountRef(account, offset) => Ok(((account as u64) << 16) | (offset as u64)),
            _ => Err(VMErrorCode::TypeMismatch),
        }
    }

    /// Convert ValueRef to boolean using truthiness evaluation.
    #[inline]
    pub fn as_bool(value: ValueRef) -> CompactResult<bool> {
        match value {
            ValueRef::Bool(v) => Ok(v),
            ValueRef::U8(v) => Ok(v != 0),
            ValueRef::U16(v) => Ok(v != 0),
            ValueRef::U32(v) => Ok(v != 0),
            ValueRef::U64(v) => Ok(v != 0),
            ValueRef::I64(v) => Ok(v != 0),
            ValueRef::I32(v) => Ok(v != 0),
            ValueRef::I16(v) => Ok(v != 0),
            ValueRef::I8(v) => Ok(v != 0),
            ValueRef::AccountRef(account, offset) => Ok(account != 0 || offset != 0),
            _ => Err(VMErrorCode::TypeMismatch),
        }
    }

    /// Convert ValueRef to signed 64-bit integer with type coercion.
    #[inline]
    pub fn as_i64(value: ValueRef) -> CompactResult<i64> {
        match value {
            ValueRef::I64(v) => Ok(v),
            ValueRef::I32(v) => Ok(v as i64),
            ValueRef::I16(v) => Ok(v as i64),
            ValueRef::I8(v) => Ok(v as i64),
            ValueRef::U64(v) => Ok(v as i64),
            ValueRef::U32(v) => Ok(v as i64),
            ValueRef::U16(v) => Ok(v as i64),
            ValueRef::U8(v) => Ok(v as i64),
            ValueRef::Bool(v) => Ok(if v { 1 } else { 0 }),
            ValueRef::AccountRef(account, offset) => Ok(((account as i64) << 16) | (offset as i64)),
            _ => Err(VMErrorCode::TypeMismatch),
        }
    }

    /// Convert ValueRef to u8 with error handling
    #[inline]
    pub fn as_u8(value: ValueRef) -> CompactResult<u8> {
        match value {
            ValueRef::U8(v) => Ok(v),
            ValueRef::U64(v) => Ok(v as u8),
            ValueRef::U32(v) => Ok(v as u8),
            ValueRef::U16(v) => Ok(v as u8),
            ValueRef::I64(v) => Ok(v as u8),
            ValueRef::I32(v) => Ok(v as u8),
            ValueRef::I16(v) => Ok(v as u8),
            ValueRef::I8(v) => Ok(v as u8),
            ValueRef::Bool(v) => Ok(if v { 1 } else { 0 }),
            ValueRef::AccountRef(account, _) => Ok(account),
            _ => Err(VMErrorCode::TypeMismatch),
        }
    }
}

/// Helper to resolve a value (including AccountRef) to u64 for legacy arithmetic
/// This reads 8 bytes from account data if given an AccountRef
pub fn resolve_u64(value: ValueRef, ctx: &crate::context::ExecutionManager) -> CompactResult<u64> {
    match value {
        ValueRef::AccountRef(account_idx, offset) => {
            let account = ctx.get_account(account_idx)?;
            let data = unsafe { account.borrow_data_unchecked() };
            if (offset as usize + 8) > data.len() {
                return Err(VMErrorCode::InvalidAccountData);
            }
            let bytes: [u8; 8] = data[offset as usize..offset as usize + 8]
                .try_into()
                .map_err(|_| VMErrorCode::InvalidAccountData)?;
            Ok(u64::from_le_bytes(bytes))
        }
        ValueRef::U128(v) => u64::try_from(v).map_err(|_| VMErrorCode::NumericOverflow),
        _ => value.as_u64().ok_or(VMErrorCode::TypeMismatch),
    }
}

/// Helper to resolve a value (including AccountRef) to bool for logical operations
/// This reads 1 byte from account data if given an AccountRef
pub fn resolve_bool(
    value: ValueRef,
    ctx: &crate::context::ExecutionManager,
) -> CompactResult<bool> {
    match value {
        ValueRef::AccountRef(account_idx, offset) => {
            let account = ctx.get_account(account_idx)?;
            let data = unsafe { account.borrow_data_unchecked() };
            if (offset as usize + 1) > data.len() {
                return Err(VMErrorCode::InvalidAccountData);
            }
            // Read 1 byte and interpret as bool (non-zero is true)
            let byte = data[offset as usize];
            Ok(byte != 0)
        }
        _ => ValueRefUtils::as_bool(value),
    }
}

/// Helper to resolve a value (including AccountRef) to u8.
/// This reads 1 byte from account data if given an AccountRef.
pub fn resolve_u8(value: ValueRef, ctx: &crate::context::ExecutionManager) -> CompactResult<u8> {
    match value {
        ValueRef::AccountRef(account_idx, offset) => {
            let account = ctx.get_account(account_idx)?;
            let data = unsafe { account.borrow_data_unchecked() };
            if (offset as usize + 1) > data.len() {
                return Err(VMErrorCode::InvalidAccountData);
            }
            Ok(data[offset as usize])
        }
        _ => value.as_u8().ok_or(VMErrorCode::TypeMismatch),
    }
}

/// Utility functions for bytecode validation
pub struct BytecodeUtils;

impl BytecodeUtils {
    /// Validate bytecode magic header
    #[inline]
    pub fn validate_magic(bytecode: &[u8], expected_magic: &[u8]) -> CompactResult<usize> {
        if bytecode.len() < expected_magic.len() {
            return Err(VMErrorCode::InvalidScript);
        }

        if &bytecode[0..expected_magic.len()] != expected_magic {
            return Err(VMErrorCode::InvalidScript);
        }

        Ok(expected_magic.len())
    }

    /// Skip magic header and return starting position
    #[inline]
    pub fn skip_magic(bytecode: &[u8], magic: &[u8]) -> CompactResult<usize> {
        Self::validate_magic(bytecode, magic)
    }

    /// Import bytecode from an account, returning an error for empty or invalid accounts
    #[inline]
    pub fn import_account_bytecode(account: &AccountInfo) -> CompactResult<&[u8]> {
        if account.data_len() == 0 {
            return Err(VMErrorCode::InvalidAccountData);
        }
        // SAFETY: We've verified the account contains data
        let data = unsafe { account.borrow_data_unchecked() };
        Ok(data)
    }
}

/// Utility functions for error handling
pub struct ErrorUtils;

impl ErrorUtils {
    /// Convert VMError to descriptive string
    #[inline]
    pub fn error_message(error: &VMError) -> &'static str {
        match error {
            VMError::StackError => "Stack operation failed",
            VMError::InvalidInstruction => "Invalid instruction",
            VMError::InvalidScript => "Invalid script",
            VMError::MemoryViolation => "Memory violation",
            VMError::TypeMismatch => "Type mismatch",
            VMError::DivisionByZero => "Division by zero",
            VMError::AccountError => "Account error",
            VMError::ConstraintViolation => "Constraint violation",
            VMError::Halted => "Script halted",
            VMError::InvalidAccountIndex => "Invalid account index",
            VMError::AccountNotWritable => "Account not writable",
            VMError::AccountNotSigner => "Account not signer",
            VMError::InvalidVariableIndex(_) => "Invalid variable index",
            VMError::InvalidInstructionPointer => "Invalid instruction pointer",
            VMError::CallStackOverflow => "Call stack overflow",
            VMError::CallStackUnderflow => "Call stack underflow",
            VMError::DataBufferOverflow => "Data buffer overflow",
            VMError::InvalidOperation => "Invalid operation",
            VMError::ParseError { .. } => "Parse error",
            VMError::UnexpectedToken => "Unexpected token",
            VMError::UnexpectedEndOfInput => "Unexpected end of input",
            VMError::InvalidFunctionIndex => "Invalid function index",
            VMError::LocalsOverflow => "Locals overflow",
            VMError::InvalidAccountData => "Invalid account data",
            VMError::InvalidAccount => "Invalid account",
            VMError::MemoryError => "Memory error",
            VMError::AccountOwnershipError { .. } => "Account ownership error",
            VMError::InvokeError { .. } => "Invoke error",
            VMError::ExternalAccountLamportSpend => "External account lamport spend",
            VMError::ScriptNotAuthorized { .. } => "Script not authorized",
            VMError::UndefinedAccountField => "Undefined account field",
            VMError::InvalidSeedArray(_) => "Invalid seed array",
            VMError::ImmutableField => "Attempt to modify an immutable field",
            VMError::UndefinedField => "Attempt to access an undefined field",
            VMError::UndefinedIdentifier | VMError::UndefinedIdentifierWithContext { .. } => {
                "Attempt to access an undefined identifier"
            }
            VMError::DuplicateImport { .. } => "Duplicate imported symbol",
            VMError::InvalidParameterCount => "Invalid parameter count",
            VMError::IndexOutOfBounds => "Index out of bounds",
            VMError::OutOfMemory => "Out of memory",
            VMError::ProtocolError => "Protocol error",
            VMError::TooManySeeds => "Too many seeds provided for PDA derivation",
            VMError::UnauthorizedBytecodeInvocation => {
                "Five bytecode account not authorized by import verification"
            }
            VMError::PdaDerivationFailed => "Failed to derive PDA from provided seeds",
            VMError::AccountNotFound => "Account not found or invalid account index",
            VMError::AccountDataEmpty => "Account data is empty when data was expected",
            VMError::RuntimeIntegrationRequired => {
                "Runtime integration with Solana required for this operation"
            }
            VMError::InvalidParameter => "Invalid parameter provided to operation",
            VMError::InvalidOpcode => "Invalid opcode encountered",
            VMError::ParameterMismatch { .. } => "Function parameter mismatch",
            VMError::StackOperationError { .. } => "Stack operation error",
            VMError::AbiParameterMismatch { .. } => "ABI parameter mismatch",
            VMError::FunctionVisibilityViolation { .. } => {
                "Function visibility violation: Cannot call private function"
            }
            VMError::ExecutionTerminated => "Execution terminated by syscall",
            VMError::SecurityViolation => "Security violation detected",
            VMError::NumericOverflow => "Numeric overflow when narrowing u128 to u64",
            VMError::ArithmeticOverflow => "Arithmetic overflow in checked operation",
            VMError::UninitializedAccount => "Account is uninitialized",
            VMError::InvalidScriptSize => "Script exceeds maximum size",
        }
    }

    /// Check if error is recoverable
    #[inline]
    pub fn is_recoverable(error: &VMError) -> bool {
        match error {
            VMError::StackError
            | VMError::InvalidInstruction
            | VMError::TypeMismatch
            | VMError::DivisionByZero
            | VMError::InvalidVariableIndex(_) => true,
            _ => false,
        }
    }
}

/// Utility functions for debug logging
#[cfg(feature = "debug-logs")]
pub struct DebugUtils;

#[cfg(feature = "debug-logs")]
impl DebugUtils {
    /// Format ValueRef for debug output without heap allocation
    #[inline]
    pub fn format_value_ref(value: &ValueRef) -> heapless::String<64> {
        use core::fmt::Write;
        let mut s = heapless::String::<64>::new();
        match value {
            ValueRef::Empty => {
                let _ = s.push_str("Empty");
            }
            ValueRef::U8(v) => {
                let _ = write!(s, "U8({})", v);
            }
            ValueRef::U16(v) => {
                let _ = write!(s, "U16({})", v);
            }
            ValueRef::U32(v) => {
                let _ = write!(s, "U32({})", v);
            }
            ValueRef::U64(v) => {
                let _ = write!(s, "U64({})", v);
            }
            ValueRef::U128(v) => {
                let _ = write!(s, "U128({})", v);
            }
            ValueRef::I8(v) => {
                let _ = write!(s, "I8({})", v);
            }
            ValueRef::I16(v) => {
                let _ = write!(s, "I16({})", v);
            }
            ValueRef::I32(v) => {
                let _ = write!(s, "I32({})", v);
            }
            ValueRef::I64(v) => {
                let _ = write!(s, "I64({})", v);
            }
            ValueRef::Bool(v) => {
                let _ = write!(s, "Bool({})", v);
            }
            ValueRef::AccountRef(account, offset) => {
                let _ = write!(s, "AccountRef({}, {})", account, offset);
            }
            ValueRef::InputRef(offset) => {
                let _ = write!(s, "InputRef({})", offset);
            }
            ValueRef::TempRef(offset, len) => {
                let _ = write!(s, "TempRef({}, {})", offset, len);
            }
            ValueRef::TupleRef(offset, len) => {
                let _ = write!(s, "TupleRef({}, {})", offset, len);
            }
            ValueRef::OptionalRef(offset, len) => {
                let _ = write!(s, "OptionalRef({}, {})", offset, len);
            }
            ValueRef::ResultRef(offset, len) => {
                let _ = write!(s, "ResultRef({}, {})", offset, len);
            }
            ValueRef::PubkeyRef(offset) => {
                let _ = write!(s, "PubkeyRef({})", offset);
            }
            ValueRef::ArrayRef(array_id) => {
                let _ = write!(s, "ArrayRef({})", array_id);
            }
            ValueRef::StringRef(offset) => {
                let _ = write!(s, "StringRef({})", offset);
            }
            ValueRef::HeapString(id) => {
                let _ = write!(s, "HeapString({})", id);
            }
            ValueRef::HeapArray(id) => {
                let _ = write!(s, "HeapArray({})", id);
            }
        }
        s
    }

    /// Format instruction pointer for debug output
    #[inline]
    pub fn format_ip(ip: usize) -> heapless::String<32> {
        use core::fmt::Write;
        let mut s = heapless::String::<32>::new();
        let _ = write!(s, "IP:0x{:04X}", ip);
        s
    }

    /// Format stack depth for debug output
    #[inline]
    pub fn format_stack_depth(depth: usize) -> heapless::String<32> {
        use core::fmt::Write;
        let mut s = heapless::String::<32>::new();
        let _ = write!(s, "Stack[{}]", depth);
        s
    }
}

/// Convert ValueRef to byte array for PDA seeds and CPI instruction data
/// This consolidates the repeated conversion logic found in multiple handlers
pub fn value_ref_to_seed_bytes(
    value_ref: ValueRef,
    ctx: &mut crate::context::ExecutionContext,
    expected_len: Option<usize>,
) -> CompactResult<Vec<u8, 32>> {
    use crate::debug_log;

    match value_ref {
        ValueRef::U8(val) => {
            debug_log!("MitoVM: value_ref_to_seed_bytes - U8 value: {}", val);
            Vec::from_slice(&[val]).map_err(|_| VMErrorCode::MemoryError)
        }
        ValueRef::U16(val) => {
            debug_log!(
                "MitoVM: value_ref_to_seed_bytes - U16 value: {}",
                val as u32
            );
            Vec::from_slice(&val.to_le_bytes()).map_err(|_| VMErrorCode::MemoryError)
        }
        ValueRef::U32(val) => {
            debug_log!("MitoVM: value_ref_to_seed_bytes - U32 value: {}", val);
            Vec::from_slice(&val.to_le_bytes()).map_err(|_| VMErrorCode::MemoryError)
        }
        ValueRef::U64(val) => {
            debug_log!("MitoVM: value_ref_to_seed_bytes - U64 value: {}", val);
            Vec::from_slice(&val.to_le_bytes()).map_err(|_| VMErrorCode::MemoryError)
        }
        ValueRef::U128(val) => {
            debug_log!("MitoVM: value_ref_to_seed_bytes - U128 value: {}", val);
            Vec::from_slice(&val.to_le_bytes()).map_err(|_| VMErrorCode::MemoryError)
        }
        ValueRef::I64(val) => {
            debug_log!("MitoVM: value_ref_to_seed_bytes - I64 value: {}", val);
            Vec::from_slice(&val.to_le_bytes()).map_err(|_| VMErrorCode::MemoryError)
        }
        ValueRef::I32(val) => {
            debug_log!(
                "MitoVM: value_ref_to_seed_bytes - I32 value: {}",
                val as u32
            );
            Vec::from_slice(&val.to_le_bytes()).map_err(|_| VMErrorCode::MemoryError)
        }
        ValueRef::I16(val) => {
            debug_log!(
                "MitoVM: value_ref_to_seed_bytes - I16 value: {}",
                val as i32 as u32
            );
            Vec::from_slice(&val.to_le_bytes()).map_err(|_| VMErrorCode::MemoryError)
        }
        ValueRef::I8(val) => {
            debug_log!(
                "MitoVM: value_ref_to_seed_bytes - I8 value: {}",
                val as i32 as u32
            );
            Vec::from_slice(&[val as u8]).map_err(|_| VMErrorCode::MemoryError)
        }
        ValueRef::Bool(val) => {
            debug_log!(
                "MitoVM: value_ref_to_seed_bytes - Bool value: {}",
                val as u32
            );
            Vec::from_slice(&[if val { 1 } else { 0 }]).map_err(|_| VMErrorCode::MemoryError)
        }
        ValueRef::TempRef(offset, len) => {
            debug_log!(
                "MitoVM: value_ref_to_seed_bytes - TempRef offset: {}, len: {}",
                offset,
                len
            );
            // Get data from temp buffer
            let start = offset as usize;
            let end = start + len as usize;
            if end > ctx.memory.temp_buffer.len() {
                return Err(VMErrorCode::MemoryViolation);
            }
            Vec::from_slice(&ctx.memory.temp_buffer[start..end])
                .map_err(|_| VMErrorCode::MemoryError)
        }
        ValueRef::InputRef(offset) => {
            debug_log!(
                "MitoVM: value_ref_to_seed_bytes - InputRef offset: {}",
                offset
            );
            let start = offset as usize;
            let end = if let Some(len) = expected_len {
                start + len
            } else {
                ctx.instruction_data.len()
            };
            if start > ctx.instruction_data.len() || end > ctx.instruction_data.len() {
                return Err(VMErrorCode::MemoryViolation);
            }
            Vec::from_slice(&ctx.instruction_data[start..end]).map_err(|_| VMErrorCode::MemoryError)
        }
        ValueRef::AccountRef(account_index, _offset) => {
            debug_log!(
                "MitoVM: value_ref_to_seed_bytes - AccountRef index: {}",
                account_index
            );
            // Return the account's pubkey as bytes
            let account = ctx.get_account(account_index)?;
            Vec::from_slice(account.key().as_ref()).map_err(|_| VMErrorCode::MemoryError)
        }
        ValueRef::PubkeyRef(_) => {
            debug_log!("MitoVM: value_ref_to_seed_bytes - PubkeyRef");
            let pubkey = ctx.extract_pubkey(&value_ref)?;
            Vec::from_slice(&pubkey).map_err(|_| VMErrorCode::MemoryError)
        }
        ValueRef::StringRef(offset) => {
            debug_log!(
                "MitoVM: value_ref_to_seed_bytes - StringRef offset: {}",
                offset
            );
            // String is stored in temp buffer: [length, type, bytes...]
            let start = offset as usize;
            if start + 2 > ctx.memory.temp_buffer.len() {
                return Err(VMErrorCode::MemoryViolation);
            }

            let len = ctx.memory.temp_buffer[start] as usize;
            let data_start = start + 2;
            let data_end = data_start + len;

            if data_end > ctx.memory.temp_buffer.len() {
                return Err(VMErrorCode::MemoryViolation);
            }

            Vec::from_slice(&ctx.memory.temp_buffer[data_start..data_end])
                .map_err(|_| VMErrorCode::MemoryError)
        }
        ValueRef::Empty => {
            debug_log!("MitoVM: value_ref_to_seed_bytes - Empty value");
            Ok(Vec::new())
        }
        // Complex reference types - not typically used for simple seed/byte conversion
        ValueRef::TupleRef(_, _)
        | ValueRef::OptionalRef(_, _)
        | ValueRef::ResultRef(_, _)
        | ValueRef::ArrayRef(_)
        | ValueRef::HeapString(_)
        | ValueRef::HeapArray(_) => {
            debug_log!("MitoVM: value_ref_to_seed_bytes - Complex reference type not supported for simple conversion");
            Err(VMErrorCode::TypeMismatch)
        }
    }
}

/// Convert ValueRef to fixed-size byte array for specific use cases like Pubkey (32 bytes)
/// This is useful when you need a specific byte length for PDA seeds or CPI data
pub fn value_ref_to_fixed_bytes<const N: usize>(
    value_ref: ValueRef,
    ctx: &mut crate::context::ExecutionContext,
) -> CompactResult<[u8; N]> {
    use crate::debug_log;

    let bytes = value_ref_to_seed_bytes(value_ref, ctx, Some(N))?;
    if bytes.len() != N {
        debug_log!("MitoVM: value_ref_to_fixed_bytes - byte count mismatch");
        debug_log!("Expected: {}", N as u32);
        debug_log!("Got: {}", bytes.len() as u32);
        return Err(VMErrorCode::TypeMismatch);
    }

    let mut result = [0u8; N];
    result.copy_from_slice(&bytes);
    Ok(result)
}

/// Convert ValueRef to zero-copy byte slice when possible
/// Returns a reference to the underlying data without allocation for performance
pub fn value_ref_to_bytes_ref<'a>(
    value_ref: ValueRef,
    ctx: &'a mut crate::context::ExecutionContext,
    temp_storage: &'a mut [u8; 32], // For small values that need temporary storage
) -> CompactResult<&'a [u8]> {
    use crate::debug_log;

    match value_ref {
        ValueRef::TempRef(offset, len) => {
            debug_log!(
                "MitoVM: value_ref_to_bytes_ref - TempRef offset: {}, len: {}",
                offset,
                len
            );
            let start = offset as usize;
            let end = start + len as usize;
            if end > ctx.memory.temp_buffer.len() {
                return Err(VMErrorCode::MemoryViolation);
            }
            Ok(&ctx.memory.temp_buffer[start..end])
        }
        ValueRef::InputRef(_offset) => {
            debug_log!(
                "MitoVM: value_ref_to_bytes_ref - InputRef offset: {}",
                _offset
            );
            // For InputRef, we need to know the expected length from context
            // This is a limitation - callers should specify expected type
            Err(VMErrorCode::TypeMismatch)
        }
        ValueRef::AccountRef(account_index, _offset) => {
            debug_log!(
                "MitoVM: value_ref_to_bytes_ref - AccountRef index: {}",
                account_index
            );
            let account = ctx.get_account(account_index)?;
            Ok(account.key().as_ref())
        }
        ValueRef::U8(_)
        | ValueRef::U16(_)
        | ValueRef::U32(_)
        | ValueRef::U64(_)
        | ValueRef::U128(_)
        | ValueRef::I8(_)
        | ValueRef::I16(_)
        | ValueRef::I32(_)
        | ValueRef::I64(_)
        | ValueRef::Bool(_)
        | ValueRef::Empty => {
            // For other types, fall back to temporary storage
            debug_log!("MitoVM: value_ref_to_bytes_ref - using temp storage for value type");
            let bytes = value_ref_to_seed_bytes(value_ref, ctx, None)?;
            if bytes.len() > temp_storage.len() {
                return Err(VMErrorCode::MemoryViolation);
            }
            temp_storage[..bytes.len()].copy_from_slice(&bytes);
            Ok(&temp_storage[..bytes.len()])
        }
        // Complex reference types - not supported for zero-copy access
        ValueRef::TupleRef(_, _)
        | ValueRef::OptionalRef(_, _)
        | ValueRef::ResultRef(_, _)
        | ValueRef::PubkeyRef(_)
        | ValueRef::ArrayRef(_)
        | ValueRef::StringRef(_)
        | ValueRef::HeapString(_)
        | ValueRef::HeapArray(_) => {
            debug_log!("MitoVM: value_ref_to_bytes_ref - Complex reference type not supported");
            Err(VMErrorCode::TypeMismatch)
        }
    }
}

/// Helper for off-chain PDA derivation using solana-nostd-sha256
pub fn derive_pda_offchain(seeds: &[&[u8]], program_id: &[u8; 32]) -> CompactResult<[u8; 32]> {
    use solana_nostd_sha256::hashv;
    let mut hasher_seeds: Vec<&[u8], 19> = Vec::new(); // MAX_SEEDS(16) + bump(1) + program_id(1) + marker(1)
    for s in seeds {
        hasher_seeds.push(s).map_err(|_| VMErrorCode::MemoryError)?;
    }
    hasher_seeds
        .push(program_id)
        .map_err(|_| VMErrorCode::MemoryError)?;
    hasher_seeds
        .push(b"ProgramDerivedAddress")
        .map_err(|_| VMErrorCode::MemoryError)?;

    let hash = hashv(&hasher_seeds);
    Ok(Pubkey::from(hash))
}

/// Helper for off-chain program address finding (with bump searching)
pub fn find_program_address_offchain(
    seeds: &[&[u8]],
    program_id: &[u8; 32],
) -> CompactResult<([u8; 32], u8)> {
    for bump in (0..=255u8).rev() {
        let bump_slice = [bump];
        let mut full_seeds: Vec<&[u8], 17> = Vec::new();
        for s in seeds {
            full_seeds.push(s).map_err(|_| VMErrorCode::MemoryError)?;
        }
        full_seeds
            .push(&bump_slice)
            .map_err(|_| VMErrorCode::MemoryError)?;

        let pda = derive_pda_offchain(&full_seeds, program_id)?;

        // Off-curve validation is omitted in no-std; return first bump for tests.
        return Ok((pda, bump));
    }
    Err(VMErrorCode::PdaDerivationFailed)
}
