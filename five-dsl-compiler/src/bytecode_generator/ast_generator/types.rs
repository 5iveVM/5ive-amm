//! AST generator type definitions.

use super::super::account_system::AccountSystem;
use super::super::types::*;
use crate::ast::{InstructionParameter, TypeNode};
use crate::type_checker::InterfaceInfo;
use std::collections::HashMap;

/// Jump instruction patch info.
pub(super) struct JumpPatch {
    pub position: usize,
    pub target_label: String,
}

/// BR_EQ_U8 patch info.
pub(super) struct BrEqU8Patch {
    pub position: usize,
    pub target_label: String,
}

/// Function call patch info.
pub(super) struct FunctionPatch {
    pub position: usize,
    pub function_name: String,
}

/// BR_EQ_U8 pattern info.
pub(super) struct BrEqU8Info {
    pub variable_node: crate::ast::AstNode,
    pub u8_value: u8,
}

/// External import information for CALL_EXTERNAL generation
/// Tracks modules that should use cross-bytecode calls instead of inline
#[derive(Debug, Clone)]
pub struct ExternalImport {
    /// Name of the external module (e.g., "math_lib")
    pub module_name: String,
    /// Account index in the accounts array where the bytecode lives
    pub account_index: u8,
    /// Whether any function name is allowed (import-all mode)
    pub allow_any_function: bool,
    /// Mapping of function names to their offsets in the external bytecode
    /// In hash-selector mode this stores function-name -> selector hash.
    pub functions: HashMap<String, u16>,
}

/// AST Generator for recursive AST processing and bytecode generation
pub struct ASTGenerator {
    /// Symbol table for global fields
    pub(crate) global_symbol_table: HashMap<String, FieldInfo>,
    /// Symbol table for local variables and parameters
    pub(crate) local_symbol_table: HashMap<String, FieldInfo>,

    /// Type cache for avoiding repeated type inference
    pub(super) type_cache: HashMap<String, String>,

    /// Current expression depth for optimization decisions
    pub(super) expression_depth: usize,

    /// Loop context stack for break/continue handling
    pub(super) loop_stack: Vec<LoopContext>,

    /// Current field counter for symbol table management
    pub(crate) field_counter: u32,

    /// Account system for proper field offset resolution
    pub(super) account_system: Option<AccountSystem>,

    /// Current function context (None if not in a function, Some(name) if in a function)
    pub(super) current_function_context: Option<String>,

    /// Current function parameters for payer resolution in @init constraints
    pub(crate) current_function_parameters: Option<Vec<InstructionParameter>>,

    /// Current function return type for tuple return handling
    pub(super) current_function_return_type: Option<TypeNode>,

    /// User-defined function parameter types for call-site lowering.
    pub(super) function_parameter_types: HashMap<String, Vec<TypeNode>>,

    /// Jumps that need to be patched with correct offsets.
    pub(super) jump_patches: Vec<JumpPatch>,
    /// BR_EQ_U8 instructions that need to be patched with correct fixed-width offsets.
    pub(super) br_eq_u8_patches: Vec<BrEqU8Patch>,
    /// Function calls that need to be patched with correct addresses.
    pub(super) function_patches: Vec<FunctionPatch>,
    /// The positions of functions in the bytecode.
    pub(super) function_positions: HashMap<String, usize>,
    /// The positions of labels in the bytecode.
    pub(super) label_positions: HashMap<String, usize>,
    /// A counter to create unique labels.
    pub(super) label_counter: usize,
    /// Interface registry for interface method calls
    pub(super) interface_registry: HashMap<String, InterfaceInfo>,

    /// V2 preview mode flag for enabling optimizations
    #[allow(dead_code)]
    pub(super) v2_preview: bool,

    /// Resource tracking for V3 header generation
    pub(super) max_locals_used: u8,
    pub(super) max_stack_depth_seen: u16,
    pub(super) current_call_depth: u8,
    pub(super) max_call_depth_seen: u8,
    pub(super) string_literals_count: u16,
    pub(super) estimated_temp_usage: u8,
    pub(super) function_call_count: u16,
    /// Function name deduplication for bytecode metadata optimization
    pub(super) name_deduplication: super::super::types::NameDeduplication,

    /// Precomputed variable allocations from ScopeAnalyzer
    pub(super) precomputed_allocations: Option<HashMap<String, usize>>,

    /// External imports for CALL_EXTERNAL generation
    /// Maps module name to ExternalImport info
    pub(crate) external_imports: HashMap<String, ExternalImport>,
    /// Module alias/full-path -> interface name for module-qualified interface calls.
    pub(crate) module_interface_aliases: HashMap<String, String>,

}
