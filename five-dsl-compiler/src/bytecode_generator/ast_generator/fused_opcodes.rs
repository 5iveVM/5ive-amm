// Fused Opcode Optimization Module
//
// This module provides pattern matching and emission of fused opcodes
// to reduce CU consumption by combining common multi-opcode patterns
// into single opcodes.

// Force println! to work even in release/test by bypassing cfg(debug_assertions) for investigation
macro_rules! debug_println {
    ($($arg:tt)*) => {
        println!($($arg)*);
    }
}


use crate::ast::AstNode;
use super::super::OpcodeEmitter;
use super::types::ASTGenerator;
use five_protocol::opcodes::*;
use five_vm_mito::error::VMError;

impl ASTGenerator {
    /// Try to emit a fused opcode for a require statement condition.
    /// Returns Ok(true) if a fused opcode was emitted, Ok(false) if not.
    pub(super) fn try_emit_fused_require<T: OpcodeEmitter>(
        &mut self,
        emitter: &mut T,
        condition: &AstNode,
    ) -> Result<bool, VMError> {
        // Pattern 1: field.gte(param) - balance >= amount (REQUIRE_GTE_U64)
        if let Some((acc_idx, offset, param_idx)) = self.match_field_gte_param(condition) {
            #[cfg(debug_assertions)]
            println!("FUSED_DEBUG: EMITTING REQUIRE_GTE_U64! acc={} offset={} param={}", acc_idx, offset, param_idx);
            emitter.emit_opcode(REQUIRE_GTE_U64);
            emitter.emit_u8(acc_idx);
            emitter.emit_vle_u32(offset);
            emitter.emit_u8(param_idx);
            return Ok(true);
        }

        // Pattern 2: not(field) - !is_frozen (REQUIRE_NOT_BOOL)
        if let Some((acc_idx, offset)) = self.match_not_bool_field(condition) {
            #[cfg(debug_assertions)]
            println!("FUSED_DEBUG: EMITTING REQUIRE_NOT_BOOL! acc={} offset={}", acc_idx, offset);
            emitter.emit_opcode(REQUIRE_NOT_BOOL);
            emitter.emit_u8(acc_idx);
            emitter.emit_vle_u32(offset);
            return Ok(true);
        }

        // Pattern 3: param.gt(0) - amount > 0 (REQUIRE_PARAM_GT_ZERO)
        if let Some(param_idx) = self.match_param_gt_zero(condition) {
            #[cfg(debug_assertions)]
            println!("FUSED_DEBUG: EMITTING REQUIRE_PARAM_GT_ZERO! param={}", param_idx);
            emitter.emit_opcode(REQUIRE_PARAM_GT_ZERO);
            emitter.emit_u8(param_idx);
            return Ok(true);
        }

        // Pattern 4: Pubkey equality check (REQUIRE_EQ_PUBKEY)
        // Matches: 
        // - require(field == account.key)
        // - require(field == field)
        if let Some((acc1_idx, offset1, acc2_idx, offset2)) = self.match_pubkey_eq_any(condition) {
            #[cfg(debug_assertions)]
            println!("FUSED_DEBUG: EMITTING REQUIRE_EQ_PUBKEY! acc1={} offset1={} acc2={} offset2={}", acc1_idx, offset1, acc2_idx, offset2);
            emitter.emit_opcode(REQUIRE_EQ_PUBKEY);
            emitter.emit_u8(acc1_idx);
            emitter.emit_vle_u32(offset1);
            emitter.emit_u8(acc2_idx);
            emitter.emit_vle_u32(offset2);
            return Ok(true);
        }

        // NEW Pattern 5: Compare two registers (REQUIRE_GTE_REG)
        if let Some((reg1, reg2)) = self.match_reg_gte_reg(condition) {
            #[cfg(debug_assertions)]
            println!("FUSED_DEBUG: EMITTING REQUIRE_GTE_REG! reg1={} reg2={}", reg1, reg2);
            emitter.emit_opcode(REQUIRE_GTE_REG);
            emitter.emit_u8(reg1);
            emitter.emit_u8(reg2);
            return Ok(true);
        }

        Ok(false)
    }

    /// Match pattern: reg1 >= reg2
    fn match_reg_gte_reg(&self, condition: &AstNode) -> Option<(u8, u8)> {
        if let AstNode::BinaryExpression { left, operator, right } = condition {
            if operator == ">=" {
                let reg1 = self.match_register(left)?;
                let reg2 = self.match_register(right)?;
                return Some((reg1, reg2));
            }
        }
        
        if let AstNode::MethodCall { object, method, args } = condition {
            if method == "gte" && args.len() == 1 {
                let reg1 = self.match_register(object)?;
                let reg2 = self.match_register(&args[0])?;
                return Some((reg1, reg2));
            }
        }
        None
    }

