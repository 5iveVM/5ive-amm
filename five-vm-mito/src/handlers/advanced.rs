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
    // Import stack operation macros
    vm_push_u64,
};
use five_protocol::{opcodes::*, ValueRef};

/// Handle advanced and experimental operations (0xF0-0xFF)
/// 🎯 LOGICAL REORGANIZATION: Optional/Result + experimental features
#[inline(never)]
pub fn handle_advanced(opcode: u8, ctx: &mut ExecutionManager) -> CompactResult<()> {
    match opcode {
        // Optional/Result type operations (0xF0-0xF6)
        RESULT_OK => {
            debug_log!("MitoVM: RESULT_OK - create Result::Ok value");
            let value = ctx.pop()?;
            let value_size = value.serialized_size();
            let total_size = 1 + value_size;
            let offset = ctx.alloc_temp(total_size as u8)?;
            ctx.temp_buffer_mut()[offset as usize] = 1; // Ok tag
            value
                .serialize_into(
                    &mut ctx.temp_buffer_mut()[offset as usize + 1..offset as usize + total_size],
                )
                .map_err(|_| VMErrorCode::ProtocolError)?;
            ctx.push(ValueRef::ResultRef(offset, total_size as u8))?;
        }
        RESULT_ERR => {
            debug_log!("MitoVM: RESULT_ERR - create Result::Err value");
            let error_value = ctx.pop()?;
            let value_size = error_value.serialized_size();
            let total_size = 1 + value_size;
            let offset = ctx.alloc_temp(total_size as u8)?;
            ctx.temp_buffer_mut()[offset as usize] = 0; // Err tag
            error_value
                .serialize_into(
                    &mut ctx.temp_buffer_mut()[offset as usize + 1..offset as usize + total_size],
                )
                .map_err(|_| VMErrorCode::ProtocolError)?;
            ctx.push(ValueRef::ResultRef(offset, total_size as u8))?;
        }
        OPTIONAL_SOME => {
            debug_log!("MitoVM: OPTIONAL_SOME - create Optional::Some value");
            let value = ctx.pop()?;
            let value_size = value.serialized_size();
            let total_size = 1 + value_size;
            let offset = ctx.alloc_temp(total_size as u8)?;
            ctx.temp_buffer_mut()[offset as usize] = 1; // Some tag
            value
                .serialize_into(
                    &mut ctx.temp_buffer_mut()[offset as usize + 1..offset as usize + total_size],
                )
                .map_err(|_| VMErrorCode::ProtocolError)?;
            ctx.push(ValueRef::OptionalRef(offset, total_size as u8))?;
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

        // Advanced bulk operations (0xF7)
        BULK_LOAD_FIELD_N => {
            debug_log!("MitoVM: BULK_LOAD_FIELD_N - bulk load N fields");
            return Err(VMErrorCode::InvalidInstruction); // Complex operation for future implementation
        }

        // Tuple operations (0xF8-0xFA) - moved from stack range
        CREATE_TUPLE => {
            debug_log!("MitoVM: CREATE_TUPLE - create tuple");
            let element_count = ctx.fetch_byte()? as usize;
            debug_log!(
                "MitoVM: Creating tuple with {} elements",
                element_count as u32
            );

            // For now, store as array in temp buffer (simplified tuple implementation)
            const MAX_TUPLE_ELEMENTS: usize = 16;
            if element_count > MAX_TUPLE_ELEMENTS {
                return Err(VMErrorCode::StackError);
            }
            if (ctx.stack.sp as usize) < element_count {
                return Err(VMErrorCode::StackError);
            }
            // Calculate size without storing elements
            let mut total_size = 2;
            for i in 0..element_count {
                let idx = ctx.stack.sp as usize - 1 - i;
                let element = ctx.stack.stack[idx];
                total_size += element.serialized_size();
                if total_size > 62 {
                    return Err(VMErrorCode::OutOfMemory);
                }
            }

            let tuple_id = ctx.alloc_temp(total_size as u8)?;
            ctx.temp_buffer_mut()[tuple_id as usize] = element_count as u8;
            ctx.temp_buffer_mut()[tuple_id as usize + 1] = 255; // Special marker for tuple

            // Serialize elements directly in reverse order
            let mut write_offset = tuple_id as usize + total_size;
            for _ in 0..element_count {
                let element = ctx.pop()?;
                let size = element.serialized_size();
                write_offset -= size;
                element
                    .serialize_into(&mut ctx.temp_buffer_mut()[write_offset..write_offset + size])
                    .map_err(|_| VMErrorCode::ProtocolError)?;
            }

            ctx.push(ValueRef::ArrayRef(tuple_id))?; // Reuse ArrayRef for tuples
        }
        TUPLE_GET => {
            debug_log!("MitoVM: TUPLE_GET - get tuple element");
            let index = ctx.pop()?.as_u8().ok_or(VMErrorCode::TypeMismatch)? as usize;
            let tuple_ref = ctx.pop()?;

            let tuple_id = match tuple_ref {
                ValueRef::ArrayRef(id) => id,
                _ => return Err(VMErrorCode::TypeMismatch),
            };

            let element_count = ctx.temp_buffer()[tuple_id as usize] as usize;
            if index >= element_count {
                return Err(VMErrorCode::IndexOutOfBounds);
            }

            // Find element (similar to array indexing)
            let mut current_offset = tuple_id as usize + 2;
            for _ in 0..index {
                let element = ValueRef::deserialize_from(&ctx.temp_buffer()[current_offset..])
                    .map_err(|_| VMErrorCode::ProtocolError)?;
                current_offset += element.serialized_size();
            }

            let element = ValueRef::deserialize_from(&ctx.temp_buffer()[current_offset..])
                .map_err(|_| VMErrorCode::ProtocolError)?;
            ctx.push(element)?;
        }
        UNPACK_TUPLE => {
            debug_log!("MitoVM: UNPACK_TUPLE - unpack tuple elements");
            let tuple_ref = ctx.pop()?;

            let tuple_id = match tuple_ref {
                ValueRef::ArrayRef(id) => id,
                _ => return Err(VMErrorCode::TypeMismatch),
            };

            let element_count = ctx.temp_buffer()[tuple_id as usize] as usize;
            let mut elements = Vec::with_capacity(element_count);

            // Deserialize all elements
            let mut current_offset = tuple_id as usize + 2;
            for _ in 0..element_count {
                let element = ValueRef::deserialize_from(&ctx.temp_buffer()[current_offset..])
                    .map_err(|_| VMErrorCode::ProtocolError)?;
                current_offset += element.serialized_size();
                elements.push(element);
            }

            // Push elements onto stack in reverse order (so first element is on top)
            for element in elements.into_iter().rev() {
                ctx.push(element)?;
            }
        }

        // Stack management operations (0xFB-0xFC) - moved from stack range
        STACK_SIZE => {
            debug_log!("MitoVM: STACK_SIZE - get current stack size");
            let size = ctx.size() as u64;
            vm_push_u64!(ctx, size);
        }
        STACK_CLEAR => {
            debug_log!("MitoVM: STACK_CLEAR - clear entire stack");
            while !ctx.is_empty() {
                ctx.pop()?;
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
