//! Unified Protocol Specification for Five
//!
//! This crate defines the authoritative protocol specification for all Five VMs
//! and compilers. It serves as the shared interface layer and follows MitoVM coding patterns for maximum performance:
//! - Zero allocations during execution
//! - Stack-allocated data structures
//! - Cold start optimized
//! - Inline optimization focus
//! - PRODUCTION ONLY: All debug overhead removed for maximum speed

#![no_std]

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

pub mod bytecode_builder;
pub mod call_convention;
pub mod encoding;
pub mod opcodes;
pub mod parser;
pub mod transport;
pub mod types;
pub mod value;

#[cfg(feature = "test-fixtures")]
pub mod test_fixtures;

pub use bytecode_builder::*;
pub use call_convention::*;
pub use encoding::*;
pub use opcodes::*;
pub use parser::*;
pub use transport::*;
pub use types::*;
pub use value::*;
// =============================================================================
// FIVE Script Header V3 - THE format for FIVE VM bytecode
// =============================================================================

/// Magic: "5IVE" = 0x45564935
pub const SCRIPT_MAGIC: u32 = 0x45564935;

/// Version 3
pub const SCRIPT_VERSION: u8 = 3;

pub const MAX_CALL_DEPTH: usize = 32;
pub const MAX_FUNCTION_PARAMS: usize = 32;
pub const MAX_LOCALS: usize = 256;
pub const MAX_FUNCTIONS: usize = 255;

// Optimized header constants
pub const FIVE_HEADER_OPTIMIZED_SIZE: usize = 10;
pub const FIVE_MAGIC: [u8; 4] = *b"5IVE";
pub const FIVE_DEPLOY_MAGIC: [u8; 4] = *b"5DEP";
pub const TEMP_BUFFER_SIZE: usize = 64;
pub const MAX_SCRIPT_SIZE: usize = 64 * 1024;

// Feature flags for header
pub const FEATURE_FUSED_BRANCH: u32 = 1 << 0;
pub const FEATURE_NO_VALIDATION: u32 = 1 << 1;
pub const FEATURE_MINIMAL_ERRORS: u32 = 1 << 2;
pub const FEATURE_COLD_START_OPT: u32 = 1 << 3;
pub const FEATURE_IMPORT_VERIFICATION: u32 = 1 << 4;  // Import verification metadata present
pub const FEATURE_FUNCTION_CONSTRAINTS: u32 = 1 << 9;  // Function constraint metadata present

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

/// Function name metadata table entry
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionNameEntry {
    pub name: String,
    pub function_index: u8,
}

/// Function name metadata section
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionNameMetadata {
    pub section_size: u16,
    pub names: Vec<FunctionNameEntry>,
}

/// Function constraint metadata entry - one per function
/// Constraint bitmask format:
///   bit 0: @signer constraint
///   bit 1: @mut constraint
///   bit 2: owner check required
///   bit 3: @init constraint
///   bit 4: @pda constraint
/// Each bit indicates if the corresponding account has that constraint
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

    /// Set constraint bitmask for an account
    pub fn set_constraint(&mut self, account_idx: u8, bitmask: u8) {
        if (account_idx as usize) < self.constraints.len() {
            self.constraints[account_idx as usize] = bitmask;
        }
    }

    /// Get constraint bitmask for an account
    pub fn get_constraint(&self, account_idx: u8) -> u8 {
        if (account_idx as usize) < self.constraints.len() {
            self.constraints[account_idx as usize]
        } else {
            0
        }
    }
}

/// Function constraint metadata section
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

// Legacy type aliases for backward compatibility
pub type LegacyResourceRequirements = ResourceRequirements;

