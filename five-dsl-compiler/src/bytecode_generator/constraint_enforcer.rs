use crate::ast::{InstructionParameter, TypeNode, Attribute, AstNode};
use super::OpcodeEmitter;
use crate::bytecode_generator::account_utils::account_index_from_param_index;
use crate::bytecode_generator::types::AccountRegistry;
use five_protocol::opcodes::*;
use five_vm_mito::error::VMError;

/// Enforce account constraints by emitting validation bytecode
pub fn emit_constraint_checks<T: OpcodeEmitter>(
    emitter: &mut T,
    parameters: &[InstructionParameter],
    account_registry: &AccountRegistry,
) -> Result<(), VMError> {
    // Emit CHECK_SIGNER for @init payer parameters first
    emit_init_payer_checks(emitter, parameters)?;

    for (param_index, param) in parameters.iter().enumerate() {
        // Skip non-account parameters for most checks
        let is_account = is_account_param(param, account_registry);
        if !is_account {
             continue;
        }
        
        // Resolve account index (absolute index in VM)
        let account_idx = account_index_from_param_index(param_index as u8);
        
        // Process attributes
        for attribute in &param.attributes {
            match attribute.name.as_str() {
                "signer" => {
                    emit_signer_check(emitter, account_idx)?;
                }
                "mut" | "writable" => {
                    // Implicitly handled by loader, but we could add explicit check:
                    emit_writable_check(emitter, account_idx)?;
                }
                "has" => {
                    emit_has_check(emitter, account_idx, param, attribute, parameters, account_registry)?;
                }

                 _ => {}
            }
        }
    }

    Ok(())
}

/// Emit CHECK_SIGNER for @init payer parameters
/// This ensures the payer account has signed the transaction
fn emit_init_payer_checks<T: OpcodeEmitter>(
    emitter: &mut T,
    parameters: &[InstructionParameter],
) -> Result<(), VMError> {
    for param in parameters.iter() {
        if let Some(ref init_config) = param.init_config {
            if let Some(ref payer_name) = init_config.payer {
                // Find the payer parameter index
                let payer_param_index = parameters.iter()
                    .position(|p| p.name == *payer_name)
                    .ok_or(VMError::InvalidScript)?;

                let payer_account_idx = account_index_from_param_index(payer_param_index as u8);

                // Emit CHECK_SIGNER for the payer
                emitter.emit_opcode(CHECK_SIGNER);
                emitter.emit_u8(payer_account_idx);
            }
        }
    }
    Ok(())
}

fn is_account_param(param: &InstructionParameter, account_registry: &AccountRegistry) -> bool {
    match &param.param_type {
        TypeNode::Account => true,
        TypeNode::Named(name) => {
            // Check if it's a registered account type
            // Also consider built-in accounts if any (TokenAccount, etc)
            // Ideally we should use AccountSystem::is_account_type logic but we don't have AccountSystem here.
            // We assume built-in accounts are NOT in registry but should be treated as accounts?
            // "Account" -> TypeNode::Account. "TokenAccount" -> Named("TokenAccount").
            // If TokenAccount is built-in and not in registry, checking registry fails.
            // But AccountSystem::is_account_type handles "TokenAccount" etc. explicitly.
            // We should replicate that or assume all Named types might be accounts?
            // NO, "Pubkey" is Named but NOT account.

            if matches!(name.as_str(), "Account" | "TokenAccount" | "ProgramAccount") {
                return true;
            }

            // Check registry
            let namespace_suffix = format!("::{}", name);
            account_registry.account_types.contains_key(name) ||
            account_registry.account_types.keys().any(|k| k.ends_with(&namespace_suffix))
        },
        _ => false,
    }
}

/// Emit CHECK_SIGNER opcode
fn emit_signer_check<T: OpcodeEmitter>(
    emitter: &mut T,
    account_idx: u8,
) -> Result<(), VMError> {
    emitter.emit_opcode(CHECK_SIGNER);
    emitter.emit_u8(account_idx); 
    Ok(())
}

/// Emit CHECK_WRITABLE opcode
fn emit_writable_check<T: OpcodeEmitter>(
    emitter: &mut T,
    account_idx: u8,
) -> Result<(), VMError> {
    emitter.emit_opcode(CHECK_WRITABLE);
    emitter.emit_u8(account_idx);
    Ok(())
}

/// Emit Bytecode for @has(field1, field2, ...)
/// Ensures: account.field == target_argument for each arg
fn emit_has_check<T: OpcodeEmitter>(
    emitter: &mut T,
    account_idx: u8,
    account_param: &InstructionParameter,
    attribute: &Attribute,
    all_parameters: &[InstructionParameter],
    account_registry: &AccountRegistry,
) -> Result<(), VMError> {
    if attribute.args.is_empty() {
        return Err(VMError::InvalidInstruction);
    }

    for arg in &attribute.args {
        // 1. Identify the target argument (the thing it must match)
        let target_name = match arg {
            AstNode::Identifier(name) => name,
            _ => return Err(VMError::InvalidInstruction), // Must be identifier
        };

        // Find target parameter index
        let (target_idx, target_param) = all_parameters.iter().enumerate()
            .find(|(_, p)| p.name == *target_name)
            .ok_or(VMError::InvalidScript)?; // Target not found

        // 2. Identify the field in the account
        // By convention, @has(x) checks account.x == x
        // The field name is same as target name
        let field_name = target_name;

        // Get field offset from registry
        let type_name = match &account_param.param_type {
            TypeNode::Named(name) => name,
            _ => continue, // generic Account has no fields? Skip for now.
        };

        let field_offset = if let Some(acct_info) = account_registry.account_types.get(type_name) {
            if let Some(field) = acct_info.fields.get(field_name) {
                field.offset
            } else {
                return Err(VMError::InvalidScript); // Field not found in account
            }
        } else {
            return Err(VMError::InvalidScript); // Account type not found
        };

        // 3. Emit Verification Bytecode
        
        // A. Load Account Field -> Stack
        emitter.emit_opcode(LOAD_FIELD);
        emitter.emit_u8(account_idx);
        emitter.emit_vle_u32(field_offset);

        // B. Load Target Argument -> Stack
        // If target is an Account, we compare KEYS.
        // If target is a Value (u64, etc), we compare VALUES.
        if is_account_param(target_param, account_registry) {
            // Get Key of target account
            let target_acct_idx = account_index_from_param_index(target_idx as u8);
            emitter.emit_opcode(GET_KEY);
            emitter.emit_u8(target_acct_idx);
        } else {
            // Load local variable (param value)
            emitter.emit_opcode(GET_LOCAL);
            emitter.emit_u8(target_idx as u8);
        }

        // C. Compare and Require
        emitter.emit_opcode(EQ);
        emitter.emit_opcode(REQUIRE);
    }

    Ok(())
}


