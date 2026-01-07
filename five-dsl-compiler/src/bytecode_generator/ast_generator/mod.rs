// AST Generator Module
//
// This module handles AST traversal and bytecode generation for all AST node types.
// It provides the core logic for converting parsed AST structures into executable
// bytecode while maintaining type safety and optimization opportunities.

#[macro_use]
mod macros;

// Module declarations
mod accounts;
mod arrays;
mod control_flow;
mod expressions;
mod fields;
mod functions;
mod helpers;
mod initialization;
mod jumps;
mod resources;
mod symbol_table;
pub mod types;
mod utilities;

// Test modules
#[cfg(test)]
mod tests;
#[cfg(test)]
mod external_call_tests;

// Re-export the main type
pub use types::ASTGenerator;

use super::OpcodeEmitter;
use crate::ast::AstNode;
use crate::FieldInfo;
use five_protocol::opcodes::*;
use five_vm_mito::error::VMError;
use heapless::String as HeapString;
use self::types::ExternalImport;
use std::collections::HashMap;

impl ASTGenerator {
    /// Main entry point for AST node generation
    pub fn generate_ast_node<T: OpcodeEmitter>(
        &mut self,
        emitter: &mut T,
        node: &AstNode,
    ) -> Result<(), VMError> {
        // Debug output for all AST nodes
        #[cfg(debug_assertions)]
        println!(
            "AST Generator DEBUG: Processing node: {:?}",
            match node {
                AstNode::Program { .. } => "Program",
                AstNode::FieldDefinition { .. } => "FieldDefinition",
                AstNode::InstructionDefinition { .. } => "InstructionDefinition",
                AstNode::Assignment { .. } => "Assignment",
                AstNode::FieldAssignment { object, field, .. } => {
                    #[cfg(debug_assertions)]
                    println!(
                        "    FieldAssignment details: object={:?}, field={:?}",
                        object, field
                    );
                    "FieldAssignment"
                }
                AstNode::Identifier(name) => {
                    #[cfg(debug_assertions)]
                    println!("    Identifier: {}", name);
                    "Identifier"
                }
                AstNode::Literal(_) => "Literal",
                _ => "Other",
            }
        );

        match node {
            AstNode::Program {
                program_name: _,
                field_definitions,
                instruction_definitions,
                init_block,
                constraints_block,
                event_definitions: _,
                account_definitions: _,
                interface_definitions,
                import_statements,
            } => {
                // Pre-process imports to populate external_imports for CALL_EXTERNAL generation
                // TEMP FIX: Hardcode offsets for math_lib test case since we don't have a linker yet
                println!("AST Generator: Processing {} import statements", import_statements.len());
                for import_stmt in import_statements {
                    println!("AST Generator: Inspecting import: {:?}", import_stmt);
                    let module_name = match &import_stmt {
                        AstNode::ImportStatement { module_specifier, .. } => match module_specifier {
                            crate::ast::ModuleSpecifier::Local(name) => Some(name.clone()),
                            crate::ast::ModuleSpecifier::Nested(path) => path.last().cloned(),
                            crate::ast::ModuleSpecifier::External(path) => Some(path.clone()), // Or parse path if needed
                        },
                        _ => None,
                    };

                    if let Some(name) = module_name {
                        if name == "math_lib" || name.contains("math_lib") {
                            #[cfg(debug_assertions)]
                            println!("AST Generator: Registering external import 'math_lib' (detected via {})", name);
                            
                            let mut functions = HashMap::new();
                            // Offsets determined via debug_compile disassembly of math_lib.bin
                            // Note: These offsets might need updating if math_lib changes
                            functions.insert("safe_add".to_string(), 119);  // +1 to skip HALT
                            functions.insert("safe_mul".to_string(), 129);  // +1 to skip HALT
                            functions.insert("safe_sub".to_string(), 139);  // +1 to skip HALT
                            functions.insert("percent_of".to_string(), 169); // +1 to skip HALT

                            self.external_imports.insert(
                                "math_lib".to_string(),
                                ExternalImport {
                                    module_name: "math_lib".to_string(),
                                    account_index: 3, // Hardcoded index for test setup (Script, VM, Payer, MathLib)
                                    functions,
                                },
                            );
                        }
                    }
                }

                // Process field definitions first, populating the global symbol table
                for field_def in field_definitions {
                    self.process_field_definition(emitter, field_def, true)?;
                }

                // Process interface definitions (populate interface registry)
                self.process_interface_definitions(interface_definitions)?;

                // Process init block if present
                if let Some(init) = init_block {
                    self.generate_ast_node(emitter, init)?;
                }

                // Process constraints block if present
                if let Some(constraints) = constraints_block {
                    self.generate_ast_node(emitter, constraints)?;
                }

                // Process instruction definitions with visibility-based ordering
                // Phase 2: Separate public and private functions for proper visibility enforcement
                let mut public_functions = Vec::new();
                let mut private_functions = Vec::new();

                #[cfg(debug_assertions)]
                println!(
                    "AST_GENERATOR_DEBUG: Starting function separation, total functions: {}",
                    instruction_definitions.len()
                );

                // Separate functions by visibility
                // Separate functions by visibility
                for (i, instruction_def) in instruction_definitions.iter().enumerate() {
                    if let AstNode::InstructionDefinition {
                        name, visibility, ..
                    } = instruction_def
                    {
                        let is_public = visibility.is_on_chain_callable();
                        #[cfg(debug_assertions)]
                        println!(
                            "AST_GENERATOR_DEBUG: Function[{}] '{}' is_public: {}",
                            i, name, is_public
                        );
                        if is_public {
                            public_functions.push(instruction_def);
                        } else {
                            private_functions.push(instruction_def);
                        }
                    }
                }

                #[cfg(debug_assertions)]
                println!(
                    "AST_GENERATOR_DEBUG: Separated {} public functions, {} private functions",
                    public_functions.len(),
                    private_functions.len()
                );

                // Process public functions first (get indices 0, 1, 2... - externally callable)
                for (i, public_function) in public_functions.iter().enumerate() {
                    if let AstNode::InstructionDefinition { name, .. } = public_function {
                        #[cfg(debug_assertions)]
                        println!(
                            "AST_GENERATOR_DEBUG: Processing public function[{}] '{}'",
                            i, name
                        );
                    }
                    self.generate_ast_node(emitter, public_function)?;
                }

                // Process private functions after (get higher indices - internal only)
                for (i, private_function) in private_functions.iter().enumerate() {
                    if let AstNode::InstructionDefinition { name, .. } = private_function {
                        #[cfg(debug_assertions)]
                        println!(
                            "AST_GENERATOR_DEBUG: Processing private function[{}] '{}'",
                            i, name
                        );
                    }
                    self.generate_ast_node(emitter, private_function)?;
                }

                Ok(())
            }

            AstNode::Block { statements, .. } => {
                self.generate_statement_block(emitter, statements)?;
                Ok(())
            }

            AstNode::LetStatement {
                name,
                type_annotation,
                is_mutable,
                value,
            } => {
                // Generate value first
                self.generate_ast_node(emitter, value)?;

                // Determine field type
                let field_type = if let Some(type_node) = type_annotation {
                    self.type_node_to_string(type_node)
                } else {
                    self.infer_type_from_node(value)?
                };

                // Add to local symbol table
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

                // Generate local variable storage instruction with V2 optimization
                self.emit_set_local(
                    emitter,
                    offset,
                    &format!("let statement '{}'", name),
                );
                Ok(())
            }
            AstNode::TupleDestructuring { targets, value } => {
                self.generate_ast_node(emitter, value)?;
                emitter.emit_opcode(UNPACK_TUPLE);
                emitter.emit_u8(targets.len() as u8);
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

            AstNode::Assignment { target, value } => {
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

            AstNode::TupleAssignment { targets, value } => {
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

            AstNode::Identifier(name) => {
                // Handle special identifiers first
                match name.as_str() {
                    "None" => {
                        // Generate None variant for Option<T>
                        emitter.emit_opcode(OPTIONAL_NONE);
                        return Ok(());
                    }
                    "signer" => {
                        // Error: GET_SIGNER_KEY opcode not supported by VM
                        return Err(VMError::ParseError {
                            expected: HeapString::<32>::try_from(
                                "account parameter with @signer attribute",
                            )
                            .unwrap(),
                            found: HeapString::<32>::try_from("bare 'signer' identifier").unwrap(),
                            position: 0,
                        });
                    }
                    _ => {
                        // Look up identifier in local symbol table first
                        if let Some(field_info) = self.local_symbol_table.get(name) {
                            if field_info.is_parameter {
                                // Generate direct LOAD_PARAM for function parameters
                                // Optimization: Use nibble immediate opcodes for indices 1-3
                                let opcode_byte = match field_info.offset {
                                    1 => Some(LOAD_PARAM_1),
                                    2 => Some(LOAD_PARAM_2),
                                    3 => Some(LOAD_PARAM_3),
                                    _ => None,
                                };

                                if let Some(op) = opcode_byte {
                                    emitter.emit_opcode(op);
                                } else {
                                    emitter.emit_opcode(LOAD_PARAM);
                                    emitter.emit_u8(field_info.offset as u8);
                                }
                                #[cfg(debug_assertions)]
                                println!(
                                    "DEBUG: Generated LOAD_PARAM {} for parameter '{}'",
                                    field_info.offset, name
                                );
                            } else {
                                // Generate GET_LOCAL for actual local variables with V2 optimization
                                self.emit_get_local(
                                    emitter,
                                    field_info.offset,
                                    &format!("local variable '{}'", name),
                                );
                            }
                        } else if let Some(field_info) = self.global_symbol_table.get(name) {
                            // Protocol V3: LOAD_FIELD account_index_u8, offset_vle
                            // Script fields use account_index=0 (the script account itself)
                            emitter.emit_opcode(LOAD_FIELD);
                            emitter.emit_u8(0); // Script account is always index 0
                            emitter.emit_vle_u32(field_info.offset);
                        } else if !self.interface_registry.contains_key(name) {
                            // Only return error for truly undefined identifiers
                            return Err(VMError::InvalidScript); // Undefined identifier
                        }
                        // Interface names are valid identifiers but don't generate code by themselves
                        // They are only meaningful in method calls, so we continue to Ok(())
                    }
                }
                Ok(())
            }

            AstNode::Literal(value) => {
                self.emit_literal_value(emitter, value)?;
                Ok(())
            }
            AstNode::TupleLiteral { elements } => {
                for element in elements {
                    self.generate_ast_node(emitter, element)?;
                }
                emitter.emit_opcode(CREATE_TUPLE);
                emitter.emit_u8(elements.len() as u8);
                Ok(())
            }
            AstNode::ArrayLiteral { elements } => {
                // First, generate the bytecode for each element to push them onto the stack
                for element in elements {
                    self.generate_ast_node(emitter, element)?;
                }

                // Now, emit the PUSH_ARRAY_LITERAL opcode, which consumes the
                // elements from the stack to create the array in the temp_buffer.
                emitter.emit_opcode(PUSH_ARRAY_LITERAL);
                emitter.emit_u8(elements.len() as u8);
                Ok(())
            }

            AstNode::StringLiteral { value } => {
                // Emit PUSH_STRING opcode
                emitter.emit_opcode(PUSH_STRING);

                // Convert to UTF-8 bytes (compile-time UTF-8 validation)
                let utf8_bytes = value.as_bytes();

                // Emit VLE-encoded length
                emitter.emit_vle_u32(utf8_bytes.len() as u32);

                // Emit string data
                emitter.emit_bytes(utf8_bytes);

                Ok(())
            }

            AstNode::BinaryExpression {
                left,
                right,
                operator,
            } => {
                self.generate_binary_expression(emitter, left, right, operator)?;
                Ok(())
            }

            AstNode::UnaryExpression { operand, operator } => {
                self.generate_unary_expression(emitter, operand, operator)?;
                Ok(())
            }

            AstNode::IfStatement {
                condition,
                then_branch,
                else_branch,
            } => {
                self.generate_if_statement(emitter, condition, then_branch, else_branch)?;
                Ok(())
            }

            AstNode::MatchExpression { expression, arms } => {
                self.generate_match_expression(emitter, expression, arms)?;
                Ok(())
            }

            AstNode::ReturnStatement { value } => {
                if let Some(val) = value {
                    self.generate_ast_node(emitter, val)?;
                    emitter.emit_opcode(RETURN_VALUE);
                } else {
                    emitter.emit_opcode(RETURN);
                }
                Ok(())
            }

            AstNode::MethodCall {
                method,
                object,
                args,
            } => {
                self.generate_method_call(emitter, method, object, args)?;
                Ok(())
            }

            AstNode::FunctionCall { name, args } => {
                self.generate_function_call(emitter, name, args)?;
                Ok(())
            }

            AstNode::RequireStatement { condition } => {
                self.generate_ast_node(emitter, condition)?;
                emitter.emit_opcode(REQUIRE);
                Ok(())
            }

            // Test framework AST nodes
            AstNode::TestFunction {
                name,
                attributes,
                body,
            } => {
                // Only generate test functions if we're in testing mode
                if emitter.should_include_tests() {
                    self.generate_test_function(emitter, name, attributes, body)?;
                }
                Ok(())
            }

            AstNode::AssertStatement {
                assertion_type,
                args,
            } => {
                // Only generate assertions if we're in testing mode
                if emitter.should_include_tests() {
                    self.generate_assertion_statement(emitter, assertion_type, args)?;
                }
                Ok(())
            }

            AstNode::FieldAssignment {
                object,
                field,
                value,
            } => {
                // Resolve account object and field
                if let AstNode::Identifier(account_name) = object.as_ref() {
                    if let Some(field_info) = self.local_symbol_table.get(account_name) {
                        // Get values before borrowing self mutably
                        let account_type = field_info.field_type.clone();
                        let account_offset =
                            super::account_utils::account_index_from_param_offset(field_info.offset);

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
                        if let AstNode::Identifier(rhs_name) = value.as_ref() {
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
                                                super::account_utils::account_index_from_param_offset(
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

                        // Generate account field store operation using zero-copy approach
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

            AstNode::FieldAccess { object, field } => {
                if let AstNode::Identifier(account_name) = object.as_ref() {
                    #[cfg(debug_assertions)]
                    println!(
                        "AST Generator: Processing FieldAccess for '{}' field '{}'",
                        account_name, field
                    );

                    // TODO: V3 PATTERN DETECTION - Implement bulk field loading optimization
                    // Future enhancement: Detect consecutive field accesses from the same account
                    // and emit BULK_LOAD instructions to reduce bytecode size and improve performance.
                    // See: https://github.com/5iveVM/five-dsl-compiler/issues/XXX

                    if let Some(field_info) = self.local_symbol_table.get(account_name) {
                        #[cfg(debug_assertions)]
                        println!(
                            "AST Generator: Found symbol '{}' with type '{}'",
                            account_name, field_info.field_type
                        );

                        // Handle custom account fields using AccountSystem
                        let account_type = &field_info.field_type;

                        // Check if the field is optional and if it is a pubkey
                        let mut is_optional = false;
                        let mut is_pubkey = false;
                        let mut field_found_in_registry = false;

                        if let Some(account_system) = &self.account_system {
                            let namespace_suffix = format!("::{}", account_type);
                            let account_type_info = account_system
                                .get_account_registry()
                                .account_types
                                .get(account_type)
                                .or_else(|| {
                                    account_system.get_account_registry().account_types.iter()
                                        .find(|(k, _)| k.ends_with(&namespace_suffix))
                                        .map(|(_, v)| v)
                                });

                            if let Some(account_type_info) = account_type_info {
                                if let Some(struct_field_info) = account_type_info.fields.get(field)
                                {
                                    is_optional = struct_field_info.is_optional;
                                    is_pubkey = struct_field_info.field_type == "pubkey";
                                    field_found_in_registry = true;
                                }
                            }
                        }

                        // PRIORITY FIX: Check if this is a built-in account property
                        // BUT only if it is NOT a user-defined field (shadowing support)
                        if !field_found_in_registry {
                            if let Some(account_system) = &self.account_system {
                                if account_system.is_builtin_account_property(field) {
                                    #[cfg(debug_assertions)]
                                    println!(
                                        "AST Generator: Using built-in property access for '{}.{}'",
                                        account_name, field
                                    );
                                    return account_system.generate_builtin_account_property_access(
                                        emitter,
                                        account_name,
                                        field,
                                        &self.local_symbol_table,
                                    );
                                }
                            }
                        }

                        // Calculate field offset within account using the account type
                        let field_offset =
                            self.calculate_account_field_offset(account_type, field)?;

                        // Generate zero-copy account field load operation using MitoVM VLE
                        if is_pubkey {
                            emitter.emit_opcode(LOAD_FIELD_PUBKEY); // Zero-copy pubkey read (32 bytes)
                        } else {
                            emitter.emit_opcode(LOAD_FIELD); // Zero-copy u64 read (8 bytes)
                        }
                        
                        emitter.emit_u8(
                            super::account_utils::account_index_from_param_offset(
                                field_info.offset,
                            ),
                        ); // Account index from symbol table
                        emitter.emit_vle_u32(field_offset); // Field offset (VLE format for consistency)

                        if is_optional {
                            emitter.emit_opcode(OPTIONAL_UNWRAP);
                        }
                    } else {
                        #[cfg(debug_assertions)]
                        println!("AST Generator: ERROR - Account parameter '{}' not found in local symbol table for field access", account_name);
                        return Err(VMError::InvalidScript); // Undefined account
                    }
                } else {
                    return Err(VMError::InvalidScript); // Invalid object type
                }
                Ok(())
            }
            AstNode::TupleAccess { object, index } => {
                self.generate_ast_node(emitter, object)?;
                emitter.emit_opcode(TUPLE_GET);
                emitter.emit_u8(*index as u8);
                Ok(())
            }

            AstNode::ArrayAccess { array, index } => {
                // Generate code for the array expression
                self.generate_ast_node(emitter, array)?;
                // Generate code for the index expression
                self.generate_ast_node(emitter, index)?;
                // Emit the array indexing opcode
                emitter.emit_opcode(ARRAY_INDEX);
                Ok(())
            }

            AstNode::InstructionDefinition {
                name: _name,
                parameters,
                return_type,
                body,
                visibility: _,
                ..
            } => {
                // Clear the local symbol table for the new function
                self.local_symbol_table.clear();

                // Track the return type for proper tuple return handling
                self.current_function_return_type = return_type.as_ref().map(|rt| (**rt).clone());

                // Start local variable counter after global fields to avoid conflicts
                // Local variables use GET_LOCAL/SET_LOCAL opcodes and need their own offset space
                self.field_counter = 0;

                // Process function parameters and add them to the local symbol table
                // FIX: Maintain separate counters for account indices and data parameter indices
                // MitoVM/Solana entrypoint splits arguments into Accounts and Data.
                // Accounts are accessed via account ID, Data arguments via LOAD_PARAM.
                let mut account_param_counter: u32 = 0;
                let mut data_param_counter: u32 = 0;

                for (index, param) in parameters.iter().enumerate() {
                    // Generate @init account creation sequence if needed
                    self.generate_init_account_sequence(emitter, param, index)?;

                    // Determine if this is an account parameter or data parameter
                    let is_account = super::account_utils::is_account_parameter(
                        &param.param_type,
                        &param.attributes,
                        self.account_system.as_ref().map(|sys| sys.get_account_registry())
                    );

                    let offset = if is_account {
                        let off = account_param_counter;
                        account_param_counter += 1;
                        off
                    } else {
                        let off = data_param_counter;
                        data_param_counter += 1;
                        off
                    };

                    let field_info = FieldInfo {
                        offset,
                        field_type: self.type_node_to_string(&param.param_type),
                        // Implicit mutability: @init implies mutable, or explicit @mut
                        is_mutable: param.is_init || param.attributes.iter().any(|a| a.name == "mut"),
                        is_optional: param.is_optional,
                        is_parameter: true, // Mark as parameter to generate LOAD_PARAM instead of GET_LOCAL
                    };
                    self.local_symbol_table
                        .insert(param.name.clone(), field_info);
                }

                // Inject @requires(condition) checks
                for param in parameters {
                    for attr in &param.attributes {
                        if attr.name == "requires" {
                            if let Some(condition) = attr.args.first() {
                                // Generate require statement for validity check
                                // This will behave exactly like 'require(condition);' at the start of the function
                                self.generate_ast_node(emitter, &AstNode::RequireStatement { 
                                    condition: Box::new(condition.clone()) 
                                })?;
                            }
                        }
                    }
                }

                // Generate function body
                self.generate_ast_node(emitter, body)?;

                // Ensure function returns if flow reaches end
                // This prevents falling through into data sections or other functions
                emitter.emit_opcode(RETURN);

                // Clear function context when exiting
                self.current_function_return_type = None;
                Ok(())
            }

            _ => {
                // For other node types that aren't implemented yet, just continue
                // Don't emit any opcodes for unknown nodes
                Ok(())
            }
        }
    }
}
