//! Memory operations handler for MitoVM
//!
//! This module handles memory operations including STORE, LOAD, STORE_FIELD,
//! LOAD_FIELD, LOAD_INPUT, STORE_GLOBAL, and LOAD_GLOBAL. It manages zero-copy
//! account data access and input data processing.

use crate::{
    context::ExecutionManager,
    debug_log,
    error::{CompactResult, VMErrorCode},
    error_log, utils,
};
use five_protocol::{opcodes::*, ValueRef};

/// Execute zero-copy memory operations for account data and input parameter access.
/// Handles the 0x40-0x4F opcode range including STORE, LOAD, and field operations.
#[inline(never)]
pub fn handle_memory(opcode: u8, ctx: &mut ExecutionManager) -> CompactResult<()> {
    match opcode {
        STORE => {
            let account_index = ctx.fetch_byte()?;
            let field_offset = ctx.fetch_u32()?;
            let value = ctx.pop()?;

            debug_log!(
                "MitoVM: STORE account_index={}, field_offset={}",
                account_index,
                field_offset
            );

            let account = ctx.get_account_for_write(account_index)?;

            // SAFETY: The account is verified by index and no other borrows exist,
            // granting exclusive mutable access to its data.
            let account_data = unsafe { account.borrow_mut_data_unchecked() };

            if (field_offset as usize + 8) > account_data.len() {
                debug_log!(
                    "MitoVM: STORE bounds check failed: offset={} + 8 > len={}",
                    field_offset,
                    account_data.len()
                );
                return Err(VMErrorCode::InvalidAccountData);
            }

            let value_u64 = utils::resolve_u64(value, ctx)?;
            let value_bytes = value_u64.to_le_bytes();

            account_data[field_offset as usize..field_offset as usize + 8]
                .copy_from_slice(&value_bytes);

            debug_log!(
                "MitoVM: STORE completed - wrote {} to account {} field {}",
                value_u64,
                account_index,
                field_offset
            );
        }
        LOAD => {
            let _address = utils::resolve_u64(ctx.pop()?, ctx)? as usize;
            debug_log!("MitoVM: LOAD address={}", _address as u32);
            return Err(VMErrorCode::InvalidInstruction); // Not implemented in MitoVM
        }
        STORE_FIELD => {
            let account_index = ctx.fetch_byte()?;
            let field_offset = ctx.fetch_u32()?;
            let value = ctx.pop()?;

            debug_log!(
                "MitoVM: STORE_FIELD account_index={}, offset={}, num_accounts={}",
                account_index,
                field_offset,
                ctx.accounts().len() as u32
            );

            // Handle writable check with specific error code mapping for STORE_FIELD compatibility
            let account = match ctx.get_account_for_write(account_index) {
                Ok(acc) => acc,
                Err(VMErrorCode::AccountNotWritable) => {
                    error_log!(
                        "STORE_FIELD REJECTED: Account {} is READ-ONLY",
                        account_index
                    );
                    return Err(VMErrorCode::InvalidOperation);
                }
                Err(e) => return Err(e),
            };

            // SAFETY: Account is verified by index and writable, granting exclusive mutable access
            let data = unsafe { account.borrow_mut_data_unchecked() };

            store_value_into_buffer(data, field_offset as usize, value, ctx)?;
        }
        LOAD_FIELD => {
            let account_index = ctx.fetch_byte()?;
            let field_offset = ctx.fetch_u32()?;

            debug_log!(
                "MitoVM: LOAD_FIELD account_index={}, offset={}",
                account_index,
                field_offset
            );

            let account = ctx.get_account_for_read(account_index)?;

            // Optimized lazy loading: push AccountRef instead of reading immediately
            // This allows the consumer (e.g. EQ, ADD) to decide how many bytes to read
            // AccountRef takes u16 offset. Check if offset fits.
            if field_offset <= u16::MAX as u32 {
                // Eager bounds check: even if lazy, we verify the data *exists*
                if (field_offset as usize + 8) > account.data_len() {
                    debug_log!(
                        "MitoVM: LOAD_FIELD eager bounds check failed: offset={} + 8 > len={}",
                        field_offset,
                        account.data_len()
                    );
                    return Err(VMErrorCode::InvalidAccountData);
                }
                ctx.push(ValueRef::AccountRef(account_index, field_offset as u16))?;
            } else {
                // Fallback for large offsets: eager load as u64
                // SAFETY: Read-only access, no mutable references active
                let data = unsafe { account.borrow_data_unchecked() };

                if (field_offset as usize + 8) > data.len() {
                    return Err(VMErrorCode::InvalidAccountData);
                }

                let value = u64::from_le_bytes(
                    data[field_offset as usize..field_offset as usize + 8]
                        .try_into()
                        .map_err(|_| VMErrorCode::InvalidAccountData)?,
                );
                ctx.push(ValueRef::U64(value))?;
            }
        }
        LOAD_FIELD_PUBKEY => {
            let account_index = ctx.fetch_byte()?;
            let field_offset = ctx.fetch_u32()?;

            debug_log!(
                "MitoVM: LOAD_FIELD_PUBKEY account_index={}, offset={}",
                account_index,
                field_offset
            );

            let account = ctx.get_account_for_read(account_index)?;
            let data = unsafe { account.borrow_data_unchecked() };

            if (field_offset as usize + 32) > data.len() {
                debug_log!(
                    "MitoVM: LOAD_FIELD_PUBKEY Out of bounds: offset {} + 32 > len {}",
                    field_offset,
                    data.len()
                );
                return Err(VMErrorCode::InvalidAccountData);
            }

            // Use lazy loading with AccountRef for normal offsets (< 64KB)
            if field_offset <= u16::MAX as u32 {
                ctx.push(ValueRef::AccountRef(account_index, field_offset as u16))?;
                debug_log!(
                    "MitoVM: LOAD_FIELD_PUBKEY pushed lazy AccountRef({}, {})",
                    account_index,
                    field_offset as u16
                );
            } else {
                // Fallback: eager load as TempRef for very large offsets (> 64KB)
                let mut pubkey_bytes = [0u8; 32];
                pubkey_bytes
                    .copy_from_slice(&data[field_offset as usize..field_offset as usize + 32]);

                let temp_offset = ctx.alloc_temp(32)?;
                let temp_buf = ctx.get_temp_data_mut(temp_offset, 32)?;
                temp_buf.copy_from_slice(&pubkey_bytes);
                ctx.push(ValueRef::TempRef(temp_offset, 32))?;
                debug_log!(
                    "MitoVM: LOAD_FIELD_PUBKEY (large offset) pushed TempRef({}, 32)",
                    temp_offset
                );
            }
        }
        LOAD_INPUT => {
            // LOAD_INPUT: Read raw input data directly (not function parameters)
            let input_len = ctx.instruction_data().len();
            let remaining = input_len.saturating_sub(ctx.input_ptr as usize);

            debug_log!(
                "MitoVM: LOAD_INPUT - input_ptr: {}, remaining: {}",
                ctx.input_ptr as u32,
                remaining as u32
            );

            if remaining == 0 {
                debug_log!("MitoVM: LOAD_INPUT - no more input data");
                ctx.push(ValueRef::Empty)?;
                return Ok(());
            }

            // Read type ID (1 byte) from INPUT DATA
            let type_id = ctx.fetch_input_u8()?;
            debug_log!("MitoVM: LOAD_INPUT - type_id: {}", type_id);

            // Decode value based on type ID (for raw input data access)
            match type_id {
                4 => {
                    // U64
                    if (ctx.input_ptr as usize) + 8 > ctx.instruction_data().len() {
                        return Err(VMErrorCode::InvalidInstruction);
                    }
                    let value = ctx.fetch_input_u64()?;
                    ctx.push(ValueRef::U64(value))?;
                    debug_log!(
                        "MitoVM: LOAD_INPUT loaded U64: {} (stack size: {})",
                        value,
                        ctx.len() as u32
                    );
                }
                1 => {
                    // U8
                    let value = ctx.fetch_input_u8()?;
                    ctx.push(ValueRef::U8(value))?;
                    debug_log!("MitoVM: LOAD_INPUT loaded U8: {}", value);
                }
                9 => {
                    // BOOL
                    let raw_value = ctx.fetch_input_u8()?;
                    let value = raw_value != 0;
                    ctx.push(ValueRef::Bool(value))?;
                    debug_log!(
                        "MitoVM: LOAD_INPUT loaded Bool: {}",
                        if value { 1u8 } else { 0u8 }
                    );
                }
                8 => {
                    // I64
                    if (ctx.input_ptr as usize) + 8 > ctx.instruction_data().len() {
                        return Err(VMErrorCode::InvalidInstruction);
                    }
                    let value = ctx.fetch_input_u64()? as i64;
                    ctx.push(ValueRef::I64(value))?;
                    debug_log!("MitoVM: LOAD_INPUT loaded I64: {}", value);
                }
                10 => {
                    // PUBKEY
                    let start = ctx.input_ptr as usize;
                    let end = start.saturating_add(32);
                    if end > ctx.instruction_data().len() {
                        return Err(VMErrorCode::InvalidInstruction);
                    }
                    // Store current offset before advancing
                    let pubkey_offset = ctx.input_ptr as u16;
                    ctx.input_ptr = ctx.input_ptr.saturating_add(32);
                    // Push reference to pubkey data
                    ctx.push(ValueRef::PubkeyRef(pubkey_offset))?;
                    debug_log!(
                        "MitoVM: LOAD_INPUT loaded PubkeyRef at offset {}",
                        pubkey_offset
                    );
                }
                _ => {
                    debug_log!("MitoVM: LOAD_INPUT - unsupported type_id: {}", type_id);
                    return Err(VMErrorCode::InvalidInstruction);
                }
            }
        }
        STORE_GLOBAL => {
            debug_log!("MitoVM: STORE_GLOBAL operation attempted - global variables not supported");
            return Err(VMErrorCode::InvalidInstruction); // Global variable operations not supported
        }
        LOAD_GLOBAL => {
            debug_log!("MitoVM: LOAD_GLOBAL operation attempted - global variables not supported");
            return Err(VMErrorCode::InvalidInstruction); // Global variable operations not supported
        }

        LOAD_EXTERNAL_FIELD => {
            // LOAD_EXTERNAL_FIELD account_index_u8 field_offset_u32
            let account_index = ctx.fetch_byte()?;
            let field_offset = ctx.fetch_u32()? as usize;
            let resolved_account_index = ctx.resolve_bound_account_index_for_context(account_index)?;

            debug_log!(
                "MitoVM: LOAD_EXTERNAL_FIELD account_index={}, field_offset={}",
                account_index as u32,
                field_offset as u32
            );

            if resolved_account_index as usize >= ctx.accounts().len() {
                debug_log!(
                    "MitoVM: LOAD_EXTERNAL_FIELD invalid account_index {} (resolved {})",
                    account_index as u32,
                    resolved_account_index as u32
                );
                return Err(VMErrorCode::InvalidAccountIndex);
            }

            let external_account = ctx.get_account_for_read(account_index)?;
            // SAFETY: Read-only access, no mutable references active
            let account_data = unsafe { external_account.borrow_data_unchecked() };

            if (field_offset + 8) > account_data.len() {
                debug_log!(
                    "MitoVM: LOAD_EXTERNAL_FIELD field_offset {} + 8 > account data length {}",
                    field_offset as u32,
                    account_data.len() as u32
                );
                return Err(VMErrorCode::InvalidAccountData);
            }

            let field_value = u64::from_le_bytes(
                account_data[field_offset..field_offset + 8]
                    .try_into()
                    .map_err(|_| VMErrorCode::InvalidAccountData)?,
            );

            debug_log!(
                "MitoVM: LOAD_EXTERNAL_FIELD account[{} -> {}] offset {} = {}",
                account_index as u32,
                resolved_account_index as u32,
                field_offset as u32,
                field_value
            );

            ctx.push(ValueRef::U64(field_value))?;
        }

        _ => return Err(VMErrorCode::InvalidInstruction),
    }
    Ok(())
}

