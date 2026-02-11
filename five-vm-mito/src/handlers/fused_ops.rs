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
fn param_u64(ctx: &ExecutionManager, param_idx: u8) -> CompactResult<u64> {
    let value = ctx
        .parameters()
        .get(param_idx as usize)
        .ok_or(VMErrorCode::InvalidParameter)?;
    value.as_u64().ok_or(VMErrorCode::TypeMismatch)
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
                debug_log!("MitoVM: REQUIRE_GTE_U64 failed: {} < {}", field_value, param_value);
                return Err(VMErrorCode::ConstraintViolation);
            }
            debug_log!("MitoVM: REQUIRE_GTE_U64 passed: {} >= {}", field_value, param_value);
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
            
            debug_log!("MitoVM: FIELD_ADD_PARAM: {} + {} = {}", current_value, param_value, new_value);
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
            
            debug_log!("MitoVM: FIELD_SUB_PARAM: {} - {} = {}", current_value, param_value, new_value);
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
            let local_u64 = local_value.as_u64().ok_or(VMErrorCode::TypeMismatch)?;

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
            debug_log!("MitoVM: CHECK_SIGNER_WRITABLE passed for account {}", acc_idx);
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
                ctx
            )?;
            
            debug_log!("MitoVM: STORE_PARAM_TO_FIELD stored param {} at offset {}", param_idx, field_offset);
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
            
            data[field_offset as usize..field_offset as usize + 32]
                .copy_from_slice(&key_bytes);
            
            debug_log!("MitoVM: STORE_KEY_TO_FIELD stored key at offset {}", field_offset);
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
            let param_value = param_u64(ctx, param_idx)?;

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
            
            debug_log!("MitoVM: FIELD_SUB_ADD_PARAM transferred {} from acc{} to acc{}", param_value, acc1_idx, acc2_idx);
        }

        // REQUIRE_PARAM_LTE_IMM: param <= immediate
        // Format: REQUIRE_PARAM_LTE_IMM param_idx, imm_u8
        REQUIRE_PARAM_LTE_IMM => {
            let param_idx = ctx.fetch_byte()?;
            let imm = ctx.fetch_byte()? as u64;

            let param_value = param_u64(ctx, param_idx)?;

            if param_value > imm {
                debug_log!("MitoVM: REQUIRE_PARAM_LTE_IMM failed: {} > {}", param_value, imm);
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
                debug_log!("MitoVM: REQUIRE_FIELD_EQ_IMM failed: {} != {}", field_val, imm);
                return Err(VMErrorCode::ConstraintViolation);
            }
        }

        _ => {
            debug_log!("MitoVM: Unknown fused opcode: {}", opcode);
            return Err(VMErrorCode::InvalidInstruction);
        }
    }
    Ok(())
}
