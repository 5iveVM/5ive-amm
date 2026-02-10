//! Stack operations handler for MitoVM (0x10-0x1F)
//!
//! This module handles stack operations in the 0x10-0x1F range including
//! POP, DUP, SWAP, PICK, ROT, DROP, OVER, and PUSH operations.

use crate::{
    context::ExecutionManager,
    error::{CompactResult, VMErrorCode},
    push_i64,
    // Import stack operation macros
    push_u8,
    vm_push_bool,
    vm_push_u128,
    vm_push_u64,
};
use five_protocol::{opcodes::*, ValueRef};

/// Process stack manipulation opcodes including PUSH variants, POP, DUP, SWAP, and PICK.
/// Handles the 0x10-0x1F opcode range exclusively.
#[inline(always)]
pub fn handle_stack_ops(opcode: u8, ctx: &mut ExecutionManager) -> CompactResult<()> {
    let pool_enabled = ctx.pool_enabled();
    match opcode {
        PUSH_U8 => {
            if pool_enabled {
                let idx = ctx.fetch_byte()? as u16;
                let val = ctx.read_pool_slot_u64(idx)? as u8;
                push_u8!(ctx, val);
            } else {
                let val = ctx.fetch_byte()?;
                push_u8!(ctx, val);
            }
        }
        PUSH_U64 => {
            if pool_enabled {
                let idx = ctx.fetch_byte()? as u16;
                let val = ctx.read_pool_slot_u64(idx)?;
                vm_push_u64!(ctx, val);
            } else {
                let val = ctx.fetch_u64()?;
                vm_push_u64!(ctx, val);
            }
        }
        PUSH_I64 => {
            if pool_enabled {
                let idx = ctx.fetch_byte()? as u16;
                let val = ctx.read_pool_slot_u64(idx)?;
                push_i64!(ctx, val as i64);
            } else {
                let val = ctx.fetch_u64()?;
                push_i64!(ctx, val as i64);
            }
        }
        PUSH_U128 => {
            if pool_enabled {
                let idx = ctx.fetch_byte()? as u16;
                let bytes = ctx.read_pool_bytes(idx, 2)?;
                let mut buf = [0u8; 16];
                buf.copy_from_slice(bytes);
                let val = u128::from_le_bytes(buf);
                vm_push_u128!(ctx, val);
            } else {
                let val = ctx.fetch_u128()?;
                vm_push_u128!(ctx, val);
            }
        }
        PUSH_BOOL => {
            if pool_enabled {
                let idx = ctx.fetch_byte()? as u16;
                let val = ctx.read_pool_slot_u64(idx)? != 0;
                vm_push_bool!(ctx, val);
            } else {
                let val = ctx.fetch_byte()? != 0;
                vm_push_bool!(ctx, val);
            }
        }
        PUSH_PUBKEY => {
            if pool_enabled {
                let idx = ctx.fetch_byte()? as u16;
                let mut bytes = [0u8; 32];
                bytes.copy_from_slice(ctx.read_pool_bytes(idx, 4)?);
                let offset = ctx.alloc_temp(32)?;
                ctx.temp_buffer_mut()[offset as usize..offset as usize + 32]
                    .copy_from_slice(&bytes);
                ctx.push(ValueRef::TempRef(offset, 32))?;
            } else {
                let offset = ctx.fetch_pubkey_to_temp()?;
                ctx.push(ValueRef::TempRef(offset, 32))?;
            }
        }
        // ===== PUSH OPERATIONS (0x18-0x1F) =====
        PUSH_U16 => {
            if pool_enabled {
                let idx = ctx.fetch_byte()? as u16;
                let val = ctx.read_pool_slot_u64(idx)? as u16;
                vm_push_u64!(ctx, val as u64);
            } else {
                let val = ctx.fetch_u16()?;
                vm_push_u64!(ctx, val as u64);
            }
        }
        PUSH_U32 => {
            if pool_enabled {
                let idx = ctx.fetch_byte()? as u16;
                let val = ctx.read_pool_slot_u64(idx)? as u32;
                vm_push_u64!(ctx, val as u64);
            } else {
                let val = ctx.fetch_u32()?;
                vm_push_u64!(ctx, val as u64);
            }
        }
        // ===== WIDE PUSH OPERATIONS (u16 index) =====
        PUSH_U8_W => {
            if !pool_enabled {
                return Err(VMErrorCode::InvalidInstruction);
            }
            let idx = ctx.fetch_u16()?;
            let val = ctx.read_pool_slot_u64(idx)? as u8;
            push_u8!(ctx, val);
        }
        PUSH_U16_W => {
            if !pool_enabled {
                return Err(VMErrorCode::InvalidInstruction);
            }
            let idx = ctx.fetch_u16()?;
            let val = ctx.read_pool_slot_u64(idx)? as u16;
            vm_push_u64!(ctx, val as u64);
        }
        PUSH_U32_W => {
            if !pool_enabled {
                return Err(VMErrorCode::InvalidInstruction);
            }
            let idx = ctx.fetch_u16()?;
            let val = ctx.read_pool_slot_u64(idx)? as u32;
            vm_push_u64!(ctx, val as u64);
        }
        PUSH_U64_W => {
            if !pool_enabled {
                return Err(VMErrorCode::InvalidInstruction);
            }
            let idx = ctx.fetch_u16()?;
            let val = ctx.read_pool_slot_u64(idx)?;
            vm_push_u64!(ctx, val);
        }
        PUSH_I64_W => {
            if !pool_enabled {
                return Err(VMErrorCode::InvalidInstruction);
            }
            let idx = ctx.fetch_u16()?;
            let val = ctx.read_pool_slot_u64(idx)?;
            push_i64!(ctx, val as i64);
        }
        PUSH_BOOL_W => {
            if !pool_enabled {
                return Err(VMErrorCode::InvalidInstruction);
            }
            let idx = ctx.fetch_u16()?;
            let val = ctx.read_pool_slot_u64(idx)? != 0;
            vm_push_bool!(ctx, val);
        }
        PUSH_U128_W => {
            if !pool_enabled {
                return Err(VMErrorCode::InvalidInstruction);
            }
            let idx = ctx.fetch_u16()?;
            let bytes = ctx.read_pool_bytes(idx, 2)?;
            let mut buf = [0u8; 16];
            buf.copy_from_slice(bytes);
            let val = u128::from_le_bytes(buf);
            vm_push_u128!(ctx, val);
        }
        PUSH_PUBKEY_W => {
            if !pool_enabled {
                return Err(VMErrorCode::InvalidInstruction);
            }
            let idx = ctx.fetch_u16()?;
            let mut bytes = [0u8; 32];
            bytes.copy_from_slice(ctx.read_pool_bytes(idx, 4)?);
            let offset = ctx.alloc_temp(32)?;
            ctx.temp_buffer_mut()[offset as usize..offset as usize + 32]
                .copy_from_slice(&bytes);
            ctx.push(ValueRef::TempRef(offset, 32))?;
        }
        // ===== BASIC STACK OPERATIONS (0x10-0x17) =====
        POP => {
            ctx.pop()?;
        }
        DUP => {
            ctx.dup()?;
        }
        DUP2 => {
            // Duplicate top 2 items on stack
            let val1 = ctx.pop()?;
            let val2 = ctx.pop()?;
            ctx.push(val2)?;
            ctx.push(val1)?;
            ctx.push(val2)?;
            ctx.push(val1)?;
        }
        SWAP => {
            ctx.swap()?;
        }
        PICK => {
            let index = ctx.fetch_byte()? as usize;
            ctx.pick(index as u8)?;
        }
        ROT => {
            // Rotate top 3 items: [a, b, c] -> [b, c, a]
            let c = ctx.pop()?;
            let b = ctx.pop()?;
            let a = ctx.pop()?;
            ctx.push(b)?;
            ctx.push(c)?;
            ctx.push(a)?;
        }
        DROP => {
            ctx.pop()?;
        } // Same as POP but semantic clarity
        OVER => {
            // Copy second item to top: [a, b] -> [a, b, a]
            let b = ctx.pop()?;
            let a = ctx.pop()?;
            ctx.push(a)?;
            ctx.push(b)?;
            ctx.push(a)?;
        }
        _ => return Err(VMErrorCode::InvalidInstruction),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    // NOTE: Full integration tests are in the main test suite
    // These are just basic unit tests for the stack operations
}
