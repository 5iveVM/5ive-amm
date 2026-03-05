// ABI generation for client-side integration.

use super::account_utils;
use super::types::*;
use crate::ast::AstNode;
use five_vm_mito::error::VMError;
use std::collections::HashMap;

/// ABI Generator for extracting interface information from AST
pub struct ABIGenerator {
    /// Current program name being processed
    program_name: String,

    /// Collected function information
    functions: Vec<ABIFunction>,

    /// Collected field information
    fields: Vec<ABIField>,

    /// Function index counter
    function_index: u8,

    /// Account registry for custom account type detection
    account_registry: Option<AccountRegistry>,
}

impl ABIGenerator {
    /// Create a new ABI generator
    pub fn new() -> Self {
        Self {
            program_name: String::new(),
            functions: Vec::new(),
            fields: Vec::new(),
            function_index: 0,
            account_registry: None,
        }
    }

    /// Create a new ABI generator with account registry
    pub fn with_account_registry(account_registry: AccountRegistry) -> Self {
        Self {
            program_name: String::new(),
            functions: Vec::new(),
            fields: Vec::new(),
            function_index: 0,
            account_registry: Some(account_registry),
        }
    }

    /// Generate a complete FIVEABI from an AST
    pub fn generate_five_abi(&mut self, ast: &AstNode) -> Result<FIVEABI, VMError> {
        self.reset();
        self.ensure_registry_from_ast(ast)?;
        self.process_ast(ast)?;

        Ok(FIVEABI {
            program_name: self.program_name.clone(),
            functions: self.functions.clone(),
            fields: self.fields.clone(),
            version: "1.0".to_string(),
        })
    }

    /// Generate a simplified ABI from an AST (for client-side function calls)
    pub fn generate_simple_abi(&mut self, ast: &AstNode) -> Result<SimpleABI, VMError> {
        self.reset();
        self.ensure_registry_from_ast(ast)?;
        self.process_ast(ast)?;

        let mut simple_functions = HashMap::new();

        for function in &self.functions {
            let simple_params: Vec<SimpleABIParameter> = function
                .parameters
                .iter()
                .filter(|param| !param.is_account) // Filter out account parameters
                .map(|param| SimpleABIParameter {
                    name: param.name.clone(),
                    param_type: param.param_type.clone(),
                })
                .collect();

            let simple_accounts: Vec<SimpleABIAccount> = function
                .parameters
                .iter()
                .filter(|param| param.is_account) // Extract account parameters
                .map(|param| SimpleABIAccount {
                    name: param.name.clone(),
                    writable: param.attributes.contains(&"mut".to_string()),
                    signer: param.attributes.contains(&"signer".to_string()),
                })
                .collect();

            simple_functions.insert(
                function.name.clone(),
                SimpleABIFunction {
                    index: function.index,
                    parameters: simple_params,
                    accounts: simple_accounts,
                },
            );
        }

        Ok(SimpleABI {
            version: "1.0".to_string(),
            name: self.program_name.clone(),
            functions: simple_functions,
        })
    }

    /// Ensure account registry is ready; if empty, populate from AST account definitions.
    fn ensure_registry_from_ast(&mut self, ast: &AstNode) -> Result<(), VMError> {
        let needs_load = match &self.account_registry {
            None => true,
            Some(reg) => reg.account_types.is_empty(),
        };
        if !needs_load {
            return Ok(());
        }

        // Build registry from AST account definitions using AccountSystem
        let mut system = crate::bytecode_generator::account_system::AccountSystem::new();
        system.process_account_definitions(ast)?;
        self.account_registry = Some(system.get_account_registry().clone());
        Ok(())
    }

    /// Reset the generator state for a new compilation
    fn reset(&mut self) {
        self.program_name.clear();
        self.functions.clear();
        self.fields.clear();
        self.function_index = 0;
        // Note: Keep account_registry as it's set during construction
    }

