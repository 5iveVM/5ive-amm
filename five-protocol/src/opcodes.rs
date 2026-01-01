//! Unified Opcode Definitions for Five VM Protocol
//!
//! This module provides the authoritative opcode definitions for all Five VMs.
//! Opcodes are organized by functional categories with no conflicts.
//!
//! # 🚨 MIGRATION NOTICE - Protocol Cleanup (August 2024)
//!
//! **10 redundant opcodes have been REMOVED** to minimize on-chain binary size and prepare for V3 optimizations:
//!
//! ## Removed Constraint Immediates (use VLE variants):
//! - `CHECK_SIGNER_IMM` (0x65) → use `CHECK_SIGNER` + VLE account_index
//! - `CHECK_WRITABLE_IMM` (0x66) → use `CHECK_WRITABLE` + VLE account_index
//! - `CHECK_OWNER_IMM` (0x67) → use `CHECK_OWNER` + VLE account_index + pubkey
//! - `CHECK_INITIALIZED_IMM` (0x68) → use `CHECK_INITIALIZED` + VLE account_index
//! - `CHECK_PDA_IMM` (0x69) → use `CHECK_PDA` + VLE account_index
//! - `CHECK_BATCH_IMM` (0x6A) → use multiple individual constraint checks
//! - `CHECK_UNINITIALIZED_IMM` (0x6B) → use `CHECK_UNINITIALIZED` + VLE account_index
//!
//! ## Removed Redundant Array Operations:
//! - `READ_DATA` (0xA2) → use `GET_DATA` (0x56) for account data access
//! - `ARRAY_LEN` (0xA7) → use `ARRAY_LENGTH` (0xAE) for all length operations
//! - `STRING_LENGTH` (0xB0) → use `ARRAY_LENGTH` (0xAE) - strings are arrays
//!
//! **See `OPCODE-MIGRATION-GUIDE.md` for detailed migration instructions.**
//!
//! ## Benefits:
//! - 🚀 **10 opcode slots freed** for V3 pattern fusion optimizations
//! - 📦 **Smaller on-chain binary** (~300 bytes handler code removed)
//! - 🎯 **Consistent API** - single way to do each operation
//! - ⚡ **Better flexibility** - VLE encoding supports dynamic parameters

/// Opcode allocation table - single source of truth
/// Each range is 16 opcodes (0x0-0xF) for future expansion
/// 🎯 LOGICAL ORGANIZATION: Similar operations grouped together
pub mod ranges {
    /// Control flow operations: 0x00-0x0F
    pub const CONTROL_BASE: u8 = 0x00;

    /// ALL Stack operations (consolidated): 0x10-0x1F
    /// Includes: core stack ops + ALL PUSH operations + tuple ops
    pub const STACK_BASE: u8 = 0x10;

    /// Arithmetic operations: 0x20-0x2F
    pub const ARITHMETIC_BASE: u8 = 0x20;

    /// Logical operations: 0x30-0x3F
    pub const LOGICAL_BASE: u8 = 0x30;

    /// Memory operations: 0x40-0x4F
    pub const MEMORY_BASE: u8 = 0x40;

    /// Account operations: 0x50-0x5F
    pub const ACCOUNT_BASE: u8 = 0x50;

    /// ALL Array & String operations: 0x60-0x6F (NEW LOGICAL GROUPING)
    /// Includes: array creation, indexing, strings, literals
    pub const ARRAY_BASE: u8 = 0x60;

    /// ALL Constraint operations: 0x70-0x7F (MOVED FROM 0x60)
    /// Includes: all validation and constraint checking
    pub const CONSTRAINT_BASE: u8 = 0x70;

    /// System operations: 0x80-0x8F
    /// Includes: init, invoke, PDA, sysvars operations
    pub const SYSTEM_BASE: u8 = 0x80;

    /// Function transport operations: 0x90-0x9F
    /// Includes: function calls, returns, and function management
    pub const FUNCTION_BASE: u8 = 0x90;

    /// Local variable operations: 0xA0-0xAF
    pub const LOCAL_BASE: u8 = 0xA0;

    // GENERAL_OPS_BASE removed - merged with LOCAL_BASE range

    /// Register operations: 0xB0-0xBF
    /// Hybrid VM register operations for performance optimization
    pub const REGISTER_BASE: u8 = 0xB0;

    /// [REMOVED] Account view operations - use zero-copy LOAD_FIELD/STORE_FIELD instead.
    /// Range 0xC0-0xCF now available for future features.
    ///
    /// Test framework operations: 0xD8-0xDF (using end of chunk range)
    pub const TEST_BASE: u8 = 0xD8;

    /// ALL Pattern Fusion operations: 0xE0-0xEF (V3 OPTIMIZATIONS)
    /// Dedicated range for all pattern fusion opcodes
    pub const PATTERN_FUSION_BASE: u8 = 0xE0;

    /// Advanced/Experimental operations: 0xF0-0xFF
    /// Optional/Result ops + future experimental features
    pub const ADVANCED_BASE: u8 = 0xF0;
}

// ===== CONTROL FLOW OPERATIONS (0x00-0x0F) =====
pub const HALT: u8 = 0x00;
pub const JUMP: u8 = 0x01; // JUMP offset_vle (VLE encoded for optimal space)
pub const JUMP_IF: u8 = 0x02; // JUMP_IF offset_vle (VLE encoded for optimal space)
pub const JUMP_IF_NOT: u8 = 0x03; // JUMP_IF_NOT offset_vle (VLE encoded for optimal space)
pub const REQUIRE: u8 = 0x04;
pub const ASSERT: u8 = 0x05;
pub const RETURN: u8 = 0x06;
pub const RETURN_VALUE: u8 = 0x07;
pub const NOP: u8 = 0x08; // No operation
pub const BR_EQ_U8: u8 = 0x09; // Fused compare-and-branch: compare with u8, jump if equal

// ===== ALL STACK OPERATIONS (0x10-0x1F) =====
// 🎯 LOGICAL GROUPING: All stack manipulation + ALL PUSH operations consolidated

// Core stack manipulation
pub const POP: u8 = 0x10;
pub const DUP: u8 = 0x11;
pub const DUP2: u8 = 0x12; // Duplicate top 2 items on stack
pub const SWAP: u8 = 0x13;
pub const PICK: u8 = 0x14; // Pick an item from the stack
pub const ROT: u8 = 0x15; // Rotate top 3 items on stack
pub const DROP: u8 = 0x16; // Drop top item from stack
pub const OVER: u8 = 0x17; // Copy second item to top of stack

// ALL PUSH operations (consolidated from 0xE0-0xEF VLE range)
pub const PUSH_U8: u8 = 0x18; // PUSH_U8 value_u8 (2 bytes total)
pub const PUSH_U16: u8 = 0x19; // PUSH_U16 value_vle (2-3 bytes, optimized for small values)
pub const PUSH_U32: u8 = 0x1A; // PUSH_U32 value_vle (2-5 bytes, optimized for small values)
pub const PUSH_U64: u8 = 0x1B; // PUSH_U64 value_vle (2-9 bytes, optimized for small values)
pub const PUSH_I64: u8 = 0x1C; // PUSH_I64 value_vle (2-9 bytes, optimized for small values)
pub const PUSH_BOOL: u8 = 0x1D; // PUSH_BOOL value_u8 (2 bytes total)
pub const PUSH_PUBKEY: u8 = 0x1E; // PUSH_PUBKEY value_32bytes (33 bytes total)
pub const PUSH_U128: u8 = 0x1F; // PUSH_U128 value_16bytes (17 bytes total) - MITO-style BPF-optimized
                                // Note: PUSH_ACCOUNT removed due to conflict with ADD (0x20) - use account references instead

