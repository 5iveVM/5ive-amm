// Fused opcode optimizations.

use super::super::OpcodeEmitter;
use super::types::ASTGenerator;
use crate::ast::AstNode;
use crate::session_support;
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
            println!(
                "FUSED_DEBUG: EMITTING REQUIRE_GTE_U64! acc={} offset={} param={}",
                acc_idx, offset, param_idx
            );
            emitter.emit_opcode(REQUIRE_GTE_U64);
            emitter.emit_u8(acc_idx);
            emitter.emit_u32(offset);
            emitter.emit_u8(param_idx);
            return Ok(true);
        }

        // Pattern 2: not(field) - !is_frozen (REQUIRE_NOT_BOOL)
        if let Some((acc_idx, offset)) = self.match_not_bool_field(condition) {
            #[cfg(debug_assertions)]
            println!(
                "FUSED_DEBUG: EMITTING REQUIRE_NOT_BOOL! acc={} offset={}",
                acc_idx, offset
            );
            emitter.emit_opcode(REQUIRE_NOT_BOOL);
            emitter.emit_u8(acc_idx);
            emitter.emit_u32(offset);
            return Ok(true);
        }

        // Pattern 3: param.gt(0) - amount > 0 (REQUIRE_PARAM_GT_ZERO)
        if let Some(param_idx) = self.match_param_gt_zero(condition) {
            #[cfg(debug_assertions)]
            println!(
                "FUSED_DEBUG: EMITTING REQUIRE_PARAM_GT_ZERO! param={}",
                param_idx
            );
            emitter.emit_opcode(REQUIRE_PARAM_GT_ZERO);
            emitter.emit_u8(param_idx);
            return Ok(true);
        }

        // Pattern 3b: local.gt(0) / local >= 1 (REQUIRE_LOCAL_GT_ZERO)
        if let Some(local_idx) = self.match_local_gt_zero(condition) {
            #[cfg(debug_assertions)]
            println!(
                "FUSED_DEBUG: EMITTING REQUIRE_LOCAL_GT_ZERO! local={}",
                local_idx
            );
            emitter.emit_opcode(REQUIRE_LOCAL_GT_ZERO);
            emitter.emit_u8(local_idx);
            return Ok(true);
        }

        // Pattern 4: pubkey field == account.key (REQUIRE_OWNER)
        if let Some((acc_idx, signer_idx, offset)) =
            self.match_pubkey_field_eq_account_key(condition)
        {
            if let Some((session_idx, session_delegate_offset, session_authority_offset)) =
                self.resolve_session_owner_context(signer_idx)
            {
                // Owner-or-session check:
                // 1) direct owner key equality passes immediately
                // 2) otherwise require delegated session:
                //    session.delegate == signer.key
                //    session.authority == business-owner field
                emitter.emit_opcode(LOAD_FIELD);
                emitter.emit_u8(acc_idx);
                emitter.emit_u32(offset);
                emitter.emit_opcode(GET_KEY);
                emitter.emit_u8(signer_idx);
                emitter.emit_opcode(EQ);
                emitter.emit_opcode(JUMP_IF);
                let bypass_patch_pos = emitter.get_position();
                emitter.emit_u16(0);

                emitter.emit_opcode(LOAD_FIELD);
                emitter.emit_u8(session_idx);
                emitter.emit_u32(session_delegate_offset);
                emitter.emit_opcode(GET_KEY);
                emitter.emit_u8(signer_idx);
                emitter.emit_opcode(EQ);
                emitter.emit_opcode(REQUIRE);

                emitter.emit_opcode(REQUIRE_EQ_PUBKEY);
                emitter.emit_u8(session_idx);
                emitter.emit_u32(session_authority_offset);
                emitter.emit_u8(acc_idx);
                emitter.emit_u32(offset);

                let bypass_target = emitter.get_position();
                emitter.patch_u16(bypass_patch_pos, bypass_target as u16);
                return Ok(true);
            }

            #[cfg(debug_assertions)]
            println!(
                "FUSED_DEBUG: EMITTING REQUIRE_OWNER! acc={} signer={} offset={}",
                acc_idx, signer_idx, offset
            );
            emitter.emit_opcode(REQUIRE_OWNER);
            emitter.emit_u8(acc_idx);
            emitter.emit_u8(signer_idx);
            emitter.emit_u32(offset);
            return Ok(true);
        }

        // Pattern 5: pubkey field == pubkey field
        if let Some((acc1_idx, offset1, acc2_idx, offset2)) = self.match_pubkey_eq_any(condition) {
            #[cfg(debug_assertions)]
            println!(
                "FUSED_DEBUG: EMITTING REQUIRE_EQ_PUBKEY! acc1={} offset1={} acc2={} offset2={}",
                acc1_idx, offset1, acc2_idx, offset2
            );
            emitter.emit_opcode(REQUIRE_EQ_PUBKEY);
            emitter.emit_u8(acc1_idx);
            emitter.emit_u32(offset1);
            emitter.emit_u8(acc2_idx);
            emitter.emit_u32(offset2);
            return Ok(true);
        }

        Ok(false)
    }

    /// Emit a single require with existing fused-opcode fallback behavior.
    pub(super) fn emit_single_require<T: OpcodeEmitter>(
        &mut self,
        emitter: &mut T,
        condition: &AstNode,
    ) -> Result<(), VMError> {
        if self.try_emit_fused_require(emitter, condition)? {
            return Ok(());
        }
        self.generate_ast_node(emitter, condition)?;
        emitter.emit_opcode(REQUIRE);
        Ok(())
    }

    /// Try to lower consecutive require statements into one or more REQUIRE_BATCH opcodes.
    /// Returns consumed statement count when the starting statement is a require.
    pub(super) fn try_emit_require_batch_block<T: OpcodeEmitter>(
        &mut self,
        emitter: &mut T,
        statements: &[AstNode],
        index: usize,
    ) -> Result<Option<usize>, VMError> {
        if !self.require_batch_enabled {
            return Ok(None);
        }

        let AstNode::RequireStatement { .. } = &statements[index] else {
            return Ok(None);
        };

        let mut end = index;
        while end < statements.len() {
            match &statements[end] {
                AstNode::RequireStatement { .. } => end += 1,
                _ => break,
            }
        }

        let mut pending_supported: Vec<(&AstNode, Vec<u8>)> = Vec::new();
        for statement in &statements[index..end] {
            let AstNode::RequireStatement { condition } = statement else {
                continue;
            };
            if let Some(clause) = self.try_build_batch_clause(condition) {
                pending_supported.push((condition.as_ref(), clause));
                if pending_supported.len() == REQUIRE_BATCH_MAX_CLAUSES as usize {
                    self.flush_require_batch_or_fallback(emitter, &mut pending_supported)?;
                }
            } else {
                self.flush_require_batch_or_fallback(emitter, &mut pending_supported)?;
                self.emit_single_require(emitter, condition)?;
            }
        }

        self.flush_require_batch_or_fallback(emitter, &mut pending_supported)?;
        Ok(Some(end - index))
    }

    fn flush_require_batch_or_fallback<T: OpcodeEmitter>(
        &mut self,
        emitter: &mut T,
        pending: &mut Vec<(&AstNode, Vec<u8>)>,
    ) -> Result<(), VMError> {
        if pending.is_empty() {
            return Ok(());
        }

        if pending.len() >= 2 {
            emitter.emit_opcode(REQUIRE_BATCH);
            emitter.emit_u8(pending.len() as u8);
            for (_, clause) in pending.iter() {
                emitter.emit_bytes(clause);
            }
        } else {
            self.emit_single_require(emitter, pending[0].0)?;
        }

        pending.clear();
        Ok(())
    }

    /// Try to map a require condition into a REQUIRE_BATCH clause encoding.
    pub(super) fn try_build_batch_clause(&self, condition: &AstNode) -> Option<Vec<u8>> {
        if let Some(param_idx) = self.match_param_gt_zero(condition) {
            return Some(vec![REQUIRE_BATCH_PARAM_GT_ZERO, param_idx]);
        }

        if let Some(local_idx) = self.match_local_gt_zero(condition) {
            return Some(vec![REQUIRE_BATCH_LOCAL_GT_ZERO, local_idx]);
        }

        if let Some((acc_idx, offset)) = self.match_not_bool_field(condition) {
            let mut clause = vec![REQUIRE_BATCH_FIELD_NOT_BOOL, acc_idx];
            clause.extend_from_slice(&offset.to_le_bytes());
            return Some(clause);
        }

        if let Some((acc_idx, offset, param_idx)) = self.match_field_gte_param(condition) {
            let mut clause = vec![REQUIRE_BATCH_FIELD_GTE_PARAM, acc_idx];
            clause.extend_from_slice(&offset.to_le_bytes());
            clause.push(param_idx);
            return Some(clause);
        }

        if let Some((acc_idx, signer_idx, offset)) =
            self.match_pubkey_field_eq_account_key(condition)
        {
            let mut clause = vec![REQUIRE_BATCH_OWNER_EQ_SIGNER, acc_idx, signer_idx];
            clause.extend_from_slice(&offset.to_le_bytes());
            return Some(clause);
        }

        if let Some((param_idx, imm)) = self.match_param_lte_imm(condition) {
            return Some(vec![REQUIRE_BATCH_PARAM_LTE_IMM, param_idx, imm]);
        }

        if let Some((acc_idx, offset, imm)) = self.match_field_eq_imm(condition) {
            let mut clause = vec![REQUIRE_BATCH_FIELD_EQ_IMM, acc_idx];
            clause.extend_from_slice(&offset.to_le_bytes());
            clause.push(imm);
            return Some(clause);
        }

        if let Some((acc_idx, offset, param_idx)) = self.match_pubkey_field_eq_param(condition) {
            let mut clause = vec![REQUIRE_BATCH_PUBKEY_FIELD_EQ_PARAM, acc_idx];
            clause.extend_from_slice(&offset.to_le_bytes());
            clause.push(param_idx);
            return Some(clause);
        }

        None
    }

    /// Match pattern: field.gte(param) OR MethodCall with method="gte"
    /// Also matches BinaryExpression with operator=">=" for backwards compat
    fn match_field_gte_param(&self, condition: &AstNode) -> Option<(u8, u32, u8)> {
        // Try MethodCall pattern first: field.gte(param)
        if let AstNode::MethodCall {
            object,
            method,
            args,
        } = condition
        {
            if method == "gte" && args.len() == 1 {
                // Object should be field access
                let (acc_idx, offset) = self.match_u64_field_access(object)?;
                // First arg should be parameter
                let param_idx = self.match_parameter(&args[0])?;
                return Some((acc_idx, offset, param_idx));
            }
        }

        // Fallback to BinaryExpression pattern
        if let AstNode::BinaryExpression {
            left,
            operator,
            right,
        } = condition
        {
            if operator == ">=" {
                let (acc_idx, offset) = self.match_u64_field_access(left)?;
                let param_idx = self.match_parameter(right)?;
                return Some((acc_idx, offset, param_idx));
            }
        }
        None
    }

    /// Match pattern: account.pubkey_field == signer.key (or reversed)
    /// Returns: (account_idx, signer_idx, field_offset)
    fn match_pubkey_field_eq_account_key(&self, condition: &AstNode) -> Option<(u8, u8, u32)> {
        if let AstNode::MethodCall {
            object,
            method,
            args,
        } = condition
        {
            if method == "eq" && args.len() == 1 {
                if let (Some((acc_idx, offset)), Some(signer_idx)) = (
                    self.match_pubkey_field_access(object),
                    self.match_account_key_access(&args[0]),
                ) {
                    return Some((acc_idx, signer_idx, offset));
                }
            }
        }

        let AstNode::BinaryExpression {
            left,
            operator,
            right,
        } = condition
        else {
            return None;
        };
        if operator != "==" {
            return None;
        }

        if let (Some((acc_idx, offset)), Some(signer_idx)) = (
            self.match_pubkey_field_access(left),
            self.match_account_key_access(right),
        ) {
            return Some((acc_idx, signer_idx, offset));
        }

        if let (Some((acc_idx, offset)), Some(signer_idx)) = (
            self.match_pubkey_field_access(right),
            self.match_account_key_access(left),
        ) {
            return Some((acc_idx, signer_idx, offset));
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

        if let AstNode::MethodCall {
            object,
            method,
            args,
        } = condition
        {
            if args.len() == 1 {
                if method == "eq" && self.is_literal_false(&args[0]) {
                    return self.match_bool_field_access(object);
                }
                if method == "ne" && self.is_literal_true(&args[0]) {
                    return self.match_bool_field_access(object);
                }
            }
        }

        if let AstNode::BinaryExpression {
            left,
            operator,
            right,
        } = condition
        {
            if operator == "==" && self.is_literal_false(right) {
                return self.match_bool_field_access(left);
            }
            if operator == "!=" && self.is_literal_true(right) {
                return self.match_bool_field_access(left);
            }
        }
        None
    }

    /// Match pattern: param.gt(0) - MethodCall with method="gt" and arg=Literal(0)
    fn match_param_gt_zero(&self, condition: &AstNode) -> Option<u8> {
        // Try MethodCall pattern: param.gt(0)
        if let AstNode::MethodCall {
            object,
            method,
            args,
        } = condition
        {
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
        if let AstNode::BinaryExpression {
            left,
            operator,
            right,
        } = condition
        {
            if operator == ">" {
                let param_idx = self.match_parameter(left)?;
                if self.is_literal_zero(right) {
                    return Some(param_idx);
                }
            }
            if operator == "!=" {
                let param_idx = self.match_parameter(left)?;
                if self.is_literal_zero(right) {
                    return Some(param_idx);
                }
            }
        }
        None
    }

    /// Match pattern: local.gt(0) or local >= 1 (non-zero locals)
    fn match_local_gt_zero(&self, condition: &AstNode) -> Option<u8> {
        // MethodCall patterns
        if let AstNode::MethodCall {
            object,
            method,
            args,
        } = condition
        {
            if args.len() == 1 {
                if method == "gt" && self.is_literal_zero(&args[0]) {
                    return self.match_local_identifier(object);
                }
                if method == "gte" && self.is_literal_one(&args[0]) {
                    return self.match_local_identifier(object);
                }
            }
        }

        // BinaryExpression patterns
        if let AstNode::BinaryExpression {
            left,
            operator,
            right,
        } = condition
        {
            if operator == ">" && self.is_literal_zero(right) {
                return self.match_local_identifier(left);
            }
            if operator == "!=" && self.is_literal_zero(right) {
                return self.match_local_identifier(left);
            }
            if operator == ">=" && self.is_literal_one(right) {
                return self.match_local_identifier(left);
            }
        }

        None
    }

    /// Match pattern: param <= imm_u8
    fn match_param_lte_imm(&self, condition: &AstNode) -> Option<(u8, u8)> {
        if let AstNode::MethodCall {
            object,
            method,
            args,
        } = condition
        {
            if method == "lte" && args.len() == 1 {
                let param_idx = self.match_parameter(object)?;
                let imm = self.literal_u8(&args[0])?;
                return Some((param_idx, imm));
            }
        }

        if let AstNode::BinaryExpression {
            left,
            operator,
            right,
        } = condition
        {
            if operator == "<=" {
                let param_idx = self.match_parameter(left)?;
                let imm = self.literal_u8(right)?;
                return Some((param_idx, imm));
            }
        }

        None
    }

    /// Match pattern: u64 field == imm_u8
    fn match_field_eq_imm(&self, condition: &AstNode) -> Option<(u8, u32, u8)> {
        if let AstNode::MethodCall {
            object,
            method,
            args,
        } = condition
        {
            if method == "eq" && args.len() == 1 {
                let (acc_idx, offset) = self.match_u64_field_access(object)?;
                let imm = self.literal_u8(&args[0])?;
                return Some((acc_idx, offset, imm));
            }
        }

        if let AstNode::BinaryExpression {
            left,
            operator,
            right,
        } = condition
        {
            if operator == "==" {
                if let Some((acc_idx, offset)) = self.match_u64_field_access(left) {
                    let imm = self.literal_u8(right)?;
                    return Some((acc_idx, offset, imm));
                }
                if let Some((acc_idx, offset)) = self.match_u64_field_access(right) {
                    let imm = self.literal_u8(left)?;
                    return Some((acc_idx, offset, imm));
                }
            }
        }

        None
    }

    /// Match pattern: pubkey field == parameter (or reversed).
    fn match_pubkey_field_eq_param(&self, condition: &AstNode) -> Option<(u8, u32, u8)> {
        if let AstNode::MethodCall {
            object,
            method,
            args,
        } = condition
        {
            if method == "eq" && args.len() == 1 {
                let (acc_idx, offset) = self.match_pubkey_field_access(object)?;
                let param_idx = self.match_parameter(&args[0])?;
                return Some((acc_idx, offset, param_idx));
            }
        }

        if let AstNode::BinaryExpression {
            left,
            operator,
            right,
        } = condition
        {
            if operator == "==" {
                if let Some((acc_idx, offset)) = self.match_pubkey_field_access(left) {
                    let param_idx = self.match_parameter(right)?;
                    return Some((acc_idx, offset, param_idx));
                }
                if let Some((acc_idx, offset)) = self.match_pubkey_field_access(right) {
                    let param_idx = self.match_parameter(left)?;
                    return Some((acc_idx, offset, param_idx));
                }
            }
        }

        None
    }

    /// Match a local variable identifier (non-parameter).
    fn match_local_identifier(&self, node: &AstNode) -> Option<u8> {
        if let AstNode::Identifier(name) = node {
            if let Some(field_info) = self.local_symbol_table.get(name) {
                if !field_info.is_parameter && field_info.offset <= u8::MAX as u32 {
                    return Some(field_info.offset as u8);
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
                    if let Ok(offset) =
                        self.calculate_account_field_offset(account_type, field, account_name)
                    {
                        let acc_idx = self.resolve_account_param_by_name(account_name)?;
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
                    if let Ok(offset) =
                        self.calculate_account_field_offset(account_type, field, account_name)
                    {
                        let acc_idx = self.resolve_account_param_by_name(account_name)?;
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

    /// Check if node is literal 1
    fn is_literal_one(&self, node: &AstNode) -> bool {
        if let AstNode::Literal(value) = node {
            return value.as_u64() == Some(1);
        }
        false
    }

    fn is_literal_false(&self, node: &AstNode) -> bool {
        if let AstNode::Literal(value) = node {
            return value.as_u64() == Some(0);
        }
        false
    }

    fn is_literal_true(&self, node: &AstNode) -> bool {
        if let AstNode::Literal(value) = node {
            return value.as_u64() == Some(1);
        }
        false
    }

    fn literal_u8(&self, node: &AstNode) -> Option<u8> {
        let AstNode::Literal(value) = node else {
            return None;
        };
        let raw = value.as_u64()?;
        if raw <= u8::MAX as u64 {
            Some(raw as u8)
        } else {
            None
        }
    }

    // ===== TIER 3: Multi-Statement Fused Opcodes (Block Level) =====

    /// Try to emit a fused opcode for a block of assignment statements.
    /// This handles multi-statement patterns like "double entry bookkeeping"
    /// Returns Ok(Some(consumed_count)) if a fused pattern was matched and emitted.
    /// Returns Ok(None) if no pattern matched.
    pub(super) fn try_emit_fused_assignment_block<T: OpcodeEmitter>(
        &mut self,
        emitter: &mut T,
        statements: &[AstNode],
        index: usize,
    ) -> Result<Option<usize>, VMError> {
        // Ensure we have at least 2 statements remaining for the smallest pattern
        if index + 1 >= statements.len() {
            return Ok(None);
        }

        // Pattern: FIELD_SUB_ADD_PARAM
        // stmt1: acc1.field -= param (or acc1.field = acc1.field - param)
        // stmt2: acc2.field += param (or acc2.field = acc2.field + param)

        let stmt1 = &statements[index];
        let stmt2 = &statements[index + 1];

        if let (
            AstNode::FieldAssignment {
                object: obj1,
                field: field1,
                value: val1,
            },
            AstNode::FieldAssignment {
                object: obj2,
                field: field2,
                value: val2,
            },
        ) = (stmt1, stmt2)
        {
            // Check first statement is SUB using param
            if let Some((acc1_idx, offset1, param1_idx)) =
                self.match_field_sub_param(obj1, field1, val1)
            {
                // Check second statement is ADD using SAME param
                if let Some((acc2_idx, offset2, param2_idx)) =
                    self.match_field_add_param(obj2, field2, val2)
                {
                    if param1_idx == param2_idx {
                        #[cfg(debug_assertions)]
                        println!("FUSED_DEBUG: EMITTING FIELD_SUB_ADD_PARAM! acc1={} off1={} acc2={} off2={} param={}", 
                            acc1_idx, offset1, acc2_idx, offset2, param1_idx);

                        emitter.emit_opcode(FIELD_SUB_ADD_PARAM);
                        emitter.emit_u8(acc1_idx);
                        emitter.emit_u32(offset1);
                        emitter.emit_u8(acc2_idx);
                        emitter.emit_u32(offset2);
                        emitter.emit_u8(param1_idx);

                        return Ok(Some(2)); // Consumed 2 statements
                    }
                }
            }
        }

        Ok(None)
    }

    /// Match field assignment with SUB param pattern: field -= param
    fn match_field_sub_param(
        &self,
        object: &AstNode,
        field: &str,
        value: &AstNode,
    ) -> Option<(u8, u32, u8)> {
        // Resolve target account/field first
        let (acc_idx, offset) = self.resolve_account_field(object, field)?;

        // Match value expression: field - param
        let param_idx = self.match_field_arithmetic_pattern(object, field, value, "sub")?;

        Some((acc_idx, offset, param_idx))
    }

    /// Match field assignment with ADD param pattern: field += param
    fn match_field_add_param(
        &self,
        object: &AstNode,
        field: &str,
        value: &AstNode,
    ) -> Option<(u8, u32, u8)> {
        // Resolve target account/field first
        let (acc_idx, offset) = self.resolve_account_field(object, field)?;

        // Match value expression: field + param
        let param_idx = self.match_field_arithmetic_pattern(object, field, value, "add")?;

        Some((acc_idx, offset, param_idx))
    }

    /// Helper to resolve account field info
    fn resolve_account_field(&self, object: &AstNode, field: &str) -> Option<(u8, u32)> {
        if let AstNode::Identifier(account_name) = object {
            if let Some(field_info) = self.local_symbol_table.get(account_name) {
                let account_type = &field_info.field_type;
                if let Ok(offset) =
                    self.calculate_account_field_offset(account_type, field, account_name)
                {
                    let acc_idx =
                        crate::bytecode_generator::account_utils::account_index_from_param_offset(
                            field_info.offset,
                        );
                    return Some((acc_idx, offset));
                }
            }
        }
        None
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
        println!(
            "FUSED_T2_DEBUG: try_emit_fused_field_assignment for field='{}' value={:?}",
            field,
            std::mem::discriminant(value)
        );

        // Get target account and field info
        let (target_acc_idx, target_offset) = match object {
            AstNode::Identifier(account_name) => {
                if let Some(field_info) = self.local_symbol_table.get(account_name) {
                    let account_type = &field_info.field_type;
                    if let Ok(offset) =
                        self.calculate_account_field_offset(account_type, field, account_name)
                    {
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

        // Pattern: field = field.add(param) -> FIELD_ADD_PARAM
        if let Some(param_idx) = self.match_field_arithmetic_pattern(object, field, value, "add") {
            #[cfg(debug_assertions)]
            println!(
                "FUSED_DEBUG: EMITTING FIELD_ADD_PARAM! acc={} offset={} param={}",
                target_acc_idx, target_offset, param_idx
            );
            emitter.emit_opcode(FIELD_ADD_PARAM);
            emitter.emit_u8(target_acc_idx);
            emitter.emit_u32(target_offset);
            emitter.emit_u8(param_idx);
            return Ok(true);
        }

        // Pattern: field = field.sub(param) -> FIELD_SUB_PARAM
        if let Some(param_idx) = self.match_field_arithmetic_pattern(object, field, value, "sub") {
            #[cfg(debug_assertions)]
            println!(
                "FUSED_DEBUG: EMITTING FIELD_SUB_PARAM! acc={} offset={} param={}",
                target_acc_idx, target_offset, param_idx
            );
            emitter.emit_opcode(FIELD_SUB_PARAM);
            emitter.emit_u8(target_acc_idx);
            emitter.emit_u32(target_offset);
            emitter.emit_u8(param_idx);
            return Ok(true);
        }

        // ===== TIER 3 PATTERNS =====

        // Pattern: field = 0 -> STORE_FIELD_ZERO
        if self.is_literal_zero(value) {
            #[cfg(debug_assertions)]
            println!(
                "FUSED_DEBUG: EMITTING STORE_FIELD_ZERO! acc={} offset={}",
                target_acc_idx, target_offset
            );
            emitter.emit_opcode(STORE_FIELD_ZERO);
            emitter.emit_u8(target_acc_idx);
            emitter.emit_u32(target_offset);
            return Ok(true);
        }

        // Pattern: field = param -> STORE_PARAM_TO_FIELD
        if let Some(param_idx) = self.match_parameter(value) {
            #[cfg(debug_assertions)]
            println!(
                "FUSED_DEBUG: EMITTING STORE_PARAM_TO_FIELD! acc={} offset={} param={}",
                target_acc_idx, target_offset, param_idx
            );
            emitter.emit_opcode(STORE_PARAM_TO_FIELD);
            emitter.emit_u8(target_acc_idx);
            emitter.emit_u32(target_offset);
            emitter.emit_u8(param_idx);
            return Ok(true);
        }

        // Pattern: field = account.key -> STORE_KEY_TO_FIELD
        if let Some(key_acc_idx) = self.match_account_key_access(value) {
            #[cfg(debug_assertions)]
            println!(
                "FUSED_DEBUG: EMITTING STORE_KEY_TO_FIELD! acc={} offset={} key_acc={}",
                target_acc_idx, target_offset, key_acc_idx
            );
            emitter.emit_opcode(STORE_KEY_TO_FIELD);
            emitter.emit_u8(target_acc_idx);
            emitter.emit_u32(target_offset);
            emitter.emit_u8(key_acc_idx);
            return Ok(true);
        }

        Ok(false)
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
        if let AstNode::MethodCall {
            object: method_obj,
            method,
            args,
        } = value
        {
            if method == operation && args.len() == 1 {
                // The MethodCall object should be a FieldAccess to same account.field
                if let AstNode::FieldAccess {
                    object: field_obj,
                    field: field_name,
                } = method_obj.as_ref()
                {
                    if field_name == target_field {
                        if let (
                            AstNode::Identifier(target_name),
                            AstNode::Identifier(source_name),
                        ) = (target_object, field_obj.as_ref())
                        {
                            if target_name == source_name {
                                return self.match_parameter(&args[0]);
                            }
                        }
                    }
                }
            }
        }

        // Pattern 2: BinaryExpression - object.field +/- param
        if let AstNode::BinaryExpression {
            left,
            operator,
            right,
        } = value
        {
            // Check operator matches
            let expected_op = if operation == "add" { "+" } else { "-" };
            if operator != expected_op {
                return None;
            }

            // Left side should be FieldAccess to same account.field
            if let AstNode::FieldAccess {
                object: field_obj,
                field: field_name,
            } = left.as_ref()
            {
                if field_name != target_field {
                    return None;
                }

                if let (AstNode::Identifier(target_name), AstNode::Identifier(source_name)) =
                    (target_object, field_obj.as_ref())
                {
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

    /// Match pattern: pubkey-field equality check (field-to-field only).
    /// Returns: (acc1_idx, offset1, acc2_idx, offset2)
    fn match_pubkey_eq_any(&self, condition: &AstNode) -> Option<(u8, u32, u8, u32)> {
        // Pattern 1: MethodCall - field.eq(other)
        if let AstNode::MethodCall {
            object,
            method,
            args,
        } = condition
        {
            if method == "eq" && args.len() == 1 {
                // Left side: pubkey field
                let (acc1_idx, offset1) = self.match_pubkey_field_access(object)?;

                // Right side: pubkey field
                if let Some((acc2_idx, offset2)) = self.match_pubkey_field_access(&args[0]) {
                    return Some((acc1_idx, offset1, acc2_idx, offset2));
                }
            }
        }

        // Pattern 2: BinaryExpression - field == other
        if let AstNode::BinaryExpression {
            left,
            operator,
            right,
        } = condition
        {
            if operator == "==" {
                #[cfg(debug_assertions)]
                println!("FUSED_DEBUG: Check BinaryExpression == for PUBKEY_EQ");

                // Left side: pubkey field
                if let Some((acc1_idx, offset1)) = self.match_pubkey_field_access(left) {
                    #[cfg(debug_assertions)]
                    println!(
                        "FUSED_DEBUG: Left side matched pubkey field: acc={} offset={}",
                        acc1_idx, offset1
                    );

                    // Right side: pubkey field
                    if let Some((acc2_idx, offset2)) = self.match_pubkey_field_access(right) {
                        #[cfg(debug_assertions)]
                        println!(
                            "FUSED_DEBUG: Right side matched pubkey field: acc={} offset={}",
                            acc2_idx, offset2
                        );
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

    fn resolve_account_field_type(&self, account_type: &str, field: &str) -> Option<String> {
        let account_system = self.account_system.as_ref()?;
        let registry = account_system.get_account_registry();

        if let Some(info) = registry.account_types.get(account_type) {
            if let Some(field_info) = info.fields.get(field) {
                return Some(field_info.field_type.clone());
            }
        }

        if account_type.contains("::") {
            let qualified_suffix = format!("::{}", account_type);
            if let Some((_, info)) = registry
                .account_types
                .iter()
                .find(|(key, _)| key.ends_with(&qualified_suffix))
            {
                if let Some(field_info) = info.fields.get(field) {
                    return Some(field_info.field_type.clone());
                }
            }
        }

        let tail = account_type.rsplit("::").next().unwrap_or(account_type);
        let mut tail_matches = registry
            .account_types
            .iter()
            .filter(|(key, _)| key.rsplit("::").next().unwrap_or(*key) == tail);
        if let Some((_, info)) = tail_matches.next() {
            if tail_matches.next().is_none() {
                if let Some(field_info) = info.fields.get(field) {
                    return Some(field_info.field_type.clone());
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
                    let field_type = self.resolve_account_field_type(account_type, field)?;
                    if field_type != "pubkey" {
                        return None;
                    }
                    if let Ok(offset) =
                        self.calculate_account_field_offset(account_type, field, account_name)
                    {
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

    /// Match account.key access pattern
    fn match_account_key_access(&self, node: &AstNode) -> Option<u8> {
        if let AstNode::FieldAccess { object, field } = node {
            if field == "key" {
                if let AstNode::Identifier(account_name) = object.as_ref() {
                    return self.resolve_account_param_by_name(account_name);
                }

                if let AstNode::FieldAccess {
                    object: inner_object,
                    field: inner_field,
                } = object.as_ref()
                {
                    if inner_field == "ctx" {
                        if let AstNode::Identifier(account_name) = inner_object.as_ref() {
                            return self.resolve_account_param_by_name(account_name);
                        }
                    }
                }
            }
        }
        None
    }

    fn resolve_session_owner_context(&self, signer_idx: u8) -> Option<(u8, u32, u32)> {
        let params = self.current_function_parameters.as_ref()?;
        let session_param = params
            .iter()
            .find(|param| param.name == session_support::IMPLICIT_SESSION_PARAM_NAME)?;
        let session_attr = session_param
            .attributes
            .iter()
            .find(|attr| attr.name == "session")?;

        let authority_name = session_attr.args.iter().find_map(|arg| match arg {
            AstNode::Assignment { target, value } if target == "authority" => {
                if let AstNode::Identifier(name) = value.as_ref() {
                    Some(name.as_str())
                } else {
                    None
                }
            }
            _ => None,
        })?;

        let authority_idx = self.resolve_account_param_by_name(authority_name)?;
        if authority_idx != signer_idx {
            return None;
        }

        let session_idx = self.resolve_account_param_by_name(&session_param.name)?;
        let session_field_type = self.local_symbol_table.get(&session_param.name)?.field_type.clone();
        let session_delegate_offset = self
            .calculate_account_field_offset(
                &session_field_type,
                "delegate",
                &session_param.name,
            )
            .ok()?;
        let session_authority_offset = self
            .calculate_account_field_offset(
                &session_field_type,
                "authority",
                &session_param.name,
            )
            .ok()?;

        Some((session_idx, session_delegate_offset, session_authority_offset))
    }
}
