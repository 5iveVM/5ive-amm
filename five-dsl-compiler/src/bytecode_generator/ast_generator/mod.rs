// AST traversal and bytecode generation.

#[macro_use]
mod macros;

// Module declarations
mod accounts;
mod arrays;
mod control_flow;
mod expressions;
mod fields;
mod functions;
mod fused_opcodes;
mod helpers;
mod initialization;
mod jumps;
mod resources;
mod symbol_table;
pub mod types;
mod utilities;

// New modules
mod assignments;
mod program;

// Test modules
// #[cfg(test)]
// mod tests;
// #[cfg(test)]
// mod external_call_tests;

// Re-export the main type
pub use types::ASTGenerator;

use super::OpcodeEmitter;
use crate::ast::AstNode;
use crate::FieldInfo;
use five_protocol::opcodes::*;
use five_vm_mito::error::VMError;
use heapless::String as HeapString;

impl ASTGenerator {
    /// Main entry point for AST node generation
    pub fn generate_ast_node<T: OpcodeEmitter>(
        &mut self,
        emitter: &mut T,
        node: &AstNode,
    ) -> Result<(), VMError> {
        #[cfg(debug_assertions)]
        println!("Processing node: {:?}", std::mem::discriminant(node));

        match node {
            AstNode::Program {
                program_name: _,
                field_definitions,
                instruction_definitions,
                init_block,
                constraints_block,
                event_definitions: _,
                account_definitions: _,
                type_definitions,
                interface_definitions,
                import_statements,
            } => self.generate_program(
                emitter,
                import_statements,
                field_definitions,
                type_definitions,
                interface_definitions,
                init_block,
                constraints_block,
                instruction_definitions,
            ),

            AstNode::Block { statements, .. } => {
                self.generate_statement_block(emitter, statements)?;
                Ok(())
            }

            AstNode::LetStatement {
                name,
                type_annotation,
                is_mutable,
                value,
            } => self.generate_let_statement(emitter, name, type_annotation, is_mutable, value),
            AstNode::TupleDestructuring { targets, value } => {
                self.generate_tuple_destructuring(emitter, targets, value)
            }

            AstNode::Assignment { target, value } => {
                self.generate_assignment(emitter, target, value)
            }

            AstNode::TupleAssignment { targets, value } => {
                self.generate_tuple_assignment(emitter, targets, value)
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
                        // Account-typed function parameters must resolve to runtime account refs.
                        if let Some(account_idx) = self.resolve_account_param_by_name(name) {
                            emitter.emit_opcode(GET_ACCOUNT);
                            emitter.emit_u8(account_idx);
                            return Ok(());
                        }

                        // Look up identifier in local symbol table first
                        if let Some(field_info) = self.local_symbol_table.get(name) {
                            if field_info.is_parameter {
                                if name == "decimals" {
                                    println!("DEBUG_COMPILER: Found parameter 'decimals' offset={} index={} is_param={}",
                                        field_info.offset, field_info.offset + 1, field_info.is_parameter);
                                }
                                // Generate direct LOAD_PARAM for function parameters
                                // Use nibble immediate opcodes.
                                // FIX: Use 1-based indexing for LOAD_PARAM (offset 0 -> index 1)
                                let param_index = field_info.offset + 1;

                                let opcode_byte = match param_index {
                                    1 => Some(LOAD_PARAM_1),
                                    2 => Some(LOAD_PARAM_2),
                                    3 => Some(LOAD_PARAM_3),
                                    _ => None,
                                };

                                if let Some(op) = opcode_byte {
                                    emitter.emit_opcode(op);
                                } else {
                                    emitter.emit_opcode(LOAD_PARAM);
                                    emitter.emit_u8(param_index as u8);
                                }
                                #[cfg(debug_assertions)]
                                println!(
                                    "DEBUG: Generated LOAD_PARAM {} for parameter '{}'",
                                    param_index, name
                                );
                            } else {
                                // Generate GET_LOCAL for actual local variables
                                self.emit_get_local(
                                    emitter,
                                    field_info.offset,
                                    &format!("local variable '{}'", name),
                                );
                            }
                        } else if let Some(field_info) = self.global_symbol_table.get(name) {
                            // Script fields use account_index=0 (the script account itself)
                            emitter.emit_opcode(LOAD_FIELD);
                            emitter.emit_u8(0);
                            emitter.emit_u32(field_info.offset);
                        } else if !self.interface_registry.contains_key(name)
                            && !self.external_imports.contains_key(name)
                        {
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
                let utf8_bytes = value.as_bytes();
                emitter.emit_const_string(utf8_bytes)?;

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

            AstNode::EmitStatement { event_name, fields } => {
                // Emit a header marker so indexers can identify event boundaries.
                let header = format!("event:{}", event_name);
                emitter.emit_const_string(header.as_bytes())?;
                emitter.emit_opcode(EMIT_EVENT);

                // Emit each field name and value as separate event records.
                // This keeps lowering simple while still preserving structured data.
                for field in fields {
                    let key = format!("field:{}", field.field_name);
                    emitter.emit_const_string(key.as_bytes())?;
                    emitter.emit_opcode(EMIT_EVENT);

                    self.generate_ast_node(emitter, &field.value)?;
                    emitter.emit_opcode(EMIT_EVENT);
                }

                Ok(())
            }

            AstNode::RequireStatement { condition } => self.emit_single_require(emitter, condition),

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

            AstNode::WhileLoop { condition, body } => {
                self.generate_while_loop(emitter, condition, body)?;
                Ok(())
            }

            AstNode::BreakStatement { .. } => {
                self.generate_break_statement(emitter)?;
                Ok(())
            }

            AstNode::ContinueStatement { .. } => {
                self.generate_continue_statement(emitter)?;
                Ok(())
            }

            AstNode::FieldAssignment {
                object,
                field,
                value,
            } => {
                // Try to emit fused opcode first for CU optimization (Tier 2)
                if self.try_emit_fused_field_assignment(emitter, object, field, value)? {
                    return Ok(());
                }
                // Fallback to standard field assignment generation
                self.generate_field_assignment(emitter, object, field, value)
            }

            AstNode::FieldAccess { object, field } => {
                if let AstNode::FieldAccess {
                    object: account_expr,
                    field: ctx_field,
                } = object.as_ref()
                {
                    if ctx_field == "ctx" {
                        let AstNode::Identifier(account_name) = account_expr.as_ref() else {
                            return Err(VMError::InvalidScript);
                        };
                        match field.as_str() {
                            "lamports" | "owner" | "key" | "data" => {
                                if let Some(account_idx) =
                                    self.resolve_account_param_by_name(account_name)
                                {
                                    match field.as_str() {
                                        "lamports" => emitter.emit_opcode(GET_LAMPORTS),
                                        "key" => emitter.emit_opcode(GET_KEY),
                                        "owner" => emitter.emit_opcode(GET_OWNER),
                                        "data" => emitter.emit_opcode(GET_DATA),
                                        _ => return Err(VMError::InvalidInstruction),
                                    }
                                    emitter.emit_u8(account_idx);
                                    return Ok(());
                                }

                                if let Some(account_system) = &self.account_system {
                                    return account_system.generate_builtin_account_property_access(
                                        emitter,
                                        account_name,
                                        field,
                                        &self.local_symbol_table,
                                    );
                                }
                                return Err(VMError::InvalidScript);
                            }
                            "bump" => {
                                let alias = Self::init_ctx_bump_alias(account_name);
                                return self
                                    .generate_ast_node(emitter, &AstNode::Identifier(alias));
                            }
                            "space" => {
                                let alias = Self::init_ctx_space_alias(account_name);
                                return self
                                    .generate_ast_node(emitter, &AstNode::Identifier(alias));
                            }
                            _ => return Err(VMError::UndefinedField),
                        }
                    }
                }

                // Tuple-backed Clock field access lowering.
                // Clock values are currently transported as tuple payloads in VM memory.
                if let Some(clock_index) = match field.as_str() {
                    "slot" => Some(0u8),
                    "epoch_start_timestamp" => Some(1u8),
                    "epoch" => Some(2u8),
                    "leader_schedule_epoch" => Some(3u8),
                    "unix_timestamp" => Some(4u8),
                    _ => None,
                } {
                    let object_is_clock = match object.as_ref() {
                        AstNode::FunctionCall { name, .. } => {
                            name == "get_clock" || name == "get_clock_sysvar"
                        }
                        AstNode::Identifier(name) => self
                            .local_symbol_table
                            .get(name)
                            .map(|f| f.field_type == "Clock")
                            .or_else(|| {
                                self.global_symbol_table
                                    .get(name)
                                    .map(|f| f.field_type == "Clock")
                            })
                            .unwrap_or(false),
                        _ => false,
                    };

                    if object_is_clock {
                        self.generate_ast_node(emitter, object)?;
                        emitter.emit_const_u8(clock_index)?;
                        emitter.emit_opcode(TUPLE_GET);
                        return Ok(());
                    }
                }

                if let AstNode::Identifier(account_name) = object.as_ref() {
                    #[cfg(debug_assertions)]
                    println!(
                        "AST Generator: Processing FieldAccess for '{}' field '{}'",
                        account_name, field
                    );

                    // TODO: Implement bulk field loading optimization

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
                        let mut field_type_name: Option<String> = None;
                        let mut field_found_in_registry = false;

                        if let Some(account_system) = &self.account_system {
                            let namespace_suffix = format!("::{}", account_type);
                            let account_type_info = account_system
                                .get_account_registry()
                                .account_types
                                .get(account_type)
                                .or_else(|| {
                                    account_system
                                        .get_account_registry()
                                        .account_types
                                        .iter()
                                        .find(|(k, _)| k.ends_with(&namespace_suffix))
                                        .map(|(_, v)| v)
                                });

                            if let Some(account_type_info) = account_type_info {
                                if let Some(struct_field_info) = account_type_info.fields.get(field)
                                {
                                    is_optional = struct_field_info.is_optional;
                                    is_pubkey = struct_field_info.field_type == "pubkey";
                                    field_type_name = Some(struct_field_info.field_type.clone());
                                    field_found_in_registry = true;
                                }
                            }
                        }

                        if !field_found_in_registry {
                            return Err(VMError::UndefinedField);
                        }

                        // Calculate field offset within account using the account type
                        let field_offset =
                            self.calculate_account_field_offset(account_type, field, account_name)?;

                        // Generate zero-copy account field load operation using MitoVM
                        if is_pubkey {
                            emitter.emit_opcode(LOAD_FIELD_PUBKEY); // Zero-copy pubkey read (32 bytes)
                        } else {
                            emitter.emit_opcode(LOAD_FIELD); // Zero-copy u64 read (8 bytes)
                        }

                        emitter.emit_u8(super::account_utils::account_index_from_param_offset(
                            field_info.offset,
                        )); // Account index from symbol table
                        emitter.emit_u32(field_offset); // Field offset (fixed format for consistency)

                        if is_optional {
                            emitter.emit_opcode(OPTIONAL_UNWRAP);
                        }

                        // LOAD_FIELD returns 8 bytes by default. Mask narrower integer fields
                        // so comparisons/arithmetic observe declared bit width.
                        if !is_pubkey {
                            if let Some(mask) = field_type_name.as_deref().and_then(|ty| match ty {
                                "u8" | "bool" => Some(0xFFu64),
                                "u16" => Some(0xFFFFu64),
                                "u32" => Some(0xFFFF_FFFFu64),
                                _ => None,
                            }) {
                                emitter.emit_const_u64(mask)?;
                                emitter.emit_opcode(BITWISE_AND);
                            }
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
                emitter.emit_const_u8(*index as u8)?;
                emitter.emit_opcode(TUPLE_GET);
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
            AstNode::Cast { value, .. } => {
                // Cast is currently compile-time type information for custom account types.
                // Runtime value is unchanged, so emit the underlying value expression.
                self.generate_ast_node(emitter, value)
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
                self.field_counter = 0;

                // Process function parameters and add them to the local symbol table
                // Unified parameter counter for both accounts and data
                // This MUST match the VM's sequential storage of parameters in the stack/param array
                let mut param_counter: u32 = 0;

                for (index, param) in parameters.iter().enumerate() {
                    // Generate @init account creation sequence if needed
                    self.generate_init_account_sequence(emitter, param, index)?;

                    // Use unified offset for all parameters
                    let offset = param_counter;
                    param_counter += 1;

                    let field_info = FieldInfo {
                        offset,
                        field_type: self.type_node_to_string(&param.param_type),
                        // Implicit mutability: @init implies mutable, or explicit @mut
                        is_mutable: param.is_init
                            || param.attributes.iter().any(|a| a.name == "mut"),
                        is_optional: param.is_optional,
                        is_parameter: true, // Mark as parameter to generate LOAD_PARAM instead of GET_LOCAL
                    };
                    self.local_symbol_table
                        .insert(param.name.clone(), field_info);
                }

                self.emit_pda_param_setup(emitter, parameters)?;

                // Inject @requires(condition) checks
                for param in parameters {
                    for attr in &param.attributes {
                        if attr.name == "requires" {
                            if let Some(condition) = attr.args.first() {
                                // Generate require statement for validity check
                                // This will behave exactly like 'require(condition);' at the start of the function
                                self.generate_ast_node(
                                    emitter,
                                    &AstNode::RequireStatement {
                                        condition: Box::new(condition.clone()),
                                    },
                                )?;
                            }
                        }
                    }
                }

                // Generate function body
                self.generate_ast_node(emitter, body)?;

                // Auto-lower @close(to=recipient) attribute to CLOSE_ACCOUNT at function epilogue.
                for param in parameters {
                    for attr in &param.attributes {
                        if attr.name != "close" {
                            continue;
                        }
                        let Some(crate::ast::AstNode::Identifier(target_name)) = attr.args.first()
                        else {
                            return Err(VMError::InvalidInstruction);
                        };

                        let source_idx = self
                            .resolve_account_param_by_name(&param.name)
                            .ok_or(VMError::InvalidScript)?;
                        let destination_idx = self
                            .resolve_account_param_by_name(target_name)
                            .ok_or(VMError::InvalidScript)?;

                        emitter.emit_const_u8(source_idx)?;
                        emitter.emit_const_u8(destination_idx)?;
                        emitter.emit_opcode(CLOSE_ACCOUNT);
                    }
                }

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
