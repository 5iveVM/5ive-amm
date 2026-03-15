use super::OpcodeEmitter;
use crate::ast::{AstNode, Attribute, InstructionParameter, TypeNode};
use crate::bytecode_generator::account_utils::account_index_from_param_index;
use crate::bytecode_generator::types::AccountRegistry;
use five_protocol::opcodes::*;
use five_protocol::Value;
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

        let mut has_signer = false;
        let mut has_writable = false;
        // Process attributes
        for attribute in &param.attributes {
            match attribute.name.as_str() {
                "signer" => {
                    has_signer = true;
                }
                "mut" | "writable" => {
                    has_writable = true;
                }
                "has" => {
                    emit_has_check(
                        emitter,
                        account_idx,
                        param,
                        attribute,
                        parameters,
                        account_registry,
                    )?;
                }
                "session" => {
                    emit_session_check(
                        emitter,
                        account_idx,
                        param,
                        attribute,
                        parameters,
                        account_registry,
                    )?;
                }

                _ => {}
            }
        }
        if has_signer && has_writable {
            emitter.emit_opcode(CHECK_SIGNER_WRITABLE);
            emitter.emit_u8(account_idx);
        } else {
            if has_signer {
                emit_signer_check(emitter, account_idx)?;
            }
            if has_writable {
                emit_writable_check(emitter, account_idx)?;
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
                let payer_param_index = parameters
                    .iter()
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
            // Check registry and built-in account type names.

            if matches!(name.as_str(), "Account" | "TokenAccount" | "ProgramAccount") {
                return true;
            }

            // Check registry
            let namespace_suffix = format!("::{}", name);
            account_registry.account_types.contains_key(name)
                || account_registry
                    .account_types
                    .keys()
                    .any(|k| k.ends_with(&namespace_suffix))
        }
        _ => false,
    }
}

/// Emit CHECK_SIGNER opcode
fn emit_signer_check<T: OpcodeEmitter>(emitter: &mut T, account_idx: u8) -> Result<(), VMError> {
    emitter.emit_opcode(CHECK_SIGNER);
    emitter.emit_u8(account_idx);
    Ok(())
}

/// Emit CHECK_WRITABLE opcode
fn emit_writable_check<T: OpcodeEmitter>(emitter: &mut T, account_idx: u8) -> Result<(), VMError> {
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
        let (target_idx, target_param) = all_parameters
            .iter()
            .enumerate()
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
        emitter.emit_u32(field_offset);

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

fn resolve_parameter<'a>(
    all_parameters: &'a [InstructionParameter],
    name: &str,
) -> Result<(usize, &'a InstructionParameter), VMError> {
    all_parameters
        .iter()
        .enumerate()
        .find(|(_, p)| p.name == name)
        .ok_or(VMError::InvalidScript)
}

fn resolve_account_param_index(
    all_parameters: &[InstructionParameter],
    param_idx: usize,
    account_registry: &AccountRegistry,
) -> Result<u8, VMError> {
    if !is_account_param(&all_parameters[param_idx], account_registry) {
        return Err(VMError::TypeMismatch);
    }

    let mut account_ordinal: u8 = 0;
    for (i, param) in all_parameters.iter().enumerate() {
        if is_account_param(param, account_registry) {
            if i == param_idx {
                return Ok(account_index_from_param_index(account_ordinal));
            }
            account_ordinal = account_ordinal.saturating_add(1);
        }
    }

    Err(VMError::InvalidScript)
}

fn session_field_offset(
    account_param: &InstructionParameter,
    account_registry: &AccountRegistry,
    field_name: &str,
) -> Result<u32, VMError> {
    let type_name = match &account_param.param_type {
        TypeNode::Named(name) => name,
        _ => return Err(VMError::InvalidScript),
    };

    let Some(acct_info) = account_registry.account_types.get(type_name) else {
        return Err(VMError::InvalidScript);
    };
    let Some(field) = acct_info.fields.get(field_name) else {
        return Err(VMError::InvalidScript);
    };
    Ok(field.offset)
}

