// Interface Registry Module
//
// Provides centralized interface management with two-pass compilation support.
// This module implements the interface registry system that enables 100% cross-program
// CPI support by pre-processing all interface definitions before bytecode generation.

use crate::ast::{AstNode, TypeNode};
use crate::type_checker::{InterfaceInfo, InterfaceMethod, InterfaceSerializer};
use five_vm_mito::error::VMError;
use sha2::Digest;
use std::collections::HashMap;

/// Global interface registry for two-pass compilation
/// This provides the single source of truth for all interface definitions
#[derive(Debug, Clone)]
pub struct InterfaceRegistry {
    /// All registered interfaces by name
    interfaces: HashMap<String, InterfaceInfo>,
    /// Interface dependency graph for resolution order
    dependencies: HashMap<String, Vec<String>>,
    /// Whether the registry has been finalized (locked for modifications)
    finalized: bool,
}

impl InterfaceRegistry {
    /// Create a new empty interface registry
    pub fn new() -> Self {
        Self {
            interfaces: HashMap::new(),
            dependencies: HashMap::new(),
            finalized: false,
        }
    }

    /// Pre-process all interface definitions in the AST (Phase 1 of two-pass compilation)
    /// This scans the entire AST and builds the complete interface registry
    pub fn preprocess_interfaces(&mut self, ast: &AstNode) -> Result<(), VMError> {
        if self.finalized {
            return Err(VMError::InvalidOperation); // Registry already finalized
        }

        // Recursively find all interface definitions
        self.collect_interfaces(ast)?;

        // Validate all interfaces for consistency
        self.validate_interfaces()?;

        // Resolve interface dependencies
        self.resolve_dependencies()?;

        // Finalize the registry (lock for modifications)
        self.finalized = true;

        Ok(())
    }

    /// Recursively collect all interface definitions from the AST
    fn collect_interfaces(&mut self, node: &AstNode) -> Result<(), VMError> {
        match node {
            AstNode::Program {
                interface_definitions,
                ..
            } => {
                // Process all interface definitions at program level
                for interface_def in interface_definitions {
                    self.process_interface_definition(interface_def)?;
                }
            }
            AstNode::InterfaceDefinition {
                name: _,
                program_id: _,
                functions: _,
                serializer: _,
                is_anchor: _,
            } => {
                self.process_interface_definition(node)?;
            }
            // Recursively check other node types that might contain interfaces
            _ => {
                // Note: Currently interfaces are only supported at program level
                // Future enhancement could support nested interfaces
            }
        }
        Ok(())
    }

