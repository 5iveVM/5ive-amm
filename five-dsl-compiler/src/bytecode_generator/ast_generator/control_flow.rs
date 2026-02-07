//! Control flow generation methods
//!
//! This module handles if statements, match expressions, and optimized
//! control flow patterns like BR_EQ_U8 fused compare-branch.

use super::super::OpcodeEmitter;
use super::types::{ASTGenerator, BrEqU8Info, BrEqU8Patch};
use crate::ast::{AstNode, MatchArm};
use crate::bytecode_generator::types::LoopContext;
use five_protocol::opcodes::*;
use five_vm_mito::error::VMError;

impl ASTGenerator {
    /// Generate while loop with break/continue support
    pub(super) fn generate_while_loop<T: OpcodeEmitter>(
        &mut self,
        emitter: &mut T,
        condition: &AstNode,
        body: &AstNode,
    ) -> Result<(), VMError> {
        let start_label = self.new_label();
        let end_label = self.new_label();

        // Place loop start label
        self.place_label(emitter, start_label.clone());
        let loop_start_pos = emitter.get_position();

        // Push new loop context for break/continue tracking
        self.loop_stack.push(LoopContext {
            loop_start: loop_start_pos,
            break_targets: Vec::new(),
            continue_targets: Vec::new(),
        });

        // Generate condition code
        self.generate_ast_node(emitter, condition)?;

        // If condition is false, jump to end
        self.emit_jump(emitter, JUMP_IF_NOT, end_label.clone());

        // Generate loop body
        self.generate_ast_node(emitter, body)?;

        // Jump back to start
        self.emit_jump(emitter, JUMP, start_label);

        // Place loop end label
        self.place_label(emitter, end_label);
        let loop_end_pos = emitter.get_position();

        // Pop loop context and patch break/continue jumps
        if let Some(ctx) = self.loop_stack.pop() {
            // Patch all break statements to jump to loop end
            for break_pos in ctx.break_targets {
                self.patch_jump_offset(emitter, break_pos, loop_end_pos)?;
            }

            // Patch all continue statements to jump to loop start
            for continue_pos in ctx.continue_targets {
                self.patch_jump_offset(emitter, continue_pos, loop_start_pos)?;
            }
        }

        Ok(())
    }

    /// Generate break statement
    pub(super) fn generate_break_statement<T: OpcodeEmitter>(
        &mut self,
        emitter: &mut T,
    ) -> Result<(), VMError> {
        if self.loop_stack.is_empty() {
            return Err(VMError::InvalidScript); // Break outside loop
        }

        // Emit JUMP with placeholder
        emitter.emit_opcode(JUMP);
        let patch_pos = emitter.get_position();
        emitter.emit_u16(0);

        // Record patch position in current loop context
        if let Some(ctx) = self.loop_stack.last_mut() {
            ctx.break_targets.push(patch_pos);
        }

        Ok(())
    }

    /// Generate continue statement
    pub(super) fn generate_continue_statement<T: OpcodeEmitter>(
        &mut self,
        emitter: &mut T,
    ) -> Result<(), VMError> {
        if self.loop_stack.is_empty() {
            return Err(VMError::InvalidScript); // Continue outside loop
        }

        // Emit JUMP with placeholder
        emitter.emit_opcode(JUMP);
        let patch_pos = emitter.get_position();
        emitter.emit_u16(0);

        // Record patch position in current loop context
        if let Some(ctx) = self.loop_stack.last_mut() {
            ctx.continue_targets.push(patch_pos);
        }

        Ok(())
    }

    /// Generate if statement with conditional jumps
    pub(super) fn generate_if_statement<T: OpcodeEmitter>(
        &mut self,
        emitter: &mut T,
        condition: &AstNode,
        then_branch: &AstNode,
        else_branch: &Option<Box<AstNode>>,
    ) -> Result<(), VMError> {
        // Check if we can optimize with BR_EQ_U8 fused compare-branch opcode
        // CRITICAL: Only apply optimization for If-Else blocks.
        // For simple If, BR_EQ_U8 (4-5 bytes) vs Standard (6 bytes) is a smaller win,
        // but for If-Else it saves dispatch overhead and jump logic effectively.
        if else_branch.is_some() {
            if let Some(br_info) = self.check_br_eq_u8_pattern(condition) {
                return self.generate_br_eq_u8_if(emitter, &br_info, then_branch, else_branch);
            }
        }

        self.generate_ast_node(emitter, condition)?;

        let else_label = self.new_label();
        let end_label = self.new_label();

        // Jump to else branch if condition is false
        self.emit_jump(emitter, JUMP_IF_NOT, else_label.clone());

        self.generate_ast_node(emitter, then_branch)?;

        if else_branch.is_some() {
            // Jump to the end of the if statement
            self.emit_jump(emitter, JUMP, end_label.clone());
        }

        self.place_label(emitter, else_label);

        if let Some(else_node) = else_branch {
            self.generate_ast_node(emitter, else_node)?;
        }

        self.place_label(emitter, end_label);

        Ok(())
    }

