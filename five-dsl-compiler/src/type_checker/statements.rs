// Statement type checking

use super::types::TypeCheckerContext;
use crate::ast::{AstNode, TypeNode};
use five_vm_mito::error::VMError;

impl TypeCheckerContext {
    pub(crate) fn check_statement(&mut self, stmt: &AstNode) -> Result<(), VMError> {
        match stmt {
            AstNode::Block { statements, .. } => {
                for stmt in statements {
                    self.check_types(stmt)?;
                }
                Ok(())
            }
            AstNode::RequireStatement { condition } => {
                self.check_types(condition)?;
                Ok(())
            }
            AstNode::Assignment { target, value } => {
                // Type check the value
                self.check_types(value)?;
                let value_type = self.infer_type(value)?;

                // Implicitly register new top-level assignments (treated as global fields)
                if !self.symbol_table.contains_key(target)
                    && !self.interface_registry.contains_key(target)
                    && !self.imported_external_interfaces.contains(target)
                {
                    self.symbol_table
                        .insert(target.clone(), (value_type.clone(), false));
                    return Ok(());
                }

                // Check type compatibility with existing variable
                if let Some(existing_type) = self.symbol_table.get(target) {
                    if !self.types_are_compatible(&existing_type.0, &value_type) {
                        // Allow numeric assignments via promotion or literal fit
                        let allow_numeric = match (&existing_type.0, &value_type) {
                            (TypeNode::Primitive(l), TypeNode::Primitive(r))
                                if Self::is_numeric_primitive_name(l)
                                    && Self::is_numeric_primitive_name(r) =>
                            {
                                true
                            }
                            (left, _node) if Self::numeric_literal_fits(left, value) => true,
                            _ => false,
                        };
                        if !allow_numeric {
                            eprintln!(
                                "Type error{}: cannot assign value of type '{}' to variable '{}' of type '{}'",
                                match &self.current_function { Some(f) => format!(" in function '{}'", f), None => "".to_string() },
                                Self::fmt_type(&value_type),
                                target,
                                Self::fmt_type(&existing_type.0)
                            );
                            return Err(VMError::TypeMismatch);
                        }
                    }
                }
                // Note: Variable existence is verified above, no auto-insertion

                Ok(())
            }
            AstNode::FieldAssignment {
                object,
                field,
                value,
            } => {
                if let AstNode::FieldAccess { field: ctx_field, .. } = object.as_ref() {
                    if ctx_field == "ctx" {
                        return Err(VMError::ImmutableField);
                    }
                }
                // Type check the object and value
                let object_type = self.infer_type(object)?;
                let value_type = self.infer_type(value)?;

                if let AstNode::Identifier(_name) = object.as_ref() {
                    match object_type {
                        TypeNode::Struct { fields } => {
                            if let Some(field_def) = fields.iter().find(|f| f.name == *field) {
                                if !field_def.is_mutable {
                                    // Allow mutation if the struct identifier refers to a writable account param (@mut)
                                    // This relaxes field-level mut when the whole account is passed as @mut.
                                    let allow_via_account_mut =
                                        if let AstNode::Identifier(obj_name) = object.as_ref() {
                                            if let Some(writable) = &self.current_writable_accounts
                                            {
                                                writable.contains(obj_name)
                                            } else {
                                                false
                                            }
                                        } else {
                                            false
                                        };
                                    if !allow_via_account_mut {
                                        return Err(VMError::ImmutableField);
                                    }
                                }
                                // Handle optionality during assignment
                                if field_def.is_optional {
                                    // If the field is optional, the assigned value can be compatible with the inner type
                                    // or it can be an Optional type itself (e.g., assigning None)
                                    if !self
                                        .types_are_compatible(&field_def.field_type, &value_type)
                                        && !(matches!(value_type, TypeNode::Generic { base, .. } if base == "Option"))
                                    {
                                        return Err(VMError::TypeMismatch);
                                    }
                                } else {
                                    // If the field is not optional, the assigned value must be directly compatible
                                    if !self
                                        .types_are_compatible(&field_def.field_type, &value_type)
                                    {
                                        // Special-case: allow numeric zero literal to initialize/clear pubkey fields
                                        if matches!(&field_def.field_type, TypeNode::Primitive(name) if name == "pubkey")
                                            && Self::is_zero_numeric_literal(value)
                                        {
                                            return Ok(());
                                        }
                                        // Allow numeric literal narrowing when it fits the target type
                                        if !(Self::numeric_literal_fits(
                                            &field_def.field_type,
                                            value,
                                        )) {
                                            eprintln!(
                                                "Type error{}: cannot assign value of type '{}' to field '{}.{}' of type '{}'",
                                                match &self.current_function { Some(f) => format!(" in function '{}'", f), None => "".to_string() },
                                                Self::fmt_type(&value_type),
                                                if let AstNode::Identifier(obj_name) = object.as_ref() { obj_name } else { "<expr>" },
                                                field,
                                                Self::fmt_type(&field_def.field_type)
                                            );
                                            return Err(VMError::TypeMismatch);
                                        }
                                    }
                                }
                            } else {
                                return Err(VMError::UndefinedField);
                            }
                        }
                        TypeNode::Account => return Err(VMError::UndefinedField),
                        TypeNode::Named(account_type_name) => {
                            // Look up account fields with namespace-aware matching
                            let namespace_suffix = format!("::{}", account_type_name);
                            eprintln!("DEBUG: FieldAssignment on TypeNode::Named('{}'), looking for field '{}', suffix='{}'", account_type_name, field, namespace_suffix);
                            let account_fields = self.account_definitions.get(&account_type_name)
                                .or_else(|| {
                                    self.account_definitions.iter()
                                        .find(|(k, _)| k.ends_with(&namespace_suffix))
                                        .map(|(_, v)| v)
                                });
                            
                            if let Some(account_fields) = account_fields {
                                eprintln!("DEBUG: Resolved account_fields for '{}': {:?}", account_type_name, account_fields.iter().map(|f| &f.name).collect::<Vec<_>>());
                                if let Some(field_def) =
                                    account_fields.iter().find(|f| f.name == *field)
                                {
                                    if !field_def.is_mutable {
                                        // Permit mutation when the account parameter was declared with @mut
                                        let allow_via_account_mut = if let AstNode::Identifier(
                                            obj_name,
                                        ) = object.as_ref()
                                        {
                                            if let Some(writable) = &self.current_writable_accounts
                                            {
                                                writable.contains(obj_name)
                                            } else {
                                                false
                                            }
                                        } else {
                                            false
                                        };
                                        if !allow_via_account_mut {
                                            return Err(VMError::ImmutableField);
                                        }
                                    }
                                    // Handle optionality during assignment
                                    if field_def.is_optional {
                                        // If the field is optional, the assigned value can be compatible with the inner type
                                        // or it can be an Optional type itself (e.g., assigning None)
                                        if !self.types_are_compatible(
                                            &field_def.field_type,
                                            &value_type,
                                        ) && !(matches!(value_type, TypeNode::Generic { base, .. } if base == "Option"))
                                        {
                                            return Err(VMError::TypeMismatch);
                                        }
                                    } else {
                                        // If the field is not optional, the assigned value must be compatible
                                        // Special-case: assigning an account param to a pubkey field implies using its .key
                                        let mut compatible = self.types_are_compatible(
                                            &field_def.field_type,
                                            &value_type,
                                        );
                                        if !compatible {
                                            if let TypeNode::Primitive(ref name) =
                                                field_def.field_type
                                            {
                                                if name == "pubkey" {
                                                    // Allow when RHS is an account type (custom or Account)
                                                    let rhs_is_account = match &value_type {
                                                        TypeNode::Account => true,
                                                        TypeNode::Named(ref n) => {
                                                            self.account_definitions.contains_key(n) ||
                                                            self.account_definitions.keys().any(|k| k.ends_with(&format!("::{}", n)))
                                                        }
                                                        _ => false,
                                                    };
                                                    if rhs_is_account {
                                                        compatible = true;
                                                    }
                                                }
                                            }
                                        }
                                        if !compatible {
                                            // Special-case: allow numeric zero literal to initialize/clear pubkey fields
                                            if matches!(&field_def.field_type, TypeNode::Primitive(name) if name == "pubkey")
                                                && Self::is_zero_numeric_literal(value)
                                            {
                                                return Ok(());
                                            }
                                            // Allow numeric literal narrowing when it fits the target type
                                            if !(Self::numeric_literal_fits(
                                                &field_def.field_type,
                                                value,
                                            )) {
                                                eprintln!(
                                                    "Type error{}: cannot assign value of type '{}' to field '{}.{}' of type '{}'",
                                                    match &self.current_function { Some(f) => format!(" in function '{}'", f), None => "".to_string() },
                                                    Self::fmt_type(&value_type),
                                                    if let AstNode::Identifier(obj_name) = object.as_ref() { obj_name } else { "<expr>" },
                                                    field,
                                                    Self::fmt_type(&field_def.field_type)
                                                );
                                                return Err(VMError::TypeMismatch);
                                            }
                                        }
                                    }
                                } else {
                                    eprintln!("DEBUG: Field '{}' not found in account fields for '{}'", field, account_type_name);
                                    return Err(VMError::UndefinedField);
                                }
                            } else {
                                eprintln!("DEBUG: No account definition found for '{}'", account_type_name);
                                return Err(VMError::UndefinedField);
                            }
                        }
                        _ => {
                            eprintln!(
                                "Type error{}: cannot assign to field '{}.{}' on non-struct/non-account type '{}'",
                                match &self.current_function { Some(f) => format!(" in function '{}'", f), None => "".to_string() },
                                if let AstNode::Identifier(obj_name) = object.as_ref() { obj_name } else { "<expr>" },
                                field,
                                Self::fmt_type(&object_type)
                            );
                            return Err(VMError::TypeMismatch); // Other types don't support field assignment
                        }
                    }
                }

                Ok(())
            }
            AstNode::TupleAssignment { targets, value } => {
                let value_type = self.infer_type(value)?;
                if let TypeNode::Tuple {
                    elements: value_elements,
                } = value_type
                {
                    if targets.len() != value_elements.len() {
                        return Err(VMError::TypeMismatch); // Mismatched number of elements
                    }
                    for (i, target) in targets.iter().enumerate() {
                        let target_type = self.infer_type(target)?;
                        if !self.types_are_compatible(&target_type, &value_elements[i]) {
                            return Err(VMError::TypeMismatch); // Type mismatch for individual elements
                        }
                        // Recursively check types for targets (e.g., FieldAccess)
                        self.check_types(target)?;
                    }
                } else {
                    return Err(VMError::TypeMismatch); // Value is not a tuple
                }
                Ok(())
            }
            AstNode::LetStatement {
                name,
                type_annotation,
                value,
                is_mutable,
            } => {
                self.check_types(value)?;
                let inferred_type = self.infer_type(value)?;
                let final_type = if let Some(annotation) = type_annotation {
                    if !self.types_are_compatible(annotation, &inferred_type) {
                        // Allow numeric widening/promotion to annotated type
                        let allow_numeric = match (&**annotation, &inferred_type) {
                            (TypeNode::Primitive(an), TypeNode::Primitive(vn)) => {
                                Self::is_numeric_primitive_name(an)
                                    && Self::is_numeric_primitive_name(vn)
                            }
                            _ => false,
                        };
                        if !allow_numeric {
                            return Err(VMError::TypeMismatch);
                        }
                    }
                    *annotation.clone()
                } else {
                    inferred_type
                };
                self.symbol_table
                    .insert(name.clone(), (final_type.clone(), *is_mutable));

                // Preserve writable-account aliasing for casts/bindings like:
                // `let mut x = acc as MyAccount;` where `acc` is an @mut account parameter.
                if *is_mutable {
                    let source_ident = match value.as_ref() {
                        AstNode::Identifier(src) => Some(src.as_str()),
                        AstNode::Cast { value, .. } => match value.as_ref() {
                            AstNode::Identifier(src) => Some(src.as_str()),
                            _ => None,
                        },
                        _ => None,
                    };
                    if let (Some(source), Some(writable)) =
                        (source_ident, self.current_writable_accounts.as_mut())
                    {
                        if writable.contains(source) {
                            writable.insert(name.clone());
                        }
                    }
                }

                // Record definition for go-to-definition feature
                self.record_definition(
                    name.clone(),
                    final_type,
                    *is_mutable,
                    None, // TODO: Add position tracking to AST nodes
                );

                Ok(())
            }
            AstNode::TupleDestructuring { targets, value } => {
                let value_type = self.infer_type(value)?;
                if let TypeNode::Tuple { elements } = value_type {
                    if targets.len() != elements.len() {
                        return Err(VMError::InvalidParameterCount);
                    }
                    for (target, element_type) in targets.iter().zip(elements) {
                        self.symbol_table
                            .insert(target.clone(), (element_type.clone(), false));

                        // Record definition for go-to-definition feature
                        self.record_definition(
                            target.clone(),
                            element_type.clone(),
                            false, // Destructured targets are immutable
                            None, // TODO: Add position tracking to AST nodes
                        );
                    }
                } else {
                    return Err(VMError::TypeMismatch);
                }
                Ok(())
            }
            AstNode::IfStatement {
                condition,
                then_branch,
                else_branch,
            } => {
                // Check condition is boolean-like
                self.check_types(condition)?;

                // Check then branch
                self.check_types(then_branch)?;

                // Check else branch if present
                if let Some(else_stmt) = else_branch {
                    self.check_types(else_stmt)?;
                }

                Ok(())
            }
            AstNode::MatchExpression { expression, arms } => {
                // Check the expression being matched
                self.check_types(expression)?;

                // Check all match arms
                for arm in arms {
                    // Save current symbol table state
                    let original_symbol_table = self.symbol_table.clone();

                    // Extract pattern variables and add them to symbol table
                    self.extract_pattern_variables(&arm.pattern)?;

                    // Check guard if present (pattern variables are available)
                    if let Some(guard) = &arm.guard {
                        self.check_types(guard)?;
                    }

                    // Check arm body (pattern variables are available)
                    self.check_types(&arm.body)?;

                    // Restore original symbol table (removes pattern variables)
                    self.symbol_table = original_symbol_table;
                }

                Ok(())
            }
            AstNode::ReturnStatement { value } => {
                // Check return value if present
                if let Some(ret_val) = value {
                    self.check_types(ret_val)?;
                }

                Ok(())
            }
            AstNode::EmitStatement {
                event_name: _,
                fields,
            } => {
                // Type check all field assignments
                for field_assignment in fields {
                    self.check_types(&field_assignment.value)?;
                }
                Ok(())
            }
            AstNode::ForLoop {
                init,
                condition,
                update,
                body,
            } => {
                if let Some(init) = init {
                    self.check_types(init)?;
                }
                if let Some(condition) = condition {
                    self.check_types(condition)?;
                }
                if let Some(update) = update {
                    self.check_types(update)?;
                }
                self.check_types(body)?;
                Ok(())
            }
            AstNode::ForInLoop {
                variable: _,
                iterable,
                body,
            }
            | AstNode::ForOfLoop {
                variable: _,
                iterable,
                body,
            } => {
                self.check_types(iterable)?;
                self.check_types(body)?;
                Ok(())
            }
            AstNode::WhileLoop { condition, body } => {
                self.check_types(condition)?;
                self.check_types(body)?;
                Ok(())
            }
            AstNode::DoWhileLoop { body, condition } => {
                self.check_types(body)?;
                self.check_types(condition)?;
                Ok(())
            }
            AstNode::SwitchStatement {
                discriminant,
                cases,
                default_case,
            } => {
                self.check_types(discriminant)?;
                for case in cases {
                    self.check_types(&case.pattern)?;
                    for stmt in &case.body {
                        self.check_types(stmt)?;
                    }
                }
                if let Some(default) = default_case {
                    self.check_types(default)?;
                }
                Ok(())
            }
            AstNode::BreakStatement { label: _ } | AstNode::ContinueStatement { label: _ } => {
                Ok(())
            }
            AstNode::ArrowFunction {
                parameters: _,
                return_type: _,
                body,
                is_async: _,
            } => {
                self.check_types(body)?;
                Ok(())
            }
            AstNode::AssertStatement {
                assertion_type: _,
                args,
            } => {
                // Type check assertion arguments
                for arg in args {
                    self.check_types(arg)?;
                }
                Ok(())
            }
            _ => {
                // Not a statement node, delegate to main check_types
                Err(VMError::InvalidScript)
            }
        }
    }

    /// Extract pattern variables from match patterns and add them to symbol table
    pub(crate) fn extract_pattern_variables(&mut self, pattern: &AstNode) -> Result<(), VMError> {
        match pattern {
            // Handle constructor patterns like Ok(value), Some(data), Err(error)
            AstNode::FunctionCall { name, args } => {
                match name.as_str() {
                    "Ok" | "Some" | "Err" | "None" => {
                        // These are constructor patterns, extract variables from arguments
                        for arg in args {
                            self.extract_pattern_variables(arg)?;
                        }
                    }
                    _ => {
                        // Regular function call pattern - treat as variable if it's an identifier
                        return Err(VMError::InvalidScript); // Unexpected pattern
                    }
                }
            }
            // Handle identifier patterns (pattern variables)
            AstNode::Identifier(name) => {
                // Pattern variable: assign a generic type until inference is available.
                self.symbol_table.insert(
                    name.clone(),
                    (TypeNode::Primitive("u64".to_string()), false),
                );
            }
            // Handle literal patterns (no variables to extract)
            AstNode::Literal(_) => {
                // Literals don't introduce variables
            }
            _ => {
                // Other pattern types not implemented yet
                return Err(VMError::InvalidScript);
            }
        }
        Ok(())
    }
}
