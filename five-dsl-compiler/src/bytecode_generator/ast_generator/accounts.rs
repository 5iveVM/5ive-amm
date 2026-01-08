//! Account field handling and initialization
//!
//! This module handles account field offset calculations and account initialization.

// use super::super::types::FieldInfo;
use super::super::OpcodeEmitter;
use super::types::ASTGenerator;
use crate::ast::InstructionParameter;
use five_protocol::opcodes::*;
use five_vm_mito::error::VMError;
use crate::ast::{AstNode, TypeNode};


impl ASTGenerator {
    /// Calculate field offset within an account structure
    /// Now properly integrates with AccountSystem registry for dynamic field resolution
    pub(super) fn calculate_account_field_offset(
        &self,
        account_name: &str,
        field_name: &str,
    ) -> Result<u32, VMError> {
        println!(
            "AST Generator: Calculating field offset for account '{}' field '{}'",
            account_name, field_name
        );

        // Use AccountSystem.account_registry to get proper field offsets
        if let Some(ref account_system) = self.account_system {
            let registry = account_system.get_account_registry();
            println!(
                "AST Generator: Account registry has {} registered types",
                registry.account_types.len()
            );

            // Debug: Print all registered account types
            for name in registry.account_types.keys() {
                println!("AST Generator: Registry contains account type: '{}'", name);
            }

            // Look up the account type in the registry with namespace-aware matching
            let namespace_suffix = format!("::{}", account_name);
            let account_info = registry.account_types.get(account_name)
                .or_else(|| {
                    registry.account_types.iter()
                        .find(|(k, _)| k.ends_with(&namespace_suffix))
                        .map(|(_, v)| v)
                });

            if let Some(account_info) = account_info {
                println!(
                    "AST Generator: Found account type '{}' with {} fields",
                    account_name,
                    account_info.fields.len()
                );

                // Debug: Print all fields in the account
                for (fname, finfo) in &account_info.fields {
                    println!(
                        "AST Generator: Account '{}' has field '{}' at offset {}",
                        account_name, fname, finfo.offset
                    );
                }

                // Look up the field in the account type
                if let Some(field_info) = account_info.fields.get(field_name) {
                    println!(
                        "AST Generator: Found field '{}' at offset {}",
                        field_name, field_info.offset
                    );
                    return Ok(field_info.offset);
                } else {
                    // Field not found in account definition
                    println!(
                        "AST Generator: ERROR - Field '{}' not found in account '{}'",
                        field_name, account_name
                    );
                    return Err(VMError::UndefinedAccountField);
                }
            } else {
                // Account type not found in registry
                println!(
                    "AST Generator: ERROR - Account type '{}' not found in registry",
                    account_name
                );
                return Err(VMError::UndefinedAccountField);
            }
        } else {
            println!("AST Generator: WARNING - No AccountSystem available");
        }

        // Fallback to legacy heuristic if AccountSystem is not available
        // This maintains backward compatibility during transition
        println!("DSL Compiler WARNING: AccountSystem not available, using legacy heuristic for field '{}' in account '{}'",
            field_name, account_name);

        match field_name {
            // Standard u64 fields (8 bytes each)
            "count" | "amount" | "balance" | "value" => Ok(0),
            "total" | "supply" | "max_supply" => Ok(8),
            "fee" | "rate" | "timestamp" => Ok(16),
            "nonce" | "sequence" | "version" => Ok(24),

            // Boolean fields (1 byte each, but aligned to 8-byte boundaries)
            "is_active" | "is_frozen" | "has_authority" => Ok(32),
            "is_mutable" | "is_initialized" => Ok(33),

            // Pubkey fields (32 bytes each)
            "authority" | "owner" | "mint" => Ok(40),
            "delegate" | "close_authority" => Ok(72),

            // Unknown field - return specific error
            _ => {
                println!("DSL Compiler ERROR: Unknown field '{}' for account '{}' and no AccountSystem available",
                    field_name, account_name);
                Err(VMError::UndefinedAccountField)
            }
        }
    }