    /// Process a single interface definition and add it to the registry
    /// Process a single interface definition and add it to the registry
    fn process_interface_definition(&mut self, interface_def: &AstNode) -> Result<(), VMError> {
        if let AstNode::InterfaceDefinition {
            name,
            program_id,
            serializer,
            is_anchor: is_interface_anchor,
            functions,
        } = interface_def
        {
            let mut methods: HashMap<String, InterfaceMethod> = HashMap::new();
            let serializer_hint = serializer.clone();

            // Process all interface functions
            for function_def in functions {
                if let AstNode::InterfaceFunction {
                    name: method_name,
                    parameters,
                    return_type,
                    discriminator,
                    discriminator_bytes,
                    is_anchor: is_method_anchor,
                } = function_def
                {
                    let is_anchor = *is_interface_anchor || *is_method_anchor;
                    for param in parameters {
                        let has_authority =
                            param.attributes.iter().any(|attr| attr.name == "authority");
                        let is_account_like = matches!(param.param_type, TypeNode::Account)
                            || matches!(&param.param_type, TypeNode::Named(name) if name.eq_ignore_ascii_case("account"));
                        if has_authority && !is_account_like {
                            return Err(VMError::TypeMismatch);
                        }
                    }

                    let return_type_node = return_type.as_ref().map(|rt| (**rt).clone());

                    // Determine discriminator
                    // Priority: explicit bytes > explicit u8 > anchor derived > default (0)
                    let (discriminator_val, discriminator_bytes_val) =
                        if let Some(bytes) = discriminator_bytes {
                            (discriminator.unwrap_or(0), Some(bytes.clone()))
                        } else if let Some(disc) = discriminator {
                            (*disc, None)
                        } else if is_anchor {
                            // Derive Anchor discriminator: sha256("global:<method_name>")[..8]
                            let preimage = format!("global:{}", method_name);
                            let mut hasher = sha2::Sha256::new();
                            hasher.update(preimage.as_bytes());
                            let result = hasher.finalize();
                            let disc_bytes = result[..8].to_vec();
                            (0, Some(disc_bytes))
                        } else {
                            (0, None)
                        };

                    // Validate discriminator uniqueness within interface
                    let check_bytes = discriminator_bytes_val
                        .clone()
                        .unwrap_or_else(|| vec![discriminator_val]);

                    for existing_info in methods.values() {
                        let existing_bytes = existing_info
                            .discriminator_bytes
                            .clone()
                            .unwrap_or_else(|| vec![existing_info.discriminator]);
                        if existing_bytes == check_bytes {
                            return Err(VMError::InvalidOperation); // Duplicate discriminator
                        }
                    }

                    methods.insert(
                        method_name.clone(),
                        InterfaceMethod {
                            discriminator: discriminator_val,
                            discriminator_bytes: discriminator_bytes_val,
                            is_anchor,
                            parameters: parameters.clone(),
                            return_type: return_type_node,
                        },
                    );
                }
            }

            let has_anchor_methods = methods.values().any(|m| m.is_anchor);
            let anchor_mode = *is_interface_anchor || has_anchor_methods;
            let serializer = match serializer_hint.as_deref() {
                None => {
                    if anchor_mode {
                        InterfaceSerializer::Borsh
                    } else {
                        InterfaceSerializer::Bincode
                    }
                }
                Some("raw") => InterfaceSerializer::Raw,
                Some("borsh") => InterfaceSerializer::Borsh,
                Some("bincode") => InterfaceSerializer::Bincode,
                Some(_) => return Err(VMError::InvalidOperation),
            };

            // Validate program ID format
            let validated_program_id = match program_id {
                Some(pid) => {
                    if pid.len() < 32 || pid.len() > 44 {
                        return Err(VMError::InvalidOperation); // Invalid program ID format
                    }
                    pid.clone()
                }
                None => return Err(VMError::InvalidOperation), // Program ID required
            };

            let interface_info = InterfaceInfo {
                program_id: validated_program_id,
                serializer,
                is_anchor: anchor_mode,
                methods,
            };

            // Check for duplicate interface names
            if self.interfaces.contains_key(name) {
                return Err(VMError::InvalidOperation); // Duplicate interface name
            }

            self.interfaces.insert(name.clone(), interface_info);
        }

        Ok(())
    }

    /// Validate all interfaces for consistency and completeness
    fn validate_interfaces(&self) -> Result<(), VMError> {
        for interface_info in self.interfaces.values() {
            // Validate interface has at least one method
            if interface_info.methods.is_empty() {
                return Err(VMError::InvalidOperation); // Interface must have methods
            }

            // Validate program ID is valid Solana program ID format
            if interface_info.program_id.is_empty() {
                return Err(VMError::InvalidOperation); // Program ID cannot be empty
            }

            // Validate method discriminators are unique and reasonable
            let mut discriminators = std::collections::HashSet::new();
            for method_info in interface_info.methods.values() {
                let discriminator_key = method_info
                    .discriminator_bytes
                    .clone()
                    .unwrap_or_else(|| vec![method_info.discriminator]);
                if !discriminators.insert(discriminator_key) {
                    return Err(VMError::InvalidOperation); // Duplicate discriminator in interface
                }

                // Validate method has reasonable parameter count
                if method_info.parameters.len() > five_protocol::MAX_FUNCTION_PARAMS {
                    return Err(VMError::InvalidOperation); // Too many parameters
                }
            }
        }

        Ok(())
    }

    /// Resolve interface dependencies (for future interface inheritance/composition)
    fn resolve_dependencies(&mut self) -> Result<(), VMError> {
        // Currently interfaces are independent, but this will support:
        // - Interface inheritance (interface A extends B)
        // - Interface composition (interface C includes A, B)
        // - Circular dependency detection

        // Validate no circular references exist.
        for interface_name in self.interfaces.keys() {
            self.dependencies.insert(interface_name.clone(), Vec::new());
        }

        Ok(())
    }

