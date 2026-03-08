//! Cryptography syscall handlers

use crate::{
    context::ExecutionManager,
    debug_log,
    error::{CompactResult, VMErrorCode},
};
use five_protocol::ValueRef;
use pinocchio::sysvars::instructions::Instructions as InstructionSysvar;
#[cfg(target_os = "solana")]
use pinocchio::log::sol_log_64;

#[cfg(target_os = "solana")]
use pinocchio::syscalls;

// Base58: Ed25519SigVerify111111111111111111111111111
const ED25519_PROGRAM_ID_BYTES: [u8; 32] = [
    0x03, 0x7d, 0x46, 0xd6, 0x7c, 0x93, 0xfb, 0xbe, 0x12, 0xf9, 0x42, 0x8f, 0x83, 0x8d, 0x40, 0xff,
    0x05, 0x70, 0x74, 0x49, 0x27, 0xf4, 0x8a, 0x64, 0xfc, 0xca, 0x70, 0x44, 0x80, 0x00, 0x00, 0x00,
];

/// Helper to parse data array (vals) for hash functions
/// Returns pointer to array of slices and count
fn parse_data_array(
    ctx: &mut ExecutionManager,
    data_ref: ValueRef,
) -> CompactResult<(*const u8, u64)> {
    // Similar to log_data, but needs to return pointer that syscall consumes.
    // Syscall expects `*const u8` which points to `&[u8]` array (iovec style).
    // This requires us to construct the array of slices in contiguous memory.

    // Only supports single data slice (ValueRef::StringRef/TempRef).
    // or we need to construct the iovec array in temp buffer.

    // If it's a simple byte buffer (StringRef/TempRef), we treat it as 1 element array.
    // We need to alloc a slot in temp buffer for the slice descriptor { ptr, len }.
    // slice descriptor in Rust is [usize; 2].

    // Constructing iovec for syscalls is tricky because we need pointers to memory.
    // Pinocchio `hash.rs` helper handles this.
    // Here we must do it manually.

    let (len, ptr) = match data_ref {
        ValueRef::StringRef(_) | ValueRef::TempRef(_, _) | ValueRef::HeapString(_) => {
            let (l, b) = ctx.extract_string_slice(&data_ref)?;
            (l as u64, b.as_ptr())
        }
        ValueRef::ArrayRef(id) => {
            // Treat array as single data slice
            let temp_buf = ctx.temp_buffer();
            let start = id as usize;

            // Validate header access
            if start + 2 > temp_buf.len() {
                return Err(VMErrorCode::MemoryViolation);
            }

            let array_len = temp_buf[start] as u64;

            // Validate data access
            if start + 2 + (array_len as usize) > temp_buf.len() {
                return Err(VMErrorCode::MemoryViolation);
            }

            let ptr = unsafe { temp_buf.as_ptr().add(start + 2) };
            (array_len, ptr)
        }
        _ => return Err(VMErrorCode::TypeMismatch),
    };

    // We need an array of `IoVec { base: *const u8, len: u64 }` ?
    // Solana syscalls take `vals: *const u8` where vals is array of `&[u8]`.
    // In Rust `&[u8]` is (ptr, len).
    // So we need to write `ptr` (u64/usize) and `len` (u64/usize) to temp buffer.

    // Allocate temp space for 1 slice (16 bytes on 64-bit, but Solana VM is 64-bit).
    // `&[u8]` layout: pointer (8 bytes), length (8 bytes).

    let vec_offset = ctx.alloc_temp(16)?;
    let ptr_u64 = ptr as u64;
    let len_u64 = len;

    // Write to temp buffer safely (handling potential misalignment)
    let temp_buf = ctx.temp_buffer_mut();
    let vec_slice = &mut temp_buf[vec_offset as usize..vec_offset as usize + 16];
    vec_slice[0..8].copy_from_slice(&ptr_u64.to_le_bytes());
    vec_slice[8..16].copy_from_slice(&len_u64.to_le_bytes());

    let vec_ptr = unsafe { temp_buf.as_ptr().add(vec_offset as usize) };
    Ok((vec_ptr, 1))
}

