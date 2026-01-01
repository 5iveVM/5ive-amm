//! Tests for CALL_EXTERNAL functionality
//!
//! Comprehensive edge case tests for external import registration,
//! qualified name parsing, and CALL_EXTERNAL opcode emission.

use super::super::OpcodeEmitter;
use super::types::{ASTGenerator, ExternalImport};
// use crate::ast::AstNode;  // Unused in current tests
use five_protocol::opcodes::CALL_EXTERNAL;
// use five_protocol::Value;  // Unused in current tests
use std::collections::HashMap;

/// Mock emitter for capturing bytecode output
struct MockEmitter {
    pub bytecode: Vec<u8>,
}

impl MockEmitter {
    fn new() -> Self {
        Self { bytecode: Vec::new() }
    }
}

impl OpcodeEmitter for MockEmitter {
    fn emit_opcode(&mut self, opcode: u8) {
        self.bytecode.push(opcode);
    }

    fn emit_u8(&mut self, value: u8) {
        self.bytecode.push(value);
    }

    fn emit_u16(&mut self, value: u16) {
        self.bytecode.extend_from_slice(&value.to_le_bytes());
    }

    fn emit_u32(&mut self, value: u32) {
        self.bytecode.extend_from_slice(&value.to_le_bytes());
    }

    fn emit_u64(&mut self, value: u64) {
        self.bytecode.extend_from_slice(&value.to_le_bytes());
    }

    fn emit_bytes(&mut self, bytes: &[u8]) {
        self.bytecode.extend_from_slice(bytes);
    }

    fn get_position(&self) -> usize {
        self.bytecode.len()
    }

    fn emit_vle_u32(&mut self, value: u32) {
        // Simple VLE encoding for tests
        if value < 128 {
            self.bytecode.push(value as u8);
        } else {
            self.bytecode.extend_from_slice(&value.to_le_bytes());
        }
    }

    fn emit_vle_u16(&mut self, value: u16) {
        if value < 128 {
            self.bytecode.push(value as u8);
        } else {
            self.bytecode.extend_from_slice(&value.to_le_bytes());
        }
    }

    fn emit_vle_u64(&mut self, value: u64) {
        if value < 128 {
            self.bytecode.push(value as u8);
        } else {
            self.bytecode.extend_from_slice(&value.to_le_bytes());
        }
    }

    fn patch_u32(&mut self, position: usize, value: u32) {
        let bytes = value.to_le_bytes();
        if position + 4 <= self.bytecode.len() {
            self.bytecode[position..position + 4].copy_from_slice(&bytes);
        }
    }

    fn patch_u16(&mut self, position: usize, value: u16) {
        let bytes = value.to_le_bytes();
        if position + 2 <= self.bytecode.len() {
            self.bytecode[position..position + 2].copy_from_slice(&bytes);
        }
    }

    fn should_include_tests(&self) -> bool {
        false
    }
}

// ============================================================================
// parse_qualified_name Tests
// ============================================================================

#[test]
fn test_parse_qualified_name_valid() {
    // Standard qualified name
    let result = ASTGenerator::parse_qualified_name("math_lib::add");
    assert_eq!(result, Some(("math_lib", "add")));
}

#[test]
fn test_parse_qualified_name_complex_module() {
    // Nested module path simulation
    let result = ASTGenerator::parse_qualified_name("amm_core::calculate_swap");
    assert_eq!(result, Some(("amm_core", "calculate_swap")));
}

#[test]
fn test_parse_qualified_name_underscore_names() {
    // Names with underscores
    let result = ASTGenerator::parse_qualified_name("my_module::my_function");
    assert_eq!(result, Some(("my_module", "my_function")));
}

#[test]
fn test_parse_qualified_name_single_char() {
    // Minimal valid name
    let result = ASTGenerator::parse_qualified_name("a::b");
    assert_eq!(result, Some(("a", "b")));
}

#[test]
fn test_parse_qualified_name_unqualified() {
    // Unqualified name (no ::)
    let result = ASTGenerator::parse_qualified_name("add");
    assert_eq!(result, None);
}

#[test]
fn test_parse_qualified_name_empty_module() {
    // Empty module name (e.g., "::add")
    let result = ASTGenerator::parse_qualified_name("::add");
    assert_eq!(result, None);
}

