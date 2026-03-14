//! Universal Fused Opcodes Handler (0xC0-0xCF)
//!
//! High-impact universal patterns that apply across all DeFi contracts.
//! Each fused opcode replaces 3-4 individual opcodes, saving ~300 CU per use.

use crate::{
    context::ExecutionManager,
    debug_log,
    error::{CompactResult, VMErrorCode},
};
use core::ptr;
use five_protocol::{opcodes::*, ValueRef};

#[inline(always)]
fn param_value(ctx: &ExecutionManager, param_idx: u8) -> CompactResult<ValueRef> {
    ctx.parameters()
        .get(param_idx as usize)
        .cloned()
        .ok_or(VMErrorCode::InvalidParameter)
}

#[inline(always)]
fn param_u64(ctx: &mut ExecutionManager, param_idx: u8) -> CompactResult<u64> {
    let value = *ctx
        .parameters()
        .get(param_idx as usize)
        .ok_or(VMErrorCode::InvalidParameter)?;
    crate::utils::resolve_u64(value, ctx)
}

#[inline(always)]
fn read_u64_le(data: &[u8], offset: usize) -> u64 {
    // Safe because callers perform bounds checks before calling.
    let raw = unsafe { ptr::read_unaligned(data.as_ptr().add(offset) as *const u64) };
    u64::from_le(raw)
}

#[inline(always)]
fn write_u64_le(data: &mut [u8], offset: usize, value: u64) {
    // Safe because callers perform bounds checks before calling.
    unsafe {
        ptr::write_unaligned(data.as_mut_ptr().add(offset) as *mut u64, value.to_le());
    }
}

#[inline(always)]
fn eq_32_bytes(a: &[u8], a_off: usize, b: &[u8], b_off: usize) -> bool {
    read_u64_le(a, a_off) == read_u64_le(b, b_off)
        && read_u64_le(a, a_off + 8) == read_u64_le(b, b_off + 8)
        && read_u64_le(a, a_off + 16) == read_u64_le(b, b_off + 16)
        && read_u64_le(a, a_off + 24) == read_u64_le(b, b_off + 24)
}