#[inline(always)]
fn resolve_hash_result_ptr(
    ctx: &mut ExecutionManager,
    result_ref: ValueRef,
) -> CompactResult<*mut u8> {
    match result_ref {
        ValueRef::TempRef(offset, len) => {
            if len < 32 {
                return Err(VMErrorCode::MemoryViolation);
            }
            Ok(unsafe { ctx.temp_buffer_mut().as_mut_ptr().add(offset as usize) })
        }
        ValueRef::ArrayRef(id) => {
            let temp_buf = ctx.temp_buffer();
            let start = id as usize;
            if start + 2 > temp_buf.len() {
                return Err(VMErrorCode::MemoryViolation);
            }
            let len = temp_buf[start];
            if len < 32 || start + 2 + 32 > temp_buf.len() {
                return Err(VMErrorCode::MemoryViolation);
            }
            Ok(unsafe { ctx.temp_buffer_mut().as_mut_ptr().add(start + 2) })
        }
        ValueRef::StringRef(offset) => {
            let start = offset as usize;
            let temp_buf = ctx.temp_buffer();
            if start + 2 > temp_buf.len() {
                return Err(VMErrorCode::MemoryViolation);
            }
            let len = temp_buf[start];
            if len < 32 || start + 2 + 32 > temp_buf.len() {
                return Err(VMErrorCode::MemoryViolation);
            }
            Ok(unsafe { ctx.temp_buffer_mut().as_mut_ptr().add(start + 2) })
        }
        ValueRef::HeapString(id) => {
            let len_bytes = ctx.get_heap_data(id, 4)?;
            let len = u32::from_le_bytes(len_bytes.try_into().unwrap());
            if len < 32 {
                return Err(VMErrorCode::MemoryViolation);
            }
            Ok(ctx.get_heap_data_mut(id + 4, len)?.as_mut_ptr())
        }
        _ => Err(VMErrorCode::TypeMismatch),
    }
}

#[inline(always)]
fn copy_hash_input_bytes(
    ctx: &mut ExecutionManager,
    data_ref: &ValueRef,
    out: &mut [u8],
) -> CompactResult<usize> {
    match data_ref {
        ValueRef::StringRef(_) | ValueRef::HeapString(_) => {
            let (len, bytes) = ctx.extract_string_slice(data_ref)?;
            let len = len as usize;
            if len > out.len() {
                return Err(VMErrorCode::OutOfMemory);
            }
            out[..len].copy_from_slice(bytes);
            Ok(len)
        }
        ValueRef::TempRef(offset, len) => {
            let start = *offset as usize;
            let len = *len as usize;
            let end = start.saturating_add(len);
            if end > ctx.temp_buffer().len() || len > out.len() {
                return Err(VMErrorCode::MemoryViolation);
            }
            out[..len].copy_from_slice(&ctx.temp_buffer()[start..end]);
            Ok(len)
        }
        ValueRef::ArrayRef(id) => {
            let start = *id as usize;
            let temp = ctx.temp_buffer();
            if start + 2 > temp.len() {
                return Err(VMErrorCode::MemoryViolation);
            }

            let len = temp[start] as usize;
            let element_type = temp[start + 1];
            if len > out.len() {
                return Err(VMErrorCode::OutOfMemory);
            }

            if element_type == 0 {
                let data_start = start + 2;
                let data_end = data_start.saturating_add(len);
                if data_end > temp.len() {
                    return Err(VMErrorCode::MemoryViolation);
                }
                out[..len].copy_from_slice(&temp[data_start..data_end]);
                return Ok(len);
            }

            let mut cursor = start + 2;
            for i in 0..len {
                if cursor >= temp.len() {
                    return Err(VMErrorCode::MemoryViolation);
                }

                match ValueRef::deserialize_from(&temp[cursor..]) {
                    Ok(v) => {
                        out[i] = match v {
                            ValueRef::U8(n) => n,
                            ValueRef::U64(n) if n <= u8::MAX as u64 => n as u8,
                            ValueRef::I64(n) if (0..=u8::MAX as i64).contains(&n) => n as u8,
                            _ => return Err(VMErrorCode::TypeMismatch),
                        };
                        cursor += v.serialized_size();
                    }
                    Err(_) => {
                        let end = cursor.saturating_add(len);
                        if end > temp.len() {
                            return Err(VMErrorCode::MemoryViolation);
                        }
                        out[..len].copy_from_slice(&temp[cursor..end]);
                        return Ok(len);
                    }
                }
            }

            Ok(len)
        }
        ValueRef::U64(0) => Ok(0),
        _ => Err(VMErrorCode::TypeMismatch),
    }
}

