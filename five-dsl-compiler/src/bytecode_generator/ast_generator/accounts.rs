//! Account field handling and initialization.

// use super::super::types::FieldInfo;
use super::super::OpcodeEmitter;
use super::types::ASTGenerator;
use crate::ast::InstructionParameter;
use crate::ast::{AstNode, TypeNode};
use five_protocol::opcodes::*;
use five_vm_mito::error::VMError;

impl ASTGenerator {
    fn resolve_account_param_index(&self, param_name: &str, fallback_param_index: usize) -> u8 {
        if let Some(params) = self.current_function_parameters.as_ref() {
            let registry = self
                .account_system
                .as_ref()
                .map(|s| s.get_account_registry());
            let mut account_ordinal: u8 = 0;
            for param in params.iter() {
                let is_account = super::super::account_utils::is_account_parameter(
                    &param.param_type,
                    &param.attributes,
                    registry,
                );
                if param.name == param_name {
                    if is_account {
                        return super::super::account_utils::account_index_from_param_index(
                            account_ordinal,
                        );
                    }
                    break;
                }
                if is_account {
                    account_ordinal = account_ordinal.saturating_add(1);
                }
            }
        }
        super::super::account_utils::account_index_from_param_index(fallback_param_index as u8)
    }

    pub(crate) fn init_ctx_bump_alias(account_name: &str) -> String {
        format!("__ctx_bump_{}", account_name)
    }

    pub(crate) fn init_ctx_space_alias(account_name: &str) -> String {
        format!("__ctx_space_{}", account_name)
    }

    fn bind_init_bump_alias<T: OpcodeEmitter>(&mut self, emitter: &mut T, alias: &str) {
        // Internal alias used by `account.ctx.bump` lowering.
        if self.local_symbol_table.contains_key(alias)
            || self.global_symbol_table.contains_key(alias)
        {
            return;
        }
        let slot = self.add_local_field(alias.to_string(), "u8".to_string(), false, false);
        emitter.emit_opcode(DUP);
        self.emit_set_local(emitter, slot, &format!("@init bump alias '{}'", alias));
    }

    fn bind_init_space_alias<T: OpcodeEmitter>(
        &mut self,
        emitter: &mut T,
        alias: &str,
        space: u64,
    ) -> Result<(), VMError> {
        if self.local_symbol_table.contains_key(alias)
            || self.global_symbol_table.contains_key(alias)
        {
            return Ok(());
        }
        let slot = self.add_local_field(alias.to_string(), "u64".to_string(), false, false);
        emitter.emit_const_u64(space)?;
        self.emit_set_local(emitter, slot, &format!("@init space alias '{}'", alias));
        Ok(())
    }

