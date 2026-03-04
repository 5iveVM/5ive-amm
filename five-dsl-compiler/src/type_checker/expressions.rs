// Expression type checking

use super::types::TypeCheckerContext;
use crate::ast::{AstNode, TypeNode};
use five_protocol::Value;
use five_vm_mito::error::VMError;

impl TypeCheckerContext {
    pub(crate) fn argument_matches_expected_type(
        &mut self,
        arg: &AstNode,
        expected_type: &TypeNode,
    ) -> Result<bool, VMError> {
        if let (AstNode::Literal(value), TypeNode::Primitive(expected_name)) = (arg, expected_type) {
            match expected_name.as_str() {
                "u8" => {
                    if value
                        .as_u64()
                        .or_else(|| value.as_i64().filter(|v| *v >= 0).map(|v| v as u64))
                        .filter(|v| *v <= u8::MAX as u64)
                        .is_some()
                    {
                        return Ok(true);
                    }
                }
                "u16" => {
                    if value
                        .as_u64()
                        .or_else(|| value.as_i64().filter(|v| *v >= 0).map(|v| v as u64))
                        .filter(|v| *v <= u16::MAX as u64)
                        .is_some()
                    {
                        return Ok(true);
                    }
                }
                "u32" => {
                    if value
                        .as_u64()
                        .or_else(|| value.as_i64().filter(|v| *v >= 0).map(|v| v as u64))
                        .filter(|v| *v <= u32::MAX as u64)
                        .is_some()
                    {
                        return Ok(true);
                    }
                }
                _ => {}
            }
        }

        let arg_type = self.infer_type(arg)?;
        if self.types_are_compatible(&arg_type, expected_type) {
            return Ok(true);
        }

        if arg_type.is_account_type() && expected_type.is_account_type() {
            return Ok(true);
        }

        if let (
            AstNode::ArrayLiteral { elements },
            TypeNode::Array {
                element_type,
                size: Some(expected_len),
            },
        ) = (arg, expected_type)
        {
            if elements.len() as u64 != *expected_len {
                return Ok(false);
            }

            if let TypeNode::Primitive(element_name) = element_type.as_ref() {
                if element_name == "u8" {
                    let all_fit = elements.iter().all(|element| {
                        matches!(element, AstNode::Literal(Value::U64(v)) if *v <= u8::MAX as u64)
                    });
                    if all_fit {
                        return Ok(true);
                    }
                }
            }
        }

        Ok(false)
    }

    fn infer_local_interface_method_call_type(
        &mut self,
        interface_name: &str,
        method: &str,
        args: &[AstNode],
    ) -> Result<TypeNode, VMError> {
        let Some(interface_info) = self.interface_registry.get(interface_name) else {
            return Err(self.undefined_identifier_error(interface_name));
        };
        let interface_info = interface_info.clone();
        self.validate_interface_method_call(&interface_info, method, args)
    }

    pub(crate) fn legacy_account_metadata_replacement(field: &str) -> Option<String> {
        match field {
            "key" | "lamports" | "owner" | "data" => Some(format!("ctx.{}", field)),
            _ => None,
        }
    }

    pub(crate) fn legacy_init_alias_replacement(&self, ident: &str) -> Option<String> {
        if let Some(account_name) = ident.strip_suffix("_bump") {
            if self.init_bump_accounts.contains(account_name) {
                return Some(format!("{}.ctx.bump", account_name));
            }
        }
        if let Some(account_name) = ident.strip_suffix("_space") {
            if self.init_space_accounts.contains(account_name) {
                return Some(format!("{}.ctx.space", account_name));
            }
        }
        None
    }

    fn resolve_account_ctx_field_type(
        &self,
        account_expr: &AstNode,
        field: &str,
    ) -> Result<TypeNode, VMError> {
        let account_name = if let AstNode::Identifier(name) = account_expr {
            name
        } else {
            return Err(VMError::TypeMismatch);
        };

        match field {
            "lamports" => Ok(TypeNode::Primitive("u64".to_string())),
            "owner" | "key" => Ok(TypeNode::Primitive("pubkey".to_string())),
            "data" => Ok(TypeNode::Array {
                element_type: Box::new(TypeNode::Primitive("u8".to_string())),
                size: None,
            }),
            "bump" => {
                if self.init_bump_accounts.contains(account_name) {
                    Ok(TypeNode::Primitive("u8".to_string()))
                } else {
                    Err(VMError::UndefinedField)
                }
            }
            "space" => {
                if self.init_space_accounts.contains(account_name) {
                    Ok(TypeNode::Primitive("u64".to_string()))
                } else {
                    Err(VMError::UndefinedField)
                }
            }
            _ => Err(VMError::UndefinedField),
        }
    }