fn emit_field_eq_param<T: OpcodeEmitter>(
    emitter: &mut T,
    session_account_idx: u8,
    field_offset: u32,
    target_idx: usize,
    target_param: &InstructionParameter,
    all_parameters: &[InstructionParameter],
    account_registry: &AccountRegistry,
) -> Result<(), VMError> {
    if is_account_param(target_param, account_registry) {
        emitter.emit_opcode(REQUIRE_OWNER);
        emitter.emit_u8(session_account_idx);
        emitter.emit_u8(resolve_account_param_index(
            all_parameters,
            target_idx,
            account_registry,
        )?);
        emitter.emit_u32(field_offset);
        return Ok(());
    }

    emitter.emit_opcode(LOAD_FIELD);
    emitter.emit_u8(session_account_idx);
    emitter.emit_u32(field_offset);
    emitter.emit_opcode(GET_LOCAL);
    emitter.emit_u8(target_idx as u8);
    emitter.emit_opcode(EQ);
    emitter.emit_opcode(REQUIRE);
    Ok(())
}

fn emit_session_check<T: OpcodeEmitter>(
    emitter: &mut T,
    session_account_idx: u8,
    session_param: &InstructionParameter,
    attribute: &Attribute,
    all_parameters: &[InstructionParameter],
    account_registry: &AccountRegistry,
) -> Result<(), VMError> {
    fn session_arg<'a>(attribute: &'a Attribute, key: &str, pos: usize) -> Option<&'a AstNode> {
        let mut has_keyed_args = false;
        for arg in &attribute.args {
            if let AstNode::Assignment { target, value } = arg {
                has_keyed_args = true;
                if target == key {
                    return Some(value.as_ref());
                }
            }
        }
        if has_keyed_args {
            return None;
        }
        attribute.args.get(pos)
    }

    // Format:
    // positional: @session(delegate, authority, target_program?, scope_hash?, bind_account?, nonce?, current_slot?, manager_script_account?, manager_code_hash?, manager_version?)
    // keyed:      @session(delegate=..., authority=..., target_program=..., scope_hash=..., ...)
    let delegate_name = match session_arg(attribute, "delegate", 0) {
        Some(AstNode::Identifier(name)) => name,
        _ => return Err(VMError::InvalidInstruction),
    };
    let authority_name = match session_arg(attribute, "authority", 1) {
        Some(AstNode::Identifier(name)) => name,
        _ => return Err(VMError::InvalidInstruction),
    };

    let (delegate_idx, delegate_param) = resolve_parameter(all_parameters, delegate_name)?;
    let (authority_idx, authority_param) = resolve_parameter(all_parameters, authority_name)?;

    if !is_account_param(delegate_param, account_registry)
        || !is_account_param(authority_param, account_registry)
    {
        return Err(VMError::TypeMismatch);
    }

    // Optional direct-owner path:
    // if __session account key == authority account key, skip session-sidecar checks.
    let authority_account_idx =
        resolve_account_param_index(all_parameters, authority_idx, account_registry)?;
    emitter.emit_opcode(GET_KEY);
    emitter.emit_u8(session_account_idx);
    emitter.emit_opcode(GET_KEY);
    emitter.emit_u8(authority_account_idx);
    emitter.emit_opcode(EQ);
    emitter.emit_opcode(JUMP_IF);
    let bypass_patch_pos = emitter.get_position();
    emitter.emit_u16(0);

    // Optional static "active" gate if field exists.
    if let Ok(status_offset) = session_field_offset(session_param, account_registry, "status") {
        emitter.emit_opcode(REQUIRE_BATCH);
        emitter.emit_u8(1); // 1 clause
        emitter.emit_u8(REQUIRE_BATCH_FIELD_EQ_IMM);
        emitter.emit_u8(session_account_idx);
        emitter.emit_u32(status_offset);
        emitter.emit_u8(1); // active
    }

    // Optional static session schema version gate.
    if let Ok(version_offset) = session_field_offset(session_param, account_registry, "version") {
        emitter.emit_opcode(REQUIRE_BATCH);
        emitter.emit_u8(1); // 1 clause
        emitter.emit_u8(REQUIRE_BATCH_FIELD_EQ_IMM);
        emitter.emit_u8(session_account_idx);
        emitter.emit_u32(version_offset);
        emitter.emit_u8(1); // v1
    }

    // delegate field == delegate account key
    if let Ok(offset) = session_field_offset(session_param, account_registry, "delegate") {
        emit_field_eq_param(
            emitter,
            session_account_idx,
            offset,
            delegate_idx,
            delegate_param,
            all_parameters,
            account_registry,
        )?;
    }

    // authority field == authority account key
    if let Ok(offset) = session_field_offset(session_param, account_registry, "authority") {
        emit_field_eq_param(
            emitter,
            session_account_idx,
            offset,
            authority_idx,
            authority_param,
            all_parameters,
            account_registry,
        )?;
    }

    // Optional args map by position.
    // target_program field == param
    if let Some(AstNode::Identifier(name)) = session_arg(attribute, "target_program", 2) {
        let (idx, target_param) = resolve_parameter(all_parameters, name)?;
        if let Ok(offset) = session_field_offset(session_param, account_registry, "target_program") {
            emit_field_eq_param(
                emitter,
                session_account_idx,
                offset,
                idx,
                target_param,
                all_parameters,
                account_registry,
            )?;
        }
    }

    // manager_script_account field == param
    if let Some(AstNode::Identifier(name)) = session_arg(attribute, "manager_script_account", 7)
        .or_else(|| session_arg(attribute, "manager_script", 7))
    {
        let (idx, target_param) = resolve_parameter(all_parameters, name)?;
        if let Ok(offset) =
            session_field_offset(session_param, account_registry, "manager_script_account")
        {
            emit_field_eq_param(
                emitter,
                session_account_idx,
                offset,
                idx,
                target_param,
                all_parameters,
                account_registry,
            )?;
        }
    }

    // manager_code_hash field == param
    if let Some(AstNode::Identifier(name)) = session_arg(attribute, "manager_code_hash", 8)
        .or_else(|| session_arg(attribute, "manager_hash", 8))
    {
        let (idx, target_param) = resolve_parameter(all_parameters, name)?;
        if let Ok(offset) = session_field_offset(session_param, account_registry, "manager_code_hash")
        {
            emit_field_eq_param(
                emitter,
                session_account_idx,
                offset,
                idx,
                target_param,
                all_parameters,
                account_registry,
            )?;
        }
    }

    // manager_version field == param or numeric literal
    if let Some(arg) = session_arg(attribute, "manager_version", 9) {
        if let Ok(offset) = session_field_offset(session_param, account_registry, "manager_version") {
            match arg {
                AstNode::Identifier(name) => {
                    let (idx, target_param) = resolve_parameter(all_parameters, name)?;
                    emit_field_eq_param(
                        emitter,
                        session_account_idx,
                        offset,
                        idx,
                        target_param,
                        all_parameters,
                        account_registry,
                    )?;
                }
                AstNode::Literal(Value::U8(value)) => {
                    emitter.emit_opcode(REQUIRE_BATCH);
                    emitter.emit_u8(1); // 1 clause
                    emitter.emit_u8(REQUIRE_BATCH_FIELD_EQ_IMM);
                    emitter.emit_u8(session_account_idx);
                    emitter.emit_u32(offset);
                    emitter.emit_u8(*value);
                }
                AstNode::Literal(Value::U64(value)) => {
                    if *value > u8::MAX as u64 {
                        return Err(VMError::InvalidInstruction);
                    }
                    emitter.emit_opcode(REQUIRE_BATCH);
                    emitter.emit_u8(1); // 1 clause
                    emitter.emit_u8(REQUIRE_BATCH_FIELD_EQ_IMM);
                    emitter.emit_u8(session_account_idx);
                    emitter.emit_u32(offset);
                    emitter.emit_u8(*value as u8);
                }
                _ => return Err(VMError::InvalidInstruction),
            }
        }
    }

    // scope_hash field == param
    if let Some(AstNode::Identifier(name)) = session_arg(attribute, "scope_hash", 3) {
        let (idx, target_param) = resolve_parameter(all_parameters, name)?;
        if let Ok(offset) = session_field_offset(session_param, account_registry, "scope_hash") {
            emit_field_eq_param(
                emitter,
                session_account_idx,
                offset,
                idx,
                target_param,
                all_parameters,
                account_registry,
            )?;
        }
    }

    // bind_account field == param (if provided)
    if let Some(AstNode::Identifier(name)) = session_arg(attribute, "bind_account", 4) {
        let (idx, target_param) = resolve_parameter(all_parameters, name)?;
        if let Ok(offset) = session_field_offset(session_param, account_registry, "bind_account") {
            emit_field_eq_param(
                emitter,
                session_account_idx,
                offset,
                idx,
                target_param,
                all_parameters,
                account_registry,
            )?;
        }
    }

    // nonce field == param
    if let Some(AstNode::Identifier(name)) = session_arg(attribute, "nonce", 5)
        .or_else(|| session_arg(attribute, "nonce_field", 5))
    {
        let (idx, target_param) = resolve_parameter(all_parameters, name)?;
        if let Ok(offset) = session_field_offset(session_param, account_registry, "nonce") {
            emit_field_eq_param(
                emitter,
                session_account_idx,
                offset,
                idx,
                target_param,
                all_parameters,
                account_registry,
            )?;
        }
    }

    // expires_at_slot >= current_slot (if provided)
    if let Some(AstNode::Identifier(name)) = session_arg(attribute, "current_slot", 6) {
        let (slot_idx, _) = resolve_parameter(all_parameters, name)?;
        if let Ok(offset) = session_field_offset(session_param, account_registry, "expires_at_slot")
        {
            emitter.emit_opcode(REQUIRE_BATCH);
            emitter.emit_u8(1); // 1 clause
            emitter.emit_u8(REQUIRE_BATCH_FIELD_GTE_PARAM);
            emitter.emit_u8(session_account_idx);
            emitter.emit_u32(offset);
            emitter.emit_u8(slot_idx as u8);
        }
    }

    let bypass_target = emitter.get_position();
    emitter.patch_u16(bypass_patch_pos, bypass_target as u16);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Attribute, InstructionParameter, TypeNode};
    use crate::bytecode_generator::types::{AccountRegistry, AccountTypeInfo, FieldInfo};
    use std::collections::HashMap;

    #[derive(Default)]
    struct TestEmitter {
        bytes: Vec<u8>,
    }

    impl OpcodeEmitter for TestEmitter {
        fn emit_opcode(&mut self, opcode: u8) {
            self.bytes.push(opcode);
        }
        fn emit_u8(&mut self, value: u8) {
            self.bytes.push(value);
        }
        fn emit_u16(&mut self, value: u16) {
            self.bytes.extend_from_slice(&value.to_le_bytes());
        }
        fn emit_u32(&mut self, value: u32) {
            self.bytes.extend_from_slice(&value.to_le_bytes());
        }
        fn emit_u64(&mut self, value: u64) {
            self.bytes.extend_from_slice(&value.to_le_bytes());
        }
        fn emit_bytes(&mut self, bytes: &[u8]) {
            self.bytes.extend_from_slice(bytes);
        }
        fn get_position(&self) -> usize {
            self.bytes.len()
        }
        fn patch_u32(&mut self, position: usize, value: u32) {
            self.bytes[position..position + 4].copy_from_slice(&value.to_le_bytes());
        }
        fn patch_u16(&mut self, position: usize, value: u16) {
            self.bytes[position..position + 2].copy_from_slice(&value.to_le_bytes());
        }
        fn should_include_tests(&self) -> bool {
            true
        }
        fn emit_const_u8(&mut self, value: u8) -> Result<(), VMError> {
            self.emit_u8(value);
            Ok(())
        }
        fn emit_const_u16(&mut self, value: u16) -> Result<(), VMError> {
            self.emit_u16(value);
            Ok(())
        }
        fn emit_const_u32(&mut self, value: u32) -> Result<(), VMError> {
            self.emit_u32(value);
            Ok(())
        }
        fn emit_const_u64(&mut self, value: u64) -> Result<(), VMError> {
            self.emit_u64(value);
            Ok(())
        }
        fn emit_const_i64(&mut self, value: i64) -> Result<(), VMError> {
            self.emit_u64(value as u64);
            Ok(())
        }
        fn emit_const_bool(&mut self, value: bool) -> Result<(), VMError> {
            self.emit_u8(if value { 1 } else { 0 });
            Ok(())
        }
        fn emit_const_u128(&mut self, _value: u128) -> Result<(), VMError> {
            Ok(())
        }
        fn emit_const_pubkey(&mut self, _value: &[u8; 32]) -> Result<(), VMError> {
            Ok(())
        }
        fn emit_const_string(&mut self, _value: &[u8]) -> Result<(), VMError> {
            Ok(())
        }
        fn intern_u16_const(&mut self, value: u16) -> Result<u16, VMError> {
            Ok(value)
        }
    }

    fn mk_param(name: &str, ty: TypeNode, attributes: Vec<Attribute>) -> InstructionParameter {
        InstructionParameter {
            name: name.to_string(),
            param_type: ty,
            is_optional: false,
            default_value: None,
            attributes,
            is_init: false,
            init_config: None,
            serializer: None,
            pda_config: None,
        }
    }

    #[test]
    fn session_constraint_emits_owner_checks_without_delegate_signer_requirement() {
        let mut registry = AccountRegistry::new();
        let mut fields = HashMap::new();
        fields.insert(
            "status".to_string(),
            FieldInfo {
                offset: 0,
                field_type: "u8".to_string(),
                is_mutable: true,
                is_optional: false,
                is_parameter: false,
            },
        );
        fields.insert(
            "delegate".to_string(),
            FieldInfo {
                offset: 1,
                field_type: "pubkey".to_string(),
                is_mutable: false,
                is_optional: false,
                is_parameter: false,
            },
        );
        fields.insert(
            "authority".to_string(),
            FieldInfo {
                offset: 33,
                field_type: "pubkey".to_string(),
                is_mutable: false,
                is_optional: false,
                is_parameter: false,
            },
        );
        registry.account_types.insert(
            "Session".to_string(),
            AccountTypeInfo {
                name: "Session".to_string(),
                fields,
                total_size: 96,
                serializer: None,
            },
        );

        let params = vec![
            mk_param(
                "session",
                TypeNode::Named("Session".to_string()),
                vec![Attribute {
                    name: "session".to_string(),
                    args: vec![
                        AstNode::Identifier("delegate".to_string()),
                        AstNode::Identifier("authority".to_string()),
                    ],
                }],
            ),
            mk_param("delegate", TypeNode::Account, vec![]),
            mk_param("authority", TypeNode::Account, vec![]),
        ];

        let mut emitter = TestEmitter::default();
        emit_constraint_checks(&mut emitter, &params, &registry).unwrap();

        assert!(emitter.bytes.contains(&JUMP_IF));
        assert!(!emitter.bytes.contains(&CHECK_SIGNER));
        assert!(emitter.bytes.contains(&REQUIRE_OWNER));
        assert!(emitter.bytes.contains(&REQUIRE_BATCH));
    }

    #[test]
    fn session_constraint_accepts_keyed_args() {
        let mut registry = AccountRegistry::new();
        let mut fields = HashMap::new();
        fields.insert(
            "status".to_string(),
            FieldInfo {
                offset: 0,
                field_type: "u8".to_string(),
                is_mutable: true,
                is_optional: false,
                is_parameter: false,
            },
        );
        fields.insert(
            "delegate".to_string(),
            FieldInfo {
                offset: 1,
                field_type: "pubkey".to_string(),
                is_mutable: false,
                is_optional: false,
                is_parameter: false,
            },
        );
        fields.insert(
            "authority".to_string(),
            FieldInfo {
                offset: 33,
                field_type: "pubkey".to_string(),
                is_mutable: false,
                is_optional: false,
                is_parameter: false,
            },
        );
        registry.account_types.insert(
            "Session".to_string(),
            AccountTypeInfo {
                name: "Session".to_string(),
                fields,
                total_size: 96,
                serializer: None,
            },
        );

        let params = vec![
            mk_param(
                "session",
                TypeNode::Named("Session".to_string()),
                vec![Attribute {
                    name: "session".to_string(),
                    args: vec![
                        AstNode::Assignment {
                            target: "delegate".to_string(),
                            value: Box::new(AstNode::Identifier("delegate".to_string())),
                        },
                        AstNode::Assignment {
                            target: "authority".to_string(),
                            value: Box::new(AstNode::Identifier("authority".to_string())),
                        },
                    ],
                }],
            ),
            mk_param("delegate", TypeNode::Account, vec![]),
            mk_param("authority", TypeNode::Account, vec![]),
        ];

        let mut emitter = TestEmitter::default();
        emit_constraint_checks(&mut emitter, &params, &registry).unwrap();

        assert!(!emitter.bytes.contains(&CHECK_SIGNER));
        assert!(emitter.bytes.contains(&REQUIRE_OWNER));
    }

    #[test]
    fn session_constraint_uses_account_ordinal_not_param_index() {
        let mut registry = AccountRegistry::new();
        let mut fields = HashMap::new();
        fields.insert(
            "status".to_string(),
            FieldInfo {
                offset: 0,
                field_type: "u8".to_string(),
                is_mutable: true,
                is_optional: false,
                is_parameter: false,
            },
        );
        fields.insert(
            "delegate".to_string(),
            FieldInfo {
                offset: 1,
                field_type: "pubkey".to_string(),
                is_mutable: false,
                is_optional: false,
                is_parameter: false,
            },
        );
        fields.insert(
            "authority".to_string(),
            FieldInfo {
                offset: 33,
                field_type: "pubkey".to_string(),
                is_mutable: false,
                is_optional: false,
                is_parameter: false,
            },
        );
        registry.account_types.insert(
            "Session".to_string(),
            AccountTypeInfo {
                name: "Session".to_string(),
                fields,
                total_size: 96,
                serializer: None,
            },
        );

        // Mixed account/data ordering: delegate is param index 3 but account ordinal 2.
        let params = vec![
            mk_param(
                "session",
                TypeNode::Named("Session".to_string()),
                vec![Attribute {
                    name: "session".to_string(),
                    args: vec![
                        AstNode::Identifier("delegate".to_string()),
                        AstNode::Identifier("authority".to_string()),
                    ],
                }],
            ),
            mk_param("authority", TypeNode::Account, vec![]),
            mk_param("target_program", TypeNode::Named("pubkey".to_string()), vec![]),
            mk_param("delegate", TypeNode::Account, vec![]),
        ];

        let mut emitter = TestEmitter::default();
        emit_constraint_checks(&mut emitter, &params, &registry).unwrap();

        assert!(!emitter.bytes.contains(&CHECK_SIGNER));
        assert!(emitter.bytes.contains(&REQUIRE_OWNER));
    }

    #[test]
    fn session_constraint_emits_manager_provenance_checks() {
        let mut registry = AccountRegistry::new();
        let mut fields = HashMap::new();
        fields.insert(
            "status".to_string(),
            FieldInfo { offset: 0, field_type: "u8".to_string(), is_mutable: true, is_optional: false, is_parameter: false },
        );
        fields.insert(
            "version".to_string(),
            FieldInfo { offset: 1, field_type: "u8".to_string(), is_mutable: false, is_optional: false, is_parameter: false },
        );
        fields.insert(
            "delegate".to_string(),
            FieldInfo { offset: 2, field_type: "pubkey".to_string(), is_mutable: false, is_optional: false, is_parameter: false },
        );
        fields.insert(
            "authority".to_string(),
            FieldInfo { offset: 34, field_type: "pubkey".to_string(), is_mutable: false, is_optional: false, is_parameter: false },
        );
        fields.insert(
            "manager_script_account".to_string(),
            FieldInfo { offset: 66, field_type: "pubkey".to_string(), is_mutable: false, is_optional: false, is_parameter: false },
        );
        fields.insert(
            "manager_code_hash".to_string(),
            FieldInfo { offset: 98, field_type: "pubkey".to_string(), is_mutable: false, is_optional: false, is_parameter: false },
        );
        fields.insert(
            "manager_version".to_string(),
            FieldInfo { offset: 130, field_type: "u8".to_string(), is_mutable: false, is_optional: false, is_parameter: false },
        );
        registry.account_types.insert(
            "Session".to_string(),
            AccountTypeInfo { name: "Session".to_string(), fields, total_size: 160, serializer: None },
        );

        let params = vec![
            mk_param(
                "session",
                TypeNode::Named("Session".to_string()),
                vec![Attribute {
                    name: "session".to_string(),
                    args: vec![
                        AstNode::Assignment {
                            target: "delegate".to_string(),
                            value: Box::new(AstNode::Identifier("delegate".to_string())),
                        },
                        AstNode::Assignment {
                            target: "authority".to_string(),
                            value: Box::new(AstNode::Identifier("authority".to_string())),
                        },
                        AstNode::Assignment {
                            target: "manager_script_account".to_string(),
                            value: Box::new(AstNode::Identifier("manager_script".to_string())),
                        },
                        AstNode::Assignment {
                            target: "manager_code_hash".to_string(),
                            value: Box::new(AstNode::Identifier("manager_hash".to_string())),
                        },
                        AstNode::Assignment {
                            target: "manager_version".to_string(),
                            value: Box::new(AstNode::Literal(Value::U8(1))),
                        },
                    ],
                }],
            ),
            mk_param("delegate", TypeNode::Account, vec![]),
            mk_param("authority", TypeNode::Account, vec![]),
            mk_param("manager_script", TypeNode::Account, vec![]),
            mk_param("manager_hash", TypeNode::Account, vec![]),
        ];

        let mut emitter = TestEmitter::default();
        emit_constraint_checks(&mut emitter, &params, &registry).unwrap();

        let require_owner_count = emitter
            .bytes
            .iter()
            .filter(|b| **b == REQUIRE_OWNER)
            .count();
        assert!(
            require_owner_count >= 4,
            "expected manager provenance pubkey checks to emit REQUIRE_OWNER; got {}",
            require_owner_count
        );

        // status + version + manager_version should each contribute a REQUIRE_BATCH opcode.
        let require_batch_count = emitter
            .bytes
            .iter()
            .filter(|b| **b == REQUIRE_BATCH)
            .count();
        assert!(
            require_batch_count >= 3,
            "expected status/version/manager_version batch checks; got {}",
            require_batch_count
        );
    }
}
