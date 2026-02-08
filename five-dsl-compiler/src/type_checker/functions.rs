// Function and interface type checking

use super::type_helpers::type_names;
use super::types::{InterfaceInfo, InterfaceMethod, InterfaceSerializer, TypeCheckerContext};
use crate::ast::{AstNode, TypeNode};
use five_vm_mito::error::VMError;
use std::collections::{HashMap, HashSet};

impl TypeCheckerContext {
    /// Process interface definitions and populate the registry
    pub fn process_interface_definitions(
        &mut self,
        interface_definitions: &[AstNode],
    ) -> Result<(), VMError> {
        for interface_def in interface_definitions {
            if let AstNode::InterfaceDefinition {
                name,
                program_id,
                serializer,
                functions,
            } = interface_def
            {
                let mut methods = HashMap::new();

                for function_def in functions {
                    if let AstNode::InterfaceFunction {
                        name: method_name,
                        parameters,
                        return_type,
                        discriminator,
                        discriminator_bytes,
                    } = function_def
                    {
                        // Convert InstructionParameter to TypeNode for storage
                        let param_types: Vec<TypeNode> = parameters
                            .iter()
                            .map(|param| param.param_type.clone())
                            .collect();

                        let return_type_node = return_type.as_ref().map(|rt| (**rt).clone());

                        methods.insert(
                            method_name.clone(),
                            InterfaceMethod {
                                discriminator: discriminator.unwrap_or(0), // Default to 0 if no discriminator
                                discriminator_bytes: discriminator_bytes.clone(),
                                parameters: param_types,
                                return_type: return_type_node,
                            },
                        );
                    }
                }

                let interface_info = InterfaceInfo {
                    program_id: program_id.clone().unwrap_or_default(), // Default to empty if no program ID
                    serializer: match serializer
                        .as_ref()
                        .map(|s| s.as_str())
                    {
                        None => InterfaceSerializer::Raw,
                        Some("raw") => InterfaceSerializer::Raw,
                        Some("borsh") => InterfaceSerializer::Borsh,
                        Some("bincode") => InterfaceSerializer::Bincode,
                        Some(_) => return Err(VMError::InvalidOperation),
                    },
                    methods,
                };

                self.interface_registry.insert(name.clone(), interface_info);
            }
        }
        Ok(())
    }

    /// Get interface information by name
    pub fn get_interface_info(&self, interface_name: &str) -> Option<&InterfaceInfo> {
        self.interface_registry.get(interface_name)
    }

    /// Validate interface method call
    pub fn validate_interface_method_call(
        &mut self,
        interface_info: &InterfaceInfo,
        method_name: &str,
        args: &[AstNode],
    ) -> Result<TypeNode, VMError> {
        if let Some(method_info) = interface_info.methods.get(method_name) {
            // Check argument count
            if args.len() != method_info.parameters.len() {
                return Err(VMError::InvalidParameterCount);
            }

            // Type check arguments
            for (i, arg) in args.iter().enumerate() {
                let arg_type = self.infer_type(arg)?;
                let expected_type = &method_info.parameters[i];
                if !self.types_are_compatible(&arg_type, expected_type) {
                    return Err(VMError::TypeMismatch);
                }
            }

            // Return the method's return type, or void if none specified
            Ok(method_info
                .return_type
                .clone()
                .unwrap_or(TypeNode::Primitive("void".to_string())))
        } else {
            Err(VMError::InvalidOperation) // Method not found in interface
        }
    }