    /// Match pattern: field.gte(param) OR MethodCall with method="gte"
    /// Also matches BinaryExpression with operator=">=" for backwards compat
    fn match_field_gte_param(&self, condition: &AstNode) -> Option<(u8, u32, u8)> {
        // Try MethodCall pattern first: field.gte(param)
        if let AstNode::MethodCall { object, method, args } = condition {
            if method == "gte" && args.len() == 1 {
                // Object should be field access
                let (acc_idx, offset) = self.match_u64_field_access(object)?;
                // First arg should be parameter
                let param_idx = self.match_parameter(&args[0])?;
                return Some((acc_idx, offset, param_idx));
            }
        }
        
        // Fallback to BinaryExpression pattern
        if let AstNode::BinaryExpression { left, operator, right } = condition {
            if operator == ">=" {
                let (acc_idx, offset) = self.match_u64_field_access(left)?;
                let param_idx = self.match_parameter(right)?;
                return Some((acc_idx, offset, param_idx));
            }
        }
        None
    }

    /// Match pattern: UnaryExpression { operator: "not", operand: FieldAccess }
    fn match_not_bool_field(&self, condition: &AstNode) -> Option<(u8, u32)> {
        if let AstNode::UnaryExpression { operator, operand } = condition {
            // Parser uses "not" instead of "!"
            if operator == "not" || operator == "!" {
                return self.match_bool_field_access(operand);
            }
        }
        None
    }

    /// Match pattern: param.gt(0) - MethodCall with method="gt" and arg=Literal(0)
    fn match_param_gt_zero(&self, condition: &AstNode) -> Option<u8> {
        // Try MethodCall pattern: param.gt(0)
        if let AstNode::MethodCall { object, method, args } = condition {
            if method == "gt" && args.len() == 1 {
                // Object should be a parameter identifier
                let param_idx = self.match_parameter(object)?;
                // Arg should be literal 0
                if self.is_literal_zero(&args[0]) {
                    return Some(param_idx);
                }
            }
        }
        
        // Fallback to BinaryExpression
        if let AstNode::BinaryExpression { left, operator, right } = condition {
            if operator == ">" {
                let param_idx = self.match_parameter(left)?;
                if self.is_literal_zero(right) {
                    return Some(param_idx);
                }
            }
        }
        None
    }

    /// Match a u64 field access: account.field
    pub(super) fn match_u64_field_access(&self, node: &AstNode) -> Option<(u8, u32)> {
        if let AstNode::FieldAccess { object, field } = node {
            if let AstNode::Identifier(account_name) = object.as_ref() {
                if let Some(field_info) = self.local_symbol_table.get(account_name) {
                    let account_type = &field_info.field_type;
                    if let Ok(offset) = self.calculate_account_field_offset(account_type, field) {
                        let acc_idx = crate::bytecode_generator::account_utils::account_index_from_param_offset(
                            field_info.offset
                        );
                        return Some((acc_idx, offset));
                    }
                }
            }
        }
        None
    }

    /// Match a bool field access: account.field
    fn match_bool_field_access(&self, node: &AstNode) -> Option<(u8, u32)> {
        if let AstNode::FieldAccess { object, field } = node {
            if let AstNode::Identifier(account_name) = object.as_ref() {
                if let Some(field_info) = self.local_symbol_table.get(account_name) {
                    let account_type = &field_info.field_type;
                    
                    // We could check if it's a bool field, but for simplicity just emit
                    // and let runtime handle type checking
                    if let Ok(offset) = self.calculate_account_field_offset(account_type, field) {
                        let acc_idx = crate::bytecode_generator::account_utils::account_index_from_param_offset(
                            field_info.offset
                        );
                        return Some((acc_idx, offset));
                    }
                }
            }
        }
        None
    }

    /// Match a parameter identifier
    fn match_parameter(&self, node: &AstNode) -> Option<u8> {
        if let AstNode::Identifier(name) = node {
            if let Some(field_info) = self.local_symbol_table.get(name) {
                if field_info.is_parameter {
                    return Some((field_info.offset + 1) as u8);
                }
            }
        }
        None
    }

    /// Check if node is literal 0
    fn is_literal_zero(&self, node: &AstNode) -> bool {
        if let AstNode::Literal(value) = node {
            return value.as_u64() == Some(0);
        }
        false
    }

    // ===== TIER 2: Field Assignment Fused Opcodes =====

