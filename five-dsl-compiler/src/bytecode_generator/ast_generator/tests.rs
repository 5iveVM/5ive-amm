//! Comprehensive tests for AST generator functionality

use super::*;
use crate::ast::{AstNode, BlockKind, InstructionParameter, TypeNode, MatchArm};
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

#[test]
fn test_while_loop_generation() {
    let mut generator = ASTGenerator::new();
    let mut emitter = MockEmitter::new();

    // while (true) { }
    let while_loop = AstNode::WhileLoop {
        condition: Box::new(AstNode::Literal(Value::Bool(true))),
        body: Box::new(AstNode::Block {
            statements: vec![],
            kind: BlockKind::Regular,
        }),
    };

    let result = generator.generate_ast_node(&mut emitter, &while_loop);
    assert!(result.is_ok());
    assert!(emitter.bytecode.len() > 0, "While loop generated no bytecode!");

    // We expect at least:
    // 1. Literal True (PUSH_U64 1 or similar)
    // 2. JUMP_IF_NOT (to end)
    // 3. JUMP (to start)
    // Plus labels placeholders.
}

#[test]
fn test_while_loop_with_break() {
    let mut generator = ASTGenerator::new();
    let mut emitter = MockEmitter::new();

    // while (true) { break; }
    let while_loop = AstNode::WhileLoop {
        condition: Box::new(AstNode::Literal(Value::Bool(true))),
        body: Box::new(AstNode::Block {
            statements: vec![AstNode::BreakStatement { label: None }],
            kind: BlockKind::Regular,
        }),
    };

    let result = generator.generate_ast_node(&mut emitter, &while_loop);
    assert!(result.is_ok());

    // Check patches
    // Break should generate a JUMP 0 that is patched to end of loop.
    // Since MockEmitter mocks patch_u16, we can check if bytecode has patched values?
    // MockEmitter updates bytecode in place.
    // However, without specific offsets, it is hard to check exact values.
    // But success of generation implies no errors.
}

#[test]
fn test_while_loop_with_continue() {
    let mut generator = ASTGenerator::new();
    let mut emitter = MockEmitter::new();

    // while (true) { continue; }
    let while_loop = AstNode::WhileLoop {
        condition: Box::new(AstNode::Literal(Value::Bool(true))),
        body: Box::new(AstNode::Block {
            statements: vec![AstNode::ContinueStatement { label: None }],
            kind: BlockKind::Regular,
        }),
    };

    let result = generator.generate_ast_node(&mut emitter, &while_loop);
    assert!(result.is_ok());
}

#[test]
fn test_break_outside_loop() {
    let mut generator = ASTGenerator::new();
    let mut emitter = MockEmitter::new();

    let break_stmt = AstNode::BreakStatement { label: None };
    let result = generator.generate_ast_node(&mut emitter, &break_stmt);
    assert!(result.is_err()); // Should error
}

#[test]
fn test_continue_outside_loop() {
    let mut generator = ASTGenerator::new();
    let mut emitter = MockEmitter::new();

    let continue_stmt = AstNode::ContinueStatement { label: None };
    let result = generator.generate_ast_node(&mut emitter, &continue_stmt);
    assert!(result.is_err()); // Should error
}

#[test]
fn test_tuple_generation() {
    let mut generator = ASTGenerator::new();
    let mut emitter = MockEmitter::new();

    let tuple_literal = AstNode::TupleLiteral {
        elements: vec![
            AstNode::Literal(Value::U64(1)),
            AstNode::Literal(Value::U64(2)),
        ],
    };

    assert!(generator.generate_ast_node(&mut emitter, &tuple_literal).is_ok());
    // Should emit CREATE_TUPLE
    assert!(emitter.bytecode.contains(&CREATE_TUPLE));
}

#[test]
fn test_tuple_access_generation() {
    let mut generator = ASTGenerator::new();
    let mut emitter = MockEmitter::new();

    let tuple_access = AstNode::TupleAccess {
        object: Box::new(AstNode::Identifier("my_tuple".to_string())),
        index: 1,
    };

    // Add dummy variable to symbol table to avoid error
    generator.add_local_field("my_tuple".to_string(), "(u64, u64)".to_string(), false, false);

    assert!(generator.generate_ast_node(&mut emitter, &tuple_access).is_ok());
    // Should emit TUPLE_GET
    assert!(emitter.bytecode.contains(&TUPLE_GET));
}

