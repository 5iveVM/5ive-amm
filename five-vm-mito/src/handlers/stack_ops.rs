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
                let val = {
                    let bytes = ctx.read_pool_bytes(idx, 2)?;
                    let lo = u64::from_le_bytes([
                        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6],
                        bytes[7],
                    ]);
                    let hi = u64::from_le_bytes([
                        bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14],
                        bytes[15],
                    ]);
                    ((hi as u128) << 64) | (lo as u128)
                };
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
                let offset = ctx.alloc_temp(32)?;
                let bytes_ptr = {
                    let bytes = ctx.read_pool_bytes(idx, 4)?;
                    bytes.as_ptr()
                };
                // SAFETY: pointer is into immutable bytecode memory and length is fixed to 32 bytes.
                let bytes = unsafe { core::slice::from_raw_parts(bytes_ptr, 32) };
                ctx.temp_buffer_mut()[offset as usize..offset as usize + 32].copy_from_slice(bytes);
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
                ctx.push(ValueRef::U16(val))?;
            } else {
                let val = ctx.fetch_u16()?;
                ctx.push(ValueRef::U16(val))?;
            }
        }
        PUSH_U32 => {
            if pool_enabled {
                let idx = ctx.fetch_byte()? as u16;
                let val = ctx.read_pool_slot_u64(idx)? as u32;
                ctx.push(ValueRef::U32(val))?;
            } else {
                let val = ctx.fetch_u32()?;
                ctx.push(ValueRef::U32(val))?;
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
            ctx.push(ValueRef::U16(val))?;
        }
        PUSH_U32_W => {
            if !pool_enabled {
                return Err(VMErrorCode::InvalidInstruction);
            }
            let idx = ctx.fetch_u16()?;
            let val = ctx.read_pool_slot_u64(idx)? as u32;
            ctx.push(ValueRef::U32(val))?;
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
            let val = {
                let bytes = ctx.read_pool_bytes(idx, 2)?;
                let lo = u64::from_le_bytes([
                    bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
                ]);
                let hi = u64::from_le_bytes([
                    bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14],
                    bytes[15],
                ]);
                ((hi as u128) << 64) | (lo as u128)
            };
            vm_push_u128!(ctx, val);
        }
        PUSH_PUBKEY_W => {
            if !pool_enabled {
                return Err(VMErrorCode::InvalidInstruction);
            }
            let idx = ctx.fetch_u16()?;
            let offset = ctx.alloc_temp(32)?;
            let bytes_ptr = {
                let bytes = ctx.read_pool_bytes(idx, 4)?;
                bytes.as_ptr()
            };
            // SAFETY: pointer is into immutable bytecode memory and length is fixed to 32 bytes.
            let bytes = unsafe { core::slice::from_raw_parts(bytes_ptr, 32) };
            ctx.temp_buffer_mut()[offset as usize..offset as usize + 32].copy_from_slice(bytes);
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
    use super::*;
    use crate::{context::ExecutionContext, stack::StackStorage};
    use five_protocol::FEATURE_CONSTANT_POOL;
    use pinocchio::{account_info::AccountInfo, pubkey::Pubkey};

    fn new_ctx<'a>(
        bytecode: &'a [u8],
        storage: &'a mut StackStorage,
        header_features: u32,
        pool_offset: u32,
        pool_slots: u16,
    ) -> ExecutionContext<'a> {
        let accounts: &'a [AccountInfo] = &[];
        let program_id = Pubkey::default();
        let mut ctx = ExecutionContext::new(
            bytecode,
            accounts,
            program_id,
            &[],
            0,
            storage,
            0,
            0,
            pool_offset,
            pool_slots,
            0,
            0,
        );
        ctx.set_header_features(header_features);
        ctx
    }

    #[test]
    fn push_u16_and_u32_push_narrow_refs() {
        let mut storage = StackStorage::new();

        let u16_bytes = 0xBEEFu16.to_le_bytes();
        let mut ctx = new_ctx(&u16_bytes, &mut storage, 0, 0, 0);
        handle_stack_ops(PUSH_U16, &mut ctx).expect("PUSH_U16 should succeed");
        assert_eq!(ctx.pop().unwrap(), ValueRef::U16(0xBEEF));

        let mut storage = StackStorage::new();
        let u32_bytes = 0x1122_3344u32.to_le_bytes();
        let mut ctx = new_ctx(&u32_bytes, &mut storage, 0, 0, 0);
        handle_stack_ops(PUSH_U32, &mut ctx).expect("PUSH_U32 should succeed");
        assert_eq!(ctx.pop().unwrap(), ValueRef::U32(0x1122_3344));
    }

    #[test]
    fn push_u32_w_reads_pool_and_pushes_narrow_ref() {
        let mut storage = StackStorage::new();

        // Layout: [u16 pool index][pad 2][pool slot 0 as u64]
        let mut bytecode = vec![0u8; 12];
        bytecode[0..2].copy_from_slice(&0u16.to_le_bytes()); // PUSH_U32_W pool index
        bytecode[4..12].copy_from_slice(&0xAABB_CCDDu64.to_le_bytes()); // pool slot 0

        let mut ctx = new_ctx(&bytecode, &mut storage, FEATURE_CONSTANT_POOL, 4, 1);
        handle_stack_ops(PUSH_U32_W, &mut ctx).expect("PUSH_U32_W should succeed");
        assert_eq!(ctx.pop().unwrap(), ValueRef::U32(0xAABB_CCDD));
    }
}