#[inline(always)]
pub fn handle_syscall_sha256(ctx: &mut ExecutionManager) -> CompactResult<()> {
    debug_log!("MitoVM: SYSCALL_SHA256");

    let result_ref = ctx.pop()?;
    let data_ref = ctx.pop()?;
    let result_ptr = resolve_hash_result_ptr(ctx, result_ref)?;

    #[cfg(target_os = "solana")]
    unsafe {
        let (vals_ptr, val_len) = parse_data_array(ctx, data_ref)?;
        syscalls::sol_sha256(vals_ptr, val_len, result_ptr);
    }

    #[cfg(not(target_os = "solana"))]
    {
        use solana_nostd_sha256::hashv;

        let mut data_buf = [0u8; 1024];
        let data_len = copy_hash_input_bytes(ctx, &data_ref, &mut data_buf)?;
        let hash = hashv(&[&data_buf[..data_len]]);
        unsafe {
            core::ptr::copy_nonoverlapping(hash.as_ptr(), result_ptr, 32);
        }
    }

    Ok(())
}

#[inline(always)]
pub fn handle_syscall_keccak256(ctx: &mut ExecutionManager) -> CompactResult<()> {
    debug_log!("MitoVM: SYSCALL_KECCAK256");

    let result_ref = ctx.pop()?;
    let data_ref = ctx.pop()?;
    let result_ptr = resolve_hash_result_ptr(ctx, result_ref)?;

    #[cfg(target_os = "solana")]
    unsafe {
        let (vals_ptr, val_len) = parse_data_array(ctx, data_ref)?;
        syscalls::sol_keccak256(vals_ptr, val_len, result_ptr);
    }

    #[cfg(not(target_os = "solana"))]
    {
        use tiny_keccak::{Hasher, Keccak};

        let mut data_buf = [0u8; 1024];
        let data_len = copy_hash_input_bytes(ctx, &data_ref, &mut data_buf)?;
        let mut out = [0u8; 32];
        let mut keccak = Keccak::v256();
        keccak.update(&data_buf[..data_len]);
        keccak.finalize(&mut out);
        unsafe {
            core::ptr::copy_nonoverlapping(out.as_ptr(), result_ptr, 32);
        }
    }

    Ok(())
}

#[inline(always)]
pub fn handle_syscall_blake3(ctx: &mut ExecutionManager) -> CompactResult<()> {
    debug_log!("MitoVM: SYSCALL_BLAKE3");

    #[cfg(target_os = "solana")]
    {
        // Mainnet runtime does not expose sol_blake3. Keep opcode defined but fail gracefully.
        let _ = ctx.pop()?;
        let _ = ctx.pop()?;
        return Err(VMErrorCode::InvalidOperation);
    }

    #[cfg(not(target_os = "solana"))]
    {
        let result_ref = ctx.pop()?;
        let data_ref = ctx.pop()?;
        let result_ptr = resolve_hash_result_ptr(ctx, result_ref)?;
        let mut data_buf = [0u8; 1024];
        let data_len = copy_hash_input_bytes(ctx, &data_ref, &mut data_buf)?;
        let out = blake3::hash(&data_buf[..data_len]);
        unsafe {
            core::ptr::copy_nonoverlapping(out.as_bytes().as_ptr(), result_ptr, 32);
        }
    }

    Ok(())
}

