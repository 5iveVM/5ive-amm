//! Register operations handler for MitoVM
//!
//! This module handles register operations including LOAD_REG_U8, LOAD_REG_U64,
//! ADD_REG, PUSH_REG, POP_REG, and CLEAR_REG. It manages the specialized register
//! file with 16 ValueRef registers optimized for performance.

use crate::{
    context::ExecutionManager,
    debug_log,
    error::{CompactResult, VMErrorCode},
};
use five_protocol::{opcodes::*, ValueRef};

/// Handle register operations (0xB0-0xBF)
/// 🎯 LOGICAL REORGANIZATION: Registers at 0xB0 (0xE0 is pattern fusion)
#[inline(never)]
pub fn handle_registers(opcode: u8, ctx: &mut ExecutionManager) -> CompactResult<()> {
    match opcode {
        LOAD_REG_U8 => {
            let reg = ctx.fetch_byte()?;
            let value = ctx.fetch_byte()? as u64;
            ctx.set_register(reg, ValueRef::U64(value))?;
            debug_log!("MitoVM: LOAD_REG_U8 reg={} value={}", reg, value);
        }
        LOAD_REG_U64 => {
            let reg = ctx.fetch_byte()?;
            let value = ctx.fetch_u64()?;
            ctx.set_register(reg, ValueRef::U64(value))?;
            debug_log!("MitoVM: LOAD_REG_U64 reg={} value={}", reg, value);
        }
        ADD_REG => {
            let dest = ctx.fetch_byte()?;
            let src1 = ctx.fetch_byte()?;
            let src2 = ctx.fetch_byte()?;

            let val1 = ctx
                .get_register(src1)?
                .as_u64()
                .ok_or(VMErrorCode::TypeMismatch)?;
            let val2 = ctx
                .get_register(src2)?
                .as_u64()
                .ok_or(VMErrorCode::TypeMismatch)?;
            let result = val1.saturating_add(val2);

            ctx.set_register(dest, ValueRef::U64(result))?;
            debug_log!(
                "MitoVM: ADD_REG dest={} src1={} src2={} result={}",
                dest,
                src1,
                src2,
                result
            );
        }
        PUSH_REG => {
            let reg = ctx.fetch_byte()?;
            let value = ctx.get_register(reg)?;
            ctx.push(value)?;
            debug_log!("MitoVM: PUSH_REG reg={}", reg);
        }
        POP_REG => {
            let reg = ctx.fetch_byte()?;
            let value = ctx.pop()?;
            ctx.set_register(reg, value)?;
            debug_log!("MitoVM: POP_REG reg={}", reg);
        }
        CLEAR_REG => {
            let reg = ctx.fetch_byte()?;
            ctx.set_register(reg, ValueRef::Empty)?;
            debug_log!("MitoVM: CLEAR_REG reg={}", reg);
        }
        _ => return Err(VMErrorCode::InvalidInstruction),
    }
    Ok(())
}