// LEGACY NOTES:
// - PUSH_STRING moved to Array operations (0x60 range) - strings are byte arrays
// - CREATE_TUPLE, TUPLE_GET, UNPACK_TUPLE moved to Logical operations (0x30 range)
// - STACK_SIZE, STACK_CLEAR moved to General operations (0xA0 range)

// ===== ARITHMETIC OPERATIONS (0x20-0x2F) =====
pub const ADD: u8 = 0x20;
pub const SUB: u8 = 0x21;
pub const MUL: u8 = 0x22;
pub const DIV: u8 = 0x23;
pub const MOD: u8 = 0x24;
pub const GT: u8 = 0x25;
pub const LT: u8 = 0x26;
pub const EQ: u8 = 0x27;
pub const GTE: u8 = 0x28;
pub const LTE: u8 = 0x29;
pub const NEQ: u8 = 0x2A;
pub const NEG: u8 = 0x2B; // Unary negation (-value)

// ===== CHECKED ARITHMETIC OPERATIONS (0x2C-0x2E) =====
// Safe arithmetic that returns error on overflow/underflow instead of wrapping
// DSL syntax: +? -? *? for explicit safety in financial calculations
pub const ADD_CHECKED: u8 = 0x2C; // Checked addition (errors on overflow)
pub const SUB_CHECKED: u8 = 0x2D; // Checked subtraction (errors on underflow)
pub const MUL_CHECKED: u8 = 0x2E; // Checked multiplication (errors on overflow)

// ===== AVAILABLE SLOT (0x2F) =====
// 0x2F available for future arithmetic operations (DIV_CHECKED, etc.)

// ===== LOGICAL OPERATIONS (0x30-0x3F) =====
pub const AND: u8 = 0x30;
pub const OR: u8 = 0x31;
pub const NOT: u8 = 0x32;
pub const XOR: u8 = 0x33;
pub const BITWISE_NOT: u8 = 0x34; // Bitwise NOT operator (~)

// ===== BITWISE OPERATIONS (0x35-0x3F) =====
pub const BITWISE_AND: u8 = 0x35; // Bitwise AND (&)
pub const BITWISE_OR: u8 = 0x36; // Bitwise OR (|)
pub const BITWISE_XOR: u8 = 0x37; // Bitwise XOR (^)
pub const SHIFT_LEFT: u8 = 0x38; // Left shift (<<)
pub const SHIFT_RIGHT: u8 = 0x39; // Logical right shift (>>)
pub const SHIFT_RIGHT_ARITH: u8 = 0x3A; // Arithmetic right shift (sign-extending)
pub const ROTATE_LEFT: u8 = 0x3B; // Rotate left (circular shift)
pub const ROTATE_RIGHT: u8 = 0x3C; // Rotate right (circular shift)

// ===== BYTE MANIPULATION OPERATIONS (0x3D-0x3F) =====
pub const BYTE_SWAP_16: u8 = 0x3D; // Swap bytes in u16 (endian conversion)
pub const BYTE_SWAP_32: u8 = 0x3E; // Swap bytes in u32 (endian conversion)
pub const BYTE_SWAP_64: u8 = 0x3F; // Swap bytes in u64 (endian conversion)

// ===== MEMORY OPERATIONS (0x40-0x4F) =====
// All memory operations use zero-copy by default where possible
pub const STORE: u8 = 0x40;
pub const LOAD: u8 = 0x41;
pub const STORE_FIELD: u8 = 0x42; // STORE_FIELD account_index_u8, offset_vle (VLE + zero-copy optimized)
pub const LOAD_FIELD: u8 = 0x43; // LOAD_FIELD account_index_u8, offset_vle (VLE + zero-copy optimized)
pub const LOAD_INPUT: u8 = 0x44;
pub const STORE_GLOBAL: u8 = 0x45;
pub const LOAD_GLOBAL: u8 = 0x46;

// External field operations (MITO-style zero-copy)
pub const LOAD_EXTERNAL_FIELD: u8 = 0x47; // LOAD_EXTERNAL_FIELD (stack: account_pubkey, field_name) -> value
                                          // Note: No STORE_EXTERNAL_FIELD - external state is read-only for security

// ===== ACCOUNT OPERATIONS (0x50-0x5F) =====
pub const CREATE_ACCOUNT: u8 = 0x50;
pub const LOAD_ACCOUNT: u8 = 0x51;
pub const SAVE_ACCOUNT: u8 = 0x52;
pub const GET_ACCOUNT: u8 = 0x53;
pub const GET_LAMPORTS: u8 = 0x54;
pub const SET_LAMPORTS: u8 = 0x55;
pub const GET_DATA: u8 = 0x56;
pub const GET_KEY: u8 = 0x57;
pub const GET_OWNER: u8 = 0x58;
pub const TRANSFER: u8 = 0x59;
pub const TRANSFER_SIGNED: u8 = 0x5A;

// ===== ALL ARRAY & STRING OPERATIONS (0x60-0x6F) =====
// 🎯 LOGICAL GROUPING: All array, string, and literal operations consolidated

// Array creation and literals
pub const CREATE_ARRAY: u8 = 0x60; // CREATE_ARRAY capacity_u8
pub const PUSH_ARRAY_LITERAL: u8 = 0x61; // Push array literal to temp buffer
pub const ARRAY_INDEX: u8 = 0x62; // Array indexing operation: array[index]
pub const ARRAY_LENGTH: u8 = 0x63; // Get array length
pub const ARRAY_SET: u8 = 0x64; // Array element assignment
pub const ARRAY_GET: u8 = 0x65; // Array element access

// String operations (strings are byte arrays)
pub const PUSH_STRING_LITERAL: u8 = 0x66; // Push string literal to temp buffer
pub const PUSH_STRING: u8 = 0x67; // PUSH_STRING length_vle + string_data (VLE encoded)

// Array utility operations
// DUP_ADD moved to 0xE2 - slot 0x68 available

// 0x69-0x6F available for additional array/string operations

// ===== ALL CONSTRAINT OPERATIONS (0x70-0x7F) =====
// 🎯 LOGICAL GROUPING: All validation and constraint checking consolidated (MOVED FROM 0x60)

// Basic constraint operations
pub const CHECK_SIGNER: u8 = 0x70; // MOVED FROM 0x60
pub const CHECK_WRITABLE: u8 = 0x71; // MOVED FROM 0x61
pub const CHECK_OWNER: u8 = 0x72; // MOVED FROM 0x62
pub const CHECK_INITIALIZED: u8 = 0x73; // MOVED FROM 0x63
pub const CHECK_PDA: u8 = 0x74; // MOVED FROM 0x64
pub const CHECK_UNINITIALIZED: u8 = 0x75; // Check account is uninitialized before creation

// Advanced constraint checking operations
pub const CHECK_DEDUPE_TABLE: u8 = 0x76;
pub const CHECK_CACHED: u8 = 0x77;
pub const CHECK_COMPLEXITY_GROUP: u8 = 0x78;
pub const CHECK_DEDUPE_MASK: u8 = 0x79;

// 0x7A-0x7F available for additional constraint operations

// ===== SYSTEM OPERATIONS (0x80-0x8F) =====
// 🎯 MOVED FROM 0x70: System operations moved to 0x80 range
pub const INVOKE: u8 = 0x80; // MOVED FROM 0x70
pub const INVOKE_SIGNED: u8 = 0x81; // MOVED FROM 0x71
pub const GET_CLOCK: u8 = 0x82; // Get blockchain time from Clock sysvar (MOVED FROM 0x72)
pub const GET_RENT: u8 = 0x83; // Get rent information from Rent sysvar (MOVED FROM 0x73)
pub const INIT_ACCOUNT: u8 = 0x84; // Initialize regular account via System Program (MOVED FROM 0x74)
pub const INIT_PDA_ACCOUNT: u8 = 0x85; // Initialize PDA account via System Program (MOVED FROM 0x75)
pub const DERIVE_PDA: u8 = 0x86; // MOVED FROM 0x76
pub const FIND_PDA: u8 = 0x87; // MOVED FROM 0x77
pub const DERIVE_PDA_PARAMS: u8 = 0x88; // MOVED FROM 0x78
pub const FIND_PDA_PARAMS: u8 = 0x89; // MOVED FROM 0x79