    pub(crate) fn check_expression(&mut self, expr: &AstNode) -> Result<(), VMError> {
        match expr {
            AstNode::Literal(_) => Ok(()),
            AstNode::StringLiteral { .. } => Ok(()),
            AstNode::Identifier(name) => {
                // Allow special identifiers that are handled by the AST generator
                match name.as_str() {
                    "None" | "signer" => {
                        // These are handled as special cases in the AST generator
                        Ok(())
                    }
                    _ => {
                        // Identifiers in expression position must be in scope variables/fields.
                        if !self.symbol_table.contains_key(name) {
                            if let Some(replacement) = self.legacy_init_alias_replacement(name) {
                                return Err(VMError::undefined_identifier(
                                    name,
                                    Some(&replacement),
                                ));
                            }
                            eprintln!(
                                "Undefined identifier{}: '{}' is not in scope",
                                match &self.current_function {
                                    Some(f) => format!(" in function '{}'", f),
                                    None => "".to_string(),
                                },
                                name
                            );
                            return Err(self.undefined_identifier_error(name));
                        }
                        Ok(())
                    }
                }
            }
            AstNode::MethodCall {
                object,
                method,
                args,
            } => {
                if let AstNode::Identifier(interface_name) = object.as_ref() {
                    if self.interface_registry.contains_key(interface_name) {
                        if !self.imported_external_interfaces.contains(interface_name) {
                            self.infer_local_interface_method_call_type(
                                interface_name,
                                method,
                                args,
                            )?;
                            return Ok(());
                        }

                        // Legacy object-style imported interface call
                        // (e.g. SPLToken.transfer(...)) is intentionally unsupported.
                        // Imported interfaces are module-qualified:
                        // use std::interfaces::spl_token; spl_token::transfer(...)
                        let suggestion = self
                            .interface_module_aliases
                            .iter()
                            .find_map(|(ns, iface)| {
                                if iface == interface_name && !ns.contains("::") {
                                    Some(format!("{}::{}", ns, method))
                                } else {
                                    None
                                }
                            })
                            .unwrap_or_else(|| format!("module_alias::{}", method));
                        return Err(VMError::undefined_identifier(
                            interface_name,
                            Some(&suggestion),
                        ));
                    }
                }

                self.check_types(object)?;
                for arg in args {
                    self.check_types(arg)?;
                }

                // Force method-call type validation so interface CPI calls
                // get strict argument checking (including account vs pubkey).
                self.infer_method_call_type(object, method, args)?;

                // Check for type mismatches in arithmetic operations
                if matches!(method.as_str(), "add" | "sub" | "mul" | "div") {
                    // Check if we're trying to do arithmetic with incompatible types
                    if let AstNode::Identifier(var_name) = object.as_ref() {
                        if let Some((TypeNode::Primitive(type_name), _)) =
                            self.symbol_table.get(var_name)
                        {
                            if type_name == "string" {
                                // String variable + number is a type error
                                if !args.is_empty() {
                                    if let AstNode::Literal(Value::U64(_)) = args[0] {
                                        return Err(VMError::TypeMismatch);
                                    }
                                }
                            }
                        }
                    }
                }

                Ok(())
            }
            AstNode::FunctionCall { name: _, args } => {
                // Type check function call arguments
                for arg in args {
                    self.check_types(arg)?;
                }
                Ok(())
            }
            AstNode::StructLiteral { fields } => {
                // Type check all field values
                for field in fields {
                    self.check_types(&field.value)?;
                }
                Ok(())
            }
            AstNode::ArrayLiteral { elements } => {
                // Type check all array elements
                for element in elements {
                    self.check_types(element)?;
                }
                Ok(())
            }
            AstNode::TupleLiteral { elements } => {
                // Type check all tuple elements
                for element in elements {
                    self.check_types(element)?;
                }
                Ok(())
            }
            AstNode::TupleAccess { object, index } => {
                let object_type = self.infer_type(object)?;
                if let TypeNode::Tuple { elements } = object_type {
                    if *index as usize >= elements.len() {
                        return Err(VMError::IndexOutOfBounds);
                    }
                } else {
                    return Err(VMError::TypeMismatch);
                }
                Ok(())
            }
            AstNode::ArrayAccess { array, index } => {
                self.check_types(array)?;
                self.check_types(index)?;

                // Check that array is actually an array type
                let array_type = self.infer_type(array)?;
                if !matches!(array_type, TypeNode::Array { .. }) {
                    return Err(VMError::TypeMismatch);
                }

                // Check that index is a numeric type
                let index_type = self.infer_type(index)?;
                if index_type.is_numeric() {
                    Ok(())
                } else {
                    Err(VMError::InvalidScript) // Index must be numeric
                }
            }
            AstNode::FieldAccess { object, field } => {
                if field == "ctx" {
                    let object_type = self.infer_type(object)?;
                    return if matches!(object_type, TypeNode::Account | TypeNode::Named(_)) {
                        Ok(())
                    } else {
                        Err(VMError::TypeMismatch)
                    };
                }
                if let AstNode::FieldAccess {
                    object: account_expr,
                    field: ctx_field,
                } = object.as_ref()
                {
                    if ctx_field == "ctx" {
                        return self
                            .resolve_account_ctx_field_type(account_expr, field)
                            .map(|_| ());
                    }
                }
                let object_type = self.infer_type(object)?;

                match object_type {
                    TypeNode::Struct { fields } => {
                        if fields.iter().any(|f| f.name == *field) {
                            Ok(())
                        } else {
                            Err(VMError::UndefinedField)
                        }
                    }
                    TypeNode::Named(name) => {
                        // Look up account fields with namespace-aware matching
                        // Account names may be namespaced (e.g., "amm_types::AMMPool") but referenced by simple name ("AMMPool")
                        let namespace_suffix = format!("::{}", name);
                        eprintln!("DEBUG: expressions.rs check_expression FieldAccess on TypeNode::Named('{}'), looking for field '{}', suffix='{}'", name, field, namespace_suffix);
                        let account_fields = self.account_definitions.get(&name).or_else(|| {
                            self.account_definitions
                                .iter()
                                .find(|(k, _)| k.ends_with(&namespace_suffix))
                                .map(|(_, v)| v)
                        });

                        if let Some(account_fields) = account_fields {
                            eprintln!(
                                "DEBUG: Resolved account_fields: {:?}",
                                account_fields.iter().map(|f| &f.name).collect::<Vec<_>>()
                            );
                            if account_fields.iter().any(|f| f.name == *field) {
                                Ok(())
                            } else {
                                if let Some(replacement) =
                                    Self::legacy_account_metadata_replacement(field)
                                {
                                    Err(VMError::undefined_identifier(field, Some(&replacement)))
                                } else {
                                    Err(VMError::UndefinedField)
                                }
                            }
                        } else {
                            eprintln!("DEBUG: No account definition found for '{}'", name);
                            if let Some(replacement) =
                                Self::legacy_account_metadata_replacement(field)
                            {
                                Err(VMError::undefined_identifier(field, Some(&replacement)))
                            } else {
                                Err(VMError::UndefinedField)
                            }
                        }
                    }
                    TypeNode::Account => {
                        if let Some(replacement) = Self::legacy_account_metadata_replacement(field)
                        {
                            Err(VMError::undefined_identifier(field, Some(&replacement)))
                        } else {
                            Err(VMError::UndefinedField)
                        }
                    }
                    _ => Err(VMError::TypeMismatch),
                }
            }
            AstNode::TemplateLiteral { parts } => {
                for part in parts {
                    self.check_types(part)?;
                }
                Ok(())
            }
            AstNode::EnumVariantAccess {
                enum_name,
                variant_name,
            } => match self.symbol_table.get(enum_name) {
                Some((TypeNode::Struct { fields }, _)) => {
                    if fields.iter().any(|f| f.name == *variant_name) {
                        Ok(())
                    } else {
                        Err(VMError::UndefinedField)
                    }
                }
                Some(_) => Err(VMError::TypeMismatch),
                None => Err(self.undefined_identifier_error(enum_name)),
            },
            AstNode::ErrorPropagation { expression } => {
                // Type check the inner expression (should return Result type)
                let expr_type = self.infer_type(expression)?;
                if let TypeNode::Generic { base, .. } = expr_type {
                    if base != "Result" {
                        return Err(VMError::TypeMismatch);
                    }
                } else {
                    return Err(VMError::TypeMismatch);
                }
                Ok(())
            }
            AstNode::UnaryExpression {
                operator: _,
                operand,
            } => {
                // Type check the operand
                self.check_types(operand)?;
                Ok(())
            }
            AstNode::BinaryExpression {
                operator,
                left,
                right,
            } => {
                // Type check both operands
                self.check_types(left)?;
                self.check_types(right)?;

                // For equality operators, perform inference to catch type errors early
                if matches!(operator.as_str(), "==" | "!=") {
                    // Infer types to trigger type compatibility checking
                    let _ = self.infer_type(expr)?;
                }

                Ok(())
            }
            AstNode::Cast {
                value,
                target_type: _,
            } => {
                // Type check the value being cast
                self.check_types(value)?;
                Ok(())
            }
            _ => {
                // Not an expression node, delegate to main check_types
                Err(VMError::InvalidScript)
            }
        }
    }

