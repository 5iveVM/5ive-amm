// Type validation utilities

use super::type_helpers::{
    is_builtin_account_property, is_numeric_type_name, numeric_type_meta, type_names,
};
use super::types::TypeCheckerContext;
use crate::ast::{AstNode, TypeNode};
use five_protocol::Value;
use five_vm_mito::error::VMError;

impl TypeCheckerContext {
    /// Format a type node for diagnostics
    pub(crate) fn fmt_type(t: &TypeNode) -> String {
        match t {
            TypeNode::Primitive(n) => n.clone(),
            TypeNode::Named(n) => n.clone(),
            TypeNode::Account => type_names::ACCOUNT_LOWER.to_string(),
            TypeNode::Array { element_type, size } => match size {
                Some(s) => format!("[{}; {}]", Self::fmt_type(element_type), s),
                None => format!("[{}]", Self::fmt_type(element_type)),
            },
            TypeNode::Tuple { elements } => {
                let inner: Vec<String> = elements.iter().map(Self::fmt_type).collect();
                format!("({})", inner.join(", "))
            }
            TypeNode::Generic { base, args } => {
                let inner: Vec<String> = args.iter().map(Self::fmt_type).collect();
                format!("{}<{}>", base, inner.join(", "))
            }
            _ => "<unknown>".to_string(),
        }
    }

    pub(crate) fn is_numeric_primitive_name(name: &str) -> bool {
        is_numeric_type_name(name)
    }

    pub(crate) fn promote_numeric_types(t1: &TypeNode, t2: &TypeNode) -> Option<TypeNode> {
        use TypeNode::Primitive;
        match (t1, t2) {
            (Primitive(n1), Primitive(n2)) => {
                let (s1, b1) = numeric_type_meta(n1.as_str())?;
                let (s2, b2) = numeric_type_meta(n2.as_str())?;
                let signed = s1 || s2;
                let bits = core::cmp::max(b1, b2);
                let name = match (signed, bits) {
                    (false, 8) => type_names::U8,
                    (false, 16) => type_names::U16,
                    (false, 32) => type_names::U32,
                    (false, 64) => type_names::U64,
                    (false, 128) => type_names::U128,
                    (true, 8) => type_names::I8,
                    (true, 16) => type_names::I16,
                    (true, 32) => type_names::I32,
                    (true, 64) => type_names::I64,
                    _ => return None,
                };
                Some(Primitive(name.to_string()))
            }
            _ => None,
        }
    }

    /// Best-effort check: does a numeric literal fit into the target integer type?
    pub(crate) fn numeric_literal_fits(target: &TypeNode, value_node: &AstNode) -> bool {
        // Only handle positive integer literals for now (Value::U64)
        let lit = match value_node {
            AstNode::Literal(Value::U64(n)) => Some(*n as i128),
            AstNode::Literal(Value::I64(n)) => Some(*n as i128),
            // Negatives may come as unary negation; skip complex handling here
            _ => None,
        };
        let lit = match lit {
            Some(v) => v,
            None => return false,
        };

        match target {
            TypeNode::Primitive(name) if is_numeric_type_name(name) => match name.as_str() {
                type_names::U8 => (0..=u8::MAX as i128).contains(&lit),
                type_names::U16 => (0..=u16::MAX as i128).contains(&lit),
                type_names::U32 => (0..=u32::MAX as i128).contains(&lit),
                type_names::U64 => lit >= 0,
                type_names::U128 => lit >= 0,
                type_names::I8 => (i8::MIN as i128..=i8::MAX as i128).contains(&lit),
                type_names::I16 => (i16::MIN as i128..=i16::MAX as i128).contains(&lit),
                type_names::I32 => (i32::MIN as i128..=i32::MAX as i128).contains(&lit),
                type_names::I64 => (i64::MIN as i128..=i64::MAX as i128).contains(&lit),
                _ => false,
            },
            _ => false,
        }
    }