// ===== FUNCTION TRANSPORT OPERATIONS (0x90-0x9F) =====
// 🎯 MOVED FROM 0x80: Function operations moved to 0x90 range
pub const CALL: u8 = 0x90; // MOVED FROM 0x80
pub const CALL_EXTERNAL: u8 = 0x91; // Call function in external account bytecode (replaces unused CALL_INDIRECT)
pub const CALL_NATIVE: u8 = 0x92; // MOVED FROM 0x82 (not implemented)
pub const PREPARE_CALL: u8 = 0x93; // MOVED FROM 0x83 (not implemented)
pub const FINISH_CALL: u8 = 0x94; // MOVED FROM 0x84 (not implemented)

// ===== LOCAL VARIABLE OPERATIONS (0xA0-0xAF) =====
// 🎯 MOVED FROM 0x90: Local variable operations moved to 0xA0 range
pub const ALLOC_LOCALS: u8 = 0xA0; // MOVED FROM 0x90
pub const DEALLOC_LOCALS: u8 = 0xA1; // MOVED FROM 0x91
pub const SET_LOCAL: u8 = 0xA2; // MOVED FROM 0x92
pub const GET_LOCAL: u8 = 0xA3; // MOVED FROM 0x93
pub const CLEAR_LOCAL: u8 = 0xA4; // MOVED FROM 0x94
pub const LOAD_PARAM: u8 = 0xA5; // MOVED FROM 0x95
pub const STORE_PARAM: u8 = 0xA6; // MOVED FROM 0x96

// General operations and utilities
pub const WRITE_DATA: u8 = 0xA7; // MOVED FROM 0xA3
pub const DATA_LEN: u8 = 0xA8; // MOVED FROM 0xA4
pub const EMIT_EVENT: u8 = 0xA9;
pub const LOG_DATA: u8 = 0xAA;
pub const GET_SIGNER_KEY: u8 = 0xAB;

// ===== AVAILABLE SLOTS (0xAC-0xAF) =====
// 0xAC-0xAF available for future operations

// NOTE: All scattered array operations have been moved to 0x60-0x6F range
// NOTE: All pattern fusion operations will be moved to 0xE0-0xEF range

// ===== REGISTER OPERATIONS (0xB0-0xBF) =====
// Hybrid VM register operations for performance optimization (moved from 0xE0-0xEF)

// Register load operations
pub const LOAD_REG_U8: u8 = 0xB0; // LOAD_REG reg, u8_value
pub const LOAD_REG_U32: u8 = 0xB1; // LOAD_REG reg, u32_value
pub const LOAD_REG_U64: u8 = 0xB2; // LOAD_REG reg, u64_value
pub const LOAD_REG_BOOL: u8 = 0xB3; // LOAD_REG reg, bool_value
pub const LOAD_REG_PUBKEY: u8 = 0xB4; // LOAD_REG reg, pubkey_value

// Register arithmetic operations
pub const ADD_REG: u8 = 0xB5; // ADD_REG dest, src1, src2
pub const SUB_REG: u8 = 0xB6; // SUB_REG dest, src1, src2
pub const MUL_REG: u8 = 0xB7; // MUL_REG dest, src1, src2
pub const DIV_REG: u8 = 0xB8; // DIV_REG dest, src1, src2

// Register comparison operations
pub const EQ_REG: u8 = 0xB9; // EQ_REG dest, src1, src2
pub const GT_REG: u8 = 0xBA; // GT_REG dest, src1, src2
pub const LT_REG: u8 = 0xBB; // LT_REG dest, src1, src2

// Register-stack bridge operations
pub const PUSH_REG: u8 = 0xBC; // PUSH_REG reg (push register to stack)
pub const POP_REG: u8 = 0xBD; // POP_REG reg (pop stack to register)
pub const COPY_REG: u8 = 0xBE; // COPY_REG dest, src
pub const CLEAR_REG: u8 = 0xBF; // CLEAR_REG reg

// ===== [REMOVED] ACCOUNT VIEW OPERATIONS (0xC0-0xCF) =====
// Account views were redundant with zero-copy LOAD_FIELD/STORE_FIELD operations
// Range 0xC0-0xCF now available for Phase 6 Dynamic Pattern Fusion
// Migration: Use LOAD_FIELD/STORE_FIELD for direct account field access

// ===== NIBBLE IMMEDIATE OPERATIONS (0xD0-0xD7) =====
// BPF optimization: single-byte encoding for common local variable operations
// GET_LOCAL and SET_LOCAL with hardcoded indices 0-3 (no VLE operand needed)

// Nibble immediate GET_LOCAL operations
pub const GET_LOCAL_0: u8 = 0xD0; // GET_LOCAL with index 0 (single byte)
pub const GET_LOCAL_1: u8 = 0xD1; // GET_LOCAL with index 1 (single byte)
pub const GET_LOCAL_2: u8 = 0xD2; // GET_LOCAL with index 2 (single byte)
pub const GET_LOCAL_3: u8 = 0xD3; // GET_LOCAL with index 3 (single byte)

// Nibble immediate SET_LOCAL operations
pub const SET_LOCAL_0: u8 = 0xD4; // SET_LOCAL with index 0 (single byte)
pub const SET_LOCAL_1: u8 = 0xD5; // SET_LOCAL with index 1 (single byte)
pub const SET_LOCAL_2: u8 = 0xD6; // SET_LOCAL with index 2 (single byte)
pub const SET_LOCAL_3: u8 = 0xD7; // SET_LOCAL with index 3 (single byte)

// ===== NIBBLE CONSTANTS & PARAMETERS (0xD8-0xDF) =====
// BPF optimization: single-byte encoding for common constants and parameters

// Nibble immediate PUSH constant operations (88% space savings: 9 bytes -> 1 byte)
pub const PUSH_0: u8 = 0xD8; // PUSH_U64(0) in single byte
pub const PUSH_1: u8 = 0xD9; // PUSH_U64(1) in single byte
pub const PUSH_2: u8 = 0xDA; // PUSH_U64(2) in single byte
pub const PUSH_3: u8 = 0xDB; // PUSH_U64(3) in single byte

// Nibble immediate LOAD_PARAM operations (50% space savings: 2 bytes -> 1 byte)
pub const LOAD_PARAM_0: u8 = 0xDC; // LOAD_PARAM 0 in single byte
pub const LOAD_PARAM_1: u8 = 0xDD; // LOAD_PARAM 1 in single byte
pub const LOAD_PARAM_2: u8 = 0xDE; // LOAD_PARAM 2 in single byte
pub const LOAD_PARAM_3: u8 = 0xDF; // LOAD_PARAM 3 in single byte

// ===== ALL PATTERN FUSION OPERATIONS (0xE0-0xEF) =====
// 🎯 LOGICAL GROUPING: All V3 pattern fusion optimizations consolidated
// 🚀 V3 PATTERN FUSION: High-impact optimizations for bytecode size reduction

// Constant optimizations (1 byte vs 2 bytes = 50% savings)
pub const PUSH_ZERO: u8 = 0xE0; // Push constant 0 (MOVED FROM 0x65)
pub const PUSH_ONE: u8 = 0xE1; // Push constant 1 (MOVED FROM 0xA2)

