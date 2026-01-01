//! Comprehensive tests for AST generator functionality

use super::*;
use crate::ast::{AstNode, BlockKind, InstructionParameter, TypeNode};
use five_protocol::{opcodes::*, Value};

/// Mock emitter for testing
struct MockEmitter {
    bytecode: Vec<u8>,
    position: usize,
}

impl MockEmitter {
    fn new() -> Self {
        Self {
            bytecode: Vec::new(),
            position: 0,
        }
    }

    fn emit_u128(&mut self, value: u128) {
        self.bytecode.extend_from_slice(&value.to_le_bytes());
        self.position += 16;
    }

    fn get_bytecode(&self) -> &[u8] {
        &self.bytecode
    }
}

impl super::super::OpcodeEmitter for MockEmitter {
    fn emit_opcode(&mut self, opcode: u8) {
        self.bytecode.push(opcode);
        self.position += 1;
    }

    fn emit_u8(&mut self, value: u8) {
        self.bytecode.push(value);
        self.position += 1;
    }

    fn emit_u16(&mut self, value: u16) {
        self.bytecode.extend_from_slice(&value.to_le_bytes());
        self.position += 2;
    }

    fn emit_u32(&mut self, value: u32) {
        self.bytecode.extend_from_slice(&value.to_le_bytes());
        self.position += 4;
    }

    fn emit_u64(&mut self, value: u64) {
        self.bytecode.extend_from_slice(&value.to_le_bytes());
        self.position += 8;
    }

    fn emit_bytes(&mut self, bytes: &[u8]) {
        self.bytecode.extend_from_slice(bytes);
        self.position += bytes.len();
    }

    fn emit_vle_u16(&mut self, value: u16) {
        // Simplified VLE encoding for testing
        if value < 128 {
            self.emit_u8(value as u8);
        } else {
            self.emit_u8((value & 0x7F) as u8 | 0x80);
            self.emit_u8((value >> 7) as u8);
        }
    }

    fn emit_vle_u32(&mut self, value: u32) {
        // Simplified VLE encoding for testing
        if value < 128 {
            self.emit_u8(value as u8);
        } else if value < 16384 {
            self.emit_u8((value & 0x7F) as u8 | 0x80);
            self.emit_u8((value >> 7) as u8);
        } else {
            self.emit_u8((value & 0x7F) as u8 | 0x80);
            self.emit_u8(((value >> 7) & 0x7F) as u8 | 0x80);
            self.emit_u8((value >> 14) as u8);
        }
    }

    fn emit_vle_u64(&mut self, value: u64) {
        // Simplified VLE encoding for testing
        let mut v = value;
        loop {
            let byte = (v & 0x7F) as u8;
            v >>= 7;
            if v == 0 {
                self.emit_u8(byte);
                break;
            } else {
                self.emit_u8(byte | 0x80);
            }
        }
    }

    fn get_position(&self) -> usize {
        self.position
    }

    fn patch_u16(&mut self, position: usize, value: u16) {
        let bytes = value.to_le_bytes();
        self.bytecode[position] = bytes[0];
        self.bytecode[position + 1] = bytes[1];
    }

    fn patch_u32(&mut self, position: usize, value: u32) {
        let bytes = value.to_le_bytes();
        self.bytecode[position..position + 4].copy_from_slice(&bytes);
    }

    fn should_include_tests(&self) -> bool {
        false
    }
}

#[test]
fn test_local_variable_generation() {
    let mut generator = ASTGenerator::new();
    let mut emitter = MockEmitter::new();

    let let_stmt = AstNode::LetStatement {
        name: "x".to_string(),
        type_annotation: Some(Box::new(TypeNode::Primitive("u64".to_string()))),
        value: Box::new(AstNode::Literal(Value::U64(42))),
        is_mutable: true,
    };

    assert!(generator.generate_ast_node(&mut emitter, &let_stmt).is_ok());
    assert_eq!(generator.field_counter, 1);
    assert!(generator.local_symbol_table.contains_key("x"));
}

#[test]
fn test_function_parameter_handling() {
    let mut generator = ASTGenerator::new();
    let mut emitter = MockEmitter::new();

    let func_def = AstNode::InstructionDefinition {
        name: "test_func".to_string(),
        visibility: crate::Visibility::Public,
        parameters: vec![InstructionParameter {
            name: "param1".to_string(),
            param_type: TypeNode::Primitive("u64".to_string()),
            is_optional: false,
            default_value: None,
            attributes: vec![],
            is_init: false,
            init_config: None,
        }],
        return_type: Some(Box::new(TypeNode::Primitive("u64".to_string()))),
        body: Box::new(AstNode::Block {
            statements: vec![AstNode::ReturnStatement {
                value: Some(Box::new(AstNode::Identifier("param1".to_string()))),
            }],
            kind: BlockKind::Regular,
        }),
        is_public: true,
    };

    assert!(generator.generate_ast_node(&mut emitter, &func_def).is_ok());

    // Verify parameter is marked correctly
    let param_info = generator.local_symbol_table.get("param1");
    assert!(param_info.is_some());
    assert!(param_info.unwrap().is_parameter); // This should be true now!
}

#[test]
fn test_binary_expression_generation() {
    let mut generator = ASTGenerator::new();
    let mut emitter = MockEmitter::new();

    let binary_expr = AstNode::BinaryExpression {
        left: Box::new(AstNode::Literal(Value::U64(10))),
        right: Box::new(AstNode::Literal(Value::U64(20))),
        operator: "+".to_string(),
    };

    assert!(generator
        .generate_ast_node(&mut emitter, &binary_expr)
        .is_ok());
    // Should generate bytecode for literals and ADD opcode
    assert!(!emitter.bytecode.is_empty());
}

