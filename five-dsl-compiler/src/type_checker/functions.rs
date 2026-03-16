// Function and interface type checking

use super::type_helpers::type_names;
use super::types::{InterfaceInfo, InterfaceMethod, InterfaceSerializer, TypeCheckerContext};
use crate::ast::{AstNode, InstructionParameter, TypeNode};
use crate::session_support;
use five_vm_mito::error::VMError;
use sha2::Digest;
use std::collections::{HashMap, HashSet};

impl TypeCheckerContext {
    fn session_attr_value<'a>(
        attr: &'a crate::ast::Attribute,
        key: &str,
        positional_index: usize,
    ) -> Option<&'a AstNode> {
        let mut has_keyed_args = false;
        for arg in &attr.args {
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
        attr.args.get(positional_index)
    }

    fn param_has_attribute(param: &InstructionParameter, attr_name: &str) -> bool {
        param.attributes.iter().any(|attr| attr.name == attr_name)
    }

    fn is_account_param_type(&self, type_node: &TypeNode) -> bool {
        match type_node {
            TypeNode::Account => true,
            TypeNode::Named(name) => self.is_named_account_type_name(name) || name.contains("::"),
            _ => false,
        }
    }

    fn contains_close_call_for_source(node: &AstNode, source_name: &str) -> bool {
        match node {
            AstNode::FunctionCall { name, args } => {
                if name == "close_account" {
                    if let Some(AstNode::Identifier(first_arg)) = args.first() {
                        if first_arg == source_name {
                            return true;
                        }
                    }
                }
                args.iter()
                    .any(|arg| Self::contains_close_call_for_source(arg, source_name))
            }
            AstNode::Block { statements, .. } => statements
                .iter()
                .any(|stmt| Self::contains_close_call_for_source(stmt, source_name)),
            AstNode::IfStatement {
                condition,
                then_branch,
                else_branch,
            } => {
                Self::contains_close_call_for_source(condition, source_name)
                    || Self::contains_close_call_for_source(then_branch, source_name)
                    || else_branch
                        .as_ref()
                        .map(|n| Self::contains_close_call_for_source(n, source_name))
                        .unwrap_or(false)
            }
            AstNode::ForLoop {
                init,
                condition,
                update,
                body,
            } => {
                init.as_ref()
                    .map(|n| Self::contains_close_call_for_source(n, source_name))
                    .unwrap_or(false)
                    || condition
                        .as_ref()
                        .map(|n| Self::contains_close_call_for_source(n, source_name))
                        .unwrap_or(false)
                    || update
                        .as_ref()
                        .map(|n| Self::contains_close_call_for_source(n, source_name))
                        .unwrap_or(false)
                    || Self::contains_close_call_for_source(body, source_name)
            }
            AstNode::ForInLoop { iterable, body, .. } => {
                Self::contains_close_call_for_source(iterable, source_name)
                    || Self::contains_close_call_for_source(body, source_name)
            }
            AstNode::ForOfLoop { iterable, body, .. } => {
                Self::contains_close_call_for_source(iterable, source_name)
                    || Self::contains_close_call_for_source(body, source_name)
            }
            AstNode::WhileLoop { condition, body } => {
                Self::contains_close_call_for_source(condition, source_name)
                    || Self::contains_close_call_for_source(body, source_name)
            }
            AstNode::DoWhileLoop { body, condition } => {
                Self::contains_close_call_for_source(body, source_name)
                    || Self::contains_close_call_for_source(condition, source_name)
            }
            AstNode::SwitchStatement {
                discriminant,
                cases,
                default_case,
            } => {
                Self::contains_close_call_for_source(discriminant, source_name)
                    || cases.iter().any(|case| {
                        Self::contains_close_call_for_source(&case.pattern, source_name)
                            || case
                                .body
                                .iter()
                                .any(|stmt| Self::contains_close_call_for_source(stmt, source_name))
                    })
                    || default_case
                        .as_ref()
                        .map(|n| Self::contains_close_call_for_source(n, source_name))
                        .unwrap_or(false)
            }
            AstNode::ReturnStatement { value } => value
                .as_ref()
                .map(|n| Self::contains_close_call_for_source(n, source_name))
                .unwrap_or(false),
            AstNode::RequireStatement { condition } => {
                Self::contains_close_call_for_source(condition, source_name)
            }
            AstNode::MatchExpression { expression, arms } => {
                Self::contains_close_call_for_source(expression, source_name)
                    || arms.iter().any(|arm| {
                        Self::contains_close_call_for_source(&arm.pattern, source_name)
                            || arm
                                .guard
                                .as_ref()
                                .map(|g| Self::contains_close_call_for_source(g, source_name))
                                .unwrap_or(false)
                            || Self::contains_close_call_for_source(&arm.body, source_name)
                    })
            }
            _ => false,
        }
    }

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
                is_anchor: is_interface_anchor,
                functions,
            } = interface_def
            {
                let mut methods = HashMap::new();
                let serializer_hint = serializer.clone();

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
                            if Self::param_has_attribute(param, "authority")
                                && !self.is_account_param_type(&param.param_type)
                            {
                                return Err(VMError::TypeMismatch);
                            }
                            if param.serializer.is_some()
                                && !self.is_account_param_type(&param.param_type)
                            {
                                return Err(VMError::TypeMismatch);
                            }
                        }

                        let return_type_node = return_type.as_ref().map(|rt| (**rt).clone());

                        // Determine discriminator (duplicate logic from registry for consistency)
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

                let has_anchor_methods = methods.values().any(|m: &InterfaceMethod| m.is_anchor);
                let anchor_mode = *is_interface_anchor || has_anchor_methods;

                let interface_info = InterfaceInfo {
                    program_id: program_id.clone().unwrap_or_default(), // Default to empty if no program ID
                    serializer: match serializer_hint.as_deref() {
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
                    },
                    is_anchor: anchor_mode,
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
                let expected_param = &method_info.parameters[i];
                let expected_type = &expected_param.param_type;
                if !self.argument_matches_expected_type(arg, expected_type)? {
                    return Err(VMError::TypeMismatch);
                }

                if Self::param_has_attribute(expected_param, "authority") {
                    if !self.is_account_param_type(expected_type) {
                        return Err(VMError::TypeMismatch);
                    }

                    let AstNode::Identifier(arg_name) = arg else {
                        return Err(VMError::TypeMismatch);
                    };

                    let Some(current_params) = self.symbol_table.get(arg_name) else {
                        return Err(VMError::InvalidScript);
                    };

                    if !current_params.0.is_account_type() {
                        return Err(VMError::TypeMismatch);
                    }

                    let signable = self
                        .current_function_parameters
                        .as_ref()
                        .and_then(|params| params.iter().find(|param| param.name == *arg_name))
                        .map(|param| {
                            Self::param_has_attribute(param, "signer") || param.pda_config.is_some()
                        })
                        .unwrap_or(false);

                    if !signable {
                        return Err(VMError::ConstraintViolation);
                    }
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
        let previous_function_parameters = self.current_function_parameters.clone();
        let previous_init_bump_accounts = self.init_bump_accounts.clone();
        let previous_init_space_accounts = self.init_space_accounts.clone();
        // Keep global fields, but parameters can shadow them
        self.current_function = Some(name.to_string());
        self.current_function_parameters = Some(parameters.to_vec());
        self.init_bump_accounts.clear();
        self.init_space_accounts.clear();

        // Check parameter types are valid and add to symbol table
        // Also capture which account parameters are marked @mut
        let mut writable_accounts: HashSet<String> = HashSet::new();
        for param in parameters {
            if !self.is_valid_type_node(&param.param_type) {
                eprintln!(
                    "Invalid param type: {} ({:?})",
                    param.name, param.param_type
                );
                return Err(VMError::InvalidScript);
            }

            // Validate @init constraints
            // Validate @init constraints
            if param.is_init {
                // @init can only be applied to Account types (built-in or user-defined)
                let is_valid_account = match &param.param_type {
                    crate::ast::TypeNode::Account => true,
                    crate::ast::TypeNode::Named(name) => {
                        if self.is_named_account_type_name(name) {
                            true
                        } else {
                            // Check module scope for imported accounts
                            if let Some(scope) = &self.module_scope {
                                if let Some(symbol) =
                                    scope.resolve_symbol(name, scope.current_module())
                                {
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
                                eprintln!(
                                    "@init payer '{}' not found in function parameters",
                                    payer_name
                                );
                                return Err(VMError::InvalidScript);
                            }
                            Some(payer) => {
                                // Validate payer is account type
                                if !matches!(
                                    payer.param_type,
                                    crate::ast::TypeNode::Account | crate::ast::TypeNode::Named(_)
                                ) {
                                    eprintln!(
                                        "@init payer '{}' must be an account type",
                                        payer_name
                                    );
                                    return Err(VMError::TypeMismatch);
                                }

                                // Validate payer has @signer
                                if !payer.attributes.iter().any(|a| a.name == "signer") {
                                    eprintln!(
                                        "@init payer '{}' must have @signer constraint",
                                        payer_name
                                    );
                                    return Err(VMError::ConstraintViolation);
                                }
                            }
                        }
                    }
                }
            }

            if param.pda_config.is_some() {
                if !self.is_account_param_type(&param.param_type) {
                    return Err(VMError::TypeMismatch);
                }
                if Self::param_has_attribute(param, "signer") {
                    return Err(VMError::InvalidInstruction);
                }
                self.init_bump_accounts.insert(param.name.clone());
            }

            if param.serializer.is_some() && !self.is_account_param_type(&param.param_type) {
                return Err(VMError::TypeMismatch);
            }

            // For account parameters, store them as Account type so field access works
            let param_type = if param.param_type.is_account_type() {
                TypeNode::Account
            } else {
                param.param_type.clone()
            };

            // Implicit mutability: @init implies mutable, or explicit @mut
            let has_close = param.attributes.iter().any(|a| a.name == "close");
            let is_mutable =
                param.is_init || param.attributes.iter().any(|a| a.name == "mut") || has_close;

            self.symbol_table
                .insert(param.name.clone(), (param_type.clone(), is_mutable));

            if let Some(init_config) = &param.init_config {
                if init_config.seeds.is_some() {
                    self.init_bump_accounts.insert(param.name.clone());
                }
                self.init_space_accounts.insert(param.name.clone());
            }

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
                crate::ast::TypeNode::Named(name) => self.is_named_account_type_name(name),
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
                    "authority" => {
                        return Err(VMError::InvalidInstruction);
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
                                let target_exists =
                                    parameters.iter().any(|p| p.name == *target_name);
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
                                let target_exists =
                                    parameters.iter().any(|p| p.name == *target_name);
                                if !target_exists {
                                    eprintln!("@owner target not found: {}", target_name);
                                    // Check imports? Return error.
                                    return Err(VMError::InvalidScript);
                                }
                            }
                        }
                    }
                    "close" => {
                        if !is_account_param {
                            return Err(VMError::TypeMismatch);
                        }
                        if attr.args.len() != 1 {
                            return Err(VMError::InvalidInstruction);
                        }
                        let target_name = match &attr.args[0] {
                            AstNode::Identifier(name) => name,
                            _ => return Err(VMError::InvalidInstruction),
                        };
                        let Some(target_param) = parameters.iter().find(|p| p.name == *target_name)
                        else {
                            return Err(VMError::InvalidScript);
                        };
                        if !self.is_account_param_type(&target_param.param_type) {
                            return Err(VMError::TypeMismatch);
                        }
                        let target_mutable = target_param.is_init
                            || target_param.attributes.iter().any(|a| a.name == "mut");
                        if !target_mutable {
                            return Err(VMError::ConstraintViolation);
                        }
                    }
                    "session" => {
                        if !is_account_param {
                            return Err(VMError::TypeMismatch);
                        }
                        let has_keyed_args = attr
                            .args
                            .iter()
                            .all(|arg| matches!(arg, AstNode::Assignment { .. }));
                        if !has_keyed_args && !attr.args.is_empty() {
                            return Err(VMError::InvalidInstruction);
                        }

                        let is_legacy_session_param = session_support::is_session_type(param);
                        if is_legacy_session_param {
                            // Immediate break: dedicated Session @session params are no longer supported.
                            return Err(VMError::InvalidInstruction);
                        }

                        let authority_name = if let Some(AstNode::Identifier(name)) =
                            Self::session_attr_value(attr, "authority", 0)
                        {
                            name.clone()
                        } else {
                            param.name.clone()
                        };

                        if Self::session_attr_value(attr, "delegate", 0).is_some() {
                            // Identity is sourced from the owner/authority slot only.
                            return Err(VMError::InvalidInstruction);
                        }

                        let Some(authority_param) =
                            parameters
                                .iter()
                                .find(|p| p.name == authority_name.as_str())
                        else {
                            return Err(VMError::InvalidScript);
                        };
                        if !self.is_account_param_type(&authority_param.param_type) {
                            return Err(VMError::TypeMismatch);
                        }

                        if !is_legacy_session_param {
                            if let Some(session_fields) =
                                self.resolve_account_definition_fields("Session")
                            {
                                let names: std::collections::HashSet<&str> = session_fields
                                    .iter()
                                    .map(|field| field.name.as_str())
                                    .collect();
                                let canonical: std::collections::HashSet<&str> =
                                    session_support::SESSION_V1_FIELDS.iter().copied().collect();
                                if names != canonical {
                                    eprintln!(
                                        "Session account definition conflicts with std::session::Session v1 fields"
                                    );
                                    return Err(VMError::InvalidScript);
                                }
                            }

                            // Bare @session on authority/owner is allowed; nonce and other
                            // provenance keys are optional and can be inferred/injected later.
                        }
                        for arg in &attr.args {
                            if let AstNode::Assignment { target, .. } = arg {
                                if matches!(
                                    target.as_str(),
                                    "manager_script_account"
                                        | "manager_script"
                                        | "manager_code_hash"
                                        | "manager_hash"
                                        | "manager_version"
                                ) {
                                    eprintln!(
                                        "unsupported @session key '{}': manager identity fields were removed",
                                        target
                                    );
                                    return Err(VMError::InvalidInstruction);
                                }
                            }
                        }

                        for (key, pos) in [
                            ("target_program", 1usize),
                            ("scope_hash", 2usize),
                            ("bind_account", 3usize),
                            ("nonce", 4usize),
                            ("nonce_field", 4usize),
                            ("current_slot", 5usize),
                        ] {
                            if let Some(arg) = Self::session_attr_value(attr, key, pos) {
                                match arg {
                                    AstNode::Identifier(name) => {
                                        if !parameters.iter().any(|p| p.name == *name) {
                                            return Err(VMError::InvalidScript);
                                        }
                                    }
                                    _ => {
                                        return Err(VMError::InvalidInstruction);
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        // Enforce: @close(source) cannot coexist with explicit close_account(source, ...)
        for param in parameters {
            let has_close_attr = param.attributes.iter().any(|a| a.name == "close");
            if has_close_attr && Self::contains_close_call_for_source(body, &param.name) {
                return Err(VMError::InvalidInstruction);
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
        self.current_function_parameters = previous_function_parameters;
        self.init_bump_accounts = previous_init_bump_accounts;
        self.init_space_accounts = previous_init_space_accounts;

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
        serializer: Option<crate::ast::AccountSerializer>,
        visibility: crate::ast::Visibility,
    ) -> Result<(), VMError> {
        self.account_definitions
            .insert(name.to_string(), fields.to_vec());
        if let Some(serializer) = serializer {
            self.account_serializers
                .insert(name.to_string(), serializer);
        }

        // Add to module scope for cross-module resolution
        self.add_to_module_scope(name.to_string(), TypeNode::Account, false, visibility);

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

    pub(crate) fn check_type_definition(
        &mut self,
        name: &str,
        definition: &TypeNode,
        visibility: crate::ast::Visibility,
    ) -> Result<(), VMError> {
        self.validate_type(definition)?;
        self.type_definitions
            .insert(name.to_string(), definition.clone());
        self.add_to_module_scope(name.to_string(), definition.clone(), false, visibility);
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