// Poseidon has extra args
#[inline(always)]
pub fn handle_syscall_poseidon(ctx: &mut ExecutionManager) -> CompactResult<()> {
    debug_log!("MitoVM: SYSCALL_POSEIDON");
    // poseidon(parameters, endianness, vals, val_len, hash_result)
    // Stack: result, vals, endianness, parameters. (Pushed in order parameters, endianness, vals, result)

    let result_ref = ctx.pop()?;
    let vals_ref = ctx.pop()?;
    let endianness = ctx.pop()?.as_u64().ok_or(VMErrorCode::TypeMismatch)?;
    let parameters = ctx.pop()?.as_u64().ok_or(VMErrorCode::TypeMismatch)?;

    let result_ptr = match result_ref {
        ValueRef::TempRef(offset, _) => unsafe {
            ctx.temp_buffer_mut().as_mut_ptr().add(offset as usize)
        },
        ValueRef::ArrayRef(id) => {
            let temp_buf = ctx.temp_buffer();
            let start = id as usize;
            if start + 2 > temp_buf.len() {
                return Err(VMErrorCode::MemoryViolation);
            }
            let len = temp_buf[start];
            if len < 32 {
                return Err(VMErrorCode::MemoryViolation);
            }
            // Ensure buffer has enough space for 32 bytes (Poseidon result size)
            if start + 2 + 32 > temp_buf.len() {
                return Err(VMErrorCode::MemoryViolation);
            }
            unsafe { ctx.temp_buffer_mut().as_mut_ptr().add(start + 2) }
        }
        ValueRef::StringRef(offset) => {
            let start = offset as usize;
            let temp_buf = ctx.temp_buffer();
            if start + 2 > temp_buf.len() {
                return Err(VMErrorCode::MemoryViolation);
            }
            let len = temp_buf[start];
            if len < 32 {
                return Err(VMErrorCode::MemoryViolation);
            }
            // Ensure buffer has enough space for 32 bytes
            if start + 2 + 32 > temp_buf.len() {
                return Err(VMErrorCode::MemoryViolation);
            }
            unsafe { ctx.temp_buffer_mut().as_mut_ptr().add(start + 2) }
        }
        ValueRef::HeapString(id) => {
            // We assume sufficient size validation happened at allocation or call site logic
            // But for safety we should get the ptr safely
            let len_bytes = ctx.get_heap_data(id, 4)?;
            let len = u32::from_le_bytes(len_bytes.try_into().unwrap());
            if len < 32 {
                return Err(VMErrorCode::MemoryViolation);
            }
            ctx.get_heap_data_mut(id + 4, len)?.as_mut_ptr()
        }
        _ => return Err(VMErrorCode::TypeMismatch),
    };

    let (vals_ptr, val_len) = parse_data_array(ctx, vals_ref)?;

    #[cfg(target_os = "solana")]
    unsafe {
        syscalls::sol_poseidon(parameters, endianness, vals_ptr, val_len, result_ptr);
    }

    #[cfg(not(target_os = "solana"))]
    {
        let _ = (endianness, parameters, vals_ptr, val_len, result_ptr);
    }

    Ok(())
}

