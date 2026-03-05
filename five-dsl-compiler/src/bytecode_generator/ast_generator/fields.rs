//! Field and type system methods.

use super::super::types::FieldInfo;
use super::super::OpcodeEmitter;
use super::types::ASTGenerator;
use crate::ast::{AstNode, TypeNode};
use five_protocol::{opcodes::*, Value};
use five_vm_mito::error::VMError;

impl ASTGenerator {
    /// Process field definition and add to symbol table
    pub(super) fn process_field_definition<T: OpcodeEmitter>(
        &mut self,
        emitter: &mut T,
        field_def: &AstNode,
        is_global: bool,
    ) -> Result<(), VMError> {
        match field_def {
            AstNode::FieldDefinition {
                name,
                field_type,
                is_mutable,
                is_optional,
                default_value,
                visibility: _,
            } => {
                let type_string = self.type_node_to_string(field_type);

                let field_info = FieldInfo {
                    offset: self.field_counter,
                    field_type: type_string,
                    is_mutable: *is_mutable,
                    is_optional: *is_optional,
                    is_parameter: false,
                };

                if is_global {
                    self.global_symbol_table.insert(name.clone(), field_info);
                } else {
                    self.local_symbol_table.insert(name.clone(), field_info);
                }

                if *is_optional {
                    if let Some(default) = default_value {
                        self.generate_ast_node(emitter, default)?;
                        emitter.emit_opcode(OPTIONAL_SOME);
                    } else {
                        emitter.emit_opcode(OPTIONAL_NONE);
                    }
                } else if let Some(default) = default_value {
                    self.generate_ast_node(emitter, default)?;
                }

                self.field_counter += 1;
                Ok(())
            }
            _ => Ok(()), // Skip non-field definitions
        }
    }

    /// Convert TypeNode to string representation
    pub(super) fn type_node_to_string(&self, type_node: &TypeNode) -> String {
        match type_node {
            TypeNode::Primitive(name) => name.clone(),
            TypeNode::Generic { base, args } => {
                let arg_strings: Vec<String> = args
                    .iter()
                    .map(|arg| self.type_node_to_string(arg))
                    .collect();
                format!("{}<{}>", base, arg_strings.join(", "))
            }
            TypeNode::Array { element_type, size } => {
                let element_string = self.type_node_to_string(element_type);
                match size {
                    Some(size) => format!("[{}; {}]", element_string, size),
                    None => format!("Vec<{}>", element_string),
                }
            }
            TypeNode::Tuple { elements: types } => {
                let type_strings: Vec<String> =
                    types.iter().map(|t| self.type_node_to_string(t)).collect();
                format!("({})", type_strings.join(", "))
            }
            TypeNode::Struct { fields, .. } => {
                format!("struct{{{}}}", fields.len())
            }
            TypeNode::Union { types } => {
                let type_strings: Vec<String> =
                    types.iter().map(|t| self.type_node_to_string(t)).collect();
                type_strings.join(" | ")
            }
            TypeNode::Sized { base_type, size } => {
                format!("{}<{}>", base_type, size)
            }
            TypeNode::Account => "Account".to_string(),
            TypeNode::Named(name) => name.clone(),
        }
    }

    /// Infer type from AST node
    pub(super) fn infer_type_from_node(&self, node: &AstNode) -> Result<String, VMError> {
        match node {
            AstNode::Literal(value) => match value {
                Value::U64(_) => Ok("u64".to_string()),
                Value::U128(_) => Ok("u128".to_string()),
                Value::Bool(_) => Ok("bool".to_string()),
                Value::String(_) => Ok("string".to_string()),
                Value::Pubkey(_) => Ok("pubkey".to_string()),
                Value::Empty => Ok("empty".to_string()),
                Value::Array(_) => Ok("array".to_string()),
                Value::U8(_) => Ok("u8".to_string()),
                Value::I64(_) => Ok("i64".to_string()),
                Value::Account(_) => Ok("Account".to_string()),
            },
            AstNode::StringLiteral { .. } => Ok("string".to_string()),
            AstNode::Identifier(name) => {
                if let Some(field_info) = self.local_symbol_table.get(name) {
                    Ok(field_info.field_type.clone())
                } else if let Some(field_info) = self.global_symbol_table.get(name) {
                    Ok(field_info.field_type.clone())
                } else {
                    Err(VMError::TypeMismatch)
                }
            }
            AstNode::FunctionCall { name, .. } => {
                // Infer return type for built-in functions
                match name.as_str() {
                    "get_clock" | "get_clock_sysvar" => Ok("Clock".to_string()),
                    "require" => Ok("void".to_string()),
                    "string_length" => Ok("u8".to_string()),
                    "string_concat" => Ok("string".to_string()),
                    "bytes_concat" => Ok("string".to_string()),
                    "verify_ed25519_instruction" | "__verify_ed25519_instruction" => {
                        Ok("bool".to_string())
                    }
                    "Some" => Ok("Option".to_string()), // Generic, but at least not unknown
                    _ => Ok("unknown".to_string()), // Custom functions have unknown return types
                }
            }
            AstNode::BinaryExpression { left, operator, .. } => {
                // For binary expressions, infer from left operand and operator
                let left_type = self.infer_type_from_node(left)?;
                match operator.as_str() {
                    "==" | "!=" | "<" | "<=" | ">" | ">=" | "&&" | "||" => Ok("bool".to_string()),
                    _ => Ok(left_type), // Assume same type as left operand
                }
            }
            AstNode::Cast { target_type, .. } => match target_type.as_ref() {
                AstNode::Identifier(type_name) => Ok(type_name.clone()),
                _ => Ok("unknown".to_string()),
            },
            _ => Ok("unknown".to_string()),
        }
    }

    /// Add a local variable to the symbol table for pattern matching
    pub(super) fn add_local_variable(&mut self, name: String, type_name: String) {
        let field_info = FieldInfo {
            offset: self.field_counter,
            field_type: type_name,
            is_mutable: true, // Pattern variables are mutable within their scope
            is_optional: false,
            is_parameter: false,
        };
        self.local_symbol_table.insert(name, field_info);
        self.field_counter += 1;
    }

    /// Get the local variable index for use with GET_LOCAL/SET_LOCAL opcodes
    pub(super) fn get_local_variable_index(&self, name: &str) -> Result<u8, VMError> {
        if let Some(field_info) = self.local_symbol_table.get(name) {
            Ok(field_info.offset as u8)
        } else {
            Err(VMError::InvalidScript) // Variable not found
        }
    }
}