    /// Try to emit a fused opcode for a field assignment.
    /// Matches patterns like: account.field = account.field + param
    /// Returns Ok(true) if a fused opcode was emitted, Ok(false) if not.
    pub(super) fn try_emit_fused_field_assignment<T: OpcodeEmitter>(
        &mut self,
        emitter: &mut T,
        object: &AstNode,
        field: &str,
        value: &AstNode,
    ) -> Result<bool, VMError> {
        #[cfg(debug_assertions)]
        println!("FUSED_T2_DEBUG: try_emit_fused_field_assignment for field='{}' value={:?}", field, std::mem::discriminant(value));
        
        // Get target account and field info
        let (target_acc_idx, target_offset) = match object {
            AstNode::Identifier(account_name) => {
                if let Some(field_info) = self.local_symbol_table.get(account_name) {
                    let account_type = &field_info.field_type;
                    if let Ok(offset) = self.calculate_account_field_offset(account_type, field) {
                        let acc_idx = crate::bytecode_generator::account_utils::account_index_from_param_offset(
                            field_info.offset
                        );
                        (acc_idx, offset)
                    } else {
                        return Ok(false);
                    }
                } else {
                    return Ok(false);
                }
            }
            _ => return Ok(false),
        };

        // NEW: Pattern field = field.add(reg) -> ADD_FIELD_REG
        if let Some(reg_idx) = self.match_field_reg_arithmetic_pattern(object, field, value, "add") {
            #[cfg(debug_assertions)]
            println!("FUSED_DEBUG: EMITTING ADD_FIELD_REG! acc={} offset={} reg={}", target_acc_idx, target_offset, reg_idx);
            emitter.emit_opcode(ADD_FIELD_REG);
            emitter.emit_u8(target_acc_idx);
            emitter.emit_vle_u32(target_offset);
            emitter.emit_u8(reg_idx);
            return Ok(true);
        }

        // NEW: Pattern field = field.sub(reg) -> SUB_FIELD_REG
        if let Some(reg_idx) = self.match_field_reg_arithmetic_pattern(object, field, value, "sub") {
            #[cfg(debug_assertions)]
            println!("FUSED_DEBUG: EMITTING SUB_FIELD_REG! acc={} offset={} reg={}", target_acc_idx, target_offset, reg_idx);
            emitter.emit_opcode(SUB_FIELD_REG);
            emitter.emit_u8(target_acc_idx);
            emitter.emit_vle_u32(target_offset);
            emitter.emit_u8(reg_idx);
            return Ok(true);
        }

        // Pattern: field = field.add(param) -> FIELD_ADD_PARAM
        if let Some(param_idx) = self.match_field_arithmetic_pattern(object, field, value, "add") {
            #[cfg(debug_assertions)]
            println!("FUSED_DEBUG: EMITTING FIELD_ADD_PARAM! acc={} offset={} param={}", target_acc_idx, target_offset, param_idx);
            emitter.emit_opcode(FIELD_ADD_PARAM);
            emitter.emit_u8(target_acc_idx);
            emitter.emit_vle_u32(target_offset);
            emitter.emit_u8(param_idx);
            return Ok(true);
        }

        // Pattern: field = field.sub(param) -> FIELD_SUB_PARAM
        if let Some(param_idx) = self.match_field_arithmetic_pattern(object, field, value, "sub") {
            #[cfg(debug_assertions)]
            println!("FUSED_DEBUG: EMITTING FIELD_SUB_PARAM! acc={} offset={} param={}", target_acc_idx, target_offset, param_idx);
            emitter.emit_opcode(FIELD_SUB_PARAM);
            emitter.emit_u8(target_acc_idx);
            emitter.emit_vle_u32(target_offset);
            emitter.emit_u8(param_idx);
            return Ok(true);
        }

        // ===== TIER 3 PATTERNS =====

        // Pattern: field = 0 -> STORE_FIELD_ZERO
        if self.is_literal_zero(value) {
            #[cfg(debug_assertions)]
            println!("FUSED_DEBUG: EMITTING STORE_FIELD_ZERO! acc={} offset={}", target_acc_idx, target_offset);
            emitter.emit_opcode(STORE_FIELD_ZERO);
            emitter.emit_u8(target_acc_idx);
            emitter.emit_vle_u32(target_offset);
            return Ok(true);
        }

        // Pattern: field = param -> STORE_PARAM_TO_FIELD
        if let Some(param_idx) = self.match_parameter(value) {
            #[cfg(debug_assertions)]
            println!("FUSED_DEBUG: EMITTING STORE_PARAM_TO_FIELD! acc={} offset={} param={}", target_acc_idx, target_offset, param_idx);
            emitter.emit_opcode(STORE_PARAM_TO_FIELD);
            emitter.emit_u8(target_acc_idx);
            emitter.emit_vle_u32(target_offset);
            emitter.emit_u8(param_idx);
            return Ok(true);
        }

        // NEW: Pattern field = reg -> STORE_FIELD_REG
        if let Some(reg_idx) = self.match_register(value) {
            #[cfg(debug_assertions)]
            println!("FUSED_DEBUG: EMITTING STORE_FIELD_REG! acc={} offset={} reg={}", target_acc_idx, target_offset, reg_idx);
            emitter.emit_opcode(STORE_FIELD_REG);
            emitter.emit_u8(reg_idx);
            emitter.emit_u8(target_acc_idx);
            emitter.emit_vle_u32(target_offset);
            return Ok(true);
        }

        // Pattern: field = account.key -> STORE_KEY_TO_FIELD
        if let Some(key_acc_idx) = self.match_account_key_access(value) {
            #[cfg(debug_assertions)]
            println!("FUSED_DEBUG: EMITTING STORE_KEY_TO_FIELD! acc={} offset={} key_acc={}", target_acc_idx, target_offset, key_acc_idx);
            emitter.emit_opcode(STORE_KEY_TO_FIELD);
            emitter.emit_u8(target_acc_idx);
            emitter.emit_vle_u32(target_offset);
            emitter.emit_u8(key_acc_idx);
            return Ok(true);
        }

        Ok(false)
    }