// Secp256k1 recover
#[inline(always)]
pub fn handle_syscall_secp256k1_recover(ctx: &mut ExecutionManager) -> CompactResult<()> {
    debug_log!("MitoVM: SYSCALL_SECP256K1_RECOVER");
    // secp256k1_recover(hash, recovery_id, signature, result)
    // Stack: result, signature, recovery_id, hash.

    let result_ref = ctx.pop()?;
    let signature_ref = ctx.pop()?;
    let recovery_id = ctx.pop()?.as_u64().ok_or(VMErrorCode::TypeMismatch)?;
    let hash_ref = ctx.pop()?;

    let result_ptr = match result_ref {
        ValueRef::TempRef(offset, _) => unsafe {
            ctx.temp_buffer_mut().as_mut_ptr().add(offset as usize)
        },
        ValueRef::ArrayRef(id) => {
            let temp_buf = ctx.temp_buffer();
            let start = id as usize;
            if start + 2 > temp_buf.len() {
                return Err(VMErrorCode::MemoryViolation);
            }
            let len = temp_buf[start];
            if len < 64 {
                return Err(VMErrorCode::MemoryViolation);
            } // Result is 64 bytes
            if start + 2 + 64 > temp_buf.len() {
                return Err(VMErrorCode::MemoryViolation);
            }
            unsafe { ctx.temp_buffer_mut().as_mut_ptr().add(start + 2) }
        }
        ValueRef::StringRef(offset) => {
            let start = offset as usize;
            let temp_buf = ctx.temp_buffer();
            if start + 2 > temp_buf.len() {
                return Err(VMErrorCode::MemoryViolation);
            }
            let len = temp_buf[start];
            if len < 64 {
                return Err(VMErrorCode::MemoryViolation);
            }
            if start + 2 + 64 > temp_buf.len() {
                return Err(VMErrorCode::MemoryViolation);
            }
            unsafe { ctx.temp_buffer_mut().as_mut_ptr().add(start + 2) }
        }
        ValueRef::HeapString(id) => {
            let len_bytes = ctx.get_heap_data(id, 4)?;
            let len = u32::from_le_bytes(len_bytes.try_into().unwrap());
            if len < 64 {
                return Err(VMErrorCode::MemoryViolation);
            }
            ctx.get_heap_data_mut(id + 4, len)?.as_mut_ptr()
        }
        _ => return Err(VMErrorCode::TypeMismatch),
    };

    // Hash (32 bytes)
    let hash_ptr = match hash_ref {
        ValueRef::TempRef(offset, _) => unsafe { ctx.temp_buffer().as_ptr().add(offset as usize) },
        ValueRef::ArrayRef(id) => {
            let temp_buf = ctx.temp_buffer();
            let start = id as usize;
            if start + 2 > temp_buf.len() {
                return Err(VMErrorCode::MemoryViolation);
            }
            let len = temp_buf[start];
            if len < 32 {
                return Err(VMErrorCode::MemoryViolation);
            }
            // Only reading, but check bounds
            if start + 2 + 32 > temp_buf.len() {
                return Err(VMErrorCode::MemoryViolation);
            }
            unsafe { ctx.temp_buffer().as_ptr().add(start + 2) }
        }
        ValueRef::StringRef(offset) => {
            let start = offset as usize;
            let temp_buf = ctx.temp_buffer();
            if start + 2 > temp_buf.len() {
                return Err(VMErrorCode::MemoryViolation);
            }
            let len = temp_buf[start];
            if len < 32 {
                return Err(VMErrorCode::MemoryViolation);
            }
            if start + 2 + 32 > temp_buf.len() {
                return Err(VMErrorCode::MemoryViolation);
            }
            unsafe { ctx.temp_buffer().as_ptr().add(start + 2) }
        }
        ValueRef::HeapString(id) => {
            let len_bytes = ctx.get_heap_data(id, 4)?;
            let len = u32::from_le_bytes(len_bytes.try_into().unwrap());
            if len < 32 {
                return Err(VMErrorCode::MemoryViolation);
            }
            ctx.get_heap_data(id + 4, len)?.as_ptr()
        }
        _ => return Err(VMErrorCode::TypeMismatch),
    };

    // Signature (64 bytes)
    let sig_ptr = match signature_ref {
        ValueRef::TempRef(offset, _) => unsafe { ctx.temp_buffer().as_ptr().add(offset as usize) },
        ValueRef::ArrayRef(id) => {
            let temp_buf = ctx.temp_buffer();
            let start = id as usize;
            if start + 2 > temp_buf.len() {
                return Err(VMErrorCode::MemoryViolation);
            }
            let len = temp_buf[start];
            if len < 64 {
                return Err(VMErrorCode::MemoryViolation);
            }
            if start + 2 + 64 > temp_buf.len() {
                return Err(VMErrorCode::MemoryViolation);
            }
            unsafe { ctx.temp_buffer().as_ptr().add(start + 2) }
        }
        ValueRef::StringRef(offset) => {
            let start = offset as usize;
            let temp_buf = ctx.temp_buffer();
            if start + 2 > temp_buf.len() {
                return Err(VMErrorCode::MemoryViolation);
            }
            let len = temp_buf[start];
            if len < 64 {
                return Err(VMErrorCode::MemoryViolation);
            }
            if start + 2 + 64 > temp_buf.len() {
                return Err(VMErrorCode::MemoryViolation);
            }
            unsafe { ctx.temp_buffer().as_ptr().add(start + 2) }
        }
        ValueRef::HeapString(id) => {
            let len_bytes = ctx.get_heap_data(id, 4)?;
            let len = u32::from_le_bytes(len_bytes.try_into().unwrap());
            if len < 64 {
                return Err(VMErrorCode::MemoryViolation);
            }
            ctx.get_heap_data(id + 4, len)?.as_ptr()
        }
        _ => return Err(VMErrorCode::TypeMismatch),
    };

    #[cfg(target_os = "solana")]
    unsafe {
        syscalls::sol_secp256k1_recover(hash_ptr, recovery_id, sig_ptr, result_ptr);
    }
    #[cfg(not(target_os = "solana"))]
    {
        // Mock success
        unsafe {
            *result_ptr = 0;
        }
        let _ = (recovery_id, hash_ptr, sig_ptr);
    }

    Ok(())
}

#[inline(always)]
fn read_u16_le(data: &[u8], offset: usize) -> CompactResult<u16> {
    if offset + 2 > data.len() {
        return Err(VMErrorCode::MemoryViolation);
    }
    Ok(u16::from_le_bytes([data[offset], data[offset + 1]]))
}

