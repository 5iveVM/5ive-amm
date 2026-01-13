//! Array and block statement generation
//!
//! This module handles generation of arrays, byte arrays, and statement blocks.

use super::super::OpcodeEmitter;
use super::types::ASTGenerator;
use crate::ast::AstNode;
use five_protocol::opcodes::*;
use five_vm_mito::error::VMError;

impl ASTGenerator {
    /// Generate bytecode for a byte array
    pub(super) fn generate_byte_array<T: OpcodeEmitter>(
        &mut self,
        emitter: &mut T,
        node: &AstNode,
    ) -> Result<(), VMError> {
        if let AstNode::ArrayLiteral { elements } = node {
            emitter.emit_opcode(PUSH_U8);
            emitter.emit_u8(elements.len() as u8);
            for element in elements {
                self.generate_ast_node(emitter, element)?;
            }
        } else {
            return Err(VMError::TypeMismatch);
        }
        Ok(())
    }

    /// Generate bytecode for an array of any type
    pub(super) fn generate_array<T: OpcodeEmitter>(
        &mut self,
        emitter: &mut T,
        node: &AstNode,
    ) -> Result<(), VMError> {
        if let AstNode::ArrayLiteral { elements } = node {
            emitter.emit_opcode(PUSH_U8);
            emitter.emit_u8(elements.len() as u8);
            for element in elements {
                self.generate_ast_node(emitter, element)?;
            }
        } else {
            return Err(VMError::TypeMismatch);
        }
        Ok(())
    }

    /// Generate statement block with Phase 4 Bulk Field Loading Optimization
    pub(super) fn generate_statement_block<T: OpcodeEmitter>(
        &mut self,
        emitter: &mut T,
        statements: &[AstNode],
    ) -> Result<(), VMError> {
        let mut i = 0;
        while i < statements.len() {
            // Check for BULK_LOAD optimization pattern: Consecutive LetStatements reading fields from same account
            // Minimum sequence length of 2 to justify optimization overhead
            let mut bulk_group: Vec<(String, String, String)> = Vec::new(); // (var_name, account_name, field_name)
            
            // Lookahead loop
            let mut j = i;
            while j < statements.len() {
                if let AstNode::LetStatement { name, value, .. } = &statements[j] {
                    if let AstNode::FieldAccess { object, field } = value.as_ref() {
                        if let AstNode::Identifier(account_name) = object.as_ref() {
                            // First item found or subsequent item matches the current account
                            if bulk_group.is_empty() || bulk_group[0].1 == *account_name {
                                bulk_group.push((name.clone(), account_name.clone(), field.clone()));
                                j += 1;
                                continue;
                            }
                        }
                    }
                }
                break; // Pattern stream broken
            }

            // Apply optimization if we found a valid group
            if bulk_group.len() >= 2 {
                let account_name = &bulk_group[0].1;
                
                // Validate all fields exist and get their offsets
                // We need to do this carefully. If any field lookup fails or logic is complex (like built-ins), 
                // we abort the bulk opt for safety and fall back to standard generation.
                
                // We need the account index
                let account_index = self.local_symbol_table.get(account_name).map(|field_info| super::super::account_utils::account_index_from_param_offset(
                        field_info.offset,
                    ));

                let mut field_offsets = Vec::new();
                let mut valid_bulk = false;

                if account_index.is_some() {
                     // Try to resolve all field offsets
                     valid_bulk = true;
                     for (_, _, field_name) in &bulk_group {
                        // We use the account type from symbol table to resolve offset
                        // Note: We need to re-fetch field_info inside loop or logic because we can't easily hold ref
                        // But we verified account name matches for all.
                        
                        // To properly resolve, we need the account type. 
                        let acc_type = &self.local_symbol_table.get(account_name).unwrap().field_type;
                         
                        match self.calculate_account_field_offset(acc_type, field_name) {
                            Ok(offset) => field_offsets.push(offset),
                            Err(_) => {
                                valid_bulk = false;
                                break;
                            }
                        }
                     }
                }

                if valid_bulk {
                    // Emit BULK_LOAD_FIELD_N
                    emitter.emit_opcode(BULK_LOAD_FIELD_N);
                    emitter.emit_u8(account_index.unwrap());
                    emitter.emit_u8(bulk_group.len() as u8); // N fields
                    
                    for offset in &field_offsets {
                        emitter.emit_vle_u32(*offset);
                    }
                    
                    // Register local variables and emit SET_LOCALs
                    // The BULK op pushes values in order: [val1, val2, val3] (Stack top is val3?)
                    // Typically bulk loads push in order, so the LAST loaded field is at the TOP of the stack.
                    // If instruction is: LOAD a.x, a.y, a.z
                    // Stack result: ..., val_x, val_y, val_z (TOP)
                    // So we must pop/set in REVERSE order: z, then y, then x.
                    
                    for (k, (var_name, _, _)) in bulk_group.iter().enumerate().rev() {
                        // Add to local symbol table (logic copied from normal LetStatement)
                        // Note: We don't know the exact type easily without inference, but FieldAccess usually implies 
                        // we can look it up in registry. For now, simplistically add as mutable
                        
                        // We need to infer type to properly register it.
                        // Re-retrieve field type from registry to be correct
                        let acc_type = &self.local_symbol_table.get(account_name).unwrap().field_type;
                        let field_type = if let Some(sys) = &self.account_system {
                            if let Some(reg) = sys.get_account_registry().account_types.get(acc_type) {
                                reg.fields.get(&bulk_group[k].2)
                                    .map(|f| f.field_type.clone())
                                    .ok_or_else(|| {
                                        println!("ERROR: Field '{}' not found in account registry", bulk_group[k].2);
                                        VMError::UndefinedAccountField
                                    })?
                            } else {
                                return Err(VMError::UndefinedAccountField);
                            }
                        } else {
                            return Err(VMError::InvalidScript);
                        };

                        // Allocation logic (simplified from mod.rs)
                        let offset = self.field_counter;
                        self.field_counter += 1;
                        
                        let field_info = super::super::types::FieldInfo {
                            offset,
                            field_type,
                            is_mutable: true, // Let vars usually mutable in this compiler
                            is_optional: false,
                            is_parameter: false,
                        };
                        self.local_symbol_table.insert(var_name.clone(), field_info);

                        // Emit SET_LOCAL
                        self.emit_set_local(emitter, offset, &format!("bulk load '{}'", var_name));
                    }
                    
                    #[cfg(debug_assertions)]
                     println!("AST Generator: Optimized {} field loads for account '{}' into BULK_LOAD", bulk_group.len(), account_name);

                    // Skip the statements we processed
                    i += bulk_group.len();
                    continue;
                }
            }

            // Fallback: Generate single statement normally
            self.generate_ast_node(emitter, &statements[i])?;
            i += 1;
        }
        Ok(())
    }
}