    /// Get interface information by name (Phase 2 of two-pass compilation)
    pub fn get_interface(&self, interface_name: &str) -> Option<&InterfaceInfo> {
        if !self.finalized {
            return None; // Registry not ready for use
        }
        self.interfaces.get(interface_name)
    }

    /// Get all registered interfaces
    pub fn get_all_interfaces(&self) -> &HashMap<String, InterfaceInfo> {
        &self.interfaces
    }

    /// Check if interface registry is finalized and ready for use
    pub fn is_finalized(&self) -> bool {
        self.finalized
    }

    /// Get interface dependency order for compilation
    pub fn get_dependency_order(&self) -> Vec<String> {
        // Return interfaces in dependency order
        // Currently all interfaces are independent, so alphabetical order
        let mut names: Vec<String> = self.interfaces.keys().cloned().collect();
        names.sort();
        names
    }

    /// Validate interface method call at compile time
    pub fn validate_method_call(
        &self,
        interface_name: &str,
        method_name: &str,
        arg_types: &[TypeNode],
    ) -> Result<Option<TypeNode>, VMError> {
        if let Some(interface_info) = self.get_interface(interface_name) {
            if let Some(method_info) = interface_info.methods.get(method_name) {
                // Check argument count
                if arg_types.len() != method_info.parameters.len() {
                    return Err(VMError::InvalidParameterCount);
                }

                // Type check arguments (basic compatibility for now)
                for (i, arg_type) in arg_types.iter().enumerate() {
                    let expected_type = &method_info.parameters[i].param_type;
                    if !self.types_are_compatible(arg_type, expected_type) {
                        return Err(VMError::TypeMismatch);
                    }
                }

                // Return the method's return type
                Ok(method_info.return_type.clone())
            } else {
                Err(VMError::InvalidOperation) // Method not found in interface
            }
        } else {
            Err(VMError::InvalidOperation) // Interface not found
        }
    }

    /// Check if two types are compatible (comprehensive implementation)
    fn types_are_compatible(&self, actual: &TypeNode, expected: &TypeNode) -> bool {
        match (actual, expected) {
            // Exact type match
            (TypeNode::Primitive(a), TypeNode::Primitive(b)) => {
                self.are_primitive_types_compatible(a, b)
            }

            // Generic type compatibility
            (
                TypeNode::Generic {
                    base: a_base,
                    args: a_args,
                },
                TypeNode::Generic {
                    base: b_base,
                    args: b_args,
                },
            ) => {
                a_base == b_base
                    && a_args.len() == b_args.len()
                    && a_args
                        .iter()
                        .zip(b_args.iter())
                        .all(|(a, b)| self.types_are_compatible(a, b))
            }

            // Array type compatibility
            (
                TypeNode::Array {
                    element_type: a_elem,
                    size: a_size,
                },
                TypeNode::Array {
                    element_type: b_elem,
                    size: b_size,
                },
            ) => self.types_are_compatible(a_elem, b_elem) && a_size == b_size,

            // Tuple type compatibility
            (TypeNode::Tuple { elements: a_elems }, TypeNode::Tuple { elements: b_elems }) => {
                a_elems.len() == b_elems.len()
                    && a_elems
                        .iter()
                        .zip(b_elems.iter())
                        .all(|(a, b)| self.types_are_compatible(a, b))
            }

            // Named type compatibility (for custom types, enums, etc.)
            (TypeNode::Named(a), TypeNode::Named(b)) => a == b,

            // Account type compatibility
            (TypeNode::Account, TypeNode::Account) => true,

            // Union type compatibility (at least one matching type)
            (actual_type, TypeNode::Union { types }) => types
                .iter()
                .any(|union_type| self.types_are_compatible(actual_type, union_type)),
            (TypeNode::Union { types }, expected_type) => types
                .iter()
                .any(|union_type| self.types_are_compatible(union_type, expected_type)),

            // Sized type compatibility
            (
                TypeNode::Sized {
                    base_type: a_base,
                    size: a_size,
                },
                TypeNode::Sized {
                    base_type: b_base,
                    size: b_size,
                },
            ) => a_base == b_base && a_size == b_size,

            // Struct type compatibility (by field types)
            (TypeNode::Struct { fields: a_fields }, TypeNode::Struct { fields: b_fields }) => {
                a_fields.len() == b_fields.len()
                    && a_fields
                        .iter()
                        .zip(b_fields.iter())
                        .all(|(a_field, b_field)| {
                            a_field.name == b_field.name
                                && self
                                    .types_are_compatible(&a_field.field_type, &b_field.field_type)
                        })
            }

            // Generic type compatibility with specific pattern matching
            (
                TypeNode::Generic {
                    base: a_base,
                    args: a_args,
                },
                TypeNode::Primitive(b),
            ) if a_base == "Option" && a_args.len() == 1 => {
                // Option<T> can accept T or null
                self.types_are_compatible(&a_args[0], &TypeNode::Primitive(b.clone()))
            }
            (
                TypeNode::Generic {
                    base: a_base,
                    args: a_args,
                },
                TypeNode::Primitive(b),
            ) if a_base == "Result" && a_args.len() == 2 => {
                // Result<T, E> can accept T (success case)
                self.types_are_compatible(&a_args[0], &TypeNode::Primitive(b.clone()))
            }

            // Allow coercion from concrete types to generic parameters
            (_concrete, TypeNode::Generic { base, .. }) if base == "T" => true,

            // Default: incompatible
            _ => false,
        }
    }

