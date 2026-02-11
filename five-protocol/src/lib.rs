//! Protocol specification for Five VMs and compilers.

#![no_std]

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

pub mod bytecode_builder;
pub mod call_convention;
pub mod opcodes;
pub mod parser;
pub mod transport;
pub mod types;
pub mod value;

#[cfg(feature = "test-fixtures")]
pub mod test_fixtures;

pub use bytecode_builder::*;
pub use call_convention::*;
pub use opcodes::*;
pub use parser::*;
pub use transport::*;
pub use types::*;
pub use value::*;
// Five script header V3.

/// Magic: "5IVE" = 0x45564935
pub const SCRIPT_MAGIC: u32 = 0x45564935;

/// Version 3
pub const SCRIPT_VERSION: u8 = 3;

/// Opcode surface compatibility marker.
/// `1` designates the first locked 5ive opcode specification.
pub const OPCODE_SPEC_VERSION: u16 = 1;

// Keep protocol/runtime aligned with on-chain VM stack limits.
pub const MAX_CALL_DEPTH: usize = 8;
pub const MAX_FUNCTION_PARAMS: usize = 24;
pub const MAX_LOCALS: usize = 32;
pub const MAX_FUNCTIONS: usize = 255;

// Optimized header constants
pub const FIVE_HEADER_OPTIMIZED_SIZE: usize = 10;
pub const FIVE_MAGIC: [u8; 4] = *b"5IVE";
pub const FIVE_DEPLOY_MAGIC: [u8; 4] = *b"5DEP";
pub const TEMP_BUFFER_SIZE: usize = 512;
pub const MAX_SCRIPT_SIZE: usize = 10_000;

// Feature flags for header
pub const FEATURE_FUSED_BRANCH: u32 = 1 << 0;
pub const FEATURE_NO_VALIDATION: u32 = 1 << 1;
pub const FEATURE_MINIMAL_ERRORS: u32 = 1 << 2;
pub const FEATURE_COLD_START_OPT: u32 = 1 << 3;
pub const FEATURE_IMPORT_VERIFICATION: u32 = 1 << 4;  // Import verification metadata present
pub const FEATURE_FUNCTION_CONSTRAINTS: u32 = 1 << 9;  // Function constraint metadata present
// Constant pool features
pub const FEATURE_CONSTANT_POOL: u32 = 1 << 10; // Constant pool descriptor + pool data present
pub const FEATURE_CONSTANT_POOL_STRINGS: u32 = 1 << 11; // String blob present (fat pointers in pool)
pub const FEATURE_PUBLIC_ENTRY_TABLE: u32 = 1 << 12; // Compact public entry offset table metadata
pub const FEATURE_FAST_DISPATCH_TABLE: u32 = 1 << 13; // Dispatcher uses direct compact entry table path
pub const FEATURE_FUSED_BRANCH_OPS: u32 = 1 << 14; // Fused branch/control-flow opcodes are present
pub const FEATURE_COUNTED_LOOPS: u32 = 1 << 15; // Counted loop opcodes are present

// Address constants
pub const MAX_U16_ADDRESS: usize = u16::MAX as usize;

/// 24-bit integer for efficient space usage
#[repr(C, packed)]
#[derive(Copy, Clone, Debug, Default)]
pub struct U24([u8; 3]);

impl U24 {
    pub const fn new(val: u32) -> Self {
        Self([val as u8, (val >> 8) as u8, (val >> 16) as u8])
    }

    pub const fn get(&self) -> u32 {
        self.0[0] as u32 | ((self.0[1] as u32) << 8) | ((self.0[2] as u32) << 16)
    }

    pub const fn max_value() -> u32 {
        0xFFFFFF
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OptimizedHeader {
    pub magic: [u8; 4],
    pub features: u32,
    pub public_function_count: u8,
    pub total_function_count: u8,
}

/// Constant pool descriptor (aligned to 16 bytes).
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConstantPoolDescriptor {
    pub pool_offset: u32,        // 8-byte aligned offset to pool data
    pub string_blob_offset: u32, // Offset to string blob (0 if none)
    pub string_blob_len: u32,    // Length of string blob
    pub pool_slots: u16,         // Number of 8-byte slots in pool
    pub reserved: u16,           // Padding / future use
}

/// Function name metadata table entry.
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionNameEntry {
    pub name: String,
    pub function_index: u8,
}

/// Function name metadata section.
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionNameMetadata {
    pub section_size: u16,
    pub names: Vec<FunctionNameEntry>,
}

/// Function constraint metadata entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FunctionConstraintEntry {
    pub account_count: u8,           // How many accounts this function needs
    pub constraints: [u8; 16],       // Up to 16 accounts, each with 8-bit constraint bitmask
}

impl FunctionConstraintEntry {
    pub fn new(account_count: u8) -> Self {
        debug_assert!(account_count <= 16, "Max 16 accounts per function");
        Self {
            account_count,
            constraints: [0u8; 16],
        }
    }

