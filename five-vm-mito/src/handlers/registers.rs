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
#[inline(always)]
pub fn handle_registers(opcode: u8, ctx: &mut ExecutionManager) -> CompactResult<()> {
    match opcode {
        LOAD_REG_U8 => {
            let reg = ctx.fetch_byte()?;
            let value = ctx.fetch_byte()?;
            ctx.set_register(reg, ValueRef::U8(value))?;
            debug_log!("MitoVM: LOAD_REG_U8 reg={} value={}", reg, value);
        }
        LOAD_REG_U32 => {
            let reg = ctx.fetch_byte()?;
            let value = ctx.fetch_u32()? as u64;
            ctx.set_register(reg, ValueRef::U64(value))?;
            debug_log!("MitoVM: LOAD_REG_U32 reg={} value={}", reg, value);
        }
        LOAD_REG_U64 => {
            let reg = ctx.fetch_byte()?;
            let value = ctx.fetch_u64()?;
            ctx.set_register(reg, ValueRef::U64(value))?;
            debug_log!("MitoVM: LOAD_REG_U64 reg={} value={}", reg, value);
        }
        LOAD_REG_BOOL => {
            let reg = ctx.fetch_byte()?;
            let value = ctx.fetch_byte()? != 0;
            ctx.set_register(reg, ValueRef::Bool(value))?;
            debug_log!("MitoVM: LOAD_REG_BOOL reg={} value={}", reg, value as u64);
        }
        LOAD_REG_PUBKEY => {
            let reg = ctx.fetch_byte()?;
            let offset = ctx.fetch_pubkey_to_temp()?;
            ctx.set_register(reg, ValueRef::PubkeyRef(offset as u16))?;
            debug_log!("MitoVM: LOAD_REG_PUBKEY reg={} offset={}", reg, offset);
        }
        ADD_REG => {
            let dest = ctx.fetch_byte()?;
            let src1 = ctx.fetch_byte()?;
            let src2 = ctx.fetch_byte()?;

            let val1 = ctx.get_register(src1)?;
            let val2 = ctx.get_register(src2)?;
            
            // Extract u64 values for arithmetic
            let num1 = match val1 {
                ValueRef::U64(v) => v,
                ValueRef::U8(v) => v as u64,
                ValueRef::Bool(b) => if b { 1 } else { 0 },
                _ => return Err(VMErrorCode::TypeMismatch),
            };
            let num2 = match val2 {
                ValueRef::U64(v) => v,
                ValueRef::U8(v) => v as u64,
                ValueRef::Bool(b) => if b { 1 } else { 0 },
                _ => return Err(VMErrorCode::TypeMismatch),
            };
            
            let result = num1.saturating_add(num2);
            ctx.set_register(dest, ValueRef::U64(result))?;
            debug_log!(
                "MitoVM: ADD_REG dest={} src1={} src2={} result={}",
                dest,
                src1,
                src2,
                result
            );
        }
        SUB_REG => {
            let dest = ctx.fetch_byte()?;
            let src1 = ctx.fetch_byte()?;
            let src2 = ctx.fetch_byte()?;

            let val1 = ctx.get_register(src1)?;
            let val2 = ctx.get_register(src2)?;
            
            let num1 = match val1 {
                ValueRef::U64(v) => v,
                ValueRef::U8(v) => v as u64,
                ValueRef::Bool(b) => if b { 1 } else { 0 },
                _ => return Err(VMErrorCode::TypeMismatch),
            };
            let num2 = match val2 {
                ValueRef::U64(v) => v,
                ValueRef::U8(v) => v as u64,
                ValueRef::Bool(b) => if b { 1 } else { 0 },
                _ => return Err(VMErrorCode::TypeMismatch),
            };
            
            let result = num1.wrapping_sub(num2);
            ctx.set_register(dest, ValueRef::U64(result))?;
            debug_log!("MitoVM: SUB_REG dest={} src1={} src2={} result={}", dest, src1, src2, result);
        }
        MUL_REG => {
            let dest = ctx.fetch_byte()?;
            let src1 = ctx.fetch_byte()?;
            let src2 = ctx.fetch_byte()?;

            let val1 = ctx.get_register(src1)?;
            let val2 = ctx.get_register(src2)?;
            
            let num1 = match val1 {
                ValueRef::U64(v) => v,
                ValueRef::U8(v) => v as u64,
                ValueRef::Bool(b) => if b { 1 } else { 0 },
                _ => return Err(VMErrorCode::TypeMismatch),
            };
            let num2 = match val2 {
                ValueRef::U64(v) => v,
                ValueRef::U8(v) => v as u64,
                ValueRef::Bool(b) => if b { 1 } else { 0 },
                _ => return Err(VMErrorCode::TypeMismatch),
            };
            
            let result = num1.wrapping_mul(num2);
            ctx.set_register(dest, ValueRef::U64(result))?;
            debug_log!("MitoVM: MUL_REG dest={} src1={} src2={} result={}", dest, src1, src2, result);
        }
        DIV_REG => {
            let dest = ctx.fetch_byte()?;
            let src1 = ctx.fetch_byte()?;
            let src2 = ctx.fetch_byte()?;

            let val1 = ctx.get_register(src1)?;
            let val2 = ctx.get_register(src2)?;
            
            let num1 = match val1 {
                ValueRef::U64(v) => v,
                ValueRef::U8(v) => v as u64,
                ValueRef::Bool(b) => if b { 1 } else { 0 },
                _ => return Err(VMErrorCode::TypeMismatch),
            };
            let num2 = match val2 {
                ValueRef::U64(v) => v,
                ValueRef::U8(v) => v as u64,
                ValueRef::Bool(b) => if b { 1 } else { 0 },
                _ => return Err(VMErrorCode::TypeMismatch),
            };
            
            if num2 == 0 {
                return Err(VMErrorCode::DivisionByZero);
            }
            
            let result = num1 / num2;
            ctx.set_register(dest, ValueRef::U64(result))?;
            debug_log!("MitoVM: DIV_REG dest={} src1={} src2={} result={}", dest, src1, src2, result);
        }
        EQ_REG => {
            let dest = ctx.fetch_byte()?;
            let src1 = ctx.fetch_byte()?;
            let src2 = ctx.fetch_byte()?;

            let val1 = ctx.get_register(src1)?;
            let val2 = ctx.get_register(src2)?;
            
            let result = val1 == val2;
            ctx.set_register(dest, ValueRef::Bool(result))?;
            debug_log!("MitoVM: EQ_REG dest={} src1={} src2={} result={}", dest, src1, src2, result as u8);
        }
        GT_REG => {
            let dest = ctx.fetch_byte()?;
            let src1 = ctx.fetch_byte()?;
            let src2 = ctx.fetch_byte()?;

            let val1 = ctx.get_register(src1)?;
            let val2 = ctx.get_register(src2)?;
            
            let num1 = match val1 {
                ValueRef::U64(v) => v,
                ValueRef::U8(v) => v as u64,
                ValueRef::Bool(b) => if b { 1 } else { 0 },
                _ => return Err(VMErrorCode::TypeMismatch),
            };
            let num2 = match val2 {
                ValueRef::U64(v) => v,
                ValueRef::U8(v) => v as u64,
                ValueRef::Bool(b) => if b { 1 } else { 0 },
                _ => return Err(VMErrorCode::TypeMismatch),
            };
            
            let result = num1 > num2;
            ctx.set_register(dest, ValueRef::Bool(result))?;
            debug_log!("MitoVM: GT_REG dest={} src1={} src2={} result={}", dest, src1, src2, result as u8);
        }
        LT_REG => {
            let dest = ctx.fetch_byte()?;
            let src1 = ctx.fetch_byte()?;
            let src2 = ctx.fetch_byte()?;

            let val1 = ctx.get_register(src1)?;
            let val2 = ctx.get_register(src2)?;
            
            let num1 = match val1 {
                ValueRef::U64(v) => v,
                ValueRef::U8(v) => v as u64,
                ValueRef::Bool(b) => if b { 1 } else { 0 },
                _ => return Err(VMErrorCode::TypeMismatch),
            };
            let num2 = match val2 {
                ValueRef::U64(v) => v,
                ValueRef::U8(v) => v as u64,
                ValueRef::Bool(b) => if b { 1 } else { 0 },
                _ => return Err(VMErrorCode::TypeMismatch),
            };
            
            let result = num1 < num2;
            ctx.set_register(dest, ValueRef::Bool(result))?;
            debug_log!("MitoVM: LT_REG dest={} src1={} src2={} result={}", dest, src1, src2, result as u8);
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
        COPY_REG => {
            let dest = ctx.fetch_byte()?;
            let src = ctx.fetch_byte()?;
            let value = ctx.get_register(src)?;
            ctx.set_register(dest, value)?;
            debug_log!("MitoVM: COPY_REG dest={} src={}", dest, src);
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