    pub(crate) fn emit_pda_param_setup<T: OpcodeEmitter>(
        &mut self,
        emitter: &mut T,
        parameters: &[InstructionParameter],
    ) -> Result<(), VMError> {
        for (index, param) in parameters.iter().enumerate() {
            let Some(pda_config) = &param.pda_config else {
                continue;
            };

            let account_index = self.resolve_account_param_index(&param.name, index);
            let bump_alias = Self::init_ctx_bump_alias(&param.name);

            if let Some(bump_var) = &pda_config.bump {
                for seed in &pda_config.seeds {
                    self.generate_ast_node(emitter, seed)?;
                }
                self.generate_ast_node(emitter, &AstNode::Identifier(bump_var.clone()))?;
                emitter.emit_const_u8((pda_config.seeds.len() + 1) as u8)?;
                emitter.emit_opcode(GET_KEY);
                emitter.emit_u8(account_index);
                emitter.emit_opcode(CHECK_PDA);
                self.generate_ast_node(emitter, &AstNode::Identifier(bump_var.clone()))?;
                self.bind_init_bump_alias(emitter, &bump_alias);
            } else {
                for seed in &pda_config.seeds {
                    self.generate_ast_node(emitter, seed)?;
                }
                emitter.emit_const_u8(pda_config.seeds.len() as u8)?;
                emitter.emit_opcode(PUSH_0);
                emitter.emit_opcode(FIND_PDA);
                emitter.emit_opcode(UNPACK_TUPLE);
                self.bind_init_bump_alias(emitter, &bump_alias);
                emitter.emit_opcode(DROP);
                emitter.emit_opcode(GET_KEY);
                emitter.emit_u8(account_index);
                emitter.emit_opcode(EQ);
                emitter.emit_opcode(REQUIRE);
            }
        }

        Ok(())
    }

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
            let account_info = registry.account_types.get(account_name).or_else(|| {
                registry
                    .account_types
                    .iter()
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

        // No fallback heuristic - AccountSystem is required for proper field offset resolution.
        // The old hardcoded heuristic returned incorrect offsets that varied by account type:
        // - TokenAccount.owner is at offset 0, but heuristic said 40
        // - TokenAccount.delegate is at offset 81, but heuristic said 72
        // These silent bugs caused functions to read/write wrong account fields at runtime.
        // Now all account types must be properly registered in AccountSystem for correct offsets.
        println!("DSL Compiler ERROR: Cannot resolve field offset without AccountSystem");
        println!("  Account: '{}'", account_name);
        println!("  Field: '{}'", field_name);
        println!("  Fix: Ensure the account type is defined in your Five script");

        Err(VMError::UndefinedAccountField)
    }

    /// Helper to extract account type name from TypeNode
    /// Handles Account<T> -> T unwrapping
    fn extract_account_type_name_static(type_node: &TypeNode, generator: &Self) -> String {
        match type_node {
            TypeNode::Generic { base, args } => {
                if base == "Account" && !args.is_empty() {
                    // Extract T from Account<T>
                    // Assume Account<T> has 1 arg.
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
        println!(
            "@@@ INIT_SEQUENCE_CHECK: param='{}', is_init={}, index={}",
            param.name, param.is_init, index
        );
        if !param.is_init {
            println!(
                "@@@ INIT_SEQUENCE_SKIP: is_init=false for param '{}'",
                param.name
            );
            return Ok(());
        }
        println!(
            "@@@ INIT_SEQUENCE_PROCEED: Generating initialization for param '{}'",
            param.name
        );

        // Get the init configuration
        let init_config = param.init_config.as_ref().ok_or(VMError::InvalidScript)?;
        let account_index = self.resolve_account_param_index(&param.name, index);

        // Generate pre-creation validation: CHECK_UNINITIALIZED
        // This ensures the account is not already initialized
        emitter.emit_opcode(CHECK_UNINITIALIZED);
        emitter.emit_u8(account_index); // Account index in instruction context

        // Generate account creation logic based on whether seeds are present
        // Auto-calculate space when omitted in @init(...).
        // Space defaults to account layout byte size from AccountSystem (data-only).
        let space = if let Some(s) = init_config.space {
            s
        } else {
            // Auto-calculate space from account struct layout size.
            // We need to resolve the account type from the parameter
            let type_name = Self::extract_account_type_name_static(&param.param_type, self);

            println!(
                "AST Generator: Auto-calculating space for account type '{}' (@init)",
                type_name
            );

            let calculated_size = if let Some(account_system) = &self.account_system {
                let registry = account_system.get_account_registry();
                // Lookup with namespace support
                let namespace_suffix = format!("::{}", type_name);
                if type_name == "Account" {
                    println!("AST Generator: generic Account type, assuming 0 data size");
                    0
                } else {
                    let account_info = registry
                        .account_types
                        .get(&type_name)
                        .or_else(|| {
                            registry
                                .account_types
                                .iter()
                                .find(|(k, _)| k.ends_with(&namespace_suffix))
                                .map(|(_, v)| v)
                        })
                        .ok_or_else(|| {
                            println!(
                                "AST Generator: ERROR - Account type '{}' not found in registry",
                                type_name
                            );
                            VMError::UndefinedAccountField
                        })?;

                    println!(
                        "AST Generator: Found account definition, size={}, total required={}",
                        account_info.total_size, account_info.total_size as u64
                    );
                    account_info.total_size as u64
                }
            } else {
                println!(
                    "AST Generator: ERROR - AccountSystem not available for space calculation"
                );
                return Err(VMError::InvalidScript);
            };

            calculated_size
        };
        let space_alias = Self::init_ctx_space_alias(&param.name);
        self.bind_init_space_alias(emitter, &space_alias, space)?;

        // Generate account creation logic based on whether seeds are present
        match &init_config.seeds {
            Some(seeds) => {
                // PDA account creation with seeds
                // Stack required: [bump, seeds..., count, owner, lamports, payer_idx, space, account_idx] (Top)
                // Note: VM pops account_idx first, then space, then payer_idx, then lamports...

                // 1. Push Bump (Bottom of stack frame for this op)
                if let Some(bump_var) = &init_config.bump {
                    // Explicit bump source: load caller-provided/parsed bump variable.
                    // If unresolved, raise a targeted error to avoid opaque identifier failures.
                    let bump_known = self.local_symbol_table.contains_key(bump_var)
                        || self.global_symbol_table.contains_key(bump_var);
                    if !bump_known {
                        println!(
                            "DSL Compiler ERROR: unresolved bump identifier '{}' for @init seeds",
                            bump_var
                        );
                        println!(
                            "  Fix: define '{}' before use, or omit bump=... to auto-derive canonical bump",
                            bump_var
                        );
                        return Err(VMError::InvalidScript);
                    }
                    // Generate code to load the bump variable
                    self.generate_ast_node(emitter, &AstNode::Identifier(bump_var.clone()))?;
                } else {
                    // Auto bump fallback: derive canonical bump using FIND_PDA.
                    // We need to push seeds first for FIND_PDA
                    for seed in seeds {
                        self.generate_ast_node(emitter, seed)?;
                    }

                    // Push seeds count
                    emitter.emit_const_u8(seeds.len() as u8)?;

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

                // Publish bump in an easy-to-discover local alias for this account.
                let bump_alias = Self::init_ctx_bump_alias(&param.name);
                self.bind_init_bump_alias(emitter, &bump_alias);

                // 2. Push Seeds
                for seed in seeds {
                    self.generate_ast_node(emitter, seed)?;
                }

                // 3. Push Seeds Count
                emitter.emit_const_u8(seeds.len() as u8)?; // Checked MAX_SEEDS in VM

                // 4. Push Owner (0 -> Current Program ID)
                emitter.emit_opcode(PUSH_0); // 0xD8 (ValueRef::U64(0))

                // 5. Push Lamports (Calculate using GET_RENT based on space)
                emitter.emit_const_u64(space)?;
                emitter.emit_opcode(GET_RENT); // Consumes space, pushes lamports

                // 6. Push Payer Index
                let payer_idx = if let Some(ref payer_name) = init_config.payer {
                    self.resolve_payer_account_index(payer_name)?
                } else {
                    self.find_first_signer_account_index()?
                };
                emitter.emit_const_u8(payer_idx)?;

                // 7. Push Space
                emitter.emit_const_u64(space)?;

                // 8. Push Account Index (Top of stack)
                emitter.emit_const_u8(account_index)?;

                // 9. Emit Opcode
                emitter.emit_opcode(INIT_PDA_ACCOUNT);
            }
            None => {
                // Regular account creation (not PDA)
                // Stack: [owner, lamports, payer_idx, space, account_idx] (Top)

                // 1. Push Owner (0 -> Current Program ID)
                emitter.emit_opcode(PUSH_0);

                // 2. Push Lamports (GET_RENT)
                emitter.emit_const_u64(space)?;
                emitter.emit_opcode(GET_RENT);

                // 3. Push Payer Index
                let payer_idx = if let Some(ref payer_name) = init_config.payer {
                    self.resolve_payer_account_index(payer_name)?
                } else {
                    self.find_first_signer_account_index()?
                };
                emitter.emit_const_u8(payer_idx)?;

                // 4. Push Space
                emitter.emit_const_u64(space)?;

                // 5. Push Account Index
                emitter.emit_const_u8(account_index)?;

                emitter.emit_opcode(INIT_ACCOUNT);
            }
        }

        Ok(())
    }

    /// Resolve payer parameter name to account index
    fn resolve_payer_account_index(&self, payer_name: &str) -> Result<u8, VMError> {
        let params = self
            .current_function_parameters
            .as_ref()
            .ok_or(VMError::InvalidScript)?;

        for (idx, param) in params.iter().enumerate() {
            if param.name == payer_name {
                // Verify this is an account type
                if !matches!(
                    param.param_type,
                    crate::ast::TypeNode::Account | crate::ast::TypeNode::Named(_)
                ) {
                    return Err(VMError::TypeMismatch);
                }

                // Account indices use centralized ACCOUNT_INDEX_OFFSET constant
                let account_idx = self.resolve_account_param_index(&param.name, idx);
                return Ok(account_idx);
            }
        }

        Err(VMError::InvalidScript) // Payer not found
    }

    /// Find first signer for default payer (when payer= not specified)
    fn find_first_signer_account_index(&self) -> Result<u8, VMError> {
        let params = self
            .current_function_parameters
            .as_ref()
            .ok_or(VMError::InvalidScript)?;

        for (idx, param) in params.iter().enumerate() {
            if param.attributes.iter().any(|a| a.name == "signer") {
                let account_idx = self.resolve_account_param_index(&param.name, idx);
                return Ok(account_idx);
            }
        }

        Err(VMError::ConstraintViolation) // No signer found
    }
}