    /// Process the top-level AST node
    fn process_ast(&mut self, ast: &AstNode) -> Result<(), VMError> {
        match ast {
            AstNode::Program {
                program_name,
                field_definitions,
                instruction_definitions,
                ..
            } => {
                self.program_name = program_name.clone();

                // Process field definitions
                for field_def in field_definitions {
                    self.process_field_definition(field_def)?;
                }

                // Process instruction definitions (functions) with visibility-based ordering
                // Phase 2: Separate public and private functions for proper visibility enforcement
                let mut public_functions = Vec::new();
                let mut private_functions = Vec::new();

                // Separate functions by visibility
                for instruction_def in instruction_definitions {
                    if let AstNode::InstructionDefinition { visibility, .. } = instruction_def {
                        if visibility.is_on_chain_callable() {
                            public_functions.push(instruction_def);
                        } else {
                            private_functions.push(instruction_def);
                        }
                    }
                }

                // Process public functions first (get indices 0, 1, 2... - externally callable)
                for public_function in public_functions {
                    self.process_instruction_definition(public_function)?;
                }

                // Process private functions after (get higher indices - internal only)
                for private_function in private_functions {
                    self.process_instruction_definition(private_function)?;
                }

                Ok(())
            }
            _ => Err(VMError::InvalidScript),
        }
    }

    /// Process a field definition and add to ABI
    fn process_field_definition(&mut self, field_def: &AstNode) -> Result<(), VMError> {
        match field_def {
            AstNode::FieldDefinition {
                name,
                field_type,
                is_mutable,
                ..
            } => {
                let type_string = account_utils::type_node_to_string(field_type);

                self.fields.push(ABIField {
                    name: name.clone(),
                    field_type: type_string,
                    is_mutable: *is_mutable,
                    memory_offset: 0,
                });

                Ok(())
            }
            _ => Ok(()), // Skip non-field definitions
        }
    }

    /// Process an instruction definition and add to ABI
    fn process_instruction_definition(&mut self, instruction_def: &AstNode) -> Result<(), VMError> {
        match instruction_def {
            AstNode::InstructionDefinition {
                name,
                parameters,
                return_type,
                visibility,
                ..
            } => {
                let mut abi_parameters = Vec::new();

                // Process each parameter
                for param in parameters {
                    let param_type_string = account_utils::type_node_to_string(&param.param_type);
                    let is_account = account_utils::is_account_parameter(
                        &param.param_type,
                        &param.attributes,
                        self.account_registry.as_ref(),
                    );

                    abi_parameters.push(ABIParameter {
                        name: param.name.clone(),
                        param_type: param_type_string,
                        is_account,
                        attributes: param.attributes.iter().map(|a| a.name.clone()).collect(),
                    });
                }

                // Process return type
                let return_type_string = return_type
                    .as_ref()
                    .map(|rt| account_utils::type_node_to_string(rt));

                self.functions.push(ABIFunction {
                    name: name.clone(),
                    index: self.function_index,
                    parameters: abi_parameters,
                    return_type: return_type_string,
                    is_public: visibility.is_on_chain_callable(), // Use visibility from AST
                    bytecode_offset: 0,
                });

                self.function_index += 1;
                Ok(())
            }
            _ => Ok(()), // Skip non-instruction definitions
        }
    }
}

impl Default for ABIGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Extension methods for the main DslBytecodeGenerator
impl super::DslBytecodeGenerator {
    /// Generate a complete FIVEABI from the AST
    pub fn generate_abi(&mut self, ast: &AstNode) -> Result<FIVEABI, VMError> {
        let mut abi_generator = ABIGenerator::with_account_registry(self.account_registry.clone());
        abi_generator.generate_five_abi(ast)
    }