#[inline(always)]
fn read_u64_le_or_zero(data: &[u8], offset: usize) -> u64 {
    if offset + 8 > data.len() {
        return 0;
    }
    u64::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
        data[offset + 4],
        data[offset + 5],
        data[offset + 6],
        data[offset + 7],
    ])
}

#[inline(always)]
fn copy_bytes_for_verify(
    ctx: &mut ExecutionManager,
    value_ref: &ValueRef,
    out: &mut [u8],
) -> CompactResult<usize> {
    match value_ref {
        ValueRef::StringRef(_) | ValueRef::HeapString(_) => {
            let (len, bytes) = ctx.extract_string_slice(value_ref)?;
            let len = len as usize;
            if len > out.len() {
                return Err(VMErrorCode::OutOfMemory);
            }
            out[..len].copy_from_slice(bytes);
            Ok(len)
        }
        ValueRef::TempRef(offset, len) => {
            let start = *offset as usize;
            let len = *len as usize;
            let end = start.saturating_add(len);
            if end > ctx.temp_buffer().len() || len > out.len() {
                return Err(VMErrorCode::MemoryViolation);
            }
            out[..len].copy_from_slice(&ctx.temp_buffer()[start..end]);
            Ok(len)
        }
        ValueRef::ArrayRef(id) => {
            let start = *id as usize;
            let temp = ctx.temp_buffer();
            if start + 2 > temp.len() {
                return Err(VMErrorCode::MemoryViolation);
            }

            let len = temp[start] as usize;
            let element_type = temp[start + 1];
            if len > out.len() {
                return Err(VMErrorCode::OutOfMemory);
            }

            if element_type == 0 {
                let data_start = start + 2;
                let data_end = data_start.saturating_add(len);
                if data_end > temp.len() {
                    return Err(VMErrorCode::MemoryViolation);
                }
                out[..len].copy_from_slice(&temp[data_start..data_end]);
                return Ok(len);
            }

            let mut cursor = start + 2;
            for i in 0..len {
                if cursor >= temp.len() {
                    return Err(VMErrorCode::MemoryViolation);
                }

                match ValueRef::deserialize_from(&temp[cursor..]) {
                    Ok(v) => {
                        out[i] = match v {
                            ValueRef::U8(n) => n,
                            ValueRef::U64(n) if n <= u8::MAX as u64 => n as u8,
                            ValueRef::I64(n) if (0..=u8::MAX as i64).contains(&n) => n as u8,
                            _ => return Err(VMErrorCode::TypeMismatch),
                        };
                        cursor += v.serialized_size();
                    }
                    Err(_) => {
                        let end = cursor.saturating_add(len);
                        if end > temp.len() {
                            return Err(VMErrorCode::MemoryViolation);
                        }
                        out[..len].copy_from_slice(&temp[cursor..end]);
                        return Ok(len);
                    }
                }
            }

            Ok(len)
        }
        ValueRef::U64(0) => Ok(0),
        _ => Err(VMErrorCode::TypeMismatch),
    }
}

