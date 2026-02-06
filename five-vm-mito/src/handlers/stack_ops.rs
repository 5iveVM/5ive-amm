//! Stack operations handler for MitoVM (0x10-0x1F)
//!
//! 🎯 LOGICAL REORGANIZATION: All stack operations consolidated
//! This module handles ONLY stack operations in the 0x10-0x1F range including
//! POP, DUP, SWAP, PICK, ROT, DROP, OVER, and PUSH operations.
//! It manages the value stack and handles type-specific serialization.

use crate::{
    context::ExecutionManager,
    debug_log,
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
    match opcode {
        PUSH_U8 => {
            let val = ctx.fetch_byte()?;
            push_u8!(ctx, val);
        }
        PUSH_U64 => {
            let val = ctx.fetch_u64()?;
            vm_push_u64!(ctx, val);
        }
        PUSH_I64 => {
            let val = ctx.fetch_u64()?;
            push_i64!(ctx, val as i64);
        }
        PUSH_U128 => {
            let val = ctx.fetch_u128()?;
            vm_push_u128!(ctx, val);
        }
        PUSH_BOOL => {
            let val = ctx.fetch_byte()? != 0;
            vm_push_bool!(ctx, val);
        }
        PUSH_PUBKEY => {
            let offset = ctx.fetch_pubkey_to_temp()?;
            ctx.push(ValueRef::TempRef(offset, 32))?;
        }
        // ===== PUSH OPERATIONS (0x18-0x1F) =====
        PUSH_U16 => {
            let val = ctx.fetch_u16()?;
            vm_push_u64!(ctx, val as u64);
        }
        PUSH_U32 => {
            let val = ctx.fetch_u32()?;
            vm_push_u64!(ctx, val as u64);
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
