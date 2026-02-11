// Expression type checking

use super::types::TypeCheckerContext;
use crate::ast::{AstNode, TypeNode};
use five_protocol::Value;
use five_vm_mito::error::VMError;

impl TypeCheckerContext {
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
                        // Check if identifier exists in symbol table or interface registry
                        if !self.symbol_table.contains_key(name)
                            && !self.interface_registry.contains_key(name)
                        {
                            eprintln!(
                                "Undefined identifier{}: '{}' is not in scope",
                                match &self.current_function {
                                    Some(f) => format!(" in function '{}'", f),
                                    None => "".to_string(),
                                },
                                name
                            );
                            return Err(VMError::UndefinedIdentifier);
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
                        let account_fields = self.account_definitions.get(&name)
                            .or_else(|| {
                                self.account_definitions.iter()
                                    .find(|(k, _)| k.ends_with(&namespace_suffix))
                                    .map(|(_, v)| v)
                            });
                        
                        if let Some(account_fields) = account_fields {
                            eprintln!("DEBUG: Resolved account_fields: {:?}", account_fields.iter().map(|f| &f.name).collect::<Vec<_>>());
                            if account_fields.iter().any(|f| f.name == *field) {
                                Ok(())
                            } else {
                                self.validate_builtin_account_property(field)?;
                                Ok(())
                            }
                        } else {
                            eprintln!("DEBUG: No account definition found for '{}'", name);
                            // Not a known account type - fallback to built-in properties
                            self.validate_builtin_account_property(field)?;
                            Ok(())
                        }
                    }
                    TypeNode::Account => {
                        // Built-in account properties are always valid
                        self.validate_builtin_account_property(field)?;
                        Ok(())
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
                None => Err(VMError::UndefinedIdentifier),
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
                operator: _,
                left,
                right,
            } => {
                // Type check both operands
                self.check_types(left)?;
                self.check_types(right)?;
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
        // Check if this is an interface method call first
        if let AstNode::Identifier(interface_name) = object {
            if let Some(interface_info) = self.interface_registry.get(interface_name) {
                // This is an interface method call - validate it
                if let Some(interface_method) = interface_info.methods.get(method) {
                    // Clone the interface method to avoid borrow checker issues
                    let method_params = interface_method.parameters.clone();
                    let method_return_type = interface_method.return_type.clone();

                    // Validate argument count
                    if args.len() != method_params.len() {
                        return Err(VMError::InvalidOperation);
                    }

                    // Type check all arguments against interface method parameters
                    for (i, arg) in args.iter().enumerate() {
                        let arg_type = self.infer_type(arg)?;
                        if !self.types_are_compatible(&arg_type, &method_params[i]) {
                            return Err(VMError::TypeMismatch);
                        }
                    }

                    // Return the interface method's return type or unit
                    return Ok(
                        method_return_type.unwrap_or(TypeNode::Primitive("unit".to_string()))
                    );
                } else {
                    return Err(VMError::InvalidOperation); // Method not found in interface
                }
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

                let ok = (is_bool(&object_type) && is_bool(&arg_type))
                    || (is_numeric(&object_type) && is_numeric(&arg_type))
                    || (is_pubkey(&object_type) && is_pubkey(&arg_type))
                    || (is_string(&object_type) && is_string(&arg_type))
                    || self.types_are_compatible(&object_type, &arg_type);
                if !ok {
                    return Err(VMError::TypeMismatch);
                }
                // Accept equality for primitives and named/custom types when compatible
                Ok(TypeNode::Primitive("bool".to_string()))
            }
            "lt" | "le" | "gt" | "ge" => {
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
            "derive_pda" => {
                // derive_pda supports multiple signatures:
                // derive_pda(seed1, seed2, ...) -> (pubkey, u8) - Find PDA
                // derive_pda(seed1, seed2, ..., bump) -> pubkey - Validate PDA with known bump
                if args.len() < 2 {
                    return Err(VMError::InvalidOperation);
                }

                // Type check all arguments - seeds can be various types (string, u64, pubkey)
                for arg in args {
                    self.infer_type(arg)?;
                }

                // Return type depends on whether bump is provided as last argument
                // If last argument is u8 (bump), return single pubkey
                // Otherwise return tuple (pubkey, u8)
                if args.len() >= 3 {
                    let last_arg_type = self.infer_type(&args[args.len() - 1])?;
                    if matches!(last_arg_type, TypeNode::Primitive(ref name) if name == "u8") {
                        // Bump provided, return single pubkey (validation mode)
                        Ok(TypeNode::Primitive("pubkey".to_string()))
                    } else {
                        // No bump, return tuple (find mode)
                        Ok(TypeNode::Tuple {
                            elements: vec![
                                TypeNode::Primitive("pubkey".to_string()),
                                TypeNode::Primitive("u8".to_string()),
                            ],
                        })
                    }
                } else {
                    // 2 arguments, return tuple (find mode)
                    Ok(TypeNode::Tuple {
                        elements: vec![
                            TypeNode::Primitive("pubkey".to_string()),
                            TypeNode::Primitive("u8".to_string()),
                        ],
                    })
                }
            }
            "invoke_signed" => {
                // invoke_signed(instruction: Instruction, signers: [Signer]) -> void
                if args.len() != 2 {
                    return Err(VMError::InvalidOperation);
                }
                // Type check arguments but allow flexible types for now
                self.infer_type(&args[0])?;
                self.infer_type(&args[1])?;
                Ok(TypeNode::Primitive("void".to_string()))
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
                    // TEMP: For namespaced calls (Module::Function), default to u64 if unknown.
                    // This allows compiling amm_core.v without full Import Resolution implemented.
                    if name.contains("::") {
                        return Ok(TypeNode::Primitive("u64".to_string()));
                    }
                    Ok(TypeNode::Primitive("void".to_string()))
                }
            }
        }
    }
}