#[inline(never)]
pub fn handle_syscall_verify_ed25519_instruction(ctx: &mut ExecutionManager) -> CompactResult<()> {
    debug_log!("MitoVM: SYSCALL_VERIFY_ED25519_INSTRUCTION");

    // Stack (push order): instruction_sysvar, expected_pubkey, message, signature.
    // Pop reverse order:
    let signature_ref = ctx.pop()?;
    let message_ref = ctx.pop()?;
    let expected_pubkey_ref = ctx.pop()?;
    let instruction_sysvar_ref = ctx.pop()?;

    let instruction_sysvar_idx = match instruction_sysvar_ref {
        ValueRef::AccountRef(idx, _) => idx,
        _ => return Err(VMErrorCode::TypeMismatch),
    };

    let expected_pubkey = ctx.extract_pubkey(&expected_pubkey_ref)?;

    // SBF stack is 4KB per frame; keep bounded buffers for verification inputs.
    let mut message_buf = [0u8; 1024];
    let mut signature_buf = [0u8; 64];
    let message_len = match copy_bytes_for_verify(ctx, &message_ref, &mut message_buf) {
        Ok(v) => v,
        Err(_) => {
            ctx.push(ValueRef::Bool(false))?;
            return Ok(());
        }
    };
    let signature_len = match copy_bytes_for_verify(ctx, &signature_ref, &mut signature_buf) {
        Ok(v) => v,
        Err(_) => {
            ctx.push(ValueRef::Bool(false))?;
            return Ok(());
        }
    };

    if signature_len != 64 && signature_len != 0 {
        ctx.push(ValueRef::Bool(false))?;
        return Ok(());
    }

    let account = ctx.get_account_for_read(instruction_sysvar_idx)?;
    let instruction_sysvar_data = unsafe { account.borrow_data_unchecked() };
    let instructions = unsafe { InstructionSysvar::new_unchecked(instruction_sysvar_data) };

    let instruction_count = instructions.num_instructions() as usize;
    #[cfg(target_os = "solana")]
    unsafe {
        // tag=0xE191: verifier entry diagnostics
        sol_log_64(
            0xE191,
            instruction_sysvar_idx as u64,
            instruction_count as u64,
            message_len as u64,
            signature_len as u64,
        );
    }
    if instruction_count == 0 {
        ctx.push(ValueRef::Bool(false))?;
        return Ok(());
    }

    let mut valid = false;
    for ix_index in 0..instruction_count {
        let ix = match instructions.load_instruction_at(ix_index) {
            Ok(ix) => ix,
            Err(_) => continue,
        };

        // Only inspect ed25519 precompile instructions.
        if ix.get_program_id() != &ED25519_PROGRAM_ID_BYTES {
            continue;
        }

        let ix_data = ix.get_instruction_data();

        // Ed25519 instruction data layout:
        // [u8 signature_count][u8 padding][14-byte offsets ...][payloads]
        if ix_data.len() < 16 {
            continue;
        }
        let signature_count = ix_data[0] as usize;
        if signature_count != 1 {
            continue;
        }

        let off = 2usize;
        let signature_offset = match read_u16_le(ix_data, off) {
            Ok(v) => v as usize,
            Err(_) => continue,
        };
        let signature_instruction_index = match read_u16_le(ix_data, off + 2) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let pubkey_offset = match read_u16_le(ix_data, off + 4) {
            Ok(v) => v as usize,
            Err(_) => continue,
        };
        let pubkey_instruction_index = match read_u16_le(ix_data, off + 6) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let message_offset = match read_u16_le(ix_data, off + 8) {
            Ok(v) => v as usize,
            Err(_) => continue,
        };
        let message_size = match read_u16_le(ix_data, off + 10) {
            Ok(v) => v as usize,
            Err(_) => continue,
        };
        let message_instruction_index = match read_u16_le(ix_data, off + 12) {
            Ok(v) => v,
            Err(_) => continue,
        };

        if signature_instruction_index != u16::MAX
            || pubkey_instruction_index != u16::MAX
            || message_instruction_index != u16::MAX
        {
            continue;
        }

        if signature_offset + 64 > ix_data.len()
            || pubkey_offset + 32 > ix_data.len()
            || message_offset + message_size > ix_data.len()
        {
            continue;
        }

        let signed_pubkey = &ix_data[pubkey_offset..pubkey_offset + 32];
        let signed_signature = &ix_data[signature_offset..signature_offset + 64];
        let signed_message = &ix_data[message_offset..message_offset + message_size];

        let pubkey_match = signed_pubkey == expected_pubkey.as_slice();
        let signature_match = if signature_len == 0 {
            true
        } else {
            signed_signature == &signature_buf[..64]
        };
        let message_len_match = message_size == message_len;
        let message_match = signed_message == &message_buf[..message_len];
        valid = pubkey_match && signature_match && message_len_match && message_match;
        #[cfg(target_os = "solana")]
        unsafe {
            // tag=0xE192: per-ed25519 instruction comparison flags
            sol_log_64(
                0xE192,
                ix_index as u64,
                pubkey_match as u64,
                signature_match as u64,
                ((message_len_match as u64) << 1) | (message_match as u64),
            );
            // tag=0xE193: expected message words [0..3]
            sol_log_64(
                0xE193,
                read_u64_le_or_zero(&message_buf[..message_len], 0),
                read_u64_le_or_zero(&message_buf[..message_len], 8),
                read_u64_le_or_zero(&message_buf[..message_len], 16),
                read_u64_le_or_zero(&message_buf[..message_len], 24),
            );
            // tag=0xE194: expected message words [4..6]
            sol_log_64(
                0xE194,
                read_u64_le_or_zero(&message_buf[..message_len], 32),
                read_u64_le_or_zero(&message_buf[..message_len], 40),
                read_u64_le_or_zero(&message_buf[..message_len], 48),
                message_len as u64,
            );
            // tag=0xE195: signed message words [0..3]
            sol_log_64(
                0xE195,
                read_u64_le_or_zero(signed_message, 0),
                read_u64_le_or_zero(signed_message, 8),
                read_u64_le_or_zero(signed_message, 16),
                read_u64_le_or_zero(signed_message, 24),
            );
            // tag=0xE196: signed message words [4..6]
            sol_log_64(
                0xE196,
                read_u64_le_or_zero(signed_message, 32),
                read_u64_le_or_zero(signed_message, 40),
                read_u64_le_or_zero(signed_message, 48),
                message_size as u64,
            );
        }

        if valid {
            break;
        }
    }

    ctx.push(ValueRef::Bool(valid))?;
    Ok(())
}