    pub(crate) fn infer_method_call_type(
        &mut self,
        object: &AstNode,
        method: &str,
        args: &[AstNode],
    ) -> Result<TypeNode, VMError> {
        if let AstNode::Identifier(interface_name) = object {
            if self.interface_registry.contains_key(interface_name)
                && !self.imported_external_interfaces.contains(interface_name)
            {
                return self.infer_local_interface_method_call_type(interface_name, method, args);
            }
        }

        let object_type = self.infer_type(object)?;

        // Type check all arguments
        for arg in args {
            self.infer_type(arg)?;
        }

        // Method-specific type checking and return type inference
        match method {
            "add" | "sub" | "mul" | "div" | "mod" => {
                // Arithmetic methods: expect one argument of compatible numeric type
                if args.len() != 1 {
                    return Err(VMError::InvalidOperation);
                }
                let arg_type = self.infer_type(&args[0])?;
                let obj_is_numeric = matches!(object_type, TypeNode::Primitive(ref name) if Self::is_numeric_primitive_name(name));
                let arg_is_numeric = matches!(arg_type, TypeNode::Primitive(ref name) if Self::is_numeric_primitive_name(name));
                if !obj_is_numeric || !arg_is_numeric {
                    return Err(VMError::TypeMismatch);
                }
                if self.types_are_compatible(&object_type, &arg_type) {
                    Ok(object_type)
                } else if let Some(promoted) = Self::promote_numeric_types(&object_type, &arg_type)
                {
                    Ok(promoted)
                } else {
                    Err(VMError::TypeMismatch)
                }
            }
            "eq" | "ne" => {
                // Equality: allow compatible types including bool, numeric, pubkey, string
                if args.len() != 1 {
                    return Err(VMError::InvalidOperation);
                }
                let arg_type = self.infer_type(&args[0])?;
                let is_bool =
                    |t: &TypeNode| matches!(t, TypeNode::Primitive(name) if name == "bool");
                let is_numeric = |t: &TypeNode| matches!(t, TypeNode::Primitive(name) if matches!(name.as_str(), "u8"|"u16"|"u32"|"u64"|"u128"|"i8"|"i16"|"i32"|"i64"));
                let is_pubkey =
                    |t: &TypeNode| matches!(t, TypeNode::Primitive(name) if name == "pubkey");
                let is_string =
                    |t: &TypeNode| matches!(t, TypeNode::Primitive(name) if name == "string");

                // Check for pubkey zero comparison (pubkey == 0 or pubkey != 0)
                let is_pubkey_zero_compare = {
                    let object_is_pubkey = is_pubkey(&object_type);
                    let arg_is_zero = matches!(&args[0], AstNode::Literal(Value::U64(0)));
                    (object_is_pubkey && arg_is_zero)
                        || (is_pubkey(&arg_type)
                            && matches!(object, AstNode::Literal(Value::U64(0))))
                };

                let ok = (is_bool(&object_type) && is_bool(&arg_type))
                    || (is_numeric(&object_type) && is_numeric(&arg_type))
                    || (is_pubkey(&object_type) && is_pubkey(&arg_type))
                    || (is_string(&object_type) && is_string(&arg_type))
                    || is_pubkey_zero_compare
                    || self.types_are_compatible(&object_type, &arg_type);
                if !ok {
                    return Err(VMError::TypeMismatch);
                }
                // Accept equality for primitives and named/custom types when compatible
                Ok(TypeNode::Primitive("bool".to_string()))
            }
            "lt" | "le" | "lte" | "gt" | "ge" | "gte" => {
                // Ordering comparisons: require numeric operands
                if args.len() != 1 {
                    return Err(VMError::InvalidOperation);
                }
                let arg_type = self.infer_type(&args[0])?;
                let is_numeric = |t: &TypeNode| matches!(t, TypeNode::Primitive(name) if Self::is_numeric_primitive_name(name));
                if !(is_numeric(&object_type) && is_numeric(&arg_type)) {
                    return Err(VMError::TypeMismatch);
                }
                Ok(TypeNode::Primitive("bool".to_string()))
            }
            "and" | "or" => {
                // Logical methods: expect boolean object and one boolean argument
                if args.len() != 1 {
                    return Err(VMError::InvalidOperation);
                }
                if !matches!(object_type, TypeNode::Primitive(ref name) if name == "bool") {
                    return Err(VMError::TypeMismatch);
                }
                let arg_type = self.infer_type(&args[0])?;
                if !matches!(arg_type, TypeNode::Primitive(ref name) if name == "bool") {
                    return Err(VMError::TypeMismatch);
                }
                Ok(TypeNode::Primitive("bool".to_string()))
            }
            "is_some" => {
                // is_some(): expects Option<T>, returns bool
                if !args.is_empty() {
                    return Err(VMError::InvalidOperation); // is_some expects no arguments
                }
                if let TypeNode::Generic { base, .. } = object_type {
                    if base == "Option" {
                        Ok(TypeNode::Primitive("bool".to_string()))
                    } else {
                        Err(VMError::TypeMismatch)
                    }
                } else {
                    Err(VMError::TypeMismatch)
                }
            }
            "get_value" => {
                // get_value(): expects Option<T>, returns T
                if !args.is_empty() {
                    return Err(VMError::InvalidOperation); // get_value expects no arguments
                }
                if let TypeNode::Generic {
                    base,
                    args: inner_args,
                } = object_type
                {
                    if base == "Option" && !inner_args.is_empty() {
                        Ok(inner_args[0].clone()) // Return the inner type of the Option
                    } else {
                        Err(VMError::TypeMismatch)
                    }
                } else {
                    Err(VMError::TypeMismatch)
                }
            }
            _ => Err(VMError::InvalidOperation),
        }
    }

