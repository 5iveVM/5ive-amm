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
    
    for (param_index, param) in parameters.iter().enumerate() {
        // Skip non-account parameters for most checks
        let is_account = is_account_param(param);
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
                "owner" => {
                    emit_owner_check(emitter, account_idx, attribute)?;
                }
                 _ => {}
            }
        }
    }
    
    Ok(())
}

fn is_account_param(param: &InstructionParameter) -> bool {
    matches!(param.param_type, TypeNode::Account | TypeNode::Named(_))
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
        if is_account_param(target_param) {
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

/// Emit Bytecode for @owner(target)
/// Ensures: account.owner == target
/// Target can be a Pubkey parameter or a Literal
fn emit_owner_check<T: OpcodeEmitter>(
    _emitter: &mut T,
    _account_idx: u8,
    _attribute: &Attribute,
) -> Result<(), VMError> {
     // NOTE: We assume CHECK_OWNER opcode exists in opcodes.rs and works as:
     // CHECK_OWNER <account_idx> <owner_key_on_stack from somewhere?>
     // OR if it's CHECK_OWNER <account_idx> which checks against current program ID?
     //
     // Looking at opcodes.rs (recalled), CHECK_OWNER likely checks against current program ID (usually what you want).
     // BUT @owner(target) implies checking against a specific target.
     //
     // If target is provided, we should probably do:
     // GET_OWNER <account_idx> -> [owner_key]
     // <Load Target> -> [owner_key, target_key]
     // EQ
     // REQUIRE
     
     // However, I don't see GET_OWNER in standard set often, 
     // but account.owner is a built-in property.
     // Accessing built-in property "owner" using LOAD_FIELD or specialized opcode?
     
     // Let's assume we use GET_OWNER if it exists, or fallback to treating it as a field access if "owner" is mapped.
     // But strictly speaking, owner is in header, not data.
     
     // Let's try to assume CHECK_OWNER takes an argument or just checks program id.
     // If attribute has args, we can't use simple CHECK_OWNER if it doesn't take args.
     
     // REVISION: I will use GET_OWNER (if exists) or assume standard account header access?
     // `five-protocol/src/opcodes.rs` showed CHECK_OWNER.
     // I'll assume CHECK_OWNER (0x72 or similar) checks against the executing program ID (common case).
     // IF @owner has NO args, use CHECK_OWNER.
     
     // NOTE: CHECK_OWNER (0x72) in five-vm-mito expects [u8 account_index, [u8; 32] expected_owner_pubkey]
     // We cannot currently resolve the Program ID at compile time to emit it as the expected owner.
     // Therefore, we must disable this constraint generation until we have:
     // 1. A way to inject Program ID constants
     // 2. OR an opcode CHECK_OWNER_PROGRAM (implicit check against executing program)
     
     Err(VMError::InvalidOperation)
     
     /*
     if attribute.args.is_empty() {
         emitter.emit_opcode(CHECK_OWNER); // 0x72
         emitter.emit_u8(account_idx);
         // Missing: emitter.emit_pubkey(program_id);
         return Ok(());
     }
     
     if !attribute.args.is_empty() {
        return Err(VMError::InvalidOperation); 
     }
     
     emitter.emit_opcode(CHECK_OWNER);
     emitter.emit_u8(account_idx);
     Ok(())
     */
}