    /// Generate a simplified ABI from the AST (for client-side function calls)
    pub fn generate_simple_abi(&mut self, ast: &AstNode) -> Result<SimpleABI, VMError> {
        let mut abi_generator = ABIGenerator::with_account_registry(self.account_registry.clone());
        abi_generator.generate_simple_abi(ast)
    }
}

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use super::*;
    use crate::ast::{AstNode, BlockKind, InstructionParameter, TypeNode};

    #[test]
    fn test_simple_abi_generation() {
        let mut generator = ABIGenerator::new();

        // Create a simple test AST
        let ast = AstNode::Program {
            program_name: "test_program".to_string(),
            field_definitions: vec![],
            instruction_definitions: vec![AstNode::InstructionDefinition {
                name: "transfer".to_string(),
                visibility: crate::Visibility::Public,
                is_public: true,
                parameters: vec![InstructionParameter {
                    name: "amount".to_string(),
                    param_type: TypeNode::Primitive("u64".to_string()),
                    is_optional: false,
                    default_value: None,
                    attributes: vec![],
                    is_init: false,
                    init_config: None,
                    serializer: None,
                    pda_config: None,
                }],
                return_type: Some(Box::new(TypeNode::Primitive("bool".to_string()))),
                body: Box::new(AstNode::Block {
                    statements: vec![],
                    kind: BlockKind::Regular,
                }),
            }],
            init_block: None,
            constraints_block: None,
            event_definitions: vec![],
            account_definitions: vec![],
            type_definitions: vec![],
            interface_definitions: vec![],
            import_statements: vec![],
        };

        let simple_abi = generator.generate_simple_abi(&ast).unwrap();

        assert_eq!(simple_abi.name, "test_program");
        assert_eq!(simple_abi.functions.len(), 1);
        assert!(simple_abi.functions.contains_key("transfer"));

        let transfer_func = &simple_abi.functions["transfer"];
        assert_eq!(transfer_func.index, 0);
        assert_eq!(transfer_func.parameters.len(), 1);
        assert_eq!(transfer_func.parameters[0].name, "amount");
        assert_eq!(transfer_func.parameters[0].param_type, "u64");
    }

    #[test]
    fn test_account_parameter_filtering() {
        let mut generator = ABIGenerator::new();

        let ast = AstNode::Program {
            program_name: "account_test".to_string(),
            field_definitions: vec![],
            instruction_definitions: vec![AstNode::InstructionDefinition {
                name: "process".to_string(),
                visibility: crate::Visibility::Public,
                is_public: true,
                parameters: vec![
                    InstructionParameter {
                        name: "signer".to_string(),
                        param_type: TypeNode::Primitive("Account".to_string()),
                        is_optional: false,
                        default_value: None,
                        attributes: vec![crate::ast::Attribute {
                            name: "signer".to_string(),
                            args: vec![],
                        }],
                        is_init: false,
                        init_config: None,
                        serializer: None,
                        pda_config: None,
                    },
                    InstructionParameter {
                        name: "amount".to_string(),
                        param_type: TypeNode::Primitive("u64".to_string()),
                        is_optional: false,
                        default_value: None,
                        attributes: vec![],
                        is_init: false,
                        init_config: None,
                        serializer: None,
                        pda_config: None,
                    },
                ],
                return_type: None,
                body: Box::new(AstNode::Block {
                    statements: vec![],
                    kind: BlockKind::Regular,
                }),
            }],
            init_block: None,
            constraints_block: None,
            event_definitions: vec![],
            account_definitions: vec![],
            type_definitions: vec![],
            interface_definitions: vec![],
            import_statements: vec![],
        };

        let simple_abi = generator.generate_simple_abi(&ast).unwrap();
        let process_func = &simple_abi.functions["process"];

        // Should have one account and one parameter
        assert_eq!(process_func.accounts.len(), 1);
        assert_eq!(process_func.parameters.len(), 1);

        // Account should be extracted correctly
        assert_eq!(process_func.accounts[0].name, "signer");
        assert!(process_func.accounts[0].signer);
        assert!(!process_func.accounts[0].writable);

        // Parameter should be non-account
        assert_eq!(process_func.parameters[0].name, "amount");
        assert_eq!(process_func.parameters[0].param_type, "u64");
    }
}
