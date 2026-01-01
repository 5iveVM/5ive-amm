//! Local variable operations handler for MitoVM
//!
//! This module handles local variable operations including ALLOC_LOCALS,
//! DEALLOC_LOCALS, SET_LOCAL, GET_LOCAL, CLEAR_LOCAL, LOAD_PARAM, and STORE_PARAM.
//! It manages local variable storage and parameter access.

use crate::{
    context::ExecutionManager,
    debug_log,
    error::{CompactResult, VMErrorCode},
    handlers::handle_option_result_ops,
    MAX_LOCALS,
};
use five_protocol::{opcodes::*, ValueRef};

/// Handle nibble immediate operations (0xD0-0xDF)
/// 🎯 BPF OPTIMIZATION: Single-byte encoding for common operations
/// Covers locals (0xD0-0xD7), constants (0xD8-0xDB), and parameters (0xDC-0xDF)
#[inline(never)]
pub fn handle_nibble_locals(opcode: u8, ctx: &mut ExecutionManager) -> CompactResult<()> {
    match opcode {
        // Nibble immediate GET_LOCAL operations (0xD0-0xD3)
        GET_LOCAL_0..=GET_LOCAL_3 => {
            let index = opcode - GET_LOCAL_0;
            let value = ctx.get_local(index)?;
            ctx.push(value)?;
            // debug_log!("MitoVM: GET_LOCAL_{} (nibble immediate)", index);
        }
        // Nibble immediate SET_LOCAL operations (0xD4-0xD7)
        SET_LOCAL_0..=SET_LOCAL_3 => {
            let index = opcode - SET_LOCAL_0;
            let value = ctx.pop()?;
            ctx.set_local(index, value)?;
            // debug_log!("MitoVM: SET_LOCAL_{} (nibble immediate)", index);
        }
        // Nibble immediate PUSH constant operations (0xD8-0xDB)
        PUSH_0..=PUSH_3 => {
            let value = (opcode - PUSH_0) as u64;
            ctx.push(ValueRef::U64(value))?;
            // debug_log!("MitoVM: PUSH_{} (nibble immediate)", value);
        }
        // Nibble immediate LOAD_PARAM operations (0xDC-0xDF)
        LOAD_PARAM_0..=LOAD_PARAM_3 => {
            let index = opcode - LOAD_PARAM_0;
            let value = ctx.parameters()[index as usize];
            if value.is_empty() {
                // debug_log!("MitoVM: LOAD_PARAM_{} - parameter is empty, returning 0", index);
                ctx.push(ValueRef::U64(0))?;
            } else {
                ctx.push(value)?;
                // debug_log!("MitoVM: LOAD_PARAM_{} (nibble immediate)", index);
            }
        }
        _ => {
            debug_log!("MitoVM: Unknown nibble immediate opcode: {}", opcode);
            return Err(VMErrorCode::InvalidInstruction);
        }
    }
    Ok(())
}

