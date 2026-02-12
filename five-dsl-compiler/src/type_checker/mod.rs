// Type Checker Module
//
// Handles semantic analysis and type checking of the AST.

mod definition_builder;
mod expressions;
mod functions;
mod inference;
mod statements;
mod statement_builder;
mod type_helpers;
mod type_safe_checker;
mod type_safe_example;
mod types;
mod validation;
pub mod module_scope;
#[cfg(test)]
mod expressions_tests;
#[cfg(test)]
mod statements_tests;

use crate::ast::{AstNode, TypeNode};
use five_vm_mito::error::VMError;
use std::collections::HashMap;

// Re-export public types
pub use types::{InterfaceInfo, InterfaceMethod, InterfaceSerializer};
pub use module_scope::{ModuleScope, ModuleSymbol, ModuleSymbolTable};

// Type alias for backward compatibility
pub type DslTypeChecker = types::TypeCheckerContext;

impl types::TypeCheckerContext {
    pub fn check_types(&mut self, ast: &AstNode) -> Result<(), VMError> {
        match ast {
            AstNode::Program {
                field_definitions,
                instruction_definitions,
                event_definitions,
                account_definitions,
                interface_definitions,
                import_statements,
                init_block,
                constraints_block,
                ..
            } => {
                // Capture imported external interface namespaces for method-call validation.
                self.imported_external_interfaces.clear();
                for import_stmt in import_statements {
                    if let AstNode::ImportStatement {
                        module_specifier: crate::ast::ModuleSpecifier::External(_),
                        imported_items: Some(items),
                    } = import_stmt
                    {
                        for item in items {
                            if let crate::ast::ImportItem::Interface(name) = item {
                                self.imported_external_interfaces.insert(name.clone());
                            }
                        }
                    }
                }

                // Process global field definitions (now supported)
                for field_def in field_definitions {
                    self.check_types(field_def)?;
                }

                // Process account definitions FIRST (they are referenced by instructions)
                for account_def in account_definitions {
                    if let Err(e) = self.check_types(account_def) {
                        eprintln!("Failed checking account definition: {:?}", e);
                        return Err(e);
                    }
                }

                // Process interface definitions (now account definitions are available)
                self.process_interface_definitions(interface_definitions)?;

                // Pre-register function return types for user-defined functions
                for instruction_def in instruction_definitions {
                    if let AstNode::InstructionDefinition {
                        name, return_type, ..
                    } = instruction_def
                    {
                        self.function_return_types
                            .insert(name.clone(), return_type.as_ref().map(|t| (**t).clone()));
                    }
                }

                // Check instruction definitions (now account definitions and interfaces are available)
                // This will handle their individual scopes correctly and populate the symbol table
                let mut all_instruction_params: HashMap<String, (crate::ast::TypeNode, bool)> =
                    HashMap::new();
                for instruction_def in instruction_definitions {
                    // Temporarily store the current symbol table
                    let original_symbol_table = self.symbol_table.clone();

                    // Check parameter types are valid and add to symbol table for this instruction
                    if let AstNode::InstructionDefinition { parameters, .. } = instruction_def {
                        for param in parameters {
                            if !self.is_valid_type_node(&param.param_type) {
                                eprintln!("Invalid param type in mod checked: {} ({:?})", param.name, param.param_type);
                                return Err(VMError::InvalidScript);
                            }

                            // Validate @init constraints
                            if param.is_init {
                                eprintln!("DEBUG: Validating @init for parameter '{}' with type '{:?}'", param.name, param.param_type);
                                // @init can only be applied to Account types (built-in or user-defined)
                                let is_valid_account = match &param.param_type {
                                    crate::ast::TypeNode::Account => true,
                                    crate::ast::TypeNode::Named(name) => {
                                        eprintln!("DEBUG: Checking named type '{}'", name);
                                        if name == "Account" || name == "account" {
                                            true
                                        } else if account_definitions.iter().any(|def| {
                                            if let AstNode::AccountDefinition { name: acc_name, .. } = def {
                                                // Match both unqualified name and namespaced name (e.g., "AMMPool" or "amm_types::AMMPool")
                                                acc_name == name || acc_name.ends_with(&format!("::{}", name))
                                            } else {
                                                false
                                            }
                                        }) {
                                            true
                                        } else {
                                            // Check module scope for imported accounts
                                            // We need to verify that the named type resolves to an Account type
                                            if let Some(scope) = &self.module_scope {
                                                eprintln!("DEBUG: Checking module scope for '{}'", name);
                                                if let Some(symbol) = scope.resolve_symbol(name, scope.current_module()) {
                                                     eprintln!("DEBUG: Resolved symbol for '{}': {:?}", name, symbol);
                                                     matches!(symbol.type_info, crate::ast::TypeNode::Account)
                                                } else {
                                                    eprintln!("DEBUG: Could not resolve symbol for '{}' in module scope", name);
                                                    false
                                                }
                                            } else {
                                                eprintln!("DEBUG: No module scope available for '{}'", name);
                                                false
                                            }
                                        }
                                    }
                                    _ => false,
                                };

                                if !is_valid_account {
                                    eprintln!("DEBUG: @init validation FAILED for parameter '{}'", param.name);
                                    return Err(VMError::ConstraintViolation);
                                }
                                eprintln!("DEBUG: @init validation PASSED for parameter '{}'", param.name);
                            }

                            let mut is_mutable = param.is_init;
                            if !is_mutable {
                                // Check for explicit @mut attribute
                                is_mutable = param.attributes.iter().any(|attr| attr.name == "mut");
                            }

                            let param_type = if param.param_type.is_account_type() {
                                TypeNode::Account
                            } else {
                                param.param_type.clone()
                            };
                            self.symbol_table
                                .insert(param.name.clone(), (param_type.clone(), is_mutable));
                            all_instruction_params.insert(param.name.clone(), (param_type, is_mutable));
                            // Aggregate all parameters
                        }
                    }
                    
                    if let Err(e) = self.check_types(instruction_def) {
                        eprintln!("Failed checking instruction definition: {:?}", e);
                        return Err(e);
                    }

                    // Restore original symbol table for the next instruction
                    self.symbol_table = original_symbol_table;
                }

                // Check event definitions
                for event_def in event_definitions {
                    self.check_types(event_def)?;
                }

                if let Some(init) = init_block {
                    self.check_types(init)?;
                }

                // Add all collected instruction parameters to the main symbol table
                // so they are visible to the constraints block.
                self.symbol_table.extend(all_instruction_params);

                // Check constraints block with the fully populated symbol table
                if let Some(constraints) = constraints_block {
                    // The symbol table should already contain all global fields and instruction parameters
                    // from the previous checks. No need to aggregate parameters separately.
                    self.check_types(constraints)?;
                }
                Ok(())
            }
            // Field definition
            AstNode::FieldDefinition {
                name,
                field_type,
                is_mutable,
                default_value,
                ..
            } => self.check_field_definition(name, field_type, *is_mutable, default_value),
            // Instruction definition
            AstNode::InstructionDefinition {
                name,
                parameters,
                return_type,
                body,
                visibility: _,
                ..
            } => self.check_instruction_definition(name, parameters, return_type, body),
            // Event definition
            AstNode::EventDefinition { name: _, fields, visibility: _ } => self.check_event_definition(fields),
            // Account definition
            AstNode::AccountDefinition { name, fields, visibility } => {
                self.check_account_definition(name, fields, *visibility)
            }
            // Error type definition
            AstNode::ErrorTypeDefinition { name, variants } => {
                self.check_error_type_definition(name, variants)
            }
            // Test function
            AstNode::TestFunction {
                name: _,
                attributes: _,
                body,
            } => self.check_test_function(body),
            // Test module - pass through for now
            AstNode::TestModule {
                name: _,
                attributes: _,
                body: _,
            } => Ok(()),
            // Interface definitions - handled separately
            AstNode::InterfaceDefinition { .. } => Ok(()),
            AstNode::InterfaceFunction { .. } => Ok(()),
            // Import statements - processed during compilation
            AstNode::ImportStatement { .. } => Ok(()),
            // Try expression checking first
            _ => {
                // Try checking as an expression
                match self.check_expression(ast) {
                    Ok(()) => return Ok(()),
                    Err(e) => {
                        if e != VMError::InvalidScript {
                            return Err(e);
                        }
                    }
                }
                // Try checking as a statement
                match self.check_statement(ast) {
                    Ok(()) => return Ok(()),
                    Err(e) => {
                        if e != VMError::InvalidScript {
                            return Err(e);
                        }
                    }
                }
                // If neither works, it's an invalid node
                eprintln!("Unhandled AST node in check_types: {:?}", ast);
                Err(VMError::InvalidScript)
            }
        }
    }
}