/// Helper function to write value into account data buffer.
#[inline(always)]
pub(crate) fn store_value_into_buffer(
    data: &mut [u8],
    offset: usize,
    value: ValueRef,
    ctx: &ExecutionManager,
) -> CompactResult<()> {
    match value {
        ValueRef::U64(v) => {
            if (offset + 8) > data.len() {
                return Err(VMErrorCode::InvalidAccountData);
            }
            data[offset..offset + 8].copy_from_slice(&v.to_le_bytes());
        }
        ValueRef::PubkeyRef(_) | ValueRef::TempRef(_, 32) => {
            // 32-byte write (Pubkey)
            if (offset + 32) > data.len() {
                return Err(VMErrorCode::InvalidAccountData);
            }
            let pubkey_bytes = ctx.extract_pubkey(&value).map_err(|e| {
                error_log!("extract_pubkey failed");
                e
            })?;
            data[offset..offset + 32].copy_from_slice(&pubkey_bytes);
        }
        ValueRef::AccountRef(account_idx, inner_offset) => {
            // Option<T> account-field encoding:
            // - AccountRef(255, _) => None  => tag=0
            // - AccountRef(0, temp) => Some(value-from-temp) => tag=1 + payload
            // Account indices are 1-based in compiler-emitted account refs, so idx=0
            // is reserved here as temp-backed Option::Some marker.
            if account_idx == 255 {
                if (offset + 1) > data.len() {
                    return Err(VMErrorCode::InvalidAccountData);
                }
                data[offset] = 0;
                return Ok(());
            }

            if account_idx == 0 {
                if (offset + 1) > data.len() {
                    return Err(VMErrorCode::InvalidAccountData);
                }
                data[offset] = 1;

                let inner = ctx
                    .read_value_from_temp(inner_offset)
                    .map_err(|_| VMErrorCode::ProtocolError)?;
                return store_value_into_buffer(data, offset + 1, inner, ctx);
            }

            // Regular AccountRef copy as u64.
            let v = utils::resolve_u64(ValueRef::AccountRef(account_idx, inner_offset), ctx)?;
            if (offset + 8) > data.len() {
                return Err(VMErrorCode::InvalidAccountData);
            }
            data[offset..offset + 8].copy_from_slice(&v.to_le_bytes());
        }
        ValueRef::StringRef(_) => {
            let (len, bytes) = ctx.extract_string_slice(&value)?;
            // Write 4-byte length prefix (u32)
            if (offset + 4 + bytes.len()) > data.len() {
                error_log!(
                    "STORE_FIELD STRING ERROR: Data too long. Offset={} Len={} AccountLen={}",
                    offset,
                    bytes.len(),
                    data.len()
                );
                return Err(VMErrorCode::InvalidAccountData);
            }
            let len_bytes = len.to_le_bytes();
            data[offset..offset + 4].copy_from_slice(&len_bytes);
            // Write string bytes
            data[offset + 4..offset + 4 + bytes.len()].copy_from_slice(bytes);

            debug_log!("STORE_FIELD STRING: offset={} len={}", offset, len);
        }
        ValueRef::TempRef(temp_offset, len) if len != 32 => {
            // Handle variable-length TempRef as string/bytes (length-prefixed)
            // This covers cases where strings are passed as raw TempRefs
            let start = temp_offset as usize;
            let end = start + (len as usize);
            let temp_buf = ctx.temp_buffer();

            if end > temp_buf.len() {
                error_log!(
                    "STORE_FIELD TEMPREF ERROR: Out of bounds start={} end={} temp_len={}",
                    start,
                    end,
                    temp_buf.len()
                );
                return Err(VMErrorCode::MemoryError);
            }

            let bytes = &temp_buf[start..end];

            // Write 4-byte length prefix (u32)
            if (offset + 4 + bytes.len()) > data.len() {
                error_log!(
                    "STORE_FIELD TEMPREF ERROR: Data too long. Offset={} Len={} AccountLen={}",
                    offset,
                    bytes.len(),
                    data.len()
                );
                return Err(VMErrorCode::InvalidAccountData);
            }
            let len_bytes = (len as u32).to_le_bytes();
            data[offset..offset + 4].copy_from_slice(&len_bytes);
            // Write bytes
            data[offset + 4..offset + 4 + bytes.len()].copy_from_slice(bytes);

            debug_log!("STORE_FIELD TEMPREF: offset={} len={}", offset, len);
        }
        ValueRef::U8(v) => {
            if (offset + 1) > data.len() {
                return Err(VMErrorCode::InvalidAccountData);
            }
            data[offset] = v;
            debug_log!("STORE_FIELD U8: offset={} val={}", offset, v);
        }
        ValueRef::Bool(v) => {
            if (offset + 1) > data.len() {
                return Err(VMErrorCode::InvalidAccountData);
            }
            data[offset] = if v { 1 } else { 0 };
            debug_log!(
                "STORE_FIELD BOOL: offset={} val={}",
                offset,
                if v { 1 } else { 0 }
            );
        }
        _ => {
            // Fallback to u64 for legacy compatibility, or fail
            if let Some(v) = value.as_u64() {
                if (offset + 8) > data.len() {
                    return Err(VMErrorCode::InvalidAccountData);
                }
                data[offset..offset + 8].copy_from_slice(&v.to_le_bytes());
                return Ok(());
            }
            return Err(VMErrorCode::TypeMismatch);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::handle_memory;
    use crate::{context::ExecutionContext, stack::StackStorage, error::VMErrorCode, MAX_PARAMETERS};
    use five_protocol::opcodes::LOAD_EXTERNAL_FIELD;
    use pinocchio::{account_info::AccountInfo, pubkey::Pubkey};

    fn create_account_info<'a>(
        key: &'a Pubkey,
        is_signer: bool,
        is_writable: bool,
        lamports: &'a mut u64,
        data: &'a mut [u8],
        owner: &'a Pubkey,
    ) -> AccountInfo {
        AccountInfo::new(key, is_signer, is_writable, lamports, data, owner, false, 0)
    }

    #[test]
    fn load_external_field_rejects_unbound_external_account_index() {
        let program_id = Pubkey::from([61u8; 32]);
        let vm_key = Pubkey::from([62u8; 32]);
        let external_key = Pubkey::from([63u8; 32]);
        let mut vm_lamports = 1u64;
        let mut external_lamports = 1u64;
        let mut vm_data = [0u8; 8];
        let mut external_data = [0u8; 8];
        external_data[..8].copy_from_slice(&7u64.to_le_bytes());

        let vm_state = create_account_info(
            &vm_key,
            false,
            true,
            &mut vm_lamports,
            &mut vm_data,
            &program_id,
        );
        let external = create_account_info(
            &external_key,
            false,
            true,
            &mut external_lamports,
            &mut external_data,
            &program_id,
        );
        let accounts = [vm_state, external];

        let mut storage = StackStorage::new();
        let mut ctx = ExecutionContext::new(
            &[1u8, 0, 0, 0, 0],
            &accounts,
            program_id,
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
        ctx.current_context = 1;
        ctx.set_external_account_remap([u8::MAX; MAX_PARAMETERS + 1]);

        let result = handle_memory(LOAD_EXTERNAL_FIELD, &mut ctx);
        assert_eq!(result, Err(VMErrorCode::InvalidAccountIndex));
    }

    #[test]
    fn load_external_field_uses_bound_external_account_mapping() {
        let program_id = Pubkey::from([64u8; 32]);
        let vm_key = Pubkey::from([65u8; 32]);
        let filler_key = Pubkey::from([66u8; 32]);
        let external_key = Pubkey::from([67u8; 32]);
        let mut vm_lamports = 1u64;
        let mut filler_lamports = 1u64;
        let mut external_lamports = 1u64;
        let mut vm_data = [0u8; 8];
        let mut filler_data = [0u8; 8];
        let mut external_data = [0u8; 8];
        external_data[..8].copy_from_slice(&11u64.to_le_bytes());

        let vm_state = create_account_info(
            &vm_key,
            false,
            true,
            &mut vm_lamports,
            &mut vm_data,
            &program_id,
        );
        let filler = create_account_info(
            &filler_key,
            false,
            true,
            &mut filler_lamports,
            &mut filler_data,
            &program_id,
        );
        let external = create_account_info(
            &external_key,
            false,
            true,
            &mut external_lamports,
            &mut external_data,
            &program_id,
        );
        let accounts = [vm_state, filler, external];

        let mut storage = StackStorage::new();
        let mut ctx = ExecutionContext::new(
            &[1u8, 0, 0, 0, 0],
            &accounts,
            program_id,
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
        ctx.current_context = 1;
        let mut remap = [u8::MAX; MAX_PARAMETERS + 1];
        remap[1] = 2;
        ctx.set_external_account_remap(remap);

        handle_memory(LOAD_EXTERNAL_FIELD, &mut ctx).expect("load external field");
        let value = ctx.pop().expect("pop loaded value");
        assert_eq!(value.as_u64(), Some(11));
    }
}