// Arithmetic fusion patterns (1 byte vs 2 bytes = 50% savings)
pub const DUP_ADD: u8 = 0xE2; // dup + add (MOVED FROM 0x68/0xA7)
pub const DUP_SUB: u8 = 0xE3; // dup + sub fusion
pub const DUP_MUL: u8 = 0xE4; // dup + mul fusion

// Validation fusion patterns
pub const VALIDATE_AMOUNT_NONZERO: u8 = 0xE5; // amount > 0 + require (MOVED FROM 0x66)
pub const VALIDATE_SUFFICIENT: u8 = 0xE6; // balance >= amount + require (MOVED FROM 0x67)
pub const EQ_ZERO_JUMP: u8 = 0xE7; // value == 0 ? jump : continue (MOVED FROM 0x6B)

// Transfer fusion patterns
pub const TRANSFER_DEBIT: u8 = 0xE8; // get_balance - amount -> store (MOVED FROM 0x68)
pub const TRANSFER_CREDIT: u8 = 0xE9; // get_balance + amount -> store (MOVED FROM 0x69)

// Control flow fusion patterns
pub const RETURN_SUCCESS: u8 = 0xEA; // return ok() (MOVED FROM 0x6A)
pub const RETURN_ERROR: u8 = 0xEB; // return err() fusion
pub const GT_ZERO_JUMP: u8 = 0xEC; // value > 0 ? jump : continue
pub const LT_ZERO_JUMP: u8 = 0xED; // value < 0 ? jump : continue

// LEGACY: Static precompile system (replaced by library accounts)
#[cfg(feature = "precompiles")]
pub const PRECOMPILE: u8 = 0xEE; // Execute precompiled pattern with parameters (LEGACY)

// 0xEF reserved for future pattern fusion opcodes

// ===== ADVANCED/EXPERIMENTAL OPERATIONS (0xF0-0xFF) =====
// 🎯 LOGICAL GROUPING: Optional/Result operations + experimental features

// Optional/Result type operations
pub const RESULT_OK: u8 = 0xF0; // Create Result::Ok value
pub const RESULT_ERR: u8 = 0xF1; // Create Result::Err value
pub const OPTIONAL_SOME: u8 = 0xF2; // Create Optional::Some value
pub const OPTIONAL_NONE: u8 = 0xF3; // Create Optional::None value
pub const OPTIONAL_UNWRAP: u8 = 0xF4; // Unwrap Optional value (panic if None)
pub const OPTIONAL_IS_SOME: u8 = 0xF5; // Check if Optional has Some value
pub const OPTIONAL_GET_VALUE: u8 = 0xF6; // Get value from Optional (unsafe)

// Advanced bulk operations
pub const BULK_LOAD_FIELD_N: u8 = 0xF7; // BULK load N fields from account (VLE encoding by default)

// Tuple operations (moved from stack range to make room for PUSH ops)
pub const CREATE_TUPLE: u8 = 0xF8; // MOVED FROM 0x18
pub const TUPLE_GET: u8 = 0xF9; // MOVED FROM 0x19
pub const UNPACK_TUPLE: u8 = 0xFA; // MOVED FROM 0x1A

// Stack management operations (moved from stack range to make room for PUSH ops)
pub const STACK_SIZE: u8 = 0xFB; // Get current stack size (MOVED FROM 0x1B)
pub const STACK_CLEAR: u8 = 0xFC; // Clear entire stack (MOVED FROM 0x1C)

// Additional Option/Result operations
pub const OPTIONAL_IS_NONE: u8 = 0xFD; // Check if Optional is None
pub const RESULT_IS_OK: u8 = 0xFE; // Check if Result is Ok
pub const RESULT_IS_ERR: u8 = 0xFF; // Check if Result is Err

// Additional Result operations - using available slots in lower ranges
pub const RESULT_UNWRAP: u8 = 0xAC; // Unwrap Result value (panic if Err)
pub const RESULT_GET_VALUE: u8 = 0xAD; // Get Ok value from Result (unsafe)
pub const RESULT_GET_ERROR: u8 = 0xAE; // Get error code from Result (unsafe)

// DSL-compatible aliases with OP_ prefix
// [REMOVED] Account view operation aliases - use LOAD_FIELD/STORE_FIELD instead
// DEPRECATED: Zero-copy specific aliases removed - all field operations are zero-copy by default
// pub const OP_STORE_FIELD_ZEROCOPY: u8 = STORE_FIELD_ZEROCOPY;  // RESERVED
// pub const OP_LOAD_FIELD_ZEROCOPY: u8 = LOAD_FIELD_ZEROCOPY;    // RESERVED
// Removed: chunk operation aliases (OP_LOAD_CHUNK_RANGE, etc.) - no longer supported

// Additional DSLR compatibility (placeholder values for missing opcodes)
// DUP2 moved to Stack Operations range at 0x12

pub const CAST: u8 = 0xAF; // Type cast operation CAST target_type_u8

// MitoVM compatibility aliases
pub const RET: u8 = RETURN;
pub const JZ: u8 = JUMP_IF_NOT;

// DEPRECATED OPERATIONS REMOVED:
// - RLE/Compact encoding operations (VLE-only architecture)
// - Duplicate register definitions (moved to 0xB0-0xBF range)
// - Compression markers (not used with VLE-only approach)

// ===== COMPACT FIELD IDs FOR BUILT-IN ACCOUNT PROPERTIES =====
pub const FIELD_LAMPORTS: u8 = 0; // account.lamports
pub const FIELD_OWNER: u8 = 1; // account.owner
pub const FIELD_KEY: u8 = 2; // account.key
pub const FIELD_DATA: u8 = 3; // account.data

// ===== CONSTRAINT BITMASKS (NOT OPCODES - DSL Compatibility Constants) =====
// These are bitmask values used for constraint validation, NOT opcode numbers
pub const CONSTRAINT_SIGNER: u8 = 0x01; // Bit 0: Account must be signer
pub const CONSTRAINT_WRITABLE: u8 = 0x02; // Bit 1: Account must be writable
pub const CONSTRAINT_OWNER: u8 = 0x44; // Bit 2: Account owner validation required - moved to avoid REQUIRE conflict
pub const CONSTRAINT_INITIALIZED: u8 = 0x08; // Bit 3: Account must be initialized
pub const CONSTRAINT_PDA: u8 = 0x10; // Bit 4: Account must be valid PDA

/// Opcode argument types for instruction decoding
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArgType {
    None,
    U8,
    U16,
    U32,
    U64,
    ValueType,
    FunctionIndex,
    LocalIndex,
    AccountIndex,
    RegisterIndex,  // Register index (0-15)
    TwoRegisters,   // Two register indices
    ThreeRegisters, // Three register indices (dest, src1, src2)
    CallExternal,   // account_index (u8) + function_offset (u16) + param_count (u8)
    AccountField,   // account_index (u8) + field_offset (VLE)
}

/// Opcode metadata for efficient VM implementation
#[derive(Debug, Clone, Copy)]
pub struct OpcodeInfo {
    pub opcode: u8,
    pub name: &'static str,
    pub arg_type: ArgType,
    pub stack_effect: i8, // Net stack change: positive = push, negative = pop
    // Special values: -127/127 indicate dynamic effects based on opcode argument
    pub compute_cost: u8, // Estimated compute units
}

