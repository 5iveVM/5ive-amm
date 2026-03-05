// Type inference logic

use super::type_helpers::{special_identifiers, type_names};
use super::types::TypeCheckerContext;
use crate::ast::{AstNode, TypeNode};
use five_protocol::Value;
use five_vm_mito::error::VMError;

impl TypeCheckerContext {
    fn infer_account_ctx_field_type(
        &self,
        account_expr: &AstNode,
        field: &str,
    ) -> Result<TypeNode, VMError> {
        let account_name = if let AstNode::Identifier(name) = account_expr {
            name
        } else {
            return Err(VMError::TypeMismatch);
        };

        match field {
            "lamports" => Ok(TypeNode::Primitive("u64".to_string())),
            "owner" | "key" => Ok(TypeNode::Primitive("pubkey".to_string())),
            "data" => Ok(TypeNode::Array {
                element_type: Box::new(TypeNode::Primitive("u8".to_string())),
                size: None,
            }),
            "bump" => {
                if self.init_bump_accounts.contains(account_name) {
                    Ok(TypeNode::Primitive("u8".to_string()))
                } else {
                    Err(VMError::UndefinedField)
                }
            }
            "space" => {
                if self.init_space_accounts.contains(account_name) {
                    Ok(TypeNode::Primitive("u64".to_string()))
                } else {
                    Err(VMError::UndefinedField)
                }
            }
            _ => Err(VMError::UndefinedField),
        }
    }

