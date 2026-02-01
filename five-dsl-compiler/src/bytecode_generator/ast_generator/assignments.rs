//! Assignment and variable declaration generation
//!
//! Handles generation for let statements, assignments, tuple destructuring,
//! and field assignments.

use super::types::ASTGenerator;
use super::super::OpcodeEmitter;
use crate::ast::{AstNode, TypeNode};
use crate::FieldInfo;
use five_protocol::opcodes::*;
use five_vm_mito::error::VMError;

impl ASTGenerator {
    pub(super) fn generate_let_statement<T: OpcodeEmitter>(
        &mut self,
        emitter: &mut T,
        name: &String,
        type_annotation: &Option<Box<TypeNode>>,
        is_mutable: &bool,
        value: &AstNode,
    ) -> Result<(), VMError> {
        // Determine field type
        let field_type = if let Some(type_node) = type_annotation {
            self.type_node_to_string(type_node)
        } else {
            self.infer_type_from_node(value)?
        };

        // Add to local symbol table based on allocations
        let offset = if let Some(allocs) = &self.precomputed_allocations {
            if let Some(&alloc_offset) = allocs.get(name) {
                let offset = alloc_offset as u32;
                if self.field_counter <= offset {
                    self.field_counter = offset + 1;
                }
                offset
            } else {
                let off = self.field_counter;
                self.field_counter += 1;
                off
            }
        } else {
            let off = self.field_counter;
            self.field_counter += 1;
            off
        };

        let field_info = FieldInfo {
            offset,
            field_type,
            is_mutable: *is_mutable,
            is_optional: false, // Let statements are not optional
            is_parameter: false,
        };

        self.local_symbol_table.insert(name.clone(), field_info);

        // If register optimization enabled, try to map local variable to register
        if self.use_registers {
            if let Some(reg) = self.register_allocator.map_local(name) {
                // Try to generate value directly into register for field loads
                if let Some((acc_idx, field_offset)) = self.match_u64_field_access(value) {
                    #[cfg(debug_assertions)]
                    println!("DEBUG: Mapping '{}' to register r{} with LOAD_FIELD_REG", name, reg);

                    emitter.emit_opcode(LOAD_FIELD_REG);
                    emitter.emit_u8(reg);
                    emitter.emit_u8(acc_idx);
                    emitter.emit_vle_u32(field_offset);
                    return Ok(());
                }

                // For other values, generate to stack then move to register
                self.generate_ast_node(emitter, value)?;
                emitter.emit_opcode(POP_REG);
                emitter.emit_u8(reg);
                return Ok(());
            }
        }

        // Fallback: stack-based local variable (no register optimization)
        // Generate value first
        self.generate_ast_node(emitter, value)?;

        // Generate local variable storage instruction with V2 optimization
        self.emit_set_local(
            emitter,
            offset,
            &format!("let statement '{}'", name),
        );
        Ok(())
    }

    pub(super) fn generate_tuple_destructuring<T: OpcodeEmitter>(
        &mut self,
        emitter: &mut T,
        targets: &[String],
        value: &AstNode,
    ) -> Result<(), VMError> {
        self.generate_ast_node(emitter, value)?;
        emitter.emit_opcode(UNPACK_TUPLE);
        emitter.emit_u8(targets.len() as u8);

        // Track max offset used to update field_counter if necessary
        let mut max_offset_used = self.field_counter;
        let using_precomputed = self.precomputed_allocations.is_some();

        for (i, target) in targets.iter().enumerate().rev() {
            let offset = if let Some(allocs) = &self.precomputed_allocations {
                if let Some(&alloc_offset) = allocs.get(target) {
                    alloc_offset as u32
                } else {
                    // Fallback if not in precomputed map (shouldn't happen if analyzer ran)
                    let off = self.field_counter + i as u32;
                    if off >= max_offset_used { max_offset_used = off + 1; }
                    off
                }
            } else {
                let off = self.field_counter + i as u32;
                max_offset_used = self.field_counter + targets.len() as u32;
                off
            };

            // Update high water mark if using precomputed
            if using_precomputed && offset >= self.field_counter && offset >= max_offset_used { max_offset_used = offset + 1; }

            let field_info = FieldInfo {
                offset,
                field_type: "unknown".to_string(), // Type is not known at this point
                is_mutable: true,
                is_optional: false,
                is_parameter: false, // This is a local variable, not a parameter
            };
            self.local_symbol_table.insert(target.clone(), field_info);
            // Use helper for V2 optimization consistency
            self.emit_set_local(
                emitter,
                offset,
                &format!("tuple destructuring '{}'", target),
            );
        }

        self.field_counter = max_offset_used;
        Ok(())
    }