/// Complete opcode information table (const for zero-allocation lookup)
pub const OPCODE_TABLE: &[OpcodeInfo] = &[
    // Control flow (now VLE optimized)
    OpcodeInfo {
        opcode: HALT,
        name: "HALT",
        arg_type: ArgType::None,
        stack_effect: 0,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: JUMP,
        name: "JUMP",
        arg_type: ArgType::U16,
        stack_effect: 0,
        compute_cost: 2,
    }, // VLE encoded offset
    OpcodeInfo {
        opcode: JUMP_IF,
        name: "JUMP_IF",
        arg_type: ArgType::U16,
        stack_effect: -1,
        compute_cost: 3,
    }, // VLE encoded offset
    OpcodeInfo {
        opcode: JUMP_IF_NOT,
        name: "JUMP_IF_NOT",
        arg_type: ArgType::U16,
        stack_effect: -1,
        compute_cost: 3,
    }, // VLE encoded offset
    OpcodeInfo {
        opcode: REQUIRE,
        name: "REQUIRE",
        arg_type: ArgType::None,
        stack_effect: -1,
        compute_cost: 3,
    },
    OpcodeInfo {
        opcode: RETURN,
        name: "RETURN",
        arg_type: ArgType::None,
        stack_effect: 0,
        compute_cost: 2,
    },
    OpcodeInfo {
        opcode: RETURN_VALUE,
        name: "RETURN_VALUE",
        arg_type: ArgType::None,
        stack_effect: -1,
        compute_cost: 2,
    },
    OpcodeInfo {
        opcode: BR_EQ_U8,
        name: "BR_EQ_U8",
        arg_type: ArgType::U8,
        stack_effect: -1,
        compute_cost: 3,
    },
    // Stack operations
    OpcodeInfo {
        opcode: POP,
        name: "POP",
        arg_type: ArgType::None,
        stack_effect: -1,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: DUP,
        name: "DUP",
        arg_type: ArgType::None,
        stack_effect: 1,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: SWAP,
        name: "SWAP",
        arg_type: ArgType::None,
        stack_effect: 0,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: PICK,
        name: "PICK",
        arg_type: ArgType::None,
        stack_effect: 1,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: CREATE_TUPLE,
        name: "CREATE_TUPLE",
        arg_type: ArgType::None,
        stack_effect: -127,
        compute_cost: 2,
    }, // Dynamic: -(n-1)
    OpcodeInfo {
        opcode: TUPLE_GET,
        name: "TUPLE_GET",
        arg_type: ArgType::None,
        stack_effect: 0,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: UNPACK_TUPLE,
        name: "UNPACK_TUPLE",
        arg_type: ArgType::None,
        stack_effect: 127,
        compute_cost: 2,
    }, // Dynamic: +(n-1)
    OpcodeInfo {
        opcode: STACK_SIZE,
        name: "STACK_SIZE",
        arg_type: ArgType::None,
        stack_effect: 1,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: STACK_CLEAR,
        name: "STACK_CLEAR",
        arg_type: ArgType::None,
        stack_effect: -127,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: OPTIONAL_SOME,
        name: "OPTIONAL_SOME",
        arg_type: ArgType::None,
        stack_effect: 0,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: OPTIONAL_NONE,
        name: "OPTIONAL_NONE",
        arg_type: ArgType::None,
        stack_effect: 1,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: OPTIONAL_UNWRAP,
        name: "OPTIONAL_UNWRAP",
        arg_type: ArgType::None,
        stack_effect: 0,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: RESULT_OK,
        name: "RESULT_OK",
        arg_type: ArgType::None,
        stack_effect: 0,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: RESULT_ERR,
        name: "RESULT_ERR",
        arg_type: ArgType::None,
        stack_effect: 0,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: PUSH_U8,
        name: "PUSH_U8",
        arg_type: ArgType::U8,
        stack_effect: 1,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: OPTIONAL_IS_SOME,
        name: "OPTIONAL_IS_SOME",
        arg_type: ArgType::None,
        stack_effect: 0,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: OPTIONAL_GET_VALUE,
        name: "OPTIONAL_GET_VALUE",
        arg_type: ArgType::None,
        stack_effect: 0,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: OPTIONAL_IS_NONE,
        name: "OPTIONAL_IS_NONE",
        arg_type: ArgType::None,
        stack_effect: 0,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: RESULT_IS_OK,
        name: "RESULT_IS_OK",
        arg_type: ArgType::None,
        stack_effect: 0,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: RESULT_IS_ERR,
        name: "RESULT_IS_ERR",
        arg_type: ArgType::None,
        stack_effect: 0,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: RESULT_UNWRAP,
        name: "RESULT_UNWRAP",
        arg_type: ArgType::None,
        stack_effect: 0,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: RESULT_GET_VALUE,
        name: "RESULT_GET_VALUE",
        arg_type: ArgType::None,
        stack_effect: 0,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: RESULT_GET_ERROR,
        name: "RESULT_GET_ERROR",
        arg_type: ArgType::None,
        stack_effect: 0,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: PUSH_U64,
        name: "PUSH_U64",
        arg_type: ArgType::U64,
        stack_effect: 1,
        compute_cost: 1,
    }, // VLE encoded value
    OpcodeInfo {
        opcode: PUSH_I64,
        name: "PUSH_I64",
        arg_type: ArgType::U64,
        stack_effect: 1,
        compute_cost: 1,
    }, // VLE encoded value
    OpcodeInfo {
        opcode: PUSH_BOOL,
        name: "PUSH_BOOL",
        arg_type: ArgType::None,
        stack_effect: 1,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: PUSH_PUBKEY,
        name: "PUSH_PUBKEY",
        arg_type: ArgType::None,
        stack_effect: 1,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: PUSH_STRING,
        name: "PUSH_STRING",
        arg_type: ArgType::U8,
        stack_effect: 1,
        compute_cost: 1,
    },
    // PUSH_ACCOUNT removed due to conflict with ADD (0x20)

    // Arithmetic operations
    OpcodeInfo {
        opcode: ADD,
        name: "ADD",
        arg_type: ArgType::None,
        stack_effect: -1,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: SUB,
        name: "SUB",
        arg_type: ArgType::None,
        stack_effect: -1,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: MUL,
        name: "MUL",
        arg_type: ArgType::None,
        stack_effect: -1,
        compute_cost: 2,
    },
    OpcodeInfo {
        opcode: DIV,
        name: "DIV",
        arg_type: ArgType::None,
        stack_effect: -1,
        compute_cost: 3,
    },
    OpcodeInfo {
        opcode: EQ,
        name: "EQ",
        arg_type: ArgType::None,
        stack_effect: -1,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: GT,
        name: "GT",
        arg_type: ArgType::None,
        stack_effect: -1,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: LT,
        name: "LT",
        arg_type: ArgType::None,
        stack_effect: -1,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: MOD,
        name: "MOD",
        arg_type: ArgType::None,
        stack_effect: -1,
        compute_cost: 3,
    },
    OpcodeInfo {
        opcode: GTE,
        name: "GTE",
        arg_type: ArgType::None,
        stack_effect: -1,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: LTE,
        name: "LTE",
        arg_type: ArgType::None,
        stack_effect: -1,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: NEQ,
        name: "NEQ",
        arg_type: ArgType::None,
        stack_effect: -1,
        compute_cost: 1,
    },
    // 128-bit literal support (polymorphic arithmetic uses generic opcodes)
    OpcodeInfo {
        opcode: PUSH_U128,
        name: "PUSH_U128",
        arg_type: ArgType::None,
        stack_effect: 1,
        compute_cost: 1,
    },
    // Logical operations
    OpcodeInfo {
        opcode: AND,
        name: "AND",
        arg_type: ArgType::None,
        stack_effect: -1,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: OR,
        name: "OR",
        arg_type: ArgType::None,
        stack_effect: -1,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: NOT,
        name: "NOT",
        arg_type: ArgType::None,
        stack_effect: 0,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: XOR,
        name: "XOR",
        arg_type: ArgType::None,
        stack_effect: -1,
        compute_cost: 1,
    },
    // Bitwise operations
    OpcodeInfo {
        opcode: BITWISE_AND,
        name: "BITWISE_AND",
        arg_type: ArgType::None,
        stack_effect: -1,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: BITWISE_OR,
        name: "BITWISE_OR",
        arg_type: ArgType::None,
        stack_effect: -1,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: BITWISE_XOR,
        name: "BITWISE_XOR",
        arg_type: ArgType::None,
        stack_effect: -1,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: SHIFT_LEFT,
        name: "SHIFT_LEFT",
        arg_type: ArgType::None,
        stack_effect: -1,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: SHIFT_RIGHT,
        name: "SHIFT_RIGHT",
        arg_type: ArgType::None,
        stack_effect: -1,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: SHIFT_RIGHT_ARITH,
        name: "SHIFT_RIGHT_ARITH",
        arg_type: ArgType::None,
        stack_effect: -1,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: ROTATE_RIGHT,
        name: "ROTATE_RIGHT",
        arg_type: ArgType::None,
        stack_effect: -1,
        compute_cost: 1,
    },
    // Memory operations
    OpcodeInfo {
        opcode: STORE,
        name: "STORE",
        arg_type: ArgType::U32,
        stack_effect: -1,
        compute_cost: 2,
    },
    OpcodeInfo {
        opcode: LOAD,
        name: "LOAD",
        arg_type: ArgType::U32,
        stack_effect: 1,
        compute_cost: 2,
    },
    OpcodeInfo {
        opcode: STORE_FIELD,
        name: "STORE_FIELD",
        arg_type: ArgType::AccountField,
        stack_effect: -1,
        compute_cost: 3,
    },
    OpcodeInfo {
        opcode: LOAD_FIELD,
        name: "LOAD_FIELD",
        arg_type: ArgType::AccountField,
        stack_effect: 1,
        compute_cost: 3,
    },
    OpcodeInfo {
        opcode: LOAD_INPUT,
        name: "LOAD_INPUT",
        arg_type: ArgType::U8,
        stack_effect: 1,
        compute_cost: 2,
    },
    OpcodeInfo {
        opcode: STORE_GLOBAL,
        name: "STORE_GLOBAL",
        arg_type: ArgType::U16,
        stack_effect: -1,
        compute_cost: 2,
    },
    OpcodeInfo {
        opcode: LOAD_GLOBAL,
        name: "LOAD_GLOBAL",
        arg_type: ArgType::U16,
        stack_effect: 1,
        compute_cost: 2,
    },
    OpcodeInfo {
        opcode: LOAD_EXTERNAL_FIELD,
        name: "LOAD_EXTERNAL_FIELD",
        arg_type: ArgType::None,
        stack_effect: -1,
        compute_cost: 5,
    }, // stack: [pubkey, field_name_key]
    // Byte manipulation operations
    OpcodeInfo {
        opcode: BYTE_SWAP_16,
        name: "BYTE_SWAP_16",
        arg_type: ArgType::None,
        stack_effect: 0,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: BYTE_SWAP_32,
        name: "BYTE_SWAP_32",
        arg_type: ArgType::None,
        stack_effect: 0,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: BYTE_SWAP_64,
        name: "BYTE_SWAP_64",
        arg_type: ArgType::None,
        stack_effect: 0,
        compute_cost: 1,
    },
    // Function transport
    OpcodeInfo {
        opcode: CALL,
        name: "CALL",
        arg_type: ArgType::FunctionIndex,
        stack_effect: 0,
        compute_cost: 5,
    }, // VLE-encoded function index
    OpcodeInfo {
        opcode: CALL_EXTERNAL,
        name: "CALL_EXTERNAL",
        arg_type: ArgType::CallExternal,
        stack_effect: 0,
        compute_cost: 8,
    }, // account_index_u8 + offset_u16 + param_count_u8
    OpcodeInfo {
        opcode: ALLOC_LOCALS,
        name: "ALLOC_LOCALS",
        arg_type: ArgType::None,
        stack_effect: 0,
        compute_cost: 2,
    },
    OpcodeInfo {
        opcode: DEALLOC_LOCALS,
        name: "DEALLOC_LOCALS",
        arg_type: ArgType::None,
        stack_effect: 0,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: SET_LOCAL,
        name: "SET_LOCAL",
        arg_type: ArgType::U8,
        stack_effect: -1,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: GET_LOCAL,
        name: "GET_LOCAL",
        arg_type: ArgType::U8,
        stack_effect: 1,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: LOAD_PARAM,
        name: "LOAD_PARAM",
        arg_type: ArgType::U8,
        stack_effect: 1,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: STORE_PARAM,
        name: "STORE_PARAM",
        arg_type: ArgType::U8,
        stack_effect: -1,
        compute_cost: 1,
    },
    // System operations
    OpcodeInfo {
        opcode: GET_CLOCK,
        name: "GET_CLOCK",
        arg_type: ArgType::None,
        stack_effect: 1,
        compute_cost: 5,
    },
    OpcodeInfo {
        opcode: DERIVE_PDA,
        name: "DERIVE_PDA",
        arg_type: ArgType::None,
        stack_effect: 1,
        compute_cost: 8,
    },
    // Account operations
    OpcodeInfo {
        opcode: CREATE_ACCOUNT,
        name: "CREATE_ACCOUNT",
        arg_type: ArgType::None,
        stack_effect: -127,
        compute_cost: 10,
    },
    OpcodeInfo {
        opcode: LOAD_ACCOUNT,
        name: "LOAD_ACCOUNT",
        arg_type: ArgType::AccountIndex,
        stack_effect: 1,
        compute_cost: 5,
    },
    OpcodeInfo {
        opcode: SAVE_ACCOUNT,
        name: "SAVE_ACCOUNT",
        arg_type: ArgType::AccountIndex,
        stack_effect: -1,
        compute_cost: 5,
    },
    OpcodeInfo {
        opcode: GET_ACCOUNT,
        name: "GET_ACCOUNT",
        arg_type: ArgType::AccountIndex,
        stack_effect: 1,
        compute_cost: 2,
    },
    OpcodeInfo {
        opcode: GET_LAMPORTS,
        name: "GET_LAMPORTS",
        arg_type: ArgType::AccountIndex,
        stack_effect: 1,
        compute_cost: 2,
    },
    OpcodeInfo {
        opcode: SET_LAMPORTS,
        name: "SET_LAMPORTS",
        arg_type: ArgType::AccountIndex,
        stack_effect: -1,
        compute_cost: 2,
    },
    OpcodeInfo {
        opcode: GET_DATA,
        name: "GET_DATA",
        arg_type: ArgType::AccountIndex,
        stack_effect: 1,
        compute_cost: 2,
    },
    OpcodeInfo {
        opcode: GET_KEY,
        name: "GET_KEY",
        arg_type: ArgType::AccountIndex,
        stack_effect: 1,
        compute_cost: 2,
    },
    OpcodeInfo {
        opcode: GET_OWNER,
        name: "GET_OWNER",
        arg_type: ArgType::AccountIndex,
        stack_effect: 1,
        compute_cost: 2,
    },
    OpcodeInfo {
        opcode: TRANSFER,
        name: "TRANSFER",
        arg_type: ArgType::None,
        stack_effect: -3,
        compute_cost: 5,
    },
    OpcodeInfo {
        opcode: TRANSFER_SIGNED,
        name: "TRANSFER_SIGNED",
        arg_type: ArgType::None,
        stack_effect: -3,
        compute_cost: 8,
    },
    // Constraint operations
    OpcodeInfo {
        opcode: CHECK_SIGNER,
        name: "CHECK_SIGNER",
        arg_type: ArgType::AccountIndex,
        stack_effect: 0,
        compute_cost: 2,
    },
    OpcodeInfo {
        opcode: CHECK_WRITABLE,
        name: "CHECK_WRITABLE",
        arg_type: ArgType::AccountIndex,
        stack_effect: 0,
        compute_cost: 2,
    },
    OpcodeInfo {
        opcode: CHECK_OWNER,
        name: "CHECK_OWNER",
        arg_type: ArgType::AccountIndex,
        stack_effect: -1,
        compute_cost: 3,
    },
    OpcodeInfo {
        opcode: CHECK_INITIALIZED,
        name: "CHECK_INITIALIZED",
        arg_type: ArgType::AccountIndex,
        stack_effect: 0,
        compute_cost: 2,
    },
    OpcodeInfo {
        opcode: CHECK_PDA,
        name: "CHECK_PDA",
        arg_type: ArgType::AccountIndex,
        stack_effect: -1,
        compute_cost: 5,
    },
    OpcodeInfo {
        opcode: CHECK_UNINITIALIZED,
        name: "CHECK_UNINITIALIZED",
        arg_type: ArgType::AccountIndex,
        stack_effect: 0,
        compute_cost: 2,
    },
    // System operations
    OpcodeInfo {
        opcode: INVOKE,
        name: "INVOKE",
        arg_type: ArgType::None,
        stack_effect: -127,
        compute_cost: 20,
    },
    OpcodeInfo {
        opcode: INVOKE_SIGNED,
        name: "INVOKE_SIGNED",
        arg_type: ArgType::None,
        stack_effect: -127,
        compute_cost: 25,
    },
    OpcodeInfo {
        opcode: INIT_ACCOUNT,
        name: "INIT_ACCOUNT",
        arg_type: ArgType::AccountIndex,
        stack_effect: 0,
        compute_cost: 15,
    },
    OpcodeInfo {
        opcode: INIT_PDA_ACCOUNT,
        name: "INIT_PDA_ACCOUNT",
        arg_type: ArgType::AccountIndex,
        stack_effect: -1,
        compute_cost: 20,
    },
    OpcodeInfo {
        opcode: DERIVE_PDA_PARAMS,
        name: "DERIVE_PDA_PARAMS",
        arg_type: ArgType::None,
        stack_effect: -127,
        compute_cost: 10,
    },
    OpcodeInfo {
        opcode: FIND_PDA_PARAMS,
        name: "FIND_PDA_PARAMS",
        arg_type: ArgType::None,
        stack_effect: -127,
        compute_cost: 12,
    },
    // Additional VLE PUSH operations
    OpcodeInfo {
        opcode: PUSH_U32,
        name: "PUSH_U32",
        arg_type: ArgType::U32,
        stack_effect: 1,
        compute_cost: 1,
    }, // VLE encoded value
    OpcodeInfo {
        opcode: PUSH_U16,
        name: "PUSH_U16",
        arg_type: ArgType::U16,
        stack_effect: 1,
        compute_cost: 1,
    }, // VLE encoded value
    // Note: JUMP_TABLE (0xB0) opcode removed from protocol

    // Register operations (hybrid VM performance optimization)
    OpcodeInfo {
        opcode: LOAD_REG_U8,
        name: "LOAD_REG_U8",
        arg_type: ArgType::RegisterIndex,
        stack_effect: 0,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: LOAD_REG_U32,
        name: "LOAD_REG_U32",
        arg_type: ArgType::RegisterIndex,
        stack_effect: 0,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: LOAD_REG_U64,
        name: "LOAD_REG_U64",
        arg_type: ArgType::RegisterIndex,
        stack_effect: 0,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: LOAD_REG_BOOL,
        name: "LOAD_REG_BOOL",
        arg_type: ArgType::RegisterIndex,
        stack_effect: 0,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: LOAD_REG_PUBKEY,
        name: "LOAD_REG_PUBKEY",
        arg_type: ArgType::RegisterIndex,
        stack_effect: 0,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: ADD_REG,
        name: "ADD_REG",
        arg_type: ArgType::ThreeRegisters,
        stack_effect: 0,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: SUB_REG,
        name: "SUB_REG",
        arg_type: ArgType::ThreeRegisters,
        stack_effect: 0,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: MUL_REG,
        name: "MUL_REG",
        arg_type: ArgType::ThreeRegisters,
        stack_effect: 0,
        compute_cost: 2,
    },
    OpcodeInfo {
        opcode: DIV_REG,
        name: "DIV_REG",
        arg_type: ArgType::ThreeRegisters,
        stack_effect: 0,
        compute_cost: 3,
    },
    OpcodeInfo {
        opcode: EQ_REG,
        name: "EQ_REG",
        arg_type: ArgType::ThreeRegisters,
        stack_effect: 0,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: GT_REG,
        name: "GT_REG",
        arg_type: ArgType::ThreeRegisters,
        stack_effect: 0,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: LT_REG,
        name: "LT_REG",
        arg_type: ArgType::ThreeRegisters,
        stack_effect: 0,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: PUSH_REG,
        name: "PUSH_REG",
        arg_type: ArgType::RegisterIndex,
        stack_effect: 1,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: POP_REG,
        name: "POP_REG",
        arg_type: ArgType::RegisterIndex,
        stack_effect: -1,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: COPY_REG,
        name: "COPY_REG",
        arg_type: ArgType::TwoRegisters,
        stack_effect: 0,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: CLEAR_REG,
        name: "CLEAR_REG",
        arg_type: ArgType::RegisterIndex,
        stack_effect: 0,
        compute_cost: 1,
    },
    // Nibble immediate operations
    OpcodeInfo {
        opcode: PUSH_0,
        name: "PUSH_0",
        arg_type: ArgType::None,
        stack_effect: 1,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: PUSH_1,
        name: "PUSH_1",
        arg_type: ArgType::None,
        stack_effect: 1,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: PUSH_2,
        name: "PUSH_2",
        arg_type: ArgType::None,
        stack_effect: 1,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: PUSH_3,
        name: "PUSH_3",
        arg_type: ArgType::None,
        stack_effect: 1,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: LOAD_PARAM_0,
        name: "LOAD_PARAM_0",
        arg_type: ArgType::None,
        stack_effect: 1,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: LOAD_PARAM_1,
        name: "LOAD_PARAM_1",
        arg_type: ArgType::None,
        stack_effect: 1,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: LOAD_PARAM_2,
        name: "LOAD_PARAM_2",
        arg_type: ArgType::None,
        stack_effect: 1,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: LOAD_PARAM_3,
        name: "LOAD_PARAM_3",
        arg_type: ArgType::None,
        stack_effect: 1,
        compute_cost: 1,
    },
    // Array and string operations
    OpcodeInfo {
        opcode: PUSH_ARRAY_LITERAL,
        name: "PUSH_ARRAY_LITERAL",
        arg_type: ArgType::U8,
        stack_effect: 1,
        compute_cost: 3,
    },
    OpcodeInfo {
        opcode: ARRAY_INDEX,
        name: "ARRAY_INDEX",
        arg_type: ArgType::None,
        stack_effect: -1,
        compute_cost: 2,
    },
    OpcodeInfo {
        opcode: ARRAY_LENGTH,
        name: "ARRAY_LENGTH",
        arg_type: ArgType::None,
        stack_effect: 0,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: PUSH_STRING_LITERAL,
        name: "PUSH_STRING_LITERAL",
        arg_type: ArgType::U8,
        stack_effect: 1,
        compute_cost: 2,
    },
    // STRING_LENGTH entry removed - opcode was deleted (use ARRAY_LENGTH instead)
    // ARRAY_CONCAT removed - use array operations in their dedicated range

    // V3 Pattern Fusion Opcodes (using freed slots)
    OpcodeInfo {
        opcode: PUSH_ZERO,
        name: "PUSH_ZERO",
        arg_type: ArgType::None,
        stack_effect: 1,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: VALIDATE_AMOUNT_NONZERO,
        name: "VALIDATE_AMOUNT_NONZERO",
        arg_type: ArgType::None,
        stack_effect: -1,
        compute_cost: 3,
    },
    OpcodeInfo {
        opcode: VALIDATE_SUFFICIENT,
        name: "VALIDATE_SUFFICIENT",
        arg_type: ArgType::None,
        stack_effect: -2,
        compute_cost: 4,
    },
    OpcodeInfo {
        opcode: TRANSFER_DEBIT,
        name: "TRANSFER_DEBIT",
        arg_type: ArgType::U8,
        stack_effect: -1,
        compute_cost: 5,
    },
    OpcodeInfo {
        opcode: TRANSFER_CREDIT,
        name: "TRANSFER_CREDIT",
        arg_type: ArgType::U8,
        stack_effect: -1,
        compute_cost: 5,
    },
    OpcodeInfo {
        opcode: RETURN_SUCCESS,
        name: "RETURN_SUCCESS",
        arg_type: ArgType::None,
        stack_effect: 0,
        compute_cost: 2,
    },
    OpcodeInfo {
        opcode: EQ_ZERO_JUMP,
        name: "EQ_ZERO_JUMP",
        arg_type: ArgType::U16,
        stack_effect: -1,
        compute_cost: 3,
    },
    OpcodeInfo {
        opcode: PUSH_ONE,
        name: "PUSH_ONE",
        arg_type: ArgType::None,
        stack_effect: 1,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: DUP_ADD,
        name: "DUP_ADD",
        arg_type: ArgType::None,
        stack_effect: 0,
        compute_cost: 2,
    },
    // SWAP_SUB removed - use individual SWAP + SUB operations

    // V3 Bulk Operations
    OpcodeInfo {
        opcode: BULK_LOAD_FIELD_N,
        name: "BULK_LOAD_FIELD_N",
        arg_type: ArgType::U8,
        stack_effect: 127,
        compute_cost: 5,
    },

    // Nibble immediate GET_LOCAL operations (0xD0-0xD3)
    OpcodeInfo {
        opcode: GET_LOCAL_0,
        name: "GET_LOCAL_0",
        arg_type: ArgType::None,
        stack_effect: 1,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: GET_LOCAL_1,
        name: "GET_LOCAL_1",
        arg_type: ArgType::None,
        stack_effect: 1,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: GET_LOCAL_2,
        name: "GET_LOCAL_2",
        arg_type: ArgType::None,
        stack_effect: 1,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: GET_LOCAL_3,
        name: "GET_LOCAL_3",
        arg_type: ArgType::None,
        stack_effect: 1,
        compute_cost: 1,
    },

    // Nibble immediate SET_LOCAL operations (0xD4-0xD7)
    OpcodeInfo {
        opcode: SET_LOCAL_0,
        name: "SET_LOCAL_0",
        arg_type: ArgType::None,
        stack_effect: -1,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: SET_LOCAL_1,
        name: "SET_LOCAL_1",
        arg_type: ArgType::None,
        stack_effect: -1,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: SET_LOCAL_2,
        name: "SET_LOCAL_2",
        arg_type: ArgType::None,
        stack_effect: -1,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: SET_LOCAL_3,
        name: "SET_LOCAL_3",
        arg_type: ArgType::None,
        stack_effect: -1,
        compute_cost: 1,
    },

    // Nibble immediate PUSH constant operations (0xD8-0xDB)
    OpcodeInfo {
        opcode: PUSH_0,
        name: "PUSH_0",
        arg_type: ArgType::None,
        stack_effect: 1,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: PUSH_1,
        name: "PUSH_1",
        arg_type: ArgType::None,
        stack_effect: 1,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: PUSH_2,
        name: "PUSH_2",
        arg_type: ArgType::None,
        stack_effect: 1,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: PUSH_3,
        name: "PUSH_3",
        arg_type: ArgType::None,
        stack_effect: 1,
        compute_cost: 1,
    },

    // Nibble immediate LOAD_PARAM operations (0xDC-0xDF)
    OpcodeInfo {
        opcode: LOAD_PARAM_0,
        name: "LOAD_PARAM_0",
        arg_type: ArgType::None,
        stack_effect: 1,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: LOAD_PARAM_1,
        name: "LOAD_PARAM_1",
        arg_type: ArgType::None,
        stack_effect: 1,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: LOAD_PARAM_2,
        name: "LOAD_PARAM_2",
        arg_type: ArgType::None,
        stack_effect: 1,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: LOAD_PARAM_3,
        name: "LOAD_PARAM_3",
        arg_type: ArgType::None,
        stack_effect: 1,
        compute_cost: 1,
    },
    // Local variable operations
    OpcodeInfo {
        opcode: CLEAR_LOCAL,
        name: "CLEAR_LOCAL",
        arg_type: ArgType::LocalIndex,
        stack_effect: 0,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: WRITE_DATA,
        name: "WRITE_DATA",
        arg_type: ArgType::None,
        stack_effect: -2,
        compute_cost: 2,
    },
    OpcodeInfo {
        opcode: DATA_LEN,
        name: "DATA_LEN",
        arg_type: ArgType::None,
        stack_effect: 0,
        compute_cost: 1,
    },
    OpcodeInfo {
        opcode: EMIT_EVENT,
        name: "EMIT_EVENT",
        arg_type: ArgType::None,
        stack_effect: -1,
        compute_cost: 10,
    },
    OpcodeInfo {
        opcode: LOG_DATA,
        name: "LOG_DATA",
        arg_type: ArgType::None,
        stack_effect: -1,
        compute_cost: 5,
    },
    OpcodeInfo {
        opcode: GET_SIGNER_KEY,
        name: "GET_SIGNER_KEY",
        arg_type: ArgType::None,
        stack_effect: 1,
        compute_cost: 2,
    },
    // Array operations
    OpcodeInfo {
        opcode: CREATE_ARRAY,
        name: "CREATE_ARRAY",
        arg_type: ArgType::U8,
        stack_effect: 1,
        compute_cost: 3,
    },
    OpcodeInfo {
        opcode: ARRAY_SET,
        name: "ARRAY_SET",
        arg_type: ArgType::None,
        stack_effect: -3,
        compute_cost: 2,
    },
    OpcodeInfo {
        opcode: ARRAY_GET,
        name: "ARRAY_GET",
        arg_type: ArgType::None,
        stack_effect: -1,
        compute_cost: 2,
    },
];

/// Get opcode information by opcode value (zero-allocation lookup)
#[inline]
pub const fn get_opcode_info(opcode: u8) -> Option<&'static OpcodeInfo> {
    let mut i = 0;
    while i < OPCODE_TABLE.len() {
        if OPCODE_TABLE[i].opcode == opcode {
            return Some(&OPCODE_TABLE[i]);
        }
        i += 1;
    }
    None
}

/// Check if opcode is valid (zero-allocation)
#[inline]
pub const fn is_valid_opcode(opcode: u8) -> bool {
    get_opcode_info(opcode).is_some()
}

/// Get opcode name for debugging (zero-allocation)
#[inline]
pub const fn opcode_name(opcode: u8) -> &'static str {
    match get_opcode_info(opcode) {
        Some(info) => info.name,
        None => "UNKNOWN",
    }
}

/// Get opcode compute cost (zero-allocation)
#[inline]
pub const fn opcode_compute_cost(opcode: u8) -> u8 {
    match get_opcode_info(opcode) {
        Some(info) => info.compute_cost,
        None => 1, // Default minimal cost
    }
}