    /// Allow assigning numeric zero literal to pubkey fields as the "zero pubkey" sentinel.
    pub(crate) fn is_zero_numeric_literal(value_node: &AstNode) -> bool {
        match value_node {
            AstNode::Literal(Value::U64(n)) => *n == 0,
            AstNode::Literal(Value::I64(n)) => *n == 0,
            _ => false,
        }
    }

    pub(crate) fn is_valid_type_node(&self, type_node: &TypeNode) -> bool {
        // println!("DEBUG: validating type: {:?}", type_node);
        match type_node {
            TypeNode::Primitive(type_name) => matches!(
                type_name.as_str(),
                type_names::U64
                    | type_names::U32
                    | type_names::U16
                    | type_names::U8
                    | type_names::U128
                    | type_names::I64
                    | type_names::I32
                    | type_names::I16
                    | type_names::I8
                    | type_names::BOOL
                    | type_names::STRING
                    | type_names::PUBKEY
                    | type_names::LAMPORTS
            ),
            TypeNode::Generic { base, args } => {
                // Allow Result, Option generics
                if base == type_names::OPTION || base == type_names::RESULT {
                    args.iter().all(|arg| self.is_valid_type_node(arg))
                } else {
                    false
                }
            }
            TypeNode::Array {
                element_type,
                size: _,
            } => self.is_valid_type_node(element_type),
            TypeNode::Account => true, // Built-in account type is always valid
            TypeNode::Named(_) => true, // Named types validated against account definitions during compilation
            TypeNode::Tuple { elements } => elements
                .iter()
                .all(|element| self.is_valid_type_node(element)),
            _ => false,
        }
    }

    /// Validate that a type is supported and well-formed
    pub(crate) fn validate_type(&self, type_node: &TypeNode) -> Result<(), VMError> {
        if self.is_valid_type_node(type_node) {
            Ok(())
        } else {
            Err(VMError::TypeMismatch)
        }
    }

    /// Validate built-in account properties
    pub(crate) fn validate_builtin_account_property(&self, property: &str) -> Result<(), VMError> {
        if is_builtin_account_property(property) {
            Ok(())
        } else {
            // Not a built-in property, which is fine - could be user-defined
            Ok(())
        }
    }

    pub(crate) fn types_are_compatible(&self, type1: &TypeNode, type2: &TypeNode) -> bool {
        // Allow Unknown placeholders to match any type
        if type1.is_unknown_placeholder() || type2.is_unknown_placeholder() {
            return true;
        }

        match (type1, type2) {
            (TypeNode::Primitive(name1), TypeNode::Primitive(name2)) => name1 == name2,
            (TypeNode::Named(name1), TypeNode::Named(name2)) => name1 == name2,
            (TypeNode::Account, TypeNode::Account) => true,
            (TypeNode::Tuple { elements: e1 }, TypeNode::Tuple { elements: e2 }) => {
                e1.len() == e2.len()
                    && e1
                        .iter()
                        .zip(e2.iter())
                        .all(|(t1, t2)| self.types_are_compatible(t1, t2))
            }
            (
                TypeNode::Array {
                    element_type: et1,
                    size: s1,
                },
                TypeNode::Array {
                    element_type: et2,
                    size: s2,
                },
            ) => s1 == s2 && self.types_are_compatible(et1, et2),
            (
                TypeNode::Generic { base: b1, args: a1 },
                TypeNode::Generic { base: b2, args: a2 },
            ) => {
                b1 == b2
                    && a1.len() == a2.len()
                    && a1.iter().zip(a2.iter()).all(|(t1, t2)| {
                        // Special handling for Result types with unknown placeholders
                        if b1 == type_names::RESULT
                            && (t1.is_unknown_placeholder() || t2.is_unknown_placeholder())
                        {
                            true
                        } else {
                            self.types_are_compatible(t1, t2)
                        }
                    })
            }
            _ => false,
        }
    }
}