    pub(super) fn generate_assignment<T: OpcodeEmitter>(
        &mut self,
        emitter: &mut T,
        target: &String,
        value: &AstNode,
    ) -> Result<(), VMError> {
        // Generate value expression
        self.generate_ast_node(emitter, value)?;

        // Look up target in local symbol table
        if let Some(field_info) = self.local_symbol_table.get(target) {
            // Allow assignments to immutable fields in init blocks (constructor)
            let is_init_context = self
                .current_function_context
                .as_ref()
                .is_some_and(|name| name == "__init");
            if !field_info.is_mutable && !is_init_context {
                return Err(VMError::InvalidScript); // Attempting to assign to immutable field
            }

            // Optimization: use nibble immediate opcodes for indices 0-3
            if field_info.offset <= 3 {
                match field_info.offset {
                    0 => emitter.emit_opcode(SET_LOCAL_0),
                    1 => emitter.emit_opcode(SET_LOCAL_1),
                    2 => emitter.emit_opcode(SET_LOCAL_2),
                    3 => emitter.emit_opcode(SET_LOCAL_3),
                    _ => unreachable!("Index checked to be 0-3"),
                }
                #[cfg(debug_assertions)]
                println!("DEBUG: Generated SET_LOCAL_{} (nibble immediate) for assignment to '{}'", field_info.offset, target);
            } else {
                // Standard SET_LOCAL with index parameter
                emitter.emit_opcode(SET_LOCAL);
                emitter.emit_u8(field_info.offset as u8);
                #[cfg(debug_assertions)]
                println!(
                    "DEBUG: Generated SET_LOCAL {} for assignment to '{}'",
                    field_info.offset, target
                );
            };
        } else if let Some(field_info) = self.global_symbol_table.get(target) {
            // It's a global script field, so we need to store it
            // Allow assignments to immutable fields in init blocks (constructor)
            let is_init_context = self
                .current_function_context
                .as_ref()
                .is_some_and(|name| name == "__init");
            if !field_info.is_mutable && !is_init_context {
                return Err(VMError::InvalidScript); // Attempting to assign to immutable field
            }

            // Protocol V3: STORE_FIELD account_index_u8, offset_vle
            // Script fields use account_index=0 (the script account itself)
            emitter.emit_opcode(STORE_FIELD);
            emitter.emit_u8(0); // Script account is always index 0
            emitter.emit_vle_u32(field_info.offset);
        } else {
            // Create new local variable (original fallback when no dispatcher)
            let field_info = FieldInfo {
                offset: self.field_counter,
                field_type: self.infer_type_from_node(value)?,
                is_mutable: true, // Assignments create mutable fields
                is_optional: false,
                is_parameter: false, // This is a local variable, not a parameter
            };

            self.local_symbol_table.insert(target.clone(), field_info);

            // Generate local variable storage with V2 optimization
            self.emit_set_local(
                emitter,
                self.field_counter,
                &format!("new local variable '{}'", target),
            );

            self.field_counter += 1;
        }

        Ok(())
    }