    pub(crate) fn check_instruction_definition(
        &mut self,
        name: &str,
        parameters: &[crate::ast::InstructionParameter],
        return_type: &Option<Box<TypeNode>>,
        body: &AstNode,
    ) -> Result<(), VMError> {
        // Create a new scope for the function (preserve global fields)
        let original_symbol_table = self.symbol_table.clone();
        let previous_writable = self.current_writable_accounts.clone();
        let previous_function = self.current_function.clone();
        // Keep global fields, but parameters can shadow them
        self.current_function = Some(name.to_string());

        // Check parameter types are valid and add to symbol table
        // Also capture which account parameters are marked @mut
        let mut writable_accounts: HashSet<String> = HashSet::new();
        for param in parameters {
            if !self.is_valid_type_node(&param.param_type) {
                eprintln!("Invalid param type: {} ({:?})", param.name, param.param_type);
                return Err(VMError::InvalidScript);
            }

            // Validate @init constraints
            // Validate @init constraints
            if param.is_init {
                // @init can only be applied to Account types (built-in or user-defined)
                let is_valid_account = match &param.param_type {
                    crate::ast::TypeNode::Account => true,
                    crate::ast::TypeNode::Named(name) => {
                        // Check for exact match or namespaced match (e.g., "AMMPool" or "amm_types::AMMPool")
                        let namespace_suffix = format!("::{}", name);
                        if name == "Account" || name == "account" 
                            || self.account_definitions.contains_key(name)
                            || self.account_definitions.keys().any(|k| k.ends_with(&namespace_suffix)) 
                        {
                            true
                        } else {
                            // Check module scope for imported accounts
                            if let Some(scope) = &self.module_scope {
                                if let Some(symbol) = scope.resolve_symbol(name, scope.current_module()) {
                                     matches!(symbol.type_info, crate::ast::TypeNode::Account)
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        }
                    }
                    _ => false,
                };

                if !is_valid_account {
                    return Err(VMError::ConstraintViolation);
                }

                // NEW: Validate payer if specified
                if let Some(ref init_config) = param.init_config {
                    if let Some(ref payer_name) = init_config.payer {
                        // Find payer in parameters
                        let payer_param = parameters.iter().find(|p| p.name == *payer_name);

                        match payer_param {
                            None => {
                                eprintln!("@init payer '{}' not found in function parameters", payer_name);
                                return Err(VMError::InvalidScript);
                            }
                            Some(payer) => {
                                // Validate payer is account type
                                if !matches!(payer.param_type, crate::ast::TypeNode::Account | crate::ast::TypeNode::Named(_)) {
                                    eprintln!("@init payer '{}' must be an account type", payer_name);
                                    return Err(VMError::TypeMismatch);
                                }

                                // Validate payer has @signer
                                if !payer.attributes.iter().any(|a| a.name == "signer") {
                                    eprintln!("@init payer '{}' must have @signer constraint", payer_name);
                                    return Err(VMError::ConstraintViolation);
                                }
                            }
                        }
                    }
                }
            }

            // For account parameters, store them as Account type so field access works
            let param_type = if param.param_type.is_account_type() {
                TypeNode::Account
            } else {
                param.param_type.clone()
            };
            
            // Implicit mutability: @init implies mutable, or explicit @mut
            let is_mutable = param.is_init || param.attributes.iter().any(|a| a.name == "mut");

            self.symbol_table
                .insert(param.name.clone(), (param_type.clone(), is_mutable));

            // Record definition for go-to-definition feature
            self.record_definition(
                param.name.clone(),
                param_type,
                is_mutable,
                None, // TODO: Add position tracking to AST nodes
            );

            // Record @mut on account parameters
            if is_mutable {
                writable_accounts.insert(param.name.clone());
            }

            // Determine if this is an account type (including custom named accounts)
            let is_account_param = match &param.param_type {
                crate::ast::TypeNode::Account => true,
                crate::ast::TypeNode::Named(name) => {
                    name == "Account" || name == "account" ||
                    self.account_definitions.contains_key(name)
                }
                _ => false,
            };

            // Validate constraints attributes
            for attr in &param.attributes {
                match attr.name.as_str() {
                    "signer" => {
                         if !is_account_param {
                             eprintln!("@signer only allowed on accounts: {}", param.name);
                             return Err(VMError::TypeMismatch); // @signer only allowed on accounts
                          }
                    }
                    "has" => {
                        if !is_account_param {
                             return Err(VMError::TypeMismatch); 
                        }
                        if attr.args.is_empty() {
                             return Err(VMError::InvalidInstruction); 
                        }
                        // Validate all targets exist in parameters
                        for arg in &attr.args {
                            if let crate::ast::AstNode::Identifier(target_name) = arg {
                                let target_exists = parameters.iter().any(|p| p.name == *target_name);
                                if !target_exists {
                                    eprintln!("@has target not found: {}", target_name);
                                    return Err(VMError::InvalidScript); // Target parameter not found
                                }
                            } else {
                                return Err(VMError::InvalidInstruction); // Arguments must be identifiers
                            }
                        }
                    }
                    "owner" => {
                        if !is_account_param {
                             return Err(VMError::TypeMismatch); 
                        }
                        if attr.args.len() > 1 {
                             return Err(VMError::InvalidInstruction); 
                        }
                        // If arg provided, validate it exists
                        if let Some(arg) = attr.args.first() {
                             if let crate::ast::AstNode::Identifier(target_name) = arg {
                                 let target_exists = parameters.iter().any(|p| p.name == *target_name);
                                 if !target_exists {
                                     eprintln!("@owner target not found: {}", target_name);
                                    // Check imports? Return error.
                                     return Err(VMError::InvalidScript); 
                                 }
                             }
                        }
                    }
                    _ => {}
                }
            }
        }

        // Set current function's writable accounts context
        self.current_writable_accounts = Some(writable_accounts);

        // Check return type is valid if present
        if let Some(ret_type) = return_type {
            if !self.is_valid_type_node(ret_type) {
                eprintln!("Invalid return type: {:?}", ret_type);
                return Err(VMError::InvalidScript);
            }
        }

        // Check body
        self.check_types(body)?;

        // Restore original symbol table
        self.symbol_table = original_symbol_table;
        // Restore writable accounts context
        self.current_writable_accounts = previous_writable;
        // Restore function name
        self.current_function = previous_function;

        Ok(())
    }

    pub(crate) fn check_field_definition(
        &mut self,
        name: &str,
        field_type: &TypeNode,
        is_mutable: bool,
        default_value: &Option<Box<AstNode>>,
    ) -> Result<(), VMError> {
        // Type check default value if present
        if let Some(default) = default_value {
            let default_type = self.infer_type(default)?;
            if !self.types_are_compatible(field_type, &default_type) {
                return Err(VMError::TypeMismatch);
            }
        }

        // Register field in symbol table for later reference
        self.symbol_table
            .insert(name.to_string(), (field_type.clone(), is_mutable));

        // Record definition for go-to-definition feature (location info not available in AST yet)
        self.record_definition(
            name.to_string(),
            field_type.clone(),
            is_mutable,
            None, // TODO: Add position tracking to AST nodes
        );

        Ok(())
    }

    pub(crate) fn check_event_definition(
        &mut self,
        fields: &[crate::ast::StructField],
    ) -> Result<(), VMError> {
        // Check all event fields have valid types
        for field in fields {
            if !self.is_valid_type_node(&field.field_type) {
                return Err(VMError::InvalidScript);
            }
        }
        Ok(())
    }

    pub(crate) fn check_account_definition(
        &mut self,
        name: &str,
        fields: &[crate::ast::StructField],
        visibility: crate::ast::Visibility,
    ) -> Result<(), VMError> {
        self.account_definitions
            .insert(name.to_string(), fields.to_vec());
        
        // Add to module scope for cross-module resolution
        self.add_to_module_scope(
            name.to_string(), 
            TypeNode::Account, 
            false, 
            visibility
        );

        // Check all account fields have valid types
        for field in fields {
            self.validate_type(&field.field_type)?;

            // Strings in accounts must be sized
            if let TypeNode::Primitive(type_name) = &field.field_type {
                if type_name == type_names::STRING {
                    eprintln!("Type error: field '{}' in account '{}' is unsized string. Accounts require explicit sized strings (e.g. string<32>).", field.name, name);
                    return Err(VMError::TypeMismatch);
                }
            }
        }
        Ok(())
    }

    pub(crate) fn check_error_type_definition(
        &mut self,
        name: &str,
        variants: &[crate::ast::ErrorVariant],
    ) -> Result<(), VMError> {
        for variant in variants {
            for field in &variant.fields {
                self.validate_type(&field.field_type)?;
            }
        }

        // Store enum variant information in the symbol table so that
        // variant accesses can be validated later.
        let variant_fields: Vec<crate::ast::StructField> = variants
            .iter()
            .map(|variant| crate::ast::StructField {
                name: variant.name.clone(),
                field_type: TypeNode::Struct {
                    fields: variant.fields.clone(),
                },
                is_mutable: false,
                is_optional: false,
            })
            .collect();

        self.symbol_table.insert(
            name.to_string(),
            (
                TypeNode::Struct {
                    fields: variant_fields,
                },
                false,
            ),
        );
        Ok(())
    }

    pub(crate) fn check_test_function(&mut self, body: &AstNode) -> Result<(), VMError> {
        self.check_types(body)
    }
}