    pub(crate) fn infer_type(&mut self, node: &AstNode) -> Result<TypeNode, VMError> {
        match node {
            AstNode::Literal(value) => {
                match value {
                    Value::U64(_) => Ok(TypeNode::primitive(type_names::U64)),
                    Value::U128(_) => Ok(TypeNode::primitive(type_names::U128)),
                    Value::Bool(_) => Ok(TypeNode::primitive(type_names::BOOL)),
                    Value::String(_) => Ok(TypeNode::primitive(type_names::STRING)),
                    Value::Pubkey(_) => Ok(TypeNode::primitive(type_names::PUBKEY)),
                    Value::U8(_) => Ok(TypeNode::primitive(type_names::U8)),
                    Value::I64(_) => Ok(TypeNode::primitive(type_names::I64)),
                    Value::Account(_) => Ok(TypeNode::Account),
                    Value::Array(_) => Err(VMError::TypeMismatch), // Arrays are complex, might need more specific handling
                    Value::Empty => Err(VMError::TypeMismatch), // Empty is not a concrete type for inference
                }
            }
            AstNode::StringLiteral { .. } => Ok(TypeNode::primitive(type_names::STRING)),
            AstNode::Identifier(name) => {
                if name == special_identifiers::NONE {
                    // Special handling for None literal
                    Ok(TypeNode::option(TypeNode::Named(
                        type_names::UNKNOWN.to_string(),
                    )))
                } else if let Some(type_node) = self.symbol_table.get(name) {
                    Ok(type_node.0.clone())
                } else {
                    if let Some(replacement) = self.legacy_init_alias_replacement(name) {
                        Err(VMError::undefined_identifier(name, Some(&replacement)))
                    } else {
                        Err(self.undefined_identifier_error(name))
                    }
                }
            }
            AstNode::TupleLiteral { elements } => {
                let mut element_types = Vec::new();
                for element in elements {
                    element_types.push(self.infer_type(element)?);
                }
                Ok(TypeNode::Tuple {
                    elements: element_types,
                })
            }
            AstNode::ArrayLiteral { elements } => {
                if elements.is_empty() {
                    // Handle empty array literal, perhaps return a generic array type or error
                    return Err(VMError::TypeMismatch);
                }
                let element_type = self.infer_type(&elements[0])?;
                // Ensure all elements have the same type
                for element in &elements[1..] {
                    let element_inferred_type = self.infer_type(element)?;
                    if !self.types_are_compatible(&element_type, &element_inferred_type) {
                        return Err(VMError::TypeMismatch);
                    }
                }
                Ok(TypeNode::Array {
                    element_type: Box::new(element_type),
                    size: Some(elements.len() as u64),
                })
            }
            AstNode::StructLiteral { fields } => {
                let mut struct_fields = Vec::new();
                for field in fields {
                    let field_type = self.infer_type(&field.value)?;
                    struct_fields.push(crate::ast::StructField {
                        name: field.field_name.clone(),
                        field_type,
                        is_mutable: true, // Inferred from literal
                        is_optional: false,
                    });
                }
                Ok(TypeNode::Struct {
                    fields: struct_fields,
                })
            }
            AstNode::EnumVariantAccess {
                enum_name,
                variant_name,
            } => match self.symbol_table.get(enum_name) {
                Some((TypeNode::Struct { fields }, _)) => {
                    if fields.iter().any(|f| f.name == *variant_name) {
                        Ok(TypeNode::Named(enum_name.clone()))
                    } else {
                        Err(VMError::UndefinedField)
                    }
                }
                Some(_) => Err(VMError::TypeMismatch),
                None => Err(self.undefined_identifier_error(enum_name)),
            },
            AstNode::ErrorPropagation { expression } => {
                let expr_type = self.infer_type(expression)?;
                if let TypeNode::Generic { base, args } = expr_type {
                    if base == "Result" && !args.is_empty() {
                        Ok(args[0].clone()) // Return the inner type of the Result
                    } else {
                        Err(VMError::TypeMismatch)
                    }
                } else {
                    Err(VMError::TypeMismatch)
                }
            }
            AstNode::UnaryExpression { operator, operand } => {
                let operand_type = self.infer_type(operand)?;
                match operator.as_str() {
                    "-" => {
                        // Numeric negation, ensure operand is numeric
                        if operand_type.is_numeric() {
                            Ok(operand_type)
                        } else {
                            Err(VMError::TypeMismatch)
                        }
                    }
                    "!" | "not" => {
                        // Logical NOT, ensure operand is boolean
                        if operand_type.is_bool() {
                            Ok(operand_type)
                        } else {
                            Err(VMError::TypeMismatch)
                        }
                    }
                    "~" => {
                        // Bitwise NOT, ensure operand is integer
                        if operand_type.is_numeric() {
                            Ok(operand_type)
                        } else {
                            Err(VMError::TypeMismatch)
                        }
                    }
                    _ => Err(VMError::InvalidOperation),
                }
            }
            AstNode::BinaryExpression {
                operator,
                left,
                right,
            } => {
                let left_type = self.infer_type(left)?;
                let right_type = self.infer_type(right)?;

                match operator.as_str() {
                    // Support both normal and checked arithmetic operator syntax:
                    //   +, +?  -> addition
                    //   -, -?  -> subtraction
                    //   *, *?  -> multiplication
                    // Checked variants (+?, -?, *?) are treated as the same numeric semantics
                    // for type inference (they error at runtime on overflow/underflow but the
                    // operand typing rules are identical).
                    "+" | "+?" | "-" | "-?" | "*" | "*?" | "/" | "/?" | "%" | "%?" => {
                        // Arithmetic operations: require compatible numeric types
                        if !left_type.is_numeric() {
                            return Err(VMError::TypeMismatch);
                        }
                        if !right_type.is_numeric() {
                            return Err(VMError::TypeMismatch);
                        }
                        // Prefer preserving the non-literal side's type when the other side is a fitting literal
                        if Self::numeric_literal_fits(&right_type, left) {
                            return Ok(right_type);
                        }
                        if Self::numeric_literal_fits(&left_type, right) {
                            return Ok(left_type);
                        }
                        if self.types_are_compatible(&left_type, &right_type) {
                            Ok(left_type)
                        } else if let Some(promoted) =
                            Self::promote_numeric_types(&left_type, &right_type)
                        {
                            Ok(promoted)
                        } else {
                            // Numeric types but incompatible (e.g., exotic mix)
                            eprintln!(
                                "DEBUG: Arithmetic type mismatch between {:?} and {:?}",
                                left_type, right_type
                            );
                            Err(VMError::TypeMismatch)
                        }
                    }
                    "==" | "!=" => {
                        // Equality: allow numeric comparisons via promotion, or identical types for others
                        let both_numeric = left_type.is_numeric() && right_type.is_numeric();
                        if both_numeric {
                            return Ok(TypeNode::primitive(type_names::BOOL));
                        }

                        // Note: == and != operators are parsed as method calls (.eq() and .ne()),
                        // not BinaryExpressions, so this code path is not reached for comparisons.
                        // The actual fix is in expressions.rs in infer_method_call_type().

                        if self.types_are_compatible(&left_type, &right_type) {
                            Ok(TypeNode::primitive(type_names::BOOL))
                        } else {
                            Err(VMError::TypeMismatch)
                        }
                    }
                    "<" | "<=" | ">" | ">=" => {
                        // Comparison: require compatible numeric types, return bool
                        if !left_type.is_numeric() || !right_type.is_numeric() {
                            return Err(VMError::TypeMismatch);
                        }
                        if self.types_are_compatible(&left_type, &right_type)
                            || Self::promote_numeric_types(&left_type, &right_type).is_some()
                        {
                            Ok(TypeNode::primitive(type_names::BOOL))
                        } else {
                            Err(VMError::TypeMismatch)
                        }
                    }
                    "&&" | "||" => {
                        // Logical: require boolean types, return bool
                        if !left_type.is_bool() || !right_type.is_bool() {
                            return Err(VMError::TypeMismatch);
                        }
                        Ok(TypeNode::primitive(type_names::BOOL))
                    }
                    "&" | "|" | "^" => {
                        // Bitwise ops require integer-like numeric operands.
                        if !left_type.is_numeric() || !right_type.is_numeric() {
                            return Err(VMError::TypeMismatch);
                        }
                        if self.types_are_compatible(&left_type, &right_type) {
                            Ok(left_type)
                        } else if let Some(promoted) =
                            Self::promote_numeric_types(&left_type, &right_type)
                        {
                            Ok(promoted)
                        } else {
                            Err(VMError::TypeMismatch)
                        }
                    }
                    "<<" | ">>" | ">>>" | "<<<" => {
                        // Shift/rotate require numeric operands and preserve lhs type.
                        if !left_type.is_numeric() || !right_type.is_numeric() {
                            return Err(VMError::TypeMismatch);
                        }
                        Ok(left_type)
                    }
                    _ => Err(VMError::InvalidOperation),
                }
            }
            AstNode::FieldAccess { object, field } => {
                if field == "ctx" {
                    let object_type = self.infer_type(object)?;
                    return match object_type {
                        TypeNode::Account => Ok(TypeNode::Named("AccountCtx".to_string())),
                        TypeNode::Named(name) if self.is_named_account_type_name(&name) => {
                            Ok(TypeNode::Named("AccountCtx".to_string()))
                        }
                        _ => Err(VMError::TypeMismatch),
                    };
                }
                if let AstNode::FieldAccess {
                    object: account_expr,
                    field: ctx_field,
                } = object.as_ref()
                {
                    if ctx_field == "ctx" {
                        return self.infer_account_ctx_field_type(account_expr, field);
                    }
                }
                let object_type = self.infer_type(object)?;

                match object_type {
                    TypeNode::Struct { fields } => {
                        if let Some(field_def) = fields.iter().find(|f| f.name == *field) {
                            if field_def.is_optional {
                                Ok(TypeNode::Generic {
                                    base: "Option".to_string(),
                                    args: vec![field_def.field_type.clone()],
                                })
                            } else {
                                Ok(field_def.field_type.clone())
                            }
                        } else {
                            Err(VMError::UndefinedField)
                        }
                    }
                    TypeNode::Named(name) => {
                        if let Some(type_def) = self.resolve_named_type_definition(&name) {
                            if let TypeNode::Struct { fields } = type_def {
                                if let Some(field_def) = fields.iter().find(|f| f.name == *field) {
                                    return if field_def.is_optional {
                                        Ok(TypeNode::Generic {
                                            base: "Option".to_string(),
                                            args: vec![field_def.field_type.clone()],
                                        })
                                    } else {
                                        Ok(field_def.field_type.clone())
                                    };
                                }
                                return Err(VMError::UndefinedField);
                            }
                        }

                        // Look up account fields with namespace-aware matching
                        // Account names may be namespaced (e.g., "amm_types::AMMPool") but referenced by simple name ("AMMPool")
                        eprintln!(
                            "DEBUG: inference.rs infer_type FieldAccess on TypeNode::Named('{}'), looking for field '{}'",
                            name, field
                        );
                        let account_fields = self.resolve_account_definition_fields(&name);

                        if let Some(account_fields) = account_fields {
                            eprintln!(
                                "DEBUG: Resolved account_fields for '{}': {:?}",
                                name,
                                account_fields.iter().map(|f| &f.name).collect::<Vec<_>>()
                            );
                            if let Some(field_def) =
                                account_fields.iter().find(|f| f.name == *field)
                            {
                                if field_def.is_optional {
                                    Ok(TypeNode::Generic {
                                        base: "Option".to_string(),
                                        args: vec![field_def.field_type.clone()],
                                    })
                                } else {
                                    Ok(field_def.field_type.clone())
                                }
                            } else {
                                eprintln!(
                                    "DEBUG: Field '{}' not found in account fields for '{}'",
                                    field, name
                                );
                                if let Some(replacement) =
                                    Self::legacy_account_metadata_replacement_for_access(
                                        object, field,
                                    )
                                {
                                    Err(VMError::undefined_identifier(field, Some(&replacement)))
                                } else {
                                    Err(VMError::UndefinedField)
                                }
                            }
                        } else {
                            eprintln!("DEBUG: No account definition found for '{}'", name);
                            if let Some(replacement) =
                                Self::legacy_account_metadata_replacement_for_access(object, field)
                            {
                                Err(VMError::undefined_identifier(field, Some(&replacement)))
                            } else {
                                Err(VMError::UndefinedField)
                            }
                        }
                    }
                    TypeNode::Account => {
                        if let Some(replacement) =
                            Self::legacy_account_metadata_replacement_for_access(object, field)
                        {
                            Err(VMError::undefined_identifier(field, Some(&replacement)))
                        } else {
                            Err(VMError::UndefinedField)
                        }
                    }
                    _ => Err(VMError::TypeMismatch),
                }
            }
            AstNode::ArrayAccess { array, index } => {
                let array_type = self.infer_type(array)?;
                let index_type = self.infer_type(index)?;

                // Validate that index is numeric
                if !matches!(index_type, TypeNode::Primitive(ref name) if name == "u64" || name == "u32" || name == "u16" || name == "u8" || name == "usize")
                {
                    return Err(VMError::TypeMismatch);
                }

                // Validate that we're accessing an array
                match array_type {
                    TypeNode::Array { element_type, .. } => Ok(*element_type),
                    _ => Err(VMError::TypeMismatch), // Not an array type
                }
            }
            AstNode::Cast {
                value: _,
                target_type,
            } => {
                // Convert target_type AST node to TypeNode
                match target_type.as_ref() {
                    AstNode::Identifier(type_name) => {
                        // Check if this is a primitive type name (e.g., u128, bool, string)
                        // If so, return Primitive type; otherwise it's a user-defined type
                        if super::type_helpers::is_primitive_type_name(type_name) {
                            Ok(TypeNode::Primitive(type_name.clone()))
                        } else {
                            Ok(TypeNode::Named(type_name.clone()))
                        }
                    }
                    _ => Err(VMError::TypeMismatch),
                }
            }
            AstNode::MethodCall {
                object,
                method,
                args,
            } => self.infer_method_call_type(object, method, args),
            AstNode::FunctionCall { name, args } => self.infer_function_call_type(name, args),
            _ => Err(VMError::TypeMismatch),
        }
    }
}