    /// Match a register identifier (requires register allocator context)
    pub(super) fn match_register(&self, node: &AstNode) -> Option<u8> {
        if let AstNode::Identifier(name) = node {
            // Check the static register allocator for mapped registers
            self.register_allocator.get_mapping(name)
        } else {
            None
        }
    }

    /// Match pattern: object.field.{add|sub}(reg) OR object.field +/- reg
    fn match_field_reg_arithmetic_pattern(
        &self,
        target_object: &AstNode,
        target_field: &str,
        value: &AstNode,
        operation: &str, // "add" or "sub"
    ) -> Option<u8> {
        // Pattern 1: MethodCall - object.field.add/sub(reg)
        if let AstNode::MethodCall { object: method_obj, method, args } = value {
            if method == operation && args.len() == 1 {
                if let AstNode::FieldAccess { object: field_obj, field: field_name } = method_obj.as_ref() {
                    if field_name == target_field {
                        if let (AstNode::Identifier(target_name), AstNode::Identifier(source_name)) = (target_object, field_obj.as_ref()) {
                            if target_name == source_name {
                                return self.match_register(&args[0]);
                            }
                        }
                    }
                }
            }
        }
        
        // Pattern 2: BinaryExpression - object.field +/- reg
        if let AstNode::BinaryExpression { left, operator, right } = value {
            let expected_op = if operation == "add" { "+" } else { "-" };
            if operator != expected_op {
                return None;
            }
            
            if let AstNode::FieldAccess { object: field_obj, field: field_name } = left.as_ref() {
                if field_name != target_field {
                    return None;
                }
                
                if let (AstNode::Identifier(target_name), AstNode::Identifier(source_name)) = (target_object, field_obj.as_ref()) {
                    if target_name == source_name {
                        return self.match_register(right);
                    }
                }
            }
        }
        
        None
    }

    /// Match pattern: object.field.{add|sub}(param) OR object.field +/- param
    /// Where the value is either a MethodCall or BinaryExpression operating on same account.field
    fn match_field_arithmetic_pattern(
        &self,
        target_object: &AstNode,
        target_field: &str,
        value: &AstNode,
        operation: &str, // "add" or "sub"
    ) -> Option<u8> {
        // Pattern 1: MethodCall - object.field.add/sub(param)
        if let AstNode::MethodCall { object: method_obj, method, args } = value {
            if method == operation && args.len() == 1 {
                // The MethodCall object should be a FieldAccess to same account.field
                if let AstNode::FieldAccess { object: field_obj, field: field_name } = method_obj.as_ref() {
                    if field_name == target_field {
                        if let (AstNode::Identifier(target_name), AstNode::Identifier(source_name)) = (target_object, field_obj.as_ref()) {
                            if target_name == source_name {
                                return self.match_parameter(&args[0]);
                            }
                        }
                    }
                }
            }
        }
        
        // Pattern 2: BinaryExpression - object.field +/- param
        if let AstNode::BinaryExpression { left, operator, right } = value {
            // Check operator matches
            let expected_op = if operation == "add" { "+" } else { "-" };
            if operator != expected_op {
                return None;
            }
            
            // Left side should be FieldAccess to same account.field
            if let AstNode::FieldAccess { object: field_obj, field: field_name } = left.as_ref() {
                if field_name != target_field {
                    return None;
                }
                
                if let (AstNode::Identifier(target_name), AstNode::Identifier(source_name)) = (target_object, field_obj.as_ref()) {
                    if target_name == source_name {
                        // Right side should be a parameter
                        return self.match_parameter(right);
                    }
                }
            }
        }
        
        None
    }