/// Production-optimized header V2 - minimal overhead
/// 🚀 PRODUCTION: Lightweight header with essential data only
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opcode_definitions() {
        // Test that all opcodes are unique
        let mut opcodes = [false; 256];
        for info in OPCODE_TABLE {
            assert!(
                !opcodes[info.opcode as usize],
                "Duplicate opcode: 0x{:02X}",
                info.opcode
            );
            opcodes[info.opcode as usize] = true;
        }
    }

    #[test]
    fn test_opcode_lookup() {
        // Test opcode information lookup
        assert_eq!(opcode_name(HALT), "HALT");
        assert_eq!(opcode_name(PUSH_U64), "PUSH_U64");
        assert_eq!(opcode_name(CALL), "CALL");
        assert_eq!(opcode_name(RESULT_IS_ERR), "RESULT_IS_ERR");

        assert!(is_valid_opcode(HALT));
        assert!(is_valid_opcode(CALL));
        assert!(is_valid_opcode(RESULT_IS_ERR));
    }

    #[test]
    fn test_value_conversions() {
        let val_u64 = Value::U64(42);
        assert_eq!(val_u64.as_u64(), Some(42));
        assert_eq!(val_u64.as_bool(), Some(true));
        assert_eq!(val_u64.type_id(), crate::types::U64); // 4 (FIXED: was 2)

        let val_bool = Value::Bool(false);
        assert_eq!(val_bool.as_bool(), Some(false));
        assert_eq!(val_bool.type_id(), crate::types::BOOL); // 9 (FIXED: was 5)

        let val_empty = Value::Empty;
        assert_eq!(val_empty.as_u64(), None);
        assert_eq!(val_empty.type_id(), crate::types::EMPTY); // 0 (correct)
    }

    #[test]
    fn test_valueref_u128_serialization() {
        // Test U128 serialization/deserialization roundtrip
        let original = ValueRef::U128(0x123456789ABCDEF0123456789ABCDEF0);

        // Test type_id is correct (now matches types::U128 constant)
        assert_eq!(original.type_id(), crate::types::U128); // 14 (FIXED: was 4)

        // Test is_immediate includes U128
        assert!(original.is_immediate());

        // Test serialization size
        let expected_size = 17; // 1 byte type_id + 16 bytes u128
        assert_eq!(original.serialized_size(), expected_size);

        // Test roundtrip serialization
        let mut buffer = [0u8; 32]; // Oversized buffer
        let serialized_size = original.serialize_into(&mut buffer).unwrap();
        assert_eq!(serialized_size, expected_size);

        // Test deserialization
        let deserialized = ValueRef::deserialize_from(&buffer[..serialized_size]).unwrap();
        assert_eq!(deserialized, original);

        // Test edge cases
        let zero = ValueRef::U128(0);
        let mut buffer = [0u8; 32];
        let size = zero.serialize_into(&mut buffer).unwrap();
        let recovered = ValueRef::deserialize_from(&buffer[..size]).unwrap();
        assert_eq!(recovered, zero);

        let max = ValueRef::U128(u128::MAX);
        let mut buffer = [0u8; 32];
        let size = max.serialize_into(&mut buffer).unwrap();
        let recovered = ValueRef::deserialize_from(&buffer[..size]).unwrap();
        assert_eq!(recovered, max);
    }

    #[test]
    fn test_call_stack() {
        let mut stack = CallStack::new();
        assert!(stack.is_empty());
        assert_eq!(stack.depth(), 0);

        let frame = CallFrame::new(100, 1, 2, 3, 0, 10);
        assert!(stack.push(frame).is_ok());
        assert!(!stack.is_empty());
        assert_eq!(stack.depth(), 1);

        let popped = stack.pop().unwrap();
        assert_eq!(popped.return_address, frame.return_address);
        assert!(stack.is_empty());
    }

    #[test]
    fn test_function_table() {
        let mut table = FunctionTable::new();
        assert_eq!(table.count(), 0);

        let sig = FunctionSignature::new(0x12345678, 2, Some(2), 4);
        let index = table.add_function(sig, 1000).unwrap();
        assert_eq!(index, 0);
        assert_eq!(table.count(), 1);

        assert_eq!(table.get_offset(0), Some(1000));
        assert_eq!(table.get_offset(1), None);

        let retrieved_sig = table.get_signature(0).unwrap();
        assert_eq!(retrieved_sig.name_hash, 0x12345678);
        assert_eq!(retrieved_sig.parameter_count, 2);
    }

    #[test]
    fn test_instruction_encoding() {
        let inst = Instruction::new(CALL, 42, 1337);
        let encoded = inst.encode();
        let decoded = Instruction::decode(&encoded).unwrap();

        assert_eq!(decoded.opcode, CALL);
        assert_eq!(decoded.arg1, 42);
        assert_eq!(decoded.arg2, 1337);
    }

    #[test]
    fn test_jump_table() {
        let mut table = JumpTable::new();

        let index1 = table.add_entry(1000).unwrap();
        let index2 = table.add_entry(2000).unwrap();
        assert_eq!(index1, 0);
        assert_eq!(index2, 1);

        assert_eq!(table.get_offset(0), Some(1000));
        assert_eq!(table.get_offset(1), Some(2000));
        assert_eq!(table.get_offset(2), None);

        // Test encoding/decoding
        let encoded = table.encode().unwrap();
        let decoded = JumpTable::decode(&encoded).unwrap();

        assert_eq!(decoded.get_offset(0), Some(1000));
        assert_eq!(decoded.get_offset(1), Some(2000));
    }

    #[test]
    fn test_call_protocol() {
        let mut protocol = CallProtocol::new();

        // Add function to table
        let mut table = FunctionTable::new();
        let mut sig = FunctionSignature::new(0x12345678, 1, Some(crate::types::U64), 2);
        sig.parameters[0] = Parameter::new(0x11111111, crate::types::U64); // U64 parameter (type_id=4)
        table.add_function(sig, 1000).unwrap();
        protocol.initialize(table);

        // Test function call preparation
        let params = [Value::U64(42)];
        let offset = protocol.prepare_call(0, &params).unwrap();
        assert_eq!(offset, 1000);
        assert!(protocol.in_function());
        assert_eq!(protocol.call_depth(), 1);

        // Test local variable access
        protocol.set_local(0, ValueRef::Bool(true)).unwrap();
        let val = protocol.get_local(0).unwrap();
        assert_eq!(*val, ValueRef::Bool(true));

        // Test function return
        let return_addr = protocol.finish_call().unwrap();
        assert_eq!(return_addr, 9); // Size of call instruction
        assert!(!protocol.in_function());
        assert_eq!(protocol.call_depth(), 0);
    }
}
