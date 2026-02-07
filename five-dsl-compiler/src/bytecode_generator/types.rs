// Type definitions for the bytecode generator module
//
// This module contains all data structures used throughout the bytecode generation process.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Information about a field/variable in the symbol table
#[derive(Debug, Clone)]
pub struct FieldInfo {
    pub offset: u32,
    pub field_type: String,
    pub is_mutable: bool,
    pub is_optional: bool,
    /// True if this is a function parameter (uses LOAD_PARAM), false if local variable (uses GET_LOCAL)
    pub is_parameter: bool,
}

/// Loop context for proper break/continue handling
#[derive(Debug, Clone)]
pub struct LoopContext {
    pub loop_start: usize,
    pub break_targets: Vec<usize>,
    pub continue_targets: Vec<usize>,
}

/// Function information for dispatch table
#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub name: String,
    pub offset: usize,
    pub parameter_count: u8,
    pub is_public: bool, // true for pub functions, false for private
    pub has_return_type: bool, // true if function has a return type (not void)
}

/// ABI structures for frontend integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ABIParameter {
    pub name: String,
    pub param_type: String,
    pub is_account: bool,
    pub attributes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ABIFunction {
    pub name: String,
    pub index: u8,
    pub parameters: Vec<ABIParameter>,
    pub return_type: Option<String>,
    pub is_public: bool, // Function visibility
    pub bytecode_offset: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ABIField {
    pub name: String,
    pub field_type: String,
    pub is_mutable: bool,
    pub memory_offset: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FIVEABI {
    pub program_name: String,
    pub functions: Vec<ABIFunction>,
    pub fields: Vec<ABIField>,
    pub version: String,
}

// New simplified ABI format for function calls
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleABIParameter {
    pub name: String,
    #[serde(rename = "type")]
    pub param_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleABIAccount {
    pub name: String,
    pub writable: bool,
    pub signer: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleABIFunction {
    pub index: u8,
    pub parameters: Vec<SimpleABIParameter>,
    pub accounts: Vec<SimpleABIAccount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleABI {
    pub version: String,
    pub name: String,
    pub functions: HashMap<String, SimpleABIFunction>,
}

/// Account type information for the account registry
#[derive(Debug, Clone)]
pub struct AccountTypeInfo {
    pub name: String,
    pub fields: HashMap<String, FieldInfo>,
    pub total_size: u32,
}

/// Account registry for storing account type definitions
#[derive(Debug, Clone)]
pub struct AccountRegistry {
    pub account_types: HashMap<String, AccountTypeInfo>,
}

impl AccountRegistry {
    pub fn new() -> Self {
        Self {
            account_types: HashMap::new(),
        }
    }

    /// Get reference to account type definitions
    pub fn get_account_definitions(&self) -> &HashMap<String, AccountTypeInfo> {
        &self.account_types
    }
}

impl Default for AccountRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Constraint deduplication entry for optimization
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ConstraintKey {
    pub account_index: u8,
    pub constraint_type: u8,
}

/// Deduplication analysis for function constraints
#[derive(Debug, Clone)]
pub struct ConstraintDeduplication {
    /// Map of constraint key to list of parameter names that need it
    pub constraint_map: HashMap<ConstraintKey, Vec<String>>,
    /// Deduplicated constraint table: (account_index, constraint_mask)
    pub dedupe_table: Vec<(u8, u8)>,
}

impl ConstraintDeduplication {
    pub fn new() -> Self {
        Self {
            constraint_map: HashMap::new(),
            dedupe_table: Vec::new(),
        }
    }
}

impl Default for ConstraintDeduplication {
    fn default() -> Self {
        Self::new()
    }
}

/// Phase 4: Cross-function constraint analysis
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct GlobalConstraintPattern {
    pub constraint_type: u8,
    pub account_pattern: String, // e.g., "caller: pubkey @signer"
    pub functions: Vec<String>,  // Functions that use this pattern
}

/// Phase 4: Constraint lifting analysis
#[derive(Debug, Clone)]
pub struct ConstraintLifting {
    /// Constraints that can be moved to script initialization
    pub lifted_constraints: Vec<(u8, u8)>, // (account_index, constraint_mask)
    /// Account validations that need to be cached
    pub cache_targets: HashMap<u8, u8>, // account_index -> constraint_mask
}

impl ConstraintLifting {
    pub fn new() -> Self {
        Self {
            lifted_constraints: Vec::new(),
            cache_targets: HashMap::new(),
        }
    }
}

impl Default for ConstraintLifting {
    fn default() -> Self {
        Self::new()
    }
}

/// Phase 4: Constraint complexity grouping
#[derive(Debug, Clone)]
pub struct ConstraintComplexityGroup {
    /// Simple constraints (single validation)
    pub simple: Vec<(u8, u8)>, // (account_index, constraint_type)
    /// Medium constraints (2-3 validations)
    pub medium: Vec<(u8, u8)>, // (account_index, constraint_mask)
    /// Complex constraints (4+ validations or expensive operations)
    pub complex: Vec<(u8, u8)>, // (account_index, constraint_mask)
}

impl ConstraintComplexityGroup {
    pub fn new() -> Self {
        Self {
            simple: Vec::new(),
            medium: Vec::new(),
            complex: Vec::new(),
        }
    }
}

impl Default for ConstraintComplexityGroup {
    fn default() -> Self {
        Self::new()
    }
}

/// Phase 4: Advanced constraint optimization system
#[derive(Debug, Clone)]
pub struct AdvancedConstraintOptimization {
    /// Cross-function constraint patterns analysis
    pub global_patterns: HashMap<String, GlobalConstraintPattern>,
    /// Constraint lifting optimization
    pub constraint_lifting: ConstraintLifting,
    /// Complexity-based constraint grouping
    pub complexity_groups: ConstraintComplexityGroup,
    /// Script-level constraint initialization
    pub script_init_constraints: Vec<(u8, u8)>, // (account_index, constraint_mask)
}

impl AdvancedConstraintOptimization {
    pub fn new() -> Self {
        Self {
            global_patterns: HashMap::new(),
            constraint_lifting: ConstraintLifting::new(),
            complexity_groups: ConstraintComplexityGroup::new(),
            script_init_constraints: Vec::new(),
        }
    }
}

impl Default for AdvancedConstraintOptimization {
    fn default() -> Self {
        Self::new()
    }
}

// Additional compression-related opcodes
// These extend the five_protocol opcodes for compression optimizations

/// Pattern compression opcode for common instruction sequences
pub const OP_PATTERN: u8 = 0xF0;

/// Bulk push operations for multiple literals
pub const BULK_PUSH_2: u8 = 0xF1;
pub const BULK_PUSH_3: u8 = 0xF2;

/// Large program chunking operations - REMOVED
/// Chunk operations are no longer supported. The 0xF3-0xFA range is available for future use.

// Performance optimization opcodes
// These extend the five_protocol opcodes for performance optimizations

/// Bulk operation opcodes for multiple values
pub const BULK_PUSH_N: u8 = 0xE0;
pub const BULK_FIELD_ACCESS: u8 = 0xE1;
pub const ARITHMETIC_CHAIN: u8 = 0xE2;

/// Account access optimizations - using standard LOAD_FIELD/STORE_FIELD with VLE + zero-copy by default
pub const BATCH_ACCOUNT_ACCESS: u8 = 0xE5;

/// Scope management opcodes
pub const SCOPE_ALLOC: u8 = 0xE8;
pub const SCOPE_DEALLOC: u8 = 0xE9;

/// Function name deduplication tracker for bytecode metadata optimization
#[derive(Debug, Clone)]
pub struct NameDeduplication {
    /// Map of function name to its first occurrence position in bytecode
    pub name_positions: HashMap<String, usize>,
    /// Track the order names were first seen for stable indexing
    pub name_order: Vec<String>,
}

impl NameDeduplication {
    pub fn new() -> Self {
        Self {
            name_positions: HashMap::new(),
            name_order: Vec::new(),
        }
    }

    /// Record first occurrence of a function name
    pub fn record_name(&mut self, name: &str, position: usize) -> bool {
        if self.name_positions.contains_key(name) {
            false // Already seen
        } else {
            self.name_positions.insert(name.to_string(), position);
            self.name_order.push(name.to_string());
            true // First occurrence
        }
    }

    /// Get index of a previously seen function name
    pub fn get_name_index(&self, name: &str) -> Option<usize> {
        self.name_order.iter().position(|n| n == name)
    }
}

impl Default for NameDeduplication {
    fn default() -> Self {
        Self::new()
    }
}