// Curve syscalls - simplified placeholders calling raw syscalls assuming correct pointers
// macro_rules! impl_curve_syscall { ... } // Unused

// Implementing one as example
#[inline(always)]
pub fn handle_syscall_alt_bn128_compression(ctx: &mut ExecutionManager) -> CompactResult<()> {
    debug_log!("MitoVM: SYSCALL_ALT_BN128_COMPRESSION");
    // op, input, input_size, result
    let result_ref = ctx.pop()?;
    let input_size = ctx.pop()?.as_u64().ok_or(VMErrorCode::TypeMismatch)?;
    let input_ref = ctx.pop()?;
    let op = ctx.pop()?.as_u64().ok_or(VMErrorCode::TypeMismatch)?;

    let result_ptr = match result_ref {
        ValueRef::TempRef(offset, _) => unsafe {
            ctx.temp_buffer_mut().as_mut_ptr().add(offset as usize)
        },
        _ => return Err(VMErrorCode::TypeMismatch),
    };

    let input_ptr = match input_ref {
        ValueRef::TempRef(offset, _) => unsafe { ctx.temp_buffer().as_ptr().add(offset as usize) },
        _ => return Err(VMErrorCode::TypeMismatch),
    };

    #[cfg(target_os = "solana")]
    unsafe {
        syscalls::sol_alt_bn128_compression(op, input_ptr, input_size, result_ptr);
    }

    #[cfg(not(target_os = "solana"))]
    {
        // Suppress unused warnings
        let _ = (input_size, op, result_ptr, input_ptr);
    }

    Ok(())
}

// Remaining curve syscalls are intentionally unsupported until runtime integration lands.
#[inline(always)]
pub fn handle_syscall_alt_bn128_group_op(_ctx: &mut ExecutionManager) -> CompactResult<()> {
    debug_log!("MitoVM: SYSCALL_ALT_BN128_GROUP_OP - runtime integration required");
    Err(VMErrorCode::RuntimeIntegrationRequired)
}

#[inline(always)]
pub fn handle_syscall_big_mod_exp(_ctx: &mut ExecutionManager) -> CompactResult<()> {
    debug_log!("MitoVM: SYSCALL_BIG_MOD_EXP - runtime integration required");
    Err(VMErrorCode::RuntimeIntegrationRequired)
}

#[inline(always)]
pub fn handle_syscall_curve_group_op(_ctx: &mut ExecutionManager) -> CompactResult<()> {
    debug_log!("MitoVM: SYSCALL_CURVE_GROUP_OP - runtime integration required");
    Err(VMErrorCode::RuntimeIntegrationRequired)
}

#[inline(always)]
pub fn handle_syscall_curve_multiscalar_mul(_ctx: &mut ExecutionManager) -> CompactResult<()> {
    debug_log!("MitoVM: SYSCALL_CURVE_MULTISCALAR_MUL - runtime integration required");
    Err(VMErrorCode::RuntimeIntegrationRequired)
}

#[inline(always)]
pub fn handle_syscall_curve_pairing_map(_ctx: &mut ExecutionManager) -> CompactResult<()> {
    debug_log!("MitoVM: SYSCALL_CURVE_PAIRING_MAP - runtime integration required");
    Err(VMErrorCode::RuntimeIntegrationRequired)
}

#[inline(always)]
pub fn handle_syscall_curve_validate_point(_ctx: &mut ExecutionManager) -> CompactResult<()> {
    debug_log!("MitoVM: SYSCALL_CURVE_VALIDATE_POINT - runtime integration required");
    Err(VMErrorCode::RuntimeIntegrationRequired)
}