    /// Helper to extract account type name from TypeNode
    /// Handles Account<T> -> T unwrapping
    fn extract_account_type_name_static(type_node: &TypeNode, generator: &Self) -> String {
        match type_node {
            TypeNode::Generic { base, args } => {
                if base == "Account" && !args.is_empty() {
                    // Extract T from Account<T>
                    // For now, we assume Account<T> has 1 arg.
                    // We need to use generator's helper because TypeNode doesn't have it
                    // But we can just use the type_node_to_string from generator
                    // Note: type_node_to_string is not pub, so we might need to rely on generator having it.
                    // Actually, let's just use the generator instance provided.
                     match &args[0] {
                        TypeNode::Primitive(name) | TypeNode::Named(name) => name.clone(),
                        _ => generator.type_node_to_string(&args[0]),
                    }
                } else {
                    generator.type_node_to_string(type_node)
                }
            }
             TypeNode::Primitive(name) | TypeNode::Named(name) => name.clone(),
            _ => generator.type_node_to_string(type_node),
        }
    }

    /// Generate account initialization sequence
    pub fn generate_init_account_sequence<T: OpcodeEmitter>(
        &mut self,
        emitter: &mut T,
        param: &InstructionParameter,
        index: usize,
    ) -> Result<(), VMError> {
        println!("@@@ INIT_SEQUENCE_CHECK: param='{}', is_init={}, index={}", param.name, param.is_init, index);
        if !param.is_init {
            println!("@@@ INIT_SEQUENCE_SKIP: is_init=false for param '{}'", param.name);
            return Ok(());
        }
        println!("@@@ INIT_SEQUENCE_PROCEED: Generating initialization for param '{}'", param.name);

        // Get the init configuration
        let init_config = param.init_config.as_ref().ok_or(VMError::InvalidScript)?;
        let account_index = super::super::account_utils::account_index_from_param_index(index as u8);

        // Generate pre-creation validation: CHECK_UNINITIALIZED
        // This ensures the account is not already initialized
        emitter.emit_opcode(CHECK_UNINITIALIZED);
        emitter.emit_u8(account_index); // Account index in instruction context

        // Generate account creation logic based on whether seeds are present
        // Check if we need to auto-calculate space
        // This is done if space is None in init_config
        let space = if let Some(s) = init_config.space {
            s
        } else {
            // Auto-calculate space: 8 bytes (discriminator) + account struct size
            // We need to resolve the account type from the parameter
            let type_name = Self::extract_account_type_name_static(&param.param_type, self);
            
            println!("AST Generator: Auto-calculating space for account type '{}' (@init)", type_name);
            
            let mut calculated_size = 1024; // Default fallback (increased from 32 to 1024 for safety)
            
            if let Some(account_system) = &self.account_system {
                let registry = account_system.get_account_registry();
                // Lookup with namespace support
                let namespace_suffix = format!("::{}", type_name);
                let account_info = registry.account_types.get(&type_name)
                    .or_else(|| {
                        registry.account_types.iter()
                            .find(|(k, _)| k.ends_with(&namespace_suffix))
                            .map(|(_, v)| v)
                    });
                
                if let Some(info) = account_info {
                    calculated_size = info.total_size as u64; // Just fields size
                    println!("AST Generator: Found account definition, size={}, total required={}", info.total_size, calculated_size);
                } else {
                    println!("AST Generator: WARNING - Account type '{}' not found in registry, using default size {}", type_name, calculated_size);
                }
            } else {
                println!("AST Generator: WARNING - AccountSystem not available for space calculation");
            }
            
            calculated_size
        };

        // Generate account creation logic based on whether seeds are present
        match &init_config.seeds {
            Some(seeds) => {
                // PDA account creation with seeds
                // Stack required: [bump, seeds..., count, owner, lamports, payer_idx, space, account_idx] (Top)
                // Note: VM pops account_idx first, then space, then payer_idx, then lamports...

                // 1. Push Bump (Bottom of stack frame for this op)
                if let Some(bump_var) = &init_config.bump {
                    // Generate code to load the bump variable
                    self.generate_ast_node(emitter, &AstNode::Identifier(bump_var.clone()))?;
                } else {
                    // Dynamic bump derivation: Calculate canonical bump using FIND_PDA
                    // We need to push seeds first for FIND_PDA
                    for seed in seeds {
                        self.generate_ast_node(emitter, seed)?;
                    }

                    // Push seeds count
                    emitter.emit_opcode(PUSH_U8);
                    emitter.emit_u8(seeds.len() as u8);

                    // Push Five VM program ID as current program (0 -> Program ID in VM)
                    // This relies on extract_pubkey handling PUSH_0 (U64(0)) correctly
                    emitter.emit_opcode(PUSH_0);

                    // Emit FIND_PDA (0x87) -> Pushes (pda_pubkey, bump) Tuple
                    emitter.emit_opcode(FIND_PDA);

                    // Unpack Tuple -> Stack: [pda_pubkey, bump] (Top)
                    emitter.emit_opcode(UNPACK_TUPLE);

                    // We only need the bump for INIT_PDA_ACCOUNT
                    // Stack: [pda_pubkey, bump]
                    emitter.emit_opcode(SWAP); // Stack: [bump, pda_pubkey]
                    emitter.emit_opcode(DROP); // Stack: [bump]
                }

                // 2. Push Seeds
                for seed in seeds {
                    self.generate_ast_node(emitter, seed)?;
                }

                // 3. Push Seeds Count
                emitter.emit_opcode(PUSH_U8);
                emitter.emit_u8(seeds.len() as u8); // Checked MAX_SEEDS in VM

                // 4. Push Owner (0 -> Current Program ID)
                emitter.emit_opcode(PUSH_0); // 0xD8 (ValueRef::U64(0))

                // 5. Push Lamports (Calculate using GET_RENT based on space)
                emitter.emit_opcode(PUSH_U64);
                emitter.emit_vle_u64(space);
                emitter.emit_opcode(GET_RENT); // Consumes space, pushes lamports

                // 6. Push Payer Index
                let payer_idx = if let Some(ref payer_name) = init_config.payer {
                    self.resolve_payer_account_index(payer_name)?
                } else {
                    self.find_first_signer_account_index()?
                };
                emitter.emit_opcode(PUSH_U8);
                emitter.emit_u8(payer_idx);

                // 7. Push Space
                emitter.emit_opcode(PUSH_U64);
                emitter.emit_vle_u64(space);

                // 8. Push Account Index (Top of stack)
                emitter.emit_opcode(PUSH_U8);
                emitter.emit_u8(account_index);

                // 9. Emit Opcode
                emitter.emit_opcode(INIT_PDA_ACCOUNT);
            }
            None => {
                // Regular account creation (not PDA)
                // Stack: [owner, lamports, payer_idx, space, account_idx] (Top)

                // 1. Push Owner (0 -> Current Program ID)
                emitter.emit_opcode(PUSH_0);

                // 2. Push Lamports (GET_RENT)
                emitter.emit_opcode(PUSH_U64);
                emitter.emit_vle_u64(space);
                emitter.emit_opcode(GET_RENT);

                // 3. Push Payer Index
                let payer_idx = if let Some(ref payer_name) = init_config.payer {
                    self.resolve_payer_account_index(payer_name)?
                } else {
                    self.find_first_signer_account_index()?
                };
                emitter.emit_opcode(PUSH_U8);
                emitter.emit_u8(payer_idx);

                // 4. Push Space
                emitter.emit_opcode(PUSH_U64);
                emitter.emit_vle_u64(space);

                // 5. Push Account Index
                emitter.emit_opcode(PUSH_U8);
                emitter.emit_u8(account_index);

                emitter.emit_opcode(INIT_ACCOUNT);
            }
        }

        Ok(())
    }