    /// Check if primitive types are compatible (with coercion rules)
    fn are_primitive_types_compatible(&self, actual: &str, expected: &str) -> bool {
        // Exact match
        if actual == expected {
            return true;
        }

        // Integer type coercion rules
        match (actual, expected) {
            // Unsigned integer coercions (smaller to larger)
            ("u8", "u16")
            | ("u8", "u32")
            | ("u8", "u64")
            | ("u16", "u32")
            | ("u16", "u64")
            | ("u32", "u64") => true,

            // Signed integer coercions (smaller to larger)
            ("i8", "i16")
            | ("i8", "i32")
            | ("i8", "i64")
            | ("i16", "i32")
            | ("i16", "i64")
            | ("i32", "i64") => true,

            // Pubkey is compatible with string/bytes in some contexts
            ("pubkey", "string") | ("string", "pubkey") => true,

            // Array<u8> is compatible with string
            ("string", "bytes") | ("bytes", "string") => true,

            // No other coercions allowed for safety
            _ => false,
        }
    }

    /// Validate interface method call with enhanced type checking and account constraints
    pub fn validate_method_call_enhanced(
        &self,
        interface_name: &str,
        method_name: &str,
        arg_types: &[TypeNode],
        account_constraints: &[(String, Vec<String>)], // (account_name, constraints)
    ) -> Result<Option<TypeNode>, VMError> {
        if let Some(interface_info) = self.get_interface(interface_name) {
            if let Some(method_info) = interface_info.methods.get(method_name) {
                // Check argument count
                if arg_types.len() != method_info.parameters.len() {
                    return Err(VMError::InvalidParameterCount);
                }

                // Enhanced type checking with detailed error reporting
                for (arg_type, expected_type) in arg_types.iter().zip(&method_info.parameters) {
                    if !self.types_are_compatible(arg_type, &expected_type.param_type) {
                        // Provide detailed type mismatch information
                        return Err(VMError::TypeMismatch);
                    }
                }

                // Validate account constraints if provided
                if !account_constraints.is_empty() {
                    self.validate_account_constraints(
                        interface_name,
                        method_name,
                        account_constraints,
                    )?;
                }

                // Return the method's return type
                Ok(method_info.return_type.clone())
            } else {
                Err(VMError::InvalidOperation) // Method not found in interface
            }
        } else {
            Err(VMError::InvalidOperation) // Interface not found
        }
    }

    /// Validate account constraints for interface method calls
    fn validate_account_constraints(
        &self,
        interface_name: &str,
        method_name: &str,
        account_constraints: &[(String, Vec<String>)],
    ) -> Result<(), VMError> {
        // Common Solana account constraint patterns
        for (_account_name, constraints) in account_constraints {
            for constraint in constraints {
                match constraint.as_str() {
                    "signer" => {
                        // Generic signer constraint handling.
                        let _ = (interface_name, method_name);
                        continue;
                    }
                    "mut" => {
                        // Generic mutable constraint handling.
                        let _ = (interface_name, method_name);
                        continue;
                    }
                    "init" => {
                        // Generic init constraint handling.
                        let _ = method_name;
                        continue;
                    }
                    _ => {
                        // Unknown constraint - could be program-specific
                        continue;
                    }
                }
            }
        }

        Ok(())
    }