    pub(crate) fn infer_function_call_type(
        &mut self,
        name: &str,
        args: &[AstNode],
    ) -> Result<TypeNode, VMError> {
        if let Some((interface_name, method_name)) = self.resolve_qualified_interface_call(name) {
            let Some(interface_info) = self.interface_registry.get(&interface_name) else {
                return Err(self.undefined_identifier_error(&interface_name));
            };
            let Some(interface_method) = interface_info.methods.get(&method_name) else {
                return Err(VMError::InvalidOperation);
            };

            let method_params = interface_method.parameters.clone();
            let method_return_type = interface_method.return_type.clone();

            if args.len() != method_params.len() {
                return Err(VMError::InvalidOperation);
            }

            for (i, arg) in args.iter().enumerate() {
                if !self.argument_matches_expected_type(arg, &method_params[i].param_type)? {
                    return Err(VMError::TypeMismatch);
                }
            }

            return Ok(method_return_type.unwrap_or(TypeNode::Primitive("unit".to_string())));
        }

        // Type check all arguments first
        for arg in args {
            self.infer_type(arg)?;
        }

        // Built-in function type checking with argument validation
        match name {
            "Some" => {
                if args.len() != 1 {
                    return Err(VMError::InvalidOperation); // Some expects exactly one argument
                }
                let inner_type = self.infer_type(&args[0])?;
                Ok(TypeNode::Generic {
                    base: "Option".to_string(),
                    args: vec![inner_type],
                })
            }
            "Ok" => {
                if args.len() != 1 {
                    return Err(VMError::InvalidOperation); // Ok expects exactly one argument
                }
                let inner_type = self.infer_type(&args[0])?;
                // For Ok(T), we don't know the error type E yet.
                // We'll represent it as Result<T, Unknown> for now, and rely on types_are_compatible
                // to match it against a concrete Result<T, E>
                Ok(TypeNode::Generic {
                    base: "Result".to_string(),
                    args: vec![inner_type, TypeNode::Named("UnknownError".to_string())],
                })
            }
            "Err" => {
                if args.len() != 1 {
                    return Err(VMError::InvalidOperation); // Err expects exactly one argument
                }
                let error_type = self.infer_type(&args[0])?;
                // For Err(E), we don't know the success type T yet.
                // We'll represent it as Result<Unknown, E> for now.
                Ok(TypeNode::Generic {
                    base: "Result".to_string(),
                    args: vec![TypeNode::Named("UnknownSuccess".to_string()), error_type],
                })
            }
            "require" => {
                // require(condition: bool) -> void
                if args.len() != 1 {
                    return Err(VMError::InvalidOperation);
                }
                let arg_type = self.infer_type(&args[0])?;
                if !matches!(arg_type, TypeNode::Primitive(ref name) if name == "bool") {
                    return Err(VMError::TypeMismatch);
                }
                Ok(TypeNode::Primitive("void".to_string()))
            }
            "get_clock" => {
                // get_clock() -> u64 (Unix timestamp, matches Solana Clock)
                if !args.is_empty() {
                    return Err(VMError::InvalidOperation);
                }
                Ok(TypeNode::Primitive("u64".to_string()))
            }
            "load_account_u64" | "load_account_u64_word" => {
                // load_account_u64(account, literal_offset) -> u64
                if args.len() != 2 {
                    return Err(VMError::InvalidOperation);
                }
                let account_ty = self.infer_type(&args[0])?;
                if !matches!(account_ty, TypeNode::Account | TypeNode::Named(_)) {
                    return Err(VMError::TypeMismatch);
                }
                let offset_ty = self.infer_type(&args[1])?;
                if !matches!(offset_ty, TypeNode::Primitive(ref name) if matches!(name.as_str(), "u8" | "u16" | "u32" | "u64"))
                {
                    return Err(VMError::TypeMismatch);
                }
                Ok(TypeNode::Primitive("u64".to_string()))
            }
            "string_concat" | "bytes_concat" => {
                if args.len() != 2 {
                    return Err(VMError::InvalidOperation);
                }
                self.infer_type(&args[0])?;
                self.infer_type(&args[1])?;
                Ok(TypeNode::Primitive("string".to_string()))
            }
            "sha256" | "keccak256" | "blake3" => {
                // Low-level syscall ABI: hash(input_bytes, out_32_bytes) -> void.
                if args.len() != 2 {
                    return Err(VMError::InvalidOperation);
                }
                self.infer_type(&args[0])?;
                self.infer_type(&args[1])?;
                Ok(TypeNode::Primitive("void".to_string()))
            }
            "verify_ed25519_instruction" | "__verify_ed25519_instruction" => {
                // verify_ed25519_instruction(instruction_sysvar, expected_pubkey, message, signature) -> bool
                if args.len() != 4 {
                    return Err(VMError::InvalidOperation);
                }

                let sysvar_ty = self.infer_type(&args[0])?;
                if !matches!(sysvar_ty, TypeNode::Account | TypeNode::Named(_)) {
                    return Err(VMError::TypeMismatch);
                }

                let expected_pubkey_ty = self.infer_type(&args[1])?;
                if !matches!(expected_pubkey_ty, TypeNode::Primitive(ref t) if t == "pubkey") {
                    return Err(VMError::TypeMismatch);
                }

                self.infer_type(&args[2])?;
                self.infer_type(&args[3])?;
                Ok(TypeNode::Primitive("bool".to_string()))
            }
            "derive_pda" => {
                // derive_pda supports multiple signatures:
                // derive_pda(seed1, seed2, ...) -> (pubkey, u8) - Find PDA
                // derive_pda(seed1, seed2, ..., bump: u8) -> pubkey - Validate PDA with known bump
                if args.is_empty() {
                    return Err(VMError::InvalidOperation);
                }

                // Type check all arguments - seeds can be various types (string, u64, pubkey)
                for arg in args {
                    self.infer_type(arg)?;
                }

                // Return type depends on whether bump is provided as last argument.
                // If last argument is u8, use validation mode and return pubkey.
                let last_arg_type = self.infer_type(&args[args.len() - 1])?;
                if matches!(last_arg_type, TypeNode::Primitive(ref name) if name == "u8") {
                    Ok(TypeNode::Primitive("pubkey".to_string()))
                } else {
                    Ok(TypeNode::Tuple {
                        elements: vec![
                            TypeNode::Primitive("pubkey".to_string()),
                            TypeNode::Primitive("u8".to_string()),
                        ],
                    })
                }
            }
            "invoke_signed" => {
                // invoke_signed(program_id, instruction_data, accounts, seeds) -> void
                if args.len() != 4 {
                    return Err(VMError::InvalidOperation);
                }
                // Type check arguments but allow flexible types for now
                self.infer_type(&args[0])?;
                self.infer_type(&args[1])?;
                self.infer_type(&args[2])?;
                self.infer_type(&args[3])?;
                Ok(TypeNode::Primitive("void".to_string()))
            }
            "transfer_lamports" => {
                // transfer_lamports(from: account, to: account, amount: u64) -> void
                if args.len() != 3 {
                    return Err(VMError::InvalidOperation);
                }
                let from_ty = self.infer_type(&args[0])?;
                let to_ty = self.infer_type(&args[1])?;
                let amount_ty = self.infer_type(&args[2])?;
                let from_ok = matches!(from_ty, TypeNode::Account | TypeNode::Named(_));
                let to_ok = matches!(to_ty, TypeNode::Account | TypeNode::Named(_));
                let amount_ok = matches!(amount_ty, TypeNode::Primitive(ref n) if n == "u64" || n == "lamports");
                if !from_ok || !to_ok || !amount_ok {
                    return Err(VMError::TypeMismatch);
                }
                Ok(TypeNode::Primitive("void".to_string()))
            }
            "pubkey" => {
                // Backward compatibility constructor.
                // Supported forms:
                // - pubkey(0)            -> pubkey zero sentinel
                // - pubkey(existing_key) -> identity
                if args.len() != 1 {
                    return Err(VMError::InvalidOperation);
                }
                let arg_type = self.infer_type(&args[0])?;
                let is_zero_literal = matches!(&args[0], AstNode::Literal(Value::U64(0)));
                let is_pubkey_arg =
                    matches!(arg_type, TypeNode::Primitive(ref name) if name == "pubkey");
                if is_zero_literal || is_pubkey_arg {
                    Ok(TypeNode::Primitive("pubkey".to_string()))
                } else {
                    Err(VMError::TypeMismatch)
                }
            }
            _ => {
                // For user-defined functions, use pre-registered return type if available; default to void
                for arg in args {
                    let _ = self.infer_type(arg)?;
                }
                if let Some(ret) = self.function_return_types.get(name) {
                    Ok(ret
                        .clone()
                        .unwrap_or(TypeNode::Primitive("void".to_string())))
                } else {
                    Ok(TypeNode::Primitive("void".to_string()))
                }
            }
        }
    }

