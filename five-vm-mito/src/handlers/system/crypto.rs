//! Cryptography syscall handlers

use crate::{
    context::ExecutionManager,
    debug_log,
    error::{CompactResult, VMErrorCode},
};
use five_protocol::ValueRef;

#[cfg(target_os = "solana")]
use pinocchio::syscalls;

/// Helper to parse data array (vals) for hash functions
/// Returns pointer to array of slices and count
fn parse_data_array(ctx: &mut ExecutionManager, data_ref: ValueRef) -> CompactResult<(*const u8, u64)> {
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
        ValueRef::StringRef(_) | ValueRef::TempRef(_,_) | ValueRef::HeapString(_) => {
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
    let vec_slice = &mut temp_buf[vec_offset as usize .. vec_offset as usize + 16];
    vec_slice[0..8].copy_from_slice(&ptr_u64.to_le_bytes());
    vec_slice[8..16].copy_from_slice(&len_u64.to_le_bytes());

    let vec_ptr = unsafe { temp_buf.as_ptr().add(vec_offset as usize) };
    Ok((vec_ptr, 1))
}

macro_rules! impl_hash_syscall {
    ($name:ident, $syscall:path, $log_name:expr) => {
        #[inline(always)]
        pub fn $name(ctx: &mut ExecutionManager) -> CompactResult<()> {
            debug_log!($log_name);

            // Pop result buffer, data array
            // Stack: push data, push result. Pop: result, data.
            let result_ref = ctx.pop()?;
            let data_ref = ctx.pop()?;

            // Result buffer
            let result_ptr = match result_ref {
                ValueRef::TempRef(offset, len) => {
                    if len < 32 { return Err(VMErrorCode::MemoryViolation); } // Most hashes 32 bytes
                    unsafe { ctx.temp_buffer_mut().as_mut_ptr().add(offset as usize) }
                }
                ValueRef::ArrayRef(id) => {
                     let temp_buf = ctx.temp_buffer();
                     let start = id as usize;
                     if start + 2 > temp_buf.len() {
                         return Err(VMErrorCode::MemoryViolation);
                     }
                     let len = temp_buf[start];
                     if len < 32 { return Err(VMErrorCode::MemoryViolation); }

                     // Ensure buffer has enough space for 32 bytes
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
                     if len < 32 { return Err(VMErrorCode::MemoryViolation); }

                     // Ensure buffer has enough space for 32 bytes
                     if start + 2 + 32 > temp_buf.len() {
                         return Err(VMErrorCode::MemoryViolation);
                     }

                     unsafe { ctx.temp_buffer_mut().as_mut_ptr().add(start + 2) }
                }
                ValueRef::HeapString(id) => {
                     // Check length stored at id
                     let len_bytes = ctx.get_heap_data(id, 4)?;
                     let len = u32::from_le_bytes(len_bytes.try_into().unwrap());
                     if len < 32 { return Err(VMErrorCode::MemoryViolation); }

                     ctx.get_heap_data_mut(id + 4, len)?.as_mut_ptr()
                }
                _ => return Err(VMErrorCode::TypeMismatch),
            };

            #[cfg(target_os = "solana")]
            unsafe {
                let (vals_ptr, val_len) = parse_data_array(ctx, data_ref)?;
                $syscall(vals_ptr, val_len, result_ptr);
            }
            #[cfg(not(target_os = "solana"))]
            {
                let (_vals_ptr, _val_len) = parse_data_array(ctx, data_ref)?;
                debug_log!(
                    "HASH SYSCALL MOCK vals_ptr={} val_len={}",
                    _vals_ptr as usize,
                    _val_len
                );
                unsafe { *result_ptr = 0; } // Zero mock
            }

            Ok(())
        }
    };
}

impl_hash_syscall!(handle_syscall_sha256, syscalls::sol_sha256, "MitoVM: SYSCALL_SHA256");
impl_hash_syscall!(handle_syscall_keccak256, syscalls::sol_keccak256, "MitoVM: SYSCALL_KECCAK256");
// impl_hash_syscall!(handle_syscall_blake3, syscalls::sol_blake3, "MitoVM: SYSCALL_BLAKE3");

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
        ValueRef::TempRef(offset, _) => {
             unsafe { ctx.temp_buffer_mut().as_mut_ptr().add(offset as usize) }
        }
        ValueRef::ArrayRef(id) => {
             let temp_buf = ctx.temp_buffer();
             let start = id as usize;
             if start + 2 > temp_buf.len() { return Err(VMErrorCode::MemoryViolation); }
             let len = temp_buf[start];
             if len < 32 { return Err(VMErrorCode::MemoryViolation); }
             // Ensure buffer has enough space for 32 bytes (Poseidon result size)
             if start + 2 + 32 > temp_buf.len() { return Err(VMErrorCode::MemoryViolation); }
             unsafe { ctx.temp_buffer_mut().as_mut_ptr().add(start + 2) }
        }
        ValueRef::StringRef(offset) => {
             let start = offset as usize;
             let temp_buf = ctx.temp_buffer();
             if start + 2 > temp_buf.len() { return Err(VMErrorCode::MemoryViolation); }
             let len = temp_buf[start];
             if len < 32 { return Err(VMErrorCode::MemoryViolation); }
             // Ensure buffer has enough space for 32 bytes
             if start + 2 + 32 > temp_buf.len() { return Err(VMErrorCode::MemoryViolation); }
             unsafe { ctx.temp_buffer_mut().as_mut_ptr().add(start + 2) }
        }
        ValueRef::HeapString(id) => {
             // We assume sufficient size validation happened at allocation or call site logic
             // But for safety we should get the ptr safely
             let len_bytes = ctx.get_heap_data(id, 4)?;
             let len = u32::from_le_bytes(len_bytes.try_into().unwrap());
             if len < 32 { return Err(VMErrorCode::MemoryViolation); }
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
         ValueRef::TempRef(offset, _) => unsafe { ctx.temp_buffer_mut().as_mut_ptr().add(offset as usize) },
         ValueRef::ArrayRef(id) => {
             let temp_buf = ctx.temp_buffer();
             let start = id as usize;
             if start + 2 > temp_buf.len() { return Err(VMErrorCode::MemoryViolation); }
             let len = temp_buf[start];
             if len < 64 { return Err(VMErrorCode::MemoryViolation); } // Result is 64 bytes
             if start + 2 + 64 > temp_buf.len() { return Err(VMErrorCode::MemoryViolation); }
             unsafe { ctx.temp_buffer_mut().as_mut_ptr().add(start + 2) }
         },
         ValueRef::StringRef(offset) => {
             let start = offset as usize;
             let temp_buf = ctx.temp_buffer();
             if start + 2 > temp_buf.len() { return Err(VMErrorCode::MemoryViolation); }
             let len = temp_buf[start];
             if len < 64 { return Err(VMErrorCode::MemoryViolation); }
             if start + 2 + 64 > temp_buf.len() { return Err(VMErrorCode::MemoryViolation); }
             unsafe { ctx.temp_buffer_mut().as_mut_ptr().add(start + 2) }
         },
         ValueRef::HeapString(id) => {
             let len_bytes = ctx.get_heap_data(id, 4)?;
             let len = u32::from_le_bytes(len_bytes.try_into().unwrap());
             if len < 64 { return Err(VMErrorCode::MemoryViolation); }
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
             if start + 2 > temp_buf.len() { return Err(VMErrorCode::MemoryViolation); }
             let len = temp_buf[start];
             if len < 32 { return Err(VMErrorCode::MemoryViolation); }
             // Only reading, but check bounds
             if start + 2 + 32 > temp_buf.len() { return Err(VMErrorCode::MemoryViolation); }
             unsafe { ctx.temp_buffer().as_ptr().add(start + 2) }
         },
         ValueRef::StringRef(offset) => {
             let start = offset as usize;
             let temp_buf = ctx.temp_buffer();
             if start + 2 > temp_buf.len() { return Err(VMErrorCode::MemoryViolation); }
             let len = temp_buf[start];
             if len < 32 { return Err(VMErrorCode::MemoryViolation); }
             if start + 2 + 32 > temp_buf.len() { return Err(VMErrorCode::MemoryViolation); }
             unsafe { ctx.temp_buffer().as_ptr().add(start + 2) }
         },
         ValueRef::HeapString(id) => {
             let len_bytes = ctx.get_heap_data(id, 4)?;
             let len = u32::from_le_bytes(len_bytes.try_into().unwrap());
             if len < 32 { return Err(VMErrorCode::MemoryViolation); }
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
             if start + 2 > temp_buf.len() { return Err(VMErrorCode::MemoryViolation); }
             let len = temp_buf[start];
             if len < 64 { return Err(VMErrorCode::MemoryViolation); }
             if start + 2 + 64 > temp_buf.len() { return Err(VMErrorCode::MemoryViolation); }
             unsafe { ctx.temp_buffer().as_ptr().add(start + 2) }
         },
         ValueRef::StringRef(offset) => {
             let start = offset as usize;
             let temp_buf = ctx.temp_buffer();
             if start + 2 > temp_buf.len() { return Err(VMErrorCode::MemoryViolation); }
             let len = temp_buf[start];
             if len < 64 { return Err(VMErrorCode::MemoryViolation); }
             if start + 2 + 64 > temp_buf.len() { return Err(VMErrorCode::MemoryViolation); }
             unsafe { ctx.temp_buffer().as_ptr().add(start + 2) }
         },
         ValueRef::HeapString(id) => {
             let len_bytes = ctx.get_heap_data(id, 4)?;
             let len = u32::from_le_bytes(len_bytes.try_into().unwrap());
             if len < 64 { return Err(VMErrorCode::MemoryViolation); }
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
         unsafe { *result_ptr = 0; }
         let _ = (recovery_id, hash_ptr, sig_ptr);
    }

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
        ValueRef::TempRef(offset, _) => unsafe { ctx.temp_buffer_mut().as_mut_ptr().add(offset as usize) },
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


// Remaining are placeholders for now to save space, but declared
#[inline(always)]
pub fn handle_syscall_alt_bn128_group_op(_ctx: &mut ExecutionManager) -> CompactResult<()> {
    debug_log!("MitoVM: SYSCALL_ALT_BN128_GROUP_OP - Stub");
    Ok(())
}

#[inline(always)]
pub fn handle_syscall_big_mod_exp(_ctx: &mut ExecutionManager) -> CompactResult<()> {
    debug_log!("MitoVM: SYSCALL_BIG_MOD_EXP - Stub");
    Ok(())
}

#[inline(always)]
pub fn handle_syscall_curve_group_op(_ctx: &mut ExecutionManager) -> CompactResult<()> {
    debug_log!("MitoVM: SYSCALL_CURVE_GROUP_OP - Stub");
    Ok(())
}

#[inline(always)]
pub fn handle_syscall_curve_multiscalar_mul(_ctx: &mut ExecutionManager) -> CompactResult<()> {
    debug_log!("MitoVM: SYSCALL_CURVE_MULTISCALAR_MUL - Stub");
    Ok(())
}

#[inline(always)]
pub fn handle_syscall_curve_pairing_map(_ctx: &mut ExecutionManager) -> CompactResult<()> {
    debug_log!("MitoVM: SYSCALL_CURVE_PAIRING_MAP - Stub");
    Ok(())
}

#[inline(always)]
pub fn handle_syscall_curve_validate_point(_ctx: &mut ExecutionManager) -> CompactResult<()> {
    debug_log!("MitoVM: SYSCALL_CURVE_VALIDATE_POINT - Stub");
    Ok(())
}