    /// Check if condition matches BR_EQ_U8 pattern: variable == u8_literal
    pub(super) fn check_br_eq_u8_pattern(&self, condition: &AstNode) -> Option<BrEqU8Info> {
        // Handle MethodCall pattern: object.eq(args)
        if let AstNode::MethodCall {
            object,
            method,
            args,
        } = condition
        {
            if method == "eq" && args.len() == 1 {
                // Extract u8 value from literal argument
                let u8_value = match &args[0] {
                    AstNode::Literal(five_protocol::Value::U8(value)) => *value,
                    AstNode::Literal(five_protocol::Value::U64(value)) => {
                        // Handle u64 literals that fit in u8 range
                        if *value <= 255 {
                            *value as u8
                        } else {
                            return None;
                        }
                    }
                    _ => return None,
                };

                return Some(BrEqU8Info {
                    variable_node: (**object).clone(),
                    u8_value,
                });
            }
        }

        // Handle BinaryExpression pattern: left == right
        if let AstNode::BinaryExpression {
            operator,
            left,
            right,
        } = condition
        {
            if operator == "==" {
                // Check for variable == u8_literal pattern
                let u8_value = match right.as_ref() {
                    AstNode::Literal(five_protocol::Value::U8(value)) => *value,
                    AstNode::Literal(five_protocol::Value::U64(value)) => {
                        if *value <= 255 {
                            *value as u8
                        } else {
                            return None;
                        }
                    }
                    _ => return None,
                };

                return Some(BrEqU8Info {
                    variable_node: (**left).clone(),
                    u8_value,
                });
            }
        }
        None
    }

    /// Generate optimized if statement using BR_EQ_U8 fused compare-branch
    pub(super) fn generate_br_eq_u8_if<T: OpcodeEmitter>(
        &mut self,
        emitter: &mut T,
        br_info: &BrEqU8Info,
        then_branch: &AstNode,
        else_branch: &Option<Box<AstNode>>,
    ) -> Result<(), VMError> {
        // Generate the variable value onto the stack
        self.generate_ast_node(emitter, &br_info.variable_node)?;

        if else_branch.is_some() {
            // Pattern: if var == u8 { then } else { else }
            // BR_EQ_U8 jumps to then branch if equal, falls through to else

            let then_label = self.new_label();
            let end_label = self.new_label();

            // BR_EQ_U8: compare and jump to then branch if equal
            self.emit_br_eq_u8(emitter, br_info.u8_value, then_label.clone());

            // Generate else branch (fall-through case)
            self.generate_ast_node(emitter, else_branch.as_ref().unwrap())?;

            // Jump over then branch
            self.emit_jump(emitter, JUMP, end_label.clone());

            // Place then label and generate then branch
            self.place_label(emitter, then_label);
            self.generate_ast_node(emitter, then_branch)?;

            // Place end label
            self.place_label(emitter, end_label);
        } else {
            // Pattern: if var == u8 { then }
            // BR_EQ_U8 jumps to then branch if equal, falls through to end

            let then_label = self.new_label();
            let end_label = self.new_label();

            // BR_EQ_U8: compare and jump to then branch if equal
            self.emit_br_eq_u8(emitter, br_info.u8_value, then_label.clone());

            // Jump to end (skip then branch if not equal)
            self.emit_jump(emitter, JUMP, end_label.clone());

            // Place then label and generate then branch
            self.place_label(emitter, then_label);
            self.generate_ast_node(emitter, then_branch)?;

            // Place end label
            self.place_label(emitter, end_label);
        }

        Ok(())
    }

    /// Emit BR_EQ_U8 fused compare-branch instruction
    pub(super) fn emit_br_eq_u8<T: OpcodeEmitter>(
        &mut self,
        emitter: &mut T,
        compare_value: u8,
        target_label: String,
    ) {
        emitter.emit_opcode(BR_EQ_U8);
        emitter.emit_u8(compare_value);

        // Store BR_EQ_U8 patch for relative offset calculation
        let position = emitter.get_position();
        self.br_eq_u8_patches.push(BrEqU8Patch {
            position,
            target_label,
        });

        // Emit placeholder for VLE offset (will be patched later)
        // CRITICAL: Reserve 2 bytes for the VLE offset to allow for consistent patching.
        // We will force 2-byte VLE encoding when patching (0x80 | low, high).
        emitter.emit_u16(0); 
    }