    pub(super) fn generate_tuple_assignment<T: OpcodeEmitter>(
        &mut self,
        emitter: &mut T,
        targets: &[AstNode],
        value: &AstNode,
    ) -> Result<(), VMError> {
        // Generate the value expression first
        self.generate_ast_node(emitter, value)?;

        // For each target, generate an assignment
        for target_node in targets {
            // Duplicate the value on the stack for each assignment
            emitter.emit_opcode(DUP);

            match target_node {
                AstNode::Identifier(name) => {
                    // Handle simple identifier assignment
                    if let Some(field_info) = self.local_symbol_table.get(name) {
                        if !field_info.is_mutable {
                            return Err(VMError::InvalidScript); // Attempting to assign to immutable field
                        }
                        self.emit_set_local(
                            emitter,
                            field_info.offset,
                            &format!("tuple assignment '{}'", name),
                        );
                    } else {
                        // Create new local variable
                        let offset = self.add_local_field(
                            name.clone(),
                            self.infer_type_from_node(target_node)?,
                            true,  // mutable
                            false, // not optional
                        );
                        self.emit_set_local(
                            emitter,
                            offset,
                            &format!("tuple assignment '{}'", name),
                        );
                    }
                }
                AstNode::FieldAccess { object, field } => {
                    // Handle field assignment (e.g., account.field = value)
                    if let AstNode::Identifier(account_name) = object.as_ref() {
                        if let Some(field_info) = self.local_symbol_table.get(account_name)
                        {
                            let field_offset = self.calculate_account_field_offset(
                                &field_info.field_type,
                                field,
                            )?; // Pass account_type string
                            emitter.emit_opcode(STORE_FIELD); // Use STORE_FIELD for now, assuming it handles account fields
                            emitter.emit_u8(field_info.offset as u8);
                            emitter.emit_vle_u32(field_offset);
                        } else {
                            return Err(VMError::InvalidScript); // Undefined account
                        }
                    } else {
                        return Err(VMError::InvalidScript); // Invalid object type for field access
                    }
                }
                _ => return Err(VMError::InvalidScript), // Only Identifier or FieldAccess allowed as targets
            }
        }
        // Pop the last duplicated value (since it was duplicated one extra time)
        emitter.emit_opcode(POP);
        Ok(())
    }