    /// Set constraint bitmask for an account.
    pub fn set_constraint(&mut self, account_idx: u8, bitmask: u8) {
        if (account_idx as usize) < self.constraints.len() {
            self.constraints[account_idx as usize] = bitmask;
        }
    }

    /// Get constraint bitmask for an account.
    pub fn get_constraint(&self, account_idx: u8) -> u8 {
        if (account_idx as usize) < self.constraints.len() {
            self.constraints[account_idx as usize]
        } else {
            0
        }
    }
}

/// Function constraint metadata section.
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionConstraintMetadata {
    pub section_size: u16,
    pub constraints: Vec<FunctionConstraintEntry>,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ResourceRequirements {
    pub max_stack: u32,
    pub max_memory: u32,
    pub max_locals: u8,
    pub max_stack_depth: u16,
    pub string_pool_bytes: u16,
    pub max_call_depth: u8,
    pub temp_buffer_size: u8,
    pub heap_string_capacity: u16,
    pub heap_array_capacity: u16,
}

// Legacy type aliases for backward compatibility.
pub type LegacyResourceRequirements = ResourceRequirements;

/// Production-optimized header V2.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct FIVEScriptHeaderV2 {
    pub magic: [u8; 4],     // "5IVE"
    pub version: u8,        // Header version = 2
    pub features: u32,      // Feature bitmap for optimizations
    pub function_count: u8, // Number of functions
}

impl Default for FIVEScriptHeaderV2 {
    fn default() -> Self {
        Self {
            magic: [b'5', b'I', b'V', b'E'],
            version: 2,
            features: 0,
            function_count: 0,
        }
    }
}

/// Legacy Five bytecode header V3 for compatibility
/// 🚀 PRODUCTION: Use OptimizedHeader for new development (94% smaller)
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct FIVEScriptHeaderV3 {
    pub magic: [u8; 4],                     // "5IVE"
    pub version: u8,                        // Header version = 3
    pub features: u32,                      // Feature bitmap for BPF optimizations
    pub function_count: u8,                 // Number of functions in metadata section
    pub has_resource_hints: u8,             // 1 = has ResourceRequirements, 0 = default allocation
    pub resource_req: ResourceRequirements, // Resource requirements (10 bytes)
}

impl Default for FIVEScriptHeaderV3 {
    fn default() -> Self {
        Self {
            magic: [b'5', b'I', b'V', b'E'],
            version: 3,
            features: 0,
            function_count: 0,
            has_resource_hints: 0, // No hints by default
            resource_req: ResourceRequirements::default(),
        }
    }
}

impl FIVEScriptHeaderV3 {
    /// Create V3 header with resource hints
    pub const fn with_resource_hints(
        function_count: u8,
        features: u32,
        resource_req: ResourceRequirements,
    ) -> Self {
        Self {
            magic: [b'5', b'I', b'V', b'E'],
            version: 3,
            features,
            function_count,
            has_resource_hints: 1,
            resource_req,
        }
    }

    /// Check if header has valid resource hints
    pub const fn has_valid_resource_hints(&self) -> bool {
        self.has_resource_hints == 1
    }
}

// Production feature constants
pub const FEATURE_FUNCTION_METADATA: u32 = 1 << 6; // Function metadata in production
pub const FEATURE_RESOURCE_HINTS: u32 = 1 << 7; // Resource hints in production
pub const FEATURE_FUNCTION_NAMES: u32 = 1 << 8; // Function name metadata
pub const FIVE_HEADER_V3_SIZE: usize = 23; // V3 header size for compatibility

/// Debug logging macro (only active with debug-logs feature)
#[cfg(feature = "debug-logs")]
#[macro_export]
macro_rules! transport_log {
    ($($arg:tt)*) => {
        // In no_std, we can't use println!, so this would need platform-specific implementation
        // For now, this is a placeholder for future debug integration
    };
}

#[cfg(not(feature = "debug-logs"))]
#[macro_export]
macro_rules! transport_log {
    ($($arg:tt)*) => {};
}
