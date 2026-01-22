//! Universal Fused Opcodes Handler (0xC0-0xCF)
//!
//! High-impact universal patterns that apply across all DeFi contracts.
//! Each fused opcode replaces 3-4 individual opcodes, saving ~300 CU per use.

use crate::{
    context::ExecutionManager,
    debug_log,
    error::{CompactResult, VMErrorCode},
};
use five_protocol::opcodes::*;

/// Handle universal fused operations (0xC0-0xCF)
#[inline(never)]
pub fn handle_fused_ops(opcode: u8, ctx: &mut ExecutionManager) -> CompactResult<()> {
    match opcode {
        // REQUIRE_GTE_U64: LOAD_FIELD + LOAD_PARAM + GTE + REQUIRE fused
        // Encoding: acc(u8) offset(VLE) param(u8)
        // Saves 300 CU by avoiding 4 opcode dispatches
        REQUIRE_GTE_U64 => {
            let acc_idx = ctx.fetch_byte()?;
            let field_offset = ctx.fetch_vle_u16()?;
            let param_idx = ctx.fetch_byte()?;

            // Load field value directly from account data
            let account = ctx.get_account_for_read(acc_idx)?;
            let data = unsafe { account.borrow_data_unchecked() };
            
            if (field_offset as usize) + 8 > data.len() {
                debug_log!("MitoVM: REQUIRE_GTE_U64 field offset out of bounds");
                return Err(VMErrorCode::InvalidAccountData);
            }
            
            let field_bytes: [u8; 8] = data[field_offset as usize..field_offset as usize + 8]
                .try_into()
                .map_err(|_| VMErrorCode::InvalidAccountData)?;
            let field_value = u64::from_le_bytes(field_bytes);

            // Load param value using same pattern as locals.rs
            let param_value = ctx.parameters()[param_idx as usize]
                .as_u64()
                .ok_or(VMErrorCode::TypeMismatch)?;

            // GTE + REQUIRE fused
            if field_value < param_value {
                debug_log!("MitoVM: REQUIRE_GTE_U64 failed: {} < {}", field_value, param_value);
                return Err(VMErrorCode::ConstraintViolation);
            }
            debug_log!("MitoVM: REQUIRE_GTE_U64 passed: {} >= {}", field_value, param_value);
        }

        // REQUIRE_NOT_BOOL: LOAD_FIELD + NOT + REQUIRE fused
        // Encoding: acc(u8) offset(VLE)
        // Saves 200 CU
        REQUIRE_NOT_BOOL => {
            let acc_idx = ctx.fetch_byte()?;
            let field_offset = ctx.fetch_vle_u16()?;

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
        // Encoding: acc(u8) offset(VLE) param(u8)
        // Saves 300 CU
        FIELD_ADD_PARAM => {
            let acc_idx = ctx.fetch_byte()?;
            let field_offset = ctx.fetch_vle_u16()?;
            let param_idx = ctx.fetch_byte()?;

            // Load param value first using same pattern as locals.rs
            let param_value = ctx.parameters()[param_idx as usize]
                .as_u64()
                .ok_or(VMErrorCode::TypeMismatch)?;

            // Get account for write
            let account = ctx.get_account_for_write(acc_idx)?;
            let data = unsafe { account.borrow_mut_data_unchecked() };
            
            if (field_offset as usize) + 8 > data.len() {
                debug_log!("MitoVM: FIELD_ADD_PARAM field offset out of bounds");
                return Err(VMErrorCode::InvalidAccountData);
            }
            
            // Read current value
            let field_bytes: [u8; 8] = data[field_offset as usize..field_offset as usize + 8]
                .try_into()
                .map_err(|_| VMErrorCode::InvalidAccountData)?;
            let current_value = u64::from_le_bytes(field_bytes);

            // Add and store
            let new_value = current_value.wrapping_add(param_value);
            data[field_offset as usize..field_offset as usize + 8]
                .copy_from_slice(&new_value.to_le_bytes());
            
            debug_log!("MitoVM: FIELD_ADD_PARAM: {} + {} = {}", current_value, param_value, new_value);
        }

        // FIELD_SUB_PARAM: LOAD_FIELD + LOAD_PARAM + SUB + STORE_FIELD fused
        // Encoding: acc(u8) offset(VLE) param(u8)
        // Saves 300 CU
        FIELD_SUB_PARAM => {
            let acc_idx = ctx.fetch_byte()?;
            let field_offset = ctx.fetch_vle_u16()?;
            let param_idx = ctx.fetch_byte()?;

            // Load param value first using same pattern as locals.rs
            let param_value = ctx.parameters()[param_idx as usize]
                .as_u64()
                .ok_or(VMErrorCode::TypeMismatch)?;

            // Get account for write
            let account = ctx.get_account_for_write(acc_idx)?;
            let data = unsafe { account.borrow_mut_data_unchecked() };
            
            if (field_offset as usize) + 8 > data.len() {
                debug_log!("MitoVM: FIELD_SUB_PARAM field offset out of bounds");
                return Err(VMErrorCode::InvalidAccountData);
            }
            
            // Read current value
            let field_bytes: [u8; 8] = data[field_offset as usize..field_offset as usize + 8]
                .try_into()
                .map_err(|_| VMErrorCode::InvalidAccountData)?;
            let current_value = u64::from_le_bytes(field_bytes);

            // Sub and store
            let new_value = current_value.wrapping_sub(param_value);
            data[field_offset as usize..field_offset as usize + 8]
                .copy_from_slice(&new_value.to_le_bytes());
            
            debug_log!("MitoVM: FIELD_SUB_PARAM: {} - {} = {}", current_value, param_value, new_value);
        }

        // REQUIRE_PARAM_GT_ZERO: LOAD_PARAM + PUSH_0 + GT + REQUIRE fused
        // Encoding: param(u8)
        // Saves 300 CU
        REQUIRE_PARAM_GT_ZERO => {
            let param_idx = ctx.fetch_byte()?;

            let param_value = ctx.parameters()[param_idx as usize]
                .as_u64()
                .ok_or(VMErrorCode::TypeMismatch)?;

            if param_value == 0 {
                debug_log!("MitoVM: REQUIRE_PARAM_GT_ZERO failed: param is 0");
                return Err(VMErrorCode::ConstraintViolation);
            }
            debug_log!("MitoVM: REQUIRE_PARAM_GT_ZERO passed: {} > 0", param_value);
        }

        // REQUIRE_EQ_PUBKEY: Compare two pubkey fields from accounts
        // Encoding: acc1(u8) offset1(VLE) acc2(u8) offset2(VLE)
        // Saves 300 CU
        REQUIRE_EQ_PUBKEY => {
            let acc1_idx = ctx.fetch_byte()?;
            let offset1 = ctx.fetch_vle_u32()?; // Use u32 for large offsets and sentinel
            let acc2_idx = ctx.fetch_byte()?;
            let offset2 = ctx.fetch_vle_u32()?; // Use u32 for large offsets and sentinel

            // Load first pubkey
            let account1 = ctx.get_account_for_read(acc1_idx)?;
            let pubkey1_ref: &[u8] = if offset1 == 0x3FFF {
                // Sentinel: Use Account Key (0x3FFF = 2-byte VLE sentinel)
                account1.key().as_ref()
            } else {
                // Use Data Field
                let data1 = unsafe { account1.borrow_data_unchecked() };
                if (offset1 as usize) + 32 > data1.len() {
                    debug_log!(
                        "MitoVM: REQUIRE_EQ_PUBKEY acc1 bounds check failed: offset={} + 32 > len={}", 
                        offset1, 
                        data1.len()
                    );
                    return Err(VMErrorCode::InvalidAccountData);
                }
                &data1[offset1 as usize..offset1 as usize + 32]
            };

            // Load second pubkey
            let account2 = ctx.get_account_for_read(acc2_idx)?;
            let pubkey2_ref: &[u8] = if offset2 == 0x3FFF {
                // Sentinel: Use Account Key (0x3FFF = 2-byte VLE sentinel)
                account2.key().as_ref()
            } else {
                // Use Data Field
                let data2 = unsafe { account2.borrow_data_unchecked() };
                if (offset2 as usize) + 32 > data2.len() {
                    debug_log!(
                        "MitoVM: REQUIRE_EQ_PUBKEY acc2 bounds check failed: offset={} + 32 > len={}", 
                        offset2, 
                        data2.len()
                    );
                    return Err(VMErrorCode::InvalidAccountData);
                }
                &data2[offset2 as usize..offset2 as usize + 32]
            };

            // Compare
            if pubkey1_ref != pubkey2_ref {
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
        // Encoding: acc(u8) offset(VLE) param(u8)
        // Saves 100 CU
        STORE_PARAM_TO_FIELD => {
            let acc_idx = ctx.fetch_byte()?;
            let field_offset = ctx.fetch_vle_u16()?;
            let param_idx = ctx.fetch_byte()?;

            // Load param value generically
            // Clone the ValueRef (cheap, just references)
            let param_value = ctx.parameters()[param_idx as usize].clone();

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
        // Encoding: acc(u8) offset(VLE)
        // Saves 100 CU
        STORE_FIELD_ZERO => {
            let acc_idx = ctx.fetch_byte()?;
            let field_offset = ctx.fetch_vle_u16()?;

            let account = ctx.get_account_for_write(acc_idx)?;
            let data = unsafe { account.borrow_mut_data_unchecked() };
            
            if (field_offset as usize) + 8 > data.len() {
                debug_log!("MitoVM: STORE_FIELD_ZERO offset out of bounds");
                return Err(VMErrorCode::InvalidAccountData);
            }
            
            // Store zero (8 bytes for u64)
            data[field_offset as usize..field_offset as usize + 8]
                .copy_from_slice(&0u64.to_le_bytes());
            
            debug_log!("MitoVM: STORE_FIELD_ZERO at offset {}", field_offset);
        }

        // STORE_KEY_TO_FIELD: GET_KEY + STORE_FIELD fused
        // Encoding: acc(u8) offset(VLE) key_acc(u8)
        // Saves 100 CU
        STORE_KEY_TO_FIELD => {
            let acc_idx = ctx.fetch_byte()?;
            let field_offset = ctx.fetch_vle_u16()?;
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
        // Encoding: acc1(u8) offset1(VLE) acc2(u8) offset2(VLE)
        // Saves 300 CU
        REQUIRE_EQ_FIELDS => {
            let acc1_idx = ctx.fetch_byte()?;
            let offset1 = ctx.fetch_vle_u16()?;
            let acc2_idx = ctx.fetch_byte()?;
            let offset2 = ctx.fetch_vle_u16()?;

            // Load first field (32 bytes for pubkey comparison)
            let account1 = ctx.get_account_for_read(acc1_idx)?;
            let data1 = unsafe { account1.borrow_data_unchecked() };
            if (offset1 as usize) + 32 > data1.len() {
                return Err(VMErrorCode::InvalidAccountData);
            }
            let field1 = &data1[offset1 as usize..offset1 as usize + 32];

            // Load second field
            let account2 = ctx.get_account_for_read(acc2_idx)?;
            let data2 = unsafe { account2.borrow_data_unchecked() };
            if (offset2 as usize) + 32 > data2.len() {
                return Err(VMErrorCode::InvalidAccountData);
            }
            let field2 = &data2[offset2 as usize..offset2 as usize + 32];

            // Compare
            if field1 != field2 {
                debug_log!("MitoVM: REQUIRE_EQ_FIELDS failed: fields don't match");
                return Err(VMErrorCode::ConstraintViolation);
            }
            debug_log!("MitoVM: REQUIRE_EQ_FIELDS passed");
        }

        _ => {
            debug_log!("MitoVM: Unknown fused opcode: {}", opcode);
            return Err(VMErrorCode::InvalidInstruction);
        }
    }
    Ok(())
}

