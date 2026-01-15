//! Value Resolution Subsystem for MitoVM
//!
//! This module handles the resolution of zero-copy ValueRefs into concrete Values,
//! and the finalization of execution results. It isolates the complex type
//! conversion logic from the core execution loop.

use crate::{
    context::ExecutionManager,
    debug_log,
    error::{CompactResult, VMErrorCode, VMError},
    utils::ValueRefUtils,
};
use five_protocol::{Value, ValueRef};
use pinocchio::pubkey::Pubkey;

/// Convert ValueRef (zero-copy reference) to concrete Value using current execution state.
/// Handles complex references like TempRef, OptionalRef, and AccountRef.
#[allow(dead_code)] // Function is used recursively
#[inline(never)]
pub fn resolve_value_ref(value_ref: &ValueRef, ctx: &ExecutionManager<'_>) -> CompactResult<Value> {
    resolve_value_ref_with_depth(value_ref, ctx, 0)
}

/// Internal recursive implementation of resolve_value_ref with depth tracking
fn resolve_value_ref_with_depth(
    value_ref: &ValueRef,
    ctx: &ExecutionManager<'_>,
    depth: u8,
) -> CompactResult<Value> {
    // Prevent infinite recursion/stack overflow
    const MAX_VALUE_REF_DEPTH: u8 = 8;
    if depth > MAX_VALUE_REF_DEPTH {
        return Err(VMErrorCode::StackOverflow);
    }

    match value_ref {
        // Immediate values - no context needed
        ValueRef::Empty => Ok(Value::Empty),
        ValueRef::U8(v) => Ok(Value::U8(*v)), // Preserve U8 type
        ValueRef::U64(v) => Ok(Value::U64(*v)),
        ValueRef::U128(v) => Ok(Value::U128(*v)),
        ValueRef::I64(v) => Ok(Value::I64(*v)),
        ValueRef::Bool(v) => Ok(Value::Bool(*v)),

        // Reference types - need context resolution
        ValueRef::TempRef(offset, size) => {
            let start = *offset as usize;
            let end = start + *size as usize;
            if end > ctx.temp_buffer().len() {
                return Err(VMErrorCode::MemoryViolation);
            }

            // Try to deserialize as ValueRef first, then extract the actual value
            match ValueRef::deserialize_from(&ctx.temp_buffer()[start..end]) {
                Ok(inner_ref) => resolve_value_ref_with_depth(&inner_ref, ctx, depth + 1),
                Err(_) => {
                    // If deserialization fails, treat as raw bytes and convert to u64 if possible
                    if *size == 8 {
                        let bytes: [u8; 8] = ctx.temp_buffer()[start..end]
                            .try_into()
                            .map_err(|_| VMErrorCode::ProtocolError)?;
                        Ok(Value::U64(u64::from_le_bytes(bytes)))
                    } else if *size == 1 {
                        Ok(Value::U64(ctx.temp_buffer()[start] as u64))
                    } else {
                        // For other sizes, return Empty
                        Ok(Value::Empty)
                    }
                }
            }
        }

        ValueRef::TupleRef(offset, _size) => {
            // Map TupleRef to Value::Array for compatibility
            Ok(Value::Array(*offset))
        }

        ValueRef::OptionalRef(offset, size) => {
            if *size == 0 {
                return Err(VMErrorCode::ProtocolError);
            }
            let tag = ctx.temp_buffer()[*offset as usize];
            if tag == 0 {
                Ok(Value::Empty)
            } else {
                if (*size as usize) <= 1 {
                    return Err(VMErrorCode::ProtocolError);
                }
                let inner_ref = ValueRef::deserialize_from(
                    &ctx.temp_buffer()[*offset as usize + 1..*offset as usize + *size as usize],
                )
                .map_err(|_| VMErrorCode::ProtocolError)?;
                resolve_value_ref_with_depth(&inner_ref, ctx, depth + 1)
            }
        }

        ValueRef::ResultRef(offset, size) => {
            if *size == 0 {
                return Err(VMErrorCode::ProtocolError);
            }
            let tag = ctx.temp_buffer()[*offset as usize];
            if (*size as usize) > 1 {
                let inner_ref = ValueRef::deserialize_from(
                    &ctx.temp_buffer()[*offset as usize + 1..*offset as usize + *size as usize],
                )
                .map_err(|_| VMErrorCode::ProtocolError)?;
                let inner_val = resolve_value_ref_with_depth(&inner_ref, ctx, depth + 1)?;
                if tag != 0 {
                    Ok(inner_val)
                } else {
                    Ok(Value::Empty)
                }
            } else {
                Ok(Value::Empty)
            }
        }

        ValueRef::AccountRef(idx, _offset) => Ok(Value::Account(*idx)),
        ValueRef::InputRef(offset) => {
            let start = *offset as usize;
            let end = start + 8;
            let data = ctx.instruction_data();
            if end > data.len() {
                return Err(VMErrorCode::InvalidOperation);
            }
            let bytes: [u8; 8] = data[start..end]
                .try_into()
                .map_err(|_| VMErrorCode::InvalidOperation)?;
            Ok(Value::U64(u64::from_le_bytes(bytes)))
        }
        ValueRef::PubkeyRef(offset) => {
            let start = *offset as usize;
            let end = start + 32;
            let data = ctx.instruction_data();
            if end <= data.len() {
                let mut pk_bytes = [0u8; 32];
                pk_bytes.copy_from_slice(&data[start..end]);
                Ok(Value::Pubkey(Pubkey::from(pk_bytes)))
            } else if start < ctx.accounts().len() {
                let pk = *ctx.accounts()[start].key();
                Ok(Value::Pubkey(pk))
            } else {
                Err(VMErrorCode::InvalidOperation)
            }
        }
        ValueRef::ArrayRef(id) => Ok(Value::Array(*id)),
        ValueRef::StringRef(id) => Ok(Value::String(*id as u8)),
        ValueRef::HeapString(id) => Ok(Value::String(*id as u8)),
        ValueRef::HeapArray(id) => Ok(Value::Array(*id as u8)),
    }
}

/// Extract final execution result, prioritizing captured return values over stack contents.
#[inline(never)]
pub fn finalize_execution_result(ctx: &mut ExecutionManager<'_>) -> CompactResult<Option<Value>> {
    // NEW APPROACH: Use captured return value instead of relying on stack
    // This fixes the stack persistence issue after ExecutionManager refactoring
    match ctx.return_value() {
        Some(value) => Ok(Some(resolve_value_ref(&value, ctx)?)),
        None => {
            // No return value captured, check if there's something on the stack as fallback
            if !ctx.is_empty() {
                let value_ref = ctx.pop()?;
                // Use full resolution for stack values too
                Ok(Some(resolve_value_ref(&value_ref, ctx)?))
            } else {
                Ok(None) // No return value
            }
        }
    }
}