    pub(super) fn generate_field_assignment<T: OpcodeEmitter>(
        &mut self,
        emitter: &mut T,
        object: &AstNode,
        field: &String,
        value: &AstNode,
    ) -> Result<(), VMError> {
        // Resolve account object and field
        if let AstNode::Identifier(account_name) = object {
            if let Some(field_info) = self.local_symbol_table.get(account_name) {
                // Get values before borrowing self mutably
                let account_type = field_info.field_type.clone();
                let account_offset =
                    super::super::account_utils::account_index_from_param_offset(field_info.offset);

                // If this field name is a built-in account property name AND it's not a user-defined field
                // of this custom account type, then disallow assignment.
                if let Some(account_system) = &self.account_system {
                    if account_system.is_builtin_account_property(field) {
                        // Check registry for a user-defined field with the same name on this account type (namespace-aware)
                        let namespace_suffix = format!("::{}", account_type);
                        let account_info = account_system
                            .get_account_registry()
                            .account_types
                            .get(&account_type)
                            .or_else(|| {
                                account_system.get_account_registry().account_types.iter()
                                    .find(|(k, _)| k.ends_with(&namespace_suffix))
                                    .map(|(_, v)| v)
                            });

                        let is_custom_field = account_info
                            .and_then(|t| t.fields.get(field))
                            .is_some();
                        if !is_custom_field {
                            #[cfg(debug_assertions)]
                            println!("AST Generator: ERROR - Cannot assign to built-in account property '{}.{}'", account_name, field);
                            return Err(VMError::ImmutableField);
                        }
                    }
                }

                // If assigning an account parameter (RHS) into a pubkey-typed field, use the account's key implicitly
                let mut value_emitted = false;
                if let AstNode::Identifier(rhs_name) = value {
                    // Determine target field type (e.g., pubkey)
                    let mut target_field_type: Option<String> = None;
                    if let Some(account_system) = &self.account_system {
                        let namespace_suffix = format!("::{}", account_type);
                        let account_type_info = account_system
                            .get_account_registry()
                            .account_types
                            .get(&account_type)
                            .or_else(|| {
                                account_system.get_account_registry().account_types.iter()
                                    .find(|(k, _)| k.ends_with(&namespace_suffix))
                                    .map(|(_, v)| v)
                            });

                        if let Some(account_type_info) = account_type_info {
                            if let Some(struct_field_info) =
                                account_type_info.fields.get(field)
                            {
                                target_field_type =
                                    Some(struct_field_info.field_type.clone());
                            }
                        }
                    }

                    // If rhs is an account parameter and target expects pubkey, emit GET_KEY rhs_param_index
                    if let Some(rhs_info) = self.local_symbol_table.get(rhs_name) {
                        let rhs_is_account_param =
                            if let Some(account_system) = &self.account_system {
                                rhs_info.is_parameter
                                    && account_system.is_account_type(&rhs_info.field_type)
                            } else {
                                false
                            };
                        if rhs_is_account_param {
                            if let Some(t) = target_field_type {
                                if t == "pubkey" {
                                    // Use GET_KEY on the RHS account parameter instead of pushing the account ref
                                    emitter.emit_opcode(GET_KEY);
                                    emitter.emit_u8(
                                        super::super::account_utils::account_index_from_param_offset(
                                            rhs_info.offset,
                                        ),
                                    );
                                    value_emitted = true;
                                }
                            }
                        }
                    }
                }

                // Fallback: generate the value expression normally if we didn't handle the RHS specially
                if !value_emitted {
                    self.generate_ast_node(emitter, value)?;
                }

                // Handle custom account fields using AccountSystem

                // Check if the field is optional
                let is_optional = if let Some(account_system) = &self.account_system {
                    let namespace_suffix = format!("::{}", account_type);
                    let account_type_info = account_system
                        .get_account_registry()
                        .account_types
                        .get(&account_type)
                        .or_else(|| {
                            account_system.get_account_registry().account_types.iter()
                                .find(|(k, _)| k.ends_with(&namespace_suffix))
                                .map(|(_, v)| v)
                        });

                    if let Some(account_type_info) = account_type_info {
                        if let Some(struct_field_info) = account_type_info.fields.get(field)
                        {
                            struct_field_info.is_optional
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else {
                    false
                };

                if is_optional {
                    emitter.emit_opcode(OPTIONAL_SOME);
                }

                // Calculate field offset within account using the account type
                let field_offset =
                    self.calculate_account_field_offset(&account_type, field)?;

                // Debug logging for zero-copy account access
                #[cfg(debug_assertions)]
                println!("DSL Compiler DEBUG: FieldAssignment for account '{}', mapped to index {}, field_offset {}",
                    account_name, account_offset, field_offset);

                // Generate account field store operation using zero-copy account field store
                emitter.emit_opcode(STORE_FIELD); // MitoVM zero-copy account field store
                emitter.emit_u8(account_offset); // Account index from symbol table
                emitter.emit_vle_u32(field_offset); // Field offset (VLE format for consistency)
            } else {
                // ENHANCED ERROR HANDLING: Check if this might be a script field assignment
                #[cfg(debug_assertions)]
                println!("AST Generator: DIAGNOSTIC - Object '{}' not found in local symbol table for field assignment '{}'", account_name, field);
                #[cfg(debug_assertions)]
                println!("AST Generator: DIAGNOSTIC - Local symbol table contents:");
                for (key, value) in &self.local_symbol_table {
                    #[cfg(debug_assertions)]
                    println!(
                        "  '{}' -> offset: {}, type: '{}'",
                        key, value.offset, value.field_type
                    );
                }

                // Check if this is a script-level field assignment
                if self.global_symbol_table.contains_key(account_name) {
                    #[cfg(debug_assertions)]
                    println!(
                        "AST Generator: Processing script field assignment: {}",
                        account_name
                    );

                    // This is a script field assignment - generate value first
                    self.generate_ast_node(emitter, value)?;

                    // Get script field info from global symbol table
                    if let Some(script_field_info) =
                        self.global_symbol_table.get(account_name)
                    {
                        // Protocol V3: STORE_FIELD account_index_u8, offset_vle
                        emitter.emit_opcode(STORE_FIELD);
                        emitter.emit_u8(0); // Script account is always index 0
                        emitter.emit_vle_u32(script_field_info.offset);

                        #[cfg(debug_assertions)]
                        println!("AST Generator: Generated script field store for '{}' at offset {}", account_name, script_field_info.offset);
                        return Ok(());
                    } else {
                        return Err(VMError::UndefinedField);
                    }
                } else {
                    #[cfg(debug_assertions)]
                    println!(
                        "AST Generator: ERROR - Object '{}' not found in any symbol table",
                        account_name
                    );
                    return Err(VMError::UndefinedIdentifier); // Object not found anywhere
                }
            }
        } else {
            return Err(VMError::InvalidScript); // Invalid object type
        }
        Ok(())
    }
}
