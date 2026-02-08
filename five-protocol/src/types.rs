//! Type identifier constants for bytecode generation.

/// Type identifier for empty values
pub const EMPTY: u8 = 0;

/// Type identifier for 8-bit unsigned integers
pub const U8: u8 = 1;

/// Type identifier for 16-bit unsigned integers
pub const U16: u8 = 2;

/// Type identifier for 32-bit unsigned integers
pub const U32: u8 = 3;

/// Type identifier for 64-bit unsigned integers
pub const U64: u8 = 4;

/// Type identifier for 128-bit unsigned integers
pub const U128: u8 = 14;

/// Type identifier for 8-bit signed integers
pub const I8: u8 = 5;

/// Type identifier for 16-bit signed integers
pub const I16: u8 = 6;

/// Type identifier for 32-bit signed integers
pub const I32: u8 = 7;

/// Type identifier for 64-bit signed integers
pub const I64: u8 = 8;

/// Type identifier for boolean values
pub const BOOL: u8 = 9;

/// Type identifier for public keys
pub const PUBKEY: u8 = 10;

/// Type identifier for string references
pub const STRING: u8 = 11;

/// Type identifier for account references
pub const ACCOUNT: u8 = 12;

/// Type identifier for array references
pub const ARRAY: u8 = 13;

// Importable account format.

/// Magic bytes for importable Five accounts ("FIVE" in ASCII)
pub const FIVE_IMPORT_MAGIC: [u8; 4] = [0x46, 0x49, 0x56, 0x45]; // "FIVE"

/// Header for importable account format
/// Layout: [MAGIC][HEADER][FUNCTION_TABLE][BYTECODE]
#[repr(C, packed)]
#[derive(Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ImportableAccountHeader {
    /// Magic bytes to identify Five importable accounts
    pub magic: [u8; 4],
    /// Number of functions in this account (1 to 5000+)
    pub function_count: u32,
    /// Offset from start of account data to function table
    pub function_table_offset: u32,
    /// Offset from start of account data to bytecode blob
    pub bytecode_offset: u32,
    /// Total size of bytecode in bytes
    pub bytecode_size: u32,
}

/// Function entry in the importable account function table.
#[repr(C, packed)]
#[derive(Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ImportableFunctionEntry {
    /// Hash of function name for O(1) lookup (FNV-1a hash)
    pub name_hash: u32,
    /// Offset within bytecode blob where function starts
    pub bytecode_offset: u32,
    /// Size of function in bytes
    pub function_size: u32,
    /// Function flags (public/private, parameter count, etc.)
    pub flags: u32,
}

/// Function flags for ImportableFunctionEntry
pub mod function_flags {
    /// Function is publicly callable (bit 0)
    pub const PUBLIC: u32 = 1 << 0;
    /// Function has return value (bit 1)
    pub const HAS_RETURN: u32 = 1 << 1;
    /// Parameter count mask (bits 8-15, supports 0-255 parameters)
    pub const PARAM_COUNT_MASK: u32 = 0xFF << 8;

    /// Extract parameter count from flags
    pub const fn param_count(flags: u32) -> u8 {
        ((flags & PARAM_COUNT_MASK) >> 8) as u8
    }

    /// Create flags with parameter count
    pub const fn with_param_count(count: u8) -> u32 {
        (count as u32) << 8
    }
}

/// FNV-1a hash for function names.
pub const fn hash_function_name(name: &[u8]) -> u32 {
    const FNV_OFFSET_BASIS: u32 = 2166136261;
    const FNV_PRIME: u32 = 16777619;

    let mut hash = FNV_OFFSET_BASIS;
    let mut i = 0;
    while i < name.len() {
        hash ^= name[i] as u32;
        hash = hash.wrapping_mul(FNV_PRIME);
        i += 1;
    }
    hash
}

impl ImportableAccountHeader {
    /// Create a new header with the given parameters.
    pub const fn new(
        function_count: u32,
        function_table_offset: u32,
        bytecode_offset: u32,
        bytecode_size: u32,
    ) -> Self {
        Self {
            magic: FIVE_IMPORT_MAGIC,
            function_count,
            function_table_offset,
            bytecode_offset,
            bytecode_size,
        }
    }

    /// Validate that this header has the correct magic bytes.
    pub const fn is_valid(&self) -> bool {
        self.magic[0] == FIVE_IMPORT_MAGIC[0]
            && self.magic[1] == FIVE_IMPORT_MAGIC[1]
            && self.magic[2] == FIVE_IMPORT_MAGIC[2]
            && self.magic[3] == FIVE_IMPORT_MAGIC[3]
    }
}

impl ImportableFunctionEntry {
    /// Create a new function entry.
    pub const fn new(name_hash: u32, bytecode_offset: u32, function_size: u32, flags: u32) -> Self {
        Self {
            name_hash,
            bytecode_offset,
            function_size,
            flags,
        }
    }

    /// Check if function is public.
    pub const fn is_public(&self) -> bool {
        (self.flags & function_flags::PUBLIC) != 0
    }

    /// Get parameter count for this function.
    pub const fn param_count(&self) -> u8 {
        function_flags::param_count(self.flags)
    }
}