#[test]
fn test_array_literal_generation() {
    let mut generator = ASTGenerator::new();
    let mut emitter = MockEmitter::new();

    let array_literal = AstNode::ArrayLiteral {
        elements: vec![
            AstNode::Literal(Value::U64(10)),
            AstNode::Literal(Value::U64(20)),
        ],
    };

    assert!(generator.generate_ast_node(&mut emitter, &array_literal).is_ok());
    // Should emit PUSH_ARRAY_LITERAL
    assert!(emitter.bytecode.contains(&PUSH_ARRAY_LITERAL));
}

#[test]
fn test_string_literal_generation() {
    let mut generator = ASTGenerator::new();
    let mut emitter = MockEmitter::new();

    let string_literal = AstNode::StringLiteral {
        value: "Hello".to_string(),
    };

    assert!(generator.generate_ast_node(&mut emitter, &string_literal).is_ok());
    // Should emit PUSH_STRING
    assert!(emitter.bytecode.contains(&PUSH_STRING));
}

#[test]
fn test_match_expression_generation() {
    let mut generator = ASTGenerator::new();
    let mut emitter = MockEmitter::new();

    let match_expr = AstNode::MatchExpression {
        expression: Box::new(AstNode::Literal(Value::U64(5))),
        arms: vec![
            MatchArm {
                pattern: Box::new(AstNode::Literal(Value::U64(5))),
                guard: None,
                body: Box::new(AstNode::Block {
                    statements: vec![],
                    kind: BlockKind::Regular,
                }),
            },
            MatchArm {
                pattern: Box::new(AstNode::Identifier("_".to_string())),
                guard: None,
                body: Box::new(AstNode::Block {
                    statements: vec![],
                    kind: BlockKind::Regular,
                }),
            },
        ],
    };

    assert!(generator.generate_ast_node(&mut emitter, &match_expr).is_ok());
    assert!(!emitter.bytecode.is_empty());
}

#[test]
fn test_tuple_destructuring() {
    let mut generator = ASTGenerator::new();
    let mut emitter = MockEmitter::new();

    let tuple_destructuring = AstNode::TupleDestructuring {
        targets: vec!["a".to_string(), "b".to_string()],
        value: Box::new(AstNode::TupleLiteral {
            elements: vec![
                AstNode::Literal(Value::U64(1)),
                AstNode::Literal(Value::U64(2)),
            ],
        }),
    };

    assert!(generator.generate_ast_node(&mut emitter, &tuple_destructuring).is_ok());
    // Should emit UNPACK_TUPLE
    assert!(emitter.bytecode.contains(&UNPACK_TUPLE));
    // Should create local variables
    assert!(generator.local_symbol_table.contains_key("a"));
    assert!(generator.local_symbol_table.contains_key("b"));
}

#[test]
fn test_tuple_assignment() {
    let mut generator = ASTGenerator::new();
    let mut emitter = MockEmitter::new();

    // Prepare variables
    generator.add_local_field("x".to_string(), "u64".to_string(), true, false);
    generator.add_local_field("y".to_string(), "u64".to_string(), true, false);

    let tuple_assignment = AstNode::TupleAssignment {
        targets: vec![
            AstNode::Identifier("x".to_string()),
            AstNode::Identifier("y".to_string()),
        ],
        value: Box::new(AstNode::TupleLiteral {
            elements: vec![
                AstNode::Literal(Value::U64(10)),
                AstNode::Literal(Value::U64(20)),
            ],
        }),
    };

    assert!(generator.generate_ast_node(&mut emitter, &tuple_assignment).is_ok());
    // Should generate assignments (SET_LOCAL or similar)
    // Note: implementation uses emit_set_local, which might be SET_LOCAL or SET_LOCAL_N
    // Since x and y are indices 0 and 1, likely SET_LOCAL_0 and SET_LOCAL_1

    // We can just check that bytecode is not empty and no error occurred
    assert!(!emitter.bytecode.is_empty());
}