#[test]
fn test_constant_folding() {
    let mut generator = ASTGenerator::new();
    let mut emitter = MockEmitter::new();

    let folded_expr = AstNode::BinaryExpression {
        left: Box::new(AstNode::Literal(Value::U64(5))),
        right: Box::new(AstNode::Literal(Value::U64(3))),
        operator: "+".to_string(),
    };

    assert!(generator
        .generate_ast_node(&mut emitter, &folded_expr)
        .is_ok());
    // Constant folding should emit PUSH_U16 with value 8
}

#[test]
fn test_if_statement_generation() {
    let mut generator = ASTGenerator::new();
    let mut emitter = MockEmitter::new();

    let if_stmt = AstNode::IfStatement {
        condition: Box::new(AstNode::Literal(Value::Bool(true))),
        then_branch: Box::new(AstNode::Block {
            statements: vec![],
            kind: BlockKind::Regular,
        }),
        else_branch: None,
    };

    assert!(generator.generate_ast_node(&mut emitter, &if_stmt).is_ok());
}

#[test]
fn test_jump_patching() {
    let mut generator = ASTGenerator::new();
    let mut emitter = MockEmitter::new();

    // Create a label and jump to it
    let label = generator.new_label();
    generator.emit_jump(&mut emitter, JUMP, label.clone());
    let _jump_pos = emitter.get_position();

    generator.place_label(&mut emitter, label);

    // Patch jumps
    assert!(generator.patch(&mut emitter).is_ok());
}

#[test]
fn test_v2_preview_local_variables() {
    let mut generator = ASTGenerator::with_v2_preview(true);
    let mut emitter = MockEmitter::new();

    // Add a local variable at index 0
    generator.add_local_field("x".to_string(), "u64".to_string(), true, false);

    // Emit SET_LOCAL for index 0 - should use SET_LOCAL_0 in v2 mode
    generator.emit_set_local(&mut emitter, 0, "test");

    // Check that SET_LOCAL_0 (nibble immediate) was emitted
    // The exact opcode value depends on the protocol definition
    assert!(!emitter.bytecode.is_empty());
}

// #[test]
// fn test_account_field_helpers() {
//     let generator = ASTGenerator::new();
//
//     // Test helper functions (these return false/None when no account system)
//     assert_eq!(
//         generator.is_account_field_optional("TestAccount", "balance"),
//         false
//     );
//     assert_eq!(
//         generator.get_account_field_type("TestAccount", "balance"),
//         None
//     );
// }

#[test]
fn test_builtin_method_emission() {
    let generator = ASTGenerator::new();
    let mut emitter = MockEmitter::new();

    // Test that built-in methods are recognized
    assert!(generator
        .try_emit_builtin_method(&mut emitter, "add")
        .is_some());
    assert!(generator
        .try_emit_builtin_method(&mut emitter, "sub")
        .is_some());
    assert!(generator
        .try_emit_builtin_method(&mut emitter, "unknown_method")
        .is_none());
}

#[test]
fn test_error_handling_invalid_script() {
    let mut generator = ASTGenerator::new();
    let mut emitter = MockEmitter::new();

    // Try to access undefined identifier
    let undefined_id = AstNode::Identifier("undefined_variable".to_string());

    let result = generator.generate_ast_node(&mut emitter, &undefined_id);
    assert!(result.is_err());
}

#[test]
fn test_resource_tracking() {
    let mut generator = ASTGenerator::new();

    generator.track_local_variable("temp");
    generator.track_function_call();
    generator.track_string_literal("test");

    let requirements = generator.get_enhanced_resource_requirements();
    assert_eq!(requirements.max_locals, 1);
    assert_eq!(requirements.max_call_depth, 1); // Initial 1 (call depth tracked)
}

#[test]
fn test_symbol_table_operations() {
    let mut generator = ASTGenerator::new();

    generator.add_local_field("x".to_string(), "u64".to_string(), true, false);
    generator.add_local_field("y".to_string(), "u128".to_string(), false, false);

    assert_eq!(generator.get_field_counter(), 2);
    assert_eq!(generator.get_symbol_table().len(), 2);

    let cloned = generator.clone_symbol_table();
    assert_eq!(cloned.len(), 2);
}

#[test]
fn test_type_node_conversion() {
    let generator = ASTGenerator::new();

    let primitive_type = TypeNode::Primitive("u64".to_string());
    assert_eq!(generator.type_node_to_string(&primitive_type), "u64");

    let array_type = TypeNode::Array {
        element_type: Box::new(TypeNode::Primitive("u8".to_string())),
        size: Some(32),
    };
    assert_eq!(generator.type_node_to_string(&array_type), "[u8; 32]");
}

#[test]
fn test_call_deduplication_safety() {
    let mut generator = ASTGenerator::new();
    let mut emitter = MockEmitter::new();

    // First call should succeed
    let result1 = generator.emit_call_with_deduplication(&mut emitter, 1, 0, "test_fn");
    assert!(result1.is_ok());

    // Second call with same name should also succeed (uses reference)
    let result2 = generator.emit_call_with_deduplication(&mut emitter, 1, 0, "test_fn");
    assert!(result2.is_ok());
}
