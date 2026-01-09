//! Memory operations handler for MitoVM
//!
//! This module handles memory operations including STORE, LOAD, STORE_FIELD,
//! LOAD_FIELD, LOAD_INPUT, STORE_GLOBAL, and LOAD_GLOBAL. It manages zero-copy
//! account data access and input data processing.

use crate::{
    context::ExecutionManager,
    debug_log,
    error::{CompactResult, VMErrorCode},
    error_log,
    utils,
};
use five_protocol::{opcodes::*, ValueRef};

/// Execute zero-copy memory operations for account data and input parameter access.
/// Handles the 0x40-0x4F opcode range including STORE, LOAD, and field operations.
#[inline(never)]
pub fn handle_memory(opcode: u8, ctx: &mut ExecutionManager) -> CompactResult<()> {
    match opcode {
        STORE => {
            // MitoVM STORE: Zero-copy account field write
            // Format: STORE account_index field_offset
            let account_index = ctx.fetch_byte()?;
            let field_offset = ctx.fetch_u32()?;
            let value = ctx.pop()?;

            debug_log!(
                "MitoVM: STORE account_index={}, field_offset={}",
                account_index,
                field_offset
            );

            // Validate account index
            if (account_index as usize) >= ctx.accounts().len() {
                return Err(VMErrorCode::InvalidAccountIndex);
            }

            // Verify account ownership
            ctx.check_bytecode_authorization(account_index)?;

            let account = ctx.get_account(account_index)?;

            // Verify account is writable
            if !account.is_writable() {
                return Err(VMErrorCode::AccountNotWritable);
            }

            // Zero-copy account data write following Pinocchio patterns
            // Write directly to account data at field offset
            // SAFETY: The account is verified by index and no other borrows exist,
            // granting exclusive mutable access to its data.
            let account_data = unsafe { account.borrow_mut_data_unchecked() };

            // Validate field offset bounds
            if (field_offset as usize + 8) > account_data.len() {
                return Err(VMErrorCode::InvalidAccountData);
            }

            // Write value as u64 (8 bytes) in little-endian format
            let value_u64 = utils::resolve_u64(value, ctx)?;
            let value_bytes = value_u64.to_le_bytes();

            // Zero-copy write: direct memory copy
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
            debug_log!("DEBUG: STORE_FIELD start (Top of block)");
            // Protocol V3: STORE_FIELD account_index_u8, offset_vle
            let account_index = ctx.fetch_byte()?;
            let field_offset = ctx.fetch_vle_u32()?;
            let value = ctx.pop()?;
            debug_log!("DEBUG: Popped value (type_id={})", value.type_id());

            // Use debug_log to reduce noise
            debug_log!(
                "STORE_FIELD: acct={} offset={} num_accounts={}",
                account_index,
                field_offset,
                ctx.accounts().len() as u32
            );

            debug_log!(
                "MitoVM: STORE_FIELD account_index={}, offset={}, num_accounts={}",
                account_index,
                field_offset,
                ctx.accounts().len() as u32
            );

            // Validate account index
            if (account_index as usize) >= ctx.accounts().len() {
                debug_log!(
                    "MitoVM: STORE_FIELD ERROR - account_index {} >= num_accounts {}",
                    account_index,
                    ctx.accounts().len() as u32
                );
                return Err(VMErrorCode::InvalidAccountIndex);
            }

            // Verify account ownership
            ctx.check_bytecode_authorization(account_index)?;

            let account = ctx.get_account(account_index)?;
            debug_log!("DEBUG: Got account key");

            // CRITICAL: Check if account is writable BEFORE attempting to write
            // pinocchio's borrow_mut_data_unchecked() does NOT check is_writable!
            if !account.is_writable() {
                error_log!(
                    "STORE_FIELD REJECTED: Account {} is READ-ONLY but bytecode tried to write to it",
                    account_index
                );
                return Err(VMErrorCode::InvalidOperation);
            }

            // CRITICAL DEBUG: Log pointer to verify we are writing to the correct location
            let data_ptr = unsafe { account.borrow_data_unchecked().as_ptr() as usize };
            debug_log!(
                "STORE_FIELD_PTR: idx={} ptr={} offset={}",
                account_index,
                data_ptr,
                field_offset
            );

            // Log account key (first 4 bytes for brevity)
            let _key_bytes = account.key();
            debug_log!(
                "MitoVM: STORE_FIELD target account key: {} {} {} {}",
                _key_bytes[0],
                _key_bytes[1],
                _key_bytes[2],
                _key_bytes[3]
            );

            // SAFETY: Account is verified by index and writable, granting exclusive mutable access

            let data = unsafe { account.borrow_mut_data_unchecked() };

            debug_log!(
                "MitoVM: STORE_FIELD data_len={}, is_writable={}",
                data.len() as u32,
                if account.is_writable() { 1u8 } else { 0u8 }
            );

            // Determine value size and bytes to write
            match value {
                ValueRef::U64(v) => {
                    if (field_offset as usize + 8) > data.len() {
                        return Err(VMErrorCode::InvalidAccountData);
                    }
                    data[field_offset as usize..field_offset as usize + 8].copy_from_slice(&v.to_le_bytes());
                }
                ValueRef::PubkeyRef(_) | ValueRef::TempRef(_, 32) => {
                    // 32-byte write (Pubkey)
                    if (field_offset as usize + 32) > data.len() {
                        return Err(VMErrorCode::InvalidAccountData);
                    }
                    let pubkey_bytes = ctx.extract_pubkey(&value).map_err(|e| {
                        error_log!("DEBUG: extract_pubkey failed");
                        e
                    })?;
                    debug_log!("DEBUG: Extracted pubkey");
                    data[field_offset as usize..field_offset as usize + 32].copy_from_slice(&pubkey_bytes);
                }
                ValueRef::AccountRef(_, _) => {
                    // AccountRef copy (u64)
                    let v = utils::resolve_u64(value, ctx)?;
                    if (field_offset as usize + 8) > data.len() {
                        return Err(VMErrorCode::InvalidAccountData);
                    }
                    data[field_offset as usize..field_offset as usize + 8].copy_from_slice(&v.to_le_bytes());
                }
                ValueRef::StringRef(_) => {
                    let (len, bytes) = ctx.extract_string_slice(&value)?;
                    // Write 4-byte length prefix (u32)
                    if (field_offset as usize + 4 + bytes.len()) > data.len() {
                        error_log!("STORE_FIELD STRING ERROR: Data too long. Offset={} Len={} AccountLen={}", field_offset, bytes.len(), data.len());
                        return Err(VMErrorCode::InvalidAccountData);
                    }
                    let len_bytes = len.to_le_bytes();
                    data[field_offset as usize..field_offset as usize + 4].copy_from_slice(&len_bytes);
                    // Write string bytes
                    data[field_offset as usize + 4..field_offset as usize + 4 + bytes.len()].copy_from_slice(bytes);
                    
                    debug_log!(
                        "STORE_FIELD STRING: idx={} offset={} len={}", 
                        account_index, field_offset, len
                    );
                }
                _ => {
                    // Fallback to u64 for legacy compatibility, or fail
                    if let Some(v) = value.as_u64() {
                        if (field_offset as usize + 8) > data.len() {
                            return Err(VMErrorCode::InvalidAccountData);
                        }
                        data[field_offset as usize..field_offset as usize + 8].copy_from_slice(&v.to_le_bytes());
                    } else {
                        return Err(VMErrorCode::TypeMismatch);
                    }
                }
            }

            // CRITICAL: Log to error_log to ensure persistence verification is visible
            debug_log!(
                "STORE_FIELD_WRITTEN: idx={} offset={} ptr={}",
                account_index,
                field_offset,
                data_ptr
            );
        }
        LOAD_FIELD => {
            // Protocol V3: LOAD_FIELD account_index_u8, offset_vle
            let account_index = ctx.fetch_byte()?;
            let field_offset = ctx.fetch_vle_u32()?;

            debug_log!(
                "MitoVM: LOAD_FIELD account_index={}, offset={}",
                account_index,
                field_offset
            );

            // Validate account index
            if (account_index as usize) >= ctx.accounts().len() {
                return Err(VMErrorCode::InvalidAccountIndex);
            }

            // Optimized lazy loading: push AccountRef instead of reading immediately
            // This allows the consumer (e.g. EQ, ADD) to decide how many bytes to read
            // AccountRef takes u16 offset. Check if offset fits.
            if field_offset <= u16::MAX as u32 {
                // Eager bounds check: even if lazy, we verify the data *exists*
                // This preserves the "Fail Fast" property and matches legacy behavior
                let account = &ctx.accounts()[account_index as usize];
                if (field_offset as usize + 8) > account.data_len() {
                    return Err(VMErrorCode::InvalidAccountData);
                }
                ctx.push(ValueRef::AccountRef(account_index, field_offset as u16))?;
            } else {
                // Fallback for large offsets: eager load as u64 (legacy behavior)
                // This limitation implies fields > 64KB must be u64 or handled differently
                let account = &ctx.accounts()[account_index as usize];
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
            // Protocol V3: LOAD_FIELD_PUBKEY account_index_u8, offset_vle -> AccountRef/TempRef
            // Loads a 32-byte pubkey field from an account using lazy loading (AccountRef) when possible
            // This enables zero-copy field access for pubkey comparisons and operations
            let account_index = ctx.fetch_byte()?;
            let field_offset = ctx.fetch_vle_u32()?;

            debug_log!(
                "MitoVM: LOAD_FIELD_PUBKEY account_index={}, offset={}",
                account_index,
                field_offset
            );

            // Validate account index
            if (account_index as usize) >= ctx.accounts().len() {
                return Err(VMErrorCode::InvalidAccountIndex);
            }

            let account = &ctx.accounts()[account_index as usize];
            let data = unsafe { account.borrow_data_unchecked() };

            // Validate that 32 bytes are available at the field offset
            if (field_offset as usize + 32) > data.len() {
                debug_log!("MitoVM: LOAD_FIELD_PUBKEY Out of bounds: offset {} + 32 > len {}", field_offset, data.len());
                return Err(VMErrorCode::InvalidAccountData);
            }

            // Use lazy loading with AccountRef for normal offsets (< 64KB)
            // This defers the 32-byte read until an operation like EQ actually needs the data
            // The EQ handler knows how to extract 32 bytes from AccountRef for pubkey comparison
            if field_offset <= u16::MAX as u32 {
                ctx.push(ValueRef::AccountRef(account_index, field_offset as u16))?;
                debug_log!(
                    "MitoVM: LOAD_FIELD_PUBKEY pushed lazy AccountRef({}, {})",
                    account_index,
                    field_offset as u16
                );
            } else {
                // Fallback: eager load as TempRef for very large offsets (> 64KB)
                // This preserves correctness for programs with large field offsets
                let mut pubkey_bytes = [0u8; 32];
                pubkey_bytes.copy_from_slice(&data[field_offset as usize..field_offset as usize + 32]);

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
                    if (ctx.input_ptr as usize) + 32 > ctx.instruction_data().len() {
                        return Err(VMErrorCode::InvalidInstruction);
                    }
                    // Store current offset before advancing
                    let pubkey_offset = ctx.input_ptr as u16;
                    // Skip pubkey bytes to advance input pointer
                    for _ in 0..32 {
                        ctx.fetch_input_u8()?;
                    }
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
            // KISS: Account index and field offset resolved at compile time
            let account_index = ctx.fetch_byte()? as usize;
            let field_offset = ctx.fetch_u32()? as usize;

            debug_log!(
                "MitoVM: LOAD_EXTERNAL_FIELD account_index={}, field_offset={}",
                account_index as u32,
                field_offset as u32
            );

            // Validate account index
            if account_index >= ctx.accounts().len() {
                debug_log!(
                    "MitoVM: LOAD_EXTERNAL_FIELD invalid account_index {}",
                    account_index as u32
                );
                return Err(VMErrorCode::InvalidAccountIndex);
            }

            let external_account = &ctx.accounts()[account_index];
            // SAFETY: We only need a read-only slice and no mutable references are active
            let account_data = unsafe { external_account.borrow_data_unchecked() };

            // Bounds check for zero-copy field access (KISS - no string lookups)
            if (field_offset + 8) > account_data.len() {
                debug_log!(
                    "MitoVM: LOAD_EXTERNAL_FIELD field_offset {} + 8 > account data length {}",
                    field_offset as u32,
                    account_data.len() as u32
                );
                return Err(VMErrorCode::InvalidAccountData);
            }

            // Zero-copy field value read (8 bytes, little-endian)
            let field_value = u64::from_le_bytes(
                account_data[field_offset..field_offset + 8]
                    .try_into()
                    .map_err(|_| VMErrorCode::InvalidAccountData)?,
            );

            debug_log!(
                "MitoVM: LOAD_EXTERNAL_FIELD account[{}] offset {} = {}",
                account_index as u32,
                field_offset as u32,
                field_value
            );

            // Push field value to stack (zero-allocation)
            ctx.push(ValueRef::U64(field_value))?;
        }

        _ => return Err(VMErrorCode::InvalidInstruction),
    }
    Ok(())
}