    fn parse_module_qualified_call(name: &str) -> Option<(&str, &str)> {
        let idx = name.rfind("::")?;
        let module_ns = &name[..idx];
        let method = &name[idx + 2..];
        if module_ns.is_empty() || method.is_empty() {
            return None;
        }
        Some((module_ns, method))
    }

    fn resolve_qualified_interface_call(&self, name: &str) -> Option<(String, String)> {
        let (qualifier, method_name) = Self::parse_module_qualified_call(name)?;

        if let Some(interface_name) = self.imported_interface_symbols.get(qualifier) {
            return Some((interface_name.clone(), method_name.to_string()));
        }

        if self.interface_registry.contains_key(qualifier) {
            return Some((qualifier.to_string(), method_name.to_string()));
        }

        let split_idx = qualifier.rfind("::")?;
        let module_ref = &qualifier[..split_idx];
        let interface_name = &qualifier[split_idx + 2..];
        let canonical_module = self
            .imported_module_aliases
            .get(module_ref)
            .cloned()
            .unwrap_or_else(|| module_ref.to_string());

        let scope = self.module_scope.as_ref()?;
        if !scope.has_module(&canonical_module)
            || !scope.module_exports_interface(&canonical_module, interface_name)
        {
            return None;
        }

        self.interface_registry
            .contains_key(interface_name)
            .then(|| (interface_name.to_string(), method_name.to_string()))
    }
}