/// Handle universal fused operations (0xC0-0xCF)
#[inline(always)]
pub fn handle_fused_ops(opcode: u8, ctx: &mut ExecutionManager) -> CompactResult<()> {
    match opcode {
        // REQUIRE_GTE_U64: LOAD_FIELD + LOAD_PARAM + GTE + REQUIRE fused
        // Encoding: acc(u8) offset(u32) param(u8)
        // Saves 300 CU by avoiding 4 opcode dispatches
        REQUIRE_GTE_U64 => {
            let acc_idx = ctx.fetch_byte()?;
            let field_offset = ctx.fetch_u32()?;
            let param_idx = ctx.fetch_byte()?;

            // Load field value directly from account data
            let account = ctx.get_account_for_read(acc_idx)?;
            let data = unsafe { account.borrow_data_unchecked() };

            if (field_offset as usize) + 8 > data.len() {
                debug_log!("MitoVM: REQUIRE_GTE_U64 field offset out of bounds");
                return Err(VMErrorCode::InvalidAccountData);
            }

            let field_value = read_u64_le(&data, field_offset as usize);

            // Load param value using same pattern as locals.rs
            let param_value = param_u64(ctx, param_idx)?;

            // GTE + REQUIRE fused
            if field_value < param_value {
                debug_log!(
                    "MitoVM: REQUIRE_GTE_U64 failed: {} < {}",
                    field_value,
                    param_value
                );
                return Err(VMErrorCode::ConstraintViolation);
            }
            debug_log!(
                "MitoVM: REQUIRE_GTE_U64 passed: {} >= {}",
                field_value,
                param_value
            );
        }

        // REQUIRE_NOT_BOOL: LOAD_FIELD + NOT + REQUIRE fused
        // Encoding: acc(u8) offset(u32)
        // Saves 200 CU
        REQUIRE_NOT_BOOL => {
            let acc_idx = ctx.fetch_byte()?;
            let field_offset = ctx.fetch_u32()?;

            let account = ctx.get_account_for_read(acc_idx)?;
            let data = unsafe { account.borrow_data_unchecked() };

            if (field_offset as usize) >= data.len() {
                debug_log!("MitoVM: REQUIRE_NOT_BOOL field offset out of bounds");
                return Err(VMErrorCode::InvalidAccountData);
            }

            let bool_value = data[field_offset as usize] != 0;

            // NOT + REQUIRE: require the field is false
            if bool_value {
                debug_log!("MitoVM: REQUIRE_NOT_BOOL failed: field is true");
                return Err(VMErrorCode::ConstraintViolation);
            }
            debug_log!("MitoVM: REQUIRE_NOT_BOOL passed");
        }

        // FIELD_ADD_PARAM: LOAD_FIELD + LOAD_PARAM + ADD + STORE_FIELD fused
        // Encoding: acc(u8) offset(u32) param(u8)
        // Saves 300 CU
        FIELD_ADD_PARAM => {
            let acc_idx = ctx.fetch_byte()?;
            let field_offset = ctx.fetch_u32()?;
            let param_idx = ctx.fetch_byte()?;

            // Load param value first using same pattern as locals.rs
            let param_value = param_u64(ctx, param_idx)?;

            // Get account for write
            let account = ctx.get_account_for_write(acc_idx)?;
            let data = unsafe { account.borrow_mut_data_unchecked() };

            if (field_offset as usize) + 8 > data.len() {
                debug_log!("MitoVM: FIELD_ADD_PARAM field offset out of bounds");
                return Err(VMErrorCode::InvalidAccountData);
            }

            // Read current value
            let current_value = read_u64_le(&data, field_offset as usize);

            // Add and store
            let new_value = current_value.wrapping_add(param_value);
            write_u64_le(data, field_offset as usize, new_value);

            debug_log!(
                "MitoVM: FIELD_ADD_PARAM: {} + {} = {}",
                current_value,
                param_value,
                new_value
            );
        }

        // FIELD_SUB_PARAM: LOAD_FIELD + LOAD_PARAM + SUB + STORE_FIELD fused
        // Encoding: acc(u8) offset(u32) param(u8)
        // Saves 300 CU
        FIELD_SUB_PARAM => {
            let acc_idx = ctx.fetch_byte()?;
            let field_offset = ctx.fetch_u32()?;
            let param_idx = ctx.fetch_byte()?;

            // Load param value first using same pattern as locals.rs
            let param_value = param_u64(ctx, param_idx)?;

            // Get account for write
            let account = ctx.get_account_for_write(acc_idx)?;
            let data = unsafe { account.borrow_mut_data_unchecked() };

            if (field_offset as usize) + 8 > data.len() {
                debug_log!("MitoVM: FIELD_SUB_PARAM field offset out of bounds");
                return Err(VMErrorCode::InvalidAccountData);
            }

            // Read current value
            let current_value = read_u64_le(&data, field_offset as usize);

            // Sub and store
            let new_value = current_value.wrapping_sub(param_value);
            write_u64_le(data, field_offset as usize, new_value);

            debug_log!(
                "MitoVM: FIELD_SUB_PARAM: {} - {} = {}",
                current_value,
                param_value,
                new_value
            );
        }

        // REQUIRE_PARAM_GT_ZERO: LOAD_PARAM + PUSH_0 + GT + REQUIRE fused
        // Encoding: param(u8)
        // Saves 300 CU
        REQUIRE_PARAM_GT_ZERO => {
            let param_idx = ctx.fetch_byte()?;
            let param_value = param_u64(ctx, param_idx)?;

            if param_value == 0 {
                debug_log!("MitoVM: REQUIRE_PARAM_GT_ZERO failed: param is 0");
                return Err(VMErrorCode::ConstraintViolation);
            }
            debug_log!("MitoVM: REQUIRE_PARAM_GT_ZERO passed: {} > 0", param_value);
        }

        // REQUIRE_LOCAL_GT_ZERO: GET_LOCAL + PUSH_0 + GT + REQUIRE fused
        // Encoding: local(u8)
        // Saves 200+ CU in tight loops
        REQUIRE_LOCAL_GT_ZERO => {
            let local_idx = ctx.fetch_byte()?;
            let local_value = ctx.get_local(local_idx)?;
            let local_u64 = crate::utils::resolve_u64(local_value, ctx)?;

            if local_u64 == 0 {
                debug_log!(
                    "MitoVM: REQUIRE_LOCAL_GT_ZERO failed: local {} is 0",
                    local_idx
                );
                return Err(VMErrorCode::ConstraintViolation);
            }
            debug_log!(
                "MitoVM: REQUIRE_LOCAL_GT_ZERO passed: local {} = {}",
                local_idx,
                local_u64
            );
        }

        // REQUIRE_EQ_PUBKEY: Compare two pubkey fields from accounts
        // Encoding: acc1(u8) offset1(u32) acc2(u8) offset2(u32)
        // Saves 300 CU
        REQUIRE_EQ_PUBKEY => {
            let acc1_idx = ctx.fetch_byte()?;
            let offset1 = ctx.fetch_u32()?;
            let acc2_idx = ctx.fetch_byte()?;
            let offset2 = ctx.fetch_u32()?;

            // Load first pubkey field
            let account1 = ctx.get_account_for_read(acc1_idx)?;
            let data1 = unsafe { account1.borrow_data_unchecked() };
            if (offset1 as usize) + 32 > data1.len() {
                debug_log!(
                    "MitoVM: REQUIRE_EQ_PUBKEY acc1 bounds check failed: offset={} + 32 > len={}",
                    offset1,
                    data1.len()
                );
                return Err(VMErrorCode::InvalidAccountData);
            }
            // Load second pubkey field
            let account2 = ctx.get_account_for_read(acc2_idx)?;
            let data2 = unsafe { account2.borrow_data_unchecked() };
            if (offset2 as usize) + 32 > data2.len() {
                debug_log!(
                    "MitoVM: REQUIRE_EQ_PUBKEY acc2 bounds check failed: offset={} + 32 > len={}",
                    offset2,
                    data2.len()
                );
                return Err(VMErrorCode::InvalidAccountData);
            }
            // Compare
            if !eq_32_bytes(&data1, offset1 as usize, &data2, offset2 as usize) {
                debug_log!("MitoVM: REQUIRE_EQ_PUBKEY failed: pubkeys don't match");
                return Err(VMErrorCode::ConstraintViolation);
            }
            debug_log!("MitoVM: REQUIRE_EQ_PUBKEY passed");
        }

        // CHECK_SIGNER_WRITABLE: CHECK_SIGNER + CHECK_WRITABLE fused
        // Encoding: acc(u8)
        // Saves 100 CU
        CHECK_SIGNER_WRITABLE => {
            let acc_idx = ctx.fetch_byte()?;
            let account = ctx.get_account_for_read(acc_idx)?;

            if !account.is_signer() {
                debug_log!("MitoVM: CHECK_SIGNER_WRITABLE failed: not signer");
                return Err(VMErrorCode::ConstraintViolation);
            }
            if !account.is_writable() {
                debug_log!("MitoVM: CHECK_SIGNER_WRITABLE failed: not writable");
                return Err(VMErrorCode::ConstraintViolation);
            }
            debug_log!(
                "MitoVM: CHECK_SIGNER_WRITABLE passed for account {}",
                acc_idx
            );
        }

        // ===== TIER 3 UNIVERSAL FUSED OPCODES =====

        // STORE_PARAM_TO_FIELD: LOAD_PARAM + STORE_FIELD fused
        // Encoding: acc(u8) offset(u32) param(u8)
        // Saves 100 CU
        STORE_PARAM_TO_FIELD => {
            let acc_idx = ctx.fetch_byte()?;
            let field_offset = ctx.fetch_u32()?;
            let param_idx = ctx.fetch_byte()?;

            // Load param value generically
            // Clone the ValueRef (cheap, just references)
            let param_value = param_value(ctx, param_idx)?;

            // Store to field
            let account = ctx.get_account_for_write(acc_idx)?;
            let data = unsafe { account.borrow_mut_data_unchecked() };

            // Use generic store function from memory handler to support all types (U64, Pubkey, String, etc.)
            crate::handlers::memory::store_value_into_buffer(
                data,
                field_offset as usize,
                param_value,
                ctx,
            )?;

            debug_log!(
                "MitoVM: STORE_PARAM_TO_FIELD stored param {} at offset {}",
                param_idx,
                field_offset
            );
        }

        // STORE_FIELD_ZERO: PUSH_0 + STORE_FIELD fused
        // Encoding: acc(u8) offset(u32)
        // Saves 100 CU
        STORE_FIELD_ZERO => {
            let acc_idx = ctx.fetch_byte()?;
            let field_offset = ctx.fetch_u32()?;

            let account = ctx.get_account_for_write(acc_idx)?;
            let data = unsafe { account.borrow_mut_data_unchecked() };

            if (field_offset as usize) + 8 > data.len() {
                debug_log!("MitoVM: STORE_FIELD_ZERO offset out of bounds");
                return Err(VMErrorCode::InvalidAccountData);
            }

            // Store zero (8 bytes for u64)
            write_u64_le(data, field_offset as usize, 0);

            debug_log!("MitoVM: STORE_FIELD_ZERO at offset {}", field_offset);
        }

        // STORE_KEY_TO_FIELD: GET_KEY + STORE_FIELD fused
        // Encoding: acc(u8) offset(u32) key_acc(u8)
        // Saves 100 CU
        STORE_KEY_TO_FIELD => {
            let acc_idx = ctx.fetch_byte()?;
            let field_offset = ctx.fetch_u32()?;
            let key_acc_idx = ctx.fetch_byte()?;

            // Get the key from key_acc
            let key_account = ctx.get_account_for_read(key_acc_idx)?;
            let key_bytes = key_account.key().as_ref();

            // Store to field
            let account = ctx.get_account_for_write(acc_idx)?;
            let data = unsafe { account.borrow_mut_data_unchecked() };

            if (field_offset as usize) + 32 > data.len() {
                debug_log!("MitoVM: STORE_KEY_TO_FIELD offset out of bounds");
                return Err(VMErrorCode::InvalidAccountData);
            }

            data[field_offset as usize..field_offset as usize + 32].copy_from_slice(&key_bytes);

            debug_log!(
                "MitoVM: STORE_KEY_TO_FIELD stored key at offset {}",
                field_offset
            );
        }

        // REQUIRE_EQ_FIELDS: Compare two u64 fields (field-to-field)
        // Encoding: acc1(u8) offset1(u32) acc2(u8) offset2(u32)
        // Saves 300 CU
        REQUIRE_EQ_FIELDS => {
            let acc1_idx = ctx.fetch_byte()?;
            let offset1 = ctx.fetch_u32()?;
            let acc2_idx = ctx.fetch_byte()?;
            let offset2 = ctx.fetch_u32()?;

            // Load first field (32 bytes for pubkey comparison)
            let account1 = ctx.get_account_for_read(acc1_idx)?;
            let data1 = unsafe { account1.borrow_data_unchecked() };
            if (offset1 as usize) + 32 > data1.len() {
                return Err(VMErrorCode::InvalidAccountData);
            }
            // Load second field
            let account2 = ctx.get_account_for_read(acc2_idx)?;
            let data2 = unsafe { account2.borrow_data_unchecked() };
            if (offset2 as usize) + 32 > data2.len() {
                return Err(VMErrorCode::InvalidAccountData);
            }
            // Compare
            if !eq_32_bytes(&data1, offset1 as usize, &data2, offset2 as usize) {
                debug_log!("MitoVM: REQUIRE_EQ_FIELDS failed: fields don't match");
                return Err(VMErrorCode::ConstraintViolation);
            }
            debug_log!("MitoVM: REQUIRE_EQ_FIELDS passed");
        }

        // FIELD_SUB_ADD_PARAM: acc1.field -= param; acc2.field += param
        // Format: FIELD_SUB_ADD_PARAM acc1, off1, acc2, off2, param_idx
        FIELD_SUB_ADD_PARAM => {
            let acc1_idx = ctx.fetch_byte()?;
            let off1 = ctx.fetch_u32()?;
            let acc2_idx = ctx.fetch_byte()?;
            let off2 = ctx.fetch_u32()?;
            let param_idx = ctx.fetch_byte()?;

            // 1. Load parameter
            let param_value_ref = param_value(ctx, param_idx)?;
            let Some(param_value) = param_value_ref.as_u64() else {
                return Err(VMErrorCode::TypeMismatch);
            };

            // 2. Process Account 1 (Subtract)
            {
                let account1 = ctx.get_account_for_write(acc1_idx)?;
                let data1 = unsafe { account1.borrow_mut_data_unchecked() };

                if (off1 as usize) + 8 > data1.len() {
                    return Err(VMErrorCode::InvalidAccountData);
                }

                let current1 = read_u64_le(&data1, off1 as usize);
                // Use wrapping sub for consistency with other ops, could use checked if desired
                let new_val1 = current1.wrapping_sub(param_value);
                write_u64_le(data1, off1 as usize, new_val1);
            }

            // 3. Process Account 2 (Add)
            {
                let account2 = ctx.get_account_for_write(acc2_idx)?;
                let data2 = unsafe { account2.borrow_mut_data_unchecked() };

                if (off2 as usize) + 8 > data2.len() {
                    return Err(VMErrorCode::InvalidAccountData);
                }

                let current2 = read_u64_le(&data2, off2 as usize);
                let new_val2 = current2.wrapping_add(param_value);
                write_u64_le(data2, off2 as usize, new_val2);
            }

            debug_log!(
                "MitoVM: FIELD_SUB_ADD_PARAM transferred {} from acc{} to acc{}",
                param_value,
                acc1_idx,
                acc2_idx
            );
        }

        // REQUIRE_PARAM_LTE_IMM: param <= immediate
        // Format: REQUIRE_PARAM_LTE_IMM param_idx, imm_u8
        REQUIRE_PARAM_LTE_IMM => {
            let param_idx = ctx.fetch_byte()?;
            let imm = ctx.fetch_byte()? as u64;

            let param_value = param_u64(ctx, param_idx)?;

            if param_value > imm {
                debug_log!(
                    "MitoVM: REQUIRE_PARAM_LTE_IMM failed: {} > {}",
                    param_value,
                    imm
                );
                return Err(VMErrorCode::ConstraintViolation);
            }
        }

        // REQUIRE_FIELD_EQ_IMM: acc.field == immediate
        // Format: REQUIRE_FIELD_EQ_IMM acc_idx, offset, imm_u8
        REQUIRE_FIELD_EQ_IMM => {
            let acc_idx = ctx.fetch_byte()?;
            let offset = ctx.fetch_u32()?;
            let imm = ctx.fetch_byte()? as u64;

            let account = ctx.get_account_for_read(acc_idx)?;
            let data = unsafe { account.borrow_data_unchecked() };

            if (offset as usize) + 8 > data.len() {
                return Err(VMErrorCode::InvalidAccountData);
            }

            let field_val = read_u64_le(&data, offset as usize);

            if field_val != imm {
                debug_log!(
                    "MitoVM: REQUIRE_FIELD_EQ_IMM failed: {} != {}",
                    field_val,
                    imm
                );
                return Err(VMErrorCode::ConstraintViolation);
            }
        }

        // REQUIRE_BATCH: Evaluate multiple guard clauses in a single dispatch.
        // Format: REQUIRE_BATCH count [tag + payload]...
        REQUIRE_BATCH => {
            let clause_count = ctx.fetch_byte()?;
            if clause_count > REQUIRE_BATCH_MAX_CLAUSES {
                return Err(VMErrorCode::InvalidInstruction);
            }

            for _ in 0..clause_count {
                let tag = ctx.fetch_byte()?;
                match tag {
                    REQUIRE_BATCH_PARAM_GT_ZERO => {
                        let param_idx = ctx.fetch_byte()?;
                        if param_u64(ctx, param_idx)? == 0 {
                            return Err(VMErrorCode::ConstraintViolation);
                        }
                    }
                    REQUIRE_BATCH_LOCAL_GT_ZERO => {
                        let local_idx = ctx.fetch_byte()?;
                        let local_value = ctx.get_local(local_idx)?;
                        let local_u64 = crate::utils::resolve_u64(local_value, ctx)?;
                        if local_u64 == 0 {
                            return Err(VMErrorCode::ConstraintViolation);
                        }
                    }
                    REQUIRE_BATCH_FIELD_NOT_BOOL => {
                        let acc_idx = ctx.fetch_byte()?;
                        let offset = ctx.fetch_u32()?;
                        let account = ctx.get_account_for_read(acc_idx)?;
                        let data = unsafe { account.borrow_data_unchecked() };
                        let off = offset as usize;
                        if off >= data.len() {
                            return Err(VMErrorCode::InvalidAccountData);
                        }
                        if data[off] != 0 {
                            return Err(VMErrorCode::ConstraintViolation);
                        }
                    }
                    REQUIRE_BATCH_FIELD_GTE_PARAM => {
                        let acc_idx = ctx.fetch_byte()?;
                        let offset = ctx.fetch_u32()?;
                        let param_idx = ctx.fetch_byte()?;
                        let account = ctx.get_account_for_read(acc_idx)?;
                        let data = unsafe { account.borrow_data_unchecked() };
                        let off = offset as usize;
                        if off + 8 > data.len() {
                            return Err(VMErrorCode::InvalidAccountData);
                        }
                        let field_value = read_u64_le(&data, off);
                        let param_value = param_u64(ctx, param_idx)?;
                        if field_value < param_value {
                            return Err(VMErrorCode::ConstraintViolation);
                        }
                    }
                    REQUIRE_BATCH_OWNER_EQ_SIGNER => {
                        let acc_idx = ctx.fetch_byte()?;
                        let signer_idx = ctx.fetch_byte()?;
                        let offset = ctx.fetch_u32()?;

                        let account = ctx.get_account_for_read(acc_idx)?;
                        let data = unsafe { account.borrow_data_unchecked() };
                        let off = offset as usize;
                        if off + 32 > data.len() {
                            return Err(VMErrorCode::InvalidAccountData);
                        }

                        let signer = ctx.get_account_for_read(signer_idx)?;
                        if !eq_32_bytes(&data, off, signer.key().as_ref(), 0) {
                            return Err(VMErrorCode::ConstraintViolation);
                        }
                    }
                    REQUIRE_BATCH_PARAM_LTE_IMM => {
                        let param_idx = ctx.fetch_byte()?;
                        let imm = ctx.fetch_byte()? as u64;
                        if param_u64(ctx, param_idx)? > imm {
                            return Err(VMErrorCode::ConstraintViolation);
                        }
                    }
                    REQUIRE_BATCH_FIELD_EQ_IMM => {
                        let acc_idx = ctx.fetch_byte()?;
                        let offset = ctx.fetch_u32()?;
                        let imm = ctx.fetch_byte()? as u64;
                        let account = ctx.get_account_for_read(acc_idx)?;
                        let data = unsafe { account.borrow_data_unchecked() };
                        let off = offset as usize;
                        if off + 8 > data.len() {
                            return Err(VMErrorCode::InvalidAccountData);
                        }
                        let field_val = read_u64_le(&data, off);
                        if field_val != imm {
                            return Err(VMErrorCode::ConstraintViolation);
                        }
                    }
                    REQUIRE_BATCH_PUBKEY_FIELD_EQ_PARAM => {
                        let acc_idx = ctx.fetch_byte()?;
                        let offset = ctx.fetch_u32()?;
                        let param_idx = ctx.fetch_byte()?;

                        let account = ctx.get_account_for_read(acc_idx)?;
                        let data = unsafe { account.borrow_data_unchecked() };
                        let off = offset as usize;
                        if off + 32 > data.len() {
                            return Err(VMErrorCode::InvalidAccountData);
                        }

                        let param_ref = param_value(ctx, param_idx)?;
                        let param_pubkey = ctx.extract_pubkey(&param_ref)?;
                        if !eq_32_bytes(&data, off, &param_pubkey, 0) {
                            return Err(VMErrorCode::ConstraintViolation);
                        }
                    }
                    _ => return Err(VMErrorCode::InvalidInstruction),
                }
            }
        }

        _ => {
            debug_log!("MitoVM: Unknown fused opcode: {}", opcode);
            return Err(VMErrorCode::InvalidInstruction);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{context::ExecutionContext, stack::StackStorage, FIVE_VM_PROGRAM_ID};
    use pinocchio::{account_info::AccountInfo, pubkey::Pubkey};

    fn make_account(
        key: Pubkey,
        owner: Pubkey,
        data: Vec<u8>,
        is_signer: bool,
        is_writable: bool,
    ) -> AccountInfo {
        let key_ref = Box::leak(Box::new(key));
        let owner_ref = Box::leak(Box::new(owner));
        let lamports_ref = Box::leak(Box::new(1u64));
        let data_ref = Box::leak(data.into_boxed_slice());

        AccountInfo::new(
            key_ref,
            is_signer,
            is_writable,
            lamports_ref,
            data_ref,
            owner_ref,
            false,
            0,
        )
    }

    fn run_require_batch(
        payload: &[u8],
        accounts: &[AccountInfo],
        params: &[(usize, ValueRef)],
        locals: &[(u8, ValueRef)],
    ) -> CompactResult<()> {
        let mut storage = StackStorage::new();
        let mut ctx = ExecutionContext::new(
            payload,
            accounts,
            FIVE_VM_PROGRAM_ID,
            &[],
            0,
            &mut storage,
            0,
            0,
            0,
            0,
            0,
            0,
        );

        for (idx, value) in params {
            ctx.set_parameter(*idx, value.clone())?;
        }

        if let Some(max_local) = locals.iter().map(|(idx, _)| *idx).max() {
            ctx.allocate_locals(max_local.saturating_add(1))?;
            for (idx, value) in locals {
                ctx.set_local(*idx, value.clone())?;
            }
        }

        handle_fused_ops(REQUIRE_BATCH, &mut ctx)
    }

    #[test]
    fn require_batch_tag_param_gt_zero_pass_and_fail() {
        let pass = run_require_batch(
            &[1, REQUIRE_BATCH_PARAM_GT_ZERO, 1],
            &[],
            &[(1, ValueRef::U64(7))],
            &[],
        );
        assert!(pass.is_ok());

        let fail = run_require_batch(
            &[1, REQUIRE_BATCH_PARAM_GT_ZERO, 1],
            &[],
            &[(1, ValueRef::U64(0))],
            &[],
        );
        assert_eq!(fail, Err(VMErrorCode::ConstraintViolation));
    }

    #[test]
    fn require_batch_tag_local_gt_zero_pass_and_fail() {
        let pass = run_require_batch(
            &[1, REQUIRE_BATCH_LOCAL_GT_ZERO, 0],
            &[],
            &[],
            &[(0, ValueRef::U64(2))],
        );
        assert!(pass.is_ok());

        let fail = run_require_batch(
            &[1, REQUIRE_BATCH_LOCAL_GT_ZERO, 0],
            &[],
            &[],
            &[(0, ValueRef::U64(0))],
        );
        assert_eq!(fail, Err(VMErrorCode::ConstraintViolation));
    }

    #[test]
    fn require_batch_tag_field_not_bool_pass_and_fail() {
        let owner = FIVE_VM_PROGRAM_ID;
        let account_pass = make_account([1; 32], owner, vec![0u8; 16], false, false);
        let pass = run_require_batch(
            &[1, REQUIRE_BATCH_FIELD_NOT_BOOL, 0, 4, 0, 0, 0],
            &[account_pass],
            &[],
            &[],
        );
        assert!(pass.is_ok());

        let mut fail_data = vec![0u8; 16];
        fail_data[4] = 1;
        let account_fail = make_account([2; 32], owner, fail_data, false, false);
        let fail = run_require_batch(
            &[1, REQUIRE_BATCH_FIELD_NOT_BOOL, 0, 4, 0, 0, 0],
            &[account_fail],
            &[],
            &[],
        );
        assert_eq!(fail, Err(VMErrorCode::ConstraintViolation));
    }

    #[test]
    fn require_batch_tag_field_gte_param_pass_and_fail() {
        let owner = FIVE_VM_PROGRAM_ID;
        let mut data = vec![0u8; 24];
        data[8..16].copy_from_slice(&10u64.to_le_bytes());
        let account = make_account([3; 32], owner, data, false, false);

        let pass = run_require_batch(
            &[1, REQUIRE_BATCH_FIELD_GTE_PARAM, 0, 8, 0, 0, 0, 1],
            &[account],
            &[(1, ValueRef::U64(6))],
            &[],
        );
        assert!(pass.is_ok());

        let mut data_fail = vec![0u8; 24];
        data_fail[8..16].copy_from_slice(&5u64.to_le_bytes());
        let account_fail = make_account([4; 32], owner, data_fail, false, false);
        let fail = run_require_batch(
            &[1, REQUIRE_BATCH_FIELD_GTE_PARAM, 0, 8, 0, 0, 0, 1],
            &[account_fail],
            &[(1, ValueRef::U64(9))],
            &[],
        );
        assert_eq!(fail, Err(VMErrorCode::ConstraintViolation));
    }

    #[test]
    fn require_batch_tag_owner_eq_signer_pass_and_fail() {
        let owner = FIVE_VM_PROGRAM_ID;
        let signer_key = [9u8; 32];
        let mut account_data = vec![0u8; 80];
        account_data[16..48].copy_from_slice(&signer_key);

        let account = make_account([5; 32], owner, account_data, false, false);
        let signer = make_account(signer_key, owner, vec![0u8; 1], true, false);
        let pass = run_require_batch(
            &[1, REQUIRE_BATCH_OWNER_EQ_SIGNER, 0, 1, 16, 0, 0, 0],
            &[account, signer],
            &[],
            &[],
        );
        assert!(pass.is_ok());

        let mut bad_data = vec![0u8; 80];
        bad_data[16..48].copy_from_slice(&[7u8; 32]);
        let account_fail = make_account([6; 32], owner, bad_data, false, false);
        let signer_fail = make_account(signer_key, owner, vec![0u8; 1], true, false);
        let fail = run_require_batch(
            &[1, REQUIRE_BATCH_OWNER_EQ_SIGNER, 0, 1, 16, 0, 0, 0],
            &[account_fail, signer_fail],
            &[],
            &[],
        );
        assert_eq!(fail, Err(VMErrorCode::ConstraintViolation));
    }

    #[test]
    fn require_batch_tag_param_lte_imm_pass_and_fail() {
        let pass = run_require_batch(
            &[1, REQUIRE_BATCH_PARAM_LTE_IMM, 1, 9],
            &[],
            &[(1, ValueRef::U64(9))],
            &[],
        );
        assert!(pass.is_ok());

        let fail = run_require_batch(
            &[1, REQUIRE_BATCH_PARAM_LTE_IMM, 1, 9],
            &[],
            &[(1, ValueRef::U64(10))],
            &[],
        );
        assert_eq!(fail, Err(VMErrorCode::ConstraintViolation));
    }

    #[test]
    fn require_batch_tag_field_eq_imm_pass_and_fail() {
        let owner = FIVE_VM_PROGRAM_ID;
        let mut data = vec![0u8; 24];
        data[8..16].copy_from_slice(&4u64.to_le_bytes());
        let account = make_account([7; 32], owner, data, false, false);
        let pass = run_require_batch(
            &[1, REQUIRE_BATCH_FIELD_EQ_IMM, 0, 8, 0, 0, 0, 4],
            &[account],
            &[],
            &[],
        );
        assert!(pass.is_ok());

        let mut data_fail = vec![0u8; 24];
        data_fail[8..16].copy_from_slice(&3u64.to_le_bytes());
        let account_fail = make_account([8; 32], owner, data_fail, false, false);
        let fail = run_require_batch(
            &[1, REQUIRE_BATCH_FIELD_EQ_IMM, 0, 8, 0, 0, 0, 4],
            &[account_fail],
            &[],
            &[],
        );
        assert_eq!(fail, Err(VMErrorCode::ConstraintViolation));
    }

    #[test]
    fn require_batch_tag_pubkey_field_eq_param_pass_and_fail() {
        let owner = FIVE_VM_PROGRAM_ID;
        let key = [11u8; 32];

        let mut data = vec![0u8; 80];
        data[24..56].copy_from_slice(&key);
        let account = make_account([10; 32], owner, data, false, false);
        let key_source = make_account(key, owner, vec![0u8; 1], false, false);
        let pass = run_require_batch(
            &[1, REQUIRE_BATCH_PUBKEY_FIELD_EQ_PARAM, 0, 24, 0, 0, 0, 1],
            &[account, key_source],
            &[(1, ValueRef::PubkeyRef(0xFF01))],
            &[],
        );
        assert!(pass.is_ok());

        let mut bad_data = vec![0u8; 80];
        bad_data[24..56].copy_from_slice(&[12u8; 32]);
        let account_fail = make_account([12; 32], owner, bad_data, false, false);
        let key_source_fail = make_account(key, owner, vec![0u8; 1], false, false);
        let fail = run_require_batch(
            &[1, REQUIRE_BATCH_PUBKEY_FIELD_EQ_PARAM, 0, 24, 0, 0, 0, 1],
            &[account_fail, key_source_fail],
            &[(1, ValueRef::PubkeyRef(0xFF01))],
            &[],
        );
        assert_eq!(fail, Err(VMErrorCode::ConstraintViolation));
    }

    #[test]
    fn require_batch_enforces_max_count_and_invalid_tag() {
        let too_many = run_require_batch(&[REQUIRE_BATCH_MAX_CLAUSES + 1], &[], &[], &[]);
        assert_eq!(too_many, Err(VMErrorCode::InvalidInstruction));

        let bad_tag = run_require_batch(&[1, 0xFE], &[], &[], &[]);
        assert_eq!(bad_tag, Err(VMErrorCode::InvalidInstruction));
    }

    #[test]
    fn require_batch_reports_invalid_param_and_oob_offset() {
        let invalid_param =
            run_require_batch(&[1, REQUIRE_BATCH_PARAM_GT_ZERO, 250], &[], &[], &[]);
        assert_eq!(invalid_param, Err(VMErrorCode::InvalidParameter));

        let owner = FIVE_VM_PROGRAM_ID;
        let account = make_account([13; 32], owner, vec![0u8; 4], false, false);
        let oob = run_require_batch(
            &[1, REQUIRE_BATCH_FIELD_NOT_BOOL, 0, 99, 0, 0, 0],
            &[account],
            &[],
            &[],
        );
        assert_eq!(oob, Err(VMErrorCode::InvalidAccountData));
    }
}