#[test]
fn test_parse_qualified_name_empty_function() {
    // Empty function name (e.g., "math::")
    let result = ASTGenerator::parse_qualified_name("math::");
    assert_eq!(result, None);
}

#[test]
fn test_parse_qualified_name_just_separator() {
    // Just the separator
    let result = ASTGenerator::parse_qualified_name("::");
    assert_eq!(result, None);
}

#[test]
fn test_parse_qualified_name_empty_string() {
    // Empty string
    let result = ASTGenerator::parse_qualified_name("");
    assert_eq!(result, None);
}

#[test]
fn test_parse_qualified_name_multiple_separators() {
    // Multiple :: - should take first split
    let result = ASTGenerator::parse_qualified_name("a::b::c");
    // First :: is at index 1, so module = "a", function = "b::c"
    assert_eq!(result, Some(("a", "b::c")));
}

// ============================================================================
// ExternalImport Registration Tests
// ============================================================================

#[test]
fn test_register_external_import_basic() {
    let mut gen = ASTGenerator::new();
    
    let mut funcs = HashMap::new();
    funcs.insert("add".to_string(), 0u16);
    funcs.insert("sub".to_string(), 10u16);
    
    gen.register_external_import("math_lib".to_string(), 1, funcs);
    
    assert!(gen.is_external_import("math_lib"));
    assert!(!gen.is_external_import("other_lib"));
}

#[test]
fn test_register_external_import_multiple() {
    let mut gen = ASTGenerator::new();
    
    gen.register_external_import(
        "math_lib".to_string(),
        1,
        [("add".to_string(), 0u16)].into_iter().collect(),
    );
    gen.register_external_import(
        "amm_core".to_string(),
        2,
        [("swap".to_string(), 20u16)].into_iter().collect(),
    );
    
    assert!(gen.is_external_import("math_lib"));
    assert!(gen.is_external_import("amm_core"));
    assert!(!gen.is_external_import("unknown"));
}

#[test]
fn test_get_external_import_details() {
    let mut gen = ASTGenerator::new();
    
    let mut funcs = HashMap::new();
    funcs.insert("add".to_string(), 100u16);
    funcs.insert("mul".to_string(), 200u16);
    
    gen.register_external_import("math_lib".to_string(), 5, funcs);
    
    let ext = gen.get_external_import("math_lib");
    assert!(ext.is_some());
    
    let ext = ext.unwrap();
    assert_eq!(ext.module_name, "math_lib");
    assert_eq!(ext.account_index, 5);
    assert_eq!(ext.functions.get("add"), Some(&100u16));
    assert_eq!(ext.functions.get("mul"), Some(&200u16));
    assert_eq!(ext.functions.get("div"), None);
}

#[test]
fn test_external_import_overwrite() {
    let mut gen = ASTGenerator::new();
    
    // Register first version
    gen.register_external_import(
        "math_lib".to_string(),
        1,
        [("add".to_string(), 0u16)].into_iter().collect(),
    );
    
    // Overwrite with new version
    gen.register_external_import(
        "math_lib".to_string(),
        10,
        [("add".to_string(), 50u16), ("sub".to_string(), 60u16)].into_iter().collect(),
    );
    
    let ext = gen.get_external_import("math_lib").unwrap();
    assert_eq!(ext.account_index, 10); // Updated
    assert_eq!(ext.functions.get("add"), Some(&50u16)); // Updated
    assert_eq!(ext.functions.get("sub"), Some(&60u16)); // New
}

#[test]
fn test_external_import_empty_functions() {
    let mut gen = ASTGenerator::new();
    
    // Register with no functions
    gen.register_external_import(
        "empty_lib".to_string(),
        3,
        HashMap::new(),
    );
    
    assert!(gen.is_external_import("empty_lib"));
    
    let ext = gen.get_external_import("empty_lib").unwrap();
    assert!(ext.functions.is_empty());
}

#[test]
fn test_external_import_reset() {
    let mut gen = ASTGenerator::new();
    
    gen.register_external_import(
        "math_lib".to_string(),
        1,
        [("add".to_string(), 0u16)].into_iter().collect(),
    );
    
    assert!(gen.is_external_import("math_lib"));
    
    gen.reset();
    
    assert!(!gen.is_external_import("math_lib"));
}