    /// Generate match expression with constructor pattern matching
    pub(super) fn generate_match_expression<T: OpcodeEmitter>(
        &mut self,
        emitter: &mut T,
        expression: &AstNode,
        arms: &[MatchArm],
    ) -> Result<(), VMError> {
        // Generate the expression being matched
        self.generate_ast_node(emitter, expression)?;

        let end_label = self.new_label();

        for (i, arm) in arms.iter().enumerate() {
            let next_arm_label = if i + 1 < arms.len() {
                self.new_label()
            } else {
                end_label.clone()
            };

            // Save current local symbol table state
            let original_local_symbols = self.local_symbol_table.clone();

            // Check if this pattern matches and extract variables
            let pattern_matches =
                self.generate_pattern_match(emitter, &arm.pattern, next_arm_label.clone())?;

            if pattern_matches {
                // Pattern matched - execute the arm body
                self.generate_ast_node(emitter, &arm.body)?;
                self.emit_jump(emitter, JUMP, end_label.clone());
            }

            // Restore original local symbol table
            self.local_symbol_table = original_local_symbols;

            if i + 1 < arms.len() {
                self.place_label(emitter, next_arm_label);
            }
        }

        self.place_label(emitter, end_label);
        // Ensure end label targets a valid instruction offset.
        // This avoids out-of-bounds JUMP targets when a match expression
        // is the final statement (e.g., all arms return).
        emitter.emit_opcode(NOP);

        Ok(())
    }

    /// Generate pattern matching code for constructor patterns like Ok(value), Err(error)
    pub(super) fn generate_pattern_match<T: OpcodeEmitter>(
        &mut self,
        emitter: &mut T,
        pattern: &AstNode,
        next_arm_label: String,
    ) -> Result<bool, VMError> {
        match pattern {
            // Handle constructor patterns like Ok(value), Some(data), Err(error)
            AstNode::FunctionCall { name, args } => {
                match name.as_str() {
                    "Ok" | "Some" => {
                        // For Result/Option types, we need to check the variant tag
                        // Duplicate the matched value for tag checking
                        emitter.emit_opcode(DUP);

                        // Option/Result encoding uses AccountRef index as tag:
                        // Ok/Some = 0, Err = 254, None = 255
                        // For successful variants (Ok/Some), check if tag == 0
                        emitter.emit_const_u64(0)?; // Success tag
                        emitter.emit_opcode(EQ);

                        // Jump to next arm if tag doesn't match
                        self.emit_jump(emitter, JUMP_IF_NOT, next_arm_label);

                        // Pattern matched - extract the inner value if there are arguments
                        if !args.is_empty() {
                            if let AstNode::Identifier(var_name) = &args[0] {
                                // For now, treat the entire matched value as the extracted variable
                                // In a full implementation, we'd extract the inner value from the enum

                                // Add the pattern variable to local symbol table
                                self.add_local_variable(var_name.clone(), "u64".to_string());

                                // Store the matched value in the pattern variable
                                let var_index = self.get_local_variable_index(var_name)?;
                                emitter.emit_opcode(SET_LOCAL);
                                emitter.emit_u8(var_index);
                            }
                        }

                        Ok(true)
                    }
                    "Err" => {
                        // For error variants, check if tag == 254
                        emitter.emit_opcode(DUP);
                        emitter.emit_const_u64(254)?; // Err tag
                        emitter.emit_opcode(EQ);

                        // Jump to next arm if tag doesn't match
                        self.emit_jump(emitter, JUMP_IF_NOT, next_arm_label);

                        // Pattern matched - extract the error value if there are arguments
                        if !args.is_empty() {
                            if let AstNode::Identifier(var_name) = &args[0] {
                                // Add the pattern variable to local symbol table
                                self.add_local_variable(var_name.clone(), "string".to_string());

                                // For now, store the matched value as the error variable
                                let var_index = self.get_local_variable_index(var_name)?;
                                emitter.emit_opcode(SET_LOCAL);
                                emitter.emit_u8(var_index);
                            }
                        }

                        Ok(true)
                    }
                    "None" => {
                        // For none variants, check if tag == 255
                        emitter.emit_opcode(DUP);
                        emitter.emit_const_u64(255)?; // None tag
                        emitter.emit_opcode(EQ);

                        // Jump to next arm if tag doesn't match
                        self.emit_jump(emitter, JUMP_IF_NOT, next_arm_label);

                        Ok(true)
                    }
                    _ => {
                        // Unknown constructor pattern
                        Err(VMError::InvalidScript)
                    }
                }
            }
            // Handle literal patterns
            AstNode::Literal(_) => {
                // Simple literal comparison
                emitter.emit_opcode(DUP);
                self.generate_ast_node(emitter, pattern)?;
                emitter.emit_opcode(EQ);
                self.emit_jump(emitter, JUMP_IF_NOT, next_arm_label);
                Ok(true)
            }
            // Handle identifier patterns (catch-all variables)
            AstNode::Identifier(var_name) => {
                // This is a catch-all pattern that matches anything
                // Add the variable to local symbol table and assign the matched value
                self.add_local_variable(var_name.clone(), "u64".to_string());
                let var_index = self.get_local_variable_index(var_name)?;
                emitter.emit_opcode(SET_LOCAL);
                emitter.emit_u8(var_index);
                Ok(true)
            }
            _ => {
                // Unsupported pattern type
                Err(VMError::InvalidScript)
            }
        }
    }
}