    // ===== REQUIRE_EQ_PUBKEY Pattern Matching =====

    /// Match pattern: Pubkey equality check
    /// Returns: (acc1_idx, offset1, acc2_idx, offset2)
    /// offset2 is 0x3FFF for account.key, or field offset for account.field
    fn match_pubkey_eq_any(&self, condition: &AstNode) -> Option<(u8, u32, u8, u32)> {
        // Pattern 1: MethodCall - field.eq(other)
        if let AstNode::MethodCall { object, method, args } = condition {
            if method == "eq" && args.len() == 1 {
                // Left side: pubkey field
                let (acc1_idx, offset1) = self.match_pubkey_field_access(object)?;
                
                // Right side: account.key OR pubkey field
                if let Some(acc2_idx) = self.match_account_key_access(&args[0]) {
                     return Some((acc1_idx, offset1, acc2_idx, 0x3FFF));
                }
                if let Some((acc2_idx, offset2)) = self.match_pubkey_field_access(&args[0]) {
                     return Some((acc1_idx, offset1, acc2_idx, offset2));
                }
            }
        }
        
        // Pattern 2: BinaryExpression - field == other
        if let AstNode::BinaryExpression { left, operator, right } = condition {
            if operator == "==" {
                #[cfg(debug_assertions)]
                println!("FUSED_DEBUG: Check BinaryExpression == for PUBKEY_EQ");

                // Left side: pubkey field
                if let Some((acc1_idx, offset1)) = self.match_pubkey_field_access(left) {
                    #[cfg(debug_assertions)]
                    println!("FUSED_DEBUG: Left side matched pubkey field: acc={} offset={}", acc1_idx, offset1);

                    // Right side: account.key OR pubkey field
                    if let Some(acc2_idx) = self.match_account_key_access(right) {
                         #[cfg(debug_assertions)]
                         println!("FUSED_DEBUG: Right side matched account key: acc={}", acc2_idx);
                         return Some((acc1_idx, offset1, acc2_idx, 0x3FFF));
                    }
                    if let Some((acc2_idx, offset2)) = self.match_pubkey_field_access(right) {
                         #[cfg(debug_assertions)]
                         println!("FUSED_DEBUG: Right side matched pubkey field: acc={} offset={}", acc2_idx, offset2);
                         return Some((acc1_idx, offset1, acc2_idx, offset2));
                    }
                    #[cfg(debug_assertions)]
                    println!("FUSED_DEBUG: Right side did NOT match key or pubkey field");
                } else {
                    #[cfg(debug_assertions)]
                    println!("FUSED_DEBUG: Left side did NOT match pubkey field");
                }
            }
        }
        
        None
    }

    /// Match a pubkey field access: account.owner, account.mint, account.delegate, etc.
    fn match_pubkey_field_access(&self, node: &AstNode) -> Option<(u8, u32)> {
        if let AstNode::FieldAccess { object, field } = node {
            if let AstNode::Identifier(account_name) = object.as_ref() {
                if let Some(field_info) = self.local_symbol_table.get(account_name) {
                    let account_type = &field_info.field_type;
                    
                    // Check if this is a pubkey field - owner, mint, delegate, authority, etc.
                    let pubkey_fields = ["owner", "mint", "delegate", "authority", "freeze_authority"];
                    if pubkey_fields.contains(&field.as_str()) {
                        if let Ok(offset) = self.calculate_account_field_offset(account_type, field) {
                            let acc_idx = crate::bytecode_generator::account_utils::account_index_from_param_offset(
                                field_info.offset
                            );
                            return Some((acc_idx, offset));
                        }
                    }
                }
            }
        }
        None
    }

    /// Match account.key access pattern
    fn match_account_key_access(&self, node: &AstNode) -> Option<u8> {
        if let AstNode::FieldAccess { object, field } = node {
            if field == "key" {
                if let AstNode::Identifier(account_name) = object.as_ref() {
                    if let Some(field_info) = self.local_symbol_table.get(account_name) {
                        // This should be an account parameter
                        let acc_idx = crate::bytecode_generator::account_utils::account_index_from_param_offset(
                            field_info.offset
                        );
                        return Some(acc_idx);
                    }
                }
            }
        }
        None
    }
}