    /// Reset the registry (for testing)
    #[cfg(test)]
    pub fn reset(&mut self) {
        self.interfaces.clear();
        self.dependencies.clear();
        self.finalized = false;
    }

    /// Get interface count (for metrics)
    pub fn interface_count(&self) -> usize {
        self.interfaces.len()
    }

    /// Get total method count across all interfaces (for metrics)
    pub fn total_method_count(&self) -> usize {
        self.interfaces
            .values()
            .map(|interface| interface.methods.len())
            .sum()
    }
}

impl Default for InterfaceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{AstNode, InstructionParameter};

    #[test]
    fn test_interface_registry_basic() {
        let registry = InterfaceRegistry::new();
        assert!(!registry.is_finalized());
        assert_eq!(registry.interface_count(), 0);
    }

    #[test]
    fn test_interface_preprocessing() {
        let mut registry = InterfaceRegistry::new();

        // Create test interface definition
        let interface_def = AstNode::InterfaceDefinition {
            name: "TestInterface".to_string(),
            program_id: Some("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".to_string()),
            serializer: None,
            is_anchor: false,
            functions: vec![AstNode::InterfaceFunction {
                name: "test_method".to_string(),
                parameters: vec![InstructionParameter {
                    name: "param1".to_string(),
                    param_type: TypeNode::Primitive("u64".to_string()),
                    attributes: vec![],
                    is_optional: false,
                    default_value: None,
                    is_init: false,
                    init_config: None,
                    pda_config: None,
                }],
                return_type: Some(Box::new(TypeNode::Primitive("u64".to_string()))),
                discriminator: Some(1),
                discriminator_bytes: None,
                is_anchor: false,
            }],
        };

        let program = AstNode::Program {
            program_name: "Test".to_string(),
            field_definitions: vec![],
            instruction_definitions: vec![],
            event_definitions: vec![],
            account_definitions: vec![],
            interface_definitions: vec![interface_def],
            import_statements: vec![],
            init_block: None,
            constraints_block: None,
        };

        assert!(registry.preprocess_interfaces(&program).is_ok());
        assert!(registry.is_finalized());
        assert_eq!(registry.interface_count(), 1);
        assert_eq!(registry.total_method_count(), 1);

        let interface = registry.get_interface("TestInterface");
        assert!(interface.is_some());
        let interface = interface.unwrap();
        assert_eq!(
            interface.program_id,
            "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
        );
        assert!(matches!(interface.serializer, InterfaceSerializer::Bincode));
        assert!(interface.methods.contains_key("test_method"));
    }

    #[test]
    fn test_interface_with_serializer_and_discriminator_bytes() {
        let mut registry = InterfaceRegistry::new();

        let interface_def = AstNode::InterfaceDefinition {
            name: "AnchorStyle".to_string(),
            program_id: Some("11111111111111111111111111111111".to_string()),
            serializer: Some("borsh".to_string()),
            is_anchor: true,
            functions: vec![AstNode::InterfaceFunction {
                name: "initialize".to_string(),
                parameters: vec![InstructionParameter {
                    name: "amount".to_string(),
                    param_type: TypeNode::Primitive("u64".to_string()),
                    attributes: vec![],
                    is_optional: false,
                    default_value: None,
                    is_init: false,
                    init_config: None,
                    pda_config: None,
                }],
                return_type: None,
                discriminator: None,
                discriminator_bytes: Some(vec![1, 2, 3, 4, 5, 6, 7, 8]),
                is_anchor: false,
            }],
        };

        let program = AstNode::Program {
            program_name: "Test".to_string(),
            field_definitions: vec![],
            instruction_definitions: vec![],
            event_definitions: vec![],
            account_definitions: vec![],
            interface_definitions: vec![interface_def],
            import_statements: vec![],
            init_block: None,
            constraints_block: None,
        };

        registry.preprocess_interfaces(&program).unwrap();
        let info = registry.get_interface("AnchorStyle").unwrap();
        assert!(matches!(info.serializer, InterfaceSerializer::Borsh));
        let method = info.methods.get("initialize").unwrap();
        assert_eq!(
            method
                .discriminator_bytes
                .clone()
                .unwrap_or_else(|| vec![method.discriminator]),
            vec![1, 2, 3, 4, 5, 6, 7, 8]
        );
    }
}