/// Handle local variable operations (0xA0-0xAF)
/// 🎯 LOGICAL REORGANIZATION: Locals moved from 0x90 to 0xA0
pub fn handle_locals(opcode: u8, ctx: &mut ExecutionManager) -> CompactResult<()> {
    match opcode {
        ALLOC_LOCALS => {
            let count = ctx.fetch_byte()?;
            debug_log!("MitoVM: ALLOC_LOCALS count: {}", count);
            if count as usize > MAX_LOCALS {
                return Err(VMErrorCode::LocalsOverflow);
            }
            ctx.allocate_locals(count)?;
        }
        DEALLOC_LOCALS => {
            debug_log!("MitoVM: DEALLOC_LOCALS");
            ctx.deallocate_locals();
        }
        SET_LOCAL => {
            let index = ctx.fetch_byte()? as u32;
            debug_log!(
                "MitoVM: SET_LOCAL index: {}, stack size before pop: {}, local_base={}, local_count={}",
                index,
                ctx.size() as u32,
                ctx.local_base() as u32,
                ctx.local_count() as u32
            );

            // Enhanced debugging: Check stack state before operation
            if ctx.is_empty() {
                debug_log!("MitoVM: SET_LOCAL ERROR - attempting to pop from empty stack");
                debug_log!(
                    "MitoVM: SET_LOCAL - this indicates a stack management issue in function calls"
                );
                debug_log!(
                    "MitoVM: SET_LOCAL - SP={}, local_base={}, index={}",
                    ctx.size() as u32,
                    ctx.local_base() as u32,
                    index
                );
                return Err(VMErrorCode::StackError);
            }

            let value = ctx.pop()?;
            ctx.set_local(index as u8, value)?;
            debug_log!(
                "MitoVM: SET_LOCAL index: {}, stack size after pop: {}",
                index,
                ctx.size() as u32
            );
        }
        GET_LOCAL => {
            let index = ctx.fetch_byte()? as u32;
            let value = ctx.get_local(index as u8)?;
            ctx.push(value)?;
            debug_log!("MitoVM: GET_LOCAL index: {}", index);
        }
        CLEAR_LOCAL => {
            let index = ctx.fetch_byte()? as u32;
            ctx.clear_local(index as u8)?;
            debug_log!("MitoVM: CLEAR_LOCAL index: {}", index);
        }
        LOAD_PARAM => {
            let compiler_param_index = ctx.fetch_byte()?;
            // debug_log!(
                // "MitoVM: LOAD_PARAM compiler requested parameter index: {}",
                // compiler_param_index
            // );

            // SIMPLE TRUTH: VLE stores [0]=func_idx, [1]=param1, [2]=param2
            // LOAD_PARAM 1 should get params[1], LOAD_PARAM 2 should get params[2]
            // NO OFFSET needed - direct mapping
            if compiler_param_index == 0 {
                debug_log!("MitoVM: LOAD_PARAM ERROR - invalid parameter index 0 (parameters are 1-based in compiler)");
                return Err(VMErrorCode::InvalidInstruction);
            }

            let actual_param_index = compiler_param_index as usize; 
            // debug_log!(
                // "MitoVM: LOAD_PARAM idx {} -> storage idx {} (param_len: {})",
                // compiler_param_index,
                // actual_param_index as u32,
                // ctx.param_len as u32
            // );

            // Validate translated parameter index bounds against actual parameter count
            if actual_param_index > ctx.param_len as usize {
                debug_log!(
                    "MitoVM: LOAD_PARAM ERROR - translated index {} > actual param_len {}",
                    actual_param_index as u32,
                    ctx.param_len as u32
                );
                return Err(VMErrorCode::InvalidInstruction);
            }

            // Get parameter value using 0-based indexing
            let param_value = ctx.parameters()[actual_param_index];
            
            // Check if parameter is empty (uninitialized)
            if param_value.is_empty() {
                debug_log!(
                    "MitoVM: LOAD_PARAM ERROR - parameter at index {} is empty/uninitialized",
                    actual_param_index as u32
                );
                
                return Err(VMErrorCode::InvalidParameter);
            }

            // Push parameter to stack
            ctx.push(param_value)?;

            // debug_log!("MitoVM: LOAD_PARAM success: p[{}] -> stack (size: {})", 
                      // actual_param_index as u32, ctx.size() as u32);
        }
        STORE_PARAM => {
            let param_index = ctx.fetch_byte()? as u32;
            let value = ctx.pop()?;
            ctx.set_parameter(param_index as usize, value)?;
            debug_log!("MitoVM: STORE_PARAM index: {}", param_index);
        }

        // Result type operations (0xAC-0xAE)
        // These are in the 0xA0-0xAF range but belong in handle_option_result_ops
        RESULT_UNWRAP | RESULT_GET_VALUE | RESULT_GET_ERROR => {
            handle_option_result_ops(opcode, ctx)?;
        }
        
        CAST => {
            let target_type = ctx.fetch_byte()?;
            let value = ctx.pop()?;
            
            // Handle U8 cast
            if target_type == five_protocol::types::U8 {
                let u8_val = value.as_u64().ok_or(VMErrorCode::TypeMismatch)? as u8;
                debug_log!("MitoVM: CAST to U8: {} -> {}", value.as_u64().unwrap(), u8_val);
                ctx.push(ValueRef::U8(u8_val))?;
            } else {
                debug_log!("MitoVM: CAST unsupported type: {}", target_type);
                return Err(VMErrorCode::InvalidInstruction);
            }
        }

        // Legacy u128 opcodes removed - all arithmetic now uses polymorphic generic opcodes
        _ => return Err(VMErrorCode::InvalidInstruction),
    }
    Ok(())
}