    /// Resolve payer parameter name to account index
    fn resolve_payer_account_index(&self, payer_name: &str) -> Result<u8, VMError> {
        let params = self.current_function_parameters.as_ref()
            .ok_or(VMError::InvalidScript)?;

        for (idx, param) in params.iter().enumerate() {
            if param.name == payer_name {
                // Verify this is an account type
                if !matches!(param.param_type,
                    crate::ast::TypeNode::Account | crate::ast::TypeNode::Named(_)) {
                    return Err(VMError::TypeMismatch);
                }

                // Account indices use centralized ACCOUNT_INDEX_OFFSET constant
                let account_idx = super::super::account_utils::account_index_from_param_index(idx as u8);
                return Ok(account_idx);
            }
        }

        Err(VMError::InvalidScript) // Payer not found
    }

    /// Find first signer for default payer (when payer= not specified)
    fn find_first_signer_account_index(&self) -> Result<u8, VMError> {
        let params = self.current_function_parameters.as_ref()
            .ok_or(VMError::InvalidScript)?;

        for (idx, param) in params.iter().enumerate() {
            if param.attributes.iter().any(|a| a.name == "signer") {
                let account_idx = super::super::account_utils::account_index_from_param_index(idx as u8);
                return Ok(account_idx);
            }
        }

        Err(VMError::ConstraintViolation) // No signer found
    }
}