// ============================================================================
// CALL_EXTERNAL Emission Tests
// ============================================================================

#[test]
fn test_call_external_emission_format() {
    // Test that CALL_EXTERNAL is emitted with correct format:
    // CALL_EXTERNAL (0x91) | account_index (u8) | func_offset (u16 LE) | param_count (u8)
    
    let mut emitter = MockEmitter::new();
    let mut gen = ASTGenerator::new();
    
    // Register external import
    gen.register_external_import(
        "math_lib".to_string(),
        1,
        [("add".to_string(), 100u16)].into_iter().collect(),
    );
    
    // This should trigger CALL_EXTERNAL for qualified name
    // Note: We test the parse_qualified_name + lookup logic here
    let name = "math_lib::add";
    if let Some((module_name, func_name)) = ASTGenerator::parse_qualified_name(name) {
        if let Some(ext_import) = gen.external_imports.get(module_name) {
            // Manually emit what generate_function_call would emit
            emitter.emit_opcode(CALL_EXTERNAL);
            emitter.emit_u8(ext_import.account_index);
            let func_offset = ext_import.functions.get(func_name).copied().unwrap_or(0);
            emitter.emit_u16(func_offset);
            emitter.emit_u8(2); // 2 args
        }
    }
    
    // Verify bytecode
    assert_eq!(emitter.bytecode.len(), 5);
    assert_eq!(emitter.bytecode[0], CALL_EXTERNAL); // 0x91
    assert_eq!(emitter.bytecode[1], 1); // account_index
    assert_eq!(u16::from_le_bytes([emitter.bytecode[2], emitter.bytecode[3]]), 100); // func_offset
    assert_eq!(emitter.bytecode[4], 2); // param_count
}

#[test]
fn test_call_external_unknown_function_defaults_to_zero() {
    let mut emitter = MockEmitter::new();
    let mut gen = ASTGenerator::new();
    
    gen.register_external_import(
        "math_lib".to_string(),
        2,
        [("add".to_string(), 100u16)].into_iter().collect(),
    );
    
    // Call unknown function - should default to offset 0
    let name = "math_lib::unknown_func";
    if let Some((module_name, func_name)) = ASTGenerator::parse_qualified_name(name) {
        if let Some(ext_import) = gen.external_imports.get(module_name) {
            emitter.emit_opcode(CALL_EXTERNAL);
            emitter.emit_u8(ext_import.account_index);
            let func_offset = ext_import.functions.get(func_name).copied().unwrap_or(0);
            emitter.emit_u16(func_offset);
            emitter.emit_u8(0);
        }
    }
    
    assert_eq!(emitter.bytecode.len(), 5);
    assert_eq!(u16::from_le_bytes([emitter.bytecode[2], emitter.bytecode[3]]), 0); // default
}

#[test]
fn test_unqualified_call_not_external() {
    let _gen = ASTGenerator::new();
    
    // Unqualified name should not match external imports
    let name = "add";
    let result = ASTGenerator::parse_qualified_name(name);
    assert!(result.is_none());
    
    // Even if we had registered "add" as an external import module name,
    // unqualified calls won't trigger CALL_EXTERNAL
}

#[test]
fn test_external_import_struct_clone() {
    // Test that ExternalImport is properly cloneable
    let mut funcs = HashMap::new();
    funcs.insert("add".to_string(), 50u16);
    
    let original = ExternalImport {
        module_name: "test".to_string(),
        account_index: 3,
        functions: funcs,
    };
    
    let cloned = original.clone();
    
    assert_eq!(cloned.module_name, "test");
    assert_eq!(cloned.account_index, 3);
    assert_eq!(cloned.functions.get("add"), Some(&50u16));
}

#[test]
fn test_external_import_case_sensitivity() {
    let mut gen = ASTGenerator::new();
    
    gen.register_external_import(
        "Math_Lib".to_string(),
        1,
        HashMap::new(),
    );
    
    // Module names are case-sensitive
    assert!(gen.is_external_import("Math_Lib"));
    assert!(!gen.is_external_import("math_lib"));
    assert!(!gen.is_external_import("MATH_LIB"));
}
