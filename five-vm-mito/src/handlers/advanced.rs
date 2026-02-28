//! Advanced and experimental operations handler for MitoVM (0xF0-0xFF)
//!
//! This module handles advanced features including:
//! - Optional/Result type operations
//! - Advanced bulk operations
//! - Tuple operations (moved from stack range)
//! - Stack management operations (moved from stack range)

use crate::{
    context::ExecutionManager,
    debug_log,
    error::{CompactResult, VMErrorCode},
};
use five_protocol::{opcodes::*, ValueRef};

/// Handle advanced and experimental operations (0xF0-0xFF)
/// 🎯 LOGICAL REORGANIZATION: Optional/Result + experimental features
#[inline(always)]
pub fn handle_advanced(opcode: u8, ctx: &mut ExecutionManager) -> CompactResult<()> {
    match opcode {
        // Optional/Result type operations (0xF0-0xF6)
        RESULT_OK => {
            debug_log!("MitoVM: RESULT_OK - create Result::Ok value");
            let value = ctx.pop()?;
            let value_size = value.serialized_size();
            let total_size = 1 + value_size;
            let alloc_size = u8::try_from(total_size).map_err(|_| VMErrorCode::OutOfMemory)?;
            let offset = ctx.alloc_temp(alloc_size)?;
            ctx.temp_buffer_mut()[offset as usize] = 1; // Ok tag
            value
                .serialize_into(
                    &mut ctx.temp_buffer_mut()[offset as usize + 1..offset as usize + total_size],
                )
                .map_err(|_| VMErrorCode::ProtocolError)?;
            ctx.push(ValueRef::ResultRef(offset, alloc_size))?;
        }
        RESULT_ERR => {
            debug_log!("MitoVM: RESULT_ERR - create Result::Err value");
            let error_value = ctx.pop()?;
            let value_size = error_value.serialized_size();
            let total_size = 1 + value_size;
            let alloc_size = u8::try_from(total_size).map_err(|_| VMErrorCode::OutOfMemory)?;
            let offset = ctx.alloc_temp(alloc_size)?;
            ctx.temp_buffer_mut()[offset as usize] = 0; // Err tag
            error_value
                .serialize_into(
                    &mut ctx.temp_buffer_mut()[offset as usize + 1..offset as usize + total_size],
                )
                .map_err(|_| VMErrorCode::ProtocolError)?;
            ctx.push(ValueRef::ResultRef(offset, alloc_size))?;
        }
        OPTIONAL_SOME => {
            debug_log!("MitoVM: OPTIONAL_SOME - create Optional::Some value");
            let value = ctx.pop()?;
            let value_size = value.serialized_size();
            let total_size = 1 + value_size;
            let alloc_size = u8::try_from(total_size).map_err(|_| VMErrorCode::OutOfMemory)?;
            let offset = ctx.alloc_temp(alloc_size)?;
            ctx.temp_buffer_mut()[offset as usize] = 1; // Some tag
            value
                .serialize_into(
                    &mut ctx.temp_buffer_mut()[offset as usize + 1..offset as usize + total_size],
                )
                .map_err(|_| VMErrorCode::ProtocolError)?;
            ctx.push(ValueRef::OptionalRef(offset, alloc_size))?;
        }
        OPTIONAL_NONE => {
            debug_log!("MitoVM: OPTIONAL_NONE - create Optional::None value");
            let offset = ctx.alloc_temp(1)?;
            ctx.temp_buffer_mut()[offset as usize] = 0; // None tag
            ctx.push(ValueRef::OptionalRef(offset, 1))?;
        }
        OPTIONAL_UNWRAP => {
            debug_log!("MitoVM: OPTIONAL_UNWRAP - unwrap Optional value");
            let opt_ref = ctx.pop()?;
            let (offset, size) = match opt_ref {
                ValueRef::OptionalRef(off, sz) => (off, sz),
                _ => return Err(VMErrorCode::TypeMismatch),
            };
            let tag = ctx.temp_buffer()[offset as usize];
            if tag != 0 {
                let inner_ref = ValueRef::deserialize_from(
                    &ctx.temp_buffer()[offset as usize + 1..offset as usize + size as usize],
                )
                .map_err(|_| VMErrorCode::ProtocolError)?;
                ctx.push(inner_ref)?;
            } else {
                return Err(VMErrorCode::ConstraintViolation); // Unwrapping None
            }
        }
        OPTIONAL_IS_SOME => {
            debug_log!("MitoVM: OPTIONAL_IS_SOME - check if Optional has value");
            let opt_ref = ctx.pop()?;
            let (offset, _size) = match opt_ref {
                ValueRef::OptionalRef(off, sz) => (off, sz),
                _ => return Err(VMErrorCode::TypeMismatch),
            };
            let has_value = ctx.temp_buffer()[offset as usize] != 0;
            ctx.push(ValueRef::Bool(has_value))?;
        }
        OPTIONAL_GET_VALUE => {
            debug_log!("MitoVM: OPTIONAL_GET_VALUE - get value from Optional (unsafe)");
            let opt_ref = ctx.pop()?;
            let (offset, size) = match opt_ref {
                ValueRef::OptionalRef(off, sz) => (off, sz),
                _ => return Err(VMErrorCode::TypeMismatch),
            };
            if size > 1 {
                let inner_ref = ValueRef::deserialize_from(
                    &ctx.temp_buffer()[offset as usize + 1..offset as usize + size as usize],
                )
                .map_err(|_| VMErrorCode::ProtocolError)?;
                ctx.push(inner_ref)?;
            } else {
                ctx.push(ValueRef::Empty)?;
            }
        }

        _ => {
            debug_log!(
                "MitoVM: Invalid advanced operation opcode {} in 0xF0-0xFF range",
                opcode
            );
            return Err(VMErrorCode::InvalidInstruction);
        }
    }
    Ok(())
}
